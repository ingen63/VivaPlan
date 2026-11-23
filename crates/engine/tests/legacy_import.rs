//! Import of old `config.json`: the result is a valid plan that can be computed. (The previous program is
//! not a reference for the values; what is checked is the takeover of the data.)

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

/// #92: All `.json` files of a folder, sorted by name; if the folder is missing, the list is empty.
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

/// #92/AK1, AK2: The list takes every `.json` file, without fixed names, and is empty if the folder is missing.
#[test]
fn issue_92_ak1_ak2_json_files_without_fixed_names() {
    let _tmp_dir = tempfile::Builder::new().prefix("pp-").tempdir().unwrap();
    let dir = _tmp_dir.path().to_path_buf();
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

/// Own configurations with properties in `local/eigene/` (local only, excluded from Git; if they are missing,
/// the test is skipped). The series `./data/MSCI.csv` of the configurations is in `local/eigene/data/`.
/// #92: every `.json` file of the folder is checked, without fixed file names.
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

/// #161/AK1, AK2: An extreme start age (fuzzer finding, outside the range of `i32`) does not crash the
/// import, neither negative nor positive. The input is exactly the finding of 18.11.2026.
#[test]
fn issue_161_ak1_ak2_extreme_start_age_does_not_panic() {
    let fund = "{  \"General\": {\"StartAge\": -566666666666666666666666666 \n}}\n";
    let _ = legacy::import(fund, 2030, Path::new("."), "fuzz");
    for age in ["-1e300", "1e300", "-2147483648", "2147483647", "-0.0"] {
        let text = format!("{{\"General\": {{\"StartAge\": {age}}}}}");
        let _ = legacy::import(&text, 2030, Path::new("."), "x");
    }
}
