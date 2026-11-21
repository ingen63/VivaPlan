//! #6: «3-Topf» with guardrails (the guardrails steer spending, the buckets steer investment) and #47:
//! fixed rate, VPW and endowment from the retirement of the first person.
//!
//! Base plan (made up, like `sim_rules`): start 2026-11, birth 1966-11, retirement and AHV 2031-11, end
//! 2061-11; without taxes, inflation and returns, spending 1000 per month, wealth 1'000'000 as cash.

use engine::market::MarketPath;
use engine::model::Plan;
use engine::optimize::{param_defs, with_params, OptStrategy};
use engine::sim::{BucketRefill, EventKind, MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    let mut v: Value = serde_json::from_str(include_str!("../assets/new.vplan")).unwrap();
    v["returns"]["fixed"] = json!({"stocks": 0, "bonds": 0, "inflation": 0, "saron": 0});
    v["taxes"] = json!({"multiplier": 1, "income": [[0, 0]], "wealth": [[0, 0]], "capital": [[0, 0]]});
    v["flows"]["spending"] = json!(1000);
    v["wealth"] = json!({"stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0});
    v
}

/// Initial rate 4.3 %: with 4 % wealth would be exactly on the upper guardrail after 50 months (rounding decides).
const RATE: f64 = 0.043;
const GUARDRAILS: fn() -> Value = || json!({"initialRate": RATE, "upper": 0.2, "lower": 0.2, "adjust": 0.1});

/// Pure strategy «Guardrails».
fn pure(mut v: Value) -> Value {
    v["withdrawal"]["strategy"] = json!("guardrails");
    v["withdrawal"]["guardrails"] = GUARDRAILS();
    v
}

/// «3-Topf» with guardrails.
fn combined(mut v: Value) -> Value {
    v["withdrawal"]["strategy"] = json!("three_buckets");
    v["withdrawal"]["buckets"] =
        json!({"cashYears": 1, "bondYears": 1, "refillThreshold": -1, "spending": "guardrails"});
    v["withdrawal"]["guardrails"] = GUARDRAILS();
    v
}

/// Married couple: A retires 2031-11 (PK and AHV pension), B works until 2033-06 (2000 per month) and then draws the PK.
fn couple() -> Value {
    let mut v = base();
    v["household"]["persons"] =
        json!([{"id": "A", "name": "A", "birth": "1966-11"}, {"id": "B", "name": "B", "birth": "1968-06"}]);
    v["pension"]["pk"] = json!({"capital": 800_000, "at": "retire", "lumpsum": 0, "conversion": 0.05});
    v["pension"]["ahv"] = json!({"monthly": 1500, "at": "retire", "indexed": false});
    v["partner"] = json!({"income": {"mode": "timed", "rows": [{"from": "start", "value": 2000}, {"from": "2033-06", "value": 0}]},
                          "pk": {"capital": 600_000, "at": "2033-06", "lumpsum": 0, "conversion": 0.05, "interest": 0},
                          "ahv": {"monthly": 2000, "at": "2033-06", "indexed": false}});
    v
}

fn ym(y: i32, m: u32) -> YearMonth {
    YearMonth::new(y, m)
}

fn run_with(v: &Value, f: impl FnOnce(&mut MarketPath, YearMonth)) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
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

fn refill<'a>(r: &'a SimResult, trigger: &str, y: i32, m: u32) -> &'a BucketRefill {
    r.bucket_refills
        .iter()
        .find(|x| x.month == ym(y, m) && x.trigger == trigger)
        .unwrap_or_else(|| panic!("kein Eintrag {trigger} {m}.{y}"))
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} != {b}");
}

fn withdrawal_events(r: &SimResult) -> Vec<(YearMonth, &str)> {
    r.events
        .iter()
        .filter(|e| e.kind == EventKind::Withdrawal)
        .map(|e| (e.month, e.text.as_str()))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// #6
// ---------------------------------------------------------------------------------------------

/// #6/AK2: With the same wealth course (without returns) the withdrawal is identical pure and combined, also after
/// a cut by the guardrail.
#[test]
fn issue_6_ak2_combined_spends_like_pure_guardrails() {
    let (p, c) = (run(&pure(base())), run(&combined(base())));
    for (a, b) in p.records.iter().zip(&c.records) {
        close(b.spending, a.spending, &format!("Ausgaben {:?}", a.month));
        close(b.liquid, a.liquid, &format!("Vermögen {:?}", a.month));
    }
    // Initial withdrawal 4.3 % of 940'000 (60 months of 1000 spent)
    close(rec(&c, 2031, 11).withdrawal * 12.0, RATE * 940_000.0, "Startentnahme");
    let cuts = |r: &SimResult| {
        withdrawal_events(r)
            .into_iter()
            .filter(|(_, t)| t.contains("cut by"))
            .map(|(m, _)| m)
            .collect::<Vec<_>>()
    };
    assert!(!cuts(&c).is_empty(), "die Leitplanke greift, wenn das Vermögen sinkt");
    assert_eq!(cuts(&p), cuts(&c));
}

/// #6/AK3 (pure and combined): The guardrails start with the retirement of the first person; the running
/// earned income of the second person counts as «Einkommen und Renten».
#[test]
fn issue_6_ak3_guardrails_start_with_the_first_retirement() {
    for v in [pure(couple()), combined(couple())] {
        let r = run(&v);
        let b0 = RATE * rec(&r, 2031, 10).liquid;
        let first = withdrawal_events(&r)
            .into_iter()
            .find(|(_, t)| t.starts_with("Withdrawal"))
            .expect("Startentnahme");
        assert_eq!(first.0, ym(2031, 11));
        let nov = rec(&r, 2031, 11);
        assert!(nov.earned == 2000.0 && nov.pk_pension > 0.0);
        close(nov.withdrawal, b0 / 12.0, "Entnahme November 2031");
        close(
            nov.spending,
            b0 / 12.0 + nov.income,
            "Lebenshaltung = Entnahme + Einkommen und Renten",
        );
    }
    // Combined: buckets and guardrails start in the same month; the net need uses the guardrails.
    let r = run(&combined(couple()));
    let a = refill(&r, "A", 2031, 11);
    assert_eq!(a.basis.living_source, "Guardrails");
    // Living costs in the net need with earned income, PK pension and AHV (November and December, extrapolated to 12 months)
    let (nov, dec) = (rec(&r, 2031, 11), rec(&r, 2031, 12));
    assert!(nov.earned > 0.0 && nov.pk_pension > 0.0 && nov.ahv > 0.0);
    // #190: the net need plans with the monthly pensions; the guardrails spend the 13th AHV pension in December as well.
    let thirteenth = dec.ahv - nov.ahv;
    assert!(thirteenth > 0.0);
    close(
        a.basis.living,
        6.0 * (nov.spending + dec.spending - thirteenth),
        "Lebenshaltung im Netto-Bedarf",
    );
}

/// #6/AK4 (pure and combined): Sale proceeds reserved for a purchase do not count towards the rate.
#[test]
fn issue_6_ak4_reserved_proceeds_do_not_count_for_the_rate() {
    let with_sale = |mut v: Value| {
        v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
        v["realestate"]["properties"] = json!([
            {"id": "alt", "name": "alt", "status": "owned", "price": 600_000, "worth": 900_000, "sell": "2031-11", "fixCosts": 0, "inflation": false},
            {"id": "neu", "name": "Neu", "status": "planned", "price": 300_000, "buy": "2031-12", "fixCosts": 0, "inflation": false, "mortgage": null}]);
        v
    };
    for (plain, sold) in [
        (pure(base()), with_sale(pure(base()))),
        (combined(base()), with_sale(combined(base()))),
    ] {
        let (a, b) = (run(&plain), run(&sold));
        assert!(withdrawal_events(&b)
            .iter()
            .any(|(m, t)| *m == ym(2031, 11) && t.contains("reserved")));
        close(
            rec(&b, 2031, 11).withdrawal,
            rec(&a, 2031, 11).withdrawal,
            "Startentnahme ohne Reservierung",
        );
        close(
            rec(&b, 2031, 11).withdrawal * 12.0,
            RATE * 940_000.0,
            "4.3 % of 940'000",
        );
    }
}

/// #6/AK5: After an inflow the guardrails recompute the withdrawal (the same pure and combined); combined, the
/// money goes to bucket 3 and the targets stay until the next January.
#[test]
fn issue_6_ak5_inflow_recomputes_the_withdrawal_and_keeps_the_targets() {
    let inflow = |mut v: Value| {
        v["flows"]["extras"] = json!([{"date": "2032-05", "amount": 400_000, "label": "Erbe"}]);
        v
    };
    let (p, c) = (run(&inflow(pure(base()))), run(&inflow(combined(base()))));
    for r in [&p, &c] {
        // Rate after the inflow below 3.44 % (lower guardrail): 10 % more from May
        close(
            rec(r, 2032, 5).withdrawal,
            1.1 * rec(r, 2032, 4).withdrawal,
            "erhöht nach dem Zufluss",
        );
        assert!(r
            .events
            .iter()
            .any(|e| e.month == ym(2032, 5) && e.text.starts_with("Withdrawal strategy applied")));
    }
    let b = refill(&c, "B", 2032, 5);
    assert_eq!(
        b.moves.iter().map(|m| (m.from, m.to, m.amount)).collect::<Vec<_>>(),
        vec![(0, 3, 400_000.0)]
    );
    // No new targets until December (only January and A set the net need)
    assert!(!c
        .bucket_refills
        .iter()
        .any(|x| x.month > ym(2032, 1) && x.month <= ym(2032, 12) && (x.trigger == "A" || x.trigger == "Januar")));
    let jan = refill(&c, "Januar", 2032, 1);
    for x in c
        .bucket_refills
        .iter()
        .filter(|x| x.month > ym(2032, 5) && x.month <= ym(2032, 12) && x.trigger != "B")
    {
        close(x.need, jan.need, "Netto-Bedarf bis Januar");
    }
}

/// #6/AK6: The net need uses the guardrails' living costs and names the source; without limits and
/// housing costs it equals the withdrawal.
#[test]
fn issue_6_ak6_need_uses_the_guardrails_living() {
    let r = run(&combined(base()));
    let b0 = rec(&r, 2031, 11).withdrawal * 12.0;
    let a = refill(&r, "A", 2031, 11);
    assert_eq!(a.basis.living_source, "Guardrails");
    close(a.need, b0, "Netto-Bedarf = Entnahme");
    close(
        a.basis.living,
        12.0 * rec(&r, 2031, 11).spending,
        "Lebenshaltung hochgerechnet",
    );
    // The floor also applies in the net need: 400 % of the planned 1000 = 4000 per month (withdrawal about 3368)
    let mut v = combined(base());
    v["withdrawal"]["floor"] = json!(4);
    let r = run(&v);
    close(
        refill(&r, "A", 2031, 11).basis.living,
        48_000.0,
        "Lebenshaltung an der Untergrenze",
    );
    // Likewise the ceiling: 200 % of the planned 1000 = 2000 per month
    let mut v = combined(base());
    v["withdrawal"]["ceiling"] = json!(2);
    let r = run(&v);
    close(rec(&r, 2031, 11).spending, 2000.0, "Ausgaben an der Obergrenze");
    close(
        refill(&r, "A", 2031, 11).basis.living,
        24_000.0,
        "Lebenshaltung an der Obergrenze",
    );
    // The limits follow inflation (planned spending in today's prices)
    v["returns"]["fixed"]["inflation"] = json!(0.05);
    let r = run(&v);
    let nov = rec(&r, 2031, 11);
    assert!(nov.price_index > 1.2);
    close(nov.spending, 2000.0 * nov.price_index, "Obergrenze mit Teuerung");
    // «3-Topf» without guardrails: planned living costs
    let mut v = combined(base());
    v["withdrawal"]["buckets"]["spending"] = json!("planned");
    let r = run(&v);
    let a = refill(&r, "A", 2031, 11);
    assert_eq!(a.basis.living_source, "geplant");
    close(a.basis.living, 12_000.0, "geplant");
}

/// #6/AK7: After a cut by the guardrail the targets fall by the cut at the next January refill.
#[test]
fn issue_6_ak7_targets_follow_a_cut() {
    let r = run(&combined(base()));
    let (cut, _) = withdrawal_events(&r)
        .into_iter()
        .find(|(_, t)| t.contains("cut by"))
        .expect("Kürzung");
    assert_eq!(cut.month(), 1);
    let now = refill(&r, "Januar", cut.year(), 1);
    let before = refill(&r, "Januar", cut.year() - 1, 1);
    close(now.need, 0.9 * before.need, "Netto-Bedarf nach der Kürzung");
    close(now.target1, 0.9 * before.target1, "Ziel Topf 1");
    close(now.target2, 0.9 * before.target2, "Ziel Topf 2");
}

/// #6/AK8: Without the new field the plan stays unchanged, also when saving.
#[test]
fn issue_6_ak8_plans_without_the_field_stay_unchanged() {
    let mut v = base();
    v["withdrawal"]["strategy"] = json!("three_buckets");
    v["withdrawal"]["buckets"] = json!({"cashYears": 1, "bondYears": 1, "refillThreshold": -1});
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    assert!(!plan.withdrawal.buckets_with_guardrails());
    let saved: Value = serde_json::from_str(&plan.to_json()).unwrap();
    assert!(
        saved["withdrawal"]["buckets"].get("spending").is_none(),
        "«wie geplant» wird nicht gespeichert"
    );
    let with_guard: Plan = serde_json::from_value(combined(base())).unwrap();
    let saved: Value = serde_json::from_str(&with_guard.to_json()).unwrap();
    assert_eq!(saved["withdrawal"]["buckets"]["spending"], json!("guardrails"));
    let mut explicit = v.clone();
    explicit["withdrawal"]["buckets"]["spending"] = json!("planned");
    let (a, b) = (run(&v), run(&explicit));
    for (x, y) in a.records.iter().zip(&b.records) {
        assert_eq!(
            (x.spending, x.cash, x.bonds, x.stocks),
            (y.spending, y.cash, y.bonds, y.stocks)
        );
    }
    // An unknown value is an error
    v["withdrawal"]["buckets"]["spending"] = json!("vpw");
    assert!(serde_json::from_value::<Plan>(v).is_err());
}

/// #6: Checks of the guardrails and the limits also apply when combined.
#[test]
fn issue_6_validation_of_the_guardrails_and_limits() {
    let errors = |v: &Value| {
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        engine::validate::validate(&plan)
            .into_iter()
            .filter(|i| i.severity == engine::validate::Severity::Error)
            .map(|i| i.message)
            .collect::<Vec<_>>()
    };
    assert!(errors(&combined(base())).is_empty());
    let mut v = combined(base());
    v["withdrawal"]["guardrails"]["initialRate"] = json!(0);
    assert!(errors(&v).iter().any(|m| m.contains("initial rate")));
    let mut v = combined(base());
    v["withdrawal"]["floor"] = json!(1.2);
    v["withdrawal"]["ceiling"] = json!(1.1);
    assert!(errors(&v).iter().any(|m| m.contains("spending floor")));
    // Without guardrails their parameters are not checked with «3-Topf»
    v["withdrawal"]["buckets"]["spending"] = json!("planned");
    v["withdrawal"]["guardrails"]["initialRate"] = json!(0);
    assert!(errors(&v).is_empty());
}

/// #6/AK10: The optimization offers «3-Topf + Guardrails»; lower guardrail and adjustment are fixed by default.
#[test]
fn issue_6_ak10_optimization_offers_the_combination() {
    let defs = param_defs(OptStrategy::ThreeBucketsGuardrails);
    let varied: Vec<_> = defs.iter().filter(|d| !d.fixed).map(|d| d.key).collect();
    assert_eq!(
        varied,
        ["cashYears", "bondYears", "refillThreshold", "initialRate", "upper"]
    );
    let fixed: Vec<_> = defs.iter().filter(|d| d.fixed).map(|d| d.key).collect();
    assert_eq!(fixed, ["lower", "adjust", "floor", "ceiling"]);
    // The buckets have the same limits as with «3-Topf»; the return threshold may go down to −100 %
    let refill = defs.iter().find(|d| d.key == "refillThreshold").unwrap();
    assert_eq!((refill.lo, refill.hi, refill.min, refill.max), (-0.1, 0.2, -1.0, 1.0));
    assert_eq!(
        param_defs(OptStrategy::ThreeBuckets)
            .iter()
            .map(|d| (d.key, d.lo, d.hi, d.min, d.max))
            .collect::<Vec<_>>(),
        defs[..3]
            .iter()
            .map(|d| (d.key, d.lo, d.hi, d.min, d.max))
            .collect::<Vec<_>>()
    );
    // The plan of the combination is «3-Topf» with guardrails; «3-Topf» alone resets spending to planned.
    let retire = ym(2031, 11);
    let v = with_params(
        &base(),
        OptStrategy::ThreeBucketsGuardrails,
        &defs[3..4],
        &[0.035],
        retire,
    );
    assert_eq!(
        (&v["withdrawal"]["strategy"], &v["withdrawal"]["buckets"]["spending"]),
        (&json!("three_buckets"), &json!("guardrails"))
    );
    assert_eq!(v["withdrawal"]["guardrails"]["initialRate"], json!(0.035));
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(OptStrategy::of(&plan.withdrawal), OptStrategy::ThreeBucketsGuardrails);
    let back = with_params(&v, OptStrategy::ThreeBuckets, &[], &[], retire);
    assert_eq!(back["withdrawal"]["buckets"]["spending"], json!("planned"));
    // Other strategies set existing buckets to planned spending
    let other = with_params(&v, OptStrategy::FixedRate, &[], &[], retire);
    assert_eq!(other["withdrawal"]["buckets"]["spending"], json!("planned"));
    // Every strategy of the optimization gives a plan that is recognized again as the same strategy
    for s in engine::optimize::all_strategies() {
        let plan: Plan = serde_json::from_value(with_params(&v, s, &[], &[], retire)).unwrap();
        assert_eq!(OptStrategy::of(&plan.withdrawal), s);
    }
    // Without buckets in the plan: the combination creates them, other strategies do not
    let mut bare = base();
    bare["withdrawal"].as_object_mut().unwrap().remove("buckets");
    let c = with_params(&bare, OptStrategy::ThreeBucketsGuardrails, &[], &[], retire);
    assert_eq!(
        c["withdrawal"]["buckets"],
        json!({"cashYears": 2.0, "bondYears": 6.0, "refillThreshold": 0.0, "spending": "guardrails"})
    );
    assert!(
        with_params(&bare, OptStrategy::FixedRate, &[], &[], retire)["withdrawal"]
            .get("buckets")
            .is_none()
    );
}

// ---------------------------------------------------------------------------------------------
// #47
// ---------------------------------------------------------------------------------------------

/// #47/AK1–AK3: Fixed rate, VPW and endowment set the withdrawal in the month of the first retirement; a running
/// earned income counts as «Einkommen und Renten»; the limits apply from the same month.
#[test]
fn issue_47_dynamic_strategies_start_with_the_first_retirement() {
    let strategies = [
        ("fixed_rate", json!({"fixedRate": {"rate": 0.04}})),
        ("vpw", json!({"vpw": {"horizonAge": 100, "realReturn": 0.02}})),
        ("endowment", json!({"endowment": {"rate": 0.04, "smoothing": 0.3}})),
    ];
    for (key, params) in strategies {
        let mut v = couple();
        v["withdrawal"]["strategy"] = json!(key);
        for (k, x) in params.as_object().unwrap() {
            v["withdrawal"][k] = x.clone();
        }
        let r = run(&v);
        let first = withdrawal_events(&r)
            .into_iter()
            .find(|(_, t)| t.starts_with("Withdrawal"))
            .expect("Startentnahme");
        assert_eq!(first.0, ym(2031, 11), "{key}");
        let nov = rec(&r, 2031, 11);
        assert_eq!(nov.earned, 2000.0, "{key}");
        let rate = if key == "vpw" {
            0.02 / (1.0 - 1.02f64.powf(-35.0))
        } else {
            0.04
        };
        close(nov.withdrawal, rate * rec(&r, 2031, 10).liquid / 12.0, key);
        close(nov.spending, nov.withdrawal + nov.income, key);
        // Floor 1200 % of the planned 1000 from the same month (above withdrawal + income and pensions)
        v["withdrawal"]["floor"] = json!(12);
        close(rec(&run(&v), 2031, 11).spending, 12_000.0, key);
    }
}

/// #47/AK4: With one person nothing changes: the withdrawal starts with retirement.
#[test]
fn issue_47_single_person_starts_at_retirement() {
    let mut v = base();
    v["withdrawal"]["strategy"] = json!("fixed_rate");
    let r = run(&v);
    close(rec(&r, 2031, 10).spending, 1000.0, "vor der Pensionierung wie geplant");
    close(
        rec(&r, 2031, 11).withdrawal * 12.0,
        0.04 * 940_000.0,
        "ab Pensionierung",
    );
}
