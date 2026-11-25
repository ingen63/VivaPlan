//! JSON interface for the user interface (desktop app and local development server).
//!
//! All functions take the plan as a JSON value, as the editor holds it, and return
//! serializable results. Errors in the plan are returned as validation messages.

use std::borrow::Cow;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;

use crate::batch::{self, Bands, BatchSummary};
use crate::library::{self, ImportReport, SeriesInfo};
use crate::market::{self, DataSet};
use crate::message::Message;
use crate::model::{Plan, ReturnSource};
use crate::scenarios::{self, ScenarioResult};
use crate::sim::{SimResult, Simulator};
use crate::validate::{self, Issue, Severity};

/// Example plan for «Neu».
/// #138: Templates of the property gain tax (Zurich, Aargau), single source for the user interface and tests.
pub fn gain_templates() -> Value {
    serde_json::from_str(include_str!("../assets/gain-templates.json")).expect("Vorlagen sind gültig")
}

/// #63: Cantons and municipalities of the built-in tax data file with tax year, source, note and deduction rules.
pub fn tax_locations() -> crate::taxdata::LocationsOut {
    crate::taxdata::locations()
}

pub fn example_plan() -> Value {
    serde_json::from_str(include_str!("../assets/example.vplan")).expect("Beispiel ist gültig")
}

pub(crate) fn parse(plan: &Value) -> Result<Plan, Issue> {
    serde_json::from_value(plan.clone()).map_err(|e| Issue {
        severity: Severity::Error,
        section: "general",
        message: msg!(plan_incomplete_detail, detail = e.to_string()),
        field: None,
    })
}

/// Result of [`upgrade_plan`]: plan brought up to date and a note on every change.
#[derive(Serialize)]
pub struct UpgradeOut {
    pub plan: Value,
    pub notes: Vec<Message>,
}

/// Bring an older plan up to date when loading (#52, #51). The limits come along as `limits` in the
/// plan (settings, #25); they are not taken over.
pub fn upgrade_plan(plan: &Value) -> UpgradeOut {
    let mut v = plan.clone();
    let notes = crate::upgrade::upgrade(&mut v);
    if let Some(o) = v.as_object_mut() {
        o.remove("limits");
    }
    UpgradeOut { plan: v, notes }
}

pub fn validate(plan: &Value) -> Vec<Issue> {
    if let Err(e) = crate::model::check_format(plan) {
        return vec![Issue {
            severity: Severity::Error,
            section: "general",
            message: e.message(),
            field: None,
        }];
    }
    match parse(plan) {
        Ok(p) => validate::validate(&p),
        Err(i) => vec![i],
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewOut {
    pub issues: Vec<Issue>,
    pub result: Option<SimResult>,
    pub error: Option<Message>,
    /// What the preview computes with, if not with the fixed values (e.g. average of the series).
    pub basis: Option<Message>,
    /// Pillar 3a payout plan per account (#55).
    pub pillar3a: Vec<crate::sim::Pillar3aView>,
    /// #191: AHV pension per person with reduction or supplement (and cap) as the simulation computes with it.
    pub ahv: Vec<crate::sim::AhvView>,
    /// #249: Taxable wealth per tax year with the properties (card «Abzüge vom Vermögen»).
    pub wealth_tax: Vec<crate::sim::WealthTaxYear>,
}

/// The preview computes one run with constant returns. For series these are their mean
/// annual returns, for the parametric model the expected geometric stock return.
fn preview_returns(p: &mut Plan, base_dir: &Path, library: Option<&Path>) -> Option<Message> {
    use crate::model::Series;
    match p.returns.source {
        ReturnSource::Fixed => None,
        ReturnSource::Parametric => {
            // #100/#101: with a series the suggested values (with «auto») and the means of bonds and inflation
            let ds = crate::batch::returns_series(p, base_dir, library).ok().flatten();
            let (q, _) = crate::calibrate::resolve(&p.returns.parametric.clone().unwrap_or_default(), ds.as_ref());
            let g = (q.mu - 0.5 * q.sigma * q.sigma + q.jump_lambda * q.jump_mu).exp() - 1.0;
            let f = &mut p.returns.fixed;
            f.stocks = Series::Number(g);
            match ds {
                Some(ds) => {
                    let (_, b, i) = ds.averages();
                    f.bonds = Series::Number(b);
                    f.inflation = Series::Number(i);
                    Some(msg!(preview_model_and_series, stocks = g, bonds = b, inflation = i))
                }
                None => Some(msg!(preview_model, stocks = g)),
            }
        }
        _ => {
            let ds = crate::batch::returns_series(p, base_dir, library).ok()??;
            let f = &mut p.returns.fixed;
            let (s, b, i) = ds.averages();
            f.stocks = Series::Number(s);
            f.bonds = Series::Number(b);
            f.inflation = Series::Number(i);
            Some(msg!(preview_series, stocks = s, bonds = b, inflation = i))
        }
    }
}

/// Single run with constant returns for the live preview in the editor.
pub fn preview(plan: &Value, base_dir: &Path, library: Option<&Path>) -> PreviewOut {
    let mut p = match parse(plan) {
        Ok(p) => p,
        Err(i) => {
            return PreviewOut {
                issues: vec![i],
                result: None,
                error: None,
                basis: None,
                pillar3a: vec![],
                ahv: vec![],
                wealth_tax: vec![],
            }
        }
    };
    let issues = validate::validate(&p);
    let ahv = crate::sim::ahv_view(&p);
    let basis = preview_returns(&mut p, base_dir, library);
    p.returns.source = ReturnSource::Fixed;
    match Simulator::new(&p) {
        Ok(sim) => {
            let path = market::fixed_path(&sim.fixed, sim.start, sim.months());
            let mut result = sim.run(&path, true);
            let pillar3a = std::mem::take(&mut result.pillar3a);
            let wealth_tax = std::mem::take(&mut result.wealth_tax);
            PreviewOut {
                issues,
                result: Some(result),
                error: None,
                basis,
                pillar3a,
                ahv,
                wealth_tax,
            }
        }
        Err(e) => PreviewOut {
            issues,
            result: None,
            error: Some(e.message()),
            basis,
            pillar3a: vec![],
            ahv,
            wealth_tax: vec![],
        },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOut {
    pub issues: Vec<Issue>,
    pub error: Option<Message>,
    /// With exactly one run: the run with all monthly values.
    pub single: Option<SimResult>,
    /// With several runs: metrics, bands and selected runs in detail.
    pub batch: Option<BatchOut>,
    /// With variations: metrics per combination.
    pub scenarios: Option<ScenariosOut>,
    pub elapsed_ms: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenariosOut {
    /// Varied parameters (e.g. `wealth.stockShare`).
    pub params: Vec<String>,
    pub results: Vec<ScenarioResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOut {
    pub summary: BatchSummary,
    pub bands: Bands,
    pub stats: batch::BatchStats,
    /// Runs at the worst end wealth (0) and at the 1st, 5th, 50th, 95th and 99th percentile, with monthly values
    /// (nominal order, `summary.percentiles`).
    pub details: Vec<(u32, SimResult)>,
    /// #265: The same in the order by the end wealth in today's purchasing power (`summary.percentiles_real`).
    /// Missing in the JSON when empty (results before #265).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details_real: Vec<(u32, SimResult)>,
}

/// Percentiles of the runs kept with all monthly values in [`BatchOut::details`].
pub const DETAIL_PERCENTS: [u32; 6] = [0, 1, 5, 50, 95, 99];

/// Runs of the percentiles [`DETAIL_PERCENTS`] with all monthly values; a run in both orders is simulated once.
fn detail_runs(sim: &Simulator, paths: &batch::Paths, orders: [&[batch::Percentile]; 2]) -> [Vec<(u32, SimResult)>; 2] {
    let mut done: std::collections::HashMap<usize, SimResult> = std::collections::HashMap::new();
    orders.map(|list| {
        list.iter()
            .filter(|x| DETAIL_PERCENTS.contains(&x.percent))
            .map(|x| {
                let run = done
                    .entry(x.index)
                    .or_insert_with(|| sim.run(&paths.get(x.index), true));
                (x.percent, run.clone())
            })
            .collect()
    })
}

/// Parameters of all strategies with default ranges and the values of the plan (for the optimization).
pub fn opt_params(plan: &Value) -> Vec<crate::optimize::CatalogStrategy> {
    crate::optimize::catalog(plan)
}

/// Computing time per run on this machine, as the basis for duration estimates.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchOut {
    /// Microseconds per run (all cores together), including generating the return path.
    pub us_per_run: f64,
    /// Of which generating the return path (with Monte Carlo; otherwise close to 0).
    pub us_generate: f64,
    pub cores: usize,
    /// Months per run.
    pub months: usize,
    /// Number of runs if fixed by the source (all historical start dates), otherwise `None`.
    pub fixed_paths: Option<usize>,
}

/// Passes over all `n` paths and number of runs of one measurement: at least 2000 runs.
fn bench_runs(n: usize) -> (usize, usize) {
    const MIN_RUNS: usize = 2000;
    let reps = MIN_RUNS.div_ceil(n);
    (reps, n * reps)
}

/// Microseconds per item from a duration in seconds.
fn micros_each(seconds: f64, count: usize) -> f64 {
    seconds * 1e6 / count as f64
}

/// Measures how long one run of the plan takes (a few thousand runs, usually under one second).
pub fn bench(plan: &Value, base_dir: &Path, library: Option<&Path>) -> Result<BenchOut, Message> {
    let mut v = plan.clone();
    for key in ["bootstrap", "parametric"] {
        if let Some(n) = v["returns"][key]["paths"].as_u64() {
            v["returns"][key]["paths"] = Value::from(n.min(1000));
        }
    }
    // #221: only in an object (indexing a list or a number would panic); anything else fails when reading the plan
    if let Some(Value::Object(s)) = v.as_object_mut().map(|o| {
        o.entry("scenarios")
            .or_insert_with(|| Value::Object(Default::default()))
    }) {
        s.insert("sweeps".into(), Value::Array(vec![]));
    }
    let p = parse(&v).map_err(|i| i.message)?;
    let sim = Simulator::new(&p).map_err(|e| e.message())?;
    let data = batch::load_series(&p, base_dir, library).map_err(|e| e.message())?;
    let paths = batch::path_source(&p, &sim, data.as_ref()).map_err(|e| e.message())?;
    let n = paths.len();
    let (reps, runs) = bench_runs(n);
    // With only one path (fixed returns, historical course) compute the same path several times.
    let single = (n == 1).then(|| vec![paths.get(0).into_owned(); reps]);
    let pass = || match &single {
        Some(repeated) => {
            batch::run_all(&sim, repeated);
        }
        None => {
            for _ in 0..reps {
                batch::run_paths(&sim, &paths);
            }
        }
    };
    let timed = || {
        let t0 = Instant::now();
        pass();
        t0.elapsed().as_secs_f64()
    };
    // The first pass warms up threads and caches; the faster one counts.
    let dt = timed().min(timed());
    let gen_pass = || {
        use rayon::prelude::*;
        let t0 = Instant::now();
        let k = (0..n).into_par_iter().map(|i| paths.get(i).stocks.len()).sum::<usize>();
        std::hint::black_box(k);
        t0.elapsed().as_secs_f64()
    };
    let generate = gen_pass().min(gen_pass());
    Ok(BenchOut {
        us_per_run: micros_each(dt, runs),
        us_generate: micros_each(generate, n),
        cores: rayon::current_num_threads(),
        months: sim.months(),
        fixed_paths: (p.returns.source == ReturnSource::Rolling).then_some(n),
    })
}

/// Applies `scenarios.select` (list of `{param, value}`) to a copy of the plan and removes the
/// variations. `None` if there is no selection.
fn select_variation(plan: &Value) -> crate::Result<Option<Value>> {
    let Some(sel) = plan["scenarios"]["select"].as_array() else {
        return Ok(None);
    };
    let mut v = plan.clone();
    for item in sel {
        let param = item["param"]
            .as_str()
            .ok_or_else(|| crate::Error::Invalid(msg!(variation_selection_without_param)))?;
        scenarios::apply(&mut v, param, &item["value"])?;
    }
    if let Some(sc) = v["scenarios"].as_object_mut() {
        sc.remove("select");
        sc.insert("sweeps".into(), Value::Array(vec![]));
    }
    Ok(Some(v))
}

/// Simulation according to `returns.source`. `base_dir`: directory of the project file (for series),
/// `library`: folder of the data library.
pub fn run(plan: &Value, base_dir: &Path, library: Option<&Path>) -> RunOut {
    run_as(None, plan, base_dir, library)
}

/// Like [`run`], as a computation with the id `job` (cancellation and progress apply only to it); if not given, the
/// engine assigns one.
pub fn run_as(job: Option<crate::progress::JobId>, plan: &Value, base_dir: &Path, library: Option<&Path>) -> RunOut {
    let t0 = Instant::now();
    let fail = |issues, e: Message| RunOut {
        issues,
        error: Some(e),
        single: None,
        batch: None,
        scenarios: None,
        elapsed_ms: 0.0,
    };
    // Compute a single variation step: `scenarios.select` = [{param, value}] replaces the variations.
    let selected;
    let plan = match select_variation(plan) {
        Ok(Some(v)) => {
            selected = v;
            &selected
        }
        Ok(None) => plan,
        Err(e) => return fail(vec![], e.message()),
    };
    let p = match parse(plan) {
        Ok(p) => p,
        Err(i) => return fail(vec![i], msg!(plan_incomplete)),
    };
    let job = crate::progress::start_as(job, 0);
    let mut issues = validate::validate(&p);
    let sim = match Simulator::new(&p) {
        Ok(s) => s,
        Err(e) => return fail(issues, e.message()),
    };
    let data = match batch::load_series(&p, base_dir, library) {
        Ok(d) => d,
        Err(e) => return fail(issues, e.message()),
    };
    if let Some(i) = data.as_ref().and_then(|d| currency_issue(d, &p.project.currency)) {
        issues.push(i);
    }
    let source = match batch::path_source(&p, &sim, data.as_ref()) {
        Ok(x) => x,
        Err(e) => return fail(issues, e.message()),
    };
    crate::progress::set_total(source.len() as u64);
    let cancelled = |issues| fail(issues, msg!(cancelled));
    // #215: With variations, all combinations and the plan itself use the same paths (generated once).
    let mut cache = batch::PathCache::default();
    let (scenarios, paths) = if p.scenarios.sweeps.is_empty() {
        (None, source)
    } else {
        match run_scenarios(plan, &p, data.as_ref(), source.len(), &mut cache) {
            Ok(s) => (Some(s), cache.paths(source)),
            Err(e) => return fail(issues, e.message()),
        }
    };
    if crate::progress::cancelled() {
        return cancelled(issues);
    }
    if paths.len() == 1 {
        let single = sim.run(&paths.get(0), true);
        return RunOut {
            issues,
            error: None,
            single: Some(single),
            batch: None,
            scenarios,
            elapsed_ms: t0.elapsed().as_secs_f64() * 1e3,
        };
    }
    let (results, bands) = batch::run_with_bands(&sim, &paths);
    if crate::progress::cancelled() {
        return cancelled(issues);
    }
    drop(job);
    let summary = batch::summarize(&results);
    let stats = batch::stats(&sim, &results);
    let [details, details_real] = detail_runs(&sim, &paths, [&summary.percentiles, &summary.percentiles_real]);
    RunOut {
        issues,
        error: None,
        single: None,
        batch: Some(BatchOut {
            summary,
            bands,
            stats,
            details,
            details_real,
        }),
        scenarios,
        elapsed_ms: t0.elapsed().as_secs_f64() * 1e3,
    }
}

/// Recomputed runs of a variation step (#24).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RerunOut {
    pub error: Option<Message>,
    /// Per entry of the request: percentile and the run with all monthly values.
    pub details: Vec<(u32, SimResult)>,
}

/// #221: Most runs that [`rerun_paths`] recomputes in one call (each with all monthly values in memory).
pub const MAX_PICKS: usize = 100;

/// Recomputes single runs of a variation step without simulating all paths: `plan.scenarios.select` selects the
/// step, `picks` are pairs of percentile and path number (`Percentile.index` of the scenarios).
pub fn rerun_paths(plan: &Value, picks: &[(u32, usize)], base_dir: &Path, library: Option<&Path>) -> RerunOut {
    let fail = |e: Message| RerunOut {
        error: Some(e),
        details: vec![],
    };
    // #221: Every run keeps all monthly values; the user interface asks for six.
    if picks.len() > MAX_PICKS {
        return fail(msg!(rerun_too_many_picks, count = picks.len(), max = MAX_PICKS));
    }
    let run = || -> crate::Result<Vec<(u32, SimResult)>> {
        let selected = select_variation(plan)?;
        let plan = selected.as_ref().unwrap_or(plan);
        let p: Plan = serde_json::from_value(plan.clone())
            .map_err(|e| crate::Error::Invalid(msg!(raw_error, detail = e.to_string())))?;
        let sim = Simulator::new(&p)?;
        let data = batch::load_series(&p, base_dir, library)?;
        let paths = batch::path_source(&p, &sim, data.as_ref())?;
        picks
            .iter()
            .map(|&(percent, index)| {
                if index >= paths.len() {
                    return Err(crate::Error::Invalid(msg!(
                        path_index_outside,
                        index = index,
                        max = paths.len().saturating_sub(1)
                    )));
                }
                Ok((percent, sim.run(&paths.get(index), true)))
            })
            .collect()
    };
    match run() {
        Ok(details) => RerunOut { error: None, details },
        Err(e) => fail(e.message()),
    }
}

/// Every combination of the variations with all return paths. Monte Carlo paths are the same in all
/// scenarios (same seed), so differences come from the parameters; `cache` generates them only once (#215).
fn run_scenarios(
    plan: &Value,
    p: &Plan,
    data: Option<&DataSet>,
    per_scenario: usize,
    cache: &mut batch::PathCache,
) -> crate::Result<ScenariosOut> {
    let sweeps = scenarios::parse(&p.scenarios.sweeps)?;
    let combos = scenarios::combinations(&sweeps)?;
    let params: Vec<String> = sweeps.iter().map(|s| s.param.clone()).collect();
    crate::progress::add_total((combos.len() * per_scenario) as u64);
    let results = combos
        .into_iter()
        .map(|values| {
            let mut v = plan.clone();
            v["scenarios"]["sweeps"] = Value::Array(vec![]);
            let outcome = (|| -> crate::Result<ScenarioResult> {
                if crate::progress::cancelled() {
                    return Err(crate::Error::Cancelled);
                }
                for (param, value) in params.iter().zip(&values) {
                    scenarios::apply(&mut v, param, value)?;
                }
                let q: Plan = serde_json::from_value(v)
                    .map_err(|e| crate::Error::Invalid(msg!(raw_error, detail = e.to_string())))?;
                let sim = Simulator::new(&q)?;
                let paths = cache.paths(batch::path_source(&q, &sim, data)?);
                let results = batch::run_paths(&sim, &paths);
                let s = batch::summarize(&results);
                let pct = |x: u32| {
                    s.percentiles
                        .iter()
                        .find(|p| p.percent == x)
                        .map_or(0.0, |p| p.end_total)
                };
                let mut at_ret: Vec<f64> = results.iter().filter_map(|r| r.summary.total_at_retirement).collect();
                at_ret.sort_by(f64::total_cmp);
                Ok(ScenarioResult {
                    values: values.clone(),
                    runs: s.runs,
                    success_rate: s.success_rate,
                    p5: pct(5),
                    p50: pct(50),
                    p95: pct(95),
                    // #265: the median of the real order, like the percentiles in today's purchasing power
                    p50_real: s
                        .percentiles_real
                        .iter()
                        .find(|p| p.percent == 50)
                        .and_then(|p| p.end_total_real)
                        .unwrap_or(0.0),
                    at_retirement: at_ret.get(at_ret.len() / 2).copied(),
                    earliest_depletion: s.earliest_depletion.map(|(m, _)| m),
                    // #130: without real end wealth per percentile, the scenarios do not show it
                    percentiles: s
                        .percentiles
                        .into_iter()
                        .map(|p| batch::Percentile {
                            end_total_real: None,
                            ..p
                        })
                        .collect(),
                    percentiles_real: s.percentiles_real,
                    error: None,
                })
            })();
            outcome.unwrap_or_else(|e| ScenarioResult {
                values: values.clone(),
                runs: 0,
                success_rate: 0.0,
                p5: 0.0,
                p50: 0.0,
                p95: 0.0,
                p50_real: 0.0,
                at_retirement: None,
                earliest_depletion: None,
                percentiles: vec![],
                percentiles_real: vec![],
                error: Some(e.message()),
            })
        })
        .collect();
    if crate::progress::cancelled() {
        return Err(crate::Error::Cancelled);
    }
    Ok(ScenariosOut { params, results })
}

/// Warning if the series is in a different currency than the plan.
pub fn currency_issue(ds: &DataSet, plan_currency: &str) -> Option<Issue> {
    let c = ds.currency()?;
    (!c.eq_ignore_ascii_case(plan_currency)).then(|| Issue {
        severity: Severity::Warning,
        section: "returns",
        message: msg!(
            currency_mismatch,
            series = ds
                .meta
                .title
                .clone()
                .unwrap_or_else(|| msg!(plain_text, text = ds.name)),
            currency = c,
            plan_currency = plan_currency
        ),
        field: None,
    })
}

/// Short info about a series (for display in the editor).
/// `series` is a reference to a file or an investment strategy (`strategy:<JSON>`, #69).
pub fn series_info(series: &str, base_dir: &Path, library: Option<&Path>) -> crate::Result<SeriesInfo> {
    let (ds, parts) = match crate::strategy::from_reference_with_parts(series, library) {
        Some(r) => r?,
        None => (
            DataSet::load_csv(&library::resolve(series, base_dir, library)?)?,
            vec![],
        ),
    };
    Ok(SeriesInfo {
        parts,
        parametric: Some(crate::calibrate::suggestion(&ds)),
        ..SeriesInfo::of(&ds)
    })
}

#[derive(Serialize)]
pub struct LibraryOut {
    pub dir: String,
    pub entries: Vec<library::Entry>,
}

/// Content of the data library; creates missing bundled series.
pub fn library_list(library: &Path) -> crate::Result<LibraryOut> {
    library::seed(library)?;
    Ok(LibraryOut {
        dir: library.to_string_lossy().into_owned(),
        entries: library::list(library)?,
    })
}

pub fn import_shiller(library: &Path, file: &Path) -> crate::Result<ImportReport> {
    library::seed(library)?;
    library::import_shiller(library, file)
}

pub fn import_snb(library: &Path, files: &[std::path::PathBuf]) -> crate::Result<ImportReport> {
    library::seed(library)?;
    library::import_snb(library, files)
}

#[derive(Serialize)]
pub struct ImportOut {
    pub plan: Value,
    pub notes: Vec<Message>,
}

/// Convert an old `config.json`; relative data paths start at `base_dir`.
pub fn import_legacy(text: &str, start_year: i32, base_dir: &Path, name: &str) -> crate::Result<ImportOut> {
    let imp = crate::legacy::import(text, start_year, base_dir, name)?;
    Ok(ImportOut {
        plan: serde_json::to_value(&imp.plan).expect("serialisierbar"),
        notes: imp.notes,
    })
}

/// Read a project file; does not detect an old `config.json` automatically (use `import_legacy` for that).
pub fn read_plan(path: &Path) -> crate::Result<Value> {
    let text = std::fs::read_to_string(path).map_err(|e| crate::io_error(path, e))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| crate::file_invalid(path, e.to_string()))?;
    if v.get("format").is_none() && v.get("General").is_some() {
        return Err(crate::Error::Invalid(msg!(legacy_config_detected)));
    }
    // #144: do not open a file of a newer version silently
    crate::model::check_format(&v).map_err(|e| crate::file_invalid(path, e.message()))?;
    Ok(v)
}

/// Writes the plan; an old format identifier (legacy: RetirementPlanner) is replaced by the new one.
pub fn write_plan(path: &Path, plan: &Value) -> crate::Result<()> {
    let mut plan = Cow::Borrowed(plan);
    if plan.get("format").and_then(Value::as_str) == Some(crate::model::LEGACY_FORMAT) {
        plan.to_mut()["format"] = Value::String(crate::model::FORMAT.into());
    }
    let text = serde_json::to_string_pretty(&plan).expect("serialisierbar");
    write_atomic(path, text.as_bytes()).map_err(|e| crate::io_error(path, e))
}

/// #136: Writes to a temporary file in the same folder first, flushes it to disk and only then replaces
/// the target file. If something fails, the previous file stays unchanged and the temporary one is removed.
/// If the target is a symbolic link, the file it points to is replaced (the link stays).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let target = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => std::fs::canonicalize(path)?,
        _ => path.to_path_buf(),
    };
    let tmp = temp_path(&target);
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, &target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// #136: Temporary file for [`write_atomic`] in the folder of the target file (only there is the replacement atomic and on the same
/// disk); without a folder in the path, in the working directory.
fn temp_path(target: &Path) -> std::path::PathBuf {
    let dir = match target.parent() {
        Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dir.join(format!(".{name}.{}.tmp", std::process::id()))
}

/// Optimization of the withdrawal strategy (see [`crate::optimize`]).
pub fn optimize(
    plan: &Value,
    request: &Value,
    base_dir: &Path,
    library: Option<&Path>,
) -> crate::Result<crate::optimize::Report> {
    let req: crate::optimize::Request = serde_json::from_value(request.clone())
        .map_err(|e| crate::Error::Invalid(msg!(optimize_problem, problem = e.to_string())))?;
    crate::optimize::run_request(plan, base_dir, library, &req)
}

/// #207: Result of [`run`] in the fixed, compact format of the command line (`run --json`).
pub fn run_report(plan: &Value, base_dir: &Path, library: Option<&Path>) -> crate::report::RunReport {
    crate::report::run(&run(plan, base_dir, library))
}

/// #214: Metrics of a plan (`run --metric`), collected on the plan's own paths as in [`run`] (without variations).
/// Errors in the metrics come together as one message with the list (`plan_errors`).
pub fn measure(
    plan: &Value,
    metrics: &[Value],
    base_dir: &Path,
    library: Option<&Path>,
) -> Result<Vec<crate::measure::MetricValue>, Message> {
    let list = |e: Vec<Message>| msg!(plan_errors, errors = e);
    let metrics = crate::measure::parse(metrics).map_err(list)?;
    let p = parse(plan).map_err(|i| i.message)?;
    let sim = Simulator::new(&p).map_err(|e| e.message())?;
    let m = crate::measure::Measurement::new(&p, &sim, metrics).map_err(list)?;
    let data = batch::load_series(&p, base_dir, library).map_err(|e| e.message())?;
    let paths = batch::path_source(&p, &sim, data.as_ref()).map_err(|e| e.message())?;
    m.run(&sim, &paths).map_err(|e| e.message())
}

/// #207: Result of [`validate`] in the fixed format of the command line (`validate --json`).
pub fn validate_report(plan: &Value) -> crate::report::ValidateReport {
    crate::report::validate(validate(plan))
}

/// #207/AK2, AK6, AK7: Sets values by field path (`path[@time]=value`). Path, type and point in time are checked here;
/// on any error the result is the list of messages and nothing is changed.
pub fn set_fields(plan: &Value, assignments: &[String]) -> Result<Value, Vec<Message>> {
    crate::fieldpath::set(plan, assignments)
}

/// #207/AK4: Differences between two plans: field path, old and new value.
pub fn diff_plans(a: &Value, b: &Value) -> Vec<crate::fieldpath::Change> {
    crate::fieldpath::diff(a, b)
}

/// #209: Reads a saved result, a run or a list of results; an error in the form as a message.
fn read<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, Message> {
    serde_json::from_value(value.clone()).map_err(|e| msg!(raw_error, detail = e.to_string()))
}

/// #209/AK1: Yearly values of a run (a `SimResult` as in `single` or `batch.details`), nominal and real.
pub fn yearly(run: &Value) -> Result<crate::digest::Yearly, Message> {
    Ok(crate::digest::yearly(&read(run)?))
}

/// #209/AK1: Digest of a result (`{single}` or `{batch}` as [`run`] returns it): key figures and the verdict (result in
/// words as a category with its values). `null` for a result without a run.
pub fn digest(result: &Value) -> Result<Option<crate::digest::Digest>, Message> {
    Ok(crate::digest::digest(&read(result)?))
}

/// #209/AK1: Comparison of several results (`[{result, plan}]`): key figures side by side, percentiles and ages at
/// depletion of all results, differences of the plans against the first.
pub fn compare(items: &Value) -> Result<crate::digest::Comparison, Message> {
    Ok(crate::digest::compare(&read::<Vec<crate::digest::CompareInput>>(
        items,
    )?))
}

/// #216: Computes an experiment (changes, variations, metrics, constraints, objective) on a plan; see
/// [`crate::experiment`]. The main tool of the MCP server later (#210–#212).
pub fn evaluate(
    plan: &Value,
    experiment: &Value,
    base_dir: &Path,
    library: Option<&Path>,
) -> crate::experiment::EvaluateOut {
    crate::experiment::evaluate(plan, experiment, base_dir, library)
}

/// #219: Catalog for AIs: fields, forms, state variables, metric building blocks, examples, measured microseconds per
/// run (on `plan`, otherwise on the example plan) and limits; all sections or one. See [`crate::describe`]; later a
/// tool of the MCP server (#210).
pub fn describe(
    section: Option<crate::describe::Section>,
    plan: Option<&Value>,
    base_dir: &Path,
    library: Option<&Path>,
) -> crate::describe::Catalog {
    crate::describe::describe(section, plan, base_dir, library)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn example() -> Value {
        serde_json::from_str(include_str!("../assets/new.vplan")).unwrap()
    }

    /// #162/AK1: The temporary file is in the folder of the target file (otherwise the replacement would not be atomic and would fail
    /// between two disks); without a folder in the path, in the working directory.
    #[test]
    fn issue_162_ak1_temporary_file_lies_next_to_the_target() {
        let pid = std::process::id();
        let dir = std::env::temp_dir().join("ordner");
        assert_eq!(
            temp_path(&dir.join("planung.vplan")),
            dir.join(format!(".planung.vplan.{pid}.tmp"))
        );
        assert_eq!(
            temp_path(Path::new("planung.vplan")),
            Path::new(".").join(format!(".planung.vplan.{pid}.tmp"))
        );
    }

    // #40: Preview with the parametric model: expected geometric stock return exp(μ − σ²/2 + λ·μJ) − 1.
    // slow: preview of the example plan for each return source
    #[test]
    fn preview_basis_for_each_source() {
        let mut v = example();
        v["returns"]["source"] = json!("parametric");
        v["returns"]["parametric"] =
            json!({"paths": 10, "mu": 0.08, "sigma": 0.2, "df": 6, "jumpLambda": 0.5, "jumpMu": -0.1});
        // 0.08 − 0.02 − 0.05 = 0.01 ⇒ e^0.01 − 1 ≈ 1.005 %
        let out = preview(&v, Path::new("."), None);
        // #101: with an investment strategy also the means of bonds and inflation of the series
        let basis = out.basis.unwrap();
        assert!(
            basis.starts_with("Preview with the model's expected stock return (1.0 % per year) and the averages of the time series: bonds "),
            "{basis}"
        );
        v["returns"]["parametric"]["mu"] = json!(0.16);
        let out = preview(&v, Path::new("."), None);
        assert!(out.basis.unwrap().contains("(9.4 % per year)"));
        // Without a series only the stock return
        let mut w = v.clone();
        w["returns"]["strategy"]["stocks"] = json!("worldCustom");
        w["returns"]["strategy"]["weights"] = json!({"ch": 1.0, "usa": 1.0, "eu": 0.0, "apac": 0.0});
        let out = preview(&w, Path::new("."), None);
        assert_eq!(
            out.basis.as_deref(),
            Some("Preview with the model's expected stock return (9.4 % per year).")
        );
        // #100: with «auto» the suggested values of the stock index instead of the saved ones
        v["returns"]["parametric"]["auto"] = json!(true);
        let out = preview(&v, Path::new("."), None);
        assert!(!out.basis.unwrap().contains("(9.4 % per year)"));

        // Fixed returns: no information, even if a series is still entered.
        let mut v = example();
        v["returns"]["source"] = json!("fixed");
        v["returns"]["series"] = json!("library:Shiller CHF.csv");
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        assert_eq!(preview(&v, Path::new("."), Some(&lib)).basis, None);
    }

    #[test]
    fn strategies_for_the_optimization() {
        let p = opt_params(&example());
        assert!(!p.is_empty());
        assert_eq!(p.len(), crate::optimize::catalog(&example()).len());
    }

    #[test]
    fn currency_warning_only_for_other_currencies() {
        let mut ds = DataSet {
            name: "Reihe".into(),
            ..Default::default()
        };
        assert!(currency_issue(&ds, "CHF").is_none());
        ds.meta.currency = Some("chf".into());
        assert!(currency_issue(&ds, "CHF").is_none());
        ds.meta.currency = Some("USD".into());
        let i = currency_issue(&ds, "CHF").unwrap();
        assert_eq!(i.severity, Severity::Warning);
        assert!(i.message.contains("“Reihe” is in USD"), "{}", i.message);
    }

    // Progress: runs of the plan plus runs of all variations; measured time in milliseconds.
    #[test]
    fn progress_total_and_elapsed_time() {
        let mut v = example();
        v["returns"]["source"] = json!("parametric");
        v["returns"]["parametric"] = json!({"paths": 3, "mu": 0.07, "sigma": 0.15, "df": 6});
        v["scenarios"] = json!({"sweeps": [{"param": "wealth.stockShare", "from": 0.4, "to": 0.6, "step": 0.2}]});
        let out = run(&v, Path::new("."), None);
        assert!(out.error.is_none(), "{:?}", out.error);
        assert_eq!(out.scenarios.as_ref().unwrap().results.len(), 2);
        assert_eq!(crate::progress::snapshot().total, 3 + 2 * 3);
        assert!(out.batch.is_some());
        assert!(out.elapsed_ms > 0.01 && out.elapsed_ms < 900.0, "{}", out.elapsed_ms);

        let mut v = example();
        v["returns"]["source"] = json!("fixed");
        let out = run(&v, Path::new("."), None);
        assert!(out.single.is_some());
        assert!(out.elapsed_ms > 0.01 && out.elapsed_ms < 900.0, "{}", out.elapsed_ms);
    }

    // Measurement: at least 2000 runs; conversion to microseconds.
    #[test]
    fn bench_run_counts_and_units() {
        assert_eq!(bench_runs(1), (2000, 2000));
        assert_eq!(bench_runs(3), (667, 2001));
        assert_eq!(bench_runs(1000), (2, 2000));
        assert_eq!(bench_runs(5000), (1, 5000));
        assert_eq!(micros_each(0.5, 1000), 500.0);
    }

    // Runs take time (microseconds per run > 0), with one and with many paths; the number of paths is fixed only
    // with «rolling».
    // slow: performance measurement with one and with many paths
    #[test]
    fn bench_measures_runs() {
        let mut v = example();
        v["returns"]["source"] = json!("fixed");
        let b = bench(&v, Path::new("."), None).unwrap();
        assert_eq!(b.fixed_paths, None);
        assert!(b.us_per_run > 0.05 && b.us_per_run.is_finite(), "{b:?}");
        let single = b.us_per_run;
        v["returns"]["source"] = json!("rolling");
        v["returns"]["series"] = json!("library:Shiller CHF.csv");
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        let b = bench(&v, Path::new("."), Some(&lib)).unwrap();
        assert!(b.fixed_paths.is_some_and(|n| n > 100), "{b:?}");
        // Same plan, runs of the same length: per run about as much time as with one path (generous limit)
        assert!(b.us_per_run > 0.2 * single, "{b:?}, ein Pfad {single}");
    }

    /// #207/AK6: path and type are checked by the engine interface; the command line only passes the texts on.
    #[test]
    fn issue_207_ak6_engine_checks_path_and_type() {
        let plan = example_plan();
        let list = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let e = set_fields(
            &plan,
            &list(&["wealth.stockShare=0.5", "wealth.nope=1", "wealth.cash=true"]),
        )
        .unwrap_err();
        assert_eq!(
            e.iter().map(Message::code).collect::<Vec<_>>(),
            ["field_path_unknown", "field_type_wrong"]
        );
        let v = set_fields(&plan, &list(&["wealth.stockShare@retire=0.5"])).unwrap();
        assert_eq!(diff_plans(&plan, &v).len(), 3);
        assert!(validate_report(&v).valid);
        let r = run_report(&v, Path::new("."), None);
        assert!(r.error.is_none() && r.single.is_some());
    }
}
