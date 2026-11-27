//! #301: Early withdrawal from pillar 3a for owner-occupied home ownership (usage `home`), at any age.
//!
//! Made-up plan that can be recomputed by hand: birth 1980-01, start 2030-01 (age 50), AHV payout at 65 in January
//! 2045, so the limit of 5 years before the AHV payout is January 2040; balance 100'000, no interest, no contribution,
//! spending 1'000 per month, an owned house without mortgage and without costs; no returns, no inflation, no taxes
//! (except where noted).

use engine::api;
use engine::model::{PayoutUse, Plan};
use engine::sim::{EventKind, MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::Severity;
use serde_json::{json, Value};

fn house() -> Value {
    json!({ "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "bought": "2010-01",
        "sell": "never", "fixCosts": 0 })
}

fn base(payouts: Value) -> Value {
    json!({
        "project": { "name": "3a-Vorbezug" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2046-12", "fixed": true }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "ahv" }
        },
        "flows": { "income": 0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } },
        "realestate": { "properties": [ house() ] },
        "pillar3a": [ { "person": "A", "balance": 100_000, "rate": 0, "payouts": payouts } ]
    })
}

fn home(date: &str, amount: Option<f64>) -> Value {
    match amount {
        Some(a) => json!({ "date": date, "amount": a, "use": "home" }),
        None => json!({ "date": date, "use": "home" }),
    }
}

fn ym(y: i32, m: u32) -> YearMonth {
    YearMonth::new(y, m)
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let sim = Simulator::new(&plan).expect("plan valid");
    let path = engine::market::fixed_path(&sim.fixed, sim.start, sim.months());
    sim.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records
        .iter()
        .find(|x| x.month == ym(y, m))
        .expect("month in the period")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} instead of {b}");
}

fn payouts(r: &SimResult) -> Vec<(YearMonth, String)> {
    r.events
        .iter()
        .filter(|e| e.kind == EventKind::Pillar3aPayout)
        .map(|e| (e.month, e.text.to_string()))
        .collect()
}

/// Errors of the section pillar 3a: (field, message).
fn errors(v: &Value) -> Vec<(Option<String>, String)> {
    api::validate(v)
        .into_iter()
        .filter(|i| i.severity == Severity::Error && i.section == "pillar3a")
        .map(|i| (i.field, i.message.to_string()))
        .collect()
}

/// #301/AK1: The usage «Wohneigentum (Vorbezug)» is read and written as `home`. 30'000 at age 52 (June 2032): the 3a
/// falls from 100'000 to 70'000, the liquid wealth rises by 30'000 − 1'000 spending = 29'000.
#[test]
fn issue_301_ak1_home_goes_into_wealth() {
    let v = base(json!([home("2032-06", Some(30_000.0))]));
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(plan.pillar3a[0].payouts[0].usage, PayoutUse::Home);
    assert_eq!(
        serde_json::to_value(&plan).unwrap()["pillar3a"][0]["payouts"][0]["use"],
        "home"
    );
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    let r = run(&v);
    close(rec(&r, 2032, 5).pillar3a, 100_000.0, "before");
    close(rec(&r, 2032, 6).pillar3a, 70_000.0, "after the early withdrawal");
    close(
        rec(&r, 2032, 6).liquid - rec(&r, 2032, 5).liquid,
        29_000.0,
        "into wealth",
    );
    let p = payouts(&r);
    assert_eq!(p[0].0, ym(2032, 6), "{p:?}");
    assert!(p[0].1.contains("30000 CHF"), "{}", p[0].1);
    // The remainder 70'000 at the AHV payout as before.
    close(rec(&r, 2045, 1).pillar3a, 0.0, "remainder at the AHV payout");
    assert!(
        p.iter().any(|x| x.0 == ym(2045, 1) && x.1.contains("70000 CHF")),
        "{p:?}"
    );
}

/// #301/AK2: Before the limit (January 2040) the usages `home` and `repay` are allowed, `wealth` stays an error at
/// the field (the message names 01.2040).
#[test]
fn issue_301_ak2_early_only_for_home_and_repay() {
    let early = |usage: &str| {
        let v = base(json!([{ "date": "2032-06", "amount": 10_000, "use": usage }]));
        errors(&v)
    };
    assert!(early("home").is_empty(), "{:?}", early("home"));
    assert!(early("repay").is_empty(), "{:?}", early("repay"));
    let e = early("wealth");
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.payouts.0.date"));
    assert!(e[0].1.contains("01.2040"), "{}", e[0].1);
    // Age as the point in time: 52.5 years = July 2032.
    let v = base(json!([{ "kind": "age", "years": 52.5, "amount": 10_000, "use": "home" }]));
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    let p = payouts(&run(&v));
    assert_eq!(p[0].0, ym(2032, 7), "{p:?}");
}

/// #301/AK3: Partial amount as above (AK1); without an amount the whole balance. #301/AK6: contributions go on
/// afterwards: 6'000 per year = 500 per month from July 2032 → 3'000 in December 2032.
#[test]
fn issue_301_ak3_ak6_whole_balance_and_contributions_afterwards() {
    let mut v = base(json!([home("2032-06", None)]));
    v["pillar3a"][0]["contribution"] = json!(6000);
    let r = run(&v);
    // June 2032: 100'000 + 30 × 500 contributions (January 2030 to June 2032) = 115'000.
    close(rec(&r, 2032, 6).pillar3a, 0.0, "whole balance");
    assert!(payouts(&r)[0].1.contains("115000 CHF"), "{:?}", payouts(&r));
    close(rec(&r, 2032, 12).pillar3a, 3_000.0, "contributions afterwards");
}

/// #301/AK4: Two early withdrawals before the limit need 5 years between them (60 months); the error names both dates
/// and stands at the later one. After the limit (January 2040) no gap is needed.
#[test]
fn issue_301_ak4_five_years_between_early_withdrawals() {
    let v = base(json!([
        home("2036-05", Some(10_000.0)),
        home("2031-06", Some(10_000.0))
    ]));
    let e = errors(&v);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.payouts.0.date"));
    assert!(e[0].1.contains("06.2031") && e[0].1.contains("05.2036"), "{}", e[0].1);
    // Exactly 60 months: allowed.
    let v = base(json!([
        home("2031-06", Some(10_000.0)),
        home("2036-06", Some(10_000.0))
    ]));
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    // «Hypothek zurückzahlen» before the limit is an early withdrawal too (BVV 3 Art. 3 para. 3 let. c).
    let v = base(json!([home("2031-06", Some(10_000.0)), { "date": "2034-06", "amount": 10_000, "use": "repay" }]));
    assert_eq!(errors(&v).len(), 1, "{:?}", errors(&v));
    // Second one after the limit: no early withdrawal, no gap.
    let v = base(json!([
        home("2036-06", Some(10_000.0)),
        home("2040-01", Some(10_000.0))
    ]));
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
}

/// #301/AK5: An early withdrawal needs an owner-occupied property in the month: none (rent only) → error; planned
/// purchase in June 2032 → allowed in June, error in May; owned and sold in January 2032 → error in June.
#[test]
fn issue_301_ak5_only_with_an_owned_home() {
    let mut v = base(json!([home("2032-06", Some(10_000.0))]));
    v["realestate"] = json!({ "properties": [] });
    let e = errors(&v);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.payouts.0.date"));
    assert!(e[0].1.contains("06.2032"), "{}", e[0].1);
    // Repayment before the limit without a property: the same error.
    v["pillar3a"][0]["payouts"] = json!([{ "date": "2032-06", "amount": 10_000, "use": "repay" }]);
    assert_eq!(errors(&v).len(), 1, "{:?}", errors(&v));

    let planned = |buy: &str| {
        let mut v = base(json!([home("2032-06", Some(10_000.0))]));
        v["realestate"] = json!({ "properties": [ { "name": "Neu", "status": "planned", "price": 500_000,
            "buy": buy, "sell": "never", "fixCosts": 0 } ] });
        errors(&v)
    };
    assert!(planned("2032-06").is_empty(), "{:?}", planned("2032-06"));
    assert_eq!(planned("2032-07").len(), 1, "{:?}", planned("2032-07"));

    let mut v = base(json!([home("2032-06", Some(10_000.0))]));
    v["realestate"]["properties"][0]["sell"] = json!("2032-01");
    assert_eq!(errors(&v).len(), 1, "{:?}", errors(&v));
}

/// #301/AK6: The early withdrawal is a lump sum with lump-sum tax in its year, added to the pension fund lump sum of
/// the same year (#150). Rate linear to 10 % at 200'000: 100'000 alone → 5 % = 5'000; with a PK lump sum of 100'000 in
/// the same year (pension fund at 58 in January 2038, early withdrawal in June 2038, still before the limit) →
/// 200'000 at 10 % = 20'000 for the year.
#[test]
fn issue_301_ak6_capital_tax_added_to_other_lump_sums() {
    let mut v = base(json!([home("2032-06", Some(100_000.0))]));
    v["taxes"]["capital"] = json!([[0, 0.0], [200_000, 0.10]]);
    let r = run(&v);
    close(rec(&r, 2032, 6).tax_capital, 5_000.0, "lump-sum tax alone");
    assert!(payouts(&r)[0].1.contains("tax 5000 CHF"), "{:?}", payouts(&r));
    v["milestones"][1]["years"] = json!(58);
    v["pillar3a"][0]["payouts"] = json!([home("2038-06", Some(100_000.0))]);
    v["pension"]["pk"] = json!({ "capital": 100_000, "at": "retire", "lumpsum": 1, "conversion": 0 });
    let r = run(&v);
    let year: f64 = r
        .records
        .iter()
        .filter(|x| x.month.year() == 2038)
        .map(|x| x.tax_capital)
        .sum();
    close(year, 20_000.0, "lump sums of the year added together");
}

/// #301 with tranches (#300): an early withdrawal before the limit still applies, the tranches compute with the smaller
/// balance: 30'000 in June 2032 → 70'000 in 5 tranches from 60 = 14'000 each. A row «Ins freie Vermögen» stays locked.
#[test]
fn issue_301_early_withdrawal_with_tranches() {
    let mut v = base(json!([home("2032-06", Some(30_000.0)), { "date": "2041-06", "amount": 5_000, "use": "wealth" }]));
    v["pillar3a"][0]["tranches"] = json!({ "enabled": true, "fromAge": 60, "count": 5 });
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    let p = payouts(&run(&v));
    assert_eq!(
        p.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![
            ym(2032, 6),
            ym(2040, 1),
            ym(2041, 1),
            ym(2042, 1),
            ym(2043, 1),
            ym(2044, 1)
        ],
        "{p:?}"
    );
    assert!(p[1..].iter().all(|x| x.1.contains("14000 CHF")), "{p:?}");
    // The early withdrawal is checked with tranches too (AK5).
    v["realestate"] = json!({ "properties": [] });
    assert_eq!(errors(&v).len(), 1, "{:?}", errors(&v));
}

/// #301: «Wohneigentum (Vorbezug)» is a single payout; for the remainder and the tranches it is an error.
#[test]
fn issue_301_home_not_for_the_remainder() {
    let mut v = base(json!([]));
    v["pillar3a"][0]["use"] = json!("home");
    let e = errors(&v);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.use"));
}

/// #301/AK9: The schema, `describe` and `plan_set` know the usage `home`, with the label of the app.
#[test]
fn issue_301_ak9_schema_describe_and_set() {
    let leaves = engine::describe::schema_leaves();
    let leaf = leaves
        .iter()
        .find(|l| l.path == "pillar3a.*.payouts.*.use")
        .expect("field in the schema");
    assert!(leaf.values.iter().any(|v| v == "home"), "{:?}", leaf.values);
    let set = api::set_fields(
        &base(json!([home("2032-06", Some(1.0))])),
        &["pillar3a.0.payouts.0.use=repay".to_string()],
    )
    .expect("set");
    assert_eq!(set["pillar3a"][0]["payouts"][0]["use"], "repay");
    let set = api::set_fields(&set, &["pillar3a.0.payouts.0.use=home".to_string()]).expect("set");
    assert_eq!(set["pillar3a"][0]["payouts"][0]["use"], "home");
    let label = engine::choices::choices("de")
        .fields
        .into_iter()
        .find(|(p, _)| p == "pillar3a.*.payouts.*.use")
        .expect("choice field")
        .1;
    assert!(
        label
            .iter()
            .any(|i| i.name == "home" && i.meaning.starts_with("«Wohneigentum (Vorbezug)»")),
        "{label:?}"
    );
}
