//! #219: Catalog for AIs (`describe`): fields, forms, state variables, metric building blocks, examples, measured
//! microseconds per run, limits. Fictitious plans only.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use engine::describe::{self, Catalog, Section, MEANINGS, UNITS, UNIT_LEGEND};
use engine::experiment::{self, Status};
use serde_json::{json, Value};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn catalog(section: Option<Section>, plan: Option<&Value>) -> Catalog {
    engine::api::describe(section, plan, &library(), Some(&library()))
}

/// The whole catalog, measured once for all tests. #226: measured on a short given plan (three years); the
/// measurement on the example plan has its own test (`issue_219_ak1_measures_microseconds_per_run`).
fn whole() -> &'static Catalog {
    static WHOLE: std::sync::OnceLock<Catalog> = std::sync::OnceLock::new();
    WHOLE.get_or_init(|| {
        let short = crate::plan_builder::PlanBuilder::new("Describe")
            .dates("2030-01", "2031-01", "2032-12")
            .build();
        catalog(None, Some(&short))
    })
}

/// Paths of the field list of the catalog.
fn catalog_paths(c: &Catalog) -> BTreeSet<String> {
    let f = c.fields.as_ref().expect("fields");
    f.groups
        .iter()
        .flat_map(|(g, rows)| {
            rows.iter().map(move |r| {
                let name = r.split_once(':').unwrap().0;
                if g.is_empty() {
                    name.to_string()
                } else {
                    format!("{g}.{name}")
                }
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// AK1: content
// ---------------------------------------------------------------------------------------------

/// #219/AK1: Fields with type, unit, range and meaning; state variables, metric building blocks, forms of the space of
/// variations and of the objective, three examples and the measured microseconds per run.
#[test]
fn issue_219_ak1_catalog_has_all_parts() {
    let c = whole();
    assert!(c.performance.as_ref().is_some_and(|p| p.us_per_run.is_some()));
    assert_eq!(c.format, "vivaplan-describe/1");
    assert_eq!(c.sections.len(), 8);

    // Fields: the meaning only in the section (the whole catalog stays small).
    let f = c.fields.as_ref().unwrap();
    let wealth = &f.groups["wealth"];
    assert!(
        wealth.contains(&"stockShare: series|curve, share, 0..1".to_string()),
        "{wealth:?}"
    );
    let full = catalog(Some(Section::Fields), None);
    let rows = &full.fields.as_ref().unwrap().groups["withdrawal.guardrails"];
    let upper = rows.iter().find(|r| r.starts_with("upper:")).unwrap();
    assert!(
        upper.starts_with("upper: number, share, 0..1 – Upper guardrail"),
        "{upper}"
    );
    let strategy = full.fields.as_ref().unwrap().groups["withdrawal"]
        .iter()
        .find(|r| r.starts_with("strategy:"))
        .unwrap()
        .clone();
    assert!(
        strategy.contains("guardrails") && strategy.contains("fixed_rate"),
        "{strategy}"
    );
    assert!(f.units.iter().any(|u| u.name == "share"));

    // Forms of a course over time, points in time.
    let s = c.series.as_ref().unwrap();
    let forms: Vec<&str> = s.forms.iter().map(|i| i.name.as_str()).collect();
    for m in ["const", "timed", "linear", "table", "factor", "curve"] {
        assert!(forms.contains(&m), "{forms:?}");
    }
    assert!(s.points_in_time.iter().any(|p| p.name == "age:<years>"));

    // State variables (#218) with their meaning.
    let state: Vec<&str> = c.state.as_ref().unwrap().iter().map(|i| i.name.as_str()).collect();
    assert_eq!(
        state,
        [
            "age",
            "liquidReal",
            "withdrawalRate",
            "fundedRatio",
            "stockReturn12m",
            "drawdown"
        ]
    );
    assert!(c.state.as_ref().unwrap().iter().all(|i| !i.meaning.is_empty()));

    // Metric building blocks (#214): every quantity, reduction and aggregation of the schema of a metric.
    let m = c.metrics.as_ref().unwrap();
    let names = |l: &[describe::Item]| l.iter().map(|i| i.name.clone()).collect::<BTreeSet<_>>();
    let quantities = names(&m.quantities);
    for q in ["consumption", "depleted", "atFloor", "total", "withdrawal", "liquid"] {
        assert!(quantities.contains(q), "{q}");
    }
    assert_eq!(quantities.len(), 38);
    assert!(names(&m.reduce).contains("annualSums") && names(&m.reduce).contains("shareOfStartLiquid"));
    assert_eq!(
        names(&m.aggregate),
        [
            "conditionalMean",
            "count",
            "max",
            "mean",
            "min",
            "probability",
            "quantile"
        ]
        .map(String::from)
        .into()
    );
    assert!(names(&m.window).contains("fromAge"));
    let metric: engine::measure::Metric = serde_json::from_value(m.shape.clone()).expect("shape is a metric");
    assert_eq!(metric.name.as_deref(), Some("cons15"));

    // Forms of the experiment (#216): space of variations, constraint, objective, search, defaults.
    let e = c.experiment.as_ref().unwrap();
    assert_eq!(
        names(&e.objective),
        ["maximize", "minimize", "score", "weighted"].map(String::from).into()
    );
    assert_eq!(names(&e.methods), ["grid", "pattern"].map(String::from).into());
    assert_eq!(e.vary.len(), 4);
    assert!(names(&e.keys).contains("vary") && names(&e.keys).contains("budget"));
    assert_eq!(e.defaults["budget"]["paths"], 1000);
    assert_eq!(e.max_variants, experiment::MAX_VARIANTS);

    // Three examples, the limits.
    assert_eq!(c.examples.as_ref().unwrap().len(), 3);
    let limits = c.limits.as_ref().unwrap();
    assert!(limits.iter().any(|l| l.contains("drawdown")) && limits.iter().any(|l| l.contains("selection")));
}

/// #219/AK1: The microseconds per run are measured at the call, on the example plan (with Monte Carlo as in the
/// examples) or on the given plan; the test checks the form and plausibility, not the value.
// slow: measures the run time on the example plan (4000 runs) and on two given plans
#[test]
fn issue_219_ak1_measures_microseconds_per_run() {
    let c = catalog(Some(Section::Performance), None);
    let p = c.performance.expect("performance");
    assert_eq!(p.plan, "example");
    let us = p.us_per_run.expect("measured");
    assert!(us.is_finite() && us > 0.0 && us < 1e6, "{us}");
    let generate = p.us_generate.unwrap();
    assert!(generate > 0.0 && generate <= us * 10.0, "{generate} {us}");
    assert_eq!(p.months, Some(472));
    assert!(p.cores >= 1 && p.measured_ms > 0.0 && p.error.is_none());

    let mut plan = engine::api::example_plan();
    plan["returns"]["parametric"]["paths"] = json!(50);
    plan["returns"]["source"] = json!("parametric");
    let p = catalog(Some(Section::Performance), Some(&plan)).performance.unwrap();
    assert_eq!(p.plan, "given");
    assert!(p.us_per_run.is_some_and(|u| u > 0.0));

    // A plan that cannot be computed: no value, the message instead.
    let mut bad = plan.clone();
    bad["wealth"]["stocks"] = json!("many");
    let p = catalog(Some(Section::Performance), Some(&bad)).performance.unwrap();
    assert!(p.us_per_run.is_none() && p.error.is_some(), "{p:?}");
}

/// #219/AK1: A section alone holds only itself; the performance is measured only when it is part of the answer.
#[test]
fn issue_219_ak1_single_section() {
    let c = catalog(Some(Section::Fields), None);
    assert!(c.fields.is_some());
    assert!(c.series.is_none() && c.state.is_none() && c.metrics.is_none() && c.experiment.is_none());
    assert!(c.examples.is_none() && c.performance.is_none() && c.limits.is_none());
    // (the performance alone: issue_219_ak1_measures_microseconds_per_run)
    for s in Section::ALL.into_iter().filter(|s| *s != Section::Performance) {
        let v = serde_json::to_value(catalog(Some(s), None)).unwrap();
        let keys: Vec<&String> = v.as_object().unwrap().keys().collect();
        assert_eq!(keys.len(), 4, "{keys:?}");
        assert!(v.get(s.name()).is_some(), "{}", s.name());
    }
}

// ---------------------------------------------------------------------------------------------
// AK2: generated from schema and code, checked in both directions
// ---------------------------------------------------------------------------------------------

/// #219/AK2: Every field of the schema is in the catalog, every field of the catalog is in the schema; the tables of
/// units and meanings name only fields of the schema and cover every number, integer and course over time.
#[test]
fn issue_219_ak2_catalog_and_schema_match() {
    let leaves = describe::schema_leaves();
    let schema: BTreeSet<String> = leaves.iter().map(|l| l.path.clone()).collect();
    assert!(schema.len() > 150, "{}", schema.len());
    assert_eq!(catalog_paths(&catalog(Some(Section::Fields), None)), schema);

    // Table of units: only fields of the schema, every number, integer and course over time, known units.
    let legend: BTreeSet<&str> = UNIT_LEGEND.iter().map(|u| u.0).collect();
    let mut seen = BTreeSet::new();
    for (path, unit, _) in UNITS {
        assert!(schema.contains(*path), "UNITS: {path} is not a field of the schema");
        assert!(seen.insert(*path), "UNITS: {path} twice");
        assert!(
            unit.split(" or ").all(|u| legend.contains(u)),
            "UNITS: unknown unit {unit} of {path}"
        );
    }
    for l in &leaves {
        if ["number", "integer", "series", "series|curve"].contains(&l.kind.as_str()) {
            assert!(
                seen.contains(l.path.as_str()),
                "UNITS: {} ({}) has no unit",
                l.path,
                l.kind
            );
        }
    }

    // Meanings: only fields of the schema; every field has one (from the schema or from the table).
    let mut seen = BTreeSet::new();
    for (path, text) in MEANINGS {
        assert!(schema.contains(*path), "MEANINGS: {path} is not a field of the schema");
        assert!(seen.insert(*path), "MEANINGS: {path} twice");
        assert!(
            !text.is_empty() && text.len() <= describe::MEANING_CHARS + 8,
            "{path}: {text}"
        );
    }
    for f in describe::fields() {
        assert!(!f.meaning.is_empty(), "{} has no meaning", f.path);
        assert!(f.kind != "any", "{} has no type", f.path);
    }
}

/// Field paths of the values in a plan (`*` for a list index); below a field of type series, array or object of the
/// catalog the path stops there.
fn value_paths(v: &Value, path: &str, stop: &BTreeSet<String>, out: &mut BTreeSet<String>) {
    if stop.contains(path) {
        out.insert(path.to_string());
        return;
    }
    let join = |k: &str| {
        if path.is_empty() {
            k.to_string()
        } else {
            format!("{path}.{k}")
        }
    };
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                value_paths(x, &join(k), stop, out);
            }
        }
        Value::Array(a) => {
            for x in a {
                value_paths(x, &join("*"), stop, out);
            }
        }
        Value::Null => {}
        _ => {
            out.insert(path.to_string());
        }
    }
}

/// #219/AK2: Independent of the walk over the schema: every value of real plans (example plan, every strategy,
/// a plan built in the tests) lies on a field of the catalog.
#[test]
fn issue_219_ak2_values_of_plans_lie_on_catalog_fields() {
    let fields = describe::fields();
    let all: BTreeSet<String> = fields.iter().map(|f| f.path.clone()).collect();
    let stop: BTreeSet<String> = fields
        .iter()
        .filter(|f| ["series", "series|curve", "array", "object"].contains(&f.kind.as_str()))
        .map(|f| f.path.clone())
        .collect();
    let mut plans = vec![
        engine::api::example_plan(),
        crate::plan_builder::PlanBuilder::new("Describe").build(),
    ];
    plans[0]["wealth"]["stockShare"] =
        json!({ "mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]] });
    let mut found = BTreeSet::new();
    for p in &plans {
        let mut v = p.clone();
        // Settings (#25) travel with the plan but are not part of its schema.
        v.as_object_mut().unwrap().remove("limits");
        value_paths(&v, "", &stop, &mut found);
    }
    let missing: Vec<&String> = found.iter().filter(|p| !all.contains(*p)).collect();
    assert!(missing.is_empty(), "values outside the catalog: {missing:?}");
    assert!(found.len() > 100, "{}", found.len());
}

// ---------------------------------------------------------------------------------------------
// AK3: the examples run
// ---------------------------------------------------------------------------------------------

/// #219/AK3: An example experiment of the catalog runs on the example plan without errors and in less than the time
/// that needs a confirmation.
fn example_runs(name: &str) {
    let plan = engine::api::example_plan();
    let e = describe::examples().into_iter().find(|e| e.name == name).expect(name);
    {
        let out = experiment::evaluate(&plan, &e.experiment, &library(), Some(&library()));
        assert_eq!(out.status, Status::Done, "{}: {:?}", e.name, out.errors);
        assert!(out.errors.is_empty(), "{}: {:?}", e.name, out.errors);
        let r = out.result.expect("result");
        let has_objective = e.experiment.get("objective").is_some();
        if has_objective {
            assert!(!r.ranking.is_empty(), "{}", e.name);
            assert!(r.ranking[0].confirm.is_some(), "{}", e.name);
        } else {
            assert_eq!(r.listed.len(), r.variants, "{}", e.name);
        }
        assert!(
            r.ranking
                .iter()
                .chain(&r.listed)
                .all(|x| x.search.figures.iter().all(|f| f.value.is_some())),
            "{}",
            e.name
        );
        let seconds = out.estimate.and_then(|x| x.seconds).unwrap_or(0.0);
        assert!(seconds < experiment::CONFIRM_SECONDS, "{}: {seconds} s", e.name);
    }
}

/// #219/AK3: Example «spending» (objective, constraint, range).
// slow: runs the shipped example experiment on the example plan with its own budget, time measurement included
#[test]
fn issue_219_ak3_example_spending_runs() {
    example_runs("spending");
}

/// #219/AK3: Example «lumpSum» (no objective: all variants listed).
// slow: runs the shipped example experiment on the example plan with its own budget, time measurement included
#[test]
fn issue_219_ak3_example_lump_sum_runs() {
    example_runs("lumpSum");
}

/// #219/AK3: Example «strategy» (points in time, coupled value, group of strategies).
// slow: runs the shipped example experiment on the example plan with its own budget, time measurement included
#[test]
fn issue_219_ak3_example_strategy_runs() {
    example_runs("strategy");
}

/// #219/AK3: There are exactly three examples, each with its own name.
#[test]
fn issue_219_ak3_three_examples() {
    let names: BTreeSet<&str> = describe::examples().iter().map(|e| e.name).collect();
    assert_eq!(names, ["lumpSum", "spending", "strategy"].into());
}

// ---------------------------------------------------------------------------------------------
// AK4: small
// ---------------------------------------------------------------------------------------------

/// #219/AK4: The catalog as JSON stays under 20 000 characters, the whole one and every section.
#[test]
fn issue_219_ak4_json_under_20000_characters() {
    let all = serde_json::to_string(whole()).unwrap();
    assert!(all.chars().count() < 20_000, "{}", all.chars().count());
    for s in Section::ALL {
        let one = match s {
            Section::Performance => {
                let mut c = whole().clone();
                c.fields = None;
                c.series = None;
                c.state = None;
                c.metrics = None;
                c.experiment = None;
                c.examples = None;
                c.limits = None;
                serde_json::to_string(&c).unwrap()
            }
            _ => serde_json::to_string(&catalog(Some(s), None)).unwrap(),
        };
        assert!(one.chars().count() < 20_000, "{}: {}", s.name(), one.chars().count());
    }
}
