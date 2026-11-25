//! #206: Data folder of the desktop app, shared by the app and the command line.
//!
//! The folder holds the data library (subfolder «Zeitreihen») and the settings (`einstellungen.json`). The app gets it
//! from Tauri (`app_data_dir`), the command line from [`default_dir`]; both give the same path (test in the app). The
//! settings that change a calculation (limits, affordability) reach the engine as part of the plan: in the app the user
//! interface adds them (`api.js`), on the command line [`apply`]. Both follow the same rules.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::model::Affordability;
use crate::settings::Limits;

/// Identifier of the app (`identifier` in `crates/app/tauri.conf.json`); name of the data folder.
pub const APP_ID: &str = "ch.vivaplan-app.desktop";

/// Subfolder of the data folder with the data library.
pub const LIBRARY_FOLDER: &str = "Zeitreihen";

/// File with the settings in the data folder (settings only, no plan data).
pub const SETTINGS_FILE: &str = "einstellungen.json";

/// Data folder of the app on this computer, as Tauri determines it (`app_data_dir`): macOS
/// `~/Library/Application Support/<id>`, Windows `%APPDATA%\<id>`, Linux `$XDG_DATA_HOME/<id>` or
/// `~/.local/share/<id>`. `None` if the home folder is unknown.
pub fn default_dir() -> Option<PathBuf> {
    data_dir_of(APP_ID)
}

/// Data folder of an app with the identifier `id` (the app tests use their own identifier).
pub fn data_dir_of(id: &str) -> Option<PathBuf> {
    dir_for(std::env::consts::OS, id, |k| std::env::var_os(k))
}

/// Data folder for the operating system `os` (as `std::env::consts::OS`) with the environment `var`.
fn dir_for(os: &str, id: &str, var: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let set = |k: &str| var(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = match os {
        "macos" => set("HOME").map(|h| h.join("Library").join("Application Support")),
        "windows" => set("APPDATA"),
        // Linux and other Unix systems (XDG): XDG_DATA_HOME only counts if it is an absolute path
        // (`has_root` equals `is_absolute` on Unix and keeps the test portable to Windows).
        _ => set("XDG_DATA_HOME")
            .filter(|p| p.has_root())
            .or_else(|| set("HOME").map(|h| h.join(".local").join("share"))),
    }?;
    Some(base.join(id))
}

/// Folder of the data library in the data folder `dir`.
pub fn library_dir(dir: &Path) -> PathBuf {
    dir.join(LIBRARY_FOLDER)
}

/// Reads the settings; a missing or unreadable file yields empty settings (the defaults apply).
pub fn read_settings(dir: &Path) -> Map<String, Value> {
    std::fs::read_to_string(dir.join(SETTINGS_FILE))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| match v {
            Value::Object(m) => Some(m),
            _ => None,
        })
        .unwrap_or_default()
}

/// Applicable limits: saved values (key `limits`), otherwise the defaults; as `limits()` in `app.js`.
pub fn limits(settings: &Map<String, Value>) -> Limits {
    overlay(Limits::default(), settings.get("limits"), &[])
}

/// Applicable affordability: saved values (key `affordability`), otherwise the defaults; as `affordability()` in
/// `app.js`. The wealth contribution is not a setting and keeps its default.
pub fn affordability(settings: &Map<String, Value>) -> Affordability {
    overlay(
        Affordability::default(),
        settings.get("affordability"),
        &["wealthContribution"],
    )
}

/// The defaults with every saved field of the same name that is a number, or a switch where the default is one (#266),
/// except `fixed`.
fn overlay<T: Serialize + DeserializeOwned>(default: T, saved: Option<&Value>, fixed: &[&str]) -> T {
    let Ok(Value::Object(mut out)) = serde_json::to_value(&default) else {
        return default;
    };
    if let Some(Value::Object(saved)) = saved {
        for (k, v) in saved {
            let same_kind = match out.get(k) {
                Some(Value::Bool(_)) => v.is_boolean(),
                Some(_) => v.as_f64().is_some_and(f64::is_finite),
                None => false,
            };
            if same_kind && !fixed.contains(&k.as_str()) {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::from_value(Value::Object(out)).unwrap_or(default)
}

/// Adds the settings to a plan as the user interface does with every call (`Engine.call` in `api.js`): the limits
/// always, the affordability only if the plan has none.
pub fn apply(plan: &mut Value, settings: &Map<String, Value>) {
    let Some(o) = plan.as_object_mut() else {
        return;
    };
    o.insert(
        "limits".into(),
        serde_json::to_value(limits(settings)).unwrap_or_default(),
    );
    let re = o.entry("realestate").or_insert_with(|| Value::Object(Map::new()));
    if !re.is_object() {
        *re = Value::Object(Map::new());
    }
    if let Some(re) = re.as_object_mut() {
        if re.get("affordability").is_none_or(Value::is_null) {
            let a = serde_json::to_value(affordability(settings)).unwrap_or_default();
            re.insert("affordability".into(), a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let pairs: Vec<(String, String)> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |k| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| OsString::from(v))
    }

    // #206/AK2: Data folder per operating system, as Tauri determines it.
    #[test]
    fn issue_206_ak2_data_folder_per_operating_system() {
        let id = APP_ID;
        assert_eq!(
            dir_for("macos", id, env(&[("HOME", "/Users/anna")])),
            Some(PathBuf::from(
                "/Users/anna/Library/Application Support/ch.vivaplan-app.desktop"
            ))
        );
        assert_eq!(
            dir_for("windows", id, env(&[("APPDATA", r"C:\Users\anna\AppData\Roaming")])),
            Some(PathBuf::from(r"C:\Users\anna\AppData\Roaming").join(id))
        );
        assert_eq!(
            dir_for("linux", id, env(&[("HOME", "/home/anna")])),
            Some(PathBuf::from("/home/anna/.local/share/ch.vivaplan-app.desktop"))
        );
        assert_eq!(
            dir_for("linux", id, env(&[("HOME", "/home/anna"), ("XDG_DATA_HOME", "/data")])),
            Some(PathBuf::from("/data/ch.vivaplan-app.desktop"))
        );
        // A relative XDG_DATA_HOME does not count (XDG specification).
        assert_eq!(
            dir_for("linux", id, env(&[("HOME", "/home/anna"), ("XDG_DATA_HOME", "data")])),
            Some(PathBuf::from("/home/anna/.local/share/ch.vivaplan-app.desktop"))
        );
        assert_eq!(dir_for("macos", id, env(&[("HOME", "")])), None);
        assert_eq!(dir_for("windows", id, env(&[])), None);
        assert_eq!(library_dir(Path::new("/d")), PathBuf::from("/d/Zeitreihen"));
    }

    // #206/AK3: Limits and affordability from the settings, as in app.js.
    #[test]
    fn issue_206_ak3_settings_overlay_the_defaults() {
        let s = |v: Value| v.as_object().unwrap().clone();
        assert_eq!(limits(&Map::new()), Limits::default());
        let l = limits(&s(
            json!({"limits": {"pkMin": 60, "ahvMax": "x", "unknown": 1, "pillar3aMax": 7000.5}}),
        ));
        assert_eq!((l.pk_min, l.ahv_max, l.pillar3a_max), (60.0, 70.0, 7000.5));
        assert_eq!(limits(&s(json!({"limits": [1]}))), Limits::default());
        let a = affordability(&s(
            json!({"affordability": {"sustainability": 0.4, "wealthContribution": 0.5}}),
        ));
        assert_eq!(
            (a.sustainability, a.wealth_contribution, a.imputed_rate),
            (0.4, 0.03, 0.05)
        );
    }

    // #266/AK1: new values with defaults; saved values (switch and numbers) are taken over.
    #[test]
    fn issue_266_ak1_affordability_defaults_and_saved_values() {
        let s = |v: Value| v.as_object().unwrap().clone();
        let a = affordability(&s(json!({"affordability": {"sustainability": 0.4}})));
        assert_eq!(
            (a.wealth_drawdown, a.wealth_drawdown_rate, a.max_loan_to_value_retired),
            (false, 0.03, 0.65)
        );
        let a = affordability(&s(
            json!({"affordability": {"wealthDrawdown": true, "wealthDrawdownRate": 0.05,
            "maxLoanToValueRetired": 0.6}}),
        ));
        assert_eq!(
            (a.wealth_drawdown, a.wealth_drawdown_rate, a.max_loan_to_value_retired),
            (true, 0.05, 0.6)
        );
        // Wrong kinds are ignored.
        let a = affordability(&s(
            json!({"affordability": {"wealthDrawdown": 1, "wealthDrawdownRate": true}}),
        ));
        assert_eq!((a.wealth_drawdown, a.wealth_drawdown_rate), (false, 0.03));
        // Older plans without the new fields still open.
        let old: crate::model::Affordability =
            serde_json::from_value(json!({"sustainability": 0.25, "imputedRate": 0.0625,
            "fixCosts": 0.01, "wealthContribution": 0.03, "maxLoanToValue": 0.8}))
            .unwrap();
        assert_eq!((old.wealth_drawdown, old.max_loan_to_value_retired), (false, 0.65));
    }

    // #206/AK3: The plan gets the limits always and the affordability only if it has none.
    #[test]
    fn issue_206_ak3_apply_adds_limits_and_missing_affordability() {
        let settings = json!({"limits": {"pkMin": 60}, "affordability": {"sustainability": 0.4}});
        let settings = settings.as_object().unwrap();
        let mut plan = json!({"limits": {"pkMin": 50}});
        apply(&mut plan, settings);
        assert_eq!(plan["limits"]["pkMin"], 60.0);
        assert_eq!(plan["realestate"]["affordability"]["sustainability"], 0.4);
        let mut plan = json!({"realestate": {"properties": [], "affordability": {"sustainability": 0.2}}});
        apply(&mut plan, settings);
        assert_eq!(plan["realestate"]["affordability"]["sustainability"], 0.2);
        assert!(plan["realestate"]["properties"].is_array());
        let mut plan = json!({"realestate": null});
        apply(&mut plan, settings);
        assert_eq!(plan["realestate"]["affordability"]["sustainability"], 0.4);
        let mut not_a_plan = json!([1]);
        apply(&mut not_a_plan, settings);
        assert_eq!(not_a_plan, json!([1]));
    }

    // #206/AK2: Settings file missing, unreadable or not an object: empty settings.
    #[test]
    fn issue_206_ak2_read_settings() {
        let d = tempfile::tempdir().unwrap();
        assert!(read_settings(d.path()).is_empty());
        std::fs::write(d.path().join(SETTINGS_FILE), "no json").unwrap();
        assert!(read_settings(d.path()).is_empty());
        std::fs::write(d.path().join(SETTINGS_FILE), "[1]").unwrap();
        assert!(read_settings(d.path()).is_empty());
        std::fs::write(d.path().join(SETTINGS_FILE), r#"{"theme":"dark"}"#).unwrap();
        assert_eq!(read_settings(d.path())["theme"], "dark");
    }
}
