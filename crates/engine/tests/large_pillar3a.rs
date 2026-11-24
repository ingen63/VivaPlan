//! #152: Large pillar 3a for persons without a pension fund: deduction and contribution limit 20 % of the earned income,
//! at most the maximum of the large pillar 3a (settings, 36'288 in 2026); for all others the small pillar 3a.
//!
//! Made-up plan that can be recomputed by hand: start 2030-01, end 2031-12, no returns, no inflation, own tables with a
//! flat income tax of 10 %, no wealth tax.

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

const LARGE: f64 = 36_288.0;
const SMALL: f64 = 7_258.0;

/// One person «A» with earned income `income` per year and pillar 3a contribution `contribution`, deduction switched on.
fn base(income: f64, contribution: f64, large: bool) -> Value {
    json!({
        "project": { "name": "Grosse Säule 3a" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01", "noPensionFund": large } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2031-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": income / 12.0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [], "capital": [[0, 0.0]],
                   "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 }, "deductPillar3a": true },
        "pillar3a": [ { "person": "A", "balance": 0, "rate": 0, "contribution": contribution } ]
    })
}

/// [`base`] with a second person «B» (income `income_b` per year, contribution `contribution_b`, with a pension fund).
fn couple(contribution_a: f64, income_b: f64, contribution_b: f64) -> Value {
    let mut v = base(100_000.0, contribution_a, true);
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1980-01", "noPensionFund": true },
        { "id": "B", "name": "B", "birth": "1980-01" } ]);
    v["household"]["married"] = json!(true);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "name": "Bezug Pensionskasse Person 2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "name": "AHV-Bezug Person 2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "income": income_b / 12.0,
        "pk": { "capital": 50_000, "at": "retire2", "lumpsum": 0, "conversion": 0 },
        "ahv": { "monthly": 0, "at": "ahv2" } });
    v["pillar3a"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "person": "B", "balance": 0, "rate": 0, "contribution": contribution_b }));
    v
}

/// Adds a property with indirect amortization of `per_year` into the pillar 3a of `person` (the plan rejects a
/// contribution above the maximum; together with the amortization only a warning).
fn indirect(v: &mut Value, person: &str, per_year: f64) {
    v["realestate"] = json!({ "properties": [ { "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000,
        "sell": "never", "fixCosts": 0, "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": "2030-01",
        "pillar3a": person, "periods": [ { "type": "fixed", "term": 20, "rate": 0, "amort": per_year / 4.0,
        "amortMode": "indirect" } ] } ] } } ] });
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let s = Simulator::new(&plan).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn december(r: &SimResult, y: i32) -> &MonthRecord {
    r.records.iter().find(|x| x.month == YearMonth::new(y, 12)).unwrap()
}

/// Deducted pillar 3a contributions of year `y`.
fn deduction(r: &SimResult, y: i32) -> f64 {
    december(r, y).tax_basis.as_ref().expect("basis").pillar3a
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

/// Messages of the validation as (severity, code, field).
fn issues(v: &Value) -> Vec<(Severity, &'static str, String)> {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    validate(&plan)
        .into_iter()
        .map(|i| (i.severity, i.message.code(), i.field.clone().unwrap_or_default()))
        .collect()
}

fn has(v: &Value, code: &str) -> bool {
    issues(v).iter().any(|i| i.1 == code)
}

/// #152/AK1: Per person the entry «ohne Pensionskasse (grosse Säule 3a)»; missing: no, and it is not written then.
#[test]
fn issue_152_ak1_entry_per_person() {
    let plan: Plan = serde_json::from_value(couple(0.0, 0.0, 0.0)).unwrap();
    let persons = &plan.household.persons;
    assert!(persons[0].no_pension_fund && !persons[1].no_pension_fund);
    let out = serde_json::to_value(&plan).unwrap();
    assert_eq!(out["household"]["persons"][0]["noPensionFund"], true);
    assert!(out["household"]["persons"][1].get("noPensionFund").is_none());
}

/// #152/AK2, AK6: Income 100'000, contribution 25'000 → deduction 20'000 (20 %); income 300'000, contribution 40'000 →
/// deduction equal to the maximum of the large pillar 3a. Tax 10 % of the rest.
#[test]
fn issue_152_ak6_twenty_percent_at_most_the_maximum() {
    let r = run(&base(100_000.0, 25_000.0, true));
    close(deduction(&r, 2030), 20_000.0, "20 % of 100'000");
    close(december(&r, 2030).assessed_income, 8_000.0, "tax on 80'000");
    let mut v = base(300_000.0, 30_000.0, true);
    indirect(&mut v, "A", 10_000.0);
    let r = run(&v);
    close(deduction(&r, 2030), LARGE, "maximum of the large pillar 3a");
    close(december(&r, 2030).assessed_income, (300_000.0 - LARGE) * 0.1, "tax");
    // Below both limits the whole contribution.
    close(
        deduction(&run(&base(300_000.0, 30_000.0, true)), 2030),
        30_000.0,
        "whole contribution",
    );
}

/// #152/AK2: The maximum comes from the settings and grows with «Tarife mit der Teuerung» (#66); the 20 % refer to the
/// earned income of the calendar year (income only until June: half).
#[test]
fn issue_152_ak2_maximum_from_settings_and_indexed() {
    let mut v = base(300_000.0, 30_000.0, true);
    indirect(&mut v, "A", 10_000.0);
    v["limits"] = json!({ "pillar3aMaxLarge": 30_000 });
    close(deduction(&run(&v), 2030), 30_000.0, "settings");
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["taxes"]["indexed"] = json!(true);
    let r = run(&v);
    let idx = december(&r, 2031).price_index;
    assert!(idx > 1.03);
    close(deduction(&r, 2031), 30_000.0 * idx, "indexed");

    let mut v = base(0.0, 25_000.0, true);
    v["flows"]["income"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 100_000.0 / 12.0 }, { "from": "2030-07", "value": 0 } ] });
    close(deduction(&run(&v), 2030), 10_000.0, "20 % of 50'000");
    close(deduction(&run(&v), 2031), 0.0, "no earned income");
}

/// #152/AK3: The other persons keep the maximum of the small pillar 3a; only the person without a pension fund gets
/// the large one.
#[test]
fn issue_152_ak3_others_keep_the_small_pillar3a() {
    let mut v = base(100_000.0, SMALL, false);
    indirect(&mut v, "A", 10_000.0);
    close(deduction(&run(&v), 2030), SMALL, "single, small");
    let mut v = couple(25_000.0, 100_000.0, SMALL);
    indirect(&mut v, "B", 10_000.0);
    close(deduction(&run(&v), 2030), 20_000.0 + SMALL, "A large, B small");
}

// #152/AK4 (warning «ohne Pensionskasse» with a pension fund) was dropped with #253: the pension fund then counts
// as 0 (`tests/no_pension_fund.rs`).

/// #152/AK5: Existing plans compute unchanged: without the entry (or with `false`) the deduction stays at the small
/// pillar 3a and the results are bit-identical.
#[test]
fn issue_152_ak5_existing_plans_unchanged() {
    let mut old = base(100_000.0, SMALL, false);
    indirect(&mut old, "A", 10_000.0);
    old["household"]["persons"][0]
        .as_object_mut()
        .unwrap()
        .remove("noPensionFund");
    let mut new = old.clone();
    new["household"]["persons"][0]["noPensionFund"] = json!(false);
    let (a, b) = (run(&old), run(&new));
    for (x, y) in a.records.iter().zip(&b.records) {
        assert_eq!(x.assessed_income, y.assessed_income, "{}", x.month);
    }
    close(deduction(&a, 2030), SMALL, "small pillar 3a");
    let plan: Plan = serde_json::from_value(old).unwrap();
    assert_eq!(plan.limits.pillar3a_max, SMALL);
    assert_eq!(plan.limits.pillar3a_max_large, LARGE);
}

/// #152/AK8: The contribution limit of the plan follows the same rule as the deduction: without a pension fund up to the
/// maximum of the large pillar 3a (above: error) and a warning above 20 % of the earned income; with a pension fund the
/// small pillar 3a.
#[test]
fn issue_152_ak8_contribution_limit_follows_the_deduction() {
    let range = (
        Severity::Error,
        "pillar3a_contribution_range",
        "pillar3a.0.contribution".to_string(),
    );
    let over = "pillar3a_over_income";
    // Large pillar 3a: 20'000 of 100'000 fine, 25'000 too much (warning), 36'288 of 300'000 fine, 40'000 error.
    assert!(!has(&base(100_000.0, 20_000.0, true), over));
    let v = base(100_000.0, 25_000.0, true);
    assert!(issues(&v).contains(&(Severity::Warning, over, String::new())));
    assert!(!issues(&v).contains(&range));
    let v = base(300_000.0, LARGE, true);
    assert!(!has(&v, over) && !issues(&v).contains(&range));
    assert!(issues(&base(300_000.0, 40_000.0, true)).contains(&range));
    // Small pillar 3a: 7'258 fine, 8'000 error, no matter the income.
    assert!(!issues(&base(300_000.0, SMALL, false)).contains(&range));
    assert!(issues(&base(300_000.0, 8_000.0, false)).contains(&range));
    assert!(!has(&base(10_000.0, SMALL, false), over));
    // The warning names the first month with too little income (here from 2031) and the limit.
    let mut v = base(0.0, 20_000.0, true);
    v["flows"]["income"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 100_000.0 / 12.0 }, { "from": "2031-01", "value": 50_000.0 / 12.0 } ] });
    let text = validate(&serde_json::from_value(v).unwrap())
        .into_iter()
        .find(|i| i.message.code() == over)
        .expect("warning")
        .message
        .to_string();
    assert!(text.contains("10'000 per year from 01.2031"), "{text}");
    // Only indirect amortization (no 3a account): the same rule.
    let mut v = base(20_000.0, 0.0, true);
    v["pillar3a"] = json!([]);
    indirect(&mut v, "A", 6_000.0);
    assert!(has(&v, over), "6'000 > 20 % of 20'000");
    v["flows"]["income"] = json!(30_000.0 / 12.0);
    assert!(!has(&v, over), "6'000 = 20 % of 30'000");
    // The contributions end before the AHV pension: no income from then on is fine.
    let mut v = base(0.0, 20_000.0, true);
    v["milestones"][2] = json!({ "id": "ahv", "name": "AHV-Bezug", "kind": "date", "date": "2031-01" });
    v["flows"]["income"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 100_000.0 / 12.0 }, { "from": "ahv", "value": 0 } ] });
    assert!(!has(&v, over), "{:?}", issues(&v));
    v["flows"]["income"]["rows"][1]["from"] = json!("2030-12");
    assert!(has(&v, over), "income ends one month before the AHV pension");
    // Second person: their own income counts (B: 25'000 of 100'000 too much, 20'000 fine).
    let mut v = couple(0.0, 100_000.0, 25_000.0);
    v["household"]["persons"][1]["noPensionFund"] = json!(true);
    let warning = validate(&serde_json::from_value(v.clone()).unwrap())
        .into_iter()
        .find(|i| i.message.code() == over)
        .expect("warning for B")
        .message
        .to_string();
    assert!(warning.starts_with("Pillar 3a B:"), "{warning}");
    v["pillar3a"][1]["contribution"] = json!(20_000);
    assert!(!has(&v, over));
}
