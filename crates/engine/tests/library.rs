//! Data library: import of the Shiller data and computing with the bundled series.

use std::path::{Path, PathBuf};

use engine::market::DataSet;
use engine::time::YearMonth;
use engine::{api, batch, import, library};
use serde_json::json;

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// The bundled series are created exactly like this from crates/engine/data/quellen/shiller-index.ods and the SNB extracts.
#[test]
fn shipped_series_match_import() {
    let shiller = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/quellen/shiller-index.ods");
    // #94: the original spreadsheet only exists locally; without it the test is skipped.
    if !engine::test_resource::DATA.available(shiller.exists()) {
        return;
    }
    let usd = import::shiller_dataset(&import::read_shiller(&shiller).unwrap()).unwrap();
    let snb = import::SnbData::builtin();
    let chf = import::to_chf(&usd, &snb).unwrap();
    let ch = import::with_ch_bonds(&chf, &snb).unwrap();
    for (name, ds) in [
        ("Shiller USD.csv", &usd),
        ("Shiller CHF.csv", &chf),
        (library::CH_BONDS_FILE, &ch),
    ] {
        let shipped = std::fs::read_to_string(data_dir().join(name)).unwrap();
        assert_eq!(shipped, ds.to_csv(), "{name} neu erzeugen: vivaplan import-shiller");
    }
    assert_eq!(usd.first, YearMonth::new(1872, 1));
    assert_eq!(chf.first, YearMonth::new(1922, 1));
    assert_eq!(
        (ch.first, ch.last()),
        (YearMonth::new(1988, 2), YearMonth::new(2025, 7))
    );
    // October 2008: monthly average S&P 968 against 1217 in September, so about −20 %.
    let i = (YearMonth::new(2008, 10).0 - usd.first.0) as usize;
    assert!((usd.stocks[i] + 0.20).abs() < 0.01, "{}", usd.stocks[i]);
}

#[test]
fn plan_uses_library_reference() {
    let mut plan = api::example_plan();
    plan["returns"]["source"] = json!("rolling");
    plan["returns"]["series"] = json!("library:Shiller CHF.csv");
    let out = api::run(&plan, Path::new("/"), Some(&data_dir()));
    assert!(out.error.is_none(), "{:?}", out.error);
    let b = out.batch.expect("mehrere Läufe");
    assert!(b.summary.runs > 100);
    assert!(!out
        .issues
        .iter()
        .any(|i| i.message.contains("Währung") || i.message.contains(" in USD")));

    // World (JST): annual series from 1900 with rolling runs.
    plan["returns"]["series"] = json!("library:Welt (JST).csv");
    let out = api::run(&plan, Path::new("/"), Some(&data_dir()));
    assert!(out.error.is_none(), "{:?}", out.error);
    assert!(out.batch.expect("mehrere Läufe").summary.runs > 10);
    assert!(!out
        .issues
        .iter()
        .any(|i| i.message.contains("Währung") || i.message.contains(" in USD")));

    // USD series in a CHF plan: warning.
    plan["returns"]["series"] = json!("library:Shiller USD.csv");
    let out = api::run(&plan, Path::new("/"), Some(&data_dir()));
    assert!(out
        .issues
        .iter()
        .any(|i| i.section == "returns" && i.message.contains("in USD")));

    // Without a library folder: understandable error.
    let out = api::run(&plan, Path::new("/"), None);
    assert!(out.error.unwrap().contains("data library"));
}

#[test]
fn saron_gaps_use_fixed_value() {
    let p: engine::model::Plan = serde_json::from_value({
        let mut v = api::example_plan();
        v["returns"]["source"] = json!("historical");
        v["returns"]["series"] = json!("library:Shiller CHF.csv");
        v["returns"]["startDate"] = json!("1990-01");
        v["returns"]["fixed"]["saron"] = json!(0.0123);
        v
    })
    .unwrap();
    let sim = engine::sim::Simulator::new(&p).unwrap();
    let ds = batch::load_series(&p, Path::new("/"), Some(&data_dir()))
        .unwrap()
        .unwrap();
    let path = &batch::market_paths(&p, &sim, Some(&ds)).unwrap()[0];
    // Until 05.1999 no SARON in the series → fixed value, afterwards the SNB values.
    let k_may99 = (YearMonth::new(1999, 5).0 - YearMonth::new(1990, 1).0) as usize;
    assert_eq!(path.saron[0], 0.0123);
    assert_eq!(path.saron[k_may99], 0.0123);
    assert!((path.saron[k_may99 + 2] - 0.0065).abs() < 1e-12);
}

#[test]
fn seed_and_list() {
    let _tmp_dir = tempfile::Builder::new().prefix("pp-").tempdir().unwrap();
    let dir = _tmp_dir.path().to_path_buf();
    let _ = std::fs::remove_dir_all(&dir);
    library::seed(&dir).unwrap();
    std::fs::write(dir.join("Kaputt.csv"), "Years,Stocks\n").unwrap();
    let list = library::list(&dir).unwrap();
    let names: Vec<&str> = list.iter().map(|e| e.file.as_str()).collect();
    assert_eq!(
        names,
        [
            "Kaputt.csv",
            library::CH_BONDS_FILE,
            "Shiller CHF.csv",
            "Shiller USD.csv",
            library::WORLD_FILE
        ]
    );
    assert!(list[0].error.is_some());
    let chf = list[2].info.as_ref().unwrap();
    assert_eq!(chf.currency.as_deref(), Some("CHF"));
    assert_eq!(chf.saron_range.unwrap().0, YearMonth::new(1999, 6));
    assert_eq!(list[2].reference, "library:Shiller CHF.csv");
    // Existing files stay unchanged when created again.
    std::fs::write(dir.join("Shiller USD.csv"), "eigene Daten").unwrap();
    library::seed(&dir).unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("Shiller USD.csv")).unwrap(),
        "eigene Daten"
    );
    assert!(DataSet::load_csv(&dir.join("Shiller CHF.csv")).is_ok());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// #90/AK1, AK5: Pictet is no longer bundled and no longer created. A copy created earlier in the
/// library folder stays unchanged and no longer counts as a bundled series (no short info from there).
#[test]
fn issue_90_ak5_pictet_is_not_seeded_and_an_old_copy_stays() {
    assert!(library::BUILTIN.iter().all(|(f, _)| !f.contains("Pictet")));
    let _tmp_dir = tempfile::Builder::new().prefix("pp-").tempdir().unwrap();
    let dir = _tmp_dir.path().to_path_buf();
    let _ = std::fs::remove_dir_all(&dir);
    library::seed(&dir).unwrap();
    assert!(!dir.join("Pictet CH.csv").exists());
    let old = "Years,Stocks,Bonds,Inflation\n2000,0.01,0.02,0.01\n2001,0.03,0.01,0.0\n";
    std::fs::write(dir.join("Pictet CH.csv"), old).unwrap();
    library::seed(&dir).unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("Pictet CH.csv")).unwrap(), old);
    let list = library::list(&dir).unwrap();
    let entry = list.iter().find(|e| e.file == "Pictet CH.csv").unwrap();
    assert_eq!(entry.info.as_ref().unwrap().info, None);
    std::fs::remove_dir_all(&dir).unwrap();
}
