//! Renditepfade zu einer Planung erzeugen und viele Läufe parallel rechnen.

use std::borrow::Cow;
use std::path::Path;

use rayon::prelude::*;
use serde::Serialize;

use crate::market::{self, DataSet, MarketPath};
use crate::model::{Plan, ReturnSource};
use crate::montecarlo::{Bootstrap, Parametric};
use crate::sim::{SimResult, Simulator};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Lädt die Zeitreihe der Planung, wenn die Renditequelle eine braucht; siehe [`returns_series`].
pub fn load_series(plan: &Plan, base_dir: &Path, library: Option<&Path>) -> Result<Option<DataSet>> {
    if !matches!(
        plan.returns.source,
        ReturnSource::Historical | ReturnSource::Rolling | ReturnSource::Bootstrap
    ) {
        return Ok(None);
    }
    returns_series(plan, base_dir, library)
}

/// Zeitreihe der Planung, unabhängig von der Renditequelle: aus `series`, falls gesetzt (ältere Planungen und die
/// Kommandozeile; relative Pfade ab dem Verzeichnis der Projektdatei, `library:<datei>` in der Datenbibliothek), sonst
/// aus der Anlagestrategie (FEA-53). Beim Öffnen in der App wird `series` zur Strategie (`upgrade`).
pub fn returns_series(plan: &Plan, base_dir: &Path, library: Option<&Path>) -> Result<Option<DataSet>> {
    if let Some(s) = &plan.returns.series {
        return DataSet::load_csv(&crate::library::resolve(s, base_dir, library)?).map(Some);
    }
    plan.returns
        .strategy
        .as_ref()
        .map(|s| crate::strategy::dataset(s, library))
        .transpose()
}

/// Alle Renditepfade gemäss `returns.source`.
pub fn market_paths(plan: &Plan, sim: &Simulator, data: Option<&DataSet>) -> Result<Vec<MarketPath>> {
    let n = sim.months();
    let need =
        || data.ok_or_else(|| Error::Invalid("Für historische Renditen muss eine Zeitreihe gewählt sein.".into()));
    match plan.returns.source {
        ReturnSource::Fixed => Ok(vec![market::fixed_path(&sim.fixed, sim.start, n)]),
        ReturnSource::Historical => {
            let ds = need()?;
            let hs = plan
                .returns
                .start_date
                .ok_or_else(|| Error::Invalid("Der historische Startmonat fehlt.".into()))?;
            Ok(vec![market::historical_path(
                ds,
                hs,
                sim.start,
                n,
                &plan.returns.extrapolation,
                &sim.fixed,
            )?])
        }
        ReturnSource::Rolling => {
            let ds = need()?;
            let starts = market::rolling_starts(ds, sim.start, n);
            if starts.is_empty() {
                return Err(Error::Invalid(format!(
                    "Die Zeitreihe {} ist kürzer als der Simulationszeitraum.",
                    ds.name
                )));
            }
            starts
                .into_iter()
                .map(|s| market::historical_path(ds, s, sim.start, n, &plan.returns.extrapolation, &sim.fixed))
                .collect()
        }
        ReturnSource::Bootstrap | ReturnSource::Parametric => Ok(path_source(plan, sim, data)?.iter_owned().collect()),
    }
}

/// Renditepfade einer Simulation: fest (fix, historisch, alle Startzeitpunkte) oder bei Monte Carlo
/// erst beim Rechnen erzeugt.
pub enum Paths<'a> {
    List(Vec<MarketPath>),
    Bootstrap(Bootstrap<'a>),
    Parametric(Parametric<'a>),
}

impl Paths<'_> {
    pub fn len(&self) -> usize {
        match self {
            Paths::List(v) => v.len(),
            Paths::Bootstrap(b) => b.len(),
            Paths::Parametric(p) => p.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, i: usize) -> Cow<'_, MarketPath> {
        match self {
            Paths::List(v) => Cow::Borrowed(&v[i]),
            Paths::Bootstrap(b) => Cow::Owned(b.path(i)),
            Paths::Parametric(p) => Cow::Owned(p.path(i)),
        }
    }

    pub fn iter_owned(&self) -> impl Iterator<Item = MarketPath> + '_ {
        (0..self.len()).map(|i| self.get(i).into_owned())
    }
}

/// Wie [`market_paths`], Monte-Carlo-Pfade aber ohne sie vorab zu erzeugen.
pub fn path_source<'a>(plan: &Plan, sim: &'a Simulator, data: Option<&'a DataSet>) -> Result<Paths<'a>> {
    let limit = |n: usize| {
        if n == 0 || n > 1_000_000 {
            Err(Error::Invalid(
                "Die Anzahl Pfade muss zwischen 1 und 1'000'000 liegen.".into(),
            ))
        } else {
            Ok(())
        }
    };
    match plan.returns.source {
        ReturnSource::Bootstrap => {
            let ds = data.ok_or_else(|| {
                Error::Invalid("Für Monte Carlo aus der Historie muss eine Zeitreihe gewählt sein.".into())
            })?;
            let params = plan.returns.bootstrap.clone().unwrap_or_default();
            limit(params.paths)?;
            Ok(Paths::Bootstrap(Bootstrap {
                data: ds,
                fixed: &sim.fixed,
                start: sim.start,
                months: sim.months(),
                params,
            }))
        }
        ReturnSource::Parametric => {
            let params = plan.returns.parametric.clone().unwrap_or_default();
            limit(params.paths)?;
            Ok(Paths::Parametric(Parametric {
                fixed: &sim.fixed,
                start: sim.start,
                months: sim.months(),
                params,
            }))
        }
        _ => market_paths(plan, sim, data).map(Paths::List),
    }
}

/// Rechnet alle Pfade parallel auf allen Prozessorkernen (ohne Monatsverläufe).
pub fn run_all(sim: &Simulator, paths: &[MarketPath]) -> Vec<SimResult> {
    paths.par_iter().map(|p| sim.run(p, false)).collect()
}

/// Wie [`run_all`] für eine [`Paths`]-Quelle; Monte-Carlo-Pfade entstehen dabei erst im Lauf.
pub fn run_paths(sim: &Simulator, paths: &Paths) -> Vec<SimResult> {
    crate::progress::par_map(paths.len(), |i| sim.run(&paths.get(i), false))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Percentile {
    pub percent: u32,
    pub end_total: f64,
    pub label: String,
    /// Index des Laufs (bzw. Renditepfads).
    pub index: usize,
    /// Erster Monat ohne Mittel in diesem Lauf (`None`: das Vermögen reicht bis zum Ende).
    pub depleted: Option<YearMonth>,
    /// Einmalzahlungen bzw. Notverkäufe wegen fehlender Tragbarkeit bei Hypothekenerneuerungen in diesem Lauf.
    pub lump_sums: usize,
    pub affordability_sales: usize,
    /// Notverkäufe in diesem Lauf (aus beliebigem Grund).
    pub emergency_sales: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub runs: usize,
    /// Anteil der Läufe, in denen das Vermögen bis zum Ende reicht.
    pub success_rate: f64,
    pub percentiles: Vec<Percentile>,
    pub earliest_depletion: Option<(YearMonth, String)>,
    /// Anzahl Läufe mit mindestens einer Einmalzahlung bzw. einem Notverkauf wegen fehlender Tragbarkeit bei Erneuerungen.
    pub lump_sum_runs: usize,
    pub affordability_sale_runs: usize,
    /// Anzahl Läufe mit mindestens einem Notverkauf (aus beliebigem Grund).
    pub emergency_sale_runs: usize,
}

pub fn summarize(results: &[SimResult]) -> BatchSummary {
    let n = results.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| results[a].summary.end_total.total_cmp(&results[b].summary.end_total));
    // 0 = schlechtester Lauf
    let percentiles = [0, 1, 5, 10, 25, 50, 75, 90, 95, 99]
        .into_iter()
        .filter(|_| n > 0)
        .map(|p| {
            let i = order[((p as f64 / 100.0) * (n - 1) as f64).round() as usize];
            Percentile {
                percent: p,
                end_total: results[i].summary.end_total,
                label: results[i].label.clone(),
                index: i,
                depleted: results[i].summary.depleted,
                lump_sums: results[i].summary.lump_sums,
                affordability_sales: results[i].summary.affordability_sales,
                emergency_sales: results[i].summary.emergency_sales,
            }
        })
        .collect();
    let ok = results.iter().filter(|r| r.summary.depleted.is_none()).count();
    let earliest_depletion = results
        .iter()
        .filter_map(|r| r.summary.depleted.map(|d| (d, r.label.clone())))
        .min_by_key(|(d, _)| *d);
    BatchSummary {
        runs: n,
        success_rate: if n > 0 { ok as f64 / n as f64 } else { 0.0 },
        percentiles,
        earliest_depletion,
        lump_sum_runs: results.iter().filter(|r| r.summary.lump_sums > 0).count(),
        affordability_sale_runs: results.iter().filter(|r| r.summary.affordability_sales > 0).count(),
        emergency_sale_runs: results.iter().filter(|r| r.summary.emergency_sales > 0).count(),
    }
}

/// Bandbreite des Gesamtvermögens pro Monat über alle Läufe (Perzentile 1, 5, 25, 50, 75, 95, 99 und
/// das Minimum), nominal und in Preisen des Startmonats (`real`).
#[derive(Clone, Debug, Serialize)]
pub struct Bands {
    pub months: Vec<YearMonth>,
    /// Schlechtester Wert des Monats über alle Läufe (kann von Monat zu Monat ein anderer Lauf sein).
    pub min: Vec<f64>,
    pub p1: Vec<f64>,
    pub p5: Vec<f64>,
    pub p25: Vec<f64>,
    pub p50: Vec<f64>,
    pub p75: Vec<f64>,
    pub p95: Vec<f64>,
    pub p99: Vec<f64>,
    pub real: Option<Box<Bands>>,
}

impl Bands {
    fn from_columns(months: Vec<YearMonth>, runs: &[Vec<f32>]) -> Bands {
        let n = months.len();
        let mut b = Bands {
            months,
            min: Vec::with_capacity(n),
            p1: Vec::with_capacity(n),
            p5: Vec::with_capacity(n),
            p25: Vec::with_capacity(n),
            p50: Vec::with_capacity(n),
            p75: Vec::with_capacity(n),
            p95: Vec::with_capacity(n),
            p99: Vec::with_capacity(n),
            real: None,
        };
        let mut column = Vec::with_capacity(runs.len());
        for k in 0..n {
            column.clear();
            column.extend(runs.iter().map(|t| t[k]));
            column.sort_by(f32::total_cmp);
            let q = |p: f64| column[((p / 100.0) * (column.len() - 1) as f64).round() as usize] as f64;
            b.min.push(column.first().copied().unwrap_or(0.0) as f64);
            b.p1.push(q(1.0));
            b.p5.push(q(5.0));
            b.p25.push(q(25.0));
            b.p50.push(q(50.0));
            b.p75.push(q(75.0));
            b.p95.push(q(95.0));
            b.p99.push(q(99.0));
        }
        b
    }
}

/// Häufigkeitsverteilung: `counts[i]` Werte im Intervall [`edges[i]`, `edges[i + 1]`).
#[derive(Clone, Debug, Serialize)]
pub struct Histogram {
    pub edges: Vec<f64>,
    pub counts: Vec<usize>,
}

impl Histogram {
    /// `bins` gleich breite Klassen zwischen dem 1. und 99. Perzentil; Ausreisser in den Randklassen.
    pub fn of(values: &[f64], bins: usize) -> Option<Histogram> {
        let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
        if v.is_empty() || bins == 0 {
            return None;
        }
        v.sort_by(f64::total_cmp);
        let q = |p: f64| v[((p * (v.len() - 1) as f64).round()) as usize];
        let (lo, hi) = (q(0.01), q(0.99));
        let width = if hi > lo { (hi - lo) / bins as f64 } else { 1.0 };
        let edges = (0..=bins).map(|i| lo + i as f64 * width).collect();
        let mut counts = vec![0; bins];
        for x in &v {
            let i = ((x - lo) / width).floor().clamp(0.0, (bins - 1) as f64) as usize;
            counts[i] += 1;
        }
        Some(Histogram { edges, counts })
    }
}

/// Auswertungen über alle Läufe für die Grafiken.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchStats {
    /// Anteil der Läufe, in denen das Vermögen bis zum jeweiligen Monat reicht (wie `bands.months`).
    pub survival: Vec<f64>,
    pub end_total: Option<Histogram>,
    pub end_total_real: Option<Histogram>,
    pub at_retirement: Option<Histogram>,
    /// Anzahl Läufe nach Alter (ganze Jahre) bei Erschöpfung.
    pub depletion_ages: Vec<(u32, usize)>,
}

pub fn stats(sim: &Simulator, results: &[SimResult]) -> BatchStats {
    let n = sim.months();
    let mut depleted_at = vec![0usize; n + 1];
    let mut ages = std::collections::BTreeMap::new();
    for r in results {
        if let Some(d) = r.summary.depleted {
            depleted_at[(sim.start.until(d).max(0) as usize).min(n)] += 1;
            *ages.entry(sim.age_at(d).floor().max(0.0) as u32).or_insert(0) += 1;
        }
    }
    let total = results.len().max(1) as f64;
    let mut gone = 0;
    let survival = (0..n)
        .map(|k| {
            gone += depleted_at[k];
            1.0 - gone as f64 / total
        })
        .collect();
    let col = |f: fn(&SimResult) -> Option<f64>| -> Vec<f64> { results.iter().filter_map(f).collect() };
    BatchStats {
        survival,
        end_total: Histogram::of(&col(|r| Some(r.summary.end_total)), 40),
        end_total_real: Histogram::of(&col(|r| Some(r.summary.end_total_real)), 40),
        at_retirement: Histogram::of(&col(|r| r.summary.total_at_retirement), 40),
        depletion_ages: ages.into_iter().collect(),
    }
}

/// Wie [`run_all`], zusätzlich mit der Bandbreite pro Monat.
pub fn run_with_bands(sim: &Simulator, paths: &Paths) -> (Vec<SimResult>, Bands) {
    let runs: Vec<(SimResult, Vec<f32>, Vec<f32>)> = crate::progress::par_map(paths.len(), |i| {
        let mut r = sim.run(&paths.get(i), true);
        let totals = r.records.iter().map(|x| x.total as f32).collect();
        let real = r.records.iter().map(|x| (x.total / x.price_index) as f32).collect();
        r.records = vec![];
        (r, totals, real)
    });
    let months: Vec<YearMonth> = (0..sim.months()).map(|k| sim.start.plus(k as i32)).collect();
    let (results, nominal, real): (Vec<_>, Vec<_>, Vec<_>) =
        runs.into_iter()
            .fold((vec![], vec![], vec![]), |(mut a, mut b, mut c), (r, t, x)| {
                a.push(r);
                b.push(t);
                c.push(x);
                (a, b, c)
            });
    let mut bands = Bands::from_columns(months.clone(), &nominal);
    bands.real = Some(Box::new(Bands::from_columns(months, &real)));
    (results, bands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Summary;

    fn result(f: impl FnOnce(&mut Summary)) -> SimResult {
        let mut summary = Summary::default();
        f(&mut summary);
        SimResult {
            label: String::new(),
            summary,
            events: vec![],
            renewals: vec![],
            property_events: vec![],
            bucket_refills: vec![],
            records: vec![],
            pillar3a: vec![],
        }
    }

    // FEA-29: Anzahl Pfade zwischen 1 und 1'000'000 (Grenzen eingeschlossen); `is_empty` folgt der Länge.
    #[test]
    fn path_count_limits() {
        let mut v: serde_json::Value = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
        v["returns"]["source"] = serde_json::json!("parametric");
        let source = |v: &serde_json::Value, n: usize| {
            let mut v = v.clone();
            v["returns"]["parametric"] = serde_json::json!({"paths": n, "mu": 0.07, "sigma": 0.15, "df": 6});
            let plan: Plan = serde_json::from_value(v).unwrap();
            // `Simulator::new` lehnt ungültige Pfadzahlen schon ab; hier wird die Grenze in `path_source` geprüft.
            let mut ok = plan.clone();
            ok.returns.parametric = None;
            let sim = Simulator::new(&ok).unwrap();
            path_source(&plan, &sim, None).map(|p| (p.len(), p.is_empty()))
        };
        assert!(source(&v, 0).is_err());
        assert_eq!(source(&v, 1).unwrap(), (1, false));
        assert_eq!(source(&v, 1_000_000).unwrap(), (1_000_000, false));
        assert!(source(&v, 1_000_001).is_err());
        assert!(Paths::List(vec![]).is_empty());
    }

    #[test]
    fn summary_of_no_runs_and_counters() {
        let s = summarize(&[]);
        assert_eq!((s.runs, s.success_rate, s.percentiles.len()), (0, 0.0, 0));

        let runs = vec![
            result(|s| s.lump_sums = 2),
            result(|s| s.affordability_sales = 1),
            result(|s| {
                s.emergency_sales = 3;
                s.affordability_sales = 1;
            }),
            result(|_| {}),
        ];
        let s = summarize(&runs);
        assert_eq!(
            (s.lump_sum_runs, s.affordability_sale_runs, s.emergency_sale_runs),
            (1, 2, 1)
        );
        assert_eq!(s.success_rate, 1.0);
        assert_eq!(s.percentiles.len(), 10);
    }

    #[test]
    fn histogram_edge_cases() {
        assert!(Histogram::of(&[1.0, 2.0], 0).is_none());
        assert!(Histogram::of(&[], 3).is_none());
        assert!(Histogram::of(&[f64::NAN], 3).is_none());
        // Alle Werte gleich: Klassenbreite 1.
        let h = Histogram::of(&[5.0, 5.0], 2).unwrap();
        assert_eq!(h.edges, vec![5.0, 6.0, 7.0]);
        assert_eq!(h.counts, vec![2, 0]);
        let h = Histogram::of(&[0.0, 1.0, 2.0, 3.0, 4.0], 2).unwrap();
        assert_eq!(h.edges, vec![0.0, 2.0, 4.0]);
        assert_eq!(h.counts, vec![2, 3]);
    }
}
