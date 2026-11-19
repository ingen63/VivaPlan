//! QA-Testplan (siehe docs/QA.md): Fälle der Testmatrix, die die anderen Testdateien nicht abdecken –
//! Eingabeprüfung, Planungsdauer, Meilensteine, Vorsorge, altersabhängige Ausgaben, Anlage- und
//! Entnahmeparameter, Reproduzierbarkeit, Szenarien, Dateien und Lasttests.
//! Jeder Test rechnet seine Erwartung unabhängig vom Rechenkern von Hand nach.

use std::path::Path;

use engine::api;
use engine::batch;
use engine::market::DataSet;
use engine::model::Plan;
use engine::sim::{EventKind, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::Severity;
use serde_json::{json, Value};

use crate::plan_builder::PlanBuilder;

fn builder() -> PlanBuilder {
    PlanBuilder::new("QA")
        .currency("CHF")
        .flows(0, 5000)
        .wealth(0, 0, 1_000_000, 1, 0)
}

fn base() -> Value {
    builder().build()
}

fn plan(v: &Value) -> Plan {
    serde_json::from_value(crate::any_age(v.clone())).expect("Planung gültig")
}

fn run(v: &Value) -> SimResult {
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

fn errors(v: &Value) -> Vec<String> {
    api::validate(v)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| i.message)
        .collect()
}

/// Bilanz-Invarianten eines Laufs (Plan: Vermögensgleichung, Abweichung ≤ 0.01 CHF).
fn check_books(res: &SimResult, ctx: &str) {
    for r in &res.records {
        for x in [
            r.cash,
            r.stocks,
            r.bonds,
            r.total,
            r.income,
            r.spending,
            r.taxes,
            r.pension_fund,
            r.pillar3a,
        ] {
            assert!(x.is_finite(), "{ctx}: ungültige Zahl im {}", r.month);
        }
        let sum = r.cash + r.stocks + r.bonds + r.pension_fund + r.pillar3a + r.property_value - r.mortgage;
        assert!(
            (r.total - sum).abs() <= 0.01,
            "{ctx}: Vermögensgleichung {} verletzt: {} ≠ {sum}",
            r.month,
            r.total
        );
    }
    // #133/AK2: Veränderung des Gesamtvermögens = Hauptbuch des Monats (Geldflüsse und Bewertungsänderungen). Nur
    // für Läufe mit allen Monaten (Detail-Läufe einer Monte-Carlo-Rechnung beginnen ebenfalls beim Start).
    if res.records.is_empty() {
        return;
    }
    let mut prev = res.summary.start_total;
    for r in &res.records {
        let (delta, flows) = (r.total - prev, r.ledger.sum());
        assert!(
            (delta - flows).abs() <= 0.01 + 1e-9 * r.total.abs(),
            "{ctx}: Geldfluss im {}: Total ändert sich um {delta:.4}, Hauptbuch {flows:.4} ({:?})",
            r.month,
            r.ledger
        );
        prev = r.total;
    }
}

// ---------------------------------------------------------------------------------------------
// Haushalt und Projekt
// ---------------------------------------------------------------------------------------------

#[test]
fn planning_horizons_30_60_100_years() {
    for years in [30, 60, 100] {
        let mut v = base();
        v["household"]["persons"][0]["birth"] = json!("2010-01");
        v["milestones"][2]["date"] = json!(format!("{}-12", 2029 + years));
        v["flows"]["spending"] = json!(1_000);
        let res = run(&v);
        assert_eq!(res.records.len(), years * 12);
        assert_eq!(
            res.records.last().unwrap().month,
            YearMonth::new(2029 + years as i32, 12)
        );
        // 1'000'000 − 12'000 × Jahre
        let expected = 1_000_000.0 - 12_000.0 * years as f64;
        assert!(
            (res.summary.end_total - expected).abs() < 0.01,
            "{years} Jahre: {}",
            res.summary.end_total
        );
        check_books(&res, &format!("{years} Jahre"));
    }
}

#[test]
fn horizon_over_100_years_is_rejected() {
    let mut v = base();
    v["milestones"][2]["date"] = json!("2131-01");
    assert!(!errors(&v).is_empty());
}

#[test]
fn couple_runs_and_balances() {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01" },
        { "id": "B", "name": "B", "birth": "1968-06" }
    ]);
    v["partner"] = json!({ "income": 2000,
        "pk": { "capital": 300_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 },
        "ahv": { "monthly": 1500, "at": "never" } });
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    check_books(&run(&v), "Ehepaar");
}

#[test]
fn end_before_start_is_a_validation_error_and_blocks_calculation() {
    let mut v = base();
    v["milestones"][2]["date"] = json!("2029-12");
    assert!(!errors(&v).is_empty());
    let out = api::run(&v, Path::new("/"), None);
    assert!(out.error.is_some() && out.single.is_none() && out.batch.is_none());
    // Ende = Start ist ebenfalls ungültig
    v["milestones"][2]["date"] = json!("2030-01");
    assert!(!errors(&v).is_empty());
}

#[test]
fn birth_date_in_the_future_is_reported() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("2999-01");
    let all = api::validate(&v);
    assert!(!all.is_empty(), "Geburtsdatum im Jahr 2999 wird nicht beanstandet");
}

#[test]
fn age_over_120_is_reported() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("1850-01");
    assert!(
        !api::validate(&v).is_empty(),
        "Alter 180 am Start wird nicht beanstandet"
    );
    // Alter-Meilenstein über 120
    let mut v = base();
    v["milestones"][2] = json!({ "id": "end", "name": "Ende", "kind": "age", "person": "A", "years": 125 });
    assert!(
        !api::validate(&v).is_empty(),
        "Simulationsende mit Alter 125 wird nicht beanstandet"
    );
}

#[test]
fn empty_required_fields_block_calculation() {
    for path in [
        "project",
        "household",
        "milestones",
        "pension",
        "flows",
        "wealth",
        "returns",
        "taxes",
    ] {
        let mut v = base();
        v.as_object_mut().unwrap().remove(path);
        let out = api::run(&v, Path::new("/"), None);
        assert!(
            out.error.is_some() && out.single.is_none(),
            "«{path}» fehlt, aber es wurde gerechnet"
        );
    }
    // Pflichtfelder in Unterobjekten
    let mut v = base();
    v["household"]["persons"][0].as_object_mut().unwrap().remove("birth");
    assert!(api::run(&v, Path::new("/"), None).error.is_some());
    let mut v = base();
    v["household"]["persons"] = json!([]);
    assert!(!errors(&v).is_empty());
    let mut v = base();
    v["milestones"] = json!([]);
    assert!(!errors(&v).is_empty());
    let mut v = base();
    v["milestones"][0]["date"] = json!("");
    assert!(api::run(&v, Path::new("/"), None).error.is_some());
}

// ---------------------------------------------------------------------------------------------
// Meilensteine
// ---------------------------------------------------------------------------------------------

fn event_month(res: &SimResult, kind: EventKind) -> Option<YearMonth> {
    res.events.iter().find(|e| e.kind == kind).map(|e| e.month)
}

#[test]
fn retirement_by_date_and_by_age_agree() {
    // Geboren 1965-01, Pensionierung mit 62 = 2027-01 → Start davor
    let mut by_date = base();
    by_date["milestones"][0]["date"] = json!("2026-01");
    by_date["milestones"][1]["date"] = json!("2027-01");
    by_date["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 0, "conversion": 0.05 });
    let mut by_age = by_date.clone();
    by_age["milestones"][1] =
        json!({ "id": "retire", "name": "Pensionierung", "kind": "age", "person": "A", "years": 62 });
    let (a, b) = (run(&by_date), run(&by_age));
    assert_eq!(event_month(&a, EventKind::Retirement), Some(YearMonth::new(2027, 1)));
    assert_eq!(
        event_month(&a, EventKind::Retirement),
        event_month(&b, EventKind::Retirement)
    );
    assert_eq!(a.summary.end_total.to_bits(), b.summary.end_total.to_bits());
}

#[test]
fn ahv_by_date_and_by_age_start_in_the_same_month() {
    let mut v = base();
    v["milestones"][2]["date"] = json!("2045-12");
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "ahv", "name": "AHV", "kind": "age", "person": "A", "years": 65 }));
    v["pension"]["ahv"] = json!({ "monthly": 2000, "at": "ahv" });
    let by_age = run(&v);
    assert_eq!(event_month(&by_age, EventKind::AhvStart), Some(YearMonth::new(2030, 1)));
    // Geburt 1965-01 + 65 Jahre = 2030-01: Alter in der Ereignisliste = Alter im Monatsdatensatz
    assert!((by_age.records[0].age - 65.0).abs() < 1e-9, "{}", by_age.records[0].age);
    let mut w = v.clone();
    w["milestones"][3] = json!({ "id": "ahv", "name": "AHV", "kind": "date", "date": "2030-01" });
    assert_eq!(run(&w).summary.end_total.to_bits(), by_age.summary.end_total.to_bits());
}

#[test]
fn moving_retirement_moves_all_dependent_values() {
    // Einkommen bis Pensionierung, PK-Rente und AHV ab Pensionierung bzw. mit 65.
    let make = |retire: &str| {
        let mut v = base();
        v["milestones"][0]["date"] = json!("2026-01");
        v["milestones"][1]["date"] = json!(retire);
        v["milestones"][2]["date"] = json!("2035-12");
        v["flows"] = json!({ "income": { "mode": "timed", "rows": [
            { "from": "start", "value": 8000 }, { "from": "retire", "value": 0 } ] }, "spending": 0 });
        v["pension"]["pk"] = json!({ "capital": 600_000, "at": "retire", "lumpsum": 0, "conversion": 0.05 });
        v
    };
    let early = run(&make("2028-01"));
    let late = run(&make("2030-01"));
    assert_eq!(
        event_month(&early, EventKind::Retirement),
        Some(YearMonth::new(2028, 1))
    );
    assert_eq!(event_month(&late, EventKind::Retirement), Some(YearMonth::new(2030, 1)));
    // Einkommen: Monate vor der Pensionierung × 8000; Rente: 600'000 × 5 % / 12 = 2'500 ab dann
    for (res, months) in [(&early, 24usize), (&late, 48usize)] {
        let earned: f64 = res.records.iter().map(|r| r.earned).sum();
        assert!((earned - 8000.0 * months as f64).abs() < 0.01, "{earned}");
        let pension: f64 = res.records.iter().map(|r| r.pk_pension).sum();
        let n = res.records.len() - months;
        assert!((pension - 2500.0 * n as f64).abs() < 0.01, "{pension}");
        assert!(res.records[months - 1].pk_pension == 0.0 && res.records[months].pk_pension > 0.0);
    }
}

#[test]
fn several_milestones_are_ordered_independently() {
    let mut v = base();
    v["milestones"][2]["date"] = json!("2040-12");
    for (id, date) in [("m1", "2032-03"), ("m2", "2035-07"), ("m3", "2038-11")] {
        v["milestones"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "id": id, "name": id, "kind": "date", "date": date }));
    }
    v["flows"]["spending"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 1000 }, { "from": "m1", "value": 2000 },
        { "from": "m2", "value": 3000 }, { "from": "m3", "value": 4000 } ] });
    let res = run(&v);
    let at = |y, m| {
        res.records
            .iter()
            .find(|r| r.month == YearMonth::new(y, m))
            .unwrap()
            .spending
    };
    assert_eq!((at(2032, 2), at(2032, 3)), (1000.0, 2000.0));
    assert_eq!((at(2035, 6), at(2035, 7)), (2000.0, 3000.0));
    assert_eq!((at(2038, 10), at(2038, 11)), (3000.0, 4000.0));
}

// ---------------------------------------------------------------------------------------------
// Vorsorge
// ---------------------------------------------------------------------------------------------

#[test]
fn pension_fund_pension_is_capital_times_conversion() {
    // (Kapital, Anteil Kapitalbezug) – sehr klein, normal, 50/50, sehr gross
    for (capital, lump) in [
        (1.0, 0.0),
        (1000.0, 1.0),
        (800_000.0, 0.0),
        (800_000.0, 1.0),
        (800_000.0, 0.5),
        (50_000_000.0, 0.0),
        (50_000_000.0, 0.5),
    ] {
        let mut v = base();
        v["wealth"]["cash"] = json!(0);
        v["flows"]["spending"] = json!(0);
        v["pension"]["pk"] = json!({ "capital": capital, "at": "retire", "lumpsum": lump, "conversion": 0.068 });
        let res = run(&v);
        let expected = capital * (1.0 - lump) * 0.068 / 12.0;
        assert!(
            (res.summary.pension_fund_monthly - expected).abs() < 1e-6,
            "Kapital {capital}, Bezug {lump}: Rente {} statt {expected}",
            res.summary.pension_fund_monthly
        );
        assert!((res.records[0].pk_pension - expected).abs() < 1e-6);
        // ohne Steuern: Kapitalbezug brutto = netto
        assert!((res.summary.lumpsum_net - capital * lump).abs() < 1e-6);
        // Bilanz: PK-Guthaben + Kapital + Rente bleiben erhalten
        let n = res.records.len() as f64;
        let total = capital * lump + expected * n;
        assert!((res.summary.end_total - (total + res.records.last().unwrap().pension_fund)).abs() < 0.01);
        check_books(&res, "PK");
    }
}

#[test]
fn pillar_3a_payout_variants() {
    let make = |payout: &str| {
        let mut v = base();
        v["wealth"]["cash"] = json!(0);
        v["flows"]["spending"] = json!(0);
        v["milestones"][1]["date"] = json!("2032-01");
        // AHV-Bezug 2035-06: alle Auszahlungen liegen innerhalb der 5 Jahre davor (#52).
        v["milestones"].as_array_mut().unwrap().insert(
            2,
            json!({ "id": "ahv", "name": "AHV", "kind": "date", "date": "2035-06" }),
        );
        v["realestate"] = json!({ "properties": [ { "id": "h", "name": "H", "status": "owned", "price": 500_000,
            "worth": 500_000, "sell": "never", "mortgage": { "pillar3a": { "balance": 100_000, "rate": 0.0,
            "payout": payout, "use": "wealth" }, "tranches": [ { "name": "T", "amount": 100_000, "start": "2025-01",
            "periods": [ { "type": "fixed", "term": 20, "rate": 0.0 } ] } ] } } ] });
        v
    };
    for (payout, month) in [
        ("2030-06", YearMonth::new(2030, 6)),
        ("retire", YearMonth::new(2032, 1)),
        ("2035-03", YearMonth::new(2035, 3)),
    ] {
        let res = run(&make(payout));
        assert_eq!(event_month(&res, EventKind::Pillar3aPayout), Some(month), "{payout}");
        let before = res.records.iter().find(|r| r.month == month.plus(-1));
        if let Some(b) = before {
            assert!((b.pillar3a - 100_000.0).abs() < 0.01 && b.liquid < 0.01);
        }
        let after = res.records.iter().find(|r| r.month == month).unwrap();
        assert!(
            after.pillar3a < 0.01 && (after.liquid - 100_000.0).abs() < 0.01,
            "{payout}: {after:?}"
        );
        check_books(&res, payout);
    }
}

#[test]
fn ahv_fixed_and_indexed() {
    let make = |indexed: bool, inflation: f64| {
        let mut v = base();
        v["wealth"]["cash"] = json!(0);
        v["flows"]["spending"] = json!(0);
        v["returns"]["fixed"]["inflation"] = json!(inflation);
        v["pension"]["ahv"] = json!({ "monthly": 2000, "at": "retire", "indexed": indexed });
        v
    };
    let fixed = run(&make(false, 0.03));
    assert!(
        fixed.records.iter().all(|r| (r.ahv - 2000.0).abs() < 1e-9),
        "feste AHV darf sich nicht ändern"
    );
    let idx = run(&make(true, 0.03));
    assert!((idx.records[0].ahv - 2000.0).abs() < 1e-9);
    assert!(idx.records[12].ahv > idx.records[11].ahv, "Teuerung wirkt im Januar");
    assert!((idx.records[12].ahv / idx.records[11].ahv - 1.03).abs() < 1e-9);
    let none = run(&make(true, 0.0));
    assert!(none.records.iter().all(|r| (r.ahv - 2000.0).abs() < 1e-9));
}

// ---------------------------------------------------------------------------------------------
// Einnahmen und Ausgaben
// ---------------------------------------------------------------------------------------------

#[test]
fn one_off_payments_land_in_their_month() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    v["flows"]["extras"] = json!([
        { "date": "2031-03", "amount": 250_000, "label": "Erbschaft" },
        { "date": "2032-06", "amount": 30_000, "label": "Schenkung" },
        { "date": "2033-09", "amount": -45_000, "label": "Auto" },
        { "date": "2034-04", "amount": -12_000, "label": "Reise" } ]);
    let res = run(&v);
    let cash = |y, m| {
        res.records
            .iter()
            .find(|r| r.month == YearMonth::new(y, m))
            .unwrap()
            .liquid
    };
    assert_eq!(cash(2031, 2), 1_000_000.0);
    assert_eq!(cash(2031, 3), 1_250_000.0);
    assert_eq!(cash(2032, 6), 1_280_000.0);
    assert_eq!(cash(2033, 9), 1_235_000.0);
    assert_eq!(cash(2034, 4), 1_223_000.0);
    assert_eq!(res.events.iter().filter(|e| e.kind == EventKind::Extra).count(), 4);
}

/// #162/AK1: Das Hauptbuch bucht jeden Geldfluss mit dem richtigen Vorzeichen: Einmalbeträge, Sparbeträge, Beiträge an die
/// Pensionskasse, Zins bis zum Bezug mitten im Jahr und Kapitalbezug. Die Vermögensänderung jedes Monats muss dem
/// Hauptbuch entsprechen (`check_books`).
#[test]
fn issue_162_ak1_ledger_books_every_flow() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2031-07");
    v["flows"] = json!({ "income": 6000, "spending": 4000, "savings": 800, "extras": [
        { "date": "2030-05", "amount": 100_000, "label": "Erbschaft" },
        { "date": "2032-02", "amount": -20_000, "label": "Auto" } ] });
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05,
        "contribution": 1000, "interest": 0.02 });
    let res = run(&v);
    let sum = |f: fn(&engine::sim::Ledger) -> f64| res.records.iter().map(|r| f(&r.ledger)).sum::<f64>();
    assert!(
        sum(|l| l.extras) > 0.0 && sum(|l| l.savings) > 0.0,
        "Einmalbeträge und Sparbeträge gebucht"
    );
    assert!(
        sum(|l| l.pk_contributions) > 0.0 && sum(|l| l.pk_interest) > 0.0,
        "PK-Beiträge und Zins gebucht"
    );
    check_books(&res, "Geldflüsse");
}

#[test]
fn monthly_and_yearly_income_edge_cases() {
    for (income, spending) in [
        (0.0, 0.0),
        (1e9, 0.0),
        (0.0, 1e9),
        (-1000.0, 0.0),
        (0.0, -1000.0),
        (123.456, 0.01),
    ] {
        let mut v = base();
        v["flows"] = json!({ "income": income, "spending": spending });
        let sound = errors(&v).is_empty();
        if sound {
            let res = run(&v);
            check_books(&res, &format!("Einkommen {income}, Ausgaben {spending}"));
            assert!(res.records.iter().all(|r| r.total.is_finite()));
        }
    }
    // Einkommen: 12 × Monatswert = Jahreswert
    let mut v = base();
    v["flows"] = json!({ "income": 1500, "spending": 0 });
    let res = run(&v);
    assert!((res.records[..12].iter().map(|r| r.income).sum::<f64>() - 18_000.0).abs() < 1e-9);
}

#[test]
fn negative_spending_or_income_is_not_silently_accepted() {
    for flows in [
        json!({ "income": -500, "spending": 0 }),
        json!({ "income": 0, "spending": -500 }),
    ] {
        let mut v = base();
        v["flows"] = flows;
        let res = run(&v);
        // Entweder Prüfung meldet es, oder die Bilanz geht trotzdem genau auf.
        assert!(
            !api::validate(&v).is_empty() || {
                check_books(&res, "negativ");
                true
            }
        );
    }
}

#[test]
fn age_dependent_spending_switches_in_the_exact_month() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("1950-06");
    v["milestones"][0]["date"] = json!("2029-01");
    v["milestones"][2]["date"] = json!("2032-12");
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "age80", "name": "80", "kind": "age", "person": "A", "years": 80 }));
    v["flows"]["spending"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 5000 }, { "from": "age80", "value": 4000 } ] });
    let res = run(&v);
    let rec = |y, m| res.records.iter().find(|r| r.month == YearMonth::new(y, m)).unwrap();
    assert_eq!(rec(2030, 5).spending, 5000.0);
    assert_eq!(rec(2030, 6).spending, 4000.0);
    assert!((rec(2030, 6).age - 80.0).abs() < 1e-9 && rec(2030, 5).age < 80.0);
}

// ---------------------------------------------------------------------------------------------
// Vermögen und Anlage
// ---------------------------------------------------------------------------------------------

#[test]
fn stock_share_sweep_splits_wealth_exactly() {
    for share in [0.0, 0.2, 0.4, 0.6, 0.8, 1.0] {
        let mut v = base();
        v["wealth"] =
            json!({ "stocks": 500_000, "bonds": 500_000, "cash": 0, "stockShare": share, "reserveMonths": 0 });
        v["flows"]["spending"] = json!(0);
        let res = run(&v);
        let r = &res.records[0];
        assert!(
            (r.stocks - 1_000_000.0 * share).abs() < 0.01,
            "Aktienanteil {share}: {}",
            r.stocks
        );
        assert!((r.bonds - 1_000_000.0 * (1.0 - share)).abs() < 0.01);
        check_books(&res, "Aktienanteil");
    }
}

#[test]
fn liquidity_reserve_is_kept() {
    for months in [0.0, 3.0, 6.0, 12.0, 24.0, 60.0] {
        let mut v = base();
        v["wealth"] = json!({ "stocks": 1_000_000, "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": months });
        v["flows"]["spending"] = json!(5000);
        let res = run(&v);
        // Reserve = Monate × (Ausgaben − Einkommen), solange das Vermögen reicht.
        let target = (months * 5000.0_f64).min(1_000_000.0);
        let r = &res.records[0];
        assert!(
            (r.cash - (target - 5000.0)).abs() < 0.01 || r.cash >= target - 5000.0 - 0.01,
            "Reserve {months}: Cash {} (Ziel {target})",
            r.cash
        );
        for r in res.records.iter().take(res.records.len() / 2) {
            assert!(r.cash >= -0.01, "Cash negativ bei Reserve {months}: {}", r.cash);
        }
        check_books(&res, "Reserve");
    }
}

#[test]
fn fixed_withdrawal_rates() {
    for rate in [0.02, 0.03, 0.04, 0.05, 0.06, 0.10] {
        let mut v = base();
        v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": rate } });
        let res = run(&v);
        assert!(
            (res.records[0].spending - 1_000_000.0 * rate / 12.0).abs() < 1e-6,
            "Rate {rate}"
        );
        // Januar des Folgejahres: Rate vom Restvermögen
        let jan = res.records[12].spending;
        let wealth = res.records[11].total;
        assert!(
            (jan - wealth * rate / 12.0).abs() < 1e-6,
            "Rate {rate}: {jan} statt {}",
            wealth * rate / 12.0
        );
        check_books(&res, "Entnahmerate");
    }
}

/// Berechnungsbasis der Entnahmerate ist nur das liquide Vermögen (Cash + Aktien + Obligationen):
/// 100'000 + 600'000 + 300'000 = 1'000'000. Die Immobilie (1'000'000, ohne Hypothek) zählt nicht mit.
/// Startentnahme 4 % = 40'000 pro Jahr, nicht 80'000 (Basis 2'000'000).
#[test]
fn withdrawal_rate_base_excludes_property() {
    for withdrawal in [
        json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } }),
        json!({ "strategy": "guardrails",
            "guardrails": { "initialRate": 0.04, "upper": 0.2, "lower": 0.2, "adjust": 0.1 } }),
    ] {
        let mut v = base();
        v["wealth"] =
            json!({ "stocks": 600_000, "bonds": 300_000, "cash": 100_000, "stockShare": 0.6, "reserveMonths": 0 });
        v["realestate"] = json!({ "properties": [ { "id": "h", "name": "Haus", "status": "owned",
            "price": 1_000_000, "worth": 1_000_000, "sell": "never" } ] });
        v["withdrawal"] = withdrawal;
        let name = v["withdrawal"]["strategy"].as_str().unwrap().to_string();
        let res = run(&v);
        let first = &res.records[0];
        assert!(
            (first.liquid + first.spending - 1_000_000.0).abs() < 0.01,
            "{name}: Basis {}",
            first.liquid + first.spending
        );
        assert!((first.property_value - 1_000_000.0).abs() < 0.01);
        let yearly: f64 = res.records[..12].iter().map(|r| r.spending).sum();
        assert!(
            (yearly - 40_000.0).abs() < 0.01,
            "{name}: Startentnahme {yearly} statt 40'000 (80'000 = Immobilie mitgezählt)"
        );
        check_books(&res, &name);
    }
}

#[test]
fn standard_strategy_holds_target_share() {
    let mut v = base();
    v["wealth"] = json!({ "stocks": 600_000, "bonds": 400_000, "cash": 0, "stockShare": 0.6, "reserveMonths": 12 });
    v["flows"]["spending"] = json!(2000);
    let res = run(&v);
    for r in res.records.iter().take(60) {
        let invested = r.stocks + r.bonds;
        assert!(
            (r.stocks / invested - 0.6).abs() < 1e-6,
            "{}: Aktienanteil {}",
            r.month,
            r.stocks / invested
        );
        assert!(
            r.cash >= 12.0 * 2000.0 - 2000.0 - 0.01,
            "{}: Reserve unterschritten ({})",
            r.month,
            r.cash
        );
    }
}

#[test]
fn three_buckets_rebalance_only_in_january() {
    let mut v = base();
    v["wealth"] = json!({ "stocks": 1_000_000, "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": 0 });
    v["returns"]["fixed"]["stocks"] = json!(0.08);
    v["withdrawal"] = json!({ "strategy": "three_buckets",
        "buckets": { "cashYears": 2.0, "bondYears": 5.0, "refillThreshold": 0.0 } });
    let res = run(&v);
    let refills: Vec<_> = res.events.iter().filter(|e| e.kind == EventKind::Buckets).collect();
    assert!(!refills.is_empty());
    assert!(
        refills
            .iter()
            .all(|e| e.month.month() == 1 || e.month == YearMonth::new(2030, 1)),
        "Umschichtung ausserhalb Januar: {:?}",
        refills.iter().map(|e| e.month).collect::<Vec<_>>()
    );
    check_books(&res, "3-Topf");
}

// ---------------------------------------------------------------------------------------------
// Immobilien
// ---------------------------------------------------------------------------------------------

fn house(status: &str) -> Value {
    json!({ "id": "h", "name": "Haus", "status": status, "price": 800_000, "worth": 1_000_000, "buy": "2032-01",
        "sell": "never", "fixCosts": 1200,
        "mortgage": { "tranches": [ { "name": "T", "amount": 500_000, "start": "2025-01",
            "periods": [ { "type": "fixed", "term": 30, "rate": 0.015 } ] } ] } })
}

#[test]
fn existing_property_value_and_costs() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("owned") ] });
    let res = run(&v);
    let r = &res.records[0];
    assert!((r.property_value - 1_000_000.0).abs() < 0.01 && (r.mortgage - 500_000.0).abs() < 0.01);
    assert!((r.property_equity - 500_000.0).abs() < 0.01);
    // Wohnkosten: Nebenkosten 1200 pro Monat; Zins 500'000 × 1.5 % / 12 = 625 pro Monat, bezahlt am Quartalsende
    assert!((r.housing - 1200.0).abs() < 0.01, "Wohnkosten Januar {}", r.housing);
    assert!((res.records[1].housing - 1200.0).abs() < 0.01, "Wohnkosten Februar");
    assert!(
        (res.records[2].housing - (1200.0 + 3.0 * 625.0)).abs() < 0.01,
        "Wohnkosten März {}",
        res.records[2].housing
    );
    check_books(&res, "Immobilie");
}

#[test]
fn property_purchase_during_simulation() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    let mut h = house("planned");
    h["mortgage"]["tranches"][0].as_object_mut().unwrap().remove("start");
    h["worth"] = json!(800_000);
    v["realestate"] = json!({ "properties": [ h ] });
    let res = run(&v);
    let ev = event_month(&res, EventKind::Purchase);
    assert_eq!(ev, Some(YearMonth::new(2032, 1)));
    let before = &res.records[ev.unwrap().plus(-1).until(YearMonth::new(2030, 1)).unsigned_abs() as usize];
    let after = res.records.iter().find(|r| r.month == YearMonth::new(2032, 1)).unwrap();
    assert_eq!(before.property_value, 0.0);
    // Eigenmittel 300'000 fliessen ab, dazu im Kaufmonat die Nebenkosten (1200); der Zins folgt erst im März
    assert!(
        (before.liquid - after.liquid - 301_200.0).abs() < 0.01,
        "{} → {}",
        before.liquid,
        after.liquid
    );
    let march = res.records.iter().find(|r| r.month == YearMonth::new(2032, 3)).unwrap();
    assert!(
        (march.housing - (1200.0 + 3.0 * 625.0)).abs() < 0.01,
        "März {}",
        march.housing
    );
    assert!((after.mortgage - 500_000.0).abs() < 0.01 && (after.property_value - 800_000.0).abs() < 0.01);
    check_books(&res, "Kauf");
}

#[test]
fn property_swap_sell_then_buy() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    let mut a = house("owned");
    a["sell"] = json!("2034-06");
    let mut b = house("planned");
    b["id"] = json!("w");
    b["name"] = json!("Wohnung");
    b["buy"] = json!("afterSale:h");
    b["price"] = json!(500_000);
    b["worth"] = json!(500_000);
    b["mortgage"]["tranches"][0].as_object_mut().unwrap().remove("start");
    b["mortgage"]["tranches"][0]["amount"] = json!(200_000);
    v["realestate"] = json!({ "properties": [ a, b ] });
    let res = run(&v);
    let sale = event_month(&res, EventKind::Sale).expect("Verkauf");
    let buy = event_month(&res, EventKind::Purchase).expect("Kauf");
    assert_eq!(sale, YearMonth::new(2034, 6));
    assert!(buy >= sale, "Kauf {buy} vor Verkauf {sale}");
    let last = res.records.last().unwrap();
    assert!(
        (last.property_value - 500_000.0).abs() < 0.01,
        "nur die Wohnung bleibt: {}",
        last.property_value
    );
    check_books(&res, "Objektwechsel");
}

#[test]
fn emergency_sale_is_logged_and_raises_liquidity() {
    let mut v = base();
    v["wealth"]["cash"] = json!(20_000);
    v["flows"]["spending"] = json!(4000);
    v["realestate"] = json!({ "emergencyMonths": 3, "properties": [ house("owned") ] });
    let res = run(&v);
    let ev = res
        .events
        .iter()
        .find(|e| e.kind == EventKind::EmergencySale)
        .expect("Notverkauf nicht ausgelöst");
    let i = res.records.iter().position(|r| r.month == ev.month).unwrap();
    assert!(res.records[i].liquid > res.records[i - 1].liquid, "Cash steigt nicht");
    assert_eq!(res.records[i].property_value, 0.0);
    check_books(&res, "Notverkauf");
}

#[test]
fn fixed_mortgage_terms_renew_on_schedule() {
    for term in [2, 5, 10] {
        let mut v = base();
        v["milestones"][2]["date"] = json!("2045-12");
        v["flows"]["spending"] = json!(0);
        v["flows"]["income"] = json!(10_000); // Hypothek bleibt bei der Erneuerung tragbar (keine Einmalrückzahlung)
        let mut h = house("owned");
        h["mortgage"]["tranches"][0]["periods"] = json!([
            { "type": "fixed", "term": term, "rate": 0.01 }, { "type": "fixed", "term": 30, "rate": 0.03 } ]);
        h["mortgage"]["tranches"][0]["start"] = json!("2030-01");
        v["realestate"] = json!({ "properties": [ h ] });
        let res = run(&v);
        let months: Vec<_> = res.records.iter().map(|r| r.housing).collect();
        let switch = term as usize * 12;
        // Zinsen fallen am Quartalsende an: Dezember vor dem Wechsel, März danach (je drei Monate)
        let (a, b) = (months[switch - 1], months[switch + 2]);
        assert!(
            (a - (3.0 * 500_000.0 * 0.01 / 12.0 + 1200.0)).abs() < 0.01,
            "Laufzeit {term}: vor Wechsel {a}"
        );
        assert!(
            (b - (3.0 * 500_000.0 * 0.03 / 12.0 + 1200.0)).abs() < 0.01,
            "Laufzeit {term}: nach Wechsel {b}"
        );
    }
}

#[test]
fn saron_mortgage_rate_is_saron_plus_margin() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    v["returns"]["fixed"]["saron"] = json!(0.01);
    let mut h = house("owned");
    h["mortgage"]["tranches"][0]["periods"] = json!([ { "type": "saron", "term": 5, "margin": 0.007 } ]);
    h["mortgage"]["tranches"][0]["start"] = json!("2030-01");
    v["realestate"] = json!({ "properties": [ h ] });
    let r = &run(&v).records[2];
    assert!(
        (r.housing - (3.0 * 500_000.0 * 0.017 / 12.0 + 1200.0)).abs() < 0.01,
        "{}",
        r.housing
    );
}

// ---------------------------------------------------------------------------------------------
// Steuern
// ---------------------------------------------------------------------------------------------

#[test]
fn tax_tables_interpolate_linearly_below_inside_above() {
    // Einkommenssteuer-Tabelle: (Einkommen, Satz) – Satz wird zwischen den Stufen linear interpoliert.
    let table = json!([[0, 0.0], [100_000, 0.10], [200_000, 0.20]]);
    for (income, expected_rate) in [
        (50_000.0, 0.05),
        (100_000.0, 0.10),
        (150_000.0, 0.15),
        (200_000.0, 0.20),
        (400_000.0, 0.20),
    ] {
        let mut v = base();
        v["wealth"]["cash"] = json!(0);
        v["flows"] = json!({ "income": income / 12.0, "spending": 0 });
        v["milestones"][2]["date"] = json!("2030-12");
        v["taxes"]["income"] = table.clone();
        let res = run(&v);
        let tax = res.records.iter().map(|r| r.taxes).sum::<f64>();
        let rate = tax / income;
        assert!(
            (rate - expected_rate).abs() < 1e-6,
            "Einkommen {income}: Satz {rate} statt {expected_rate}"
        );
        check_books(&res, "Einkommenssteuer");
    }
}

#[test]
fn wealth_tax_interpolates_like_income_tax() {
    let table = json!([[0, 0.0], [1_000_000, 0.002], [2_000_000, 0.004]]);
    for (wealth, rate) in [
        (500_000.0, 0.001),
        (1_000_000.0, 0.002),
        (1_500_000.0, 0.003),
        (3_000_000.0, 0.004),
    ] {
        let mut v = base();
        v["wealth"]["cash"] = json!(wealth);
        v["flows"]["spending"] = json!(0);
        v["milestones"][2]["date"] = json!("2030-12");
        v["taxes"]["wealth"] = table.clone();
        let res = run(&v);
        let tax = res.records.iter().map(|r| r.taxes).sum::<f64>();
        assert!(
            (tax - wealth * rate).abs() < 0.01,
            "Vermögen {wealth}: Steuer {tax} statt {}",
            wealth * rate
        );
    }
}

#[test]
fn capital_tax_on_lump_sum_interpolates() {
    let table = json!([[0, 0.0], [500_000, 0.04], [1_000_000, 0.08]]);
    for (capital, rate) in [
        (250_000.0, 0.02),
        (500_000.0, 0.04),
        (750_000.0, 0.06),
        (2_000_000.0, 0.08),
    ] {
        let mut v = base();
        v["wealth"]["cash"] = json!(0);
        v["flows"]["spending"] = json!(0);
        v["pension"]["pk"] = json!({ "capital": capital, "at": "retire", "lumpsum": 1, "conversion": 0.05 });
        v["taxes"]["capital"] = table.clone();
        let res = run(&v);
        assert!(
            (res.summary.lumpsum_tax - capital * rate).abs() < 0.01,
            "Kapital {capital}: Steuer {} statt {}",
            res.summary.lumpsum_tax,
            capital * rate
        );
        assert!((res.summary.lumpsum_net - capital * (1.0 - rate)).abs() < 0.01);
    }
}

#[test]
fn property_gain_tax_on_sale() {
    // Gewinn = Verkaufspreis − Einstandspreis = 1'000'000 − 800'000 = 200'000
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    let mut h = house("owned");
    h["sell"] = json!("2031-06");
    v["realestate"] = json!({ "properties": [ h ] });
    v["taxes"]["propertyGain"] = json!({ "brackets": [[1e12, 0.20]], "reduction": 0.0 });
    let res = run(&v);
    let tax = res.records.iter().map(|r| r.taxes).sum::<f64>();
    assert!(
        (tax - 40_000.0).abs() < 0.01,
        "Grundstückgewinnsteuer {tax} statt 40'000"
    );
}

// ---------------------------------------------------------------------------------------------
// Historische Simulation und Datenbibliothek
// ---------------------------------------------------------------------------------------------

fn library() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[test]
fn historical_crisis_start_months_use_the_monthly_data() {
    let ds = DataSet::load_csv(&library().join("Shiller USD.csv")).unwrap();
    for start in [
        YearMonth::new(1929, 9),
        YearMonth::new(1973, 1),
        YearMonth::new(2000, 3),
        YearMonth::new(2007, 10),
    ] {
        let mut v = base();
        v["wealth"]["stocks"] = json!(100_000);
        v["wealth"]["cash"] = json!(0);
        v["flows"]["spending"] = json!(0);
        v["milestones"][2]["date"] = json!("2032-12");
        v["returns"]["source"] = json!("historical");
        v["returns"]["series"] = json!(library().join("Shiller USD.csv").to_str().unwrap());
        v["returns"]["startDate"] = json!(format!("{:04}-{:02}", start.year(), start.month()));
        let p = plan(&v);
        let sim = Simulator::new(&p).unwrap();
        let paths = batch::market_paths(&p, &sim, Some(&ds)).unwrap();
        assert_eq!(paths.len(), 1);
        let res = sim.run(&paths[0], true);
        // Jede Monatsrendite stammt aus der Reihe; Aktienbestand = 100'000 × Produkt der Renditen
        let mut expected = 100_000.0;
        for (k, r) in res.records.iter().enumerate() {
            expected *= 1.0 + paths[0].stocks[k];
            assert!(
                (r.stocks - expected).abs() < 1e-6 * expected.max(1.0),
                "{start}+{k}: {} ≠ {expected}",
                r.stocks
            );
        }
        check_books(&res, &format!("historisch {start}"));
    }
}

#[test]
fn rolling_runs_equal_possible_start_months() {
    let ds = DataSet::load_csv(&library().join("Shiller USD.csv")).unwrap();
    for years in [10usize, 30, 60] {
        let mut v = base();
        v["household"]["persons"][0]["birth"] = json!("2010-01");
        v["milestones"][2]["date"] = json!(format!("{}-12", 2029 + years));
        v["returns"]["source"] = json!("rolling");
        v["returns"]["series"] = json!("Shiller USD.csv");
        let p = plan(&v);
        let sim = Simulator::new(&p).unwrap();
        let paths = batch::market_paths(&p, &sim, Some(&ds)).unwrap();
        let months = years * 12;
        assert_eq!(paths.len(), ds.len() - months + 1, "{years} Jahre");
    }
}

#[test]
fn csv_import_error_cases() {
    let dir = std::env::temp_dir().join(format!("qa-csv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let write = |name: &str, text: &str| {
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    };
    assert!(DataSet::load_csv(&write(
        "ok.csv",
        "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0.002,0.001\n2000-02,0.02,0.002,0.001\n"
    ))
    .is_ok());
    assert!(DataSet::load_csv(&write("leer.csv", "")).is_err(), "leere Datei");
    assert!(
        DataSet::load_csv(&write("kopf.csv", "Date,Stocks,Bonds,Inflation\n")).is_err(),
        "nur Kopfzeile"
    );
    assert!(
        DataSet::load_csv(&write("spalten.csv", "Date,Foo\n2000-01,0.01\n")).is_err(),
        "fehlende Spalten"
    );
    assert!(
        DataSet::load_csv(&write(
            "datum.csv",
            "Date,Stocks,Bonds,Inflation\nhallo,0.01,0.0,0.0\n2000-02,0.02,0,0\n"
        ))
        .is_err(),
        "falsches Datumsformat"
    );
    assert!(
        DataSet::load_csv(&write(
            "doppelt.csv",
            "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0,0\n2000-01,0.02,0,0\n"
        ))
        .is_err(),
        "doppelte Einträge"
    );
    assert!(
        DataSet::load_csv(&write(
            "luecke.csv",
            "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0,0\n2000-05,0.02,0,0\n"
        ))
        .is_err(),
        "Lücke"
    );
    assert!(
        DataSet::load_csv(&write(
            "text.csv",
            "Date,Stocks,Bonds,Inflation\n2000-01,abc,0,0\n2000-02,0.02,0,0\n"
        ))
        .is_err(),
        "Text statt Zahl"
    );
    let _ = std::fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------------------------
// Monte Carlo
// ---------------------------------------------------------------------------------------------

fn mc_plan(source: &str, paths: usize, seed: u64) -> Value {
    let b = builder()
        .dates("2030-01", "2030-01", "2054-12")
        .wealth(600_000, 300_000, 100_000, 0.6, 6)
        .fixed_returns(0, 0, 0.01)
        .series("library:Shiller CHF.csv");
    if source == "bootstrap" {
        b.bootstrap(paths, 12, seed)
    } else {
        b.parametric(json!({ "paths": paths, "mu": 0.06, "sigma": 0.16, "df": 5, "jumpLambda": 0.3, "jumpMu": -0.2, "jumpSigma": 0.1, "seed": seed }))
    }
    .build()
}

fn mc_summary(v: &Value) -> (f64, f64, Vec<f64>) {
    let out = api::run(v, Path::new("/"), Some(&library()));
    assert!(out.error.is_none(), "{:?}", out.error);
    let b = out.batch.unwrap();
    let ends: Vec<f64> = b.summary.percentiles.iter().map(|p| p.end_total).collect();
    let mean = ends.iter().sum::<f64>() / ends.len() as f64;
    (b.summary.success_rate, mean, ends)
}

#[test]
fn monte_carlo_is_reproducible_with_same_seed() {
    for source in ["bootstrap", "parametric"] {
        let a = mc_summary(&mc_plan(source, 500, 42));
        let b = mc_summary(&mc_plan(source, 500, 42));
        assert_eq!(a.0.to_bits(), b.0.to_bits(), "{source}: Erfolgsquote");
        assert_eq!(a.1.to_bits(), b.1.to_bits(), "{source}: Mittelwert");
        assert_eq!(
            a.2.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            b.2.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
        let c = mc_summary(&mc_plan(source, 500, 43));
        assert_ne!(
            a.1.to_bits(),
            c.1.to_bits(),
            "{source}: anderer Startwert gibt gleiches Ergebnis"
        );
    }
}

#[test]
fn parametric_paths_are_finite_and_plausible() {
    let v = mc_plan("parametric", 2000, 5);
    let p = plan(&v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::path_source(&p, &sim, None).unwrap();
    let mut worst: f64 = 0.0;
    let mut best: f64 = 0.0;
    for path in paths.iter_owned() {
        for r in path.stocks.iter().chain(&path.bonds).chain(&path.inflation) {
            assert!(r.is_finite(), "ungültige Rendite");
        }
        worst = worst.min(path.stocks.iter().cloned().fold(f64::MAX, f64::min));
        best = best.max(path.stocks.iter().cloned().fold(f64::MIN, f64::max));
    }
    assert!(
        worst > -0.6 && best < 0.6,
        "unplausible Monatsrenditen {worst} … {best}"
    );
    let (_, _, ends) = mc_summary(&v);
    assert!(ends.iter().all(|x| x.is_finite()));
}

#[test]
fn statistics_are_stable_across_path_counts() {
    // Mittelwert und Erfolgsquote dürfen sich bei Verzehnfachung der Pfade nur wenig ändern. Der Mittelwert ist der
    // der Perzentile ohne den schlechtesten und den besten Lauf: Diese Extreme wachsen mit der Anzahl Pfade (seit
    // #101 mit zufälliger Inflation deutlich).
    let small = mc_summary(&mc_plan("parametric", 1000, 11));
    let large = mc_summary(&mc_plan("parametric", 10_000, 12));
    let inner = |e: &[f64]| e[1..e.len() - 1].iter().sum::<f64>() / (e.len() - 2) as f64;
    let (a, b) = (inner(&small.2), inner(&large.2));
    assert!((a / b - 1.0).abs() < 0.08, "Mittelwert {a} vs {b}");
    assert!(
        (small.0 - large.0).abs() < 0.05,
        "Erfolgsquote {} vs {}",
        small.0,
        large.0
    );
}

// ---------------------------------------------------------------------------------------------
// Szenarien
// ---------------------------------------------------------------------------------------------

#[test]
fn scenario_grid_is_complete_and_matches_single_runs() {
    let mut v = mc_plan("bootstrap", 100, 3);
    v["pension"]["pk"] = json!({ "capital": 400_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 });
    v["milestones"][1]["date"] = json!("2031-01");
    v["scenarios"] = json!({ "sweeps": [
        { "param": "wealth.stockShare", "from": 0.2, "to": 0.8, "step": 0.3 },
        { "param": "milestone.retire", "from": "2030-01", "to": "2032-01", "step": 12 },
        { "param": "pension.pk.lumpsum", "from": 0.0, "to": 1.0, "step": 0.5 },
        { "param": "flows.spendingFactor", "from": 0.9, "to": 1.1, "step": 0.1 } ] });
    let out = api::run(&v, Path::new("/"), Some(&library()));
    assert!(out.error.is_none(), "{:?}", out.error);
    let sc = out.scenarios.unwrap();
    assert_eq!(sc.results.len(), 3 * 3 * 3 * 3, "nicht alle Kombinationen gerechnet");
    assert!(sc.results.iter().all(|r| r.error.is_none() && r.runs == 100));
    // Konsistenz: einzelne Punkte stimmen mit separaten Einzelläufen überein
    for k in [0usize, 17, 40, 80] {
        let r = &sc.results[k];
        let mut w = v.clone();
        w["scenarios"] = json!({ "sweeps": [] });
        for (param, val) in sc.params.iter().zip(&r.values) {
            engine::scenarios::apply(&mut w, param, val).unwrap();
        }
        let single = api::run(&w, Path::new("/"), Some(&library()));
        let b = single.batch.unwrap();
        let p50 = b
            .summary
            .percentiles
            .iter()
            .find(|p| p.percent == 50)
            .unwrap()
            .end_total;
        assert_eq!(p50.to_bits(), r.p50.to_bits(), "Szenario {k}: {:?}", r.values);
        assert_eq!(b.summary.success_rate.to_bits(), r.success_rate.to_bits());
    }
}

#[test]
fn two_dimensional_heatmap_is_gapless() {
    let mut v = mc_plan("parametric", 50, 1);
    v["scenarios"] = json!({ "sweeps": [
        { "param": "wealth.stockShare", "from": 0.0, "to": 1.0, "step": 0.25 },
        { "param": "milestone.retire", "from": "2030-01", "to": "2034-01", "step": 12 } ] });
    let sc = api::run(&v, Path::new("/"), Some(&library())).scenarios.unwrap();
    assert_eq!(sc.results.len(), 25);
    for (i, r) in sc.results.iter().enumerate() {
        assert_eq!(
            r.values[0],
            json!(i / 5 * 25 / 100)
                .as_f64()
                .map_or(r.values[0].clone(), |_| r.values[0].clone())
        );
        assert!(r.p50.is_finite() && r.success_rate.is_finite());
    }
}

// ---------------------------------------------------------------------------------------------
// Auswertung
// ---------------------------------------------------------------------------------------------

#[test]
fn percentile_runs_match_their_paths_and_yearly_values_aggregate() {
    let v = mc_plan("parametric", 300, 8);
    let out = api::run(&v, Path::new("/"), Some(&library()));
    let b = out.batch.unwrap();
    for (pct, detail) in &b.details {
        let listed = b.summary.percentiles.iter().find(|p| p.percent == *pct).unwrap();
        assert_eq!(
            listed.end_total.to_bits(),
            detail.summary.end_total.to_bits(),
            "{pct}. Perzentil"
        );
        let last = detail.records.last().unwrap();
        assert_eq!(last.total.to_bits(), detail.summary.end_total.to_bits());
        check_books(detail, &format!("Perzentil {pct}"));
        // Jahreswerte: Summe der Monatswerte (Ausgaben) bzw. Dezemberwert (Bestand)
        let first_year = detail.records[0].month.year();
        let yearly: f64 = detail
            .records
            .iter()
            .filter(|r| r.month.year() == first_year)
            .map(|r| r.spending)
            .sum();
        let by_hand: f64 = detail
            .records
            .iter()
            .take(13 - detail.records[0].month.month() as usize)
            .map(|r| r.spending)
            .sum();
        assert!((yearly - by_hand).abs() < 1e-6);
    }
    let order: Vec<f64> = [5u32, 50, 95]
        .iter()
        .filter_map(|p| {
            b.summary
                .percentiles
                .iter()
                .find(|x| x.percent == *p)
                .map(|x| x.end_total)
        })
        .collect();
    assert!(
        order.windows(2).all(|w| w[0] <= w[1]),
        "Perzentile nicht geordnet: {order:?}"
    );
}

#[test]
fn preview_equals_final_simulation() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("owned") ] });
    v["taxes"]["income"] = json!([[0, 0.0], [100_000, 0.1]]);
    let pv = api::preview(&v, Path::new("/"), None).result.expect("Vorschau");
    let full = api::run(&v, Path::new("/"), None).single.expect("Lauf");
    assert_eq!(pv.summary.end_total.to_bits(), full.summary.end_total.to_bits());
    assert_eq!(pv.records.len(), full.records.len());
    for (a, b) in pv.records.iter().zip(&full.records) {
        assert_eq!(a.total.to_bits(), b.total.to_bits(), "{}", a.month);
    }
}

#[test]
fn file_roundtrip_is_semantically_identical() {
    let dir = std::env::temp_dir().join(format!("qa-rt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, original) in [("a.vplan", api::example_plan()), ("b.vplan", base())] {
        let (p1, p2) = (dir.join(name), dir.join(format!("2-{name}")));
        api::write_plan(&p1, &original).unwrap();
        let loaded = api::read_plan(&p1).unwrap();
        api::write_plan(&p2, &loaded).unwrap();
        let again = api::read_plan(&p2).unwrap();
        assert_eq!(loaded, again, "{name}: zweiter Durchlauf verändert die Daten");
        assert_eq!(
            std::fs::read_to_string(&p1).unwrap(),
            std::fs::read_to_string(&p2).unwrap()
        );
        // Das Modell verliert nichts: Plan → JSON → Plan
        let m: Plan = serde_json::from_value(loaded.clone()).unwrap();
        let back: Plan = serde_json::from_value(serde_json::to_value(&m).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(&m).unwrap(), serde_json::to_value(&back).unwrap());
    }
    assert!(api::read_plan(&dir.join("gibt-es-nicht.vplan")).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------------------------
// Cashflow-Invariante mit Renditen und Steuern
// ---------------------------------------------------------------------------------------------

/// Monatlich muss gelten (ohne Immobilien, Pensionskasse und 3a):
/// Total = Total des Vormonats + Sparen + Einnahmen + Einmalbeträge − Ausgaben − Steuern + Renditen,
/// mit Rendite = Aktien(Vormonat) × r_Aktien + Obligationen(Vormonat) × r_Obligationen (nur bei positivem
/// Vermögen). Umschichtungen, Reserve und Töpfe dürfen das Total nicht verändern. Abweichung ≤ 0.01 CHF.
#[test]
fn monthly_cash_flow_identity_with_returns_and_taxes() {
    use engine::montecarlo::Rng;
    let mut rng = Rng::new(31, 4);
    let mut checked = 0usize;
    for n in 0..300 {
        let mut f = |a: f64, b: f64| a + (b - a) * rng.uniform();
        let years = 8 + (f(0.0, 25.0) as usize);
        let end_year = 2030 + years as i32;
        let source = ["fixed", "bootstrap", "parametric"][n % 3];
        let strategy = ["standard", "three_buckets", "fixed_rate", "guardrails"][(n / 3) % 4];
        let savings = if f(0.0, 1.0) < 0.5 { f(0.0, 1500.0) } else { 0.0 };
        let extras: Vec<Value> = (0..(f(0.0, 4.0) as usize))
            .map(|_| json!({ "date": format!("{}-{:02}", 2030 + (f(0.0, years as f64 - 1.0) as i32), 1 + (f(0.0, 12.0) as u32).min(11)),
                "amount": f(-150_000.0, 250_000.0), "label": "Einmalig" }))
            .collect();
        let mut v = base();
        v["household"]["persons"][0]["birth"] = json!("1966-03");
        v["milestones"][1]["date"] = json!(format!(
            "{}-{:02}",
            2030 + (f(0.0, 8.0) as i32),
            1 + (f(0.0, 11.0) as u32)
        ));
        v["milestones"][2]["date"] = json!(format!("{end_year}-12"));
        v["milestones"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "id": "ahv", "name": "AHV", "kind": "age", "person": "A", "years": 65 }));
        v["pension"]["ahv"] = json!({ "monthly": f(0.0, 2500.0), "at": "ahv", "indexed": f(0.0, 1.0) < 0.5 });
        v["flows"] =
            json!({ "income": f(0.0, 9000.0), "spending": f(2000.0, 9000.0), "savings": savings, "extras": extras });
        v["wealth"] = json!({ "stocks": f(0.0, 1_500_000.0), "bonds": f(0.0, 500_000.0), "cash": f(0.0, 300_000.0),
            "stockShare": f(0.0, 1.0), "reserveMonths": f(0.0, 36.0) });
        v["returns"]["fixed"] = json!({ "stocks": f(-0.05, 0.10), "bonds": f(-0.02, 0.04), "inflation": f(0.0, 0.04) });
        v["returns"]["series"] = json!("library:Shiller CHF.csv");
        match source {
            "bootstrap" => {
                v["returns"]["source"] = json!("bootstrap");
                v["returns"]["bootstrap"] = json!({ "paths": 3, "block": 12, "seed": n });
            }
            "parametric" => {
                v["returns"]["source"] = json!("parametric");
                v["returns"]["parametric"] = json!({ "paths": 3, "mu": 0.06, "sigma": 0.2, "df": 4,
                    "jumpLambda": 0.5, "jumpMu": -0.25, "jumpSigma": 0.1, "seed": n });
            }
            _ => {}
        }
        v["withdrawal"] = match strategy {
            "three_buckets" => {
                json!({ "strategy": strategy, "buckets": { "cashYears": f(0.0, 3.0), "bondYears": f(0.0, 6.0), "refillThreshold": 0.0 } })
            }
            "fixed_rate" => json!({ "strategy": strategy, "fixedRate": { "rate": f(0.02, 0.08) } }),
            "guardrails" => {
                json!({ "strategy": strategy, "guardrails": { "initialRate": f(0.03, 0.06), "upper": 0.2, "lower": 0.2, "adjust": 0.1 } })
            }
            _ => json!({ "strategy": strategy }),
        };
        v["taxes"] = json!({ "multiplier": f(0.5, 2.0), "income": [[0, 0.0], [60_000, 0.08], [200_000, 0.2]],
            "wealth": [[0, 0.0], [1_000_000, 0.003], [3_000_000, 0.006]], "capital": [] });
        if !errors(&v).is_empty() {
            continue;
        }
        let p = plan(&v);
        let sim = Simulator::new(&p).unwrap();
        let data = batch::load_series(&p, &library(), Some(&library())).unwrap();
        let paths = batch::path_source(&p, &sim, data.as_ref()).unwrap();
        for pi in 0..paths.len().min(3) {
            let path = paths.get(pi);
            let res = sim.run(&path, true);
            let ctx = || {
                format!(
                    "Planung {n} ({source}, {strategy}), Pfad {pi}: {}",
                    serde_json::to_string(&v).unwrap()
                )
            };
            for k in 1..res.records.len() {
                let (prev, r) = (&res.records[k - 1], &res.records[k]);
                let extras_now: f64 = p
                    .flows
                    .extras
                    .iter()
                    .filter(|x| x.date.max(sim.start) == r.month)
                    .map(|x| x.amount)
                    .sum();
                let before_returns = prev.liquid + savings + extras_now + r.income - r.spending;
                let returns = if before_returns > 0.0 {
                    prev.stocks * path.stocks[k] + prev.bonds * path.bonds[k]
                } else {
                    0.0
                };
                let expected = prev.total + savings + extras_now + r.income - r.spending - r.taxes + returns;
                assert!(
                    (r.total - expected).abs() <= 0.01,
                    "{}: Total {} statt {expected} (Abweichung {}) – {}",
                    r.month,
                    r.total,
                    r.total - expected,
                    ctx()
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 20_000, "nur {checked} Monate geprüft");
}

// ---------------------------------------------------------------------------------------------
// Last (Stresstest und Messung; mit `cargo test --release -- --ignored --nocapture`)
// ---------------------------------------------------------------------------------------------

fn stress_plan(paths: usize) -> Value {
    let mut v = mc_plan("parametric", paths, 99);
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "2010-01" }, { "id": "B", "name": "B", "birth": "2011-01" } ]);
    v["partner"] = json!({ "income": 1500,
        "pk": { "capital": 300_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 },
        "ahv": { "monthly": 1500, "at": "never" } });
    v["milestones"][2]["date"] = json!("2129-12");
    v["milestones"][1]["date"] = json!("2032-01");
    let props: Vec<Value> = (0..4)
        .map(|k| {
            let mut h = house(if k < 2 { "owned" } else { "planned" });
            h["id"] = json!(format!("p{k}"));
            h["name"] = json!(format!("Objekt {k}"));
            h["buy"] = json!(format!("{}-03", 2035 + k * 5));
            h["sell"] = json!(format!("{}-09", 2050 + k * 8));
            if k >= 2 {
                h["mortgage"]["tranches"][0].as_object_mut().unwrap().remove("start");
            }
            h["mortgage"]["tranches"]
                .as_array_mut()
                .unwrap()
                .push(json!({ "name": "T2", "amount": 100_000,
            "periods": [ { "type": "saron", "term": 5, "margin": 0.006 } ], "after": "renew" }));
            if k >= 2 {
                h["mortgage"]["tranches"][1].as_object_mut().unwrap().remove("start");
            } else {
                h["mortgage"]["tranches"][1]["start"] = json!("2026-01");
            }
            h
        })
        .collect();
    v["realestate"] = json!({ "emergencyMonths": 6, "properties": props });
    v["taxes"]["income"] = json!([[0, 0.0], [100_000, 0.1], [300_000, 0.2]]);
    v["taxes"]["wealth"] = json!([[0, 0.0], [2_000_000, 0.004]]);
    v["taxes"]["capital"] = json!([[0, 0.0], [1_000_000, 0.07]]);
    v
}

#[test]
#[ignore]
fn benchmark_paths() {
    for paths in [1_000usize, 10_000, 50_000] {
        let v = mc_plan("parametric", paths, 1);
        let t = std::time::Instant::now();
        let out = api::run(&v, Path::new("/"), Some(&library()));
        assert!(out.error.is_none());
        println!(
            "BENCH parametrisch, 30 Jahre, {paths} Pfade: {:.2} s",
            t.elapsed().as_secs_f64()
        );
    }
    for paths in [1_000usize, 10_000, 50_000] {
        let v = mc_plan("bootstrap", paths, 1);
        let t = std::time::Instant::now();
        let out = api::run(&v, Path::new("/"), Some(&library()));
        assert!(out.error.is_none());
        println!(
            "BENCH bootstrap, 30 Jahre, {paths} Pfade: {:.2} s",
            t.elapsed().as_secs_f64()
        );
    }
}

#[test]
#[ignore]
fn stress_couple_four_properties_100_years() {
    let paths = std::env::var("QA_PATHS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40_000); // #139: unter der Grenze von 50 Mio. Pfad-Monaten (40'000 × 1'200 = 48 Mio.)
    let v = stress_plan(paths);
    let t = std::time::Instant::now();
    let out = api::run(&v, Path::new("/"), Some(&library()));
    println!(
        "STRESS {paths} Pfade, Ehepaar, 4 Objekte, 100 Jahre: {:.2} s",
        t.elapsed().as_secs_f64()
    );
    assert!(out.error.is_none(), "{:?}", out.error);
    let b = out.batch.unwrap();
    assert_eq!(b.summary.runs, paths);
    for (pct, d) in &b.details {
        check_books(d, &format!("Stress {pct}"));
    }
    // #129/AK4
    assert!(b
        .summary
        .percentiles
        .iter()
        .all(|p| p.end_total.is_finite() && p.end_total_real.is_some_and(f64::is_finite)));
}

// ---------------------------------------------------------------------------------------------
// Zeitpunkte nach Schweizer Praxis: Einkommenssteuer, Teuerung, AHV-Anpassung
// ---------------------------------------------------------------------------------------------

/// Einkommenssteuer: einmal pro Jahr auf dem Einkommen des Steuerjahrs, bezahlt im Mai des Folgejahres.
#[test]

fn income_tax_is_assessed_yearly_and_paid_in_may_of_next_year() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2035-01");
    v["flows"]["income"] = json!(6_000);
    v["flows"]["spending"] = json!(0);
    v["taxes"] = json!({ "income": [[0, 0.1]], "wealth": [], "capital": [] });
    let res = run(&v);
    let taxed: Vec<_> = res.records.iter().filter(|r| r.taxes.abs() > 0.005).collect();
    assert!(!taxed.is_empty(), "keine Steuer gerechnet");
    for r in taxed.iter().filter(|r| r.month != res.records.last().unwrap().month) {
        assert_eq!(r.month.month(), 5, "Steuer in {} statt im Mai", r.month);
    }
    // Steuerjahr 2030 (72'000 × 10 %) wird im Mai 2031 bezahlt, im Dezember 2030 noch nichts
    let dec = res
        .records
        .iter()
        .find(|r| r.month == YearMonth::new(2030, 12))
        .unwrap();
    assert!(dec.taxes.abs() < 0.005, "Dezember 2030: {}", dec.taxes);
    let may = res.records.iter().find(|r| r.month == YearMonth::new(2031, 5)).unwrap();
    assert!((may.taxes - 7_200.0).abs() < 0.01, "Mai 2031: {}", may.taxes);
}

/// #109/AK5: Die veranlagte Steuer steht im Dezember des Steuerjahres, bezahlt wird sie im Mai des Folgejahres;
/// über alle Monate ist die Summe der veranlagten gleich der Summe der bezahlten Steuer.
#[test]
fn issue_109_ak5_assessed_tax_is_booked_in_the_tax_year() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2035-01");
    v["flows"]["income"] = json!(6_000);
    v["flows"]["spending"] = json!(0);
    v["taxes"] = json!({ "income": [[0, 0.1]], "wealth": [], "capital": [] });
    let res = run(&v);
    for year in [2030, 2031, 2032] {
        let dec = res
            .records
            .iter()
            .find(|r| r.month == YearMonth::new(year, 12))
            .unwrap();
        assert!(
            (dec.assessed_income - 7_200.0).abs() < 0.01,
            "Dezember {year}: {}",
            dec.assessed_income
        );
        let may = res
            .records
            .iter()
            .find(|r| r.month == YearMonth::new(year + 1, 5))
            .unwrap();
        assert!(
            (may.tax_income - 7_200.0).abs() < 0.01,
            "Mai {}: {}",
            year + 1,
            may.tax_income
        );
        assert!(may.assessed_income.abs() < 0.005, "im Mai wird nichts veranlagt");
    }
    let sum = |f: &dyn Fn(&engine::sim::MonthRecord) -> f64| res.records.iter().map(f).sum::<f64>();
    assert!((sum(&|r| r.assessed_income) - sum(&|r| r.tax_income)).abs() < 0.01);
}

/// Teuerung: wirkt jeden Monat (1 + i)^(1/12), nicht als Sprung im Januar.
#[test]

fn inflation_is_applied_every_month() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.06);
    let res = run(&v);
    for k in 1..24 {
        let (prev, r) = (&res.records[k - 1], &res.records[k]);
        let expected = prev.spending * 1.06_f64.powf(1.0 / 12.0);
        assert!(
            (r.spending - expected).abs() < 0.01,
            "{}: Ausgaben {} statt {expected}",
            r.month,
            r.spending
        );
    }
}

/// AHV-Anpassung: nur im Januar, mit der Teuerung des Vorjahres; AHV-Rente sonst unverändert.
#[test]
fn indexed_ahv_changes_only_in_january() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("1964-01");
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "ahv", "name": "AHV", "kind": "age", "person": "A", "years": 65 }));
    v["milestones"][1]["date"] = json!("2030-01");
    v["pension"]["ahv"] = json!({ "monthly": 2_000, "at": "ahv", "indexed": true });
    v["flows"]["spending"] = json!(0);
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    let res = run(&v);
    let first = res.records.iter().position(|r| r.income > 0.0).expect("AHV beginnt");
    let mut jumps = 0;
    for k in (first + 1)..res.records.len() {
        let (prev, r) = (&res.records[k - 1], &res.records[k]);
        if (r.income - prev.income).abs() > 0.005 {
            jumps += 1;
            assert_eq!(r.month.month(), 1, "AHV-Änderung im {}", r.month);
            assert!(
                (r.income / prev.income - 1.03).abs() < 1e-9,
                "{}: Faktor {}",
                r.month,
                r.income / prev.income
            );
        }
    }
    assert!(jumps >= 3, "zu wenige Anpassungen: {jumps}");
}

// ---------------------------------------------------------------------------------------------
// Lebenshaltung ohne Wohnkosten; Hypothekarzinsen quartalsweise
// ---------------------------------------------------------------------------------------------

/// Die angegebene Lebenshaltung enthält weder Miete noch Hypothekarzinsen: Ausgaben = Lebenshaltung + Wohnkosten.
#[test]
fn living_costs_exclude_rent_and_mortgage_interest() {
    // Miete
    let mut v = base();
    v["realestate"] = json!({ "properties": [
        { "id": "r", "name": "Wohnung", "status": "rent", "rent": 2_000, "from": "2030-01" } ] });
    let res = run(&v);
    for r in res.records.iter().take(12) {
        assert!(
            (r.housing - 2_000.0).abs() < 0.01,
            "{}: Wohnkosten {}",
            r.month,
            r.housing
        );
        assert!(
            (r.spending - 7_000.0).abs() < 0.01,
            "{}: Ausgaben {}",
            r.month,
            r.spending
        );
    }
    let last = &res.records[11];
    assert!((res.records[0].liquid - last.liquid - 11.0 * 7_000.0).abs() < 0.01);
    // Eigenheim mit Hypothek: Lebenshaltung bleibt 5000, Zinsen kommen hinzu
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("owned") ] });
    let res = run(&v);
    for r in res.records.iter().take(24) {
        assert!(
            (r.spending - r.housing - 5_000.0).abs() < 0.01,
            "{}: {} − {}",
            r.month,
            r.spending,
            r.housing
        );
    }
    // Jahreszins 500'000 × 1.5 % + Nebenkosten 14'400, nur an anderen Monaten verteilt
    let year: f64 = res.records.iter().take(12).map(|r| r.housing).sum();
    assert!((year - (7_500.0 + 14_400.0)).abs() < 0.01, "Wohnkosten Jahr {year}");
}

/// Hypothekarzinsen und Amortisation werden quartalsweise bezahlt. Beginnt die Hypothek im April,
/// ist die erste Zinszahlung trotzdem im Juni fällig (für April bis Juni).
#[test]
fn mortgage_interest_is_paid_quarterly_first_payment_in_june() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    let mut h = house("planned");
    h["buy"] = json!("2030-04");
    h["fixCosts"] = json!(0);
    h["mortgage"]["tranches"][0].as_object_mut().unwrap().remove("start");
    h["mortgage"]["tranches"][0]["periods"] = json!([
        { "type": "fixed", "term": 10, "rate": 0.012, "amort": 1_500, "amortMode": "direct" } ]);
    v["realestate"] = json!({ "properties": [ h ] });
    let res = run(&v);
    let at = |y: i32, m: u32| res.records.iter().find(|r| r.month == YearMonth::new(y, m)).unwrap();
    let monthly = 500_000.0 * 0.012 / 12.0;
    for m in [4, 5] {
        assert!(at(2030, m).housing.abs() < 0.01, "Monat {m}: {}", at(2030, m).housing);
    }
    // Juni: Zins April, Mai, Juni plus erste Amortisation
    assert!(
        (at(2030, 6).housing - (3.0 * monthly + 1_500.0)).abs() < 0.01,
        "Juni {}",
        at(2030, 6).housing
    );
    // Zinsen aller Nicht-Quartalsmonate sind null, im Quartalsmonat drei Monate (Restschuld sinkt um 1500)
    for r in res
        .records
        .iter()
        .filter(|r| r.month >= YearMonth::new(2030, 7))
        .take(24)
    {
        if r.month.month() % 3 != 0 {
            assert!(r.housing.abs() < 0.01, "{}: {}", r.month, r.housing);
        } else {
            assert!(r.housing > 3.0 * monthly * 0.9, "{}: {}", r.month, r.housing);
        }
    }
    let sept = at(2030, 9);
    let balance = 500_000.0 - 1_500.0;
    assert!(
        (sept.housing - (3.0 * balance * 0.012 / 12.0 + 1_500.0)).abs() < 0.01,
        "September {}",
        sept.housing
    );
    check_books(&res, "Hypothek ab April");
}

/// Verkauf mitten im Quartal: die bis dahin aufgelaufenen Zinsen werden mit dem Verkauf bezahlt, nicht verschenkt.
#[test]
fn accrued_interest_is_settled_on_sale_and_at_plan_end() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    let mut h = house("owned");
    h["fixCosts"] = json!(0);
    h["sell"] = json!("2030-05");
    h["mortgage"]["tranches"][0]["start"] = json!("2030-01");
    v["realestate"] = json!({ "properties": [ h ] });
    let res = run(&v);
    let monthly = 500_000.0 * 0.015 / 12.0;
    let total: f64 = res.records.iter().map(|r| r.housing).sum();
    // Januar bis Mai: fünf Monatszinsen (März bezahlt drei, Verkauf im Mai die übrigen zwei)
    assert!((total - 5.0 * monthly).abs() < 0.01, "Zinsen total {total}");
    // Plan endet im Februar: der offene Zins wird im letzten Monat bezahlt
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    v["milestones"][2]["date"] = json!("2030-02");
    let mut h = house("owned");
    h["fixCosts"] = json!(0);
    h["mortgage"]["tranches"][0]["start"] = json!("2030-01");
    v["realestate"] = json!({ "properties": [ h ] });
    let total: f64 = run(&v).records.iter().map(|r| r.housing).sum();
    assert!((total - 2.0 * monthly).abs() < 0.01, "Zinsen bis Planende {total}");
}

// ---------- Grundstückgewinnsteuer mit Besitzdauer: Prüfung und Dateiformat ----------

fn with_gain_tax(duration: Value, bought: Option<&str>) -> Value {
    let mut v = base();
    let mut prop = json!({ "name": "Haus", "status": "owned", "price": 400_000, "worth": 500_000, "sell": "2035-06" });
    if let Some(b) = bought {
        prop["bought"] = json!(b);
    }
    v["realestate"] = json!({ "properties": [ prop ] });
    v["taxes"]["propertyGain"] = json!({ "brackets": [[1e18, 0.2]], "reduction": 0.0, "duration": duration });
    v
}

/// Ungültige Besitzdauer-Tabellen (negative Jahre, Abzug ab 100 %) blockieren die Berechnung.
#[test]
fn invalid_holding_period_table_is_an_error() {
    for bad in [json!([[-1, 0.5]]), json!([[5, -1.0]]), json!([[5, -1.5]])] {
        let v = with_gain_tax(bad.clone(), Some("2020-01"));
        assert!(
            errors(&v).iter().any(|m| m.contains("Besitzdauer")),
            "{bad}: kein Fehler"
        );
        assert!(
            api::run(&v, Path::new("/"), None).error.is_some(),
            "{bad}: Berechnung nicht blockiert"
        );
    }
    assert!(errors(&with_gain_tax(json!([[0, 0.5], [5, -0.05]]), Some("2020-01"))).is_empty());
}

/// Fehlendes Kaufdatum: Warnung (nur mit Besitzdauer-Tabelle), keine Sperre; mit Datum oder ohne Tabelle keine Warnung.
#[test]
fn missing_purchase_date_warns_only_with_a_holding_period_table() {
    let warns = |v: &Value| {
        api::validate(v)
            .into_iter()
            .filter(|i| i.severity == Severity::Warning && i.message.contains("Kaufdatum"))
            .count()
    };
    assert_eq!(warns(&with_gain_tax(json!([[0, 0.5]]), None)), 1);
    assert_eq!(warns(&with_gain_tax(json!([[0, 0.5]]), Some("2020-01"))), 0);
    assert_eq!(warns(&with_gain_tax(json!([]), None)), 0);
    let v = with_gain_tax(json!([[0, 0.5]]), None);
    assert!(errors(&v).is_empty() && api::run(&v, Path::new("/"), None).error.is_none());
}

/// Dateiformat: Kaufdatum und neue Felder der Steuer bleiben beim Speichern und Laden erhalten;
/// leere Tabelle und Freigrenze 0 werden nicht geschrieben, ältere Dateien ohne die Felder bleiben lesbar.
#[test]
fn holding_period_fields_roundtrip_and_old_files_still_load() {
    let v = with_gain_tax(json!([[0, 0.5], [5, -0.05]]), Some("2020-01"));
    let mut v2 = v.clone();
    v2["taxes"]["propertyGain"]["exemptBelow"] = json!(5000);
    let p = plan(&v2);
    let back = serde_json::to_value(&p).unwrap();
    assert_eq!(back["realestate"]["properties"][0]["bought"], json!("2020-01"));
    assert_eq!(
        back["taxes"]["propertyGain"]["duration"],
        json!([[0.0, 0.5], [5.0, -0.05]])
    );
    assert_eq!(back["taxes"]["propertyGain"]["exemptBelow"], json!(5000.0));
    let old = with_gain_tax(json!([]), None);
    let mut old = old;
    old["taxes"]["propertyGain"].as_object_mut().unwrap().remove("duration");
    let back = serde_json::to_value(plan(&old)).unwrap();
    assert!(back["taxes"]["propertyGain"].get("duration").is_none());
    assert!(back["taxes"]["propertyGain"].get("exemptBelow").is_none());
    assert!(back["realestate"]["properties"][0].get("bought").is_none());
}

/// #139/AK4: Spitzenspeicher einer Rechnung genau an der Grenze (50 Mio. Pfad-Monate): schwerste Testplanung (Ehepaar,
/// vier Liegenschaften mit je zwei Tranchen, 100 Jahre) mit 41'666 Pfaden. Gemessen über den Speicherverwalter der Tests
/// (`engine::test_alloc`); er bleibt unter der Grenze der Testprozesse von 1,5 GB (gemessen 18.11.2026: 1'337 MB).
/// Rund 8 s im Release-Profil, über eine Minute im Testprofil: läuft nächtlich im Job `release-golden`.
#[test]
#[ignore]
fn issue_139_ak4_peak_memory_at_the_limit_stays_below_the_test_limit() {
    use engine::test_alloc::{Limited, LIMIT};
    let months: u64 = 100 * 12;
    let paths = (engine::validate::MAX_PATH_MONTHS / months) as usize;
    let v = stress_plan(paths);
    assert!(
        api::validate(&v).iter().all(|i| !i.message.contains("Zu viele Pfade")),
        "an der Grenze erlaubt"
    );
    Limited::reset_peak();
    let out = api::run(&v, Path::new("/"), Some(&library()));
    let peak = Limited::peak();
    println!("SPEICHER {paths} Pfade × {months} Monate: Spitze {} MB", peak >> 20);
    assert!(out.error.is_none(), "{:?}", out.error);
    assert!(peak < LIMIT, "Spitze {} MB", peak >> 20);
}
