//! #211: Computing tools of the MCP server: `simulation_run` (the simulation of the plan as in the app) and `evaluate`
//! (an experiment, #216: changes, variations, metrics, constraints, objective; optimization and comparison run through
//! it). Both estimate the duration first and start from 60 s only with `"confirm": true`. The computation runs in a
//! thread of its own with an id of `engine::progress`, so that the server reports its progress and can cancel it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};

use super::session::{dispatch, Session};
use super::tools::{self, arg, Answer, Outcome};

/// Names of the computing tools.
pub(crate) const NAMES: &[&str] = &["simulation_run", "evaluate"];

/// Ids of the computations of the server; large, so that they do not meet the ids the engine assigns itself.
static NEXT_JOB: AtomicU64 = AtomicU64::new(1 << 40);

/// A new id for a computation.
pub(crate) fn next_job() -> u64 {
    NEXT_JOB.fetch_add(1, Ordering::Relaxed)
}

/// Hints of the protocol: the computing tools change no file and reach nothing outside the computer; every call keeps a
/// new result in the session.
fn computing(mut tool: Value) -> Value {
    tool["annotations"]["idempotentHint"] = json!(false);
    tool
}

/// The computing tools of `tools/list`.
pub(crate) fn list() -> Vec<Value> {
    let confirm = json!({
        "type": "boolean",
        "description": "Compute even if the estimate is 60 s or more. Set it only after the user agreed to the duration.",
    });
    vec![
        computing(tools::tool(
            "simulation_run",
            "Simulate a plan",
            "Simulates an open plan as the desktop app does: according to its source of returns (fixed, historical, rolling, \
             bootstrap, parametric) and with the variations stored in the plan. Returns the id of the result (r1, …), the \
             key figures, the result in words and the estimate of the duration. First the duration is estimated; from 60 s \
             nothing is computed without confirm: true (the answer then gives the estimate). Long computations report \
             progress and can be cancelled. Details afterwards with result_get (yearly, events, bands, variations). To \
             compare changes or search for better values use evaluate.",
            json!({
                "plan": { "type": "string", "description": "Id of the open plan (p1, …)." },
                "paths": { "type": "integer", "description": "Number of paths for this run only (sources bootstrap and parametric)." },
                "confirm": confirm,
            }),
            &["plan"],
        )),
        computing(tools::tool(
            "evaluate",
            "Experiment on a plan",
            "Computes an experiment on an open plan in one call: changes by field path (set), variations as a grid or a \
             pattern search (vary), metrics, constraints and an objective; the ranking of the variants comes with values \
             on independent confirmation paths. This is the way to optimize and to compare variants. The format of the \
             experiment: describe with section experiment. The plan itself stays unchanged. First the duration is \
             estimated; from 60 s nothing is computed without confirm: true (status needsConfirmation with the estimate). \
             Long computations report progress and can be cancelled.",
            json!({
                "plan": { "type": "string", "description": "Id of the open plan (p1, …)." },
                "experiment": { "type": "object", "description": "The experiment (set, vary, metrics, constraints, objective, search, budget)." },
                "confirm": confirm,
            }),
            &["plan", "experiment"],
        )),
    ]
}

/// What the thread computes.
enum Task {
    /// Simulation of the plan as the app does it (`run_simulation`).
    Run,
    /// The experiment (with `confirm` inserted).
    Evaluate(Value),
}

/// A computation ready to start: everything it needs, taken from the session.
pub(crate) struct Prepared {
    plan_id: String,
    /// The plan with the settings of the app, as it computes with it.
    plan: Value,
    file: Option<PathBuf>,
    library: Option<PathBuf>,
    confirm: bool,
    task: Task,
}

/// Checks the arguments of a computing tool and takes the plan out of the session; an error is a tool error.
pub(crate) fn prepare(s: &Session, name: &str, args: &Value) -> Result<Prepared, String> {
    let open = s.plan(arg(args, "plan").unwrap_or_default())?;
    let mut plan = s.with_settings(&open.plan);
    let confirm = args.get("confirm").and_then(Value::as_bool).unwrap_or(false);
    let task = match name {
        "simulation_run" => {
            if let Some(n) = args.get("paths").and_then(Value::as_i64) {
                let source = plan["returns"]["source"].as_str().unwrap_or_default().to_string();
                if source != "bootstrap" && source != "parametric" {
                    return Err(format!(
                        "paths applies only to the sources bootstrap and parametric; the plan uses {source}"
                    ));
                }
                if n < 1 {
                    return Err("paths must be at least 1".into());
                }
                plan["returns"][source.as_str()]["paths"] = json!(n);
            }
            Task::Run
        }
        "evaluate" => {
            let mut exp = args["experiment"].clone();
            if confirm {
                exp["confirm"] = json!(true);
            }
            Task::Evaluate(exp)
        }
        _ => unreachable!("only computing tools are prepared"),
    };
    Ok(Prepared {
        plan_id: open.id.clone(),
        plan,
        file: open.file.clone(),
        library: s.data.library.clone(),
        confirm,
        task,
    })
}

/// Estimate of a simulation as the app makes it (#21): measured microseconds per run times the runs (paths times the
/// combinations of the variations). `None` if the plan cannot be measured; the simulation then reports why.
fn estimate_run(p: &Prepared) -> Option<Value> {
    let lib = p.library.as_deref();
    let bench = dispatch(lib, "bench_plan", &json!({ "plan": p.plan }), p.file.as_deref()).ok()?;
    let source = p.plan["returns"]["source"].as_str().unwrap_or_default();
    let paths = bench["fixedPaths"]
        .as_u64()
        .or_else(|| p.plan["returns"][source]["paths"].as_u64())
        .unwrap_or(1);
    let sweeps = p.plan["scenarios"]["sweeps"].as_array().cloned().unwrap_or_default();
    let combinations = if sweeps.is_empty() {
        1
    } else {
        engine::scenarios::parse(&sweeps)
            .and_then(|s| engine::scenarios::count_combinations(&s))
            .ok()? as u64
    };
    let runs = paths.saturating_mul(combinations);
    let seconds = runs as f64 * bench["usPerRun"].as_f64()? / 1e6;
    Some(json!({ "runs": runs, "seconds": (seconds * 10.0).round() / 10.0 }))
}

/// Computes in the thread of the computation with the id `job`; the answer of the engine.
pub(crate) fn compute(p: &Prepared, job: u64) -> Value {
    let lib = p.library.as_deref();
    match &p.task {
        Task::Run => {
            let estimate = estimate_run(p);
            let seconds = estimate.as_ref().and_then(|e| e["seconds"].as_f64()).unwrap_or(0.0);
            if !p.confirm && seconds >= engine::experiment::CONFIRM_SECONDS {
                return json!({ "estimate": estimate, "needsConfirmation": true });
            }
            let run = dispatch(
                lib,
                "run_simulation",
                &json!({ "plan": p.plan, "jobId": job }),
                p.file.as_deref(),
            )
            .unwrap_or_else(|e| json!({ "error": { "text": e } }));
            json!({ "estimate": estimate, "run": run })
        }
        Task::Evaluate(exp) => {
            let base = p
                .file
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_default();
            let out = engine::experiment::evaluate_as(Some(job), &p.plan, exp, &base, lib);
            serde_json::to_value(out).unwrap_or_else(|e| json!({ "status": "failed", "errors": [e.to_string()] }))
        }
    }
}

/// Texts of a list of messages of the engine (`{code, params, text}`).
fn texts(v: &Value) -> Vec<Value> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|m| m.get("text").cloned().unwrap_or_else(|| m.clone()))
        .collect()
}

/// Answer of a computation that needs the confirmation of the user.
fn needs_confirmation(estimate: &Value, message: Value) -> Outcome {
    Ok(Answer::data(json!({
        "status": "needsConfirmation",
        "estimate": estimate,
        "message": message,
        "next": "Tell the user the estimated duration; only if they agree call the tool again with confirm: true.",
    })))
}

/// Shapes the answer of the engine into the answer of the tool and keeps a result in the session.
pub(crate) fn finish(s: &mut Session, p: Prepared, out: Value) -> Outcome {
    match p.task {
        Task::Run => {
            if out["needsConfirmation"] == true {
                let seconds = out["estimate"]["seconds"].as_f64().unwrap_or_default();
                let runs = out["estimate"]["runs"].as_u64().unwrap_or_default();
                return needs_confirmation(
                    &out["estimate"],
                    json!(format!(
                        "The computation takes about {seconds:.0} s ({runs} runs). Confirm with confirm: true."
                    )),
                );
            }
            let run = &out["run"];
            if let Some(e) = run["error"].get("text").and_then(Value::as_str) {
                let errors: Vec<String> = run["issues"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|i| i["severity"] == "error")
                    .map(|i| {
                        let at = i["field"].as_str().or(i["section"].as_str()).unwrap_or("plan");
                        format!("{at}: {}", i["message"].as_str().unwrap_or_default())
                    })
                    .collect();
                return Err(if errors.is_empty() {
                    e.to_string()
                } else {
                    format!("{e}\n{}", errors.join("\n"))
                });
            }
            let id = s.add_result(&p.plan_id, "simulation_run", run.clone());
            let mut answer = tools::result_get(s, &json!({ "result": id }))?;
            if let Some(o) = answer.data.as_object_mut() {
                o.insert("estimate".into(), out["estimate"].clone());
                o.insert(
                    "next".into(),
                    json!("result_get with view yearly, events, bands or variations for details; evaluate to compare changes"),
                );
            }
            Ok(answer)
        }
        Task::Evaluate(exp) => match out["status"].as_str().unwrap_or_default() {
            "done" => {
                // #212: with the fixed values of the experiment, which plan_apply takes over with a variant.
                let mut kept = out.clone();
                kept["set"] = exp.get("set").cloned().unwrap_or_else(|| json!({}));
                let id = s.add_result(&p.plan_id, "evaluate", kept);
                let mut result = out["result"].clone();
                if let Some(o) = result.as_object_mut() {
                    o.insert("hints".into(), Value::Array(texts(&out["result"]["hints"])));
                }
                Ok(Answer::results(json!({
                    "result": id,
                    "plan": p.plan_id,
                    "status": "done",
                    "estimate": out["estimate"],
                    "experiment": result,
                })))
            }
            "needsConfirmation" => needs_confirmation(
                &out["estimate"],
                texts(&out["errors"]).into_iter().next().unwrap_or(Value::Null),
            ),
            _ => {
                let errors: Vec<String> = texts(&out["errors"])
                    .iter()
                    .map(|t| t.as_str().map_or_else(|| t.to_string(), str::to_string))
                    .collect();
                Err(if errors.is_empty() {
                    "The computation failed".into()
                } else {
                    errors.join("\n")
                })
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    fn session_with_example() -> Session {
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/data");
        let mut s = Session::new(
            crate::AppData {
                library: Some(lib),
                settings: Map::new(),
            },
            false,
        );
        let plan = s.call("example_plan", &json!({}), None).unwrap();
        s.add_plan(None, plan, None);
        s
    }

    /// #211/AK1: Wrong arguments of the computing tools are tool errors with a hint.
    #[test]
    fn issue_211_ak1_arguments_of_the_computing_tools() {
        let mut s = session_with_example();
        let err = |s: &Session, args: Value| prepare(s, "simulation_run", &args).err().unwrap();
        assert!(err(&s, json!({ "plan": "p1", "paths": 10 })).contains("uses fixed"));
        s.plans[0].plan["returns"]["source"] = json!("parametric");
        assert!(err(&s, json!({ "plan": "p1", "paths": 0 })).contains("at least 1"));
        let p = prepare(&s, "simulation_run", &json!({ "plan": "p1", "paths": 7 })).unwrap();
        assert_eq!(p.plan["returns"]["parametric"]["paths"], 7);
        let p = prepare(
            &s,
            "evaluate",
            &json!({ "plan": "p1", "experiment": {}, "confirm": true }),
        )
        .unwrap();
        assert!(matches!(&p.task, Task::Evaluate(e) if e["confirm"] == true));
        assert!(list().iter().all(|t| t["annotations"]["idempotentHint"] == false));
    }

    /// #211/AK1: A plan that cannot be computed: the error with the check messages of the fields, no result.
    #[test]
    fn issue_211_ak1_errors_of_the_plan_come_back_as_messages() {
        let mut s = session_with_example();
        let p = prepare(&s, "simulation_run", &json!({ "plan": "p1" })).unwrap();
        let out = json!({ "run": {
            "error": { "text": "The plan is incomplete" },
            "issues": [
                { "severity": "error", "section": "general", "field": "household.persons.0.birth", "message": "Birth missing" },
                { "severity": "warning", "section": "general", "message": "Only a warning" },
            ],
        } });
        let e = finish(&mut s, p, out).err().unwrap();
        assert_eq!(e, "The plan is incomplete\nhousehold.persons.0.birth: Birth missing");
        let p = prepare(&s, "evaluate", &json!({ "plan": "p1", "experiment": {} })).unwrap();
        let e = finish(
            &mut s,
            p,
            json!({ "status": "cancelled", "errors": [{ "text": "Cancelled" }] }),
        );
        assert_eq!(e.err().unwrap(), "Cancelled");
        let p = prepare(&s, "evaluate", &json!({ "plan": "p1", "experiment": {} })).unwrap();
        assert_eq!(
            finish(&mut s, p, json!({ "status": "failed" })).err().unwrap(),
            "The computation failed"
        );
        assert!(s.results.is_empty());
    }
}
