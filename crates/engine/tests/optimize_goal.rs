//! #20: Goal selection of the optimization. Goal 1 (consumption) stays unchanged, goal 2 («Kein Notverkauf von Immobilien») ranks by the
//! share of runs with an unplanned property sale and on a tie by the probability of ruin.

use engine::model::{Goal, OptimizerSettings};
use engine::optimize::{run_request, Metrics, Objective, Request};
use serde_json::{json, Value};
use std::path::Path;

use crate::plan_builder::{parametric, PlanBuilder};

fn example() -> Value {
    PlanBuilder::example().parametric(parametric(100, 3)).build()
}

fn request(goal: Goal) -> Request {
    Request {
        goal,
        strategies: vec!["standard".into(), "fixed_rate".into(), "guardrails".into()],
        paths: 100,
        check_paths: 200,
        samples: 2,
        max_ruin: 0.5,
        min_consumption: 0.0,
        ..Request::default()
    }
}

fn metrics(median: f64, ruin: f64, share: f64, planned: f64) -> Metrics {
    Metrics {
        runs: 100,
        ruin,
        median,
        p10: median * 0.8,
        volatility: 0.0,
        draw_rate: 0.04,
        emergency_share: share,
        planned,
        end_total: [0.0; 5],
    }
}

#[test]
fn goal_one_is_the_default_and_scores_consumption_as_before() {
    assert_eq!(Goal::default(), Goal::Consumption);
    assert_eq!(Request::default().goal, Goal::Consumption);
    assert_eq!(OptimizerSettings::default().goal, Goal::Consumption);
    assert_eq!(OptimizerSettings::default().min_consumption, 0.9);
    let o = Objective::default();
    // 0.5 × median + 0.5 × P10, penalty 1 × fluctuation (here 0)
    let m = metrics(100_000.0, 0.01, 0.3, 120_000.0);
    assert!((m.score(&o) - 90_000.0).abs() < 1e-9);
    // Goal 1 knows no minimum consumption condition
    assert!(m.feasible(&Objective {
        min_consumption: 0.99,
        ..o
    }));
}

#[test]
fn goal_two_prefers_fewer_emergency_sales_then_lower_ruin() {
    let o = Objective {
        goal: Goal::KeepProperty,
        min_consumption: 0.9,
        max_ruin: 0.05,
        ..Objective::default()
    };
    let few = metrics(100_000.0, 0.04, 0.10, 100_000.0);
    let many = metrics(100_000.0, 0.00, 0.20, 100_000.0);
    assert!(few.score(&o) > many.score(&o));
    let safer = metrics(100_000.0, 0.01, 0.10, 100_000.0);
    assert!(safer.score(&o) > few.score(&o));
}

#[test]
fn goal_two_constraints_are_ruin_limit_and_minimum_consumption() {
    let o = Objective {
        goal: Goal::KeepProperty,
        min_consumption: 0.9,
        max_ruin: 0.05,
        ..Objective::default()
    };
    assert!(metrics(95_000.0, 0.02, 0.0, 100_000.0).feasible(&o));
    assert!(
        !metrics(85_000.0, 0.02, 0.0, 100_000.0).feasible(&o),
        "Konsum unter 90 % der geplanten Ausgaben"
    );
    assert!(
        !metrics(95_000.0, 0.06, 0.0, 100_000.0).feasible(&o),
        "Ruin über der Grenze"
    );
    // The larger the violation, the further back
    assert!(
        metrics(50_000.0, 0.02, 0.0, 100_000.0).violation(&o) > metrics(85_000.0, 0.02, 0.0, 100_000.0).violation(&o)
    );
}

#[test]
fn goal_two_report_is_sorted_by_emergency_share() {
    let base = example();
    assert!(
        base["realestate"]["properties"]
            .as_array()
            .is_some_and(|p| !p.is_empty()),
        "Beispielplanung hat Immobilien"
    );
    let report = run_request(&base, Path::new("."), None, &request(Goal::KeepProperty)).expect("Optimierung");
    assert_eq!(report.goal, Goal::KeepProperty);
    let feasible: Vec<_> = report.outcomes.iter().filter(|o| o.feasible).collect();
    for w in feasible.windows(2) {
        let (a, b) = (&w[0].validated, &w[1].validated);
        assert!(
            a.emergency_share < b.emergency_share || (a.emergency_share == b.emergency_share && a.ruin <= b.ruin),
            "Reihenfolge: {} / {} gegen {} / {}",
            a.emergency_share,
            a.ruin,
            b.emergency_share,
            b.ruin
        );
    }
    // Goal 1 returns the same metrics fields but ranks by consumption
    let one = run_request(&base, Path::new("."), None, &request(Goal::Consumption)).expect("Optimierung");
    assert_eq!(one.goal, Goal::Consumption);
    let fp = |r: &engine::optimize::Report| {
        r.outcomes
            .iter()
            .filter(|o| o.feasible)
            .map(|o| o.validated.emergency_share)
            .fold(f64::INFINITY, f64::min)
    };
    // Goal 2 finds at most as many unplanned sales as goal 1
    assert!(fp(&report) <= fp(&one) + 1e-12 || report.outcomes.iter().all(|o| !o.feasible));
}

#[test]
fn goal_two_needs_a_property_and_a_sensible_minimum() {
    let mut base = example();
    let mut req = request(Goal::KeepProperty);
    req.min_consumption = 3.0;
    assert!(run_request(&base, Path::new("."), None, &req)
        .unwrap_err()
        .to_string()
        .contains("minimum consumption"));
    base["realestate"]["properties"] = json!([]);
    req.min_consumption = 0.9;
    assert!(run_request(&base, Path::new("."), None, &req)
        .unwrap_err()
        .to_string()
        .contains("property"));
    // Goal 1 needs no property
    assert!(run_request(&base, Path::new("."), None, &request(Goal::Consumption)).is_ok());
}

#[test]
fn goal_is_saved_in_the_plan_settings() {
    let mut v = example();
    v["optimization"] = json!({ "goal": "keepProperty", "minConsumption": 0.8 });
    let p: engine::model::Plan = serde_json::from_value(v).unwrap();
    let req = Request::from(p.optimization.as_ref().unwrap());
    assert_eq!(req.goal, Goal::KeepProperty);
    assert_eq!(req.min_consumption, 0.8);
    let back = serde_json::to_value(&p).unwrap();
    assert_eq!(back["optimization"]["goal"], "keepProperty");
}
