//! Gespeicherte Einstellungen der Optimierung in der Planung.

use engine::api;
use engine::model::{OptimizerSettings, Plan};
use engine::optimize::Request;
use serde_json::json;

fn plan_with(opt: serde_json::Value) -> Plan {
    let mut v = api::example_plan();
    v["optimization"] = opt;
    serde_json::from_value(v).expect("Planung mit Optimierung")
}

#[test]
fn settings_round_trip_with_the_plan() {
    let p = plan_with(json!({
        "maxRuin": 0.02, "ruinAge": 90, "weightMedian": 0.7, "volWeight": 2, "maxVol": 0.12,
        "paths": 3000, "checkPaths": 8000, "samples": 10, "strategies": ["guardrails", "vpw"]
    }));
    let o = p.optimization.clone().unwrap();
    assert_eq!((o.max_ruin, o.ruin_age, o.max_vol), (0.02, Some(90.0), Some(0.12)));
    assert_eq!(o.strategies, ["guardrails", "vpw"]);
    // Speichern und wieder lesen ergibt dieselben Einstellungen.
    let back = Plan::from_json(&p.to_json()).unwrap();
    assert_eq!(back.optimization, Some(o));
}

#[test]
fn missing_fields_use_defaults_and_absent_settings_are_not_written() {
    let p = plan_with(json!({ "maxRuin": 0.1 }));
    let o = p.optimization.unwrap();
    assert_eq!(
        o,
        OptimizerSettings {
            max_ruin: 0.1,
            ..OptimizerSettings::default()
        }
    );
    let plain: Plan = serde_json::from_value(api::example_plan()).unwrap();
    assert!(plain.optimization.is_none());
    assert!(!plain.to_json().contains("optimization"));
}

#[test]
fn request_takes_the_saved_settings() {
    let o = OptimizerSettings {
        max_ruin: 0.03,
        ruin_age: Some(95.0),
        strategies: vec!["fixed_rate".into()],
        ..Default::default()
    };
    let r = Request::from(&o);
    assert_eq!(
        (r.max_ruin, r.ruin_age, r.paths, r.check_paths, r.samples),
        (0.03, Some(95.0), 5000, 10_000, 30)
    );
    assert_eq!(r.strategy_list().unwrap().len(), 1);
}

#[test]
fn invalid_settings_are_reported_but_do_not_block() {
    let p = plan_with(json!({ "maxRuin": 1.5, "paths": 0 }));
    let issues = engine::validate::validate(&p);
    let opt: Vec<_> = issues.iter().filter(|i| i.section == "optimize").collect();
    assert_eq!(opt.len(), 2, "{issues:?}");
    assert!(opt
        .iter()
        .all(|i| matches!(i.severity, engine::validate::Severity::Warning)));
}
