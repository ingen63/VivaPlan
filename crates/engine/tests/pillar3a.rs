//! #52 (pillar 3a as an account of its own) and #50 (indirect amortization only until the AHV payout).
//!
//! Made-up plan that can be recomputed by hand: birth 2000-01 (age = year − 2000), start 2040-01, no returns,
//! no inflation, no taxes (except where noted). AHV payout at 65 in January 2045.

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
        .map(|e| (e.month, e.text.to_string()))
        .collect()
}

/// #52/AK3: Balance 120'000, plan 2041 and 2042 50'000 each, AHV payout 2045: remainder 20'000 in the month of the AHV payout.
#[test]
fn issue_52_ak3_payout_plan_and_rest_at_ahv() {
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
        p[2].1
            .contains("Payout of the remainder at the start of the AHV pension 20000 CHF"),
        "{}",
        p[2].1
    );
    // The money flows into wealth (no taxes): January 2041 + 50'000 − 1'000 spending.
    close(
        rec(&r, 2041, 1).liquid - rec(&r, 2040, 12).liquid,
        49_000.0,
        "Zufluss 2041",
    );

    // AHV payout one year later (2046-01): the plan stays valid, the remainder comes in 2046.
    v["milestones"][2]["years"] = json!(66);
    let r = run(&v);
    assert_eq!(
        payouts(&r).iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2041, 1), ym(2042, 1), ym(2046, 1)]
    );
    close(rec(&r, 2045, 12).pillar3a, 20_000.0, "bis 2046");
}

/// #52/AK5: An amount larger than the balance only pays out the balance, with a note in the events.
#[test]
fn issue_52_ak5_amount_larger_than_balance() {
    let mut v = base();
    v["pillar3a"] =
        json!([{ "person": "A", "balance": 30_000, "rate": 0, "payouts": [{ "date": "2041-06", "amount": 50_000 }] }]);
    let r = run(&v);
    close(rec(&r, 2041, 6).pillar3a, 0.0, "empty");
    let p = payouts(&r);
    assert!(
        p.iter()
            .any(|x| x.0 == ym(2041, 6) && x.1.contains("planned 50000 CHF, the balance only covers 30000 CHF")),
        "{p:?}"
    );
    assert!(
        p.iter().any(|x| x.0 == ym(2041, 6) && x.1.contains("Payout 30000 CHF")),
        "{p:?}"
    );
    assert!(!p.iter().any(|x| x.0 == ym(2045, 1)), "kein Rest mehr");
    // Without an amount: the whole balance
    v["pillar3a"][0]["payouts"] = json!([{ "date": "2041-06" }]);
    let r = run(&v);
    close(rec(&r, 2041, 6).pillar3a, 0.0, "ganzes Guthaben");
}

/// #52/AK6: Every payout is a lump sum; added together with the PK lump sum in the same year.
#[test]
fn issue_52_ak6_capital_tax_with_pk_lump_sum_in_the_same_year() {
    let mut v = base();
    // Rate linear between the grid points: 100'000 → 5 %, 200'000 → 10 %.
    v["taxes"]["capital"] = json!([[0, 0.0], [200_000, 0.10]]);
    v["milestones"][1]["years"] = json!(61); // pension fund payout 2041-01
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

/// #52/AK8: The contribution raises the balance and counts as spending (500 per month with 6'000 per year).
#[test]
fn issue_52_ak8_contribution_counts_as_expense() {
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
        close(a.withdrawal - b.withdrawal, 500.0, "Withdrawal");
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

/// #52/AK9: Contributions end in the month before the AHV payout.
#[test]
fn issue_52_ak9_contributions_end_before_ahv() {
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": 1200 }]);
    let r = run(&v);
    close(rec(&r, 2044, 12).pillar3a_in, 100.0, "Dezember 2044");
    close(rec(&r, 2045, 1).pillar3a_in, 0.0, "AHV pension start");
    assert!(r
        .records
        .iter()
        .filter(|x| x.month >= ym(2045, 1))
        .all(|x| x.pillar3a_in == 0.0 && x.pillar3a == 0.0));
    // 5 years × 1'200 are paid out at the AHV payout
    assert!(payouts(&r)
        .iter()
        .any(|x| x.0 == ym(2045, 1) && x.1.contains("6000 CHF")));
}

/// #52: The contribution belongs to the net need of the 3-bucket strategy.
#[test]
fn issue_52_contribution_is_part_of_the_net_need() {
    let mut v = base();
    v["milestones"][1]["years"] = json!(60); // withdrawal phase from the start
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

/// #50/AK1–AK3: Mortgage 400'000, 750 per quarter indirect, AHV payout M = 2045-01.
#[test]
fn issue_50_ak1_to_ak3_indirect_until_ahv_then_direct() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    let r = run(&v);
    // Before M: 3a +750 per quarter, mortgage stays 400'000 (value 800'000 − debt = equity).
    close(rec(&r, 2040, 3).pillar3a, 750.0, "März 2040");
    close(
        rec(&r, 2044, 12).pillar3a,
        20.0 * 750.0,
        "20 Quartale bis Dezember 2044",
    );
    close(rec(&r, 2044, 12).property_equity, 400_000.0, "Schuld unverändert");
    // M: remainder paid out; afterwards the mortgage falls by 750 per quarter, the 3a stays empty.
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

/// #162/AK1: Indirect amortization into pillar 3a stays in wealth: the change in wealth of every month matches the
/// ledger, also in the quarters with a contribution into the 3a and after the switch to direct amortization.
#[test]
fn issue_162_ak1_indirect_amortization_keeps_the_ledger() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    let r = run(&v);
    let mut prev = r.summary.start_total;
    for x in &r.records {
        let (delta, flows) = (x.total - prev, x.ledger.sum());
        assert!(
            (delta - flows).abs() <= 0.01,
            "{}: Total {delta:.4}, Hauptbuch {flows:.4} ({:?})",
            x.month,
            x.ledger
        );
        prev = x.total;
    }
    close(
        rec(&r, 2040, 3).ledger.amortization,
        750.0,
        "Amortisation in die 3a im Hauptbuch",
    );
}

/// #50/AK4: A period that only starts from the AHV payout (renewal) amortizes directly.
#[test]
fn issue_50_ak4_period_from_ahv_is_direct() {
    let mut v = base();
    // First period until 2045-01 without amortization, the renewal from 2045-01 entered as indirect.
    v["realestate"] = json!({ "properties": [ house(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 0 },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]), "2040-01") ],
        // Affordability at renewal given (without income otherwise a one-off repayment)
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
        .map(|e| (e.month, e.text.to_string()))
        .collect()
}

/// #50: The switch from indirect to direct amortization at the AHV payout appears once in the events.
#[test]
fn issue_50_switch_to_direct_is_an_event() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    let r = run(&v);
    assert_eq!(
        switches(&r),
        vec![(
            ym(2045, 1),
            "Amortization of mortgage Haus from the start of the AHV pension directly instead of via pillar 3a: 3000 CHF per year".to_string()
        )]
    );
    // Renewal from the AHV payout (indirect before): also a switch.
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
    assert!(switches(&r)[0].1.contains("4000 CHF per year"), "{:?}", switches(&r));
    // No switch: direct from the beginning, indirect only from the AHV payout (AK4) or start after the AHV payout.
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
    late["milestones"][0]["date"] = json!("2045-01"); // start = AHV payout
    late["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2045-01") ] });
    assert!(switches(&run(&late)).is_empty());
}

/// #52: Payout «Hypothek zurückzahlen» first repays the tranche with indirect amortization.
#[test]
fn issue_52_payout_repays_the_indirect_tranche() {
    let mut v = base();
    v["realestate"] = json!({ "properties": [ house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2040-01") ] });
    v["pillar3a"] = json!([{ "person": "A", "balance": 10_000, "rate": 0, "use": "repay" }]);
    let r = run(&v);
    // Remainder at the AHV payout: 10'000 + 20 × 750 = 25'000 to the mortgage
    close(rec(&r, 2045, 1).property_equity, 425_000.0, "repayment");
    assert!(payouts(&r).iter().any(|x| x.1.contains("to mortgage 25000 CHF")));
}

/// #52/AK13: An old pillar 3a on the mortgage is taken over when computing as when loading.
#[test]
fn issue_52_ak13_legacy_pillar3a_of_a_mortgage() {
    let mut v = base();
    let mut h = house(
        json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]),
        "2040-01",
    );
    h["mortgage"]["pillar3a"] = json!({ "balance": 10_000, "rate": 0, "payout": "2042-06", "use": "repay" });
    v["realestate"] = json!({ "properties": [h] });
    let r = run(&v);
    // 2042-06: 10'000 + 10 quarters × 750 = 17'500 to the mortgage
    close(
        rec(&r, 2042, 6).property_equity,
        417_500.0,
        "the whole balance is paid out",
    );
    close(rec(&r, 2042, 6).pillar3a, 0.0, "empty");
    // Afterwards indirect again until the AHV payout, remainder 2045-01: 10 quarters × 750
    assert!(payouts(&r)
        .iter()
        .any(|x| x.0 == ym(2045, 1) && x.1.contains("7500 CHF")));
}

/// #52/AK12: The pension fund contains no 3a balance; the evaluations show both separately.
#[test]
fn issue_52_ak12_pension_fund_and_pillar3a_are_kept_apart() {
    let mut v = base();
    v["pension"]["pk"]["capital"] = json!(200_000);
    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0 }]);
    let r = run(&v);
    close(rec(&r, 2040, 1).pension_fund, 200_000.0, "Pensionskasse ohne 3a");
    close(rec(&r, 2040, 1).pillar3a, 50_000.0, "Säule 3a für sich");
    // The 3a balance also does not count towards liquid wealth (cash 1'000'000 minus the month's spending)
    close(rec(&r, 2040, 1).liquid, 999_000.0, "liquid ohne 3a und PK");
}

// --- #124: Tests for undetected mutants of the full mutation run ---

/// Two persons: B (birth 1983-01) draws AHV in January 2048, A in January 2045; end of plan 2049-12.
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

/// Affordability always given (without income otherwise a one-off repayment at purchase and at renewal).
fn affordable() -> Value {
    json!({ "sustainability": 1, "imputedRate": 0, "fixCosts": 0, "wealthContribution": 0, "maxLoanToValue": 1 })
}

/// #124/AK1: The indirect amortization flows into the pillar 3a of the person named on the tranche (here B), and is
/// paid out at their AHV payout, not at that of the first person. Applies to an owned property and to a purchase.
#[test]
fn issue_124_ak1_indirect_amortization_goes_to_the_second_person() {
    let indirect = json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]);
    let mut owned = house(indirect.clone(), "2040-01");
    owned["mortgage"]["tranches"][0]["pillar3a"] = json!("B");
    let mut v = couple();
    v["realestate"] = json!({ "properties": [owned] });
    let r = run(&v);
    // Until December 2047 32 quarters of 750 in the 3a of B; nothing is paid out at the AHV payout of A (2045-01).
    close(rec(&r, 2045, 1).pillar3a, 15_000.0, "AHV-Bezug A: 3a von B bleibt");
    close(rec(&r, 2047, 12).pillar3a, 24_000.0, "vor dem AHV-Bezug B");
    close(rec(&r, 2048, 1).pillar3a, 0.0, "Rest im AHV-Bezug B");
    assert_eq!(payouts(&r).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2048, 1)]);
    assert_eq!(switches(&r).iter().map(|x| x.0).collect::<Vec<_>>(), vec![ym(2048, 1)]);

    // Purchase in January 2041 with the same tranche: also into the 3a of B (end of 2047: 28 quarters).
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

    // Purchase «nach Verkauf» (January 2041, after the sale of another house in December 2040): also into the 3a of B.
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

/// #124/AK3: Switch to direct amortization at the AHV payout only if it was really amortized indirectly before,
/// amortization continues from now on and debt remains; with several tranches the event names the tranche.
#[test]
fn issue_124_ak3_switch_to_direct_needs_amortization_before_and_after() {
    // With an AHV pension the renewal at the AHV payout is affordable (without income: emergency sale instead of a switch).
    let p = |periods: Value| {
        let mut v = base();
        v["realestate"] = json!({ "properties": [house(periods, "2040-01")], "affordability": affordable() });
        v["pension"]["ahv"]["monthly"] = json!(5000);
        v
    };
    // Cross-check: indirect before, amortized further afterwards: switch.
    let v = p(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 750, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]));
    assert_eq!(
        switches(&run(&v)).iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![ym(2045, 1)]
    );
    // Indirect before, but with amortization 0: no switch.
    let v = p(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 0, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 1000, "amortMode": "indirect" } ]));
    assert!(switches(&run(&v)).is_empty(), "vorher nichts amortisiert");
    // Amortized indirectly before, no amortization from the AHV payout: no switch.
    let v = p(json!([
        { "type": "fixed", "term": 5, "rate": 0, "amort": 750, "amortMode": "indirect" },
        { "type": "fixed", "term": 10, "rate": 0, "amort": 0, "amortMode": "indirect" } ]));
    assert!(switches(&run(&v)).is_empty(), "danach nichts amortisiert");

    // Tranche fully repaid before the AHV payout (payout of the 3a to the mortgage): no switch any more.
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

    // The plan starts at the AHV payout, the tranche already ran indirectly before: the switch lies before the plan.
    let mut v = base();
    v["milestones"][0]["date"] = json!("2045-01");
    v["realestate"] = json!({ "properties": [house(json!([{ "type": "fixed", "term": 20, "rate": 0, "amort": 750, "amortMode": "indirect" }]), "2035-01")] });
    assert!(switches(&run(&v)).is_empty());

    // Two tranches: the event names the tranche.
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
            "Amortization of mortgage Haus (T1) from the start of the AHV pension directly instead of via pillar 3a: 3000 CHF per year"
                .to_string()
        )]
    );
}

/// #124/AK2: A planned payout in the month of the AHV payout is booked as a payout of its own, the remainder afterwards.
/// A planned amount that exactly matches the balance triggers no note «reicht nur».
#[test]
fn issue_124_ak2_planned_payout_in_the_ahv_month_and_exact_amount() {
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
    assert!(!p.iter().any(|x| x.1.contains("only covers")), "{p:?}");

    v["pillar3a"] = json!([{ "person": "A", "balance": 50_000, "rate": 0, "payouts": [
        { "date": "2041-01", "amount": 50_000 } ] }]);
    let p = payouts(&run(&v));
    assert!(!p.iter().any(|x| x.1.contains("only covers")), "{p:?}");
}

/// Withdrawal phase from the start (retirement at 60), AHV payout at 65 (2045-01), 3a contribution 2'400 per year.
fn retired_with_contribution(contribution: f64) -> Value {
    let mut v = base();
    v["milestones"][1]["years"] = json!(60);
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0, "contribution": contribution }]);
    v
}

/// #124/AK2: From retirement the 3a contribution keeps counting as spending: in the reserve of the
/// strategy «Standard», in the spending of the 3-bucket strategy and as housing or fixed costs of the guardrails.
#[test]
fn issue_124_ak2_contribution_after_retirement_is_an_expense() {
    // Standard: reserve 12 months of the shortfall (spending 1'000 + contribution 200), the rest in stocks.
    let standard = |c: f64| {
        let mut v = retired_with_contribution(c);
        v["wealth"]["stockShare"] = json!(1);
        v["wealth"]["reserveMonths"] = json!(12);
        run(&v)
    };
    let (with, without) = (standard(2400.0), standard(0.0));
    close(rec(&with, 2041, 6).cash, 12.0 * 1200.0, "Reserve mit Einzahlung");
    close(rec(&without, 2041, 6).cash, 12.0 * 1000.0, "Reserve ohne Einzahlung");

    // «3-Topf»: the contribution is part of the month's spending.
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
    // Bucket 1 after the January refill: target higher by 12 × 200, the month's spending by 200.
    close(
        rec(&with, 2041, 1).cash - rec(&without, 2041, 1).cash,
        2400.0 - 200.0,
        "Topf 1 im Januar",
    );

    // Guardrails: the contribution is deducted from the withdrawal like the housing costs; living costs fall by 200.
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

/// #124/AK2: Net need of the 3-bucket strategy with a 3a contribution: extrapolated to 12 months from July, with
/// guardrails living costs and without contribution in the year of the AHV payout (it ends before the AHV payout).
#[test]
fn issue_124_ak2_need_with_contribution() {
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
            .expect("refill")
            .clone()
    };
    let r = run_b(2400.0, false);
    let jul = refill(&r, 2040, 7);
    assert_eq!(jul.basis.months, 6);
    close(jul.basis.pillar3a, 2400.0, "6 Monate auf 12 hochgerechnet");
    close(refill(&r, 2044, 1).basis.pillar3a, 2400.0, "letztes volles Jahr");
    // In January 2045 two refills count: first the inflow from the 3a (need as before), then the planned one.
    close(refill(&r, 2045, 1).basis.pillar3a, 0.0, "Jahr des AHV-Bezugs");

    // Guardrails: the withdrawal stays at the budget, the living costs in the net need are smaller by 12 × 200.
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
