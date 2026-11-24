//! #216: Command `evaluate`: an experiment (changes, variations, constraints, objective) on a plan in one call.

use std::path::Path;

use engine::experiment::{self, EvaluateOut, Evaluation, Ranked, Status};

use crate::{chf, json_out, library_dir, positionals, print_json, JsonOut, R};

/// A number: amounts with thousands separator, shares and small numbers with four decimals.
fn number(v: f64) -> String {
    if v.abs() >= 1000.0 {
        chf(v)
    } else {
        format!("{v:.4}")
    }
}

fn evaluation_text(e: &Evaluation) -> String {
    let mut parts = vec![];
    if let Some(o) = e.objective {
        parts.push(format!("objective {}", number(o)));
    }
    parts.push(
        if e.feasible {
            "constraints met"
        } else {
            "constraints NOT met"
        }
        .to_string(),
    );
    for f in &e.figures {
        let se = f.std_error.map(|s| format!(" ± {}", number(s))).unwrap_or_default();
        parts.push(format!("{} {}{se}", f.name, f.value.map_or("–".into(), number)));
    }
    if let Some(m) = &e.optimization {
        parts.push(format!(
            "ruin {:.2} %, median {}, P10 {}",
            m.ruin * 100.0,
            chf(m.median),
            chf(m.p10)
        ));
    }
    parts.join(", ")
}

fn ranked_text(r: &Ranked) -> String {
    let values: Vec<String> = r.settings.iter().map(|s| format!("{}={}", s.path, s.value)).collect();
    let mut out = format!(
        "{:>3}. {}\n       search:    {}",
        r.rank,
        values.join(", "),
        evaluation_text(&r.search)
    );
    if let Some(c) = &r.confirm {
        out.push_str(&format!("\n       confirmed: {}", evaluation_text(c)));
    }
    out
}

fn print_text(out: &EvaluateOut) {
    for e in &out.errors {
        println!("{e}");
    }
    if let Some(e) = &out.estimate {
        let time = e.seconds.map(|s| format!(", about {s:.1} s")).unwrap_or_default();
        println!("Estimate: {} variants, {} runs{time}", e.variants, chf(e.runs as f64));
    }
    let Some(r) = &out.result else {
        return;
    };
    let seed = |s: Option<u64>| s.map(|s| format!(", seed {s}")).unwrap_or_default();
    println!(
        "Experiment: {} variants ({:?}), search on {} paths{}{}",
        r.variants,
        r.method,
        r.paths.search,
        seed(r.paths.search_seed),
        if r.ranking.is_empty() {
            String::new()
        } else {
            format!(
                ", confirmation on {} {} paths{}",
                r.paths.confirm,
                if r.paths.independent { "independent" } else { "the same" },
                seed(r.paths.confirm_seed)
            )
        }
    );
    for x in r.ranking.iter().chain(&r.listed) {
        println!("{}", ranked_text(x));
    }
    if r.omitted > 0 {
        println!("… {} more variants not listed", r.omitted);
    }
    for h in &r.hints {
        println!("Hint: {h}");
    }
    println!(
        "Engine {}, {:.0} ms. Model calculation without guarantee.",
        r.version, r.elapsed_ms
    );
}

/// `vivaplan evaluate <plan.vplan> <experiment.json> [--json] [--yes]`.
pub(crate) fn cmd_evaluate(args: &[String]) -> R {
    let list = positionals(args);
    let [plan_path, exp_path] = list.as_slice() else {
        return Err("Project file and experiment expected".into());
    };
    let plan = engine::api::read_plan(Path::new(plan_path.as_str()))?;
    let text = std::fs::read_to_string(exp_path.as_str()).map_err(|e| format!("{exp_path}: {e}"))?;
    let mut exp: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{exp_path}: {e}"))?;
    if args.iter().any(|a| a == "--yes") {
        if let Some(o) = exp.as_object_mut() {
            o.insert("confirm".into(), serde_json::Value::Bool(true));
        }
    }
    let base = Path::new(plan_path.as_str()).parent().unwrap_or(Path::new(""));
    let library = library_dir(args);
    let out = experiment::evaluate(&plan, &exp, base, Some(&library));
    if json_out(args) == JsonOut::Stdout {
        print_json(&out)?;
    } else {
        print_text(&out);
    }
    match out.status {
        Status::Done => Ok(()),
        Status::Invalid => Err("The experiment contains errors; nothing was computed.".into()),
        Status::NeedsConfirmation => Err("Not computed: run again with --yes.".into()),
        Status::Cancelled => Err("Cancelled.".into()),
        Status::Failed => Err("The computation failed.".into()),
    }
}
