//! Messung der Rechenzeit pro Lauf (Grundlage der Dauerschätzung in der Oberfläche).

use std::path::Path;

use engine::api;
use serde_json::json;

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn plan(source: &str) -> serde_json::Value {
    let mut p = api::example_plan();
    p["returns"]["source"] = json!(source);
    p["returns"]["series"] = json!("library:Shiller CHF.csv");
    p
}

#[test]
fn bench_measures_one_run_for_each_source() {
    let lib = data_dir();
    for source in ["fixed", "bootstrap", "parametric", "rolling"] {
        let b = api::bench(&plan(source), Path::new("/"), Some(&lib)).unwrap_or_else(|e| panic!("{source}: {e}"));
        assert!(
            b.us_per_run > 0.0 && b.us_per_run < 1e6,
            "{source}: {} µs",
            b.us_per_run
        );
        assert!(b.cores >= 1 && b.months > 100, "{source}");
        assert_eq!(b.fixed_paths.is_some(), source == "rolling", "{source}");
    }
    let n = api::bench(&plan("rolling"), Path::new("/"), Some(&lib))
        .unwrap()
        .fixed_paths
        .unwrap();
    assert!(n > 100, "{n}");
}

#[test]
fn bench_caps_the_number_of_paths_and_reports_errors() {
    let mut p = plan("bootstrap");
    p["returns"]["bootstrap"]["paths"] = json!(1_000_000);
    let t0 = std::time::Instant::now();
    api::bench(&p, Path::new("/"), Some(&data_dir())).unwrap();
    assert!(
        t0.elapsed().as_secs_f64() < 20.0,
        "Messung darf nicht von der Pfadzahl abhängen"
    );
    // ohne Bibliothek: verständlicher Fehler statt Absturz
    assert!(api::bench(&plan("bootstrap"), Path::new("/"), None)
        .unwrap_err()
        .contains("Datenbibliothek"));
    assert!(api::bench(&json!({"x": 1}), Path::new("/"), None).is_err());
}
