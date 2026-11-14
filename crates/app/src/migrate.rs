//! FEA-34: Einmalige Übernahme der App-Daten des bisherigen Identifiers.
//!
//! Der neue Bundle-Identifier ergibt neue Datenordner. Ist ein neuer Ordner leer oder fehlt er und gibt es den alten,
//! kopiert der erste Start den alten Inhalt hinüber: Einstellungen mit der zuletzt geöffneten Datei und den
//! Bestätigungen, die Zeitreihen und (im Datenordner der Webansicht) die gespeicherten Ergebnisse. Der alte Ordner
//! bleibt unverändert und wird nie gelöscht.

use std::io;
use std::path::{Path, PathBuf};

/// legacy: RetirementPlanner – bisheriger Bundle-Identifier, nur für die Übernahme der App-Daten.
pub const LEGACY_IDENTIFIER: &str = "ch.speckhofer.pensionsplaner";

/// Paare (alter Ordner, neuer Ordner). `dirs` sind die neuen Ordner, deren letzter Teil der neue Identifier ist
/// (Datenordner, lokaler Datenordner und unter macOS der Ordner der Webansicht in `~/Library/WebKit`).
/// Doppelte Ordner erscheinen nur einmal.
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

/// Ordner der Webansicht unter macOS (WebKit legt dort u. a. IndexedDB ab).
pub fn webkit_dir(home: &Path, identifier: &str) -> PathBuf {
    home.join("Library").join("WebKit").join(identifier)
}

/// Kopiert `old` nach `new`, wenn `old` ein Ordner ist und `new` fehlt oder leer ist. Ergebnis: ob kopiert wurde.
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

/// Übernimmt alle Paare; Fehler werden gemeldet, verhindern aber den Start nicht. Ergebnis: übernommene Ordner.
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

    /// FEA-34/AK4: Einstellungen, Zeitreihen und Unterordner werden kopiert, der alte Ordner bleibt bestehen.
    #[test]
    fn alter_ordner_wird_in_den_leeren_neuen_kopiert_und_bleibt_bestehen() {
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
        // Ein zweiter Start ändert nichts mehr (der neue Ordner ist nicht leer).
        std::fs::write(old.join("einstellungen.json"), "{}").unwrap();
        assert!(!take_over(&old, &new).unwrap());
        assert_ne!(std::fs::read_to_string(new.join("einstellungen.json")).unwrap(), "{}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn leerer_neuer_ordner_wird_gefuellt_ohne_alten_ordner_geschieht_nichts() {
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
        // Eine Datei statt eines Ordners als neues Ziel ist ein Fehler, der den Start nicht verhindert.
        let file = root.join("datei");
        std::fs::write(&file, "x").unwrap();
        assert!(take_over(&old, &file).is_err());
        assert!(take_over_all(&[(old.clone(), file)]).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn paare_aus_den_neuen_ordnern() {
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
        // Hat der neue Ordner schon den alten Namen, gibt es nichts zu übernehmen.
        assert!(legacy_pairs(&[PathBuf::from("/a").join(LEGACY_IDENTIFIER)]).is_empty());
    }

    #[test]
    fn mehrere_paare_werden_uebernommen() {
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
