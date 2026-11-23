//! #136: Plans are saved atomically. On any error the existing file stays unchanged, and no
//! temporary file remains in the folder.

use std::path::{Path, PathBuf};

use engine::api;
use serde_json::{json, Value};

/// Temporary folder that is deleted when the returned guard is dropped (end of the test).
fn dir(name: &str) -> (tempfile::TempDir, PathBuf) {
    let d = tempfile::Builder::new()
        .prefix(&format!("vivaplan-files-{name}-"))
        .tempdir()
        .unwrap();
    let p = d.path().to_path_buf();
    (d, p)
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

/// #136/AK1: Successful saving replaces the content; afterwards only the plan is in the folder.
#[test]
fn issue_136_ak1_saving_replaces_the_file_and_leaves_nothing_else() {
    let (_tmp, d) = dir("ok");
    let f = d.join("planung.vplan");
    api::write_plan(&f, &plan("A")).unwrap();
    api::write_plan(&f, &plan("B")).unwrap();
    assert_eq!(api::read_plan(&f).unwrap()["project"]["name"], "B");
    assert_eq!(entries(&d), vec!["planung.vplan"]);
    let _ = std::fs::remove_dir_all(&d);
}

/// #136/AK2, AK3: If writing fails (read-only folder), `write_plan` returns an error with the path,
/// the previous file has the old content, and no temporary file remains.
#[cfg(unix)]
#[test]
fn issue_136_ak2_failed_write_keeps_the_previous_file() {
    use std::os::unix::fs::PermissionsExt;
    let (_tmp, d) = dir("readonly");
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

/// #136/AK3: The temporary file is created, but the replacement fails (a folder is at the target): error, and the
/// temporary file is removed again.
#[test]
fn issue_136_ak3_failed_replace_removes_the_temporary_file() {
    let (_tmp, d) = dir("replace");
    let target = d.join("ordner.vplan");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("inhalt"), "bleibt").unwrap();
    assert!(api::write_plan(&target, &plan("Neu")).is_err());
    assert_eq!(entries(&d), vec!["ordner.vplan"]);
    assert_eq!(std::fs::read_to_string(target.join("inhalt")).unwrap(), "bleibt");
    let _ = std::fs::remove_dir_all(&d);
}

/// Decision on #136: If the file is a symbolic link, the file it points to is replaced; the link stays.
#[cfg(unix)]
#[test]
fn issue_136_saving_through_a_symlink_replaces_its_target() {
    let (_tmp, d) = dir("symlink");
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
