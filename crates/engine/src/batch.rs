//! Generate return paths for a plan and compute many runs in parallel.

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

/// Loads the plan's series if the return source needs one; see [`returns_series`]. The parametric
/// model needs it for the suggested values and the correlation with bonds and inflation (#100, #101).
pub fn load_series(plan: &Plan, base_dir: &Path, library: Option<&Path>) -> Result<Option<DataSet>> {
    if !matches!(
        plan.returns.source,
        ReturnSource::Historical | ReturnSource::Rolling | ReturnSource::Bootstrap | ReturnSource::Parametric
    ) {
        return Ok(None);
    }
    returns_series(plan, base_dir, library)
}

/// Series of the plan, independent of the return source: from `series` if set (older plans and the
/// command line; relative paths from the directory of the project file, `library:<file>` in the data library), otherwise
/// from the investment strategy (#69). When opened in the app, `series` becomes the strategy (`upgrade`).
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

/// All return paths according to `returns.source`.
pub fn market_paths(plan: &Plan, sim: &Simulator, data: Option<&DataSet>) -> Result<Vec<MarketPath>> {
    let n = sim.months();
    let need = || data.ok_or_else(|| Error::Invalid(msg!(historical_series_missing)));
    match plan.returns.source {
        ReturnSource::Fixed => Ok(vec![market::fixed_path(&sim.fixed, sim.start, n)]),
        ReturnSource::Historical => {
            let ds = need()?;
            let hs = plan
                .returns
                .start_date
                .ok_or_else(|| Error::Invalid(msg!(historical_start_missing)))?;
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
                return Err(Error::Invalid(msg!(series_too_short, name = ds.name)));
            }
            starts
                .into_iter()
                .map(|s| market::historical_path(ds, s, sim.start, n, &plan.returns.extrapolation, &sim.fixed))
                .collect()
        }
        ReturnSource::Bootstrap | ReturnSource::Parametric => Ok(path_source(plan, sim, data)?.iter_owned().collect()),
    }
}

/// Return paths of a simulation: fixed (fixed, historical, all start dates) or, with Monte Carlo,
/// generated only while computing.
pub enum Paths<'a> {
    List(Vec<MarketPath>),
    /// #215: Paths generated once and used for several variants (see [`PathCache`], [`prepare`]).
    Shared(&'a [MarketPath]),
    Bootstrap(Bootstrap<'a>),
    Parametric(Parametric<'a>),
}

impl Paths<'_> {
    pub fn len(&self) -> usize {
        match self {
            Paths::List(v) => v.len(),
            Paths::Shared(v) => v.len(),
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
            Paths::Shared(v) => Cow::Borrowed(&v[i]),
            Paths::Bootstrap(b) => Cow::Owned(b.path(i)),
            Paths::Parametric(p) => Cow::Owned(p.path(i)),
        }
    }

    pub fn iter_owned(&self) -> impl Iterator<Item = MarketPath> + '_ {
        (0..self.len()).map(|i| self.get(i).into_owned())
    }

    /// #215: Everything a Monte Carlo path depends on; `None` for fixed paths (they are not generated per run).
    fn key(&self) -> Option<PathKey> {
        let (fixed, start, months, generator) = match self {
            Paths::List(_) | Paths::Shared(_) => return None,
            Paths::Bootstrap(b) => (
                b.fixed,
                b.start,
                b.months,
                Generator::Bootstrap {
                    data: std::ptr::from_ref(b.data) as usize,
                    params: b.params.clone(),
                },
            ),
            Paths::Parametric(p) => (
                p.fixed,
                p.start,
                p.months,
                Generator::Parametric {
                    params: p.params.clone(),
                    correlated: p.correlated.clone(),
                },
            ),
        };
        Some(PathKey {
            fixed: fixed.clone(),
            start,
            months,
            generator,
        })
    }

    /// Generates all paths in parallel, in the order of their index.
    fn generate(&self) -> Vec<MarketPath> {
        (0..self.len())
            .into_par_iter()
            .map(|i| self.get(i).into_owned())
            .collect()
    }
}

/// #215/AK5: Upper limit for return paths kept in memory to be used for several variants (512 MB). Above it, every
/// variant generates its paths again while computing (as before #215): slower, but without additional memory.
pub const PATH_CACHE_BYTES: usize = 512 * 1024 * 1024;

/// Memory of `n` paths over `months` months (four monthly series plus the lead-in).
pub fn path_bytes(n: usize, months: usize) -> usize {
    let per_path = (4 * months + market::PRE_MONTHS) * std::mem::size_of::<f64>() + std::mem::size_of::<MarketPath>();
    n.saturating_mul(per_path)
}

/// Inputs of a Monte Carlo path generator: equal keys give bit-identical paths for every index.
#[derive(Clone, PartialEq)]
struct PathKey {
    fixed: market::FixedCurves,
    start: YearMonth,
    months: usize,
    generator: Generator,
}

#[derive(Clone, PartialEq)]
enum Generator {
    /// `data`: address of the series (the same series for all variants of a computation).
    Bootstrap {
        data: usize,
        params: crate::model::BootstrapParams,
    },
    Parametric {
        params: crate::model::ParametricParams,
        correlated: Option<crate::calibrate::Correlated>,
    },
}

/// #215/AK1: Keeps the Monte Carlo paths of the last variant, so that the next variant with the same path inputs
/// (seed, number, model, fixed values, period) uses them instead of generating them again. At most `limit` bytes;
/// above it the paths are generated while computing.
pub struct PathCache {
    limit: usize,
    key: Option<PathKey>,
    paths: Vec<MarketPath>,
    #[cfg(test)]
    generated: usize,
}

impl Default for PathCache {
    fn default() -> Self {
        PathCache::with_limit(PATH_CACHE_BYTES)
    }
}

impl PathCache {
    pub fn with_limit(limit: usize) -> Self {
        PathCache {
            limit,
            key: None,
            paths: vec![],
            #[cfg(test)]
            generated: 0,
        }
    }

    /// The paths of `source`: from the cache if it holds them, otherwise generated and kept (within the limit).
    pub fn paths<'c>(&'c mut self, source: Paths<'c>) -> Paths<'c> {
        let Some(key) = source.key() else {
            return source;
        };
        if self.key.as_ref() != Some(&key) {
            // Release the old paths before generating new ones.
            self.key = None;
            self.paths = vec![];
            if path_bytes(source.len(), key.months) > self.limit {
                return source;
            }
            self.paths = source.generate();
            self.key = Some(key);
            #[cfg(test)]
            {
                self.generated += 1;
            }
        }
        Paths::Shared(&self.paths)
    }
}

/// #215/AK2: Generates Monte Carlo paths once for all evaluations of an optimization, if they fit into the remaining
/// `budget` (bytes, reduced by the paths kept); otherwise they are generated while computing.
pub fn prepare<'a>(source: Paths<'a>, budget: &mut usize) -> Paths<'a> {
    let Some(key) = source.key() else {
        return source;
    };
    let bytes = path_bytes(source.len(), key.months);
    if bytes > *budget {
        return source;
    }
    *budget -= bytes;
    Paths::List(source.generate())
}

/// Like [`market_paths`], but without generating Monte Carlo paths in advance.
pub fn path_source<'a>(plan: &Plan, sim: &'a Simulator, data: Option<&'a DataSet>) -> Result<Paths<'a>> {
    let limit = |n: usize| {
        if n == 0 || n > 1_000_000 {
            Err(Error::Invalid(msg!(paths_range)))
        } else {
            Ok(())
        }
    };
    match plan.returns.source {
        ReturnSource::Bootstrap => {
            let ds = data.ok_or_else(|| Error::Invalid(msg!(bootstrap_series_missing)))?;
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
            let (params, correlated) =
                crate::calibrate::resolve(&plan.returns.parametric.clone().unwrap_or_default(), data);
            limit(params.paths)?;
            Ok(Paths::Parametric(Parametric {
                fixed: &sim.fixed,
                start: sim.start,
                months: sim.months(),
                params,
                correlated,
            }))
        }
        _ => market_paths(plan, sim, data).map(Paths::List),
    }
}

/// Computes all paths in parallel on all processor cores (without monthly records and event lists).
pub fn run_all(sim: &Simulator, paths: &[MarketPath]) -> Vec<SimResult> {
    paths.par_iter().map(|p| summary_only(sim.run(p, false))).collect()
}

/// Like [`run_all`] for a [`Paths`] source; Monte Carlo paths are only created during the run.
pub fn run_paths(sim: &Simulator, paths: &Paths) -> Vec<SimResult> {
    crate::progress::par_map(paths.len(), |i| summary_only(sim.run(&paths.get(i), false)))
}

/// #171: A run of many keeps only summary and label (the details are computed again for single paths); the lists of
/// events with their messages are dropped right away, in parallel, instead of all at once at the end.
fn summary_only(mut r: SimResult) -> SimResult {
    r.records = vec![];
    r.events = vec![];
    r.property_events = vec![];
    r.bucket_refills = vec![];
    r.renewals = vec![];
    r
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Percentile {
    pub percent: u32,
    pub end_total: f64,
    /// End wealth of this run in today's purchasing power (with the run's own inflation; #126). Missing in the
    /// percentiles of the scenarios, which do not show it (#130).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_total_real: Option<f64>,
    pub label: crate::message::Message,
    /// Index of the run (or return path).
    pub index: usize,
    /// First month without funds in this run (`None`: wealth lasts until the end).
    pub depleted: Option<YearMonth>,
    /// Lump sums or emergency sales because of insufficient affordability at mortgage renewals in this run.
    pub lump_sums: usize,
    pub affordability_sales: usize,
    /// Emergency sales in this run (for any reason).
    pub emergency_sales: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub runs: usize,
    /// Share of runs in which wealth lasts until the end.
    pub success_rate: f64,
    pub percentiles: Vec<Percentile>,
    pub earliest_depletion: Option<(YearMonth, crate::message::Message)>,
    /// Number of runs with at least one lump sum or emergency sale because of insufficient affordability at renewals.
    pub lump_sum_runs: usize,
    pub affordability_sale_runs: usize,
    /// Number of runs with at least one emergency sale (for any reason).
    pub emergency_sale_runs: usize,
    /// Number of runs with at least one renewal that was not affordable and was renewed in full only because of
    /// «Immobilien nie verkaufen» (#199). Missing in the JSON when 0.
    #[serde(skip_serializing_if = "crate::sim::is_zero_count")]
    pub unaffordable_renewal_runs: usize,
}

pub fn summarize(results: &[SimResult]) -> BatchSummary {
    let n = results.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| results[a].summary.end_total.total_cmp(&results[b].summary.end_total));
    // 0 = worst run
    let percentiles = [0, 1, 5, 10, 25, 50, 75, 90, 95, 99]
        .into_iter()
        .filter(|_| n > 0)
        .map(|p| {
            let i = order[((p as f64 / 100.0) * (n - 1) as f64).round() as usize];
            Percentile {
                percent: p,
                end_total: results[i].summary.end_total,
                end_total_real: Some(results[i].summary.end_total_real),
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
        unaffordable_renewal_runs: results.iter().filter(|r| r.summary.unaffordable_renewals > 0).count(),
    }
}

/// Bands of total wealth per month over all runs (percentiles 1, 5, 25, 50, 75, 95, 99 and
/// the minimum), nominal and in prices of the start month (`real`).
#[derive(Clone, Debug, Serialize)]
pub struct Bands {
    pub months: Vec<YearMonth>,
    /// Worst value of the month over all runs (may be a different run from month to month).
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
            // #135: without runs (cancelled before the first block) 0, as for the minimum
            let q = |p: f64| {
                column
                    .len()
                    .checked_sub(1)
                    .map_or(0.0, |last| column[((p / 100.0) * last as f64).round() as usize] as f64)
            };
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

/// Frequency distribution: `counts[i]` values in the interval [`edges[i]`, `edges[i + 1]`).
#[derive(Clone, Debug, Serialize)]
pub struct Histogram {
    pub edges: Vec<f64>,
    pub counts: Vec<usize>,
}

impl Histogram {
    /// `bins` classes of equal width between the 1st and 99th percentile; outliers in the edge classes.
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

/// Evaluations over all runs for the charts.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchStats {
    /// Share of runs in which wealth lasts until the respective month (like `bands.months`).
    pub survival: Vec<f64>,
    pub end_total: Option<Histogram>,
    pub end_total_real: Option<Histogram>,
    pub at_retirement: Option<Histogram>,
    /// Number of runs by age (whole years) at depletion.
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

/// Like [`run_all`], additionally with the bands per month.
pub fn run_with_bands(sim: &Simulator, paths: &Paths) -> (Vec<SimResult>, Bands) {
    let runs: Vec<(SimResult, Vec<f32>, Vec<f32>)> = crate::progress::par_map(paths.len(), |i| {
        let r = sim.run(&paths.get(i), true);
        let totals = r.records.iter().map(|x| x.total as f32).collect();
        let real = r.records.iter().map(|x| (x.total / x.price_index) as f32).collect();
        (summary_only(r), totals, real)
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

    // ---- #215 ----

    fn parametric(paths: usize, seed: u64, spending_factor: f64) -> Plan {
        let mut v = crate::api::example_plan();
        v["returns"]["source"] = serde_json::json!("parametric");
        v["returns"]["parametric"] =
            serde_json::json!({"paths": paths, "seed": seed, "mu": 0.05, "sigma": 0.12, "df": 0});
        v["flows"]["spendingFactor"] = serde_json::json!(spending_factor);
        serde_json::from_value(v).unwrap()
    }

    /// Bit for bit equal paths.
    fn same(a: &Paths, b: &Paths) -> bool {
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        a.len() == b.len()
            && (0..a.len()).all(|i| {
                let (p, q) = (a.get(i), b.get(i));
                p.label == q.label
                    && [
                        (&p.stocks, &q.stocks),
                        (&p.bonds, &q.bonds),
                        (&p.inflation, &q.inflation),
                    ]
                    .into_iter()
                    .chain([(&p.saron, &q.saron), (&p.pre_stocks, &q.pre_stocks)])
                    .all(|(x, y)| bits(x) == bits(y))
            })
    }

    /// #215/AK1: A second variant with the same path inputs (only the spending differs) uses the kept paths; other
    /// inputs (seed) generate new ones. The kept paths are those generated while computing.
    #[test]
    fn issue_215_ak1_cache_generates_paths_once_per_inputs() {
        let mut cache = PathCache::default();
        let (a, b, c) = (parametric(4, 1, 1.0), parametric(4, 1, 1.2), parametric(4, 2, 1.0));
        for (plan, generated) in [(&a, 1), (&b, 1), (&a, 1), (&c, 2), (&a, 3)] {
            let sim = Simulator::new(plan).unwrap();
            let paths = cache.paths(path_source(plan, &sim, None).unwrap());
            assert!(matches!(paths, Paths::Shared(_)));
            assert!(same(&paths, &path_source(plan, &sim, None).unwrap()));
            drop(paths);
            assert_eq!(cache.generated, generated);
        }
        // Fixed paths are not generated per run and pass through unchanged.
        let mut fixed = parametric(4, 1, 1.0);
        fixed.returns.source = ReturnSource::Fixed;
        let sim = Simulator::new(&fixed).unwrap();
        assert!(matches!(
            cache.paths(path_source(&fixed, &sim, None).unwrap()),
            Paths::List(_)
        ));
        assert_eq!(cache.generated, 3);
    }

    /// #215/AK5: Above the limit nothing is kept: the paths are generated while computing, and earlier kept paths
    /// are released.
    #[test]
    fn issue_215_ak5_cache_respects_the_limit() {
        assert_eq!(PATH_CACHE_BYTES, 512 << 20);
        let plan = parametric(4, 1, 1.0);
        let sim = Simulator::new(&plan).unwrap();
        let bytes = path_bytes(4, sim.months());
        assert_eq!(
            bytes,
            4 * ((4 * sim.months() + market::PRE_MONTHS) * 8 + std::mem::size_of::<MarketPath>())
        );
        assert_eq!(path_bytes(usize::MAX, 1), usize::MAX);
        let mut cache = PathCache::with_limit(bytes);
        assert!(matches!(
            cache.paths(path_source(&plan, &sim, None).unwrap()),
            Paths::Shared(_)
        ));
        cache.limit = bytes - 1;
        let other = parametric(4, 2, 1.0);
        let other_sim = sim_of(&other);
        let paths = cache.paths(path_source(&other, &other_sim, None).unwrap());
        assert!(matches!(paths, Paths::Parametric(_)));
        drop(paths);
        assert!(cache.key.is_none() && cache.paths.is_empty());
    }

    fn sim_of(plan: &Plan) -> Simulator {
        Simulator::new(plan).unwrap()
    }

    /// #215/AK2, AK5: `prepare` keeps the paths within the budget and reduces it by their memory; above it the paths
    /// stay a generator and the budget unchanged.
    #[test]
    fn issue_215_ak2_prepare_keeps_paths_within_the_budget() {
        let plan = parametric(3, 1, 1.0);
        let sim = sim_of(&plan);
        let bytes = path_bytes(3, sim.months());
        let mut budget = bytes + 5;
        let kept = prepare(path_source(&plan, &sim, None).unwrap(), &mut budget);
        assert!(matches!(kept, Paths::List(_)));
        assert!(same(&kept, &path_source(&plan, &sim, None).unwrap()));
        assert_eq!(budget, 5);
        // Exactly at the limit the paths are still kept.
        let mut budget = bytes;
        assert!(matches!(
            prepare(path_source(&plan, &sim, None).unwrap(), &mut budget),
            Paths::List(_)
        ));
        assert_eq!(budget, 0);
        let mut budget = bytes - 1;
        assert!(matches!(
            prepare(path_source(&plan, &sim, None).unwrap(), &mut budget),
            Paths::Parametric(_)
        ));
        assert_eq!(budget, bytes - 1);
        let mut fixed = plan.clone();
        fixed.returns.source = ReturnSource::Fixed;
        let mut budget = 0;
        assert!(matches!(
            prepare(path_source(&fixed, &sim_of(&fixed), None).unwrap(), &mut budget),
            Paths::List(_)
        ));
    }

    fn result(f: impl FnOnce(&mut Summary)) -> SimResult {
        let mut summary = Summary::default();
        f(&mut summary);
        SimResult {
            label: crate::message::Message::default(),
            summary,
            events: vec![],
            renewals: vec![],
            property_events: vec![],
            bucket_refills: vec![],
            records: vec![],
            pillar3a: vec![],
            wealth_tax: vec![],
        }
    }

    // #40: Number of paths between 1 and 1'000'000 (limits included); `is_empty` follows the length.
    #[test]
    fn path_count_limits() {
        let mut v: serde_json::Value = serde_json::from_str(include_str!("../assets/new.vplan")).unwrap();
        v["returns"]["source"] = serde_json::json!("parametric");
        let source = |v: &serde_json::Value, n: usize| {
            let mut v = v.clone();
            v["returns"]["parametric"] = serde_json::json!({"paths": n, "mu": 0.07, "sigma": 0.15, "df": 6});
            let plan: Plan = serde_json::from_value(v).unwrap();
            // `Simulator::new` already rejects invalid path counts; here the limit is checked in `path_source`.
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
            result(|s| s.unaffordable_renewals = 2),
            result(|_| {}),
        ];
        let s = summarize(&runs);
        assert_eq!(
            (s.lump_sum_runs, s.affordability_sale_runs, s.emergency_sale_runs),
            (1, 2, 1)
        );
        // #199/AK2: runs with at least one renewal kept only because of «Immobilien nie verkaufen».
        assert_eq!(s.unaffordable_renewal_runs, 1);
        assert_eq!(s.success_rate, 1.0);
        assert_eq!(s.percentiles.len(), 10);
    }

    // #135/AK1: Bands with 0, 1 and 2 runs. Without runs all values are 0 (previously a panic from `len() - 1`).
    // With 2 runs [1, 3]: index = round(p × 1), so up to p25 the first, from p50 the second run (round(0.5) = 1).
    #[test]
    fn issue_135_ak1_bands_with_zero_one_and_two_runs() {
        let months = vec![YearMonth::new(2030, 1), YearMonth::new(2030, 2)];
        let b = Bands::from_columns(months.clone(), &[]);
        for v in [&b.min, &b.p1, &b.p5, &b.p25, &b.p50, &b.p75, &b.p95, &b.p99] {
            assert_eq!(v, &vec![0.0, 0.0]);
        }
        assert_eq!(b.months, months);

        let b = Bands::from_columns(months.clone(), &[vec![1.0, 2.0]]);
        for v in [&b.min, &b.p1, &b.p5, &b.p25, &b.p50, &b.p75, &b.p95, &b.p99] {
            assert_eq!(v, &vec![1.0, 2.0]);
        }

        let b = Bands::from_columns(months, &[vec![3.0, 4.0], vec![1.0, 5.0]]);
        assert_eq!(b.min, vec![1.0, 4.0]);
        assert_eq!((b.p1[0], b.p5[0], b.p25[0]), (1.0, 1.0, 1.0));
        assert_eq!((b.p50[0], b.p75[0], b.p95[0], b.p99[0]), (3.0, 3.0, 3.0, 3.0));
        assert_eq!((b.p25[1], b.p50[1]), (4.0, 5.0));
    }

    #[test]
    fn histogram_edge_cases() {
        assert!(Histogram::of(&[1.0, 2.0], 0).is_none());
        assert!(Histogram::of(&[], 3).is_none());
        assert!(Histogram::of(&[f64::NAN], 3).is_none());
        // All values equal: class width 1.
        let h = Histogram::of(&[5.0, 5.0], 2).unwrap();
        assert_eq!(h.edges, vec![5.0, 6.0, 7.0]);
        assert_eq!(h.counts, vec![2, 0]);
        let h = Histogram::of(&[0.0, 1.0, 2.0, 3.0, 4.0], 2).unwrap();
        assert_eq!(h.edges, vec![0.0, 2.0, 4.0]);
        assert_eq!(h.counts, vec![2, 3]);
    }
}
