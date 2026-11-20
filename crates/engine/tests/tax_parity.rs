//! #156: Shared tax cases (`tests/fixtures/tax-cases.json`) against `tax::rate_at` and `tax::tax`. The same cases
//! run in the user interface (`e2e/unit/tax-parity.test.js`), so both compute the tax alike.

use engine::model::Taxes;
use engine::tax::{rate_at, tax};
use serde_json::{json, Value};
use std::path::Path;

fn cases() -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tax-cases.json");
    let text = std::fs::read_to_string(path).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    v["cases"].as_array().unwrap().clone()
}

/// The multiplier as the engine reads it from a plan: via `Taxes` (default 1 if missing, error if invalid).
fn read_taxes(c: &Value) -> Result<Taxes, serde_json::Error> {
    let mut t = json!({ "income": c["table"], "wealth": [], "capital": [] });
    if let Some(m) = c.get("multiplier") {
        t["multiplier"] = m.clone();
    }
    serde_json::from_value(t)
}

#[test]
fn issue_156_ak1_shared_cases_cover_the_edge_cases() {
    let cs = cases();
    let has = |f: &dyn Fn(&Value) -> bool| cs.iter().any(f);
    assert!(has(&|c| c["amount"] == 0));
    assert!(has(&|c| c["amount"].as_f64().unwrap() < 0.0));
    assert!(has(&|c| c["multiplier"] == 0));
    assert!(has(&|c| c.get("multiplier").is_none()));
    assert!(has(&|c| c["tax"].is_null()));
    // On a bracket, between brackets and above the last bracket
    let brackets = |c: &Value| -> Vec<f64> {
        c["table"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r[0].as_f64().unwrap())
            .collect()
    };
    let amount = |c: &Value| c["amount"].as_f64().unwrap();
    assert!(has(&|c| brackets(c).len() > 1 && brackets(c)[1..].contains(&amount(c))));
    assert!(has(&|c| {
        let b = brackets(c);
        b.windows(2).any(|w| w[0] < amount(c) && amount(c) < w[1])
    }));
    assert!(has(&|c| {
        let b = brackets(c);
        !b.is_empty() && b.iter().all(|&x| x < amount(c))
    }));
}

#[test]
fn issue_156_ak1_engine_computes_the_shared_cases() {
    for c in cases() {
        let name = c["name"].as_str().unwrap();
        let taxes = read_taxes(&c);
        let table = match &taxes {
            Ok(t) => t.income.clone(),
            Err(_) => serde_json::from_value(c["table"].clone()).unwrap(),
        };
        let x = c["amount"].as_f64().unwrap();
        let rate = rate_at(&table, x);
        assert!(
            (rate - c["rate"].as_f64().unwrap()).abs() < 1e-12,
            "{name}: rate {rate}"
        );
        match c["tax"].as_f64() {
            Some(expected) => {
                let m = taxes.unwrap_or_else(|e| panic!("{name}: {e}")).multiplier;
                let got = tax(&table, x, m);
                assert!((got - expected).abs() < 1e-9, "{name}: tax {got}, expected {expected}");
            }
            None => assert!(taxes.is_err(), "{name}: the engine should refuse the multiplier"),
        }
    }
}
