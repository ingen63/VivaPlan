//! JSON-Schnittstelle für die Oberfläche (Desktop-App und lokaler Entwicklungsserver).
//!
//! Alle Funktionen nehmen die Planung als JSON-Wert entgegen, wie der Editor sie hält, und liefern
//! serialisierbare Ergebnisse. Fehler in der Planung werden als Prüfmeldungen zurückgegeben.

use std::borrow::Cow;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;

use crate::batch::{self, Bands, BatchSummary};
use crate::library::{self, ImportReport, SeriesInfo};
use crate::market::{self, DataSet};
use crate::model::{Plan, ReturnSource};
use crate::scenarios::{self, ScenarioResult};
use crate::sim::{SimResult, Simulator};
use crate::validate::{self, Issue, Severity};

/// Beispielplanung für «Neu».
pub fn example_plan() -> Value {
    serde_json::from_str(include_str!("../assets/beispiel.vplan")).expect("Beispiel ist gültig")
}

fn parse(plan: &Value) -> Result<Plan, Issue> {
    serde_json::from_value(plan.clone()).map_err(|e| Issue {
        severity: Severity::Error,
        section: "general",
        message: format!("Die Projektdatei ist unvollständig: {e}"),
        field: None,
    })
}

/// Ergebnis von [`upgrade_plan`]: Planung auf dem aktuellen Stand und Hinweise zu jeder Änderung.
#[derive(Serialize)]
pub struct UpgradeOut {
    pub plan: Value,
    pub notes: Vec<String>,
}

/// Ältere Planung beim Laden auf den aktuellen Stand bringen (#52, #51). Die Grenzen kommen als `limits` in der
/// Planung mit (Einstellungen, #25); sie werden nicht übernommen.
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
            message: e.to_string(),
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
    pub error: Option<String>,
    /// Womit die Vorschau rechnet, falls nicht mit den fixen Werten (z. B. Durchschnitt der Reihe).
    pub basis: Option<String>,
    /// Auszahlungsplan der Säule 3a je Konto (#55).
    pub pillar3a: Vec<crate::sim::Pillar3aView>,
}

fn pct(v: f64) -> String {
    format!("{:.1} %", v * 100.0)
}

/// Die Vorschau rechnet einen Lauf mit konstanten Renditen. Bei Zeitreihen sind das deren mittlere
/// Jahresrenditen, beim parametrischen Modell die erwartete geometrische Aktienrendite.
fn preview_returns(p: &mut Plan, base_dir: &Path, library: Option<&Path>) -> Option<String> {
    use crate::model::Series;
    match p.returns.source {
        ReturnSource::Fixed => None,
        ReturnSource::Parametric => {
            // #100/#101: mit Zeitreihe die Vorschlagswerte (bei «auto») und die Mittel von Obligationen und Inflation
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
                    Some(format!(
                        "Vorschau mit der erwarteten Aktienrendite des Modells ({} pro Jahr) und den Durchschnitten der Zeitreihe: Obligationen {}, Inflation {} pro Jahr.",
                        pct(g),
                        pct(b),
                        pct(i)
                    ))
                }
                None => Some(format!(
                    "Vorschau mit der erwarteten Aktienrendite des Modells ({} pro Jahr).",
                    pct(g)
                )),
            }
        }
        _ => {
            let ds = crate::batch::returns_series(p, base_dir, library).ok()??;
            let f = &mut p.returns.fixed;
            let (s, b, i) = ds.averages();
            f.stocks = Series::Number(s);
            f.bonds = Series::Number(b);
            f.inflation = Series::Number(i);
            Some(format!(
                "Vorschau mit den Durchschnitten der Zeitreihe: Aktien {}, Obligationen {}, Inflation {} pro Jahr.",
                pct(s),
                pct(b),
                pct(i)
            ))
        }
    }
}

/// Einzellauf mit konstanten Renditen für die Live-Vorschau im Editor.
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
            }
        }
    };
    let issues = validate::validate(&p);
    let basis = preview_returns(&mut p, base_dir, library);
    p.returns.source = ReturnSource::Fixed;
    match Simulator::new(&p) {
        Ok(sim) => {
            let path = market::fixed_path(&sim.fixed, sim.start, sim.months());
            let mut result = sim.run(&path, true);
            let pillar3a = std::mem::take(&mut result.pillar3a);
            PreviewOut {
                issues,
                result: Some(result),
                error: None,
                basis,
                pillar3a,
            }
        }
        Err(e) => PreviewOut {
            issues,
            result: None,
            error: Some(e.to_string()),
            basis,
            pillar3a: vec![],
        },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOut {
    pub issues: Vec<Issue>,
    pub error: Option<String>,
    /// Bei genau einem Lauf: der Lauf mit allen Monatswerten.
    pub single: Option<SimResult>,
    /// Bei mehreren Läufen: Kennzahlen, Bandbreite und ausgewählte Läufe im Detail.
    pub batch: Option<BatchOut>,
    /// Bei Variationen: Kennzahlen pro Kombination.
    pub scenarios: Option<ScenariosOut>,
    pub elapsed_ms: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenariosOut {
    /// Variierte Parameter (z. B. `wealth.stockShare`).
    pub params: Vec<String>,
    pub results: Vec<ScenarioResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOut {
    pub summary: BatchSummary,
    pub bands: Bands,
    pub stats: batch::BatchStats,
    /// Läufe beim schlechtesten Endvermögen (0) sowie beim 1., 5., 50., 95. und 99. Perzentil, mit Monatswerten.
    pub details: Vec<(u32, SimResult)>,
}

/// Parameter aller Strategien mit Standard-Bandbreiten und den Werten der Planung (für die Optimierung).
pub fn opt_params(plan: &Value) -> Vec<crate::optimize::CatalogStrategy> {
    crate::optimize::catalog(plan)
}

/// Rechenzeit pro Lauf auf diesem Rechner, als Grundlage für Dauerschätzungen.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchOut {
    /// Mikrosekunden pro Lauf (alle Kerne zusammen), inklusive Erzeugen des Renditepfads.
    pub us_per_run: f64,
    /// Davon das Erzeugen des Renditepfads (bei Monte Carlo; sonst nahe 0).
    pub us_generate: f64,
    pub cores: usize,
    /// Monate pro Lauf.
    pub months: usize,
    /// Anzahl Läufe, wenn sie durch die Quelle feststeht (alle historischen Startzeitpunkte), sonst `None`.
    pub fixed_paths: Option<usize>,
}

/// Durchgänge über alle `n` Pfade und Anzahl Läufe einer Messung: mindestens 2000 Läufe.
fn bench_runs(n: usize) -> (usize, usize) {
    const MIN_RUNS: usize = 2000;
    let reps = MIN_RUNS.div_ceil(n);
    (reps, n * reps)
}

/// Mikrosekunden pro Stück aus einer Dauer in Sekunden.
fn micros_each(seconds: f64, count: usize) -> f64 {
    seconds * 1e6 / count as f64
}

/// Misst, wie lange ein Lauf der Planung dauert (einige tausend Läufe, meist unter einer Sekunde).
pub fn bench(plan: &Value, base_dir: &Path, library: Option<&Path>) -> Result<BenchOut, String> {
    let mut v = plan.clone();
    for key in ["bootstrap", "parametric"] {
        if let Some(n) = v["returns"][key]["paths"].as_u64() {
            v["returns"][key]["paths"] = Value::from(n.min(1000));
        }
    }
    v["scenarios"]["sweeps"] = Value::Array(vec![]);
    let p = parse(&v).map_err(|i| i.message)?;
    let sim = Simulator::new(&p).map_err(|e| e.to_string())?;
    let data = batch::load_series(&p, base_dir, library).map_err(|e| e.to_string())?;
    let paths = batch::path_source(&p, &sim, data.as_ref()).map_err(|e| e.to_string())?;
    let n = paths.len();
    let (reps, runs) = bench_runs(n);
    // Bei nur einem Pfad (fixe Renditen, historischer Verlauf) denselben Pfad mehrfach rechnen.
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
    // Der erste Durchgang wärmt Threads und Zwischenspeicher auf; der schnellere zählt.
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

/// Wendet `scenarios.select` (Liste aus `{param, value}`) auf eine Kopie der Planung an und entfernt die
/// Variationen. `None`, wenn keine Auswahl vorliegt.
fn select_variation(plan: &Value) -> crate::Result<Option<Value>> {
    let Some(sel) = plan["scenarios"]["select"].as_array() else {
        return Ok(None);
    };
    let mut v = plan.clone();
    for item in sel {
        let param = item["param"]
            .as_str()
            .ok_or_else(|| crate::Error::Invalid("Auswahl der Variation ohne Parameter".into()))?;
        scenarios::apply(&mut v, param, &item["value"])?;
    }
    if let Some(sc) = v["scenarios"].as_object_mut() {
        sc.remove("select");
        sc.insert("sweeps".into(), Value::Array(vec![]));
    }
    Ok(Some(v))
}

/// Simulation gemäss `returns.source`. `base_dir`: Verzeichnis der Projektdatei (für Zeitreihen),
/// `library`: Ordner der Datenbibliothek.
pub fn run(plan: &Value, base_dir: &Path, library: Option<&Path>) -> RunOut {
    run_as(None, plan, base_dir, library)
}

/// Wie [`run`], als Berechnung mit der Kennung `job` (Abbruch und Fortschritt gelten nur für sie); ohne Angabe vergibt die
/// Engine eine.
pub fn run_as(job: Option<crate::progress::JobId>, plan: &Value, base_dir: &Path, library: Option<&Path>) -> RunOut {
    let t0 = Instant::now();
    let fail = |issues, e: String| RunOut {
        issues,
        error: Some(e),
        single: None,
        batch: None,
        scenarios: None,
        elapsed_ms: 0.0,
    };
    // Eine einzelne Variationsstufe rechnen: `scenarios.select` = [{param, value}] ersetzt die Variationen.
    let selected;
    let plan = match select_variation(plan) {
        Ok(Some(v)) => {
            selected = v;
            &selected
        }
        Ok(None) => plan,
        Err(e) => return fail(vec![], e.to_string()),
    };
    let p = match parse(plan) {
        Ok(p) => p,
        Err(i) => return fail(vec![i], "Die Projektdatei ist unvollständig.".into()),
    };
    let job = crate::progress::start_as(job, 0);
    let mut issues = validate::validate(&p);
    let sim = match Simulator::new(&p) {
        Ok(s) => s,
        Err(e) => return fail(issues, e.to_string()),
    };
    let data = match batch::load_series(&p, base_dir, library) {
        Ok(d) => d,
        Err(e) => return fail(issues, e.to_string()),
    };
    if let Some(i) = data.as_ref().and_then(|d| currency_issue(d, &p.project.currency)) {
        issues.push(i);
    }
    let paths = match batch::path_source(&p, &sim, data.as_ref()) {
        Ok(x) => x,
        Err(e) => return fail(issues, e.to_string()),
    };
    crate::progress::set_total(paths.len() as u64);
    let cancelled = |issues| fail(issues, crate::progress::CANCELLED.into());
    let scenarios = if p.scenarios.sweeps.is_empty() {
        None
    } else {
        match run_scenarios(plan, &p, data.as_ref(), paths.len()) {
            Ok(s) => Some(s),
            Err(e) => return fail(issues, e.to_string()),
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
    let details = summary
        .percentiles
        .iter()
        .filter(|x| matches!(x.percent, 0 | 1 | 5 | 50 | 95 | 99))
        .map(|x| (x.percent, sim.run(&paths.get(x.index), true)))
        .collect();
    RunOut {
        issues,
        error: None,
        single: None,
        batch: Some(BatchOut {
            summary,
            bands,
            stats,
            details,
        }),
        scenarios,
        elapsed_ms: t0.elapsed().as_secs_f64() * 1e3,
    }
}

/// Nachgerechnete Läufe einer Variationsstufe (#24).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RerunOut {
    pub error: Option<String>,
    /// Pro Eintrag der Anfrage: Perzentil und der Lauf mit allen Monatswerten.
    pub details: Vec<(u32, SimResult)>,
}

/// Rechnet einzelne Läufe einer Variationsstufe nach, ohne alle Pfade zu simulieren: `plan.scenarios.select` wählt die
/// Stufe, `picks` sind Paare aus Perzentil und Pfadnummer (`Percentile.index` der Szenarien).
pub fn rerun_paths(plan: &Value, picks: &[(u32, usize)], base_dir: &Path, library: Option<&Path>) -> RerunOut {
    let fail = |e: String| RerunOut {
        error: Some(e),
        details: vec![],
    };
    let run = || -> crate::Result<Vec<(u32, SimResult)>> {
        let selected = select_variation(plan)?;
        let plan = selected.as_ref().unwrap_or(plan);
        let p: Plan = serde_json::from_value(plan.clone()).map_err(|e| crate::Error::Invalid(e.to_string()))?;
        let sim = Simulator::new(&p)?;
        let data = batch::load_series(&p, base_dir, library)?;
        let paths = batch::path_source(&p, &sim, data.as_ref())?;
        picks
            .iter()
            .map(|&(percent, index)| {
                if index >= paths.len() {
                    return Err(crate::Error::Invalid(format!(
                        "Pfadnummer {index} ausserhalb von 0 bis {}",
                        paths.len().saturating_sub(1)
                    )));
                }
                Ok((percent, sim.run(&paths.get(index), true)))
            })
            .collect()
    };
    match run() {
        Ok(details) => RerunOut { error: None, details },
        Err(e) => fail(e.to_string()),
    }
}

/// Jede Kombination der Variationen mit allen Renditepfaden. Monte-Carlo-Pfade sind in allen
/// Szenarien dieselben (gleicher Startwert), damit Unterschiede von den Parametern stammen.
fn run_scenarios(plan: &Value, p: &Plan, data: Option<&DataSet>, per_scenario: usize) -> crate::Result<ScenariosOut> {
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
                let q: Plan = serde_json::from_value(v).map_err(|e| crate::Error::Invalid(e.to_string()))?;
                let sim = Simulator::new(&q)?;
                let paths = batch::path_source(&q, &sim, data)?;
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
                let mut real: Vec<f64> = results.iter().map(|r| r.summary.end_total_real).collect();
                real.sort_by(f64::total_cmp);
                Ok(ScenarioResult {
                    values: values.clone(),
                    runs: s.runs,
                    success_rate: s.success_rate,
                    p5: pct(5),
                    p50: pct(50),
                    p95: pct(95),
                    p50_real: real.get(real.len() / 2).copied().unwrap_or(0.0),
                    at_retirement: at_ret.get(at_ret.len() / 2).copied(),
                    earliest_depletion: s.earliest_depletion.map(|(m, _)| m),
                    // #130: ohne reales Endvermögen je Perzentil, die Szenarien zeigen es nicht
                    percentiles: s
                        .percentiles
                        .into_iter()
                        .map(|p| batch::Percentile {
                            end_total_real: None,
                            ..p
                        })
                        .collect(),
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
                error: Some(e.to_string()),
            })
        })
        .collect();
    if crate::progress::cancelled() {
        return Err(crate::Error::Cancelled);
    }
    Ok(ScenariosOut { params, results })
}

/// Hinweis, wenn die Zeitreihe in einer anderen Währung vorliegt als die Planung.
pub fn currency_issue(ds: &DataSet, plan_currency: &str) -> Option<Issue> {
    let c = ds.currency()?;
    (!c.eq_ignore_ascii_case(plan_currency)).then(|| Issue {
        severity: Severity::Warning,
        section: "returns",
        message: format!(
            "Die Zeitreihe «{}» ist in {c}, die Planung in {plan_currency}. Renditen und Inflation gelten dann für {c}.",
            ds.meta.title.as_deref().unwrap_or(&ds.name)
        ),
        field: None,
    })
}

/// Kurzinfo zu einer Zeitreihe (für die Anzeige im Editor).
/// `series` ist ein Verweis auf eine Datei oder eine Anlagestrategie (`strategy:<JSON>`, #69).
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

/// Inhalt der Datenbibliothek; legt fehlende ausgelieferte Zeitreihen an.
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
    pub notes: Vec<String>,
}

/// Alte `config.json` umwandeln; relative Datenpfade gelten ab `base_dir`.
pub fn import_legacy(text: &str, start_year: i32, base_dir: &Path, name: &str) -> crate::Result<ImportOut> {
    let imp = crate::legacy::import(text, start_year, base_dir, name)?;
    Ok(ImportOut {
        plan: serde_json::to_value(&imp.plan).expect("serialisierbar"),
        notes: imp.notes,
    })
}

/// Projektdatei lesen; erkennt alte `config.json` nicht automatisch (dafür `import_legacy`).
pub fn read_plan(path: &Path) -> crate::Result<Value> {
    let text = std::fs::read_to_string(path).map_err(|e| crate::Error::Io(format!("{}: {e}", path.display())))?;
    let v: Value =
        serde_json::from_str(&text).map_err(|e| crate::Error::Invalid(format!("{}: {e}", path.display())))?;
    if v.get("format").is_none() && v.get("General").is_some() {
        return Err(crate::Error::Invalid(
            "Das ist eine Konfiguration des bisherigen Programms – bitte «Alte config.json importieren» verwenden."
                .into(),
        ));
    }
    // #144: Datei einer neueren Version nicht still öffnen
    crate::model::check_format(&v).map_err(|e| crate::Error::Invalid(format!("{}: {e}", path.display())))?;
    Ok(v)
}

/// Schreibt die Planung; eine alte Format-Kennung (legacy: RetirementPlanner) wird dabei durch die neue ersetzt.
pub fn write_plan(path: &Path, plan: &Value) -> crate::Result<()> {
    let mut plan = Cow::Borrowed(plan);
    if plan.get("format").and_then(Value::as_str) == Some(crate::model::LEGACY_FORMAT) {
        plan.to_mut()["format"] = Value::String(crate::model::FORMAT.into());
    }
    let text = serde_json::to_string_pretty(&plan).expect("serialisierbar");
    write_atomic(path, text.as_bytes()).map_err(|e| crate::Error::Io(format!("{}: {e}", path.display())))
}

/// #136: Schreibt zuerst in eine temporäre Datei im selben Ordner, bringt sie auf den Datenträger und ersetzt erst
/// dann die Zieldatei. Schlägt etwas fehl, bleibt die bisherige Datei unverändert und die temporäre wird entfernt.
/// Ist das Ziel ein symbolischer Link, wird die Datei ersetzt, auf die er zeigt (der Link bleibt).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let target = match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => std::fs::canonicalize(path)?,
        _ => path.to_path_buf(),
    };
    let dir = match target.parent() {
        Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
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

/// Optimierung der Entnahmestrategie (siehe [`crate::optimize`]).
pub fn optimize(
    plan: &Value,
    request: &Value,
    base_dir: &Path,
    library: Option<&Path>,
) -> crate::Result<crate::optimize::Report> {
    let req: crate::optimize::Request =
        serde_json::from_value(request.clone()).map_err(|e| crate::Error::Invalid(format!("Optimierung: {e}")))?;
    crate::optimize::run_request(plan, base_dir, library, &req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn example() -> Value {
        serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap()
    }

    // #40: Vorschau beim parametrischen Modell: erwartete geometrische Aktienrendite exp(μ − σ²/2 + λ·μJ) − 1.
    #[test]
    fn preview_basis_for_each_source() {
        let mut v = example();
        v["returns"]["source"] = json!("parametric");
        v["returns"]["parametric"] =
            json!({"paths": 10, "mu": 0.08, "sigma": 0.2, "df": 6, "jumpLambda": 0.5, "jumpMu": -0.1});
        // 0.08 − 0.02 − 0.05 = 0.01 ⇒ e^0.01 − 1 ≈ 1.005 %
        let out = preview(&v, Path::new("."), None);
        // #101: mit Anlagestrategie dazu die Mittel von Obligationen und Inflation der Reihe
        let basis = out.basis.unwrap();
        assert!(
            basis.starts_with("Vorschau mit der erwarteten Aktienrendite des Modells (1.0 % pro Jahr) und den Durchschnitten der Zeitreihe: Obligationen "),
            "{basis}"
        );
        v["returns"]["parametric"]["mu"] = json!(0.16);
        let out = preview(&v, Path::new("."), None);
        assert!(out.basis.unwrap().contains("(9.4 % pro Jahr)"));
        // Ohne Zeitreihe nur die Aktienrendite
        let mut w = v.clone();
        w["returns"]["strategy"]["stocks"] = json!("worldCustom");
        w["returns"]["strategy"]["weights"] = json!({"ch": 1.0, "usa": 1.0, "eu": 0.0, "apac": 0.0});
        let out = preview(&w, Path::new("."), None);
        assert_eq!(
            out.basis.as_deref(),
            Some("Vorschau mit der erwarteten Aktienrendite des Modells (9.4 % pro Jahr).")
        );
        // #100: mit «auto» die Vorschlagswerte des Aktienindex statt der gespeicherten
        v["returns"]["parametric"]["auto"] = json!(true);
        let out = preview(&v, Path::new("."), None);
        assert!(!out.basis.unwrap().contains("(9.4 % pro Jahr)"));

        // Fixe Renditen: keine Angabe, auch wenn noch eine Zeitreihe eingetragen ist.
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
        assert!(i.message.contains("«Reihe» ist in USD"), "{}", i.message);
    }

    // Fortschritt: Läufe der Planung plus Läufe aller Variationen; gemessene Zeit in Millisekunden.
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

    // Messung: mindestens 2000 Läufe; Umrechnung in Mikrosekunden.
    #[test]
    fn bench_run_counts_and_units() {
        assert_eq!(bench_runs(1), (2000, 2000));
        assert_eq!(bench_runs(3), (667, 2001));
        assert_eq!(bench_runs(1000), (2, 2000));
        assert_eq!(bench_runs(5000), (1, 5000));
        assert_eq!(micros_each(0.5, 1000), 500.0);
    }

    // Läufe kosten Zeit (Mikrosekunden pro Lauf > 0), bei einem und bei vielen Pfaden; die Zahl der Pfade steht nur
    // bei «rolling» fest.
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
        // Gleiche Planung, gleich lange Läufe: pro Lauf ähnlich viel Zeit wie bei einem Pfad (grosszügige Grenze)
        assert!(b.us_per_run > 0.2 * single, "{b:?}, ein Pfad {single}");
    }
}
