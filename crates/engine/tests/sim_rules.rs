//! FEA-29: Einzelregeln des Rechenkerns mit nachgerechneten Werten (Entnahmestrategien, Immobilien, Hypotheken,
//! 3-Topf-Strategie, Monatsablauf). Jeder Test deckt Fälle ab, die sonst kein Test prüft.
//!
//! Grundplanung (erfunden): Start 2026-11, Geburt 1966-11, Pensionierung und AHV 2031-11 (Alter 65), Ende 2061-11;
//! ohne Steuern, Teuerung und Renditen, Ausgaben 1000 pro Monat, Vermögen 1'000'000 als Bargeld.

use engine::market::MarketPath;
use engine::model::Plan;
use engine::sim::{EventKind, MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    let mut v: Value = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
    v["returns"]["fixed"] = json!({"stocks": 0, "bonds": 0, "inflation": 0, "saron": 0});
    v["taxes"] = json!({"multiplier": 1, "income": [[0, 0]], "wealth": [[0, 0]], "capital": [[0, 0]]});
    v["flows"]["spending"] = json!(1000);
    // Die Regeln hier rechnen die Wohnkosten zusätzlich zu den Ausgaben (FEA-39 prüft crates/engine/tests/cash_housing.rs).
    v["flows"]["spendingIncludesHousing"] = json!(false);
    // Einkommen nominal gleich, wie die Regeln hier gerechnet sind (FEA-43 prüft crates/engine/tests/income_indexed.rs).
    v["flows"]["incomeIndexed"] = json!(false);
    v["wealth"] = json!({"stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0});
    v
}

fn ym(y: i32, m: u32) -> YearMonth {
    YearMonth::new(y, m)
}

/// Rechnet die Planung mit den fixen Werten; `f` darf den Renditepfad anpassen.
fn run_with(v: &Value, f: impl FnOnce(&mut MarketPath, YearMonth)) -> SimResult {
    let plan: Plan = serde_json::from_value(crate::any_age(v.clone())).unwrap();
    let sim = Simulator::new(&plan).unwrap();
    let mut path = engine::market::fixed_path(&sim.fixed, sim.start, sim.months());
    f(&mut path, sim.start);
    sim.run(&path, true)
}

fn run(v: &Value) -> SimResult {
    run_with(v, |_, _| {})
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records.iter().find(|x| x.month == ym(y, m)).unwrap()
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} != {b}");
}

/// Annuitätenfaktor: Anteil, der ein Vermögen in `n` Jahren bei Rendite `r` aufbraucht.
fn annuity(r: f64, n: f64) -> f64 {
    r / (1.0 - (1.0 + r).powf(-n))
}

fn index(start: YearMonth, m: YearMonth) -> usize {
    start.until(m) as usize
}

// ---------------------------------------------------------------------------------------------
// Entnahmestrategien
// ---------------------------------------------------------------------------------------------

#[test]
fn vpw_uses_the_remaining_years() {
    let mut v = base();
    v["withdrawal"]["strategy"] = json!("vpw");
    v["withdrawal"]["vpw"] = json!({"horizonAge": 100, "realReturn": 0.02});
    let r = run(&v);
    // Pensionierung: 60 Monate zu 1000 ausgegeben, 35 Jahre bis Alter 100
    let b0 = annuity(0.02, 35.0) * 940_000.0;
    close(rec(&r, 2031, 11).spending, b0 / 12.0, "Nov 2031");
    // Januar: neu mit dem Vermögen nach zwei Monaten und 100 − (65 + 2/12) Jahren
    let b1 = annuity(0.02, 100.0 - (65.0 + 2.0 / 12.0)) * (940_000.0 - b0 / 6.0);
    close(rec(&r, 2032, 1).spending, b1 / 12.0, "Jan 2032");
}

#[test]
fn endowment_smooths_with_inflation() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.12);
    v["withdrawal"]["strategy"] = json!("endowment");
    v["withdrawal"]["endowment"] = json!({"rate": 0.04, "smoothing": 0.3});
    let r = run(&v);
    let b0 = 0.04 * rec(&r, 2031, 10).liquid;
    close(rec(&r, 2031, 11).spending, b0 / 12.0, "Nov 2031");
    let f = rec(&r, 2032, 1).price_index / rec(&r, 2031, 1).price_index;
    assert!(f > 1.1);
    let b1 = 0.7 * b0 * f + 0.3 * 0.04 * rec(&r, 2031, 12).liquid;
    close(rec(&r, 2032, 1).spending, b1 / 12.0, "Jan 2032");
}

/// Guardrails 4 %, Leitplanken ±20 %, Anpassung 10 %, Teuerung 12 %; Aktienrendite im Dezember 2031 wie angegeben.
fn guardrails(december: f64) -> (f64, f64, f64) {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.12);
    v["wealth"]["stockShare"] = json!(1);
    v["withdrawal"]["strategy"] = json!("guardrails");
    v["withdrawal"]["guardrails"] = json!({"initialRate": 0.04, "upper": 0.2, "lower": 0.2, "adjust": 0.1});
    let r = run_with(&v, |p, start| p.stocks[index(start, ym(2031, 12))] = december);
    let b0 = 0.04 * rec(&r, 2031, 10).liquid;
    close(rec(&r, 2031, 11).spending, b0 / 12.0, "Nov 2031");
    let f = rec(&r, 2032, 1).price_index / rec(&r, 2031, 1).price_index;
    (b0, f, rec(&r, 2032, 1).spending * 12.0)
}

#[test]
fn guardrails_without_change_follow_inflation() {
    let (b0, f, b1) = guardrails(0.0);
    close(b1, b0 * f, "Teuerung");
}

#[test]
fn guardrails_cut_after_a_loss_without_inflation() {
    // Verlustjahr und Rate über der Startrate: keine Teuerung; Rate über 4.8 % ⇒ 10 % weniger
    let (b0, _, b1) = guardrails(-0.3);
    close(b1, b0 * 0.9, "gekürzt");
}

#[test]
fn guardrails_raise_after_a_gain() {
    // Gewinnjahr: Teuerung; Rate unter 3.2 % ⇒ 10 % mehr
    let (b0, f, b1) = guardrails(0.5);
    close(b1, b0 * f * 1.1, "erhöht");
}

#[test]
fn guardrails_events() {
    let mut v = base();
    v["wealth"]["stockShare"] = json!(1);
    v["withdrawal"]["strategy"] = json!("guardrails");
    v["withdrawal"]["guardrails"] = json!({"initialRate": 0.04, "upper": 0.2, "lower": 0.2, "adjust": 0.1});
    let r = run_with(&v, |p, start| p.stocks[index(start, ym(2031, 12))] = -0.3);
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == EventKind::Withdrawal && e.month == ym(2032, 1) && e.text.contains("gekürzt")));
}

// ---------------------------------------------------------------------------------------------
// 3-Topf-Strategie: Netto-Bedarf, Renditekriterium, Auffüllung
// ---------------------------------------------------------------------------------------------

fn buckets(v: &mut Value, cash_years: f64, bond_years: f64, threshold: f64) {
    v["withdrawal"]["strategy"] = json!("three_buckets");
    v["withdrawal"]["buckets"] =
        json!({"cashYears": cash_years, "bondYears": bond_years, "refillThreshold": threshold});
    // Ohne Cash-Topf (0 Jahre) hält C2 keine Monatsbedarfe zurück: Topf 1 wird jeden Monat nachgefüllt.
    if cash_years == 0.0 {
        v["withdrawal"]["buckets"]["reserveMonths"] = json!(1);
    }
}

fn refill<'a>(r: &'a SimResult, trigger: &str, y: i32, m: u32) -> &'a engine::sim::BucketRefill {
    r.bucket_refills
        .iter()
        .find(|x| x.month == ym(y, m) && x.trigger == trigger)
        .unwrap_or_else(|| {
            panic!(
                "kein Eintrag {trigger} {m}.{y}: {:?}",
                r.bucket_refills
                    .iter()
                    .map(|x| (x.month, x.trigger.clone()))
                    .collect::<Vec<_>>()
            )
        })
}

/// Ehepaar: A (PK 800'000 ab Pensionierung 2031-11), B (PK 600'000 ab 2031-12, Zins 1.2 %); AHV je 2500 ab 2031-11,
/// zusammen plafoniert auf 3780.
fn couple() -> Value {
    let mut v = base();
    v["household"]["persons"] =
        json!([{"id": "A", "name": "A", "birth": "1966-11"}, {"id": "B", "name": "B", "birth": "1967-01"}]);
    v["pension"]["pk"] = json!({"capital": 800_000, "at": "retire", "lumpsum": 0, "conversion": 0.05});
    v["pension"]["ahv"] = json!({"monthly": 2500, "at": "ahv", "indexed": true});
    v["partner"] = json!({"income": 0, "pk": {"capital": 600_000, "at": "2031-12", "lumpsum": 0, "conversion": 0.05, "interest": 0.012},
                          "ahv": {"monthly": 2500, "at": "ahv", "indexed": true}});
    buckets(&mut v, 1.0, 1.0, -1.0);
    v
}

#[test]
fn need_counts_pensions_that_start_later_in_the_year() {
    let r = run(&couple());
    let a = refill(&r, "A", 2031, 11);
    // November und Dezember, auf 12 Monate hochgerechnet (× 6)
    assert_eq!(a.basis.months, 2);
    assert_eq!(a.basis.source, "ab 11.2031, auf 12 Monate hochgerechnet");
    let pa = rec(&r, 2031, 11).pk_pension;
    let pb = rec(&r, 2031, 12).pk_pension - pa;
    assert!(pa > 0.0 && pb > 0.0);
    close(a.basis.pk, 6.0 * (2.0 * pa + pb), "PK-Renten");
    // AHV zusammen 5000, plafoniert auf 3780
    close(a.basis.ahv, 12.0 * 3780.0, "AHV plafoniert");
    close(a.basis.living, 12_000.0, "Lebenshaltung");
}

#[test]
fn need_estimates_a_later_pension_from_todays_capital() {
    // B bezieht die PK ab April 2032 (Kapitalbezug 25 %, ohne Zins): 600'000 × 0.75 × 5 % / 12 = 1875 ab April.
    let mut v = couple();
    v["partner"]["pk"] =
        json!({"capital": 600_000, "at": "2032-04", "lumpsum": 0.25, "conversion": 0.05, "interest": 0});
    let r = run(&v);
    let j = refill(&r, "Januar", 2032, 1);
    let pa = rec(&r, 2032, 1).pk_pension;
    close(rec(&r, 2032, 4).pk_pension - pa, 1875.0, "tatsächliche Rente B");
    close(j.basis.pk, 12.0 * pa + 9.0 * 1875.0, "PK-Renten im Bedarf");
}

#[test]
fn need_counts_the_planned_earned_income() {
    // B verdient 1000 pro Monat; Bedarf ab November 2031 aus zwei Monaten, auf 12 hochgerechnet.
    let mut v = couple();
    v["partner"]["income"] = json!(1000);
    let r = run(&v);
    let a = refill(&r, "A", 2031, 11);
    let earned = rec(&r, 2031, 11).earned + rec(&r, 2031, 12).earned;
    assert!(earned > 0.0);
    close(a.basis.earned, 6.0 * earned, "Erwerbseinkommen");
}

#[test]
fn need_uses_the_indexed_ahv_cap() {
    let mut v = couple();
    v["returns"]["fixed"]["inflation"] = json!(0.12);
    let r = run(&v);
    let j = refill(&r, "Januar", 2032, 1);
    assert_eq!(j.basis.months, 12);
    assert_eq!(j.basis.source, "Januar, geplant");
    // Plafond mit der Teuerung angepasst, wie die ausbezahlte AHV im Januar
    assert!(rec(&r, 2032, 1).ahv > 3780.0 * 1.1);
    close(j.basis.ahv, 12.0 * rec(&r, 2032, 1).ahv, "AHV");
}

#[test]
fn need_in_the_last_year_counts_only_the_remaining_months() {
    let mut v = base();
    v["milestones"][3] = json!({"id": "end", "kind": "date", "name": "Ende", "date": "2033-03", "fixed": true});
    buckets(&mut v, 1.0, 1.0, -1.0);
    let r = run(&v);
    let j = refill(&r, "Januar", 2033, 1);
    assert_eq!(j.basis.months, 3);
    close(j.basis.living, 12_000.0, "Lebenshaltung hochgerechnet");
}

#[test]
fn stock_return_over_the_last_twelve_months_and_since_january() {
    let mut v = base();
    v["wealth"]["stockShare"] = json!(1);
    buckets(&mut v, 1.0, 1.0, -1.0);
    // Aktien +1 % pro Monat 2032, +2 % pro Monat 2031
    let r = run_with(&v, |p, start| {
        for (k, x) in p.stocks.iter_mut().enumerate() {
            let m = start.plus(k as i32);
            *x = if m.year() == 2032 {
                0.01
            } else if m.year() == 2031 {
                0.02
            } else {
                0.0
            };
        }
    });
    let j = refill(&r, "Januar", 2033, 1);
    assert_eq!(j.return_basis, "letzte 12 Monate");
    close(j.stock_return, 1.01f64.powi(11) * 1.0 - 1.0, "Feb 2032 bis Jan 2033");

    // Pensionierung im Januar 2027 (2. Monat): Rendite seit Januar, also nur der Januar
    let mut v = base();
    v["wealth"]["stockShare"] = json!(1);
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2027-01"});
    buckets(&mut v, 1.0, 1.0, -1.0);
    let r = run_with(&v, |p, _| {
        p.stocks[0] = 0.10; // November 2026
        p.stocks[1] = 0.20; // Dezember 2026
        p.stocks[2] = 0.05; // Januar 2027
    });
    let a = refill(&r, "A", 2027, 1);
    assert_eq!(a.return_basis, "seit Januar");
    close(a.stock_return, 0.05, "Januar 2027");
}

// ---------------------------------------------------------------------------------------------
// Hypotheken: Tragbarkeit bei der Erneuerung
// ---------------------------------------------------------------------------------------------

/// Pensionierung und AHV im Januar 2027; Einkommen 10'000 + PK-Rente 1'000 + AHV 1'500 = 12'500 pro Monat.
/// Tragbarkeit: 25 % des Einkommens, kalkulatorischer Zins 6.25 % ⇒ tragbar = (0.25 × 150'000 − Nebenkosten) / 0.0625.
/// Haus mit einer Tranche ab 2025-11; die zweite Periode beginnt im November 2027.
fn renewal_plan(amount: f64, periods: Value) -> Value {
    let mut v = base();
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2027-01"});
    v["milestones"][2] = json!({"id": "ahv", "kind": "date", "name": "AHV", "date": "2027-01"});
    v["pension"]["pk"] = json!({"capital": 240_000, "at": "retire", "lumpsum": 0, "conversion": 0.05});
    v["pension"]["ahv"] = json!({"monthly": 1500, "at": "ahv", "indexed": true});
    v["flows"]["income"] = json!(10_000);
    v["wealth"]["stockShare"] = json!(0.5);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    v["realestate"]["emergencyMonths"] = json!(0);
    v["realestate"]["affordability"] = json!({"sustainability": 0.25, "imputedRate": 0.0625, "fixCosts": 0.01,
        "wealthContribution": 0.03, "maxLoanToValue": 0.8});
    v["realestate"]["properties"] = json!([{"name": "Haus", "status": "owned", "price": 1_000_000, "worth": 1_000_000,
        "sell": "never", "fixCosts": 100, "inflation": false,
        "mortgage": {"tranches": [{"name": "T1", "amount": amount, "start": "2025-11", "periods": periods}]}}]);
    v
}

fn two_periods() -> Value {
    json!([{"type": "fixed", "term": 2, "rate": 0.02}, {"type": "fixed", "term": 1, "rate": 0.03}, {"type": "fixed", "term": 10, "rate": 0.04}])
}

#[test]
fn renewal_needs_a_lump_sum_of_fifty_rappen() {
    // tragbar (0.25 × 12'500 × 12 − 1'200) / 0.0625 = 580'800; es fehlen 0.50
    let r = run(&renewal_plan(580_800.5, two_periods()));
    let n = &r.renewals[0];
    assert_eq!(n.month, ym(2027, 11));
    assert!(!n.affordable);
    assert_eq!((n.lump_sum, n.debt_before, n.debt_after), (0.5, 580_800.5, 580_800.0));
    assert_eq!(r.summary.lump_sums, 1);
    // Mit 0.50 weniger Schuld ist sie tragbar.
    let r = run(&renewal_plan(580_800.0, two_periods()));
    assert!(r.renewals[0].affordable && r.renewals[0].lump_sum == 0.0);
    assert_eq!(r.summary.lump_sums, 0);
}

#[test]
fn renewal_rate_is_weighted_by_the_balances() {
    let mut v = renewal_plan(300_000.0, two_periods());
    v["realestate"]["properties"][0]["mortgage"]["tranches"] = json!([
        {"name": "T1", "amount": 300_000, "start": "2025-11", "periods": two_periods()},
        {"name": "T2", "amount": 100_000, "start": "2025-11",
         "periods": [{"type": "fixed", "term": 2, "rate": 0.01}, {"type": "fixed", "term": 1, "rate": 0.05}, {"type": "fixed", "term": 10, "rate": 0.06}]}]);
    let r = run(&v);
    let e = r
        .property_events
        .iter()
        .find(|e| e.month == ym(2027, 11) && e.rate.is_some())
        .unwrap();
    // (300'000 × 3 % + 100'000 × 5 %) / 400'000
    assert!((e.rate.unwrap() - 0.035).abs() < 1e-15, "{:?}", e.rate);
    assert!(r
        .events
        .iter()
        .any(|x| x.kind == EventKind::MortgageRenewal && x.text.contains("Haus (T2): 5.00 %")));
}

#[test]
fn renewal_without_any_affordable_debt_repays_everything() {
    // Kein Einkommen: tragbar 0, die ganze Schuld wird aus dem Vermögen bezahlt; Satz der leeren Hypothek 0.
    let mut v = renewal_plan(200_000.0, two_periods());
    v["flows"]["income"] = json!(0);
    v["pension"]["pk"]["capital"] = json!(0);
    v["pension"]["ahv"]["monthly"] = json!(0);
    let r = run(&v);
    let n = &r.renewals[0];
    assert_eq!((n.lump_sum, n.debt_after), (200_000.0, 0.0));
    let e = r
        .property_events
        .iter()
        .find(|e| e.month == ym(2027, 11) && e.rate.is_some())
        .unwrap();
    assert_eq!(e.rate, Some(0.0));
}

#[test]
fn renewal_fails_when_the_lump_sum_exceeds_the_wealth() {
    // Vermögen etwa 33'000 (30'000 plus wenig Überschuss), nötig wären 100'000
    let mut v = renewal_plan(680_800.0, two_periods());
    v["wealth"]["cash"] = json!(30_000);
    v["flows"]["spending"] = json!(11_000);
    let r = run(&v);
    let n = &r.renewals[0];
    assert_eq!(n.emergency_sale.as_deref(), Some("Vermögen < 0"));
    assert_eq!((n.lump_sum, n.debt_after), (0.0, 680_800.0));
    assert_eq!(r.summary.affordability_sales, 1);
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == EventKind::EmergencySale && e.month == ym(2027, 12)));
}

/// Wie `renewal_plan` mit Teuerung 12 %, Nebenkosten mit Teuerung, Amortisation 300 pro Quartal und Ausgaben 20'000.
fn criterion_plan(emergency_months: f64) -> Value {
    let mut v = renewal_plan(700_000.0, two_periods());
    v["returns"]["fixed"]["inflation"] = json!(0.12);
    v["flows"]["spending"] = json!(20_000);
    v["wealth"]["cash"] = json!(400_000);
    v["realestate"]["emergencyMonths"] = json!(emergency_months);
    v["realestate"]["properties"][0]["inflation"] = json!(true);
    for p in v["realestate"]["properties"][0]["mortgage"]["tranches"][0]["periods"]
        .as_array_mut()
        .unwrap()
    {
        p["amort"] = json!(300);
    }
    v
}

#[test]
fn renewal_checks_the_emergency_criterion_after_the_payment() {
    // Erster Lauf ohne Kriterium: Vermögen vor der Erneuerung, Teuerung und Schuld messen.
    let r = run(&criterion_plan(0.0));
    let n = &r.renewals[0];
    let pi = rec(&r, 2027, 11).price_index;
    assert!(pi > 1.05);
    let liquid = rec(&r, 2027, 10).liquid;
    // Nebenkosten 100 pro Monat mit Teuerung; tragbar = (37'500 − 1'200 × Teuerung) / 0.0625
    let max = (0.25 * 12_500.0 * 12.0 - 1200.0 * pi) / 0.0625;
    let lump = n.debt_before - max;
    assert!((n.lump_sum - lump).abs() < 1e-6, "{} statt {lump}", n.lump_sum);
    // Fehlbetrag pro Monat nach der Zahlung: Ausgaben + Nebenkosten + Zins 3 % auf der neuen Schuld + Amortisation − Einkommen
    let shortfall = 20_000.0 * pi + 100.0 * pi + max * 0.03 / 12.0 + 100.0 - 12_500.0;
    let critical = (liquid - lump) / shortfall;
    assert!(critical > 1.0, "{critical}");
    let below = run(&criterion_plan(critical * (1.0 - 1e-7)));
    assert_eq!(below.renewals[0].emergency_sale, None);
    assert!(below.renewals[0].lump_sum > 0.0);
    let above = run(&criterion_plan(critical * (1.0 + 1e-7)));
    assert_eq!(
        above.renewals[0].emergency_sale.as_deref(),
        Some("Notverkaufskriterium")
    );
}

/// Tranche «T0» über 600 mit direkter Amortisation 300 pro Quartal (Dezember 2026 und März 2027): ab März 2027
/// abbezahlt, die Amortisation bleibt aber in jeder Periode erfasst. Perioden wie `two_periods` (neue Periode 2027-11).
fn paid_tranche() -> Value {
    let mut periods = two_periods();
    for p in periods.as_array_mut().unwrap() {
        p["amort"] = json!(300);
        p["amortMode"] = json!("direct");
    }
    json!({"name": "T0", "amount": 600, "start": "2025-11", "periods": periods})
}

#[test]
fn paid_off_tranche_does_not_count_in_the_emergency_criterion() {
    // Wie oben, mit einer zusätzlichen abbezahlten Tranche: Ihre Amortisation zählt im geschätzten Fehlbetrag nicht.
    let plan = |emergency_months: f64| {
        let mut v = criterion_plan(emergency_months);
        v["realestate"]["properties"][0]["mortgage"]["tranches"]
            .as_array_mut()
            .unwrap()
            .push(paid_tranche());
        v
    };
    let r = run(&plan(0.0));
    assert_eq!(
        rec(&r, 2027, 6).mortgage,
        rec(&run(&criterion_plan(0.0)), 2027, 6).mortgage,
        "T0 ist abbezahlt"
    );
    let n = &r.renewals[0];
    assert_eq!(n.month, ym(2027, 11));
    let pi = rec(&r, 2027, 11).price_index;
    let liquid = rec(&r, 2027, 10).liquid;
    let max = (0.25 * 12_500.0 * 12.0 - 1200.0 * pi) / 0.0625;
    let lump = n.debt_before - max;
    // Nur die Amortisation von T1 (100 pro Monat), nicht die von T0
    let shortfall = 20_000.0 * pi + 100.0 * pi + max * 0.03 / 12.0 + 100.0 - 12_500.0;
    let critical = (liquid - lump) / shortfall;
    assert_eq!(run(&plan(critical * (1.0 - 1e-7))).renewals[0].emergency_sale, None);
    assert_eq!(
        run(&plan(critical * (1.0 + 1e-7))).renewals[0]
            .emergency_sale
            .as_deref(),
        Some("Notverkaufskriterium")
    );
}

#[test]
fn new_period_of_a_paid_off_tranche_is_no_renewal() {
    // T1 (900'000, Periode bis 2035-11) ist nicht tragbar (tragbar 580'800), wird aber erst 2035 erneuert. Die neue
    // Periode der abbezahlten T0 im November 2027 löst keine Prüfung der Tragbarkeit aus.
    let mut v = renewal_plan(900_000.0, json!([{"type": "fixed", "term": 10, "rate": 0.02}]));
    v["realestate"]["properties"][0]["mortgage"]["tranches"]
        .as_array_mut()
        .unwrap()
        .push(paid_tranche());
    let r = run(&v);
    assert_eq!(rec(&r, 2027, 6).mortgage, 900_000.0, "T0 ist abbezahlt");
    assert_eq!(
        r.renewals.first().map(|n| n.month),
        Some(ym(2035, 11)),
        "{:?}",
        r.renewals.iter().map(|n| n.month).collect::<Vec<_>>()
    );
}

/// Kette von Verkäufen in der 3-Topf-Strategie mit Phase 8: A (900'000, ohne Schuld) wird im Januar 2030 verkauft, B
/// (500'000, Hypothek 400'000) im Februar 2030 gekauft; C (9 Mio., nicht finanzierbar) soll nach dem Verkauf von B
/// folgen. Einmalausgaben im Januar und Februar 2030 leeren die Töpfe: Phase 8 beginnt im Verkaufsmonat von A und dauert
/// nach dem Kauf von B an (Topf 1 −1000). Einkommen 12'500, Ausgaben 12'500 und ab Februar 2030 18'500 pro Monat.
fn phase8_chain() -> Value {
    let mut v = estate(json!([
        owned("A", "2030-01", json!({})),
        {"id": "B", "name": "B", "status": "planned", "price": 500_000, "buy": "afterSale:A", "sell": "never", "fixCosts": 0,
         "inflation": false, "mortgage": {"tranches": [{"name": "T", "amount": 400_000, "periods": [{"type": "fixed", "term": 10, "rate": 0.0}]}]}},
        {"id": "C", "name": "C", "status": "planned", "price": 9_000_000, "buy": "afterSale:B", "fixCosts": 0, "inflation": false, "mortgage": null},
    ]));
    v["flows"]["spending"] =
        json!({"mode": "timed", "rows": [{"from": "start", "value": 12_500}, {"from": "2030-02", "value": 18_500}]});
    v["wealth"]["cash"] = json!(50_000);
    v["flows"]["extras"] = json!([{"date": "2030-01", "amount": -300_000, "label": "Ausgabe 1"},
                                  {"date": "2030-02", "amount": -540_000, "label": "Ausgabe 2"}]);
    buckets(&mut v, 1.0, 1.0, -1.0);
    v
}

#[test]
fn restart_after_phase_eight_does_not_count_the_reserved_proceeds() {
    // Im März 2030 wird B notverkauft (Vermögen negativ); der Erlös von 100'000 ist für C reserviert. Topf 1 ist mit dem
    // Erlös positiv, ohne ihn negativ: Phase 8 endet erst im April, wenn der Kauf von C entfallen und die Reservierung
    // aufgelöst ist.
    let r = run(&phase8_chain());
    let phase8: Vec<_> = r
        .events
        .iter()
        .filter(|e| e.text.contains("alle Töpfe leer"))
        .map(|e| e.month)
        .collect();
    assert_eq!(phase8.first(), Some(&ym(2030, 1)));
    assert_eq!(rec(&r, 2030, 2).cash, -1000.0);
    let b = r
        .property_events
        .iter()
        .find(|e| e.property == "B" && e.kind == engine::sim::PropertyEventKind::EmergencySale)
        .expect("Notverkauf B");
    assert_eq!(
        (b.month, b.net, b.note.as_deref()),
        (ym(2030, 3), Some(100_000.0), Some(RESERVED))
    );
    let reserved_month = rec(&r, 2030, 3).cash;
    assert!(reserved_month > 0.0 && reserved_month < 100_000.0, "{reserved_month}");
    let restart: Vec<_> = r
        .bucket_refills
        .iter()
        .filter(|x| x.reason.contains("Neustart nach Phase 8"))
        .map(|x| x.month)
        .collect();
    assert_eq!(restart.first(), Some(&ym(2030, 4)), "{restart:?}");
}

// ---------------------------------------------------------------------------------------------
// Immobilien: Kauf, Verkauf, Folgekauf, Miete, Amortisation, Säule 3a
// ---------------------------------------------------------------------------------------------

/// Wie `renewal_plan` (Einkommen 12'500 pro Monat ab 2027), mit den angegebenen Immobilien.
fn estate(props: Value) -> Value {
    let mut v = renewal_plan(0.0, json!([]));
    v["realestate"]["properties"] = props;
    v
}

fn planned(price: f64, buy: &str, mortgage: Value) -> Value {
    json!({"id": "neu", "name": "Neu", "status": "planned", "price": price, "buy": buy, "fixCosts": 0, "inflation": false, "mortgage": mortgage})
}

fn owned(id: &str, sell: &str, extra: Value) -> Value {
    let mut p = json!({"id": id, "name": id, "status": "owned", "price": 600_000, "worth": 900_000, "sell": sell, "fixCosts": 0, "inflation": false});
    for (k, x) in extra.as_object().unwrap() {
        p[k] = x.clone();
    }
    p
}

fn purchase(r: &SimResult) -> &engine::sim::PropertyEvent {
    r.property_events.iter().find(|e| e.price.is_some()).expect("Kauf")
}

#[test]
fn purchase_with_highest_loan_is_limited_by_affordability() {
    // Nebenkosten 1 % von 800'000 = 8'000; tragbar (37'500 − 8'000) / 0.0625 = 472'000 (statt 640'000), Rest aus dem Vermögen.
    let half = json!({"name": "A", "amount": 0, "periods": [{"type": "fixed", "term": 10, "rate": 0.02}]});
    let mut b = half.clone();
    b["name"] = json!("B");
    let r = run(&estate(json!([planned(
        800_000.0,
        "2030-01",
        json!({"amountMode": "maxLoan", "tranches": [half, b]})
    )])));
    let e = purchase(&r);
    assert_eq!(
        (e.month, e.price, e.mortgage, e.equity),
        (ym(2030, 1), Some(800_000.0), Some(472_000.0), Some(328_000.0))
    );
    // Ohne Beträge je Tranche: je die Hälfte
    assert_eq!(rec(&r, 2030, 1).mortgage, 472_000.0);
}

#[test]
fn purchase_prices_follow_inflation() {
    let mortgage = json!({"amountMode": "maxLoan", "tranches": [{"name": "A", "amount": 1, "periods": [{"type": "fixed", "term": 10, "rate": 0.02}]}]});
    let mut v = estate(json!([planned(800_000.0, "2030-01", mortgage)]));
    v["returns"]["fixed"]["inflation"] = json!(0.12);
    v["realestate"]["properties"][0]["inflation"] = json!(true);
    let r = run(&v);
    let pi = rec(&r, 2030, 1).price_index;
    let e = purchase(&r);
    assert!((e.price.unwrap() - 800_000.0 * pi).abs() < 1e-6);
    // Einkommen im Kaufmonat (AHV inzwischen mit der Teuerung angepasst)
    let income = 12.0 * rec(&r, 2030, 1).income;
    assert!(income > 150_000.0);
    assert!((e.mortgage.unwrap() - (0.25 * income - 0.01 * 800_000.0 * pi) / 0.0625).abs() < 1e-6);
    // Kauf mit erfassten Beträgen (Datum fest): ebenfalls zum teuerungsbereinigten Preis
    let fixed = json!({"tranches": [{"name": "A", "amount": 400_000, "periods": [{"type": "fixed", "term": 10, "rate": 0.02}]}]});
    let mut v = estate(json!([planned(800_000.0, "2030-01", fixed)]));
    v["returns"]["fixed"]["inflation"] = json!(0.12);
    v["realestate"]["properties"][0]["inflation"] = json!(true);
    let e = run(&v);
    let e = purchase(&e);
    assert!((e.price.unwrap() - 800_000.0 * pi).abs() < 1e-6);
    assert_eq!(e.mortgage, Some(400_000.0));
}

fn sale(r: &SimResult) -> &engine::sim::PropertyEvent {
    r.property_events
        .iter()
        .find(|e| e.proceeds.is_some())
        .expect("Verkauf")
}

const RESERVED: &str = "Verkaufserlös reserviert für Kauf im Folgemonat";

#[test]
fn sale_followed_by_a_purchase_keeps_the_proceeds_liquid() {
    let mortgage = json!({"tranches": [{"name": "T", "amount": 300_000, "start": "2025-11", "periods": [{"type": "fixed", "term": 10, "rate": 0.0}]}]});
    let alt = owned("alt", "2030-01", json!({"mortgage": mortgage}));
    let mut v = estate(json!([alt, planned(500_000.0, "2030-02", json!(null))]));
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0.1]]});
    let r = run(&v);
    let s = sale(&r);
    let net = s.proceeds.unwrap() - s.debt_repaid.unwrap() - s.gain_tax.unwrap();
    assert!(s.gain_tax.unwrap() > 0.0 && net > 0.0);
    assert_eq!(s.net, Some(net));
    assert_eq!(s.note.as_deref(), Some(RESERVED));
    // Der reservierte Erlös bleibt als Bargeld (Reserve 0 Monate)
    assert!(
        (rec(&r, 2030, 1).cash - net).abs() < 1e-6,
        "{} statt {net}",
        rec(&r, 2030, 1).cash
    );
}

fn note_after_sale(props: Value) -> Option<String> {
    sale(&run(&estate(props))).note.clone()
}

#[test]
fn follow_up_purchase_after_a_specific_or_the_last_sale() {
    let alt = owned("alt", "2030-01", json!({}));
    // nach Verkauf von «alt»
    assert_eq!(
        note_after_sale(json!([alt.clone(), planned(500_000.0, "afterSale:alt", json!(null))])).as_deref(),
        Some(RESERVED)
    );
    // nach Verkauf einer anderen Immobilie: kein Folgekauf
    let other = owned("andere", "never", json!({}));
    assert_ne!(
        note_after_sale(json!([
            alt.clone(),
            other.clone(),
            planned(500_000.0, "afterSale:andere", json!(null))
        ]))
        .as_deref(),
        Some(RESERVED)
    );
    // nach dem letzten Verkauf; eine spätere Miete zählt nicht als Besitz
    let rent = json!({"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "2050-01", "rent": 1000});
    assert_eq!(
        note_after_sale(json!([alt.clone(), rent, planned(500_000.0, "afterSale", json!(null))])).as_deref(),
        Some(RESERVED)
    );
    // … solange eine andere Immobilie im Besitz bleibt, aber nicht
    assert_ne!(
        note_after_sale(json!([
            alt.clone(),
            other,
            planned(500_000.0, "afterSale", json!(null))
        ]))
        .as_deref(),
        Some(RESERVED)
    );
}

#[test]
fn purchase_after_an_emergency_sale_follows_in_the_next_month() {
    // Notverkaufskriterium sofort erfüllt ⇒ Notverkauf im zweiten Monat (Dezember 2026), Kauf im Januar 2027
    let mut v = estate(json!([
        owned("alt", "never", json!({})),
        planned(500_000.0, "afterSale:alt", json!(null))
    ]));
    v["realestate"]["emergencyMonths"] = json!(1000);
    v["flows"]["income"] = json!(0);
    let r = run(&v);
    assert_eq!(sale(&r).month, ym(2026, 12));
    assert_eq!(purchase(&r).month, ym(2027, 1));
}

#[test]
fn failed_purchase_after_sale_starts_the_rent() {
    let rent = json!({"name": "Wohnung", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "afterSale", "rent": 1500});
    let r = run(&estate(json!([
        owned("alt", "2030-01", json!({})),
        planned(9_000_000.0, "afterSale:alt", json!(null)),
        rent
    ])));
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == EventKind::Purchase && e.text.contains("Kauf Neu entfällt")));
    let start = r.events.iter().find(|e| e.kind == EventKind::RentStart).unwrap();
    assert_eq!(start.text, "Miete ab jetzt: Wohnung");
}

#[test]
fn failed_purchase_does_not_start_the_rent_while_another_property_is_owned() {
    let rent = json!({"name": "Wohnung", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "afterSale", "rent": 1500});
    let props = json!([
        owned("alt", "2030-01", json!({})),
        owned("andere", "never", json!({})),
        planned(9_000_000.0, "afterSale:alt", json!(null)),
        rent
    ]);
    let r = run(&estate(props));
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == EventKind::Purchase && e.text.contains("Kauf Neu entfällt")));
    assert!(!r.events.iter().any(|e| e.kind == EventKind::RentStart));
}

#[test]
fn lowest_liquidity_is_dated_to_the_first_month() {
    // Einkommen = Ausgaben: Das Vermögen bleibt gleich, der tiefste Stand gilt ab dem ersten Monat.
    let mut v = base();
    v["flows"]["income"] = json!(1000);
    v["pension"]["pk"]["capital"] = json!(0);
    v["pension"]["ahv"]["monthly"] = json!(0);
    let r = run(&v);
    assert!(
        r.records.iter().all(|x| x.liquid == 1_000_000.0),
        "{:?}",
        r.records.iter().map(|x| x.liquid).collect::<Vec<_>>()
    );
    assert_eq!(r.summary.min_liquid_month, Some(ym(2026, 11)));
}

#[test]
fn sale_without_proceeds_does_not_apply_the_strategy() {
    // Wert = Schuld, kein Gewinn: Erlös 0 ⇒ keine Anwendung der Entnahmestrategie im März
    let mortgage = json!({"tranches": [{"name": "T", "amount": 900_000, "start": "2025-11", "periods": [{"type": "fixed", "term": 10, "rate": 0.0}]}]});
    let mut alt = owned("alt", "2030-03", json!({"mortgage": mortgage}));
    alt["price"] = json!(900_000);
    let mut v = estate(json!([alt]));
    v["withdrawal"]["strategy"] = json!("fixed_rate");
    let r = run(&v);
    assert_eq!(sale(&r).net, Some(0.0));
    assert!(!r
        .events
        .iter()
        .any(|e| e.month == ym(2030, 3) && e.text.starts_with("Entnahmestrategie angewendet")));
}

#[test]
fn rent_ends_before_the_until_month() {
    let rent = json!({"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "start", "until": "2030-01", "rent": 2000});
    let r = run(&estate(json!([rent])));
    assert_eq!(rec(&r, 2029, 12).housing, 2000.0);
    assert_eq!(rec(&r, 2030, 1).housing, 0.0);
}

/// Haus im Besitz mit Nebenkosten 50, Hypothek 600 mit direkter Amortisation 300 pro Quartal (nach zwei Quartalen
/// abbezahlt), Rückzahlung der Restschuld 2028-11; Reserve 1 Monat, alles in Aktien.
fn amortized() -> Value {
    let mut v = base();
    // FEA-40: Die Reserve der Strategie gilt erst ab der Pensionierung; hier beginnt sie mit dem Start.
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Bezug Pensionskasse", "date": "2026-11"});
    v["wealth"]["stockShare"] = json!(1);
    v["wealth"]["reserveMonths"] = json!(1);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    let mortgage = json!({"tranches": [{"name": "T", "amount": 600, "start": "2026-11", "after": "repay",
        "periods": [{"type": "fixed", "term": 2, "rate": 0.0, "amort": 300, "amortMode": "direct"}]}]});
    v["realestate"]["properties"] = json!([owned("haus", "never", json!({"fixCosts": 50, "mortgage": mortgage}))]);
    v
}

#[test]
fn running_costs_include_amortization_only_while_there_is_debt() {
    let r = run(&amortized());
    // Reserve = Ausgaben 1000 + Nebenkosten 50 + Amortisation 100 pro Monat, solange Schuld besteht
    close(rec(&r, 2026, 12).cash, 1150.0, "mit Amortisation");
    assert_eq!(rec(&r, 2027, 6).mortgage, 0.0);
    close(rec(&r, 2027, 8).cash, 1050.0, "ohne Schuld");
    // Keine Rückzahlung von 0
    assert!(!r.events.iter().any(|e| e.kind == EventKind::MortgageRepayment));
}

#[test]
fn pillar3a_collects_indirect_amortization_and_pays_out() {
    let mut v = base();
    v["taxes"]["capital"] = json!([[0, 0.05]]);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    let mortgage = json!({"tranches": [{"name": "T", "amount": 100_000, "start": "2026-11",
        "periods": [{"type": "fixed", "term": 10, "rate": 0.0, "amort": 300, "amortMode": "indirect"}]}],
        "pillar3a": {"balance": 0, "rate": 0, "payout": "2028-01", "use": "wealth"}});
    let empty = json!({"tranches": [{"name": "T", "amount": 100_000, "start": "2026-11",
        "periods": [{"type": "fixed", "term": 10, "rate": 0.0}]}],
        "pillar3a": {"balance": 0, "rate": 0, "payout": "2028-01", "use": "wealth"}});
    v["realestate"]["properties"] = json!([
        owned("haus", "never", json!({"mortgage": mortgage})),
        owned("leer", "never", json!({"mortgage": empty}))
    ]);
    let r = run(&v);
    // Dezember 2026, März, Juni, September, Dezember 2027: fünf Einzahlungen
    assert_eq!(rec(&r, 2026, 12).pillar3a, 300.0);
    assert_eq!(rec(&r, 2027, 12).pillar3a, 1500.0);
    // Das leere Konto ohne indirekte Amortisation entfällt beim Übernehmen (FEA-37); im Januar 2028 eine Auszahlung.
    let payouts: Vec<_> = r
        .events
        .iter()
        .filter(|e| e.kind == EventKind::Pillar3aPayout && e.month == ym(2028, 1))
        .collect();
    assert_eq!(payouts.len(), 1, "keine Auszahlung ohne Guthaben");
    close(rec(&r, 2028, 1).tax_capital, 0.05 * 1500.0, "Kapitalsteuer");
}

// ---------------------------------------------------------------------------------------------
// Monatsablauf
// ---------------------------------------------------------------------------------------------

#[test]
fn initial_split_reserves_the_running_costs() {
    // Haus: Nebenkosten 1000, Hypothek 120'000 (Periode ab 2026-11 zu 2 %, vorher 5 %), Amortisation 300 pro Quartal.
    // Lücke = Ausgaben 2000 + (1000 + 200 Zins + 100 Amortisation) − Erwerb 500 = 2800; Reserve 12 Monate = 33'600.
    // Eine geplante Immobilie (noch nicht gekauft) zählt nicht.
    let mut v = base();
    // FEA-40: Die Reserve der Strategie gilt erst ab der Pensionierung; hier beginnt sie mit dem Start.
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Bezug Pensionskasse", "date": "2026-11"});
    v["flows"]["spending"] = json!(2000);
    v["flows"]["income"] = json!(500);
    v["wealth"]["stockShare"] = json!(1);
    v["wealth"]["reserveMonths"] = json!(12);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    let mortgage = json!({"tranches": [{"name": "T", "amount": 120_000, "start": "2016-11",
        "periods": [{"type": "fixed", "term": 10, "rate": 0.05, "amort": 300}, {"type": "fixed", "term": 10, "rate": 0.02, "amort": 300},
                    {"type": "fixed", "term": 10, "rate": 0.01, "amort": 0}]}]});
    let mut later = planned(500_000.0, "2040-01", json!(null));
    later["fixCosts"] = json!(5000);
    v["realestate"]["properties"] = json!([
        owned("haus", "never", json!({"fixCosts": 1000, "mortgage": mortgage})),
        later
    ]);
    // Aktien +10 % im ersten Monat: nur der angelegte Teil wächst.
    let r = run_with(&v, |p, _| p.stocks[0] = 0.10);
    let expected = (1_000_000.0 - 33_600.0) * 1.1 + 33_600.0 + 500.0 - 2000.0 - 1000.0;
    close(rec(&r, 2026, 11).liquid, expected, "Ende November 2026");
}

#[test]
fn zero_extra_does_not_apply_the_strategy() {
    let mut v = base();
    v["withdrawal"]["strategy"] = json!("fixed_rate");
    v["flows"]["extras"] = json!([{"date": "2032-03", "amount": 0, "label": "Nichts"}, {"date": "2032-05", "amount": 1000, "label": "Etwas"}]);
    let r = run(&v);
    let applied = |m: YearMonth| {
        r.events
            .iter()
            .any(|e| e.month == m && e.text.starts_with("Entnahmestrategie angewendet"))
    };
    assert!(!applied(ym(2032, 3)));
    assert!(applied(ym(2032, 5)));
}

#[test]
fn dynamic_living_adds_savings_and_subtracts_housing() {
    // Fixe Rate 4 % auf 820'000 (60 Monate zu 1000 + 2000 Miete); Konsum = Budget/12 + Sparbetrag 300 − Miete 2000
    let mut v = base();
    v["withdrawal"]["strategy"] = json!("fixed_rate");
    v["flows"]["savings"] = json!(300);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    v["realestate"]["properties"] = json!([{"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "start", "rent": 2000}]);
    let r = run(&v);
    close(
        rec(&r, 2031, 10).liquid,
        1_000_000.0 - 60.0 * 3000.0 + 60.0 * 300.0,
        "vor der Pensionierung",
    );
    let budget = 0.04 * rec(&r, 2031, 10).liquid;
    // Die Ausgaben im Protokoll enthalten die Miete: Konsum (Budget/12 + 300 − 2000) + 2000
    close(rec(&r, 2031, 11).spending, budget / 12.0 + 300.0, "Ausgaben");
    close(rec(&r, 2031, 11).housing, 2000.0, "Miete");
}

#[test]
fn indices_lowest_liquidity_and_depletion() {
    let mut v = base();
    v["flows"]["spending"] = json!(0);
    let r = run_with(&v, |p, _| p.bonds.iter_mut().for_each(|x| *x = 0.01));
    close(rec(&r, 2027, 1).bond_index, 1.01f64.powi(3), "Obligationenindex");
    // Gleichbleibende Cash: tiefster Stand im ersten Monat
    assert_eq!(r.summary.min_liquid_month, Some(ym(2026, 11)));

    // Schulden bei gedeckten Ausgaben: aufgebraucht ab Start
    let mut v = base();
    v["wealth"]["cash"] = json!(-10_000);
    v["flows"]["income"] = json!(2000);
    let r = run(&v);
    assert_eq!(r.summary.depleted, Some(ym(2026, 11)));
}

#[test]
fn lump_sum_after_tax_goes_to_pot_three() {
    // Person B bezieht im Juni 2032 die Hälfte von 600'000 als Kapital (Steuer 5 %), während die Töpfe schon laufen.
    let mut v = couple();
    v["partner"]["pk"] = json!({"capital": 600_000, "at": "2032-06", "lumpsum": 0.5, "conversion": 0.05});
    v["taxes"]["capital"] = json!([[0, 0.05]]);
    let r = run(&v);
    assert_eq!(r.summary.lumpsum_net, 285_000.0);
    let b = refill(&r, "B", 2032, 6);
    assert_eq!(
        b.moves.iter().map(|m| (m.from, m.to, m.amount)).collect::<Vec<_>>(),
        vec![(0, 3, 285_000.0)]
    );
    assert!(refill(&r, "A", 2032, 6).reason.contains("PK-Kapitalbezug"));
    // Ohne Kapitalbezug kein Anlass «PK-Kapitalbezug»
    let r = run(&couple());
    assert!(!r.bucket_refills.iter().any(|x| x.reason.contains("PK-Kapitalbezug")));
}

// ---------------------------------------------------------------------------------------------
// 3-Topf-Strategie: Januar-Auffüllung, Anlass A, Reservierung
// ---------------------------------------------------------------------------------------------

fn spending_from_2032(v: &mut Value, before: f64, after: f64) {
    v["flows"]["spending"] =
        json!({"mode": "timed", "rows": [{"from": "start", "value": before}, {"from": "2032-01", "value": after}]});
}

#[test]
fn january_refill_moves_surplus_to_pot_three() {
    // Bedarf sinkt im Januar 2032 von 36'000 auf 12'000: Topf 1 und 2 haben zu viel, der Überschuss geht in Topf 3.
    let mut v = base();
    spending_from_2032(&mut v, 3000.0, 1000.0);
    buckets(&mut v, 2.0, 2.0, -1.0);
    let r = run(&v);
    let j = refill(&r, "Januar", 2032, 1);
    let moved = |from: u8, to: u8| {
        j.moves
            .iter()
            .filter(|m| m.from == from && m.to == to)
            .map(|m| m.amount)
            .sum::<f64>()
    };
    close(moved(1, 3), j.before[0] - j.target1, "Überschuss Topf 1");
    close(moved(2, 3), j.before[1] - j.target2, "Überschuss Topf 2");
    close(j.after[0], j.target1, "Topf 1");
    close(j.after[1], j.target2, "Topf 2");
}

#[test]
fn january_refill_fills_pot_two_from_stocks() {
    // Bedarf steigt im Januar 2032 von 12'000 auf 36'000: Topf 1 zuerst aus Topf 2, dann Topf 2 aus Topf 3.
    let mut v = base();
    spending_from_2032(&mut v, 1000.0, 3000.0);
    buckets(&mut v, 1.0, 5.0, -1.0);
    let r = run(&v);
    let j = refill(&r, "Januar", 2032, 1);
    assert!(
        j.before[1] > j.target1 - j.before[0],
        "Topf 2 deckt die Lücke von Topf 1"
    );
    close(j.after[0], j.target1, "Topf 1");
    close(j.after[1], j.target2, "Topf 2");
}

#[test]
fn start_in_january_needs_no_extrapolation() {
    let mut v = base();
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2032-01"});
    buckets(&mut v, 1.0, 1.0, -1.0);
    let r = run(&v);
    let a = refill(&r, "A", 2032, 1);
    assert_eq!(a.basis.source, "ab 01.2032");
    assert_eq!(a.basis.months, 12);
}

#[test]
fn sale_proceeds_for_a_purchase_are_reserved() {
    // Pensioniert, 3-Topf; Verkauf März 2033 (Erlös 900'000), Kauf April 2033 für 300'000 ohne Hypothek.
    let mut v = base();
    buckets(&mut v, 1.0, 1.0, -1.0);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    v["realestate"]["properties"] = json!([
        owned("alt", "2033-03", json!({})),
        planned(300_000.0, "2033-04", json!(null))
    ]);
    let r = run(&v);
    let s = sale(&r);
    assert_eq!((s.month, s.net), (ym(2033, 3), Some(900_000.0)));
    assert_eq!(s.note.as_deref(), Some(RESERVED));
    let into = refill(&r, "B", 2033, 3);
    assert_eq!(
        into.moves.iter().map(|m| (m.from, m.to, m.amount)).collect::<Vec<_>>(),
        vec![(0, 4, 900_000.0)]
    );
    // Der Verkaufserlös verändert die Töpfe im Verkaufsmonat nicht
    assert_eq!(into.before, into.after);
    // Nach dem Kauf: Rest der Reservierung nach Topf 3
    let rest = r
        .bucket_refills
        .iter()
        .find(|x| x.month == ym(2033, 4) && x.moves.iter().any(|m| m.from == 4))
        .expect("Reservierung → Topf 3");
    assert_eq!(
        rest.moves.iter().map(|m| (m.from, m.to, m.amount)).collect::<Vec<_>>(),
        vec![(4, 3, 600_000.0)]
    );
    assert!(rest.reason.contains("Überschuss der Reservierung"));
    // Vermögen bleibt erhalten: Cash = Vormonat + Erlös − Kaufpreis − Ausgaben
    close(
        rec(&r, 2033, 4).liquid,
        rec(&r, 2033, 2).liquid + 900_000.0 - 300_000.0 - 2.0 * 1000.0,
        "Cash",
    );
    for x in &r.records {
        assert!(
            x.cash >= -1e-6 && x.bonds >= -1e-6 && x.stocks >= -1e-6,
            "{:?}",
            x.month
        );
    }
}

#[test]
fn inflows_before_the_first_bucket_step_are_not_logged_separately() {
    // Zufluss und Verkauf mit Folgekauf im Monat der Pensionierung: Anlass A sammelt alles, kein eigener Eintrag B.
    let mut v = base();
    buckets(&mut v, 1.0, 1.0, -1.0);
    v["flows"]["extras"] = json!([{"date": "2031-11", "amount": 50_000, "label": "Erbe"}]);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    v["realestate"]["properties"] = json!([
        owned("alt", "2031-11", json!({})),
        planned(300_000.0, "2031-12", json!(null))
    ]);
    let r = run(&v);
    assert!(!r
        .bucket_refills
        .iter()
        .any(|x| x.month == ym(2031, 11) && x.trigger == "B"));
}

#[test]
fn inflow_to_pot_three_is_logged_with_the_pots_before() {
    let mut v = base();
    buckets(&mut v, 1.0, 1.0, -1.0);
    v["flows"]["extras"] = json!([{"date": "2033-03", "amount": 50_000, "label": "Erbe"}]);
    let r = run(&v);
    let b = refill(&r, "B", 2033, 3);
    // Stände vor den Ausgaben des Monats (ohne Einnahmen gleich dem Vormonatsende)
    close(b.before[0], rec(&r, 2033, 2).cash, "Topf 1");
    close(b.before[2], rec(&r, 2033, 2).stocks, "Topf 3");
    close(b.after[2], b.before[2] + 50_000.0, "Topf 3 nachher");
    assert!(r
        .events
        .iter()
        .any(|e| e.month == ym(2033, 3) && e.text.starts_with("3-Topf extern nach Topf 3 («Erbe»)")));
}

#[test]
fn reservation_used_up_completely_leaves_nothing_for_pot_three() {
    let mut v = base();
    buckets(&mut v, 1.0, 1.0, -1.0);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    v["realestate"]["properties"] = json!([
        owned("alt", "2033-03", json!({})),
        planned(900_000.0, "2033-04", json!(null))
    ]);
    let r = run(&v);
    assert!(!r.bucket_refills.iter().any(|x| x.moves.iter().any(|m| m.from == 4)));
}

/// 3-Topf ohne Cash-Topf: Topf 1 deckt die Ausgaben jeden Monat erst nach der Vorausschau C.
fn lean_buckets() -> Value {
    let mut v = base();
    buckets(&mut v, 0.0, 1.0, -1.0);
    v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
    v
}

fn c_moves(r: &SimResult, m: YearMonth) -> Vec<(String, Vec<(u8, u8, i64)>)> {
    r.bucket_refills
        .iter()
        .filter(|x| x.month == m && x.trigger.starts_with('C'))
        .map(|x| {
            (
                x.trigger.clone(),
                x.moves
                    .iter()
                    .map(|y| (y.from, y.to, (y.amount * 100.0).round() as i64))
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn reservation_does_not_change_the_pots() {
    // Mit Verkauf (Erlös reserviert) und Kauf im Folgemonat wie ohne Immobilien: dieselben Umbuchungen der Vorausschau C.
    let plain = run(&lean_buckets());
    let mut v = lean_buckets();
    v["realestate"]["properties"] = json!([
        owned("alt", "2033-03", json!({})),
        planned(300_000.0, "2033-04", json!(null))
    ]);
    let with_sale = run(&v);
    for m in [ym(2033, 2), ym(2033, 3)] {
        assert!(!c_moves(&plain, m).is_empty());
        assert_eq!(c_moves(&with_sale, m), c_moves(&plain, m), "{m:?}");
    }
}

#[test]
fn equity_beyond_the_reservation_is_a_one_off_payment() {
    // Erlös 900'000 reserviert, Kauf für 1'000'000: 100'000 kommen aus den Töpfen und zählen als einmalige Ausgabe.
    let mut v = lean_buckets();
    v["realestate"]["properties"] = json!([
        owned("alt", "2033-03", json!({})),
        planned(1_000_000.0, "2033-04", json!(null))
    ]);
    let r = run(&v);
    let c = r
        .bucket_refills
        .iter()
        .find(|x| x.month == ym(2033, 4) && x.trigger.starts_with('C'))
        .unwrap();
    close(c.payment, 1000.0 + 100_000.0, "Zahlung des Monats");
    pots_follow_the_moves(&r);
}

/// Jede protokollierte Auffüllung: Töpfe nachher = vorher + Zuflüsse − Abflüsse (Topf 1/2/3 = Index 0/1/2; 0 = extern,
/// 4 = Reservierung zählen nicht als Topf).
fn pots_follow_the_moves(r: &SimResult) {
    assert!(
        r.bucket_refills.iter().any(|x| x.trigger == "C2"),
        "keine C2-Auffüllung"
    );
    for x in &r.bucket_refills {
        let mut pots = x.before;
        for mv in &x.moves {
            if (1..=3).contains(&mv.from) {
                pots[mv.from as usize - 1] -= mv.amount;
            }
            if (1..=3).contains(&mv.to) {
                pots[mv.to as usize - 1] += mv.amount;
            }
        }
        for i in 0..3 {
            close(
                x.after[i],
                pots[i],
                &format!("{} {:?} Topf {}", x.trigger, x.month, i + 1),
            );
        }
    }
}

#[test]
fn lump_sums_and_repayments_are_one_off_payments() {
    // Erneuerung mit Einmalzahlung 0.50 bzw. Rückzahlung der Restschuld: beide in der Zahlung des Monats.
    // Ausgaben 20'000 über dem Einkommen von 12'500, damit Topf 1 jeden Monat nachgefüllt wird.
    let mut v = renewal_plan(580_800.5, two_periods());
    v["flows"]["spending"] = json!(20_000);
    buckets(&mut v, 0.0, 1.0, -1.0);
    let r = run(&v);
    let c = r
        .bucket_refills
        .iter()
        .find(|x| x.month == ym(2027, 11) && x.trigger.starts_with('C'))
        .unwrap();
    // Ausgaben 20'000 + Nebenkosten 100 + Zins (im November nicht fällig) + Einmalzahlung 0.50
    close(c.payment, 20_000.0 + 100.0 + 0.5, "Zahlung mit Einmalzahlung");

    let mut v = amortized();
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2027-01"});
    v["realestate"]["properties"][0]["mortgage"]["tranches"][0]["periods"][0]["amort"] = json!(0);
    buckets(&mut v, 0.0, 1.0, -1.0);
    let r = run(&v);
    let c = r
        .bucket_refills
        .iter()
        .find(|x| x.month == ym(2028, 11) && x.trigger.starts_with('C'))
        .unwrap();
    close(c.payment, 1000.0 + 50.0 + 600.0, "Zahlung mit Rückzahlung");
}

#[test]
fn housing_need_excludes_amortization() {
    // Wohnkosten im Bedarf: Nebenkosten 50 + Zins 0, ohne Amortisation 100 pro Monat (Periode bis 2036, direkt und
    // indirekt)
    for mode in ["direct", "indirect"] {
        let mut v = amortized();
        let t = &mut v["realestate"]["properties"][0]["mortgage"]["tranches"][0];
        t["amount"] = json!(100_000);
        t["periods"][0]["term"] = json!(10);
        t["periods"][0]["amortMode"] = json!(mode);
        buckets(&mut v, 1.0, 1.0, -1.0);
        let r = run(&v);
        assert!(rec(&r, 2032, 1).mortgage > 0.0);
        close(refill(&r, "Januar", 2032, 1).basis.housing, 12.0 * 50.0, mode);
    }
}

#[test]
fn indirect_amortization_is_paid_from_the_wealth() {
    // 300 pro Quartal in die Säule 3a: Das liquide Vermögen sinkt um die Einzahlungen.
    let with = |amort: f64| {
        let mut v = amortized();
        let t = &mut v["realestate"]["properties"][0]["mortgage"]["tranches"][0];
        t["amount"] = json!(100_000);
        t["periods"][0]["term"] = json!(10);
        t["periods"][0]["amortMode"] = json!("indirect");
        t["periods"][0]["amort"] = json!(amort);
        let r = run(&v);
        let x = rec(&r, 2027, 12);
        (x.cash + x.stocks + x.bonds, x.pillar3a)
    };
    let (liquid, p3a) = with(300.0);
    let (liquid0, _) = with(0.0);
    assert_eq!(p3a, 1500.0);
    close(liquid0 - liquid, 1500.0, "Einzahlungen Dezember 2026 bis Dezember 2027");
}

#[test]
fn renewal_details() {
    // Einzelne Tranche: Ereignis ohne Tranchennamen
    let r = run(&renewal_plan(300_000.0, two_periods()));
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == EventKind::MortgageRenewal && e.text.starts_with("Neue Hypothekarperiode Haus: 3.00 %")));
    // Abbezahlte Tranche: keine Erneuerung, keine Prüfung
    let mut v = renewal_plan(600.0, two_periods());
    for p in v["realestate"]["properties"][0]["mortgage"]["tranches"][0]["periods"]
        .as_array_mut()
        .unwrap()
    {
        p["amort"] = json!(300);
    }
    let r = run(&v);
    assert!(r.renewals.is_empty());
    assert!(!r.events.iter().any(|e| e.kind == EventKind::MortgageRenewal));
    // Kauf mit Datum: Die erste Periode ist keine Erneuerung
    let fixed = json!({"tranches": [{"name": "A", "amount": 400_000, "periods": [{"type": "fixed", "term": 10, "rate": 0.02}]}]});
    let r = run(&estate(json!([planned(800_000.0, "2030-01", fixed)])));
    assert!(!r
        .events
        .iter()
        .any(|e| e.kind == EventKind::MortgageRenewal && e.month == ym(2030, 1)));
}

#[test]
fn no_renewal_in_the_month_of_the_sale() {
    // Periode endet im November 2027, Verkauf im selben Monat: keine Erneuerung (auch keine Tragbarkeitsprüfung).
    let mut v = renewal_plan(580_800.5, two_periods());
    v["realestate"]["properties"][0]["sell"] = json!("2027-11");
    let r = run(&v);
    assert!(r.renewals.is_empty(), "{:?}", r.renewals);
    assert_eq!(sale(&r).month, ym(2027, 11));
}
