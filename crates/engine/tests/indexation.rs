//! #306: Amounts as in reality with inflation, exceptions «nominal» under «Experte».
//!
//! Made-up plan: person A born 1975-01, start 2030-01, end 2040-12, inflation 2 % per year (price index after n
//! months 1.02^(n/12)), no returns, no taxes (except where noted). Hand values with 2 %: after 12 months × 1.02,
//! after 5 years × 1.104081 (40 000 → 44 163.23, 2 500 → 2 760.20).

use engine::api;
use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Teuerung" },
        "indexedByDefault": true,
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1975-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2040-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "2039-01", "lumpsum": 0, "conversion": 0, "contribution": 1000 },
                     "ahv": { "monthly": 2000, "at": "2033-01" } },
        "flows": { "income": 6000, "spending": 5000, "spendingIncludesHousing": true,
                   "extras": [ { "date": "2035-01", "amount": -40000, "label": "Auto" } ] },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 3_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0.02, "saron": 0 } },
        "taxes": { "income": [[0, 0]], "wealth": [], "capital": [[0, 0]], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } },
        "pillar3a": [ { "person": "A", "contribution": 6000 } ]
    })
}

fn plan(v: &Value) -> Plan {
    serde_json::from_value(crate::any_age(v.clone())).expect("plan readable")
}

fn run(v: &Value) -> SimResult {
    let s = Simulator::new(&plan(v)).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records.iter().find(|x| x.month == YearMonth::new(y, m)).unwrap()
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 0.01, "{what}: {a} instead of {b}");
}

fn rent(from: &str) -> Value {
    json!({ "name": "Miete", "status": "rent", "from": from, "rent": 2500 })
}

/// #306/AK1: In a plan with the rules of reality every amount of AK1 grows with the price index of the path.
#[test]
fn issue_306_ak1_amounts_grow_with_inflation() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = json!({ "at": "2032-01", "monthly": 9000 });
    let r = run(&v);
    let jan31 = rec(&r, 2031, 1);
    close(jan31.price_index, 1.02, "price index after 12 months");
    close(jan31.ledger.living, 5100.0, "living costs");
    close(jan31.earned, 6120.0, "earned income");
    close(jan31.ledger.pk_contributions, 1020.0, "pension fund contributions");
    close(jan31.pillar3a_in, 510.0, "pillar 3a contribution");
    // #284: the AHV follows the mixed index (real wage growth 0.5 % by default) in odd years, also before the payout
    let mix = |n: i32| (1.02_f64.powi(n) + (1.02_f64 * 1.005).powi(n)) / 2.0;
    // early payout at 58 (birth 1975-01): reduced by 13.6 %, in whole francs
    let paid = |b: f64| b + (b * -0.136).round();
    close(rec(&r, 2033, 1).ahv, paid(2000.0 * mix(3)), "AHV pension at the payout");
    close(
        rec(&r, 2034, 1).ahv,
        rec(&r, 2033, 6).ahv,
        "AHV pension unchanged in an even year",
    );
    close(
        rec(&r, 2035, 1).ahv,
        paid(2000.0 * mix(5)),
        "AHV pension adjusted in an odd year",
    );
    close(rec(&r, 2032, 1).care, 9363.6, "care home");
    close(rec(&r, 2035, 1).ledger.extras, -44_163.23, "one-off amount");
}

/// #306/AK1: One-off deductions from income and wealth grow until their tax year (index of December 2035
/// 1.02^(71/12) = 1.124306).
#[test]
fn issue_306_ak1_one_off_deductions_grow_with_inflation() {
    let mut v = base();
    v["taxes"]["oneOffDeductions"] = json!([{ "year": 2035, "label": "Einkauf", "amount": 10_000 }]);
    v["taxes"]["oneOffWealthDeductions"] = json!([{ "year": 2035, "label": "Schuld", "amount": 20_000 }]);
    let r = run(&v);
    let b = rec(&r, 2035, 12).tax_basis.as_ref().expect("basis of the income tax");
    close(b.one_off, 11_243.06, "one-off deduction from income");
    close(
        b.wealth.as_ref().expect("basis of the wealth").one_off,
        22_486.11,
        "one-off deduction from wealth",
    );
    v["taxes"]["oneOffIndexed"] = json!(false);
    v["taxes"]["oneOffWealthIndexed"] = json!(false);
    let r = run(&v);
    let b = rec(&r, 2035, 12).tax_basis.as_ref().unwrap();
    close(b.one_off, 10_000.0, "exception nominal (income)");
    close(
        b.wealth.as_ref().unwrap().one_off,
        20_000.0,
        "exception nominal (wealth)",
    );
}

/// #306/AK2: The pension fund pension stays nominal.
#[test]
fn issue_306_ak2_pension_fund_pension_stays_nominal() {
    let mut v = base();
    v["pension"]["pk"] = json!({ "capital": 600_000, "at": "start", "lumpsum": 0, "conversion": 0.05 });
    let r = run(&v);
    close(rec(&r, 2030, 6).pk_pension, 2500.0, "pension 2030");
    close(rec(&r, 2038, 6).pk_pension, 2500.0, "pension 2038");
}

/// #306/AK3: A rent in today's prices grows until moving in (2 500 → 2 760.20 after 5 years) and then stays nominal;
/// a rent running at the start stays nominal from the start.
#[test]
fn issue_306_ak3_rent_grows_until_moving_in() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [rent("2035-01")] });
    let r = run(&v);
    close(rec(&r, 2035, 1).housing, 2760.20, "rent at moving in");
    close(rec(&r, 2038, 6).housing, 2760.20, "rent after moving in");
    v["realestate"] = json!({ "properties": [rent("start")] });
    let r = run(&v);
    close(rec(&r, 2030, 1).housing, 2500.0, "running rent at the start");
    close(rec(&r, 2036, 1).housing, 2500.0, "running rent later");
}

/// #306/AK4: Every amount of AK1 can be set to «nominal, without inflation»; the rent «nominal» stays at the entered
/// amount, «throughout» (older plans) grows on after moving in.
#[test]
fn issue_306_ak4_exceptions_keep_amounts_nominal() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = json!({ "at": "2032-01", "monthly": 9000, "indexed": false });
    v["flows"]["spendingIndexed"] = json!(false);
    v["flows"]["incomeIndexed"] = json!(false);
    v["flows"]["extrasIndexed"] = json!(false);
    v["pension"]["pk"]["contributionIndexed"] = json!(false);
    v["pension"]["ahv"]["indexed"] = json!(false);
    v["pillar3a"][0]["contributionIndexed"] = json!(false);
    let r = run(&v);
    let jan31 = rec(&r, 2031, 1);
    close(jan31.ledger.living, 5000.0, "living costs");
    close(jan31.earned, 6000.0, "earned income");
    close(jan31.ledger.pk_contributions, 1000.0, "pension fund contributions");
    close(jan31.pillar3a_in, 500.0, "pillar 3a contribution");
    close(rec(&r, 2034, 1).ahv / rec(&r, 2033, 6).ahv, 1.0, "AHV pension");
    close(rec(&r, 2032, 1).care, 9000.0, "care home");
    close(rec(&r, 2035, 1).ledger.extras, -40_000.0, "one-off amount");

    let mut v = base();
    let mut p = rent("2035-01");
    p["inflationRent"] = json!(false);
    v["realestate"] = json!({ "properties": [p.clone()] });
    close(rec(&run(&v), 2036, 1).housing, 2500.0, "rent nominal");
    p["inflationRent"] = json!(true);
    v["realestate"] = json!({ "properties": [p] });
    close(
        rec(&run(&v), 2036, 1).housing,
        2500.0 * 1.02f64.powi(6),
        "rent throughout",
    );
}

/// #306/AK4: Value and fix costs of a property follow inflation; the exception keeps them nominal.
#[test]
fn issue_306_ak4_property_value_and_fix_costs() {
    let mut v = base();
    let mut flat = json!({ "id": "w", "name": "Wohnung", "status": "owned", "price": 500_000, "worth": 600_000,
        "fixCosts": 800, "sell": "never" });
    v["realestate"] = json!({ "properties": [flat.clone()] });
    let r = run(&v);
    close(rec(&r, 2031, 1).property_value, 612_000.0, "value with inflation");
    close(rec(&r, 2031, 1).housing, 816.0, "fix costs with inflation");
    flat["inflationValue"] = json!(false);
    flat["inflationFixCosts"] = json!(false);
    v["realestate"] = json!({ "properties": [flat] });
    let r = run(&v);
    close(rec(&r, 2031, 1).property_value, 600_000.0, "value nominal");
    close(rec(&r, 2031, 1).housing, 800.0, "fix costs nominal");
}

/// Older plan without the marker with every kind of amount: earned income, contributions, one-off amounts and
/// deductions, partner, rent in the future, property with nominal value.
fn older() -> Value {
    let mut v = base();
    v.as_object_mut().unwrap().remove("indexedByDefault");
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01", "careHome": { "at": "2039-01", "monthly": 8000 } },
        { "id": "B", "name": "B", "birth": "1967-01" } ]);
    v["household"]["married"] = json!(true);
    v["household"]["marriedNoted"] = json!(true);
    v["partner"] = json!({ "income": 3000,
        "pk": { "capital": 100_000, "at": "2033-01", "lumpsum": 1, "conversion": 0, "contribution": 400 },
        "ahv": { "monthly": 1500, "at": "2032-01", "indexed": true } });
    v["taxes"] = json!({ "income": [[0, 0.0], [200_000, 0.1]], "wealth": [[0, 0.0], [5_000_000, 0.005]],
        "capital": [[0, 0.0], [1_000_000, 0.05]], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 },
        "deductPillar3a": true,
        "oneOffDeductions": [{ "year": 2033, "label": "Einkauf", "amount": 15_000 }],
        "oneOffWealthDeductions": [{ "year": 2034, "label": "Schuld", "amount": 30_000 }] });
    v["realestate"] = json!({ "properties": [rent("2034-07"),
        { "id": "w", "name": "Wohnung", "status": "owned", "price": 500_000, "worth": 600_000, "fixCosts": 800,
          "sell": "never", "inflationFixCosts": true }] });
    crate::any_age(v)
}

/// #306/AK5: Opening an older plan writes its earlier rules as exceptions (with a note) and the marker; the result
/// stays the same, for the golden plans as well. A second opening changes nothing.
#[test]
fn issue_306_ak5_older_plans_compute_unchanged() {
    let lib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let result = |v: &Value| {
        let mut out = serde_json::to_value(api::run(v, &lib, Some(&lib))).unwrap();
        out.as_object_mut().unwrap().remove("elapsedMs");
        out
    };
    let mut cases = crate::golden::cases();
    cases.push(("older", older()));
    for (name, v) in cases {
        let mut up = v.clone();
        engine::upgrade::indexation(&mut up, &mut vec![]);
        assert_eq!(up["indexedByDefault"], true, "{name}");
        assert_eq!(result(&up), result(&v), "{name}: result changed");
        let mut notes = vec![];
        engine::upgrade::indexation(&mut up.clone(), &mut notes);
        assert!(notes.is_empty(), "{name}: second opening");
    }
    let up = api::upgrade_plan(&older());
    assert!(up.notes.iter().any(|n| n.code() == "note_indexation"));
    let p = &up.plan;
    // AK5: earned income, tariffs, one-off amounts and deductions, contributions become exceptions «nominal»
    assert_eq!(p["flows"]["incomeIndexed"], false);
    assert_eq!(p["flows"]["extrasIndexed"], false);
    assert_eq!(p["partner"]["incomeIndexed"], false);
    assert_eq!(p["pension"]["pk"]["contributionIndexed"], false);
    assert_eq!(p["partner"]["pk"]["contributionIndexed"], false);
    assert_eq!(p["pension"]["ahv"]["indexed"], false);
    assert_eq!(
        p["partner"]["ahv"]["indexed"], true,
        "the choice «with inflation» stays"
    );
    assert_eq!(p["pillar3a"][0]["contributionIndexed"], false);
    assert_eq!(p["taxes"]["indexed"], false);
    assert_eq!(p["taxes"]["pillar3aMaxIndexed"], false);
    assert_eq!(p["taxes"]["oneOffIndexed"], false);
    assert_eq!(p["taxes"]["oneOffWealthIndexed"], false);
    assert_eq!(
        p["realestate"]["properties"][0]["inflationRent"], true,
        "rent throughout as before"
    );
    assert_eq!(p["realestate"]["properties"][1]["inflationValue"], false);
    assert_eq!(p["realestate"]["properties"][1]["inflationFixCosts"], true);
    // living costs and care home grew before: no exception
    assert_eq!(p["flows"].get("spendingIndexed"), None);
    assert_eq!(p["household"]["persons"][0]["careHome"].get("indexed"), None);
}

/// #306/AK6: A new plan has no exception: every amount of AK1 grows with inflation.
#[test]
fn issue_306_ak6_new_plans_have_no_exception() {
    let v = api::blank_plan(YearMonth::new(2026, 7));
    assert_eq!(v["indexedByDefault"], true);
    let p: Plan = serde_json::from_value(v.clone()).unwrap();
    let ix = engine::indexation::Indexation::of(&p);
    assert!(ix.spending && ix.taxes && ix.extras && ix.one_off && ix.one_off_wealth && ix.pillar3a_max);
    assert_eq!((ix.income[0], ix.pk_contribution[0], ix.ahv[0]), (true, true, true));
    assert!(
        api::upgrade_plan(&v)
            .notes
            .iter()
            .all(|n| n.code() != "note_indexation"),
        "nothing to take over"
    );
}

/// #306/AK7: The pillar 3a contribution grows at most to the maximum of the year (maximum × price index of January);
/// with the maximum nominal it stays at the maximum in nominal francs. The validation compares in prices of the start.
#[test]
fn issue_306_ak7_pillar3a_contribution_at_most_the_maximum() {
    let mut v = base();
    v["pillar3a"][0]["contribution"] = json!(7258);
    let r = run(&v);
    close(rec(&r, 2031, 6).pillar3a_in, 616.93, "maximum 2031 = 7 258 × 1.02 / 12");
    assert!(
        rec(&r, 2031, 6).price_index > 1.02 + 1e-6,
        "the contribution alone would be higher"
    );
    v["taxes"]["pillar3aMaxIndexed"] = json!(false);
    close(rec(&run(&v), 2031, 6).pillar3a_in, 604.83, "maximum nominal");
    v["pillar3a"][0]["contribution"] = json!(7258);
    let errors = |v: &Value| {
        engine::validate::validate(&plan(v))
            .into_iter()
            .filter(|i| i.severity == engine::validate::Severity::Error && i.section == "pillar3a")
            .count()
    };
    assert_eq!(errors(&v), 0, "the maximum in prices of the start is allowed");
    v["pillar3a"][0]["contribution"] = json!(7259);
    assert_eq!(errors(&v), 1, "above the maximum in prices of the start");
}

/// #306/AK9: The catalog, the schema and `set` know the exceptions.
#[test]
fn issue_306_ak9_catalog_schema_and_set() {
    let paths: Vec<String> = engine::describe::fields().into_iter().map(|f| f.path).collect();
    let schema = engine::schema_json();
    for p in [
        "indexedByDefault",
        "flows.spendingIndexed",
        "flows.extrasIndexed",
        "flows.incomeIndexed",
        "household.persons.*.careHome.indexed",
        "pension.pk.contributionIndexed",
        "pension.ahv.indexed",
        "pillar3a.*.contributionIndexed",
        "taxes.indexed",
        "taxes.pillar3aMaxIndexed",
        "taxes.oneOffIndexed",
        "taxes.oneOffWealthIndexed",
        "realestate.properties.*.inflationRent",
    ] {
        assert!(paths.iter().any(|x| x == p), "describe: {p}");
        let key = p.rsplit('.').next().unwrap();
        assert!(schema.contains(&format!("\"{key}\"")), "schema: {p}");
    }
    let v = crate::any_age(base());
    let set = engine::fieldpath::set(
        &v,
        &["flows.extrasIndexed=false".into(), "taxes.oneOffIndexed=false".into()],
    )
    .expect("set");
    assert_eq!(set["flows"]["extrasIndexed"], false);
    assert_eq!(set["taxes"]["oneOffIndexed"], false);
}
