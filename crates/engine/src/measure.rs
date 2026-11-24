//! #214: Measurement layer. A metric is composed of building blocks instead of being a fixed field with a fixed formula,
//! and it is collected while the simulation runs (no monthly records).
//!
//! A metric is a chain of five steps (analysis `rechenkern-parametrisieren.md`, section 5):
//!
//! 1. **quantity**: a field of the monthly record or a derived quantity ([`Quantity`]);
//! 2. **nominal or real** (`real`: divided by the price index of the month, prices of the start month);
//! 3. **window**: the months that count ([`Window`]);
//! 4. **reduction per path**: one value per path, or one value per year with [`Reduce::AnnualSums`];
//! 5. **aggregation over paths** ([`Aggregate`]); with annual sums over all values of all paths (pooled, as the
//!    consumption metrics of the optimization).
//!
//! Metrics are plain JSON (no expression parser) and described by the JSON schema of [`Metric`] ([`schema`]). The
//! fixed metrics of the optimization (`optimize::evaluate`) are computed with this layer; those of
//! `batch::summarize` can be expressed with it ([`summary_metrics`]) and give the same values.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::batch::Paths;
use crate::market::MarketPath;
use crate::message::Message;
use crate::model::Plan;
use crate::resolve::Resolver;
use crate::sim::{MonthObserver, MonthRecord, Simulator, Summary};
use crate::time::{TimeRef, YearMonth};
use crate::{Error, Result};

/// A metric: quantity × nominal/real × window × reduction per path × aggregation over paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metric {
    /// Name in the result (default `#1`, `#2`, … in the order given).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub quantity: Quantity,
    /// In prices of the start month (amount divided by the price index of the month). Only for amounts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub real: bool,
    /// Months that count (default: the whole simulation).
    #[serde(default, skip_serializing_if = "Window::is_whole")]
    pub window: Window,
    pub reduce: Reduce,
    /// Condition on the monthly value for `firstWhen`, `share` and `any` (default: greater than 0, which is «yes» for
    /// `depleted` and `atFloor`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
    pub aggregate: Aggregate,
}

/// Quantity of a month: a field of the monthly record (amounts nominal, in the currency of the plan) or derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Quantity {
    /// Cash (liquid part outside stocks and bonds).
    Cash,
    Stocks,
    Bonds,
    /// Liquid wealth: cash, stocks and bonds.
    Liquid,
    /// Market value of the properties minus mortgages.
    PropertyEquity,
    PensionFund,
    Pillar3a,
    /// Total wealth: liquid wealth, property equity, pension fund and pillar 3a.
    Total,
    /// Pillar 3a contribution of the month.
    Pillar3aIn,
    /// Income including pensions.
    Income,
    /// Living costs including housing costs and pillar 3a contributions.
    Spending,
    Taxes,
    TaxIncome,
    TaxWealth,
    TaxCapital,
    TaxGain,
    AssessedIncome,
    AssessedWealth,
    /// Planned living costs from the plan (without housing costs).
    PlannedSpending,
    /// Withdrawal from wealth: spending minus income (before taxes).
    Withdrawal,
    Earned,
    PkPension,
    Ahv,
    /// Housing costs (interest, amortization, fix costs, rent).
    Housing,
    PropertyValue,
    Mortgage,
    /// Price index since the start (1 = prices of the start month).
    PriceIndex,
    StockIndex,
    BondIndex,
    /// Age of the first person in years.
    Age,
    /// Spending that is actually paid: after depletion only what the income covers.
    Consumption,
    /// Planned living costs plus housing costs.
    PlannedConsumption,
    /// 1 from the month in which wealth is depleted, otherwise 0.
    Depleted,
    /// 1 when the floor of the spending rule sets the living costs this month, otherwise 0.
    AtFloor,
    /// Number of emergency sales of properties so far.
    EmergencySales,
    /// Number of lump sums because of insufficient affordability at mortgage renewals so far.
    LumpSums,
    /// Number of emergency sales because of insufficient affordability at renewals so far.
    AffordabilitySales,
    /// Number of unaffordable renewals renewed in full because properties are never sold, so far.
    UnaffordableRenewals,
}

impl Quantity {
    /// An amount of money (can be shown in prices of the start month).
    pub fn is_amount(self) -> bool {
        !matches!(
            self,
            Quantity::PriceIndex
                | Quantity::StockIndex
                | Quantity::BondIndex
                | Quantity::Age
                | Quantity::Depleted
                | Quantity::AtFloor
                | Quantity::EmergencySales
                | Quantity::LumpSums
                | Quantity::AffordabilitySales
                | Quantity::UnaffordableRenewals
        )
    }

    /// Value of the month (nominal).
    fn of(self, r: &MonthRecord, s: &Summary) -> f64 {
        let flag = |b: bool| if b { 1.0 } else { 0.0 };
        let depleted = s.depleted.is_some_and(|d| r.month >= d);
        match self {
            Quantity::Cash => r.cash,
            Quantity::Stocks => r.stocks,
            Quantity::Bonds => r.bonds,
            Quantity::Liquid => r.liquid,
            Quantity::PropertyEquity => r.property_equity,
            Quantity::PensionFund => r.pension_fund,
            Quantity::Pillar3a => r.pillar3a,
            Quantity::Total => r.total,
            Quantity::Pillar3aIn => r.pillar3a_in,
            Quantity::Income => r.income,
            Quantity::Spending => r.spending,
            Quantity::Taxes => r.taxes,
            Quantity::TaxIncome => r.tax_income,
            Quantity::TaxWealth => r.tax_wealth,
            Quantity::TaxCapital => r.tax_capital,
            Quantity::TaxGain => r.tax_gain,
            Quantity::AssessedIncome => r.assessed_income,
            Quantity::AssessedWealth => r.assessed_wealth,
            Quantity::PlannedSpending => r.planned_spending,
            Quantity::Withdrawal => r.withdrawal,
            Quantity::Earned => r.earned,
            Quantity::PkPension => r.pk_pension,
            Quantity::Ahv => r.ahv,
            Quantity::Housing => r.housing,
            Quantity::PropertyValue => r.property_value,
            Quantity::Mortgage => r.mortgage,
            Quantity::PriceIndex => r.price_index,
            Quantity::StockIndex => r.stock_index,
            Quantity::BondIndex => r.bond_index,
            Quantity::Age => r.age,
            // After depletion only what income covers counts (as the optimization has always counted).
            Quantity::Consumption if depleted => r.spending.min(r.income).max(0.0),
            Quantity::Consumption => r.spending,
            Quantity::PlannedConsumption => r.planned_spending + r.housing,
            Quantity::Depleted => flag(depleted),
            Quantity::AtFloor => flag(r.at_floor),
            Quantity::EmergencySales => s.emergency_sales as f64,
            Quantity::LumpSums => s.lump_sums as f64,
            Quantity::AffordabilitySales => s.affordability_sales as f64,
            Quantity::UnaffordableRenewals => s.unaffordable_renewals as f64,
        }
    }
}

/// Months that count. All given limits apply together; without limits the whole simulation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Window {
    /// First month: `start`, `retirement` (last pension fund payout, as in the optimization), `end`, a milestone id
    /// or `YYYY-MM`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Last month (included), same choices as `from`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// From this age of the first person (years).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_age: Option<f64>,
    /// Up to and including this age of the first person (years).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_age: Option<f64>,
    /// Only the first `years` years from the first month of the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub years: Option<u32>,
}

impl Window {
    fn is_whole(&self) -> bool {
        *self == Window::default()
    }
}

/// Reduction of the months of a path in the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Reduce {
    /// Sum per year (blocks of 12 months from the first month of the window; an incomplete last year is left out).
    /// Gives a series per path; the aggregation takes all values of all paths together.
    AnnualSums,
    Sum,
    Mean,
    Min,
    Max,
    /// Value in the first month of the window.
    AtStart,
    /// Value in the last month of the window.
    AtEnd,
    /// Age of the first person in the first month that meets the condition (no value if never).
    FirstWhen,
    /// Share of the months that meet the condition (0 without months).
    Share,
    /// 1 if at least one month meets the condition, otherwise 0.
    Any,
    /// Fluctuation: mean absolute relative change of the annual sums against the previous year.
    Volatility,
    /// Sum over the window relative to the liquid wealth before the first month of the window (a negative sum counts
    /// as 0; 0 without liquid wealth). With `withdrawal` and one year from `retirement`: the withdrawal rate.
    ShareOfStartLiquid,
}

impl Reduce {
    /// One value per path (all reductions except [`Reduce::AnnualSums`]).
    pub fn is_scalar(self) -> bool {
        self != Reduce::AnnualSums
    }
}

/// Comparison with a value.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub op: Op,
    pub value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Op {
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
}

impl Condition {
    /// Default condition: greater than 0.
    pub const POSITIVE: Condition = Condition { op: Op::Gt, value: 0.0 };

    pub fn holds(&self, x: f64) -> bool {
        match self.op {
            Op::Gt => x > self.value,
            Op::Ge => x >= self.value,
            Op::Lt => x < self.value,
            Op::Le => x <= self.value,
        }
    }
}

/// Aggregation over the paths (with annual sums over all values of all paths).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Aggregate {
    /// Value at the quantile `q` (0 to 1; 0.5 = median), the value at rank round((n − 1) · q).
    Quantile {
        q: f64,
    },
    /// Mean (with standard error for one value per path).
    Mean,
    Min,
    Max,
    /// Share of the values that meet the condition (default: greater than 0), with standard error.
    Probability {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<Condition>,
    },
    /// Number of values that meet the condition (default: greater than 0).
    Count {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<Condition>,
    },
    /// Mean of a part of the values: the lowest or highest share (0 to 1) or those that meet a condition; exactly
    /// one of the three.
    #[serde(rename_all = "camelCase")]
    ConditionalMean {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lowest: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        highest: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<Condition>,
    },
}

/// Value of a metric over all paths.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricValue {
    pub name: String,
    /// `null` if there is no value (for example `firstWhen` without any such month).
    pub value: Option<f64>,
    /// Standard error of a probability or of a mean of one value per path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub std_error: Option<f64>,
    /// Number of values aggregated (paths, or path years with annual sums).
    pub values: usize,
    /// Index of the path the value comes from (quantile, minimum, maximum).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<usize>,
    /// The definition, for traceability.
    pub metric: Metric,
}

impl Metric {
    pub fn new(quantity: Quantity, reduce: Reduce, aggregate: Aggregate) -> Self {
        Metric {
            name: None,
            quantity,
            real: false,
            window: Window::default(),
            reduce,
            condition: None,
            aggregate,
        }
    }

    pub fn named(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn real(mut self) -> Self {
        self.real = true;
        self
    }

    pub fn window(mut self, window: Window) -> Self {
        self.window = window;
        self
    }

    pub fn when(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// The part computed per path; metrics with the same part share the collector.
    fn path_part(&self) -> (Quantity, bool, &Window, Reduce, Option<Condition>) {
        (self.quantity, self.real, &self.window, self.reduce, self.condition)
    }
}

/// JSON schema of a metric (catalog of the building blocks).
pub fn schema() -> Value {
    serde_json::to_value(schemars::schema_for!(Metric)).expect("schema is serializable")
}

/// Reads metrics from JSON values; each value is a metric or a list of metrics. Errors as a list.
pub fn parse(values: &[Value]) -> std::result::Result<Vec<Metric>, Vec<Message>> {
    let mut out = vec![];
    let mut errors = vec![];
    for v in values {
        let items = match v {
            Value::Array(a) => a.clone(),
            other => vec![other.clone()],
        };
        for item in items {
            let n = out.len() + errors.len() + 1;
            match serde_json::from_value::<Metric>(item) {
                Ok(m) => out.push(m),
                Err(e) => errors.push(msg!(
                    metric_invalid,
                    metric = format!("#{n}"),
                    problem = msg!(raw_error, detail = e.to_string())
                )),
            }
        }
    }
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors)
    }
}

/// A per-path part with its window as months (`lo..=hi`; empty if `lo > hi`).
#[derive(Clone, Debug)]
struct Part {
    quantity: Quantity,
    real: bool,
    reduce: Reduce,
    condition: Condition,
    lo: YearMonth,
    hi: YearMonth,
}

/// Metrics prepared for a simulation: windows resolved, equal per-path parts merged.
#[derive(Clone, Debug)]
pub struct Measurement {
    metrics: Vec<Metric>,
    names: Vec<String>,
    parts: Vec<Part>,
    /// Index into `parts` per metric.
    part_of: Vec<usize>,
}

impl Measurement {
    /// Checks and prepares the metrics; all errors as a list (an empty window is an error).
    pub fn new(plan: &Plan, sim: &Simulator, metrics: Vec<Metric>) -> std::result::Result<Self, Vec<Message>> {
        Self::build(plan, sim, metrics, false)
    }

    /// Like [`Measurement::new`], but an empty window is allowed (no values, or 0 for `share` and `any`); for the
    /// fixed metrics of the optimization.
    pub fn allowing_empty(plan: &Plan, sim: &Simulator, metrics: Vec<Metric>) -> Result<Self> {
        Self::build(plan, sim, metrics, true)
            .map_err(|e| Error::Invalid(e.into_iter().next().unwrap_or_else(|| msg!(plan_incomplete))))
    }

    fn build(
        plan: &Plan,
        sim: &Simulator,
        metrics: Vec<Metric>,
        allow_empty: bool,
    ) -> std::result::Result<Self, Vec<Message>> {
        let resolver = Resolver::new(plan).map_err(|e| vec![e.message()])?;
        Self::build_with(Some(&resolver), sim, metrics, allow_empty)
    }

    /// Prepares metrics whose windows refer to no milestone (for the tests of the optimization).
    #[cfg(test)]
    pub(crate) fn without_plan(sim: &Simulator, metrics: Vec<Metric>) -> Self {
        Self::build_with(None, sim, metrics, true).expect("metrics without milestones")
    }

    /// Values per metric of a recorded run: the monthly records fed in with the final summary (exact for the depletion
    /// and for the counters at the end).
    #[cfg(test)]
    pub(crate) fn replay(&self, res: &crate::sim::SimResult) -> Vec<Vec<f64>> {
        let mut c = Collectors {
            parts: &self.parts,
            states: self.parts.iter().map(State::new).collect(),
        };
        for r in &res.records {
            c.month(r, &res.summary);
        }
        let per_part: Vec<Vec<f64>> = c
            .states
            .into_iter()
            .zip(&self.parts)
            .map(|(s, p)| s.finish(p))
            .collect();
        self.part_of.iter().map(|&k| per_part[k].clone()).collect()
    }

    fn build_with(
        resolver: Option<&Resolver>,
        sim: &Simulator,
        metrics: Vec<Metric>,
        allow_empty: bool,
    ) -> std::result::Result<Self, Vec<Message>> {
        let mut errors = vec![];
        let mut parts: Vec<Part> = vec![];
        let mut keys: Vec<usize> = vec![];
        let mut part_of = vec![];
        let names: Vec<String> = metrics
            .iter()
            .enumerate()
            .map(|(i, m)| m.name.clone().unwrap_or_else(|| format!("#{}", i + 1)))
            .collect();
        for (i, m) in metrics.iter().enumerate() {
            let mut problems = vec![];
            if m.real && !m.quantity.is_amount() {
                problems.push(msg!(metric_real_not_amount, quantity = enum_name(&m.quantity)));
            }
            check_aggregate(&m.aggregate, &mut problems);
            let range = window_range(&m.window, sim, resolver, &mut problems);
            if let Some((lo, hi)) = range {
                if lo > hi && !allow_empty {
                    problems.push(msg!(metric_window_empty));
                }
            }
            match (problems.is_empty(), range) {
                (true, Some((lo, hi))) => {
                    let key = keys.iter().position(|&k| metrics[k].path_part() == m.path_part());
                    part_of.push(key.unwrap_or_else(|| {
                        keys.push(i);
                        parts.push(Part {
                            quantity: m.quantity,
                            real: m.real,
                            reduce: m.reduce,
                            condition: m.condition.unwrap_or(Condition::POSITIVE),
                            lo,
                            hi,
                        });
                        parts.len() - 1
                    }));
                }
                _ => errors.extend(
                    problems
                        .into_iter()
                        .map(|p| msg!(metric_invalid, metric = names[i], problem = p)),
                ),
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Measurement {
            metrics,
            names,
            parts,
            part_of,
        })
    }

    pub fn metrics(&self) -> &[Metric] {
        &self.metrics
    }

    /// Runs one path and returns the values per part (no monthly records).
    fn path_values(&self, sim: &Simulator, path: &MarketPath) -> Vec<Vec<f64>> {
        let mut c = Collectors {
            parts: &self.parts,
            states: self.parts.iter().map(State::new).collect(),
        };
        sim.run_observed(path, false, Some(&mut c));
        c.states
            .into_iter()
            .zip(&self.parts)
            .map(|(s, p)| s.finish(p))
            .collect()
    }

    /// Runs all paths in parallel and aggregates.
    pub fn run(&self, sim: &Simulator, paths: &Paths) -> Result<Vec<MetricValue>> {
        let per_path = crate::progress::par_map(paths.len(), |i| self.path_values(sim, &paths.get(i)));
        if crate::progress::cancelled() {
            return Err(Error::Cancelled);
        }
        Ok(self.aggregate(&per_path))
    }

    /// Aggregates the values per path (`per_path[path][part]`).
    fn aggregate(&self, per_path: &[Vec<Vec<f64>>]) -> Vec<MetricValue> {
        // Values with their path, per part, in the order of the paths.
        let pooled: Vec<Vec<(f64, usize)>> = (0..self.parts.len())
            .map(|k| {
                per_path
                    .iter()
                    .enumerate()
                    .flat_map(|(i, v)| v[k].iter().map(move |&x| (x, i)))
                    .collect()
            })
            .collect();
        self.metrics
            .iter()
            .zip(&self.names)
            .zip(&self.part_of)
            .map(|((m, name), &k)| {
                let (value, std_error, path) = aggregate(&m.aggregate, &pooled[k], self.parts[k].reduce.is_scalar());
                MetricValue {
                    name: name.clone(),
                    value,
                    std_error,
                    values: pooled[k].len(),
                    path,
                    metric: m.clone(),
                }
            })
            .collect()
    }
}

/// Name of a building block as in the JSON.
fn enum_name<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn check_aggregate(a: &Aggregate, problems: &mut Vec<Message>) {
    let share = |x: f64| x > 0.0 && x <= 1.0;
    match a {
        Aggregate::Quantile { q } if !(0.0..=1.0).contains(q) => problems.push(msg!(metric_quantile_invalid)),
        Aggregate::ConditionalMean {
            lowest,
            highest,
            condition,
        } => {
            let given =
                usize::from(lowest.is_some()) + usize::from(highest.is_some()) + usize::from(condition.is_some());
            if given != 1 || lowest.or(*highest).is_some_and(|x| !share(x)) {
                problems.push(msg!(metric_conditional_mean_invalid));
            }
        }
        _ => {}
    }
}

/// Resolves the window to months of the simulation; `None` after an error.
fn window_range(
    w: &Window,
    sim: &Simulator,
    resolver: Option<&Resolver>,
    problems: &mut Vec<Message>,
) -> Option<(YearMonth, YearMonth)> {
    let first = sim.start;
    let last = sim.start.plus(sim.months() as i32 - 1);
    let point = |p: &str, problems: &mut Vec<Message>| -> Option<YearMonth> {
        match p {
            "start" => Some(first),
            "end" => Some(last),
            "retirement" => sim.retire_at().or_else(|| {
                problems.push(msg!(optimize_no_retirement));
                None
            }),
            _ => {
                let r = TimeRef(p.to_string());
                let resolved = match resolver {
                    Some(resolver) => resolver.resolve(&r),
                    None => Err(Error::Invalid(msg!(time_ref_unknown, reference = p))),
                };
                match resolved {
                    Ok(Some(m)) if !r.is_dynamic() => Some(m),
                    Ok(_) => {
                        problems.push(msg!(metric_point_invalid, point = p));
                        None
                    }
                    Err(e) => {
                        problems.push(e.message());
                        None
                    }
                }
            }
        }
    };
    let from = w.from.as_deref().map(|p| point(p, problems));
    let to = w.to.as_deref().map(|p| point(p, problems));
    if w.years == Some(0) {
        problems.push(msg!(metric_years_invalid));
    }
    if !problems.is_empty() {
        return None;
    }
    let mut lo = from.flatten().unwrap_or(first).max(first);
    let mut hi = to.flatten().unwrap_or(last).min(last);
    // Ages: the same function as everywhere (`Simulator::age_at`), month by month.
    if let Some(a) = w.from_age {
        while lo <= hi && sim.age_at(lo) < a {
            lo = lo.plus(1);
        }
    }
    if let Some(a) = w.to_age {
        while hi >= lo && sim.age_at(hi) > a {
            hi = hi.plus(-1);
        }
    }
    if let Some(y) = w.years {
        hi = hi.min(lo.plus(12 * y as i32 - 1));
    }
    Some((lo, hi))
}

/// Collected values of one part on one path.
struct State {
    n: usize,
    sum: f64,
    min: f64,
    max: f64,
    first: Option<f64>,
    last: Option<f64>,
    first_when: Option<f64>,
    hits: usize,
    block: f64,
    block_n: u32,
    annual: Vec<f64>,
    start_liquid: Option<f64>,
}

impl State {
    fn new(p: &Part) -> Self {
        State {
            n: 0,
            // As `Iterator::sum` (starts with −0.0), so that the withdrawal rate is the same as before.
            sum: if p.reduce == Reduce::ShareOfStartLiquid {
                -0.0
            } else {
                0.0
            },
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            first: None,
            last: None,
            first_when: None,
            hits: 0,
            block: 0.0,
            block_n: 0,
            annual: vec![],
            start_liquid: None,
        }
    }

    fn month(&mut self, p: &Part, r: &MonthRecord, s: &Summary) {
        if r.month < p.lo || r.month > p.hi {
            return;
        }
        let v = p.quantity.of(r, s);
        let v = if p.real { v / r.price_index } else { v };
        self.n += 1;
        match p.reduce {
            Reduce::AnnualSums | Reduce::Volatility => {
                self.block += v;
                self.block_n += 1;
                if self.block_n == 12 {
                    self.annual.push(self.block);
                    self.block = 0.0;
                    self.block_n = 0;
                }
            }
            Reduce::Sum | Reduce::Mean => self.sum += v,
            Reduce::Min => self.min = self.min.min(v),
            Reduce::Max => self.max = self.max.max(v),
            Reduce::AtStart => {
                self.first.get_or_insert(v);
            }
            Reduce::AtEnd => self.last = Some(v),
            Reduce::FirstWhen => {
                if self.first_when.is_none() && p.condition.holds(v) {
                    self.first_when = Some(r.age);
                }
            }
            Reduce::Share | Reduce::Any => self.hits += usize::from(p.condition.holds(v)),
            Reduce::ShareOfStartLiquid => {
                self.start_liquid
                    .get_or_insert(r.liquid + r.spending - r.income + r.taxes);
                self.sum += v;
            }
        }
    }

    fn finish(self, p: &Part) -> Vec<f64> {
        let some = |b: bool, x: f64| if b { vec![x] } else { vec![] };
        match p.reduce {
            Reduce::AnnualSums => self.annual,
            Reduce::Sum => vec![self.sum],
            Reduce::Mean => some(self.n > 0, self.sum / self.n as f64),
            Reduce::Min => some(self.n > 0, self.min),
            Reduce::Max => some(self.n > 0, self.max),
            Reduce::AtStart => self.first.into_iter().collect(),
            Reduce::AtEnd => self.last.into_iter().collect(),
            Reduce::FirstWhen => self.first_when.into_iter().collect(),
            Reduce::Share => vec![if self.n > 0 {
                self.hits as f64 / self.n as f64
            } else {
                0.0
            }],
            Reduce::Any => vec![if self.hits > 0 { 1.0 } else { 0.0 }],
            Reduce::Volatility => vec![volatility(&self.annual)],
            Reduce::ShareOfStartLiquid => vec![match self.start_liquid {
                Some(start) if start > 0.0 => self.sum.max(0.0) / start,
                _ => 0.0,
            }],
        }
    }
}

/// Mean absolute relative change against the previous year (years with a previous value near 0 are left out).
fn volatility(annual: &[f64]) -> f64 {
    let changes: Vec<f64> = annual
        .windows(2)
        .filter(|w| w[0] > 1e-9)
        .map(|w| ((w[1] - w[0]) / w[0]).abs())
        .collect();
    if changes.is_empty() {
        0.0
    } else {
        changes.iter().sum::<f64>() / changes.len() as f64
    }
}

/// The collectors of all parts on one path.
struct Collectors<'p> {
    parts: &'p [Part],
    states: Vec<State>,
}

impl MonthObserver for Collectors<'_> {
    fn month(&mut self, record: &MonthRecord, summary: &Summary) {
        for (state, part) in self.states.iter_mut().zip(self.parts) {
            state.month(part, record, summary);
        }
    }
}

/// Value at rank round((n − 1) · q) of sorted values (the rule of `run` and of the optimization).
pub fn quantile_index(n: usize, q: f64) -> Option<usize> {
    n.checked_sub(1).map(|last| (last as f64 * q).round() as usize)
}

/// Aggregates values with their paths: (value, standard error, path).
fn aggregate(a: &Aggregate, values: &[(f64, usize)], scalar: bool) -> (Option<f64>, Option<f64>, Option<usize>) {
    let n = values.len();
    let sorted = || {
        let mut v = values.to_vec();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        v
    };
    let proportion = |c: &Option<Condition>| {
        values
            .iter()
            .filter(|v| c.unwrap_or(Condition::POSITIVE).holds(v.0))
            .count()
    };
    let mean = |v: &[(f64, usize)]| (!v.is_empty()).then(|| v.iter().map(|x| x.0).sum::<f64>() / v.len() as f64);
    match a {
        Aggregate::Quantile { q } => {
            let s = sorted();
            match quantile_index(n, *q) {
                Some(i) => (Some(s[i].0), None, Some(s[i].1)),
                None => (None, None, None),
            }
        }
        Aggregate::Mean => {
            let m = mean(values);
            let se = m.filter(|_| scalar && n > 1).map(|m| {
                let var = values.iter().map(|x| (x.0 - m).powi(2)).sum::<f64>() / (n - 1) as f64;
                (var / n as f64).sqrt()
            });
            (m, se, None)
        }
        Aggregate::Min => values
            .iter()
            .fold(None, |best: Option<(f64, usize)>, &x| match best {
                Some(b) if b.0 <= x.0 => Some(b),
                _ => Some(x),
            })
            .map_or((None, None, None), |b| (Some(b.0), None, Some(b.1))),
        Aggregate::Max => values
            .iter()
            .fold(None, |best: Option<(f64, usize)>, &x| match best {
                Some(b) if b.0 >= x.0 => Some(b),
                _ => Some(x),
            })
            .map_or((None, None, None), |b| (Some(b.0), None, Some(b.1))),
        Aggregate::Probability { condition } => {
            if n == 0 {
                return (None, None, None);
            }
            let p = proportion(condition) as f64 / n as f64;
            (Some(p), Some((p * (1.0 - p) / n as f64).sqrt()), None)
        }
        Aggregate::Count { condition } => (Some(proportion(condition) as f64), None, None),
        Aggregate::ConditionalMean {
            lowest,
            highest,
            condition,
        } => {
            let part: Vec<(f64, usize)> = match (lowest, highest, condition) {
                (Some(s), _, _) => {
                    let k = ((n as f64 * s).ceil() as usize).min(n);
                    sorted()[..k].to_vec()
                }
                (_, Some(s), _) => {
                    let k = ((n as f64 * s).ceil() as usize).min(n);
                    sorted()[n - k..].to_vec()
                }
                (_, _, c) => values
                    .iter()
                    .copied()
                    .filter(|v| c.unwrap_or(Condition::POSITIVE).holds(v.0))
                    .collect(),
            };
            (mean(&part), None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::SimResult;
    use serde_json::json;

    /// Made-up plan without taxes, inflation and returns: person born 1966-11, start 2026-11 (age 60), retirement
    /// 2031-11 (65), end 2061-11 (95), spending 1000 per month, wealth as cash only.
    fn calm(cash: f64) -> Plan {
        let mut v: Value = serde_json::from_str(include_str!("../assets/new.vplan")).unwrap();
        v["limits"] = json!({"pkMax": 130, "ahvMax": 130});
        v["returns"]["fixed"] = json!({"stocks": 0, "bonds": 0, "inflation": 0, "saron": 0});
        v["taxes"] = json!({"multiplier": 1, "income": [[0, 0]], "wealth": [[0, 0]], "capital": [[0, 0]]});
        v["flows"]["spending"] = json!(1000);
        v["wealth"] = json!({"stocks": 0, "bonds": 0, "cash": cash, "stockShare": 0, "reserveMonths": 0});
        serde_json::from_value(v).unwrap()
    }

    /// Values per metric on the plan's single path, plus the recorded run.
    fn measure(plan: &Plan, metrics: Vec<Metric>) -> (Vec<Vec<f64>>, SimResult) {
        let sim = Simulator::new(plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let m = Measurement::new(plan, &sim, metrics).unwrap();
        let per_part = m.path_values(&sim, &path);
        let values = m.part_of.iter().map(|&k| per_part[k].clone()).collect();
        (values, sim.run(&path, true))
    }

    fn win(from: Option<&str>, to: Option<&str>, ages: (Option<f64>, Option<f64>), years: Option<u32>) -> Window {
        Window {
            from: from.map(str::to_string),
            to: to.map(str::to_string),
            from_age: ages.0,
            to_age: ages.1,
            years,
        }
    }

    fn m(q: Quantity, r: Reduce, w: Window) -> Metric {
        Metric::new(q, r, Aggregate::Mean).window(w)
    }

    #[test]
    fn issue_214_ak1_windows() {
        use Quantity::{Age, Spending};
        use Reduce::*;
        let plan = calm(1_000_000.0);
        let none = (None, None);
        let (v, _) = measure(
            &plan,
            vec![
                m(Age, AtStart, Window::default()),
                m(Age, AtEnd, Window::default()),
                m(Age, AtStart, win(Some("retirement"), None, none, None)),
                m(Age, AtStart, win(Some("ahv"), Some("2040-06"), none, None)),
                m(Age, AtEnd, win(Some("ahv"), Some("2040-06"), none, None)),
                m(Age, Min, win(None, None, (Some(70.0), Some(75.0)), None)),
                m(Age, Max, win(None, None, (Some(70.0), Some(75.0)), None)),
                m(Spending, Sum, win(None, None, (Some(70.0), Some(75.0)), None)),
                m(Spending, AnnualSums, win(Some("retirement"), None, none, Some(2))),
                m(Age, AtEnd, win(Some("retirement"), None, none, Some(2))),
                // Points outside the simulation are cut to it; the last year is incomplete.
                m(Spending, AnnualSums, win(Some("2020-01"), Some("2028-03"), none, None)),
                m(Age, AtStart, win(Some("end"), None, none, None)),
            ],
        );
        let one = |k: usize| v[k][0];
        assert_eq!((one(0), one(1), one(2)), (60.0, 95.0, 65.0));
        assert_eq!(one(3), 65.0);
        assert!((one(4) - (2040.0 + 5.0 / 12.0 - 1966.0 - 10.0 / 12.0)).abs() < 1e-9);
        assert_eq!((one(5), one(6)), (70.0, 75.0));
        assert_eq!(one(7), 61_000.0);
        assert_eq!(v[8], vec![12_000.0, 12_000.0]);
        assert!((one(9) - (67.0 - 1.0 / 12.0)).abs() < 1e-9);
        assert_eq!(v[10], vec![12_000.0]);
        assert_eq!(one(11), 95.0);
    }

    #[test]
    fn issue_214_ak1_reductions_and_conditions() {
        use Quantity::*;
        use Reduce::*;
        // 100'000: depleted from February 2035 (age 68.25), afterwards consumption 0.
        let plan = calm(100_000.0);
        let all = Window::default();
        let from_retirement = win(Some("retirement"), None, (None, None), None);
        let below = Condition {
            op: Op::Lt,
            value: 50_000.0,
        };
        let (v, res) = measure(
            &plan,
            vec![
                m(Depleted, FirstWhen, all.clone()),
                m(Depleted, Any, all.clone()),
                m(Depleted, Share, all.clone()),
                m(Liquid, FirstWhen, all.clone()).when(below),
                m(Liquid, Share, all.clone()).when(below),
                m(Liquid, Min, all.clone()),
                m(Consumption, Volatility, from_retirement.clone()),
                m(
                    Withdrawal,
                    ShareOfStartLiquid,
                    win(Some("retirement"), None, (None, None), Some(1)),
                ),
                m(Consumption, AnnualSums, from_retirement.clone()),
                m(AtFloor, Any, all.clone()),
                m(Consumption, Mean, from_retirement.clone()),
                m(Spending, Max, from_retirement),
                m(Liquid, FirstWhen, all).when(Condition { op: Op::Gt, value: 1e9 }),
            ],
        );
        assert_eq!(v[0], vec![68.25]);
        assert_eq!(v[1], vec![1.0]);
        let months = res.records.len() as f64;
        let depleted = res
            .records
            .iter()
            .filter(|r| r.month >= res.summary.depleted.unwrap())
            .count() as f64;
        assert_eq!(v[2], vec![depleted / months]);
        let first = res.records.iter().find(|r| r.liquid < 50_000.0).unwrap();
        assert_eq!(v[3], vec![first.age]);
        let below_n = res.records.iter().filter(|r| r.liquid < 50_000.0).count() as f64;
        assert_eq!(v[4], vec![below_n / months]);
        let min = res.records.iter().map(|r| r.liquid).fold(f64::INFINITY, f64::min);
        assert_eq!(v[5], vec![min]);
        assert!((v[6][0] - 1.75 / 4.0).abs() < 1e-12);
        assert!((v[7][0] - 0.3).abs() < 1e-12);
        assert_eq!(&v[8][..5], &[12_000.0, 12_000.0, 12_000.0, 3_000.0, 0.0]);
        assert_eq!(v[9], vec![0.0]);
        let retire = Simulator::new(&plan).unwrap().retire_at().unwrap();
        let after: Vec<f64> = res
            .records
            .iter()
            .filter(|r| r.month >= retire)
            .map(|r| Quantity::Consumption.of(r, &res.summary))
            .collect();
        assert_eq!(v[10], vec![after.iter().sum::<f64>() / after.len() as f64]);
        assert_eq!(v[8].iter().sum::<f64>(), 39_000.0);
        assert_eq!(v[11], vec![1000.0]);
        assert!(v[12].is_empty(), "never");
    }

    #[test]
    fn issue_214_ak1_aggregations() {
        let values: Vec<(f64, usize)> = [5.0, 1.0, 3.0, 2.0, 4.0, 1.0].iter().copied().zip(0..).collect();
        let ag = |a: Aggregate| aggregate(&a, &values, true);
        let c = |op, value| Some(Condition { op, value });
        assert_eq!(ag(Aggregate::Quantile { q: 0.5 }), (Some(3.0), None, Some(2)));
        assert_eq!(ag(Aggregate::Quantile { q: 0.0 }), (Some(1.0), None, Some(1)));
        assert_eq!(ag(Aggregate::Quantile { q: 1.0 }), (Some(5.0), None, Some(0)));
        let (mean, se, _) = ag(Aggregate::Mean);
        assert_eq!(mean, Some(16.0 / 6.0));
        let var = values.iter().map(|v| (v.0 - 16.0 / 6.0).powi(2)).sum::<f64>() / 5.0;
        assert_eq!(se, Some((var / 6.0).sqrt()));
        assert_eq!(
            aggregate(&Aggregate::Mean, &values, false).1,
            None,
            "no standard error when pooled"
        );
        // The first path with the extreme value.
        assert_eq!(ag(Aggregate::Min), (Some(1.0), None, Some(1)));
        assert_eq!(ag(Aggregate::Max), (Some(5.0), None, Some(0)));
        let (p, se, _) = ag(Aggregate::Probability {
            condition: c(Op::Ge, 3.0),
        });
        assert_eq!(p, Some(0.5));
        assert_eq!(se, Some((0.25f64 / 6.0).sqrt()));
        assert_eq!(
            ag(Aggregate::Count {
                condition: c(Op::Le, 1.0)
            })
            .0,
            Some(2.0)
        );
        assert_eq!(ag(Aggregate::Count { condition: None }).0, Some(6.0));
        let cm = |lowest, highest, condition| {
            ag(Aggregate::ConditionalMean {
                lowest,
                highest,
                condition,
            })
            .0
        };
        assert_eq!(cm(Some(0.5), None, None), Some(4.0 / 3.0));
        assert_eq!(cm(None, Some(0.33), None), Some(4.5));
        assert_eq!(cm(None, Some(0.34), None), Some(4.0));
        assert_eq!(cm(None, None, c(Op::Gt, 2.0)), Some(4.0));
        assert_eq!(cm(None, None, c(Op::Gt, 9.0)), None);
        // Without values
        for a in [
            Aggregate::Quantile { q: 0.5 },
            Aggregate::Mean,
            Aggregate::Min,
            Aggregate::Max,
            Aggregate::Probability { condition: None },
        ] {
            assert_eq!(aggregate(&a, &[], true), (None, None, None), "{a:?}");
        }
        assert_eq!(aggregate(&Aggregate::Count { condition: None }, &[], true).0, Some(0.0));
    }

    /// #214/AK2: The metrics are collected during the run without monthly records, with the same values as from the
    /// records.
    #[test]
    fn issue_214_ak2_collected_during_the_run_without_records() {
        struct Count(usize);
        impl MonthObserver for Count {
            fn month(&mut self, _: &MonthRecord, _: &Summary) {
                self.0 += 1;
            }
        }
        let plan = calm(100_000.0);
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let mut count = Count(0);
        let res = sim.run_observed(&path, false, Some(&mut count));
        assert!(res.records.is_empty());
        assert_eq!(count.0, sim.months());
        let metrics = crate::optimize::evaluate_metrics(&crate::optimize::Objective::default());
        let m = Measurement::new(&plan, &sim, metrics).unwrap();
        let per_part = m.path_values(&sim, &path);
        let streamed: Vec<Vec<f64>> = m.part_of.iter().map(|&k| per_part[k].clone()).collect();
        assert_eq!(streamed, m.replay(&sim.run(&path, true)));
        // Equal parts share one collector (median and P10 of consumption; end wealth at five percentiles).
        assert_eq!(m.parts.len(), m.metrics().len() - 1 - 4);
    }

    /// Codes of the problems inside the messages «Metric …: problem».
    fn codes(e: &[Message]) -> Vec<&'static str> {
        e.iter()
            .map(|m| match m.param("problem") {
                Some(crate::message::Arg::Msg(p)) => p.code(),
                _ => m.code(),
            })
            .collect()
    }

    /// #214/AK1: Errors as a list, one per problem, with the name of the metric.
    #[test]
    fn issue_214_ak1_errors_as_a_list() {
        use Quantity::*;
        use Reduce::*;
        let plan = calm(100_000.0);
        let sim = Simulator::new(&plan).unwrap();
        let none = (None, None);
        let metrics = vec![
            m(Total, AtEnd, win(Some("unbekannt"), None, none, None)).named("a"),
            m(Total, AtEnd, win(None, Some("never"), none, None)),
            Metric::new(Depleted, Any, Aggregate::Mean).real(),
            Metric::new(Total, AtEnd, Aggregate::Quantile { q: 1.5 }),
            Metric::new(
                Total,
                AtEnd,
                Aggregate::ConditionalMean {
                    lowest: Some(0.1),
                    highest: Some(0.1),
                    condition: None,
                },
            ),
            Metric::new(
                Total,
                AtEnd,
                Aggregate::ConditionalMean {
                    lowest: Some(0.0),
                    highest: None,
                    condition: None,
                },
            ),
            m(Total, AtEnd, win(None, None, none, Some(0))),
            m(Total, AtEnd, win(None, None, (Some(100.0), None), None)),
            m(Total, AtEnd, win(Some("2050-01"), Some("2040-01"), none, None)),
            m(Total, AtEnd, win(Some("afterSale"), None, none, None)),
            m(
                Total,
                AtEnd,
                win(Some("retire"), Some("end"), (Some(66.0), Some(90.0)), Some(3)),
            ),
        ];
        let e = Measurement::new(&plan, &sim, metrics.clone()).unwrap_err();
        assert_eq!(
            codes(&e),
            [
                "time_ref_unknown",
                "metric_point_invalid",
                "metric_real_not_amount",
                "metric_quantile_invalid",
                "metric_conditional_mean_invalid",
                "metric_conditional_mean_invalid",
                "metric_years_invalid",
                "metric_window_empty",
                "metric_window_empty",
                "metric_point_invalid",
            ]
        );
        assert!(e.iter().all(|m| m.code() == "metric_invalid"));
        let unknown = msg!(time_ref_unknown, reference = "unbekannt");
        assert_eq!(e[0].to_string(), format!("Metric a: {unknown}"));
        assert!(
            e[1].to_string()
                .starts_with("Metric #2: “never” is not a point in time"),
            "{}",
            e[1]
        );
        assert_eq!(
            e[2].to_string(),
            "Metric #3: “real” applies only to amounts, not to “depleted”"
        );
        // An empty window is allowed for the fixed metrics of the optimization.
        let empty = vec![m(Total, AtEnd, win(None, None, (Some(100.0), None), None))];
        assert!(Measurement::allowing_empty(&plan, &sim, empty).is_ok());
        // Without retirement
        let mut v = serde_json::to_value(&plan).unwrap();
        v["pension"]["pk"]["at"] = json!("never");
        let p: Plan = serde_json::from_value(v).unwrap();
        let s = Simulator::new(&p).unwrap();
        let e = Measurement::new(&p, &s, vec![m(Total, AtEnd, win(Some("retirement"), None, none, None))]).unwrap_err();
        assert_eq!(codes(&e), ["optimize_no_retirement"]);
    }

    #[test]
    fn issue_214_ak1_parse_json_and_schema() {
        let ok = json!({"name": "Ruin bis 85", "quantity": "depleted", "window": {"toAge": 85.0}, "reduce": "any",
            "aggregate": {"kind": "probability"}});
        let list = json!([{"quantity": "consumption", "real": true, "window": {"from": "retirement", "years": 15},
            "reduce": "annualSums", "aggregate": {"kind": "quantile", "q": 0.1}},
            {"quantity": "atFloor", "reduce": "share", "condition": {"op": "gt", "value": 0},
             "aggregate": {"kind": "conditionalMean", "lowest": 0.1}}]);
        let metrics = parse(&[ok.clone(), list]).unwrap();
        assert_eq!(metrics.len(), 3);
        assert_eq!(metrics[0].window.to_age, Some(85.0));
        assert_eq!(metrics[1].window.years, Some(15));
        // Round trip: the definition in the result is the JSON given.
        assert_eq!(serde_json::to_value(&metrics[0]).unwrap(), ok);
        let e = parse(&[
            ok.clone(),
            json!({"quantity": "unbekannt", "reduce": "sum", "aggregate": {"kind": "mean"}}),
            json!([{"quantity": "total", "reduce": "sum", "aggregate": {"kind": "mean"}, "extra": 1}]),
        ])
        .unwrap_err();
        assert_eq!(e.len(), 2);
        assert!(
            e[0].to_string().starts_with("Metric #2: unknown variant `unbekannt`"),
            "{}",
            e[0]
        );
        assert!(
            e[1].to_string().starts_with("Metric #3: unknown field `extra`"),
            "{}",
            e[1]
        );
        let s = schema().to_string();
        for word in [
            "consumption",
            "atFloor",
            "annualSums",
            "shareOfStartLiquid",
            "conditionalMean",
            "toAge",
        ] {
            assert!(s.contains(word), "{word}");
        }
    }

    #[test]
    fn issue_214_ak1_quantities() {
        let plan = calm(100_000.0);
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let res = sim.run(&path, true);
        let r = &res.records[0];
        let s = Summary::default();
        let all = schema()["definitions"]["Quantity"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|x| x["enum"].as_array().cloned().unwrap_or_default())
            .collect::<Vec<_>>();
        assert!(all.len() > 30, "{all:?}");
        for name in all {
            let q: Quantity = serde_json::from_value(name.clone()).unwrap();
            let x = q.of(r, &s);
            assert!(x.is_finite(), "{name}");
            assert_eq!(enum_name(&q), name.as_str().unwrap());
        }
        assert_eq!(Quantity::Total.of(r, &s), r.total);
        assert_eq!(Quantity::PlannedConsumption.of(r, &s), r.planned_spending + r.housing);
        assert!(Quantity::Total.is_amount() && !Quantity::Age.is_amount() && !Quantity::LumpSums.is_amount());
    }
}
