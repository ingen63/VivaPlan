//! #197: Comparison of the withdrawal strategies without optimization: every strategy with the parameter values of
//! the plan (default values where the plan has none), on the plan's own paths as in `run`, optionally with the switch
//! «Immobilien nie verkaufen» (#196). End wealth at 1, 5, 50, 95 and 99 % in every result.

use engine::api;
use engine::model::PropertyMode;
use engine::optimize::{run_request, Outcome, Report, Request, END_PERCENTS};
use serde_json::{json, Value};
use std::path::Path;

use crate::plan_builder::{parametric, PlanBuilder};

/// The example plan with 200 parametric paths.
fn example() -> Value {
    PlanBuilder::example().parametric(parametric(200, 5)).build()
}

fn compare(base: &Value, property: PropertyMode, strategies: &[&str]) -> Report {
    let req = Request {
        compare: true,
        property,
        strategies: strategies.iter().map(|s| s.to_string()).collect(),
        max_ruin: 0.5,
        ..Request::default()
    };
    run_request(base, Path::new("/"), None, &req).unwrap()
}

/// The plan with the result taken over (as «Übernehmen» in the user interface).
fn adopted(base: &Value, o: &Outcome) -> Value {
    let mut v = base.clone();
    v["withdrawal"] = o.patch["withdrawal"].clone();
    v["wealth"]["stockShare"] = o.patch["wealth"]["stockShare"].clone();
    v["wealth"]["reserveMonths"] = o.patch["wealth"]["reserveMonths"].clone();
    if o.patch["neverSell"] == json!(true) {
        v["realestate"]["neverSell"] = json!(true);
    }
    v
}

/// Figures of the result equal those of `run` of the same plan.
fn assert_same_as_run(base: &Value, o: &Outcome) {
    let out = api::run(&adopted(base, o), Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let s = out.batch.expect("several runs").summary;
    let m = &o.validated;
    let name = o.strategy.to_string();
    assert_eq!(m.runs, s.runs, "{name}");
    assert!(
        (m.ruin - (1.0 - s.success_rate)).abs() < 1e-12,
        "{name}: ruin {} vs {}",
        m.ruin,
        s.success_rate
    );
    assert!(
        (m.emergency_share - s.emergency_sale_runs as f64 / s.runs as f64).abs() < 1e-12,
        "{name}: emergency sales"
    );
    for (p, v) in END_PERCENTS.iter().zip(m.end_total) {
        let run = s.percentiles.iter().find(|x| x.percent == *p).unwrap().end_total;
        assert_eq!(v, run, "{name}: end wealth {p} %");
    }
}

// #197/AK5: with the example plan the figures of every strategy are those of `run` of the same plan.
#[test]
fn issue_197_ak5_compare_gives_the_figures_of_run() {
    let base = example();
    let r = compare(&base, PropertyMode::Planned, &["all"]);
    assert!(r.compare);
    assert_eq!(r.outcomes.len(), 7);
    assert_eq!((r.fit_paths, r.check_paths), (200, 200));
    for o in &r.outcomes {
        assert_same_as_run(&base, o);
    }
    // the plan as it is: same as run of the plan itself
    let out = api::run(&base, Path::new("/"), None);
    let s = out.batch.unwrap().summary;
    assert!((r.reference.metrics.ruin - (1.0 - s.success_rate)).abs() < 1e-12);
    let p50 = s.percentiles.iter().find(|x| x.percent == 50).unwrap().end_total;
    assert_eq!(r.reference.metrics.end_total[2], p50);
}

// #197/AK1: without search; parameter values of the plan, default values for a strategy the plan never used;
// the settings «variieren» (ranges) do not matter.
#[test]
fn issue_197_ak1_plan_values_without_search() {
    let mut base = example();
    // the plan never used VPW
    base["withdrawal"].as_object_mut().unwrap().remove("vpw");
    base["withdrawal"]["guardrails"]["initialRate"] = json!(0.043);
    let r = compare(&base, PropertyMode::Planned, &["guardrails", "vpw"]);
    let get = |key: &str| {
        r.outcomes
            .iter()
            .find(|o| o.patch["withdrawal"]["strategy"] == key)
            .unwrap()
    };
    let value = |o: &Outcome, k: &str| o.fixed.iter().find(|f| f.0 == k).unwrap().2;
    for o in &r.outcomes {
        assert!(o.params.is_empty(), "nothing varied");
        assert_eq!(o.evaluations, 1);
        assert_eq!(o.fitted.end_total, o.validated.end_total);
    }
    assert_eq!(value(get("guardrails"), "initialRate"), 0.043);
    let vpw = engine::model::Vpw::default();
    assert_eq!(value(get("vpw"), "horizonAge"), vpw.horizon_age);
    assert_eq!(value(get("vpw"), "realReturn"), vpw.real_return);
    // ranges and «variieren» are ignored
    let mut req = Request {
        compare: true,
        strategies: vec!["guardrails".into()],
        max_ruin: 0.5,
        ..Request::default()
    };
    req.ranges.insert(
        "guardrails".into(),
        [(
            "initialRate".to_string(),
            engine::model::ParamRange {
                fixed: Some(false),
                lo: Some(0.03),
                hi: Some(0.035),
            },
        )]
        .into_iter()
        .collect(),
    );
    let with_ranges = run_request(&base, Path::new("/"), None, &req).unwrap();
    assert_eq!(value(&with_ranges.outcomes[0], "initialRate"), 0.043);
    assert_eq!(
        with_ranges.outcomes[0].validated.end_total,
        get("guardrails").validated.end_total
    );
}

// #197/AK3: «both» shows every strategy twice (as planned and «never sell»), each with the figures of `run`;
// without a property only «as planned».
#[test]
fn issue_197_ak3_property_setting() {
    let base = example();
    let r = compare(&base, PropertyMode::Both, &["standard", "fixed_rate"]);
    assert_eq!(r.property, PropertyMode::Both);
    assert_eq!(r.outcomes.len(), 4);
    for key in ["standard", "fixed_rate"] {
        let mut flags: Vec<bool> = r
            .outcomes
            .iter()
            .filter(|o| o.patch["withdrawal"]["strategy"] == key)
            .map(|o| o.never_sell)
            .collect();
        flags.sort();
        assert_eq!(flags, [false, true], "{key}");
    }
    for o in &r.outcomes {
        assert_eq!(o.patch.get("neverSell").is_some(), o.never_sell);
        assert_same_as_run(&base, o);
        if o.never_sell {
            assert_eq!(o.validated.emergency_share, 0.0);
        }
    }
    let never = compare(&base, PropertyMode::NeverSell, &["standard"]);
    assert_eq!(never.outcomes.len(), 1);
    assert!(never.outcomes[0].never_sell);

    // without a property of its own, «both» and «never sell» compute only «as planned»
    let mut rented = base.clone();
    rented["realestate"]["properties"] = json!([]);
    for mode in [PropertyMode::Both, PropertyMode::NeverSell] {
        let r = compare(&rented, mode, &["standard"]);
        assert_eq!(r.property, PropertyMode::Planned);
        assert_eq!(r.outcomes.len(), 1);
        assert!(!r.outcomes[0].never_sell);
    }
}

// #197/AK2: the end wealth at 1, 5, 50, 95 and 99 % also after an optimization (on the check paths), for the
// plan and every strategy; the property setting applies to the optimization too.
#[test]
fn issue_197_ak2_end_wealth_after_optimization() {
    let req = Request {
        strategies: vec!["fixed_rate".into()],
        paths: 40,
        check_paths: 60,
        samples: 1,
        max_ruin: 0.5,
        property: PropertyMode::Both,
        ..Request::default()
    };
    let r = run_request(&example(), Path::new("/"), None, &req).unwrap();
    assert!(!r.compare);
    assert_eq!(r.outcomes.len(), 2);
    for m in std::iter::once(&r.reference.metrics).chain(r.outcomes.iter().map(|o| &o.validated)) {
        assert!(m.end_total.windows(2).all(|w| w[0] <= w[1]), "{:?}", m.end_total);
        assert!(m.end_total[4] > 0.0);
    }
    let json = serde_json::to_value(&r).unwrap();
    assert_eq!(
        json["outcomes"][0]["validated"]["endTotal"].as_array().unwrap().len(),
        5
    );
    assert_eq!(json["property"], "both");
    assert_eq!(json["compare"], false);
}

// #197: the property setting is saved with the optimization settings («as planned» is not written).
#[test]
fn issue_197_property_setting_is_saved() {
    let mut v = example();
    v["optimization"] = json!({ "property": "neverSell" });
    let p: engine::model::Plan = serde_json::from_value(v).unwrap();
    let s = p.optimization.as_ref().unwrap();
    assert_eq!(s.property, PropertyMode::NeverSell);
    assert_eq!(Request::from(s).property, PropertyMode::NeverSell);
    let back = serde_json::to_value(engine::model::OptimizerSettings::default()).unwrap();
    assert!(back.get("property").is_none());
}
