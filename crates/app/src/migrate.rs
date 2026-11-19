//! #44: One-time takeover of the app data of the previous identifier.
//!
//! The new bundle identifier yields new data folders. If a new folder is empty or missing and the old one exists,
//! the first start copies the old content over: settings with the file last opened and the
//! confirmations, the time series and (in the data folder of the web view) the saved results. The old folder
//! stays unchanged and is never deleted.

use std::io;
use std::path::{Path, PathBuf};

/// legacy: RetirementPlanner – previous bundle identifier, only for taking over the app data.
pub const LEGACY_IDENTIFIER: &str = "ch.speckhofer.pensionsplaner";

/// Pairs (old folder, new folder). `dirs` are the new folders whose last part is the new identifier
/// (data folder, local data folder and on macOS the folder of the web view in `~/Library/WebKit`).
/// Duplicate folders appear only once.
pub fn legacy_pairs(dirs: &[PathBuf]) -> Vec<(PathBuf, PathBuf)> {
    let mut out: Vec<(PathBuf, PathBuf)> = vec![];
    for new in dirs {
        let Some(parent) = new.parent() else { continue };
        let old = parent.join(LEGACY_IDENTIFIER);
        if old != *new && !out.iter().any(|(_, n)| n == new) {
            out.push((old, new.clone()));
        }
    }
    out
}

/// Folder of the web view on macOS (WebKit stores IndexedDB there, among other things).
pub fn webkit_dir(home: &Path, identifier: &str) -> PathBuf {
    home.join("Library").join("WebKit").join(identifier)
}

/// Copies `old` to `new` if `old` is a folder and `new` is missing or empty. Result: whether it copied.
pub fn take_over(old: &Path, new: &Path) -> io::Result<bool> {
    if !old.is_dir() || !is_empty_or_missing(new)? {
        return Ok(false);
    }
    copy_dir(old, new)?;
    Ok(true)
}

fn is_empty_or_missing(dir: &Path) -> io::Result<bool> {
    match std::fs::read_dir(dir) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(e),
    }
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Takes over all pairs; errors are reported but do not prevent the start. Result: folders taken over.
pub fn take_over_all(pairs: &[(PathBuf, PathBuf)]) -> Vec<PathBuf> {
    pairs
        .iter()
        .filter_map(|(old, new)| match take_over(old, new) {
            Ok(true) => Some(new.clone()),
            Ok(false) => None,
            Err(e) => {
                eprintln!("App-Daten aus {} nicht übernommen: {e}", old.display());
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vivaplan-migrate-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// #44/AK4: settings, time series and subfolders are copied, the old folder stays in place.
    #[test]
    fn old_folder_is_copied_into_the_empty_new_one_and_kept() {
        let root = tmp("copy");
        let old = root.join(LEGACY_IDENTIFIER);
        let new = root.join("ch.vivaplan-app.desktop");
        std::fs::create_dir_all(old.join("Zeitreihen/sub")).unwrap();
        std::fs::write(old.join("einstellungen.json"), r#"{"lastPlan":"/x/a.vplan"}"#).unwrap();
        std::fs::write(old.join("Zeitreihen/sub/r.csv"), "1").unwrap();
        assert!(take_over(&old, &new).unwrap());
        assert_eq!(
            std::fs::read_to_string(new.join("einstellungen.json")).unwrap(),
            r#"{"lastPlan":"/x/a.vplan"}"#
        );
        assert_eq!(std::fs::read_to_string(new.join("Zeitreihen/sub/r.csv")).unwrap(), "1");
        assert!(old.join("einstellungen.json").is_file());
        // A second start changes nothing (the new folder is not empty).
        std::fs::write(old.join("einstellungen.json"), "{}").unwrap();
        assert!(!take_over(&old, &new).unwrap());
        assert_ne!(std::fs::read_to_string(new.join("einstellungen.json")).unwrap(), "{}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn empty_new_folder_is_filled_without_old_folder_nothing_happens() {
        let root = tmp("empty");
        let old = root.join(LEGACY_IDENTIFIER);
        let new = root.join("neu");
        assert!(!take_over(&old, &new).unwrap());
        assert!(!new.exists());
        std::fs::create_dir_all(&new).unwrap();
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("a"), "x").unwrap();
        assert!(take_over(&old, &new).unwrap());
        assert!(new.join("a").is_file());
        // A file instead of a folder as new target is an error that does not prevent the start.
        let file = root.join("datei");
        std::fs::write(&file, "x").unwrap();
        assert!(take_over(&old, &file).is_err());
        assert!(take_over_all(&[(old.clone(), file)]).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn empty_or_missing_is_detected_other_errors_stay_errors() {
        let root = tmp("leer");
        assert!(is_empty_or_missing(&root.join("gibt-es-nicht")).unwrap());
        assert!(is_empty_or_missing(&root).unwrap());
        std::fs::write(root.join("a"), "x").unwrap();
        assert!(!is_empty_or_missing(&root).unwrap());
        // A file instead of a folder is neither empty nor missing: the error is reported, not swallowed.
        assert!(is_empty_or_missing(&root.join("a")).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn pairs_from_the_new_folders() {
        let data = PathBuf::from("/h/Library/Application Support/ch.vivaplan-app.desktop");
        let web = webkit_dir(Path::new("/h"), "ch.vivaplan-app.desktop");
        assert_eq!(web, PathBuf::from("/h/Library/WebKit/ch.vivaplan-app.desktop"));
        let pairs = legacy_pairs(&[data.clone(), data.clone(), web.clone(), PathBuf::from("/")]);
        assert_eq!(
            pairs,
            vec![
                (
                    Path::new("/h/Library/Application Support").join(LEGACY_IDENTIFIER),
                    data
                ),
                (Path::new("/h/Library/WebKit").join(LEGACY_IDENTIFIER), web),
            ]
        );
        // If the new folder already has the old name, there is nothing to take over.
        assert!(legacy_pairs(&[PathBuf::from("/a").join(LEGACY_IDENTIFIER)]).is_empty());
    }

    #[test]
    fn several_pairs_are_taken_over() {
        let root = tmp("all");
        let old = root.join("a").join(LEGACY_IDENTIFIER);
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("f"), "1").unwrap();
        let new = root.join("a").join("neu");
        let pairs = vec![(old, new.clone()), (root.join("b/fehlt"), root.join("b/neu"))];
        assert_eq!(take_over_all(&pairs), vec![new]);
        let _ = std::fs::remove_dir_all(&root);
    }
}
