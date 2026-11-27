//! #307: Mortgage rates follow the interest and inflation environment of the path.

use engine::batch;
use engine::calibrate::{self, corr};
use engine::market::{self, DataSet, FixedCurves};
use engine::model::{Investment, ParametricParams, Plan};
use engine::montecarlo::{Bootstrap, Parametric};
use engine::rates::RateHistory;
use engine::resolve::Curve;
use engine::sim::{EventKind, PropertyEventKind, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

use crate::plan_builder::PlanBuilder;

/// The series of the default investment strategy (world stocks, federal bonds 10 years), as the simulation uses it.
fn world() -> DataSet {
    let mut ds = engine::strategy::dataset(&Investment::default(), None).unwrap();
    RateHistory::load(None).unwrap().complete(&mut ds);
    ds
}

fn fixed(saron: f64) -> FixedCurves {
    FixedCurves {
        stocks: Curve::constant(0.0),
        bonds: Curve::constant(0.02),
        inflation: Curve::constant(0.01),
        saron: Curve::constant(saron),
    }
}

/// House with a mortgage of 400'000 from 01.2030, periods of one year, renewal in January 2031; fixed bonds 3 %,
/// SARON 0.5 %.
fn plan(period: Value, income: f64, affordability: Option<Value>) -> Plan {
    let mut v = PlanBuilder::new("Test").dates("2030-01", "2040-01", "2031-12").build();
    v["wealth"]["cash"] = json!(2_000_000);
    v["flows"]["income"] = json!(income);
    v["returns"]["fixed"]["bonds"] = json!(0.03);
    v["returns"]["fixed"]["saron"] = json!(0.005);
    v["realestate"] = json!({ "emergencyMonths": 6, "properties": [ {
        "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "never", "fixCosts": 0,
        "mortgage": { "tranches": [ { "amount": 400_000, "start": "2030-01", "after": "renew",
            "periods": [ period ] } ] }
    } ] });
    if let Some(a) = affordability {
        v["realestate"]["affordability"] = a;
    }
    serde_json::from_value(crate::any_age(v)).unwrap()
}

fn run(p: &Plan) -> SimResult {
    let sim = Simulator::new(p).unwrap();
    let paths = batch::market_paths(p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

fn renewal_text(r: &SimResult) -> String {
    r.events
        .iter()
        .find(|e| e.kind == EventKind::MortgageRenewal && e.month == YearMonth::new(2031, 1))
        .map(|e| e.text.to_string())
        .expect("Erneuerung Januar 2031")
}

fn renewal_rate(r: &SimResult) -> f64 {
    r.property_events
        .iter()
        .find(|e| e.kind == PropertyEventKind::Renewal && e.month == YearMonth::new(2031, 1))
        .and_then(|e| e.rate)
        .unwrap()
}

/// #307/AK3, AK4: A period «from the interest environment» takes the yield of the 10-year bonds of the path at the renewal
/// plus the margin of the settings: 3 % + 1.2 % = 4.2 %; the event shows the rate and its origin. The first period,
/// running at the start, keeps the entered 1 %.
#[test]
fn issue_307_ak3_ak4_fixed_rate_from_the_interest_environment_at_renewal() {
    let r = run(&plan(
        json!({ "type": "market", "term": 1, "rate": 0.01 }),
        20_000.0,
        None,
    ));
    assert!((renewal_rate(&r) - 0.042).abs() < 1e-12, "{}", renewal_rate(&r));
    assert_eq!(
        renewal_text(&r),
        "New mortgage period Haus: 4.20 % (10-year Swiss Confederation bonds 3.00 % + margin 1.20 %), 400000 CHF"
    );
    // Interest, paid at the end of the quarter: 400'000 × 1 % / 4 = 1'000 in 2030, 400'000 × 4.2 % / 4 = 4'200 from 2031
    let housing = |y: i32, m: u32| {
        r.records
            .iter()
            .find(|x| x.month == YearMonth::new(y, m))
            .unwrap()
            .housing
    };
    assert!((housing(2030, 6) - 1_000.0).abs() < 1e-6, "{}", housing(2030, 6));
    assert!((housing(2031, 6) - 4_200.0).abs() < 1e-6, "{}", housing(2031, 6));
    // Own margin of the settings: 3 % + 1.5 %
    let a = json!({ "sustainability": 0.33, "imputedRate": 0.05, "fixCosts": 0.01, "wealthContribution": 0.03,
        "maxLoanToValue": 0.8, "fixedMargin": 0.015 });
    let r = run(&plan(
        json!({ "type": "market", "term": 1, "rate": 0.01 }),
        20_000.0,
        Some(a),
    ));
    assert!((renewal_rate(&r) - 0.045).abs() < 1e-12);
}

/// #307/AK4: SARON and fixed periods name their origin, too.
#[test]
fn issue_307_ak4_events_name_the_origin_of_the_rate() {
    let r = run(&plan(
        json!({ "type": "saron", "term": 1, "margin": 0.01 }),
        20_000.0,
        None,
    ));
    assert_eq!(
        renewal_text(&r),
        "New mortgage period Haus: 1.50 % (SARON 0.50 % + margin 1.00 %), 400000 CHF"
    );
    let r = run(&plan(
        json!({ "type": "fixed", "term": 1, "rate": 0.02 }),
        20_000.0,
        None,
    ));
    assert_eq!(
        renewal_text(&r),
        "New mortgage period Haus: 2.00 % (fixed rate), 400000 CHF"
    );
}

/// #307/AK5: The affordability keeps computing with the imputed values: the same lump sum (income 6'000/month →
/// affordable 320'000, payment 80'000) whatever the rate of the new period.
#[test]
fn issue_307_ak5_affordability_unchanged() {
    let check = |period: Value| {
        let r = run(&plan(period, 6_000.0, None));
        let n = r
            .renewals
            .iter()
            .find(|x| x.month == YearMonth::new(2031, 1))
            .unwrap()
            .clone();
        (n.affordable, n.lump_sum, n.debt_after)
    };
    let fixed = check(json!({ "type": "fixed", "term": 1, "rate": 0.01 }));
    assert_eq!(fixed, (false, 80_000.0, 320_000.0));
    assert_eq!(check(json!({ "type": "market", "term": 1, "rate": 0.01 })), fixed);
}

/// #307/AK1: Historical path before SARON: the short-term rate of the same year (JST 1974: 5.97 %), not the fixed value;
/// the yield of the 10-year bonds of the same year (7.15 %).
#[test]
fn issue_307_ak1_historical_path_uses_the_short_term_rate() {
    let ds = world();
    let f = fixed(0.123);
    let p = market::historical_path(
        &ds,
        YearMonth::new(1974, 1),
        YearMonth::new(2030, 1),
        24,
        &Default::default(),
        &f,
    )
    .unwrap();
    assert!((p.saron[0] - 0.0597).abs() < 1e-6 && (p.saron[11] - 0.0597).abs() < 1e-6);
    assert!((p.saron[12] - 0.0275).abs() < 1e-6, "1975");
    assert!((p.yield10[0] - 0.0715).abs() < 1e-6);
}

/// #307/AK1: Bootstrap blocks take SARON from the same year as the inflation, also before 1999.
#[test]
fn issue_307_ak1_bootstrap_blocks_keep_saron_with_the_year() {
    let ds = world();
    let f = fixed(0.123);
    let b = Bootstrap {
        data: &ds,
        fixed: &f,
        start: YearMonth::new(2030, 1),
        months: 600,
        params: engine::model::BootstrapParams {
            paths: 5,
            block: 60.0,
            seed: 7,
        },
    };
    let (s, i, y) = (ds.saron.as_ref().unwrap(), &ds.inflation, ds.yield10.as_ref().unwrap());
    for n in 0..5 {
        let p = b.path(n);
        for k in 0..600 {
            assert_ne!(p.saron[k], 0.123, "fester Wert in Pfad {n}, Monat {k}");
            assert!(
                (0..ds.len()).any(|j| i[j] == p.inflation[k] && s[j] == p.saron[k] && y[j] == p.yield10[k]),
                "Pfad {n}, Monat {k}"
            );
        }
    }
}

/// #307/AK2: Parametric model: SARON = drawn inflation + real short-term rate (at least 0), bonds 10 years the term spread
/// above it; without the correlation (no series) the fixed SARON applies as before.
#[test]
fn issue_307_ak2_parametric_saron_follows_the_drawn_inflation() {
    let ds = world();
    let q = ParametricParams {
        paths: 3,
        real_short_rate: Some(0.01),
        ..Default::default()
    };
    let (q, c) = calibrate::resolve(&q, Some(&ds), 1950);
    let c = c.unwrap();
    assert_eq!(c.real_short, 0.01);
    let f = fixed(0.123);
    let g = Parametric {
        fixed: &f,
        start: YearMonth::new(2030, 1),
        months: 360,
        params: q.clone(),
        correlated: Some(c.clone()),
    };
    let p = g.path(0);
    for k in 0..360 {
        let s = (p.inflation[k] + 0.01).max(0.0);
        assert_eq!(p.saron[k], s, "Monat {k}");
        assert_eq!(p.yield10[k], (p.inflation[k] + 0.01 + c.term_spread).max(0.0));
    }
    assert!(
        p.saron.iter().any(|&s| s != p.saron[0]),
        "der SARON schwankt mit der Inflation"
    );
    let g = Parametric { correlated: None, ..g };
    assert!(g.path(0).saron.iter().all(|&s| s == 0.123));
    // Suggested value from the series: mean of SARON − inflation from 1950 (between −2 % and +2 %)
    let j = calibrate::suggestion(&ds, 1950).joint.unwrap();
    let r = j.real_short.unwrap();
    assert!((-0.02..0.02).contains(&r), "{r}");
    assert!(j.term_spread.unwrap() > 0.0);
}

/// #307/AK7: Test over the data: in the historical paths inflation and mortgage rate are positively correlated; the
/// path from 1972 has a SARON above 4 % in the high-interest phase (1973–1982).
#[test]
fn issue_307_ak7_rates_follow_inflation_in_history() {
    let ds = world();
    let f = fixed(0.0);
    let (start, months) = (YearMonth::new(2030, 1), 30 * 12);
    let (mut infl, mut rate) = (vec![], vec![]);
    for s in market::rolling_starts(&ds, start, months) {
        let p = market::historical_path(&ds, s, start, months, &Default::default(), &f).unwrap();
        // Mortgage rate of a SARON period with margin 1 %
        infl.extend(p.inflation.iter().step_by(12));
        rate.extend(p.saron.iter().step_by(12).map(|s| s.max(0.0) + 0.01));
    }
    let c = corr(&infl, &rate);
    assert!(c > 0.0, "Korrelation {c}");
    let p = market::historical_path(&ds, YearMonth::new(1972, 1), start, months, &Default::default(), &f).unwrap();
    let high = p.saron[12..132].iter().copied().fold(f64::MIN, f64::max);
    assert!(high > 0.04, "{high}");
}
