//! 3-bucket strategy according to the specification: net need of the current year, January refill, return criterion and the
//! special cases A (start of the withdrawal phase, PK lump sum, restart), B (external inflows) and C (look-ahead).
//!
//! Net need = living costs + housing + income tax − regular income (AHV, PK pension, earned income). Not
//! included: one-off expenses, amortization, property purchases, special taxes. Target buckets 1 and 2 = years × net need.
//! The balances in `before`/`after` of the logs apply before the month's expenses.

use engine::batch;
use engine::model::Plan;
use engine::sim::{BucketRefill, EventKind, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

/// Living costs 100'000, rent 24'000, income tax rate 4'000 / 84'000, pensions AHV 2'500 + PK pension 4'500 per month;
/// #198: AHV in the need with the 13th pension 32'500, pensions 86'500, income tax 4'119.05 → net need [`NEED`];
/// bucket 1 = 2 years, bucket 2 = 1 year; stocks 6 %, bonds 1 %.
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

/// #198/AK2: Net need of [`base`]: 100'000 + 24'000 + 4'119.05 − 32'500 − 54'000.
const NEED: f64 = 41_619.05;

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

/// No bucket ever falls below 0 (at the end of the month and in all logs).
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

/// #198/AK2, #198/AK3: AHV with the 13th pension (13 × 2'500), income tax on 86'500 including the 13th pension.
#[test]
fn issue_198_ak2_example_january_2031_gives_83238_10_and_41619_05() {
    let res = run(base());
    let r = january(&res, 2031);
    assert_eq!(r.basis.year, 2031);
    assert!(r.basis.source.contains("Januar"), "{}", r.basis.source);
    assert_eq!(r.basis.months, 12);
    close(r.basis.living, 100_000.0, "Lebenshaltung");
    close(r.basis.housing, 24_000.0, "Wohnen");
    close(r.basis.income_tax, 4_119.05, "Einkommenssteuer auf 86'500");
    close(r.basis.ahv, 32_500.0, "AHV mit 13. Rente");
    close(r.basis.pk, 54_000.0, "PK-Rente");
    close(r.need, 41_619.05, "Netto-Bedarf");
    close(r.target1, 83_238.10, "Ziel Topf 1");
    close(r.target2, 41_619.05, "Ziel Topf 2");
    close(r.after[0], 83_238.10, "Topf 1 nach der Auffüllung");
    close(r.after[1], 41_619.05, "Topf 2 nach der Auffüllung");
    close(
        r.after[0] + r.after[1] + r.after[2],
        r.before.iter().sum::<f64>(),
        "Summe unverändert",
    );
    assert!(r.condition_met && !r.condition_ignored);
}

/// #198/AK1: Before 2026 the AHV counts 12 monthly pensions in the need, from 2026 with the 13th pension 13.
#[test]
fn issue_198_ak1_the_13th_pension_counts_in_the_need_from_2026() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("1959-01"); // 65 at the start: no early-payout reduction
    v["milestones"][0]["date"] = json!("2024-01");
    v["milestones"][1]["date"] = json!("2024-01");
    v["milestones"][2]["date"] = json!("2027-12");
    let res = run(v);
    let (r25, r26) = (january(&res, 2025), january(&res, 2026));
    close(r25.basis.ahv, 30_000.0, "AHV 2025: 12 × 2'500");
    close(r25.basis.income_tax, 4_000.0, "Einkommenssteuer 2025 auf 84'000");
    close(r25.need, 44_000.0, "Netto-Bedarf 2025");
    close(r26.basis.ahv, 32_500.0, "AHV 2026: 13 × 2'500");
    close(r26.need, NEED, "Netto-Bedarf 2026");
    close(r26.target1, 2.0 * NEED, "Ziel Topf 1 2026");
}

#[test]
fn ten_years_above_the_threshold_reach_the_targets_every_year() {
    let res = run(base());
    table(&res, "10 Jahre, Rendite über der Schwelle");
    for y in 2031..=2041 {
        let r = january(&res, y);
        close(r.need, NEED, &format!("Netto-Bedarf {y}"));
        close(r.after[0], 2.0 * NEED, &format!("Topf 1 {y}"));
        close(r.after[1], NEED, &format!("Topf 2 {y}"));
        // Bucket 1 has shrunk by the year's spending and is brought back to the target every year
        assert!(r.before[0] < r.after[0]);
        // (in the first year the previous year's income tax, only due in May, is missing; #190: the 13th AHV pension
        // of 2'500 in December lowers the year's withdrawal (#198: it is part of the net need); the income tax on it,
        // 2'500 × 4'000 / 84'000, is paid in May of the following year)
        let paid = if y == 2031 {
            40_000.0 - 2_500.0
        } else {
            44_000.0 - 2_500.0 + 2_500.0 * 4_000.0 / 84_000.0
        };
        close(
            r.before[0] + paid,
            2.0 * NEED,
            &format!("Topf 1 vor der Auffüllung {y}"),
        );
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
    // One-off expense 2033 (trip) and special taxes: property gain tax 2033 (sale), lump-sum tax
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
    // AHV only starts in January 2032: the net need falls from 72'571 to 44'000; bucket 2 is then above its target
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
    close(r.target2, 0.5 * NEED, "Ziel Topf 2");
    // Shortfall bucket 1 ≈ one year's spending; bucket 2 (half a need) is not enough, the rest comes from bucket 3, then
    // bucket 2 again from bucket 3
    let m: Vec<(u8, u8)> = r.moves.iter().map(|m| (m.from, m.to)).collect();
    assert_eq!(m, vec![(2, 1), (3, 1), (3, 2)], "{:?}", r.moves);
    close(r.after[0], 2.0 * NEED, "Topf 1");
    close(r.after[1], 0.5 * NEED, "Topf 2");
}

/// Plan with the threshold 100 % (the stock return of 6 % never reaches it).
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
    // Bucket 2 is not refilled: after the transfer to bucket 1 it is smaller than the target
    let r = january(&res, 2031);
    assert!(r.after[1] < r.target2 - 1.0);
    // During the year C applies: bucket 1 does not cover the expenses, the funds then come from bucket 3
    let c = res.bucket_refills.iter().find(|r| r.trigger == "C2").expect("Fall C2");
    assert!(!c.condition_met);
    assert!(c.moves.iter().any(|m| (m.from, m.to) == (3, 1)), "{:?}", c.moves);
    never_negative(&res);
}

/// #103/AK2: Start in July 2030; the lead-in of the return path (July 2029 to June 2030) supplies the months before the
/// start. The January refill 2031 measures the last 12 months (February 2030 to January 2031): 5 lead-in months,
/// July to December 2030 +2 % each, January 2031 0 %. Threshold 5 %.
#[test]
fn issue_103_ak2_start_in_july_measures_the_last_twelve_months_with_the_pre_run() {
    let mut v = base();
    v["milestones"][0]["date"] = json!("2030-07");
    v["milestones"][1]["date"] = json!("2030-07");
    v["withdrawal"]["buckets"]["refillThreshold"] = json!(0.05);
    let run_pre = |pre: f64| {
        let p: Plan = serde_json::from_value(crate::any_age(v.clone())).expect("Planung gültig");
        let sim = Simulator::new(&p).unwrap();
        let mut path = batch::market_paths(&p, &sim, None).unwrap().remove(0);
        assert_eq!(path.pre_stocks.len(), 12, "Vorlauf bei fixen Renditen");
        path.pre_stocks = vec![pre; 12];
        for (k, x) in path.stocks.iter_mut().enumerate() {
            *x = if k < 6 { 0.02 } else { 0.0 };
        }
        sim.run(&path, true)
    };

    // Lead-in −1 % per month: 0.99⁵ × 1.02⁶ − 1 = +7.1 % ≥ 5 % → met, bucket 3 is sold
    let res = run_pre(-0.01);
    let j = january(&res, 2031);
    assert_eq!(j.return_basis, "last 12 months");
    close(
        j.stock_return,
        0.99f64.powi(5) * 1.02f64.powi(6) - 1.0,
        "Februar 2030 bis Januar 2031",
    );
    assert!(j.condition_met);
    assert!(j.moves.iter().any(|m| m.from == 3), "{:?}", j.moves);

    // Lead-in −3 % per month: 0.97⁵ × 1.02⁶ − 1 = −3.3 % < 5 % → not met, no sale from bucket 3
    let res = run_pre(-0.03);
    let j = january(&res, 2031);
    close(
        j.stock_return,
        0.97f64.powi(5) * 1.02f64.powi(6) - 1.0,
        "Februar 2030 bis Januar 2031",
    );
    assert!(!j.condition_met);
    assert!(j.moves.iter().all(|m| m.from != 3), "{:?}", j.moves);

    // From the 13th month the criterion only measures months of the path
    let j = january(&res, 2032);
    assert_eq!(j.return_basis, "last 12 months");
    close(j.stock_return, 0.0, "Februar 2031 bis Januar 2032");
    never_negative(&res);
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
    assert!(a.basis.source.contains("extrapolated"));
    // #198/AK1: AHV of the 10 months from March, extrapolated: 10 × 2'500 × 12/10 × 13/12 = 32'500
    close(a.basis.ahv, 32_500.0, "AHV hochgerechnet mit 13. Rente");
    close(a.need, NEED, "Netto-Bedarf auf 12 Monate hochgerechnet");
    close(a.target1, (24.0 - 2.0) / 12.0 * NEED, "Ziel Topf 1");
    close(a.target2, (12.0 - 2.0) / 12.0 * NEED, "Ziel Topf 2");
    assert!(a.condition_ignored);
    close(a.after[0], a.target1, "Topf 1");
    close(a.after[1], a.target2, "Topf 2");
    // everything was in bucket 3 first
    assert_eq!((a.moves[0].to, a.moves[1].to), (3, 3));
    never_negative(&res);
}

/// Two persons: person B draws half of their PK as capital in July 2030 (200'000).
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
    assert!(a.reason.contains("PK lump sum"), "{}", a.reason);
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
    // no surplus booking during the year: bucket 1 above the target stays unchanged
    assert!(a.moves.iter().all(|m| !(m.to == 3 && m.from != 0)), "{:?}", a.moves);
    assert!(a.after[0] >= a.target1 - 1e-6);
    never_negative(&res);
}

/// #99/AK4: A PK lump sum in the middle of the year (trigger A) determines the net need anew, from the months from the
/// payout extrapolated to 12 months; this need applies until the next January.
#[test]
fn issue_99_ak4_a_pension_fund_withdrawal_recomputes_the_need_from_the_withdrawal_month() {
    let res = run(two_persons());
    let a = find(&res, "A", 2030, 7);
    assert_eq!(a.basis.year, 2030);
    assert_eq!(a.basis.months, 6, "Juli bis Dezember");
    assert!(a.basis.source.starts_with("from 07.2030"), "{}", a.basis.source);
    assert!(a.basis.source.contains("extrapolated"), "{}", a.basis.source);
    close(a.need, a.basis.need(), "Bedarf aus der neuen Grundlage");
    // until December this basis applies; only January determines the need anew
    for r in res
        .bucket_refills
        .iter()
        .filter(|r| r.month.year() == 2030 && r.month.month() > 7)
    {
        assert_eq!(r.basis.source, a.basis.source, "{} {}", r.trigger, r.month);
    }
    assert_eq!(january(&res, 2031).basis.source, "January, planned");
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
    // Target bucket 1 = (2 × 12 − 2) / 12 × net need + one-off expense, target bucket 2 = (12 − 2) / 12 × net need
    close(c.target1, 22.0 / 12.0 * NEED + 300_000.0, "Ziel Topf 1");
    close(c.target2, 10.0 / 12.0 * NEED, "Ziel Topf 2");
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

/// Bucket 1 without a target, return criterion never met: case C2 makes sure bucket 1 covers the payment.
fn no_cash_target() -> Value {
    let mut v = base();
    v["withdrawal"]["buckets"] = json!({ "cashYears": 0, "bondYears": 0, "refillThreshold": 1.0 });
    v
}

#[test]
fn c_keeps_two_monthly_needs_in_june_and_nothing_in_december() {
    let month = NEED / 12.0;
    let mut v = no_cash_target();
    v["flows"]["extras"] = json!([ { "date": "2030-06", "amount": -20_000, "label": "Ausgabe" } ]);
    let res = run(v);
    let c = find(&res, "C2", 2030, 6);
    // June: 7 months until January → min(3, 7) − 1 = 2 monthly needs
    let j = res.records.iter().find(|x| x.month == YearMonth::new(2030, 6)).unwrap();
    close(j.cash, 2.0 * month, "Topf 1 nach der Zahlung im Juni");
    assert!(!c.condition_met);
    let mut v = no_cash_target();
    v["flows"]["extras"] = json!([ { "date": "2030-12", "amount": -20_000, "label": "Ausgabe" } ]);
    let res = run(v);
    find(&res, "C2", 2030, 12);
    // December: 1 month until January → min(3, 1) − 1 = 0
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
    close(j.cash, 4.0 * NEED / 12.0, "5 Monate − 1");
}

#[test]
fn earned_income_is_deducted_from_the_need() {
    let mut v = base();
    v["flows"]["income"] = json!(3_000); // 36'000 per year
    let res = run(v);
    let r = january(&res, 2031);
    close(r.basis.earned, 36_000.0, "earned income");
    // #198: AHV with the 13th pension
    close(r.basis.ahv + r.basis.pk, 86_500.0, "Renten");
    close(r.basis.income_tax, 122_500.0 * 4_000.0 / 84_000.0, "Einkommenssteuer");
    close(
        r.need,
        124_000.0 + r.basis.income_tax - 86_500.0 - 36_000.0,
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
    close(b.target1, 2.0 * NEED, "Ziel steigt schon im Januar 2033");
    close(c.target1, 2.0 * NEED, "Ziel 2034");
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
        .expect("emergency sale");
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
        .any(|e| e.kind == EventKind::Buckets && e.text.contains("all buckets empty")));
    let neg = res.records.iter().filter(|r| r.cash < -1.0).count();
    assert!(neg > 5, "Topf 1 im Minus: {neg} Monate");
    for r in res.records.iter().filter(|r| r.cash < -1.0) {
        assert!(
            r.bonds.abs() < 1e-9 && r.stocks.abs() < 1e-9,
            "Topf 2 und 3 stillgelegt"
        );
    }
    // The inflow reduces the deficit; as soon as wealth is positive, A runs again
    let a = res
        .bucket_refills
        .iter()
        .find(|r| r.trigger == "A" && r.reason.contains("restart"))
        .expect("restart");
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
    // Threshold exactly equal to the return of 6 %: met
    let mut v = base();
    v["withdrawal"]["buckets"]["refillThreshold"] = json!(0.06 - 1e-9);
    let res = run(v);
    assert!(january(&res, 2032).condition_met);
}

#[test]
fn c2_with_9_months_reserve_refills_8_months_from_pot_2_then_pot_3_and_does_not_refill_pot_2() {
    let month = NEED / 12.0;
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
    // Source: first bucket 2 down to 0, then bucket 3; bucket 2 is not refilled
    assert_eq!((c.moves[0].from, c.moves[0].to), (2, 1));
    assert_eq!(c.after[1], 0.0);
    assert_eq!((c.moves[1].from, c.moves[1].to), (3, 1));
    assert!(c.moves.iter().all(|m| m.to != 2));
    // April: 9 months until January → min(9, 9) − 1 = 8 monthly needs remain after the expenses
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
        1.75 * NEED + 400_000.0,
        "Ziel Topf 1 = 1.75 × Netto-Bedarf + Einmalausgabe",
    );
    close(c.target2, 0.75 * NEED, "Ziel Topf 2 = 0.75 × Netto-Bedarf");
    close(c.after[0], c.target1, "Topf 1");
    close(c.after[1], c.target2, "Topf 2");
    let m = met.records.iter().find(|x| x.month == YearMonth::new(2035, 4)).unwrap();
    assert!(m.cash >= 0.0);
    assert!(!met
        .bucket_refills
        .iter()
        .any(|r| r.month == YearMonth::new(2035, 4) && r.trigger == "C2"));
    never_negative(&met);
    // same case, criterion not met → C2: cover the expenses, afterwards (min(3, 9) − 1) = 2 monthly needs, bucket 2 stays
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
    close(m.cash, 2.0 * NEED / 12.0, "Topf 1 nach den Ausgaben");
    never_negative(&locked);
}

#[test]
fn c1_in_december_does_not_move_pot_2_above_the_shortened_target_to_pot_3() {
    let mut v = base();
    v["withdrawal"]["buckets"]["bondYears"] = json!(10);
    // #190: 2'500 more than before because of the 13th AHV pension in December
    v["flows"]["extras"] = json!([ { "date": "2035-12", "amount": -40_500, "label": "Ausgabe" } ]);
    let res = run(v);
    let c = find(&res, "C1", 2035, 12);
    assert_eq!(c.passed_months, 11);
    close(c.target2, (120.0 - 11.0) / 12.0 * NEED, "Ziel Topf 2 gekürzt");
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
    close(j.target1, 2.0 * NEED + 300_000.0, "Ziel Topf 1 + Einmalausgabe");
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
    let month = NEED / 12.0;
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
