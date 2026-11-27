//! #219: Command `describe`: catalog for AIs (what can be changed, varied and measured), as text or JSON.

use std::fmt::Write;
use std::path::Path;

use engine::describe::{Catalog, Item, Section};

use crate::{app_data, json_out, opt, positionals, print_json, JsonOut, R};

fn items(out: &mut String, title: &str, list: &[Item]) {
    let _ = writeln!(out, "{title}:");
    for i in list {
        if i.meaning.is_empty() {
            let _ = writeln!(out, "  {}", i.name);
        } else {
            let _ = writeln!(out, "  {}: {}", i.name, i.meaning);
        }
    }
}

/// The catalog as compact text (one line per field or building block).
pub(crate) fn text(c: &Catalog) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "VivaPlan {} catalog for AIs ({}); sections: {} (vivaplan describe <section>)",
        c.engine,
        c.format,
        c.sections.join(", ")
    );
    if let Some(f) = &c.fields {
        let _ = writeln!(out, "\n## fields");
        for n in &f.notes {
            let _ = writeln!(out, "{n}");
        }
        let units: Vec<String> = f.units.iter().map(|u| format!("{} = {}", u.name, u.meaning)).collect();
        let _ = writeln!(out, "Units: {}", units.join("; "));
        let _ = writeln!(out, "Rows: {}", f.row);
        for (group, rows) in &f.groups {
            let _ = writeln!(out, "{}", if group.is_empty() { "(top level)" } else { group });
            for row in rows {
                let _ = writeln!(out, "  {row}");
            }
        }
    }
    if let Some(s) = &c.series {
        let _ = writeln!(out, "\n## series");
        items(&mut out, "Forms (mode)", &s.forms);
        items(&mut out, "Points in time", &s.points_in_time);
    }
    if let Some(s) = &c.state {
        let _ = writeln!(out, "\n## state");
        items(&mut out, "State variables (input of a curve)", s);
    }
    if let Some(m) = &c.metrics {
        let _ = writeln!(out, "\n## metrics");
        let _ = writeln!(out, "Shape: {}", m.shape);
        let names: Vec<&str> = m.quantities.iter().map(|q| q.name.as_str()).collect();
        let _ = writeln!(out, "quantity: {}", names.join(", "));
        let described: Vec<Item> = m.quantities.iter().filter(|q| !q.meaning.is_empty()).cloned().collect();
        items(&mut out, "Quantities with a meaning", &described);
        items(&mut out, "window", &m.window);
        items(&mut out, "reduce", &m.reduce);
        items(&mut out, "aggregate (kind)", &m.aggregate);
        items(&mut out, "op", &m.ops);
        for n in &m.notes {
            let _ = writeln!(out, "{n}");
        }
    }
    if let Some(e) = &c.experiment {
        let _ = writeln!(out, "\n## experiment");
        items(&mut out, "Keys", &e.keys);
        items(&mut out, "vary", &e.vary);
        items(&mut out, "objective (exactly one)", &e.objective);
        let _ = writeln!(out, "constraint: {}", e.constraint);
        items(&mut out, "search.method", &e.methods);
        let _ = writeln!(out, "Defaults: {}", e.defaults);
        let _ = writeln!(
            out,
            "At most {} variants; from an estimated {} s only with \"confirm\": true (vivaplan evaluate --yes).",
            e.max_variants, e.confirm_seconds
        );
    }
    if let Some(x) = &c.examples {
        let _ = writeln!(out, "\n## examples (vivaplan evaluate <plan.vplan> <experiment.json>)");
        for e in x {
            let _ = writeln!(out, "{}: {}\n{}", e.name, e.question, e.experiment);
        }
    }
    if let Some(p) = &c.performance {
        let _ = writeln!(out, "\n## performance");
        match (p.us_per_run, &p.error) {
            (Some(us), _) => {
                let _ = writeln!(
                    out,
                    "Measured on the {} plan in {:.0} ms: {:.1} µs per run (of which {:.1} µs generating the path), \
                     {} cores, {} months.",
                    p.plan,
                    p.measured_ms,
                    us,
                    p.us_generate.unwrap_or(0.0),
                    p.cores,
                    p.months.unwrap_or(0)
                );
            }
            (None, Some(e)) => {
                let _ = writeln!(out, "Not measured on the {} plan: {e}", p.plan);
            }
            (None, None) => {}
        }
        let _ = writeln!(out, "Estimate: {}", p.estimate);
    }
    if let Some(ch) = &c.choices {
        let _ = writeln!(out, "\n## choices (labels in the language {})", ch.language);
        for n in &ch.notes {
            let _ = writeln!(out, "{n}");
        }
        for (path, values) in &ch.fields {
            items(&mut out, path, values);
        }
    }
    if let Some(l) = &c.limits {
        let _ = writeln!(out, "\n## limits");
        for x in l {
            let _ = writeln!(out, "- {x}");
        }
    }
    out
}

/// `vivaplan describe [<section>] [--plan <plan.vplan>] [--json]`.
pub(crate) fn cmd_describe(args: &[String]) -> R {
    let section = match positionals(args).as_slice() {
        [] => None,
        [name] => Some(Section::parse(name).ok_or_else(|| {
            let names: Vec<&str> = Section::ALL.iter().map(|s| s.name()).collect();
            format!("Unknown section {name}; sections: {}", names.join(", "))
        })?),
        _ => return Err("At most one section expected".into()),
    };
    let plan_path = opt(args, &["--plan"]);
    let data = app_data(args)?;
    let plan = match &plan_path {
        Some(p) => Some(data.read_plan(Path::new(p))?),
        None => None,
    };
    let base = plan_path
        .as_deref()
        .and_then(|p| Path::new(p).parent())
        .unwrap_or(Path::new(""))
        .to_path_buf();
    let catalog = engine::api::describe(section, plan.as_ref(), &base, data.library(), data.language());
    if json_out(args) == JsonOut::Stdout {
        print_json(&catalog)
    } else {
        print!("{}", text(&catalog));
        Ok(())
    }
}
