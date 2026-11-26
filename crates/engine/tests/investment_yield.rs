//! #149: Taxable yield of the wealth: per asset class (stocks, bonds) a rate per year on the balance; the yield of a
//! year counts to the taxable income (both tax modes, before the deductions), is part of the return (no additional
//! inflow) and raises the net need of the 3-bucket strategy through the income tax.
//!
//! Made-up plan: person A born 1965-01, start 2030-01, end 2032-12, stocks 500 000, bonds 300 000, no cash (cash has
//! no yield, #149 decision of 25.11.2026), no returns and no inflation (except where noted), income tax 10 % flat, no wealth tax.

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Ertrag" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2032-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "start", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "start" } },
        "flows": { "spending": 0, "spendingIncludesHousing": true },
        "wealth": { "stocks": 500_000, "bonds": 300_000, "cash": 0, "stockShare": 0.625, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [], "capital": [[0, 0]],
                   "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

/// `base` with the yields of the decision (stocks 2 %, bonds 1 %).
fn with_yield(mut v: Value) -> Value {
    v["taxes"]["investmentYield"] = json!({ "stocks": 0.02, "bonds": 0.01 });
    v
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
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

/// #149/AK1: the rates are fields of the section taxes; missing they are 0 (existing plans compute unchanged), and the
/// file writes them only if one is set.
#[test]
fn issue_149_ak1_rates_per_asset_class() {
    let p = plan(&with_yield(base()));
    close(p.taxes.investment_yield.stocks, 0.02, "stocks");
    close(p.taxes.investment_yield.bonds, 0.01, "bonds");
    let old = plan(&base());
    assert!(old.taxes.investment_yield.is_zero());
    let written = serde_json::to_value(&old).unwrap();
    assert!(written["taxes"].get("investmentYield").is_none());
    let written = serde_json::to_value(&p).unwrap();
    assert_eq!(
        written["taxes"]["investmentYield"],
        json!({ "stocks": 0.02, "bonds": 0.01 })
    );
    // one rate alone is written too
    for (stocks, bonds) in [(0.02, 0.0), (0.0, 0.01)] {
        let mut v = base();
        v["taxes"]["investmentYield"] = json!({ "stocks": stocks, "bonds": bonds });
        let p = plan(&v);
        assert!(!p.taxes.investment_yield.is_zero());
        let written = serde_json::to_value(&p).unwrap();
        assert_eq!(
            written["taxes"]["investmentYield"],
            json!({ "stocks": stocks, "bonds": bonds })
        );
    }
}

/// #149/AK2: example of the issue by hand: stocks 500 000 × 2 % + bonds 300 000 × 1 % = 13 000,
/// taxed with the income: 10 % = 1 300.
#[test]
fn issue_149_ak2_example_by_hand() {
    let r = run(&with_yield(base()));
    let dec = rec(&r, 2030, 12);
    close(dec.assessed_yield, 13_000.0, "taxable yield");
    close(dec.assessed_income, 1_300.0, "income tax");
    // only in the month of the assessment
    assert_eq!(rec(&r, 2030, 11).assessed_yield, 0.0);
    // without the rates: no yield, no income tax (as before #149)
    let r0 = run(&base());
    assert_eq!(rec(&r0, 2030, 12).assessed_yield, 0.0);
    assert_eq!(rec(&r0, 2030, 12).assessed_income, 0.0);
}

/// #149/AK2: the yield is the average balance of the year × rate: a plan starting in July has half a year of balances.
#[test]
fn issue_149_ak2_partial_year_counts_its_months() {
    let mut v = with_yield(base());
    v["milestones"][0]["date"] = json!("2030-07");
    let r = run(&v);
    close(rec(&r, 2030, 12).assessed_yield, 6_500.0, "half a year");
}

/// #149/AK2, AK6: the yearly values show the yield of the tax year (column «Steuerbarer Ertrag»), nominal and real.
#[test]
fn issue_149_ak6_yearly_values_show_the_yield() {
    let r = run(&with_yield(base()));
    let stored = engine::digest::StoredRun {
        summary: r.summary.clone(),
        records: r.records.clone(),
    };
    let y = engine::digest::yearly(&stored);
    close(y.nominal[0].tax_yield, 13_000.0, "nominal");
    close(y.real[0].tax_yield, 13_000.0, "real without inflation");
    close(y.nominal[0].tax_income, 1_300.0, "income tax");
    let json = serde_json::to_value(&y.nominal[0]).unwrap();
    close(json["taxYield"].as_f64().unwrap(), 13_000.0, "field taxYield");
    // without a yield the field is missing
    let r0 = run(&base());
    let y0 = engine::digest::yearly(&engine::digest::StoredRun {
        summary: r0.summary.clone(),
        records: r0.records.clone(),
    });
    assert!(serde_json::to_value(&y0.nominal[0]).unwrap().get("taxYield").is_none());
}

/// #149/AK3: the market return and the balances do not change: the same plan with 0 % and with 2 %/1 % has the same
/// balances until the first tax payment (May of the following year); then the liquid wealth differs exactly by the
/// higher tax. The yield is no inflow.
#[test]
fn issue_149_ak3_return_stays_the_same() {
    let mut v = base();
    v["returns"]["fixed"] = json!({ "stocks": 0.06, "bonds": 0.02, "inflation": 0, "saron": 0 });
    let without = run(&v);
    let with = run(&with_yield(v));
    for (a, b) in without.records.iter().zip(&with.records) {
        if b.month >= YearMonth::new(2031, 5) {
            break;
        }
        assert_eq!(a.stocks, b.stocks, "{}", a.month.iso());
        assert_eq!(a.bonds, b.bonds, "{}", a.month.iso());
        assert_eq!(a.cash, b.cash, "{}", a.month.iso());
        assert_eq!(a.stock_index, b.stock_index, "{}", a.month.iso());
    }
    let (a, b) = (rec(&without, 2031, 5), rec(&with, 2031, 5));
    assert!(b.taxes > 0.0);
    close(a.liquid - b.liquid, b.taxes - a.taxes, "only the tax differs");
    close(b.taxes, rec(&with, 2030, 12).assessed_income, "tax of 2030 paid in May");
}

/// #149/AK4: capital gains are not taxable: without a yield rate a return of 10 % gives no income tax; with the rates
/// the yield is the rate on the balances before the month's return, not on the gains.
#[test]
fn issue_149_ak4_capital_gains_are_not_taxable() {
    let mut v = base();
    v["returns"]["fixed"] = json!({ "stocks": 0.1, "bonds": 0.05, "inflation": 0, "saron": 0 });
    let r = run(&v);
    assert_eq!(rec(&r, 2030, 12).assessed_income, 0.0);
    let r = run(&with_yield(v));
    // balances before the return of month k: those at the end of month k − 1 (no flows, no rebalancing differences)
    let mut expected = (500_000.0 * 0.02 + 300_000.0 * 0.01) / 12.0;
    for m in 1..12 {
        let x = rec(&r, 2030, m);
        expected += (x.stocks * 0.02 + x.bonds * 0.01) / 12.0;
    }
    close(rec(&r, 2030, 12).assessed_yield, expected, "yield on the balances");
    assert!(expected < 13_000.0 * 1.1, "no gains in the yield");
}

/// #149/AK8: rates below 0 % or above 100 % are errors in the section taxes with the field path.
#[test]
fn issue_149_ak8_invalid_rates_are_errors() {
    let ok = validate(&plan(&with_yield(base())));
    assert!(!ok.iter().any(|i| i.severity == Severity::Error), "{ok:?}");
    for (stocks, bonds, field) in [
        (-0.01, 0.01, "taxes.investmentYield.stocks"),
        (1.01, 0.01, "taxes.investmentYield.stocks"),
        (0.02, -0.5, "taxes.investmentYield.bonds"),
        (0.02, 1.5, "taxes.investmentYield.bonds"),
    ] {
        let mut v = base();
        v["taxes"]["investmentYield"] = json!({ "stocks": stocks, "bonds": bonds });
        let issues = validate(&plan(&v));
        let errors: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Error).collect();
        assert_eq!(errors.len(), 1, "{stocks}/{bonds}: {issues:?}");
        assert_eq!(errors[0].section, "taxes");
        assert_eq!(errors[0].field.as_deref(), Some(field));
    }
    // limits 0 % and 100 % are valid
    let mut v = base();
    v["taxes"]["investmentYield"] = json!({ "stocks": 0.0, "bonds": 1.0 });
    assert!(!validate(&plan(&v)).iter().any(|i| i.severity == Severity::Error));
}

/// #149, #271: the yield belongs to the net income on which the self-share of 5 % of the care home costs is computed:
/// income 60 000 + yield 13 000 = 73 000, deduction 24 000 − 3 650 = 20 350, taxable 52 650, tax 5 265.
#[test]
fn issue_149_care_home_self_share_includes_the_yield() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("1945-01");
    v["household"]["persons"][0]["careHome"] = json!({ "age": 85, "monthly": 2000, "sell": false });
    v["flows"] = json!({ "income": 5000, "spending": 3000, "spendingIncludesHousing": true });
    let r = run(&with_yield(v.clone()));
    let dec = rec(&r, 2030, 12);
    close(dec.assessed_yield, 13_000.0, "yield");
    close(dec.assessed_income, 5_265.0, "tax with the yield in the net income");
    let r0 = run(&v);
    close(rec(&r0, 2030, 12).assessed_income, 3_900.0, "tax without the yield");
}

/// #149: in the mode «Wohnort» the yield counts to the income before the deductions of canton and federal tax; the
/// choice of the deductions (working or retired) stays with the earned income.
#[test]
fn issue_149_location_mode_counts_the_yield_before_the_deductions() {
    let mut v = base();
    v["taxes"]["mode"] = json!("location");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    v["pension"]["ahv"] = json!({ "monthly": 3000, "at": "start" });
    v["flows"] = json!({ "spending": 3000, "spendingIncludesHousing": true });
    let without = run(&v);
    let with = run(&with_yield(v));
    let (a, b) = (rec(&without, 2030, 12), rec(&with, 2030, 12));
    let (ba, bb) = (a.tax_basis.as_ref().unwrap(), b.tax_basis.as_ref().unwrap());
    close(b.assessed_yield, 13_000.0, "yield");
    close(bb.income - ba.income, 13_000.0, "income before the deductions");
    assert_eq!(
        ba.parts_canton.iter().map(|p| &p.0).collect::<Vec<_>>(),
        bb.parts_canton.iter().map(|p| &p.0).collect::<Vec<_>>()
    );
    assert!(bb.taxable_canton > ba.taxable_canton && bb.taxable_federal > ba.taxable_federal);
    assert!(b.assessed_income > a.assessed_income);
}

/// #149/AK7: the net need of the 3-bucket strategy contains the income tax on the yield (current balances × rates),
/// but the yield is no receipt: the need rises exactly by the tax on it.
#[test]
fn issue_149_ak7_net_need_of_the_buckets() {
    let mut v = base();
    v["milestones"] = json!([
        { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
        { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
        { "id": "end", "name": "Ende", "kind": "date", "date": "2032-12", "fixed": true }
    ]);
    v["pension"]["ahv"] = json!({ "monthly": 2500, "at": "start" });
    v["flows"] = json!({ "spending": 6000, "spendingIncludesHousing": true });
    v["withdrawal"] =
        json!({ "strategy": "three_buckets", "buckets": { "cashYears": 1, "bondYears": 1, "refillThreshold": -1 } });
    let without = run(&v);
    let with = run(&with_yield(v));
    let jan = |r: &SimResult| {
        r.bucket_refills
            .iter()
            .find(|x| x.month == YearMonth::new(2030, 1))
            .expect("refill at the start")
            .clone()
    };
    let (a, b) = (jan(&without), jan(&with));
    assert_eq!(a.basis.investment_yield, 0.0);
    let y = b.basis.investment_yield;
    close(y, b.before[2] * 0.02 + b.before[1] * 0.01, "current balances × rates");
    assert!(y > 0.0);
    close(
        b.basis.income_tax,
        0.1 * (b.basis.receipts() + y),
        "tax on income and yield",
    );
    close(b.basis.receipts(), a.basis.receipts(), "the yield is no receipt");
    close(b.need - a.need, 0.1 * y, "need rises by the tax on the yield");
}
