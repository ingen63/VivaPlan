//! #131: Deductions from income: pillar 3a from the plan, proposal of the canton and own value, time-dependent.
//!
//! Made-up plan that can be recomputed by hand: start 2030-01, end 2031-12, no returns, no inflation, earned income
//! 100'000 per year, own tables with a flat income tax of 10 % (tax = 10 % of the taxable income), no wealth tax.

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator, TaxBasis};
use engine::taxdata;
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Abzüge" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2031-12", "fixed": true }
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

/// [`base`] in the mode «Wohnort» (Zurich, no church), with income `income` per year.
fn location(canton: &str, bfs: u32, income: f64) -> Value {
    let mut v = base();
    v["flows"]["income"] = json!(income / 12.0);
    v["taxes"]["mode"] = json!("location");
    v["taxes"]["location"] = json!({ "canton": canton, "municipality": bfs, "church": "none" });
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

/// Assessed income tax of year `y`.
fn tax(r: &SimResult, y: i32) -> f64 {
    december(r, y).assessed_income
}

fn basis(r: &SimResult, y: i32) -> &TaxBasis {
    december(r, y).tax_basis.as_ref().expect("basis of the income tax")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

fn own(amount: f64, rate: f64) -> Value {
    json!([{ "from": "start", "kind": "own", "amount": amount, "rate": rate }])
}

fn errors(v: &Value) -> Vec<(String, String)> {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    validate(&plan)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| (i.section.to_string(), i.field.clone().unwrap_or_default()))
        .collect()
}

/// #131/AK1, AK16: Income 100'000, other deductions 0, contribution 7'000 → tax as on 93'000 (10 % = 9'300).
#[test]
fn issue_131_ak1_pillar3a_contribution_is_deducted() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 7000 }]);
    v["taxes"]["deductPillar3a"] = json!(true);
    let r = run(&v);
    close(tax(&r, 2030), 9300.0, "tax on 93'000");
    close(basis(&r, 2030).pillar3a, 7000.0, "deduction pillar 3a");
    close(basis(&r, 2030).taxable_canton, 93_000.0, "taxable");
}

/// #131/AK1: Two persons: each person's contribution counts (7'000 + 5'000); in the mode «Wohnort» for canton
/// and federal tax.
#[test]
fn issue_131_ak1_pillar3a_per_person_and_for_canton_and_federal() {
    let mut v = location("ZH", 261, 100_000.0);
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1980-01" }, { "id": "B", "name": "B", "birth": "1980-01" } ]);
    v["household"]["married"] = json!(true);
    v["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                           "ahv": { "monthly": 0, "at": "ahv" } });
    v["pillar3a"] = json!([
        { "person": "A", "balance": 0, "rate": 0, "contribution": 7000 },
        { "person": "B", "balance": 0, "rate": 0, "contribution": 5000 } ]);
    let without = run(&v);
    v["taxes"]["deductPillar3a"] = json!(true);
    let with = run(&v);
    let (a, b) = (basis(&without, 2030), basis(&with, 2030));
    close(b.pillar3a, 12_000.0, "7'000 + 5'000");
    close(a.taxable_canton - b.taxable_canton, 12_000.0, "canton");
    close(a.taxable_federal - b.taxable_federal, 12_000.0, "federal");
    assert!(tax(&with, 2030) < tax(&without, 2030));
}

/// #131/AK1: The indirect amortization is a pillar 3a contribution too (4 × 750 + 3'000 = 6'000).
#[test]
fn issue_131_ak1_indirect_amortization_counts_as_pillar3a() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 3000 }]);
    v["taxes"]["deductPillar3a"] = json!(true);
    v["realestate"] = json!({ "properties": [ { "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000,
        "sell": "never", "fixCosts": 0, "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": "2030-01",
        "periods": [ { "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" } ] } ] } } ] });
    let r = run(&v);
    close(
        basis(&r, 2030).pillar3a,
        6000.0,
        "contribution and indirect amortization",
    );
    close(tax(&r, 2030), 9400.0, "tax on 94'000");
}

/// #131/AK2: Without `deductPillar3a` the contribution is not deducted (tax on 100'000).
#[test]
fn issue_131_ak2_pillar3a_deduction_can_be_switched_off() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 7000 }]);
    close(tax(&run(&v), 2030), 10_000.0, "missing: no deduction");
    v["taxes"]["deductPillar3a"] = json!(false);
    close(tax(&run(&v), 2030), 10_000.0, "switched off");
}

/// #131/AK3, AK16: Time-dependent rows: own value 10'000 from the start, 20'000 from 2031 → taxable 90'000 and
/// 80'000; a row from July counts for half of the year (6 of 12 months of 12'000 = 6'000).
#[test]
fn issue_131_ak3_rows_apply_from_their_point_in_time() {
    let mut v = base();
    v["taxes"]["deductions"] = json!([
        { "from": "start", "kind": "own", "amount": 10_000 },
        { "from": "2031-01", "kind": "own", "amount": 20_000 } ]);
    let r = run(&v);
    close(tax(&r, 2030), 9000.0, "2030 on 90'000");
    close(tax(&r, 2031), 8000.0, "2031 on 80'000");
    v["taxes"]["deductions"] = json!([
        { "from": "start", "kind": "own", "amount": 0 },
        { "from": "2030-07", "kind": "own", "amount": 12_000 } ]);
    let r = run(&v);
    close(basis(&r, 2030).own, 6000.0, "half a year");
    close(basis(&r, 2031).own, 12_000.0, "whole year");
}

/// #131/AK4, AK5: A row «Vorschlag des Kantons» computes with the rules of the tax data file: the deduction for
/// canton and federal tax and its parts (insurance premiums, professional expenses) as in the file.
#[test]
fn issue_131_ak5_proposal_of_the_canton_with_parts() {
    let mut v = location("ZH", 261, 100_000.0);
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "canton" }]);
    let r = run(&v);
    let b = basis(&r, 2030);
    assert_eq!(b.proposal_share, 1.0, "whole year with the proposal");
    let resolved = taxdata::builtin()
        .resolve(&serde_json::from_value(v["taxes"]["location"].clone()).unwrap(), false)
        .unwrap();
    let income = b.income;
    let (pc, pf) = resolved.proposal(income, &[income], 1.0);
    close(b.proposal_canton, pc, "canton");
    close(b.proposal_federal, pf, "federal");
    // ZH, single, working: insurance 2'900 + professional expenses 3 % of 100'000 = 3'000 (2'000 to 4'000)
    close(pc, 2900.0 + 3000.0, "ZH by hand");
    let part = |parts: &[(String, f64)], k: &str| parts.iter().find(|p| p.0 == k).map_or(0.0, |p| p.1);
    close(part(&b.parts_canton, "insurance"), 2900.0, "insurance canton");
    close(part(&b.parts_canton, "professional"), 3000.0, "professional canton");
    // federal: insurance 1'800 + 3'000
    close(part(&b.parts_federal, "insurance"), 1800.0, "insurance federal");
    close(b.parts_federal.iter().map(|p| p.1).sum::<f64>(), pf, "parts federal");
    close(
        tax(&r, 2030),
        resolved.income_tax_on(income - pc, income - pf, 1.0),
        "tax",
    );
}

/// #131/AK6, AK16: Own value 40'000 with income 120'000 → tax as on 80'000, for canton and federal tax; with a share of
/// the income (25 % of 120'000 = 30'000).
#[test]
fn issue_131_ak6_own_value_for_canton_and_federal() {
    let mut v = base();
    v["flows"]["income"] = json!(10_000);
    v["taxes"]["deductions"] = own(40_000.0, 0.0);
    close(tax(&run(&v), 2030), 8000.0, "own tables: tax on 80'000");
    v["taxes"]["deductions"] = own(0.0, 0.25);
    close(tax(&run(&v), 2030), 9000.0, "25 %: tax on 90'000");

    let mut v = location("ZH", 261, 120_000.0);
    v["taxes"]["deductions"] = own(40_000.0, 0.0);
    let r = run(&v);
    let b = basis(&r, 2030);
    close(b.taxable_canton, 80_000.0, "canton");
    close(b.taxable_federal, 80_000.0, "federal");
    assert_eq!(b.proposal_canton, 0.0);
    assert!(b.parts_canton.is_empty() && b.parts_federal.is_empty() && b.proposal_share == 0.0);
    let resolved = taxdata::builtin()
        .resolve(&serde_json::from_value(v["taxes"]["location"].clone()).unwrap(), false)
        .unwrap();
    close(tax(&r, 2030), resolved.income_tax_on(80_000.0, 80_000.0, 1.0), "tax");
}

/// #131/AK6: With «Tarife mit der Teuerung» the own value grows with the price index (prices of the start).
#[test]
fn issue_131_ak6_own_value_grows_with_indexed_tariffs() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["taxes"]["deductions"] = own(10_000.0, 0.0);
    close(basis(&run(&v), 2031).own, 10_000.0, "nominal without the check box");
    v["taxes"]["indexed"] = json!(true);
    let r = run(&v);
    let idx = december(&r, 2031).price_index;
    assert!(idx > 1.03);
    close(basis(&r, 2031).own, 10_000.0 * idx, "indexed");
}

/// #131/AK8: An own value stays when canton or marital status change; a proposal is computed anew.
#[test]
fn issue_131_ak8_own_value_stays_proposal_changes() {
    let taxable = |canton: &str, bfs: u32, kind: &str| {
        let mut v = location(canton, bfs, 100_000.0);
        v["taxes"]["deductions"] = json!([{ "from": "start", "kind": kind, "amount": 15_000 }]);
        basis(&run(&v), 2030).taxable_canton
    };
    close(taxable("ZH", 261, "own"), 85_000.0, "ZH own");
    close(taxable("TI", 5192, "own"), 85_000.0, "TI own");
    assert_ne!(taxable("ZH", 261, "canton"), taxable("TI", 5192, "canton"));
}

/// #131/AK10: Mortgage interest is not deducted automatically (rate 2 % on 400'000, taxable stays the income).
#[test]
fn issue_131_ak10_no_automatic_mortgage_interest() {
    let mut v = base();
    v["taxes"]["deductPillar3a"] = json!(true);
    v["realestate"] = json!({ "properties": [ { "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000,
        "sell": "never", "fixCosts": 0, "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": "2030-01",
        "periods": [ { "type": "fixed", "term": 20, "rate": 0.02 } ] } ] } } ] });
    let r = run(&v);
    close(basis(&r, 2030).taxable_canton, 100_000.0, "without interest");
    close(tax(&r, 2030), 10_000.0, "tax");
}

/// #131/AK11: With own tables there is no proposal: a row «canton» deducts nothing.
#[test]
fn issue_131_ak11_no_proposal_with_own_tables() {
    let mut v = base();
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "canton" }]);
    let r = run(&v);
    assert_eq!(basis(&r, 2030).proposal_canton, 0.0);
    close(tax(&r, 2030), 10_000.0, "no deduction");
    assert!(errors(&v).is_empty());
}

/// #131/AK12: The taxable income never becomes negative (own value 200'000 on 100'000).
#[test]
fn issue_131_ak12_taxable_income_not_negative() {
    let mut v = base();
    v["taxes"]["deductions"] = own(200_000.0, 0.0);
    let r = run(&v);
    assert_eq!(basis(&r, 2030).taxable_canton, 0.0);
    assert_eq!(tax(&r, 2030), 0.0);
    let mut v = location("ZH", 261, 100_000.0);
    v["taxes"]["deductions"] = own(200_000.0, 0.0);
    let b = basis(&run(&v), 2030).clone();
    assert_eq!((b.taxable_canton, b.taxable_federal), (0.0, 0.0));
}

/// #131/AK13: Existing plans compute as before: mode «Wohnort» without rows = proposal of the canton throughout (bit
/// for bit as #63), own tables without rows = no deductions and no basis in the result; the file stays without the new
/// fields.
#[test]
fn issue_131_ak13_existing_plans_unchanged() {
    let v = location("BE", 351, 90_000.0);
    let before = run(&v);
    let resolved = taxdata::builtin()
        .resolve(&serde_json::from_value(v["taxes"]["location"].clone()).unwrap(), false)
        .unwrap();
    let income = basis(&before, 2030).income;
    assert_eq!(tax(&before, 2030), resolved.income_tax(income, &[income], 1.0));
    let mut explicit = v.clone();
    explicit["taxes"]["deductions"] = json!([{ "from": "start", "kind": "canton" }]);
    let after = run(&explicit);
    for (a, b) in before.records.iter().zip(&after.records) {
        assert_eq!(a.assessed_income, b.assessed_income, "{}", a.month);
    }

    let r = run(&base());
    assert!(r.records.iter().all(|x| x.tax_basis.is_none()));
    let plan: Plan = serde_json::from_value(base()).unwrap();
    let out = serde_json::to_value(&plan).unwrap();
    assert!(out["taxes"].get("deductions").is_none() && out["taxes"].get("deductPillar3a").is_none());
}

/// #131/AK14: Negative amount, negative share or share above 100 % are errors in the section «Steuern».
#[test]
fn issue_131_ak14_invalid_values_are_errors() {
    for (row, field) in [
        (json!({ "from": "start", "kind": "own", "amount": -1 }), "amount"),
        (json!({ "from": "start", "kind": "own", "rate": -0.1 }), "rate"),
        (json!({ "from": "start", "kind": "own", "rate": 1.5 }), "rate"),
    ] {
        let mut v = base();
        v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "own", "amount": 0 }, row]);
        assert_eq!(
            errors(&v),
            vec![("taxes".to_string(), format!("taxes.deductions.1.{field}"))]
        );
    }
    let mut v = base();
    v["taxes"]["deductions"] = json!([{ "from": "nirgends", "kind": "own" }]);
    assert!(errors(&v).iter().any(|e| e.0 == "taxes"));
}

/// #131/AK15: The basis shows income, deductions by origin and taxable income (own tables: same for both).
#[test]
fn issue_131_ak15_basis_by_origin() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 7000 }]);
    v["taxes"]["deductPillar3a"] = json!(true);
    v["taxes"]["deductions"] = own(10_000.0, 0.0);
    let r = run(&v);
    let b = basis(&r, 2030);
    close(b.income, 100_000.0, "income");
    close(b.own, 10_000.0, "own value");
    close(b.pillar3a, 7000.0, "pillar 3a");
    close(b.taxable_canton, 83_000.0, "taxable");
    close(b.taxable_federal, 83_000.0, "taxable federal");
    // only in December
    assert!(r
        .records
        .iter()
        .filter(|x| x.month.month() != 12)
        .all(|x| x.tax_basis.is_none()));
    let json = serde_json::to_value(b).unwrap();
    for k in [
        "income",
        "proposalCanton",
        "own",
        "pillar3a",
        "taxableCanton",
        "taxableFederal",
    ] {
        assert!(json.get(k).is_some(), "{k}");
    }
}

/// #131: The net need of the 3-bucket strategy uses the same deductions (own value 10'000: income tax 4000/84000 on
/// 86'500 − 10'000).
#[test]
fn issue_131_net_need_uses_the_same_deductions() {
    let mut v = json!({
        "project": { "name": "3-Topf" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2033-12" }
        ],
        "pension": { "pk": { "capital": 1_000_000, "at": "retire", "lumpsum": 0, "conversion": 0.054 }, "ahv": { "monthly": 2_500, "at": "start" } },
        "flows": { "income": 0, "spending": 100_000.0 / 12.0 },
        "wealth": { "stocks": 600_000, "bonds": 200_000, "cash": 700_000, "stockShare": 0.6, "reserveMonths": 6 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.06, "bonds": 0.01, "inflation": 0 } },
        "taxes": { "income": [[0, 4000.0 / 84000.0]], "wealth": [], "capital": [], "propertyGain": { "brackets": [], "reduction": 0.0 },
                   "deductions": [ { "from": "start", "kind": "own", "amount": 10_000 } ] },
        "withdrawal": { "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 1, "refillThreshold": -1 } }
    });
    let need = |v: &Value, y: i32, m: u32| {
        let r = run(&crate::any_age(v.clone()));
        r.bucket_refills
            .iter()
            .find(|x| x.month == YearMonth::new(y, m))
            .unwrap()
            .basis
            .clone()
    };
    let rate = 4000.0 / 84000.0;
    close(need(&v, 2031, 1).income_tax, rate * 76_500.0, "income tax on 76'500");
    // the income tax of the need is rounded to centimes
    let less = |a: f64, b: f64, by: f64, what: &str| {
        assert!(
            (a - b - rate * by).abs() < 0.01,
            "{what}: {} instead of {}",
            a - b,
            rate * by
        );
    };
    // #131/AK1: planned pillar 3a contributions until before the person's milestone «AHV-Bezug» (65 in July 2031):
    // January to June = 6 × 500
    v["household"]["persons"][0]["birth"] = json!("1966-07");
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 6000 }]);
    let without = need(&v, 2031, 1).income_tax;
    v["taxes"]["deductPillar3a"] = json!(true);
    less(
        without,
        need(&v, 2031, 1).income_tax,
        3000.0,
        "pillar 3a until the AHV payout",
    );
    // share of the income: 10 % of AHV (with the 13th pension), PK pension and earned income (12 × 2'000)
    v["flows"]["income"] = json!(2000);
    let b = need(&v, 2031, 1);
    close(b.earned, 24_000.0, "earned income");
    v["taxes"]["deductions"][0]["rate"] = json!(0.1);
    less(
        b.income_tax,
        need(&v, 2031, 1).income_tax,
        0.1 * (b.ahv + b.pk + b.earned),
        "10 % of the income",
    );
    // part of a year from the start in July: extrapolated to 12 months (6 × 500 × 2 = 6'000)
    v["milestones"][0]["date"] = json!("2030-07");
    v["milestones"][1]["date"] = json!("2030-07");
    v["household"]["persons"][0]["birth"] = json!("1970-01");
    v["taxes"]["deductions"][0]["rate"] = json!(0);
    let b = need(&v, 2030, 7);
    assert_eq!(b.months, 6);
    v["taxes"]["deductPillar3a"] = json!(false);
    less(
        need(&v, 2030, 7).income_tax,
        b.income_tax,
        6000.0,
        "pillar 3a extrapolated",
    );
    v["taxes"]["deductions"][0]["rate"] = json!(0.1);
    let c = need(&v, 2030, 7);
    v["taxes"]["deductions"][0]["rate"] = json!(0);
    let d = need(&v, 2030, 7);
    less(
        d.income_tax,
        c.income_tax,
        0.1 * (c.ahv + c.pk + c.earned),
        "share extrapolated",
    );
}
