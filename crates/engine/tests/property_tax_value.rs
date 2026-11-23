//! #229: Tax value per property for the wealth tax.
//!
//! Made-up plan (`PlanBuilder::new`): start 2030-01, end 2039-12, no income, no spending, cash 100'000 (so no
//! emergency sale when the tax is paid), no returns; wealth tax flat 1 % (multiplier 1). The cash stays 100'000
//! until the first payment of the tax in May 2031, so the wealth tax assessed in December 2030 is 1 % of
//! 100'000 plus the property wealth.

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

use crate::plan_builder::PlanBuilder;

/// Owned house: market value 1'000'000, price paid 800'000, mortgage 400'000 at 0 % without amortization, no fix
/// costs, never sold; `extra` sets further fields (for example `taxValue`).
fn house(extra: Value) -> Value {
    let mut h = json!({ "id": "h", "name": "Haus", "status": "owned", "price": 800_000, "worth": 1_000_000,
        "fixCosts": 0, "sell": "never",
        "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": "2029-01", "periods": [
            { "type": "fixed", "term": 20, "rate": 0.0, "amort": 0 } ] } ] } });
    for (k, v) in extra.as_object().expect("object") {
        h[k] = v.clone();
    }
    h
}

fn plan(property: Value) -> Value {
    PlanBuilder::new("Steuerwert")
        .property(property)
        .set(
            "taxes",
            json!({ "multiplier": 1.0, "income": [], "wealth": [[0, 0.01], [10_000_000, 0.01]], "capital": [],
                "propertyGain": { "brackets": [[0, 0.2]], "reduction": 0.0 } }),
        )
        .set(
            "wealth",
            json!({ "stocks": 0, "bonds": 0, "cash": 100_000, "stockShare": 1, "reserveMonths": 0 }),
        )
        .build()
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let s = Simulator::new(&plan).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records
        .iter()
        .find(|x| x.month == YearMonth::new(y, m))
        .expect("month in range")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} instead of {b}");
}

fn errors(v: &Value) -> Vec<(String, Option<String>)> {
    engine::api::validate(v)
        .into_iter()
        .filter(|i| i.severity == engine::validate::Severity::Error)
        .map(|i| (i.message.to_string(), i.field))
        .collect()
}

/// #229/AK2, AK8: Market value 1'000'000, tax value 650'000, mortgage 400'000: taxable wealth 250'000 instead of
/// 600'000; with the cash of 100'000 at 1 % a wealth tax of 3'500 instead of 7'000 (hand value).
#[test]
fn issue_229_ak2_tax_value_minus_mortgage() {
    let r = run(&plan(house(
        json!({ "taxValue": { "kind": "amount", "value": 650_000 } }),
    )));
    let dec = rec(&r, 2030, 12);
    close(dec.liquid, 100_000.0, "cash");
    close(dec.assessed_wealth, 3_500.0, "1 % of 100'000 + 250'000");
    let basis = dec.tax_basis.as_ref().expect("tax basis with a tax value");
    close(
        basis.property_tax_value.expect("tax value of the properties"),
        650_000.0,
        "tax value",
    );
    // The same as a share of the market value
    let r = run(&plan(house(json!({ "taxValue": { "kind": "rate", "value": 0.65 } }))));
    close(rec(&r, 2030, 12).assessed_wealth, 3_500.0, "65 % of the market value");
}

/// #229/AK4, AK8: Without a tax value the market value applies as before: 1 % of 700'000, and no tax basis (the
/// plan has no deductions); a rental ignores a tax value.
#[test]
fn issue_229_ak4_without_tax_value_unchanged() {
    let r = run(&plan(house(json!({}))));
    let dec = rec(&r, 2030, 12);
    close(dec.assessed_wealth, 7_000.0, "1 % of 100'000 + 600'000");
    assert!(dec.tax_basis.is_none(), "no tax basis without a tax value");
    let rent = json!({ "name": "Miete", "status": "rent", "from": "start", "rent": 0,
        "taxValue": { "kind": "amount", "value": -1 } });
    let v = PlanBuilder::new("Miete").property(rent).build();
    assert!(errors(&v).is_empty(), "a rental has no tax value: {:?}", errors(&v));
    assert!(
        run(&v).records.iter().all(|x| x.tax_basis.is_none()),
        "no tax basis for a rental"
    );
}

/// #229/AK3: An amount keeps its ratio to the market value: with 2 % inflation the tax value grows like the market
/// value; for a planned purchase the share relates to the purchase price.
#[test]
fn issue_229_ak3_ratio_follows_market_value() {
    let mut v = plan(house(
        json!({ "inflationValue": true, "taxValue": { "kind": "amount", "value": 650_000 } }),
    ));
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    let r = run(&v);
    for y in [2031, 2035] {
        let dec = rec(&r, y, 12);
        assert!(dec.property_value > 1_000_000.0, "market value grows");
        let tv = dec
            .tax_basis
            .as_ref()
            .and_then(|b| b.property_tax_value)
            .expect("tax value");
        close(tv / dec.property_value, 0.65, "ratio stays 65 %");
    }
    // Planned purchase in 2031-01 at 1'000'000 without a mortgage: an amount of 500'000 is 50 % of the price.
    let planned = json!({ "id": "p", "name": "Kauf", "status": "planned", "price": 1_000_000, "buy": "2031-01",
        "sell": "never", "fixCosts": 0, "inflationValue": true, "taxValue": { "kind": "amount", "value": 500_000 } });
    let mut v = PlanBuilder::new("Kauf").property(planned).build();
    v["wealth"]["cash"] = json!(2_000_000);
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    let r = run(&v);
    let dec = rec(&r, 2030, 12);
    close(
        dec.tax_basis
            .as_ref()
            .and_then(|b| b.property_tax_value)
            .expect("basis"),
        0.0,
        "before the purchase",
    );
    let dec = rec(&r, 2032, 12);
    let tv = dec
        .tax_basis
        .as_ref()
        .and_then(|b| b.property_tax_value)
        .expect("tax value");
    close(tv / dec.property_value, 0.5, "50 % of the purchase price");
}

/// #229/AK5: Wealth, results and the property gain tax keep the market value; only the wealth tax changes.
#[test]
fn issue_229_ak5_only_wealth_tax_uses_tax_value() {
    let sold = |extra: Value| {
        let mut h = house(extra);
        h["sell"] = json!("2033-01");
        run(&plan(h))
    };
    let (with, without) = (
        sold(json!({ "taxValue": { "kind": "rate", "value": 0.5 } })),
        sold(json!({})),
    );
    let (a, b) = (rec(&with, 2030, 12), rec(&without, 2030, 12));
    close(a.property_value, 1_000_000.0, "market value");
    close(a.property_value, b.property_value, "property value");
    close(a.property_equity, b.property_equity, "property equity");
    close(a.property_equity, 600_000.0, "equity with the market value");
    close(a.assessed_wealth, 2_000.0, "1 % of 100'000 + 500'000 - 400'000");
    close(b.assessed_wealth, 7_000.0, "1 % of 100'000 + 600'000");
    // Property gain tax at the sale: 20 % of the gain on the market value (1'000'000 - 800'000)
    let (a, b) = (rec(&with, 2033, 1), rec(&without, 2033, 1));
    close(a.tax_gain, 40_000.0, "gain tax with the market value");
    close(a.tax_gain, b.tax_gain, "gain tax unchanged");
}

/// #229/AK6: A negative amount and a share outside 0–100 % are errors at the field of the property; 0 and 100 %
/// are allowed.
#[test]
fn issue_229_ak6_invalid_tax_value_is_error() {
    let field = Some("realestate.properties.0.taxValue.value".to_string());
    let errs = errors(&plan(house(json!({ "taxValue": { "kind": "amount", "value": -1 } }))));
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(errs[0].1, field);
    assert!(errs[0].0.contains("negative"), "{errs:?}");
    for bad in [-0.01, 1.01] {
        let errs = errors(&plan(house(json!({ "taxValue": { "kind": "rate", "value": bad } }))));
        assert_eq!(errs.len(), 1, "{bad}: {errs:?}");
        assert_eq!(errs[0].1, field);
        assert!(errs[0].0.contains("100 %"), "{errs:?}");
    }
    for (kind, ok) in [("rate", 0.0), ("rate", 1.0), ("amount", 0.0), ("amount", 2_000_000.0)] {
        let errs = errors(&plan(house(json!({ "taxValue": { "kind": kind, "value": ok } }))));
        assert!(errs.is_empty(), "{kind} {ok}: {errs:?}");
    }
}

/// #229/AK3: The ratio of an amount to the market value; on a base of 0 there is no ratio and the market value applies.
#[test]
fn issue_229_ak3_ratio_of_amount_and_share() {
    use engine::model::{TaxValue, TaxValueKind};
    let amount = TaxValue {
        kind: TaxValueKind::Amount,
        value: 650_000.0,
    };
    close(amount.ratio(1_000_000.0), 0.65, "amount / market value");
    close(amount.ratio(0.0), 1.0, "base 0: market value");
    close(amount.ratio(-1.0), 1.0, "negative base: market value");
    let rate = TaxValue {
        kind: TaxValueKind::Rate,
        value: 0.7,
    };
    close(rate.ratio(0.0), 0.7, "share independent of the base");
}
