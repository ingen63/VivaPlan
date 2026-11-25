//! #210: State of an MCP session: open plans (working copies in memory) and results, each with an id (`p1`, `r1`).
//! Everything stays in the memory of the server and disappears with it; the results of the desktop app are separate.
//! #212: Every plan keeps the state of its file and the steps of its changes (concept, section 7).

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};

use super::privacy::Redactor;
use crate::AppData;

/// An open plan.
pub(crate) struct OpenPlan {
    pub(crate) id: String,
    /// File of the plan; `None` for the example plan and a new plan.
    pub(crate) file: Option<PathBuf>,
    /// #269: a new, empty plan (`plan_open` with `new: true`), not the example plan.
    pub(crate) blank: bool,
    /// Working copy, brought up to date (without the limits of the settings).
    pub(crate) plan: Value,
    /// #212: The plan as in the file (brought up to date): base of `plan_changes`.
    pub(crate) original: Value,
    /// #212: The working copy before each step of `plan_set` and `plan_apply`, oldest first (for `plan_undo`).
    pub(crate) history: Vec<Value>,
    /// #212: Time of change and content of the file when it was opened or last written (rule 4 of `plan_save`).
    pub(crate) stamp: Option<Stamp>,
}

/// #212: What the file looked like: a change by someone else changes the time or the content.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Stamp {
    pub(crate) modified: Option<std::time::SystemTime>,
    pub(crate) content: Vec<u8>,
}

impl Stamp {
    /// The file as it is now; `None` if it cannot be read.
    pub(crate) fn of(file: &Path) -> Option<Stamp> {
        Some(Stamp {
            modified: std::fs::metadata(file).ok()?.modified().ok(),
            content: std::fs::read(file).ok()?,
        })
    }
}

impl OpenPlan {
    /// File name as answers show it.
    pub(crate) fn file_name(&self) -> String {
        self.file.as_deref().and_then(Path::file_name).map_or_else(
            || if self.blank { "(new plan)" } else { "(example plan)" }.into(),
            |n| n.to_string_lossy().to_string(),
        )
    }
}

/// A result of the session.
pub(crate) struct SessionResult {
    pub(crate) id: String,
    /// Id of the plan it was computed from.
    pub(crate) plan: String,
    /// What computed it (tool name).
    pub(crate) kind: String,
    /// Answer of `run_simulation` (`api::RunOut` as JSON).
    pub(crate) result: Value,
    pub(crate) at: Instant,
}

pub(crate) struct Session {
    pub(crate) data: AppData,
    /// `--names`: names are not replaced.
    pub(crate) names: bool,
    /// #212/AK3: `--read-only`: no tool writes a file (`plan_save` is not offered).
    pub(crate) read_only: bool,
    /// #212: Folders released with `--allow-dir` for `plan_save` (rule 6), canonical; #263: with at least one, plans
    /// are read and written only inside them (subfolders included).
    pub(crate) allowed: Vec<PathBuf>,
    pub(crate) plans: Vec<OpenPlan>,
    pub(crate) results: Vec<SessionResult>,
    /// The note on privacy and the full disclaimer come with the first `plan_open`.
    pub(crate) notice_shown: bool,
    /// #270/AK9: Not activated with a beta token: every tool call is answered with this text.
    pub(crate) locked: Option<String>,
    opened: usize,
    computed: usize,
}

impl Session {
    pub(crate) fn new(data: AppData, names: bool) -> Self {
        Session {
            data,
            names,
            read_only: false,
            allowed: Vec::new(),
            plans: Vec::new(),
            results: Vec::new(),
            notice_shown: false,
            locked: None,
            opened: 0,
            computed: 0,
        }
    }

    /// #263/AK2: The canonical path of a plan file to read. With folders released by `--allow-dir` only a file inside
    /// one of them (`..` and symbolic links are resolved first); without, every path as before.
    pub(crate) fn readable(&self, path: &Path) -> Result<PathBuf, String> {
        if self.allowed.is_empty() {
            return Ok(path.to_path_buf());
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let canonical = std::fs::canonicalize(path).map_err(|_| format!("{name} was not found"))?;
        if self.allowed.iter().any(|d| canonical.starts_with(d)) {
            Ok(canonical)
        } else {
            Err(format!(
                "{name} is outside the plan folder: this server opens and saves plans only in the folders the user \
                 released (--allow-dir). Ask the user to move the plan into the plan folder or to choose another plan \
                 folder when setting up VivaPlan"
            ))
        }
    }

    /// #210/AK3: Every computation of the server goes through the dispatcher of the app (`engine::dispatch::call`), with
    /// the same arguments as the user interface; an error is the English text of the message.
    pub(crate) fn call(&self, cmd: &str, args: &Value, file: Option<&Path>) -> Result<Value, String> {
        dispatch(self.data.library(), cmd, args, file)
    }

    /// The plan as the app computes with it: with the limits and the affordability of the settings (#206).
    pub(crate) fn with_settings(&self, plan: &Value) -> Value {
        let mut p = plan.clone();
        engine::appdata::apply(&mut p, &self.data.settings);
        p
    }

    pub(crate) fn plan(&self, id: &str) -> Result<&OpenPlan, String> {
        self.plans.iter().find(|p| p.id == id).ok_or_else(|| {
            let open: Vec<&str> = self.plans.iter().map(|p| p.id.as_str()).collect();
            if open.is_empty() {
                format!("Unknown plan {id}; no plan is open, open one with plan_open")
            } else {
                format!("Unknown plan {id}; open plans: {}", open.join(", "))
            }
        })
    }

    pub(crate) fn add_plan(&mut self, file: Option<PathBuf>, plan: Value, stamp: Option<Stamp>) -> &OpenPlan {
        self.opened += 1;
        self.plans.push(OpenPlan {
            id: format!("p{}", self.opened),
            file,
            blank: false,
            original: plan.clone(),
            plan,
            history: Vec::new(),
            stamp,
        });
        self.plans.last().expect("just added")
    }

    /// Keeps a result of the plan `plan`; the computing tools (#211) store their results here.
    pub(crate) fn add_result(&mut self, plan: &str, kind: &str, result: Value) -> String {
        self.computed += 1;
        let id = format!("r{}", self.computed);
        self.results.push(SessionResult {
            id: id.clone(),
            plan: plan.into(),
            kind: kind.into(),
            result,
            at: Instant::now(),
        });
        id
    }

    /// #212: The open plan `id` to change.
    pub(crate) fn plan_mut(&mut self, id: &str) -> Result<&mut OpenPlan, String> {
        self.plan(id)?;
        Ok(self.plans.iter_mut().find(|p| p.id == id).expect("checked above"))
    }

    pub(crate) fn result(&self, id: &str) -> Result<&SessionResult, String> {
        self.results.iter().find(|r| r.id == id).ok_or_else(|| {
            if self.results.is_empty() {
                format!("Unknown result {id}; the session has no results yet")
            } else {
                let ids: Vec<&str> = self.results.iter().map(|r| r.id.as_str()).collect();
                format!("Unknown result {id}; results: {}", ids.join(", "))
            }
        })
    }

    /// #210/AK5: What answers replace: names of all open plans (unless `--names`) and their folders.
    pub(crate) fn redactor(&self) -> Redactor {
        let mut r = Redactor::new(Redactor::home());
        for p in &self.plans {
            if !self.names {
                // #212: also the names of earlier states, which changes and undo show as old values.
                for v in std::iter::once(&p.plan).chain(&p.history).chain([&p.original]) {
                    r.add_plan(v);
                }
            }
            if let Some(f) = &p.file {
                r.add_file(f);
            }
        }
        if let Some(l) = self.data.library() {
            r.add_dir(l);
        }
        r
    }
}

/// [`Session::call`] without a session, for the thread that computes (#211): `library` is the data library, `file` the
/// project file of the plan (folder for relative paths).
pub(crate) fn dispatch(library: Option<&Path>, cmd: &str, args: &Value, file: Option<&Path>) -> Result<Value, String> {
    let fallback = file
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    let mut args = args.clone();
    if let (Some(f), Some(o)) = (file, args.as_object_mut()) {
        o.insert("path".into(), json!(f.display().to_string()));
    }
    engine::dispatch::call(cmd, &args, library.unwrap_or(Path::new("")), &fallback).map_err(|m| m.text().to_string())
}
