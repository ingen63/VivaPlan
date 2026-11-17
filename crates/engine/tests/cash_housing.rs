//! #53 (Lebenshaltung inklusive Wohnkosten) und #54 (Cash getrennt vom Portfolio, Strategien ab Pensionierung).
//!
//! Erfundene Planung: Geburt 1980-01, Start 2040-01, Bezug Pensionskasse und AHV-Bezug mit 65 (2045-01), keine
//! Renditen, keine Teuerung (ausser wo vermerkt), keine Steuern.

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

/// «Ausgaben total» eines Jahres wie in den Jahreswerten: geplante Lebenshaltung + Wohnkosten + 3a, Monatsdurchschnitt.
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
        .map(|i| i.message)
        .collect()
}

/// Haus: Nebenkosten 500, Hypothek 600'000 zu 2 % (Zins 1'000), Amortisation 1'500 pro Quartal (500 pro Monat),
/// indirekt, damit die Schuld und damit der Zins gleich bleiben. Erneuerung 2042-01 zu 2.6 % (Zins 1'300).
fn house(sell: &str) -> Value {
    json!({ "id": "h", "name": "Haus", "status": "owned", "price": 1_000_000, "worth": 1_000_000, "fixCosts": 500, "sell": sell,
        "mortgage": { "tranches": [ { "name": "T", "amount": 600_000, "start": "2037-01", "periods": [
            { "type": "fixed", "term": 5, "rate": 0.02, "amort": 1500, "amortMode": "indirect" },
            { "type": "fixed", "term": 10, "rate": 0.026, "amort": 1500, "amortMode": "indirect" } ] } ] } })
}

/// #53/AK3: Eingabe 8'000 inklusive Wohnkosten 2'000: Ausgaben total 8'000; Zins +300 ergibt 8'300; Miete 2'500
/// nach dem Verkauf ergibt 8'500.
#[test]
fn issue_53_ak3_real_housing_costs_replace_the_share() {
    let mut v = base();
    v["flows"]["income"] = json!(20_000); // tragbar bei der Erneuerung
    v["realestate"] = json!({ "properties": [ house("2043-01"),
        { "name": "Miete", "status": "rent", "from": "afterSale", "rent": 2500 } ] });
    let r = run(&v);
    close(rec(&r, 2040, 1).planned_spending, 6000.0, "Lebenshaltung ohne Wohnen");
    close(spending_total(&r, 2041), 8000.0, "2041");
    close(spending_total(&r, 2042), 8300.0, "nach der Erneuerung");
    close(spending_total(&r, 2044), 8500.0, "mit Miete");
}

/// #53/AK4, AK5: Wohnanteil grösser als eine Zeile ist ein Fehler; ohne Wohnkosten ändert sich nichts.
#[test]
fn issue_53_ak4_ak5_check_and_no_housing() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("never") ] });
    v["flows"]["spending"] = json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "2044-01", "value": 1999 } ] });
    let e = errors(&v);
    assert_eq!(e, vec!["Die Lebenshaltungskosten (1'999) sind kleiner als die Wohnkosten zum Simulationsstart (2'000), die sie enthalten.".to_string()]);
    let i = engine::api::validate(&v)
        .into_iter()
        .find(|i| i.message.contains("1'999"))
        .unwrap();
    assert_eq!(i.field.as_deref(), Some("flows.spending.rows.1.value"));
    v["flows"]["spending"]["rows"][1]["value"] = json!(2000);
    assert!(errors(&v).is_empty(), "genau der Wohnanteil ist gültig");
    // AK2: gleicher Abzug für jede Zeile
    let r = run(&v);
    close(rec(&r, 2044, 1).planned_spending, 0.0, "2'000 − 2'000");
    // AK5: ohne Immobilie und Miete
    let r = run(&base());
    close(rec(&r, 2040, 1).planned_spending, 8000.0, "ohne Wohnkosten");
}

/// #53/AK6: Die 3a-Einzahlung ist nicht in der Eingabe enthalten und kommt dazu.
#[test]
fn issue_53_ak6_pillar3a_stays_separate() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("never") ] });
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 1200 }]);
    let r = run(&v);
    close(spending_total(&r, 2041), 8100.0, "8'000 + 100 Säule 3a");
}

/// #53/AK7: Ältere Planungen werden beim Öffnen umgerechnet; das Ergebnis bleibt gleich.
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
            .any(|n| n.contains("Die Wohnkosten zum Simulationsstart (2000 pro Monat) sind zu jedem Wert addiert")),
        "{:?}",
        out.notes
    );
    let (a, b) = (run(&old), run(&out.plan));
    for (x, y) in a.records.iter().zip(&b.records) {
        close(x.total, y.total, &x.month.to_string());
    }
    // Zweites Öffnen: nichts mehr zu tun
    assert!(engine::api::upgrade_plan(&out.plan).notes.is_empty());
}

/// #54/AK8: Eine ältere Planung bekommt beim Öffnen den Hinweis zur Cash-Regel, auch ohne Wohnkosten; einmal.
#[test]
fn issue_54_ak8_note_for_older_plans() {
    let mut old = base();
    old["flows"] = json!({ "income": 8000, "spending": 6000 });
    let out = engine::api::upgrade_plan(&old);
    assert_eq!(out.notes, vec![engine::upgrade::CASH_NOTE.to_string()]);
    assert_eq!(out.plan["flows"]["spending"], 6000, "ohne Wohnkosten bleibt der Wert");
    assert!(engine::api::upgrade_plan(&out.plan).notes.is_empty());
    assert!(
        engine::api::upgrade_plan(&base()).notes.is_empty(),
        "neue Planung ohne Hinweis"
    );
}

/// #53/AK8: Der Ausgaben-Faktor der Variation gilt für die Lebenshaltung ohne Wohnen.
#[test]
fn issue_53_ak8_spending_factor_on_living_without_housing() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("never") ] });
    v["flows"]["spendingFactor"] = json!(0.5);
    let r = run(&v);
    close(rec(&r, 2040, 1).planned_spending, 3000.0, "0.5 × (8'000 − 2'000)");
}

/// #54/AK2: Einnahmen gleich Ausgaben: bis zur Pensionierung bleiben die Bestände wie erfasst.
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

/// #54/AK3: Überschuss 2'000 pro Monat geht ins Portfolio (1'600 Aktien, 400 Obligationen), Cash bleibt.
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

/// #54: Fehlbetrag wird aus dem Portfolio nachgefüllt; der Cash-Betrag folgt der Teuerung.
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

/// #54/AK4, AK5: Die Cash-Reserve von «Standard» wirkt erst ab der Pensionierung.
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
