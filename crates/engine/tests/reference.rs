//! Reference cases: small plans whose result is recomputed by hand (computation in the comment).
//! They check the domain rules independently of the implementation.

use engine::batch;
use engine::model::Plan;
use engine::sim::{EventKind, SimResult, Simulator};
use engine::tax;
use engine::time::YearMonth;
use engine::validate::{self, Severity};
use serde_json::{json, Value};

/// From January 2030 to December 2031, no returns, no inflation, no taxes.
fn base() -> Value {
    json!({
        "project": { "name": "Referenz" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2040-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2031-12" }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "never" }
        },
        "flows": { "income": 0, "spending": 0 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [], "reduction": 0.0 } }
    })
}

fn plan(v: Value) -> Plan {
    serde_json::from_value(crate::any_age(v)).expect("Planung gültig")
}

fn run(v: Value) -> SimResult {
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} statt {b}");
}

fn month(res: &SimResult, y: i32, m: u32) -> &engine::sim::MonthRecord {
    res.records
        .iter()
        .find(|r| r.month == YearMonth::new(y, m))
        .expect("Monat im Zeitraum")
}

// ---------- Inflation ----------

/// Start in March: spending is entered in March prices; inflation acts every month.
/// In January only 10 months have passed, so 1.1^(10/12) applies; from the next January a full year.
#[test]
fn indexation_runs_monthly_from_start() {
    let mut v = base();
    v["milestones"][0]["date"] = json!("2030-03");
    v["milestones"][2]["date"] = json!("2032-01");
    v["wealth"]["cash"] = json!(100_000);
    v["flows"]["spending"] = json!(1_000);
    v["returns"]["fixed"]["inflation"] = json!(0.1);
    let res = run(v);
    close(month(&res, 2030, 3).spending, 1_000.0, "Startmonat");
    close(
        month(&res, 2030, 12).spending,
        1_000.0 * 1.1f64.powf(9.0 / 12.0),
        "Dezember 2030",
    );
    let f = 1.1f64.powf(10.0 / 12.0);
    close(month(&res, 2031, 1).spending, 1_000.0 * f, "Januar 2031");
    close(month(&res, 2032, 1).spending, 1_000.0 * f * 1.1, "Januar 2032");
}

// ---------- Pension fund ----------

/// Interest accrues monthly on the balance at the start of the month and is credited in December.
/// Contributions 1'000 per month from 0: interest = Σ (1'000 × k) × 2 % / 12 for k = 0 … 11 = 110.
#[test]
fn pension_fund_interest_accrues_monthly() {
    let mut v = base();
    v["pension"]["pk"] = json!({ "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0,
        "contribution": 1_000, "interest": 0.02 });
    let res = run(v);
    close(month(&res, 2030, 12).pension_fund, 12_000.0 + 110.0, "PK Ende 2030");
}

/// Payout in July: interest for January to June (6 months) is credited at the payout.
/// 100'000 × 2 % × 6/12 = 1'000 → capital 101'000, all drawn as capital, tax 5 %.
#[test]
fn pension_fund_pro_rata_interest_at_retirement() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-07");
    v["pension"]["pk"] = json!({ "capital": 100_000, "at": "retire", "lumpsum": 1, "conversion": 0.05,
        "interest": 0.02 });
    v["taxes"]["capital"] = json!([[0, 0.05]]);
    let res = run(v);
    close(res.summary.lumpsum_tax, 101_000.0 * 0.05, "Kapitalsteuer");
    close(res.summary.lumpsum_net, 101_000.0 * 0.95, "Kapitalbezug netto");
    close(month(&res, 2030, 7).pension_fund, 0.0, "PK leer");
}

// ---------- Taxes ----------

/// PK payout 100'000 in June and 3a payout 50'000 in September of the same year: rate from the
/// sum. Table 0 → 0 %, 200'000 → 10 %: 100'000 at 5 % = 5'000; 150'000 at 7.5 % = 11'250,
/// so 6'250 on the 3a payout. Both taxes count in «Steuern total».
#[test]
fn capital_withdrawals_in_same_year_are_combined_and_counted() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-06");
    // AHV payout 2035-01: the 3a payout in September 2030 is allowed (#52).
    v["milestones"].as_array_mut().unwrap().insert(
        2,
        json!({ "id": "ahv", "name": "AHV", "kind": "date", "date": "2035-01" }),
    );
    v["wealth"]["cash"] = json!(1_000_000);
    v["pension"]["pk"] = json!({ "capital": 100_000, "at": "retire", "lumpsum": 1, "conversion": 0 });
    v["taxes"]["capital"] = json!([[0, 0.0], [200_000, 0.10]]);
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 500_000, "sell": "never",
        "mortgage": { "tranches": [ { "amount": 100_000, "start": "2029-01",
            "periods": [ { "type": "fixed", "term": 10, "rate": 0.0 } ] } ],
            "pillar3a": { "balance": 50_000, "rate": 0.0, "payout": "2030-09", "use": "wealth" } }
    } ] });
    let res = run(v);
    close(res.summary.lumpsum_tax, 5_000.0, "PK-Kapitalsteuer");
    close(month(&res, 2030, 9).taxes, 6_250.0, "3a-Steuer im September");
    close(res.summary.taxes_total, 11_250.0, "Steuern total");
}

/// Property gain tax: brackets in the entered order; above the last bracket its rate applies.
/// Gain 30'000 with [4'000 at 10 %, 6'000 at 20 %, then 30 %]: 400 + 1'200 + 20'000 × 30 % = 7'600.
#[test]
fn property_gain_tax_brackets_in_order_and_open_ended() {
    let def = engine::model::PropertyGainTax {
        brackets: vec![(4_000.0, 0.10), (6_000.0, 0.20), (1e18, 0.30)],
        reduction: 0.0,
        ..Default::default()
    };
    close(
        tax::property_gain_tax(Some(&def), 30_000.0),
        7_600.0,
        "mit offener Stufe",
    );
    // Without an open last bracket: the rest at the last rate.
    let def = engine::model::PropertyGainTax {
        brackets: vec![(10_000.0, 0.30), (4_000.0, 0.10)],
        reduction: 0.0,
        ..Default::default()
    };
    // 10'000 × 30 % + 4'000 × 10 % + 16'000 × 10 % = 3'000 + 400 + 1'600
    close(
        tax::property_gain_tax(Some(&def), 30_000.0),
        5_000.0,
        "Reihenfolge wie erfasst",
    );
}

/// Sale with a gain: property gain tax is booked and counted in «Steuern total».
#[test]
fn property_gain_tax_is_counted() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "sell": "2030-06"
    } ] });
    v["taxes"]["propertyGain"] = json!({ "brackets": [[1e18, 0.2]], "reduction": 0.0 });
    let res = run(v);
    close(month(&res, 2030, 6).liquid, 500_000.0 - 20_000.0, "Erlös nach Steuer");
    close(month(&res, 2030, 6).taxes, 20_000.0, "Steuer im Monat");
    close(res.summary.taxes_total, 20_000.0, "Steuern total");
}

/// Canton of Zurich (tariff § 225 StG, StA eForm. 225.1, as of 09/2025).
fn zurich() -> Value {
    let mut duration = vec![json!([0, 0.5]), json!([1, 0.25]), json!([2, 0.0])];
    for y in 5..=20 {
        duration.push(json!([y, -(0.05 + 0.03 * f64::from(y - 5))]));
    }
    json!({
        "brackets": [[4000, 0.10], [6000, 0.15], [8000, 0.20], [12000, 0.25], [20000, 0.30], [50000, 0.35], [1e18, 0.40]],
        "reduction": 0.0, "duration": duration, "exemptBelow": 5000
    })
}

/// Tax amounts of the Zurich table (basic tariff) and exemption limit.
#[test]
fn zurich_base_tariff_matches_the_official_table() {
    let def: engine::model::PropertyGainTax = serde_json::from_value(zurich()).unwrap();
    for (gain, expected) in [
        (5_000.0, 550.0),
        (20_000.0, 3_400.0),
        (50_000.0, 11_900.0),
        (100_000.0, 29_400.0),
        (150_000.0, 49_400.0),
    ] {
        close(
            tax::property_gain_tax(Some(&def), gain),
            expected,
            &format!("Gewinn {gain}"),
        );
    }
    close(
        tax::property_gain_tax(Some(&def), 4_999.0),
        0.0,
        "unter 5000 steuerfrei",
    );
}

/// Surcharges and reductions by holding period in full years.
#[test]
fn zurich_holding_period_adjustments() {
    let def: engine::model::PropertyGainTax = serde_json::from_value(zurich()).unwrap();
    let base = 29_400.0;
    for (years, factor) in [
        (0, 1.5),
        (1, 1.25),
        (2, 1.0),
        (4, 1.0),
        (5, 0.95),
        (10, 0.80),
        (15, 0.65),
        (20, 0.50),
        (35, 0.50),
    ] {
        close(
            tax::property_gain_tax_held(Some(&def), 100_000.0, Some(years), 1.0),
            base * factor,
            &format!("{years} Jahre"),
        );
    }
    close(
        tax::property_gain_tax_held(Some(&def), 100_000.0, None, 1.0),
        base,
        "Dauer unbekannt",
    );
}

/// Sale: the holding period follows from the purchase and sale months.
#[test]
fn property_gain_tax_uses_the_holding_period() {
    let sell = |bought: Option<&str>| {
        let mut v = base();
        let mut prop =
            json!({ "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "sell": "2030-06" });
        if let Some(b) = bought {
            prop["bought"] = json!(b);
        }
        v["realestate"] = json!({ "properties": [prop] });
        v["taxes"]["propertyGain"] = zurich();
        run(v).summary.taxes_total
    };
    let full = 29_400.0;
    close(sell(None), full, "ohne Kaufdatum");
    close(sell(Some("2020-06")), full * 0.80, "genau 10 Jahre");
    close(
        sell(Some("2020-07")),
        full * 0.83,
        "9 Jahre und 11 Monate = 9 volle Jahre",
    );
    close(sell(Some("2029-12")), full * 1.5, "weniger als 1 Jahr");
}

/// The taxes of every month are broken down by kind and add up to the monthly amount.
#[test]
fn taxes_are_split_by_kind() {
    let mut v = base();
    v["milestones"][2]["date"] = json!("2032-12");
    v["flows"] = json!({ "income": 8_000, "spending": 3_000 });
    v["pension"]["pk"] = json!({ "capital": 300_000, "at": "retire", "lumpsum": 1.0, "conversion": 0.05 });
    v["milestones"][1]["date"] = json!("2031-06");
    v["wealth"]["cash"] = json!(600_000);
    v["taxes"] = json!({
        "income": [[0, 0.1]], "wealth": [[0, 0.001]], "capital": [[0, 0.05]], "multiplier": 1.0,
        "propertyGain": { "brackets": [[1e18, 0.2]], "reduction": 0.0 }
    });
    v["realestate"] = json!({ "properties": [ { "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "sell": "2032-02" } ] });
    let res = run(v);
    let sum = |f: &dyn Fn(&engine::sim::MonthRecord) -> f64| res.records.iter().map(f).sum::<f64>();
    for r in &res.records {
        close(
            r.tax_income + r.tax_wealth + r.tax_capital + r.tax_gain,
            r.taxes,
            &format!("Monat {}", r.month.iso()),
        );
    }
    close(sum(&|r| r.tax_capital), 300_000.0 * 0.05, "Kapitalbezugssteuer");
    close(sum(&|r| r.tax_gain), 100_000.0 * 0.2, "Grundstückgewinnsteuer");
    assert!(sum(&|r| r.tax_income) > 0.0 && sum(&|r| r.tax_wealth) > 0.0);
    // Income and wealth tax are paid in May
    assert!(res
        .records
        .iter()
        .filter(|r| r.tax_income > 0.0)
        .all(|r| r.month.month() == 5 || r.month.iso() == "2032-12"));
    close(res.records[0].planned_spending, 3_000.0, "geplante Ausgaben");
    close(
        res.records[0].withdrawal,
        3_000.0 - 8_000.0,
        "Entnahme = Ausgaben − Einkommen",
    );
}

/// A property bought during the simulation counts from the purchase month: purchase March 2030, sale June 2031
/// is 1 full year (surcharge 25 %), sale February 2032 is 1 full year, sale March 2032 2 years (no surcharge).
#[test]
fn planned_property_holding_period_starts_at_purchase() {
    let total = |sell: &str, duration: Value| {
        let mut v = base();
        v["milestones"][2]["date"] = json!("2033-12");
        v["wealth"]["cash"] = json!(2_000_000);
        v["returns"]["fixed"]["inflation"] = json!(0.05);
        v["realestate"] = json!({ "properties": [ {
            "name": "Wohnung", "status": "planned", "buy": "2030-03", "sell": sell, "price": 500_000, "inflation": true,
            "mortgage": { "tranches": [ { "amount": 0, "periods": [ { "term": 5, "rate": 0.0 } ] } ] }
        } ] });
        v["taxes"]["propertyGain"] = json!({ "brackets": [[1e18, 0.2]], "reduction": 0.0, "duration": duration });
        run(v).summary.taxes_total
    };
    let table = json!([[0, 0.5], [1, 0.25], [2, 0.0]]);
    for (sell, factor) in [("2030-12", 1.5), ("2031-06", 1.25), ("2032-02", 1.25), ("2032-03", 1.0)] {
        let plain = total(sell, json!([]));
        assert!(plain > 0.0, "{sell}: keine Steuer");
        close(total(sell, table.clone()), plain * factor, sell);
    }
}

// ---------- Properties ----------

/// A property bought during the simulation is sold at the planned sale date.
#[test]
fn planned_property_is_sold_at_its_sale_date() {
    let mut v = base();
    v["wealth"]["cash"] = json!(1_000_000);
    v["realestate"] = json!({ "properties": [ {
        "name": "Wohnung", "status": "planned", "buy": "2030-03", "sell": "2031-03", "price": 500_000,
        "mortgage": { "tranches": [ { "amount": 300_000, "periods": [ { "term": 5, "rate": 0.0 } ] } ] }
    } ] });
    let res = run(v);
    close(month(&res, 2030, 3).liquid, 800_000.0, "nach Kauf");
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::Sale && e.month == YearMonth::new(2031, 3)));
    close(month(&res, 2031, 3).liquid, 1_000_000.0, "nach Verkauf");
    close(month(&res, 2031, 3).property_equity, 0.0, "no property left");
}

/// A purchase date before the simulation start is never reached: the validation reports it.
#[test]
fn planned_purchase_before_start_is_reported() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ {
        "name": "Wohnung", "status": "planned", "buy": "2029-06", "price": 500_000
    } ] });
    let issues = validate::validate(&plan(v));
    assert!(
        issues
            .iter()
            .any(|i| i.section == "realestate" && i.message.contains("before the simulation start")),
        "{issues:?}"
    );
}

/// #2: A sale date before the simulation start is never reached: the validation warns (no error).
#[test]
fn sale_before_start_is_reported_as_a_warning() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 600_000, "sell": "2029-06"
    } ] });
    let issues = validate::validate(&plan(v));
    let hit = issues
        .iter()
        .find(|i| i.section == "realestate" && i.message.contains("the sale date is before the simulation start"));
    assert!(
        hit.is_some_and(|i| i.severity == validate::Severity::Warning),
        "{issues:?}"
    );
    // A sale within the period stays without a warning
    let mut ok = base();
    ok["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 600_000, "sell": "2031-06"
    } ] });
    assert!(!validate::validate(&plan(ok))
        .iter()
        .any(|i| i.message.contains("Verkaufszeitpunkt")));
}

/// Mortgage interest per month = debt × rate / 12; SARON is floored at 0, then the margin is added.
#[test]
fn mortgage_interest_and_saron_floor() {
    let mut v = base();
    v["wealth"]["cash"] = json!(100_000);
    v["returns"]["fixed"]["saron"] = json!(-0.0075);
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 500_000, "sell": "never",
        "mortgage": { "tranches": [
            { "name": "Fest", "amount": 240_000, "start": "2029-01", "periods": [ { "type": "fixed", "term": 10, "rate": 0.015 } ] },
            { "name": "SARON", "amount": 120_000, "start": "2029-01", "periods": [ { "type": "saron", "term": 3, "margin": 0.009 } ] }
        ] }
    } ] });
    let res = run(v);
    // 240'000 × 1.5 % / 12 + 120'000 × (0 % + 0.9 %) / 12 = 300 + 90 per month, paid at quarter end
    close(month(&res, 2030, 1).spending, 0.0, "Januar: noch kein Zins fällig");
    close(month(&res, 2030, 3).spending, 3.0 * 390.0, "Zins 1. Quartal");
}

// ---------- Wealth depleted ----------

/// Without liquid wealth but with covered spending, wealth is not «aufgebraucht».
#[test]
fn zero_wealth_with_covered_spending_is_not_depleted() {
    let mut v = base();
    v["flows"]["income"] = json!(5_000);
    v["flows"]["spending"] = json!(5_000);
    let res = run(v);
    assert_eq!(res.summary.depleted, None);
    assert!(!res.events.iter().any(|e| e.kind == EventKind::Depleted));
}

/// With uncovered spending: depleted in the month in which cash reaches 0.
#[test]
fn depleted_when_liquidity_runs_out() {
    let mut v = base();
    v["wealth"]["cash"] = json!(3_000);
    v["flows"]["spending"] = json!(1_000);
    let res = run(v);
    assert_eq!(res.summary.depleted, Some(YearMonth::new(2030, 3)));
}

// ---------- Success rate ----------

/// A run with an emergency sale that suffices afterwards counts as successful; one without money at the end does not.
#[test]
fn success_rate_counts_runs_that_last() {
    let mut v = base();
    v["wealth"]["cash"] = json!(10_000);
    v["flows"]["spending"] = json!(1_000);
    let res = run(v);
    let s = batch::summarize(&[res]);
    assert_eq!(s.success_rate, 0.0);
}

// ---------- Validation ----------

#[test]
fn example_plan_has_no_errors() {
    let p: Plan = serde_json::from_value(engine::api::example_plan()).unwrap();
    let errs: Vec<_> = validate::validate(&p)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errs.is_empty(), "{errs:?}");
}

// ---------- Series ----------

/// If the series ends before the end of the simulation, with «Fixe Werte» the fixed values of the
/// respective month apply – even if they depend on time.
#[test]
fn extrapolation_with_fixed_values_follows_timed_curve() {
    use engine::market::{self, DataSet};
    let ds = DataSet::parse_csv(
        "t",
        "Years,Stocks,Bonds,Inflation\n2000,0.1,0.02,0.01\n2001,0.1,0.02,0.01\n",
    )
    .unwrap();
    let mut v = base();
    v["returns"]["source"] = json!("historical");
    v["returns"]["series"] = json!("t.csv");
    v["returns"]["startDate"] = json!("2000-01");
    v["returns"]["fixed"]["stocks"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 0.05 }, { "from": "2031-07", "value": 0.0 } ] });
    v["milestones"][2]["date"] = json!("2032-12");
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let path = &batch::market_paths(&p, &sim, Some(&ds)).unwrap()[0];
    // 2030, 2031: series; from 2032 extrapolation with the month's fixed value (0 % from 07.2031).
    let k = |y: i32, m: u32| (YearMonth::new(y, m).0 - YearMonth::new(2030, 1).0) as usize;
    assert!((path.stocks[k(2031, 12)] - market::monthly(0.1)).abs() < 1e-15);
    assert_eq!(path.stocks[k(2032, 1)], 0.0);
}

// ---------- Property gain tax Aargau, tax multiplier, inflation per property ----------

/// Canton of Aargau (§ 109 StG): direct rate on the gain by holding period in full years.
fn aargau() -> Value {
    let mut rates = vec![];
    for y in 0..=10 {
        rates.push(json!([y, 0.40 - 0.02 * f64::from(y)]));
    }
    for y in 11..=24 {
        rates.push(json!([y, 0.30 - 0.01 * f64::from(y)]));
    }
    rates.push(json!([25, 0.05]));
    json!({ "model": "aargau", "durationRates": rates })
}

#[test]
fn aargau_rate_depends_on_full_years_held() {
    let def: engine::model::PropertyGainTax = serde_json::from_value(aargau()).unwrap();
    for (years, rate) in [
        (0, 0.40),
        (1, 0.38),
        (10, 0.20),
        (11, 0.19),
        (24, 0.06),
        (25, 0.05),
        (40, 0.05),
    ] {
        close(
            tax::property_gain_tax_held(Some(&def), 100_000.0, Some(years), 1.0),
            100_000.0 * rate,
            &format!("{years} Jahre"),
        );
    }
    // Without a purchase date the highest rate applies; a gain <= 0 stays tax-free.
    close(
        tax::property_gain_tax_held(Some(&def), 100_000.0, None, 1.0),
        40_000.0,
        "Dauer unbekannt",
    );
    close(
        tax::property_gain_tax_held(Some(&def), -5.0, Some(3), 1.0),
        0.0,
        "Verlust",
    );
}

#[test]
fn aargau_without_rates_has_no_tax_and_is_flagged() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ { "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "sell": "2030-06" } ] });
    v["taxes"]["propertyGain"] = json!({ "model": "aargau" });
    let issues = validate::validate(&plan(v));
    assert!(
        issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("Aargau")),
        "{issues:?}"
    );
    let mut v = base();
    v["taxes"]["propertyGain"] = json!({ "model": "aargau", "durationRates": [[0, 1.5]] });
    let issues = validate::validate(&plan(v));
    assert!(
        issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("invalid")),
        "{issues:?}"
    );
}

#[test]
fn aargau_warns_about_missing_purchase_date() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ { "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "sell": "2030-06" } ] });
    v["taxes"]["propertyGain"] = aargau();
    let issues = validate::validate(&plan(v));
    assert!(
        issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("purchase date")),
        "{issues:?}"
    );
}

#[test]
fn gain_tax_multiplier_applies_only_when_flagged() {
    for (model, gain_tax) in [("aargau", aargau()), ("zurich", json!({ "brackets": [[1e18, 0.2]] }))] {
        let rate = if model == "aargau" { 0.40 } else { 0.20 };
        let mut def = gain_tax.clone();
        let parsed: engine::model::PropertyGainTax = serde_json::from_value(def.clone()).unwrap();
        close(
            tax::property_gain_tax_held(Some(&parsed), 100_000.0, Some(0), 1.2),
            100_000.0 * rate,
            model,
        );
        def["useMultiplier"] = json!(true);
        let parsed: engine::model::PropertyGainTax = serde_json::from_value(def).unwrap();
        close(
            tax::property_gain_tax_held(Some(&parsed), 100_000.0, Some(0), 1.2),
            100_000.0 * rate * 1.2,
            model,
        );
    }
}

#[test]
fn aargau_sale_uses_tax_multiplier_and_holding_period() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "bought": "2025-06", "sell": "2030-06"
    } ] });
    let mut g = aargau();
    g["useMultiplier"] = json!(true);
    v["taxes"]["propertyGain"] = g;
    v["taxes"]["multiplier"] = json!(1.1);
    // 5 full years: 30 % of 100'000, times tax multiplier 1.1
    close(run(v).summary.taxes_total, 33_000.0, "Steuern total");
}

fn rent_plan(flags: Value) -> Value {
    let mut v = base();
    v["wealth"]["cash"] = json!(1_000_000);
    v["returns"]["fixed"]["inflation"] = json!(0.1);
    let mut p = json!({ "name": "Wohnung", "status": "rent", "from": "2030-01", "rent": 1_000 });
    for (k, val) in flags.as_object().unwrap() {
        p[k] = val.clone();
    }
    v["realestate"] = json!({ "properties": [p] });
    v
}

#[test]
fn rent_inflation_is_selectable_and_defaults_to_on() {
    let default = run(rent_plan(json!({})));
    let on = run(rent_plan(json!({ "inflationRent": true })));
    let off = run(rent_plan(json!({ "inflationRent": false })));
    close(
        month(&default, 2031, 1).housing,
        month(&on, 2031, 1).housing,
        "Standard = Inflation",
    );
    assert!(month(&on, 2031, 1).housing > 1_000.0 * 1.05);
    close(month(&off, 2031, 1).housing, 1_000.0, "Miete ohne Inflation");
}

#[test]
fn value_and_fix_costs_inflation_are_independent() {
    let make = |flags: Value| {
        let mut v = base();
        v["wealth"]["cash"] = json!(1_000_000);
        v["returns"]["fixed"]["inflation"] = json!(0.1);
        let mut p = json!({ "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "fixCosts": 1_000 });
        for (k, val) in flags.as_object().unwrap() {
            p[k] = val.clone();
        }
        v["realestate"] = json!({ "properties": [p] });
        run(v)
    };
    let both = make(json!({ "inflation": true }));
    let value_only = make(json!({ "inflationValue": true, "inflationFixCosts": false }));
    let costs_only = make(json!({ "inflationValue": false, "inflationFixCosts": true }));
    let none = make(json!({}));
    let (v_both, v_val, v_cost, v_none) = (
        month(&both, 2031, 1),
        month(&value_only, 2031, 1),
        month(&costs_only, 2031, 1),
        month(&none, 2031, 1),
    );
    close(v_none.property_value, 500_000.0, "ohne Inflation");
    close(v_val.property_value, v_both.property_value, "Wert mit Inflation");
    close(v_cost.property_value, 500_000.0, "Wert ohne Inflation");
    assert!(v_both.property_value > 520_000.0);
    close(v_val.housing, 1_000.0, "Nebenkosten ohne Inflation");
    close(v_cost.housing, v_both.housing, "Nebenkosten mit Inflation");
    assert!(v_both.housing > 1_050.0);
}

// ---------- Templates of the property gain tax (#138) ----------

fn template(name: &str) -> engine::model::PropertyGainTax {
    serde_json::from_value(engine::api::gain_templates()[name].clone()).expect("Vorlage passt zum Modell")
}

fn gain(name: &str, profit: f64, years: i32) -> f64 {
    tax::property_gain_tax_held(Some(&template(name)), profit, Some(years), 1.0)
}

/// #138/AK2, AK3, AK4: Zurich template against the official tariff sheet «Tarif für die Grundstückgewinnsteuer, Kanton Zürich»
/// (§ 225 StG, StA eForm. 225.1, as of 09.25, www.steueramt.zh.ch). Part B of the sheet gives the basic tariff: gain
/// 5'000 → 550, 30'000 → 5'900, 50'000 → 11'900, 100'000 → 29'400, 150'000 → 49'400. Part A: surcharge 50 % (under 1 year)
/// or 25 % (under 2 years), reduction 5 % after 5 full years, 3 percentage points more per year, 50 % from 20 years;
/// gains under 5'000 are tax-free.
#[test]
fn issue_138_ak3_zurich_template_matches_the_official_tariff() {
    // Basic tariff (2 to 4 full years: neither surcharge nor reduction)
    for (profit, official) in [
        (5_000.0, 550.0),
        (30_000.0, 5_900.0),
        (50_000.0, 11_900.0),
        (100_000.0, 29_400.0),
        (150_000.0, 49_400.0),
    ] {
        assert_eq!(gain("zurich", profit, 3), official, "Grundtarif {profit}");
    }
    // AK3, example: 100'000 after 10 full years: 29'400 × (1 − 20 %) = 23'520.00
    assert_eq!(gain("zurich", 100_000.0, 10), 23_520.0);
    // Surcharge: under 1 year 29'400 × 1.5 = 44'100; under 2 years 29'400 × 1.25 = 36'750
    assert_eq!(gain("zurich", 100_000.0, 0), 44_100.0);
    assert_eq!(gain("zurich", 100_000.0, 1), 36_750.0);
    // Reduction: 5 full years 11'900 × 0.95 = 11'305; 18 years 29'400 × 0.56 = 16'464; 20 and 30 years 49'400 × 0.5
    assert_eq!(gain("zurich", 50_000.0, 5), 11_305.0);
    assert_eq!(gain("zurich", 100_000.0, 18), 16_464.0);
    assert_eq!(gain("zurich", 150_000.0, 20), 24_700.0);
    assert_eq!(gain("zurich", 150_000.0, 30), 24_700.0);
    // Exemption limit: under 5'000 tax-free, from 5'000 the full tariff
    assert_eq!(gain("zurich", 4_999.0, 3), 0.0);
    assert_eq!(gain("zurich", 5_000.0, 3), 550.0);
}

/// #138/AK2, AK5: Aargau template against the canton's tariff (§ 109 StG, ag.ch «Grundstückgewinnsteuer», published
/// 01.07.2025): 40 % until the completed 1st year of ownership, 2 percentage points less per year down to 20 % (until the completed
/// 11th year of ownership), then 1 percentage point less per year, 5 % from the completed 25th year of ownership. Holding period in full years.
#[test]
fn issue_138_ak5_aargau_template_matches_the_official_tariff() {
    for (years, rate) in [
        (0, 0.40),
        (1, 0.38),
        (9, 0.22),
        (10, 0.20),
        (11, 0.19),
        (24, 0.06),
        (25, 0.05),
        (40, 0.05),
    ] {
        assert_eq!(gain("aargau", 100_000.0, years), 100_000.0 * rate, "{years} Jahre");
    }
    // Without a purchase date the highest rate applies
    let t = tax::property_gain_tax_held(Some(&template("aargau")), 100_000.0, None, 1.0);
    assert_eq!(t, 40_000.0);
}

/// #138/AK1: The templates exist only in the calculation engine; the user interface loads them with the command `gain_templates` and contains
/// no rates of its own any more.
#[test]
fn issue_138_ak1_templates_have_a_single_source() {
    let ui = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../app/ui/app.js")).unwrap();
    assert!(ui.contains(r#"Engine.call("gain_templates")"#));
    for numbers in [
        "[4000, 0.10]",
        "[4000, 0.1]",
        "40 - 2 * y",
        "(30 - y) / 100",
        "0.05 + 0.03 * i",
    ] {
        assert!(!ui.contains(numbers), "Vorlage noch in app.js: {numbers}");
    }
    let t = engine::api::gain_templates();
    assert_eq!(t["zurich"]["model"], "zurich");
    assert_eq!(t["aargau"]["model"], "aargau");
}
