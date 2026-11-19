//! #86: fuzz target for the whole calculation. The input changes the example plan in a structured way (four bytes each:
//! two for the field, one for the kind of change, one for the value) and calls `api::run`. Crashes and invalid numbers in the
//! result are findings. To keep a run fast, Monte Carlo calculates at most 255 paths and there are no sweeps.
#![no_main]
use std::path::Path;

use engine::api;
use libfuzzer_sys::fuzz_target;
use serde_json::{json, Value};

const LIBRARY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../crates/engine/data");

/// Paths of all leaves and nodes of the plan (JSON pointers).
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
    if !path.is_empty() {
        out.push(path);
    }
}

fn base() -> Value {
    let mut p = api::example_plan();
    p["returns"]["bootstrap"]["paths"] = json!(20);
    p["returns"]["parametric"]["paths"] = json!(20);
    p["scenarios"]["sweeps"] = json!([]);
    p
}

/// Replaces or removes a field.
fn mutate(plan: &mut Value, field: u16, kind: u8, value: u8) {
    let mut paths = vec![];
    leaves(plan, String::new(), &mut paths);
    let p = paths[field as usize % paths.len()].clone();
    let numbers = [
        0.0,
        -1.0,
        0.5,
        1e15,
        -1e15,
        1e-300,
        1e9,
        -0.99,
        f64::from(value),
        f64::from(value) / 100.0,
    ];
    let strings = [
        "never",
        "start",
        "end",
        "retire",
        "afterSale",
        "",
        "2026-13",
        "1800-01",
        "2300-01",
        "fixed",
        "historical",
        "rolling",
        "bootstrap",
        "parametric",
        "three_buckets",
        "guardrails",
        "vpw",
        "endowment",
        "fixed_rate",
    ];
    let replacement = match kind % 8 {
        0..=3 => json!(numbers[value as usize % numbers.len()]),
        4 | 5 => json!(strings[value as usize % strings.len()]),
        6 => [json!(null), json!([]), json!({}), json!(true)][value as usize % 4].clone(),
        _ => {
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
            return;
        }
    };
    if let Some(x) = plan.pointer_mut(&p) {
        *x = replacement;
    }
}

/// At most 255 paths and no sweeps, so that a run takes milliseconds.
fn keep_small(plan: &mut Value) {
    for key in ["bootstrap", "parametric"] {
        if let Some(n) = plan["returns"][key]["paths"].as_f64() {
            plan["returns"][key]["paths"] = json!(n.clamp(0.0, 255.0));
        }
    }
    if plan["scenarios"].is_object() {
        plan["scenarios"]["sweeps"] = json!([]);
    }
}

fn finite_result(r: &engine::sim::SimResult) -> bool {
    let s = &r.summary;
    s.end_total.is_finite()
        && s.end_total_real.is_finite()
        && s.taxes_total.is_finite()
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
                x.taxes,
            ]
            .iter()
            .all(|v| v.is_finite())
        })
}

fuzz_target!(|data: &[u8]| {
    let mut plan = base();
    let (ops, _) = data.as_chunks::<4>();
    for c in ops.iter().take(8) {
        mutate(&mut plan, u16::from_le_bytes([c[0], c[1]]), c[2], c[3]);
    }
    keep_small(&mut plan);
    let out = api::run(&plan, Path::new("/"), Some(Path::new(LIBRARY)));
    if let Some(r) = &out.single {
        assert!(
            finite_result(r),
            "ungültige Zahl: {}",
            serde_json::to_string(&plan).unwrap()
        );
    }
    if let Some(b) = &out.batch {
        assert!(
            b.details.iter().all(|(_, r)| finite_result(r))
                && b.summary
                    .percentiles
                    .iter()
                    .all(|p| p.end_total.is_finite() && p.end_total_real.is_some_and(f64::is_finite)),
            "ungültige Zahl: {}",
            serde_json::to_string(&plan).unwrap()
        );
    }
});
