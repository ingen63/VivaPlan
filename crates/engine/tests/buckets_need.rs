//! 3-Topf-Strategie nach Spezifikation: Netto-Bedarf des laufenden Jahres, Januar-Auffüllung, Renditekriterium und die
//! Sonderfälle A (Start der Entnahmephase, PK-Kapitalbezug, Neustart), B (externe Zuflüsse) und C (Vorausschau).
//!
//! Netto-Bedarf = Lebenshaltung + Wohnen + Einkommenssteuer − regelmässige Einnahmen (AHV, PK-Rente, Erwerb). Nicht
//! enthalten: einmalige Ausgaben, Amortisationen, Immobilienkäufe, Sondersteuern. Ziel Topf 1 und 2 = Jahre × Netto-Bedarf.
//! Die Stände in `before`/`after` der Protokolle gelten vor den Ausgaben des Monats.

use engine::batch;
use engine::model::Plan;
use engine::sim::{BucketRefill, EventKind, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

/// Lebenshaltung 100'000, Miete 24'000, Einkommenssteuer 4'000 (Satz auf 84'000 Renten), Renten 84'000 (AHV 2'500 +
/// PK-Rente 4'500 pro Monat) → Netto-Bedarf 44'000; Topf 1 = 2 Jahre, Topf 2 = 1 Jahr; Aktien 6 %, Obligationen 1 %.
fn base() -> Value {
    json!({
        "project": { "name": "3-Topf" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2041-12" }
        ],
        "pension": { "pk": { "capital": 1_000_000, "at": "retire", "lumpsum": 0, "conversion": 0.054 }, "ahv": { "monthly": 2_500, "at": "start" } },
        "flows": { "income": 0, "spending": 100_000.0 / 12.0 },
        "wealth": { "stocks": 600_000, "bonds": 200_000, "cash": 700_000, "stockShare": 0.6, "reserveMonths": 6 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.06, "bonds": 0.01, "inflation": 0 } },
        "taxes": { "income": [[0, 4000.0 / 84000.0]], "wealth": [], "capital": [], "propertyGain": { "brackets": [], "reduction": 0.0 } },
        "withdrawal": { "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 1, "refillThreshold": -1 } },
        "realestate": { "properties": [ { "name": "Miete", "status": "rent", "from": "start", "rent": 2000 } ] }
    })
}

fn run(v: Value) -> SimResult {
    let p: Plan = serde_json::from_value(crate::any_age(v)).expect("Planung gültig");
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

fn find<'a>(res: &'a SimResult, trigger: &str, y: i32, m: u32) -> &'a BucketRefill {
    res.bucket_refills
        .iter()
        .find(|r| r.month == YearMonth::new(y, m) && r.trigger == trigger)
        .unwrap_or_else(|| panic!("kein Eintrag {trigger} {m}.{y}"))
}

fn january(res: &SimResult, y: i32) -> &BucketRefill {
    find(res, "Januar", y, 1)
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} statt {b}");
}

/// Kein Topf fällt je unter 0 (am Monatsende und in allen Protokollen).
fn never_negative(res: &SimResult) {
    for r in &res.records {
        assert!(
            r.cash >= -1e-6 && r.bonds >= -1e-6 && r.stocks >= -1e-6,
            "{}: {} / {} / {}",
            r.month.iso(),
            r.cash,
            r.bonds,
            r.stocks
        );
    }
    for r in &res.bucket_refills {
        assert!(
            r.before.iter().all(|x| *x >= -1e-6),
            "Protokoll {} vorher {:?}",
            r.month.iso(),
            r.before
        );
        assert!(
            r.after.iter().all(|x| *x >= -1e-6),
            "Protokoll {} nachher {:?}",
            r.month.iso(),
            r.after
        );
    }
}

fn table(res: &SimResult, what: &str) {
    println!("== {what}");
    println!(
        "{:8} {:7} {:>9} {:>9} {:>9} {:>34} {:>34}",
        "Monat", "Fall", "Bedarf", "Ziel 1", "Ziel 2", "vorher Topf 1/2/3", "nachher Topf 1/2/3"
    );
    for r in &res.bucket_refills {
        println!(
            "{:8} {:7} {:9.0} {:9.0} {:9.0} {:>11.0}/{:>10.0}/{:>10.0} {:>11.0}/{:>10.0}/{:>10.0}",
            r.month.iso(),
            r.trigger,
            r.need,
            r.target1,
            r.target2,
            r.before[0],
            r.before[1],
            r.before[2],
            r.after[0],
            r.after[1],
            r.after[2]
        );
    }
}

#[test]
fn example_january_2031_gives_88000_and_44000() {
    let res = run(base());
    let r = january(&res, 2031);
    assert_eq!(r.basis.year, 2031);
    assert!(r.basis.source.contains("Januar"), "{}", r.basis.source);
    assert_eq!(r.basis.months, 12);
    close(r.basis.living, 100_000.0, "Lebenshaltung");
    close(r.basis.housing, 24_000.0, "Wohnen");
    close(r.basis.income_tax, 4_000.0, "Einkommenssteuer");
    close(r.basis.ahv, 30_000.0, "AHV");
    close(r.basis.pk, 54_000.0, "PK-Rente");
    close(r.need, 44_000.0, "Netto-Bedarf");
    close(r.target1, 88_000.0, "Ziel Topf 1");
    close(r.target2, 44_000.0, "Ziel Topf 2");
    close(r.after[0], 88_000.0, "Topf 1 nach der Auffüllung");
    close(r.after[1], 44_000.0, "Topf 2 nach der Auffüllung");
    close(
        r.after[0] + r.after[1] + r.after[2],
        r.before.iter().sum::<f64>(),
        "Summe unverändert",
    );
    assert!(r.condition_met && !r.condition_ignored);
}

#[test]
fn ten_years_above_the_threshold_reach_the_targets_every_year() {
    let res = run(base());
    table(&res, "10 Jahre, Rendite über der Schwelle");
    for y in 2031..=2041 {
        let r = january(&res, y);
        close(r.need, 44_000.0, &format!("Netto-Bedarf {y}"));
        close(r.after[0], 88_000.0, &format!("Topf 1 {y}"));
        close(r.after[1], 44_000.0, &format!("Topf 2 {y}"));
        // Topf 1 ist durch die Ausgaben des Jahres geschrumpft und wird jedes Jahr wieder auf das Ziel gebracht
        assert!(r.before[0] < r.after[0]);
        // (im ersten Jahr fehlt die Einkommenssteuer des Vorjahres, die erst im Mai fällig wird)
        let paid = if y == 2031 { 40_000.0 } else { 44_000.0 };
        close(r.before[0] + paid, 88_000.0, &format!("Topf 1 vor der Auffüllung {y}"));
    }
    never_negative(&res);
}

#[test]
fn inflation_makes_the_pots_grow() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    let res = run(v);
    let t: Vec<f64> = (2031..=2036).map(|y| january(&res, y).target1).collect();
    assert!(
        t.windows(2).all(|w| w[1] > w[0]),
        "Ziele wachsen mit den Ausgaben: {t:?}"
    );
}

#[test]
fn special_taxes_and_one_time_outflows_do_not_change_the_target() {
    let reference = run(base());
    // Einmalige Ausgabe 2033 (Reise) und Sondersteuern: Grundstückgewinnsteuer 2033 (Verkauf), Kapitalbezugssteuer
    let mut v = base();
    v["flows"]["extras"] = json!([ { "date": "2033-03", "amount": -50_000, "label": "Reise" } ]);
    v["taxes"]["propertyGain"] = json!({ "brackets": [[1e18, 0.2]], "reduction": 0.0 });
    v["taxes"]["capital"] = json!([[0, 0.05]]);
    v["realestate"]["properties"] = json!([
        { "name": "Miete", "status": "rent", "from": "start", "rent": 2000 },
        { "id": "h", "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "2033-06", "fixCosts": 0 }
    ]);
    let res = run(v);
    let sale = res
        .property_events
        .iter()
        .find(|e| e.gain_tax.unwrap_or(0.0) > 0.0)
        .expect("Verkauf mit Steuer");
    assert_eq!(sale.month, YearMonth::new(2033, 6));
    for y in [2034, 2035] {
        let (a, b) = (january(&res, y), january(&reference, y));
        close(a.need, b.need, &format!("Netto-Bedarf {y}"));
        close(a.target1, b.target1, &format!("Ziel Topf 1 {y}"));
        close(a.basis.income_tax, b.basis.income_tax, &format!("Einkommenssteuer {y}"));
    }
}

#[test]
fn amortization_is_not_part_of_the_need() {
    let house = |amort: f64| {
        json!([ { "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 300_000, "start": "2025-01", "periods": [ { "type": "fixed", "term": 30, "rate": 0.0, "amort": amort, "amortMode": "direct" } ] } ] } } ])
    };
    let mut with = base();
    with["realestate"]["properties"] = house(6_000.0);
    let mut without = base();
    without["realestate"]["properties"] = house(0.0);
    let (a, b) = (run(with), run(without));
    let (ra, rb) = (january(&a, 2031), january(&b, 2031));
    close(ra.basis.housing, 0.0, "Wohnen mit Amortisation");
    close(ra.need, rb.need, "Netto-Bedarf mit und ohne Amortisation");
}

#[test]
fn net_need_zero_gives_zero_targets_and_everything_goes_to_pot_3() {
    let mut v = base();
    v["flows"]["spending"] = json!(2_000);
    v["realestate"]["properties"] = json!([]);
    v["taxes"]["income"] = json!([]);
    let res = run(v);
    let r = january(&res, 2031);
    assert_eq!(r.need, 0.0);
    assert_eq!((r.target1, r.target2), (0.0, 0.0));
    close(r.after[0], 0.0, "Topf 1");
    close(r.after[1], 0.0, "Topf 2");
    assert!(r.after[2] > 1_000_000.0, "alles im Topf 3");
}

#[test]
fn a_falling_need_sends_the_surplus_to_pot_3() {
    // AHV beginnt erst Januar 2032: Der Netto-Bedarf sinkt von 72'571 auf 44'000; Topf 2 liegt dann über seinem Ziel
    let mut v = base();
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "ahvm", "name": "AHV", "kind": "date", "date": "2032-01" }));
    v["pension"]["ahv"] = json!({ "monthly": 2_500, "at": "ahvm" });
    let res = run(v);
    let (a, b) = (january(&res, 2031), january(&res, 2032));
    assert!(b.need < a.need && b.target2 < a.target2, "{} -> {}", a.need, b.need);
    close(b.after[1], b.target2, "Topf 2 auf dem neuen Ziel");
    let m = b
        .moves
        .iter()
        .find(|m| (m.from, m.to) == (2, 3))
        .expect("Überschuss Topf 2 → Topf 3");
    assert!(m.amount > 10_000.0, "{m:?}");
    never_negative(&res);
}

#[test]
fn pot_2_smaller_than_the_gap_of_pot_1_takes_the_rest_from_pot_3_then_pot_2_from_pot_3() {
    let mut v = base();
    v["withdrawal"]["buckets"]["bondYears"] = json!(0.5);
    let res = run(v);
    let r = january(&res, 2032);
    close(r.target2, 22_000.0, "Ziel Topf 2");
    // Fehlbetrag Topf 1 = 44'000; Topf 2 (22'000) reicht nicht, 22'000 kommen aus Topf 3, dann Topf 2 wieder aus Topf 3
    let m: Vec<(u8, u8)> = r.moves.iter().map(|m| (m.from, m.to)).collect();
    assert_eq!(m, vec![(2, 1), (3, 1), (3, 2)], "{:?}", r.moves);
    close(r.after[0], 88_000.0, "Topf 1");
    close(r.after[1], 22_000.0, "Topf 2");
}

/// Plan mit der Schwelle 100 % (die Aktienrendite von 6 % erreicht sie nie).
fn locked_plan() -> Value {
    let mut v = base();
    v["withdrawal"]["buckets"]["refillThreshold"] = json!(1.0);
    v
}

#[test]
fn years_below_the_threshold_sell_nothing_from_pot_3_in_january_and_pot_1_comes_from_pot_2() {
    let res = run(locked_plan());
    table(&res, "Schwelle nicht erreicht");
    for y in 2031..=2033 {
        let r = january(&res, y);
        assert!(!r.condition_met && !r.condition_ignored);
        assert!(
            r.moves.iter().all(|m| m.from != 3),
            "Jahr {y}: Verkauf aus Topf 3: {:?}",
            r.moves
        );
    }
    // Topf 2 wird nicht nachgefüllt: nach dem Transfer in Topf 1 ist er kleiner als das Ziel
    let r = january(&res, 2031);
    assert!(r.after[1] < r.target2 - 1.0);
    // Unterjährig greift C: Topf 1 deckt die Ausgaben nicht, die Mittel kommen dann aus Topf 3
    let c = res.bucket_refills.iter().find(|r| r.trigger == "C2").expect("Fall C2");
    assert!(!c.condition_met);
    assert!(c.moves.iter().any(|m| (m.from, m.to) == (3, 1)), "{:?}", c.moves);
    never_negative(&res);
}

#[test]
fn a_simulation_younger_than_12_months_uses_the_return_since_january() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-03");
    let res = run(v);
    let a = find(&res, "A", 2030, 3);
    assert_eq!(a.return_basis, "seit Januar");
    assert!(a.stock_return > 0.0 && a.stock_return < 0.06, "{}", a.stock_return);
    let j = january(&res, 2031);
    assert_eq!(j.return_basis, "letzte 12 Monate");
    close(j.stock_return, 0.06, "Rendite 12 Monate");
}

#[test]
fn start_in_march_reduces_the_targets_by_two_months_and_extrapolates_the_need() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-03");
    let res = run(v);
    table(&res, "Start im März");
    let a = find(&res, "A", 2030, 3);
    assert_eq!(a.passed_months, 2);
    assert_eq!(a.basis.months, 10);
    assert!(a.basis.source.contains("hochgerechnet"));
    close(a.need, 44_000.0, "Netto-Bedarf auf 12 Monate hochgerechnet");
    close(a.target1, (24.0 - 2.0) / 12.0 * 44_000.0, "Ziel Topf 1");
    close(a.target2, (12.0 - 2.0) / 12.0 * 44_000.0, "Ziel Topf 2");
    assert!(a.condition_ignored);
    close(a.after[0], a.target1, "Topf 1");
    close(a.after[1], a.target2, "Topf 2");
    // alles lag zuerst in Topf 3
    assert_eq!((a.moves[0].to, a.moves[1].to), (3, 3));
    never_negative(&res);
}

/// Zwei Personen: Person B bezieht im Juli 2030 ihre PK zur Hälfte als Kapital (200'000).
fn two_persons() -> Value {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01" },
        { "id": "B", "name": "B", "birth": "1965-01" }
    ]);
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "retireb", "name": "Pensionierung B", "kind": "date", "date": "2030-07" }));
    v["partner"] = json!({
        "income": 0,
        "pk": { "capital": 400_000, "at": "retireb", "lumpsum": 0.5, "conversion": 0.054 },
        "ahv": { "monthly": 0, "at": "never" }
    });
    v
}

#[test]
fn a_later_pension_fund_withdrawal_goes_to_pot_3_and_triggers_a_with_reduced_targets() {
    let res = run(two_persons());
    table(&res, "PK-Kapitalbezug im Juli");
    let b = find(&res, "B", 2030, 7);
    assert!((b.moves[0].amount - 200_000.0).abs() < 1.0, "{:?}", b.moves);
    assert_eq!((b.moves[0].from, b.moves[0].to), (0, 3));
    let a = find(&res, "A", 2030, 7);
    assert!(a.reason.contains("PK-Kapitalbezug"), "{}", a.reason);
    assert_eq!(a.passed_months, 6);
    assert!(a.condition_ignored);
    close(
        a.target1,
        (24.0 - 6.0) / 12.0 * a.need,
        "Ziel Topf 1 um 6 Monate reduziert",
    );
    close(
        a.target2,
        (12.0 - 6.0) / 12.0 * a.need,
        "Ziel Topf 2 um 6 Monate reduziert",
    );
    // unterjährig keine Überschussbuchung: Topf 1 über dem Ziel bleibt unverändert
    assert!(a.moves.iter().all(|m| !(m.to == 3 && m.from != 0)), "{:?}", a.moves);
    assert!(a.after[0] >= a.target1 - 1e-6);
    never_negative(&res);
}

/// GEN-03/AK4: Ein PK-Kapitalbezug mitten im Jahr (Auslöser A) bestimmt den Netto-Bedarf neu, aus den Monaten ab dem
/// Bezug auf 12 Monate hochgerechnet; dieser Bedarf gilt bis zum nächsten Januar.
#[test]
fn gen03_ak4_a_pension_fund_withdrawal_recomputes_the_need_from_the_withdrawal_month() {
    let res = run(two_persons());
    let a = find(&res, "A", 2030, 7);
    assert_eq!(a.basis.year, 2030);
    assert_eq!(a.basis.months, 6, "Juli bis Dezember");
    assert!(a.basis.source.starts_with("ab 07.2030"), "{}", a.basis.source);
    assert!(a.basis.source.contains("hochgerechnet"), "{}", a.basis.source);
    close(a.need, a.basis.need(), "Bedarf aus der neuen Grundlage");
    // bis Dezember gilt diese Grundlage, erst der Januar bestimmt den Bedarf neu
    for r in res
        .bucket_refills
        .iter()
        .filter(|r| r.month.year() == 2030 && r.month.month() > 7)
    {
        assert_eq!(r.basis.source, a.basis.source, "{} {}", r.trigger, r.month);
    }
    assert_eq!(january(&res, 2031).basis.source, "Januar, geplant");
}

#[test]
fn external_inflows_go_to_pot_3_only() {
    let reference = run(base());
    let mut v = base();
    v["flows"]["extras"] = json!([ { "date": "2032-05", "amount": 100_000, "label": "Erbschaft" } ]);
    let res = run(v);
    let b = find(&res, "B", 2032, 5);
    assert_eq!((b.moves[0].from, b.moves[0].to), (0, 3));
    close(b.moves[0].amount, 100_000.0, "Zufluss");
    assert!(
        res.bucket_refills
            .iter()
            .all(|r| !(r.month == YearMonth::new(2032, 5) && r.trigger != "B")),
        "keine Auffüllung"
    );
    let at = |r: &SimResult| {
        r.records
            .iter()
            .find(|x| x.month == YearMonth::new(2032, 5))
            .unwrap()
            .clone()
    };
    let (x, y) = (at(&res), at(&reference));
    close(x.cash, y.cash, "Topf 1 unverändert");
    close(x.bonds, y.bonds, "Topf 2 unverändert");
    assert!(
        x.stocks > y.stocks + 99_000.0,
        "Topf 3 +100'000: {} gegen {}",
        x.stocks,
        y.stocks
    );
}

#[test]
fn a_large_one_time_outflow_with_the_criterion_met_is_covered_by_c1_with_shortened_targets() {
    let mut v = base();
    v["flows"]["extras"] = json!([ { "date": "2033-03", "amount": -300_000, "label": "Kapitalauszahlungssteuer" } ]);
    let res = run(v);
    table(&res, "C1: Einmalausgabe 03.2033");
    let c = find(&res, "C1", 2033, 3);
    assert!(c.payment > 300_000.0);
    assert_eq!(c.passed_months, 2);
    assert!(c.condition_met && !c.condition_ignored);
    // Ziel Topf 1 = (2 × 12 − 2) / 12 × Netto-Bedarf + Einmalausgabe, Ziel Topf 2 = (12 − 2) / 12 × Netto-Bedarf
    close(c.target1, 22.0 / 12.0 * 44_000.0 + 300_000.0, "Ziel Topf 1");
    close(c.target2, 10.0 / 12.0 * 44_000.0, "Ziel Topf 2");
    close(c.after[0], c.target1, "Topf 1");
    close(c.after[1], c.target2, "Topf 2");
    assert!(
        c.moves.iter().all(|m| m.to != 3),
        "keine Überschussbuchung in Topf 3: {:?}",
        c.moves
    );
    let m = res.records.iter().find(|x| x.month == YearMonth::new(2033, 3)).unwrap();
    assert!(m.cash >= 0.0, "{}", m.cash);
    never_negative(&res);
}

/// Topf 1 ohne Ziel, Renditekriterium nie erfüllt: Fall C2 sorgt dafür, dass Topf 1 die Zahlung deckt.
fn no_cash_target() -> Value {
    let mut v = base();
    v["withdrawal"]["buckets"] = json!({ "cashYears": 0, "bondYears": 0, "refillThreshold": 1.0 });
    v
}

#[test]
fn c_keeps_two_monthly_needs_in_june_and_nothing_in_december() {
    let month = 44_000.0 / 12.0;
    let mut v = no_cash_target();
    v["flows"]["extras"] = json!([ { "date": "2030-06", "amount": -20_000, "label": "Ausgabe" } ]);
    let res = run(v);
    let c = find(&res, "C2", 2030, 6);
    // Juni: 7 Monate bis Januar → Min(3, 7) − 1 = 2 Monatsbedarfe
    let j = res.records.iter().find(|x| x.month == YearMonth::new(2030, 6)).unwrap();
    close(j.cash, 2.0 * month, "Topf 1 nach der Zahlung im Juni");
    assert!(!c.condition_met);
    let mut v = no_cash_target();
    v["flows"]["extras"] = json!([ { "date": "2030-12", "amount": -20_000, "label": "Ausgabe" } ]);
    let res = run(v);
    find(&res, "C2", 2030, 12);
    // Dezember: 1 Monat bis Januar → Min(3, 1) − 1 = 0
    let d = res
        .records
        .iter()
        .find(|x| x.month == YearMonth::new(2030, 12))
        .unwrap();
    assert!(d.cash.abs() < 1e-6, "{}", d.cash);
    never_negative(&res);
}

#[test]
fn the_reserve_months_setting_is_configurable() {
    let mut v = no_cash_target();
    v["withdrawal"]["buckets"]["reserveMonths"] = json!(5);
    v["flows"]["extras"] = json!([ { "date": "2030-06", "amount": -20_000, "label": "Ausgabe" } ]);
    let res = run(v);
    let j = res.records.iter().find(|x| x.month == YearMonth::new(2030, 6)).unwrap();
    close(j.cash, 4.0 * 44_000.0 / 12.0, "5 Monate − 1");
}

#[test]
fn earned_income_is_deducted_from_the_need() {
    let mut v = base();
    v["flows"]["income"] = json!(3_000); // 36'000 pro Jahr
    let res = run(v);
    let r = january(&res, 2031);
    close(r.basis.earned, 36_000.0, "Erwerbseinkommen");
    close(r.basis.ahv + r.basis.pk, 84_000.0, "Renten");
    close(r.basis.income_tax, 120_000.0 * 4_000.0 / 84_000.0, "Einkommenssteuer");
    close(
        r.need,
        124_000.0 + r.basis.income_tax - 84_000.0 - 36_000.0,
        "Netto-Bedarf",
    );
    close(r.target1, 2.0 * r.need, "Ziel Topf 1");
}

#[test]
fn the_target_rises_in_the_first_year_after_working_income_ends() {
    let mut v = base();
    v["flows"]["income"] =
        json!({ "mode": "timed", "rows": [ { "from": "start", "value": 3_000 }, { "from": "2033-01", "value": 0 } ] });
    let res = run(v);
    let (a, b, c) = (january(&res, 2032), january(&res, 2033), january(&res, 2034));
    close(a.basis.earned, 36_000.0, "Erwerb 2032");
    close(b.basis.earned, 0.0, "kein Erwerb 2033 (geplant für das laufende Jahr)");
    close(b.target1, 88_000.0, "Ziel steigt schon im Januar 2033");
    close(c.target1, 88_000.0, "Ziel 2034");
}

#[test]
fn all_pots_empty_with_a_property_sells_it_puts_the_proceeds_into_pot_3_and_covers_again() {
    let mut v = base();
    v["wealth"] = json!({ "stocks": 0, "bonds": 0, "cash": 30_000, "stockShare": 0, "reserveMonths": 0 });
    v["flows"]["extras"] = json!([ { "date": "2030-05", "amount": -60_000, "label": "Ausgabe" } ]);
    v["realestate"]["properties"] = json!([
        { "name": "Miete", "status": "rent", "from": "start", "rent": 2000 },
        { "id": "h", "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0 }
    ]);
    let res = run(v);
    table(&res, "Alle Töpfe leer mit Immobilie");
    assert_eq!(res.summary.emergency_sales, 1);
    assert!(
        res.bucket_refills
            .iter()
            .any(|r| r.trigger == "C1" && r.month == YearMonth::new(2030, 5)),
        "C1 vor dem Notverkauf"
    );
    let sale = res
        .property_events
        .iter()
        .find(|e| e.net.is_some_and(|n| n > 0.0))
        .expect("Notverkauf");
    let b = find(&res, "B", sale.month.year(), sale.month.month());
    assert_eq!((b.moves[0].from, b.moves[0].to), (0, 3), "Erlös in Topf 3");
    never_negative(&res);
}

#[test]
fn all_pots_empty_without_a_property_goes_negative_takes_inflows_into_pot_1_and_restarts_with_a() {
    let mut v = base();
    v["wealth"] = json!({ "stocks": 0, "bonds": 0, "cash": 20_000, "stockShare": 0, "reserveMonths": 0 });
    v["pension"]["ahv"] = json!({ "monthly": 0, "at": "never" });
    v["pension"]["pk"]["capital"] = json!(0);
    v["realestate"]["properties"] = json!([]);
    v["flows"]["extras"] = json!([ { "date": "2032-01", "amount": 400_000, "label": "Erbschaft" } ]);
    let res = run(v);
    table(&res, "Alle Töpfe leer ohne Immobilie");
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::Buckets && e.text.contains("alle Töpfe leer")));
    let neg = res.records.iter().filter(|r| r.cash < -1.0).count();
    assert!(neg > 5, "Topf 1 im Minus: {neg} Monate");
    for r in res.records.iter().filter(|r| r.cash < -1.0) {
        assert!(
            r.bonds.abs() < 1e-9 && r.stocks.abs() < 1e-9,
            "Topf 2 und 3 stillgelegt"
        );
    }
    // Der Zufluss baut das Minus ab; sobald das Vermögen positiv ist, läuft A erneut
    let a = res
        .bucket_refills
        .iter()
        .find(|r| r.trigger == "A" && r.reason.contains("Neustart"))
        .expect("Neustart");
    assert!(a.after.iter().all(|x| *x >= 0.0), "{:?}", a.after);
    close(a.after[0], a.target1, "Topf 1 wird wieder auf das Ziel gebracht");
    let after: Vec<_> = res
        .records
        .iter()
        .filter(|r| r.month >= a.month && r.month < a.month.plus(12))
        .collect();
    assert!(after.iter().all(|r| r.cash >= -1e-6));
}

#[test]
fn threshold_semantics_use_greater_or_equal() {
    // Schwelle genau gleich der Rendite von 6 %: erfüllt
    let mut v = base();
    v["withdrawal"]["buckets"]["refillThreshold"] = json!(0.06 - 1e-9);
    let res = run(v);
    assert!(january(&res, 2032).condition_met);
}

#[test]
fn c2_with_9_months_reserve_refills_8_months_from_pot_2_then_pot_3_and_does_not_refill_pot_2() {
    let month = 44_000.0 / 12.0;
    let mut v = locked_plan();
    v["withdrawal"]["buckets"]["reserveMonths"] = json!(9);
    v["flows"]["extras"] = json!([ { "date": "2030-04", "amount": -400_000, "label": "Anschaffung" } ]);
    let res = run(v);
    table(&res, "C2: Reserve 9 Monate, Einmalausgabe 04.2030");
    let c = find(&res, "C2", 2030, 4);
    assert!(
        c.before[0] >= 0.0,
        "Topf 1 vor der Vorausschau nie negativ: {}",
        c.before[0]
    );
    assert!(!c.condition_met);
    // Quelle: zuerst Topf 2 bis 0, danach Topf 3; Topf 2 wird nicht aufgefüllt
    assert_eq!((c.moves[0].from, c.moves[0].to), (2, 1));
    assert_eq!(c.after[1], 0.0);
    assert_eq!((c.moves[1].from, c.moves[1].to), (3, 1));
    assert!(c.moves.iter().all(|m| m.to != 2));
    // April: 9 Monate bis Januar → Min(9, 9) − 1 = 8 Monatsbedarfe bleiben nach den Ausgaben
    let r = res.records.iter().find(|x| x.month == YearMonth::new(2030, 4)).unwrap();
    close(r.cash, 8.0 * month, "Topf 1 nach den Ausgaben des Aprils");
    never_negative(&res);
}

#[test]
fn april_2035_with_the_criterion_met_is_c1_and_not_met_is_c2() {
    let mut v = base();
    v["flows"]["extras"] = json!([ { "date": "2035-04", "amount": -400_000, "label": "Anschaffung" } ]);
    let met = run(v.clone());
    table(&met, "04.2035 Kriterium erfüllt");
    let c = find(&met, "C1", 2035, 4);
    assert_eq!(c.passed_months, 3);
    close(
        c.target1,
        1.75 * 44_000.0 + 400_000.0,
        "Ziel Topf 1 = 1.75 × Netto-Bedarf + Einmalausgabe",
    );
    close(c.target2, 0.75 * 44_000.0, "Ziel Topf 2 = 0.75 × Netto-Bedarf");
    close(c.after[0], c.target1, "Topf 1");
    close(c.after[1], c.target2, "Topf 2");
    let m = met.records.iter().find(|x| x.month == YearMonth::new(2035, 4)).unwrap();
    assert!(m.cash >= 0.0);
    assert!(!met
        .bucket_refills
        .iter()
        .any(|r| r.month == YearMonth::new(2035, 4) && r.trigger == "C2"));
    never_negative(&met);
    // gleicher Fall, Kriterium nicht erfüllt → C2: Ausgaben decken, danach (Min(3, 9) − 1) = 2 Monatsbedarfe, Topf 2 bleibt
    let mut v = locked_plan();
    v["flows"]["extras"] = json!([ { "date": "2035-04", "amount": -400_000, "label": "Anschaffung" } ]);
    let locked = run(v);
    let c = find(&locked, "C2", 2035, 4);
    assert!(!c.condition_met && c.moves.iter().all(|m| m.to != 2));
    let m = locked
        .records
        .iter()
        .find(|x| x.month == YearMonth::new(2035, 4))
        .unwrap();
    close(m.cash, 2.0 * 44_000.0 / 12.0, "Topf 1 nach den Ausgaben");
    never_negative(&locked);
}

#[test]
fn c1_in_december_does_not_move_pot_2_above_the_shortened_target_to_pot_3() {
    let mut v = base();
    v["withdrawal"]["buckets"]["bondYears"] = json!(10);
    v["flows"]["extras"] = json!([ { "date": "2035-12", "amount": -38_000, "label": "Ausgabe" } ]);
    let res = run(v);
    let c = find(&res, "C1", 2035, 12);
    assert_eq!(c.passed_months, 11);
    close(c.target2, (120.0 - 11.0) / 12.0 * 44_000.0, "Ziel Topf 2 gekürzt");
    assert!(
        c.after[1] > c.target2,
        "Topf 2 liegt über dem gekürzten Ziel: {} / {}",
        c.after[1],
        c.target2
    );
    assert!(
        c.moves.iter().all(|m| !(m.from == 2 && m.to == 3)),
        "keine Umbuchung in Topf 3: {:?}",
        c.moves
    );
    never_negative(&res);
}

#[test]
fn a_large_one_time_outflow_in_january_is_covered_by_the_january_refill_without_c() {
    let mut v = base();
    v["flows"]["extras"] = json!([ { "date": "2034-01", "amount": -300_000, "label": "Anschaffung" } ]);
    let res = run(v);
    let j = january(&res, 2034);
    close(j.target1, 2.0 * 44_000.0 + 300_000.0, "Ziel Topf 1 + Einmalausgabe");
    close(j.after[0], j.target1, "Topf 1");
    assert!(!res
        .bucket_refills
        .iter()
        .any(|r| r.month == YearMonth::new(2034, 1) && r.trigger.starts_with('C')));
    never_negative(&res);
}

#[test]
fn a_pension_fund_withdrawal_in_october_books_no_surplus_into_pot_3() {
    let mut v = two_persons();
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|m| m["id"] == "retireb")
        .unwrap()["date"] = json!("2030-10");
    let res = run(v);
    let a = find(&res, "A", 2030, 10);
    assert_eq!(a.passed_months, 9);
    assert!(a.moves.iter().all(|m| !(m.to == 3 && m.from != 0)), "{:?}", a.moves);
    close(a.target1, (24.0 - 9.0) / 12.0 * a.need, "Ziel Topf 1");
    never_negative(&res);
}

#[test]
fn with_3_months_reserve_c_fires_repeatedly_after_the_outflow_with_at_most_2_months_left() {
    let month = 44_000.0 / 12.0;
    let mut v = no_cash_target();
    v["flows"]["extras"] = json!([ { "date": "2035-04", "amount": -400_000, "label": "Anschaffung" } ]);
    let res = run(v);
    let cs: Vec<_> = res
        .bucket_refills
        .iter()
        .filter(|r| r.trigger == "C2" && r.month >= YearMonth::new(2035, 4))
        .collect();
    assert!(cs.len() >= 4, "C löst mehrfach aus: {}", cs.len());
    for c in &cs {
        let r = res.records.iter().find(|x| x.month == c.month).unwrap();
        assert!(
            r.cash >= -1e-6 && r.cash <= 2.0 * month + 1e-6,
            "{}: Topf 1 nach C {}",
            c.month.iso(),
            r.cash
        );
    }
    never_negative(&res);
}
