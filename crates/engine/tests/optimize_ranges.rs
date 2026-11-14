//! Bandbreiten und fixe Parameter der Optimierung.

use std::path::Path;

use engine::api;
use engine::model::ParamRange;
use engine::optimize::{catalog, run_request, split_defs, Ranges, Request};
use serde_json::json;

fn lib() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn plan() -> serde_json::Value {
    let mut p = api::example_plan();
    p["returns"]["source"] = json!("bootstrap");
    p["returns"]["series"] = json!("library:Shiller CHF.csv");
    p["returns"]["bootstrap"]["paths"] = json!(300);
    p["scenarios"]["sweeps"] = json!([]);
    p["withdrawal"]["buckets"] = json!({ "cashYears": 3.0, "bondYears": 5.0, "refillThreshold": 0.05 });
    p
}

fn range(fixed: Option<bool>, lo: Option<f64>, hi: Option<f64>) -> ParamRange {
    ParamRange { fixed, lo, hi }
}

fn ranges(strategy: &str, items: &[(&str, ParamRange)]) -> Ranges {
    let mut r = Ranges::new();
    r.insert(
        strategy.into(),
        items.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
    );
    r
}

fn request(strategy: &str, r: Ranges) -> Request {
    Request {
        strategies: vec![strategy.into()],
        paths: 300,
        check_paths: 300,
        samples: 6,
        ranges: r,
        ..Request::default()
    }
}

fn value(o: &engine::optimize::Outcome, key: &str) -> f64 {
    o.params
        .iter()
        .chain(&o.fixed)
        .find(|p| p.0 == key)
        .unwrap_or_else(|| panic!("{key}"))
        .2
}

#[test]
fn fixed_parameters_keep_the_plan_value_and_varied_ones_stay_in_their_range() {
    // Beispiel des Auftrags: Topf 1 fix, Topf 2 von 0 bis 2 Jahre, Nachfüllschwelle von -30 % bis +10 %.
    let r = ranges(
        "three_buckets",
        &[
            ("cashYears", range(Some(true), None, None)),
            ("bondYears", range(None, Some(0.0), Some(2.0))),
            ("refillThreshold", range(None, Some(-0.3), Some(0.1))),
        ],
    );
    let rep = run_request(&plan(), Path::new("/"), Some(&lib()), &request("three_buckets", r)).unwrap();
    let o = &rep.outcomes[0];
    assert_eq!(value(o, "cashYears"), 3.0);
    assert!(o.fixed.iter().any(|p| p.0 == "cashYears") && !o.params.iter().any(|p| p.0 == "cashYears"));
    let bond = value(o, "bondYears");
    assert!((0.0..=2.0).contains(&bond), "{bond}");
    let thr = value(o, "refillThreshold");
    assert!((-0.3..=0.1).contains(&thr), "{thr}");
    // Die Übernahme in die Planung enthält den fixen Wert unverändert.
    assert_eq!(o.patch["withdrawal"]["buckets"]["cashYears"], json!(3.0));
}

#[test]
fn everything_fixed_just_evaluates_the_plan() {
    let items: Vec<(&str, ParamRange)> = ["cashYears", "bondYears", "refillThreshold"]
        .iter()
        .map(|k| (*k, range(Some(true), None, None)))
        .collect();
    let rep = run_request(
        &plan(),
        Path::new("/"),
        Some(&lib()),
        &request("three_buckets", ranges("three_buckets", &items)),
    )
    .unwrap();
    let o = &rep.outcomes[0];
    assert!(o.params.is_empty() && o.fixed.len() == 3);
    assert_eq!(o.evaluations, 1);
    assert_eq!(value(o, "bondYears"), 5.0);
}

#[test]
fn invalid_ranges_are_reported() {
    let bad = ranges("guardrails", &[("initialRate", range(None, Some(0.08), Some(0.02)))]);
    let e = run_request(&plan(), Path::new("/"), Some(&lib()), &request("guardrails", bad))
        .unwrap_err()
        .to_string();
    assert!(e.contains("Startrate") && e.contains("ungültig"), "{e}");
    let out = ranges("three_buckets", &[("cashYears", range(None, Some(0.0), Some(99.0)))]);
    let e = split_defs(engine::optimize::OptStrategy::ThreeBuckets, &out)
        .unwrap_err()
        .to_string();
    assert!(e.contains("muss zwischen"), "{e}");
    // Die Prüfung der Planung meldet es ebenfalls, bevor etwas gerechnet wird.
    let mut p = plan();
    p["optimization"] = json!({ "ranges": { "guardrails": { "initialRate": { "lo": 0.08, "hi": 0.02 } } } });
    let issues = api::validate(&p);
    assert!(
        issues
            .iter()
            .any(|i| i.section == "optimize" && i.message.contains("Startrate")),
        "{issues:?}"
    );
}

#[test]
fn ranges_are_saved_with_the_plan_and_reach_the_request() {
    let mut p = plan();
    p["optimization"] =
        json!({ "ranges": { "three_buckets": { "bondYears": { "lo": 0, "hi": 2 }, "cashYears": { "fixed": true } } } });
    let parsed: engine::model::Plan = serde_json::from_value(p).unwrap();
    let back = engine::model::Plan::from_json(&parsed.to_json()).unwrap();
    assert_eq!(back.optimization, parsed.optimization);
    let req = Request::from(back.optimization.as_ref().unwrap());
    assert_eq!(
        req.ranges["three_buckets"]["bondYears"],
        range(None, Some(0.0), Some(2.0))
    );
    assert_eq!(req.ranges["three_buckets"]["cashYears"].fixed, Some(true));
    // ohne Bandbreiten wird nichts geschrieben
    let plain = engine::model::OptimizerSettings::default();
    assert!(!serde_json::to_string(&plain).unwrap().contains("ranges"));
}

#[test]
fn catalog_lists_every_parameter_with_unit_and_plan_value() {
    let c = catalog(&plan());
    assert_eq!(c.len(), 7);
    let b = c.iter().find(|s| s.key == "three_buckets").unwrap();
    let cash = b.params.iter().find(|p| p.key == "cashYears").unwrap();
    assert_eq!((cash.unit, cash.current, cash.percent), ("Jahre", Some(3.0), false));
    let refill = b.params.iter().find(|p| p.key == "refillThreshold").unwrap();
    assert_eq!((refill.unit, refill.percent), ("%", true));
    // Ober- und Untergrenze der Lebenshaltung sind wählbar, aber ohne eigene Einstellung fix
    let g = c.iter().find(|s| s.key == "guardrails").unwrap();
    assert!(g
        .params
        .iter()
        .filter(|p| p.fixed)
        .map(|p| p.key)
        .eq(["floor", "ceiling"]));
    // jeder Parameter hat eine Standard-Bandbreite innerhalb der äussersten Grenzen
    for s in &c {
        for p in &s.params {
            assert!(p.min <= p.lo && p.lo < p.hi && p.hi <= p.max, "{}/{}", s.key, p.key);
        }
    }
}

#[test]
fn results_show_the_withdrawal_rate_of_the_first_retirement_year() {
    // Fixe Rate von 4 %: Die Entnahme im ersten Jahr liegt nahe bei 4 % des liquiden Vermögens.
    let r = ranges(
        "fixed_rate",
        &[
            ("rate", range(Some(true), None, None)),
            ("stockShare", range(Some(true), None, None)),
        ],
    );
    let mut p = plan();
    p["withdrawal"]["fixedRate"] = json!({ "rate": 0.04 });
    let rep = run_request(&p, Path::new("/"), Some(&lib()), &request("fixed_rate", r)).unwrap();
    let rate = rep.outcomes[0].validated.draw_rate;
    assert!((0.02..0.07).contains(&rate), "Entnahmerate {rate}");
    assert!(rep.reference.metrics.draw_rate > 0.0);
}
