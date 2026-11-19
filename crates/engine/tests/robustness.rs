//! Robustness: randomly corrupted plans must not crash the calculation engine.
//! Validation messages and error texts are allowed, but not crashes or invalid numbers.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use engine::api;
use engine::montecarlo::Rng;
use serde_json::{json, Value};

const MUTANTS: u64 = 600;

fn leaves(v: &Value, path: String, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                leaves(x, format!("{path}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                leaves(x, format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
    out.push(path);
}

/// Corrupts 1 to 4 places of the plan. `destructive`: any places, also type changes and deleted fields
/// (checks reading and the input validation). Otherwise only numbers by extreme values (checks the computation itself, #81).
fn mutate(rng: &mut Rng, plan: &mut Value, destructive: bool) {
    let mut paths = vec![];
    leaves(plan, String::new(), &mut paths);
    if !destructive {
        paths.retain(|p| plan.pointer(p).is_some_and(Value::is_number));
    }
    for _ in 0..1 + rng.below(4) {
        let p = &paths[rng.below(paths.len())];
        if p.is_empty() {
            continue;
        }
        let current = plan.pointer(p).cloned().unwrap_or(Value::Null);
        let numbers = [
            json!(0),
            json!(-1),
            json!(0.5),
            json!(1e15),
            json!(-1e15),
            json!(1e-300),
            json!(1e9),
            json!(-0.99),
        ];
        let strings = [
            json!("never"),
            json!("unbekannt"),
            json!("2026-13"),
            json!("1800-01"),
            json!("2300-01"),
            json!("start"),
            json!("end"),
            json!("afterSale"),
            json!("afterSale:xyz"),
            json!(""),
        ];
        let kind = if destructive { rng.below(10) } else { rng.below(7) };
        let replacement = match (kind, &current) {
            // mostly replace by the same type, so that the plan stays readable
            (0..=6, Value::Number(_)) => numbers[rng.below(numbers.len())].clone(),
            (0..=6, Value::String(_)) => strings[rng.below(strings.len())].clone(),
            (0..=6, Value::Array(a)) if !a.is_empty() => json!([]),
            (7, _) => [json!(null), json!([]), json!({}), json!(1), json!("x")][rng.below(5)].clone(),
            (8, _) | (9, _) => {
                // delete the field
                let (parent, key) = p.rsplit_once('/').unwrap();
                match plan.pointer_mut(parent) {
                    Some(Value::Object(o)) => {
                        o.remove(key);
                    }
                    Some(Value::Array(a)) => {
                        if let Ok(i) = key.parse::<usize>() {
                            if i < a.len() {
                                a.remove(i);
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }
            _ => continue,
        };
        if let Some(x) = plan.pointer_mut(p) {
            *x = replacement;
        }
    }
}

/// serde_json writes NaN as null; hence check the numbers directly (#129/AK4: also the real end wealth).
fn finite_result(r: &engine::sim::SimResult) -> bool {
    let s = &r.summary;
    [
        s.end_total,
        s.end_total_real,
        s.end_liquid,
        s.min_liquid.min(0.0),
        s.pension_fund_monthly,
        s.lumpsum_net,
        s.lumpsum_tax,
        s.taxes_total,
    ]
    .iter()
    .chain(s.total_at_retirement.iter())
    .all(|x| x.is_finite())
        && r.records.iter().all(|x| {
            [
                x.cash,
                x.stocks,
                x.bonds,
                x.liquid,
                x.property_equity,
                x.pension_fund,
                x.pillar3a,
                x.total,
                x.income,
                x.spending,
                x.taxes,
            ]
            .iter()
            .all(|v| v.is_finite())
        })
}

fn finite_run(out: &api::RunOut) -> bool {
    out.single.as_ref().is_none_or(finite_result)
        && out.batch.as_ref().is_none_or(|b| {
            b.details.iter().all(|(_, r)| finite_result(r))
                && b.summary
                    .percentiles
                    .iter()
                    .all(|p| p.end_total.is_finite() && p.end_total_real.is_some_and(f64::is_finite))
                && b.bands
                    .p50
                    .iter()
                    .chain(&b.bands.p5)
                    .chain(&b.bands.p95)
                    .all(|v| v.is_finite())
        })
        && out.scenarios.as_ref().is_none_or(|s| {
            s.results
                .iter()
                .all(|r| r.p5.is_finite() && r.p50.is_finite() && r.p95.is_finite())
        })
}

#[test]
fn mutated_plans_never_crash() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let mut base = api::example_plan();
    base["returns"]["bootstrap"] = json!({ "paths": 20, "block": 12, "seed": 1 });
    base["returns"]["parametric"] = json!({ "paths": 20, "mu": 0.07, "sigma": 0.15, "df": 6 });
    base["scenarios"]["sweeps"] = json!([{ "param": "wealth.stockShare", "from": 0.4, "to": 0.6, "step": 0.1 }]);
    let mut rng = Rng::new(99, 0);
    let sources = ["fixed", "historical", "rolling", "bootstrap", "parametric"];
    // computed without error: [only numbers corrupted, corrupted arbitrarily]
    let mut ran = [0u64; 2];
    for n in 0..MUTANTS {
        let mut plan = base.clone();
        plan["returns"]["source"] = json!(sources[rng.below(sources.len())]);
        mutate(&mut rng, &mut plan, n % 2 == 1);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let issues = api::validate(&plan);
            let preview = api::preview(&plan, Path::new("/"), Some(&lib));
            let out = api::run(&plan, Path::new("/"), Some(&lib));
            (issues, preview, out)
        }));
        let Ok((_, preview, out)) = outcome else {
            panic!("Absturz bei Mutation {n}: {}", serde_json::to_string(&plan).unwrap());
        };
        if out.error.is_none() {
            ran[(n % 2) as usize] += 1;
        } else if std::env::var("ROBUST_DEBUG").is_ok() {
            eprintln!(
                "FEHLER {}",
                out.error.as_deref().unwrap_or("").lines().next().unwrap_or("")
            );
        }
        assert!(
            preview.result.as_ref().is_none_or(finite_result) && finite_run(&out),
            "ungültige Zahl bei Mutation {n}: {}",
            serde_json::to_string(&plan).unwrap()
        );
    }
    eprintln!("gerechnet: {ran:?} von je {}", MUTANTS / 2);
    // #81/AK2: Minimum shares, measured on 14.11.2026 (after #98): 97 and 69 of 300 each.
    // Only numbers corrupted: the input validation often rejects extreme values (intended); a quarter computed means that
    // the computation goes through at least 75 plans with extreme values. If the share falls below that, the test mainly
    // checks the input validation and should choose other extreme values.
    // Corrupted arbitrarily: this half mainly checks reading and the input validation; a tenth must still compute,
    // so that plans with wrong references and missing fields also reach the simulation.
    let half = MUTANTS / 2;
    assert!(
        ran[0] * 4 >= half,
        "nur {} von {half} Mutationen mit Extremwerten gerechnet",
        ran[0]
    );
    assert!(
        ran[1] * 10 >= half,
        "nur {} von {half} beliebigen Mutationen gerechnet",
        ran[1]
    );
}
