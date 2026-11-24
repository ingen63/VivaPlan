//! #210: State of an MCP session: open plans (working copies in memory) and results, each with an id (`p1`, `r1`).
//! Everything stays in the memory of the server and disappears with it; the results of the desktop app are separate.

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};

use super::privacy::Redactor;
use crate::AppData;

/// An open plan.
pub(crate) struct OpenPlan {
    pub(crate) id: String,
    /// File of the plan; `None` for the example plan.
    pub(crate) file: Option<PathBuf>,
    /// Working copy, brought up to date (without the limits of the settings).
    pub(crate) plan: Value,
}

impl OpenPlan {
    /// File name as answers show it.
    pub(crate) fn file_name(&self) -> String {
        self.file
            .as_deref()
            .and_then(Path::file_name)
            .map_or_else(|| "(example plan)".into(), |n| n.to_string_lossy().to_string())
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
    pub(crate) plans: Vec<OpenPlan>,
    pub(crate) results: Vec<SessionResult>,
    /// The note on privacy and the full disclaimer come with the first `plan_open`.
    pub(crate) notice_shown: bool,
    opened: usize,
    computed: usize,
}

impl Session {
    pub(crate) fn new(data: AppData, names: bool) -> Self {
        Session {
            data,
            names,
            plans: Vec::new(),
            results: Vec::new(),
            notice_shown: false,
            opened: 0,
            computed: 0,
        }
    }

    /// Data library for the engine; without one an empty path (only the built-in series).
    pub(crate) fn library(&self) -> &Path {
        self.data.library().unwrap_or(Path::new(""))
    }

    /// #210/AK3: Every computation of the server goes through the dispatcher of the app (`engine::dispatch::call`), with
    /// the same arguments as the user interface; an error is the English text of the message.
    pub(crate) fn call(&self, cmd: &str, args: &Value, file: Option<&Path>) -> Result<Value, String> {
        let fallback = file
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_default();
        let mut args = args.clone();
        if let (Some(f), Some(o)) = (file, args.as_object_mut()) {
            o.insert("path".into(), json!(f.display().to_string()));
        }
        engine::dispatch::call(cmd, &args, self.library(), &fallback).map_err(|m| m.text().to_string())
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

    pub(crate) fn add_plan(&mut self, file: Option<PathBuf>, plan: Value) -> &OpenPlan {
        self.opened += 1;
        self.plans.push(OpenPlan {
            id: format!("p{}", self.opened),
            file,
            plan,
        });
        self.plans.last().expect("just added")
    }

    /// Keeps a result of the plan `plan`; the computing tools (#211) store their results here.
    #[cfg_attr(not(test), allow(dead_code))]
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
                r.add_plan(&p.plan);
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
