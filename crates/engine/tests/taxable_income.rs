//! #281: Taxable income of the tax year in the yearly values, next to the taxes.
//!
//! Made-up plan that can be recomputed by hand: start 2030-01, end 2032-06 (the last year without a December), no
//! returns, earned income 100'000 per year, own tables with a flat income tax of 10 %, no wealth tax.

use engine::digest::{yearly, StoredRun, Yearly};
use engine::model::Plan;
use engine::sim::{SimResult, Simulator};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Steuerbar" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2032-06", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 100_000.0 / 12.0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [], "capital": [[0, 0.0]],
                   "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let s = Simulator::new(&plan).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn years(v: &Value) -> Yearly {
    let r = run(v);
    yearly(&StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    })
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

/// #281/AK1, AK2: the yearly values carry the taxable income of the tax year (canton), separate from the income:
/// without deductions it is the income, with an own deduction of 10'000 it is 90'000 and the tax 10 % of it.
#[test]
fn issue_281_ak2_taxable_income_of_the_tax_year() {
    let y = years(&base());
    close(y.nominal[0].taxable_income, 100_000.0, "without deductions");
    close(y.nominal[0].tax_income, 10_000.0, "tax");
    let mut v = base();
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "own", "amount": 10_000 }]);
    let y = years(&v);
    let row = &y.nominal[0];
    close(row.taxable_income, 90_000.0, "with an own deduction");
    close(row.tax_income, 9_000.0, "tax on the taxable income");
    // the value is that of the canton in the assessment
    close(row.basis.as_ref().unwrap().taxable_canton, row.taxable_income, "canton");
    // the income (receipts) stays the same with and without deductions
    close(
        row.flows.income,
        years(&base()).nominal[0].flows.income,
        "income unchanged",
    );
    let json = serde_json::to_value(row).unwrap();
    close(json["taxableIncome"].as_f64().unwrap(), 90_000.0, "field taxableIncome");
}

/// #281/AK2: in the mode «Wohnort» canton and federal tax differ; the row shows the canton, the basis both.
#[test]
fn issue_281_ak2_canton_value_with_federal_in_the_basis() {
    let mut v = base();
    v["taxes"]["mode"] = json!("location");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    let y = years(&v);
    let row = &y.nominal[0];
    let b = row.basis.as_ref().expect("basis");
    close(row.taxable_income, b.taxable_canton, "canton");
    assert!(b.taxable_canton > 0.0 && (b.taxable_canton - b.taxable_federal).abs() > 0.5);
}

/// #281/AK3: a year without an assessment (the last year ends in June) and a year without taxable income show 0.
#[test]
fn issue_281_ak3_zero_without_an_assessment() {
    let y = years(&base());
    assert_eq!(y.nominal.last().unwrap().year, 2032);
    assert_eq!(y.nominal.last().unwrap().taxable_income, 0.0);
    let json = serde_json::to_value(y.nominal.last().unwrap()).unwrap();
    assert_eq!(json["taxableIncome"], json!(0.0));
    let mut v = base();
    v["flows"]["income"] = json!(0);
    let y = years(&v);
    assert_eq!(y.nominal[0].taxable_income, 0.0);
    assert_eq!(y.nominal[0].tax_total, 0.0);
}

/// #281/AK4: real values in prices of the start like the other amounts: with 2 % inflation the taxable income of
/// December 2030 divided by the price index of December.
#[test]
fn issue_281_ak4_real_like_the_other_amounts() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    let r = run(&v);
    let dec = r.records.iter().find(|m| m.month.is_december()).unwrap();
    assert!(dec.price_index > 1.0);
    let y = yearly(&StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    });
    close(y.nominal[0].taxable_income, dec.assessed_taxable, "nominal");
    close(y.real[0].taxable_income, dec.assessed_taxable / dec.price_index, "real");
}

/// #281/AK2: results saved before #281 have no `assessedTaxable`; with an assessment the yearly values take the value
/// of the canton from the basis.
#[test]
fn issue_281_ak2_older_results_take_the_basis() {
    let mut v = base();
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "own", "amount": 10_000 }]);
    let mut r = run(&v);
    for m in &mut r.records {
        m.assessed_taxable = 0.0;
    }
    let y = yearly(&StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    });
    close(y.nominal[0].taxable_income, 90_000.0, "from the basis");
    // without a basis (plan without deductions) it stays 0
    let mut r = run(&base());
    for m in &mut r.records {
        m.assessed_taxable = 0.0;
    }
    let y = yearly(&StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    });
    assert_eq!(y.nominal[0].taxable_income, 0.0);
}
