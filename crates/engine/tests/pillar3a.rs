//! FEA-37 (Säule 3a als eigenes Konto) und FEA-36 (indirekte Amortisation nur bis zum AHV-Bezug).
//!
//! Erfundene, von Hand nachrechenbare Planung: Geburt 2000-01 (Alter = Jahr − 2000), Start 2040-01, keine Renditen,
//! keine Teuerung, keine Steuern (ausser wo vermerkt). AHV-Bezug mit 65 Jahren im Januar 2045.

use engine::model::Plan;
use engine::sim::{EventKind, MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "3a" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "ahv" }
        },
        "flows": { "income": 0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

fn ym(y: i32, m: u32) -> YearMonth {
    YearMonth::new(y, m)
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("Planung lesbar");
    let sim = Simulator::new(&plan).expect("Planung gültig");
    let path = engine::market::fixed_path(&sim.fixed, sim.start, sim.months());
    sim.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records
        .iter()
        .find(|x| x.month == ym(y, m))
        .expect("Monat im Zeitraum")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} statt {b}");
}

fn payouts(r: &SimResult) -> Vec<(YearMonth, String)> {
    r.events
        .iter()
        .filter(|e| e.kind == EventKind::Pillar3aPayout)
        .map(|e| (e.month, e.text.clone()))
        .collect()
}

/// FEA-37/AK3: Guthaben 120'000, Plan 2041 und 2042 je 50'000, AHV-Bezug 2045: Rest 20'000 im Monat des AHV-Bezugs.
#[test]
fn fea37_ak3_payout_plan_and_rest_at_ahv() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 120_000, "rate": 0, "payouts": [
        { "date": "2041-01", "amount": 50_000 }, { "date": "2042-01", "amount": 50_000 } ] }]);
    let r = run(&v);
    close(rec(&r, 2040, 12).pillar3a, 120_000.0, "vor der ersten Auszahlung");
    close(rec(&r, 2041, 1).pillar3a, 70_000.0, "nach 2041");
    close(rec(&r, 2042, 1).pillar3a, 20_000.0, "nach 2042");
    close(rec(&r, 2044, 12).pillar3a, 20_000.0, "bis zum AHV-Bezug");
    close(rec(&r, 2045, 1).pillar3a, 0.0, "Rest im AHV-Bezug");
    close(rec(&r, 2047, 12).pillar3a, 0.0, "danach leer");
    let p = payouts(&r);
    assert_eq!(
        p.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2041, 1), ym(2042, 1), ym(2045, 1)]
    );
    assert!(
        p[2].1.contains("Auszahlung des Rests beim AHV-Bezug 20000 CHF"),
        "{}",
        p[2].1
    );
    // Das Geld fliesst ins Vermögen (keine Steuern): Januar 2041 + 50'000 − 1'000 Ausgaben.
    close(
        rec(&r, 2041, 1).liquid - rec(&r, 2040, 12).liquid,
        49_000.0,
        "Zufluss 2041",
    );

    // AHV-Bezug ein Jahr später (2046-01): Plan bleibt gültig, der Rest kommt 2046.
    v["milestones"][2]["years"] = json!(66);
    let r = run(&v);
    assert_eq!(
        payouts(&r).iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2041, 1), ym(2042, 1), ym(2046, 1)]
    );
    close(rec(&r, 2045, 12).pillar3a, 20_000.0, "bis 2046");
}

/// FEA-37/AK5: Ein Betrag grösser als das Guthaben zahlt nur das Guthaben aus, mit Hinweis in den Ereignissen.
#[test]
fn fea37_ak5_amount_larger_than_balance() {
    let mut v = base();
    v["pillar3a"] =
        json!([{ "person": "A", "balance": 30_000, "rate": 0, "payouts": [{ "date": "2041-06", "amount": 50_000 }] }]);
    let r = run(&v);
    close(rec(&r, 2041, 6).pillar3a, 0.0, "leer");
    let p = payouts(&r);
    assert!(
        p.iter()
            .any(|x| x.0 == ym(2041, 6) && x.1.contains("Geplant 50000 CHF, das Guthaben reicht nur für 30000 CHF")),
        "{p:?}"
    );
    assert!(
        p.iter()
            .any(|x| x.0 == ym(2041, 6) && x.1.contains("Auszahlung 30000 CHF")),
        "{p:?}"
    );
    assert!(!p.iter().any(|x| x.0 == ym(2045, 1)), "kein Rest mehr");
    // Ohne Betrag: das ganze Guthaben
    v["pillar3a"][0]["payouts"] = json!([{ "date": "2041-06" }]);
    let r = run(&v);
    close(rec(&r, 2041, 6).pillar3a, 0.0, "ganzes Guthaben");
}

/// FEA-37/AK6: Jede Auszahlung ist ein Kapitalbezug; im selben Jahr wie der PK-Kapitalbezug zusammengerechnet.
#[test]
fn fea37_ak6_capital_tax_with_pk_lump_sum_in_the_same_year() {
    let mut v = base();
    // Satz linear zwischen den Stützstellen: 100'000 → 5 %, 200'000 → 10 %.
    v["taxes"]["capital"] = json!([[0, 0.0], [200_000, 0.10]]);
    v["milestones"][1]["years"] = json!(61); // Bezug Pensionskasse 2041-01
    v["pension"]["pk"] = json!({ "capital": 100_000, "at": "retire", "lumpsum": 1, "conversion": 0 });
    v["pillar3a"] = json!([{ "person": "A", "balance": 100_000, "rate": 0, "payouts": [{ "date": "2041-06" }] }]);
    let r = run(&v);
    close(rec(&r, 2041, 1).tax_capital, 5_000.0, "PK allein: 100'000 zu 5 %");
    close(
        rec(&r, 2041, 6).tax_capital,
        15_000.0,
        "3a mit PK zusammen: 200'000 zu 10 % = 20'000, abzüglich der 5'000 der PK",
    );
}

/// FEA-37/AK8: Die Einzahlung erhöht das Guthaben und zählt zu den Ausgaben (500 pro Monat bei 6'000 pro Jahr).
#[test]
fn fea37_ak8_contribution_counts_as_expense() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 10_000, "rate": 0, "contribution": 6000 }]);
    let with = run(&v);
    v["pillar3a"][0]["contribution"] = json!(0);
    let without = run(&v);
    close(rec(&with, 2040, 12).pillar3a, 16_000.0, "nach einem Jahr");
    for m in 1..=12 {
        let (a, b) = (rec(&with, 2040, m), rec(&without, 2040, m));
        close(a.pillar3a_in, 500.0, "Einzahlung des Monats");
        close(a.spending - b.spending, 500.0, "Ausgaben");
        close(a.withdrawal - b.withdrawal, 500.0, "Entnahme");
    }
    close(
        rec(&without, 2040, 12).liquid - rec(&with, 2040, 12).liquid,
        6000.0,
        "weniger liquides Vermögen",
    );
    close(
        rec(&with, 2040, 12).total,
        rec(&without, 2040, 12).total,
        "Gesamtvermögen gleich",
    );
}

/// FEA-37/AK9: Einzahlungen enden im Monat vor dem AHV-Bezug.
#[test]
fn fea37_ak9_contributions_end_before_ahv() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 1200 }]);
    let r = run(&v);
    close(rec(&r, 2044, 12).pillar3a_in, 100.0, "Dezember 2044");
    close(rec(&r, 2045, 1).pillar3a_in, 0.0, "AHV-Bezug");
    assert!(r
        .records
        .iter()
        .filter(|x| x.month >= ym(2045, 1))
        .all(|x| x.pillar3a_in == 0.0 && x.pillar3a == 0.0));
    // 5 Jahre × 1'200 werden im AHV-Bezug ausbezahlt
    assert!(payouts(&r)
        .iter()
        .any(|x| x.0 == ym(2045, 1) && x.1.contains("6000 CHF")));
}

/// FEA-37: Die Einzahlung gehört zum Netto-Bedarf der 3-Topf-Strategie.
#[test]
fn fea37_contribution_is_part_of_the_net_need() {
    let mut v = base();
    v["milestones"][1]["years"] = json!(60); // Entnahmephase ab Start
    v["withdrawal"] =
        json!({ "strategy": "three_buckets", "buckets": { "cashYears": 1, "bondYears": 1, "refillThreshold": -1 } });
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 2400 }]);
    let r = run(&v);
    let jan = r
        .bucket_refills
        .iter()
        .find(|b| b.month == ym(2041, 1))
        .expect("Januar-Auffüllung");
    close(jan.basis.pillar3a, 2400.0, "3a im Netto-Bedarf");
    close(jan.basis.need(), 12_000.0 + 2400.0, "Netto-Bedarf");
}

fn house(periods: Value, start: &str) -> Value {
    json!({ "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "name": "T", "amount": 400_000, "start": start, "periods": periods } ] } })
}

/// FEA-36/AK1–AK3: Hypothek 400'000, 750 pro Quartal indirekt, AHV-Bezug M = 2045-01.
#[test]
fn fea36_ak1_to_ak3_indirect_until_ahv_then_direct() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    let r = run(&v);
    // Vor M: 3a +750 pro Quartal, Hypothek bleibt 400'000 (Wert 800'000 − Schuld = Eigenkapital).
    close(rec(&r, 2040, 3).pillar3a, 750.0, "März 2040");
    close(
        rec(&r, 2044, 12).pillar3a,
        20.0 * 750.0,
        "20 Quartale bis Dezember 2044",
    );
    close(rec(&r, 2044, 12).property_equity, 400_000.0, "Schuld unverändert");
    // M: Rest ausbezahlt; danach sinkt die Hypothek um 750 pro Quartal, die 3a bleibt leer.
    close(rec(&r, 2045, 1).pillar3a, 0.0, "Rest im AHV-Bezug");
    close(
        rec(&r, 2045, 3).property_equity,
        400_750.0,
        "erste direkte Amortisation",
    );
    close(rec(&r, 2045, 12).property_equity, 403_000.0, "vier Quartale direkt");
    close(rec(&r, 2047, 12).pillar3a, 0.0, "3a bleibt leer");
    assert!(payouts(&r)
        .iter()
        .any(|x| x.0 == ym(2045, 1) && x.1.contains("15000 CHF")));
}

/// FEA-36/AK4: Eine Periode, die erst ab dem AHV-Bezug beginnt (Erneuerung), amortisiert direkt.
#[test]
fn fea36_ak4_period_from_ahv_is_direct() {
    let mut v = base();
    // Erste Periode bis 2045-01 ohne Amortisation, die Erneuerung ab 2045-01 indirekt erfasst.
    v["realestate"] = json!({ "properties": [ house(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 0 },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]), "2040-01") ],
        // Tragbarkeit bei der Erneuerung gegeben (ohne Einkommen sonst Einmalrückzahlung)
        "affordability": { "sustainability": 1, "imputedRate": 0, "fixCosts": 0, "wealthContribution": 0, "maxLoanToValue": 1 } });
    v["pension"]["ahv"]["monthly"] = json!(5000);
    let r = run(&v);
    close(rec(&r, 2045, 3).property_equity, 401_000.0, "direkt ab dem AHV-Bezug");
    assert!(r.records.iter().all(|x| x.pillar3a == 0.0), "nichts in die 3a");
}

fn switches(r: &SimResult) -> Vec<(YearMonth, String)> {
    r.events
        .iter()
        .filter(|e| e.kind == EventKind::AmortizationSwitch)
        .map(|e| (e.month, e.text.clone()))
        .collect()
}

/// FEA-36: Der Wechsel von indirekter auf direkte Amortisation im AHV-Bezug erscheint einmal in den Ereignissen.
#[test]
fn fea36_switch_to_direct_is_an_event() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    let r = run(&v);
    assert_eq!(
        switches(&r),
        vec![(
            ym(2045, 1),
            "Amortisation Hypothek Haus ab AHV-Bezug direkt statt über die Säule 3a: 3000 CHF pro Jahr".to_string()
        )]
    );
    // Erneuerung ab dem AHV-Bezug (vorher indirekt): ebenfalls ein Wechsel.
    v["realestate"]["properties"][0] = house(
        json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 750, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "direct" } ]),
        "2040-01",
    );
    v["realestate"]["affordability"] =
        json!({ "sustainability": 1, "imputedRate": 0, "fixCosts": 0, "wealthContribution": 0, "maxLoanToValue": 1 });
    v["pension"]["ahv"]["monthly"] = json!(5000);
    let r = run(&v);
    assert_eq!(switches(&r).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2045, 1)]);
    assert!(switches(&r)[0].1.contains("4000 CHF pro Jahr"), "{:?}", switches(&r));
    // Kein Wechsel: direkt von Anfang an, indirekt erst ab dem AHV-Bezug (AK4) oder Start nach dem AHV-Bezug.
    v["realestate"]["properties"][0] = house(
        json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "direct" }]),
        "2040-01",
    );
    assert!(switches(&run(&v)).is_empty());
    v["realestate"]["properties"][0] = house(
        json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 0 },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]),
        "2040-01",
    );
    assert!(switches(&run(&v)).is_empty());
    let mut late = base();
    late["milestones"][0]["date"] = json!("2045-01"); // Start = AHV-Bezug
    late["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2045-01") ] });
    assert!(switches(&run(&late)).is_empty());
}

/// FEA-37: Auszahlung «Hypothek zurückzahlen» zahlt zuerst die Tranche mit indirekter Amortisation zurück.
#[test]
fn fea37_payout_repays_the_indirect_tranche() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    v["pillar3a"] = json!([{ "person": "A", "balance": 10_000, "rate": 0, "use": "repay" }]);
    let r = run(&v);
    // Rest im AHV-Bezug: 10'000 + 20 × 750 = 25'000 an die Hypothek
    close(rec(&r, 2045, 1).property_equity, 425_000.0, "Rückzahlung");
    assert!(payouts(&r).iter().any(|x| x.1.contains("an Hypothek 25000 CHF")));
}

/// FEA-37/AK13: Eine alte Säule 3a an der Hypothek wird beim Rechnen wie beim Laden übernommen.
#[test]
fn fea37_ak13_legacy_pillar3a_of_a_mortgage() {
    let mut v = base();
    let mut h = house(
        json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]),
        "2040-01",
    );
    h["mortgage"]["pillar3a"] = json!({ "balance": 10_000, "rate": 0, "payout": "2042-06", "use": "repay" });
    v["realestate"] = json!({ "properties": [h] });
    let r = run(&v);
    // 2042-06: 10'000 + 10 Quartale × 750 = 17'500 an die Hypothek
    close(
        rec(&r, 2042, 6).property_equity,
        417_500.0,
        "Auszahlung des ganzen Guthabens",
    );
    close(rec(&r, 2042, 6).pillar3a, 0.0, "leer");
    // Danach weiter indirekt bis zum AHV-Bezug, Rest 2045-01: 10 Quartale × 750
    assert!(payouts(&r)
        .iter()
        .any(|x| x.0 == ym(2045, 1) && x.1.contains("7500 CHF")));
}
