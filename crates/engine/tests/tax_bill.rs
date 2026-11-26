//! #274: Deductions from the last tax bill: deduction until the retirement, rest after it, retirement before the start,
//! taxable wealth, check with the total of the taxes, origin of the rows and errors at the fields. The plans are
//! made up.

use std::path::Path;

use engine::api;
use engine::digest::{self, StoredRun};
use engine::taxbill::{self, TaxBill, TaxBillOut, TaxBillResult};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn ym(s: &str) -> YearMonth {
    s.parse().unwrap()
}

/// New plan starting in `start` in Zurich with an earned income of 10'000 per month and wealth (person born 60 years
/// before the start, retirement at 65).
fn plan(start: &str) -> Value {
    let mut v = api::blank_plan(ym(start));
    v["household"]["persons"][0]["name"] = json!("Anna");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    v["flows"]["income"] = json!({ "mode": "const", "value": 10_000 });
    v["flows"]["spending"] = json!({ "mode": "const", "value": 5_000 });
    v["pension"]["ahv"]["monthly"] = json!(2_500);
    for (k, x) in [("stocks", 400_000), ("bonds", 200_000), ("cash", 100_000)] {
        v["wealth"][k] = json!(x);
    }
    v
}

fn bill(year: i32, total: f64, taxable: f64, wealth: f64, tax: Option<f64>) -> Value {
    json!({ "year": year, "totalIncome": total, "taxableIncome": taxable, "taxableWealth": wealth, "totalTax": tax })
}

fn take(v: &Value, b: &Value) -> TaxBillOut {
    taxbill::tax_bill(v, b)
}

fn result(v: &Value, b: &Value) -> TaxBillResult {
    let out = take(v, b);
    assert!(
        out.issues.is_empty() && out.error.is_none(),
        "{:?} {:?}",
        out.issues,
        out.error
    );
    out.result.expect("result")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 0.01, "{what}: {a} ≠ {b}");
}

/// Yearly values (nominal) of the preview of `v`.
fn yearly(v: &Value) -> Vec<digest::YearRow> {
    let p = serde_json::to_value(api::preview(v, Path::new("."), None)).unwrap();
    assert!(p["result"].is_object(), "{}", p["issues"]);
    let run: StoredRun = serde_json::from_value(p["result"].clone()).unwrap();
    digest::yearly(&run).nominal
}

/// #274/AK3: example by hand: income 120'000, taxable 95'000 → deduction 25'000; standard deduction with earned
/// income 21'000 → rest 4'000; standard deduction without earned income 9'000 → from the retirement 13'000. The
/// pillar 3a contributions are deducted separately and come off the own value; nothing is negative.
#[test]
fn issue_274_ak3_example_by_hand() {
    let (deduction, own, rest) = taxbill::amounts(120_000.0, 95_000.0, 0.0, Some(21_000.0));
    assert_eq!((deduction, own, rest), (25_000.0, 25_000.0, 4_000.0));
    assert_eq!(9_000.0 + rest, 13_000.0);
    assert_eq!(
        taxbill::amounts(120_000.0, 95_000.0, 7_000.0, Some(15_000.0)),
        (25_000.0, 18_000.0, 3_000.0)
    );
    // #274/AK2: never negative
    assert_eq!(
        taxbill::amounts(90_000.0, 95_000.0, 7_000.0, Some(21_000.0)),
        (0.0, 0.0, 0.0)
    );
    assert_eq!(
        taxbill::amounts(120_000.0, 95_000.0, 0.0, Some(30_000.0)),
        (25_000.0, 25_000.0, 0.0)
    );
    // #274/AK4: retired, no rest
    assert_eq!(
        taxbill::amounts(120_000.0, 95_000.0, 0.0, None),
        (25_000.0, 25_000.0, 0.0)
    );
}

/// #274/AK2, AK3: the row from the start is the own value (total of the income − taxable income, prices of the
/// start), the row from the retirement the proposal of the canton with the rest as additional amount; the simulation
/// with these rows deducts exactly that: until the retirement the own value, after it the standard deduction without
/// earned income plus the rest.
#[test]
fn issue_274_ak2_ak3_rows_until_and_from_the_retirement() {
    let v = plan("2026-01");
    let r = result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, None));
    assert_eq!(
        (r.year, r.deduction, r.own, r.pillar3a),
        (2025, 25_000.0, 25_000.0, 0.0)
    );
    let working = r.working.expect("standard deduction with earned income");
    assert_eq!(r.working_year, Some(2026));
    assert!(working > 2_000.0 && working < 25_000.0, "{working}");
    close(r.rest, 25_000.0 - working, "rest");
    assert_eq!(r.retired_year, Some(2031));
    let retired = r.retired.expect("standard deduction without earned income");
    assert!(retired > 0.0 && retired < working, "{retired} {working}");
    assert!(!r.retired_before_start);
    let rows = serde_json::to_value(&r.deductions).unwrap();
    assert_eq!(
        rows,
        json!([
            { "from": "start", "kind": "own", "amount": 25_000.0, "rate": 0.0, "taxBill": 2025 },
            { "from": "retire", "kind": "canton", "amount": 0.0, "rate": 0.0, "extra": r.rest, "taxBill": 2025 },
        ])
    );
    // the simulation with the rows taken over (#274/AK7: the origin in the yearly values)
    let mut v = v;
    v["taxes"]["deductions"] = rows;
    let years = yearly(&v);
    let at = |y: i32| years.iter().find(|r| r.year == y).expect("year");
    let price = |y: i32| {
        let row = at(y);
        row.basis.as_ref().map(|b| b.own).unwrap_or_default() / 25_000.0
    };
    let b = at(2027).basis.as_ref().expect("basis 2027");
    assert_eq!((b.proposal_canton, b.extra, b.tax_bill), (0.0, 0.0, Some(2025)));
    assert!(b.own > 25_000.0, "own value grows with the prices: {}", b.own);
    let index = price(2027);
    close(b.taxable_canton, b.income - 25_000.0 * index, "until the retirement");
    let b = at(2033).basis.as_ref().expect("basis 2033");
    assert_eq!((b.own, b.tax_bill), (0.0, Some(2025)));
    assert!(b.proposal_canton > 0.0 && b.extra > 0.0, "{b:?}");
    // retired standard deduction and rest of the prices of the start grow with the same index
    close(
        b.extra / b.proposal_canton,
        r.rest / retired,
        "rest and standard deduction in the same prices",
    );
    close(
        b.taxable_canton,
        b.income - b.proposal_canton - b.extra,
        "from the retirement",
    );
}

/// #274/AK2: with pillar 3a contributions (deducted separately) the own value is the deduction without them; the
/// sum of both is the deduction of the tax bill.
#[test]
fn issue_274_ak2_pillar3a_counts_once() {
    let mut v = plan("2026-01");
    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0.01, "contribution": 7_000 }]);
    let r = result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, None));
    assert!(r.pillar3a > 6_000.0 && r.pillar3a <= 7_258.0, "{}", r.pillar3a);
    close(r.own + r.pillar3a, 25_000.0, "own value + pillar 3a");
    close(r.rest, (r.own - r.working.unwrap()).max(0.0), "rest without pillar 3a");
}

/// #274/AK4: retirement before the start: one row from the start with the deduction, no rest.
#[test]
fn issue_274_ak4_retired_before_the_start() {
    let mut v = plan("2026-07");
    v["household"]["persons"][0]["birth"] = json!("1955-03");
    v["flows"]["income"] = json!({ "mode": "const", "value": 0 });
    let r = result(&v, &bill(2025, 80_000.0, 62_000.0, 600_000.0, None));
    assert!(r.retired_before_start);
    assert_eq!((r.deduction, r.own, r.rest, r.working), (18_000.0, 18_000.0, 0.0, None));
    assert_eq!(
        serde_json::to_value(&r.deductions).unwrap(),
        json!([{ "from": "start", "kind": "own", "amount": 18_000.0, "rate": 0.0, "taxBill": 2025 }])
    );
}

/// #274/AK3: in a couple the row from the retirement starts with the payout of the person who retires last.
#[test]
fn issue_274_ak3_couple_row_from_the_last_retirement() {
    let mut v = plan("2026-01");
    let a = v["household"]["persons"][0].clone();
    v["household"]["persons"] = json!([a, { "id": "B", "name": "Ben", "birth": "1970-01" }]);
    v["household"]["married"] = json!(true);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "income": 4000, "incomeEndsAtRetirement": true,
        "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv2" } });
    let r = result(&v, &bill(2025, 200_000.0, 150_000.0, 600_000.0, None));
    assert_eq!(r.deductions[1].from.0, "retire2");
    // B retires in January 2035: the first full year without earned income
    assert_eq!(r.retired_year, Some(2035));
    // the standard deduction with earned income of the household includes the second-earner deduction
    let single = result(&plan("2026-01"), &bill(2025, 200_000.0, 150_000.0, 600_000.0, None));
    assert!(
        r.working.unwrap() > single.working.unwrap() + 5_000.0,
        "{:?} {:?}",
        r.working,
        single.working
    );
    // later person first: the milestone of person A
    v["household"]["persons"][1]["birth"] = json!("1960-01");
    let r = result(&v, &bill(2025, 200_000.0, 150_000.0, 600_000.0, None));
    assert_eq!(r.deductions[1].from.0, "retire");
}

/// #274/AK9: errors at the fields: tax year after the start, negative amounts, missing values; the total of the taxes
/// is optional.
#[test]
fn issue_274_ak9_errors_at_the_fields() {
    let v = plan("2026-07");
    let errors = |b: Value| -> Vec<(String, String)> {
        take(&v, &b)
            .issues
            .into_iter()
            .map(|i| (i.field.unwrap(), i.message.code().to_string()))
            .collect()
    };
    assert_eq!(
        errors(bill(2027, 1.0, 1.0, 1.0, None)),
        vec![("year".into(), "tax_bill_year_after_start".into())]
    );
    assert!(
        errors(bill(2026, 1.0, 1.0, 1.0, None)).is_empty(),
        "the start year is allowed"
    );
    assert_eq!(
        errors(bill(2025, -1.0, 0.0, 0.0, Some(-5.0))),
        vec![
            ("totalIncome".into(), "tax_bill_negative".into()),
            ("totalTax".into(), "tax_bill_negative".into())
        ]
    );
    assert_eq!(
        errors(json!({ "taxableIncome": 0 })),
        vec![
            ("year".into(), "tax_bill_missing".into()),
            ("totalIncome".into(), "tax_bill_missing".into()),
            ("taxableWealth".into(), "tax_bill_missing".into())
        ]
    );
    let out = take(&v, &bill(2027, 1.0, 1.0, 1.0, None));
    assert!(out.result.is_none());
    // the same check without a plan (start month given)
    let b = TaxBill {
        year: Some(2026),
        total_income: Some(0.0),
        taxable_income: Some(0.0),
        taxable_wealth: Some(0.0),
        total_tax: None,
    };
    assert!(taxbill::check(&b, ym("2026-01")).is_empty());
    assert_eq!(taxbill::check(&b, ym("2025-12")).len(), 1);
}

/// #274/AK8: only in the mode «Wohnort»: with own tables the takeover is an error; without a municipality too.
#[test]
fn issue_274_ak8_only_with_the_place_of_residence() {
    let mut v = plan("2026-01");
    v["taxes"]["mode"] = json!("own");
    let out = take(&v, &bill(2025, 1.0, 1.0, 1.0, None));
    assert_eq!(out.error.map(|m| m.code().to_string()), Some("tax_bill_mode".into()));
    let mut v = plan("2026-01");
    v["taxes"]["location"] = json!({ "canton": "", "church": "none" });
    let out = take(&v, &bill(2025, 1.0, 1.0, 1.0, None));
    assert!(out.error.is_some() && out.result.is_none());
    // an invalid plan: error, no result
    let out = take(&json!({ "nonsense": true }), &bill(2025, 1.0, 1.0, 1.0, None));
    assert!(out.error.is_some());
}

/// #274/AK6: with the total of the taxes the own estimate for the same taxable values and the deviation; from 15 %
/// a hint. The estimate does not depend on the plan's income.
#[test]
fn issue_274_ak6_check_with_the_total_of_the_taxes() {
    let v = plan("2026-01");
    assert!(result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, None))
        .check
        .is_none());
    let c = result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, Some(10_000.0)))
        .check
        .expect("check");
    assert!(c.estimate > 5_000.0 && c.estimate < 40_000.0, "{}", c.estimate);
    close(c.deviation, c.estimate / 10_000.0 - 1.0, "deviation");
    let at = |total: f64| {
        result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, Some(total)))
            .check
            .expect("check")
    };
    let small = at(c.estimate / 1.14);
    assert!(!small.hint && (small.deviation - 0.14).abs() < 1e-9, "{small:?}");
    assert!(at(c.estimate / 1.16).hint);
    assert!(at(c.estimate / 0.84).hint);
    let mut other = v.clone();
    other["flows"]["income"] = json!({ "mode": "const", "value": 3_000 });
    let c2 = result(&other, &bill(2025, 120_000.0, 95_000.0, 600_000.0, Some(10_000.0)))
        .check
        .expect("check");
    close(c2.estimate, c.estimate, "same taxable values, same estimate");
    // a higher taxable wealth raises the estimate (wealth tax)
    let c3 = result(&v, &bill(2025, 120_000.0, 95_000.0, 900_000.0, Some(10_000.0)))
        .check
        .expect("check");
    assert!(c3.estimate > c.estimate);
}

/// #274/AK5: taxable wealth: the plan's value on 31 December of the first year after the social deductions, the
/// difference and the proposal: without properties a deduction from wealth if the tax bill is lower, none if it is
/// higher or about the same.
#[test]
fn issue_274_ak5_wealth_without_properties() {
    let v = plan("2026-01");
    let w = result(&v, &bill(2025, 1.0, 1.0, 0.0, None)).wealth;
    assert_eq!(w.year, 2026);
    assert!(
        w.allowance > 75_000.0 && w.allowance < 100_000.0,
        "ZH single: {}",
        w.allowance
    );
    assert!(w.plan > 500_000.0, "{}", w.plan);
    let lower = result(&v, &bill(2025, 1.0, 1.0, w.plan - 100_000.0, None)).wealth;
    close(lower.difference, -100_000.0, "difference");
    assert_eq!(
        serde_json::to_value(&lower.proposal).unwrap(),
        json!({ "kind": "wealthDeduction", "amount": 100_000.0 })
    );
    assert!(result(&v, &bill(2025, 1.0, 1.0, w.plan + 50_000.0, None))
        .wealth
        .proposal
        .is_none());
    assert!(result(&v, &bill(2025, 1.0, 1.0, w.plan - 500.0, None))
        .wealth
        .proposal
        .is_none());
}

/// #274/AK5: with an owned property the proposal is the share of the tax value of the market value that explains the
/// difference (between 0 and 100 %).
#[test]
fn issue_274_ak5_wealth_with_a_property() {
    let mut v = plan("2026-01");
    v["realestate"]["properties"] = json!([{ "id": "h", "name": "Haus", "status": "owned", "price": 800_000,
        "worth": 1_000_000, "fixCosts": 0, "sell": "never", "bought": "2010-01",
        "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": "2026-01", "periods": [
            { "type": "fixed", "term": 20, "rate": 0.0, "amort": 0 } ] } ] } }]);
    let w = result(&v, &bill(2025, 1.0, 1.0, 0.0, None)).wealth;
    // default 50 % of the market value: the tax value 500'000 explains a difference of 100'000 per 10 %
    let rate = |bill: f64| match result(
        &v,
        &json!({ "year": 2025, "totalIncome": 1, "taxableIncome": 1,
        "taxableWealth": bill }),
    )
    .wealth
    .proposal
    {
        Some(taxbill::WealthProposal::TaxValueRate { rate, properties }) => {
            assert_eq!(properties, vec![0]);
            rate
        }
        p => panic!("{p:?}"),
    };
    let r = rate(w.plan - 200_000.0);
    assert!((r - 0.3).abs() < 0.02, "lower by 200'000: {r}");
    let r = rate(w.plan + 100_000.0);
    assert!((r - 0.6).abs() < 0.02, "higher by 100'000: {r}");
    assert_eq!(rate(0.0), 0.0, "never below 0 %");
    assert_eq!(rate(w.plan + 5_000_000.0), 1.0, "never above 100 %");
}

/// #274: the social deductions from wealth estimated from the wealth curve.
#[test]
fn issue_274_ak5_wealth_allowance_from_the_curve() {
    let resolved = |wealth: Vec<(f64, f64)>| engine::taxdata::Resolved {
        cantonal: vec![],
        federal: vec![],
        wealth,
        capital: vec![],
        canton_deductions: Default::default(),
        federal_deductions: Default::default(),
    };
    // tax 100 at 100'000 and 300 at 150'000: marginal 0.4 ‰ → no tax below 75'000
    let r = resolved(vec![
        (0.0, 0.0),
        (50_000.0, 0.0),
        (100_000.0, 0.001),
        (150_000.0, 0.002),
    ]);
    close(r.wealth_allowance(), 75_000.0, "traced back");
    // never below the last grid point without a rate
    let r = resolved(vec![
        (0.0, 0.0),
        (90_000.0, 0.0),
        (100_000.0, 0.001),
        (150_000.0, 0.002),
    ]);
    close(r.wealth_allowance(), 90_000.0, "clamped");
    // without a next grid point: the last without a rate
    let r = resolved(vec![(0.0, 0.0), (50_000.0, 0.0), (100_000.0, 0.001)]);
    close(r.wealth_allowance(), 50_000.0, "no marginal rate");
    // a rate from the first grid point: none; no rate at all: the whole curve
    close(
        resolved(vec![(0.0, 0.001), (1.0, 0.001)]).wealth_allowance(),
        0.0,
        "rate from 0",
    );
    close(
        resolved(vec![(0.0, 0.0), (80_000.0, 0.0)]).wealth_allowance(),
        80_000.0,
        "no rate",
    );
}

/// #274/AK3: a pension fund payout at a date instead of the milestone: the row from the retirement starts at that
/// month.
#[test]
fn issue_274_ak3_row_from_a_payout_date() {
    let mut v = plan("2026-01");
    v["pension"]["pk"]["at"] = json!("2032-06");
    let r = result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, None));
    assert_eq!(r.deductions[1].from.0, "2032-06");
    assert_eq!(r.retired_year, Some(2033));
}

/// Real (prices of the start) and nominal yearly values of the preview of `v`.
fn yearly_both(v: &Value) -> digest::Yearly {
    let p = serde_json::to_value(api::preview(v, Path::new("."), None)).unwrap();
    let run: StoredRun = serde_json::from_value(p["result"].clone()).unwrap();
    digest::yearly(&run)
}

/// #274/AK3: the standard deduction with earned income and the pillar 3a come from the first full year with earned
/// income, in prices of the start (with «tariffs with inflation»), else nominal.
#[test]
fn issue_274_ak3_working_year_and_prices() {
    let mut v = plan("2026-07");
    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0.01, "contribution": 7_000 }]);
    let r = result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, None));
    assert_eq!(r.working_year, Some(2027), "first full year");
    let y = yearly_both(&v);
    let real = y.real.iter().find(|r| r.year == 2027).unwrap().basis.clone().unwrap();
    close(
        r.working.unwrap(),
        real.proposal_canton,
        "standard deduction in prices of the start",
    );
    close(r.pillar3a, real.pillar3a, "pillar 3a in prices of the start");
    // without «tariffs with inflation» the amounts of the year
    v["taxes"]["indexed"] = json!(false);
    let r = result(&v, &bill(2025, 120_000.0, 95_000.0, 600_000.0, None));
    let nominal = yearly_both(&v)
        .nominal
        .into_iter()
        .find(|r| r.year == 2027)
        .unwrap()
        .basis
        .unwrap();
    close(r.working.unwrap(), nominal.proposal_canton, "nominal");
    assert!(
        nominal.proposal_canton > real.proposal_canton,
        "the earned income grows with the prices"
    );
    // no earned income at all: the first year of the plan
    let mut v = plan("2026-07");
    v["flows"]["income"] = json!({ "mode": "const", "value": 0 });
    let r = result(&v, &bill(2025, 50_000.0, 40_000.0, 600_000.0, None));
    assert_eq!(r.working_year, Some(2026));
}

/// #274/AK5, AK6: the plan's taxable wealth is the one of the composition (#249) minus the social deductions; the
/// estimate is the income tax and the wealth tax (with the social deductions) of the place of residence; without a
/// total of the taxes (or 0) no check.
#[test]
fn issue_274_ak5_ak6_values_by_hand() {
    let v = plan("2026-01");
    let out = api::preview(&v, Path::new("."), None);
    let r = result(&v, &bill(2025, 120_000.0, 95_000.0, 450_000.0, Some(10_000.0)));
    close(
        r.wealth.plan,
        out.wealth_tax[0].taxable - r.wealth.allowance,
        "plan after the social deductions",
    );
    close(r.wealth.difference, 450_000.0 - r.wealth.plan, "difference");
    let loc: engine::model::TaxLocation =
        serde_json::from_value(json!({ "canton": "ZH", "municipality": 261, "church": "none" })).unwrap();
    let z = engine::taxdata::builtin().resolve(&loc, false).unwrap();
    let expected = z.income_tax_on(95_000.0, 95_000.0, 1.0) + z.wealth_tax(450_000.0 + r.wealth.allowance, 1.0);
    close(r.check.unwrap().estimate, expected, "estimate");
    assert!(result(&v, &bill(2025, 120_000.0, 95_000.0, 450_000.0, Some(0.0)))
        .check
        .is_none());
}
