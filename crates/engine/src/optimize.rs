//! Optimization of the withdrawal strategy for a fixed base scenario.
//!
//! Goal: keep the probability of ruin up to an age below a limit while making real
//! consumption (median and 10 % percentile over all paths and years from retirement) as high as possible
//! and its year-to-year fluctuation as small as possible.
//!
//! Procedure: improve a strategy's parameters by random search followed by pattern search (coordinate
//! search with shrinking step size) on a set of return paths, then check the result
//! on a second, independent set of paths (different seed). All candidates see
//! the same paths, so differences come from the parameters.

use serde::Serialize;
use serde_json::{json, Value};

use crate::batch::Paths;
use crate::measure::{Measurement, Metric, Window};
use crate::message::{Code, Message};
use crate::model::{Goal, Plan, PropertyMode};
use crate::sim::Simulator;
use crate::time::YearMonth;
use crate::{Error, Result};

/// Targets.
#[derive(Clone, Debug)]
pub struct Objective {
    /// Highest allowed probability of ruin (0.05 = 5 %).
    pub max_ruin: f64,
    pub goal: Goal,
    /// Goal «Kein Notverkauf von Immobilien»: minimum share of the planned spending the median consumption must reach.
    pub min_consumption: f64,
    /// Ruin counts only up to this age of the main person (`None` = until the end of the simulation).
    pub ruin_age: Option<f64>,
    /// Weight of the median in the consumption goal; the rest goes to the 10 % percentile.
    pub weight_median: f64,
    /// Penalty for fluctuation: goal × (1 − `vol_weight` × fluctuation).
    pub vol_weight: f64,
    /// Highest allowed fluctuation (mean absolute annual change, 0.1 = 10 %).
    pub max_vol: Option<f64>,
}

impl Default for Objective {
    fn default() -> Self {
        Objective {
            max_ruin: 0.05,
            goal: Goal::Consumption,
            min_consumption: 0.9,
            ruin_age: None,
            weight_median: 0.5,
            vol_weight: 1.0,
            max_vol: None,
        }
    }
}

/// Metrics of a strategy over all paths. Consumption in prices of the start month per year.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub runs: usize,
    /// Share of paths in which wealth is depleted by the target age.
    pub ruin: f64,
    /// Median of annual consumption (all paths and years from retirement).
    pub median: f64,
    /// 10 % percentile of annual consumption.
    pub p10: f64,
    /// Mean absolute change of annual consumption against the previous year, averaged over the paths.
    pub volatility: f64,
    /// Withdrawal rate in the first year of retirement: withdrawal from wealth (living costs and housing costs
    /// minus income, before taxes) relative to liquid wealth at the start; median over the paths.
    pub draw_rate: f64,
    /// Share of paths with at least one unplanned property sale (emergency sale).
    pub emergency_share: f64,
    /// Median of the planned spending per year from retirement (including housing costs, prices of the start month).
    pub planned: f64,
    /// #197: Nominal end wealth at the percentiles [`END_PERCENTS`], chosen as in `run` (`batch::summarize`).
    pub end_total: [f64; 5],
    /// #205: Share of the months from retirement in which the floor sets the living costs, averaged over the paths.
    /// Missing for strategies without a floor (Standard, 3 buckets without guardrails, floor 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor_share: Option<f64>,
}

/// #205: From this share of months at the floor the result shows the hint that the floor mostly sets the spending.
pub const FLOOR_HINT_SHARE: f64 = 0.5;

/// #197: Percentiles of the end wealth in the results (same as the result of `run` shows).
pub const END_PERCENTS: [u32; 5] = [1, 5, 50, 95, 99];

impl Metrics {
    /// Objective value (higher is better), without constraints. Goal 1: consumption per year. Goal 2: fewer unplanned
    /// sales before a lower probability of ruin (the share weighs so much that ruin only counts on a tie).
    pub fn score(&self, o: &Objective) -> f64 {
        if o.goal == Goal::KeepProperty {
            return -(self.emergency_share * 1e6 + self.ruin);
        }
        let w = o.weight_median.clamp(0.0, 1.0);
        (w * self.median + (1.0 - w) * self.p10) * (1.0 - o.vol_weight * self.volatility).max(0.0)
    }

    /// Violation of the constraints (0 = met).
    pub fn violation(&self, o: &Objective) -> f64 {
        let ruin = (self.ruin - o.max_ruin).max(0.0);
        if o.goal == Goal::KeepProperty {
            let short = if self.planned > 0.0 {
                (o.min_consumption - self.median / self.planned).max(0.0)
            } else {
                0.0
            };
            return ruin + short;
        }
        ruin + o.max_vol.map_or(0.0, |v| (self.volatility - v).max(0.0))
    }

    pub fn feasible(&self, o: &Objective) -> bool {
        self.violation(o) <= 1e-12
    }

    /// Comparison value: feasible candidates by objective, infeasible ones below them by violation.
    /// With the same objective (e.g. fixed spending) the smaller probability of ruin wins.
    pub(crate) fn fitness(&self, o: &Objective) -> f64 {
        let v = self.violation(o);
        if v > 1e-12 {
            -1e9 * (1.0 + v)
        } else if o.goal == Goal::KeepProperty {
            self.score(o)
        } else {
            self.score(o) * (1.0 - 0.05 * self.ruin)
        }
    }
}

/// An adjustable parameter with the default search range.
#[derive(Clone, Copy, Debug)]
pub struct ParamDef {
    pub key: &'static str,
    /// #177: Name (code of the message, `engine.opt_param_*` in the text tables).
    pub label: Code,
    pub lo: f64,
    pub hi: f64,
    /// Displayed as percent.
    pub percent: bool,
    /// Display unit: `unit_percent`, `unit_years`, `unit_months` or `unit_age`.
    pub unit: Code,
    /// Outermost allowed limits for user-chosen ranges.
    pub min: f64,
    pub max: f64,
    /// If not given in the plan, the parameter stays fixed.
    pub fixed: bool,
}

const fn p(key: &'static str, label: Code, lo: f64, hi: f64, unit: Code, min: f64, max: f64) -> ParamDef {
    ParamDef {
        key,
        label,
        lo,
        hi,
        percent: matches!(unit, Code::unit_percent),
        unit,
        min,
        max,
        fixed: false,
    }
}

impl ParamDef {
    /// Name of the parameter.
    pub fn name(&self) -> Message {
        Message::new(self.label, vec![])
    }

    /// Unit of the parameter.
    pub fn unit_name(&self) -> Message {
        Message::new(self.unit, vec![])
    }

    /// Display name with unit, unless it is percent.
    pub fn shown(&self) -> Message {
        if self.percent {
            self.name()
        } else {
            msg!(opt_param_with_unit, name = self.name(), unit = self.unit_name())
        }
    }

    const fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
}

const STOCK_SHARE: ParamDef = p("stockShare", Code::opt_param_stock_share, 0.1, 1.0, PCT, 0.0, 1.0);
const PCT: Code = Code::unit_percent;

/// Ranges per strategy and parameter (key of the strategy, then of the parameter).
pub type Ranges = std::collections::BTreeMap<String, std::collections::BTreeMap<String, crate::model::ParamRange>>;

/// Strategy of the optimization: the withdrawal strategies of the plan plus «3-Topf» with guardrails (#6), which in the
/// plan is «3-Topf» with `buckets.spending = "guardrails"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptStrategy {
    Standard,
    ThreeBuckets,
    ThreeBucketsGuardrails,
    Guardrails,
    FixedRate,
    Vpw,
    Endowment,
}

impl OptStrategy {
    /// Strategy of a plan.
    pub fn of(w: &crate::model::Withdrawal) -> Self {
        use crate::model::Strategy;
        match w.strategy {
            Strategy::Standard => OptStrategy::Standard,
            Strategy::ThreeBuckets if w.buckets_with_guardrails() => OptStrategy::ThreeBucketsGuardrails,
            Strategy::ThreeBuckets => OptStrategy::ThreeBuckets,
            Strategy::Guardrails => OptStrategy::Guardrails,
            Strategy::FixedRate => OptStrategy::FixedRate,
            Strategy::Vpw => OptStrategy::Vpw,
            Strategy::Endowment => OptStrategy::Endowment,
        }
    }
}

/// #177: Name of the strategy (`engine.opt_strategy_<key>` in the text tables).
pub fn strategy_name(s: OptStrategy) -> Message {
    match s {
        OptStrategy::Standard => msg!(opt_strategy_standard),
        OptStrategy::ThreeBuckets => msg!(opt_strategy_three_buckets),
        OptStrategy::ThreeBucketsGuardrails => msg!(opt_strategy_three_buckets_guardrails),
        OptStrategy::Guardrails => msg!(opt_strategy_guardrails),
        OptStrategy::FixedRate => msg!(opt_strategy_fixed_rate),
        OptStrategy::Vpw => msg!(opt_strategy_vpw),
        OptStrategy::Endowment => msg!(opt_strategy_endowment),
    }
}

pub fn strategy_key(s: OptStrategy) -> &'static str {
    match s {
        OptStrategy::Standard => "standard",
        OptStrategy::ThreeBuckets => "three_buckets",
        OptStrategy::ThreeBucketsGuardrails => "three_buckets_guardrails",
        OptStrategy::Guardrails => "guardrails",
        OptStrategy::FixedRate => "fixed_rate",
        OptStrategy::Vpw => "vpw",
        OptStrategy::Endowment => "endowment",
    }
}

pub fn all_strategies() -> [OptStrategy; 7] {
    [
        OptStrategy::Standard,
        OptStrategy::ThreeBuckets,
        OptStrategy::ThreeBucketsGuardrails,
        OptStrategy::Guardrails,
        OptStrategy::FixedRate,
        OptStrategy::Vpw,
        OptStrategy::Endowment,
    ]
}

/// Parameters optimized for a strategy. Floor and ceiling stay as in the plan.
pub fn param_defs(s: OptStrategy) -> Vec<ParamDef> {
    let floor = p("floor", Code::opt_param_floor, 0.5, 1.0, PCT, 0.0, 2.0).fixed();
    let ceiling = p("ceiling", Code::opt_param_ceiling, 1.0, 1.5, PCT, 0.0, 5.0).fixed();
    match s {
        OptStrategy::Standard => vec![
            STOCK_SHARE,
            p(
                "reserveMonths",
                Code::opt_param_reserve_months,
                3.0,
                36.0,
                Code::unit_months,
                0.0,
                120.0,
            ),
        ],
        OptStrategy::FixedRate => vec![
            p("rate", Code::opt_param_rate, 0.02, 0.08, PCT, 0.0, 0.5),
            STOCK_SHARE,
            floor,
            ceiling,
        ],
        OptStrategy::Vpw => vec![
            p(
                "horizonAge",
                Code::opt_param_horizon_age,
                85.0,
                105.0,
                Code::unit_age,
                60.0,
                120.0,
            ),
            p("realReturn", Code::opt_param_real_return, -0.01, 0.05, PCT, -0.1, 0.2),
            STOCK_SHARE,
            floor,
            ceiling,
        ],
        OptStrategy::Endowment => vec![
            p("eRate", Code::opt_param_e_rate, 0.02, 0.08, PCT, 0.0, 0.5),
            p("smoothing", Code::opt_param_smoothing, 0.05, 1.0, PCT, 0.0, 1.0),
            STOCK_SHARE,
            floor,
            ceiling,
        ],
        OptStrategy::Guardrails => vec![
            p("initialRate", Code::opt_param_initial_rate, 0.02, 0.08, PCT, 0.0, 0.5),
            p("upper", Code::opt_param_upper, 0.05, 0.5, PCT, 0.0, 2.0),
            p("lower", Code::opt_param_lower, 0.05, 0.5, PCT, 0.0, 2.0),
            p("adjust", Code::opt_param_adjust, 0.02, 0.3, PCT, 0.0, 1.0),
            STOCK_SHARE,
            floor,
            ceiling,
        ],
        OptStrategy::ThreeBuckets => bucket_defs().to_vec(),
        // #6: Buckets and guardrails; lower guardrail and adjustment are fixed by default.
        OptStrategy::ThreeBucketsGuardrails => {
            let mut v = bucket_defs().to_vec();
            v.extend([
                p("initialRate", Code::opt_param_initial_rate, 0.02, 0.08, PCT, 0.0, 0.5),
                p("upper", Code::opt_param_upper, 0.05, 0.5, PCT, 0.0, 2.0),
                p("lower", Code::opt_param_lower, 0.05, 0.5, PCT, 0.0, 2.0).fixed(),
                p("adjust", Code::opt_param_adjust, 0.02, 0.3, PCT, 0.0, 1.0).fixed(),
                floor,
                ceiling,
            ]);
            v
        }
    }
}

const fn bucket_defs() -> [ParamDef; 3] {
    [
        p(
            "cashYears",
            Code::opt_param_cash_years,
            0.5,
            4.0,
            Code::unit_years,
            0.0,
            20.0,
        ),
        p(
            "bondYears",
            Code::opt_param_bond_years,
            0.0,
            10.0,
            Code::unit_years,
            0.0,
            30.0,
        ),
        p(
            "refillThreshold",
            Code::opt_param_refill_threshold,
            -0.1,
            0.2,
            PCT,
            -1.0,
            1.0,
        ),
    ]
}

/// Parameters of a strategy according to the settings: `(varied, fixed)`. Varied parameters carry
/// the chosen range; fixed ones keep the value of the plan.
pub fn split_defs(s: OptStrategy, ranges: &Ranges) -> Result<(Vec<ParamDef>, Vec<ParamDef>)> {
    let (mut varied, mut fixed) = (vec![], vec![]);
    for mut d in param_defs(s) {
        let r = ranges.get(strategy_key(s)).and_then(|m| m.get(d.key));
        if r.and_then(|r| r.fixed).unwrap_or(d.fixed) {
            fixed.push(d);
            continue;
        }
        let (lo, hi) = (
            r.and_then(|r| r.lo).unwrap_or(d.lo),
            r.and_then(|r| r.hi).unwrap_or(d.hi),
        );
        let show = |v: f64| {
            if d.percent {
                crate::message::arg(&msg!(percent_value, value = (v * 1e4).round() / 100.0))
            } else {
                crate::message::arg(&v)
            }
        };
        if !lo.is_finite() || !hi.is_finite() || lo > hi {
            return Err(Error::Invalid(msg!(
                optimize_range_invalid,
                strategy = strategy_name(s),
                param = d.name(),
                lo = show(lo),
                hi = show(hi)
            )));
        }
        if lo < d.min || hi > d.max {
            return Err(Error::Invalid(msg!(
                optimize_range_limits,
                strategy = strategy_name(s),
                param = d.name(),
                min = show(d.min),
                max = show(d.max)
            )));
        }
        (d.lo, d.hi) = (lo, hi);
        varied.push(d);
    }
    Ok((varied, fixed))
}

/// Details of a parameter for the user interface.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogParam {
    pub key: &'static str,
    /// #177: Name and unit as messages (the user interface translates them by the code).
    pub label: Message,
    pub unit: Message,
    pub percent: bool,
    pub lo: f64,
    pub hi: f64,
    pub min: f64,
    pub max: f64,
    /// Without a setting of its own the parameter stays fixed.
    pub fixed: bool,
    /// Value in the plan (`None`: not set, the strategy uses its default value).
    pub current: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStrategy {
    pub key: &'static str,
    pub name: Message,
    pub params: Vec<CatalogParam>,
}

/// All strategies with their parameters, default ranges and the values of the plan.
pub fn catalog(plan: &Value) -> Vec<CatalogStrategy> {
    all_strategies()
        .into_iter()
        .map(|s| CatalogStrategy {
            key: strategy_key(s),
            name: strategy_name(s),
            params: param_defs(s)
                .into_iter()
                .map(|d| CatalogParam {
                    key: d.key,
                    label: d.name(),
                    unit: d.unit_name(),
                    percent: d.percent,
                    lo: d.lo,
                    hi: d.hi,
                    min: d.min,
                    max: d.max,
                    fixed: d.fixed,
                    current: current(plan, d.key).or_else(|| model_default(d.key)),
                })
                .collect(),
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Adjust the plan (JSON)
// ---------------------------------------------------------------------------------------------

fn last_value(series: &Value) -> Option<f64> {
    match series {
        Value::Number(n) => n.as_f64(),
        Value::Object(o) => match o.get("rows") {
            Some(Value::Array(rows)) => rows.last().and_then(|r| r["value"].as_f64()),
            _ => o.get("value").and_then(Value::as_f64),
        },
        _ => None,
    }
}

/// Sets the value from retirement; earlier entries remain. A characteristic curve (#218) cannot be cut at a point in
/// time; the value then applies from the start (#248).
fn set_from_retirement(series: &mut Value, retire: &str, v: f64) {
    let mut rows: Vec<Value> = match &*series {
        Value::Object(o) => match o.get("rows") {
            Some(Value::Array(rows)) => rows
                .iter()
                .filter(|r| match r["from"].as_str() {
                    Some(f) if f.len() == 7 && f.as_bytes()[4] == b'-' => f < retire,
                    Some("retire") => false,
                    _ => true,
                })
                .cloned()
                .collect(),
            _ => vec![json!({"from": "start", "value": last_value(series).unwrap_or(v)})],
        },
        Value::Number(n) => vec![json!({"from": "start", "value": n})],
        _ => vec![],
    };
    rows.push(json!({"from": retire, "value": v}));
    *series = json!({"mode": "timed", "rows": rows});
}

/// #248: A straight line, a table or a factor (#217) in `wealth.<key>` as steps until retirement, month by month as
/// the simulation reads them, so that the optimized value can follow from retirement on. `None` for the other forms
/// or a plan whose points in time cannot be resolved.
fn course_as_steps(plan: &Value, key: &str, retire: &str) -> Option<Value> {
    let series = &plan["wealth"][key];
    if !matches!(series["mode"].as_str(), Some("linear" | "table" | "factor")) {
        return None;
    }
    let parsed: Plan = serde_json::from_value(plan.clone()).ok()?;
    let course: crate::model::Series = serde_json::from_value(series.clone()).ok()?;
    let r = crate::resolve::Resolver::new(&parsed).ok()?;
    let curve = crate::resolve::Curve::compile(&course, &r).ok()?;
    let start = r.milestone("start").ok()?;
    let end: YearMonth = retire.parse().ok()?;
    let mut last = curve.at(start);
    let mut rows = vec![json!({"from": "start", "value": last})];
    let mut m = start.plus(1);
    while m < end {
        let x = curve.at(m);
        if x != last {
            rows.push(json!({"from": m.iso(), "value": x}));
            last = x;
        }
        m = m.plus(1);
    }
    Some(json!({"mode": "timed", "rows": rows}))
}

fn current(plan: &Value, key: &str) -> Option<f64> {
    match key {
        "stockShare" => last_value(&plan["wealth"]["stockShare"]),
        "reserveMonths" => last_value(&plan["wealth"]["reserveMonths"]),
        "rate" => plan["withdrawal"]["fixedRate"]["rate"].as_f64(),
        "floor" | "ceiling" => plan["withdrawal"][key].as_f64(),
        "initialRate" | "upper" | "lower" | "adjust" => plan["withdrawal"]["guardrails"][key].as_f64(),
        "cashYears" | "bondYears" | "refillThreshold" => plan["withdrawal"]["buckets"][key].as_f64(),
        "horizonAge" | "realReturn" => plan["withdrawal"]["vpw"][key].as_f64(),
        "eRate" => plan["withdrawal"]["endowment"]["rate"].as_f64(),
        "smoothing" => plan["withdrawal"]["endowment"]["smoothing"].as_f64(),
        _ => None,
    }
}

/// Default value of a parameter if the plan does not set anything for the strategy.
fn model_default(key: &str) -> Option<f64> {
    use crate::model::{Buckets, Endowment, FixedRate, Guardrails, Vpw};
    let (b, g, v, e) = (
        Buckets::default(),
        Guardrails::default(),
        Vpw::default(),
        Endowment::default(),
    );
    Some(match key {
        "cashYears" => b.cash_years,
        "bondYears" => b.bond_years,
        "refillThreshold" => b.refill_threshold,
        "initialRate" => g.initial_rate,
        "upper" => g.upper,
        "lower" => g.lower,
        "adjust" => g.adjust,
        "horizonAge" => v.horizon_age,
        "realReturn" => v.real_return,
        "eRate" => e.rate,
        "smoothing" => e.smoothing,
        "rate" => FixedRate::default().rate,
        _ => return None,
    })
}

fn set(plan: &mut Value, key: &str, v: f64, retire: &str) {
    match key {
        "stockShare" | "reserveMonths" => {
            if let Some(steps) = course_as_steps(plan, key, retire) {
                plan["wealth"][key] = steps;
            }
            set_from_retirement(&mut plan["wealth"][key], retire, v);
        }
        "rate" => plan["withdrawal"]["fixedRate"] = json!({ "rate": v }),
        "floor" | "ceiling" => plan["withdrawal"][key] = json!(v),
        "initialRate" | "upper" | "lower" | "adjust" => {
            if !plan["withdrawal"]["guardrails"].is_object() {
                plan["withdrawal"]["guardrails"] =
                    json!({"initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1});
            }
            plan["withdrawal"]["guardrails"][key] = json!(v);
        }
        "cashYears" | "bondYears" | "refillThreshold" => {
            if !plan["withdrawal"]["buckets"].is_object() {
                plan["withdrawal"]["buckets"] = json!({"cashYears": 2.0, "bondYears": 6.0, "refillThreshold": 0.0});
            }
            plan["withdrawal"]["buckets"][key] = json!(v);
        }
        "horizonAge" | "realReturn" => {
            if !plan["withdrawal"]["vpw"].is_object() {
                plan["withdrawal"]["vpw"] = json!({"horizonAge": 100.0, "realReturn": 0.02});
            }
            plan["withdrawal"]["vpw"][key] = json!(v);
        }
        "eRate" | "smoothing" => {
            if !plan["withdrawal"]["endowment"].is_object() {
                plan["withdrawal"]["endowment"] = json!({"rate": 0.04, "smoothing": 0.3});
            }
            let k = if key == "eRate" { "rate" } else { key };
            plan["withdrawal"]["endowment"][k] = json!(v);
        }
        _ => {}
    }
}

/// Plan with strategy and parameters.
pub fn with_params(base: &Value, strategy: OptStrategy, defs: &[ParamDef], x: &[f64], retire: YearMonth) -> Value {
    let mut v = base.clone();
    let combined = strategy == OptStrategy::ThreeBucketsGuardrails;
    v["withdrawal"]["strategy"] = json!(if combined {
        "three_buckets"
    } else {
        strategy_key(strategy)
    });
    // #6: The buckets' spending rule follows the strategy, not the plan.
    if v["withdrawal"]["buckets"].is_object() || combined {
        if !v["withdrawal"]["buckets"].is_object() {
            v["withdrawal"]["buckets"] = json!({"cashYears": 2.0, "bondYears": 6.0, "refillThreshold": 0.0});
        }
        v["withdrawal"]["buckets"]["spending"] = json!(if combined { "guardrails" } else { "planned" });
    }
    v["scenarios"]["sweeps"] = json!([]);
    let r = retire.iso();
    for (d, x) in defs.iter().zip(x) {
        set(&mut v, d.key, *x, &r);
    }
    v
}

// ---------------------------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------------------------

/// #214: The metrics of the optimization, composed of the building blocks of the measurement layer (in the order of
/// [`evaluate`]): ruin, median and 10 % percentile of real annual consumption from retirement, fluctuation, withdrawal
/// rate, emergency sales, planned spending, end wealth at [`END_PERCENTS`], months at the floor.
pub fn evaluate_metrics(o: &Objective) -> Vec<Metric> {
    use crate::measure::{Aggregate as A, Quantity as Q, Reduce as R};
    let from_retirement = Window {
        from: Some("retirement".into()),
        ..Window::default()
    };
    let consumption = || {
        Metric::new(Q::Consumption, R::AnnualSums, A::Mean)
            .real()
            .window(from_retirement.clone())
    };
    let quantile = |m: Metric, q: f64| Metric {
        aggregate: A::Quantile { q },
        ..m
    };
    let probability = A::Probability { condition: None };
    let mut out = vec![
        Metric::new(Q::Depleted, R::Any, probability).window(Window {
            to_age: o.ruin_age,
            ..Window::default()
        }),
        quantile(consumption(), 0.5),
        quantile(consumption(), 0.1),
        Metric::new(Q::Consumption, R::Volatility, A::Mean)
            .real()
            .window(from_retirement.clone()),
        Metric::new(Q::Withdrawal, R::ShareOfStartLiquid, A::Quantile { q: 0.5 }).window(Window {
            years: Some(1),
            ..from_retirement.clone()
        }),
        Metric::new(Q::EmergencySales, R::AtEnd, probability),
        Metric::new(Q::PlannedConsumption, R::AnnualSums, A::Quantile { q: 0.5 })
            .real()
            .window(from_retirement.clone()),
    ];
    out.extend(END_PERCENTS.map(|p| {
        Metric::new(
            Q::Total,
            R::AtEnd,
            A::Quantile {
                q: f64::from(p) / 100.0,
            },
        )
    }));
    out.push(Metric::new(Q::AtFloor, R::Share, A::Mean).window(from_retirement));
    out
}

/// Runs a plan with all paths and summarizes (#214: collected during the simulation, without monthly records).
pub fn evaluate(plan: &Plan, paths: &Paths, o: &Objective) -> Result<Metrics> {
    let sim = Simulator::new(plan)?;
    sim.retire_at()
        .ok_or_else(|| Error::Invalid(msg!(optimize_no_retirement)))?;
    let measurement = Measurement::allowing_empty(plan, &sim, evaluate_metrics(o))?;
    let v: Vec<f64> = measurement
        .run(&sim, paths)?
        .iter()
        .map(|m| m.value.unwrap_or(0.0))
        .collect();
    Ok(Metrics {
        runs: paths.len(),
        ruin: v[0],
        median: v[1],
        p10: v[2],
        volatility: v[3],
        draw_rate: v[4],
        emergency_share: v[5],
        planned: v[6],
        end_total: [v[7], v[8], v[9], v[10], v[11]],
        floor_share: sim.has_floor().then_some(v[12]),
    })
}

// ---------------------------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------------------------

pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next(&mut self) -> f64 {
        // splitmix64
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Result of optimizing one strategy.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    /// #177: Name of the strategy (message with code).
    pub strategy: Message,
    /// Parameters (key, display name, value, as percent).
    pub params: Vec<(String, Message, f64, bool)>,
    /// Fixed parameters with their value from the plan (same form as `params`).
    pub fixed: Vec<(String, Message, f64, bool)>,
    /// Metrics on the optimization paths.
    pub fitted: Metrics,
    /// Metrics on independent paths.
    pub validated: Metrics,
    pub feasible: bool,
    pub evaluations: usize,
    /// Strategy and investment parameters of the plan (`withdrawal`, `wealth.stockShare`, `wealth.reserveMonths`),
    /// to be taken over into the plan; `neverSell: true` if computed with «Immobilien nie verkaufen» (#197).
    pub patch: Value,
    /// #197: Computed with the switch «Immobilien nie verkaufen» on (property setting «never sell» or «both»).
    pub never_sell: bool,
}

pub struct Settings {
    /// Random starting points.
    pub samples: usize,
    pub seed: u64,
}

fn parse_plan(v: &Value) -> Result<Plan> {
    serde_json::from_value(v.clone()).map_err(|e| Error::Invalid(msg!(raw_error, detail = e.to_string())))
}

/// Paths of the optimization: `fit` for the search, `check` for the independent check.
pub struct PathSets<'a, 'p> {
    pub fit: &'a Paths<'p>,
    pub check: &'a Paths<'p>,
}

/// Targets of the search on `n` paths. Otherwise the search exhausts the ruin limit to the full and exceeds it on
/// new paths (random error of the estimate): safety margin of 1.5 standard errors.
fn fit_objective(o: &Objective, n: usize) -> Objective {
    fit_objective_with(o, n, 1.5)
}

/// #216: [`fit_objective`] with a margin of `margin` standard errors (the experiment sets it in its budget).
pub(crate) fn fit_objective_with(o: &Objective, n: usize, margin: f64) -> Objective {
    let n = n.max(1) as f64;
    let margin = margin * (o.max_ruin * (1.0 - o.max_ruin) / n).sqrt();
    Objective {
        max_ruin: (o.max_ruin - margin).max(0.0),
        ..o.clone()
    }
}

/// Starting points of the search: values of the plan (where present), middle of the range, random.
/// The value of the plan only counts if it lies within the chosen range (otherwise the middle).
fn start_points(base: &Value, defs: &[ParamDef], st: &Settings) -> Vec<Vec<f64>> {
    let mid = |d: &ParamDef| (d.lo + d.hi) / 2.0;
    let mut starts: Vec<Vec<f64>> = vec![defs
        .iter()
        .map(|d| {
            current(base, d.key)
                .filter(|v| (d.lo..=d.hi).contains(v))
                .unwrap_or(mid(d))
        })
        .collect()];
    // Without varied parameters only the plan itself is left to check.
    if defs.is_empty() {
        return starts;
    }
    starts.push(defs.iter().map(mid).collect());
    let mut rng = Rng(st.seed ^ START_SALT);
    for _ in 0..st.samples {
        starts.push(defs.iter().map(|d| d.lo + (d.hi - d.lo) * rng.next()).collect());
    }
    starts
}

/// Searches the maximum of `eval` within the limits `bounds` (lower, upper per parameter): the best starting point is
/// the initial value, followed by a pattern search (± step per axis, halve the step on failure).
/// Result: parameters, extra value of `eval` and fitness of the best point.
pub(crate) fn search<T>(
    starts: Vec<Vec<f64>>,
    bounds: &[(f64, f64)],
    mut eval: impl FnMut(&[f64]) -> Result<(T, f64)>,
) -> Result<(Vec<f64>, T, f64)> {
    let clamp = |x: &mut Vec<f64>| {
        for (v, &(lo, hi)) in x.iter_mut().zip(bounds) {
            *v = v.clamp(lo, hi);
        }
    };

    let mut best: Option<(Vec<f64>, T, f64)> = None;
    for mut x in starts {
        clamp(&mut x);
        let (m, f) = eval(&x)?;
        if best.as_ref().is_none_or(|b| f > b.2) {
            best = Some((x, m, f));
        }
    }
    let (mut bx, mut bm, mut bf) = best.expect("mindestens ein Startpunkt");

    let mut step = 0.2;
    while step > 0.01 {
        let mut improved = false;
        for (i, &(lo, hi)) in bounds.iter().enumerate() {
            for dir in [1.0, -1.0] {
                let mut x = bx.clone();
                x[i] += dir * step * (hi - lo);
                clamp(&mut x);
                if x[i] == bx[i] {
                    continue;
                }
                let (m, f) = eval(&x)?;
                if f > bf {
                    (bx, bm, bf) = (x, m, f);
                    improved = true;
                    break;
                }
            }
        }
        if !improved {
            step /= 2.0;
        }
    }
    Ok((bx, bm, bf))
}

/// Optimizes a strategy with the paths `paths`.
pub fn optimize_strategy(
    base: &Value,
    strategy: OptStrategy,
    retire: YearMonth,
    paths: PathSets<'_, '_>,
    o: &Objective,
    st: &Settings,
    ranges: &Ranges,
) -> Result<Outcome> {
    let PathSets { fit, check } = paths;
    let (defs, fixed_defs) = split_defs(strategy, ranges)?;
    let fit_obj = fit_objective(o, fit.len());
    let o_fit = &fit_obj;
    let mut evals = 0usize;
    let eval = |x: &[f64]| -> Result<(Metrics, f64)> {
        evals += 1;
        let plan = parse_plan(&with_params(base, strategy, &defs, x, retire))?;
        let m = evaluate(&plan, fit, o_fit)?;
        let f = m.fitness(o_fit);
        Ok((m, f))
    };
    let bounds: Vec<(f64, f64)> = defs.iter().map(|d| (d.lo, d.hi)).collect();
    let (bx, bm, _) = search(start_points(base, &defs, st), &bounds, eval)?;

    let best_plan = with_params(base, strategy, &defs, &bx, retire);
    let validated = evaluate(&parse_plan(&best_plan)?, check, o)?;
    Ok(Outcome {
        patch: json!({
            "withdrawal": best_plan["withdrawal"],
            "wealth": { "stockShare": best_plan["wealth"]["stockShare"], "reserveMonths": best_plan["wealth"]["reserveMonths"] },
        }),
        strategy: strategy_name(strategy),
        params: defs
            .iter()
            .zip(&bx)
            .map(|(d, v)| (d.key.to_string(), d.shown(), *v, d.percent))
            .collect(),
        fixed: fixed_defs
            .iter()
            .filter_map(|d| {
                let v = current(&best_plan, d.key).or_else(|| model_default(d.key))?;
                Some((d.key.to_string(), d.shown(), v, d.percent))
            })
            .collect(),
        fitted: bm,
        validated,
        feasible: validated.feasible(o),
        evaluations: evals,
        never_sell: false,
    })
}

/// #197: A strategy with the parameter values of the plan (default values where the plan has none), without search,
/// on the paths `paths`. All parameters are listed as fixed.
pub fn compare_strategy(
    base: &Value,
    strategy: OptStrategy,
    retire: YearMonth,
    paths: &Paths<'_>,
    o: &Objective,
) -> Result<Outcome> {
    let plan = with_params(base, strategy, &[], &[], retire);
    let m = evaluate(&parse_plan(&plan)?, paths, o)?;
    Ok(Outcome {
        patch: json!({
            "withdrawal": plan["withdrawal"],
            "wealth": { "stockShare": plan["wealth"]["stockShare"], "reserveMonths": plan["wealth"]["reserveMonths"] },
        }),
        strategy: strategy_name(strategy),
        params: vec![],
        fixed: param_defs(strategy)
            .iter()
            .filter_map(|d| {
                let v = current(&plan, d.key).or_else(|| model_default(d.key))?;
                Some((d.key.to_string(), d.shown(), v, d.percent))
            })
            .collect(),
        fitted: m,
        validated: m,
        feasible: m.feasible(o),
        evaluations: 1,
        never_sell: false,
    })
}

// ---------------------------------------------------------------------------------------------
// Request from the CLI and the user interface
// ---------------------------------------------------------------------------------------------

fn d_ruin() -> f64 {
    0.05
}
fn d_weight() -> f64 {
    0.5
}
fn d_one() -> f64 {
    1.0
}
fn d_fit() -> usize {
    5000
}
fn d_check() -> usize {
    10_000
}
fn d_samples() -> usize {
    30
}
fn d_seed() -> u64 {
    1
}

/// Settings of an optimization (JSON of the user interface; shares as numbers, 0.05 = 5 %).
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Request {
    /// Keys of the strategies (`guardrails`, `vpw` …); empty = all.
    pub strategies: Vec<String>,
    pub max_ruin: f64,
    /// Goal of the optimization (default: consumption).
    pub goal: Goal,
    /// Goal «Kein Notverkauf von Immobilien»: minimum share of the planned spending for the median consumption.
    pub min_consumption: f64,
    pub ruin_age: Option<f64>,
    pub weight_median: f64,
    pub vol_weight: f64,
    pub max_vol: Option<f64>,
    pub paths: usize,
    pub check_paths: usize,
    pub samples: usize,
    pub seed: u64,
    /// Ranges and fixed parameters per strategy (see [`split_defs`]).
    pub ranges: Ranges,
    /// #197: Compare the strategies with the parameter values of the plan, without search, on the plan's own paths
    /// (as `run`); `paths`, `check_paths`, `samples` and `ranges` do not apply.
    pub compare: bool,
    /// #197: Property setting of the runs (as planned, never sell, both). Without a property only «as planned».
    pub property: PropertyMode,
    /// Id of the computation for progress and cancellation (chosen by the user interface; otherwise the engine assigns one).
    pub job_id: Option<crate::progress::JobId>,
}

impl Default for Request {
    fn default() -> Self {
        Request {
            strategies: vec![],
            max_ruin: d_ruin(),
            goal: Goal::Consumption,
            min_consumption: 0.9,
            ruin_age: None,
            weight_median: d_weight(),
            vol_weight: d_one(),
            max_vol: None,
            paths: d_fit(),
            check_paths: d_check(),
            samples: d_samples(),
            seed: d_seed(),
            ranges: Ranges::new(),
            compare: false,
            property: PropertyMode::Planned,
            job_id: None,
        }
    }
}

impl From<&crate::model::OptimizerSettings> for Request {
    fn from(s: &crate::model::OptimizerSettings) -> Self {
        Request {
            strategies: s.strategies.clone(),
            max_ruin: s.max_ruin,
            goal: s.goal,
            min_consumption: s.min_consumption,
            ruin_age: s.ruin_age,
            weight_median: s.weight_median,
            vol_weight: s.vol_weight,
            max_vol: s.max_vol,
            paths: s.paths,
            check_paths: s.check_paths,
            samples: s.samples,
            seed: d_seed(),
            ranges: s.ranges.clone(),
            compare: false,
            property: s.property,
            job_id: None,
        }
    }
}

impl Request {
    pub fn objective(&self) -> Objective {
        Objective {
            max_ruin: self.max_ruin,
            goal: self.goal,
            min_consumption: self.min_consumption,
            ruin_age: self.ruin_age,
            weight_median: self.weight_median,
            vol_weight: self.vol_weight,
            max_vol: self.max_vol,
        }
    }

    pub fn strategy_list(&self) -> Result<Vec<OptStrategy>> {
        if self.strategies.is_empty() || self.strategies.iter().any(|k| k == "all") {
            return Ok(all_strategies().to_vec());
        }
        self.strategies
            .iter()
            .map(|k| {
                all_strategies()
                    .into_iter()
                    .find(|s| strategy_key(*s) == k.trim())
                    .ok_or_else(|| Error::Invalid(msg!(optimize_strategy_unknown, strategy = k)))
            })
            .collect::<Result<Vec<_>>>()
            .map(|mut list| {
                // #221: a strategy listed twice is optimized once
                let mut seen = vec![];
                list.retain(|s| {
                    let new = !seen.contains(s);
                    seen.push(*s);
                    new
                });
                list
            })
    }
}

/// The plan as it is, on the check paths.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub strategy: Message,
    pub metrics: Metrics,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub goal: Goal,
    /// #197: Comparison without search (see [`Request::compare`]).
    pub compare: bool,
    /// #197: Property setting actually used (always «planned» without a property).
    pub property: PropertyMode,
    pub min_consumption: f64,
    pub retire: String,
    pub fit_paths: usize,
    pub check_paths: usize,
    pub source: String,
    pub reference: Reference,
    /// Best first: feasible ones by objective, infeasible ones by smallest violation.
    pub outcomes: Vec<Outcome>,
    pub elapsed_ms: f64,
}

/// #216: Salt of the random starting points (the experiment draws them the same way).
pub(crate) const START_SALT: u64 = 0xA5A5_5A5A;

/// Offset of the seed of the independent check paths against the search paths.
pub(crate) const CHECK_SEED_OFFSET: u64 = 7919;

pub(crate) fn with_paths(base: &Value, n: usize, seed_offset: u64) -> Value {
    let mut v = base.clone();
    for key in ["bootstrap", "parametric"] {
        if v["returns"][key].is_object() {
            v["returns"][key]["paths"] = json!(n);
            let seed = v["returns"][key]["seed"].as_u64().unwrap_or(42);
            v["returns"][key]["seed"] = json!(seed + seed_offset);
        }
    }
    v
}

/// Typical number of evaluations of the pattern search (only for the progress display).
fn search_evals(s: OptStrategy) -> u64 {
    match strategy_key(s) {
        "standard" => 22,
        "fixed_rate" => 14,
        "three_buckets" => 46,
        "endowment" => 40,
        "vpw" => 74,
        "three_buckets_guardrails" => 110,
        _ => 90,
    }
}

/// #221: Most runs of one optimization (search and check, as counted by the progress). The defaults of the user interface
/// (6 strategies, 5'000 search paths, 30 starting points, 10'000 check paths) need about 1.7 million; more than
/// [`MAX_RUNS`] would compute for hours.
pub const MAX_RUNS: u64 = 20_000_000;

/// Estimated number of runs for the progress display: per strategy starting points plus pattern search
/// (empirical values, proportional to the varied parameters) on the search paths, plus the check paths for every strategy
/// and for the plan itself.
fn progress_total(req: &Request, strategies: &[OptStrategy], variants: usize) -> Result<u64> {
    let mut search: u64 = 0;
    for s in strategies {
        let (varied, _) = split_defs(*s, &req.ranges)?;
        let usual = param_defs(*s).iter().filter(|d| !d.fixed).count().max(1) as u64;
        search = search.saturating_add(if varied.is_empty() {
            1
        } else {
            (req.samples as u64).saturating_add(2 + search_evals(*s) * varied.len() as u64 / usual)
        });
    }
    let variants = variants as u64;
    // #221: saturating, so that absurd numbers of paths or starting points end in the limit [`MAX_RUNS`], not in an overflow
    Ok(variants
        .saturating_mul(search)
        .saturating_mul(req.paths as u64)
        .saturating_add((variants * strategies.len() as u64 + 1).saturating_mul(req.check_paths as u64)))
}

/// The plan has a property of its own (not only rented ones).
fn has_property(base: &Value) -> bool {
    base["realestate"]["properties"]
        .as_array()
        .is_some_and(|p| p.iter().any(|x| x["status"] != "rent"))
}

/// #197: Runs per strategy: `false` = as planned, `true` = with «Immobilien nie verkaufen».
fn variants(mode: PropertyMode) -> &'static [bool] {
    match mode {
        PropertyMode::Planned => &[false],
        PropertyMode::NeverSell => &[true],
        PropertyMode::Both => &[false, true],
    }
}

/// The plan with the switch «Immobilien nie verkaufen» on (#196).
fn never_selling(base: &Value) -> Value {
    let mut v = base.clone();
    if !v["realestate"].is_object() {
        v["realestate"] = json!({});
    }
    v["realestate"]["neverSell"] = json!(true);
    v
}

/// Ranking of the results: feasible ones by objective on the check paths, infeasible ones below by smallest violation.
fn rank(o: &Outcome, obj: &Objective) -> f64 {
    if o.feasible {
        o.validated.score(obj)
    } else {
        -1e9 * (1.0 + o.validated.violation(obj))
    }
}

/// Searches the best parameters for the chosen strategies. `base_dir`: folder of the project file,
/// `library`: data library.
pub fn run_request(
    base: &Value,
    base_dir: &std::path::Path,
    library: Option<&std::path::Path>,
    req: &Request,
) -> Result<Report> {
    run_request_within(base, base_dir, library, req, crate::batch::PATH_CACHE_BYTES)
}

/// [`run_request`] with at most `budget` bytes for paths kept in memory (#215).
fn run_request_within(
    base: &Value,
    base_dir: &std::path::Path,
    library: Option<&std::path::Path>,
    req: &Request,
    mut budget: usize,
) -> Result<Report> {
    use crate::batch;
    let t0 = std::time::Instant::now();
    let obj = req.objective();
    if !(0.0..=1.0).contains(&obj.max_ruin) {
        return Err(Error::Invalid(msg!(optimize_ruin_range)));
    }
    if req.paths == 0 || req.check_paths == 0 {
        return Err(Error::Invalid(msg!(optimize_paths_positive)));
    }
    let has_property = has_property(base);
    if req.goal == Goal::KeepProperty {
        if !has_property {
            return Err(Error::Invalid(msg!(optimize_keep_property_without_property)));
        }
        if !(0.0..=2.0).contains(&req.min_consumption) {
            return Err(Error::Invalid(msg!(optimize_min_consumption_range)));
        }
    }
    let strategies = req.strategy_list()?;
    // #197: Without a property of its own, «never sell» changes nothing: only «as planned».
    let property = if has_property {
        req.property
    } else {
        PropertyMode::Planned
    };
    let variants = variants(property);
    let total = if req.compare {
        0
    } else {
        progress_total(req, &strategies, variants.len())?
    };
    if total > MAX_RUNS {
        return Err(Error::Invalid(msg!(
            optimize_too_many_runs,
            runs = total.min(i64::MAX as u64),
            max = MAX_RUNS
        )));
    }
    let _job = crate::progress::start_as(req.job_id, total);
    let plan = parse_plan(base)?;
    let sim = Simulator::new(&plan)?;
    let retire = sim
        .retire_at()
        .ok_or_else(|| Error::Invalid(msg!(optimize_no_pk_payout)))?;
    // #221: Monte Carlo search and check paths within the memory limit of the plan's own paths (#139)
    if !req.compare
        && matches!(
            plan.returns.source,
            crate::model::ReturnSource::Bootstrap | crate::model::ReturnSource::Parametric
        )
    {
        let months = sim.months().max(1) as u64;
        let max_paths = crate::validate::MAX_PATH_MONTHS / months;
        if let Some(&n) = [req.paths, req.check_paths].iter().find(|&&n| n as u64 > max_paths) {
            return Err(Error::Invalid(msg!(
                paths_memory,
                paths = (n as u64).min(i64::MAX as u64),
                months = months,
                max = max_paths
            )));
        }
    }
    let data = batch::load_series(&plan, base_dir, library)?;
    let bases: Vec<(bool, Value)> = variants
        .iter()
        .map(|&never| (never, if never { never_selling(base) } else { base.clone() }))
        .collect();
    let mut outcomes = Vec::new();
    let (fit_paths, check_paths, reference);
    if req.compare {
        // #197: The plan's own paths, as in `run`; #215: generated once for all strategies.
        let paths = batch::prepare(batch::path_source(&plan, &sim, data.as_ref())?, &mut budget);
        crate::progress::set_total(((variants.len() * strategies.len() + 1) * paths.len()) as u64);
        reference = Reference {
            strategy: strategy_name(OptStrategy::of(&plan.withdrawal)),
            metrics: evaluate(&plan, &paths, &obj)?,
        };
        for &s in &strategies {
            for (never, b) in &bases {
                let mut o = compare_strategy(b, s, retire, &paths, &obj)?;
                o.never_sell = *never;
                outcomes.push(o);
            }
        }
        (fit_paths, check_paths) = (paths.len(), paths.len());
    } else {
        let p_fit = parse_plan(&with_paths(base, req.paths, 0))?;
        let p_check = parse_plan(&with_paths(base, req.check_paths, CHECK_SEED_OFFSET))?;
        // Create the search and check paths once, so that not every candidate recomputes them (#215: within the
        // memory limit, the search paths first).
        let fit = batch::prepare(batch::path_source(&p_fit, &sim, data.as_ref())?, &mut budget);
        let check = batch::prepare(batch::path_source(&p_check, &sim, data.as_ref())?, &mut budget);
        reference = Reference {
            strategy: strategy_name(OptStrategy::of(&plan.withdrawal)),
            metrics: evaluate(&plan, &check, &obj)?,
        };
        let settings = Settings {
            samples: req.samples,
            seed: req.seed,
        };
        for &s in &strategies {
            for (never, b) in &bases {
                let mut o = optimize_strategy(
                    b,
                    s,
                    retire,
                    PathSets {
                        fit: &fit,
                        check: &check,
                    },
                    &obj,
                    &settings,
                    &req.ranges,
                )?;
                o.never_sell = *never;
                outcomes.push(o);
            }
        }
        (fit_paths, check_paths) = (fit.len(), check.len());
    }
    for o in outcomes.iter_mut().filter(|o| o.never_sell) {
        o.patch["neverSell"] = json!(true);
    }
    outcomes.sort_by(|a, b| rank(b, &obj).total_cmp(&rank(a, &obj)));
    Ok(Report {
        goal: req.goal,
        compare: req.compare,
        property,
        min_consumption: req.min_consumption,
        retire: retire.iso(),
        fit_paths,
        check_paths,
        source: format!("{:?}", plan.returns.source).to_lowercase(),
        reference,
        outcomes,
        elapsed_ms: t0.elapsed().as_secs_f64() * 1e3,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::SimResult;

    /// #221: The defaults of the user interface (all strategies except 3 buckets with guardrails, both property
    /// settings) stay well below the limit; absurd numbers saturate instead of overflowing.
    #[test]
    fn issue_221_ak1_default_request_stays_below_the_run_limit() {
        let ui: Vec<OptStrategy> = all_strategies()
            .into_iter()
            .filter(|s| strategy_key(*s) != "three_buckets_guardrails")
            .collect();
        let total = progress_total(&Request::default(), &ui, 2).unwrap();
        assert!(total > 1_000_000 && total < MAX_RUNS / 4, "{total}");
        let absurd = Request {
            samples: usize::MAX,
            paths: usize::MAX,
            ..Request::default()
        };
        assert_eq!(progress_total(&absurd, &ui, 2).unwrap(), u64::MAX);
    }

    /// #221: A strategy listed twice is optimized once (otherwise a long list would multiply the computation).
    #[test]
    fn issue_221_ak1_strategies_are_listed_once() {
        let req = Request {
            strategies: vec!["vpw".into(), "fixed_rate".into(), "vpw".into(), " vpw ".into()],
            ..Request::default()
        };
        let keys: Vec<&str> = req.strategy_list().unwrap().into_iter().map(strategy_key).collect();
        assert_eq!(keys, ["vpw", "fixed_rate"]);
    }

    #[test]
    fn stock_share_keeps_earlier_rows() {
        let mut s = json!({"mode": "timed", "rows": [
            {"from": "2026-03", "value": 0.3}, {"from": "2028-02", "value": 0.8}, {"from": "2030-01", "value": 0.5}]});
        set_from_retirement(&mut s, "2028-02", 0.6);
        assert_eq!(s["rows"].as_array().unwrap().len(), 2);
        assert_eq!(last_value(&s), Some(0.6));
        let mut n = json!(0.4);
        set_from_retirement(&mut n, "2030-01", 0.7);
        assert_eq!(n["rows"][0]["from"], "start");
        assert_eq!(last_value(&n), Some(0.7));
    }

    #[test]
    fn quantiles_and_fitness() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(v[crate::measure::quantile_index(v.len(), 0.5).unwrap()], 3.0);
        let o = Objective::default();
        let ok = Metrics {
            runs: 1,
            ruin: 0.01,
            median: 100.0,
            p10: 80.0,
            volatility: 0.0,
            draw_rate: 0.0,
            emergency_share: 0.0,
            planned: 0.0,
            end_total: [0.0; 5],
            floor_share: None,
        };
        let bad = Metrics { ruin: 0.2, ..ok };
        assert!(ok.fitness(&o) > bad.fitness(&o));
        assert!(!bad.feasible(&o));
        assert!((ok.score(&o) - 90.0).abs() < 1e-9);
    }

    // ---- #40 ----

    /// Made-up plan without taxes, inflation and returns: start 2026-11, retirement 2031-11, end 2061-11,
    /// spending 1000 per month, wealth as cash only.
    fn calm(cash: f64) -> Value {
        let mut v: Value = serde_json::from_str(include_str!("../assets/new.vplan")).unwrap();
        // Pension fund payout also after 65 (the test moves it); #51 checks the limits elsewhere.
        v["limits"] = json!({"pkMax": 130, "ahvMax": 130});
        v["returns"]["fixed"] = json!({"stocks": 0, "bonds": 0, "inflation": 0, "saron": 0});
        v["taxes"] = json!({"multiplier": 1, "income": [[0, 0]], "wealth": [[0, 0]], "capital": [[0, 0]]});
        v["flows"]["spending"] = json!(1000);
        v["wealth"] = json!({"stocks": 0, "bonds": 0, "cash": cash, "stockShare": 0, "reserveMonths": 0});
        v
    }

    /// #214: Values of one recorded path, from the building blocks of [`evaluate_metrics`].
    struct PathStats {
        ruined: bool,
        annual: Vec<f64>,
        vol: f64,
        draw_rate: f64,
        emergency: bool,
        planned: Vec<f64>,
        floor_share: f64,
    }

    fn path_stats(sim: &Simulator, res: &SimResult, retire: YearMonth, o: &Objective) -> PathStats {
        assert_eq!(sim.retire_at(), Some(retire));
        let v = crate::measure::Measurement::without_plan(sim, evaluate_metrics(o)).replay(res);
        PathStats {
            ruined: v[0][0] > 0.0,
            annual: v[1].clone(),
            vol: v[3][0],
            draw_rate: v[4][0],
            emergency: v[5][0] > 0.0,
            planned: v[6].clone(),
            floor_share: v[12][0],
        }
    }

    fn run_calm(cash: f64) -> (Simulator, SimResult) {
        let plan = parse_plan(&calm(cash)).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let res = sim.run(&path, true);
        (sim, res)
    }

    fn metrics() -> Metrics {
        Metrics {
            runs: 1,
            ruin: 0.02,
            median: 100.0,
            p10: 80.0,
            volatility: 0.1,
            draw_rate: 0.0,
            emergency_share: 0.1,
            planned: 120.0,
            end_total: [0.0; 5],
            floor_share: None,
        }
    }

    #[test]
    fn score_violation_and_fitness() {
        let m = metrics();
        let o = Objective {
            weight_median: 0.7,
            vol_weight: 2.0,
            ..Default::default()
        };
        // (0.7 × 100 + 0.3 × 80) × (1 − 2 × 0.1) = 94 × 0.8
        assert!((m.score(&o) - 75.2).abs() < 1e-9);
        assert!((m.fitness(&o) - 75.2 * (1.0 - 0.05 * 0.02)).abs() < 1e-9);
        let keep = Objective {
            goal: Goal::KeepProperty,
            min_consumption: 0.9,
            ..Default::default()
        };
        assert_eq!(m.score(&keep), -(0.1 * 1e6 + 0.02));
        let enough = Metrics { planned: 100.0, ..m };
        assert_eq!(enough.fitness(&keep), enough.score(&keep));
        // Median 100 of planned 120 ⇒ 0.9 − 0.8333 missing
        assert!((m.violation(&keep) - (0.9 - 100.0 / 120.0)).abs() < 1e-12);
        // Ruin 0.08 at limit 0.05, fluctuation 0.15 at limit 0.1
        let bad = Metrics {
            ruin: 0.08,
            volatility: 0.15,
            ..m
        };
        let o = Objective {
            max_ruin: 0.05,
            max_vol: Some(0.1),
            ..Default::default()
        };
        assert!((bad.violation(&o) - 0.08).abs() < 1e-12);
        assert!((bad.fitness(&o) - -1e9 * 1.08).abs() < 1e-3);
        // A violation of exactly 1e-12 still counts as met.
        let tiny = Metrics {
            ruin: 1e-12,
            volatility: 0.0,
            ..m
        };
        let o = Objective {
            max_ruin: 0.0,
            ..Default::default()
        };
        assert!(tiny.feasible(&o) && tiny.fitness(&o) > 0.0);
    }

    #[test]
    fn names_and_param_display() {
        let names: Vec<_> = all_strategies().into_iter().map(strategy_name).collect();
        assert_eq!(
            names,
            [
                "Standard",
                "3 buckets",
                "3 buckets + guardrails",
                "Guardrails",
                "Fixed rate",
                "VPW",
                "Endowment"
            ]
        );
        let defs = param_defs(OptStrategy::Standard);
        assert_eq!(defs[0].shown(), "Stock allocation");
        assert_eq!(defs[1].shown(), "Cash reserve (months)");
        assert_eq!(defs[1].shown().code(), "opt_param_with_unit");
    }

    #[test]
    fn ranges_are_checked() {
        let r = |lo: f64, hi: f64| -> Ranges {
            [(
                "fixed_rate".to_string(),
                [(
                    "rate".to_string(),
                    crate::model::ParamRange {
                        fixed: None,
                        lo: Some(lo),
                        hi: Some(hi),
                    },
                )]
                .into(),
            )]
            .into()
        };
        let (varied, _) = split_defs(OptStrategy::FixedRate, &r(0.04, 0.04)).unwrap();
        assert_eq!((varied[0].lo, varied[0].hi), (0.04, 0.04));
        assert!(split_defs(OptStrategy::FixedRate, &r(0.05, 0.04)).is_err());
        assert!(split_defs(OptStrategy::FixedRate, &r(0.04, f64::INFINITY)).is_err());
        assert!(split_defs(OptStrategy::FixedRate, &r(f64::NAN, 0.04)).is_err());
    }

    #[test]
    fn retirement_rows_and_current_values() {
        // Date entries before retirement and other references remain; «retire» and later dates are dropped.
        let mut s = json!({"mode": "timed", "rows": [
            {"from": "start", "value": 0.1}, {"from": "end", "value": 0.15}, {"from": "pension", "value": 0.2},
            {"from": "2026-03", "value": 0.3}, {"from": "retire", "value": 0.4}, {"from": "2030-01", "value": 0.5}]});
        set_from_retirement(&mut s, "2028-02", 0.6);
        let from: Vec<_> = s["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["from"].as_str().unwrap())
            .collect();
        assert_eq!(from, ["start", "end", "pension", "2026-03", "2028-02"]);

        let plan = json!({
            "wealth": {"stockShare": 0.45, "reserveMonths": {"mode": "timed", "rows": [{"from": "start", "value": 7}]}},
            "withdrawal": {"fixedRate": {"rate": 0.031}, "floor": 0.6, "ceiling": 1.3,
                "guardrails": {"initialRate": 0.041, "upper": 0.21, "lower": 0.22, "adjust": 0.11},
                "buckets": {"cashYears": 2.5, "bondYears": 5.5, "refillThreshold": 0.03},
                "vpw": {"horizonAge": 98, "realReturn": 0.015}, "endowment": {"rate": 0.035, "smoothing": 0.4}}
        });
        let expected = [
            ("stockShare", 0.45),
            ("reserveMonths", 7.0),
            ("rate", 0.031),
            ("floor", 0.6),
            ("ceiling", 1.3),
            ("initialRate", 0.041),
            ("upper", 0.21),
            ("lower", 0.22),
            ("adjust", 0.11),
            ("cashYears", 2.5),
            ("bondYears", 5.5),
            ("refillThreshold", 0.03),
            ("horizonAge", 98.0),
            ("realReturn", 0.015),
            ("eRate", 0.035),
            ("smoothing", 0.4),
        ];
        for (k, x) in expected {
            assert_eq!(current(&plan, k), Some(x), "{k}");
        }
        assert_eq!(current(&plan, "unbekannt"), None);

        use crate::model::{Buckets, Endowment, FixedRate, Guardrails, Vpw};
        let (b, g, v, e) = (
            Buckets::default(),
            Guardrails::default(),
            Vpw::default(),
            Endowment::default(),
        );
        let defaults = [
            ("cashYears", b.cash_years),
            ("bondYears", b.bond_years),
            ("refillThreshold", b.refill_threshold),
            ("initialRate", g.initial_rate),
            ("upper", g.upper),
            ("lower", g.lower),
            ("adjust", g.adjust),
            ("horizonAge", v.horizon_age),
            ("realReturn", v.real_return),
            ("eRate", e.rate),
            ("smoothing", e.smoothing),
            ("rate", FixedRate::default().rate),
        ];
        for (k, x) in defaults {
            assert_eq!(model_default(k), Some(x), "{k}");
        }
        assert_eq!(model_default("stockShare"), None);
    }

    #[test]
    fn set_writes_each_parameter() {
        let retire = "2031-11";
        // Missing sections are created with default values.
        let mut p = json!({"wealth": {"stockShare": 0.5, "reserveMonths": 6}, "withdrawal": {}});
        for (k, x) in [
            ("stockShare", 0.7),
            ("reserveMonths", 9.0),
            ("rate", 0.03),
            ("floor", 0.8),
            ("ceiling", 1.2),
            ("upper", 0.3),
            ("bondYears", 4.0),
            ("realReturn", 0.01),
            ("eRate", 0.05),
        ] {
            set(&mut p, k, x, retire);
            if !["stockShare", "reserveMonths"].contains(&k) {
                assert_eq!(current(&p, k), Some(x), "{k}");
            }
        }
        assert_eq!(
            p["wealth"]["stockShare"]["rows"][1],
            json!({"from": retire, "value": 0.7})
        );
        assert_eq!(
            p["wealth"]["reserveMonths"]["rows"][1],
            json!({"from": retire, "value": 9.0})
        );
        assert_eq!(
            p["withdrawal"]["guardrails"],
            json!({"initialRate": 0.05, "upper": 0.3, "lower": 0.2, "adjust": 0.1})
        );
        assert_eq!(
            p["withdrawal"]["buckets"],
            json!({"cashYears": 2.0, "bondYears": 4.0, "refillThreshold": 0.0})
        );
        assert_eq!(p["withdrawal"]["vpw"], json!({"horizonAge": 100.0, "realReturn": 0.01}));
        assert_eq!(p["withdrawal"]["endowment"], json!({"rate": 0.05, "smoothing": 0.3}));
        // Existing sections keep their other values.
        set(&mut p, "lower", 0.25, retire);
        set(&mut p, "cashYears", 3.0, retire);
        set(&mut p, "horizonAge", 95.0, retire);
        set(&mut p, "smoothing", 0.6, retire);
        assert_eq!(p["withdrawal"]["guardrails"]["upper"], json!(0.3));
        assert_eq!(p["withdrawal"]["buckets"]["bondYears"], json!(4.0));
        assert_eq!(p["withdrawal"]["vpw"]["realReturn"], json!(0.01));
        assert_eq!(p["withdrawal"]["endowment"], json!({"rate": 0.05, "smoothing": 0.6}));
        let before = p.clone();
        set(&mut p, "unbekannt", 1.0, retire);
        assert_eq!(p, before);
    }

    #[test]
    fn path_statistics() {
        // Plenty of wealth: 12'000 consumption every year, no fluctuation; withdrawal rate 12'000 / wealth before retirement.
        let (sim, res) = run_calm(1_000_000.0);
        let retire = sim.retire_at().unwrap();
        let st = path_stats(&sim, &res, retire, &Objective::default());
        assert!(!st.ruined && !st.emergency);
        assert_eq!(st.annual, vec![12_000.0; 30]);
        assert_eq!(st.planned, vec![12_000.0; 30]);
        assert_eq!(st.vol, 0.0);
        assert!((st.draw_rate - 12_000.0 / 940_000.0).abs() < 1e-12);

        // 100'000: depleted from February 2035 (age 68.25), afterwards only what income (0) covers counts.
        let (sim, res) = run_calm(100_000.0);
        let st = path_stats(&sim, &res, retire, &Objective::default());
        assert!(st.ruined);
        assert_eq!(&st.annual[..5], &[12_000.0, 12_000.0, 12_000.0, 3_000.0, 0.0]);
        // Changes 0, 0, 0.75, 1 (years with 0 do not count as a starting value)
        assert!((st.vol - 1.75 / 4.0).abs() < 1e-12);
        assert!((st.draw_rate - 12_000.0 / 40_000.0).abs() < 1e-12);
        let ruin_until = |age: f64| {
            path_stats(
                &sim,
                &res,
                retire,
                &Objective {
                    ruin_age: Some(age),
                    ..Default::default()
                },
            )
            .ruined
        };
        assert!(ruin_until(68.25) && !ruin_until(68.0));

        // Without wealth at retirement: withdrawal rate 0
        let (sim, res) = run_calm(60_000.0);
        assert_eq!(path_stats(&sim, &res, retire, &Objective::default()).draw_rate, 0.0);

        // With AHV (500) and taxes in the first month (retirement in May 2032): wealth before the month from the previous month.
        let mut v = calm(500_000.0);
        v["pension"]["ahv"]["monthly"] = json!(500);
        v["taxes"]["income"] = json!([[0, 0.1]]);
        v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2032-05"});
        let sim = Simulator::new(&parse_plan(&v).unwrap()).unwrap();
        let res = sim.run(&crate::market::fixed_path(&sim.fixed, sim.start, sim.months()), true);
        let retire = sim.retire_at().unwrap();
        let k = res.records.iter().position(|r| r.month == retire).unwrap();
        assert!(res.records[k].taxes > 0.0 && res.records[k].income > 0.0);
        let st = path_stats(&sim, &res, retire, &Objective::default());
        // #190: the 13th AHV pension in December 2032 lowers the withdrawal of the first 12 months.
        let dec = res
            .records
            .iter()
            .find(|r| r.month == YearMonth::new(2032, 12))
            .unwrap();
        let thirteenth = dec.ahv - 500.0;
        assert!(thirteenth > 0.0);
        assert!((st.draw_rate - (12.0 * 500.0 - thirteenth) / res.records[k - 1].liquid).abs() < 1e-12);
    }

    #[test]
    fn path_statistics_in_real_terms_with_rent() {
        // Inflation 2 %, rent 500 (indexed): consumption and plan real 12 × (1000 + 500) every year.
        let mut v = calm(1_000_000.0);
        v["returns"]["fixed"]["inflation"] = json!(0.02);
        v["flows"]["spendingIncludesHousing"] = json!(false);
        v["realestate"]["properties"] = json!([{"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0,
            "inflation": true, "inflationRent": true, "from": "start", "rent": 500}]);
        let sim = Simulator::new(&parse_plan(&v).unwrap()).unwrap();
        let res = sim.run(&crate::market::fixed_path(&sim.fixed, sim.start, sim.months()), true);
        assert!(res.records.last().unwrap().price_index > 1.5);
        let st = path_stats(&sim, &res, sim.retire_at().unwrap(), &Objective::default());
        assert_eq!(st.annual.len(), 30);
        for (a, p) in st.annual.iter().zip(&st.planned) {
            assert!(
                (a / 18_000.0 - 1.0).abs() < 0.01 && (p / 18_000.0 - 1.0).abs() < 0.01,
                "{:?} {:?}",
                st.annual,
                st.planned
            );
        }
    }

    /// Objective function that records the points checked.
    fn traced<'a>(f: impl Fn(f64) -> f64 + 'a, seen: &'a mut Vec<f64>) -> impl FnMut(&[f64]) -> Result<((), f64)> + 'a {
        move |x: &[f64]| {
            seen.push(x[0]);
            Ok(((), f(x[0])))
        }
    }

    #[test]
    fn search_takes_the_best_start_point() {
        // Plateaus without slope: from one starting point the search gets nowhere, the best start decides.
        let plateau = |x: f64| {
            if x > 0.85 {
                2.0
            } else if x < 0.15 {
                1.0
            } else {
                0.0
            }
        };
        let mut seen = Vec::new();
        let starts = vec![vec![0.5], vec![0.9], vec![0.1], vec![0.95], vec![-3.0]];
        let (x, _, f) = search(starts, &[(0.0, 1.0)], traced(plateau, &mut seen)).unwrap();
        // With equal fitness the earlier starting point wins; starting points outside are clamped.
        assert_eq!((x, f), (vec![0.9], 2.0));
        assert_eq!(&seen[..5], &[0.5, 0.9, 0.1, 0.95, 0.0]);
    }

    #[test]
    fn search_steps_and_directions() {
        // Without improvement: +20 %, then −20 % of the range per axis (here 0.5 to 2.5), then half steps down to 1.25 %.
        let bounds = [(0.5, 2.5)];
        let mut seen = Vec::new();
        let (x, _, _) = search(vec![vec![1.0]], &bounds, traced(|_| 0.0, &mut seen)).unwrap();
        assert_eq!(x, vec![1.0]);
        let expected = [0.4, -0.4, 0.2, -0.2, 0.1, -0.1, 0.05, -0.05, 0.025, -0.025];
        assert_eq!(seen.len(), 1 + expected.len(), "{seen:?}");
        for (v, d) in seen[1..].iter().zip(expected) {
            assert!((v - 1.0 - d).abs() < 1e-12, "{seen:?}");
        }

        // The first improvement is taken: first it goes up to the limit.
        let (x, _, _) = search(vec![vec![1.0]], &bounds, |x: &[f64]| Ok(((), (x[0] - 1.0).powi(2)))).unwrap();
        assert_eq!(x, vec![2.5]);

        // A maximum inside is found to within one step (in both directions).
        for peak in [0.87, 2.11] {
            let (x, _, _) = search(vec![vec![1.0]], &bounds, |x: &[f64]| Ok(((), -(x[0] - peak).powi(2)))).unwrap();
            assert!((x[0] - peak).abs() <= 0.025, "{x:?} statt {peak}");
        }
    }

    #[test]
    fn progress_estimate_of_the_search() {
        let req = Request {
            samples: 10,
            paths: 100,
            check_paths: 1000,
            ..Default::default()
        };
        // Fixed rate: 10 samples + plan + middle + 14 steps, plus checking strategy and plan.
        assert_eq!(
            progress_total(&req, &[OptStrategy::FixedRate], 1).unwrap(),
            26 * 100 + 2 * 1000
        );
        // All: 7 × 12 starting points + 22 + 14 + 46 + 110 + 40 + 74 + 90 steps (#6: «3-Topf» + guardrails)
        assert_eq!(
            progress_total(&req, &all_strategies(), 1).unwrap(),
            (84 + 396) * 100 + 8 * 1000
        );
        // One of two parameters held fixed: half the number of steps; all fixed: one run.
        let fix = |keys: &[&str]| -> Ranges {
            [(
                "fixed_rate".to_string(),
                keys.iter()
                    .map(|k| {
                        (
                            k.to_string(),
                            crate::model::ParamRange {
                                fixed: Some(true),
                                lo: None,
                                hi: None,
                            },
                        )
                    })
                    .collect(),
            )]
            .into()
        };
        let one = Request {
            ranges: fix(&["stockShare"]),
            ..req.clone()
        };
        assert_eq!(
            progress_total(&one, &[OptStrategy::FixedRate], 1).unwrap(),
            19 * 100 + 2 * 1000
        );
        let all = Request {
            ranges: fix(&["stockShare", "rate"]),
            ..req.clone()
        };
        assert_eq!(
            progress_total(&all, &[OptStrategy::FixedRate], 1).unwrap(),
            100 + 2 * 1000
        );
    }

    fn emergency_plan() -> Value {
        let mut e = calm(20_000.0);
        e["realestate"]["emergencyMonths"] = json!(6);
        e["realestate"]["properties"] = json!([{"name": "Haus", "status": "owned", "price": 300000, "worth": 300000,
            "sell": "never", "fixCosts": 0, "inflation": false}]);
        e["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
        e
    }

    #[test]
    fn evaluate_averages_over_paths() {
        let plan = parse_plan(&calm(100_000.0)).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let paths = Paths::List(vec![path.clone(), path.clone(), path]);
        let m = evaluate(&plan, &paths, &Objective::default()).unwrap();
        assert_eq!((m.runs, m.ruin, m.emergency_share), (3, 1.0, 0.0));
        assert!((m.volatility - 0.4375).abs() < 1e-12);
        assert!((m.draw_rate - 0.3).abs() < 1e-12);
        assert_eq!(m.planned, 12_000.0);

        let plan = parse_plan(&emergency_plan()).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let res = sim.run(&path, true);
        assert!(path_stats(&sim, &res, sim.retire_at().unwrap(), &Objective::default()).emergency);
        let m = evaluate(
            &plan,
            &Paths::List(vec![path.clone(), path.clone(), path]),
            &Objective::default(),
        )
        .unwrap();
        assert_eq!(m.emergency_share, 1.0);
    }

    #[test]
    fn splitmix_values() {
        let mut r = Rng(0);
        assert_eq!(r.next(), 0.8833108082136426);
        assert_eq!(r.next(), 0.43152799704850997);
    }

    #[test]
    fn search_margin_and_start_points() {
        // 1.5 standard errors at 100 paths: 1.5 × √(0.05 × 0.95 / 100)
        let o = Objective::default();
        let f = fit_objective(&o, 100);
        assert!((f.max_ruin - (0.05 - 1.5 * (0.05f64 * 0.95 / 100.0).sqrt())).abs() < 1e-15);
        assert_eq!(fit_objective(&o, 1).max_ruin, 0.0);
        assert_eq!(fit_objective(&o, 0).max_ruin, 0.0);

        let defs = param_defs(OptStrategy::FixedRate)[..2].to_vec(); // rate 2–8 %, stock share 10–100 %
        let plan = json!({"withdrawal": {"fixedRate": {"rate": 0.03}}, "wealth": {"stockShare": 1.5}});
        let st = Settings { samples: 20, seed: 4 };
        let s = start_points(&plan, &defs, &st);
        assert_eq!(s.len(), 22);
        // Value of the plan (stock share outside ⇒ middle), then the middle
        assert_eq!(s[0], vec![0.03, 0.55]);
        assert_eq!(s[1], vec![0.05, 0.55]);
        for x in &s[2..] {
            assert!((0.02..=0.08).contains(&x[0]) && (0.1..=1.0).contains(&x[1]), "{x:?}");
        }
        let spread = |i: usize| {
            s[2..].iter().map(|x| x[i]).fold(f64::INFINITY, f64::min) < s[2..].iter().map(|x| x[i]).fold(0.0, f64::max)
        };
        assert!(spread(0) && spread(1));
        assert_eq!(start_points(&plan, &[], &st).len(), 1);
        // Random starting points from Rng(seed XOR 0xA5A5_5A5A); seed 10 overlaps the bits of the constant.
        let s = start_points(&plan, &defs, &Settings { samples: 1, seed: 10 });
        let mut r = Rng(10 ^ 0xA5A5_5A5A);
        let at = |d: &ParamDef, u: f64| d.lo + (d.hi - d.lo) * u;
        assert_eq!(s[2], vec![at(&defs[0], r.next()), at(&defs[1], r.next())]);
    }

    // The pattern search finds the optimum of the withdrawal rate (fine grid for comparison).
    // slow: pattern search compared with a fine grid
    #[test]
    fn search_finds_the_optimum() {
        let mut f = calm(400_000.0);
        f["withdrawal"]["strategy"] = json!("fixed_rate");
        let plan = parse_plan(&f).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let retire = sim.retire_at().unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let paths = Paths::List(vec![path.clone(), path]);
        let ranges: Ranges = [(
            "fixed_rate".to_string(),
            [(
                "stockShare".to_string(),
                crate::model::ParamRange {
                    fixed: Some(true),
                    lo: None,
                    hi: None,
                },
            )]
            .into(),
        )]
        .into();
        let o = Objective::default();
        let out = optimize_strategy(
            &f,
            OptStrategy::FixedRate,
            retire,
            PathSets {
                fit: &paths,
                check: &paths,
            },
            &o,
            &Settings { samples: 3, seed: 1 },
            &ranges,
        )
        .unwrap();
        assert!(out.feasible);
        assert_eq!(out.params.len(), 1);
        assert_eq!(
            out.fixed.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(),
            ["stockShare", "floor", "ceiling"]
        );
        let rate = out.params[0].2;
        let fit = |r: f64| {
            let p = parse_plan(&with_params(
                &f,
                OptStrategy::FixedRate,
                &param_defs(OptStrategy::FixedRate)[..1],
                &[r],
                retire,
            ))
            .unwrap();
            evaluate(&p, &paths, &o).unwrap().fitness(&o)
        };
        let (best_r, best_f) = (0..=120)
            .map(|i| 0.02 + i as f64 * 0.0005)
            .map(|r| (r, fit(r)))
            .fold((0.0, f64::MIN), |a, b| if b.1 > a.1 { b } else { a });
        assert!((rate - best_r).abs() < 0.002, "Suche {rate}, Raster {best_r}");
        assert!(fit(rate) >= best_f - 1.0, "{} < {best_f}", fit(rate));
        assert_eq!(out.patch["withdrawal"]["fixedRate"]["rate"], json!(rate));
    }

    #[test]
    fn request_defaults_and_checks() {
        let r: Request = serde_json::from_value(json!({})).unwrap();
        assert_eq!((r.max_ruin, r.weight_median, r.vol_weight), (0.05, 0.5, 1.0));
        assert_eq!((r.paths, r.check_paths, r.samples, r.seed), (5000, 10_000, 30, 1));
        let r = Request {
            max_ruin: 0.1,
            weight_median: 0.6,
            vol_weight: 2.0,
            min_consumption: 0.8,
            ruin_age: Some(90.0),
            max_vol: Some(0.2),
            goal: Goal::KeepProperty,
            ..Default::default()
        };
        let o = r.objective();
        assert_eq!(
            (
                o.max_ruin,
                o.weight_median,
                o.vol_weight,
                o.min_consumption,
                o.ruin_age,
                o.max_vol,
                o.goal
            ),
            (0.1, 0.6, 2.0, 0.8, Some(90.0), Some(0.2), Goal::KeepProperty)
        );
        let list = |s: &[&str]| {
            Request {
                strategies: s.iter().map(|x| x.to_string()).collect(),
                ..Default::default()
            }
            .strategy_list()
            .unwrap()
            .len()
        };
        assert_eq!(
            (list(&[]), list(&["all"]), list(&["vpw", "all"]), list(&["vpw"])),
            (7, 7, 7, 1)
        );

        let dir = std::path::Path::new(".");
        let err = |base: &Value, r: Request| run_request(base, dir, None, &r).unwrap_err().to_string();
        let plain = calm(100_000.0);
        assert!(err(
            &plain,
            Request {
                check_paths: 0,
                ..Default::default()
            }
        )
        .contains("number of paths"));
        assert!(err(
            &plain,
            Request {
                paths: 0,
                ..Default::default()
            }
        )
        .contains("number of paths"));
        // Goal 1 needs no property (the unknown strategy ends the request afterwards).
        let unknown = || Request {
            strategies: vec!["x".into()],
            ..Default::default()
        };
        assert!(err(&plain, unknown()).contains("Unknown strategy"));
        // Goal 2 needs a property that is not rented, and a minimum consumption between 0 and 200 %.
        let keep = |min: f64| Request {
            goal: Goal::KeepProperty,
            min_consumption: min,
            ..unknown()
        };
        assert!(err(&plain, keep(0.9)).contains("at least one property"));
        let mut rent = calm(100_000.0);
        rent["realestate"]["properties"] = json!([{"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "start", "rent": 1000}]);
        assert!(err(&rent, keep(0.9)).contains("at least one property"));
        let owned = emergency_plan();
        assert!(err(&owned, keep(2.5)).contains("minimum consumption"));
        assert!(err(&owned, keep(0.9)).contains("Unknown strategy"));
    }

    fn outcome(feasible: bool, ruin: f64) -> Outcome {
        let m = Metrics {
            ruin,
            volatility: 0.0,
            ..metrics()
        };
        Outcome {
            strategy: Message::default(),
            params: vec![],
            fixed: vec![],
            fitted: m,
            validated: m,
            feasible,
            evaluations: 0,
            patch: Value::Null,
            never_sell: false,
        }
    }

    #[test]
    fn ranking_of_outcomes() {
        let o = Objective::default();
        assert!((rank(&outcome(true, 0.01), &o) - outcome(true, 0.01).validated.score(&o)).abs() < 1e-12);
        // Infeasible: −1e9 × (1 + violation)
        assert!((rank(&outcome(false, 0.15), &o) - -1e9 * 1.1).abs() < 1e-3);
        assert!(rank(&outcome(false, 0.10), &o) > rank(&outcome(false, 0.15), &o));
    }

    #[test]
    fn report_time_in_milliseconds() {
        let mut v = calm(400_000.0);
        v["returns"]["source"] = json!("parametric");
        v["returns"]["parametric"] = json!({"paths": 2, "mu": 0.05, "sigma": 0.1, "df": 0});
        let r = Request {
            strategies: vec!["standard".into()],
            paths: 2,
            check_paths: 2,
            samples: 0,
            ranges: [(
                "standard".to_string(),
                [(
                    "reserveMonths".to_string(),
                    crate::model::ParamRange {
                        fixed: Some(true),
                        lo: None,
                        hi: None,
                    },
                )]
                .into(),
            )]
            .into(),
            ..Default::default()
        };
        let t = std::time::Instant::now();
        let rep = run_request(&v, std::path::Path::new("."), None, &r).unwrap();
        let outer_ms = t.elapsed().as_secs_f64() * 1e3;
        assert_eq!((rep.fit_paths, rep.check_paths, rep.outcomes.len()), (2, 2, 1));
        // Measured time in milliseconds, at most as long as the whole call.
        assert!(
            rep.elapsed_ms > 0.01 && rep.elapsed_ms <= outer_ms,
            "{} > {outer_ms}",
            rep.elapsed_ms
        );
    }

    // ---- #205 ----

    /// #205/AK4: wealth as cash, AHV 3'000 and spending 9'000 per month, fixed rate 4 %, floor 80 %, fixed returns 0.
    fn floor_plan(cash: f64) -> Value {
        let mut v = calm(cash);
        v["pension"]["ahv"]["monthly"] = json!(3000);
        v["flows"]["spending"] = json!(9000);
        v["withdrawal"]["strategy"] = json!("fixed_rate");
        v["withdrawal"]["fixedRate"] = json!({"rate": 0.04});
        v["withdrawal"]["floor"] = json!(0.8);
        v
    }

    fn floor_metrics(v: &Value) -> Metrics {
        let plan = parse_plan(v).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        evaluate(&plan, &Paths::List(vec![path.clone(), path]), &Objective::default()).unwrap()
    }

    #[test]
    fn issue_205_ak1_ak4_floor_share_with_little_wealth_is_full() {
        // #205/AK4: 4 % of 300'000 = 1'000 per month plus AHV is below 80 % of 9'000: every month at the floor.
        let m = floor_metrics(&floor_plan(300_000.0));
        assert_eq!(m.floor_share, Some(1.0));
        assert!(m.floor_share.unwrap() >= FLOOR_HINT_SHARE);
    }

    #[test]
    fn issue_205_ak4_no_floor_month_in_the_first_year_with_much_wealth() {
        // #205/AK4: 4 % of 3'000'000 = 10'000 per month plus AHV is above the floor in the first year.
        let plan = parse_plan(&floor_plan(3_000_000.0)).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let res = sim.run(&crate::market::fixed_path(&sim.fixed, sim.start, sim.months()), true);
        let retire = sim.retire_at().unwrap();
        let first: Vec<_> = res.records.iter().filter(|r| r.month >= retire).take(12).collect();
        assert_eq!(first.len(), 12);
        assert!(first.iter().all(|r| !r.at_floor));
        // Over all months from retirement the share is the mean of the flags and below the hint threshold.
        let share = path_stats(&sim, &res, retire, &Objective::default()).floor_share;
        let flags: Vec<bool> = res
            .records
            .iter()
            .filter(|r| r.month >= retire)
            .map(|r| r.at_floor)
            .collect();
        let expected = flags.iter().filter(|f| **f).count() as f64 / flags.len() as f64;
        assert!((share - expected).abs() < 1e-12);
        assert!(share < FLOOR_HINT_SHARE, "{share}");
        assert_eq!(floor_metrics(&floor_plan(3_000_000.0)).floor_share, Some(share));
    }

    #[test]
    fn issue_205_ak1_floor_share_is_averaged_over_the_paths() {
        // Wealth in stocks: a path without returns and one with high stock returns give different shares.
        let mut v = floor_plan(0.0);
        v["wealth"] = json!({"stocks": 1_000_000, "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": 0});
        let plan = parse_plan(&v).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let retire = sim.retire_at().unwrap();
        let flat = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let mut rich = flat.clone();
        rich.stocks.iter_mut().for_each(|r| *r = 0.02);
        let share = |p| path_stats(&sim, &sim.run(p, true), retire, &Objective::default()).floor_share;
        let (a, b) = (share(&flat), share(&rich));
        assert!(a > b, "{a} {b}");
        let m = evaluate(&plan, &Paths::List(vec![flat, rich]), &Objective::default()).unwrap();
        assert!((m.floor_share.unwrap() - (a + b) / 2.0).abs() < 1e-12);
    }

    #[test]
    fn issue_205_ak3_strategies_without_floor_have_no_share() {
        let base = floor_plan(300_000.0);
        let without = |f: &dyn Fn(&mut Value)| {
            let mut v = base.clone();
            f(&mut v);
            floor_metrics(&v).floor_share
        };
        assert_eq!(without(&|v| v["withdrawal"]["strategy"] = json!("standard")), None);
        assert_eq!(without(&|v| v["withdrawal"]["strategy"] = json!("three_buckets")), None);
        assert_eq!(without(&|v| v["withdrawal"]["floor"] = json!(0)), None);
        // 3 buckets with guardrails have a floor (#204).
        let with = without(&|v| {
            v["withdrawal"]["strategy"] = json!("three_buckets");
            v["withdrawal"]["buckets"]["spending"] = json!("guardrails");
        });
        assert!(with.is_some(), "{with:?}");
        // Missing in the JSON without a floor, present with one.
        let mut m = metrics();
        assert!(serde_json::to_value(m).unwrap().get("floorShare").is_none());
        m.floor_share = Some(0.25);
        assert_eq!(serde_json::to_value(m).unwrap()["floorShare"], json!(0.25));
    }

    // ---- #215 ----

    /// Made-up plan on parametric paths without a series, for the optimization with and without kept paths.
    fn reuse_plan() -> Value {
        let mut v = calm(400_000.0);
        v["returns"]["source"] = json!("parametric");
        v["returns"]["parametric"] = json!({"paths": 6, "mu": 0.04, "sigma": 0.15, "df": 0});
        v
    }

    fn report_without_time(r: Report) -> Value {
        Value::from_iter(
            serde_json::to_value(Report { elapsed_ms: 0.0, ..r })
                .unwrap()
                .as_object()
                .unwrap()
                .clone(),
        )
    }

    /// #215/AK2, AK3, AK5: The search keeps the search and check paths for all evaluations; the report is bit for bit
    /// the same as without kept paths (budget 0: every evaluation generates its paths again).
    // slow: runs the search twice, with and without kept paths
    #[test]
    fn issue_215_ak2_search_with_kept_paths_is_bit_identical() {
        let v = reuse_plan();
        let r = Request {
            strategies: vec!["fixed_rate".into(), "guardrails".into()],
            paths: 5,
            check_paths: 7,
            samples: 1,
            ..Default::default()
        };
        let dir = std::path::Path::new(".");
        let kept = run_request(&v, dir, None, &r).unwrap();
        assert_eq!((kept.fit_paths, kept.check_paths, kept.outcomes.len()), (5, 7, 2));
        let fresh = run_request_within(&v, dir, None, &r, 0).unwrap();
        assert_eq!(report_without_time(kept), report_without_time(fresh));
    }

    /// #215/AK2, AK3, AK5: The comparison of the strategies is the same with and without kept paths.
    // slow: compares all strategies twice, with and without kept paths
    #[test]
    fn issue_215_ak2_compare_with_kept_paths_is_bit_identical() {
        let v = reuse_plan();
        let r = Request {
            compare: true,
            ..Default::default()
        };
        let dir = std::path::Path::new(".");
        let kept = run_request(&v, dir, None, &r).unwrap();
        assert_eq!((kept.fit_paths, kept.outcomes.len()), (6, 7));
        let fresh = run_request_within(&v, dir, None, &r, 0).unwrap();
        assert_eq!(report_without_time(kept), report_without_time(fresh));
    }

    // ---- #248 ----

    /// Monthly course of `wealth.<key>` as the simulation reads it, from the start to the end of [`calm`].
    fn issue_248_course(v: &Value, key: &str) -> Vec<(YearMonth, f64)> {
        let p = parse_plan(v).unwrap();
        // The plan with the changed course must still be accepted by the simulation.
        Simulator::new(&p).unwrap();
        let r = crate::resolve::Resolver::new(&p).unwrap();
        let s = match key {
            "stockShare" => p.wealth.stock_share.as_series().unwrap(),
            _ => &p.wealth.reserve_months,
        };
        let c = crate::resolve::Curve::compile(s, &r).unwrap();
        let (start, end) = (r.milestone("start").unwrap(), r.milestone("end").unwrap());
        (0..=start.until(end))
            .map(|k| start.plus(k))
            .map(|m| (m, c.at(m)))
            .collect()
    }

    /// #248/AK1: The optimization of `key` with the value `x` keeps the course `course` before retirement (2031-11 in
    /// [`calm`]) month by month and sets `x` from retirement on.
    fn issue_248_keeps_course(key: &str, course: Value, x: f64) {
        let retire = "2031-11";
        let mut before = calm(100_000.0);
        before["wealth"][key] = course.clone();
        let mut after = before.clone();
        set(&mut after, key, x, retire);
        let (old, new) = (issue_248_course(&before, key), issue_248_course(&after, key));
        let r: YearMonth = retire.parse().unwrap();
        for ((m, a), (_, b)) in old.iter().zip(&new) {
            let expected = if *m < r { *a } else { x };
            assert_eq!(*b, expected, "{key} {course} in {}", m.iso());
        }
        // The course differs from the optimized value before retirement, so a replaced course would be noticed.
        assert!(old.iter().any(|(m, a)| *m < r && *a != x), "{course}");
    }

    #[test]
    fn issue_248_ak2_const_stock_share_and_reserve() {
        issue_248_keeps_course("stockShare", json!({"mode": "const", "value": 0.4}), 0.7);
        issue_248_keeps_course("stockShare", json!(0.4), 0.7);
        issue_248_keeps_course("reserveMonths", json!({"mode": "const", "value": 6}), 9.0);
        issue_248_keeps_course("reserveMonths", json!(6), 9.0);
    }

    #[test]
    fn issue_248_ak2_timed_stock_share_and_reserve() {
        let rows = |a: f64, b: f64, c: f64| {
            json!({"mode": "timed", "rows": [{"from": "start", "value": a}, {"from": "2029-01", "value": b},
                {"from": "retire", "value": c}, {"from": "2034-01", "value": a}]})
        };
        issue_248_keeps_course("stockShare", rows(0.3, 0.5, 0.8), 0.7);
        issue_248_keeps_course("reserveMonths", rows(3.0, 6.0, 12.0), 9.0);
    }

    #[test]
    fn issue_248_ak2_linear_stock_share_and_reserve() {
        // From age 61 to 70, across retirement at 65; also entirely before retirement (by date).
        let line = |a: f64, b: f64| json!({"mode": "linear", "start": {"age": 61, "value": a}, "end": {"age": 70, "value": b}});
        issue_248_keeps_course("stockShare", line(0.8, 0.4), 0.7);
        issue_248_keeps_course("reserveMonths", line(4.0, 12.0), 9.0);
        let early =
            json!({"mode": "linear", "start": {"at": "2027-01", "value": 2}, "end": {"at": "2030-01", "value": 8}});
        issue_248_keeps_course("reserveMonths", early, 9.0);
    }

    #[test]
    fn issue_248_ak2_table_stock_share_and_reserve() {
        for interpolation in ["linear", "step"] {
            let table = |a: f64, b: f64, c: f64| {
                json!({"mode": "table", "interpolation": interpolation, "points": [{"age": 60, "value": a},
                    {"at": "2029-06", "value": b}, {"age": 68, "value": c}]})
            };
            issue_248_keeps_course("stockShare", table(0.9, 0.7, 0.3), 0.6);
            issue_248_keeps_course("reserveMonths", table(3.0, 6.0, 12.0), 9.0);
        }
    }

    #[test]
    fn issue_248_ak2_factor_stock_share_and_reserve() {
        let factor = |base: f64| {
            json!({"mode": "factor", "base": base, "factor": {"mode": "linear",
                "start": {"at": "2028-01", "value": 1}, "end": {"at": "retire", "value": 0.5}}})
        };
        issue_248_keeps_course("stockShare", factor(0.8), 0.7);
        issue_248_keeps_course("reserveMonths", factor(12.0), 9.0);
    }

    /// #248: A characteristic curve (#218) cannot be cut at a point in time; the optimized stock share applies from
    /// the start (decision in the issue).
    #[test]
    fn issue_248_ak2_curve_stock_share_becomes_the_optimized_value() {
        let mut v = calm(100_000.0);
        v["wealth"]["stockShare"] = json!({"mode": "curve", "input": "age", "points": [[60, 0.8], [70, 0.4]]});
        set(&mut v, "stockShare", 0.7, "2031-11");
        assert!(issue_248_course(&v, "stockShare").iter().all(|(_, a)| *a == 0.7));
    }

    /// #248/AK1: The optimization itself (search and patch) keeps the straight line before retirement.
    #[test]
    fn issue_248_ak1_optimized_patch_keeps_the_line_before_retirement() {
        let mut v = calm(100_000.0);
        let line = json!({"mode": "linear", "start": {"age": 61, "value": 0.8}, "end": {"age": 70, "value": 0.4}});
        v["wealth"]["stockShare"] = line;
        let retire = YearMonth::new(2031, 11);
        let x = [0.55];
        let defs = [STOCK_SHARE];
        let patched = with_params(&v, OptStrategy::FixedRate, &defs, &x, retire);
        let old = issue_248_course(&v, "stockShare");
        let new = issue_248_course(&patched, "stockShare");
        for ((m, a), (_, b)) in old.iter().zip(&new) {
            assert_eq!(*b, if *m < retire { *a } else { 0.55 }, "{}", m.iso());
        }
    }
}
