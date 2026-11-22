//! #215: Variations and optimization reuse the return paths; the results stay bit-identical to computing every
//! variant on its own paths.

use std::path::{Path, PathBuf};

use engine::api;
use engine::batch::{self, Percentile};
use engine::model::Plan;
use engine::optimize::{self, Request};
use engine::sim::Simulator;
use serde_json::{json, Value};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn example(source: &str, paths: usize) -> Value {
    let mut p = api::example_plan();
    p["returns"]["source"] = json!(source);
    p["returns"]["series"] = json!("library:Shiller CHF.csv");
    p["returns"][source]["paths"] = json!(paths);
    p
}

fn without_real(percentiles: &[Percentile]) -> Value {
    let stripped: Vec<Percentile> = percentiles
        .iter()
        .map(|p| Percentile {
            end_total_real: None,
            ..p.clone()
        })
        .collect();
    serde_json::to_value(stripped).unwrap()
}

/// #215/AK3: Every combination of the variations of the example plan gives exactly (bit for bit) the result of a
/// run of that combination alone, which generates its own paths.
#[test]
fn issue_215_ak3_variations_match_single_runs() {
    let lib = library();
    for source in ["parametric", "bootstrap"] {
        let mut plan = example(source, 40);
        plan["scenarios"]["sweeps"] = json!([
            { "param": "flows.spendingFactor", "from": 0.9, "to": 1.1, "step": 0.1 },
            { "param": "wealth.stockShare", "from": 0.4, "to": 0.6, "step": 0.2 }
        ]);
        let out = api::run(&plan, Path::new("/"), Some(&lib));
        assert!(out.error.is_none(), "{source}: {:?}", out.error);
        let scenarios = out.scenarios.unwrap();
        assert_eq!(scenarios.results.len(), 6, "{source}");
        for r in &scenarios.results {
            assert!(r.error.is_none(), "{source} {:?}: {:?}", r.values, r.error);
            let mut single = plan.clone();
            single["scenarios"]["select"] = Value::Array(
                scenarios
                    .params
                    .iter()
                    .zip(&r.values)
                    .map(|(param, value)| json!({ "param": param, "value": value }))
                    .collect(),
            );
            let alone = api::run(&single, Path::new("/"), Some(&lib));
            let s = alone.batch.unwrap().summary;
            let at = |x: u32| s.percentiles.iter().find(|p| p.percent == x).unwrap().end_total;
            assert_eq!(r.runs, s.runs, "{source} {:?}", r.values);
            assert_eq!(
                r.success_rate.to_bits(),
                s.success_rate.to_bits(),
                "{source} {:?}",
                r.values
            );
            assert_eq!(
                [r.p5.to_bits(), r.p50.to_bits(), r.p95.to_bits()],
                [at(5).to_bits(), at(50).to_bits(), at(95).to_bits()],
                "{source} {:?}",
                r.values
            );
            assert_eq!(r.earliest_depletion, s.earliest_depletion.map(|(m, _)| m));
            assert_eq!(without_real(&r.percentiles), without_real(&s.percentiles));
        }
    }
}

/// #215/AK3: The comparison of the strategies on the example plan (8 evaluations on the same paths) gives exactly
/// the metrics of every strategy evaluated on its own, freshly generated paths.
#[test]
fn issue_215_ak3_optimization_compare_matches_own_paths() {
    let lib = library();
    let plan = example("parametric", 40);
    let req = Request {
        compare: true,
        ..Request::default()
    };
    let report = optimize::run_request(&plan, Path::new("/"), Some(&lib), &req).unwrap();
    let p: Plan = serde_json::from_value(plan.clone()).unwrap();
    let sim = Simulator::new(&p).unwrap();
    let retire = sim.retire_at().unwrap();
    let data = batch::load_series(&p, Path::new("/"), Some(&lib)).unwrap();
    let obj = req.objective();
    let fresh = || batch::path_source(&p, &sim, data.as_ref()).unwrap();
    assert_eq!(
        serde_json::to_value(report.reference.metrics).unwrap(),
        serde_json::to_value(optimize::evaluate(&p, &fresh(), &obj).unwrap()).unwrap()
    );
    assert_eq!(report.outcomes.len(), optimize::all_strategies().len());
    for s in optimize::all_strategies() {
        let alone = optimize::compare_strategy(&plan, s, retire, &fresh(), &obj).unwrap();
        let o = report
            .outcomes
            .iter()
            .find(|o| o.strategy == optimize::strategy_name(s))
            .unwrap();
        assert_eq!(
            serde_json::to_value(o).unwrap(),
            serde_json::to_value(&alone).unwrap(),
            "{:?}",
            optimize::strategy_key(s)
        );
    }
}
