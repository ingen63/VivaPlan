//! #216: Experiment. A question about a plan as one JSON structure, computed in one call without changing the engine:
//! changes (`set`), the space of variations (`vary`: lists, ranges, groups, coupled values, all with the field paths of
//! #207 including a point in time), metrics of the measurement layer (#214), constraints and an objective, the search
//! (grid or pattern search, in two stages) and the budget. Everything is structured JSON without expressions in text.
//!
//! Flow of [`evaluate`]: check the experiment completely (a list of messages with path, reason and valid values, nothing
//! is computed) → estimate the duration (from about 60 s only with `confirm`) → search on few paths with a margin on the
//! constraints → confirm the best `k` on **independent** paths (other seed) → ranking with both values.
//!
//! The functions of today are special cases: variations = grid without an objective, comparison of strategies = a
//! group of strategies without a search, optimization = pattern search with the objective `score`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::batch::{self, Paths};
use crate::market::DataSet;
use crate::measure::{Aggregate, Measurement, Metric, MetricValue, Op};
use crate::message::{arg, Message};
use crate::model::{Plan, ReturnSource};
use crate::optimize::{self, Objective};
use crate::sim::Simulator;
use crate::{Error, Result};

/// Most variants of a grid.
pub const MAX_VARIANTS: usize = 10_000;

/// From this estimated duration (seconds) the computation starts only with confirmation (`confirm`, `--yes`).
pub const CONFIRM_SECONDS: f64 = 60.0;

/// Least share of the measured time per run that a run without generating its return path takes (#311). The
/// generation is measured on its own and briefly; under load the measurement can come out as large as the whole run,
/// which would put the estimate near 0 s and skip the confirmation. Too small a share only overestimates.
const MIN_REUSE_SHARE: f64 = 0.5;

/// Estimated seconds of `runs` runs of which `generated` generate their return path (#216/AK7), from the measurement
/// of the machine.
fn estimate_seconds(runs: u64, generated: f64, us_per_run: f64, us_generate: f64) -> f64 {
    let us_reuse = (us_per_run - us_generate).max(us_per_run * MIN_REUSE_SHARE);
    (runs as f64 * us_reuse + generated * us_generate) / 1e6
}

/// Most variants listed in the result of an experiment without an objective (the result stays small).
pub const MAX_LISTED: usize = 50;

// ---------------------------------------------------------------------------------------------
// The experiment (JSON)
// ---------------------------------------------------------------------------------------------

/// An experiment on a plan. Field paths as in `vivaplan set` (#207): `wealth.stockShare`, with a point in time
/// `wealth.stockShare@retire`, `@age:70` or `@2035-01`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Experiment {
    /// Changes for all variants: field path → value.
    #[serde(default)]
    pub set: BTreeMap<String, Value>,
    /// Space of variations: the variations are combined with each other (each with each), the members of a group not.
    #[serde(default)]
    pub vary: Vec<Variation>,
    /// Metrics of the measurement layer, each with a `name` for constraints and objective.
    #[serde(default)]
    pub metrics: Vec<Metric>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Without an objective all variants are listed (no ranking and no confirmation).
    #[serde(default)]
    pub objective: Option<ExperimentObjective>,
    #[serde(default)]
    pub search: ExperimentSearch,
    #[serde(default)]
    pub budget: ExperimentBudget,
    /// Compute even if the estimated duration is 60 s or more.
    #[serde(default)]
    pub confirm: bool,
}

/// A variation: a field path with a list of values, a range or a coupling, or a group.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Variation {
    /// Field path, with a point in time if needed (not with `group`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Values as a list (numbers, texts, objects).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<Value>>,
    /// Range [from, to, step]; the grid takes from, from + step, … up to to; the pattern search the interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<[f64; 3]>,
    /// Coupled value: the value of this set or varied field path plus `offset`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follows: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
    /// Group: alternatives that are not combined with each other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<GroupMember>>,
}

/// A member of a group: its own changes and variations.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupMember {
    #[serde(default)]
    pub set: BTreeMap<String, Value>,
    #[serde(default)]
    pub vary: Vec<Variation>,
}

/// Constraint on a metric: `{"metric": "ruin", "op": "le", "value": 0.02}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Constraint {
    /// Name of a metric.
    pub metric: String,
    pub op: Op,
    pub value: f64,
}

/// Objective: exactly one of `maximize`, `minimize` (name of a metric), `weighted` (maximize the weighted sum; a
/// negative weight counts against) or `score` (the objective of the optimization).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExperimentObjective {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximize: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimize: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weighted: Option<Vec<Weight>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<Score>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Weight {
    pub metric: String,
    pub weight: f64,
}

/// The objective of the optimization (`vivaplan optimize`), with the same defaults: its metrics and its score
/// (consumption from retirement, risk of ruin, fluctuation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct Score {
    pub max_ruin: f64,
    pub goal: crate::model::Goal,
    pub min_consumption: f64,
    pub ruin_age: Option<f64>,
    pub weight_median: f64,
    pub vol_weight: f64,
    pub max_vol: Option<f64>,
}

impl Default for Score {
    fn default() -> Self {
        let o = Objective::default();
        Score {
            max_ruin: o.max_ruin,
            goal: o.goal,
            min_consumption: o.min_consumption,
            ruin_age: o.ruin_age,
            weight_median: o.weight_median,
            vol_weight: o.vol_weight,
            max_vol: o.max_vol,
        }
    }
}

impl Score {
    fn objective(&self) -> Objective {
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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Method {
    /// All variants (ranges in steps).
    #[default]
    Grid,
    /// Pattern search within the ranges (as the optimization), per combination of the other values.
    Pattern,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct ExperimentSearch {
    pub method: Method,
    /// Pattern search: random starting points in addition to the plan's value and the middle.
    pub samples: usize,
    /// Pattern search: seed of the random starting points.
    pub seed: u64,
}

impl Default for ExperimentSearch {
    fn default() -> Self {
        ExperimentSearch {
            method: Method::Grid,
            samples: 30,
            seed: 1,
        }
    }
}

/// Budget of the two stages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct ExperimentBudget {
    /// Paths of the search (Monte Carlo; fixed paths of the plan stay as they are).
    pub paths: usize,
    /// Margin on the constraints in the search, in standard errors.
    pub margin: f64,
    /// Number of best variants confirmed.
    pub top: usize,
    /// Paths of the confirmation, with another seed than the search.
    pub confirm_paths: usize,
}

impl Default for ExperimentBudget {
    fn default() -> Self {
        ExperimentBudget {
            paths: 1000,
            margin: 2.0,
            top: 5,
            confirm_paths: 10_000,
        }
    }
}

/// JSON schema of the experiment.
pub fn schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_json::to_value(schemars::schema_for!(Experiment)).expect("schema is serializable"))
}

// ---------------------------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    /// The experiment or the plan has errors; nothing was computed.
    Invalid,
    /// Estimated 60 s or more: computed only when called again with `"confirm": true`.
    NeedsConfirmation,
    Done,
    Cancelled,
    /// Computing failed (message in `errors`).
    Failed,
}

/// Answer of [`evaluate`].
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateOut {
    pub status: Status,
    pub errors: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate: Option<Estimate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ExperimentResult>,
}

/// Estimate before the start.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    /// Variants of the search (pattern search: expected evaluations).
    pub variants: usize,
    /// Runs of search and confirmation.
    pub runs: u64,
    /// Estimated seconds (`None` with `confirm`: not measured).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub us_per_run: Option<f64>,
    pub cores: usize,
}

/// A value of the variant: field path and value.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Setting {
    pub path: String,
    pub value: Value,
}

/// A metric of a variant (compact: without its definition).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Figure {
    pub name: String,
    pub value: Option<f64>,
    /// Standard error of a probability or a mean.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub std_error: Option<f64>,
}

/// Values of a variant on one set of paths.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evaluation {
    /// Value of the objective (`None` without an objective).
    pub objective: Option<f64>,
    /// Constraints met (in the search with the margin, in the confirmation without).
    pub feasible: bool,
    pub figures: Vec<Figure>,
    /// With the objective `score`: the metrics of the optimization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization: Option<optimize::Metrics>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ranked {
    pub rank: usize,
    pub settings: Vec<Setting>,
    /// On the search paths.
    pub search: Evaluation,
    /// On the independent confirmation paths (`None` without an objective).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<Evaluation>,
}

/// Paths and seeds of both stages.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsInfo {
    pub source: String,
    pub search: usize,
    pub confirm: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm_seed: Option<u64>,
    /// The confirmation runs on other paths than the search.
    pub independent: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentResult {
    pub method: Method,
    /// Variants evaluated in the search.
    pub variants: usize,
    /// The best `top` variants, best first (with an objective).
    pub ranking: Vec<Ranked>,
    /// Without an objective: the variants in their order (at most [`MAX_LISTED`]).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub listed: Vec<Ranked>,
    #[serde(skip_serializing_if = "is_zero")]
    pub omitted: usize,
    /// Hints, e.g. the best value at the edge of the grid.
    pub hints: Vec<Message>,
    pub paths: PathsInfo,
    /// The complete experiment with all defaults.
    pub experiment: Experiment,
    /// Version of the engine.
    pub version: String,
    pub elapsed_ms: f64,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

// ---------------------------------------------------------------------------------------------
// Check against the schema: path, reason, valid values
// ---------------------------------------------------------------------------------------------

fn resolve(node: &Value) -> &Value {
    match node.get("$ref").and_then(Value::as_str) {
        Some(r) => r
            .strip_prefix("#/definitions/")
            .and_then(|name| schema()["definitions"].get(name))
            .unwrap_or(&Value::Null),
        None => node,
    }
}

fn alternatives(node: &Value) -> Vec<&Value> {
    let node = resolve(node);
    let mut out = vec![node];
    for key in ["allOf", "anyOf", "oneOf"] {
        for sub in node.get(key).and_then(Value::as_array).into_iter().flatten() {
            out.extend(alternatives(sub));
        }
    }
    out
}

fn type_fits(t: &str, v: &Value) -> bool {
    match (t, v) {
        ("null", Value::Null) | ("boolean", Value::Bool(_)) | ("number", Value::Number(_)) => true,
        ("integer", Value::Number(n)) => n.is_u64() || n.is_i64(),
        ("string", Value::String(_)) | ("array", Value::Array(_)) | ("object", Value::Object(_)) => true,
        _ => false,
    }
}

/// A value shortened for a message.
fn short(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() > 40 {
        format!("{}…", s.chars().take(40).collect::<String>())
    } else {
        s
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

/// Checks `v` against the schema nodes `nodes` (alternatives): type and fixed values, unknown and missing keys, list
/// elements. All problems go to `out`.
fn check_nodes(nodes: &[&Value], v: &Value, path: &str, out: &mut Vec<Message>) {
    let alts: Vec<&Value> = nodes.iter().flat_map(|n| alternatives(n)).collect();
    let mut expected = BTreeSet::new();
    let mut fits = false;
    let mut typed = false;
    for a in &alts {
        if let Some(values) = a.get("enum").and_then(Value::as_array) {
            typed = true;
            fits |= values.contains(v);
            expected.extend(values.iter().map(|x| x.as_str().map_or(x.to_string(), str::to_string)));
        } else if let Some(t) = a.get("type") {
            typed = true;
            let ts: Vec<&str> = match t {
                Value::String(s) => vec![s.as_str()],
                Value::Array(l) => l.iter().filter_map(Value::as_str).collect(),
                _ => vec![],
            };
            fits |= ts.iter().any(|t| type_fits(t, v));
            expected.extend(ts.iter().map(|t| t.to_string()));
        } else if a.get("allOf").or(a.get("anyOf")).or(a.get("oneOf")).is_none() {
            // No restriction (any value).
            fits = true;
        }
    }
    if typed && !fits {
        out.push(msg!(
            experiment_value_wrong,
            path = path,
            value = short(v),
            expected = arg(&expected.into_iter().collect::<Vec<_>>())
        ));
        return;
    }
    match v {
        Value::Object(o) => {
            let mut props: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
            let mut closed = false;
            let mut with_props = 0;
            for a in &alts {
                if let Some(p) = a.get("properties").and_then(Value::as_object) {
                    with_props += 1;
                    for (k, s) in p {
                        props.entry(k.as_str()).or_default().push(s);
                    }
                }
                closed |= a.get("additionalProperties") == Some(&Value::Bool(false));
            }
            if props.is_empty() {
                return;
            }
            for (k, x) in o {
                match props.get(k.as_str()) {
                    Some(subs) => check_nodes(subs, x, &join(path, k), out),
                    None if closed => out.push(msg!(
                        experiment_key_unknown,
                        path = join(path, k),
                        valid = arg(&props.keys().map(|k| k.to_string()).collect::<Vec<_>>())
                    )),
                    None => {}
                }
            }
            if with_props == 1 {
                for a in &alts {
                    for r in a.get("required").and_then(Value::as_array).into_iter().flatten() {
                        if let Some(r) = r.as_str().filter(|r| !o.contains_key(*r)) {
                            out.push(msg!(experiment_key_missing, path = path, key = r));
                        }
                    }
                }
            }
        }
        Value::Array(items) => {
            let subs: Vec<&Value> = alts.iter().filter_map(|a| a.get("items")).collect();
            for a in &alts {
                let n = items.len() as u64;
                let lo = a.get("minItems").and_then(Value::as_u64);
                let hi = a.get("maxItems").and_then(Value::as_u64);
                if lo.is_some_and(|lo| n < lo) || hi.is_some_and(|hi| n > hi) {
                    out.push(msg!(
                        experiment_items_count,
                        path = path,
                        count = n,
                        min = lo.unwrap_or(0),
                        max = hi.unwrap_or(n)
                    ));
                    return;
                }
            }
            if !subs.is_empty() {
                for (i, x) in items.iter().enumerate() {
                    check_nodes(&subs, x, &format!("{path}[{i}]"), out);
                }
            }
        }
        _ => {}
    }
}

/// Reads the experiment: first against the schema (messages with path), then as structure.
pub fn parse(v: &Value) -> std::result::Result<Experiment, Vec<Message>> {
    let mut out = vec![];
    check_nodes(&[schema()], v, "", &mut out);
    if !out.is_empty() {
        return Err(out);
    }
    serde_json::from_value(v.clone()).map_err(|e| {
        vec![msg!(
            experiment_invalid,
            path = "",
            problem = msg!(raw_error, detail = e.to_string())
        )]
    })
}

// ---------------------------------------------------------------------------------------------
// Space of variations
// ---------------------------------------------------------------------------------------------

/// A coupled value: `path` = value of `follows` + `offset`.
#[derive(Clone, Debug)]
struct Couple {
    path: String,
    follows: String,
    offset: f64,
}

/// A part of the space without further choice: fixed values, ranges (pattern search) and couplings.
#[derive(Clone, Debug, Default)]
struct Branch {
    fixed: Vec<(String, Value)>,
    ranges: Vec<(String, f64, f64)>,
    couples: Vec<Couple>,
    /// Numeric variations of this branch with their values (hint «at the edge of the grid»).
    grid: Vec<(String, Vec<f64>)>,
}

fn cat<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    a.iter().chain(b).cloned().collect()
}

impl Branch {
    fn merged(&self, other: &Branch) -> Branch {
        Branch {
            fixed: cat(&self.fixed, &other.fixed),
            ranges: cat(&self.ranges, &other.ranges),
            couples: cat(&self.couples, &other.couples),
            grid: cat(&self.grid, &other.grid),
        }
    }
}

/// Rounded to 12 decimals (a range in steps without binary noise: 0.1 + 0.2 = 0.3).
fn tidy(x: f64) -> f64 {
    (x * 1e12).round() / 1e12
}

/// Values of a range [from, to, step].
fn steps([from, to, step]: [f64; 3]) -> Vec<f64> {
    let n = ((to - from) / step + 1e-9).floor() as usize;
    (0..=n).map(|i| tidy(from + i as f64 * step)).collect()
}

/// Number of variants of the grid (saturating).
fn count(vary: &[Variation]) -> u128 {
    vary.iter().fold(1u128, |acc, d| {
        let n = if let Some(g) = &d.group {
            g.iter().map(|m| count(&m.vary)).fold(0u128, u128::saturating_add)
        } else if let Some(v) = &d.values {
            v.len() as u128
        } else if let Some(r) = d.range {
            steps(r).len() as u128
        } else {
            1
        };
        acc.saturating_mul(n).min(u128::from(u64::MAX))
    })
}

/// Branches of the space; with `grid` the ranges become values.
fn expand(vary: &[Variation], grid: bool) -> Vec<Branch> {
    let mut out = vec![Branch::default()];
    for d in vary {
        let options: Vec<Branch> = if let Some(g) = &d.group {
            g.iter()
                .flat_map(|m| {
                    let set: Vec<(String, Value)> = m.set.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                    expand(&m.vary, grid).into_iter().map(move |b| Branch {
                        fixed: set.iter().cloned().chain(b.fixed).collect(),
                        ..b
                    })
                })
                .collect()
        } else {
            let path = d.path.clone().unwrap_or_default();
            if let Some(f) = &d.follows {
                vec![Branch {
                    couples: vec![Couple {
                        path,
                        follows: f.clone(),
                        offset: d.offset.unwrap_or(0.0),
                    }],
                    ..Branch::default()
                }]
            } else if let (Some([lo, hi, _]), false) = (d.range, grid) {
                vec![Branch {
                    ranges: vec![(path, lo, hi)],
                    ..Branch::default()
                }]
            } else {
                let values: Vec<Value> = match (&d.values, d.range) {
                    (Some(v), _) => v.clone(),
                    (None, Some(r)) => steps(r).into_iter().map(Value::from).collect(),
                    _ => vec![],
                };
                let numbers: Vec<f64> = values.iter().filter_map(Value::as_f64).collect();
                let grid_dim = (numbers.len() == values.len()).then(|| (path.clone(), numbers));
                values
                    .into_iter()
                    .map(|v| Branch {
                        fixed: vec![(path.clone(), v)],
                        grid: grid_dim.clone().into_iter().collect(),
                        ..Branch::default()
                    })
                    .collect()
            }
        };
        out = out.iter().flat_map(|b| options.iter().map(|o| b.merged(o))).collect();
    }
    out
}

/// Assignments `path=value` for the field paths of #207.
fn assignments(settings: &[(String, Value)]) -> Vec<String> {
    settings.iter().map(|(p, v)| format!("{p}={v}")).collect()
}

/// The values of a variant: fixed values, the point `x` in the ranges, then the coupled values.
fn settings_of(b: &Branch, x: &[f64], set: &BTreeMap<String, Value>) -> Result<Vec<(String, Value)>> {
    let mut out: Vec<(String, Value)> = b.fixed.clone();
    out.extend(b.ranges.iter().zip(x).map(|((p, _, _), v)| (p.clone(), json!(v))));
    for c in &b.couples {
        let base = out
            .iter()
            .rev()
            .find(|(p, _)| *p == c.follows)
            .map(|(_, v)| v)
            .or_else(|| set.get(&c.follows))
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                Error::Invalid(msg!(
                    experiment_follows_number,
                    path = c.path.as_str(),
                    follows = c.follows.as_str()
                ))
            })?;
        out.push((c.path.clone(), json!(tidy(base + c.offset))));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Checks of the content (after the schema)
// ---------------------------------------------------------------------------------------------

/// Field paths set or varied (for couplings).
fn known_paths(set: &BTreeMap<String, Value>, vary: &[Variation], out: &mut BTreeSet<String>) {
    out.extend(set.keys().cloned());
    for d in vary {
        if let Some(p) = &d.path {
            out.insert(p.clone());
        }
        for m in d.group.iter().flatten() {
            known_paths(&m.set, &m.vary, out);
        }
    }
}

/// Checks one value at a field path against the plan (path, type, point in time).
fn check_value(base: &Value, where_: &str, path: &str, value: &Value, out: &mut Vec<Message>) {
    if let Err(e) = crate::fieldpath::set(base, &assignments(&[(path.to_string(), value.clone())])) {
        out.extend(
            e.into_iter()
                .map(|m| msg!(experiment_invalid, path = where_, problem = m)),
        );
    }
}

fn check_vary(base: &Value, vary: &[Variation], at: &str, known: &BTreeSet<String>, out: &mut Vec<Message>) {
    for (i, d) in vary.iter().enumerate() {
        let here = format!("{at}[{i}]");
        let forms = [d.values.is_some(), d.range.is_some(), d.follows.is_some()]
            .iter()
            .filter(|x| **x)
            .count();
        if let Some(g) = &d.group {
            if d.path.is_some() || forms > 0 || d.offset.is_some() {
                out.push(msg!(experiment_variation_form, path = here));
                continue;
            }
            for (j, m) in g.iter().enumerate() {
                let member = format!("{here}.group[{j}]");
                for (p, v) in &m.set {
                    check_value(base, &format!("{member}.set.{p}"), p, v, out);
                }
                check_vary(base, &m.vary, &format!("{member}.vary"), known, out);
            }
            if g.is_empty() {
                out.push(msg!(experiment_values_empty, path = format!("{here}.group")));
            }
            continue;
        }
        let Some(path) = d.path.as_deref().filter(|_| forms == 1) else {
            out.push(msg!(experiment_variation_form, path = here));
            continue;
        };
        if d.offset.is_some() && d.follows.is_none() {
            out.push(msg!(experiment_variation_form, path = here));
            continue;
        }
        if let Some(values) = &d.values {
            if values.is_empty() {
                out.push(msg!(experiment_values_empty, path = format!("{here}.values")));
            }
            for (j, v) in values.iter().enumerate() {
                check_value(base, &format!("{here}.values[{j}]"), path, v, out);
            }
        }
        if let Some(r @ [from, to, step]) = d.range {
            if !(from <= to && step > 0.0 && step.is_finite() && from.is_finite() && to.is_finite()) {
                out.push(msg!(experiment_range_invalid, path = format!("{here}.range")));
            } else {
                for (j, v) in [from, *steps(r).last().unwrap_or(&from)].into_iter().enumerate() {
                    check_value(base, &format!("{here}.range[{j}]"), path, &json!(v), out);
                }
            }
        }
        if let Some(f) = &d.follows {
            if !known.contains(f) || f == path {
                out.push(msg!(
                    experiment_follows_unknown,
                    path = format!("{here}.follows"),
                    follows = f.as_str()
                ));
            }
        }
    }
}

/// Names of the metrics as in the result (`#1`, `#2`, … without a name).
fn metric_names(metrics: &[Metric]) -> Vec<String> {
    metrics
        .iter()
        .enumerate()
        .map(|(i, m)| m.name.clone().unwrap_or_else(|| format!("#{}", i + 1)))
        .collect()
}

fn check_metric_ref(names: &[String], at: &str, name: &str, out: &mut Vec<Message>) {
    if !names.iter().any(|n| n == name) {
        out.push(msg!(
            experiment_metric_unknown,
            path = at,
            metric = name,
            valid = arg(&names.to_vec())
        ));
    }
}

/// Checks the content of the experiment against the plan `base` (with the changes `set` applied).
fn check(exp: &Experiment, base: &Value, out: &mut Vec<Message>) {
    let mut known = BTreeSet::new();
    known_paths(&exp.set, &exp.vary, &mut known);
    check_vary(base, &exp.vary, "vary", &known, out);
    let names = metric_names(&exp.metrics);
    for (i, n) in names.iter().enumerate() {
        if names[..i].contains(n) {
            out.push(msg!(
                experiment_metric_duplicate,
                path = format!("metrics[{i}].name"),
                metric = n.as_str()
            ));
        }
    }
    for (i, c) in exp.constraints.iter().enumerate() {
        check_metric_ref(&names, &format!("constraints[{i}].metric"), &c.metric, out);
    }
    if let Some(o) = &exp.objective {
        let forms = [
            o.maximize.is_some(),
            o.minimize.is_some(),
            o.weighted.is_some(),
            o.score.is_some(),
        ];
        if forms.iter().filter(|x| **x).count() != 1 {
            out.push(msg!(experiment_objective_form, path = "objective"));
        }
        for (k, m) in [("maximize", &o.maximize), ("minimize", &o.minimize)] {
            if let Some(m) = m {
                check_metric_ref(&names, &format!("objective.{k}"), m, out);
            }
        }
        for (i, w) in o.weighted.iter().flatten().enumerate() {
            check_metric_ref(&names, &format!("objective.weighted[{i}].metric"), &w.metric, out);
        }
        if o.weighted.as_ref().is_some_and(Vec::is_empty) {
            out.push(msg!(experiment_values_empty, path = "objective.weighted"));
        }
    } else if exp.search.method == Method::Pattern {
        out.push(msg!(experiment_pattern_objective, path = "objective"));
    }
    let b = &exp.budget;
    for (k, ok) in [
        ("budget.paths", b.paths > 0),
        ("budget.top", b.top > 0),
        ("budget.confirmPaths", b.confirm_paths > 0),
        ("budget.margin", b.margin >= 0.0 && b.margin.is_finite()),
    ] {
        if !ok {
            out.push(msg!(experiment_budget_invalid, path = k));
        }
    }
    if exp.metrics.is_empty() && exp.objective.as_ref().is_none_or(|o| o.score.is_none()) {
        out.push(msg!(experiment_values_empty, path = "metrics"));
    }
}

// ---------------------------------------------------------------------------------------------
// Evaluation of a variant
// ---------------------------------------------------------------------------------------------

/// Raw values of a variant on a set of paths.
struct Values {
    metrics: Vec<MetricValue>,
    optimization: Option<optimize::Metrics>,
    paths: usize,
}

struct Context<'a> {
    exp: &'a Experiment,
    score: Option<Objective>,
    base: Value,
    base_returns: Value,
    base_dir: &'a Path,
    library: Option<&'a Path>,
}

/// Paths and their key for one stage.
struct Stage<'a> {
    paths: Paths<'a>,
    data: Option<&'a DataSet>,
    key: Option<batch::PathKey>,
    start: crate::time::YearMonth,
    months: usize,
    n: usize,
    seed_offset: u64,
}

impl Context<'_> {
    fn plan(&self, settings: &[(String, Value)]) -> Result<Value> {
        crate::fieldpath::set(&self.base, &assignments(settings))
            .map_err(|e| Error::Invalid(msg!(plan_errors, errors = e)))
    }

    /// Runs the variant `v` on the paths of the stage (its own paths if the variant changes what they depend on).
    fn run(&self, v: &Value, stage: &Stage) -> Result<Values> {
        let read = |v: Value| -> Result<Plan> {
            serde_json::from_value(v).map_err(|e| Error::Invalid(msg!(raw_error, detail = e.to_string())))
        };
        let q = read(v.clone())?;
        let sim = Simulator::new(&q)?;
        let same_inputs = serde_json::to_value(&q.returns).ok().as_ref() == Some(&self.base_returns);
        let own_data;
        let data = if same_inputs {
            stage.data
        } else {
            own_data = batch::load_series(&q, self.base_dir, self.library)?;
            own_data.as_ref()
        };
        let mine = batch::path_source(&read(optimize::with_paths(v, stage.n, stage.seed_offset))?, &sim, data)?;
        let reuse = match (mine.key(), &stage.key) {
            (Some(a), Some(b)) => a == *b,
            (None, None) => same_inputs && sim.start == stage.start && sim.months() == stage.months,
            _ => false,
        };
        let paths = if reuse { &stage.paths } else { &mine };
        let metrics = if self.exp.metrics.is_empty() {
            vec![]
        } else {
            Measurement::new(&q, &sim, self.exp.metrics.clone())
                .map_err(|e| Error::Invalid(msg!(plan_errors, errors = e)))?
                .run(&sim, paths)?
        };
        let optimization = match &self.score {
            Some(o) => Some(optimize::evaluate(&q, paths, o)?),
            None => None,
        };
        Ok(Values {
            metrics,
            optimization,
            paths: paths.len(),
        })
    }

    fn value(&self, v: &Values, name: &str) -> Option<f64> {
        v.metrics.iter().find(|m| m.name == name).and_then(|m| m.value)
    }

    /// Violation of the constraints; with a margin of `margin` standard errors (search) or without (confirmation).
    fn violation(&self, v: &Values, margin: f64) -> f64 {
        self.exp
            .constraints
            .iter()
            .map(|c| {
                let Some(m) = v.metrics.iter().find(|m| m.name == c.metric) else {
                    return 0.0;
                };
                let Some(x) = m.value else {
                    return 1.0;
                };
                // Standard error: of a probability at the limit (as the optimization), otherwise as measured.
                let se = if matches!(m.metric.aggregate, Aggregate::Probability { .. }) {
                    let p = c.value.clamp(0.0, 1.0);
                    (p * (1.0 - p) / m.values.max(1) as f64).sqrt()
                } else {
                    m.std_error.unwrap_or(0.0)
                };
                let strict = |d: f64| if d == 0.0 { 1e-12 } else { d.max(0.0) };
                match c.op {
                    Op::Le => (x - (c.value - margin * se)).max(0.0),
                    Op::Lt => strict(x - (c.value - margin * se)),
                    Op::Ge => (c.value + margin * se - x).max(0.0),
                    Op::Gt => strict(c.value + margin * se - x),
                }
            })
            .sum()
    }

    /// Objective value (higher is better; without an objective `None`).
    fn objective(&self, v: &Values) -> Option<f64> {
        let o = self.exp.objective.as_ref()?;
        let get = |n: &str| self.value(v, n).unwrap_or(f64::NEG_INFINITY);
        if let Some(m) = &o.maximize {
            Some(get(m))
        } else if let Some(m) = &o.minimize {
            Some(-self.value(v, m).unwrap_or(f64::INFINITY))
        } else if let Some(ws) = &o.weighted {
            Some(ws.iter().map(|w| w.weight * get(&w.metric)).sum())
        } else {
            match (&self.score, &v.optimization) {
                (Some(s), Some(m)) => Some(m.score(s)),
                _ => None,
            }
        }
    }

    /// Fitness in the search: feasible by objective, infeasible below by violation (as the optimization).
    fn fitness(&self, v: &Values) -> f64 {
        let margin = self.exp.budget.margin;
        let mut viol = self.violation(v, margin);
        if let (Some(o), Some(m)) = (&self.score, &v.optimization) {
            let fit = optimize::fit_objective_with(o, v.paths, margin);
            if viol <= 1e-12 {
                return m.fitness(&fit);
            }
            viol += m.violation(&fit);
        }
        if viol > 1e-12 {
            -1e9 * (1.0 + viol)
        } else {
            self.objective(v).unwrap_or(0.0).max(-1e8)
        }
    }

    /// Ranking after the confirmation: feasible by objective, infeasible below by violation (without margin).
    fn rank(&self, v: &Values) -> f64 {
        let mut viol = self.violation(v, 0.0);
        if let (Some(o), Some(m)) = (&self.score, &v.optimization) {
            viol += m.violation(o);
        }
        if viol > 1e-12 {
            -1e9 * (1.0 + viol)
        } else {
            self.objective(v).unwrap_or(0.0).max(-1e8)
        }
    }

    fn evaluation(&self, v: &Values, margin: f64) -> Evaluation {
        let mut viol = self.violation(v, margin);
        if let (Some(o), Some(m)) = (&self.score, &v.optimization) {
            viol += m.violation(&optimize::fit_objective_with(o, v.paths, margin));
        }
        Evaluation {
            objective: self.objective(v).filter(|x| x.is_finite()),
            feasible: viol <= 1e-12,
            figures: v
                .metrics
                .iter()
                .map(|m| Figure {
                    name: m.name.clone(),
                    value: m.value,
                    std_error: m.std_error,
                })
                .collect(),
            optimization: v.optimization,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Evaluate
// ---------------------------------------------------------------------------------------------

/// Ranking, listed variants, variants not listed, evaluations and hints.
type Outcome = (Vec<Ranked>, Vec<Ranked>, usize, usize, Vec<Message>);

/// A variant found in the search: its values, values on the search paths, fitness and branch.
type Found = (Vec<(String, Value)>, Values, f64, usize);

fn invalid(errors: Vec<Message>) -> EvaluateOut {
    EvaluateOut {
        status: Status::Invalid,
        errors,
        estimate: None,
        result: None,
    }
}

fn seed_of(p: &Plan) -> Option<u64> {
    match p.returns.source {
        ReturnSource::Bootstrap => p.returns.bootstrap.as_ref().map(|b| b.seed),
        ReturnSource::Parametric => p.returns.parametric.as_ref().map(|b| b.seed),
        _ => None,
    }
}

/// Expected evaluations of the pattern search of a branch (starting points plus pattern steps).
fn pattern_evals(b: &Branch, samples: usize) -> usize {
    if b.ranges.is_empty() {
        1
    } else {
        samples + 2 + 12 * b.ranges.len()
    }
}

/// #216: Computes an experiment on the plan `plan`. `base_dir`: folder of the plan, `library`: data library.
pub fn evaluate(plan: &Value, experiment: &Value, base_dir: &Path, library: Option<&Path>) -> EvaluateOut {
    evaluate_as(None, plan, experiment, base_dir, library)
}

/// Like [`evaluate`], as a computation with the id `job` (progress and cancellation).
pub fn evaluate_as(
    job: Option<crate::progress::JobId>,
    plan: &Value,
    experiment: &Value,
    base_dir: &Path,
    library: Option<&Path>,
) -> EvaluateOut {
    let t0 = Instant::now();
    let exp = match parse(experiment) {
        Ok(e) => e,
        Err(e) => return invalid(e),
    };
    // The plan without its own variations, with the changes of the experiment.
    let mut base = plan.clone();
    if let Some(s) = base.get_mut("scenarios").and_then(Value::as_object_mut) {
        s.remove("select");
        s.insert("sweeps".into(), json!([]));
    }
    if let Err(e) = crate::api::parse(&base) {
        return invalid(vec![e.message]);
    }
    let base = match crate::fieldpath::set(&base, &assignments(&exp.set.clone().into_iter().collect::<Vec<_>>())) {
        Ok(b) => b,
        Err(e) => {
            return invalid(
                e.into_iter()
                    .map(|m| msg!(experiment_invalid, path = "set", problem = m))
                    .collect(),
            )
        }
    };
    let mut errors = vec![];
    check(&exp, &base, &mut errors);
    let p: Plan = match serde_json::from_value(base.clone()) {
        Ok(p) => p,
        Err(e) => return invalid(vec![msg!(plan_incomplete_detail, detail = e.to_string())]),
    };
    let sim = match Simulator::new(&p) {
        Ok(s) => s,
        Err(e) => return invalid(vec![e.message()]),
    };
    if !exp.metrics.is_empty() {
        if let Err(e) = Measurement::new(&p, &sim, exp.metrics.clone()) {
            errors.extend(e);
        }
    }
    let grid = exp.search.method == Method::Grid;
    let variants = count(&exp.vary);
    if grid && variants > MAX_VARIANTS as u128 {
        errors.push(msg!(
            experiment_too_many_variants,
            variants = variants.min(i64::MAX as u128) as u64,
            max = MAX_VARIANTS
        ));
    }
    if !errors.is_empty() {
        return invalid(errors);
    }
    let branches = expand(&exp.vary, grid);
    // Every variant of the grid (pattern search: the corners of every branch) must give a valid plan.
    let ctx = Context {
        exp: &exp,
        score: exp
            .objective
            .as_ref()
            .and_then(|o| o.score.as_ref())
            .map(Score::objective),
        base_returns: serde_json::to_value(&p.returns).unwrap_or(Value::Null),
        base,
        base_dir,
        library,
    };
    let mut plans = vec![];
    for (i, b) in branches.iter().enumerate() {
        let corners: Vec<Vec<f64>> = if b.ranges.is_empty() {
            vec![vec![]]
        } else {
            vec![
                b.ranges.iter().map(|r| r.1).collect(),
                b.ranges.iter().map(|r| r.2).collect(),
            ]
        };
        for x in corners {
            let plan = settings_of(b, &x, &exp.set).and_then(|s| ctx.plan(&s)).and_then(|v| {
                let q: Plan = serde_json::from_value(v.clone())
                    .map_err(|e| Error::Invalid(msg!(raw_error, detail = e.to_string())))?;
                Simulator::new(&q)?;
                Ok(v)
            });
            match plan {
                Ok(v) => plans.push(v),
                Err(e) => errors.push(msg!(experiment_variant_invalid, variant = i + 1, problem = e.message())),
            }
            if errors.len() >= 20 {
                return invalid(errors);
            }
        }
    }
    if !errors.is_empty() {
        return invalid(errors);
    }

    // Paths of both stages.
    let data = match batch::load_series(&p, base_dir, library) {
        Ok(d) => d,
        Err(e) => return invalid(vec![e.message()]),
    };
    let stage_plan = |n: usize, offset: u64| -> Result<Plan> {
        serde_json::from_value(optimize::with_paths(&ctx.base, n, offset))
            .map_err(|e| Error::Invalid(msg!(raw_error, detail = e.to_string())))
    };
    let (p_fit, p_check) = match (
        stage_plan(exp.budget.paths, 0),
        stage_plan(exp.budget.confirm_paths, optimize::CHECK_SEED_OFFSET),
    ) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => return invalid(vec![e.message()]),
    };
    let sources = batch::path_source(&p_fit, &sim, data.as_ref())
        .and_then(|f| batch::path_source(&p_check, &sim, data.as_ref()).map(|c| (f, c)));
    let (fit, check_src) = match sources {
        Ok(x) => x,
        Err(e) => return invalid(vec![e.message()]),
    };
    let monte_carlo = matches!(p.returns.source, ReturnSource::Bootstrap | ReturnSource::Parametric);
    let max_paths = crate::validate::MAX_PATH_MONTHS / sim.months().max(1) as u64;
    if monte_carlo {
        if let Some(&n) = [exp.budget.paths, exp.budget.confirm_paths]
            .iter()
            .find(|&&n| n as u64 > max_paths)
        {
            return invalid(vec![msg!(
                paths_memory,
                paths = n as u64,
                months = sim.months(),
                max = max_paths
            )]);
        }
    }

    // Estimate.
    let has_objective = exp.objective.is_some();
    let passes = (u64::from(!exp.metrics.is_empty()) + u64::from(ctx.score.is_some())).max(1);
    let search_variants: usize = if grid {
        branches.len()
    } else {
        branches.iter().map(|b| pattern_evals(b, exp.search.samples)).sum()
    };
    let confirmed = if has_objective {
        exp.budget.top.min(branches.len())
    } else {
        0
    };
    let runs = (search_variants as u64)
        .saturating_mul(fit.len() as u64)
        .saturating_add(confirmed as u64 * check_src.len() as u64)
        .saturating_mul(passes);
    if runs > optimize::MAX_RUNS {
        return invalid(vec![msg!(
            experiment_too_many_runs,
            runs = runs.min(i64::MAX as u64),
            max = optimize::MAX_RUNS
        )]);
    }
    let mut estimate = Estimate {
        variants: search_variants,
        runs,
        seconds: None,
        us_per_run: None,
        cores: rayon::current_num_threads(),
    };
    if !exp.confirm {
        match crate::api::bench(&ctx.base, base_dir, library) {
            Ok(b) => {
                let generated = if monte_carlo {
                    (fit.len() + if confirmed > 0 { check_src.len() } else { 0 }) as f64
                } else {
                    0.0
                };
                let seconds = estimate_seconds(runs, generated, b.us_per_run, b.us_generate);
                estimate.seconds = Some(seconds);
                estimate.us_per_run = Some(b.us_per_run);
                if seconds >= CONFIRM_SECONDS {
                    return EvaluateOut {
                        status: Status::NeedsConfirmation,
                        errors: vec![msg!(experiment_confirm, seconds = seconds, runs = runs)],
                        estimate: Some(estimate),
                        result: None,
                    };
                }
            }
            Err(e) => return invalid(vec![e]),
        }
    }

    let _job = crate::progress::start_as(job, runs);
    let mut budget = batch::PATH_CACHE_BYTES;
    let fit_stage = Stage {
        key: fit.key(),
        n: exp.budget.paths,
        seed_offset: 0,
        start: sim.start,
        months: sim.months(),
        data: data.as_ref(),
        paths: batch::prepare(fit, &mut budget),
    };
    let outcome = (|| -> Result<Outcome> {
        // Stage 1: search.
        let mut found: Vec<Found> = vec![];
        let mut evaluations = 0usize;
        let mut hints = vec![];
        for (bi, b) in branches.iter().enumerate() {
            if grid || b.ranges.is_empty() {
                let s = settings_of(b, &[], &exp.set)?;
                let v = ctx.run(&ctx.plan(&s)?, &fit_stage)?;
                evaluations += 1;
                let f = ctx.fitness(&v);
                found.push((s, v, f, bi));
            } else {
                // Starting points as the optimization: the plan's value, the middle, random points.
                let bounds: Vec<(f64, f64)> = b.ranges.iter().map(|r| (r.1, r.2)).collect();
                let start_plan = ctx.plan(&b.fixed)?;
                let mid = |&(lo, hi): &(f64, f64)| (lo + hi) / 2.0;
                let mut starts = vec![b
                    .ranges
                    .iter()
                    .zip(&bounds)
                    .map(|((path, lo, hi), bd)| {
                        current(&start_plan, path)
                            .filter(|v| (*lo..=*hi).contains(v))
                            .unwrap_or(mid(bd))
                    })
                    .collect::<Vec<f64>>()];
                starts.push(bounds.iter().map(mid).collect());
                let mut rng = optimize::Rng(exp.search.seed ^ optimize::START_SALT);
                for _ in 0..exp.search.samples {
                    starts.push(bounds.iter().map(|(lo, hi)| lo + (hi - lo) * rng.next()).collect());
                }
                let (x, (s, v), f) = optimize::search(starts, &bounds, |x| {
                    evaluations += 1;
                    let s = settings_of(b, x, &exp.set)?;
                    let v = ctx.run(&ctx.plan(&s)?, &fit_stage)?;
                    let f = ctx.fitness(&v);
                    Ok(((s, v), f))
                })?;
                for ((path, lo, hi), x) in b.ranges.iter().zip(&x) {
                    if (x - lo).abs() <= 1e-9 * (hi - lo).abs().max(1.0) {
                        hints.push(msg!(experiment_edge_low, path = path.as_str(), value = *x));
                    } else if (x - hi).abs() <= 1e-9 * (hi - lo).abs().max(1.0) {
                        hints.push(msg!(experiment_edge_high, path = path.as_str(), value = *x));
                    }
                }
                found.push((s, v, f, bi));
            }
        }
        let shown = |s: &[(String, Value)]| -> Vec<Setting> {
            s.iter()
                .map(|(p, v)| Setting {
                    path: p.clone(),
                    value: v.clone(),
                })
                .collect()
        };
        if !has_objective {
            let total = found.len();
            let listed = found
                .iter()
                .take(MAX_LISTED)
                .enumerate()
                .map(|(i, (s, v, _, _))| Ranked {
                    rank: i + 1,
                    settings: shown(s),
                    search: ctx.evaluation(v, exp.budget.margin),
                    confirm: None,
                })
                .collect();
            return Ok((vec![], listed, total.saturating_sub(MAX_LISTED), evaluations, hints));
        }
        // Stage 2: the best `top` on independent paths.
        found.sort_by(|a, b| b.2.total_cmp(&a.2));
        found.truncate(exp.budget.top);
        let mut budget = budget;
        let check_stage = Stage {
            key: check_src.key(),
            n: exp.budget.confirm_paths,
            seed_offset: optimize::CHECK_SEED_OFFSET,
            start: sim.start,
            months: sim.months(),
            data: data.as_ref(),
            paths: batch::prepare(check_src, &mut budget),
        };
        let mut ranked = vec![];
        for (s, v, _, bi) in found {
            let c = ctx.run(&ctx.plan(&s)?, &check_stage)?;
            ranked.push((ctx.rank(&c), s, v, c, bi));
        }
        ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
        if let Some((_, s, _, _, bi)) = ranked.first() {
            for (path, values) in &branches[*bi].grid {
                let chosen = s.iter().rev().find(|(p, _)| p == path).and_then(|(_, v)| v.as_f64());
                let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                match chosen {
                    Some(x) if values.len() >= 3 && x == lo => {
                        hints.push(msg!(experiment_edge_low, path = path.as_str(), value = x))
                    }
                    Some(x) if values.len() >= 3 && x == hi => {
                        hints.push(msg!(experiment_edge_high, path = path.as_str(), value = x))
                    }
                    _ => {}
                }
            }
        }
        let ranking = ranked
            .into_iter()
            .enumerate()
            .map(|(i, (_, s, v, c, _))| Ranked {
                rank: i + 1,
                settings: shown(&s),
                search: ctx.evaluation(&v, exp.budget.margin),
                confirm: Some(ctx.evaluation(&c, 0.0)),
            })
            .collect();
        Ok((ranking, vec![], 0, evaluations, hints))
    })();
    let (ranking, listed, omitted, evaluated, mut hints) = match outcome {
        Ok(x) => x,
        Err(Error::Cancelled) => {
            return EvaluateOut {
                status: Status::Cancelled,
                errors: vec![msg!(cancelled)],
                estimate: Some(estimate),
                result: None,
            }
        }
        Err(e) => {
            return EvaluateOut {
                status: Status::Failed,
                errors: vec![e.message()],
                estimate: Some(estimate),
                result: None,
            }
        }
    };
    let independent = monte_carlo;
    if has_objective && !independent {
        hints.push(msg!(experiment_confirm_same_paths));
    }
    EvaluateOut {
        status: Status::Done,
        errors: vec![],
        result: Some(ExperimentResult {
            method: exp.search.method,
            variants: evaluated,
            ranking,
            listed,
            omitted,
            hints,
            paths: PathsInfo {
                source: format!("{:?}", p.returns.source).to_lowercase(),
                search: fit_stage.paths.len(),
                confirm: if has_objective {
                    check_src_len(&p_check, &sim, data.as_ref())
                } else {
                    0
                },
                search_seed: seed_of(&p_fit),
                confirm_seed: if has_objective { seed_of(&p_check) } else { None },
                independent: independent && has_objective,
            },
            experiment: exp.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            elapsed_ms: t0.elapsed().as_secs_f64() * 1e3,
        }),
        estimate: Some(estimate),
    }
}

fn check_src_len(p: &Plan, sim: &Simulator, data: Option<&DataSet>) -> usize {
    batch::path_source(p, sim, data).map_or(0, |s| s.len())
}

/// Current number at a field path without a point in time (`None` otherwise: the middle of the range is the start).
fn current(plan: &Value, path: &str) -> Option<f64> {
    if path.contains('@') {
        return None;
    }
    plan.pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #311: A run without generating its path counts at least half the measured time, also if the measurement of
    /// the generation under load comes out as large as the whole run; otherwise the measured difference counts.
    #[test]
    fn issue_311_estimate_keeps_half_the_run_time_under_load() {
        let quiet = estimate_seconds(1_000_000, 50.0, 100.0, 20.0);
        assert!((quiet - (80.0 + 0.001)).abs() < 1e-9, "{quiet}");
        for us_generate in [60.0, 100.0, 150.0] {
            let loaded = estimate_seconds(1_000_000, 0.0, 100.0, us_generate);
            assert!((loaded - 50.0).abs() < 1e-9, "{us_generate}: {loaded}");
        }
        // 19 million runs at 10 µs stay above the confirmation even with a generation measured too large.
        assert!(estimate_seconds(19_000_000, 1.0, 10.0, 12.0) >= CONFIRM_SECONDS);
    }
}
