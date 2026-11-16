//! Kommandozeile für den Rechenkern (bis die Desktop-App steht; auch für Tests und Vergleiche).

mod devserver;
mod optimize;

// ISS-22: Speichergrenze je Testprozess (engine::test_alloc).
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
VivaPlan – Rechenkern

Verwendung:
  vivaplan run <planung.vplan> [--csv <datei>] [--json <datei>] [--events]
  vivaplan validate <planung.vplan>
  vivaplan import-legacy <config.json> --start-year <JJJJ> [-o <planung.vplan>] [--base <verzeichnis>]
  vivaplan bench <planung.vplan> [--paths <anzahl>]
  vivaplan optimize <planung.vplan> [--strategy all|standard|three_buckets|guardrails|fixed_rate|vpw|endowment]
        [--goal 1|2] [--min-consumption 90] [--max-ruin 5] [--ruin-age 95] [--weight-median 0.5] [--vol-weight 1] [--max-vol 10]
        [--paths 2000] [--check-paths 10000] [--samples 30] [--seed 1] [--json <datei>]
  vivaplan schema
  vivaplan --version
  vivaplan library [--library <ordner>]          (Zeitreihen der Datenbibliothek)
  vivaplan import-shiller <ie_data.xls|.xlsx|.ods> [--library <ordner>]
  vivaplan import-snb <würfel.csv>… [--library <ordner>]   (devkum, plkopr, zimoma, rendoblim von data.snb.ch)
  vivaplan ui-dev [--port 8787] [--ui <ordner>] [--library <ordner>]   (Oberfläche im Browser, nur lokal)

Die Datenbibliothek ist ohne --library der Ordner crates/engine/data/ des Projekts; Planungen verweisen
mit «library:<datei>» darauf.

Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("validate") => cmd_validate(&args[1..]),
        Some("import-legacy") => cmd_import(&args[1..]),
        Some("bench") => cmd_bench(&args[1..]),
        Some("optimize") => optimize::cmd_optimize(&args[1..]),
        Some("library") => cmd_library(&args[1..]),
        Some("import-shiller") => cmd_import_shiller(&args[1..]),
        Some("import-snb") => cmd_import_snb(&args[1..]),
        Some("ui-dev") => {
            let port = opt(&args, &["--port"]).and_then(|p| p.parse().ok()).unwrap_or(8787);
            let ui = opt(&args, &["--ui"])
                .map(PathBuf::from)
                .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/ui"));
            devserver::serve(port, ui, library_dir(&args)).map_err(Into::into)
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
            eprintln!("Fehler: {e}");
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

pub(crate) fn library_dir(args: &[String]) -> PathBuf {
    opt(args, &["--library"]).map(PathBuf::from).unwrap_or_else(|| {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/data");
        std::fs::canonicalize(&p).unwrap_or(p)
    })
}

fn positionals(args: &[String]) -> Vec<&String> {
    let mut out = vec![];
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
        } else if a.starts_with('-') {
            skip = !matches!(a.as_str(), "--events");
        } else {
            out.push(a);
        }
    }
    out
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
            "Geschrieben: {file}  ({} – {}, {} Monate{}; Aktien {}, Obligationen {}, Inflation {} pro Jahr)",
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
        println!("Hinweis: {n}");
    }
}

fn cmd_library(args: &[String]) -> R {
    let dir = library_dir(args);
    println!("Datenbibliothek: {}", dir.display());
    for e in engine::api::library_list(&dir)?.entries {
        match (&e.info, &e.error) {
            (Some(i), _) => println!(
                "  {:<28} {:<4} {} {} – {}{}  Aktien {}, Obligationen {}, Inflation {}",
                e.file,
                i.currency.as_deref().unwrap_or("?"),
                if i.monthly { "monatlich" } else { "jährlich " },
                i.first,
                i.last,
                if i.has_saron { " · SARON" } else { "" },
                pct(i.avg_stocks),
                pct(i.avg_bonds),
                pct(i.avg_inflation)
            ),
            (None, Some(err)) => println!("  {:<28} FEHLER: {err}", e.file),
            _ => {}
        }
    }
    Ok(())
}

fn cmd_import_shiller(args: &[String]) -> R {
    let file = positional(args).ok_or("Shiller-Datei fehlt")?;
    let r = engine::api::import_shiller(&library_dir(args), Path::new(file))?;
    print_report(&r);
    Ok(())
}

fn cmd_import_snb(args: &[String]) -> R {
    let files: Vec<PathBuf> = positionals(args).into_iter().map(PathBuf::from).collect();
    if files.is_empty() {
        return Err("SNB-Datei fehlt".into());
    }
    let r = engine::api::import_snb(&library_dir(args), &files)?;
    print_report(&r);
    Ok(())
}

pub(crate) fn positional(args: &[String]) -> Option<&String> {
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if a.starts_with('-') {
            skip = !matches!(a.as_str(), "--events");
            continue;
        }
        return Some(a);
    }
    None
}

fn load(path: &str) -> Result<(Plan, PathBuf), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let plan = Plan::from_json(&text)?;
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
            "FEHLER "
        } else {
            "Hinweis"
        };
        println!("  {tag} [{}] {}", i.section, i.message);
    }
    issues.iter().any(|i| i.severity == Severity::Error)
}

fn cmd_validate(args: &[String]) -> R {
    let path = positional(args).ok_or("Projektdatei fehlt")?;
    let (plan, _) = load(path)?;
    if print_issues(&plan) {
        return Err("Die Planung enthält Fehler.".into());
    }
    println!("Planung ist gültig.");
    Ok(())
}

fn cmd_run(args: &[String]) -> R {
    let path = positional(args).ok_or("Projektdatei fehlt")?;
    let (plan, base) = load(path)?;
    print_issues(&plan);
    let t0 = Instant::now();
    let sim = Simulator::new(&plan)?;
    let data = batch::load_series(&plan, &base, Some(&library_dir(args)))?;
    let paths = batch::path_source(&plan, &sim, data.as_ref())?;
    let prep = t0.elapsed();

    if paths.len() == 1 {
        let t1 = Instant::now();
        let res = sim.run(&paths.get(0), true);
        let dt = t1.elapsed();
        print_single(&sim, &res, args.iter().any(|a| a == "--events"));
        println!(
            "\nRechenzeit: {:.3} ms (Vorbereitung {:.3} ms)",
            dt.as_secs_f64() * 1e3,
            prep.as_secs_f64() * 1e3
        );
        if let Some(csv) = opt(args, &["--csv"]) {
            write_csv(&csv, &res)?;
            println!("Monatswerte: {csv}");
        }
        if let Some(json) = opt(args, &["--json"]) {
            std::fs::write(&json, serde_json::to_string_pretty(&res)?)?;
            println!("Ergebnis: {json}");
        }
    } else {
        let t1 = Instant::now();
        let results = batch::run_paths(&sim, &paths);
        let dt = t1.elapsed();
        let s = batch::summarize(&results);
        println!("{} Läufe ({})", s.runs, plan.returns.source_label());
        println!(
            "Vermögen reicht bis zum Ende in {:.1} % der Läufe",
            s.success_rate * 100.0
        );
        if let Some((m, label)) = &s.earliest_depletion {
            println!("Früheste Erschöpfung: {m} (Alter {:.1}) – {label}", sim.age_at(*m));
        }
        println!("\nGesamtvermögen am Ende (nominal):");
        for p in &s.percentiles {
            let name = if p.percent == 0 {
                "Schlechtester Lauf".to_string()
            } else {
                format!("{:>3}. Perzentil", p.percent)
            };
            println!("  {name:<18} {:>15}  {}", chf(p.end_total), p.label);
        }
        println!(
            "\nRechenzeit: {:.1} ms für {} Läufe ({:.1} µs pro Lauf, {} Kerne)",
            dt.as_secs_f64() * 1e3,
            s.runs,
            dt.as_secs_f64() * 1e6 / s.runs as f64,
            rayon_threads()
        );
        if let Some(json) = opt(args, &["--json"]) {
            std::fs::write(&json, serde_json::to_string_pretty(&s)?)?;
        }
    }
    Ok(())
}

/// Rechnet den ersten Renditepfad N-mal parallel (Last wie bei einer Monte-Carlo-Simulation).
fn cmd_bench(args: &[String]) -> R {
    let path = positional(args).ok_or("Projektdatei fehlt")?;
    let n: usize = opt(args, &["--paths"]).map_or(Ok(50_000), |s| s.parse())?;
    let (plan, base) = load(path)?;
    let sim = Simulator::new(&plan)?;
    let data = batch::load_series(&plan, &base, Some(&library_dir(args)))?;
    let first = batch::path_source(&plan, &sim, data.as_ref())?.get(0).into_owned();
    let paths = vec![first; n];
    let t0 = Instant::now();
    let results = batch::run_all(&sim, &paths);
    let dt = t0.elapsed();
    println!(
        "{n} Läufe à {} Monate in {:.0} ms ({:.1} µs pro Lauf, {} Kerne); Kontrolle: {}",
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
            Fixed => "fixe Renditen",
            Historical => "historisch",
            Rolling => "alle historischen Startzeitpunkte",
            Bootstrap => "Monte Carlo historisch",
            Parametric => "Monte Carlo parametrisch",
        }
    }
}

fn print_single(sim: &Simulator, res: &SimResult, events: bool) {
    let s = &res.summary;
    println!("{}  ·  {} – {}", res.label, sim.start, sim.end);
    println!(
        "\n{:>7} {:>6} {:>14} {:>14} {:>14} {:>12} {:>14}",
        "Jahr", "Alter", "Liquid", "Immobilien", "Pensionskasse", "Säule 3a", "Total"
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
        println!("Vermögen bei Pensionierung:   {}", chf(t));
    }
    println!(
        "Kapitalbezug PK netto:        {} (Steuer {})",
        chf(s.lumpsum_net),
        chf(s.lumpsum_tax)
    );
    println!("PK-Rente pro Monat:           {}", chf(s.pension_fund_monthly));
    println!("Vermögen am Ende:             {}", chf(s.end_total));
    println!("Steuern total:                {}", chf(s.taxes_total));
    match s.depleted {
        Some(m) => println!("Vermögen aufgebraucht:        {m} (Alter {:.1})", sim.age_at(m)),
        None => println!("Vermögen reicht bis zum Ende."),
    }
    if let Some(m) = s.min_liquid_month {
        println!("Tiefstes liquides Vermögen:   {} ({m})", chf(s.min_liquid));
    }
    if events {
        println!("\nEreignisse:");
        for e in &res.events {
            println!("  {} (Alter {:.1})  {}", e.month, sim.age_at(e.month), e.text);
        }
    }
}

fn write_csv(path: &str, res: &SimResult) -> R {
    let mut out = String::from(
        "month,age,cash,stocks,bonds,liquid,property_equity,pension_fund,pillar3a,total,income,spending,taxes\n",
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
    let path = positional(args).ok_or("config.json fehlt")?;
    let year: i32 = opt(args, &["--start-year"]).ok_or("--start-year fehlt")?.parse()?;
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
        eprintln!("Hinweis: {n}");
    }
    let json = imp.plan.to_json();
    match opt(args, &["-o", "--out"]) {
        Some(out) => {
            std::fs::write(&out, json)?;
            eprintln!("Gespeichert: {out}");
        }
        None => println!("{json}"),
    }
    Ok(())
}

fn schemars_schema() -> String {
    engine::schema_json()
}
