//! Zweite Person (Double Income, Ehepaar-Regeln), Entnahmestrategien und Auswertung über alle Läufe.

use std::path::Path;

use engine::batch;
use engine::market::DataSet;
use engine::model::Plan;
use engine::sim::{EventKind, SimResult, Simulator};
use engine::validate::{self, Severity};
use serde_json::{json, Value};

/// 10 Jahre ab Januar 2030, keine Renditen, keine Inflation, keine Steuern, Pensionierung am Start.
fn base() -> Value {
    json!({
        "project": { "name": "Test" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2039-12" }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "never" }
        },
        "flows": { "income": 0, "spending": 5000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 1, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn with_partner(mut v: Value) -> Value {
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01" },
        { "id": "B", "name": "B", "birth": "1967-01" }
    ]);
    v["partner"] = json!({
        "income": 0,
        "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
        "ahv": { "monthly": 0, "at": "never" }
    });
    v
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

// ---------------------------------------------------------------------------------------------
// Zweite Person
// ---------------------------------------------------------------------------------------------

#[test]
fn partner_income_adds_to_household() {
    let mut v = with_partner(base());
    v["flows"]["income"] = json!(3_000);
    v["partner"]["income"] = json!(2_000);
    v["milestones"][1]["date"] = json!("2040-01");
    let res = run(v);
    let r = &res.records[0];
    assert!((r.earned - 5_000.0).abs() < 1e-9);
    assert!((r.income - 5_000.0).abs() < 1e-9);
    // Einkommen deckt die Ausgaben: Vermögen bleibt.
    assert!((res.summary.end_total - 1_000_000.0).abs() < 1e-6);
}

#[test]
fn partner_without_second_person_is_an_error() {
    let mut v = base();
    v["partner"] = with_partner(base())["partner"].clone();
    let issues = validate::validate(&plan(v));
    assert!(issues
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "general"));
}

#[test]
fn more_than_two_persons_is_an_error() {
    let mut v = with_partner(base());
    v["household"]["persons"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "C", "name": "C", "birth": "1990-01" }));
    let issues = validate::validate(&plan(v));
    assert!(issues
        .iter()
        .any(|i| i.severity == Severity::Error && i.message.contains("zwei")));
}

#[test]
fn each_person_retires_on_own_date_and_lump_sums_are_added() {
    let mut v = with_partner(base());
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "retireB", "name": "Pensionierung B", "kind": "age", "person": "B", "years": 66 }));
    v["milestones"][1]["date"] = json!("2031-01");
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 0.0, "conversion": 0.06 });
    // B ist 2 Jahre jünger: 66 Jahre = Januar 2033
    v["partner"]["pk"] = json!({ "capital": 300_000, "at": "retireB", "lumpsum": 1.0, "conversion": 0.06 });
    v["taxes"]["capital"] = json!([[0, 0.1]]);
    let res = run(v);
    let retirements: Vec<_> = res.events.iter().filter(|e| e.kind == EventKind::Retirement).collect();
    assert_eq!(retirements.len(), 2);
    assert_eq!(retirements[0].month.to_string(), "01.2031");
    assert_eq!(retirements[1].month.to_string(), "01.2033");
    // Rente A: 500'000 × 6 % / 12 = 2'500; Kapital B netto 270'000
    assert!((res.summary.pension_fund_monthly - 2_500.0).abs() < 1e-6);
    assert!((res.summary.lumpsum_net - 270_000.0).abs() < 1e-6);
}

#[test]
fn couple_ahv_is_capped_but_single_pension_is_not() {
    let mut v = with_partner(base());
    v["pension"]["ahv"] = json!({ "monthly": 2_520, "at": "start" });
    v["partner"]["ahv"] = json!({ "monthly": 2_520, "at": "start" });
    v["household"]["ahvCoupleCap"] = json!(3_780);
    let res = run(v.clone());
    assert!((res.records[0].ahv - 3_780.0).abs() < 1e-9, "{}", res.records[0].ahv);

    // Nur eine Rente fällig: keine Kürzung.
    v["partner"]["ahv"]["at"] = json!("never");
    let res = run(v.clone());
    assert!((res.records[0].ahv - 2_520.0).abs() < 1e-9);

    // Nicht verheiratet: keine Plafonierung.
    v["partner"]["ahv"]["at"] = json!("start");
    v["household"]["married"] = json!(false);
    let res = run(v);
    assert!((res.records[0].ahv - 5_040.0).abs() < 1e-9);
}

/// #145: Ehepaar mit Plafond 3'780 und Teuerung 3 % (fix); AHV im Monat `k` (0 = Januar 2030).
fn capped_ahv(a_indexed: bool, b_indexed: bool, b_at: &str) -> SimResult {
    let mut v = with_partner(base());
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "b_ahv", "name": "AHV B", "kind": "date", "date": "2032-01" }));
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    v["pension"]["ahv"] = json!({ "monthly": 2_520, "at": "start", "indexed": a_indexed });
    v["partner"]["ahv"] = json!({ "monthly": 2_520, "at": b_at, "indexed": b_indexed });
    v["household"]["ahvCoupleCap"] = json!(3_780);
    run(v)
}

fn ahv_in(res: &SimResult, k: usize) -> f64 {
    res.records[k].ahv
}

// #145/AK1, AK2: Beide Renten mit Teuerung: Der Plafond steigt je Januar einmal um 3 %, nicht um 3 % je Person.
// 01.2031: 3'780 × 1.03 = 3'893.40; 01.2033: 3'780 × 1.03³ = 4'130.51 (vorher 4'010.20 und 4'513.52).
#[test]
fn issue_145_ak2_couple_cap_follows_inflation_once_per_year() {
    let res = capped_ahv(true, true, "start");
    assert!((ahv_in(&res, 11) - 3_780.0).abs() < 0.01, "{}", ahv_in(&res, 11));
    assert!((ahv_in(&res, 12) - 3_893.40).abs() < 0.01, "{}", ahv_in(&res, 12));
    assert!((ahv_in(&res, 36) - 4_130.51).abs() < 0.01, "{}", ahv_in(&res, 36));
}

// #145/AK3: Nur die Rente von A mit Teuerung, beide laufen. 01.2031: A = 2'520 × 1.03 = 2'595.60, B = 2'520 fest,
// Summe 5'115.60; Plafond 3'780 × 1.03 = 3'893.40 → 3'893.40. Ohne Plafond (unverheiratet) die volle Summe.
#[test]
fn issue_145_ak3_cap_with_only_one_indexed_pension() {
    let res = capped_ahv(true, false, "start");
    assert!((ahv_in(&res, 12) - 3_893.40).abs() < 0.01, "{}", ahv_in(&res, 12));
    // Plafond greift erst, wenn er die Summe unterschreitet: A 1'000 mit Teuerung, B 2'000 fest.
    let mut v = with_partner(base());
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    v["pension"]["ahv"] = json!({ "monthly": 1_000, "at": "start", "indexed": true });
    v["partner"]["ahv"] = json!({ "monthly": 2_000, "at": "start", "indexed": false });
    v["household"]["ahvCoupleCap"] = json!(3_780);
    let res = run(v);
    // 01.2031: 1'030 + 2'000 = 3'030 < 3'893.40
    assert!((ahv_in(&res, 12) - 3_030.0).abs() < 0.01, "{}", ahv_in(&res, 12));
}

// #145/AK4: Die zweite Rente beginnt im Januar 2032. 01.2031: nur A, 2'520 × 1.03 = 2'595.60 (unter dem Plafond).
// 01.2032: A = 2'520 × 1.03² = 2'673.47, B beginnt mit 2'520; Summe 5'193.47, Plafond 3'780 × 1.03² = 4'010.20.
// 01.2033: beide angepasst, Plafond 3'780 × 1.03³ = 4'130.51 (vorher 4'254.42).
#[test]
fn issue_145_ak4_cap_when_second_pension_starts_later() {
    let res = capped_ahv(true, true, "b_ahv");
    assert!((ahv_in(&res, 12) - 2_595.60).abs() < 0.01, "{}", ahv_in(&res, 12));
    assert!((ahv_in(&res, 24) - 4_010.20).abs() < 0.01, "{}", ahv_in(&res, 24));
    assert!((ahv_in(&res, 36) - 4_130.51).abs() < 0.01, "{}", ahv_in(&res, 36));
}

#[test]
fn couple_tax_tables_replace_single_ones() {
    let mut v = with_partner(base());
    v["milestones"][1]["date"] = json!("2040-01");
    v["flows"]["income"] = json!(10_000);
    v["taxes"] = json!({
        "multiplier": 1.0,
        "income": [[0, 0.2]], "wealth": [], "capital": [],
        "couple": { "income": [[0, 0.1]], "wealth": [], "capital": [] }
    });
    let single = run(v.clone());
    assert!(
        (single.records[16].taxes - 120_000.0 * 0.1).abs() < 1e-6,
        "Verheiratetentarif"
    );
    v["household"]["married"] = json!(false);
    let unmarried = run(v);
    assert!((unmarried.records[16].taxes - 120_000.0 * 0.2).abs() < 1e-6);
}

#[test]
fn single_person_plan_ignores_couple_settings() {
    let mut v = base();
    v["taxes"]["couple"] = json!({ "income": [[0, 0.9]], "wealth": [], "capital": [] });
    v["flows"]["income"] = json!(1_000);
    v["milestones"][1]["date"] = json!("2040-01");
    v["taxes"]["income"] = json!([[0, 0.1]]);
    let res = run(v);
    assert!((res.records[16].taxes - 1_200.0).abs() < 1e-6);
}

// ---------------------------------------------------------------------------------------------
// Entnahmestrategien
// ---------------------------------------------------------------------------------------------

#[test]
fn fixed_rate_sets_spending_from_wealth() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } });
    let res = run(v);
    // 4 % von 1'000'000 = 40'000 pro Jahr = 3'333.33 pro Monat (statt 5'000 geplant)
    assert!((res.records[0].spending - 40_000.0 / 12.0).abs() < 1e-6);
    assert!((res.records[11].spending - 40_000.0 / 12.0).abs() < 1e-6);
    // Januar: 4 % des verbleibenden Vermögens (960'000)
    assert!(
        (res.records[12].spending - 38_400.0 / 12.0).abs() < 1e-6,
        "{}",
        res.records[12].spending
    );
    assert!(
        res.summary.depleted.is_none(),
        "eine Rate in Prozent des Vermögens erschöpft es nie"
    );
}

#[test]
fn fixed_rate_counts_income_and_floor_applies() {
    let mut v = base();
    v["flows"]["income"] = json!(1_000);
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } });
    let res = run(v.clone());
    // Lebenshaltung = Entnahme + Einkommen
    assert!((res.records[0].spending - (40_000.0 / 12.0 + 1_000.0)).abs() < 1e-6);
    // Untergrenze 100 % der geplanten Ausgaben, Obergrenze nicht gesetzt
    v["withdrawal"]["floor"] = json!(1.0);
    let res = run(v.clone());
    assert!((res.records[0].spending - 5_000.0).abs() < 1e-6);
    // Obergrenze 50 %
    v["withdrawal"].as_object_mut().unwrap().remove("floor");
    v["withdrawal"]["ceiling"] = json!(0.5);
    let res = run(v);
    assert!((res.records[0].spending - 2_500.0).abs() < 1e-6);
}

#[test]
fn strategy_starts_with_retirement_only() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2031-01");
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } });
    let res = run(v);
    assert!(
        (res.records[0].spending - 5_000.0).abs() < 1e-6,
        "vor der Pensionierung gilt die Planung"
    );
    assert!((res.records[12].spending - 0.04 * 940_000.0 / 12.0).abs() < 1e-6);
}

#[test]
fn guardrails_cut_spending_when_rate_gets_too_high() {
    let mut v = base();
    v["withdrawal"] = json!({
        "strategy": "guardrails",
        "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 }
    });
    let res = run(v);
    let monthly = |year: usize| res.records[year * 12].spending * 12.0;
    assert!((monthly(0) - 50_000.0).abs() < 1e-6);
    // Ohne Rendite und Teuerung steigt die Rate von 5 % auf über 6 % ab dem 5. Jahr
    // (Vermögen 800'000 → 6.25 %): Kürzung um 10 % auf 45'000.
    assert!((monthly(3) - 50_000.0).abs() < 1e-6);
    assert!((monthly(4) - 45_000.0).abs() < 1e-6, "{}", monthly(4));
    let cuts = res
        .events
        .iter()
        .filter(|e| e.kind == EventKind::Withdrawal && e.text.contains("gekürzt"))
        .count();
    assert!(cuts >= 1);
}

#[test]
fn guardrails_raise_spending_after_strong_markets() {
    let mut v = base();
    v["wealth"]["stocks"] = json!(1_000_000);
    v["wealth"]["cash"] = json!(0);
    v["returns"]["fixed"]["stocks"] = json!(0.20);
    v["withdrawal"] = json!({
        "strategy": "guardrails",
        "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 }
    });
    let res = run(v);
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::Withdrawal && e.text.contains("erhöht")));
    assert!(res.records[24].spending > res.records[0].spending);
}

#[test]
fn guardrails_follow_inflation() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["returns"]["fixed"]["stocks"] = json!(0.02);
    v["wealth"]["stocks"] = json!(1_000_000);
    v["wealth"]["cash"] = json!(0);
    v["withdrawal"] = json!({
        "strategy": "guardrails",
        "guardrails": { "initialRate": 0.04, "upper": 0.9, "lower": 0.9, "adjust": 0.1 }
    });
    let res = run(v);
    // Leitplanken weit weg: Entnahme wächst mit der Teuerung
    let a = res.records[0].spending;
    let b = res.records[12].spending;
    assert!((b / a - 1.02).abs() < 1e-9, "{}", b / a);
}

#[test]
fn invalid_strategy_parameters_are_reported() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.0 } });
    assert!(validate::validate(&plan(v.clone()))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "withdrawal"));
    v["withdrawal"] = json!({ "strategy": "guardrails", "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 1.5 } });
    assert!(validate::validate(&plan(v.clone()))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "withdrawal"));
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 }, "floor": 1.2, "ceiling": 0.8 });
    assert!(validate::validate(&plan(v))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "withdrawal"));
}

// ---------------------------------------------------------------------------------------------
// Auswertung
// ---------------------------------------------------------------------------------------------

#[test]
fn records_carry_market_indices() {
    let mut v = base();
    v["returns"]["fixed"]["stocks"] = json!(0.06);
    v["returns"]["fixed"]["bonds"] = json!(0.02);
    v["milestones"][1]["date"] = json!("2040-01");
    v["flows"]["spending"] = json!(0);
    let res = run(v);
    assert!((res.records[11].stock_index - 1.06).abs() < 1e-9);
    assert!((res.records[11].bond_index - 1.02).abs() < 1e-9);
    assert!((res.records[0].stock_index - 1.06f64.powf(1.0 / 12.0)).abs() < 1e-9);
}

#[test]
fn bands_and_percentiles_include_extremes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let ds = DataSet::load_csv(&dir.join("renditen-muster.csv")).unwrap();
    let mut v = base();
    v["milestones"][1]["date"] = json!("2040-01");
    v["flows"]["spending"] = json!(0);
    v["wealth"]["cash"] = json!(0);
    v["wealth"]["stocks"] = json!(100_000);
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["series"] = json!("renditen-muster.csv");
    v["returns"]["bootstrap"] = json!({ "paths": 300, "block": 12, "seed": 3 });
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::path_source(&p, &sim, Some(&ds)).unwrap();
    let (results, bands) = batch::run_with_bands(&sim, &paths);
    let summary = batch::summarize(&results);
    let q = |x: u32| summary.percentiles.iter().find(|p| p.percent == x).unwrap();
    let worst = results
        .iter()
        .map(|r| r.summary.end_total)
        .fold(f64::INFINITY, f64::min);
    assert_eq!(q(0).end_total, worst);
    assert!(q(0).end_total <= q(1).end_total && q(1).end_total <= q(99).end_total);
    for k in 0..sim.months() {
        assert!(bands.min[k] <= bands.p1[k] && bands.p1[k] <= bands.p5[k], "{k}");
        assert!(bands.p95[k] <= bands.p99[k], "{k}");
    }
    let last = sim.months() - 1;
    assert!((bands.min[last] - worst).abs() < 0.01 * worst.abs().max(1.0));
    let real = bands.real.as_ref().unwrap();
    assert!(real.min[last] <= real.p1[last]);
}
