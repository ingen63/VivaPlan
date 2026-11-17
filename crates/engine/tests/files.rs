//! #136: Planungen werden atomar gespeichert. Die vorhandene Datei bleibt bei jedem Fehler unverändert, und es bleibt
//! keine temporäre Datei im Ordner zurück.

use std::path::{Path, PathBuf};

use engine::api;
use serde_json::{json, Value};

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vivaplan-files-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn plan(name: &str) -> Value {
    let mut p = api::example_plan();
    p["project"]["name"] = json!(name);
    p
}

fn entries(d: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// #136/AK1: Erfolgreiches Speichern ersetzt den Inhalt, im Ordner liegt danach nur die Planung.
#[test]
fn issue_136_ak1_saving_replaces_the_file_and_leaves_nothing_else() {
    let d = dir("ok");
    let f = d.join("planung.vplan");
    api::write_plan(&f, &plan("A")).unwrap();
    api::write_plan(&f, &plan("B")).unwrap();
    assert_eq!(api::read_plan(&f).unwrap()["project"]["name"], "B");
    assert_eq!(entries(&d), vec!["planung.vplan"]);
    let _ = std::fs::remove_dir_all(&d);
}

/// #136/AK2, AK3: Schlägt das Schreiben fehl (schreibgeschützter Ordner), gibt `write_plan` einen Fehler mit dem Pfad,
/// die bisherige Datei hat den alten Inhalt, und es bleibt keine temporäre Datei zurück.
#[cfg(unix)]
#[test]
fn issue_136_ak2_failed_write_keeps_the_previous_file() {
    use std::os::unix::fs::PermissionsExt;
    let d = dir("readonly");
    let f = d.join("planung.vplan");
    api::write_plan(&f, &plan("Alt")).unwrap();
    std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o555)).unwrap();
    let e = api::write_plan(&f, &plan("Neu")).unwrap_err().to_string();
    std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(e.contains("planung.vplan"), "{e}");
    assert_eq!(api::read_plan(&f).unwrap()["project"]["name"], "Alt");
    assert_eq!(entries(&d), vec!["planung.vplan"]);
    let _ = std::fs::remove_dir_all(&d);
}

/// #136/AK3: Die temporäre Datei entsteht, das Ersetzen schlägt aber fehl (am Ziel liegt ein Ordner): Fehler, und die
/// temporäre Datei ist wieder entfernt.
#[test]
fn issue_136_ak3_failed_replace_removes_the_temporary_file() {
    let d = dir("replace");
    let target = d.join("ordner.vplan");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("inhalt"), "bleibt").unwrap();
    assert!(api::write_plan(&target, &plan("Neu")).is_err());
    assert_eq!(entries(&d), vec!["ordner.vplan"]);
    assert_eq!(std::fs::read_to_string(target.join("inhalt")).unwrap(), "bleibt");
    let _ = std::fs::remove_dir_all(&d);
}

/// Entscheid zu #136: Ist die Datei ein symbolischer Link, wird die Datei ersetzt, auf die er zeigt; der Link bleibt.
#[cfg(unix)]
#[test]
fn issue_136_saving_through_a_symlink_replaces_its_target() {
    let d = dir("symlink");
    let real = d.join("echt.vplan");
    let link = d.join("link.vplan");
    api::write_plan(&real, &plan("Alt")).unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    api::write_plan(&link, &plan("Neu")).unwrap();
    assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert_eq!(api::read_plan(&real).unwrap()["project"]["name"], "Neu");
    assert_eq!(entries(&d), vec!["echt.vplan", "link.vplan"]);
    let _ = std::fs::remove_dir_all(&d);
}
