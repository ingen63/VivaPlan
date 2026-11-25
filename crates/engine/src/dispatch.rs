//! #142: Dispatcher of the compute commands with the arguments as the user interface sends them (`crates/app/ui/api.js`).
//!
//! The development server (`vivaplan ui-dev`, basis of the user interface tests) answers `/api/<command>` with it. The
//! desktop app has its own Tauri commands; the contract test in `crates/app/src/main.rs` sends every entry of the table
//! `crates/app/ipc-commands.json` through the Tauri IPC and through [`call`] and requires the same answer.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Value};

use crate::{api, progress};

/// All commands that [`call`] answers. Commands with a file dialog, settings or windows exist only in the
/// desktop app.
pub const COMMANDS: &[&str] = &[
    "app_info",
    "bench_plan",
    "progress_get",
    "cancel_job",
    "opt_params",
    "example_plan",
    "gain_templates",
    "tax_locations",
    "validate",
    "upgrade_plan",
    "preview",
    "run_simulation",
    "rerun_paths",
    "series_info",
    "optimize",
    "library_list",
    "yearly",
    "digest",
    "compare_results",
];

/// Folder of a file in the argument `key` (project file of the plan); if not given, `fallback`.
fn dir_of(args: &Value, key: &str, fallback: &Path) -> PathBuf {
    args.get(key)
        .and_then(Value::as_str)
        .and_then(|p| Path::new(p).parent().map(Path::to_path_buf))
        .unwrap_or_else(|| fallback.to_path_buf())
}

/// Id of the computation (`jobId`) that the user interface passes at the start.
fn job_id(args: &Value) -> Option<u64> {
    args.get("jobId").and_then(Value::as_u64)
}

/// #221: serde_json would write NaN and ∞ as `null`; a result with such a number is an error that names the field.
fn to(v: &impl Serialize) -> Result<Value, crate::message::Message> {
    if let Some(field) = crate::finite::first_non_finite(v) {
        return Err(msg!(result_not_finite, field = field));
    }
    Ok(serde_json::to_value(v).expect("serialisierbar"))
}

/// Answers a compute command. `library` is the data library, `fallback` the folder for relative paths if
/// the plan has not been saved yet (desktop app: user folder, development server: working directory).
/// #171: An error is a message (code, parameters, German text) like the errors of the desktop app.
pub fn call(cmd: &str, args: &Value, library: &Path, fallback: &Path) -> Result<Value, crate::message::Message> {
    let plan = args.get("plan").cloned().unwrap_or(Value::Null);
    let base = || dir_of(args, "path", fallback);
    let lib = Some(library);
    Ok(match cmd {
        "app_info" => {
            json!({ "version": env!("CARGO_PKG_VERSION"), "buildEpoch": null, "cliPath": null, "planDir": null })
        }
        "bench_plan" => to(&api::bench(&plan, &base(), lib)?)?,
        "progress_get" => to(&match job_id(args) {
            Some(id) => progress::snapshot_of(id),
            None => progress::snapshot(),
        })?,
        "cancel_job" => {
            match job_id(args) {
                Some(id) => {
                    progress::cancel_job(id);
                }
                None => progress::cancel(),
            }
            Value::Null
        }
        "opt_params" => to(&api::opt_params(&plan))?,
        "example_plan" => api::example_plan(),
        "gain_templates" => api::gain_templates(),
        "tax_locations" => to(&api::tax_locations())?,
        "validate" => to(&api::validate(&plan))?,
        "upgrade_plan" => to(&api::upgrade_plan(&plan))?,
        "preview" => to(&api::preview(&plan, &base(), lib))?,
        "run_simulation" => to(&api::run_as(job_id(args), &plan, &base(), lib))?,
        "rerun_paths" => {
            let picks: Vec<(u32, usize)> = serde_json::from_value(args.get("picks").cloned().unwrap_or_default())
                .map_err(|e| msg!(raw_error, detail = e.to_string()))?;
            to(&api::rerun_paths(&plan, &picks, &base(), lib))?
        }
        "series_info" => {
            let p = args
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| msg!(path_missing))?;
            // #142: like the desktop app, relative to the folder of the project file (`planPath`)
            to(&api::series_info(p, &dir_of(args, "planPath", fallback), lib).map_err(|e| e.message())?)?
        }
        "optimize" => {
            let mut req = args.get("request").cloned().unwrap_or(json!({}));
            if let (Some(id), Some(obj)) = (job_id(args), req.as_object_mut()) {
                obj.insert("jobId".into(), id.into());
            }
            to(&api::optimize(&plan, &req, &base(), lib).map_err(|e| e.message())?)?
        }
        "library_list" => to(&api::library_list(library).map_err(|e| e.message())?)?,
        // #209: evaluation of results the user interface holds (no plan, no computation of runs)
        "yearly" => to(&api::yearly(args.get("run").unwrap_or(&Value::Null))?)?,
        "digest" => to(&api::digest(args.get("result").unwrap_or(&Value::Null))?)?,
        "compare_results" => to(&api::compare(args.get("items").unwrap_or(&Value::Null))?)?,
        _ => return Err(msg!(command_desktop_only, cmd = cmd)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    /// #162/AK1: The answer is the serialized result of the command, not empty. The contract test (#142) compares
    /// desktop app and development server; if both returned `null`, it would not notice there.
    #[test]
    fn issue_162_ak1_answer_is_the_serialized_result() {
        let plan: Value = serde_json::from_str(include_str!("../assets/example.vplan")).unwrap();
        let args = json!({ "plan": plan });
        let issues = call("validate", &args, &lib(), Path::new("/")).unwrap();
        assert_eq!(issues, serde_json::to_value(crate::api::validate(&plan)).unwrap());
        assert!(issues.is_array());
        let params = call("opt_params", &args, &lib(), Path::new("/")).unwrap();
        assert!(params.as_array().is_some_and(|p| !p.is_empty()), "{params}");
    }

    /// #142/AK3: The list `COMMANDS` matches exactly the branches of `call` and the compute commands of the table.
    #[test]
    fn issue_142_ak3_commands_match_the_branches_and_the_table() {
        let src = include_str!("dispatch.rs");
        let body = &src[src.find("Ok(match cmd {").unwrap()..src.find("_ => return Err").unwrap()];
        let mut arms: Vec<&str> = body
            .lines()
            .filter_map(|l| l.trim().strip_prefix('"')?.split_once("\" =>").map(|(c, _)| c))
            .collect();
        arms.sort_unstable();
        let mut listed = COMMANDS.to_vec();
        listed.sort_unstable();
        assert_eq!(arms, listed);

        let table: Value = serde_json::from_str(include_str!("../../app/ipc-commands.json")).unwrap();
        let mut compute: Vec<&str> = table["compute"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["cmd"].as_str().unwrap())
            .collect();
        compute.sort_unstable();
        compute.dedup();
        assert_eq!(compute, listed, "Tabelle crates/app/ipc-commands.json");
        for c in table["desktopOnly"].as_array().unwrap() {
            let c = c.as_str().unwrap();
            assert!(!COMMANDS.contains(&c), "{c}");
            assert!(call(c, &json!({}), &lib(), Path::new("/"))
                .unwrap_err()
                .contains("only available in the desktop app"));
        }
    }

    /// #142: Every command is reachable (no message «nur in der Desktop-App»); missing arguments are errors.
    #[test]
    fn every_command_is_answered() {
        for c in COMMANDS {
            if let Err(e) = call(c, &json!({ "plan": {}, "path": "x/p.vplan" }), &lib(), Path::new("/")) {
                assert!(!e.contains("only available in the desktop app"), "{c}: {e}");
            }
        }
        assert_eq!(
            call("series_info", &json!({}), &lib(), Path::new("/")).unwrap_err(),
            "Path missing"
        );
        assert!(call("rerun_paths", &json!({ "picks": "x" }), &lib(), Path::new("/")).is_err());
        assert_eq!(
            call("cancel_job", &json!({ "jobId": 7 }), &lib(), Path::new("/")).unwrap(),
            Value::Null
        );
        assert_eq!(
            call("app_info", &json!({}), &lib(), Path::new("/")).unwrap()["buildEpoch"],
            Value::Null
        );
    }

    /// #213/AK2: the dev server reports `cliPath` as an explicit null; the key must exist because the contract test
    /// compares the field names of app and dispatch (indexing a missing key would also yield null).
    #[test]
    fn issue_213_ak2_app_info_has_cli_path_key_with_null() {
        let info = call("app_info", &json!({}), &lib(), Path::new("/")).unwrap();
        assert_eq!(info.as_object().unwrap().get("cliPath"), Some(&Value::Null));
    }

    /// #142: `series_info` reads a relative path in the folder of the project file (`planPath`), without a project file in
    /// the fallback folder; the plan's `path` does not count here.
    // slow: copies and reads a whole return series
    #[test]
    fn series_info_is_relative_to_the_plan_file() {
        let _tmp_d = tempfile::Builder::new().prefix("vivaplan-").tempdir().unwrap();
        let d = _tmp_d.path().to_path_buf();
        std::fs::create_dir_all(d.join("plan")).unwrap();
        std::fs::copy(lib().join("Shiller CHF.csv"), d.join("plan/s.csv")).unwrap();
        let plan_path = d.join("plan/p.vplan").to_string_lossy().into_owned();
        let ok = call(
            "series_info",
            &json!({ "path": "s.csv", "planPath": plan_path }),
            &lib(),
            Path::new("/"),
        );
        assert!(ok.is_ok(), "{ok:?}");
        let fb = call(
            "series_info",
            &json!({ "path": "s.csv", "planPath": null }),
            &lib(),
            &d.join("plan"),
        );
        assert!(fb.is_ok(), "{fb:?}");
        // Without a project file the fallback folder applies; the file does not exist there
        assert!(call(
            "series_info",
            &json!({ "path": "s.csv", "planPath": null }),
            &lib(),
            Path::new("/")
        )
        .is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// #142: Id and folder from the arguments.
    #[test]
    fn arguments_are_read_like_the_user_interface_sends_them() {
        assert_eq!(job_id(&json!({ "jobId": 42 })), Some(42));
        assert_eq!(job_id(&json!({ "job_id": 42 })), None);
        assert_eq!(
            dir_of(&json!({ "path": "/a/b.vplan" }), "path", Path::new("/x")),
            PathBuf::from("/a")
        );
        assert_eq!(
            dir_of(&json!({ "path": null }), "path", Path::new("/x")),
            PathBuf::from("/x")
        );
    }
}
