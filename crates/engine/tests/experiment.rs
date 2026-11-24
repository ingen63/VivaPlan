//! #216: Experiment: changes, space of variations, metrics, constraints and objective in one call
//! (`experiment::evaluate`). Fictitious plans only.

use std::path::{Path, PathBuf};

use engine::experiment::{self, EvaluateOut, Ranked, Status};
use engine::optimize;
use serde_json::{json, Value};

use crate::plan_builder::{parametric, PlanBuilder};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn evaluate(plan: &Value, exp: &Value) -> EvaluateOut {
    experiment::evaluate(plan, exp, &library(), Some(&library()))
}

/// Computed result (status `done`).
fn done(plan: &Value, exp: &Value) -> experiment::ExperimentResult {
    let out = evaluate(plan, exp);
    assert_eq!(out.status, Status::Done, "{:?}", out.errors);
    out.result.expect("result")
}

/// Fictitious household: 55 years old at the start, retirement in 10 years (2040-01), end at 85.
fn plan(paths: usize) -> Value {
    let mut v = PlanBuilder::new("Experiment")
        .dates("2030-01", "2040-01", "2059-12")
        .parametric(parametric(paths, 7))
        .build();
    v["household"]["persons"][0]["birth"] = json!("1975-01");
    v["flows"] = json!({
        "income": { "mode": "timed", "rows": [
            { "from": "start", "value": 9000 }, { "from": "retire", "value": 2500 } ] },
        "spending": 6500
    });
    v["wealth"] = json!({ "stocks": 500000, "bonds": 300000, "cash": 50000, "stockShare": 0.6, "reserveMonths": 6 });
    v["withdrawal"] = json!({ "strategy": "standard", "endowment": { "rate": 0.04, "smoothing": 0.5 } });
    v
}

fn ruin() -> Value {
    json!({ "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } })
}

/// Mean of the real annual consumption in the first 15 years of retirement.
/// Median of the nominal end wealth.
fn end50() -> Value {
    json!({ "name": "end50", "quantity": "total", "reduce": "atEnd", "aggregate": { "kind": "quantile", "q": 0.5 } })
}

fn cons15() -> Value {
    json!({ "name": "cons15", "quantity": "consumption", "real": true, "window": { "from": "retire", "years": 15 },
        "reduce": "annualSums", "aggregate": { "kind": "mean" } })
}

fn setting<'a>(r: &'a Ranked, path: &str) -> Option<&'a Value> {
    r.settings.iter().find(|s| s.path == path).map(|s| &s.value)
}

fn figure(e: &experiment::Evaluation, name: &str) -> f64 {
    e.figures
        .iter()
        .find(|f| f.name == name)
        .and_then(|f| f.value)
        .expect(name)
}

fn codes(out: &EvaluateOut) -> Vec<&'static str> {
    out.errors.iter().map(|m| m.code()).collect()
}

/// Parameter `name` of the message as JSON.
fn param(m: &engine::message::Message, name: &str) -> Value {
    serde_json::to_value(m).unwrap()["params"][name].clone()
}

/// Metric values of `plan` with the changes `set` on its own paths (reference without the experiment).
fn measured(plan: &Value, set: &[&str], metrics: &[Value]) -> Vec<Option<f64>> {
    let set: Vec<String> = set.iter().map(|s| s.to_string()).collect();
    let v = engine::api::set_fields(plan, &set).expect("set");
    engine::api::measure(&v, metrics, &library(), Some(&library()))
        .expect("measure")
        .into_iter()
        .map(|m| m.value)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// AK1: schema and messages
// ---------------------------------------------------------------------------------------------

/// #216/AK1: The schema of the experiment is part of `schema_json()`.
#[test]
fn issue_216_ak1_schema_contains_the_experiment() {
    let v: Value = serde_json::from_str(&engine::schema_json()).unwrap();
    assert_eq!(v["title"], "Plan");
    let e = &v["definitions"]["Experiment"];
    for key in [
        "set",
        "vary",
        "metrics",
        "constraints",
        "objective",
        "search",
        "budget",
        "confirm",
    ] {
        assert!(e["properties"].get(key).is_some(), "{key}");
    }
    for def in [
        "Variation",
        "GroupMember",
        "Constraint",
        "ExperimentObjective",
        "ExperimentBudget",
        "Metric",
    ] {
        assert!(v["definitions"].get(def).is_some(), "{def}");
    }
    // The definitions of the plan stay as they are.
    let plan_only = serde_json::to_value(schemars::schema_for!(engine::model::Plan)).unwrap();
    for (k, d) in plan_only["definitions"].as_object().unwrap() {
        assert_eq!(&v["definitions"][k], d, "{k}");
    }
}

/// #216/AK1: An invalid experiment gives a list of messages with path, reason and valid values; nothing is computed.
#[test]
fn issue_216_ak1_invalid_experiment_lists_messages() {
    let p = plan(50);
    let out = evaluate(
        &p,
        &json!({ "varry": [], "constraints": [ { "metric": "ruin", "op": "<=", "value": 0.02 } ] }),
    );
    assert_eq!(out.status, Status::Invalid);
    assert!(out.result.is_none() && out.estimate.is_none());
    let unknown = out
        .errors
        .iter()
        .find(|m| m.code() == "experiment_key_unknown")
        .expect("unknown key");
    assert_eq!(param(unknown, "path"), "varry");
    assert!(param(unknown, "valid").as_array().unwrap().contains(&json!("vary")));
    let wrong = out
        .errors
        .iter()
        .find(|m| m.code() == "experiment_value_wrong")
        .expect("op");
    assert_eq!(param(wrong, "path"), "constraints[0].op");
    assert_eq!(param(wrong, "expected"), json!(["ge", "gt", "le", "lt"]));

    // Content: unknown field path, empty range, unknown metric, two objectives, missing coupling.
    let out = evaluate(
        &p,
        &json!({
            "vary": [
                { "path": "wealth.stockShar", "values": [0.5] },
                { "path": "wealth.stockShare@start", "range": [0.8, 0.4, 0.1] },
                { "path": "wealth.stockShare@retire", "follows": "wealth.cash", "offset": -0.2 },
                { "path": "wealth.stockShare", "values": [0.5], "range": [0.1, 0.2, 0.1] }
            ],
            "metrics": [ ruin() ],
            "constraints": [ { "metric": "ruin15", "op": "le", "value": 0.02 } ],
            "objective": { "maximize": "ruin", "minimize": "ruin" }
        }),
    );
    assert_eq!(out.status, Status::Invalid);
    assert!(out.result.is_none() && out.estimate.is_none());
    let c = codes(&out);
    for code in [
        "experiment_invalid",
        "experiment_range_invalid",
        "experiment_follows_unknown",
        "experiment_variation_form",
        "experiment_metric_unknown",
        "experiment_objective_form",
    ] {
        assert!(c.contains(&code), "{code}: {c:?}");
    }
    let path = out.errors.iter().find(|m| m.code() == "experiment_invalid").unwrap();
    assert_eq!(param(path, "path"), "vary[0].values[0]");
    assert_eq!(param(path, "problem")["code"], "field_path_unknown");
    let metric = out
        .errors
        .iter()
        .find(|m| m.code() == "experiment_metric_unknown")
        .unwrap();
    assert_eq!(param(metric, "valid"), json!(["ruin"]));
    let range = out
        .errors
        .iter()
        .find(|m| m.code() == "experiment_range_invalid")
        .unwrap();
    assert_eq!(param(range, "path"), "vary[1].range");

    // Wrong type of a value and a window that does not exist.
    let out = evaluate(
        &p,
        &json!({
            "vary": [ { "path": "withdrawal.strategy", "values": ["fixed_rate", "sometimes"] } ],
            "metrics": [ { "name": "x", "quantity": "total", "window": { "from": "nowhere" }, "reduce": "atEnd",
                "aggregate": { "kind": "mean" } } ]
        }),
    );
    assert_eq!(out.status, Status::Invalid);
    let wrong = out
        .errors
        .iter()
        .find(|m| m.code() == "experiment_invalid")
        .expect("value");
    assert_eq!(param(wrong, "path"), "vary[0].values[1]");
    assert_eq!(param(wrong, "problem")["code"], "field_type_wrong");
    assert!(codes(&out).contains(&"metric_invalid"), "{:?}", codes(&out));
}

// ---------------------------------------------------------------------------------------------
// AK2, AK3: field paths, lists, ranges, groups, couplings
// ---------------------------------------------------------------------------------------------

/// #216/AK2: Field paths with a point in time, as a list, a range and a group; members of a group are not combined
/// with each other, and every variant computes as the plan with the same changes.
#[test]
fn issue_216_ak2_lists_ranges_groups_and_points_in_time() {
    let p = plan(40);
    let r = done(
        &p,
        &json!({
            "set": { "flows.spending": 6000 },
            "vary": [
                { "path": "wealth.stockShare@start", "values": [0.4, 0.8] },
                { "path": "wealth.stockShare@retire", "range": [0.3, 0.5, 0.1] },
                { "group": [
                    { "set": { "withdrawal.strategy": "fixed_rate" },
                      "vary": [ { "path": "withdrawal.fixedRate.rate", "values": [0.03, 0.05] } ] },
                    { "set": { "withdrawal.strategy": "endowment" } }
                ] }
            ],
            "metrics": [ ruin(), cons15() ],
            "budget": { "paths": 40 }
        }),
    );
    // 2 × 3 × (2 + 1)
    assert_eq!(r.variants, 18);
    assert_eq!(r.listed.len(), 18);
    assert!(r.ranking.is_empty());
    let retire: Vec<f64> = r
        .listed
        .iter()
        .filter_map(|x| setting(x, "wealth.stockShare@retire"))
        .map(|v| v.as_f64().unwrap())
        .collect();
    let mut distinct = retire.clone();
    distinct.sort_by(f64::total_cmp);
    distinct.dedup();
    assert_eq!(distinct, [0.3, 0.4, 0.5], "range without binary noise");
    for x in &r.listed {
        let strategy = setting(x, "withdrawal.strategy").unwrap();
        let rate = setting(x, "withdrawal.fixedRate.rate");
        assert_eq!(
            rate.is_some(),
            strategy == "fixed_rate",
            "group members are not combined: {:?}",
            x.settings
        );
    }
    // A variant computes as the plan with the same changes (#207 field paths).
    let x = r
        .listed
        .iter()
        .find(|x| {
            setting(x, "withdrawal.fixedRate.rate") == Some(&json!(0.05))
                && setting(x, "wealth.stockShare@start") == Some(&json!(0.8))
                && setting(x, "wealth.stockShare@retire") == Some(&json!(0.4))
        })
        .expect("variant");
    let want = measured(
        &p,
        &[
            "flows.spending=6000",
            "wealth.stockShare@start=0.8",
            "wealth.stockShare@retire=0.4",
            "withdrawal.strategy=\"fixed_rate\"",
            "withdrawal.fixedRate.rate=0.05",
        ],
        &[ruin(), cons15()],
    );
    let got: Vec<Option<f64>> = x.search.figures.iter().map(|f| f.value).collect();
    assert_eq!(got, want);
}

/// #216/AK3: A coupled value follows another one with an offset.
#[test]
fn issue_216_ak3_coupled_value_follows_with_offset() {
    let p = plan(40);
    let r = done(
        &p,
        &json!({
            "vary": [
                { "path": "wealth.stockShare@start", "values": [0.5, 0.7] },
                { "path": "wealth.stockShare@retire", "follows": "wealth.stockShare@start", "offset": -0.2 }
            ],
            "metrics": [ cons15() ],
            "budget": { "paths": 40 }
        }),
    );
    assert_eq!(r.listed.len(), 2);
    for (x, (start, retire)) in r.listed.iter().zip([(0.5, 0.3), (0.7, 0.5)]) {
        assert_eq!(setting(x, "wealth.stockShare@start"), Some(&json!(start)));
        assert_eq!(setting(x, "wealth.stockShare@retire"), Some(&json!(retire)));
    }
    let want = measured(
        &p,
        &["wealth.stockShare@start=0.7", "wealth.stockShare@retire=0.5"],
        &[cons15()],
    );
    assert_eq!(r.listed[1].search.figures[0].value, want[0]);
}

// ---------------------------------------------------------------------------------------------
// AK4: constraints and objective
// ---------------------------------------------------------------------------------------------

fn spending_experiment(objective: Value, constraints: Value) -> Value {
    json!({
        "vary": [ { "path": "flows.spending", "values": [4000, 6000, 8000, 11000] } ],
        "metrics": [ ruin(), cons15() ],
        "constraints": constraints,
        "objective": objective,
        "budget": { "paths": 300, "confirmPaths": 300, "top": 4, "margin": 0 },
        "confirm": true
    })
}

fn spending_order(r: &experiment::ExperimentResult) -> Vec<i64> {
    r.ranking
        .iter()
        .map(|x| setting(x, "flows.spending").unwrap().as_i64().unwrap())
        .collect()
}

/// #216/AK4: Constraints `{metric, op, value}`; objective `maximize`, `minimize` or a weighted list.
#[test]
fn issue_216_ak4_constraints_and_objectives() {
    let p = plan(300);
    // Without a constraint more spending means more consumption.
    let r = done(&p, &spending_experiment(json!({ "maximize": "cons15" }), json!([])));
    assert_eq!(spending_order(&r), [11000, 8000, 6000, 4000]);
    // The weighted list with one weight is the same objective.
    let w = done(
        &p,
        &spending_experiment(
            json!({ "weighted": [ { "metric": "cons15", "weight": 1.0 } ] }),
            json!([]),
        ),
    );
    assert_eq!(spending_order(&w), spending_order(&r));
    // Minimize the ruin: the smallest spending first.
    let m = done(&p, &spending_experiment(json!({ "minimize": "ruin" }), json!([])));
    assert_eq!(spending_order(&m)[0], 4000);
    // With the ruin limit the highest spending is out: it is ranked last and not feasible.
    let c = done(
        &p,
        &spending_experiment(
            json!({ "maximize": "cons15" }),
            json!([ { "metric": "ruin", "op": "le", "value": 0.02 } ]),
        ),
    );
    let last = c.ranking.last().unwrap();
    let conf = last.confirm.as_ref().unwrap();
    assert!(figure(conf, "ruin") > 0.02 && !conf.feasible, "{:?}", conf);
    assert!(c.ranking[0].confirm.as_ref().unwrap().feasible);
    assert!(figure(c.ranking[0].confirm.as_ref().unwrap(), "ruin") <= 0.02);
}

// ---------------------------------------------------------------------------------------------
// AK5, AK6: two stages, ranking, hints, traceability
// ---------------------------------------------------------------------------------------------

/// #216/AK5: Search on few paths, confirmation of the best `top` on independent paths (other seed); the result shows
/// both values.
#[test]
fn issue_216_ak5_search_and_independent_confirmation() {
    let p = plan(100);
    let exp = json!({
        "vary": [ { "path": "wealth.stockShare", "values": [0.3, 0.6, 0.9] } ],
        "metrics": [ ruin(), cons15(), end50() ],
        "constraints": [ { "metric": "ruin", "op": "le", "value": 0.05 } ],
        "objective": { "maximize": "cons15" },
        "budget": { "paths": 200, "confirmPaths": 400, "top": 2 },
        "confirm": true
    });
    let r = done(&p, &exp);
    assert_eq!((r.paths.search, r.paths.confirm), (200, 400));
    assert!(r.paths.independent);
    assert_eq!(r.paths.search_seed, Some(7));
    assert_eq!(r.paths.confirm_seed, Some(7 + 7919));
    assert_eq!(r.ranking.len(), 2);
    let best = &r.ranking[0];
    let share = setting(best, "wealth.stockShare").unwrap().to_string();
    // Search values on the search paths, confirmed values on the independent paths, both shown.
    let mut search_plan = p.clone();
    search_plan["returns"]["parametric"]["paths"] = json!(200);
    let mut confirm_plan = p.clone();
    confirm_plan["returns"]["parametric"]["paths"] = json!(400);
    confirm_plan["returns"]["parametric"]["seed"] = json!(7 + 7919);
    let set = format!("wealth.stockShare={share}");
    let s = measured(&search_plan, &[&set], &[ruin(), cons15(), end50()]);
    let c = measured(&confirm_plan, &[&set], &[ruin(), cons15(), end50()]);
    let got_s: Vec<Option<f64>> = best.search.figures.iter().map(|f| f.value).collect();
    let got_c: Vec<Option<f64>> = best.confirm.as_ref().unwrap().figures.iter().map(|f| f.value).collect();
    assert_eq!(got_s, s);
    assert_eq!(got_c, c);
    assert_ne!(got_s, got_c);

    // Pattern search within the range (as `search` of the optimization), confirmed in the same way.
    let pattern = json!({
        "vary": [ { "path": "flows.spending", "range": [4000, 12000, 100] } ],
        "metrics": [ ruin(), cons15() ],
        "constraints": [ { "metric": "ruin", "op": "le", "value": 0.05 } ],
        "objective": { "maximize": "cons15" },
        "search": { "method": "pattern", "samples": 3 },
        "budget": { "paths": 200, "confirmPaths": 400 },
        "confirm": true
    });
    let r = done(&p, &pattern);
    assert_eq!(r.ranking.len(), 1);
    let spending = setting(&r.ranking[0], "flows.spending").unwrap().as_f64().unwrap();
    assert!((4000.0..=12000.0).contains(&spending), "{spending}");
    assert!(r.variants > 5, "{}", r.variants);
    let search = &r.ranking[0].search;
    assert!(search.feasible && figure(search, "ruin") <= 0.05);
    assert!(r.ranking[0].confirm.is_some());
}

/// #216/AK6: Ranking of the best `top` with values, metrics and standard errors, the hint «at the edge of the grid»
/// and the complete experiment with version and seed.
#[test]
fn issue_216_ak6_ranking_edge_hint_and_traceability() {
    let p = plan(100);
    let r = done(
        &p,
        &json!({
            "vary": [ { "path": "flows.spending", "values": [4000, 5000, 6000] } ],
            "metrics": [ ruin(), cons15() ],
            "objective": { "maximize": "cons15" },
            "budget": { "paths": 100, "confirmPaths": 200, "top": 2 },
            "confirm": true
        }),
    );
    assert_eq!(r.ranking.len(), 2);
    assert_eq!(r.ranking.iter().map(|x| x.rank).collect::<Vec<_>>(), [1, 2]);
    let a = r.ranking[0].confirm.as_ref().unwrap().objective.unwrap();
    let b = r.ranking[1].confirm.as_ref().unwrap().objective.unwrap();
    assert!(a >= b);
    let ruin = r.ranking[0].search.figures.iter().find(|f| f.name == "ruin").unwrap();
    assert!(ruin.std_error.is_some(), "standard error of the probability");
    // The best value 6000 is the upper edge of the grid.
    assert_eq!(setting(&r.ranking[0], "flows.spending"), Some(&json!(6000)));
    let hint = r
        .hints
        .iter()
        .find(|h| h.code() == "experiment_edge_high")
        .expect("hint");
    assert_eq!(param(hint, "path"), "flows.spending");
    // The complete experiment with defaults, the version of the engine and the seeds.
    assert_eq!(r.experiment.budget.margin, 2.0);
    assert_eq!(r.experiment.search.method, experiment::Method::Grid);
    assert_eq!(r.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(r.paths.search_seed, Some(7));
    let json = serde_json::to_value(&r).unwrap();
    assert_eq!(json["experiment"]["budget"]["confirmPaths"], 200);
    assert_eq!(json["experiment"]["vary"][0]["values"], json!([4000, 5000, 6000]));
}

// ---------------------------------------------------------------------------------------------
// AK7: estimate, progress, confirmation
// ---------------------------------------------------------------------------------------------

/// #216/AK7: The duration is estimated before the start; from 60 s the computation needs the confirmation. Here a long
/// plan with 19 million runs (at least about a minute on a fast computer).
#[test]
fn issue_216_ak7_long_experiment_needs_confirmation() {
    let mut p = plan(100);
    p["milestones"][2]["date"] = json!("2079-12");
    let exp = json!({
        "vary": [ { "path": "flows.spending", "range": [3000, 12495, 5] } ],
        "metrics": [ cons15() ],
        "budget": { "paths": 10000 }
    });
    let out = evaluate(&p, &exp);
    assert_eq!(out.status, Status::NeedsConfirmation, "{:?}", out.errors);
    assert!(out.result.is_none());
    let e = out.estimate.expect("estimate");
    assert_eq!(e.variants, 1900);
    assert_eq!(e.runs, 19_000_000);
    assert!(e.seconds.unwrap() >= experiment::CONFIRM_SECONDS, "{e:?}");
    assert_eq!(out.errors[0].code(), "experiment_confirm");
}

/// #216/AK7: A short experiment computes at once, with the estimate; the progress counts all runs.
#[test]
fn issue_216_ak7_estimate_and_progress_of_a_short_experiment() {
    let p = plan(100);
    let exp = json!({
        "vary": [ { "path": "flows.spending", "values": [5000, 6000] } ],
        "metrics": [ cons15() ],
        "objective": { "maximize": "cons15" },
        "budget": { "paths": 100, "confirmPaths": 150, "top": 1 }
    });
    let out = evaluate(&p, &exp);
    assert_eq!(out.status, Status::Done, "{:?}", out.errors);
    let e = out.estimate.expect("estimate");
    assert_eq!((e.variants, e.runs), (2, 2 * 100 + 150));
    assert!(e.seconds.unwrap() < experiment::CONFIRM_SECONDS);
    let progress = engine::progress::snapshot();
    assert!(
        !progress.active && progress.done == 350 && progress.total == 350,
        "{progress:?}"
    );
}

/// #216/AK7: A cancellation stops the experiment (process of its own, as all tests with nextest).
#[test]
fn issue_216_ak7_cancel_stops_the_experiment() {
    use std::time::{Duration, Instant};
    let p = plan(100);
    let exp = json!({
        "vary": [ { "path": "flows.spending", "range": [3000, 12000, 50] } ],
        "metrics": [ cons15() ],
        "budget": { "paths": 2000 },
        "confirm": true
    });
    let w = std::thread::spawn(|| {
        let t0 = Instant::now();
        loop {
            let s = engine::progress::snapshot();
            if s.active && s.done > 0 {
                engine::progress::cancel();
                return;
            }
            assert!(t0.elapsed() < Duration::from_secs(60), "not started");
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    let out = evaluate(&p, &exp);
    w.join().unwrap();
    assert_eq!(out.status, Status::Cancelled, "{:?}", out.errors);
    assert!(out.result.is_none());
}

// ---------------------------------------------------------------------------------------------
// AK8: variations, comparison of strategies and optimization as experiments
// ---------------------------------------------------------------------------------------------

/// #216/AK8: Variations (`scenarios.sweeps`) as a grid without an objective: same numbers as `run`.
#[test]
fn issue_216_ak8_variations_as_experiment() {
    let mut p = plan(150);
    p["scenarios"] = json!({ "sweeps": [ { "param": "wealth.stockShare", "from": 0.3, "to": 0.9, "step": 0.3 } ] });
    let run = engine::api::run(&p, &library(), Some(&library()));
    let sc = run.scenarios.expect("variations");
    let values: Vec<Value> = sc.results.iter().map(|s| s.values[0].clone()).collect();
    let never = json!({ "op": "le", "value": 0.0 });
    let quantile = |name: &str, q: f64| json!({ "name": name, "quantity": "total", "reduce": "atEnd", "aggregate": { "kind": "quantile", "q": q } });
    let r = done(
        &p,
        &json!({
            "vary": [ { "path": "wealth.stockShare", "values": values } ],
            "metrics": [
                { "name": "success", "quantity": "depleted", "reduce": "atEnd",
                  "aggregate": { "kind": "probability", "condition": never } },
                quantile("p5", 0.05), quantile("p50", 0.5), quantile("p95", 0.95)
            ],
            "budget": { "paths": 150 }
        }),
    );
    assert_eq!(r.listed.len(), sc.results.len());
    for (x, s) in r.listed.iter().zip(&sc.results) {
        let f = |n: &str| figure(&x.search, n).to_bits();
        assert_eq!(f("success"), s.success_rate.to_bits());
        assert_eq!(f("p5"), s.p5.to_bits());
        assert_eq!(f("p50"), s.p50.to_bits());
        assert_eq!(f("p95"), s.p95.to_bits());
    }
}

fn optimization_metrics(m: &optimize::Metrics) -> Value {
    serde_json::to_value(m).unwrap()
}

/// #216/AK8: The comparison of strategies (`optimize --compare`) as a group of strategies with the objective `score`
/// on the plan's own paths: same numbers.
#[test]
fn issue_216_ak8_strategy_comparison_as_experiment() {
    let mut p = plan(200);
    p["pension"]["pk"] = json!({ "capital": 200000, "at": "retire", "lumpsum": 1, "conversion": 0 });
    let strategies = ["fixed_rate", "vpw", "endowment"];
    let req = json!({ "compare": true, "strategies": strategies });
    let report = engine::api::optimize(&p, &req, &library(), Some(&library())).expect("compare");
    let r = done(
        &p,
        &json!({
            "vary": [ { "group": strategies.iter().map(|s| json!({ "set": { "withdrawal.strategy": s } })).collect::<Vec<_>>() } ],
            "objective": { "score": {} },
            "budget": { "paths": 200, "top": 3, "confirmPaths": 200 },
            "confirm": true
        }),
    );
    assert_eq!(r.ranking.len(), 3);
    for o in &report.outcomes {
        let key = strategies
            .iter()
            .find(|k| {
                o.strategy
                    == optimize::strategy_name(
                        optimize::all_strategies()
                            .into_iter()
                            .find(|s| optimize::strategy_key(*s) == **k)
                            .unwrap(),
                    )
            })
            .unwrap();
        let x = r
            .ranking
            .iter()
            .find(|x| setting(x, "withdrawal.strategy") == Some(&json!(key)))
            .unwrap();
        assert_eq!(
            optimization_metrics(x.search.optimization.as_ref().unwrap()),
            optimization_metrics(&o.fitted),
            "{key}"
        );
    }
}

/// #216/AK8: The optimization of a strategy as a pattern search with the objective `score`: same parameter and the
/// same numbers on search and check paths.
#[test]
fn issue_216_ak8_optimization_as_experiment() {
    let mut p = plan(100);
    p["pension"]["pk"] = json!({ "capital": 200000, "at": "retire", "lumpsum": 1, "conversion": 0 });
    let req = json!({
        "strategies": ["fixed_rate"], "paths": 200, "checkPaths": 400, "samples": 4, "seed": 3, "maxRuin": 0.1,
        "ranges": { "fixed_rate": { "stockShare": { "fixed": true } } }
    });
    let report = engine::api::optimize(&p, &req, &library(), Some(&library())).expect("optimize");
    let o = &report.outcomes[0];
    let r = done(
        &p,
        &json!({
            "set": { "withdrawal.strategy": "fixed_rate" },
            "vary": [ { "path": "withdrawal.fixedRate.rate", "range": [0.02, 0.08, 0.001] } ],
            "objective": { "score": { "maxRuin": 0.1 } },
            "search": { "method": "pattern", "samples": 4, "seed": 3 },
            "budget": { "paths": 200, "margin": 1.5, "top": 1, "confirmPaths": 400 },
            "confirm": true
        }),
    );
    let best = &r.ranking[0];
    assert_eq!(setting(best, "withdrawal.fixedRate.rate"), Some(&json!(o.params[0].2)));
    assert_eq!(
        optimization_metrics(best.search.optimization.as_ref().unwrap()),
        optimization_metrics(&o.fitted)
    );
    assert_eq!(
        optimization_metrics(best.confirm.as_ref().unwrap().optimization.as_ref().unwrap()),
        optimization_metrics(&o.validated)
    );
    assert_eq!(best.confirm.as_ref().unwrap().feasible, o.feasible);
}

// ---------------------------------------------------------------------------------------------
// AK10: example of the issue
// ---------------------------------------------------------------------------------------------

/// The example of the issue: stock share before and after retirement, group of fixed rate and endowment, objective
/// mean real consumption in the first 15 years of retirement, ruin at most 2 %.
pub fn example_experiment(paths: usize, confirm_paths: usize) -> Value {
    json!({
        "vary": [
            { "path": "wealth.stockShare@start", "values": [0.4, 0.6, 0.8] },
            { "path": "wealth.stockShare@retire", "values": [0.3, 0.5] },
            { "group": [
                { "set": { "withdrawal.strategy": "fixed_rate" },
                  "vary": [ { "path": "withdrawal.fixedRate.rate", "values": [0.03, 0.04, 0.05] } ] },
                { "set": { "withdrawal.strategy": "endowment" },
                  "vary": [ { "path": "withdrawal.endowment.rate", "values": [0.03, 0.04] } ] }
            ] }
        ],
        "metrics": [ ruin(), cons15() ],
        "constraints": [ { "metric": "ruin", "op": "le", "value": 0.02 } ],
        "objective": { "maximize": "cons15" },
        "budget": { "paths": paths, "confirmPaths": confirm_paths },
        "confirm": true
    })
}

/// #216/AK10: 30 variants, a ranking, the best variant confirmed.
#[test]
fn issue_216_ak10_example_with_30_variants() {
    let p = plan(100);
    let r = done(&p, &example_experiment(300, 1000));
    assert_eq!(r.variants, 30);
    assert_eq!(r.ranking.len(), 5);
    let best = &r.ranking[0];
    assert!(setting(best, "wealth.stockShare@start").is_some());
    assert!(setting(best, "wealth.stockShare@retire").is_some());
    let c = best.confirm.as_ref().expect("confirmed");
    assert!(c.feasible, "{c:?}");
    assert!(figure(c, "ruin") <= 0.02);
    assert!(figure(c, "cons15") > 0.0);
    assert!(r.paths.independent);
    // AK9: the answer stays small.
    let text = serde_json::to_string(&engine::experiment::EvaluateOut {
        status: Status::Done,
        errors: vec![],
        estimate: None,
        result: Some(r),
    })
    .unwrap();
    assert!(text.len() < 20_000, "{}", text.len());
}
