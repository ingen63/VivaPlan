//! #207: Field paths of the plan, as the validation messages name them (`wealth.stockShare`,
//! `household.persons.0.name`): set values by path with a check of path and type against the schema of the plan,
//! choose a step of a course over time with a point in time (`@<milestone>`, `@age:<years>`, `@<YYYY-MM>`), and list
//! the differences between two plans.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::message::{arg, Message};
use crate::model::Plan;
use crate::resolve::Resolver;
use crate::time::YearMonth;

/// Reference of the definition of a course over time in the schema.
const SERIES_REF: &str = "#/definitions/Series";

fn schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_json::to_value(schemars::schema_for!(Plan)).expect("schema is serializable"))
}

/// Follows a `$ref` to its definition.
fn resolve_ref(node: &Value) -> &Value {
    match node.get("$ref").and_then(Value::as_str) {
        Some(r) => r
            .strip_prefix("#/definitions/")
            .and_then(|name| schema()["definitions"].get(name))
            .unwrap_or(&Value::Null),
        None => node,
    }
}

/// The node and all alternatives it combines (`allOf`, `anyOf`, `oneOf`), each with its references followed.
fn alternatives(node: &Value) -> Vec<&Value> {
    let node = resolve_ref(node);
    let mut out = vec![node];
    for key in ["allOf", "anyOf", "oneOf"] {
        for sub in node.get(key).and_then(Value::as_array).into_iter().flatten() {
            out.extend(alternatives(sub));
        }
    }
    out
}

/// Schema nodes of the child `key` (property name or array index).
fn child<'a>(node: &'a Value, key: &str) -> Vec<&'a Value> {
    let index = key.parse::<usize>().is_ok();
    let mut out = vec![];
    for alt in alternatives(node) {
        if let Some(p) = alt.get("properties").and_then(|p| p.get(key)) {
            out.push(p);
        } else if let (true, Some(items)) = (index, alt.get("items")) {
            out.push(items);
        } else if let Some(extra) = alt.get("additionalProperties").filter(|a| a.is_object()) {
            out.push(extra);
        }
    }
    out
}

/// Schema nodes of a path (empty: the path does not exist in the schema).
fn nodes_at(path: &[&str]) -> Vec<&'static Value> {
    let mut nodes = vec![schema()];
    for key in path {
        nodes = nodes.iter().flat_map(|n| child(n, key)).collect();
        if nodes.is_empty() {
            break;
        }
    }
    nodes
}

/// Whether the node refers to the course over time (`Series`), directly or in a combination.
fn is_series(node: &Value) -> bool {
    node.get("$ref").and_then(Value::as_str) == Some(SERIES_REF)
        || ["allOf", "anyOf", "oneOf"]
            .iter()
            .filter_map(|k| node.get(*k).and_then(Value::as_array))
            .flatten()
            .any(is_series)
}

/// JSON type names of a value («integer» also for a whole number).
fn type_names(v: &Value) -> &'static [&'static str] {
    match v {
        Value::Null => &["null"],
        Value::Bool(_) => &["boolean"],
        Value::Number(n) if n.is_i64() || n.is_u64() => &["integer", "number"],
        Value::Number(_) => &["number"],
        Value::String(_) => &["string"],
        Value::Array(_) => &["array"],
        Value::Object(_) => &["object"],
    }
}

/// Allowed values of one alternative: type names and fixed values (`enum`). `None`: any value.
fn allowed(alt: &Value) -> Option<Vec<String>> {
    if let Some(values) = alt.get("enum").and_then(Value::as_array) {
        return Some(values.iter().map(Value::to_string).collect());
    }
    match alt.get("type") {
        Some(Value::String(t)) => Some(vec![t.clone()]),
        Some(Value::Array(ts)) => Some(ts.iter().filter_map(Value::as_str).map(str::to_string).collect()),
        _ => None,
    }
}

/// Checks the type of a value against the schema nodes of its path; `Err` with the expected types.
fn check_type(nodes: &[&Value], value: &Value) -> Result<(), Vec<String>> {
    let mut expected = BTreeSet::new();
    for alt in nodes.iter().flat_map(|n| alternatives(n)) {
        if alt.get("allOf").or(alt.get("anyOf")).or(alt.get("oneOf")).is_some() && allowed(alt).is_none() {
            // Only a combination: its alternatives decide.
            continue;
        }
        let Some(list) = allowed(alt) else {
            return Ok(());
        };
        let fits = if alt.get("enum").is_some() {
            list.contains(&value.to_string())
        } else {
            type_names(value).iter().any(|t| list.iter().any(|l| l == t))
        };
        if fits {
            return Ok(());
        }
        expected.extend(list);
    }
    Err(expected.into_iter().collect())
}

/// One change `path[@time]=value`.
struct Assignment<'a> {
    path: &'a str,
    keys: Vec<&'a str>,
    at: Option<&'a str>,
    value: Value,
}

/// Splits `path[@time]=value`. The value is read as JSON (number, `true`, `"text"`, `{…}`); anything else is a text.
fn parse_assignment(text: &str) -> Result<Assignment<'_>, Message> {
    let (target, raw) = text
        .split_once('=')
        .filter(|(p, _)| !p.trim().is_empty())
        .ok_or_else(|| msg!(field_assignment_invalid, text = text))?;
    let (path, at) = match target.split_once('@') {
        Some((p, a)) => (p.trim(), Some(a.trim())),
        None => (target.trim(), None),
    };
    let value = serde_json::from_str(raw.trim()).unwrap_or_else(|_| Value::String(raw.to_string()));
    Ok(Assignment {
        path,
        keys: path.split('.').collect(),
        at,
        value,
    })
}

/// Month and time reference of a point in time `@<milestone>`, `@age:<years>` (first person) or `@<YYYY-MM>`.
fn point_in_time(plan: &Plan, path: &str, at: &str) -> Result<(YearMonth, String), Message> {
    let unknown = || msg!(field_time_unknown, path = path, at = at);
    if let Some(years) = at.strip_prefix("age:") {
        let years: f64 = years.trim().parse().map_err(|_| unknown())?;
        let birth = plan.household.persons.first().ok_or_else(unknown)?.birth;
        let m = birth.plus((years * 12.0).round() as i32);
        return Ok((m, m.iso()));
    }
    if let Ok(m) = at.parse::<YearMonth>() {
        return Ok((m, m.iso()));
    }
    let r = Resolver::new(plan).map_err(|e| e.message())?;
    let m = r.milestone(at).map_err(|_| unknown())?;
    Ok((m, at.to_string()))
}

/// Sets the step of a course over time from month `m` (time reference `from`): the step that starts in this month,
/// otherwise a new step. A constant value becomes a course that starts with it.
fn set_step(series: &Value, plan: &Plan, m: YearMonth, from: String, value: Value) -> Result<Value, Message> {
    let mut rows = match series {
        Value::Object(o) if o.get("mode") == Some(&json!("timed")) => {
            o.get("rows").and_then(Value::as_array).cloned().unwrap_or_default()
        }
        Value::Object(o) => vec![json!({ "from": "start", "value": o.get("value").cloned().unwrap_or(json!(0.0)) })],
        Value::Null => vec![json!({ "from": "start", "value": 0.0 })],
        other => vec![json!({ "from": "start", "value": other })],
    };
    let r = Resolver::new(plan).map_err(|e| e.message())?;
    let month_of = |row: &Value| {
        row.get("from")
            .and_then(Value::as_str)
            .and_then(|f| r.resolve(&crate::time::TimeRef(f.to_string())).ok().flatten())
    };
    if let Some(row) = rows.iter_mut().rev().find(|row| month_of(row) == Some(m)) {
        row["value"] = value;
    } else {
        let pos = rows
            .iter()
            .rposition(|row| month_of(row).is_some_and(|x| x <= m))
            .map_or(0, |i| i + 1);
        rows.insert(pos, json!({ "from": from, "value": value }));
    }
    Ok(json!({ "mode": "timed", "rows": rows }))
}

/// Writes `value` at `keys`; missing objects on the way are created, array indexes must exist.
fn put(target: &mut Value, keys: &[&str], value: Value) -> bool {
    let Some((key, rest)) = keys.split_first() else {
        *target = value;
        return true;
    };
    if target.is_null() {
        *target = Value::Object(Map::new());
    }
    let next = match target {
        Value::Object(o) => o.entry(key.to_string()).or_insert(Value::Null),
        Value::Array(a) => match key.parse::<usize>().ok().and_then(|i| a.get_mut(i)) {
            Some(x) => x,
            None => return false,
        },
        _ => return false,
    };
    put(next, rest, value)
}

/// The value at `keys` (`null` if missing).
fn get<'a>(v: &'a Value, keys: &[&str]) -> &'a Value {
    v.pointer(&format!("/{}", keys.join("/"))).unwrap_or(&Value::Null)
}

/// Parses the plan for points in time; the plan before the change must be valid.
fn parsed(plan: &Value) -> Result<Plan, Message> {
    serde_json::from_value(plan.clone()).map_err(|e| msg!(plan_incomplete_detail, detail = e.to_string()))
}

/// Applies one change to `plan`.
fn apply(plan: &mut Value, a: &Assignment) -> Result<(), Message> {
    let nodes = nodes_at(&a.keys);
    if nodes.is_empty() || a.keys.iter().any(|k| k.is_empty()) {
        return Err(msg!(field_path_unknown, path = a.path));
    }
    let wrong_type = |expected: Vec<String>| {
        msg!(
            field_type_wrong,
            path = a.path,
            value = a.value.to_string(),
            expected = arg(&expected)
        )
    };
    let value = match a.at {
        None => {
            check_type(&nodes, &a.value).map_err(wrong_type)?;
            a.value.clone()
        }
        Some(at) => {
            if !nodes.iter().any(|n| is_series(n)) {
                return Err(msg!(field_not_series, path = a.path));
            }
            if !a.value.is_number() {
                return Err(wrong_type(vec!["number".into()]));
            }
            let p = parsed(plan)?;
            let (m, from) = point_in_time(&p, a.path, at)?;
            set_step(get(plan, &a.keys), &p, m, from, a.value.clone())?
        }
    };
    if put(plan, &a.keys, value) {
        Ok(())
    } else {
        Err(msg!(field_path_unknown, path = a.path))
    }
}

/// Sets all changes `path[@time]=value` in a copy of the plan. All or none: every unknown path, wrong type or unknown
/// point in time gives a message, and the plan stays unchanged. The changed plan must still be readable as a plan;
/// the content is checked by the validation as usual.
pub fn set(plan: &Value, assignments: &[String]) -> Result<Value, Vec<Message>> {
    let mut v = plan.clone();
    let mut errors = vec![];
    for text in assignments {
        if let Err(e) = parse_assignment(text).and_then(|a| apply(&mut v, &a)) {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        if let Err(e) = serde_json::from_value::<Plan>(v.clone()) {
            errors.push(msg!(field_changes_invalid, detail = e.to_string()));
        }
    }
    if errors.is_empty() {
        Ok(v)
    } else {
        Err(errors)
    }
}

/// A difference between two plans: field path, old and new value (`null` if missing).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Change {
    pub path: String,
    pub old: Value,
    pub new: Value,
}

fn diff_into(path: &str, a: Option<&Value>, b: Option<&Value>, out: &mut Vec<Change>) {
    let join = |k: &str| {
        if path.is_empty() {
            k.to_string()
        } else {
            format!("{path}.{k}")
        }
    };
    match (a, b) {
        (Some(Value::Object(x)), Some(Value::Object(y))) => {
            let keys: BTreeSet<&String> = x.keys().chain(y.keys()).collect();
            for k in keys {
                diff_into(&join(k), x.get(k), y.get(k), out);
            }
        }
        (Some(Value::Array(x)), Some(Value::Array(y))) => {
            for i in 0..x.len().max(y.len()) {
                diff_into(&join(&i.to_string()), x.get(i), y.get(i), out);
            }
        }
        _ if a != b => out.push(Change {
            path: path.to_string(),
            old: a.cloned().unwrap_or(Value::Null),
            new: b.cloned().unwrap_or(Value::Null),
        }),
        _ => {}
    }
}

/// All differences from `a` to `b`, in the order of the paths (object keys sorted, array elements by index).
pub fn diff(a: &Value, b: &Value) -> Vec<Change> {
    let mut out = vec![];
    diff_into("", Some(a), Some(b), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> Value {
        crate::api::example_plan()
    }

    fn set_ok(plan: &Value, list: &[&str]) -> Value {
        set(plan, &list.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
    }

    fn set_err(plan: &Value, list: &[&str]) -> Vec<Message> {
        set(plan, &list.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap_err()
    }

    /// #207/AK2: values of different types by the paths of the validation messages.
    #[test]
    fn issue_207_ak2_sets_values_by_path() {
        let v = set_ok(
            &example(),
            &[
                "wealth.stockShare=0.7",
                "household.persons.0.name=Max Muster",
                "withdrawal.strategy=\"fixed_rate\"",
                "withdrawal.fixedRate.rate=0.035",
                "milestones.1.years=60",
                "flows.spendingIncludesHousing=false",
                "returns.startDate=1990-01",
            ],
        );
        assert_eq!(v["wealth"]["stockShare"], json!(0.7));
        assert_eq!(v["household"]["persons"][0]["name"], "Max Muster");
        assert_eq!(v["withdrawal"]["strategy"], "fixed_rate");
        assert_eq!(v["withdrawal"]["fixedRate"]["rate"], json!(0.035));
        assert_eq!(v["milestones"][1]["years"], json!(60));
        assert_eq!(v["flows"]["spendingIncludesHousing"], json!(false));
        assert_eq!(v["returns"]["startDate"], "1990-01");
        // A text without quotes is a text; an enum value without quotes as well.
        let v = set_ok(&example(), &["withdrawal.strategy=vpw", "project.name=Test 2"]);
        assert_eq!(v["withdrawal"]["strategy"], "vpw");
        assert_eq!(v["project"]["name"], "Test 2");
        // A missing optional object is created.
        let mut base = example();
        base["withdrawal"].as_object_mut().unwrap().remove("vpw");
        let v = set_ok(
            &base,
            &["withdrawal.vpw.horizonAge=95", "withdrawal.vpw.realReturn=0.01"],
        );
        assert_eq!(v["withdrawal"]["vpw"], json!({ "horizonAge": 95, "realReturn": 0.01 }));
    }

    /// #207/AK2: an unknown path or a wrong type gives a message, and none of the changes is applied.
    #[test]
    fn issue_207_ak2_unknown_path_or_wrong_type_changes_nothing() {
        let base = example();
        let e = set_err(&base, &["wealth.stockShare=0.7", "wealth.stockShar=0.5"]);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].code(), "field_path_unknown");
        assert_eq!(e[0].text(), "Unknown field path “wealth.stockShar”");
        let e = set_err(&base, &["wealth.cash=\"viel\"", "withdrawal.strategy=\"best\""]);
        assert_eq!(e.iter().map(Message::code).collect::<Vec<_>>(), ["field_type_wrong"; 2]);
        assert!(
            e[0].text().contains("wealth.cash") && e[0].text().contains("number"),
            "{}",
            e[0]
        );
        assert!(e[1].text().contains("\"standard\""), "{}", e[1]);
        for bad in [
            "household.persons.5.name=X",
            "household.persons.x.name=X",
            "wealth..cash=1",
            "wealth.cash.value=1",
            "nothing=1",
        ] {
            let e = set_err(&base, &[bad]);
            assert_eq!(e[0].code(), "field_path_unknown", "{bad}");
        }
        let e = set_err(&base, &["wealth.stockShare"]);
        assert_eq!(e[0].code(), "field_assignment_invalid");
        let e = set_err(&base, &["=1"]);
        assert_eq!(e[0].code(), "field_assignment_invalid");
        // Types inside an object are checked when reading the plan.
        let e = set_err(&base, &["wealth={\"cash\": \"x\"}"]);
        assert_eq!(e[0].code(), "field_changes_invalid");
        // A wrong month is reported as well (pattern of the schema, checked when reading).
        let e = set_err(&base, &["returns.startDate=1990-13"]);
        assert_eq!(e[0].code(), "field_changes_invalid");
        // A null where allowed is fine.
        set_ok(&base, &["withdrawal.floor=null"]);
        // A list of types: a text does not fit `["number", "null"]`.
        let e = set_err(&base, &["withdrawal.floor=\"x\""]);
        assert_eq!(e[0].code(), "field_type_wrong");
        assert!(e[0].text().contains("null, number"), "{}", e[0]);
        // A negative whole number is an `integer`; the range (no negative count) is checked when reading.
        let e = set_err(&base, &["returns.bootstrap.paths=-5"]);
        assert_eq!(e[0].code(), "field_changes_invalid");
        let e = set_err(&base, &["returns.bootstrap.paths=1.5"]);
        assert_eq!(e[0].code(), "field_type_wrong");
        assert!(e[0].text().contains("expected integer"), "{}", e[0]);
        set_ok(&base, &["returns.bootstrap.paths=500"]);
        // A field without a type in the schema (variations) takes any value.
        let v = set_ok(
            &base,
            &["scenarios.sweeps.0={\"param\": \"withdrawal.rate\", \"from\": 0.03, \"to\": 0.05, \"step\": 0.01}"],
        );
        assert_eq!(v["scenarios"]["sweeps"][0]["param"], "withdrawal.rate");
    }

    fn rows(v: &Value) -> Vec<(String, f64)> {
        v["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["from"].as_str().unwrap().to_string(), r["value"].as_f64().unwrap()))
            .collect()
    }

    /// #207/AK7: a point in time chooses the step from this point; an existing step is changed, otherwise a new step
    /// begins there.
    #[test]
    fn issue_207_ak7_point_in_time_chooses_the_step() {
        let r = |list: &[(&str, f64)]| list.iter().map(|(a, b)| (a.to_string(), *b)).collect::<Vec<_>>();
        // Constant stock share: before and after retirement
        let v = set_ok(
            &example(),
            &["wealth.stockShare@start=0.8", "wealth.stockShare@retire=0.5"],
        );
        assert_eq!(rows(&v["wealth"]["stockShare"]), r(&[("start", 0.8), ("retire", 0.5)]));
        // Existing step by milestone, new step by date and by age (birth 1970-06 → age 80 = 2050-06, existing row)
        let v = set_ok(
            &example(),
            &[
                "flows.spending@retire=9000",
                "flows.spending@2035-01=8900",
                "flows.spending@age:80=7000",
                "flows.spending@age:62.5=9100",
            ],
        );
        assert_eq!(
            rows(&v["flows"]["spending"]),
            r(&[
                ("start", 10200.0),
                ("retire", 9000.0),
                ("2032-12", 9100.0),
                ("2035-01", 8900.0),
                ("sell_house", 8700.0),
                ("2050-06", 7000.0),
            ])
        );
        // A constant course (`mode: const`) keeps its value before the new step.
        let v = set_ok(&example(), &["wealth.stockShare@retire=0.5"]);
        assert_eq!(rows(&v["wealth"]["stockShare"]), r(&[("start", 0.6), ("retire", 0.5)]));
        // A plain number as a course
        let mut base = example();
        base["wealth"]["stockShare"] = json!(0.6);
        let v = set_ok(&base, &["wealth.stockShare@ahv=0.4"]);
        assert_eq!(rows(&v["wealth"]["stockShare"]), r(&[("start", 0.6), ("ahv", 0.4)]));
        // The plan still computes, with the new step.
        let p: Plan = serde_json::from_value(v).unwrap();
        let res = crate::resolve::Resolver::new(&p).unwrap();
        let c = crate::resolve::Curve::compile(&p.wealth.stock_share, &res).unwrap();
        assert_eq!(c.at(res.milestone("ahv").unwrap()), 0.4);
        assert_eq!(c.at(res.milestone("ahv").unwrap().plus(-1)), 0.6);
    }

    /// #207/AK7: errors with points in time.
    #[test]
    fn issue_207_ak7_point_in_time_errors() {
        let base = example();
        for (bad, code) in [
            ("wealth.stockShare@nowhere=0.5", "field_time_unknown"),
            ("wealth.stockShare@age:x=0.5", "field_time_unknown"),
            ("wealth.cash@retire=5", "field_not_series"),
            ("wealth.stockShare@retire=\"half\"", "field_type_wrong"),
        ] {
            let e = set_err(&base, &[bad]);
            assert_eq!(e[0].code(), code, "{bad}: {}", e[0]);
        }
        let mut no_person = base.clone();
        no_person["household"]["persons"] = json!([]);
        let e = set_err(&no_person, &["wealth.stockShare@age:60=0.5"]);
        assert!(!e.is_empty());
        // The plan before the change must be readable for a point in time.
        let mut broken = base.clone();
        broken["wealth"]["cash"] = json!("x");
        let e = set_err(&broken, &["wealth.stockShare@retire=0.5"]);
        assert_eq!(e[0].code(), "plan_incomplete_detail");
    }

    /// #207/AK4: differences with path, old and new value.
    #[test]
    fn issue_207_ak4_diff_lists_path_old_and_new() {
        let a = example();
        let b = set_ok(&a, &["wealth.stockShare=0.7", "household.persons.0.name=Max"]);
        let mut b2 = b.clone();
        b2["pillar3a"].as_array_mut().unwrap().clear();
        b2["project"]
            .as_object_mut()
            .unwrap()
            .insert("note".into(), json!("neu"));
        let d = diff(&a, &b2);
        assert_eq!(
            d.iter().map(|c| c.path.as_str()).collect::<Vec<_>>(),
            [
                "household.persons.0.name",
                "pillar3a.0",
                "project.note",
                "wealth.stockShare"
            ]
        );
        assert_eq!(d[0].old, "Erika Muster");
        assert_eq!(d[0].new, "Max");
        assert_eq!(d[1].new, Value::Null);
        assert_eq!(d[2].old, Value::Null);
        assert_eq!(d[3].old, json!({ "mode": "const", "value": 0.6 }));
        assert_eq!(d[3].new, json!(0.7));
        assert!(diff(&a, &a).is_empty());
    }

    #[test]
    fn issue_207_schema_helpers() {
        assert!(nodes_at(&["wealth", "stockShare"]).iter().any(|n| is_series(n)));
        assert!(!nodes_at(&["wealth", "cash"]).iter().any(|n| is_series(n)));
        assert!(nodes_at(&["milestones", "0", "kind"]).len() >= 2);
        assert_eq!(type_names(&json!(1)), ["integer", "number"]);
        assert_eq!(type_names(&json!(1.5)), ["number"]);
        assert_eq!(resolve_ref(&json!({ "$ref": "#/other/X" })), &Value::Null);
    }
}
