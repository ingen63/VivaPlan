//! Import alter `config.json`: Ergebnis ist eine gültige, rechenbare Planung. (Das frühere Programm ist
//! keine Referenz für die Werte; geprüft wird die Übernahme der Angaben.)

use std::path::Path;

use engine::sim::Simulator;
use engine::validate::Severity;
use engine::{batch, legacy};

fn import(file: &Path, base: &Path) -> legacy::LegacyImport {
    let text = std::fs::read_to_string(file).unwrap();
    legacy::import(&text, 2026, base, "Test").unwrap()
}

fn runs(imp: &legacy::LegacyImport, base: &Path) {
    let errors: Vec<_> = engine::validate::validate(&imp.plan)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let sim = Simulator::new(&imp.plan).unwrap();
    let data = batch::load_series(&imp.plan, base, None).unwrap();
    let paths = batch::market_paths(&imp.plan, &sim, data.as_ref()).unwrap();
    let r = sim.run(&paths[0], true);
    assert_eq!(r.records.len(), sim.months());
    assert!(r.records.iter().all(|x| x.total.is_finite()));
}

#[test]
fn fixtures_import_and_run() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let fixed = import(&dir.join("legacy_fixed.json"), &dir);
    assert_eq!(fixed.plan.returns.source, engine::model::ReturnSource::Fixed);
    assert_eq!(fixed.plan.household.persons.len(), 1);
    runs(&fixed, &dir);
    let hist = import(&dir.join("legacy_historical.json"), &dir);
    assert_eq!(hist.plan.returns.source, engine::model::ReturnSource::Historical);
    assert!(hist
        .plan
        .returns
        .series
        .as_deref()
        .unwrap()
        .ends_with("renditen-muster.csv"));
    runs(&hist, &dir);
}

/// FEA-74: Alle `.json`-Dateien eines Ordners, nach Namen sortiert; fehlt der Ordner, ist die Liste leer.
fn json_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut files: Vec<_> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("json")))
        .collect();
    files.sort();
    files
}

/// FEA-74/AK1, AK2: Die Liste nimmt jede `.json`-Datei, ohne feste Namen, und ist leer, wenn der Ordner fehlt.
#[test]
fn fea74_ak1_ak2_json_files_without_fixed_names() {
    let dir = std::env::temp_dir().join(format!("pp-eigene-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(json_files(&dir).is_empty(), "fehlender Ordner");
    std::fs::create_dir_all(dir.join("data.json")).unwrap();
    for f in ["b.json", "a.JSON", "notiz.txt", "ohne-endung"] {
        std::fs::write(dir.join(f), "{}").unwrap();
    }
    let names: Vec<String> = json_files(&dir)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a.JSON", "b.json"], "nur Dateien mit Endung .json, sortiert");
    std::fs::remove_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    assert!(json_files(&dir).is_empty(), "leerer Ordner");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Eigene Konfigurationen mit Immobilien in `local/eigene/` (nur lokal, von Git ausgeschlossen; fehlen sie,
/// wird der Test übersprungen). Die Zeitreihe `./data/MSCI.csv` der Konfigurationen liegt in `local/eigene/data/`.
/// FEA-74: geprüft wird jede `.json`-Datei des Ordners, ohne feste Dateinamen.
#[test]
fn own_configs_with_real_estate() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local/eigene");
    for f in json_files(&dir) {
        let name = f.file_name().unwrap().to_string_lossy().into_owned();
        let imp = import(&f, &dir);
        assert!(
            !imp.plan.realestate.properties.is_empty(),
            "{name}: Immobilien übernommen"
        );
        assert!(
            imp.notes.iter().any(|n| n.contains("Immobilien")),
            "{name}: Hinweis zu Immobilien"
        );
        let has_mc = std::fs::read_to_string(&f).unwrap().contains("\"MonteCarlo\"");
        assert_eq!(
            imp.plan.returns.parametric.is_some(),
            has_mc,
            "{name}: Monte-Carlo-Parameter"
        );
        runs(&imp, &dir);
    }
}

#[test]
fn rejects_garbage() {
    assert!(legacy::import("kein json", 2026, Path::new("."), "x").is_err());
    assert!(legacy::import("{}", 2026, Path::new("."), "x").is_err());
}
