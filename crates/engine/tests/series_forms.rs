//! #217: the forms of a course «linear», «table» and «factor», translated into monthly values before the simulation.
//!
//! Made-up plan: one person born 1975-01, start 2030-01, end 2060-12, no returns and no taxes.

use engine::api;
use engine::model::{Plan, Series};
use engine::resolve::{Curve, Resolver};
use engine::sim::{SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Verlaufsformen" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1975-01" } ] },
        "milestones": [
            { "id": "start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "kind": "age", "person": "A", "years": 65 },
            { "id": "derisk", "name": "Umschichten", "kind": "date", "date": "2035-01" },
            { "id": "end", "kind": "date", "date": "2060-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 0, "spending": 4000 },
        "wealth": { "stocks": 500_000, "bonds": 500_000, "cash": 2_000_000, "stockShare": 0.5, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.05, "bonds": 0.01, "inflation": 0, "saron": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn plan(v: &Value) -> Plan {
    serde_json::from_value(v.clone()).expect("plan readable")
}

/// The monthly values of a course of the plan `v` (the series at `ptr`, a JSON pointer).
fn curve(v: &Value, ptr: &str) -> Curve {
    let p = plan(v);
    let s: Series = serde_json::from_value(v.pointer(ptr).expect("series present").clone()).expect("series readable");
    Curve::compile(&s, &Resolver::new(&p).expect("resolvable")).expect("compilable")
}

fn at(c: &Curve, ym: &str) -> f64 {
    c.at(ym.parse::<YearMonth>().unwrap())
}

fn with(f: impl FnOnce(&mut Value)) -> Value {
    let mut v = base();
    f(&mut v);
    v
}

fn run(v: &Value) -> SimResult {
    let s = Simulator::new(&plan(v)).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn errors(v: &Value) -> Vec<String> {
    api::validate(v)
        .into_iter()
        .filter(|i| i.severity == engine::validate::Severity::Error)
        .map(|i| i.message.to_string())
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

// #217/AK5: stock share linear from 0.9 at 55 to 0.5 at 65 gives 0.7 at 60.
#[test]
fn issue_217_ak5_stock_share_linear_from_55_to_65_is_0_7_at_60() {
    let v = with(|v| {
        v["wealth"]["stockShare"] =
            json!({ "mode": "linear", "start": { "age": 55, "value": 0.9 }, "end": { "age": 65, "value": 0.5 } })
    });
    let c = curve(&v, "/wealth/stockShare");
    // Born 1975-01: 55 in 2030-01, 60 in 2035-01, 65 in 2040-01.
    assert!(close(at(&c, "2035-01"), 0.7), "{}", at(&c, "2035-01"));
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    // The simulation computes with it: in 2035-01 the stock share of the invested wealth is 0.7 after rebalancing.
    let r = run(&v);
    let m = r.records.iter().find(|m| m.month == YearMonth::new(2035, 1)).unwrap();
    let share = m.stocks / (m.stocks + m.bonds);
    assert!((share - 0.7).abs() < 0.01, "share {share}");
}

// #217/AK1: before the start the start value, after the end the end value, in between month by month on the line;
// points in time as date, age or milestone.
#[test]
fn issue_217_ak1_linear_before_between_after_with_date_age_and_milestone() {
    for (start, end) in [
        (
            json!({ "at": "2030-01", "value": 1000 }),
            json!({ "at": "2035-01", "value": 4000 }),
        ),
        (
            json!({ "at": "start", "value": 1000 }),
            json!({ "at": "derisk", "value": 4000 }),
        ),
        (
            json!({ "age": 55, "value": 1000 }),
            json!({ "age": 60, "person": "A", "value": 4000 }),
        ),
        (
            json!({ "at": "start", "value": 1000 }),
            json!({ "age": 60, "value": 4000 }),
        ),
    ] {
        let v = with(|v| v["flows"]["income"] = json!({ "mode": "linear", "start": start, "end": end }));
        let c = curve(&v, "/flows/income");
        assert_eq!(at(&c, "1990-01"), 1000.0);
        assert_eq!(at(&c, "2030-01"), 1000.0);
        assert!(close(at(&c, "2030-02"), 1050.0), "{}", at(&c, "2030-02"));
        assert!(close(at(&c, "2032-07"), 2500.0));
        assert_eq!(at(&c, "2035-01"), 4000.0);
        assert_eq!(at(&c, "2099-12"), 4000.0);
        assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    }
    // Start and end in the same month: a step.
    let v = with(|v| {
        v["flows"]["income"] =
            json!({ "mode": "linear", "start": { "at": "derisk", "value": 1 }, "end": { "at": "2035-01", "value": 2 } })
    });
    let c = curve(&v, "/flows/income");
    assert_eq!((at(&c, "2034-12"), at(&c, "2035-01")), (1.0, 2.0));
}

// #217/AK1: a line over a very long time takes coarser steps, but keeps start and end.
#[test]
fn issue_217_ak1_linear_over_centuries_stays_small() {
    let v = with(
        |v| v["flows"]["income"] = json!({ "mode": "linear", "start": { "at": "1000-01", "value": 0 }, "end": { "at": "9000-01", "value": 96_000 } }),
    );
    let c = curve(&v, "/flows/income");
    assert!(c.values().count() <= 2 * 12 * 200 + 2);
    assert_eq!(at(&c, "0999-12"), 0.0);
    assert_eq!(at(&c, "9000-01"), 96_000.0);
    let mid = at(&c, "5000-01");
    assert!((mid - 48_000.0).abs() < 50.0, "{mid}");
}

// #217/AK1: invalid support points are errors of the check (and of the simulation), not a crash.
#[test]
fn issue_217_ak1_invalid_support_points_are_errors() {
    let cases = [
        (
            json!({ "at": "2035-01", "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "before its start",
        ),
        (
            json!({ "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "either a point in time",
        ),
        (
            json!({ "at": "2030-01", "age": 50, "value": 1 }),
            json!({ "at": "2031-01", "value": 2 }),
            "either a point in time",
        ),
        (
            json!({ "at": "never", "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "not a fixed point in time",
        ),
        (
            json!({ "at": "afterSale", "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "not a fixed point in time",
        ),
        (
            json!({ "at": "nowhere", "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "Unknown time reference",
        ),
        (
            json!({ "age": 200, "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "age 200 is not possible",
        ),
        (
            json!({ "age": -1, "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "is not possible",
        ),
        (
            json!({ "age": 50, "person": "X", "value": 1 }),
            json!({ "at": "2030-01", "value": 2 }),
            "unknown person",
        ),
    ];
    for (start, end, text) in cases {
        let v = with(|v| v["wealth"]["reserveMonths"] = json!({ "mode": "linear", "start": start, "end": end }));
        let e = errors(&v);
        assert!(e.iter().any(|x| x.contains(text)), "{text}: {e:?}");
        assert!(Simulator::new(&plan(&v)).is_err());
    }
}

// #217/AK2: support points by age or date, linear or as steps; the first value before, the last after.
#[test]
fn issue_217_ak2_table_linear_and_steps_by_age_and_date() {
    let points = json!([
        { "age": 60, "value": 3000 },
        { "at": "2030-01", "value": 5000 },
        { "age": 70, "value": 6000 }
    ]);
    // Out of order on purpose: 2030-01 (55) comes first after sorting.
    let v = with(|v| v["flows"]["spending"] = json!({ "mode": "table", "points": points }));
    let c = curve(&v, "/flows/spending");
    assert_eq!(at(&c, "2000-01"), 5000.0);
    assert_eq!(at(&c, "2030-01"), 5000.0);
    assert!(close(at(&c, "2032-07"), 4000.0));
    assert_eq!(at(&c, "2035-01"), 3000.0);
    assert!(close(at(&c, "2040-01"), 4500.0));
    assert_eq!(at(&c, "2045-01"), 6000.0);
    assert_eq!(at(&c, "2090-01"), 6000.0);
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));

    let v = with(|v| v["flows"]["spending"] = json!({ "mode": "table", "interpolation": "step", "points": points }));
    let c = curve(&v, "/flows/spending");
    assert_eq!(at(&c, "2032-07"), 5000.0);
    assert_eq!(at(&c, "2034-12"), 5000.0);
    assert_eq!(at(&c, "2035-01"), 3000.0);
    assert_eq!(at(&c, "2044-12"), 3000.0);
    assert_eq!(at(&c, "2045-01"), 6000.0);

    // Two points in the same month: the later entry applies from it on.
    let v = with(|v| {
        v["flows"]["spending"] = json!({ "mode": "table", "points": [
            { "at": "2030-01", "value": 1000 }, { "at": "derisk", "value": 2000 }, { "at": "2035-01", "value": 3000 }
        ] })
    });
    let c = curve(&v, "/flows/spending");
    assert_eq!(at(&c, "2035-01"), 3000.0);
    assert!(at(&c, "2034-12") < 2000.0);
}

// #217/AK2: a table without points is an error; a single point applies throughout.
#[test]
fn issue_217_ak2_table_empty_or_single_point() {
    let v = with(|v| v["flows"]["spending"] = json!({ "mode": "table", "points": [] }));
    assert!(errors(&v).iter().any(|x| x.contains("no entries")), "{:?}", errors(&v));
    let v = with(|v| v["flows"]["spending"] = json!({ "mode": "table", "points": [ { "age": 70, "value": 4500 } ] }));
    let c = curve(&v, "/flows/spending");
    assert_eq!((at(&c, "2000-01"), at(&c, "2099-01")), (4500.0, 4500.0));
}

// #217/AK3: base value times a course: spending 100 % up to 80, 80 % after it; computes like the same steps.
#[test]
fn issue_217_ak3_factor_spending_100_percent_up_to_80_then_80_percent() {
    let factor = with(|v| {
        v["flows"]["spending"] = json!({ "mode": "factor", "base": 6000, "factor": { "mode": "table", "interpolation": "step",
            "points": [ { "at": "start", "value": 1.0 }, { "age": 80, "value": 0.8 } ] } })
    });
    let c = curve(&factor, "/flows/spending");
    assert_eq!(at(&c, "2054-12"), 6000.0);
    assert!(close(at(&c, "2055-01"), 4800.0));
    assert!(errors(&factor).is_empty(), "{:?}", errors(&factor));

    let steps = with(
        |v| v["flows"]["spending"] = json!({ "mode": "timed", "rows": [ { "from": "start", "value": 6000 }, { "from": "2055-01", "value": 4800 } ] }),
    );
    let (a, b) = (run(&factor), run(&steps));
    assert_eq!(a.records.len(), b.records.len());
    for (x, y) in a.records.iter().zip(&b.records) {
        assert!(
            close(x.spending, y.spending) && close(x.total, y.total),
            "{:?}",
            x.month
        );
    }
    assert!(a
        .records
        .iter()
        .any(|m| m.month == YearMonth::new(2055, 1) && close(m.spending, 4800.0)));
}

// #217/AK3: a factor of steps without a value from the start warns like steps; a constant factor scales.
#[test]
fn issue_217_ak3_factor_of_late_steps_warns_and_constant_factor_scales() {
    let v = with(
        |v| v["flows"]["income"] = json!({ "mode": "factor", "base": 1000, "factor": { "mode": "timed", "rows": [ { "from": "2031-01", "value": 1 } ] } }),
    );
    assert!(api::validate(&v)
        .iter()
        .any(|i| i.message.to_string().contains("no value from the simulation start")));
    let v = with(|v| {
        v["flows"]["income"] = json!({ "mode": "factor", "base": 1000, "factor": { "mode": "timed", "rows": [] } })
    });
    assert!(errors(&v).iter().any(|x| x.contains("no entries")), "{:?}", errors(&v));
    let v = with(|v| v["flows"]["income"] = json!({ "mode": "factor", "base": 1000, "factor": 0.5 }));
    assert_eq!(at(&curve(&v, "/flows/income"), "2040-01"), 500.0);
}

// #217/AK4: the limits of a field apply to the values of the course; spending with housing to its lowest value.
#[test]
fn issue_217_ak4_limits_apply_to_the_values_of_the_course() {
    let v = with(
        |v| v["wealth"]["stockShare"] = json!({ "mode": "factor", "base": 2, "factor": { "mode": "linear", "start": { "age": 55, "value": 0.25 }, "end": { "age": 65, "value": 0.75 } } }),
    );
    assert!(
        errors(&v).iter().any(|x| x.contains("between 0 and 1")),
        "{:?}",
        errors(&v)
    );
    let mut v = with(|v| {
        v["flows"]["spending"] =
            json!({ "mode": "table", "points": [ { "age": 55, "value": 5000 }, { "age": 80, "value": 100 } ] })
    });
    v["flows"]["spendingIncludesHousing"] = json!(true);
    v["realestate"] =
        json!({ "properties": [ { "id": "w", "name": "Miete", "status": "rent", "rent": 2000, "from": "start" } ] });
    let issues = api::validate(&v);
    assert!(
        issues.iter().any(|i| i.field.as_deref() == Some("flows.spending")),
        "{:?}",
        issues.iter().map(|i| i.message.to_string()).collect::<Vec<_>>()
    );
}

/// Paths of all values of the type «Series» in the schema of the plan (`*` for an array element).
fn series_paths() -> Vec<String> {
    let schema: Value = serde_json::from_str(&engine::schema_json()).unwrap();
    let defs = schema["definitions"].clone();
    let mut out = vec![];
    fn walk(node: &Value, defs: &Value, path: &str, seen: &mut Vec<String>, out: &mut Vec<String>) {
        if let Some(r) = node.get("$ref").and_then(Value::as_str) {
            let name = r.trim_start_matches("#/definitions/");
            if name == "Series" {
                out.push(path.to_string());
                return;
            }
            if seen.iter().any(|s| s == name) {
                return;
            }
            seen.push(name.to_string());
            walk(&defs[name], defs, path, seen, out);
            seen.pop();
            return;
        }
        for k in ["allOf", "anyOf", "oneOf"] {
            for n in node.get(k).and_then(Value::as_array).into_iter().flatten() {
                walk(n, defs, path, seen, out);
            }
        }
        if let Some(items) = node.get("items") {
            walk(items, defs, &format!("{path}/*"), seen, out);
        }
        if let Some(props) = node.get("properties").and_then(Value::as_object) {
            for (k, n) in props {
                walk(n, defs, &format!("{path}/{k}"), seen, out);
            }
        }
    }
    walk(&schema, &defs, "", &mut vec![], &mut out);
    out.sort();
    out.dedup();
    out
}

/// The same course in a new form: a constant as a line between equal values, steps from the start as a factor of a
/// table of steps.
fn rewrite(s: &Value) -> Option<Value> {
    match s {
        Value::Number(_) => Some(
            json!({ "mode": "linear", "start": { "at": "start", "value": s }, "end": { "at": "end", "value": s } }),
        ),
        Value::Object(o) if o.get("mode") == Some(&json!("const")) => rewrite(&o["value"]),
        Value::Object(o) if o.get("mode") == Some(&json!("timed")) => {
            let rows = o["rows"].as_array()?;
            if rows.first()?.get("from") != Some(&json!("start")) {
                return None;
            }
            let points: Vec<Value> = rows
                .iter()
                .map(|r| json!({ "at": r["from"], "value": r["value"] }))
                .collect();
            Some(
                json!({ "mode": "factor", "base": 1, "factor": { "mode": "table", "interpolation": "step", "points": points } }),
            )
        }
        _ => None,
    }
}

/// Every value of the plan at a path of `paths` (with `*` over all array elements), as JSON pointers.
fn pointers(v: &Value, path: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for key in path.split('/').skip(1) {
        out = out
            .into_iter()
            .flat_map(|p| match (key, v.pointer(&p)) {
                ("*", Some(Value::Array(a))) => (0..a.len()).map(|i| format!("{p}/{i}")).collect(),
                ("*", _) => vec![],
                (k, Some(x)) if x.get(k).is_some() => vec![format!("{p}/{k}")],
                _ => vec![],
            })
            .collect();
    }
    out
}

// #217/AK4: every field that is a course takes the new forms; written in a new form, the example plan computes the
// same as before.
#[test]
fn issue_217_ak4_every_series_field_takes_the_new_forms() {
    let paths = series_paths();
    for p in [
        "/flows/income",
        "/flows/spending",
        "/flows/savings",
        "/wealth/stockShare",
        "/wealth/reserveMonths",
        "/returns/fixed/stocks",
        "/partner/income",
    ] {
        assert!(paths.iter().any(|x| x == p), "{p} not in {paths:?}");
    }
    let mut original = api::example_plan();
    original["returns"]["source"] = json!("fixed");
    let mut changed = original.clone();
    let mut done = vec![];
    for path in &paths {
        for ptr in pointers(&original, path) {
            if let Some(new) = rewrite(original.pointer(&ptr).unwrap()) {
                *changed.pointer_mut(&ptr).unwrap() = new;
                done.push(ptr);
            }
        }
    }
    assert!(done.len() >= 8, "{done:?}");
    assert!(errors(&changed).is_empty(), "{:?}", errors(&changed));
    let (a, b) = (run(&original), run(&changed));
    for (x, y) in a.records.iter().zip(&b.records) {
        assert!(
            (x.total - y.total).abs() < 1e-6,
            "{:?}: {} vs {}",
            x.month,
            x.total,
            y.total
        );
    }
    // Every path of a course compiles in every new form (the schema accepts them, the check knows them).
    for path in &paths {
        let mut v = base();
        for ptr in pointers(&v, path) {
            *v.pointer_mut(&ptr).unwrap() =
                json!({ "mode": "table", "points": [ { "at": "start", "value": 0.0 }, { "age": 70, "value": 0.0 } ] });
        }
        assert!(serde_json::from_value::<Plan>(v).is_ok(), "{path}");
    }
}

// #217/AK4: the upgrade leaves the new forms as they are; file round trip without loss.
#[test]
fn issue_217_ak4_upgrade_and_round_trip_keep_the_new_forms() {
    let v = with(|v| {
        v["flows"]["spendingIncludesHousing"] = json!(true);
        v["flows"]["spending"] = json!({ "mode": "factor", "base": 6000.0, "factor": { "mode": "table", "interpolation": "step",
            "points": [ { "at": "start", "value": 1.0 }, { "age": 80.0, "person": "A", "value": 0.8 } ] } });
        v["wealth"]["stockShare"] = json!({ "mode": "linear", "start": { "age": 55.0, "value": 0.9 }, "end": { "at": "2040-01", "value": 0.5 } });
    });
    let up = api::upgrade_plan(&v).plan;
    assert_eq!(up["flows"]["spending"], v["flows"]["spending"]);
    assert_eq!(up["wealth"]["stockShare"], v["wealth"]["stockShare"]);
    let back = serde_json::to_value(plan(&v)).unwrap();
    assert_eq!(back["flows"]["spending"], v["flows"]["spending"]);
    assert_eq!(back["wealth"]["stockShare"], v["wealth"]["stockShare"]);
    // A table without `interpolation` is linear and is written back as such.
    let t = with(|v| v["flows"]["income"] = json!({ "mode": "table", "points": [ { "age": 60, "value": 1 } ] }));
    assert_eq!(
        serde_json::to_value(plan(&t)).unwrap()["flows"]["income"]["interpolation"],
        "linear"
    );
}

// #217/AK4: the schema names the new forms.
#[test]
fn issue_217_ak4_schema_names_the_new_forms() {
    let s = engine::schema_json();
    for word in [
        "\"linear\"",
        "\"table\"",
        "\"factor\"",
        "\"step\"",
        "SeriesPoint",
        "interpolation",
    ] {
        assert!(s.contains(word), "{word}");
    }
}

// #217/AK4: a point in time with @ cannot set a step in a line, table or factor (message instead of losing the course).
#[test]
fn issue_217_ak4_field_assignment_with_at_refuses_the_new_forms() {
    let v = with(|v| {
        v["wealth"]["stockShare"] =
            json!({ "mode": "linear", "start": { "age": 55, "value": 0.9 }, "end": { "age": 65, "value": 0.5 } })
    });
    let e = api::set_fields(&v, &["wealth.stockShare@retire=0.4".to_string()]).unwrap_err();
    assert!(e[0].to_string().contains("not for the form “linear”"), "{e:?}");
    // Without @ the field gets a new value as before.
    let ok = api::set_fields(&v, &["wealth.stockShare=0.4".to_string()]).unwrap();
    assert_eq!(ok["wealth"]["stockShare"], json!(0.4));
}
