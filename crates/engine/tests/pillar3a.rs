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

/// FEA-37/AK12: Die Pensionskasse enthält kein 3a-Guthaben; in den Auswertungen stehen beide getrennt.
#[test]
fn fea37_ak12_pension_fund_and_pillar3a_are_kept_apart() {
    let mut v = base();
    v["pension"]["pk"]["capital"] = json!(200_000);
    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0 }]);
    let r = run(&v);
    close(rec(&r, 2040, 1).pension_fund, 200_000.0, "Pensionskasse ohne 3a");
    close(rec(&r, 2040, 1).pillar3a, 50_000.0, "Säule 3a für sich");
    // Das 3a-Guthaben zählt auch nicht zum liquiden Vermögen (Cash 1'000'000 abzüglich der Ausgaben des Monats)
    close(rec(&r, 2040, 1).liquid, 999_000.0, "liquid ohne 3a und PK");
}

// --- FEA-97: Tests zu nicht erkannten Mutanten des vollen Mutationslaufs ---

/// Zwei Personen: B (Geburt 1983-01) bezieht die AHV im Januar 2048, A im Januar 2045; Planende 2049-12.
fn couple() -> Value {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1980-01" },
        { "id": "B", "name": "B", "birth": "1983-01" }
    ]);
    v["milestones"][3]["date"] = json!("2049-12");
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "name": "Bezug PK 2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "name": "AHV-Bezug 2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv2" } });
    v
}

/// Tragbarkeit immer gegeben (ohne Einkommen sonst Einmalrückzahlung beim Kauf und bei der Erneuerung).
fn affordable() -> Value {
    json!({ "sustainability": 1, "imputedRate": 0, "fixCosts": 0, "wealthContribution": 0, "maxLoanToValue": 1 })
}

/// FEA-97/AK1: Die indirekte Amortisation fliesst in die Säule 3a der Person, die an der Tranche steht (hier B), und wird
/// bei deren AHV-Bezug ausbezahlt, nicht bei dem der ersten Person. Gilt für eine Immobilie im Besitz und für einen Kauf.
#[test]
fn fea97_ak1_indirect_amortization_goes_to_the_second_person() {
    let indirect = json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]);
    let mut owned = house(indirect.clone(), "2040-01");
    owned["mortgage"]["tranches"][0]["pillar3a"] = json!("B");
    let mut v = couple();
    v["realestate"] = json!({ "properties": [owned] });
    let r = run(&v);
    // Bis Dezember 2047 32 Quartale à 750 im 3a von B; nichts wird im AHV-Bezug von A (2045-01) ausbezahlt.
    close(rec(&r, 2045, 1).pillar3a, 15_000.0, "AHV-Bezug A: 3a von B bleibt");
    close(rec(&r, 2047, 12).pillar3a, 24_000.0, "vor dem AHV-Bezug B");
    close(rec(&r, 2048, 1).pillar3a, 0.0, "Rest im AHV-Bezug B");
    assert_eq!(payouts(&r).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2048, 1)]);
    assert_eq!(switches(&r).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2048, 1)]);

    // Kauf im Januar 2041 mit derselben Tranche: ebenfalls ins 3a von B (Ende 2047: 28 Quartale).
    let mut bought = house(indirect, "2041-01");
    bought["status"] = json!("planned");
    bought["buy"] = json!("2041-01");
    bought["mortgage"]["tranches"][0]["pillar3a"] = json!("B");
    let mut v = couple();
    v["realestate"] = json!({ "properties": [bought], "affordability": affordable() });
    let r = run(&v);
    close(rec(&r, 2045, 1).pillar3a, 12_000.0, "AHV-Bezug A: 3a von B bleibt");
    close(rec(&r, 2047, 12).pillar3a, 21_000.0, "vor dem AHV-Bezug B");
    assert_eq!(payouts(&r).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2048, 1)]);

    // Kauf «nach Verkauf» (Januar 2041, nach dem Verkauf eines anderen Hauses im Dezember 2040): ebenfalls ins 3a von B.
    bought["buy"] = json!("afterSale");
    let old = json!({ "name": "Alt", "status": "owned", "price": 300_000, "worth": 300_000, "sell": "2040-12", "fixCosts": 0 });
    let mut v = couple();
    v["flows"]["income"] = json!(50_000);
    v["realestate"] = json!({ "properties": [old, bought],
        "affordability": { "sustainability": 1, "imputedRate": 0.01, "fixCosts": 0, "wealthContribution": 0, "maxLoanToValue": 1 } });
    let r = run(&v);
    assert!(
        r.events
            .iter()
            .any(|e| e.kind == EventKind::Purchase && e.month == ym(2041, 1)),
        "Kauf im Januar 2041"
    );
    close(rec(&r, 2045, 1).pillar3a, 12_000.0, "AHV-Bezug A: 3a von B bleibt");
    close(rec(&r, 2047, 12).pillar3a, 21_000.0, "vor dem AHV-Bezug B");
}

/// FEA-97/AK3: Wechsel auf direkte Amortisation beim AHV-Bezug nur, wenn vorher wirklich indirekt amortisiert wurde,
/// ab jetzt weiter amortisiert wird und noch Schuld besteht; mit mehreren Tranchen nennt das Ereignis die Tranche.
#[test]
fn fea97_ak3_switch_to_direct_needs_amortization_before_and_after() {
    // Mit AHV-Rente ist die Erneuerung im AHV-Bezug tragbar (ohne Einkommen: Notverkauf statt Wechsel).
    let p = |periods: Value| {
        let mut v = base();
        v["realestate"] = json!({ "properties": [house(periods, "2040-01")], "affordability": affordable() });
        v["pension"]["ahv"]["monthly"] = json!(5000);
        v
    };
    // Gegenprobe: vorher indirekt, danach weiter amortisiert: Wechsel.
    let v = p(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 750, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]));
    assert_eq!(
        switches(&run(&v)).iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2045, 1)]
    );
    // Vorher indirekt, aber mit Amortisation 0: kein Wechsel.
    let v = p(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 0, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]));
    assert!(switches(&run(&v)).is_empty(), "vorher nichts amortisiert");
    // Vorher indirekt amortisiert, ab dem AHV-Bezug keine Amortisation mehr: kein Wechsel.
    let v = p(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 750, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 0, "amortMode": "indirect" } ]));
    assert!(switches(&run(&v)).is_empty(), "danach nichts amortisiert");

    // Tranche vor dem AHV-Bezug ganz zurückbezahlt (Auszahlung der 3a an die Hypothek): kein Wechsel mehr.
    let mut v = base();
    let mut h = house(
        json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]),
        "2040-01",
    );
    h["mortgage"]["tranches"][0]["amount"] = json!(10_000);
    v["realestate"] = json!({ "properties": [h] });
    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0,
        "payouts": [{ "date": "2042-01", "amount": 20_000, "use": "repay" }] }]);
    let r = run(&v);
    close(rec(&r, 2042, 1).mortgage, 0.0, "Tranche zurückbezahlt");
    assert!(switches(&r).is_empty(), "{:?}", switches(&r));

    // Planung beginnt im AHV-Bezug, die Tranche lief schon vorher indirekt: Der Wechsel liegt vor der Planung.
    let mut v = base();
    v["milestones"][0]["date"] = json!("2045-01");
    v["realestate"] = json!({ "properties": [house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2035-01")] });
    assert!(switches(&run(&v)).is_empty());

    // Zwei Tranchen: Das Ereignis nennt die Tranche.
    let mut v = base();
    let mut h = house(
        json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]),
        "2040-01",
    );
    h["mortgage"]["tranches"][0]["amount"] = json!(200_000);
    h["mortgage"]["tranches"][0]["name"] = json!("T1");
    let mut t2 = h["mortgage"]["tranches"][0].clone();
    t2["name"] = json!("T2");
    t2["periods"][0]["amortMode"] = json!("direct");
    h["mortgage"]["tranches"].as_array_mut().unwrap().push(t2);
    v["realestate"] = json!({ "properties": [h] });
    assert_eq!(
        switches(&run(&v)),
        vec![(
            ym(2045, 1),
            "Amortisation Hypothek Haus (T1) ab AHV-Bezug direkt statt über die Säule 3a: 3000 CHF pro Jahr"
                .to_string()
        )]
    );
}

/// FEA-97/AK2: Eine geplante Auszahlung im Monat des AHV-Bezugs wird als eigene Auszahlung gebucht, der Rest danach.
/// Ein geplanter Betrag, der genau dem Guthaben entspricht, löst keinen Hinweis «reicht nur» aus.
#[test]
fn fea97_ak2_planned_payout_in_the_ahv_month_and_exact_amount() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 120_000, "rate": 0, "payouts": [
        { "date": "2041-01", "amount": 20_000 }, { "date": "2045-01", "amount": 50_000 } ] }]);
    let r = run(&v);
    let p = payouts(&r);
    assert_eq!(
        p.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2041, 1), ym(2045, 1), ym(2045, 1)],
        "{p:?}"
    );
    assert!(p[1].1.contains("50000 CHF") && p[2].1.contains("50000 CHF"), "{p:?}");
    assert!(!p.iter().any(|x| x.1.contains("reicht nur")), "{p:?}");

    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0, "payouts": [
        { "date": "2041-01", "amount": 50_000 } ] }]);
    let p = payouts(&run(&v));
    assert!(!p.iter().any(|x| x.1.contains("reicht nur")), "{p:?}");
}

/// Entnahmephase ab Start (Pensionierung mit 60), AHV-Bezug mit 65 (2045-01), Einzahlung in die 3a 2'400 pro Jahr.
fn retired_with_contribution(contribution: f64) -> Value {
    let mut v = base();
    v["milestones"][1]["years"] = json!(60);
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": contribution }]);
    v
}

/// FEA-97/AK2: Ab der Pensionierung zählt die Einzahlung in die 3a weiter zu den Ausgaben: in der Reserve der
/// Strategie «Standard», in den Ausgaben der 3-Topf-Strategie und als Wohn- bzw. Fixkosten der Guardrails.
#[test]
fn fea97_ak2_contribution_after_retirement_is_an_expense() {
    // Standard: Reserve 12 Monate des Fehlbetrags (Ausgaben 1'000 + Einzahlung 200), der Rest in Aktien.
    let standard = |c: f64| {
        let mut v = retired_with_contribution(c);
        v["wealth"]["stockShare"] = json!(1);
        v["wealth"]["reserveMonths"] = json!(12);
        run(&v)
    };
    let (with, without) = (standard(2400.0), standard(0.0));
    close(rec(&with, 2041, 6).cash, 12.0 * 1200.0, "Reserve mit Einzahlung");
    close(rec(&without, 2041, 6).cash, 12.0 * 1000.0, "Reserve ohne Einzahlung");

    // 3-Topf: Die Einzahlung ist Teil der Ausgaben des Monats.
    let buckets = |c: f64| {
        let mut v = retired_with_contribution(c);
        v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 1, "bondYears": 1, "refillThreshold": -1 } });
        run(&v)
    };
    let (with, without) = (buckets(2400.0), buckets(0.0));
    close(
        rec(&without, 2041, 6).liquid - rec(&with, 2041, 6).liquid,
        18.0 * 200.0,
        "18 Einzahlungen weniger liquid",
    );
    // Topf 1 nach der Januar-Auffüllung: Ziel um 12 × 200 höher, die Ausgaben des Monats um 200.
    close(
        rec(&with, 2041, 1).cash - rec(&without, 2041, 1).cash,
        2400.0 - 200.0,
        "Topf 1 im Januar",
    );

    // Guardrails: Die Einzahlung wird wie die Wohnkosten von der Entnahme abgezogen; die Lebenshaltung sinkt um 200.
    let guardrails = |c: f64| {
        let mut v = retired_with_contribution(c);
        v["withdrawal"] = json!({ "strategy": "guardrails",
            "guardrails": { "initialRate": 0.043, "upper": 0.2, "lower": 0.2, "adjust": 0.1 } });
        run(&v)
    };
    let (with, without) = (guardrails(2400.0), guardrails(0.0));
    close(
        rec(&without, 2040, 3).spending - rec(&with, 2040, 3).spending,
        0.0,
        "gleiche Ausgaben insgesamt",
    );
    close(
        rec(&without, 2040, 3).liquid - rec(&with, 2040, 3).liquid,
        0.0,
        "gleiche Entnahme",
    );
}

/// FEA-97/AK2: Netto-Bedarf der 3-Topf-Strategie mit Einzahlung in die 3a: ab Juli auf 12 Monate hochgerechnet, mit
/// Guardrails-Lebenshaltung und im Jahr des AHV-Bezugs ohne Einzahlung (sie endet vor dem AHV-Bezug).
#[test]
fn fea97_ak2_need_with_contribution() {
    let run_b = |c: f64, guardrails: bool| {
        let mut v = retired_with_contribution(c);
        v["milestones"][1] =
            json!({ "id": "retire", "name": "Bezug Pensionskasse", "kind": "date", "date": "2040-07" });
        let spending = if guardrails { "guardrails" } else { "planned" };
        v["withdrawal"] = json!({ "strategy": "three_buckets",
            "buckets": { "cashYears": 1, "bondYears": 1, "refillThreshold": -1, "spending": spending },
            "guardrails": { "initialRate": 0.043, "upper": 0.2, "lower": 0.2, "adjust": 0.1 } });
        run(&v)
    };
    let refill = |r: &SimResult, y: i32, m: u32| {
        r.bucket_refills
            .iter()
            .find(|b| b.month == ym(y, m) && (m != 1 || b.trigger == "Januar"))
            .expect("Auffüllung")
            .clone()
    };
    let r = run_b(2400.0, false);
    let jul = refill(&r, 2040, 7);
    assert_eq!(jul.basis.months, 6);
    close(jul.basis.pillar3a, 2400.0, "6 Monate auf 12 hochgerechnet");
    close(refill(&r, 2044, 1).basis.pillar3a, 2400.0, "letztes volles Jahr");
    // Im Januar 2045 zählen zwei Auffüllungen: zuerst der Zufluss aus der 3a (Bedarf wie bisher), dann die geplante.
    close(refill(&r, 2045, 1).basis.pillar3a, 0.0, "Jahr des AHV-Bezugs");

    // Guardrails: Die Entnahme bleibt beim Budget, die Lebenshaltung im Netto-Bedarf ist um 12 × 200 kleiner.
    let r = run_b(2400.0, true);
    let a = refill(&r, 2041, 1);
    assert_eq!(a.basis.living_source, "Guardrails");
    close(a.basis.pillar3a, 2400.0, "3a im Netto-Bedarf");
    close(
        a.basis.living + 2400.0,
        12.0 * rec(&r, 2041, 1).withdrawal,
        "Lebenshaltung + 3a = Entnahme",
    );
}
