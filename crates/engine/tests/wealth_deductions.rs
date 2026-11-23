//! #231: Deductions from wealth: time-dependent rows and one-off deductions, on 31 December of the tax year.
//!
//! Made-up plan that can be recomputed by hand: start 2026-01, end 2028-12, no returns, no inflation, no income, no
//! spending, cash 800'000, own tables without income tax and with a wealth tax of 0 % (wealth stays exactly 800'000)
//! or of a flat 1 % (tax = 1 % of the taxable wealth).

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator, WealthBasis};
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

fn base(wealth_rate: f64) -> Value {
    json!({
        "project": { "name": "Abzüge vom Vermögen" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1970-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2026-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2028-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 0, "spending": 0 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 800_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.0]], "wealth": [[0, wealth_rate]], "capital": [[0, 0.0]],
                   "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

/// AK4 example: ongoing 50'000 from the start, one-off 100'000 in 2027.
fn example(wealth_rate: f64) -> Value {
    let mut v = base(wealth_rate);
    v["taxes"]["wealthDeductions"] = json!([{ "from": "start", "amount": 50_000 }]);
    v["taxes"]["oneOffWealthDeductions"] = json!([{ "year": 2027, "label": "Sanierung", "amount": 100_000 }]);
    v
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

fn wealth(r: &SimResult, y: i32) -> WealthBasis {
    december(r, y)
        .tax_basis
        .as_ref()
        .and_then(|b| b.wealth)
        .expect("basis of the wealth tax")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

fn errors(v: &Value) -> Vec<(String, String)> {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    validate(&plan)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| (i.section.to_string(), i.field.clone().unwrap_or_default()))
        .collect()
}

/// #231/AK4, AK5: Wealth 800'000, ongoing 50'000, one-off 100'000 in 2027 → taxable 2027 650'000, otherwise
/// 750'000; the basis shows the deductions by origin.
#[test]
fn issue_231_ak4_hand_value_example() {
    let r = run(&example(0.0));
    for (y, one_off, taxable) in [
        (2026, 0.0, 750_000.0),
        (2027, 100_000.0, 650_000.0),
        (2028, 0.0, 750_000.0),
    ] {
        let w = wealth(&r, y);
        close(w.wealth, 800_000.0, &format!("wealth {y}"));
        close(w.ongoing, 50_000.0, &format!("ongoing {y}"));
        close(w.one_off, one_off, &format!("one-off {y}"));
        close(w.taxable, taxable, &format!("taxable {y}"));
    }
}

/// #231/AK3, AK4: With own tables the amounts are the whole deduction: flat 1 % on 750'000 = 7'500 (without
/// deductions 8'000), in 2027 on 650'000 = 6'500 minus 1 % of the tax paid in May (7'500 → 75).
#[test]
fn issue_231_ak3_own_tables_whole_deduction() {
    let r = run(&example(0.01));
    close(december(&r, 2026).assessed_wealth, 7500.0, "tax 2026");
    close(december(&r, 2027).assessed_wealth, 6500.0 - 75.0, "tax 2027");
    close(
        december(&run(&base(0.01)), 2026).assessed_wealth,
        8000.0,
        "without deductions",
    );
}

/// #231/AK4: Never negative: a deduction larger than the wealth gives 0 and no tax.
#[test]
fn issue_231_ak4_never_negative() {
    let mut v = base(0.01);
    v["taxes"]["wealthDeductions"] = json!([{ "from": "start", "amount": 1_000_000 }]);
    let r = run(&v);
    assert_eq!(wealth(&r, 2026).taxable, 0.0);
    assert_eq!(december(&r, 2026).assessed_wealth, 0.0);
}

/// #231/AK4: The row of 31 December applies for the whole tax year (a row from 2027-12 already in 2027, one from
/// 2028-01 not); several one-off deductions in the same year are added.
#[test]
fn issue_231_ak4_row_of_december_and_one_offs_added() {
    let mut v = base(0.0);
    v["taxes"]["wealthDeductions"] = json!([
        { "from": "start", "amount": 10_000 },
        { "from": "2027-12", "amount": 20_000 },
        { "from": "2028-01", "amount": 30_000 }
    ]);
    v["taxes"]["oneOffWealthDeductions"] = json!([
        { "year": 2028, "label": "a", "amount": 1000 },
        { "year": 2028, "label": "b", "amount": 2000 }
    ]);
    let r = run(&v);
    close(wealth(&r, 2026).ongoing, 10_000.0, "2026");
    close(wealth(&r, 2027).ongoing, 20_000.0, "2027");
    close(wealth(&r, 2028).ongoing, 30_000.0, "2028");
    close(wealth(&r, 2028).one_off, 3000.0, "one-off 2028");
    close(wealth(&r, 2028).taxable, 800_000.0 - 33_000.0, "taxable 2028");
}

/// #231/AK4: Only one-off deductions (no row): the other years have the basis with no deduction; a row that starts
/// after December does not count in that year.
#[test]
fn issue_231_ak4_one_off_without_row() {
    let mut v = base(0.0);
    v["taxes"]["oneOffWealthDeductions"] = json!([{ "year": 2027, "label": "", "amount": 100_000 }]);
    let r = run(&v);
    assert_eq!(wealth(&r, 2026).ongoing, 0.0);
    close(wealth(&r, 2026).taxable, 800_000.0, "2026");
    close(wealth(&r, 2027).taxable, 700_000.0, "2027");
    let mut v = base(0.0);
    v["taxes"]["wealthDeductions"] = json!([{ "from": "2027-01", "amount": 50_000 }]);
    let r = run(&v);
    assert_eq!(wealth(&r, 2026).ongoing, 0.0);
    close(wealth(&r, 2027).taxable, 750_000.0, "2027");
}

/// #231: With «tariffs with inflation» (#66) the ongoing amount grows with the price index, the one-off stays nominal.
#[test]
fn issue_231_ongoing_indexed_one_off_nominal() {
    let mut v = example(0.0);
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["taxes"]["indexed"] = json!(true);
    let r = run(&v);
    let idx = december(&r, 2027).price_index;
    assert!(idx > 1.0);
    close(wealth(&r, 2027).ongoing, 50_000.0 * idx, "ongoing indexed");
    close(wealth(&r, 2027).one_off, 100_000.0, "one-off nominal");
}

/// #231/AK2: In the mode «Wohnort» the deductions come on top of the social deductions of the canton in the wealth
/// curves: 800'000 with 50'000 deductions is taxed like 750'000 without (Zurich).
#[test]
fn issue_231_ak2_location_on_top_of_canton() {
    let location = |cash: f64, deduction: Option<f64>| {
        let mut v = base(0.0);
        v["wealth"]["cash"] = json!(cash);
        v["taxes"]["mode"] = json!("location");
        v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
        if let Some(d) = deduction {
            v["taxes"]["wealthDeductions"] = json!([{ "from": "start", "amount": d }]);
        }
        december(&run(&v), 2026).assessed_wealth
    };
    let with = location(800_000.0, Some(50_000.0));
    let without = location(800_000.0, None);
    assert!(with > 0.0 && with < without, "{with} < {without}");
    close(with, location(750_000.0, None), "like 750'000 without deductions");
}

/// #231/AK6: Negative amounts and a tax year outside the simulation are errors at the field.
#[test]
fn issue_231_ak6_invalid_values_are_errors() {
    let mut v = base(0.0);
    v["taxes"]["wealthDeductions"] = json!([{ "from": "start", "amount": 0 }, { "from": "2027-01", "amount": -1 }]);
    assert_eq!(
        errors(&v),
        vec![("taxes".to_string(), "taxes.wealthDeductions.1.amount".to_string())]
    );
    let mut v = base(0.0);
    v["taxes"]["wealthDeductions"] = json!([{ "from": "nirgends", "amount": 0 }]);
    assert_eq!(errors(&v).len(), 1, "unknown point in time");
    for (row, field) in [
        (json!({ "year": 2026, "label": "", "amount": -1 }), "amount"),
        (json!({ "year": 2025, "label": "", "amount": 1 }), "year"),
        (json!({ "year": 2029, "label": "", "amount": 1 }), "year"),
    ] {
        let mut v = base(0.0);
        v["taxes"]["oneOffWealthDeductions"] = json!([{ "year": 2028, "label": "", "amount": 0 }, row]);
        assert_eq!(
            errors(&v),
            vec![("taxes".to_string(), format!("taxes.oneOffWealthDeductions.1.{field}"))]
        );
    }
    let mut v = base(0.0);
    v["taxes"]["oneOffWealthDeductions"] = json!([{ "year": 2026, "amount": 1 }, { "year": 2028, "amount": 1 }]);
    assert!(errors(&v).is_empty(), "first and last year of the simulation");
}

/// #231/AK7: Plans without deductions from wealth compute as before (no basis of the wealth tax, same tax) and keep
/// the new fields out of the file.
#[test]
fn issue_231_ak7_existing_plans_unchanged() {
    let v = base(0.01);
    let r = run(&v);
    assert!(december(&r, 2026).tax_basis.is_none());
    close(december(&r, 2026).assessed_wealth, 8000.0, "tax as before");
    let plan: Plan = serde_json::from_value(v).unwrap();
    let out = serde_json::to_value(&plan).unwrap();
    assert!(out["taxes"].get("wealthDeductions").is_none());
    assert!(out["taxes"].get("oneOffWealthDeductions").is_none());
    let back: Plan = serde_json::from_value(example(0.0)).unwrap();
    let out = serde_json::to_value(&back).unwrap();
    assert_eq!(out["taxes"]["wealthDeductions"][0]["amount"], json!(50_000.0));
    assert_eq!(out["taxes"]["oneOffWealthDeductions"][0]["year"], json!(2027));
}
