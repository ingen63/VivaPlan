//! #142: Verteiler der Rechenbefehle mit den Argumenten, wie die Oberfläche sie sendet (`crates/app/ui/api.js`).
//!
//! Der Entwicklungsserver (`vivaplan ui-dev`, Grundlage der Oberflächentests) beantwortet `/api/<befehl>` hiermit. Die
//! Desktop-App hat eigene Tauri-Befehle; der Vertragstest in `crates/app/src/main.rs` schickt jeden Eintrag der Tabelle
//! `crates/app/ipc-commands.json` durch die Tauri-IPC und durch [`call`] und verlangt dieselbe Antwort.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Value};

use crate::{api, progress};

/// Alle Befehle, die [`call`] beantwortet. Befehle mit Dateidialog, Einstellungen oder Fenster gibt es nur in der
/// Desktop-App.
pub const COMMANDS: &[&str] = &[
    "app_info",
    "bench_plan",
    "progress_get",
    "cancel_job",
    "opt_params",
    "example_plan",
    "gain_templates",
    "validate",
    "upgrade_plan",
    "preview",
    "run_simulation",
    "rerun_paths",
    "series_info",
    "optimize",
    "library_list",
];

/// Ordner einer Datei im Argument `key` (Projektdatei der Planung); ohne Angabe `fallback`.
fn dir_of(args: &Value, key: &str, fallback: &Path) -> PathBuf {
    args.get(key)
        .and_then(Value::as_str)
        .and_then(|p| Path::new(p).parent().map(Path::to_path_buf))
        .unwrap_or_else(|| fallback.to_path_buf())
}

/// Kennung der Berechnung (`jobId`), die die Oberfläche beim Start mitgibt.
fn job_id(args: &Value) -> Option<u64> {
    args.get("jobId").and_then(Value::as_u64)
}

fn to(v: &impl Serialize) -> Value {
    serde_json::to_value(v).expect("serialisierbar")
}

/// Beantwortet einen Rechenbefehl. `library` ist die Datenbibliothek, `fallback` der Ordner für relative Pfade, wenn
/// die Planung noch nicht gespeichert ist (Desktop-App: Benutzerordner, Entwicklungsserver: Arbeitsverzeichnis).
pub fn call(cmd: &str, args: &Value, library: &Path, fallback: &Path) -> Result<Value, String> {
    let plan = args.get("plan").cloned().unwrap_or(Value::Null);
    let base = || dir_of(args, "path", fallback);
    let lib = Some(library);
    Ok(match cmd {
        "app_info" => json!({ "version": env!("CARGO_PKG_VERSION"), "buildEpoch": null }),
        "bench_plan" => to(&api::bench(&plan, &base(), lib)?),
        "progress_get" => to(&match job_id(args) {
            Some(id) => progress::snapshot_of(id),
            None => progress::snapshot(),
        }),
        "cancel_job" => {
            match job_id(args) {
                Some(id) => {
                    progress::cancel_job(id);
                }
                None => progress::cancel(),
            }
            Value::Null
        }
        "opt_params" => to(&api::opt_params(&plan)),
        "example_plan" => api::example_plan(),
        "gain_templates" => api::gain_templates(),
        "validate" => to(&api::validate(&plan)),
        "upgrade_plan" => to(&api::upgrade_plan(&plan)),
        "preview" => to(&api::preview(&plan, &base(), lib)),
        "run_simulation" => to(&api::run_as(job_id(args), &plan, &base(), lib)),
        "rerun_paths" => {
            let picks: Vec<(u32, usize)> =
                serde_json::from_value(args.get("picks").cloned().unwrap_or_default()).map_err(|e| e.to_string())?;
            to(&api::rerun_paths(&plan, &picks, &base(), lib))
        }
        "series_info" => {
            let p = args.get("path").and_then(Value::as_str).ok_or("Pfad fehlt")?;
            // #142: wie die Desktop-App relativ zum Ordner der Projektdatei (`planPath`)
            to(&api::series_info(p, &dir_of(args, "planPath", fallback), lib).map_err(|e| e.to_string())?)
        }
        "optimize" => {
            let mut req = args.get("request").cloned().unwrap_or(json!({}));
            if let (Some(id), Some(obj)) = (job_id(args), req.as_object_mut()) {
                obj.insert("jobId".into(), id.into());
            }
            to(&api::optimize(&plan, &req, &base(), lib).map_err(|e| e.to_string())?)
        }
        "library_list" => to(&api::library_list(library).map_err(|e| e.to_string())?),
        _ => return Err(format!("Befehl «{cmd}» gibt es nur in der Desktop-App")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    /// #142/AK3: Die Liste `COMMANDS` entspricht genau den Zweigen von `call` und den Rechenbefehlen der Tabelle.
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
                .contains("nur in der Desktop-App"));
        }
    }

    /// #142: Jeder Befehl ist erreichbar (keine Meldung «nur in der Desktop-App»); fehlende Argumente sind Fehler.
    #[test]
    fn every_command_is_answered() {
        for c in COMMANDS {
            if let Err(e) = call(c, &json!({ "plan": {}, "path": "x/p.vplan" }), &lib(), Path::new("/")) {
                assert!(!e.contains("nur in der Desktop-App"), "{c}: {e}");
            }
        }
        assert_eq!(
            call("series_info", &json!({}), &lib(), Path::new("/")).unwrap_err(),
            "Pfad fehlt"
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

    /// #142: `series_info` liest einen relativen Pfad im Ordner der Projektdatei (`planPath`), ohne Projektdatei im
    /// Ersatzordner; `path` der Planung zählt hier nicht.
    #[test]
    fn series_info_is_relative_to_the_plan_file() {
        let d = std::env::temp_dir().join(format!("vivaplan-dispatch-{}", std::process::id()));
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
        // Ohne Projektdatei gilt der Ersatzordner; dort gibt es die Datei nicht
        assert!(call(
            "series_info",
            &json!({ "path": "s.csv", "planPath": null }),
            &lib(),
            Path::new("/")
        )
        .is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// #142: Kennung und Ordner aus den Argumenten.
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
