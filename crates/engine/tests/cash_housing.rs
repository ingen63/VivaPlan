//! #53 (living costs including housing costs) and #54 (cash separate from the portfolio, strategies from retirement).
//!
//! Made-up plan: birth 1980-01, start 2040-01, pension fund payout and AHV payout at 65 (2045-01), no
//! returns, no inflation (except where noted), no taxes.

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Cash" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 8000, "spending": 8000, "spendingIncludesHousing": true },
        "wealth": { "stocks": 80_000, "bonds": 20_000, "cash": 50_000, "stockShare": 0.8, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("Planung lesbar");
    let sim = Simulator::new(&plan).expect("Planung gültig");
    let path = engine::market::fixed_path(&sim.fixed, sim.start, sim.months());
    sim.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records.iter().find(|x| x.month == YearMonth::new(y, m)).unwrap()
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} statt {b}");
}

/// «Ausgaben total» of a year as in the annual values: planned living costs + housing costs + 3a, monthly average.
fn spending_total(r: &SimResult, y: i32) -> f64 {
    let rs: Vec<_> = r.records.iter().filter(|x| x.month.year() == y).collect();
    rs.iter()
        .map(|x| x.planned_spending + x.housing + x.pillar3a_in)
        .sum::<f64>()
        / rs.len() as f64
}

fn errors(v: &Value) -> Vec<String> {
    engine::api::validate(v)
        .into_iter()
        .filter(|i| i.severity == engine::validate::Severity::Error)
        .map(|i| i.message.to_string())
        .collect()
}

/// House: fix costs 500, mortgage 600'000 at 2 % (interest 1'000), amortization 1'500 per quarter (500 per month),
/// indirect, so the debt and thus the interest stay the same. Renewal 2042-01 at 2.6 % (interest 1'300).
fn house(sell: &str) -> Value {
    json!({ "id": "h", "name": "Haus", "status": "owned", "price": 1_000_000, "worth": 1_000_000, "fixCosts": 500, "sell": sell,
        "mortgage": { "tranches": [ { "name": "T", "amount": 600_000, "start": "2037-01", "periods": [
            { "type": "fixed", "term": 5, "rate": 0.02, "amort": 1500, "amortMode": "indirect" },
            { "type": "fixed", "term": 10, "rate": 0.026, "amort": 1500, "amortMode": "indirect" } ] } ] } })
}

/// #53/AK3: Input 8'000 including housing costs 2'000: total spending 8'000; interest +300 gives 8'300; rent 2'500
/// after the sale gives 8'500.
#[test]
fn issue_53_ak3_real_housing_costs_replace_the_share() {
    let mut v = base();
    v["flows"]["income"] = json!(20_000); // affordable at renewal
    v["realestate"] = json!({ "properties": [ house("2043-01"),
        { "name": "Miete", "status": "rent", "from": "afterSale", "rent": 2500 } ] });
    let r = run(&v);
    close(rec(&r, 2040, 1).planned_spending, 6000.0, "Lebenshaltung ohne Wohnen");
    close(spending_total(&r, 2041), 8000.0, "2041");
    close(spending_total(&r, 2042), 8300.0, "nach der Erneuerung");
    close(spending_total(&r, 2044), 8500.0, "mit Miete");
}

/// #53/AK4, AK5: A housing share larger than a row is an error; without housing costs nothing changes.
#[test]
fn issue_53_ak4_ak5_check_and_no_housing() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("never") ] });
    v["flows"]["spending"] = json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "2044-01", "value": 1999 } ] });
    let e = errors(&v);
    assert_eq!(e, vec!["Living expenses (1'999) are lower than the housing costs at the simulation start (2'000) that they include.".to_string()]);
    let i = engine::api::validate(&v)
        .into_iter()
        .find(|i| i.message.contains("1'999"))
        .unwrap();
    assert_eq!(i.field.as_deref(), Some("flows.spending.rows.1.value"));
    v["flows"]["spending"]["rows"][1]["value"] = json!(2000);
    assert!(errors(&v).is_empty(), "genau der Wohnanteil ist gültig");
    // AK2: same deduction for every row
    let r = run(&v);
    close(rec(&r, 2044, 1).planned_spending, 0.0, "2'000 − 2'000");
    // AK5: without property and rent
    let r = run(&base());
    close(rec(&r, 2040, 1).planned_spending, 8000.0, "ohne Wohnkosten");
}

/// #53/AK6: The 3a contribution is not contained in the input and is added.
#[test]
fn issue_53_ak6_pillar3a_stays_separate() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("never") ] });
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 1200 }]);
    let r = run(&v);
    close(spending_total(&r, 2041), 8100.0, "8'000 + 100 Säule 3a");
}

/// #53/AK7: Older plans are converted on opening; the result stays the same.
#[test]
fn issue_53_ak7_upgrade_keeps_the_result() {
    let mut old = base();
    old["flows"] = json!({ "income": 8000, "spending": { "mode": "timed", "rows": [ { "from": "start", "value": 6000 }, { "from": "2044-01", "value": 5000 } ] } });
    old["realestate"] = json!({ "properties": [ house("never") ] });
    let out = engine::api::upgrade_plan(&old);
    assert_eq!(out.plan["flows"]["spendingIncludesHousing"], true);
    assert_eq!(out.plan["flows"]["spending"]["rows"][0]["value"], 8000.0);
    assert_eq!(out.plan["flows"]["spending"]["rows"][1]["value"], 7000.0);
    assert!(
        out.notes
            .iter()
            .any(|n| n
                .contains("The housing costs at the simulation start (2000 per month) have been added to each value")),
        "{:?}",
        out.notes
    );
    let (a, b) = (run(&old), run(&out.plan));
    for (x, y) in a.records.iter().zip(&b.records) {
        close(x.total, y.total, &x.month.to_string());
    }
    // Second opening: nothing left to do
    assert!(engine::api::upgrade_plan(&out.plan).notes.is_empty());
}

/// #54/AK8: An older plan gets the note about the cash rule on opening, also without housing costs; once.
#[test]
fn issue_54_ak8_note_for_older_plans() {
    let mut old = base();
    old["flows"] = json!({ "income": 8000, "spending": 6000 });
    let out = engine::api::upgrade_plan(&old);
    assert_eq!(out.notes, vec![engine::upgrade::CASH_NOTE]);
    assert_eq!(out.plan["flows"]["spending"], 6000, "ohne Wohnkosten bleibt der Wert");
    assert!(engine::api::upgrade_plan(&out.plan).notes.is_empty());
    assert!(
        engine::api::upgrade_plan(&base()).notes.is_empty(),
        "neue Planung ohne Hinweis"
    );
}

/// #53/AK8: The spending factor of the variation applies to the living costs without housing.
#[test]
fn issue_53_ak8_spending_factor_on_living_without_housing() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("never") ] });
    v["flows"]["spendingFactor"] = json!(0.5);
    let r = run(&v);
    close(rec(&r, 2040, 1).planned_spending, 3000.0, "0.5 × (8'000 − 2'000)");
}

/// #54/AK2: Income equal to spending: until retirement the holdings stay as entered.
#[test]
fn issue_54_ak2_holdings_unchanged_before_retirement() {
    let r = run(&base());
    for (y, m) in [(2040, 1), (2042, 6), (2044, 12)] {
        let x = rec(&r, y, m);
        close(x.cash, 50_000.0, "Cash");
        close(x.stocks, 80_000.0, "Aktien");
        close(x.bonds, 20_000.0, "Obligationen");
    }
}

/// #54/AK3: A surplus of 2'000 per month goes into the portfolio (1'600 stocks, 400 bonds), cash stays.
#[test]
fn issue_54_ak3_surplus_goes_to_the_portfolio() {
    let mut v = base();
    v["flows"]["income"] = json!(10_000);
    let r = run(&v);
    let x = rec(&r, 2040, 1);
    close(x.cash, 50_000.0, "Cash");
    close(x.stocks, 81_600.0, "Aktien");
    close(x.bonds, 20_400.0, "Obligationen");
}

/// #54: A shortfall is refilled from the portfolio; the cash amount follows inflation.
#[test]
fn issue_54_refill_and_inflation() {
    let mut v = base();
    v["flows"]["income"] = json!(7000);
    v["flows"]["spendingIncludesHousing"] = json!(false);
    v["flows"]["spending"] = json!(8000.0 / 1.0);
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    let r = run(&v);
    let x = rec(&r, 2041, 1);
    close(x.cash, 50_000.0 * x.price_index, "Cash mit Teuerung");
    assert!(
        (x.cash - 51_000.0).abs() < 10.0,
        "rund 51'000 nach einem Jahr: {}",
        x.cash
    );
    assert!(
        x.stocks < 80_000.0 && (x.stocks / (x.stocks + x.bonds) - 0.8).abs() < 1e-9,
        "aus dem Portfolio nach der Aktienquote"
    );
}

/// #54/AK4, AK5: The cash reserve of «Standard» only takes effect from retirement.
#[test]
fn issue_54_ak4_ak5_reserve_only_after_retirement() {
    let mut v = base();
    v["flows"]["income"] =
        json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "retire", "value": 0 } ] });
    v["wealth"]["cash"] = json!(10_000);
    v["wealth"]["stocks"] = json!(1_000_000);
    v["wealth"]["bonds"] = json!(0);
    let with = |months: f64| {
        let mut w = v.clone();
        w["wealth"]["reserveMonths"] = json!(months);
        run(&w)
    };
    let (a, b) = (with(0.0), with(12.0));
    close(
        rec(&a, 2044, 12).cash,
        rec(&b, 2044, 12).cash,
        "vor der Pensionierung gleich",
    );
    close(rec(&b, 2044, 12).cash, 10_000.0, "Cash-Betrag der Planung");
    close(
        rec(&b, 2045, 1).cash,
        12.0 * 8000.0,
        "ab der Pensionierung: 12 Monate ungedeckte Ausgaben",
    );
    close(rec(&a, 2045, 1).cash, 0.0, "Reserve 0");
}
