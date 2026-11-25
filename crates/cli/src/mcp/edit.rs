//! #212: Changing and saving over the MCP server (concept `docs/VivaPlan in Claude (MCP).md`, sections 5 and 7):
//! `plan_set`, `plan_changes`, `plan_undo` and `plan_apply` change only the working copy in memory, step by step and
//! undoable; `plan_save` is the only tool that writes a file and keeps the rules 1–6 of section 7 itself, whatever the
//! AI or its program does.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::session::{Session, Stamp};
use super::tools::{arg, tool, Answer, Outcome};

/// Suffix of the new file of rule 1: `<name> (AI).vplan`, `<name> (AI 2).vplan`, …
const NEW_FILE_TAG: &str = "AI";

/// Changes shown in the question before replacing a file.
const ASKED_CHANGES: usize = 30;

/// Hints of the protocol: the tool changes only the working copy in memory.
fn in_memory(mut t: Value) -> Value {
    t["annotations"] =
        json!({ "readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false });
    t
}

/// The changing tools of `tools/list`.
pub(crate) fn list() -> Vec<Value> {
    let plan = json!({ "type": "string", "description": "Id of the open plan (p1, …)." });
    let mut save = tool(
        "plan_save",
        "Save a plan",
        "Saves the working copy as a project file (.vplan). Without path and overwrite a new file next to the opened one \
         (<name> (AI).vplan, with a number if needed); an existing file is never touched. path saves to another new file \
         in the folder of the plan or a folder released with --allow-dir. overwrite: true replaces an existing file, but \
         only after the program asked the user and showed the changes; a backup <file>.bak is kept, a file changed since \
         opening and old .rplan files are never replaced. Answers the file name written.",
        json!({
            "plan": plan,
            "path": { "type": "string", "description": "File to write (.vplan); relative to the folder of the plan." },
            "overwrite": { "type": "boolean", "description": "Replace an existing file (asks the user first)." },
        }),
        &["plan"],
    );
    save["annotations"] =
        json!({ "readOnlyHint": false, "destructiveHint": true, "idempotentHint": false, "openWorldHint": false });
    vec![
        in_memory(tool(
            "plan_set",
            "Change values",
            "Changes values of the working copy by field path as in the check messages (wealth.stockShare, \
             household.persons.0.birth); a step of a course over time with path@<milestone id>, path@age:<years> or \
             path@YYYY-MM. All changes of a call are one step: if one path or type is wrong, none is made. Answers every \
             changed field with old and new value and the check messages afterwards. Nothing is written to a file; \
             plan_undo takes the step back, plan_save saves.",
            json!({
                "plan": plan,
                "changes": {
                    "type": "array",
                    "description": "List of {path, value}; value as JSON (number, text, true/false, object).",
                    "items": {
                        "type": "object",
                        "properties": { "path": { "type": "string" }, "value": {} },
                        "required": ["path", "value"],
                    },
                },
            }),
            &["plan", "changes"],
        )),
        in_memory(tool(
            "plan_changes",
            "Changes against the file",
            "Lists all differences of the working copy against the file as it was opened or last saved: field path, \
             value in the file, current value; with the number of steps plan_undo can take back.",
            json!({ "plan": plan }),
            &["plan"],
        )),
        in_memory(tool(
            "plan_undo",
            "Undo",
            "Takes back the last step of plan_set or plan_apply, with all: true all steps. Answers the values restored and \
             the steps left.",
            json!({
                "plan": plan,
                "all": { "type": "boolean", "description": "Take back all steps." },
            }),
            &["plan"],
        )),
        in_memory(tool(
            "plan_apply",
            "Take over a variant",
            "Takes over a variant of an evaluate result into the plan it was computed from: the fixed values of the \
             experiment (set) and the values of the variant with the rank given (1 = best), as one step like plan_set. \
             The variations of the plan itself stay as they are.",
            json!({
                "result": { "type": "string", "description": "Id of an evaluate result (r1, …)." },
                "rank": { "type": "integer", "description": "Rank of the variant in the result; default 1." },
            }),
            &["result"],
        )),
        save,
    ]
}

/// Answers a call of a changing tool (all but `plan_save`, which the server handles with its question).
pub(crate) fn call(s: &mut Session, name: &str, args: &Value) -> Outcome {
    match name {
        "plan_set" => plan_set(s, args),
        "plan_changes" => plan_changes(s, args),
        "plan_undo" => plan_undo(s, args),
        "plan_apply" => plan_apply(s, args),
        _ => unreachable!("only changing tools are called here"),
    }
}

/// Check messages of the plan as the app computes it: counts, errors and warnings with their field.
fn checks(s: &Session, plan: &Value, file: Option<&Path>) -> Result<Value, String> {
    let issues = s.call("validate", &json!({ "plan": s.with_settings(plan) }), file)?;
    let list = issues.as_array().cloned().unwrap_or_default();
    let count = |sev: &str| list.iter().filter(|i| i["severity"] == sev).count();
    let shown: Vec<Value> = list
        .iter()
        .filter(|i| i["severity"] != "info")
        .map(|i| json!({ "severity": i["severity"], "field": i["field"], "section": i["section"], "message": i["message"] }))
        .collect();
    Ok(json!({
        "errors": count("error"),
        "warnings": count("warning"),
        "infos": count("info"),
        "messages": shown,
    }))
}

fn changes(a: &Value, b: &Value) -> Vec<Value> {
    engine::api::diff_plans(a, b)
        .into_iter()
        .map(|c| json!({ "path": c.path, "old": c.old, "new": c.new }))
        .collect()
}

/// One step: sets the changes `path[@time]=value` in the working copy of `id`, all or none.
fn step(s: &mut Session, id: &str, assignments: &[String]) -> Outcome {
    let before = s.plan(id)?.plan.clone();
    let after = engine::api::set_fields(&before, assignments).map_err(|errors| {
        let lines: Vec<String> = errors.iter().map(|m| m.text().to_string()).collect();
        format!("No change made:\n{}", lines.join("\n"))
    })?;
    let changed = changes(&before, &after);
    let open = s.plan_mut(id)?;
    if !changed.is_empty() {
        open.history.push(before);
        open.plan = after;
    }
    let (plan, file, steps) = (open.plan.clone(), open.file.clone(), open.history.len());
    Ok(Answer::data(json!({
        "plan": id,
        "changes": changed,
        "checks": checks(s, &plan, file.as_deref())?,
        "steps": steps,
        "next": "plan_undo takes the step back, plan_changes lists all changes, plan_save saves; nothing is written yet",
    })))
}

fn plan_set(s: &mut Session, args: &Value) -> Outcome {
    let id = arg(args, "plan").unwrap_or_default();
    s.plan(id)?;
    let list = args["changes"]
        .as_array()
        .ok_or("Argument changes must be of type array")?;
    if list.is_empty() {
        return Err("changes is empty; give a list of {path, value}".into());
    }
    let mut assignments = vec![];
    for (i, c) in list.iter().enumerate() {
        let (Some(path), Some(value)) = (c.get("path").and_then(Value::as_str), c.get("value")) else {
            return Err(format!("changes.{i} must be an object with path (text) and value"));
        };
        assignments.push(format!("{path}={value}"));
    }
    step(s, id, &assignments)
}

fn plan_changes(s: &mut Session, args: &Value) -> Outcome {
    let open = s.plan(arg(args, "plan").unwrap_or_default())?;
    let list: Vec<Value> = engine::api::diff_plans(&open.original, &open.plan)
        .into_iter()
        .map(|c| json!({ "path": c.path, "file": c.old, "current": c.new }))
        .collect();
    Ok(Answer::data(json!({
        "plan": open.id,
        "file": open.file_name(),
        "count": list.len(),
        "changes": list,
        "steps": open.history.len(),
    })))
}

fn plan_undo(s: &mut Session, args: &Value) -> Outcome {
    let all = args.get("all").and_then(Value::as_bool).unwrap_or(false);
    let open = s.plan_mut(arg(args, "plan").unwrap_or_default())?;
    if open.history.is_empty() {
        return Err(format!("The plan {} has no step to undo", open.id));
    }
    let restored = if all {
        let first = open.history[0].clone();
        open.history.clear();
        first
    } else {
        open.history.pop().expect("not empty")
    };
    let undone = changes(&open.plan, &restored);
    open.plan = restored;
    Ok(Answer::data(json!({
        "plan": open.id,
        "undone": undone,
        "steps": open.history.len(),
        "changesAgainstFile": engine::api::diff_plans(&open.original, &open.plan).len(),
    })))
}

fn plan_apply(s: &mut Session, args: &Value) -> Outcome {
    let rank = args.get("rank").and_then(Value::as_i64).unwrap_or(1);
    let r = s.result(arg(args, "result").unwrap_or_default())?;
    if r.kind != "evaluate" {
        return Err(format!(
            "The result {} is a {}; plan_apply takes over variants of evaluate results",
            r.id, r.kind
        ));
    }
    let found = &r.result["result"];
    let variants = found["ranking"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(found["listed"].as_array().into_iter().flatten());
    let Some(variant) = variants.into_iter().find(|v| v["rank"].as_i64() == Some(rank)) else {
        return Err(format!("The result {} has no variant with rank {rank}", r.id));
    };
    let mut assignments: Vec<String> = r.result["set"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(p, v)| format!("{p}={v}"))
        .collect();
    for setting in variant["settings"].as_array().into_iter().flatten() {
        assignments.push(format!(
            "{}={}",
            setting["path"].as_str().unwrap_or_default(),
            setting["value"]
        ));
    }
    let (plan, result) = (r.plan.clone(), r.id.clone());
    let mut answer = step(s, &plan, &assignments)?;
    answer.data["from"] = json!({ "result": result, "rank": rank });
    Ok(answer)
}

// ---------------------------------------------------------------------------------------------------------------------
// Saving

/// A file to replace, waiting for the answer of the user (rule 2).
pub(crate) struct Pending {
    pub(crate) plan: String,
    target: PathBuf,
    /// The working copy at the time of the question: written as shown.
    content: Value,
    /// The question with the list of changes.
    pub(crate) message: String,
}

/// What `plan_save` does next.
pub(crate) enum Save {
    Done(Outcome),
    /// Ask the user before replacing the file.
    Ask(Pending),
}

/// Rule 1: `<name> (AI).vplan`, `<name> (AI 2).vplan`, … next to the opened file: the first that does not exist.
fn new_file(file: &Path) -> PathBuf {
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    (1..)
        .map(|n| {
            let tag = if n == 1 {
                NEW_FILE_TAG.to_string()
            } else {
                format!("{NEW_FILE_TAG} {n}")
            };
            file.with_file_name(format!("{stem} ({tag}).vplan"))
        })
        .find(|p| std::fs::symlink_metadata(p).is_err())
        .expect("a free name exists")
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn extension(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Rule 6: the folder of the target, if it is the folder of the opened plan or a folder released with `--allow-dir`.
fn allowed_folder(s: &Session, file: Option<&Path>, target: &Path) -> Result<PathBuf, String> {
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let folder =
        std::fs::canonicalize(parent).map_err(|_| format!("The folder of {} does not exist", name_of(target)))?;
    let own = file.and_then(Path::parent).and_then(|d| std::fs::canonicalize(d).ok());
    // #263: a released folder includes its subfolders, as for reading.
    if own.iter().any(|d| *d == folder) || s.allowed.iter().any(|d| folder.starts_with(d)) {
        Ok(folder)
    } else {
        Err(format!(
            "{} is outside the folder of the plan; the server writes only into the folder of the opened plan or a \
             folder released with --allow-dir (rule 6)",
            name_of(target)
        ))
    }
}

/// Rule 4: the opened file is as it was when it was opened or last saved.
fn unchanged(file: &Path, stamp: Option<&Stamp>) -> Result<(), String> {
    if Stamp::of(file).as_ref() == stamp {
        return Ok(());
    }
    Err(format!(
        "{} was changed since it was opened (or removed); it is not replaced (rule 4). Save without overwrite to a new \
         file, or open it again with plan_open and repeat the changes",
        name_of(file)
    ))
}

/// `plan_save` up to the question of rule 2; `can_ask`: the program of the AI can ask the user (elicitation).
pub(crate) fn save(s: &mut Session, args: &Value, can_ask: bool) -> Save {
    match prepare_save(s, args, can_ask) {
        Ok(p) if p.message.is_empty() => Save::Done(write(s, p)),
        Ok(p) => Save::Ask(p),
        Err(e) => Save::Done(Err(e)),
    }
}

/// What to write; with a question (`message`) if an existing file is to be replaced.
fn prepare_save(s: &Session, args: &Value, can_ask: bool) -> Result<Pending, String> {
    let open = s.plan(arg(args, "plan").unwrap_or_default())?;
    let overwrite = args.get("overwrite").and_then(Value::as_bool).unwrap_or(false);
    let file = open.file.as_deref();
    let no_file = || {
        format!(
            "The plan {} has no file (example plan); give a path in a folder released with --allow-dir",
            open.id
        )
    };
    let target = match (arg(args, "path"), overwrite) {
        (Some(p), _) => {
            let p = PathBuf::from(p);
            let base = file.and_then(Path::parent).map(Path::to_path_buf);
            let p = match (p.is_absolute(), base) {
                (true, _) => p,
                (false, Some(b)) => b.join(p),
                (false, None) => std::env::current_dir().map_err(|e| e.to_string())?.join(p),
            };
            match extension(&p).as_str() {
                "vplan" => p,
                "rplan" => return Err("Old .rplan files are never written; plans are saved as .vplan (rule 5)".into()),
                _ => return Err("The path must end in .vplan".into()),
            }
        }
        (None, true) => {
            let f = file.ok_or_else(no_file)?;
            if extension(f) == "rplan" {
                return Err(format!(
                    "{} is an old .rplan file and is never replaced (rule 5); save without overwrite to a new .vplan file",
                    name_of(f)
                ));
            }
            f.to_path_buf()
        }
        (None, false) => new_file(file.ok_or_else(no_file)?),
    };
    let folder = allowed_folder(s, file, &target)?;
    let target = folder.join(target.file_name().ok_or("The path has no file name")?);
    let meta = std::fs::symlink_metadata(&target).ok();
    if meta.as_ref().is_some_and(|m| m.file_type().is_symlink()) {
        return Err(format!(
            "{} is a link; the server does not write through links",
            name_of(&target)
        ));
    }
    if is_opened(file, &target) {
        unchanged(&target, open.stamp.as_ref())?;
    }
    let pending = Pending {
        plan: open.id.clone(),
        target: target.clone(),
        content: open.plan.clone(),
        message: String::new(),
    };
    if meta.is_none() {
        return Ok(pending);
    }
    if !overwrite {
        return Err(format!(
            "{} exists and is not touched (rule 1); save without path to a new file, or with overwrite: true to replace \
             it after asking the user",
            name_of(&target)
        ));
    }
    if !can_ask {
        return Err(format!(
            "{} exists. The program of your AI cannot ask the user (no elicitation), so the server does not replace \
             files (rule 2). Save without path and overwrite: a new file next to the plan (rule 1)",
            name_of(&target)
        ));
    }
    let base = if is_opened(file, &target) {
        open.original.clone()
    } else {
        engine::api::read_plan(&target).unwrap_or(Value::Null)
    };
    let list = engine::api::diff_plans(&base, &open.plan);
    let mut lines: Vec<String> = list
        .iter()
        .take(ASKED_CHANGES)
        .map(|c| format!("- {}: {} → {}", c.path, c.old, c.new))
        .collect();
    if list.len() > ASKED_CHANGES {
        lines.push(format!("- … and {} more", list.len() - ASKED_CHANGES));
    }
    let message = format!(
        "Replace {} with the working copy {} of VivaPlan? A backup {}.bak is kept. Changes ({}):\n{}",
        name_of(&target),
        open.id,
        name_of(&target),
        list.len(),
        lines.join("\n")
    );
    Ok(Pending {
        message: s.redactor().text(&message),
        ..pending
    })
}

/// The target is the opened file.
fn is_opened(file: Option<&Path>, target: &Path) -> bool {
    file.and_then(|f| Some(std::fs::canonicalize(f.parent()?).ok()?.join(f.file_name()?)))
        .is_some_and(|f| f == target)
}

/// Writes the plan (rules 3 and 4): backup `<file>.bak` before replacing, through a temporary file renamed afterwards.
pub(crate) fn write(s: &mut Session, p: Pending) -> Outcome {
    let open = s.plan(&p.plan)?;
    let opened = is_opened(open.file.as_deref(), &p.target);
    if opened {
        // Again: the file can have changed while the user was asked.
        unchanged(&p.target, open.stamp.as_ref())?;
    }
    let exists = std::fs::symlink_metadata(&p.target).is_ok();
    let backup = exists.then(|| {
        let mut name = p.target.file_name().unwrap_or_default().to_os_string();
        name.push(".bak");
        p.target.with_file_name(name)
    });
    if let Some(b) = &backup {
        std::fs::copy(&p.target, b).map_err(|e| format!("Backup {} not written: {e}", name_of(b)))?;
    }
    engine::api::write_plan(&p.target, &p.content).map_err(|e| e.message().text().to_string())?;
    let open = s.plan_mut(&p.plan)?;
    let changed = engine::api::diff_plans(&open.original, &p.content).len();
    if opened {
        open.original = p.content.clone();
        open.stamp = Stamp::of(&p.target);
    }
    let file = open.file_name();
    let next = if opened {
        "plan_changes now lists the changes against the saved file".to_string()
    } else {
        format!("The working copy stays linked to {file}; open the new file with plan_open to go on with it")
    };
    Ok(Answer::data(json!({
        "plan": p.plan,
        "saved": name_of(&p.target),
        "backup": backup.as_deref().map(name_of),
        "changesAgainstOpenedFile": changed,
        "next": next,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

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

    /// A session with the example plan saved as `dir/<name>` and opened as p1.
    fn opened(dir: &Path, name: &str) -> (Session, PathBuf) {
        let file = dir.join(name);
        std::fs::write(&file, engine::api::example_plan().to_string()).unwrap();
        let mut s = session();
        let r = super::super::tools::call(&mut s, "plan_open", &json!({ "path": file.display().to_string() }));
        assert!(r.unwrap().is_ok());
        (s, file)
    }

    fn ok(o: Outcome) -> Value {
        match o {
            Ok(a) => a.data,
            Err(e) => panic!("{e}"),
        }
    }

    fn set(s: &mut Session, changes: Value) -> Outcome {
        call(s, "plan_set", &json!({ "plan": "p1", "changes": changes }))
    }

    fn save_done(s: &mut Session, args: Value, can_ask: bool) -> Outcome {
        match save(s, &args, can_ask) {
            Save::Done(o) => o,
            Save::Ask(p) => panic!("asks: {}", p.message),
        }
    }

    fn files(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        v.sort();
        v
    }

    /// #212/AK1: plan_set changes by path with old and new value and the checks; a wrong path changes nothing.
    #[test]
    fn issue_212_ak1_plan_set_all_or_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, _) = opened(tmp.path(), "plan.vplan");
        let before = s.plan("p1").unwrap().plan.clone();
        let a = ok(set(
            &mut s,
            json!([{ "path": "wealth.stockShare", "value": 0.3 }, { "path": "withdrawal.strategy", "value": "vpw" }]),
        ));
        assert_eq!(a["steps"], 1);
        let paths: Vec<&str> = a["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["path"].as_str().unwrap())
            .collect();
        assert_eq!(paths, ["wealth.stockShare", "withdrawal.strategy"]);
        assert_eq!(a["changes"][0]["old"], before["wealth"]["stockShare"]);
        assert_eq!(a["changes"][0]["new"], json!(0.3));
        assert!(a["checks"]["errors"].is_u64());
        let e = set(
            &mut s,
            json!([{ "path": "wealth.stockShare", "value": 0.9 }, { "path": "wealth.nope", "value": 1 }]),
        )
        .err()
        .unwrap();
        assert!(e.starts_with("No change made"), "{e}");
        assert_eq!(s.plan("p1").unwrap().plan["wealth"]["stockShare"], json!(0.3));
        assert_eq!(s.plan("p1").unwrap().history.len(), 1);
        // The same value again is no step.
        let same = ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.3 }])));
        assert_eq!(same["changes"], json!([]));
        assert_eq!(same["steps"], 1);
        for bad in [
            json!([]),
            json!([{ "value": 1 }]),
            json!([{ "path": "wealth.stockShare" }]),
        ] {
            assert!(set(&mut s, bad.clone()).is_err(), "{bad}");
        }
    }

    /// #212/AK1: plan_changes against the file, plan_undo step by step and all.
    #[test]
    fn issue_212_ak1_changes_and_undo() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, _) = opened(tmp.path(), "plan.vplan");
        let changes = |s: &mut Session| ok(call(s, "plan_changes", &json!({ "plan": "p1" })));
        assert_eq!(changes(&mut s)["count"], 0);
        ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.3 }])));
        ok(set(
            &mut s,
            json!([{ "path": "wealth.stockShare", "value": 0.4 }, { "path": "withdrawal.strategy", "value": "vpw" }]),
        ));
        let c = changes(&mut s);
        assert_eq!(c["count"], 2);
        assert_eq!(c["steps"], 2);
        assert_eq!(c["file"], "plan.vplan");
        assert_eq!(c["changes"][0]["current"], json!(0.4));
        let u = ok(call(&mut s, "plan_undo", &json!({ "plan": "p1" })));
        assert_eq!(u["steps"], 1);
        assert_eq!(u["undone"].as_array().unwrap().len(), 2);
        assert_eq!(u["changesAgainstFile"], 1);
        assert_eq!(s.plan("p1").unwrap().plan["wealth"]["stockShare"], json!(0.3));
        ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.5 }])));
        let all = ok(call(&mut s, "plan_undo", &json!({ "plan": "p1", "all": true })));
        assert_eq!(all["steps"], 0);
        assert_eq!(all["changesAgainstFile"], 0);
        assert_eq!(changes(&mut s)["count"], 0);
        assert!(call(&mut s, "plan_undo", &json!({ "plan": "p1" })).is_err());
    }

    /// #212/AK1: plan_apply takes over the fixed values and a variant of an evaluate result as one step.
    #[test]
    fn issue_212_ak1_plan_apply() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, _) = opened(tmp.path(), "plan.vplan");
        let result = json!({
            "status": "done",
            "set": { "withdrawal.strategy": "vpw" },
            "result": { "ranking": [
                { "rank": 1, "settings": [{ "path": "wealth.stockShare", "value": 0.6 }] },
                { "rank": 2, "settings": [{ "path": "wealth.stockShare", "value": 0.4 }] },
            ] },
        });
        let id = s.add_result("p1", "evaluate", result);
        let a = ok(call(&mut s, "plan_apply", &json!({ "result": id, "rank": 2 })));
        assert_eq!(a["from"]["rank"], 2);
        assert_eq!(a["steps"], 1);
        let p = &s.plan("p1").unwrap().plan;
        assert_eq!(p["wealth"]["stockShare"], json!(0.4));
        assert_eq!(p["withdrawal"]["strategy"], "vpw");
        ok(call(&mut s, "plan_apply", &json!({ "result": id })));
        assert_eq!(s.plan("p1").unwrap().plan["wealth"]["stockShare"], json!(0.6));
        assert!(call(&mut s, "plan_apply", &json!({ "result": id, "rank": 3 })).is_err());
        let run = s.add_result("p1", "simulation_run", json!({}));
        assert!(call(&mut s, "plan_apply", &json!({ "result": run }))
            .unwrap_err()
            .contains("evaluate"));
    }

    /// #212/AK2, rule 1: without path and overwrite a new file next to the opened one; nothing existing is touched.
    #[test]
    fn issue_212_ak2_rule1_new_file_next_to_the_plan() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, file) = opened(tmp.path(), "plan.vplan");
        let original = std::fs::read(&file).unwrap();
        ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.3 }])));
        let a = ok(save_done(&mut s, json!({ "plan": "p1" }), true));
        assert_eq!(a["saved"], "plan (AI).vplan");
        assert_eq!(a["backup"], Value::Null);
        let b = ok(save_done(&mut s, json!({ "plan": "p1" }), true));
        assert_eq!(b["saved"], "plan (AI 2).vplan");
        assert_eq!(std::fs::read(&file).unwrap(), original);
        let written = engine::api::read_plan(&tmp.path().join("plan (AI).vplan")).unwrap();
        assert_eq!(written["wealth"]["stockShare"], json!(0.3));
        assert_eq!(
            files(tmp.path()),
            ["plan (AI 2).vplan", "plan (AI).vplan", "plan.vplan"]
        );
        // An existing file named by path is not touched without overwrite.
        let e = save_done(&mut s, json!({ "plan": "p1", "path": "plan (AI).vplan" }), true).unwrap_err();
        assert!(e.contains("rule 1"), "{e}");
        let c = ok(save_done(&mut s, json!({ "plan": "p1", "path": "other.vplan" }), true));
        assert_eq!(c["saved"], "other.vplan");
    }

    /// #212/AK2, rule 2: an existing file only with overwrite: true and after the question with the changes; without a
    /// way to ask the server refuses and names rule 1.
    #[test]
    fn issue_212_ak2_rule2_overwrite_only_after_asking() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, file) = opened(tmp.path(), "plan.vplan");
        let original = std::fs::read(&file).unwrap();
        ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.3 }])));
        let e = save_done(&mut s, json!({ "plan": "p1", "overwrite": true }), false).unwrap_err();
        assert!(e.contains("cannot ask") && e.contains("rule 1"), "{e}");
        assert_eq!(std::fs::read(&file).unwrap(), original);
        let Save::Ask(p) = save(&mut s, &json!({ "plan": "p1", "overwrite": true }), true) else {
            panic!("no question");
        };
        assert!(
            p.message.contains("plan.vplan") && p.message.contains("wealth.stockShare"),
            "{}",
            p.message
        );
        assert!(!p.message.contains(tmp.path().to_str().unwrap()));
        assert_eq!(
            std::fs::read(&file).unwrap(),
            original,
            "nothing written before the answer"
        );
        ok(write(&mut s, p));
        assert_eq!(
            engine::api::read_plan(&file).unwrap()["wealth"]["stockShare"],
            json!(0.3)
        );
        // overwrite on a file that does not exist yet needs no question.
        ok(save_done(
            &mut s,
            json!({ "plan": "p1", "path": "new.vplan", "overwrite": true }),
            false,
        ));
    }

    /// #212/AK2, rule 3: a backup <file>.bak before replacing; written through a temporary file (no rest left).
    #[test]
    fn issue_212_ak2_rule3_backup_and_temporary_file() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, file) = opened(tmp.path(), "plan.vplan");
        let original = std::fs::read(&file).unwrap();
        ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.3 }])));
        let Save::Ask(p) = save(&mut s, &json!({ "plan": "p1", "overwrite": true }), true) else {
            panic!("no question");
        };
        let a = ok(write(&mut s, p));
        assert_eq!(a["saved"], "plan.vplan");
        assert_eq!(a["backup"], "plan.vplan.bak");
        assert_eq!(std::fs::read(tmp.path().join("plan.vplan.bak")).unwrap(), original);
        assert_eq!(files(tmp.path()), ["plan.vplan", "plan.vplan.bak"]);
        // After saving, the changes count against the saved file.
        assert_eq!(ok(call(&mut s, "plan_changes", &json!({ "plan": "p1" })))["count"], 0);
        // The writing goes through engine::api::write_plan, which writes a temporary file and renames it (#136).
        let code = include_str!("edit.rs").split("#[cfg(test)]").next().unwrap();
        assert!(code.contains("engine::api::write_plan(&p.target"));
        assert!(!code.contains("std::fs::write("));
    }

    /// #212/AK2, rule 4: a file changed since opening (time or content) is not replaced, also not after the question.
    #[test]
    fn issue_212_ak2_rule4_file_changed_since_opening() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, file) = opened(tmp.path(), "plan.vplan");
        ok(set(&mut s, json!([{ "path": "wealth.stockShare", "value": 0.3 }])));
        let mut other = engine::api::example_plan();
        other["wealth"]["stockShare"] = json!(0.9);
        std::fs::write(&file, other.to_string()).unwrap();
        let e = save_done(&mut s, json!({ "plan": "p1", "overwrite": true }), true).unwrap_err();
        assert!(e.contains("rule 4"), "{e}");
        // Changed while the user is asked.
        let (mut s, file) = opened(tmp.path(), "second.vplan");
        let Save::Ask(p) = save(&mut s, &json!({ "plan": "p1", "overwrite": true }), true) else {
            panic!("no question");
        };
        std::fs::write(&file, other.to_string()).unwrap();
        assert!(write(&mut s, p).unwrap_err().contains("rule 4"));
        assert_eq!(
            engine::api::read_plan(&file).unwrap()["wealth"]["stockShare"],
            json!(0.9)
        );
        assert!(!tmp.path().join("second.vplan.bak").exists());
        // Only the time changed: also refused.
        let (mut s, file) = opened(tmp.path(), "third.vplan");
        let f = std::fs::File::options().write(true).open(&file).unwrap();
        f.set_modified(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let Save::Done(Err(e)) = save(&mut s, &json!({ "plan": "p1", "overwrite": true }), true) else {
            panic!("not refused");
        };
        assert!(e.contains("rule 4"), "{e}");
    }

    /// #212/AK2, rule 5: an old .rplan file is never replaced; saved as .vplan.
    #[test]
    fn issue_212_ak2_rule5_rplan_never_replaced() {
        let tmp = tempfile::tempdir().unwrap();
        let (mut s, file) = opened(tmp.path(), "old.rplan");
        let original = std::fs::read(&file).unwrap();
        let e = save_done(&mut s, json!({ "plan": "p1", "overwrite": true }), true).unwrap_err();
        assert!(e.contains("rule 5"), "{e}");
        let e = save_done(
            &mut s,
            json!({ "plan": "p1", "path": "old.rplan", "overwrite": true }),
            true,
        )
        .unwrap_err();
        assert!(e.contains("rule 5"), "{e}");
        let a = ok(save_done(&mut s, json!({ "plan": "p1" }), true));
        assert_eq!(a["saved"], "old (AI).vplan");
        assert_eq!(std::fs::read(&file).unwrap(), original);
        assert!(save_done(&mut s, json!({ "plan": "p1", "path": "x.json" }), true).is_err());
    }

    /// #212/AK2, rule 6: only into the folder of the opened plan or a folder released with --allow-dir.
    #[test]
    fn issue_212_ak2_rule6_only_allowed_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let (plans, elsewhere, released) = (
            tmp.path().join("plans"),
            tmp.path().join("other"),
            tmp.path().join("out"),
        );
        for d in [&plans, &elsewhere, &released] {
            std::fs::create_dir(d).unwrap();
        }
        let (mut s, _) = opened(&plans, "plan.vplan");
        for path in [
            elsewhere.join("x.vplan").display().to_string(),
            "../other/x.vplan".into(),
            "sub/x.vplan".into(),
        ] {
            let e = save_done(&mut s, json!({ "plan": "p1", "path": path }), true).unwrap_err();
            assert!(e.contains("rule 6") || e.contains("does not exist"), "{path}: {e}");
        }
        assert!(files(&elsewhere).is_empty());
        let e = save_done(
            &mut s,
            json!({ "plan": "p1", "path": released.join("x.vplan").display().to_string() }),
            true,
        )
        .unwrap_err();
        assert!(e.contains("rule 6"), "{e}");
        s.allowed.push(std::fs::canonicalize(&released).unwrap());
        let a = ok(save_done(
            &mut s,
            json!({ "plan": "p1", "path": released.join("x.vplan").display().to_string() }),
            true,
        ));
        assert_eq!(a["saved"], "x.vplan");
        // The example plan has no folder of its own: only released folders.
        let mut ex = session();
        assert!(
            super::super::tools::call(&mut ex, "plan_open", &json!({ "example": true }))
                .unwrap()
                .is_ok()
        );
        assert!(save_done(&mut ex, json!({ "plan": "p1" }), true)
            .unwrap_err()
            .contains("--allow-dir"));
        // A link in the folder is not written through.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(elsewhere.join("t.vplan"), plans.join("link.vplan")).unwrap();
            let e = save_done(
                &mut s,
                json!({ "plan": "p1", "path": "link.vplan", "overwrite": true }),
                true,
            )
            .unwrap_err();
            assert!(e.contains("link"), "{e}");
            assert!(files(&elsewhere).is_empty());
        }
    }

    /// #263/AK2: With released folders plan_open reads only inside them (subfolders included); `..`, a linked file
    /// and a linked folder leading out are refused, the error names only the file. Without them every path as before.
    #[test]
    fn issue_263_ak2_plan_open_only_inside_released_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let (a, b) = (tmp.path().join("A"), tmp.path().join("B"));
        std::fs::create_dir_all(a.join("sub")).unwrap();
        std::fs::create_dir(&b).unwrap();
        let example = engine::api::example_plan().to_string();
        for f in [a.join("p.vplan"), a.join("sub").join("p.vplan"), b.join("q.vplan")] {
            std::fs::write(f, &example).unwrap();
        }
        let open = |s: &mut Session, p: &Path| {
            super::super::tools::call(s, "plan_open", &json!({ "path": p.display().to_string() })).unwrap()
        };
        let mut s = session();
        assert!(
            open(&mut s, &b.join("q.vplan")).is_ok(),
            "without --allow-dir every path"
        );
        s.allowed.push(std::fs::canonicalize(&a).unwrap());
        assert!(open(&mut s, &a.join("p.vplan")).is_ok());
        assert!(open(&mut s, &a.join("sub").join("p.vplan")).is_ok());
        let mut refused = vec![
            b.join("q.vplan"),
            a.join("sub").join("..").join("..").join("B").join("q.vplan"),
        ];
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(b.join("q.vplan"), a.join("link.vplan")).unwrap();
            std::os::unix::fs::symlink(&b, a.join("out")).unwrap();
            refused.push(a.join("link.vplan"));
            refused.push(a.join("out").join("q.vplan"));
        }
        for p in refused {
            let e = open(&mut s, &p).unwrap_err();
            assert!(
                e.contains("outside the plan folder") && e.contains("--allow-dir"),
                "{e}"
            );
            assert!(!e.contains(&tmp.path().display().to_string()), "{e}");
        }
        let e = open(&mut s, &a.join("none.vplan")).unwrap_err();
        assert_eq!(e, "none.vplan was not found");
    }

    /// #263: A released folder includes its subfolders for saving as for reading; outside stays refused (rule 6).
    #[test]
    fn issue_263_ak3_save_into_subfolders_of_a_released_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let (a, b) = (tmp.path().join("A"), tmp.path().join("B"));
        std::fs::create_dir_all(a.join("sub")).unwrap();
        std::fs::create_dir(&b).unwrap();
        let (mut s, _) = opened(&a, "plan.vplan");
        s.allowed.push(std::fs::canonicalize(&a).unwrap());
        let a2 = ok(save_done(&mut s, json!({ "plan": "p1", "path": "sub/x.vplan" }), true));
        assert_eq!(a2["saved"], "x.vplan");
        assert!(a.join("sub").join("x.vplan").is_file());
        let e = save_done(
            &mut s,
            json!({ "plan": "p1", "path": b.join("x.vplan").display().to_string() }),
            true,
        )
        .unwrap_err();
        assert!(e.contains("rule 6"), "{e}");
    }
}
