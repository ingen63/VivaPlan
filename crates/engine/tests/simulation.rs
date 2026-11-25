//! Behaviour tests of the calculation engine on small plans that can be recomputed by hand.

use std::path::Path;

use engine::batch;
use engine::market::{self, DataSet};
use engine::model::Plan;
use engine::sim::{EventKind, Simulator};
use engine::time::YearMonth;
use engine::validate::{self, Severity};
use serde_json::{json, Value};

use crate::plan_builder::PlanBuilder;

/// Minimal plan: 2 years from January 2030, no returns, no inflation, no taxes.
fn base() -> Value {
    PlanBuilder::new("Test").dates("2030-01", "2040-01", "2031-12").build()
}

fn plan(v: Value) -> Plan {
    serde_json::from_value(crate::any_age(v)).expect("Planung gültig")
}

fn run(p: &Plan) -> engine::sim::SimResult {
    let sim = Simulator::new(p).unwrap();
    let paths = batch::market_paths(p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

#[test]
fn money_runs_out_after_twelve_months() {
    let mut v = base();
    v["wealth"]["cash"] = json!(12_000);
    v["flows"]["spending"] = json!(1_000);
    let res = run(&plan(v));
    assert_eq!(res.summary.depleted, Some(YearMonth::new(2030, 12)));
    assert!(res.events.iter().any(|e| e.kind == EventKind::Depleted));
}

#[test]
fn stocks_grow_with_yearly_return() {
    let mut v = base();
    v["wealth"]["stocks"] = json!(100_000);
    v["returns"]["fixed"]["stocks"] = json!(0.06);
    let res = run(&plan(v));
    assert!((res.records[11].total - 106_000.0).abs() < 1e-6);
    assert!((res.summary.end_total - 100_000.0 * 1.06f64.powi(2)).abs() < 1e-6);
}

#[test]
fn spending_is_indexed_monthly_and_ahv_every_january() {
    let mut v = base();
    v["wealth"]["cash"] = json!(100_000);
    v["flows"]["spending"] = json!(1_000);
    v["returns"]["fixed"]["inflation"] = json!(0.1);
    v["pension"]["ahv"] = json!({ "monthly": 2_000, "at": "start", "indexed": true });
    let res = run(&plan(v));
    let r = &res.records;
    assert!(
        (r[0].spending - 1_000.0).abs() < 1e-9,
        "Start im Januar: keine Anpassung"
    );
    assert!(
        (r[11].spending - 1_000.0 * 1.1f64.powf(11.0 / 12.0)).abs() < 1e-9,
        "Dezember: elf Monate Teuerung"
    );
    assert!((r[12].spending - 1_100.0).abs() < 1e-9, "Januar: volles Jahr");
    assert!((r[10].income - 2_000.0).abs() < 1e-9, "AHV bis November unverändert");
    // #190: December with the 13th AHV pension (from 2026 in every plan year of this test).
    assert!((r[11].income - 4_000.0).abs() < 1e-9, "Dezember mit 13. AHV-Rente");
    assert!((r[12].income - 2_200.0).abs() < 1e-9, "AHV im Januar angepasst");
}

#[test]
fn pension_fund_lumpsum_and_annuity() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-06");
    v["pension"]["pk"] = json!({ "capital": 1_000_000, "at": "retire", "lumpsum": 0.4, "conversion": 0.06 });
    v["taxes"]["capital"] = json!([[0, 0.05]]);
    let res = run(&plan(v));
    let s = &res.summary;
    assert!((s.lumpsum_tax - 20_000.0).abs() < 1e-6);
    assert!((s.lumpsum_net - 380_000.0).abs() < 1e-6);
    assert!((s.pension_fund_monthly - 3_000.0).abs() < 1e-6);
    // June to December: 7 monthly pensions
    assert!((res.records[11].liquid - (380_000.0 + 7.0 * 3_000.0)).abs() < 1e-6);
}

#[test]
fn income_and_wealth_tax_paid_in_may_of_next_year() {
    let mut v = base();
    v["wealth"]["cash"] = json!(500_000);
    v["flows"]["income"] = json!(10_000);
    v["taxes"] = json!({ "multiplier": 2.0, "income": [[0, 0.05]], "wealth": [[0, 0.001]], "capital": [] });
    let res = run(&plan(v));
    // 120'000 × 5 % × 2 + (500'000 + 120'000) × 0.1 % × 2
    let expected = 12_000.0 + 1_240.0;
    assert!(res.records[11].taxes.abs() < 1e-6, "Dezember: nur veranlagt");
    assert!((res.records[16].taxes - expected).abs() < 1e-6, "Mai 2031");
    assert!((res.records[16].liquid - (500_000.0 + 17.0 * 10_000.0 - expected)).abs() < 1e-6);
}

fn with_house(mortgage: Value) -> Plan {
    let mut v = base();
    v["wealth"]["cash"] = json!(1_000_000);
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000,
        "sell": "never", "fixCosts": 0, "mortgage": mortgage
    } ] });
    plan(v)
}

#[test]
fn direct_amortization_each_quarter_and_repayment() {
    let p = with_house(json!({ "tranches": [ {
        "amount": 100_000, "start": "2029-01", "after": "repay",
        "periods": [ { "type": "fixed", "term": 2, "rate": 0.012, "amort": 1_000 } ]
    } ] }));
    let res = run(&p);
    let r = &res.records;
    // Period 01.2029 – 01.2031, then repayment of the remaining debt
    assert!(
        (r[0].property_equity - (800_000.0 - 100_000.0)).abs() < 1e-6,
        "Januar: keine Amortisation"
    );
    assert!(
        (r[2].property_equity - (800_000.0 - 99_000.0)).abs() < 1e-6,
        "März: erste Amortisation"
    );
    assert!((r[11].property_equity - (800_000.0 - 96_000.0)).abs() < 1e-6);
    assert!(
        (r[12].property_equity - 800_000.0).abs() < 1e-6,
        "Januar 2031: Restschuld zurückbezahlt"
    );
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::MortgageRepayment && e.month == YearMonth::new(2031, 1)));
    // Interest is paid at quarter end: nothing in January, in March three months of 100'000 × 1.2 % / 12
    assert!(r[0].spending.abs() < 1e-9);
    assert!(
        (r[2].spending - (3.0 * 100.0 + 1_000.0)).abs() < 1e-9,
        "März {}",
        r[2].spending
    );
}

#[test]
fn indirect_amortization_goes_to_pillar_3a() {
    let mut p = with_house(json!({
        "tranches": [
            { "name": "T1", "amount": 300_000, "start": "2029-01",
              "periods": [ { "type": "fixed", "term": 10, "rate": 0.01, "amort": 0 } ] },
            { "name": "T2", "amount": 100_000, "start": "2029-01",
              "periods": [ { "type": "fixed", "term": 10, "rate": 0.01, "amort": 1_500, "amortMode": "indirect" } ] }
        ],
        "pillar3a": { "balance": 10_000, "rate": 0.0, "payout": "2031-06", "use": "repay" }
    }));
    // AHV payout 2035-01: indirect until then (#50), payout 2031-06 allowed (#52).
    p.milestones.push(
        serde_json::from_value(json!({ "id": "ahv", "name": "AHV", "kind": "date", "date": "2035-01" })).unwrap(),
    );
    let res = run(&p);
    let r = &res.records;
    assert!((r[11].pillar3a - 16_000.0).abs() < 1e-6, "4 Quartale à 1'500");
    assert!((r[11].property_equity - 400_000.0).abs() < 1e-6, "Schuld bleibt gleich");
    // Payout June 2031 after the June contribution: 10'000 + 6 × 1'500 = 19'000, repays T2
    let june = &r[17];
    assert_eq!(june.pillar3a, 0.0);
    assert!((june.property_equity - (800_000.0 - 300_000.0 - 81_000.0)).abs() < 1e-6);
}

#[test]
fn saron_period_uses_market_rate_plus_margin() {
    let mut p = with_house(json!({ "tranches": [ {
        "amount": 120_000, "start": "2030-01",
        "periods": [ { "type": "saron", "term": 3, "margin": 0.008 } ]
    } ] }));
    p.returns.fixed.saron = engine::model::Series::Number(0.004);
    let res = run(&p);
    assert!(res.records[1].spending.abs() < 1e-9);
    assert!((res.records[2].spending - 3.0 * 120_000.0 * 0.012 / 12.0).abs() < 1e-9);
}

#[test]
fn planned_purchase_and_scheduled_sale() {
    let mut v = base();
    v["wealth"]["cash"] = json!(1_000_000);
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "move", "name": "Umzug", "kind": "date", "date": "2031-01" }));
    v["realestate"] = json!({ "properties": [
        { "name": "Haus", "status": "owned", "price": 400_000, "worth": 600_000, "sell": "move" },
        { "name": "Wohnung", "status": "planned", "buy": "move", "price": 500_000,
          "mortgage": { "tranches": [ { "amount": 200_000, "periods": [ { "term": 5, "rate": 0.0 } ] } ] } }
    ] });
    let res = run(&plan(v));
    let jan = &res.records[12];
    // Sale +600'000, purchase −(500'000 − 200'000)
    assert!((jan.liquid - 1_300_000.0).abs() < 1e-6);
    assert!((jan.property_equity - 300_000.0).abs() < 1e-6);
}

#[test]
fn rolling_runs_every_start_year_in_parallel() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let ds = DataSet::load_csv(&dir.join("renditen-muster.csv")).unwrap();
    let mut v = base();
    v["wealth"]["stocks"] = json!(100_000);
    v["returns"]["source"] = json!("rolling");
    v["returns"]["series"] = json!("renditen-muster.csv");
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, Some(&ds)).unwrap();
    assert_eq!(paths.len(), ds.len() - 1, "2 Kalenderjahre pro Lauf");
    let parallel = batch::run_all(&sim, &paths);
    for (path, res) in paths.iter().zip(&parallel) {
        assert_eq!(sim.run(path, false).summary.end_total, res.summary.end_total);
    }
    // First run: years 1956 and 1957
    let expected = 100_000.0 * (1.0 + ds.stocks[0]) * (1.0 + ds.stocks[1]);
    assert!((parallel[0].summary.end_total - expected).abs() < 1e-6);
    assert_eq!(market::rolling_starts(&ds, sim.start, sim.months()).len(), paths.len());
    let summary = batch::summarize(&parallel);
    assert_eq!(summary.runs, paths.len());
    assert_eq!(summary.success_rate, 1.0);
}

#[test]
fn validation_reports_errors() {
    let mut v = base();
    v["milestones"].as_array_mut().unwrap().retain(|m| m["id"] != "end");
    let issues = validate::validate(&plan(v));
    assert!(issues
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "milestones"));

    let mut v = base();
    v["pension"]["pk"]["lumpsum"] = json!(1.5);
    v["flows"]["spending"] = json!({ "mode": "timed", "rows": [ { "from": "nirgends", "value": 1 } ] });
    let p = plan(v);
    let issues = validate::validate(&p);
    assert!(issues.iter().any(|i| i.message.contains("lump sum")));
    assert!(issues.iter().any(|i| i.message.contains("nirgends")));
    assert!(Simulator::new(&p).is_err());
}

#[test]
fn example_plan_runs_and_roundtrips() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/example.vplan");
    let p = Plan::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let res = run(&p);
    assert!(res.summary.depleted.is_none());
    let again = Plan::from_json(&p.to_json()).unwrap();
    assert_eq!(run(&again).summary.end_total, res.summary.end_total);
}

/// House sold in January 2031; afterwards a planned flat «nach Verkauf» and rent.
fn after_sale_plan(cash: f64, income: f64, flat_price: f64) -> Plan {
    let mut v = base();
    v["wealth"]["cash"] = json!(cash);
    v["flows"]["income"] = json!(income);
    v["realestate"] = json!({ "properties": [
        { "name": "Haus", "status": "owned", "price": 300_000, "worth": 400_000, "sell": "2031-01" },
        { "name": "Wohnung", "status": "planned", "buy": "afterSale", "price": flat_price, "fixCosts": 500,
          "mortgage": { "tranches": [ { "amount": flat_price * 0.6, "periods": [ { "term": 10, "rate": 0.01 } ] } ] } },
        { "name": "Miete", "status": "rent", "from": "afterSale", "rent": 2_000 }
    ] });
    plan(v)
}

#[test]
fn buys_next_property_after_sale_if_affordable() {
    let res = run(&after_sale_plan(100_000.0, 10_000.0, 500_000.0));
    let feb = &res.records[13];
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::Purchase && e.month == YearMonth::new(2031, 2)));
    assert!(
        (feb.property_equity - 200_000.0).abs() < 1e-6,
        "Wohnung 500'000 − Hypothek 300'000"
    );
    assert!(!res.events.iter().any(|e| e.kind == EventKind::RentStart));
}

#[test]
fn reduces_mortgage_when_not_sustainable() {
    // Affordable from income only: (1/3 × annual income − fix costs) / 5 %; wealth does not count.
    let res = run(&after_sale_plan(300_000.0, 2_000.0, 500_000.0));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    let debt = 500_000.0 - res.records[13].property_equity;
    // (1/3 × 24'000 − 6'000) / 5 % = 40'000
    assert!((debt - 40_000.0).abs() < 1.0, "gekürzte Hypothek {debt}: {}", buy.text);
}

/// #233/AK3, AK4: The plan file has no affordability any more (the app inserts the values of its settings before every
/// calculation): without it the defaults apply (loan-to-value 80 %); 70 % in the plan gives a smaller mortgage.
#[test]
fn issue_233_ak3_loan_to_value_from_the_plan_or_the_default() {
    let debt = |ltv: Option<f64>| {
        let mut p = after_sale_plan(100_000.0, 10_000.0, 500_000.0);
        p.realestate.properties[1].mortgage.as_mut().unwrap().tranches[0].amount = 450_000.0;
        if let Some(v) = ltv {
            p.realestate.affordability.max_loan_to_value = v;
        }
        500_000.0 - run(&p).records[13].property_equity
    };
    // after_sale_plan has no `affordability` in its JSON: the defaults apply
    let a = after_sale_plan(0.0, 0.0, 1.0).realestate.affordability;
    assert_eq!(
        (a.sustainability, a.imputed_rate, a.fix_costs, a.max_loan_to_value),
        (1.0 / 3.0, 0.05, 0.01, 0.8)
    );
    assert!((debt(None) - 400_000.0).abs() < 1e-6, "{}", debt(None));
    assert!((debt(Some(0.7)) - 350_000.0).abs() < 1e-6, "{}", debt(Some(0.7)));
}

/// Mortgage after the purchase (price minus equity of the flat in February 2031).
fn debt_after_buy(cash: f64) -> (f64, String) {
    let res = run(&after_sale_plan(cash, 2_000.0, 500_000.0));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    (500_000.0 - res.records[13].property_equity, buy.text.to_string())
}

#[test]
fn affordability_counts_only_income_and_reports_the_lump_sum_repayment() {
    let (low, text) = debt_after_buy(300_000.0);
    let (high, _) = debt_after_buy(3_000_000.0);
    assert!(
        (low - high).abs() < 1e-6,
        "Vermögen darf die Tragbarkeit nicht verändern: {low} gegen {high}"
    );
    assert!(text.contains("one-off repayment") && text.contains("CHF"), "{text}");
    assert!(!text.contains("Amortisation"), "{text}");
}

#[test]
fn no_lump_sum_repayment_when_the_mortgage_is_sustainable() {
    let res = run(&after_sale_plan(100_000.0, 10_000.0, 500_000.0));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    assert!(!buy.text.contains("one-off repayment"), "{}", buy.text);
}

/// House with a mortgage of 400'000 (period 1 until the end of 2030, renewal in January 2031) and the given income and wealth.
fn renewal_plan(income: f64, cash: f64) -> Plan {
    let mut v = base();
    v["wealth"]["cash"] = json!(cash);
    v["flows"]["income"] = json!(income);
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 400_000, "start": "2030-01", "after": "renew",
            "periods": [ { "type": "fixed", "term": 1, "rate": 0.01 } ] } ] }
    } ] });
    plan(v)
}

/// Four cases of the affordability check at renewal (mortgage 400'000, house 800'000, renewal January 2031).
/// `emergency_months`: threshold of the emergency sale criterion; `spending`: spending per month.
fn renewal_case(income: f64, cash: f64, spending: f64, emergency_months: f64) -> engine::sim::SimResult {
    let mut v = base();
    v["wealth"]["cash"] = json!(cash);
    v["flows"]["income"] = json!(income);
    v["flows"]["spending"] = json!(spending);
    v["realestate"] = json!({ "emergencyMonths": emergency_months, "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 400_000, "start": "2030-01", "after": "renew",
            "periods": [ { "type": "fixed", "term": 1, "rate": 0.01 } ] } ] }
    } ] });
    run(&plan(v))
}

fn jan_2031(res: &engine::sim::SimResult) -> &engine::sim::Renewal {
    res.renewals
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1))
        .expect("Erneuerung Januar 2031")
}

#[test]
fn renewal_cases_affordable_lump_sum_and_two_emergency_sales() {
    // 1. Affordable: income 20'000/month → normal renewal
    let a = renewal_case(20_000.0, 2_000_000.0, 0.0, 6.0);
    let r = jan_2031(&a);
    assert!(r.affordable && r.lump_sum == 0.0 && r.emergency_sale.is_none());
    assert!((r.debt_after - 400_000.0).abs() < 1e-6);
    assert_eq!((a.summary.lump_sums, a.summary.affordability_sales), (0, 0));
    assert!(a.events.iter().any(|e| e.text.starts_with("New mortgage period")));

    // 2. Not affordable, lump sum possible: income 6'000 → affordable 320'000, payment 80'000, renewal with 320'000
    let b = renewal_case(6_000.0, 2_000_000.0, 0.0, 6.0);
    let r = jan_2031(&b);
    assert!(!r.affordable && r.emergency_sale.is_none());
    assert!((r.lump_sum - 80_000.0).abs() < 1e-6 && (r.debt_after - 320_000.0).abs() < 1e-6);
    assert_eq!((b.summary.lump_sums, b.summary.affordability_sales), (1, 0));
    assert!(b
        .events
        .iter()
        .any(|e| e.text.starts_with("New mortgage period") && e.text.contains("320000")));
    assert!(!b.events.iter().any(|e| e.kind == EventKind::EmergencySale));

    // 3. The lump sum would bring wealth below 0: income 2'000 → payment 400'000, wealth 10'000
    let c = renewal_case(2_000.0, 10_000.0, 0.0, 6.0);
    let r = jan_2031(&c);
    assert!(!r.affordable && r.lump_sum == 0.0);
    assert_eq!(r.emergency_sale.as_deref(), Some("Wealth < 0"));
    assert_eq!((c.summary.lump_sums, c.summary.affordability_sales), (0, 1));
    assert!(
        !c.events
            .iter()
            .any(|e| e.month == YearMonth::new(2031, 1) && e.text.starts_with("New mortgage period")),
        "not renewed"
    );
    assert_eq!(
        c.events
            .iter()
            .find(|e| e.kind == EventKind::EmergencySale)
            .expect("emergency sale")
            .month,
        YearMonth::new(2031, 2)
    );

    // 4. Payment possible (wealth >= 0 afterwards), but afterwards the emergency sale criterion triggers:
    //    income 6'000, spending 8'000 → wealth in January 2031 about 92'000, payment 80'000 → rest 12'000; 6 × shortfall (8'000 + interest − 6'000) ≈ 13'600 > rest
    let d = renewal_case(6_000.0, 120_000.0, 8_000.0, 6.0);
    let r = jan_2031(&d);
    assert!(!r.affordable && r.lump_sum == 0.0);
    assert_eq!(r.emergency_sale.as_deref(), Some("Emergency sale criterion"), "{r:?}");
    assert_eq!((d.summary.lump_sums, d.summary.affordability_sales), (0, 1));
    assert!(d.events.iter().any(|e| e.text.contains("emergency sale criterion")));
    // Without an emergency sale threshold the same payment is made
    let d0 = renewal_case(6_000.0, 120_000.0, 8_000.0, 0.0);
    assert!((jan_2031(&d0).lump_sum - 80_000.0).abs() < 1e-6);
}

#[test]
fn renewal_counts_show_up_in_the_batch_summary() {
    let mut v = base();
    v["wealth"]["cash"] = json!(2_000_000);
    v["flows"]["income"] = json!(6_000);
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 400_000, "start": "2030-01", "after": "renew",
            "periods": [ { "type": "fixed", "term": 1, "rate": 0.01 } ] } ] }
    } ] });
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    let results: Vec<_> = paths.iter().map(|path| sim.run(path, false)).collect();
    let s = batch::summarize(&results);
    assert_eq!(s.lump_sum_runs, results.len());
    assert_eq!(s.affordability_sale_runs, 0);
    assert!(s
        .percentiles
        .iter()
        .all(|p| p.lump_sums == 1 && p.affordability_sales == 0));
}

fn debt_in(res: &engine::sim::SimResult, y: i32, m: u32) -> f64 {
    res.records
        .iter()
        .find(|r| r.month == YearMonth::new(y, m))
        .expect("Monat")
        .mortgage
}

#[test]
fn renewal_checks_affordability_with_income_only() {
    // Income 2'000/month: affordable is (1/3 × 24'000 − 1 % × 800'000 fix costs) / 5 % = 0
    // → the whole mortgage (400'000) is repaid in January 2031.
    let res = run(&renewal_plan(2_000.0, 2_000_000.0));
    let ev = res
        .events
        .iter()
        .find(|e| e.text.contains("One-off mortgage repayment"))
        .expect("One-off mortgage repayment");
    assert_eq!(ev.month, YearMonth::new(2031, 1), "{}", ev.text);
    assert!(ev.text.contains("CHF"), "{}", ev.text);
    assert!(debt_in(&res, 2030, 12) > 399_000.0);
    assert!(
        debt_in(&res, 2031, 1) < 1.0,
        "Hypothek nach Erneuerung {}",
        debt_in(&res, 2031, 1)
    );
    // Wealth falls by the one-off repayment.
    let before = res
        .records
        .iter()
        .find(|r| r.month == YearMonth::new(2030, 12))
        .unwrap()
        .liquid;
    let after = res
        .records
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1))
        .unwrap()
        .liquid;
    assert!(before - after > 390_000.0, "{before} → {after}");
}

#[test]
fn renewal_reduces_only_the_unaffordable_part() {
    // Income 6'000/month: (1/3 × 72'000 − 8'000) / 5 % = 320'000 affordable → 80'000 one-off repayment.
    let res = run(&renewal_plan(6_000.0, 2_000_000.0));
    let ev = res
        .events
        .iter()
        .find(|e| e.text.contains("One-off mortgage repayment"))
        .expect("One-off mortgage repayment");
    assert!(ev.text.contains("80000 CHF"), "{}", ev.text);
    assert!(
        (debt_in(&res, 2031, 1) - 320_000.0).abs() < 1.0,
        "{}",
        debt_in(&res, 2031, 1)
    );
}

#[test]
fn renewal_without_affordability_problem_has_no_lump_sum() {
    let res = run(&renewal_plan(20_000.0, 2_000_000.0));
    assert!(!res.events.iter().any(|e| e.text.contains("One-off mortgage repayment")));
    assert!((debt_in(&res, 2031, 1) - 400_000.0).abs() < 1.0);
}

#[test]
fn renewal_triggers_an_emergency_sale_when_the_wealth_cannot_pay_the_lump_sum() {
    let res = run(&renewal_plan(2_000.0, 10_000.0));
    let ev = res
        .events
        .iter()
        .find(|e| e.text.contains("no longer affordable"))
        .expect("Hinweis");
    assert_eq!(ev.month, YearMonth::new(2031, 1));
    assert!(ev.text.contains("emergency sale"), "{}", ev.text);
    assert!(!res
        .events
        .iter()
        .any(|e| e.text.starts_with("One-off mortgage repayment")));
    assert!(
        debt_in(&res, 2031, 1) > 399_000.0,
        "Hypothek bleibt bis zum Verkauf unverändert"
    );
    // The emergency sale follows in the following month: mortgage repaid, proceeds in wealth.
    let sale = res
        .events
        .iter()
        .find(|e| e.kind == EventKind::EmergencySale)
        .expect("emergency sale");
    assert_eq!(sale.month, YearMonth::new(2031, 2));
    assert!(debt_in(&res, 2031, 2) < 1.0);
    let liquid = |y, m| {
        res.records
            .iter()
            .find(|r| r.month == YearMonth::new(y, m))
            .unwrap()
            .liquid
    };
    assert!(
        liquid(2031, 2) - liquid(2031, 1) > 300_000.0,
        "Erlös nach Hypothek (800'000 − 400'000)"
    );
}

#[test]
fn renewal_with_several_properties_and_tranches_and_without_balance() {
    let mut v = base();
    v["wealth"]["cash"] = json!(2_000_000);
    v["flows"]["income"] = json!(6_000);
    v["flows"]["spending"] = json!(1_000);
    let tr = |amount: f64, amort: f64| {
        json!({ "amount": amount, "start": "2030-01", "after": "renew",
            "periods": [ { "type": "fixed", "term": 1, "rate": 0.01, "amort": amort } ] })
    };
    v["realestate"] = json!({ "emergencyMonths": 6, "properties": [
        { "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
          "mortgage": { "tranches": [ tr(250_000.0, 0.0), tr(150_000.0, 1_000.0) ] } },
        { "name": "Ferienhaus", "status": "owned", "price": 100_000, "worth": 200_000, "sell": "never", "fixCosts": 100,
          "mortgage": { "tranches": [ tr(4_000.0, 1_000.0), tr(20_000.0, 500.0) ] } },
        { "name": "Garage", "status": "owned", "price": 10_000, "worth": 20_000, "sell": "never", "fixCosts": 0,
          "mortgage": { "tranches": [ tr(4_000.0, 1_000.0) ] } }
    ] });
    let res = run(&plan(v));
    let haus = res
        .renewals
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1) && r.property == "Haus")
        .unwrap();
    assert!(!haus.affordable);
    assert!(haus.lump_sum > 0.0 && haus.debt_after < haus.debt_before);
    assert!(
        haus.emergency_sale.is_none(),
        "Vermögen reicht: Einmalzahlung statt Notverkauf"
    );
    // Holiday home: tranche 1 is amortized, tranche 2 (about 18'000) is affordable
    let holiday = res
        .renewals
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1) && r.property == "Ferienhaus")
        .unwrap();
    assert!(holiday.affordable && holiday.lump_sum == 0.0, "{holiday:?}");
    assert!(holiday.debt_before < 20_000.0);
    assert!(
        !res.renewals
            .iter()
            .any(|r| r.property == "Garage" && r.month == YearMonth::new(2031, 1)),
        "vollständig amortisiert: keine Prüfung"
    );
}

#[test]
fn rents_after_sale_if_purchase_is_not_possible() {
    let res = run(&after_sale_plan(0.0, 3_000.0, 2_000_000.0));
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::RentStart && e.month == YearMonth::new(2031, 2)));
    assert!(res.records[12].spending.abs() < 1e-9, "Januar: noch keine Miete");
    assert!((res.records[13].spending - 2_000.0).abs() < 1e-9, "Februar: Miete");
}

#[test]
fn emergency_sale_then_rent() {
    let mut v = base();
    v["wealth"]["cash"] = json!(30_000);
    v["flows"]["spending"] = json!(5_000);
    v["realestate"] = json!({ "emergencyMonths": 3, "properties": [
        { "name": "Haus", "status": "owned", "price": 300_000, "worth": 400_000, "sell": "never" },
        { "name": "Miete", "status": "rent", "from": "afterSale", "rent": 1_500 }
    ] });
    let res = run(&plan(v));
    let sale = res
        .events
        .iter()
        .find(|e| e.kind == EventKind::EmergencySale)
        .expect("emergency sale");
    let rent = res
        .events
        .iter()
        .find(|e| e.kind == EventKind::RentStart)
        .expect("Miete");
    assert_eq!(rent.month, sale.month.plus(1));
    assert!(res.summary.depleted.is_none());
}

#[test]
fn buys_after_sale_of_a_specific_property() {
    let mut v = base();
    v["wealth"]["cash"] = json!(500_000);
    v["flows"]["income"] = json!(10_000);
    v["realestate"] = json!({ "properties": [
        { "id": "haus", "name": "Haus", "status": "owned", "price": 300_000, "worth": 400_000, "sell": "2031-01" },
        { "id": "ferien", "name": "Ferienwohnung", "status": "owned", "price": 200_000, "worth": 250_000, "sell": "never" },
        { "id": "whg", "name": "Wohnung", "status": "planned", "buy": "afterSale:haus", "price": 500_000,
          "mortgage": { "tranches": [ { "amount": 300_000, "periods": [ { "term": 10, "rate": 0.01 } ] } ] } }
    ] });
    let res = run(&plan(v));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    assert_eq!(buy.month, YearMonth::new(2031, 2), "{}", buy.text);
    // Holiday flat 250'000 + flat 500'000 − 300'000
    assert!((res.records[13].property_equity - 450_000.0).abs() < 1e-6);
}

fn max_loan_plan(income: f64) -> Plan {
    let mut v = base();
    v["wealth"]["cash"] = json!(600_000);
    v["flows"]["income"] = json!(income);
    v["realestate"] = json!({ "properties": [
        { "id": "whg", "name": "Wohnung", "status": "planned", "buy": "2030-06", "price": 500_000, "fixCosts": 400,
          "mortgage": { "amountMode": "maxLoan", "tranches": [
            { "name": "T1", "amount": 3, "periods": [ { "term": 5, "rate": 0.01 } ] },
            { "name": "T2", "amount": 1, "periods": [ { "term": 10, "rate": 0.015 } ] } ] } }
    ] });
    plan(v)
}

#[test]
fn max_loan_takes_eighty_percent_split_by_shares() {
    let res = run(&max_loan_plan(15_000.0));
    let june = &res.records[5];
    assert!(
        (june.property_equity - 100_000.0).abs() < 1e-6,
        "Hypothek 400'000 = 80 %"
    );
    // Interest June: 300'000 × 1 % / 12 + 100'000 × 1.5 % / 12 + fix costs 400
    assert!((june.spending - (250.0 + 125.0 + 400.0)).abs() < 1e-6);
}

#[test]
fn max_loan_is_reduced_when_not_sustainable() {
    let res = run(&max_loan_plan(3_000.0));
    let debt = 500_000.0 - res.records[5].property_equity;
    assert!(debt > 0.0 && debt < 400_000.0, "gekürzte Hypothek {debt}");
}

#[test]
fn reports_unknown_sale_reference() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [
        { "name": "Wohnung", "status": "planned", "buy": "afterSale:gibtsnicht", "price": 500_000 }
    ] });
    let p = plan(v);
    assert!(validate::validate(&p)
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "realestate"));
    assert!(Simulator::new(&p).is_err());
}

fn three_buckets(threshold: f64) -> Value {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-01");
    v["wealth"]["stocks"] = json!(100_000);
    v["flows"]["spending"] = json!(1_000);
    v["withdrawal"] = json!({
        "strategy": "three_buckets",
        "buckets": { "cashYears": 1, "bondYears": 2, "refillThreshold": threshold }
    });
    v
}

#[test]
fn three_buckets_split_and_refill_in_january() {
    let res = run(&plan(three_buckets(0.0)));
    // Retirement in January 2030: trigger A, everything in bucket 3, then 12'000 / 24'000 / 64'000 (before the payment of 1'000).
    let first = &res.bucket_refills[0];
    assert_eq!(first.trigger, "A");
    assert_eq!(first.after, [12_000.0, 24_000.0, 64_000.0]);
    let r = &res.records[0];
    assert_eq!((r.cash, r.bonds, r.stocks), (11_000.0, 24_000.0, 64_000.0));
    assert!(res.events.iter().any(|e| e.kind == EventKind::Buckets));
    // Until December only bucket 1 pays (12'000 suffice for 12 payments).
    let dec = &res.records[11];
    assert_eq!((dec.cash, dec.bonds, dec.stocks), (0.0, 24_000.0, 64_000.0));
    // January 2031: bucket 1 is refilled up to 12'000 from bucket 2, bucket 2 afterwards again from bucket 3 up to 24'000.
    let jan = &res.records[12];
    assert_eq!((jan.cash, jan.bonds, jan.stocks), (11_000.0, 24_000.0, 52_000.0));

    // Threshold 1 % not reached (return 0 %): nothing is sold from bucket 3, bucket 1 comes from bucket 2.
    let res = run(&plan(three_buckets(0.01)));
    let jan = &res.records[12];
    assert_eq!((jan.cash, jan.bonds, jan.stocks), (11_000.0, 12_000.0, 64_000.0));
}

#[test]
fn three_buckets_top_up_pot_1_before_the_payment_from_bonds_then_stocks() {
    // Threshold 1 %: the return criterion is not met, so C2 applies
    let mut v = three_buckets(0.01);
    v["withdrawal"]["buckets"]["cashYears"] = json!(0);
    v["withdrawal"]["buckets"]["bondYears"] = json!(0.25);
    let res = run(&plan(v));
    // Bucket 1 has no target and does not cover the payment of 1'000 (case C): it is refilled so that it covers it and
    // afterwards still holds (3 − 1) monthly needs, first from bucket 2 (3'000), then from the stocks.
    let jan = &res.records[0];
    assert_eq!((jan.cash, jan.bonds, jan.stocks), (2_000.0, 0.0, 97_000.0));
    let mar = &res.records[2];
    assert_eq!((mar.cash, mar.bonds, mar.stocks), (0.0, 0.0, 97_000.0));
    // April: bucket 1 does not cover the payment, bucket 2 is empty and is not refilled during the year → from bucket 3
    let apr = &res.records[3];
    assert_eq!((apr.cash, apr.bonds, apr.stocks), (2_000.0, 0.0, 94_000.0));
    let c: Vec<_> = res.bucket_refills.iter().filter(|r| r.trigger == "C2").collect();
    assert_eq!(c[0].month, YearMonth::new(2030, 1));
    assert_eq!((c[0].moves[0].from, c[0].moves[0].to), (2, 1), "zuerst aus Topf 2");
    assert!((c[0].moves[0].amount - 3_000.0).abs() < 1e-9, "{:?}", c[0].moves);
    assert_eq!((c[1].moves[0].from, c[1].moves[0].to), (3, 1), "dann aus Topf 3");
}

#[test]
fn monte_carlo_sources_run() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let ds = DataSet::load_csv(&dir.join("renditen-muster.csv")).unwrap();
    let mut v = base();
    v["wealth"]["stocks"] = json!(100_000);
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["series"] = json!("renditen-muster.csv");
    v["returns"]["bootstrap"] = json!({ "paths": 200, "block": 24, "seed": 7 });
    let p = plan(v.clone());
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::path_source(&p, &sim, Some(&ds)).unwrap();
    assert_eq!(paths.len(), 200);
    let (results, bands) = batch::run_with_bands(&sim, &paths);
    assert_eq!(results.len(), 200);
    assert_eq!(bands.p50.len(), sim.months());
    // Annual data: every month of a calendar year has the same return from the series.
    let p0 = paths.get(0);
    assert_eq!(p0.stocks[0], p0.stocks[11]);
    assert!(ds
        .stocks
        .iter()
        .any(|r| (market::monthly(*r) - p0.stocks[0]).abs() < 1e-15));

    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 2000, "mu": 0.07, "sigma": 0.15, "df": 6 });
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::path_source(&p, &sim, None).unwrap();
    let results = batch::run_all(&sim, &paths.iter_owned().collect::<Vec<_>>());
    let summary = batch::summarize(&results);
    assert_eq!(summary.runs, 2000);
    // Without withdrawals: median after 2 years close to 100'000 × e^(2 × (7 % − 1.125 %)).
    let median = summary.percentiles.iter().find(|x| x.percent == 50).unwrap().end_total;
    assert!((median / 100_000.0 - (2.0 * 0.05875f64).exp()).abs() < 0.03, "{median}");
}

#[test]
fn one_variation_step_can_be_selected_for_a_single_run() {
    let mut v = base();
    v["wealth"]["cash"] = json!(500_000);
    v["flows"]["spending"] = json!(3_000);
    v["scenarios"] = json!({ "sweeps": [ { "param": "flows.spendingFactor", "from": 0.5, "to": 1.5, "step": 0.5 } ] });
    let end_total = |plan: &Value| {
        let out = engine::api::run(plan, Path::new("."), None);
        assert!(out.error.is_none(), "{:?}", out.error);
        out.single.expect("Einzellauf").summary.end_total
    };
    // Selection 0.5: the same as a plan with half the spending, and without further scenarios
    let mut selected = v.clone();
    selected["scenarios"]["select"] = json!([ { "param": "flows.spendingFactor", "value": 0.5 } ]);
    let out = engine::api::run(&selected, Path::new("."), None);
    assert!(out.scenarios.is_none(), "die Auswahl ersetzt die Variationen");
    let mut halved = base();
    halved["wealth"]["cash"] = json!(500_000);
    halved["flows"]["spending"] = json!(1_500);
    assert!((end_total(&selected) - end_total(&halved)).abs() < 1e-6);
    // Without a selection the variations are preserved
    let out = engine::api::run(&v, Path::new("."), None);
    assert_eq!(out.scenarios.expect("Szenarien").results.len(), 3);
    // Unknown parameter: error
    let mut bad = v.clone();
    bad["scenarios"]["select"] = json!([ { "param": "gibt.es.nicht", "value": 1 } ]);
    assert!(engine::api::run(&bad, Path::new("."), None).error.is_some());
}

// ---------- Property events (purchase, sale, emergency sale, renewal) in one list ----------

use engine::sim::{PropertyEvent, PropertyEventKind};

fn of_kind(res: &engine::sim::SimResult, kind: PropertyEventKind) -> Vec<&PropertyEvent> {
    res.property_events.iter().filter(|e| e.kind == kind).collect()
}

#[test]
fn property_events_purchase_after_sale_carries_price_equity_and_mortgage() {
    let res = run(&after_sale_plan(100_000.0, 10_000.0, 500_000.0));
    let buy = of_kind(&res, PropertyEventKind::Purchase);
    assert_eq!(buy.len(), 1);
    let e = buy[0];
    assert_eq!((e.property.as_str(), e.month), ("Wohnung", YearMonth::new(2031, 2)));
    assert!((e.price.unwrap() - 500_000.0).abs() < 1e-6);
    assert!((e.mortgage.unwrap() - 300_000.0).abs() < 1e-6, "{e:?}");
    assert!(
        (e.equity.unwrap() - 200_000.0).abs() < 1e-6,
        "Eigenmittel = Preis − Hypothek"
    );
    // The sale of the house before it is also in the list, before it
    let sale = of_kind(&res, PropertyEventKind::Sale);
    assert_eq!(sale.len(), 1);
    assert_eq!(sale[0].property, "Haus");
    assert!(sale[0].month < e.month);
}

#[test]
fn property_events_planned_sale_has_proceeds_repaid_debt_and_net() {
    let mut v = base();
    v["wealth"]["cash"] = json!(100_000);
    v["realestate"] = json!({ "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "2030-06", "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 200_000, "start": "2029-01", "after": "renew",
            "periods": [ { "type": "fixed", "term": 10, "rate": 0.01 } ] } ] }
    } ] });
    let res = run(&plan(v));
    let sale = of_kind(&res, PropertyEventKind::Sale);
    assert_eq!(sale.len(), 1);
    let e = sale[0];
    assert_eq!(e.month, YearMonth::new(2030, 6));
    assert!((e.proceeds.unwrap() - 800_000.0).abs() < 1e-6);
    assert!((e.debt_repaid.unwrap() - 200_000.0).abs() < 1e-6);
    assert_eq!(e.gain_tax, Some(0.0));
    assert!(
        (e.net.unwrap() - 600_000.0).abs() < 1e-6,
        "Nettoerlös = Erlös − Hypothek − Steuer"
    );
    assert!(e.reason.is_none(), "kein Notverkauf");
    assert!(of_kind(&res, PropertyEventKind::EmergencySale).is_empty());
}

#[test]
fn property_events_renewal_with_lump_sum_shows_before_after_and_rate() {
    let res = renewal_case(6_000.0, 2_000_000.0, 0.0, 6.0);
    let ren = of_kind(&res, PropertyEventKind::Renewal);
    assert_eq!(ren.len(), 1);
    let e = ren[0];
    assert_eq!(e.month, YearMonth::new(2031, 1));
    assert_eq!(e.affordable, Some(false));
    assert!((e.debt_before.unwrap() - 400_000.0).abs() < 1e-6 && (e.debt_after.unwrap() - 320_000.0).abs() < 1e-6);
    assert!((e.lump_sum.unwrap() - 80_000.0).abs() < 1e-6);
    assert!((e.rate.unwrap() - 0.01).abs() < 1e-12, "Zinssatz der neuen Periode");
    assert!(e.reason.is_none());
    // Affordable: renewal without a lump sum
    let ok = renewal_case(20_000.0, 2_000_000.0, 0.0, 6.0);
    let e = of_kind(&ok, PropertyEventKind::Renewal)[0].clone();
    assert_eq!((e.affordable, e.lump_sum), (Some(true), None));
}

#[test]
fn property_events_emergency_sale_from_renewal_names_the_reason() {
    // Wealth is not enough for the lump sum → renewal not carried out, emergency sale in the following month
    let res = renewal_case(2_000.0, 10_000.0, 0.0, 6.0);
    let ren = of_kind(&res, PropertyEventKind::Renewal);
    assert_eq!(ren[0].reason.as_deref(), Some("Wealth < 0"));
    let sale = of_kind(&res, PropertyEventKind::EmergencySale);
    assert_eq!(sale.len(), 1);
    assert_eq!(sale[0].month, YearMonth::new(2031, 2));
    assert_eq!(sale[0].reason.as_deref(), Some("Wealth < 0"));
    assert!((sale[0].proceeds.unwrap() - 800_000.0).abs() < 1e-6);
    assert!((sale[0].debt_repaid.unwrap() - 400_000.0).abs() < 1e-6);
    assert_eq!(res.summary.emergency_sales, 1);
    // Emergency sale criterion after the payment
    let d = renewal_case(6_000.0, 120_000.0, 8_000.0, 6.0);
    let sale = of_kind(&d, PropertyEventKind::EmergencySale);
    assert_eq!(sale[0].reason.as_deref(), Some("Emergency sale criterion"));
}

#[test]
fn property_events_regular_emergency_sale_uses_the_criterion() {
    let mut v = base();
    v["wealth"]["cash"] = json!(20_000);
    v["flows"]["spending"] = json!(4_000);
    v["realestate"] = json!({ "emergencyMonths": 3, "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0
    } ] });
    let res = run(&plan(v));
    let sale = of_kind(&res, PropertyEventKind::EmergencySale);
    assert_eq!(sale.len(), 1, "{:?}", res.property_events);
    assert_eq!(sale[0].reason.as_deref(), Some("Emergency sale criterion"));
    assert!(sale[0].net.unwrap() > 0.0);
    assert_eq!(res.summary.emergency_sales, 1);
}

#[test]
fn property_events_are_chronological_and_empty_without_properties() {
    let res = run(&after_sale_plan(100_000.0, 10_000.0, 500_000.0));
    assert!(res.property_events.windows(2).all(|w| w[0].month <= w[1].month));
    assert!(run(&plan(base())).property_events.is_empty());
}

#[test]
fn property_events_planned_purchase_at_a_fixed_date() {
    let mut v = base();
    v["wealth"]["cash"] = json!(500_000);
    v["realestate"] = json!({ "properties": [ {
        "name": "Neubau", "status": "planned", "buy": "2030-06", "price": 500_000, "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 200_000, "periods": [ { "type": "fixed", "term": 10, "rate": 0.01 } ] } ] }
    } ] });
    let res = run(&plan(v));
    let buy = of_kind(&res, PropertyEventKind::Purchase);
    assert_eq!(buy.len(), 1, "{:?}", res.property_events);
    assert_eq!(buy[0].month, YearMonth::new(2030, 6));
    assert!((buy[0].equity.unwrap() - 300_000.0).abs() < 1e-6 && (buy[0].mortgage.unwrap() - 200_000.0).abs() < 1e-6);
}

#[test]
fn summary_reports_total_and_liquid_wealth_at_simulation_start() {
    // Holdings 100 + 200 + 300 = 600 liquid, no spending, no returns: start and end balance are equal.
    let mut v = base();
    v["wealth"]["stocks"] = json!(100_000);
    v["wealth"]["bonds"] = json!(200_000);
    v["wealth"]["cash"] = json!(300_000);
    let res = run(&plan(v));
    assert!((res.summary.start_liquid - 600_000.0).abs() < 1e-6);
    assert!((res.summary.start_total - 600_000.0).abs() < 1e-6);
    assert!((res.summary.end_total - res.summary.start_total).abs() < 1e-6);
}

/// #2: If the purchase «nach Verkauf» is not financeable, it is dropped at once and the follow-up rules apply:
/// the rent «nach Verkauf» of the flat starts in the month after (instead of never).
#[test]
fn unaffordable_purchase_after_sale_triggers_the_follow_up_rent() {
    let mut v = base();
    v["wealth"]["cash"] = json!(100_000);
    v["flows"]["income"] = json!(0);
    v["realestate"] = json!({ "properties": [
        { "id": "haus", "name": "Haus", "status": "owned", "price": 300_000, "worth": 400_000, "sell": "2030-06" },
        { "id": "whg", "name": "Wohnung", "status": "planned", "buy": "afterSale:haus", "price": 900_000,
          "mortgage": { "amountMode": "maxLoan", "tranches": [ { "amount": 1, "periods": [ { "term": 10, "rate": 0.01 } ] } ] } },
        { "id": "miete", "name": "Mietwohnung", "status": "rent", "from": "afterSale:whg", "rent": 2_000 }
    ] });
    let res = run(&plan(v));
    let sale = res
        .events
        .iter()
        .find(|e| e.text.starts_with("Sale Haus"))
        .expect("Verkauf");
    assert_eq!(sale.month, YearMonth::new(2030, 6));
    let failed = res
        .events
        .iter()
        .find(|e| e.text.contains("not possible: equity"))
        .expect("Kauf nicht möglich");
    assert_eq!(failed.month, YearMonth::new(2030, 7));
    let dropped = res
        .events
        .iter()
        .find(|e| e.text.contains("dropped"))
        .expect("Folgeregel");
    assert_eq!(dropped.month, YearMonth::new(2030, 7));
    // No property bought, rent from August 2030 (index 7), no housing costs before
    assert!(res.events.iter().all(|e| !e.text.starts_with("Purchase Wohnung:")));
    assert!(res.records[6].housing.abs() < 1e-9, "Juli: noch keine Miete");
    assert!(
        (res.records[7].housing - 2_000.0).abs() < 1e-6,
        "August: Miete {}",
        res.records[7].housing
    );
}

/// #124/AK3: The one-off repayment at renewal comes from liquid wealth (case 2 above: 80'000).
#[test]
fn issue_124_ak3_lump_sum_is_paid_from_liquid_wealth() {
    let r = renewal_case(6_000.0, 2_000_000.0, 0.0, 6.0);
    let at = |y: i32, m: u32| {
        r.records
            .iter()
            .find(|x| x.month == YearMonth::new(y, m))
            .expect("Monat")
    };
    let (dec, jan) = (at(2030, 12), at(2031, 1));
    // Income 6'000 of the month, lump sum 80'000; the interest only accrues in January.
    assert!(
        (dec.liquid - jan.liquid - 74_000.0).abs() < 1e-6,
        "{} → {}",
        dec.liquid,
        jan.liquid
    );
    assert!(
        (dec.mortgage - jan.mortgage - 80_000.0).abs() < 1e-6,
        "{} → {}",
        dec.mortgage,
        jan.mortgage
    );
}

/// #124/AK3: After retirement «Standard» keeps the reserve for the ongoing costs; mortgage interest
/// belongs to them (mortgage 400'000 at 1.2 %: 400 per month, reserve 12 months).
#[test]
fn issue_124_ak3_mortgage_interest_counts_for_the_reserve() {
    let cash = |rate: f64| {
        let mut v = base();
        v["milestones"][1]["date"] = json!("2030-01");
        v["flows"]["spending"] = json!(1000);
        v["wealth"]["cash"] = json!(1_000_000);
        v["wealth"]["reserveMonths"] = json!(12);
        v["realestate"] = json!({ "properties": [ {
            "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
            "mortgage": { "tranches": [ { "amount": 400_000, "start": "2030-01",
                "periods": [ { "type": "fixed", "term": 10, "rate": rate } ] } ] }
        } ] });
        let r = run(&plan(v));
        r.records
            .iter()
            .find(|x| x.month == YearMonth::new(2030, 5))
            .expect("Mai")
            .cash
    };
    assert!((cash(0.0) - 12.0 * 1000.0).abs() < 1e-6, "ohne Zins: {}", cash(0.0));
    assert!((cash(0.012) - 12.0 * 1400.0).abs() < 1e-6, "mit Zins: {}", cash(0.012));
}

/// #126/AK1, #129/AK3: Every percentile of the summary carries the real end wealth of its run. Parametric with
/// random inflation: the runs have different inflation; real and nominal differ differently per run.
#[test]
fn issue_126_ak1_percentiles_carry_the_real_end_wealth_of_their_run() {
    let mut v = engine::api::example_plan();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 60, "mu": 0.06, "sigma": 0.15, "df": 0, "seed": 3 });
    v["scenarios"]["sweeps"] = json!([]);
    let out = engine::api::run(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let pcts = out.batch.expect("mehrere Läufe").summary.percentiles;
    assert_eq!(pcts.len(), 10);
    // Recompute every percentile run via its path number: same run, same nominal and real end value.
    let picks: Vec<(u32, usize)> = pcts.iter().map(|p| (p.percent, p.index)).collect();
    let re = engine::api::rerun_paths(&v, &picks, Path::new("/"), None);
    assert!(re.error.is_none(), "{:?}", re.error);
    assert_eq!(re.details.len(), pcts.len());
    for (p, (q, run)) in pcts.iter().zip(&re.details) {
        assert_eq!(p.percent, *q);
        assert_eq!(p.end_total.to_bits(), run.summary.end_total.to_bits(), "nominal {q}");
        assert_eq!(
            p.end_total_real.expect("real").to_bits(),
            run.summary.end_total_real.to_bits(),
            "real {q}"
        );
    }
    // Random inflation: the ratio of real to nominal is not the same in all runs.
    let ratios: Vec<f64> = pcts
        .iter()
        .filter(|p| p.end_total != 0.0)
        .map(|p| p.end_total_real.unwrap() / p.end_total)
        .collect();
    assert!(ratios.iter().any(|r| (r - ratios[0]).abs() > 1e-6), "{ratios:?}");
}

/// #130/AK1: The percentiles of the scenarios carry no real end wealth, the summary of the run still does.
#[test]
fn issue_130_ak1_scenario_percentiles_without_real_end_wealth() {
    let mut v = engine::api::example_plan();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 20, "mu": 0.06, "sigma": 0.15, "df": 0, "seed": 3 });
    let out = engine::api::run(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let json = serde_json::to_value(&out).unwrap();
    let sc = json["scenarios"]["results"].as_array().expect("Szenarien");
    assert!(!sc.is_empty());
    for p in sc.iter().flat_map(|r| r["percentiles"].as_array().unwrap()) {
        assert!(p.get("endTotalReal").is_none(), "{p}");
        assert!(p["endTotal"].is_f64());
    }
    let batch = json["batch"]["summary"]["percentiles"].as_array().expect("Lauf");
    assert_eq!(batch.len(), 10);
    assert!(batch.iter().all(|p| p["endTotalReal"].is_f64()));
}

/// #265/AK1, AK3: Parametric with random inflation: the summary carries both orders, the detail runs of each order are
/// the runs of its percentiles (same label, same end wealth), and the real order differs from the nominal one.
#[test]
fn issue_265_ak3_detail_runs_follow_each_order() {
    let mut v = engine::api::example_plan();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 60, "mu": 0.06, "sigma": 0.15, "df": 0, "seed": 3 });
    v["scenarios"]["sweeps"] = json!([]);
    let out = engine::api::run(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let b = out.batch.expect("mehrere Läufe");
    let s = &b.summary;
    let ascending = |v: Vec<f64>| v.windows(2).all(|w| w[0] <= w[1]);
    assert!(ascending(s.percentiles.iter().map(|p| p.end_total).collect()));
    assert!(ascending(
        s.percentiles_real.iter().map(|p| p.end_total_real.unwrap()).collect()
    ));
    assert_ne!(
        s.percentiles.iter().map(|p| p.index).collect::<Vec<_>>(),
        s.percentiles_real.iter().map(|p| p.index).collect::<Vec<_>>(),
        "Ordnungen gleich: Testfall ohne Aussage"
    );
    for (list, details) in [(&s.percentiles, &b.details), (&s.percentiles_real, &b.details_real)] {
        assert_eq!(details.len(), engine::api::DETAIL_PERCENTS.len());
        for (q, run) in details {
            let p = list.iter().find(|p| p.percent == *q).expect("Perzentil");
            assert_eq!(p.label, run.label, "{q}");
            assert_eq!(p.end_total.to_bits(), run.summary.end_total.to_bits(), "{q}");
            assert_eq!(
                p.end_total_real.unwrap().to_bits(),
                run.summary.end_total_real.to_bits(),
                "{q}"
            );
            assert_eq!(p.emergency_sales, run.summary.emergency_sales, "{q}");
        }
    }
}

/// #265/AK4: The real median of the scenarios is the run at the 50th percentile of the real order; the scenarios
/// carry the real order with its runs (path numbers for the single runs of a step).
#[test]
fn issue_265_ak4_scenarios_follow_the_real_order() {
    let mut v = engine::api::example_plan();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 21, "mu": 0.06, "sigma": 0.15, "df": 0, "seed": 3 });
    let out = engine::api::run(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let sc = out.scenarios.expect("Szenarien");
    assert!(!sc.results.is_empty());
    for r in &sc.results {
        assert_eq!(r.percentiles_real.len(), 10);
        let median = r.percentiles_real.iter().find(|p| p.percent == 50).unwrap();
        assert_eq!(r.p50_real.to_bits(), median.end_total_real.unwrap().to_bits());
        let real: Vec<f64> = r.percentiles_real.iter().map(|p| p.end_total_real.unwrap()).collect();
        assert!(real.windows(2).all(|w| w[0] <= w[1]), "{real:?}");
    }
}
