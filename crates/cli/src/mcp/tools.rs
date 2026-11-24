//! #210: Reading tools of the MCP server: open, read, look up and check plans, results of the session, data library and
//! the catalog for AIs (`describe`). Thin layer: check the arguments, call the engine through `dispatch::call`, shape
//! the answer. Computing tools follow with #211, changing and saving with #212.

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use super::privacy::neutral_plan;
use super::session::Session;

/// Answer of a tool before names, paths and size are handled.
pub(crate) struct Answer {
    pub(crate) data: Value,
    /// Answer with results: ends with the disclaimer (AK6).
    pub(crate) results: bool,
}

impl Answer {
    fn data(data: Value) -> Self {
        Answer { data, results: false }
    }
    fn results(data: Value) -> Self {
        Answer { data, results: true }
    }
}

/// Error of a tool: comes back as a tool result with `isError`, so that the AI can correct the call.
pub(crate) type Outcome = Result<Answer, String>;

/// Hints of the protocol: all tools of #210 only read and touch nothing outside the computer.
fn read_only() -> Value {
    json!({ "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false })
}

fn tool(name: &str, title: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "title": title,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
        },
        "annotations": read_only(),
    })
}

const VIEWS: &[&str] = &["summary", "yearly", "events", "bands", "variations"];

/// The tool list of `tools/list`.
pub(crate) fn list() -> Vec<Value> {
    let sections: Vec<&str> = engine::describe::Section::ALL.iter().map(|s| s.name()).collect();
    vec![
        tool(
            "describe",
            "Catalog for AIs",
            "What VivaPlan can change, vary and measure: field paths with units, forms of time series, state variables, \
             metric building blocks, experiment format, examples, measured computing time and limits. Without section a \
             compact overview of all sections; read the section you need. With plan the computing time is measured on that plan.",
            json!({
                "section": { "type": "string", "enum": sections, "description": "Part of the catalog; without: all sections." },
                "plan": { "type": "string", "description": "Id of an open plan (p1, …) to measure the computing time on." },
            }),
            &[],
        ),
        tool(
            "plan_open",
            "Open a plan",
            "Opens a VivaPlan project file (.vplan or an older .rplan) as a working copy in memory, brought up to the current \
             format, and returns its id (p1, …) with an overview without values: period, number of persons and properties, \
             notes of the update, number of check messages. Only the named file is read; folders are not searched. \
             With example: true the example plan is opened instead.",
            json!({
                "path": { "type": "string", "description": "Path of the file (.vplan or .rplan)." },
                "example": { "type": "boolean", "description": "Open the example plan instead of a file." },
            }),
            &[],
        ),
        tool(
            "plan_get",
            "Read part of a plan",
            "Reads the values of a section of an open plan (e.g. household, flows, wealth, pension, realestate, taxes, \
             returns, withdrawal, scenarios) or of a single field by field path as in the check messages \
             (household.persons.0.birth). Without section and path: the list of sections with their size. Shares are \
             fractions (0.05 = 5 %), amounts are per year in the currency of the plan unless the field says otherwise.",
            json!({
                "plan": { "type": "string", "description": "Id of the open plan (p1, …)." },
                "section": { "type": "string", "description": "Top-level section of the plan." },
                "path": { "type": "string", "description": "Field path, dot separated, list entries by number from 0." },
            }),
            &["plan"],
        ),
        tool(
            "plan_schema",
            "Structure of the project file",
            "Looks up the structure of the project file (JSON Schema with descriptions). Without arguments the sections \
             with their description; with section the part of the schema with the definitions it uses; with definition a \
             single definition. Units and the meaning of the fields in short: describe with section fields.",
            json!({
                "section": { "type": "string", "description": "Top-level section of the plan." },
                "definition": { "type": "string", "description": "Name of a definition of the schema (e.g. Person)." },
            }),
            &[],
        ),
        tool(
            "plan_validate",
            "Check a plan",
            "Checks an open plan as the app does: messages with severity (error, warning, info), section, field path and text. \
             A plan with errors cannot be computed.",
            json!({ "plan": { "type": "string", "description": "Id of the open plan (p1, …)." } }),
            &["plan"],
        ),
        tool(
            "result_list",
            "Results of the session",
            "Lists the results computed in this session: id (r1, …), plan, tool, number of runs, main key figure. \
             Results stay only in the memory of the server; the results of the desktop app are separate.",
            json!({}),
            &[],
        ),
        tool(
            "result_get",
            "Details of a result",
            "Details of a result of the session. view: summary (key figures and the result in words, default), yearly \
             (one row per calendar year, nominal or real), events (events of the run), bands (wealth per year at the \
             percentiles over all runs), variations (key figures per combination of the variations). With several runs \
             percentile picks the run for yearly and events (0 worst, 1, 5, 50, 95, 99; default 50). from and to limit \
             the calendar years.",
            json!({
                "result": { "type": "string", "description": "Id of the result (r1, …)." },
                "view": { "type": "string", "enum": VIEWS },
                "percentile": { "type": "integer", "enum": [0, 1, 5, 50, 95, 99] },
                "real": { "type": "boolean", "description": "Values in today's purchasing power instead of nominal (yearly, bands)." },
                "from": { "type": "integer", "description": "First calendar year." },
                "to": { "type": "integer", "description": "Last calendar year." },
            }),
            &["result"],
        ),
        tool(
            "library_list",
            "Data library",
            "Time series of the data library of the app (folder «Zeitreihen» of its data folder): file, currency, period, \
             monthly or yearly, average returns of stocks and bonds and inflation. Plans refer to them as library:<file>.",
            json!({}),
            &[],
        ),
    ]
}

/// Answers a tool call; `None` for an unknown tool (an error of the protocol).
pub(crate) fn call(s: &mut Session, name: &str, args: &Value) -> Option<Outcome> {
    let args = match args {
        Value::Null => &Value::Object(Map::new()),
        a => a,
    };
    let known = list().into_iter().find(|t| t["name"] == name)?;
    Some(check_args(&known, args).and_then(|()| match name {
        "describe" => describe(s, args),
        "plan_open" => plan_open(s, args),
        "plan_get" => plan_get(s, args),
        "plan_schema" => plan_schema(args),
        "plan_validate" => plan_validate(s, args),
        "result_list" => Ok(result_list(s)),
        "result_get" => result_get(s, args),
        "library_list" => library_list(s),
        _ => unreachable!("every listed tool has a branch"),
    }))
}

/// Unknown arguments and wrong types are errors the AI can correct.
fn check_args(tool: &Value, args: &Value) -> Result<(), String> {
    let Some(given) = args.as_object() else {
        return Err("The arguments must be an object".into());
    };
    let props = tool["inputSchema"]["properties"].as_object().expect("schema");
    for (k, v) in given {
        let Some(p) = props.get(k) else {
            let known: Vec<&str> = props.keys().map(String::as_str).collect();
            return Err(format!("Unknown argument {k}; arguments: {}", known.join(", ")));
        };
        let ok = match p["type"].as_str() {
            Some("string") => v.is_string(),
            Some("boolean") => v.is_boolean(),
            Some("integer") => v.is_i64(),
            _ => true,
        };
        if !ok {
            return Err(format!(
                "Argument {k} must be of type {}",
                p["type"].as_str().unwrap_or("?")
            ));
        }
        if let Some(allowed) = p["enum"].as_array().filter(|a| !a.contains(v)) {
            let list: Vec<String> = allowed.iter().map(|a| a.to_string().replace('"', "")).collect();
            return Err(format!("Argument {k}: {v} is not one of {}", list.join(", ")));
        }
    }
    for r in tool["inputSchema"]["required"].as_array().into_iter().flatten() {
        let r = r.as_str().expect("name");
        if !given.contains_key(r) {
            return Err(format!("Argument {r} missing"));
        }
    }
    Ok(())
}

fn arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

// ---------------------------------------------------------------------------------------------------------------------
// Plans

/// First answer of `plan_open` in a session: data leaves the computer only through the AI the user connected.
pub(crate) const PRIVACY_NOTE: &str = "The data you query through VivaPlan becomes part of the conversation with your AI \
     and is processed by its provider. VivaPlan itself transfers nothing; with a local model nothing leaves this computer. \
     Names are replaced by neutral ones and answers name only file names.";

/// Full disclaimer of the app (English text of the user interface, #61).
pub(crate) fn full_disclaimer() -> String {
    let en: Value = serde_json::from_str(include_str!("../../../app/ui/i18n/en.json")).expect("valid JSON");
    en["disclaimer.short"]
        .as_str()
        .expect("text of the disclaimer")
        .to_string()
}

fn plan_open(s: &mut Session, args: &Value) -> Outcome {
    let example = args.get("example").and_then(Value::as_bool).unwrap_or(false);
    let (file, raw) = match (arg(args, "path"), example) {
        (Some(_), true) => return Err("Give either path or example: true, not both".into()),
        (None, false) => return Err("Give the path of a plan (.vplan or .rplan) or example: true".into()),
        (None, true) => (None, s.call("example_plan", &json!({}), None)?),
        (Some(p), false) => {
            let path = PathBuf::from(p);
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if ext != "vplan" && ext != "rplan" {
                return Err("Only project files ending in .vplan or .rplan are opened".into());
            }
            let path = if path.is_absolute() {
                path
            } else {
                std::env::current_dir().map_err(|e| e.to_string())?.join(path)
            };
            let plan = engine::api::read_plan(&path).map_err(|e| e.message().text().to_string())?;
            (Some(path), plan)
        }
    };
    let up = s.call("upgrade_plan", &json!({ "plan": raw }), file.as_deref())?;
    let plan = up["plan"].clone();
    let notes: Vec<Value> = up["notes"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|n| n["text"].clone())
        .collect();
    let issues = s.call("validate", &json!({ "plan": s.with_settings(&plan) }), file.as_deref())?;
    let count = |sev: &str| {
        issues
            .as_array()
            .map_or(0, |a| a.iter().filter(|i| i["severity"] == sev).count())
    };
    let overview = overview(&plan);
    let name = if s.names {
        plan["project"]["name"].clone()
    } else {
        neutral_plan(&plan)["project"]["name"].clone()
    };
    let p = s.add_plan(file, plan);
    let mut out = json!({
        "plan": p.id,
        "file": p.file_name(),
        "name": name,
    });
    let o = out.as_object_mut().expect("object");
    o.extend(overview.as_object().expect("object").clone());
    o.insert("updateNotes".into(), Value::Array(notes));
    o.insert(
        "checks".into(),
        json!({ "errors": count("error"), "warnings": count("warning"), "infos": count("info") }),
    );
    o.insert(
        "next".into(),
        json!(
            "plan_get for the values of a section, plan_validate for the messages, describe for what can be measured"
        ),
    );
    if !s.notice_shown {
        s.notice_shown = true;
        o.insert("privacy".into(), json!(PRIVACY_NOTE));
        o.insert("disclaimer".into(), json!(full_disclaimer()));
    }
    Ok(Answer::data(out))
}

/// Overview of a plan without values: period, counts, sources.
fn overview(plan: &Value) -> Value {
    let len = |v: &Value| v.as_array().map_or(0, Vec::len);
    let period = serde_json::from_value::<engine::model::Plan>(plan.clone())
        .ok()
        .and_then(|p| engine::resolve::Resolver::new(&p).ok())
        .and_then(|r| Some(json!({ "start": r.milestone("start").ok()?, "end": r.milestone("end").ok()? })));
    json!({
        "currency": plan["project"]["currency"],
        "period": period,
        "persons": len(&plan["household"]["persons"]),
        "partner": plan.get("partner").is_some_and(|p| !p.is_null()),
        "properties": len(&plan["realestate"]["properties"]),
        "pillar3aAccounts": len(&plan["pillar3a"]),
        "oneOffExpenses": len(&plan["flows"]["extras"]),
        "milestones": len(&plan["milestones"]),
        "variations": len(&plan["scenarios"]["sweeps"]),
        "returnSource": plan["returns"]["source"],
        "withdrawalStrategy": plan["withdrawal"]["strategy"],
        "sections": plan.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
    })
}

/// Value at a dot separated field path (`household.persons.0.birth`).
fn at_path<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(v, |v, seg| match v {
        Value::Object(o) => o.get(seg),
        Value::Array(a) => a.get(seg.parse::<usize>().ok()?),
        _ => None,
    })
}

fn plan_get(s: &mut Session, args: &Value) -> Outcome {
    let p = s.plan(arg(args, "plan").unwrap_or_default())?;
    let plan = if s.names { p.plan.clone() } else { neutral_plan(&p.plan) };
    let sections = || {
        plan.as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    };
    match (arg(args, "section"), arg(args, "path")) {
        (Some(_), Some(_)) => Err("Give either section or path, not both".into()),
        (None, None) => {
            let list: Vec<Value> = plan
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, v)| json!({ "section": k, "characters": v.to_string().chars().count() }))
                .collect();
            Ok(Answer::data(json!({
                "plan": p.id,
                "sections": list,
                "next": "plan_get with section or path",
            })))
        }
        (Some(sec), None) => match plan.get(sec) {
            Some(v) => Ok(Answer::data(json!({ "plan": p.id, "section": sec, "value": v }))),
            None => Err(format!("Unknown section {sec}; sections: {}", sections().join(", "))),
        },
        (None, Some(path)) => match at_path(&plan, path) {
            Some(v) => Ok(Answer::data(json!({ "plan": p.id, "path": path, "value": v }))),
            None => Err(format!(
                "Field path {path} does not exist in plan {}; plan_get with a section shows its fields, plan_schema the possible ones",
                p.id
            )),
        },
    }
}

fn schema() -> Value {
    serde_json::from_str(&engine::schema_json()).expect("schema is JSON")
}

/// Names of the definitions a part of the schema refers to (`#/definitions/<name>`).
fn refs(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            if let Some(name) = o
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|r| r.strip_prefix("#/definitions/"))
            {
                if !out.iter().any(|n| n == name) {
                    out.push(name.to_string());
                }
            }
            o.values().for_each(|x| refs(x, out));
        }
        Value::Array(a) => a.iter().for_each(|x| refs(x, out)),
        _ => {}
    }
}

/// Part of the schema that still fits easily in one answer.
const SCHEMA_BUDGET: usize = super::MAX_CHARS * 3 / 4;

fn plan_schema(args: &Value) -> Outcome {
    let schema = schema();
    let defs = &schema["definitions"];
    let props = schema["properties"].as_object().expect("properties");
    match (arg(args, "section"), arg(args, "definition")) {
        (Some(_), Some(_)) => Err("Give either section or definition, not both".into()),
        (None, None) => {
            let list: Map<String, Value> = props
                .iter()
                .map(|(k, v)| {
                    let mut names = Vec::new();
                    refs(v, &mut names);
                    let desc = v
                        .get("description")
                        .or_else(|| names.first().and_then(|n| defs[n].get("description")))
                        .cloned()
                        .unwrap_or(Value::Null);
                    (k.clone(), desc)
                })
                .collect();
            Ok(Answer::data(json!({
                "sections": list,
                "next": "plan_schema with section or definition; describe with section fields for units",
            })))
        }
        (None, Some(d)) => match defs.get(d) {
            Some(v) => Ok(Answer::data(json!({ "definition": d, "schema": v }))),
            None => {
                let names: Vec<&String> = defs.as_object().expect("definitions").keys().collect();
                Err(format!(
                    "Unknown definition {d}; definitions: {}",
                    names.iter().map(|n| n.as_str()).collect::<Vec<_>>().join(", ")
                ))
            }
        },
        (Some(sec), None) => {
            let Some(part) = props.get(sec) else {
                let names: Vec<&str> = props.keys().map(String::as_str).collect();
                return Err(format!("Unknown section {sec}; sections: {}", names.join(", ")));
            };
            // All definitions the section uses, also indirectly.
            let mut names = Vec::new();
            refs(part, &mut names);
            let mut i = 0;
            while i < names.len() {
                let mut more = Vec::new();
                refs(&defs[&names[i]], &mut more);
                for n in more {
                    if !names.contains(&n) {
                        names.push(n);
                    }
                }
                i += 1;
            }
            let all: Map<String, Value> = names.iter().map(|n| (n.clone(), defs[n].clone())).collect();
            if Value::Object(all.clone()).to_string().chars().count() <= SCHEMA_BUDGET {
                return Ok(Answer::data(
                    json!({ "section": sec, "schema": part, "definitions": all }),
                ));
            }
            // Too large: the directly used definitions, the others by name.
            let mut direct = Vec::new();
            refs(part, &mut direct);
            let shown: Map<String, Value> = direct.iter().map(|n| (n.clone(), defs[n].clone())).collect();
            let rest: Vec<&String> = names.iter().filter(|n| !direct.contains(n)).collect();
            Ok(Answer::data(json!({
                "section": sec,
                "schema": part,
                "definitions": shown,
                "moreDefinitions": rest,
                "next": "plan_schema with definition for the further definitions",
            })))
        }
    }
}

fn plan_validate(s: &mut Session, args: &Value) -> Outcome {
    let p = s.plan(arg(args, "plan").unwrap_or_default())?;
    let issues = s.call(
        "validate",
        &json!({ "plan": s.with_settings(&p.plan) }),
        p.file.as_deref(),
    )?;
    let list: Vec<Value> = issues
        .as_array()
        .into_iter()
        .flatten()
        .map(|i| {
            json!({
                "severity": i["severity"],
                "section": i["section"],
                "field": i["field"],
                "code": i["code"],
                "text": i["message"],
            })
        })
        .collect();
    let count = |sev: &str| list.iter().filter(|i| i["severity"] == sev).count();
    Ok(Answer::data(json!({
        "plan": p.id,
        "errors": count("error"),
        "warnings": count("warning"),
        "infos": count("info"),
        "messages": list,
    })))
}

fn library_list(s: &mut Session) -> Outcome {
    if s.data.library().is_none() {
        return Ok(Answer::data(json!({
            "entries": [],
            "note": "No data library found (data folder of the app missing); the engine uses its built-in series.",
        })));
    }
    let out = s.call("library_list", &json!({}), None)?;
    let entries: Vec<Value> = out["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|e| match (&e["info"], &e["error"]) {
            (Value::Object(i), _) => json!({
                "file": e["file"],
                "reference": format!("library:{}", e["file"].as_str().unwrap_or_default()),
                "currency": i.get("currency"),
                "monthly": i.get("monthly"),
                "first": i.get("first"),
                "last": i.get("last"),
                "saron": i.get("hasSaron"),
                "avgStocks": i.get("avgStocks"),
                "avgBonds": i.get("avgBonds"),
                "avgInflation": i.get("avgInflation"),
            }),
            (_, err) => json!({ "file": e["file"], "error": err.get("text").unwrap_or(err) }),
        })
        .collect();
    Ok(Answer::data(json!({
        "entries": entries,
        "note": "Average returns per year as fractions (0.05 = 5 %).",
    })))
}

fn describe(s: &mut Session, args: &Value) -> Outcome {
    let section = arg(args, "section").and_then(engine::describe::Section::parse);
    let (plan, file) = match arg(args, "plan") {
        Some(id) => {
            let p = s.plan(id)?;
            (Some(s.with_settings(&p.plan)), p.file.clone())
        }
        None => (None, None),
    };
    let base = file
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let catalog = engine::api::describe(section, plan.as_ref(), &base, s.data.library());
    Ok(Answer::data(serde_json::to_value(catalog).map_err(|e| e.to_string())?))
}

// ---------------------------------------------------------------------------------------------------------------------
// Results

/// Swiss format of an amount: 1'234'567.
fn money(v: &Value) -> String {
    v.as_f64().map_or_else(|| "?".into(), crate::chf)
}

/// `2041-06` → `06.2041`.
fn month(v: &Value) -> String {
    match v.as_str().and_then(|s| s.split_once('-')) {
        Some((y, m)) => format!("{m}.{y}"),
        None => "?".into(),
    }
}

/// #209/#210: The result in words (English, without HTML) from the digest of the engine; the numbers come from it.
pub(crate) fn verdict_text(d: &Value, currency: &str) -> String {
    let end = d["endWealth"]
        .as_array()
        .and_then(|a| a.iter().find(|e| e["percent"] == 50));
    let mut out = match d["verdict"].as_str() {
        Some("lasts") => format!(
            "The wealth lasts until the end of the plan and then amounts to {currency} {} nominal ({currency} {} in today's purchasing power).",
            money(&end.map_or(Value::Null, |e| e["nominal"].clone())),
            money(&end.map_or(Value::Null, |e| e["real"].clone()))
        ),
        Some("depleted") => format!("The wealth is depleted in {}.", month(&d["depleted"])),
        Some(class) => {
            let rate = d["successRate"].as_f64().unwrap_or(0.0) * 100.0;
            let mut t = format!("In {rate:.1} % of the runs ({class}) the wealth lasts until the end of the plan.");
            if !d["depleted"].is_null() {
                t.push_str(&format!(" In the worst run the money is depleted in {}.", month(&d["depleted"])));
            }
            t
        }
        None => String::new(),
    };
    match d["next"].as_str() {
        Some("scenarios") => out.push_str(
            " Next step: variations show how strongly the result depends on spending, stock share or lump sum.",
        ),
        Some("optimize") => {
            out.push_str(" Next step: the optimization looks for a withdrawal strategy that fits the goals better.")
        }
        _ => {}
    }
    out
}

fn result_list(s: &Session) -> Answer {
    let list: Vec<Value> = s
        .results
        .iter()
        .map(|r| {
            let d = s
                .call("digest", &json!({ "result": r.result }), None)
                .unwrap_or(Value::Null);
            json!({
                "result": r.id,
                "plan": r.plan,
                "tool": r.kind,
                "runs": d["runs"],
                "successRate": d.get("successRate"),
                "verdict": d["verdict"],
                "secondsAgo": r.at.elapsed().as_secs(),
            })
        })
        .collect();
    Answer::results(json!({ "results": list }))
}

/// Run of a result for yearly values and events: the single run or a percentile of the batch.
fn pick_run(result: &Value, percentile: Option<i64>) -> Result<(&Value, Option<i64>), String> {
    if !result["single"].is_null() {
        return Ok((&result["single"], None));
    }
    let details = result["batch"]["details"].as_array().ok_or("The result has no run")?;
    let want = percentile.unwrap_or(50);
    details
        .iter()
        .find(|d| d[0].as_i64() == Some(want))
        .map(|d| (&d[1], Some(want)))
        .ok_or_else(|| {
            let have: Vec<String> = details.iter().map(|d| d[0].to_string()).collect();
            format!("No run at percentile {want}; percentiles: {}", have.join(", "))
        })
}

fn year_of(v: &Value) -> Option<i64> {
    v.as_str()?.split('-').next()?.parse().ok()
}

fn result_get(s: &mut Session, args: &Value) -> Outcome {
    let r = s.result(arg(args, "result").unwrap_or_default())?;
    let plan = s.plans.iter().find(|p| p.id == r.plan);
    let currency = plan
        .and_then(|p| p.plan["project"]["currency"].as_str())
        .unwrap_or("CHF")
        .to_string();
    let file = plan.and_then(|p| p.file.clone());
    let from = args.get("from").and_then(Value::as_i64).unwrap_or(i64::MIN);
    let to = args.get("to").and_then(Value::as_i64).unwrap_or(i64::MAX);
    let real = args.get("real").and_then(Value::as_bool).unwrap_or(false);
    let basis = if real {
        "real (today's purchasing power)"
    } else {
        "nominal"
    };
    let percentile = args.get("percentile").and_then(Value::as_i64);
    if let Some(e) = r.result["error"].get("text") {
        return Err(format!(
            "The result {} has no run: {}",
            r.id,
            e.as_str().unwrap_or_default()
        ));
    }
    let head = json!({ "result": r.id, "plan": r.plan, "currency": currency });
    let mut out = head.as_object().expect("object").clone();
    match arg(args, "view").unwrap_or("summary") {
        "summary" => {
            let d = s.call("digest", &json!({ "result": r.result }), file.as_deref())?;
            out.insert("verdict".into(), json!(verdict_text(&d, &currency)));
            out.insert("digest".into(), d);
            out.insert(
                "checks".into(),
                json!(r.result["issues"].as_array().map_or(0, Vec::len)),
            );
            out.insert(
                "variations".into(),
                json!(r.result["scenarios"]["results"].as_array().map_or(0, Vec::len)),
            );
            out.insert("elapsedMs".into(), r.result["elapsedMs"].clone());
        }
        "yearly" => {
            let (run, pct) = pick_run(&r.result, percentile)?;
            let y = s.call("yearly", &json!({ "run": run }), file.as_deref())?;
            let rows: Vec<Value> = y[if real { "real" } else { "nominal" }]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|row| row["year"].as_i64().is_some_and(|y| (from..=to).contains(&y)))
                .cloned()
                .collect();
            out.insert("percentile".into(), json!(pct));
            out.insert("basis".into(), json!(basis));
            out.insert("rows".into(), Value::Array(rows));
        }
        "events" => {
            let (run, pct) = pick_run(&r.result, percentile)?;
            let events: Vec<Value> = run["events"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|e| year_of(&e["month"]).is_some_and(|y| (from..=to).contains(&y)))
                .map(|e| json!({ "month": e["month"], "kind": e["kind"], "text": e["text"] }))
                .collect();
            out.insert("percentile".into(), json!(pct));
            out.insert("events".into(), Value::Array(events));
        }
        "bands" => {
            let b = &r.result["batch"]["bands"];
            if b.is_null() {
                return Err("The result is a single run without bands; use view yearly".into());
            }
            let b = if real { &b["real"] } else { b };
            let keys = ["min", "p1", "p5", "p25", "p50", "p75", "p95", "p99"];
            let months = b["months"].as_array().cloned().unwrap_or_default();
            // One row per calendar year: the last month of the year in the plan.
            let rows: Vec<Value> = (0..months.len())
                .filter(|&i| i + 1 == months.len() || year_of(&months[i]) != year_of(&months[i + 1]))
                .filter(|&i| year_of(&months[i]).is_some_and(|y| (from..=to).contains(&y)))
                .map(|i| {
                    let mut row = Map::new();
                    row.insert("month".into(), months[i].clone());
                    for k in keys {
                        row.insert(k.into(), b[k][i].clone());
                    }
                    Value::Object(row)
                })
                .collect();
            out.insert("basis".into(), json!(basis));
            out.insert(
                "meaning".into(),
                json!("Total wealth at the end of each year at the percentiles over all runs; min is the worst value of the month."),
            );
            out.insert("rows".into(), Value::Array(rows));
        }
        "variations" => {
            let sc = &r.result["scenarios"];
            if sc.is_null() {
                return Err("The result has no variations".into());
            }
            out.insert("params".into(), sc["params"].clone());
            out.insert("variations".into(), sc["results"].clone());
        }
        _ => unreachable!("checked against the enum"),
    }
    Ok(Answer::results(Value::Object(out)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn session() -> Session {
        let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/data");
        Session::new(
            crate::AppData {
                library: Some(lib),
                settings: Map::new(),
            },
            false,
        )
    }

    fn ok(o: Option<Outcome>) -> Value {
        match o.expect("known tool") {
            Ok(a) => a.data,
            Err(e) => panic!("{e}"),
        }
    }

    fn err(o: Option<Outcome>) -> String {
        match o.expect("known tool") {
            Ok(a) => panic!("expected an error, got {}", a.data),
            Err(e) => e,
        }
    }

    /// #210/AK2: The tool list holds exactly the reading tools of #210, each marked as read-only.
    #[test]
    fn issue_210_ak2_tool_list_reading_only() {
        let names: Vec<String> = list().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
        assert_eq!(
            names,
            [
                "describe",
                "plan_open",
                "plan_get",
                "plan_schema",
                "plan_validate",
                "result_list",
                "result_get",
                "library_list"
            ]
        );
        for t in list() {
            assert_eq!(t["annotations"]["readOnlyHint"], true, "{}", t["name"]);
            assert_eq!(t["inputSchema"]["type"], "object");
        }
    }

    /// #210/AK2: plan_open, plan_get, plan_validate on the example plan; the first opening brings the notes.
    #[test]
    fn issue_210_ak2_open_get_validate_the_example() {
        let mut s = session();
        let o = ok(call(&mut s, "plan_open", &json!({ "example": true })));
        assert_eq!(o["plan"], "p1");
        assert_eq!(o["persons"], 1);
        assert_eq!(o["properties"], 3);
        assert_eq!(o["period"]["start"], "2026-03");
        assert_eq!(o["name"], "Plan");
        assert!(o["privacy"].as_str().unwrap().contains("your AI"));
        let again = ok(call(&mut s, "plan_open", &json!({ "example": true })));
        assert_eq!(again["plan"], "p2");
        assert!(again.get("privacy").is_none());

        let get = ok(call(
            &mut s,
            "plan_get",
            &json!({ "plan": "p1", "section": "household" }),
        ));
        assert_eq!(get["value"]["persons"][0]["name"], "Person 1");
        let field = ok(call(
            &mut s,
            "plan_get",
            &json!({ "plan": "p1", "path": "household.persons.0.birth" }),
        ));
        assert_eq!(field["value"], "1970-06");
        let sections = ok(call(&mut s, "plan_get", &json!({ "plan": "p1" })));
        assert!(sections["sections"].as_array().unwrap().len() > 5);
        assert!(err(call(
            &mut s,
            "plan_get",
            &json!({ "plan": "p1", "path": "household.nobody" })
        ))
        .contains("does not exist"));
        assert!(err(call(&mut s, "plan_get", &json!({ "plan": "p9" }))).contains("open plans: p1, p2"));

        let v = ok(call(&mut s, "plan_validate", &json!({ "plan": "p1" })));
        assert_eq!(v["errors"], 0);
        assert!(v["messages"].is_array());
    }

    /// #210/AK2: Wrong calls come back as tool errors with a hint, not as protocol errors.
    #[test]
    fn issue_210_ak2_wrong_arguments_are_tool_errors() {
        let mut s = session();
        assert!(call(&mut s, "plan_preview", &json!({})).is_none());
        assert!(err(call(&mut s, "plan_open", &json!({}))).contains("example: true"));
        assert!(err(call(&mut s, "plan_open", &json!({ "path": "/etc/passwd" }))).contains(".vplan"));
        assert!(err(call(&mut s, "plan_open", &json!({ "pfad": "x" }))).contains("Unknown argument pfad"));
        assert!(err(call(&mut s, "plan_get", &json!({}))).contains("Argument plan missing"));
        assert!(err(call(&mut s, "plan_get", &json!({ "plan": 1 }))).contains("type string"));
        assert!(err(call(&mut s, "result_get", &json!({ "result": "r1", "view": "all" }))).contains("not one of"));
        assert!(err(call(&mut s, "result_get", &json!({ "result": "r1" }))).contains("no results yet"));
        let missing = err(call(&mut s, "plan_open", &json!({ "path": "/nowhere/x.vplan" })));
        assert!(missing.contains("x.vplan"), "{missing}");
    }

    /// #210/AK2: plan_schema gives the sections, a section with its definitions, a single definition.
    // slow: reads the whole schema once per section of the plan
    #[test]
    fn issue_210_ak2_schema_parts() {
        let all = ok(call(&mut session(), "plan_schema", &json!({})));
        assert!(all["sections"]["household"].is_string() || all["sections"]["household"].is_null());
        assert!(all["sections"].as_object().unwrap().contains_key("realestate"));
        let h = ok(call(&mut session(), "plan_schema", &json!({ "section": "household" })));
        assert!(h["definitions"]["Person"].is_object(), "{h}");
        let p = ok(call(&mut session(), "plan_schema", &json!({ "definition": "Person" })));
        assert!(p["schema"]["properties"]["birth"].is_object());
        assert!(err(call(&mut session(), "plan_schema", &json!({ "section": "x" }))).contains("Unknown section"));
        assert!(err(call(&mut session(), "plan_schema", &json!({ "definition": "x" }))).contains("Unknown definition"));
        for sec in schema()["properties"].as_object().unwrap().keys() {
            let a = ok(call(&mut session(), "plan_schema", &json!({ "section": sec })));
            assert!(a.to_string().chars().count() < super::super::MAX_CHARS, "{sec}");
        }
    }

    /// #210/AK2, AK4: library_list shows the files of the data library with references; describe answers per section.
    #[test]
    fn issue_210_ak2_library_and_describe() {
        let mut s = session();
        let l = ok(call(&mut s, "library_list", &json!({})));
        let entries = l["entries"].as_array().unwrap();
        assert!(!entries.is_empty());
        assert!(entries[0]["reference"].as_str().unwrap().starts_with("library:"));
        let mut none = Session::new(
            crate::AppData {
                library: None,
                settings: Map::new(),
            },
            false,
        );
        assert_eq!(ok(call(&mut none, "library_list", &json!({})))["entries"], json!([]));
        let d = ok(call(&mut s, "describe", &json!({ "section": "metrics" })));
        assert!(d["metrics"].is_object(), "{d}");
        assert!(d.get("fields").is_none_or(Value::is_null));
    }

    /// The example plan computed as the app does (dispatch `run_simulation`), kept as a result of the session.
    fn computed(s: &mut Session) -> (String, Value) {
        ok(call(s, "plan_open", &json!({ "example": true })));
        let plan = s.with_settings(&s.plans[0].plan);
        let run = s.call("run_simulation", &json!({ "plan": plan }), None).unwrap();
        (s.add_result("p1", "simulation_run", run.clone()), run)
    }

    /// #210/AK3: The key figures of result_get are those of the app: digest and yearly values come from the same
    /// command of the dispatcher that the desktop app answers (contract test #142), unchanged.
    // slow: computes the example plan as the app does
    #[test]
    fn issue_210_ak3_key_figures_as_in_the_app() {
        let mut s = session();
        let (id, run) = computed(&mut s);
        let lib = s.library().to_path_buf();
        let app = |cmd: &str, args: Value| engine::dispatch::call(cmd, &args, &lib, Path::new("/")).unwrap();
        let sum = ok(call(&mut s, "result_get", &json!({ "result": id })));
        assert_eq!(sum["digest"], app("digest", json!({ "result": run })));
        assert!(sum["verdict"].as_str().unwrap().starts_with("The wealth"));
        let run_value = if run["single"].is_null() {
            run["batch"]["details"][3][1].clone()
        } else {
            run["single"].clone()
        };
        let y = ok(call(
            &mut s,
            "result_get",
            &json!({ "result": id, "view": "yearly", "real": true }),
        ));
        assert_eq!(y["rows"], app("yearly", json!({ "run": run_value }))["real"]);
        let part = ok(call(
            &mut s,
            "result_get",
            &json!({ "result": id, "view": "yearly", "from": 2030, "to": 2031 }),
        ));
        let years: Vec<i64> = part["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["year"].as_i64().unwrap())
            .collect();
        assert_eq!(years, [2030, 2031]);
        let ev = ok(call(&mut s, "result_get", &json!({ "result": id, "view": "events" })));
        assert!(!ev["events"].as_array().unwrap().is_empty());
        let list = ok(call(&mut s, "result_list", &json!({})));
        assert_eq!(list["results"][0]["result"], id);
    }

    /// #210/AK3: Only reading the file, the schema and the catalog bypass the dispatcher; every computation goes through it.
    #[test]
    fn issue_210_ak3_computations_only_through_dispatch() {
        let mut uses = BTreeMap::new();
        for src in [
            include_str!("tools.rs"),
            include_str!("session.rs"),
            include_str!("mod.rs"),
        ] {
            let code: String = src
                .split("#[cfg(test)]")
                .next()
                .unwrap()
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            for (i, _) in code.match_indices("engine::") {
                let call = code[i..].split(['(', ' ', ';', ')']).next().unwrap();
                *uses.entry(call.to_string()).or_insert(0) += 1;
            }
        }
        let allowed = [
            "engine::api::describe",
            "engine::api::read_plan",
            "engine::appdata::apply",
            "engine::describe::Section::ALL.iter",
            "engine::describe::Section::parse",
            "engine::dispatch::call",
            "engine::model::Plan>",
            "engine::resolve::Resolver::new",
            "engine::schema_json",
        ];
        for u in uses.keys() {
            assert!(allowed.contains(&u.as_str()), "{u} bypasses engine::dispatch::call");
        }
    }

    /// #209/#210: The result in words for the five verdicts, with the numbers of the digest in Swiss format.
    #[test]
    fn issue_210_ak3_verdict_in_words() {
        let lasts = json!({ "verdict": "lasts", "next": "scenarios", "endWealth": [{ "percent": 50, "nominal": 1234567.4, "real": 800000.0 }] });
        assert_eq!(
            verdict_text(&lasts, "CHF"),
            "The wealth lasts until the end of the plan and then amounts to CHF 1'234'567 nominal (CHF 800'000 in today's \
             purchasing power). Next step: variations show how strongly the result depends on spending, stock share or lump sum."
        );
        let dep = json!({ "verdict": "depleted", "depleted": "2041-06", "next": "optimize" });
        assert!(verdict_text(&dep, "CHF").starts_with("The wealth is depleted in 06.2041. Next step: the optimization"));
        let low = json!({ "verdict": "low", "successRate": 0.7234, "depleted": "2039-01", "next": "optimize" });
        assert!(verdict_text(&low, "CHF").starts_with(
            "In 72.3 % of the runs (low) the wealth lasts until the end of the plan. In the worst run the money is depleted in 01.2039."
        ));
        let high = json!({ "verdict": "high", "successRate": 0.99, "depleted": null });
        assert_eq!(
            verdict_text(&high, "CHF"),
            "In 99.0 % of the runs (high) the wealth lasts until the end of the plan."
        );
    }
}
