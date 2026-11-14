//! Zusätzliche Prüfungen der Validierung: Person 2, Monte-Carlo-Grenzen, Warnungen zu Strategie und Variationen.

use engine::api;
use engine::validate::{Issue, Severity};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Test" },
        "household": { "persons": [ { "id": "A", "name": "Anna", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2040-01" },
            { "id": "ahv", "name": "AHV", "kind": "date", "date": "2040-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2031-12" }
        ],
        "pension": {
            "pk": { "capital": 100_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 },
            "ahv": { "monthly": 2_000, "at": "ahv" }
        },
        "flows": { "income": 5_000, "spending": 4_000 },
        "wealth": { "stocks": 1_000, "bonds": 1_000, "cash": 1_000, "stockShare": 0.5, "reserveMonths": 3 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.04, "bonds": 0.01, "inflation": 0.01 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn has(v: &Value, severity: Severity, section: &str, text: &str) -> bool {
    let all: Vec<Issue> = api::validate(v);
    all.iter()
        .any(|i| i.severity == severity && i.section == section && i.message.contains(text))
}

fn with_partner(lumpsum: f64, conversion: f64) -> Value {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "Anna", "birth": "1965-01" },
        { "id": "B", "name": "Ben", "birth": "1965-01" }
    ]);
    v["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": lumpsum, "conversion": conversion }, "ahv": { "monthly": 0, "at": "ahv" } });
    v
}

#[test]
fn partner_pension_fund_inputs_are_checked() {
    assert!(has(
        &with_partner(1.5, 0.05),
        Severity::Error,
        "pension",
        "Person 2: Kapitalbezug"
    ));
    assert!(has(
        &with_partner(0.5, 0.2),
        Severity::Warning,
        "pension",
        "Person 2: Umwandlungssatz"
    ));
    assert!(!has(&with_partner(0.5, 0.05), Severity::Error, "pension", "Person 2"));
}

#[test]
fn second_person_without_data_is_only_a_warning() {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "Anna", "birth": "1965-01" },
        { "id": "B", "name": "Ben", "birth": "1965-01" }
    ]);
    assert!(has(
        &v,
        Severity::Warning,
        "general",
        "zweite Person hat weder Einkommen"
    ));
}

#[test]
fn monte_carlo_paths_and_series_are_checked() {
    let mut v = base();
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["bootstrap"] = json!({ "paths": 10, "block": 12, "seed": 1 });
    assert!(has(&v, Severity::Error, "returns", "Zeitreihe gewählt"));
    v["returns"]["bootstrap"]["paths"] = json!(1_000_001);
    assert!(has(&v, Severity::Error, "returns", "zwischen 1 und 1'000'000"));
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 1_000_001, "mu": 0.07, "sigma": 0.15, "df": 0, "jumpLambda": 0, "jumpMu": 0, "jumpSigma": 0, "seed": 1 });
    assert!(has(&v, Severity::Error, "returns", "zwischen 1 und 1'000'000"));
    v["returns"]["parametric"]["paths"] = json!(1_000_000);
    assert!(!has(&v, Severity::Error, "returns", "zwischen 1 und 1'000'000"));
}

#[test]
fn three_buckets_covering_more_than_fifteen_years_warn() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 8, "bondYears": 8 } });
    assert!(has(&v, Severity::Warning, "withdrawal", "mehr als 15 Jahre"));
}

#[test]
fn sweeps_that_have_no_effect_are_reported() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 3 } });
    v["scenarios"] = json!({ "sweeps": [ { "param": "wealth.stockShare", "from": 0.0, "to": 0.5, "step": 0.5 } ] });
    assert!(has(
        &v,
        Severity::Warning,
        "scenarios",
        "3-Topf-Strategie wirkt der Aktienanteil"
    ));
}

#[test]
fn too_many_combinations_are_an_error_in_the_plan() {
    let mut v = base();
    let sweep = |param: &str, from: f64, to: f64| json!({ "param": param, "from": from, "to": to, "step": 0.01 });
    v["scenarios"] = json!({ "sweeps": [
        sweep("wealth.stockShare", 0.0, 1.0),
        sweep("pension.pk.lumpsum", 0.0, 0.99),
        sweep("flows.spendingFactor", 1.0, 1.99)
    ] });
    assert!(has(&v, Severity::Error, "scenarios", "1'010'000 Kombinationen"));
}
