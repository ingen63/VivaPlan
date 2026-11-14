//! Entnahmestrategie bei Mittelzuflüssen: Zuflüsse (einmalige Beträge, PK-Auszahlung, Verkaufserlöse) wenden die
//! Strategie im Zuflussmonat sofort an, nicht erst im Januar. Ein Verkaufserlös, für den im Folgemonat ein Kauf
//! geplant ist, wird reserviert; die Strategie folgt dann nach dem Kauf.
//!
//! Strategie «Fixe Rate» 4 %: Entnahme pro Monat = 4 % des liquiden Vermögens / 12 (ohne Renditen, Steuern, Teuerung).

use engine::batch;
use engine::model::Plan;
use engine::sim::{EventKind, PropertyEventKind, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Zufluss" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2032-12" }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "never" }
        },
        "flows": { "income": 0, "spending": 0 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 1, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [], "reduction": 0.0 } },
        "withdrawal": { "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } }
    })
}

fn run(v: Value) -> SimResult {
    let p: Plan = serde_json::from_value(crate::any_age(v)).expect("Planung gültig");
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

fn rec(res: &SimResult, y: i32, m: u32) -> &engine::sim::MonthRecord {
    res.records
        .iter()
        .find(|r| r.month == YearMonth::new(y, m))
        .expect("Monat")
}

/// Entnahme dieses Monats (monatlicher Anteil der jährlichen Entnahme).
fn spent(res: &SimResult, y: i32, m: u32) -> f64 {
    rec(res, y, m).spending
}

fn applied(res: &SimResult, y: i32, m: u32) -> Vec<&str> {
    res.events
        .iter()
        .filter(|e| e.month == YearMonth::new(y, m) && e.text.starts_with("Entnahmestrategie angewendet"))
        .map(|e| e.text.as_str())
        .collect()
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} statt {b}");
}

fn house(sell: &str) -> Value {
    json!({ "id": "h", "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": sell, "fixCosts": 0 })
}

fn flat(buy: Value) -> Value {
    json!({ "id": "w", "name": "Wohnung", "status": "planned", "buy": buy, "price": 600_000, "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 300_000, "periods": [ { "type": "fixed", "term": 10, "rate": 0.0 } ] } ] } })
}

#[test]
fn pk_payout_in_january_is_unchanged() {
    // Pensionierung = Start im Januar: die Strategie beginnt mit dem Kapitalbezug, keine zusätzliche Anwendung.
    let mut v = base();
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 });
    let res = run(v);
    close(spent(&res, 2030, 1), 0.04 * 1_500_000.0 / 12.0, "Entnahme Januar");
    assert!(applied(&res, 2030, 1).is_empty());
    // Februar: unverändert 4 % des Vermögens vom Januar-Budget
    close(spent(&res, 2030, 2), 0.04 * 1_500_000.0 / 12.0, "Entnahme Februar");
}

#[test]
fn pk_payout_in_july_starts_the_strategy_with_the_lump_sum() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-07");
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 });
    let res = run(v);
    // Die Strategie beginnt im Juli (Pensionierung) und enthält den Kapitalbezug sofort
    close(spent(&res, 2030, 7), 0.04 * 1_500_000.0 / 12.0, "Entnahme Juli");
    assert!(res.events.iter().any(|e| e.month == YearMonth::new(2030, 7)
        && e.kind == EventKind::Withdrawal
        && e.text.starts_with("Entnahme ")));
    assert!(
        applied(&res, 2030, 7).is_empty(),
        "kein zweiter Eintrag für den Beginn der Strategie"
    );
}

#[test]
fn external_inflow_mid_year_applies_the_strategy_immediately() {
    let mut v = base();
    v["flows"]["extras"] = json!([ { "date": "2030-07", "amount": 600_000, "label": "Erbschaft" } ]);
    let res = run(v);
    let liquid_june = rec(&res, 2030, 6).liquid;
    close(spent(&res, 2030, 6), 0.04 * 1_000_000.0 / 12.0, "vor dem Zufluss");
    close(
        spent(&res, 2030, 7),
        0.04 * (liquid_june + 600_000.0) / 12.0,
        "Zuflussmonat: neues Budget",
    );
    close(spent(&res, 2030, 8), spent(&res, 2030, 7), "bleibt bis Januar");
    let ev = applied(&res, 2030, 7);
    assert_eq!(ev.len(), 1, "{ev:?}");
    assert!(ev[0].contains("Erbschaft"), "{}", ev[0]);
}

#[test]
fn two_inflows_in_the_same_month_apply_the_strategy_once_after_both() {
    let mut v = base();
    v["flows"]["extras"] = json!([
        { "date": "2030-07", "amount": 200_000, "label": "Erbschaft" },
        { "date": "2030-07", "amount": 100_000, "label": "Schenkung" }
    ]);
    let res = run(v);
    let liquid_june = rec(&res, 2030, 6).liquid;
    close(
        spent(&res, 2030, 7),
        0.04 * (liquid_june + 300_000.0) / 12.0,
        "Budget nach beiden Zuflüssen",
    );
    let ev = applied(&res, 2030, 7);
    assert_eq!(ev.len(), 1, "{ev:?}");
    assert!(ev[0].contains("Erbschaft") && ev[0].contains("Schenkung"), "{}", ev[0]);
}

#[test]
fn sale_in_june_without_follow_up_purchase_applies_the_strategy_in_june() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("2030-06") ] });
    let res = run(v);
    let liquid_may = rec(&res, 2030, 5).liquid;
    close(
        spent(&res, 2030, 6),
        0.04 * (liquid_may + 800_000.0) / 12.0,
        "Juni: Strategie auf den Verkaufserlös",
    );
    assert_eq!(applied(&res, 2030, 6).len(), 1);
    let sale = &res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .unwrap();
    assert!(
        sale.note.as_deref().unwrap().contains("kein Folgekauf"),
        "{:?}",
        sale.note
    );
}

#[test]
fn sale_in_june_with_purchase_in_july_reserves_the_proceeds_and_applies_after_the_purchase() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("2030-06"), flat(json!("2030-07")) ] });
    let res = run(v);
    // Juni: keine Anwendung, Erlös reserviert (Entnahme bleibt beim alten Budget)
    assert!(applied(&res, 2030, 6).is_empty());
    close(spent(&res, 2030, 6), 0.04 * 1_000_000.0 / 12.0, "Juni: altes Budget");
    let sale = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .unwrap();
    assert!(sale.note.as_deref().unwrap().contains("reserviert"), "{:?}", sale.note);
    assert!(res
        .events
        .iter()
        .any(|e| e.month == YearMonth::new(2030, 6) && e.text.contains("reserviert")));
    // Juli: zuerst der Kauf (Eigenmittel 300'000), danach die Strategie auf das verbleibende liquide Vermögen
    let buy = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Purchase)
        .unwrap();
    assert_eq!(buy.month, YearMonth::new(2030, 7));
    let liquid_june = rec(&res, 2030, 6).liquid;
    close(
        spent(&res, 2030, 7),
        0.04 * (liquid_june - buy.equity.unwrap()) / 12.0,
        "Juli: Budget nach dem Kauf",
    );
    let ev = applied(&res, 2030, 7);
    assert_eq!(ev.len(), 1);
    assert!(ev[0].contains("Überschuss aus dem Verkaufserlös"), "{}", ev[0]);
}

#[test]
fn sale_in_june_with_purchase_in_september_is_no_follow_up_purchase() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("2030-06"), flat(json!("2030-09")) ] });
    let res = run(v);
    let liquid_may = rec(&res, 2030, 5).liquid;
    close(
        spent(&res, 2030, 6),
        0.04 * (liquid_may + 800_000.0) / 12.0,
        "Juni: Strategie angewendet",
    );
    assert_eq!(applied(&res, 2030, 6).len(), 1);
    let sale = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .unwrap();
    assert!(sale.note.as_deref().unwrap().contains("kein Folgekauf"));
}

#[test]
fn sale_in_december_with_purchase_in_january_buys_first_and_applies_once() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house("2030-12"), flat(json!("2031-01")) ] });
    let res = run(v);
    assert!(applied(&res, 2030, 12).is_empty());
    let sale = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .unwrap();
    assert!(sale.note.as_deref().unwrap().contains("reserviert"));
    let buy = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Purchase)
        .unwrap();
    assert_eq!(buy.month, YearMonth::new(2031, 1));
    // Januar: ein Budget, berechnet nach dem Kauf (nicht doppelt, kein zusätzlicher Eintrag)
    let liquid_dec = rec(&res, 2030, 12).liquid;
    close(
        spent(&res, 2031, 1),
        0.04 * (liquid_dec - buy.equity.unwrap()) / 12.0,
        "Januar: Budget nach dem Kauf",
    );
    assert!(
        applied(&res, 2031, 1).is_empty(),
        "im Januar wendet die Strategie ohnehin an"
    );
    close(spent(&res, 2031, 2), spent(&res, 2031, 1), "Budget bleibt");
}

#[test]
fn three_bucket_strategy_books_an_inflow_into_pot_3_only() {
    let mut v = base();
    v["withdrawal"] =
        json!({ "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 6, "refillThreshold": -1 } });
    v["flows"]["spending"] = json!(3_000);
    v["flows"]["extras"] = json!([ { "date": "2030-07", "amount": 900_000, "label": "Erbschaft" } ]);
    let res = run(v);
    let june = rec(&res, 2030, 6);
    let july = rec(&res, 2030, 7);
    // Sonderfall B: Der Zufluss geht ganz in Topf 3, Topf 1 und 2 werden nicht aufgefüllt
    assert!(july.cash <= june.cash, "Cash {} nach dem Zufluss", july.cash);
    assert!((july.bonds - june.bonds).abs() < 1e-6);
    assert!(july.stocks >= june.stocks + 900_000.0 - 1.0, "Zufluss in Aktien");
    let b: Vec<_> = res
        .bucket_refills
        .iter()
        .filter(|r| r.month == YearMonth::new(2030, 7))
        .collect();
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].trigger, "B");
    assert_eq!((b[0].moves[0].from, b[0].moves[0].to), (0, 3));
}

fn three_bucket_plan() -> Value {
    let mut v = base();
    v["withdrawal"] =
        json!({ "strategy": "three_buckets", "buckets": { "cashYears": 1, "bondYears": 1, "refillThreshold": -1 } });
    v["flows"]["spending"] = json!(2_000);
    v["wealth"] = json!({ "stocks": 100_000, "bonds": 0, "cash": 100_000, "stockShare": 1, "reserveMonths": 0 });
    v
}

#[test]
fn three_bucket_sale_in_december_and_purchase_in_january_reserves_buys_first_and_sends_the_surplus_to_pot_3() {
    let mut v = three_bucket_plan();
    v["realestate"] = json!({ "properties": [ house("2030-12"), flat(json!("2031-01")) ] });
    let res = run(v);
    let sale = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Sale)
        .unwrap();
    assert!(sale.note.as_deref().unwrap().contains("reserviert"));
    let buy = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Purchase)
        .unwrap();
    assert_eq!(buy.month, YearMonth::new(2031, 1));
    // Dezember: Der Erlös geht direkt in die Reservierung (extern → Reservierung), nicht in einen Topf; Topf 1 bleibt unverändert
    let dec: Vec<_> = res
        .bucket_refills
        .iter()
        .filter(|r| r.month == YearMonth::new(2030, 12) && r.trigger == "B")
        .collect();
    assert_eq!(dec.len(), 1);
    assert_eq!((dec[0].moves[0].from, dec[0].moves[0].to), (0, 4));
    assert_eq!(dec[0].before, dec[0].after);
    assert!(
        dec[0].before[0] < 800_000.0,
        "Topf 1 enthält den Erlös nicht: {}",
        dec[0].before[0]
    );
    // Januar: Überschuss der Reservierung (800'000 Erlös − 300'000 Eigenmittel = 500'000) in Topf 3
    let b = res
        .bucket_refills
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1) && r.trigger == "B")
        .expect("Überschuss");
    assert_eq!((b.moves[0].from, b.moves[0].to), (4, 3), "Reservierung → Topf 3");
    assert!(
        (b.moves[0].amount - (800_000.0 - buy.equity.unwrap())).abs() < 1.0,
        "{:?}",
        b.moves
    );
    // die Januar-Auffüllung kommt nach dem Kauf: Eigenmittel wurden aus der Reservierung bezahlt, nicht aus Topf 1
    let jan = res
        .bucket_refills
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 1) && r.trigger == "Januar")
        .expect("Januar");
    assert!(jan.after[0] > 0.0);
    assert!(res
        .records
        .iter()
        .all(|r| r.cash >= -1e-6 && r.bonds >= -1e-6 && r.stocks >= -1e-6));
}

#[test]
fn three_bucket_sale_without_follow_up_purchase_goes_into_pot_3_only() {
    let mut v = three_bucket_plan();
    v["realestate"] = json!({ "properties": [ house("2030-06") ] });
    let res = run(v);
    let b = res
        .bucket_refills
        .iter()
        .find(|r| r.month == YearMonth::new(2030, 6) && r.trigger == "B")
        .expect("B");
    assert!((b.moves[0].amount - 800_000.0).abs() < 1.0, "{:?}", b.moves);
    assert_eq!((b.moves[0].from, b.moves[0].to), (0, 3));
}

#[test]
fn three_bucket_purchase_remainder_is_covered_by_c_and_never_burdens_pot_1_beforehand() {
    let mut v = three_bucket_plan();
    v["wealth"] = json!({ "stocks": 1_000_000, "bonds": 0, "cash": 100_000, "stockShare": 1, "reserveMonths": 0 });
    v["realestate"] = json!({ "properties": [ flat(json!("2031-04")) ] });
    let res = run(v);
    let buy = res
        .property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Purchase)
        .expect("Kauf");
    assert_eq!(buy.month, YearMonth::new(2031, 4));
    // Die Eigenmittel (300'000) sind eine einmalige Ausgabe des Monats: C deckt sie, Topf 1 ist nie vorher negativ
    let c = res
        .bucket_refills
        .iter()
        .find(|r| r.month == YearMonth::new(2031, 4) && r.trigger == "C1")
        .expect("C1");
    assert!(
        c.payment >= buy.equity.unwrap(),
        "{} < {}",
        c.payment,
        buy.equity.unwrap()
    );
    for r in &res.bucket_refills {
        assert!(
            r.before[0] >= -1e-6 && r.after[0] >= -1e-6,
            "{} {:?} {:?}",
            r.month.iso(),
            r.before,
            r.after
        );
    }
    assert!(res
        .records
        .iter()
        .all(|r| r.cash >= -1e-6 && r.bonds >= -1e-6 && r.stocks >= -1e-6));
}
