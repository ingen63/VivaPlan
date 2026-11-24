//! Command line for the calculation engine (until the desktop app is ready; also for tests and comparisons).

mod describe;
mod devserver;
mod evaluate;
mod mcp;
mod optimize;

// #116: memory limit per test process (engine::test_alloc).
#[cfg(test)]
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use engine::batch;
use engine::model::Plan;
use engine::sim::{SimResult, Simulator};
use engine::validate::{self, Severity};

const USAGE: &str = "\
VivaPlan – calculation engine

Usage:
  vivaplan run <plan.vplan> [--csv <file>] [--json [<file.json>]] [--events] [--metric <json>]…
        (--metric: a metric as JSON, or a list of them; several times possible; see the help, reference)
  vivaplan validate <plan.vplan> [--json]
  vivaplan set <plan.vplan> <path>[@<time>]=<value>… [-o <plan.vplan>] [--overwrite]
        (<time>: milestone id, age:<years> or YYYY-MM; without -o a new file plan-1.vplan, plan-2.vplan, …)
  vivaplan diff <a.vplan> <b.vplan> [--json]
  vivaplan import-legacy <config.json> --start-year <YYYY> [-o <plan.vplan>] [--base <directory>]
  vivaplan bench <plan.vplan> [--paths <count>] [--json]
  vivaplan optimize <plan.vplan> [--strategy all|standard|three_buckets|guardrails|fixed_rate|vpw|endowment]
        [--goal 1|2] [--min-consumption 90] [--max-ruin 5] [--ruin-age 95] [--weight-median 0.5] [--vol-weight 1] [--max-vol 10]
        [--paths 2000] [--check-paths 10000] [--samples 30] [--seed 1] [--json [<file.json>]]
        [--compare] [--property planned|never-sell|both]
        (--compare: strategies with the plan's values, without search, on the plan's paths as in run)
  vivaplan evaluate <plan.vplan> <experiment.json> [--json] [--yes]
        (experiment: changes, variations, metrics, constraints, objective; see the help, reference;
         from an estimated 60 s only with --yes)
  vivaplan describe [<section>] [--plan <plan.vplan>] [--json]
        (catalog for AIs: fields, series, state, metrics, experiment, examples, performance, limits;
         performance measured in about 0.5 s on the plan, otherwise on the example plan)
  vivaplan mcp [--data-dir <folder>] [--library <folder>] [--names] [--read-only] [--allow-dir <folder>]…
        (MCP server over stdio for AI programs: plans, checks, simulation, evaluate, change and undo, save;
         names replaced unless --names; saves only into the folder of the plan or an --allow-dir folder,
         --read-only offers no saving)
  vivaplan schema
  vivaplan --version
  vivaplan --help   (this help)
  vivaplan library [--library <folder>] [--data-dir <folder>]   (time series of the data library)
  vivaplan import-shiller <ie_data.xls|.xlsx|.ods> [--library <folder>]
  vivaplan import-snb <cube.csv>… [--library <folder>]   (devkum, plkopr, zimoma, rendoblim from data.snb.ch)
  vivaplan ui-dev [--port 8787] [--ui <folder>] [--library <folder>]   (user interface in the browser, local only)

--json alone prints the result in a fixed format to the standard output (see the help, reference);
--json <file.json> saves the detailed result as before.
Data folder: like the app, the command line calculates with the data folder of the app (time series in
“Zeitreihen”, limits from “einstellungen.json”); --data-dir <folder> uses another one, --library <folder> another
data library. Without the data folder: built-in series and default limits. Plans refer to the data library with
“library:<file>”; ui-dev uses the folder crates/engine/data/ of the project. Messages and events are in English.

Model calculation without guarantee. No tax or investment advice. Taxes are estimates.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("validate") => cmd_validate(&args[1..]),
        Some("set") => cmd_set(&args[1..]),
        Some("diff") => cmd_diff(&args[1..]),
        Some("import-legacy") => cmd_import(&args[1..]),
        Some("bench") => cmd_bench(&args[1..]),
        Some("optimize") => optimize::cmd_optimize(&args[1..]),
        Some("evaluate") => evaluate::cmd_evaluate(&args[1..]),
        Some("describe") => describe::cmd_describe(&args[1..]),
        Some("mcp") => mcp::cmd_mcp(&args[1..]),
        Some("library") => cmd_library(&args[1..]),
        Some("import-shiller") => cmd_import_shiller(&args[1..]),
        Some("import-snb") => cmd_import_snb(&args[1..]),
        Some("ui-dev") => {
            let port = opt(&args, &["--port"]).and_then(|p| p.parse().ok()).unwrap_or(8787);
            let ui = opt(&args, &["--ui"])
                .map(PathBuf::from)
                .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/ui"));
            devserver::serve(port, ui, dev_library_dir(&args)).map_err(Into::into)
        }
        // #208: the help on request goes to the standard output and ends without an error code.
        Some("--help" | "-h" | "help") => {
            print!("{USAGE}");
            Ok(())
        }
        Some("--version" | "-V") => {
            println!("vivaplan {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("schema") => {
            let schema = schemars_schema();
            println!("{schema}");
            Ok(())
        }
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

pub(crate) type R = Result<(), Box<dyn std::error::Error>>;

pub(crate) fn opt(args: &[String], names: &[&str]) -> Option<String> {
    args.windows(2)
        .find(|w| names.contains(&w[0].as_str()))
        .map(|w| w[1].clone())
}

/// Data library of the development server `ui-dev`: `--library`, otherwise the folder crates/engine/data/ of the project.
fn dev_library_dir(args: &[String]) -> PathBuf {
    opt(args, &["--library"]).map(PathBuf::from).unwrap_or_else(|| {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/data");
        std::fs::canonicalize(&p).unwrap_or(p)
    })
}

/// #206: What the command line calculates with, as the app: data library and settings of the app's data folder.
pub(crate) struct AppData {
    /// Data library; `None` = only the built-in series.
    pub(crate) library: Option<PathBuf>,
    /// Settings (`einstellungen.json`); empty = defaults.
    pub(crate) settings: serde_json::Map<String, serde_json::Value>,
}

impl AppData {
    pub(crate) fn library(&self) -> Option<&Path> {
        self.library.as_deref()
    }

    /// Reads a plan and adds the settings as the app does with every calculation (limits, affordability).
    pub(crate) fn read_plan(&self, path: &Path) -> Result<serde_json::Value, engine::Error> {
        let mut plan = engine::api::read_plan(path)?;
        engine::appdata::apply(&mut plan, &self.settings);
        Ok(plan)
    }
}

/// #206/AK5: The note on the missing data folder appears once per call.
static MISSING_NOTE: std::sync::Once = std::sync::Once::new();

/// #206/AK2–AK5: Data folder of the app (`--data-dir`, otherwise as in the app per operating system) with its library
/// («Zeitreihen») and settings; `--library` replaces the library. Without the app's data folder: built-in series and
/// default limits, with a note on the standard error output.
pub(crate) fn app_data(args: &[String]) -> Result<AppData, String> {
    let given = opt(args, &["--data-dir"]).map(PathBuf::from);
    if let Some(d) = given.as_ref().filter(|d| !d.is_dir()) {
        return Err(format!("Data folder not found: {}", d.display()));
    }
    let candidate = given.or_else(engine::appdata::default_dir);
    let dir = candidate.clone().filter(|d| d.is_dir());
    if dir.is_none() {
        MISSING_NOTE.call_once(|| {
            let place = candidate.map(|d| format!(" ({})", d.display())).unwrap_or_default();
            eprintln!(
                "Note: data folder of the app not found{place}; calculating with the built-in series and the default limits."
            );
        });
    }
    let library = opt(args, &["--library"])
        .map(PathBuf::from)
        .or_else(|| dir.as_deref().map(engine::appdata::library_dir).filter(|l| l.is_dir()));
    let settings = dir.as_deref().map(engine::appdata::read_settings).unwrap_or_default();
    Ok(AppData { library, settings })
}

/// #206: Folder of the data library for `library` and the imports: `--library`, otherwise «Zeitreihen» in the data folder
/// of the app (`--data-dir` or as in the app); created if needed, as in the app.
fn library_folder(args: &[String]) -> Result<PathBuf, String> {
    if let Some(l) = opt(args, &["--library"]) {
        return Ok(PathBuf::from(l));
    }
    opt(args, &["--data-dir"])
        .map(PathBuf::from)
        .or_else(engine::appdata::default_dir)
        .map(|d| engine::appdata::library_dir(&d))
        .ok_or_else(|| "Data folder of the app unknown; give the data library with --library".into())
}

/// Whether the option `flag` takes the next argument as its value. #207: `--json` only takes a file ending in `.json`.
fn takes_value(flag: &str, next: Option<&String>) -> bool {
    match flag {
        "--events" | "--compare" | "--overwrite" | "--yes" => false,
        "--json" => next.is_some_and(|n| n.ends_with(".json")),
        _ => true,
    }
}

pub(crate) fn positionals(args: &[String]) -> Vec<&String> {
    let mut out = vec![];
    let mut skip = false;
    for (i, a) in args.iter().enumerate() {
        if skip {
            skip = false;
        } else if a.starts_with('-') {
            skip = takes_value(a, args.get(i + 1));
        } else {
            out.push(a);
        }
    }
    out
}

/// #207: Where the result goes as JSON.
#[derive(Debug, PartialEq)]
pub(crate) enum JsonOut {
    None,
    /// `--json` alone: fixed format on the standard output.
    Stdout,
    /// `--json <file.json>`: detailed result in a file.
    File(String),
}

pub(crate) fn json_out(args: &[String]) -> JsonOut {
    match args.iter().position(|a| a == "--json") {
        None => JsonOut::None,
        Some(i) => match args.get(i + 1) {
            Some(f) if takes_value("--json", Some(f)) => JsonOut::File(f.clone()),
            _ => JsonOut::Stdout,
        },
    }
}

/// #207: Prints a fixed format on the standard output.
pub(crate) fn print_json(v: &impl serde::Serialize) -> R {
    println!("{}", serde_json::to_string(v)?);
    Ok(())
}

fn pct(v: f64) -> String {
    if v.is_nan() {
        "–".into()
    } else {
        format!("{:.2} %", v * 100.0)
    }
}

fn print_report(r: &engine::library::ImportReport) {
    for (file, i) in &r.written {
        println!(
            "Written: {file}  ({} – {}, {} months{}; stocks {}, bonds {}, inflation {} per year)",
            i.first,
            i.last,
            i.entries,
            i.saron_range
                .map(|(a, b)| format!(", SARON {a} – {b}"))
                .unwrap_or_default(),
            pct(i.avg_stocks),
            pct(i.avg_bonds),
            pct(i.avg_inflation)
        );
    }
    for n in &r.notes {
        println!("Note: {n}");
    }
}

fn cmd_library(args: &[String]) -> R {
    let dir = library_folder(args)?;
    println!("Data library: {}", dir.display());
    for e in engine::api::library_list(&dir)?.entries {
        match (&e.info, &e.error) {
            (Some(i), _) => println!(
                "  {:<28} {:<4} {} {} – {}{}  stocks {}, bonds {}, inflation {}",
                e.file,
                i.currency.as_deref().unwrap_or("?"),
                if i.monthly { "monthly" } else { "yearly " },
                i.first,
                i.last,
                if i.has_saron { " · SARON" } else { "" },
                pct(i.avg_stocks),
                pct(i.avg_bonds),
                pct(i.avg_inflation)
            ),
            (None, Some(err)) => println!("  {:<28} ERROR: {err}", e.file),
            _ => {}
        }
    }
    Ok(())
}

fn cmd_import_shiller(args: &[String]) -> R {
    let file = positional(args).ok_or("Shiller file missing")?;
    let r = engine::api::import_shiller(&library_folder(args)?, Path::new(file))?;
    print_report(&r);
    Ok(())
}

fn cmd_import_snb(args: &[String]) -> R {
    let files: Vec<PathBuf> = positionals(args).into_iter().map(PathBuf::from).collect();
    if files.is_empty() {
        return Err("SNB file missing".into());
    }
    let r = engine::api::import_snb(&library_folder(args)?, &files)?;
    print_report(&r);
    Ok(())
}

pub(crate) fn positional(args: &[String]) -> Option<&String> {
    positionals(args).into_iter().next()
}

/// Reads the plan with the settings of the app (#206).
fn load(path: &str, data: &AppData) -> Result<(Plan, PathBuf), Box<dyn std::error::Error>> {
    let value = data.read_plan(Path::new(path))?;
    let plan = Plan::from_json(&value.to_string())?;
    let base = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
    Ok((plan, base))
}

pub(crate) fn chf(v: f64) -> String {
    let neg = v < 0.0;
    let digits = format!("{:.0}", v.abs());
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\'');
        }
        out.push(ch);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

fn print_issues(plan: &Plan) -> bool {
    let issues = validate::validate(plan);
    for i in &issues {
        let tag = if i.severity == Severity::Error {
            "ERROR  "
        } else {
            "Note   "
        };
        println!("  {tag} [{}] {}", i.section, i.message);
    }
    issues.iter().any(|i| i.severity == Severity::Error)
}

fn cmd_validate(args: &[String]) -> R {
    let path = positional(args).ok_or("Project file missing")?;
    let data = app_data(args)?;
    if json_out(args) == JsonOut::Stdout {
        let report = engine::api::validate_report(&data.read_plan(Path::new(path))?);
        print_json(&report)?;
        return if report.valid {
            Ok(())
        } else {
            Err("The plan contains errors.".into())
        };
    }
    let (plan, _) = load(path, &data)?;
    if print_issues(&plan) {
        return Err("The plan contains errors.".into());
    }
    println!("The plan is valid.");
    Ok(())
}

/// #214: The metrics given with `--metric` (JSON; each a metric or a list of metrics).
fn metric_args(args: &[String]) -> Result<Vec<serde_json::Value>, String> {
    if args.last().is_some_and(|a| a == "--metric") {
        return Err("--metric needs a metric as JSON".into());
    }
    args.windows(2)
        .filter(|w| w[0] == "--metric")
        .map(|w| serde_json::from_str(&w[1]).map_err(|e| format!("--metric {}: {e}", w[1])))
        .collect()
}

/// #214: Prints the metrics after the result of `run`.
fn print_metrics(path: &str, metrics: &[serde_json::Value], data: &AppData) -> R {
    let base = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
    let plan = data.read_plan(Path::new(path))?;
    let values = engine::api::measure(&plan, metrics, &base, data.library()).map_err(|e| e.to_string())?;
    println!("\nMetrics:");
    let number = |v: f64| {
        if v.abs() >= 100.0 {
            chf(v)
        } else {
            format!("{v:.4}")
        }
    };
    for m in values {
        let value = m.value.map_or_else(|| "–".to_string(), number);
        let error = m.std_error.map(|e| format!(" ± {}", number(e))).unwrap_or_default();
        println!("  {:<36} {value:>15}{error}  (n = {})", m.name, m.values);
    }
    Ok(())
}

fn cmd_run(args: &[String]) -> R {
    let path = positional(args).ok_or("Project file missing")?;
    let json = json_out(args);
    let metrics = metric_args(args)?;
    let data = app_data(args)?;
    if json == JsonOut::Stdout {
        let base = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
        let plan = data.read_plan(Path::new(path))?;
        let mut report = engine::api::run_report(&plan, &base, data.library());
        if !metrics.is_empty() && report.error.is_none() {
            match engine::api::measure(&plan, &metrics, &base, data.library()) {
                Ok(v) => report.metrics = Some(v),
                Err(e) => report.error = Some(e),
            }
        }
        print_json(&report)?;
        return match report.error {
            Some(e) => Err(e.to_string().into()),
            None => Ok(()),
        };
    }
    let (plan, base) = load(path, &data)?;
    print_issues(&plan);
    let t0 = Instant::now();
    let sim = Simulator::new(&plan)?;
    let series = batch::load_series(&plan, &base, data.library())?;
    let paths = batch::path_source(&plan, &sim, series.as_ref())?;
    let prep = t0.elapsed();

    if paths.len() == 1 {
        let t1 = Instant::now();
        let res = sim.run(&paths.get(0), true);
        let dt = t1.elapsed();
        print_single(&sim, &res, args.iter().any(|a| a == "--events"));
        println!(
            "\nComputing time: {:.3} ms (preparation {:.3} ms)",
            dt.as_secs_f64() * 1e3,
            prep.as_secs_f64() * 1e3
        );
        if let Some(csv) = opt(args, &["--csv"]) {
            write_csv(&csv, &res)?;
            println!("Monthly values: {csv}");
        }
        if let JsonOut::File(json) = &json {
            std::fs::write(json, serde_json::to_string_pretty(&res)?)?;
            println!("Result: {json}");
        }
    } else {
        let t1 = Instant::now();
        let results = batch::run_paths(&sim, &paths);
        let dt = t1.elapsed();
        let s = batch::summarize(&results);
        println!("{} runs ({})", s.runs, plan.returns.source_label());
        println!(
            "Wealth lasts until the end in {:.1} % of the runs",
            s.success_rate * 100.0
        );
        if let Some((m, label)) = &s.earliest_depletion {
            println!("Earliest depletion: {m} (age {:.1}) – {label}", sim.age_at(*m));
        }
        if s.unaffordable_renewal_runs > 0 {
            println!(
                "Unaffordable mortgage renewed in full (never sell) in {} runs ({:.1} %)",
                s.unaffordable_renewal_runs,
                s.unaffordable_renewal_runs as f64 / s.runs as f64 * 100.0
            );
        }
        println!("\nTotal wealth at the end (nominal):");
        for p in &s.percentiles {
            let name = if p.percent == 0 {
                "Worst run".to_string()
            } else {
                format!("{:>3}th percentile", p.percent)
            };
            println!("  {name:<18} {:>15}  {}", chf(p.end_total), p.label);
        }
        println!(
            "\nComputing time: {:.1} ms for {} runs ({:.1} µs per run, {} cores)",
            dt.as_secs_f64() * 1e3,
            s.runs,
            dt.as_secs_f64() * 1e6 / s.runs as f64,
            rayon_threads()
        );
        if let JsonOut::File(json) = &json {
            std::fs::write(json, serde_json::to_string_pretty(&s)?)?;
        }
    }
    if !metrics.is_empty() {
        print_metrics(path, &metrics, &data)?;
    }
    Ok(())
}

/// Runs the first return path N times in parallel (load as in a Monte Carlo simulation).
fn cmd_bench(args: &[String]) -> R {
    let path = positional(args).ok_or("Project file missing")?;
    let n: usize = opt(args, &["--paths"]).map_or(Ok(50_000), |s| s.parse())?;
    let data = app_data(args)?;
    let (plan, base) = load(path, &data)?;
    let sim = Simulator::new(&plan)?;
    let series = batch::load_series(&plan, &base, data.library())?;
    let first = batch::path_source(&plan, &sim, series.as_ref())?.get(0).into_owned();
    let paths = vec![first; n];
    let t0 = Instant::now();
    let results = batch::run_all(&sim, &paths);
    let dt = t0.elapsed();
    if json_out(args) == JsonOut::Stdout {
        let check = results[n - 1].summary.end_total;
        return print_json(&engine::report::BenchReport::new(
            n,
            sim.months(),
            dt.as_secs_f64(),
            rayon_threads(),
            check,
        ));
    }
    println!(
        "{n} runs of {} months in {:.0} ms ({:.1} µs per run, {} cores); check: {}",
        sim.months(),
        dt.as_secs_f64() * 1e3,
        dt.as_secs_f64() * 1e6 / n as f64,
        rayon_threads(),
        chf(results[n - 1].summary.end_total)
    );
    Ok(())
}

fn rayon_threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

trait SourceLabel {
    fn source_label(&self) -> &'static str;
}
impl SourceLabel for engine::model::Returns {
    fn source_label(&self) -> &'static str {
        use engine::model::ReturnSource::*;
        match self.source {
            Fixed => "fixed returns",
            Historical => "historical",
            Rolling => "all historical start dates",
            Bootstrap => "Monte Carlo from history",
            Parametric => "parametric Monte Carlo",
        }
    }
}

fn print_single(sim: &Simulator, res: &SimResult, events: bool) {
    let s = &res.summary;
    println!("{}  ·  {} – {}", res.label, sim.start, sim.end);
    println!(
        "\n{:>7} {:>6} {:>14} {:>14} {:>14} {:>12} {:>14}",
        "Year", "Age", "Liquid", "Real estate", "Pension fund", "Pillar 3a", "Total"
    );
    for (i, r) in res.records.iter().enumerate() {
        if r.month.is_december() || i == 0 || i + 1 == res.records.len() {
            println!(
                "{:>7} {:>6.1} {:>14} {:>14} {:>14} {:>12} {:>14}",
                r.month.to_string(),
                r.age,
                chf(r.liquid),
                chf(r.property_equity),
                chf(r.pension_fund),
                chf(r.pillar3a),
                chf(r.total)
            );
        }
    }
    println!();
    if let Some(t) = s.total_at_retirement {
        println!("Wealth at retirement:         {}", chf(t));
    }
    println!(
        "Pension fund lump sum net:    {} (estimated tax {})",
        chf(s.lumpsum_net),
        chf(s.lumpsum_tax)
    );
    println!("Pension fund annuity / month: {}", chf(s.pension_fund_monthly));
    println!("Wealth at the end:            {}", chf(s.end_total));
    println!("Estimated taxes total:        {}", chf(s.taxes_total));
    match s.depleted {
        Some(m) => println!("Wealth depleted:              {m} (age {:.1})", sim.age_at(m)),
        None => println!("Wealth lasts until the end."),
    }
    if let Some(m) = s.min_liquid_month {
        println!("Lowest liquid wealth:         {} ({m})", chf(s.min_liquid));
    }
    if s.unaffordable_renewals > 0 {
        println!("Unaffordable renewals kept:   {} (never sell)", s.unaffordable_renewals);
    }
    if events {
        println!("\nEvents:");
        for e in &res.events {
            println!("  {} (age {:.1})  {}", e.month, sim.age_at(e.month), e.text);
        }
    }
}

fn write_csv(path: &str, res: &SimResult) -> R {
    let mut out = String::from(
        "month,age,cash,stocks,bonds,liquid,property_equity,pension_fund,pillar3a,total,income,spending,taxes_estimated\n",
    );
    for r in &res.records {
        out.push_str(&format!(
            "{},{:.4},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}\n",
            r.month.iso(),
            r.age,
            r.cash,
            r.stocks,
            r.bonds,
            r.liquid,
            r.property_equity,
            r.pension_fund,
            r.pillar3a,
            r.total,
            r.income,
            r.spending,
            r.taxes
        ));
    }
    std::fs::write(path, out)?;
    Ok(())
}

fn cmd_import(args: &[String]) -> R {
    let path = positional(args).ok_or("config.json missing")?;
    let year: i32 = opt(args, &["--start-year"]).ok_or("--start-year missing")?.parse()?;
    let base = opt(args, &["--base"])
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let text = std::fs::read_to_string(path)?;
    let name = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let imp = engine::legacy::import(&text, year, &base, &name)?;
    for n in &imp.notes {
        eprintln!("Note: {n}");
    }
    let json = imp.plan.to_json();
    match opt(args, &["-o", "--out"]) {
        Some(out) => {
            // #136: atomic; an existing file stays unchanged on error
            engine::api::write_atomic(Path::new(&out), json.as_bytes()).map_err(|e| format!("{out}: {e}"))?;
            eprintln!("Saved: {out}");
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// #207/AK2, AK3: Sets values by field path. The engine checks path and type; on an error nothing is written. Without
/// `-o` the result goes to the next free file `<name>-1.vplan`, `<name>-2.vplan`, … next to the plan; an existing file
/// is only replaced with `--overwrite`, after a backup `<file>.bak`.
fn cmd_set(args: &[String]) -> R {
    let list = positionals(args);
    let (path, assignments) = list.split_first().ok_or("Project file missing")?;
    if assignments.is_empty() {
        return Err("Changes missing: <path>=<value>".into());
    }
    let plan = engine::api::read_plan(Path::new(path.as_str()))?;
    let assignments: Vec<String> = assignments.iter().map(|s| s.to_string()).collect();
    let changed = match engine::api::set_fields(&plan, &assignments) {
        Ok(v) => v,
        Err(errors) => {
            for e in &errors {
                eprintln!("Error: {e}");
            }
            return Err("No change made.".into());
        }
    };
    let overwrite = args.iter().any(|a| a == "--overwrite");
    let target = match opt(args, &["-o", "--out"]) {
        Some(out) => PathBuf::from(out),
        None if overwrite => PathBuf::from(path.as_str()),
        None => next_free(Path::new(path.as_str())),
    };
    if target.exists() {
        if !overwrite {
            return Err(format!("{} exists; use --overwrite to replace it", target.display()).into());
        }
        let backup = backup_path(&target);
        std::fs::copy(&target, &backup).map_err(|e| format!("{}: {e}", backup.display()))?;
        eprintln!("Backup: {}", backup.display());
    }
    engine::api::write_plan(&target, &changed)?;
    for i in engine::api::validate(&changed) {
        let tag = if i.severity == Severity::Error { "ERROR" } else { "Note" };
        eprintln!("{tag} [{}] {}", i.section, i.message);
    }
    println!("{}", target.display());
    Ok(())
}

/// `<name>-1.<ext>`, `<name>-2.<ext>`, … : the first that does not exist, next to `path`.
fn next_free(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1..)
        .map(|n| path.with_file_name(format!("{stem}-{n}{ext}")))
        .find(|p| !p.exists())
        .expect("a free name exists")
}

/// `<file>.bak` next to the file.
fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".bak");
    path.with_file_name(name)
}

/// #207/AK4: Lists the differences between two plans: field path, old and new value.
fn cmd_diff(args: &[String]) -> R {
    let list = positionals(args);
    let [a, b] = list.as_slice() else {
        return Err("Two project files expected".into());
    };
    let a = engine::api::read_plan(Path::new(a.as_str()))?;
    let b = engine::api::read_plan(Path::new(b.as_str()))?;
    let changes = engine::api::diff_plans(&a, &b);
    if json_out(args) == JsonOut::Stdout {
        return print_json(&engine::report::diff(changes));
    }
    for c in &changes {
        println!("{}: {} → {}", c.path, c.old, c.new);
    }
    if changes.is_empty() {
        println!("No differences.");
    }
    Ok(())
}

fn schemars_schema() -> String {
    engine::schema_json()
}
