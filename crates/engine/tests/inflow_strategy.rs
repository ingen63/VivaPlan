//! Withdrawal strategy with inflows of funds: inflows (one-off amounts, PK payout, sale proceeds) apply the
//! strategy at once in the month of the inflow, not only in January. Sale proceeds for which a purchase is
//! planned in the following month are reserved; the strategy then follows after the purchase.
//!
//! Strategy «Fixe Rate» 4 %: withdrawal per month = 4 % of liquid wealth / 12 (without returns, taxes, inflation).

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

/// Withdrawal of this month (monthly share of the annual withdrawal).
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
    // Retirement = start in January: the strategy starts with the lump sum, no additional application.
    let mut v = base();
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 });
    let res = run(v);
    close(spent(&res, 2030, 1), 0.04 * 1_500_000.0 / 12.0, "Entnahme Januar");
    assert!(applied(&res, 2030, 1).is_empty());
    // February: unchanged 4 % of the wealth from the January budget
    close(spent(&res, 2030, 2), 0.04 * 1_500_000.0 / 12.0, "Entnahme Februar");
}

#[test]
fn pk_payout_in_july_starts_the_strategy_with_the_lump_sum() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2030-07");
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 });
    let res = run(v);
    // The strategy starts in July (retirement) and includes the lump sum at once
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
    // June: no application, proceeds reserved (withdrawal stays at the old budget)
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
    // July: first the purchase (equity 300'000), then the strategy on the remaining liquid wealth
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
    // January: one budget, computed after the purchase (not twice, no additional entry)
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
    // Special case B: the inflow goes entirely to bucket 3, buckets 1 and 2 are not refilled
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
    // December: the proceeds go directly into the reservation (external → reservation), not into a bucket; bucket 1 stays unchanged
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
    // January: surplus of the reservation (800'000 proceeds − 300'000 equity = 500'000) into bucket 3
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
    // the January refill comes after the purchase: equity was paid from the reservation, not from bucket 1
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
    // The equity (300'000) is a one-off expense of the month: C covers it, bucket 1 is never negative before
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
