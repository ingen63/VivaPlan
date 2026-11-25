//! Scenario variations: vary single parameters of the plan systematically and compute every combination
//! with all return paths.

use serde::Serialize;
use serde_json::{json, Value};

use crate::time::YearMonth;
use crate::{Error, Result};

/// Technical upper limit of the combinations per simulation (memory protection). Below it there is no fixed limit;
/// the user interface warns about long computing times.
pub const MAX_COMBINATIONS: usize = 1_000_000;

/// A varied parameter with its values (number or month `YYYY-MM`).
#[derive(Clone, Debug, PartialEq)]
pub struct Sweep {
    pub param: String,
    pub values: Vec<Value>,
}

const PARAMS: [&str; 6] = [
    "wealth.stockShare",
    "withdrawal.rate",
    "pension.pk.lumpsum",
    "returns.startDate",
    "milestone.retire",
    "flows.spendingFactor",
];

fn is_month(param: &str) -> bool {
    matches!(param, "returns.startDate" | "milestone.retire")
}

/// Reads `scenarios.sweeps` (`{param, from, to, step}`; for months `step` in months).
pub fn parse(sweeps: &[Value]) -> Result<Vec<Sweep>> {
    let bad = |m: crate::message::Message| Error::Invalid(msg!(scenario_problem, problem = m));
    sweeps
        .iter()
        .map(|s| {
            let param = s
                .get("param")
                .and_then(Value::as_str)
                .ok_or_else(|| bad(msg!(scenario_param_missing)))?
                .to_string();
            if !PARAMS.contains(&param.as_str()) {
                return Err(bad(msg!(scenario_param_unknown, param = param)));
            }
            let step = s.get("step").and_then(Value::as_f64).unwrap_or(0.0);
            if step <= 0.0 {
                return Err(bad(msg!(scenario_step, param = param)));
            }
            // One value more than allowed, so that `count_combinations` reports the excess.
            let cap = MAX_COMBINATIONS + 1;
            let values: Vec<Value> = if is_month(&param) {
                let ym = |k: &str| -> Result<YearMonth> {
                    let raw = s
                        .get(k)
                        .and_then(Value::as_str)
                        .ok_or_else(|| bad(msg!(scenario_value_missing, param = param, key = k)))?;
                    raw.parse::<YearMonth>()
                        .map_err(|_| bad(msg!(scenario_month_invalid, param = param, value = raw)))
                };
                let (a, b) = (ym("from")?, ym("to")?);
                let step = step.round().max(1.0) as i32;
                (0..)
                    .map(|i| a.plus(i * step))
                    .take_while(|m| *m <= b)
                    .take(cap)
                    .map(|m| json!(m.iso()))
                    .collect()
            } else {
                let num = |k: &str| {
                    s.get(k)
                        .and_then(Value::as_f64)
                        .ok_or_else(|| bad(msg!(scenario_value_missing, param = param, key = k)))
                };
                let (a, b) = (num("from")?, num("to")?);
                (0..)
                    .map(|i| a + i as f64 * step)
                    .take_while(|v| *v <= b + step * 1e-9)
                    .take(cap)
                    // Rounding, so that 0.1 steps appear as 0.3, 0.4 …
                    .map(|v| json!((v * 1e9).round() / 1e9))
                    .collect()
            };
            if values.is_empty() {
                return Err(bad(msg!(scenario_to_before_from, param = param)));
            }
            Ok(Sweep { param, values })
        })
        .collect()
}

/// Number of combinations, without creating them; more than [`MAX_COMBINATIONS`] is an error.
pub fn count_combinations(sweeps: &[Sweep]) -> Result<usize> {
    let mut n: usize = 1;
    for s in sweeps {
        n = n.saturating_mul(s.values.len());
    }
    if n > MAX_COMBINATIONS {
        return Err(Error::Invalid(if n == usize::MAX {
            msg!(scenario_too_many_overflow, max = MAX_COMBINATIONS)
        } else {
            msg!(scenario_too_many, count = n, max = MAX_COMBINATIONS)
        }));
    }
    Ok(n)
}

/// All combinations (Cartesian product), each as a list of values in the order of the variations.
pub fn combinations(sweeps: &[Sweep]) -> Result<Vec<Vec<Value>>> {
    count_combinations(sweeps)?;
    let mut out = vec![vec![]];
    for s in sweeps {
        out = out
            .into_iter()
            .flat_map(|c| {
                s.values.iter().map(move |v| {
                    let mut c = c.clone();
                    c.push(v.clone());
                    c
                })
            })
            .collect();
    }
    Ok(out)
}

/// Sets a varied value in the plan (JSON).
pub fn apply(plan: &mut Value, param: &str, value: &Value) -> Result<()> {
    match param {
        "wealth.stockShare" => plan["wealth"]["stockShare"] = value.clone(),
        "pension.pk.lumpsum" => plan["pension"]["pk"]["lumpsum"] = value.clone(),
        "returns.startDate" => plan["returns"]["startDate"] = value.clone(),
        // Withdrawal rate of the fixed rate or initial rate of the guardrails
        "withdrawal.rate" => {
            plan["withdrawal"]["fixedRate"] = json!({ "rate": value });
            if plan["withdrawal"]["guardrails"].is_object() {
                plan["withdrawal"]["guardrails"]["initialRate"] = value.clone();
            }
        }
        "milestone.retire" => {
            let ms = plan["milestones"]
                .as_array_mut()
                .and_then(|a| a.iter_mut().find(|m| m["id"] == "retire"))
                .ok_or_else(|| Error::Invalid(msg!(scenario_retire_missing, param = param)))?;
            let name = ms["name"].clone();
            *ms = json!({ "id": "retire", "name": name, "kind": "date", "date": value });
        }
        "flows.spendingFactor" => {
            let f = value
                .as_f64()
                .ok_or_else(|| Error::Invalid(msg!(scenario_factor_missing, param = param)))?;
            // #53: The factor applies to the living costs without housing (for older plans the same as before).
            plan["flows"]["spendingFactor"] = json!(f);
        }
        _ => return Err(Error::Invalid(msg!(scenario_param_unknown, param = param))),
    }
    Ok(())
}

/// Metrics of a scenario.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioResult {
    /// Values of the variations (order as in `params`).
    pub values: Vec<Value>,
    pub runs: usize,
    pub success_rate: f64,
    /// Total wealth at the end: 5th, 50th and 95th percentile.
    pub p5: f64,
    pub p50: f64,
    pub p95: f64,
    /// Median end wealth in prices of the start month: the run at the 50th percentile of the real order (#265).
    pub p50_real: f64,
    /// Median total wealth at retirement.
    pub at_retirement: Option<f64>,
    pub earliest_depletion: Option<YearMonth>,
    /// End wealth and depletion of the runs at the percentiles (0 = worst run); basis for the optimum per percentile.
    pub percentiles: Vec<crate::batch::Percentile>,
    /// #265: The runs at the percentiles ordered by the end wealth in today's purchasing power, with their real end
    /// wealth. Missing in the JSON when empty (error, results before #265).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub percentiles_real: Vec<crate::batch::Percentile>,
    /// Error if the scenario could not be computed (e.g. start month outside the series).
    pub error: Option<crate::message::Message>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_values_and_combinations() {
        let s = parse(&[
            json!({"param": "wealth.stockShare", "from": 0.3, "to": 1.0, "step": 0.1}),
            json!({"param": "milestone.retire", "from": "2026-09", "to": "2027-03", "step": 3}),
        ])
        .unwrap();
        assert_eq!(s[0].values.len(), 8);
        assert_eq!(s[0].values[1], json!(0.4));
        assert_eq!(s[1].values, vec![json!("2026-09"), json!("2026-12"), json!("2027-03")]);
        let c = combinations(&s).unwrap();
        assert_eq!(c.len(), 24);
        assert_eq!(c[1], vec![json!(0.3), json!("2026-12")]);
        assert!(parse(&[json!({"param": "x", "from": 0, "to": 1, "step": 1})]).is_err());
    }

    #[test]
    fn combinations_are_limited_only_technically_to_one_million() {
        let sweep =
            |param: &str, from: f64, to: f64, step: f64| json!({"param": param, "from": from, "to": to, "step": step});
        // 11 × 10 × 10 = 1100: rejected earlier, allowed now
        let many = parse(&[
            sweep("wealth.stockShare", 0.0, 1.0, 0.1),
            sweep("pension.pk.lumpsum", 0.0, 0.9, 0.1),
            sweep("flows.spendingFactor", 1.0, 1.9, 0.1),
        ])
        .unwrap();
        assert_eq!(count_combinations(&many).unwrap(), 1100);
        assert_eq!(combinations(&many).unwrap().len(), 1100);
        // 101 × 100 × 100 = 1'010'000: above the technical limit, without creating the combinations
        let too_many = parse(&[
            sweep("wealth.stockShare", 0.0, 1.0, 0.01),
            sweep("pension.pk.lumpsum", 0.0, 0.99, 0.01),
            sweep("flows.spendingFactor", 1.0, 1.99, 0.01),
        ])
        .unwrap();
        let err = count_combinations(&too_many).unwrap_err().to_string();
        assert_eq!(
            err,
            "The variations result in 1'010'000 combinations; at most 1'000'000 are technically possible."
        );
        assert!(combinations(&too_many).is_err());
        // exactly 1'000'000 are allowed
        assert_eq!(crate::message::group(1_000_000.0), "1'000'000");
        assert_eq!(crate::message::group(999.0), "999");
    }

    #[test]
    fn apply_scales_spending_and_moves_retirement() {
        let mut p = json!({
            "milestones": [{"id": "retire", "name": "Pensionierung", "kind": "age", "person": "A", "years": 64}],
            "flows": {"spending": {"mode": "timed", "rows": [{"from": "start", "value": 1000}, {"from": "retire", "value": 800}]}}
        });
        // #53: The factor is a field of its own (applies to the living costs without housing); the series stays.
        apply(&mut p, "flows.spendingFactor", &json!(1.1)).unwrap();
        assert_eq!(p["flows"]["spendingFactor"], 1.1);
        assert_eq!(p["flows"]["spending"]["rows"][1]["value"], 800);
        apply(&mut p, "milestone.retire", &json!("2027-01")).unwrap();
        assert_eq!(p["milestones"][0]["date"], "2027-01");
        assert_eq!(p["milestones"][0]["kind"], "date");
    }

    // #40: Start month and withdrawal rate (fixed rate and initial rate of the guardrails).
    #[test]
    fn apply_start_date_and_withdrawal_rate() {
        let mut p = json!({"returns": {}, "withdrawal": {"guardrails": {"initialRate": 0.04}}});
        apply(&mut p, "returns.startDate", &json!("1990-01")).unwrap();
        assert_eq!(p["returns"]["startDate"], "1990-01");
        apply(&mut p, "withdrawal.rate", &json!(0.035)).unwrap();
        assert_eq!(p["withdrawal"]["fixedRate"], json!({"rate": 0.035}));
        assert_eq!(p["withdrawal"]["guardrails"]["initialRate"], json!(0.035));
        let mut p = json!({"withdrawal": {}});
        apply(&mut p, "withdrawal.rate", &json!(0.03)).unwrap();
        assert!(p["withdrawal"].get("guardrails").is_none());
        assert!(apply(&mut p, "x", &json!(1)).is_err());
    }

    // #40: Exactly 1'000'000 values are allowed, one value more is not; longer ranges are capped when created.
    // slow: creates a sweep with one million values
    #[test]
    fn one_million_values_is_the_limit() {
        let sweep =
            |to: f64| parse(&[json!({"param": "wealth.stockShare", "from": 1.0, "to": to, "step": 1.0})]).unwrap();
        assert_eq!(count_combinations(&sweep(1_000_000.0)).unwrap(), 1_000_000);
        assert!(count_combinations(&sweep(1_000_001.0)).is_err());
        let capped = sweep(5_000_000.0);
        assert_eq!(capped[0].values.len(), 1_000_001);
        assert!(count_combinations(&capped).is_err());
    }
}
