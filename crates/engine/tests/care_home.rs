//! #271: Care home per person: costs from the entry, end of the household's housing, sale at the entry, no rent after
//! the sale, note in the validation and deduction of the costs from the income tax.
//!
//! Made-up plan: person A born 1960-01, start 2044-01, end 2047-12, entry into the care home at 85 (2045-01) with
//! 9'000 per month, no returns, no inflation (except where noted), no taxes (except where noted).

use engine::model::Plan;
use engine::sim::{EventKind, MonthRecord, PropertyEventKind, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Heim" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1960-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2044-01", "fixed": true },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "start", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 3000, "at": "start" } },
        "flows": { "spending": 6000, "spendingIncludesHousing": true },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 2_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "income": [[0, 0]], "wealth": [], "capital": [[0, 0]], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

fn care(age: f64, monthly: f64, sell: bool) -> Value {
    json!({ "age": age, "monthly": monthly, "sell": sell })
}

fn rent(from: &str) -> Value {
    json!({ "name": "Miete", "status": "rent", "from": from, "rent": 2000 })
}

/// Owned flat without a mortgage, fix costs 800, never sold as planned.
fn flat() -> Value {
    json!({ "id": "w", "name": "Wohnung", "status": "owned", "price": 500_000, "worth": 600_000, "fixCosts": 800, "sell": "never" })
}

fn plan(v: &Value) -> Plan {
    serde_json::from_value(any_age(v.clone())).expect("plan readable")
}

fn any_age(v: Value) -> Value {
    crate::any_age(v)
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
    assert!((a - b).abs() < 1e-6, "{what}: {a} instead of {b}");
}

fn codes(v: &Value) -> Vec<(Severity, String)> {
    validate(&plan(v))
        .into_iter()
        .map(|i| (i.severity, i.message.code().to_string()))
        .collect()
}

fn has_code(v: &Value, code: &str) -> bool {
    codes(v).iter().any(|(_, c)| c == code)
}

/// #271/AK1: entry as an age or a date, costs with inflation; without an entry the plan is written as before (the
/// golden files stay unchanged, `golden::*`).
#[test]
fn issue_271_ak1_entry_age_or_date_and_costs_with_inflation() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    let r = run(&v);
    close(rec(&r, 2044, 12).care, 0.0, "before the entry");
    let jan = rec(&r, 2045, 1);
    close(jan.care, 9000.0 * jan.price_index, "costs with inflation");
    assert!(jan.price_index > 1.0);
    let event = r.events.iter().find(|e| e.kind == EventKind::CareHome).expect("event");
    assert_eq!(event.month, YearMonth::new(2045, 1));
    assert!(!event.text.text().contains("person"), "{}", event.text.text());
    // the same entry as a date
    v["household"]["persons"][0]["careHome"] = json!({ "at": "2045-01", "monthly": 9000 });
    let d = run(&v);
    close(rec(&d, 2045, 1).care, jan.care, "entry as a date");
    // without an entry: no field `careHome`, no field `care` in the records
    let r0 = run(&base());
    let text = serde_json::to_string(&plan(&base())).unwrap();
    assert!(!text.contains("careHome"));
    assert!(r0.records.iter().all(|x| x.care == 0.0));
    assert!(!serde_json::to_string(&r0.records[0]).unwrap().contains("\"care\""));
}

/// #271/AK1: entry missing, age outside 0..150 or negative costs are errors of the validation.
#[test]
fn issue_271_ak1_invalid_entries_are_errors() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = json!({ "monthly": 9000 });
    assert!(has_code(&v, "care_home_entry_missing"));
    v["household"]["persons"][0]["careHome"] = care(151.0, 9000.0, false);
    assert!(has_code(&v, "care_home_age_invalid"));
    v["household"]["persons"][0]["careHome"] = care(85.0, -1.0, false);
    assert!(has_code(&v, "care_home_costs_invalid"));
    v["household"]["persons"][0]["careHome"] = json!({ "at": "nope", "monthly": 9000 });
    assert!(codes(&v).iter().any(|(s, _)| *s == Severity::Error));
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    assert!(!codes(&v).iter().any(|(s, _)| *s == Severity::Error));
}

/// #271/AK2: single person with a rented flat: from the entry month the rent ends and the care home costs appear in
/// the spending and in a column of the yearly values.
#[test]
fn issue_271_ak2_rent_ends_at_the_entry() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [rent("start")] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    let r = run(&v);
    let dec = rec(&r, 2044, 12);
    close(dec.housing, 2000.0, "rent before the entry");
    close(dec.care, 0.0, "no care home before the entry");
    let jan = rec(&r, 2045, 1);
    close(jan.housing, 0.0, "no rent in the care home");
    close(jan.care, 9000.0, "care home costs");
    // living costs 6'000 include the rent 2'000: 4'000 + 9'000
    close(jan.spending, 4000.0 + 9000.0, "spending");
    let yearly = engine::digest::yearly(&engine::digest::StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    });
    let y45 = yearly.nominal.iter().find(|y| y.year == 2045).unwrap();
    close(y45.care, 9000.0, "column care home (monthly average)");
    close(y45.flows.care, 108_000.0, "care home in the bars");
    let y44 = yearly.nominal.iter().find(|y| y.year == 2044).unwrap();
    close(y44.care, 0.0, "no care home in 2044");
    let row = serde_json::to_value(y44).unwrap();
    assert!(row.get("care").is_none(), "no column without costs");
}

/// #271/AK3: owned flat, «verkaufen ja», rent «nach Verkauf»: sale in the entry month, the rent does not start, the
/// property costs end.
#[test]
fn issue_271_ak3_sale_at_the_entry_without_rent() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [flat(), rent("afterSale")] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, true);
    let r = run(&v);
    close(rec(&r, 2044, 12).housing, 800.0, "fix costs before");
    let sale = r
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .expect("sale");
    assert_eq!(sale.month, YearMonth::new(2045, 1));
    assert!(!r.events.iter().any(|e| e.kind == EventKind::RentStart), "no rent");
    for m in [2, 6, 12] {
        let x = rec(&r, 2045, m);
        close(x.housing, 0.0, "no housing costs after the sale");
        close(x.property_value, 0.0, "property sold");
        close(x.mortgage, 0.0, "no mortgage");
        close(x.care, 9000.0, "care home costs");
    }
    // without a care home the rent «nach Verkauf» starts after a planned sale as before
    let mut w = base();
    let mut f = flat();
    f["sell"] = json!("2045-01");
    w["realestate"] = json!({ "properties": [f, rent("afterSale")] });
    let r = run(&w);
    assert!(r.events.iter().any(|e| e.kind == EventKind::RentStart));
    close(rec(&r, 2045, 3).housing, 2000.0, "rent after the sale");
}

/// #271/AK3: a mortgage ends with the sale at the entry.
#[test]
fn issue_271_ak3_mortgage_ends() {
    let mut v = base();
    let mut f = flat();
    f["mortgage"] = json!({ "tranches": [ { "name": "T", "amount": 200_000, "start": "2040-01",
        "periods": [ { "type": "fixed", "term": 20, "rate": 0.015 } ] } ] });
    v["realestate"] = json!({ "properties": [f] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, true);
    let r = run(&v);
    close(rec(&r, 2044, 12).mortgage, 200_000.0, "mortgage before");
    close(rec(&r, 2045, 2).mortgage, 0.0, "mortgage repaid with the sale");
    close(rec(&r, 2045, 2).housing, 0.0, "no interest after the sale");
}

/// #271/AK4: «verkaufen nein»: the property stays with its costs, the care home costs come on top; an emergency
/// sale still happens and no rent starts afterwards.
#[test]
fn issue_271_ak4_keep_the_property() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [flat(), rent("afterSale")] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    let r = run(&v);
    assert!(r.property_events.is_empty());
    let x = rec(&r, 2046, 6);
    close(x.housing, 800.0, "property costs stay");
    close(x.care, 9000.0, "care home costs on top");
    assert!(x.property_value > 0.0);
    // little wealth: emergency sale; the rent «nach Verkauf» does not start
    v["wealth"]["cash"] = json!(60_000);
    v["realestate"]["emergencyMonths"] = json!(3);
    let r = run(&v);
    let sale = r
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::EmergencySale)
        .expect("emergency sale");
    assert!(sale.month >= YearMonth::new(2045, 1));
    assert!(!r.events.iter().any(|e| e.kind == EventKind::RentStart), "no rent");
    let after = r.records.iter().filter(|x| x.month > sale.month.plus(1));
    for x in after {
        assert_eq!(
            x.housing, 0.0,
            "no housing costs after the emergency sale ({})",
            x.month
        );
    }
}

/// #271/AK5: couple, one person in the care home: the household keeps its housing costs, no sale despite
/// «verkaufen»; once both live in a care home the flat is sold and the housing costs end.
#[test]
fn issue_271_ak5_couple_sells_once_both_are_in_the_care_home() {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1960-01", "careHome": care(85.0, 9000.0, true) },
        { "id": "B", "name": "B", "birth": "1962-01", "careHome": care(85.0, 8000.0, false) }
    ]);
    v["partner"] = json!({ "pk": { "capital": 0, "at": "start", "lumpsum": 0, "conversion": 0 },
                            "ahv": { "monthly": 2000, "at": "start" } });
    v["realestate"] = json!({ "properties": [flat(), rent("afterSale")] });
    let r = run(&v);
    let texts: Vec<String> = r
        .events
        .iter()
        .filter(|e| e.kind == EventKind::CareHome)
        .map(|e| e.text.text().to_string())
        .collect();
    assert_eq!(texts.len(), 2);
    assert!(
        texts[0].contains("(person 1)") && texts[1].contains("(person 2)"),
        "{texts:?}"
    );
    let x = rec(&r, 2046, 6);
    close(x.housing, 800.0, "housing of the household goes on");
    close(x.care, 9000.0, "costs of A");
    assert!(x.property_value > 0.0, "no sale while B lives at home");
    let sale = r
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .expect("sale");
    assert_eq!(sale.month, YearMonth::new(2047, 1));
    let y = rec(&r, 2047, 3);
    close(y.housing, 0.0, "no housing costs once both live in a care home");
    close(y.care, 17_000.0, "costs of both");
    assert!(!r.events.iter().any(|e| e.kind == EventKind::RentStart));
    // a rent of the household also goes on until both live in a care home
    v["realestate"] = json!({ "properties": [rent("start")] });
    let r = run(&v);
    close(rec(&r, 2046, 6).housing, 2000.0, "rent while B lives at home");
    close(rec(&r, 2047, 1).housing, 0.0, "no rent once both live in a care home");
}

/// #271/AK6: note when after the sale of the last property neither a rent nor a care home follows.
#[test]
fn issue_271_ak6_note_without_housing_after_the_sale() {
    const NOTE: &str = "housing_missing_after_sale";
    let mut f = flat();
    f["sell"] = json!("2045-01");
    let mut v = base();
    v["realestate"] = json!({ "properties": [f.clone()] });
    assert!(codes(&v).contains(&(Severity::Warning, NOTE.to_string())));
    // a rent «nach Verkauf»
    v["realestate"] = json!({ "properties": [f.clone(), rent("afterSale")] });
    assert!(!has_code(&v, NOTE));
    // a rent with a fixed start that goes on after the sale
    v["realestate"] = json!({ "properties": [f.clone(), rent("2046-01")] });
    assert!(!has_code(&v, NOTE));
    // a care home from the sale on
    v["realestate"] = json!({ "properties": [f.clone()] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    assert!(!has_code(&v, NOTE));
    // a care home only years after the sale: note
    v["household"]["persons"][0]["careHome"] = care(87.0, 9000.0, false);
    assert!(has_code(&v, NOTE));
    // sale because of the care home, no rent: no note (the care home follows)
    v["realestate"] = json!({ "properties": [flat()] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, true);
    assert!(!has_code(&v, NOTE));
    // no sale at all: no note
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    assert!(!has_code(&v, NOTE));
    // the existing plans without a sale or with a rent get no note
    assert!(!has_code(&base(), NOTE));
}

/// #271/AK6: limits of the note: planned purchases, sale at or after the end, rents that end or never start, couples.
#[test]
fn issue_271_ak6_limits_of_the_note() {
    const NOTE: &str = "housing_missing_after_sale";
    let mut f = flat();
    f["sell"] = json!("2045-01");
    let with = |props: Value| {
        let mut v = base();
        v["realestate"] = json!({ "properties": props });
        v
    };
    // a purchase that never takes place does not count, a planned purchase without a sale stays
    let never = json!({ "id": "p", "name": "Plan", "status": "planned", "price": 400_000, "buy": "never" });
    assert!(has_code(&with(json!([f.clone(), never])), NOTE));
    let later = json!({ "id": "p", "name": "Plan", "status": "planned", "price": 400_000, "buy": "2046-01" });
    assert!(!has_code(&with(json!([f.clone(), later])), NOTE));
    // sale in the last month of the plan: note; after the end: none
    let mut at_end = flat();
    at_end["sell"] = json!("2047-12");
    assert!(has_code(&with(json!([at_end])), NOTE));
    let mut after = flat();
    after["sell"] = json!("2048-01");
    assert!(!has_code(&with(json!([after])), NOTE));
    // a rent that ends with the sale or never starts does not count, one that goes on does
    let rent_until =
        |until: &str| json!({ "name": "Miete", "status": "rent", "from": "2040-01", "until": until, "rent": 2000 });
    assert!(has_code(&with(json!([f.clone(), rent_until("2045-01")])), NOTE));
    assert!(!has_code(&with(json!([f.clone(), rent_until("2046-01")])), NOTE));
    assert!(has_code(&with(json!([f.clone(), rent("never")])), NOTE));
    // the fields «from» and «until» of an owned property are no rent
    let mut odd = f.clone();
    odd["from"] = json!("2040-01");
    assert!(has_code(&with(json!([odd])), NOTE));
    // «Immobilien nie verkaufen»: no sale, no note
    let mut v = with(json!([f.clone()]));
    v["realestate"]["neverSell"] = json!(true);
    assert!(!has_code(&v, NOTE));
    // couple: both in the care home after the sale is needed
    let mut v = with(json!([f.clone()]));
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1960-01", "careHome": care(85.0, 9000.0, false) },
        { "id": "B", "name": "B", "birth": "1962-01" }
    ]);
    v["partner"] = json!({ "pk": { "capital": 0, "at": "start", "lumpsum": 0, "conversion": 0 },
                            "ahv": { "monthly": 2000, "at": "start" } });
    assert!(has_code(&v, NOTE));
    v["household"]["persons"][1]["careHome"] = care(83.0, 9000.0, false);
    assert!(!has_code(&v, NOTE));
    v["household"]["persons"][1]["careHome"] = care(84.0, 9000.0, false);
    assert!(has_code(&v, NOTE), "B enters only in 2046");
}

/// #271/AK7: the warning about «Immobilien nie verkaufen» only with a property that could be sold.
#[test]
fn issue_271_ak7_warning_only_with_a_property() {
    let mut v = base();
    v["realestate"] = json!({ "neverSell": true, "properties": [rent("start")] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, true);
    assert!(!has_code(&v, "care_home_never_sell"));
}

/// #271/AK7: «Immobilien nie verkaufen» has priority over «verkaufen»: no sale, care home costs on top, a warning.
#[test]
fn issue_271_ak7_never_sell_has_priority() {
    let mut v = base();
    v["realestate"] = json!({ "neverSell": true, "properties": [flat(), rent("afterSale")] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, true);
    assert!(has_code(&v, "care_home_never_sell"));
    let r = run(&v);
    assert!(r.property_events.is_empty(), "no sale");
    let x = rec(&r, 2046, 1);
    close(x.housing, 800.0, "property costs stay");
    close(x.care, 9000.0, "care home costs on top");
    v["realestate"]["neverSell"] = json!(false);
    assert!(!has_code(&v, "care_home_never_sell"));
}

/// #271/AK9: example: net income 60'000, care home costs 108'000 a year: deduction 105'000 (108'000 − 5 % of 60'000),
/// taxable income not below 0.
#[test]
fn issue_271_ak9_deduction_above_five_percent() {
    close(engine::sim::care_deduction(108_000.0, 60_000.0), 105_000.0, "example");
    close(engine::sim::care_deduction(2_000.0, 60_000.0), 0.0, "below the share");
    close(engine::sim::care_deduction(3_500.0, 60_000.0), 500.0, "above the share");
    close(
        engine::sim::care_deduction(1_000.0, -10_000.0),
        1_000.0,
        "negative net income",
    );
    // own tables, flat 10 %: income 60'000 a year (earned income 5'000 a month, no AHV)
    let mut v = base();
    v["pension"]["ahv"]["monthly"] = json!(0);
    v["flows"]["income"] = json!(5000);
    v["taxes"]["income"] = json!([[0, 0.1]]);
    let without = run(&v);
    close(rec(&without, 2045, 12).assessed_income, 6000.0, "without a care home");
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    let with = run(&v);
    let dec = rec(&with, 2045, 12);
    close(dec.assessed_income, 0.0, "taxable income 0");
    let b = dec.tax_basis.as_ref().expect("basis");
    close(b.care, 108_000.0, "costs of the year");
    close(b.care_canton, 105_000.0, "deduction");
    close(b.taxable_canton, 0.0, "not below 0");
    // a year with lower costs: 3'000 a month from 2046 (36'000 − 3'000 = 33'000 deduction)
    v["household"]["persons"][0]["careHome"] = care(86.0, 3000.0, false);
    let low = run(&v);
    let b = rec(&low, 2046, 12).tax_basis.clone().expect("basis");
    close(b.care_canton, 33_000.0, "deduction");
    close(b.taxable_canton, 27_000.0, "taxable income");
    close(rec(&low, 2046, 12).assessed_income, 2700.0, "tax");
}

/// #271/AK9: the 5 % refer to the net income after the other deductions (own tables, own deduction 20'000).
#[test]
fn issue_271_ak9_share_of_the_net_income_after_other_deductions() {
    let mut v = base();
    v["pension"]["ahv"]["monthly"] = json!(0);
    v["flows"]["income"] = json!(5000);
    v["taxes"]["income"] = json!([[0, 0.1]]);
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "own", "amount": 20_000 }]);
    v["household"]["persons"][0]["careHome"] = care(85.0, 3000.0, false);
    let r = run(&v);
    let b = rec(&r, 2045, 12).tax_basis.clone().expect("basis");
    // net income 60'000 − 20'000 = 40'000; deduction 36'000 − 2'000 = 34'000; taxable 6'000
    close(b.care_canton, 34_000.0, "deduction");
    close(b.taxable_canton, 6000.0, "taxable income");
    close(rec(&r, 2045, 12).assessed_income, 600.0, "tax");
}

/// #271/AK9: mode «Wohnort»: the deduction applies to canton and federal tax with their own net income.
#[test]
fn issue_271_ak9_deduction_in_the_mode_residence() {
    let mut v = base();
    v["pension"]["ahv"]["monthly"] = json!(0);
    v["flows"]["income"] = json!(5000);
    v["taxes"]["mode"] = json!("location");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    let without = run(&v);
    v["household"]["persons"][0]["careHome"] = care(85.0, 3000.0, false);
    let with = run(&v);
    let b0 = rec(&without, 2045, 12).tax_basis.clone().expect("basis");
    let b = rec(&with, 2045, 12).tax_basis.clone().expect("basis");
    let canton = 36_000.0 - 0.05 * b0.taxable_canton;
    let federal = 36_000.0 - 0.05 * b0.taxable_federal;
    close(b.care_canton, canton, "deduction canton");
    close(b.care_federal, federal, "deduction federal tax");
    close(
        b.taxable_canton,
        (b0.taxable_canton - canton).max(0.0),
        "taxable canton",
    );
    close(
        b.taxable_federal,
        (b0.taxable_federal - federal).max(0.0),
        "taxable federal",
    );
    assert!(rec(&with, 2045, 12).assessed_income < rec(&without, 2045, 12).assessed_income);
}

/// #271: the care home costs are part of the ledger (change of the total wealth).
#[test]
fn issue_271_ledger_balances() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [flat()] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, true);
    v["returns"]["fixed"]["inflation"] = json!(0.01);
    let r = run(&v);
    for w in r.records.windows(2) {
        close(w[1].total - w[0].total, w[1].ledger.sum(), "ledger");
    }
    assert!(r.records.iter().any(|x| x.ledger.care > 0.0));
}

/// #271/AK2: yearly values: total spending = living costs + housing + care home + pillar 3a (monthly average), the bars
/// sum the year; `care` is written only in years with care home costs.
#[test]
fn issue_271_ak2_yearly_values_add_the_care_home() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [rent("start")] });
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    // pillar 3a contribution 100 a month until before the AHV payout (2047-01)
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "ahv", "kind": "date", "date": "2047-01" }));
    v["pension"]["ahv"]["at"] = json!("ahv");
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 1200 }]);
    // earned income 1'000, a pension fund pension of 500 from the start and a flat income tax of 10 %, so every part
    // of the income and the taxes is not 0
    v["flows"]["income"] = json!(1000);
    v["pension"]["pk"] = json!({ "capital": 120_000, "at": "start", "lumpsum": 0, "conversion": 0.05 });
    v["taxes"]["income"] = json!([[0, 0.1]]);
    let r = run(&v);
    let yearly = engine::digest::yearly(&engine::digest::StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    });
    let year = |y: i32| yearly.nominal.iter().find(|x| x.year == y).unwrap();
    let (y44, y45) = (year(2044), year(2045));
    // 2044: living costs 4'000 + rent 2'000 + 3a 100
    close(y44.spending, 6100.0, "spending 2044");
    close(y44.flows.living, 48_000.0, "living 2044");
    close(y44.flows.housing, 24_000.0, "housing 2044");
    close(y44.flows.p3a, 1200.0, "3a 2044");
    close(y44.flows.care, 0.0, "care 2044");
    // 2045: living costs 4'000 + care home 9'000 + 3a 100, no rent
    close(y45.spending, 13_100.0, "spending 2045");
    close(y45.care, 9000.0, "care 2045");
    close(y45.flows.living, 48_000.0, "living 2045");
    close(y45.flows.housing, 0.0, "housing 2045");
    close(y45.flows.care, 108_000.0, "care bars 2045");
    close(y45.flows.p3a, 1200.0, "3a 2045");
    close(
        y45.flows.spending,
        y45.flows.living + y45.flows.housing + y45.flows.care + y45.flows.p3a + y45.flows.taxes,
        "spending bars 2045",
    );
    close(
        y45.flows.spending - y45.flows.taxes,
        48_000.0 + 108_000.0 + 1200.0,
        "spending bars without taxes",
    );
    close(y44.flows.spending, 48_000.0 + 24_000.0 + 1200.0, "spending bars 2044");
    assert!(y45.flows.taxes > 0.0);
    let y47 = year(2047);
    assert!(y47.flows.earned > 0.0 && y47.flows.pk > 0.0 && y47.flows.ahv > 0.0);
    close(
        y47.flows.income,
        y47.flows.earned + y47.flows.pk + y47.flows.ahv,
        "income bars",
    );
    close(
        y47.flows.earned + y47.flows.pk,
        12_000.0 + 6000.0,
        "earned and pension fund",
    );
    close(
        y47.flows.draw,
        y47.flows.spending - y47.flows.earned - y47.flows.pk - y47.flows.ahv,
        "withdrawal bars",
    );
    let row45 = serde_json::to_value(y45).unwrap();
    assert_eq!(row45["care"], json!(9000.0));
    assert_eq!(row45["flows"]["care"], json!(108_000.0));
    let row44 = serde_json::to_value(y44).unwrap();
    assert!(row44.get("care").is_none() && row44["flows"].get("care").is_none());
    // real values: the same without inflation
    let real = yearly.real.iter().find(|x| x.year == 2045).unwrap();
    close(real.care, 9000.0, "care real");
}

/// #271: the care home costs are part of the records as written for the user interface and of the net need of the
/// 3-bucket strategy.
#[test]
fn issue_271_records_and_net_need_of_the_buckets() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    v["withdrawal"] = json!({ "strategy": "three_buckets" });
    let r = run(&v);
    let json = serde_json::to_value(rec(&r, 2045, 1)).unwrap();
    assert_eq!(json["care"], json!(9000.0));
    let jan = |y: i32| {
        r.bucket_refills
            .iter()
            .find(|b| b.month == YearMonth::new(y, 1))
            .expect("refill in January")
    };
    let b44 = &jan(2044).basis;
    close(b44.care, 0.0, "no care home in 2044");
    let b = &jan(2045).basis;
    close(b.care, 108_000.0, "care home costs of the year");
    close(
        b.expenses(),
        b.living + b.housing + b.care + b.income_tax + b.pillar3a,
        "expenses with the care home",
    );
    close(
        b.expenses() - b44.expenses(),
        108_000.0,
        "the care home raises the need",
    );
    let s = serde_json::to_value(b).unwrap();
    assert_eq!(s["care"], json!(108_000.0));
    assert!(serde_json::to_value(b44).unwrap().get("care").is_none());
}

/// #271: the stock share as a curve over the funded ratio counts the care home costs in the remaining need.
#[test]
fn issue_271_funded_ratio_counts_the_care_home() {
    let mut v = base();
    v["wealth"] = json!({ "stocks": 1_000_000, "bonds": 1_000_000, "cash": 0, "reserveMonths": 0,
        "stockShare": { "mode": "curve", "input": "fundedRatio", "points": [[0.0, 0.0], [20.0, 1.0]], "eval": "yearly" } });
    let share = |r: &SimResult| {
        let x = rec(r, 2044, 1);
        x.stocks / (x.stocks + x.bonds)
    };
    let without = share(&run(&v));
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    let with = share(&run(&v));
    assert!(with < without - 0.01, "{with} vs {without}");
}

/// #271: net need of the 3-bucket strategy with guardrails, started in the middle of the year (extrapolated to 12
/// months): care home costs with inflation, their deduction in the estimated income tax, living costs of the
/// spending rule after the care home costs.
#[test]
fn issue_271_net_need_mid_year_with_inflation_taxes_and_guardrails() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["taxes"]["income"] = json!([[0, 0.1]]);
    // retirement 2045-07: the strategy starts then (case A, 6 months × 2)
    v["pension"]["pk"] = json!({ "capital": 480_000, "at": "2045-07", "lumpsum": 0, "conversion": 0.05 });
    v["withdrawal"] =
        json!({ "strategy": "three_buckets", "buckets": { "cashYears": 1, "bondYears": 3, "spending": "guardrails" } });
    let r = run(&v);
    let refill = r
        .bucket_refills
        .iter()
        .find(|b| b.month == YearMonth::new(2045, 7))
        .expect("start of the strategy");
    let b = &refill.basis;
    assert_eq!(b.months, 6);
    let jul = rec(&r, 2045, 7);
    close(b.care, 12.0 * 9000.0 * jul.price_index, "care home costs of the year");
    // income about 3'000 × 13 + 2'000 × 12 = 63'000 < care home costs: no income tax
    assert!(b.ahv + b.pk > 60_000.0, "{} {}", b.ahv, b.pk);
    close(b.income_tax, 0.0, "estimated income tax");
    // the spending rule: living costs of the month after the care home costs, the same in every month of the year
    let living = jul.spending - jul.housing - jul.care - jul.pillar3a_in;
    assert!(living > 0.0);
    assert!(
        (b.living - 12.0 * living).abs() < 1e-3,
        "{} vs {}",
        b.living,
        12.0 * living
    );
}

/// #271: the care home costs count in the uncovered spending of the month: «Standard» keeps 3 months of it as cash.
#[test]
fn issue_271_cash_reserve_covers_the_care_home() {
    let mut v = base();
    v["wealth"] = json!({ "stocks": 1_000_000, "bonds": 1_000_000, "cash": 0, "stockShare": 0.5, "reserveMonths": 3 });
    let without = run(&v);
    let x = rec(&without, 2045, 3);
    // uncovered: living costs 6'000 − AHV (with the supplement for the deferral)
    close(x.cash, 3.0 * (6000.0 - x.ahv), "reserve without a care home");
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    let with = run(&v);
    close(
        rec(&with, 2045, 3).cash,
        3.0 * (6000.0 + 9000.0 - x.ahv),
        "reserve with the care home",
    );
}

/// #271: withdrawal of the month (record, payment of the 3-bucket strategy, curve over the withdrawal rate) with the
/// care home costs and the housing costs.
#[test]
fn issue_271_withdrawal_includes_care_and_housing() {
    let mut v = base();
    v["flows"]["spendingIncludesHousing"] = json!(false);
    v["realestate"] = json!({ "properties": [rent("start")] });
    v["household"]["persons"][0]["careHome"] = care(84.5, 9000.0, false);
    v["withdrawal"] = json!({ "strategy": "three_buckets" });
    let r = run(&v);
    let jan = rec(&r, 2045, 1);
    close(jan.housing, 0.0, "no rent in the care home");
    let dec = rec(&r, 2044, 12);
    close(dec.withdrawal, dec.spending - dec.income, "withdrawal 2044");
    close(jan.withdrawal, jan.spending - jan.income, "withdrawal 2045");
    close(jan.spending, 6000.0 + 9000.0, "spending 2045");
    // payment of the month in the refill of January (no taxes, no one-off amounts)
    let payment = |y: i32| {
        r.bucket_refills
            .iter()
            .find(|b| b.month == YearMonth::new(y, 1))
            .expect("refill")
            .payment
    };
    close(payment(2044), rec(&r, 2044, 1).spending, "payment 2044 with the rent");
    close(payment(2045), jan.spending, "payment 2045 with the care home");

    // stock share as a curve over the withdrawal rate: more expenses, lower share
    let share_at = |v: &Value, y: i32, m: u32| {
        let mut v = v.clone();
        v["withdrawal"] = json!({ "strategy": "standard" });
        v["wealth"] = json!({ "stocks": 1_000_000, "bonds": 1_000_000, "cash": 0, "reserveMonths": 0,
            "stockShare": { "mode": "curve", "input": "withdrawalRate", "points": [[0.0, 1.0], [0.2, 0.0]], "eval": "monthly" } });
        let r = run(&v);
        let x = rec(&r, y, m);
        x.stocks / (x.stocks + x.bonds)
    };
    let share = |v: &Value| share_at(v, 2045, 12);
    let with_both = share(&v);
    let mut no_rent = v.clone();
    no_rent["realestate"] = json!({ "properties": [] });
    let mut no_care = v.clone();
    no_care["household"]["persons"][0]
        .as_object_mut()
        .unwrap()
        .remove("careHome");
    assert!(with_both < share(&no_care) - 1e-6, "care home");
    // the rent ends in the care home; it counts in 2044, so the share at the end of 2045 is still lower
    assert!(with_both < share(&no_rent) - 1e-6, "rent");
    // while the rent is paid (2044) it raises the withdrawal rate
    assert!(share_at(&v, 2044, 5) < share_at(&no_rent, 2044, 5) - 1e-6, "rent 2044");
}

/// #271: the payment of the 3-bucket strategy in a month with taxes contains the care home costs and the taxes.
#[test]
fn issue_271_bucket_payment_with_taxes() {
    let mut v = base();
    v["household"]["persons"][0]["careHome"] = care(85.0, 9000.0, false);
    v["taxes"]["income"] = json!([[0, 0.3]]);
    v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 0.05, "bondYears": 2 } });
    let r = run(&v);
    let with_tax: Vec<_> = r
        .bucket_refills
        .iter()
        .filter(|b| rec(&r, b.month.year(), b.month.month()).taxes > 0.0)
        .collect();
    assert!(
        !with_tax.is_empty(),
        "{:?}",
        r.bucket_refills.iter().map(|b| b.month).collect::<Vec<_>>()
    );
    for b in with_tax {
        let x = rec(&r, b.month.year(), b.month.month());
        close(b.payment, x.spending + x.taxes, "payment with taxes");
    }
}
