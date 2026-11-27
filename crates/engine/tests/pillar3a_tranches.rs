//! #300: Pillar 3a paid out in yearly tranches (switch «In Tranchen beziehen» with age of the first tranche and number
//! of tranches).
//!
//! Made-up plan that can be recomputed by hand: birth 1980-01 (age = year − 1980, birth month January), start 2043-01
//! (age 63), AHV payout at 65 in January 2045; no returns, no inflation, no taxes (except where noted).

use engine::api;
use engine::model::Plan;
use engine::sim::{EventKind, MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::Severity;
use serde_json::{json, Value};

fn base(tranches: Value) -> Value {
    json!({
        "project": { "name": "3a-Tranchen" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2043-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "ahv" }
        },
        "flows": { "income": 0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } },
        "pillar3a": [ { "person": "A", "balance": 150_000, "rate": 0, "tranches": tranches } ]
    })
}

fn on(from_age: u32, count: u32) -> Value {
    json!({ "enabled": true, "fromAge": from_age, "count": count })
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

fn errors(v: &Value) -> Vec<(Option<String>, String)> {
    api::validate(v)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| (i.field, i.message.to_string()))
        .collect()
}

/// #300/AK1: The switch and its two values are read from the plan; missing values give 60 years and 5 tranches.
#[test]
fn issue_300_ak1_switch_and_values_in_the_plan() {
    let plan: Plan = serde_json::from_value(base(on(63, 3))).unwrap();
    let t = plan.pillar3a[0].tranches_on().expect("switched on");
    assert_eq!((t.from_age, t.count), (63, 3));
    let plan: Plan = serde_json::from_value(base(json!({ "enabled": true }))).unwrap();
    let t = plan.pillar3a[0].tranches_on().unwrap();
    assert_eq!((t.from_age, t.count), (60, 5));
    let plan: Plan = serde_json::from_value(base(json!({ "enabled": false, "fromAge": 63, "count": 3 }))).unwrap();
    assert!(
        plan.pillar3a[0].tranches_on().is_none(),
        "off: values stay, no tranches"
    );
    assert_eq!(plan.pillar3a[0].tranches.as_ref().unwrap().count, 3);
}

/// #300/AK2: Balance 150'000 at 63, interest 0 %, no contributions, from 63 in 3 tranches: 50'000 at 63, 64 and 65,
/// each in the birth month (January); the last one is the remainder, nothing is left at the AHV payout.
#[test]
fn issue_300_ak2_three_tranches_of_50000() {
    let r = run(&base(on(63, 3)));
    close(rec(&r, 2043, 1).pillar3a, 100_000.0, "after tranche 1");
    close(rec(&r, 2043, 12).pillar3a, 100_000.0, "until tranche 2");
    close(rec(&r, 2044, 1).pillar3a, 50_000.0, "after tranche 2");
    close(rec(&r, 2045, 1).pillar3a, 0.0, "after tranche 3");
    let p = payouts(&r);
    assert_eq!(
        p.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2043, 1), ym(2044, 1), ym(2045, 1)],
        "{p:?}"
    );
    assert!(p[0].1.contains("Tranche 1 of 3 50000 CHF"), "{}", p[0].1);
    assert!(p[1].1.contains("Tranche 2 of 3 50000 CHF"), "{}", p[1].1);
    assert!(p[2].1.contains("Tranche 3 of 3 50000 CHF"), "{}", p[2].1);
    // into wealth: January 2044 + 50'000 − 1'000 spending
    close(
        rec(&r, 2044, 1).liquid - rec(&r, 2043, 12).liquid,
        49_000.0,
        "inflow 2044",
    );
}

/// #300/AK2: Tranches before the simulation start are past: from 60 in 6 tranches, start at 63 → the tranches at 63, 64
/// and 65 divide the balance by 3, 2 and 1 tranches left (150'000 → 50'000 each).
#[test]
fn issue_300_ak2_tranches_before_the_start_are_past() {
    let r = run(&base(on(60, 6)));
    let p = payouts(&r);
    assert_eq!(p.len(), 3, "{p:?}");
    assert!(p[0].1.contains("Tranche 4 of 6 50000 CHF"), "{}", p[0].1);
    assert!(p[2].1.contains("Tranche 6 of 6 50000 CHF"), "{}", p[2].1);
}

/// #300/AK3: With contributions the tranche follows the balance of the month. Contribution 6'000 per year (500 per
/// month until before the AHV payout), first added in the month:
/// 2043-01: (150'000 + 500) / 3 = 50'166.67; 2044-01: (100'333.33 + 12 × 500) / 2 = 53'166.67;
/// 2045-01 (AHV, no contribution): 53'166.67 + 11 × 500 = 58'666.67.
#[test]
fn issue_300_ak3_tranches_follow_contributions() {
    let mut v = base(on(63, 3));
    v["pillar3a"][0]["contribution"] = json!(6000);
    let r = run(&v);
    let p = payouts(&r);
    assert!(p[0].1.contains("Tranche 1 of 3 50167 CHF"), "{}", p[0].1);
    assert!(p[1].1.contains("Tranche 2 of 3 53167 CHF"), "{}", p[1].1);
    assert!(p[2].1.contains("Tranche 3 of 3 58667 CHF"), "{}", p[2].1);
    close(rec(&r, 2045, 1).pillar3a, 0.0, "empty after the last tranche");
}

/// #300/AK3: The tranches depend on the path: with inflation 2 % the indexed contribution (#306) grows and the second
/// and third tranche are larger than without inflation.
#[test]
fn issue_300_ak3_tranches_depend_on_the_path() {
    let amounts = |inflation: f64| -> Vec<f64> {
        let mut v = base(on(63, 3));
        v["pillar3a"][0]["contribution"] = json!(6000);
        v["pillar3a"][0]["contributionIndexed"] = json!(true);
        v["returns"]["fixed"]["inflation"] = json!(inflation);
        let r = run(&v);
        let mut out = vec![];
        for y in [2043, 2044, 2045] {
            let before = if y == 2043 {
                150_000.0
            } else {
                rec(&r, y - 1, 12).pillar3a
            };
            out.push(before - rec(&r, y, 1).pillar3a);
        }
        out
    };
    let flat = amounts(0.0);
    let grown = amounts(0.02);
    assert!(grown[1] > flat[1] && grown[2] > flat[2], "{flat:?} {grown:?}");
}

/// #300/AK4: The last tranche at the latest in the month of the AHV payout; otherwise an error with the age of the
/// last tranche and the AHV payout. Ages outside 60–65 and counts outside 1–6 are errors at their fields.
#[test]
fn issue_300_ak4_last_tranche_at_the_latest_at_ahv() {
    assert!(errors(&base(on(63, 3))).is_empty(), "last at 65 = AHV month");
    assert!(errors(&base(on(60, 6))).is_empty());
    let e = errors(&base(on(64, 3)));
    assert_eq!(
        e,
        vec![(
            Some("pillar3a.0.tranches.count".into()),
            "Pillar 3a: the last tranche at age 66 (01.2046) is after the start of the AHV pension (01.2045); it must be paid out at the latest in that month.".into()
        )]
    );
    let e = errors(&base(on(59, 7)));
    assert_eq!(
        e,
        vec![
            (
                Some("pillar3a.0.tranches.fromAge".into()),
                "Pillar 3a: the age of the first tranche must be between 60 and 65.".into()
            ),
            (
                Some("pillar3a.0.tranches.count".into()),
                "Pillar 3a: the number of tranches must be between 1 and 6.".into()
            ),
        ]
    );
    assert_eq!(errors(&base(on(66, 0))).len(), 2);
    // AHV payout at 66: the first tranche at 60 is more than 5 years before it (rule of the payout plan, #52).
    let mut v = base(on(60, 3));
    v["milestones"][2]["years"] = json!(66);
    let e = errors(&v);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.tranches.fromAge"));
    // switched off: no check of the tranches
    assert!(errors(&base(json!({ "enabled": false, "fromAge": 64, "count": 6 }))).is_empty());
}

/// #300/AK5: With the switch on, the payout plan is locked (ignored and not checked); the usage of the account applies
/// to every tranche.
#[test]
fn issue_300_ak5_payout_plan_locked_and_usage_for_all_tranches() {
    let mut v = base(on(63, 3));
    v["pillar3a"][0]["payouts"] = json!([{ "date": "2043-06", "amount": 10_000 }, { "date": "2030-01" }]);
    assert!(errors(&v).is_empty(), "rows of the locked plan are not checked");
    let p = payouts(&run(&v));
    assert_eq!(p.len(), 3, "only the tranches: {p:?}");
    assert!(!p.iter().any(|x| x.0 == ym(2043, 6)));
    // Usage «Hypothek zurückzahlen»: every tranche first repays the tranche with indirect amortization.
    v["pillar3a"][0]["payouts"] = json!([]);
    v["pillar3a"][0]["use"] = json!("repay");
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": "2043-01",
            "periods": [{ "type": "fixed", "term": 20, "rate": 0, "amort": 0, "amortMode": "indirect" }] } ] } } ] });
    let p = payouts(&run(&v));
    assert_eq!(p.len(), 3, "{p:?}");
    assert!(p.iter().all(|x| x.1.contains("to mortgage 50000 CHF")), "{p:?}");
}

/// #300/AK7: Lump-sum tax per year on each tranche. Rate linear: 50'000 → 2.5 % = 1'250 per tranche, 3'750 together
/// (one payout of 150'000: 7.5 % = 11'250). The yearly values show the tax of each tranche in its year.
#[test]
fn issue_300_ak7_capital_tax_per_tranche_and_yearly_values() {
    let mut v = base(on(63, 3));
    v["taxes"]["capital"] = json!([[0, 0.0], [200_000, 0.10]]);
    let r = run(&v);
    for y in [2043, 2044, 2045] {
        close(rec(&r, y, 1).tax_capital, 1_250.0, "tax of the tranche");
    }
    assert!(payouts(&r)[0].1.contains("tax 1250 CHF"));
    let rows = api::yearly(&serde_json::to_value(&r).unwrap())
        .expect("yearly values")
        .nominal;
    let tax = |y: i32| rows.iter().find(|x| x.year == y).unwrap().tax_capital;
    for y in [2043, 2044, 2045] {
        close(tax(y), 1_250.0, "yearly lump-sum tax");
    }
    close(tax(2046), 0.0, "nothing after the AHV payout");
    // PK lump sum 100'000 in the year of the second tranche: added together (150'000 at 7.5 % = 11'250 − 5'000 PK).
    v["milestones"][1]["years"] = json!(64);
    v["pension"]["pk"] = json!({ "capital": 100_000, "at": "retire", "lumpsum": 1, "conversion": 0 });
    let r = run(&v);
    let year: f64 = r
        .records
        .iter()
        .filter(|x| x.month.year() == 2044)
        .map(|x| x.tax_capital)
        .sum();
    close(year, 11_250.0, "tranche and PK lump sum together");
}

/// #300/AK8: Without the switch (missing or off) the result is the same as before; a missing switch is not written.
#[test]
fn issue_300_ak8_without_switch_unchanged() {
    let mut without = base(Value::Null);
    without["pillar3a"][0].as_object_mut().unwrap().remove("tranches");
    let off = base(json!({ "enabled": false, "fromAge": 63, "count": 3 }));
    let (a, b) = (run(&without), run(&off));
    for (x, y) in a.records.iter().zip(&b.records) {
        assert_eq!(
            (x.pillar3a, x.total, x.tax_capital),
            (y.pillar3a, y.total, y.tax_capital),
            "{}",
            x.month
        );
    }
    // the whole balance as the remainder at the AHV payout
    assert_eq!(payouts(&a).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2045, 1)]);
    let plan: Plan = serde_json::from_value(without).unwrap();
    assert!(serde_json::to_value(&plan).unwrap()["pillar3a"][0]
        .get("tranches")
        .is_none());
}

/// #300/AK9: Schema, `describe` and `plan_set` know the switch and the two values.
#[test]
fn issue_300_ak9_schema_describe_and_set() {
    let schema = engine::schema_json();
    for name in ["Pillar3aTranches", "fromAge", "\"count\""] {
        assert!(schema.contains(name), "schema without {name}");
    }
    let lib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let c = api::describe(Some(engine::describe::Section::Fields), None, &lib, Some(&lib), "de");
    let rows = &c.fields.as_ref().unwrap().groups["pillar3a.*.tranches"];
    let joined = rows.join("\n");
    assert!(joined.contains("enabled: boolean"), "{joined}");
    assert!(joined.contains("fromAge: integer, age, 60..65"), "{joined}");
    assert!(joined.contains("count: integer, count, 1..6"), "{joined}");
    // plan_set (field paths): switch on, missing values with their defaults; then both values
    let mut v = base(Value::Null);
    v["pillar3a"][0].as_object_mut().unwrap().remove("tranches");
    let set = api::set_fields(&v, &["pillar3a.0.tranches.enabled=true".into()]).expect("set");
    assert_eq!(set["pillar3a"][0]["tranches"], json!({ "enabled": true }));
    let set = api::set_fields(
        &set,
        &[
            "pillar3a.0.tranches.fromAge=63".into(),
            "pillar3a.0.tranches.count=3".into(),
        ],
    )
    .expect("set");
    assert_eq!(
        set["pillar3a"][0]["tranches"],
        json!({ "enabled": true, "fromAge": 63, "count": 3 })
    );
    assert!(api::set_fields(&set, &["pillar3a.0.tranches.count=\"drei\"".into()]).is_err());
}
