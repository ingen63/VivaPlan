//! Verhaltenstests des Rechenkerns an kleinen, von Hand nachrechenbaren Planungen.

use std::path::Path;

use engine::batch;
use engine::market::{self, DataSet};
use engine::model::Plan;
use engine::sim::{EventKind, Simulator};
use engine::time::YearMonth;
use engine::validate::{self, Severity};
use serde_json::{json, Value};

/// Minimale Planung: 2 Jahre ab Januar 2030, keine Renditen, keine Inflation, keine Steuern.
fn base() -> Value {
    crate::any_age(json!({
        "project": { "name": "Test" },
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
    }))
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
    assert!((r[11].income - 2_000.0).abs() < 1e-9, "AHV bis Dezember unverändert");
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
    // Juni bis Dezember: 7 Monatsrenten
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
    // Periode 01.2029 – 01.2031, danach Rückzahlung der Restschuld
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
    // Zins wird am Quartalsende bezahlt: Januar nichts, März drei Monate à 100'000 × 1.2 % / 12
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
    // AHV-Bezug 2035-01: bis dahin indirekt (FEA-36), Auszahlung 2031-06 erlaubt (FEA-37).
    p.milestones.push(
        serde_json::from_value(json!({ "id": "ahv", "name": "AHV", "kind": "date", "date": "2035-01" })).unwrap(),
    );
    let res = run(&p);
    let r = &res.records;
    assert!((r[11].pillar3a - 16_000.0).abs() < 1e-6, "4 Quartale à 1'500");
    assert!((r[11].property_equity - 400_000.0).abs() < 1e-6, "Schuld bleibt gleich");
    // Auszahlung Juni 2031 nach der Juni-Einzahlung: 10'000 + 6 × 1'500 = 19'000, zahlt T2 zurück
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
    // Verkauf +600'000, Kauf −(500'000 − 200'000)
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
    // Erster Lauf: Jahre 1956 und 1957
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
    assert!(issues.iter().any(|i| i.message.contains("Kapitalbezug")));
    assert!(issues.iter().any(|i| i.message.contains("nirgends")));
    assert!(Simulator::new(&p).is_err());
}

#[test]
fn example_plan_runs_and_roundtrips() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/beispiel.vplan");
    let p = Plan::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let res = run(&p);
    assert!(res.summary.depleted.is_none());
    let again = Plan::from_json(&p.to_json()).unwrap();
    assert_eq!(run(&again).summary.end_total, res.summary.end_total);
}

/// Haus, das im Januar 2031 verkauft wird; danach geplante Wohnung «nach Verkauf» und Miete.
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
    // Tragbar sind nur aus dem Einkommen (1/3 × Jahreseinkommen − Nebenkosten) / 5 %; das Vermögen zählt nicht.
    let res = run(&after_sale_plan(300_000.0, 2_000.0, 500_000.0));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    let debt = 500_000.0 - res.records[13].property_equity;
    // (1/3 × 24'000 − 6'000) / 5 % = 40'000
    assert!((debt - 40_000.0).abs() < 1.0, "gekürzte Hypothek {debt}: {}", buy.text);
}

/// Hypothek nach dem Kauf (Kaufpreis minus Eigenkapital der Wohnung im Februar 2031).
fn debt_after_buy(cash: f64) -> (f64, String) {
    let res = run(&after_sale_plan(cash, 2_000.0, 500_000.0));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    (500_000.0 - res.records[13].property_equity, buy.text.clone())
}

#[test]
fn affordability_counts_only_income_and_reports_the_lump_sum_repayment() {
    let (low, text) = debt_after_buy(300_000.0);
    let (high, _) = debt_after_buy(3_000_000.0);
    assert!(
        (low - high).abs() < 1e-6,
        "Vermögen darf die Tragbarkeit nicht verändern: {low} gegen {high}"
    );
    assert!(text.contains("Einmalrückzahlung") && text.contains("CHF"), "{text}");
    assert!(!text.contains("Amortisation"), "{text}");
}

#[test]
fn no_lump_sum_repayment_when_the_mortgage_is_sustainable() {
    let res = run(&after_sale_plan(100_000.0, 10_000.0, 500_000.0));
    let buy = res.events.iter().find(|e| e.kind == EventKind::Purchase).expect("Kauf");
    assert!(!buy.text.contains("Einmalrückzahlung"), "{}", buy.text);
}

/// Haus mit Hypothek 400'000 (Periode 1 bis Ende 2030, Erneuerung im Januar 2031) und gegebenem Einkommen und Vermögen.
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

/// Vier Fälle der Tragbarkeitsprüfung bei der Erneuerung (Hypothek 400'000, Haus 800'000, Erneuerung Januar 2031).
/// `emergency_months`: Schwelle des Notverkaufskriteriums; `spending`: Ausgaben pro Monat.
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
    // 1. Tragbar: Einkommen 20'000/Monat → normale Erneuerung
    let a = renewal_case(20_000.0, 2_000_000.0, 0.0, 6.0);
    let r = jan_2031(&a);
    assert!(r.affordable && r.lump_sum == 0.0 && r.emergency_sale.is_none());
    assert!((r.debt_after - 400_000.0).abs() < 1e-6);
    assert_eq!((a.summary.lump_sums, a.summary.affordability_sales), (0, 0));
    assert!(a.events.iter().any(|e| e.text.starts_with("Neue Hypothekarperiode")));

    // 2. Nicht tragbar, Einmalzahlung möglich: Einkommen 6'000 → tragbar 320'000, Zahlung 80'000, Erneuerung mit 320'000
    let b = renewal_case(6_000.0, 2_000_000.0, 0.0, 6.0);
    let r = jan_2031(&b);
    assert!(!r.affordable && r.emergency_sale.is_none());
    assert!((r.lump_sum - 80_000.0).abs() < 1e-6 && (r.debt_after - 320_000.0).abs() < 1e-6);
    assert_eq!((b.summary.lump_sums, b.summary.affordability_sales), (1, 0));
    assert!(b
        .events
        .iter()
        .any(|e| e.text.starts_with("Neue Hypothekarperiode") && e.text.contains("320000")));
    assert!(!b.events.iter().any(|e| e.kind == EventKind::EmergencySale));

    // 3. Einmalzahlung würde das Vermögen unter 0 bringen: Einkommen 2'000 → Zahlung 400'000, Vermögen 10'000
    let c = renewal_case(2_000.0, 10_000.0, 0.0, 6.0);
    let r = jan_2031(&c);
    assert!(!r.affordable && r.lump_sum == 0.0);
    assert_eq!(r.emergency_sale.as_deref(), Some("Vermögen < 0"));
    assert_eq!((c.summary.lump_sums, c.summary.affordability_sales), (0, 1));
    assert!(
        !c.events
            .iter()
            .any(|e| e.month == YearMonth::new(2031, 1) && e.text.starts_with("Neue Hypothekarperiode")),
        "nicht erneuert"
    );
    assert_eq!(
        c.events
            .iter()
            .find(|e| e.kind == EventKind::EmergencySale)
            .expect("Notverkauf")
            .month,
        YearMonth::new(2031, 2)
    );

    // 4. Zahlung möglich (Vermögen danach >= 0), aber danach löst das Notverkaufskriterium aus:
    //    Einkommen 6'000, Ausgaben 8'000 → Vermögen im Januar 2031 rund 92'000, Zahlung 80'000 → Rest 12'000; 6 × Fehlbetrag (8'000 + Zins − 6'000) ≈ 13'600 > Rest
    let d = renewal_case(6_000.0, 120_000.0, 8_000.0, 6.0);
    let r = jan_2031(&d);
    assert!(!r.affordable && r.lump_sum == 0.0);
    assert_eq!(r.emergency_sale.as_deref(), Some("Notverkaufskriterium"), "{r:?}");
    assert_eq!((d.summary.lump_sums, d.summary.affordability_sales), (0, 1));
    assert!(d.events.iter().any(|e| e.text.contains("Notverkaufskriterium")));
    // Ohne Notverkaufsschwelle wird dieselbe Zahlung geleistet
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
    // Einkommen 2'000/Monat: tragbar sind (1/3 × 24'000 − 1 % × 800'000 Nebenkosten) / 5 % = 0
    // → die ganze Hypothek (400'000) wird im Januar 2031 zurückbezahlt.
    let res = run(&renewal_plan(2_000.0, 2_000_000.0));
    let ev = res
        .events
        .iter()
        .find(|e| e.text.contains("Einmalrückzahlung"))
        .expect("Einmalrückzahlung");
    assert_eq!(ev.month, YearMonth::new(2031, 1), "{}", ev.text);
    assert!(ev.text.contains("CHF"), "{}", ev.text);
    assert!(debt_in(&res, 2030, 12) > 399_000.0);
    assert!(
        debt_in(&res, 2031, 1) < 1.0,
        "Hypothek nach Erneuerung {}",
        debt_in(&res, 2031, 1)
    );
    // Das Vermögen sinkt um die Einmalrückzahlung.
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
    // Einkommen 6'000/Monat: (1/3 × 72'000 − 8'000) / 5 % = 320'000 tragbar → 80'000 Einmalrückzahlung.
    let res = run(&renewal_plan(6_000.0, 2_000_000.0));
    let ev = res
        .events
        .iter()
        .find(|e| e.text.contains("Einmalrückzahlung"))
        .expect("Einmalrückzahlung");
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
    assert!(!res.events.iter().any(|e| e.text.contains("Einmalrückzahlung")));
    assert!((debt_in(&res, 2031, 1) - 400_000.0).abs() < 1.0);
}

#[test]
fn renewal_triggers_an_emergency_sale_when_the_wealth_cannot_pay_the_lump_sum() {
    let res = run(&renewal_plan(2_000.0, 10_000.0));
    let ev = res
        .events
        .iter()
        .find(|e| e.text.contains("nicht mehr tragbar"))
        .expect("Hinweis");
    assert_eq!(ev.month, YearMonth::new(2031, 1));
    assert!(ev.text.contains("Notverkauf"), "{}", ev.text);
    assert!(!res.events.iter().any(|e| e.text.starts_with("Einmalrückzahlung")));
    assert!(
        debt_in(&res, 2031, 1) > 399_000.0,
        "Hypothek bleibt bis zum Verkauf unverändert"
    );
    // Der Notverkauf folgt im Folgemonat: Hypothek zurückgezahlt, Erlös im Vermögen.
    let sale = res
        .events
        .iter()
        .find(|e| e.kind == EventKind::EmergencySale)
        .expect("Notverkauf");
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
    // Ferienhaus: Tranche 1 ist amortisiert, Tranche 2 (rund 18'000) ist tragbar
    let ferien = res
        .renewals
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1) && r.property == "Ferienhaus")
        .unwrap();
    assert!(ferien.affordable && ferien.lump_sum == 0.0, "{ferien:?}");
    assert!(ferien.debt_before < 20_000.0);
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
        .expect("Notverkauf");
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
    // Ferienwohnung 250'000 + Wohnung 500'000 − 300'000
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
    // Zins Juni: 300'000 × 1 % / 12 + 100'000 × 1.5 % / 12 + Nebenkosten 400
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
    // Pensionierung im Januar 2030: Auslöser A, alles in Topf 3, dann 12'000 / 24'000 / 64'000 (vor der Zahlung von 1'000).
    let first = &res.bucket_refills[0];
    assert_eq!(first.trigger, "A");
    assert_eq!(first.after, [12_000.0, 24_000.0, 64_000.0]);
    let r = &res.records[0];
    assert_eq!((r.cash, r.bonds, r.stocks), (11_000.0, 24_000.0, 64_000.0));
    assert!(res.events.iter().any(|e| e.kind == EventKind::Buckets));
    // Bis Dezember zahlt nur Topf 1 (12'000 reichen für 12 Zahlungen).
    let dec = &res.records[11];
    assert_eq!((dec.cash, dec.bonds, dec.stocks), (0.0, 24_000.0, 64_000.0));
    // Januar 2031: Topf 1 wird bis 12'000 aus Topf 2 aufgefüllt, Topf 2 danach wieder aus Topf 3 auf 24'000.
    let jan = &res.records[12];
    assert_eq!((jan.cash, jan.bonds, jan.stocks), (11_000.0, 24_000.0, 52_000.0));

    // Schwelle 1 % nicht erreicht (Rendite 0 %): Es wird nichts aus Topf 3 verkauft, Topf 1 kommt aus Topf 2.
    let res = run(&plan(three_buckets(0.01)));
    let jan = &res.records[12];
    assert_eq!((jan.cash, jan.bonds, jan.stocks), (11_000.0, 12_000.0, 64_000.0));
}

#[test]
fn three_buckets_top_up_pot_1_before_the_payment_from_bonds_then_stocks() {
    // Schwelle 1 %: Das Renditekriterium ist nicht erfüllt, also gilt C2
    let mut v = three_buckets(0.01);
    v["withdrawal"]["buckets"]["cashYears"] = json!(0);
    v["withdrawal"]["buckets"]["bondYears"] = json!(0.25);
    let res = run(&plan(v));
    // Topf 1 hat kein Ziel und deckt die Zahlung von 1'000 nicht (Fall C): Er wird so aufgefüllt, dass er sie deckt und
    // danach noch (3 − 1) Monatsbedarfe enthält, zuerst aus Topf 2 (3'000), danach aus den Aktien.
    let jan = &res.records[0];
    assert_eq!((jan.cash, jan.bonds, jan.stocks), (2_000.0, 0.0, 97_000.0));
    let mar = &res.records[2];
    assert_eq!((mar.cash, mar.bonds, mar.stocks), (0.0, 0.0, 97_000.0));
    // April: Topf 1 deckt die Zahlung nicht, Topf 2 ist leer und wird unterjährig nicht aufgefüllt → aus Topf 3
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
    // Jahresdaten: jeder Monat eines Kalenderjahres hat dieselbe Rendite aus der Reihe.
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
    // Ohne Entnahmen: Median nach 2 Jahren nahe 100'000 × e^(2 × (7 % − 1.125 %)).
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
    // Auswahl 0.5: gleich wie eine Planung mit halben Ausgaben, und ohne weitere Szenarien
    let mut selected = v.clone();
    selected["scenarios"]["select"] = json!([ { "param": "flows.spendingFactor", "value": 0.5 } ]);
    let out = engine::api::run(&selected, Path::new("."), None);
    assert!(out.scenarios.is_none(), "die Auswahl ersetzt die Variationen");
    let mut halved = base();
    halved["wealth"]["cash"] = json!(500_000);
    halved["flows"]["spending"] = json!(1_500);
    assert!((end_total(&selected) - end_total(&halved)).abs() < 1e-6);
    // Ohne Auswahl bleiben die Variationen erhalten
    let out = engine::api::run(&v, Path::new("."), None);
    assert_eq!(out.scenarios.expect("Szenarien").results.len(), 3);
    // Unbekannter Parameter: Fehler
    let mut bad = v.clone();
    bad["scenarios"]["select"] = json!([ { "param": "gibt.es.nicht", "value": 1 } ]);
    assert!(engine::api::run(&bad, Path::new("."), None).error.is_some());
}

// ---------- Immobilienereignisse (Kauf, Verkauf, Notverkauf, Erneuerung) in einer Liste ----------

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
    // Der Verkauf des Hauses davor steht ebenfalls in der Liste, davor
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
    // Tragbar: Erneuerung ohne Einmalzahlung
    let ok = renewal_case(20_000.0, 2_000_000.0, 0.0, 6.0);
    let e = of_kind(&ok, PropertyEventKind::Renewal)[0].clone();
    assert_eq!((e.affordable, e.lump_sum), (Some(true), None));
}

#[test]
fn property_events_emergency_sale_from_renewal_names_the_reason() {
    // Vermögen reicht nicht für die Einmalzahlung → Erneuerung nicht erfolgt, Notverkauf im Folgemonat
    let res = renewal_case(2_000.0, 10_000.0, 0.0, 6.0);
    let ren = of_kind(&res, PropertyEventKind::Renewal);
    assert_eq!(ren[0].reason.as_deref(), Some("Vermögen < 0"));
    let sale = of_kind(&res, PropertyEventKind::EmergencySale);
    assert_eq!(sale.len(), 1);
    assert_eq!(sale[0].month, YearMonth::new(2031, 2));
    assert_eq!(sale[0].reason.as_deref(), Some("Vermögen < 0"));
    assert!((sale[0].proceeds.unwrap() - 800_000.0).abs() < 1e-6);
    assert!((sale[0].debt_repaid.unwrap() - 400_000.0).abs() < 1e-6);
    assert_eq!(res.summary.emergency_sales, 1);
    // Notverkaufskriterium nach der Zahlung
    let d = renewal_case(6_000.0, 120_000.0, 8_000.0, 6.0);
    let sale = of_kind(&d, PropertyEventKind::EmergencySale);
    assert_eq!(sale[0].reason.as_deref(), Some("Notverkaufskriterium"));
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
    assert_eq!(sale[0].reason.as_deref(), Some("Notverkaufskriterium"));
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
    // Bestände 100 + 200 + 300 = 600 liquid, keine Ausgaben, keine Renditen: Start- und Endstand sind gleich.
    let mut v = base();
    v["wealth"]["stocks"] = json!(100_000);
    v["wealth"]["bonds"] = json!(200_000);
    v["wealth"]["cash"] = json!(300_000);
    let res = run(&plan(v));
    assert!((res.summary.start_liquid - 600_000.0).abs() < 1e-6);
    assert!((res.summary.start_total - 600_000.0).abs() < 1e-6);
    assert!((res.summary.end_total - res.summary.start_total).abs() < 1e-6);
}

/// ISS-2: Ist der Kauf «nach Verkauf» nicht finanzierbar, entfällt er sofort und die Folgeregeln greifen:
/// Die Miete «nach Verkauf» der Wohnung beginnt im Monat danach (statt nie).
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
        .find(|e| e.text.starts_with("Verkauf Haus"))
        .expect("Verkauf");
    assert_eq!(sale.month, YearMonth::new(2030, 6));
    let failed = res
        .events
        .iter()
        .find(|e| e.text.contains("nicht möglich"))
        .expect("Kauf nicht möglich");
    assert_eq!(failed.month, YearMonth::new(2030, 7));
    let dropped = res
        .events
        .iter()
        .find(|e| e.text.contains("entfällt"))
        .expect("Folgeregel");
    assert_eq!(dropped.month, YearMonth::new(2030, 7));
    // Keine Immobilie gekauft, Miete ab August 2030 (Index 7), davor keine Wohnkosten
    assert!(res.events.iter().all(|e| !e.text.starts_with("Kauf Wohnung:")));
    assert!(res.records[6].housing.abs() < 1e-9, "Juli: noch keine Miete");
    assert!(
        (res.records[7].housing - 2_000.0).abs() < 1e-6,
        "August: Miete {}",
        res.records[7].housing
    );
}

/// FEA-97/AK3: Die Einmalrückzahlung bei der Erneuerung kommt aus dem liquiden Vermögen (Fall 2 oben: 80'000).
#[test]
fn fea97_ak3_lump_sum_is_paid_from_liquid_wealth() {
    let r = renewal_case(6_000.0, 2_000_000.0, 0.0, 6.0);
    let at = |y: i32, m: u32| {
        r.records
            .iter()
            .find(|x| x.month == YearMonth::new(y, m))
            .expect("Monat")
    };
    let (dec, jan) = (at(2030, 12), at(2031, 1));
    // Einkommen 6'000 des Monats, Einmalzahlung 80'000; die Zinsen laufen im Januar nur auf.
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

/// FEA-97/AK3: Nach der Pensionierung hält «Standard» die Reserve für die laufenden Kosten; die Hypothekarzinsen
/// gehören dazu (Hypothek 400'000 zu 1.2 %: 400 pro Monat, Reserve 12 Monate).
#[test]
fn fea97_ak3_mortgage_interest_counts_for_the_reserve() {
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

/// ISS-23/AK1: Jedes Perzentil der Zusammenfassung trägt das reale Endvermögen seines Laufs. Parametrisch mit zufälliger
/// Inflation: Die Läufe haben verschiedene Inflation, real und nominal unterscheiden sich je Lauf anders.
#[test]
fn iss_23_ak1_percentiles_carry_the_real_end_wealth_of_their_run() {
    let mut v = engine::api::example_plan();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 60, "mu": 0.06, "sigma": 0.15, "df": 0, "jumpLambda": 0, "jumpMu": 0, "jumpSigma": 0, "seed": 3 });
    v["scenarios"]["sweeps"] = json!([]);
    let out = engine::api::run(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let b = out.batch.expect("mehrere Läufe");
    let json = serde_json::to_value(&b.summary).unwrap();
    let pcts = json["percentiles"].as_array().unwrap();
    assert!(
        pcts.len() > 6,
        "alle Perzentile, nicht nur die der Detailläufe: {}",
        pcts.len()
    );
    // Mit den Detailläufen (0, 1, 5, 50, 95, 99) vergleichbar: gleicher Lauf, gleicher realer Endwert.
    let mut checked = 0;
    for (q, run) in &b.details {
        let p = pcts.iter().find(|p| p["percent"] == *q).expect("Perzentil");
        let real = p["endTotalReal"].as_f64().expect("endTotalReal");
        assert!(
            (real - run.summary.end_total_real).abs() < 1e-6,
            "{q}: {real} gegen {}",
            run.summary.end_total_real
        );
        assert!((p["endTotal"].as_f64().unwrap() - run.summary.end_total).abs() < 1e-6);
        checked += 1;
    }
    assert_eq!(checked, 6);
    // Auch die übrigen Perzentile haben einen Wert, und real ≠ nominal bei positiver Inflation.
    assert!(pcts.iter().all(|p| p["endTotalReal"].is_f64()));
    assert!(pcts.iter().any(|p| p["endTotalReal"] != p["endTotal"]));
}
