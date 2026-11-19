//! #69: Investment strategy instead of choosing the series.

use std::path::{Path, PathBuf};

use engine::batch;
use engine::jst::{self, JstData};
use engine::market::{DataSet, Freq};
use engine::model::{BondIndex, Investment, Plan, RegionWeights, StockIndex};
use engine::strategy;
use engine::time::YearMonth;
use engine::{api, library};
use serde_json::{json, Value};

fn lib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn ds(stocks: StockIndex, bonds: BondIndex) -> DataSet {
    strategy::dataset(
        &Investment {
            stocks,
            bonds,
            weights: RegionWeights::default(),
        },
        None,
    )
    .unwrap()
}

fn custom(ch: f64, usa: f64, eu: f64, apac: f64) -> DataSet {
    strategy::dataset(
        &Investment {
            stocks: StockIndex::WorldCustom,
            bonds: BondIndex::Ch,
            weights: RegionWeights { ch, usa, eu, apac },
        },
        None,
    )
    .unwrap()
}

fn shipped(file: &str) -> DataSet {
    DataSet::load_csv(&lib().join(file)).unwrap()
}

/// #69/AK3: 100 % USA in «Welt individuell» gives the USA series of the world index; 100 % Switzerland the Swiss one.
#[test]
fn issue_69_ak3_hundred_percent_usa_is_the_usa_series() {
    let d = JstData::builtin();
    assert_eq!(
        custom(0.0, 1.0, 0.0, 0.0).stocks,
        d.region_stocks(jst::USA.countries).unwrap()
    );
    assert_eq!(
        custom(1.0, 0.0, 0.0, 0.0).stocks,
        ds(StockIndex::ChJst, BondIndex::Ch).stocks
    );
}

/// #69/AK2: If the weight of the EU changes, the shares of all EU countries change in the same proportion: the
/// return is the sum of the regions times their weight; within the EU the distribution by GDP stays.
#[test]
fn issue_69_ak2_eu_weight_scales_all_eu_countries_alike() {
    let d = JstData::builtin();
    let eu = d.region_stocks(jst::EU.countries).unwrap();
    let usa = d.region_stocks(jst::USA.countries).unwrap();
    let a = custom(0.0, 0.6, 0.4, 0.0);
    let b = custom(0.0, 0.8, 0.2, 0.0);
    for i in 0..eu.len() {
        assert!((a.stocks[i] - (0.6 * usa[i] + 0.4 * eu[i])).abs() < 1e-12);
        // EU from 40 % to 20 %: the EU part halves (every EU country halves)
        assert!((a.stocks[i] - b.stocks[i] - 0.2 * (eu[i] - usa[i])).abs() < 1e-12);
    }
    // Within the EU by GDP: Germany and France together do not give the simple mean.
    let y = 2000;
    let w = d.weights(jst::EU.countries, y, jst::Asset::Stocks);
    let de = w.iter().find(|(c, _)| c == "DEU").unwrap().1;
    let gb = w.iter().find(|(c, _)| c == "GBR").unwrap().1;
    assert!(de > gb && (w.iter().map(|x| x.1).sum::<f64>() - 1.0).abs() < 1e-12);
}

/// #69/AK2: Weights not 100 % together or outside 0–100 %: error at the field, no run.
#[test]
fn issue_69_ak2_invalid_weights_are_reported_at_the_field() {
    let mut v: Value = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
    v["returns"]["strategy"] =
        json!({ "stocks": "worldCustom", "bonds": "ch", "weights": { "ch": 0.5, "usa": 0.5, "eu": 0.5, "apac": 0 } });
    let issues = api::validate(&v);
    let i = issues
        .iter()
        .find(|i| i.field.as_deref() == Some("returns.strategy.weights"))
        .expect("Fehler an den Gewichten");
    assert_eq!(i.section, "wealth");
    assert!(i.message.contains("150 %"), "{}", i.message);
    // Other stock choice: the weights do not matter
    v["returns"]["strategy"]["stocks"] = json!("worldJst");
    assert!(api::validate(&v)
        .iter()
        .all(|i| i.field.as_deref() != Some("returns.strategy.weights")));
    let bad = Investment {
        stocks: StockIndex::WorldCustom,
        bonds: BondIndex::Ch,
        weights: RegionWeights {
            ch: 0.5,
            usa: 0.6,
            eu: 0.0,
            apac: 0.0,
        },
    };
    assert!(strategy::dataset(&bad, None).unwrap_err().to_string().contains("110 %"));
}

/// Combinations that exist as a bundled series give exactly that series (same results for older plans).
#[test]
fn issue_69_shipped_combinations_are_the_shipped_series() {
    for (s, b, file) in [
        (StockIndex::Sp500, BondIndex::Usa, "Shiller CHF.csv"),
        (StockIndex::Sp500, BondIndex::Ch, library::CH_BONDS_FILE),
    ] {
        let (a, f) = (ds(s, b), shipped(file));
        assert_eq!(
            (a.freq, a.first, &a.stocks, &a.bonds, &a.inflation),
            (f.freq, f.first, &f.stocks, &f.bonds, &f.inflation),
            "{file}"
        );
        assert_eq!(a.saron.is_some(), f.saron.is_some());
    }
    assert_eq!(ds(StockIndex::Sp500, BondIndex::Usa).freq, Freq::Monthly);
}

/// Decision on #69: annual as soon as one part is annual; period = common years; monthly values linked to
/// calendar years, inflation from the source of the stocks.
#[test]
fn issue_69_mixed_strategies_are_yearly_over_the_common_years() {
    let shiller = shipped("Shiller CHF.csv");
    let (short_first, short) = jst::ch_short_yearly(&JstData::builtin(), &engine::import::SnbData::builtin()).unwrap();
    let s = ds(StockIndex::Sp500, BondIndex::ChShort);
    assert_eq!(s.freq, Freq::Yearly);
    assert_eq!(s.first.year(), 1922);
    let i = |y: i32, m: u32| (YearMonth::new(y, m).0 - shiller.first.0) as usize;
    let sp1990 = (1..=12).map(|m| 1.0 + shiller.stocks[i(1990, m)]).product::<f64>() - 1.0;
    let at = |v: &[f64], y: i32| v[(y - s.first.year()) as usize];
    assert!((at(&s.stocks, 1990) - sp1990).abs() < 1e-12);
    assert_eq!(at(&s.inflation, 1990), shiller.inflation[i(1990, 12)]);
    assert_eq!(at(&s.bonds, 1990), short[(1990 - short_first) as usize]);
    // SARON: annual average from the first whole year (2000), fixed value before
    let saron = s.saron.as_ref().unwrap();
    assert!(at(saron, 1999).is_nan());
    let mean2000 = (1..=12)
        .map(|m| shiller.saron.as_ref().unwrap()[i(2000, m)])
        .sum::<f64>()
        / 12.0;
    assert!((at(saron, 2000) - mean2000).abs() < 1e-12);

    // JST stocks with Swiss Confederation bonds: 1900–2020, the same as the series «Welt (JST)»
    let w = ds(StockIndex::WorldJst, BondIndex::Ch);
    let world = shipped(library::WORLD_FILE);
    assert_eq!((w.first, w.last()), (world.first, world.last()));
    for k in 0..w.len() {
        assert!((w.stocks[k] - world.stocks[k]).abs() < 1e-9 && (w.bonds[k] - world.bonds[k]).abs() < 1e-9);
        assert!((w.inflation[k] - world.inflation[k]).abs() < 1e-9);
    }
    // JST stocks with US bonds: inflation from the source of the stocks (JST)
    let p = ds(StockIndex::ChJst, BondIndex::Usa);
    assert_eq!(p.inflation[0], JstData::builtin().ch_inflation(p.first.year()).unwrap());
    assert_eq!(p.meta.currency.as_deref(), Some("CHF"));
}

fn plan_with(series: Option<&str>) -> Value {
    let mut v: Value = serde_json::from_str(include_str!("../assets/beispiel.vplan")).unwrap();
    let r = v["returns"].as_object_mut().unwrap();
    r.remove("strategy");
    if let Some(s) = series {
        r.insert("series".into(), json!(s));
    }
    v
}

/// #69/AK5: Older plans are mapped to the strategy on opening.
#[test]
fn issue_69_ak5_old_series_become_strategies() {
    for (series, stocks, bonds, note) in [
        (Some("library:Shiller CHF.csv"), "sp500", "usa", false),
        (Some("library:Shiller CHF + CH-Obligationen.csv"), "sp500", "ch", false),
        // #90/AK4: Pictet is no longer bundled: Switzerland (JST) with Swiss Confederation bonds, with a note.
        (Some("library:Pictet CH.csv"), "chJst", "ch", true),
        (Some("library:Welt (JST).csv"), "worldJst", "ch", false),
        (Some("library:Shiller USD.csv"), "sp500", "usa", true),
        (Some("/irgendwo/MSCI.csv"), "worldJst", "ch", true),
        (None, "sp500", "usa", false),
    ] {
        let out = api::upgrade_plan(&plan_with(series));
        let r = &out.plan["returns"];
        assert_eq!(r["strategy"], json!({ "stocks": stocks, "bonds": bonds }), "{series:?}");
        assert!(r.get("series").is_none());
        let strategy_notes: Vec<&String> = out
            .notes
            .iter()
            .filter(|n| n.contains("Zeitreihe") || n.contains("Pictet"))
            .collect();
        assert_eq!(!strategy_notes.is_empty(), note, "{series:?}: {:?}", out.notes);
    }
    assert!(api::upgrade_plan(&plan_with(Some("x/MSCI.csv")))
        .notes
        .iter()
        .any(|n| n.contains("«x/MSCI.csv»")));
    // An existing strategy stays unchanged
    let mut v = plan_with(None);
    v["returns"]["strategy"] =
        json!({ "stocks": "worldCustom", "bonds": "usa", "weights": { "ch": 1, "usa": 0, "eu": 0, "apac": 0 } });
    assert_eq!(
        api::upgrade_plan(&v).plan["returns"]["strategy"],
        v["returns"]["strategy"]
    );
}

fn end_totals(v: &Value) -> Vec<f64> {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    let sim = engine::sim::Simulator::new(&plan).unwrap();
    let data = batch::load_series(&plan, Path::new("."), Some(&lib())).unwrap();
    let paths = batch::market_paths(&plan, &sim, data.as_ref()).unwrap();
    batch::run_all(&sim, &paths)
        .iter()
        .map(|r| r.summary.end_total)
        .collect()
}

/// #69/AK5: The plan taken over computes exactly as before (all historical start dates).
#[test]
fn issue_69_ak5_upgraded_plans_give_the_same_results() {
    let series = "library:Shiller CHF.csv";
    let mut old = plan_with(Some(series));
    old["returns"]["source"] = json!("rolling");
    let new = api::upgrade_plan(&old).plan;
    assert!(new["returns"].get("series").is_none());
    let (a, b) = (end_totals(&old), end_totals(&new));
    assert!(a.len() > 5);
    assert_eq!(a, b, "{series}");
}

/// #69/AK4: Fixed returns work as before; the strategy then does not matter.
#[test]
fn issue_69_ak4_fixed_returns_ignore_the_strategy() {
    let mut a = plan_with(None);
    a["returns"]["source"] = json!("fixed");
    let mut b = a.clone();
    b["returns"]["strategy"] = json!({ "stocks": "chJst", "bonds": "ch" });
    assert_eq!(end_totals(&a), end_totals(&b));
}

/// Short info of the strategy for the user interface (`strategy:<JSON>`) and new plans with «Welt (JST)».
#[test]
fn issue_69_strategy_info_and_new_plans() {
    let r = format!("{}{}", strategy::PREFIX, json!({ "stocks": "chJst", "bonds": "usa" }));
    let i = api::series_info(&r, Path::new("."), None).unwrap();
    assert!(!i.monthly);
    assert_eq!((i.first, i.last), (YearMonth::new(1922, 1), YearMonth::new(2020, 12)));
    assert_eq!(i.currency.as_deref(), Some("CHF"));
    assert!(i.title.unwrap().contains("Schweiz (JST)"));
    assert!(api::series_info("strategy:{kaputt", Path::new("."), None).is_err());
    let new_plan: Plan = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
    assert_eq!(new_plan.returns.strategy, Some(Investment::default()));
    assert!(new_plan.returns.series.is_none());
    // With a strategy «rolling» or «bootstrap» no longer needs a series; without both it is an error.
    let mut v: Value = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
    let series_error = |v: &Value| {
        api::validate(v)
            .iter()
            .any(|i| i.message.contains("Zeitreihe gewählt sein"))
    };
    for src in ["rolling", "bootstrap"] {
        v["returns"]["source"] = json!(src);
        assert!(!series_error(&v), "{src}");
    }
    v["returns"].as_object_mut().unwrap().remove("strategy");
    assert!(series_error(&v));
}

/// #69/AK8: The short info names the parts of the strategy with their own years. The means apply to the
/// common period: stocks JST (1900–2020) with bonds USA (from 1922) are cut to 1922–2020; the
/// mean of the bonds is that of these years. Without cutting both parts span exactly the series.
#[test]
fn issue_69_ak8_parts_and_common_period_of_the_averages() {
    let info = |stocks: &str, bonds: &str| {
        let r = format!("{}{}", strategy::PREFIX, json!({ "stocks": stocks, "bonds": bonds }));
        api::series_info(&r, Path::new("."), None).unwrap()
    };
    let years = |p: &library::SeriesPart| (p.label.clone(), p.first, p.last);

    // Stocks JST (1900–2020), bonds USA (Shiller, from 1922): together 1922–2020, both parts are cut.
    let cut = info("chJst", "usa");
    assert_eq!(
        (cut.first, cut.last),
        (YearMonth::new(1922, 1), YearMonth::new(2020, 12))
    );
    assert_eq!(
        years(&cut.parts[0]),
        ("Aktien Schweiz (JST)".to_string(), jst::FIRST_YEAR, 2020)
    );
    let bonds = years(&cut.parts[1]);
    assert_eq!(bonds.0, "Obligationen USA 10 Jahre (Shiller)");
    assert!(bonds.1 <= 1922 && bonds.2 > 2020, "{bonds:?}");
    // The mean of the bonds applies to the common period.
    let geo = |v: &[f64]| (v.iter().map(|r| (1.0 + r).ln()).sum::<f64>() / v.len() as f64).exp() - 1.0;
    let common = ds(StockIndex::ChJst, BondIndex::Usa);
    assert_eq!(common.len(), (2020 - 1922 + 1) as usize);
    assert!((cut.avg_bonds - geo(&common.bonds)).abs() < 1e-12, "{}", cut.avg_bonds);

    // S&P 500 with Swiss Confederation bonds (bundled monthly series from 1988): the stocks are cut.
    let sp = info("sp500", "ch");
    assert!(sp.parts[0].first < sp.first.year(), "{:?}", sp.parts[0]);
    assert_eq!((sp.parts[1].first, sp.parts[1].last), (sp.first.year(), sp.last.year()));

    // A single series has no parts.
    let plain = api::series_info("library:Welt (JST).csv", &lib(), Some(&lib())).unwrap();
    assert!(plain.parts.is_empty());
}

/// #69/AK8: S&P 500 with US bonds is the bundled series; no part is cut. An unreadable
/// reference gives an error, a foreign reference `None`.
#[test]
fn issue_69_ak8_parts_without_cut_and_reference_errors() {
    let inv = Investment {
        stocks: StockIndex::Sp500,
        bonds: BondIndex::Usa,
        weights: RegionWeights::default(),
    };
    let (d, parts) = strategy::dataset_with_parts(&inv, None).unwrap();
    assert_eq!(parts.len(), 2);
    for p in &parts {
        assert_eq!((p.first, p.last), (d.first.year(), d.last().year()), "{}", p.label);
    }
    assert!(parts[0].label.starts_with("Aktien "));
    assert!(parts[1].label.starts_with("Obligationen "));

    let bad = format!("{}{{kaputt", strategy::PREFIX);
    let err = strategy::from_reference_with_parts(&bad, None).unwrap().unwrap_err();
    assert!(err.to_string().contains("Strategie nicht lesbar"), "{err}");
    assert!(strategy::from_reference_with_parts("library:x.csv", None).is_none());
}

/// #76/AK2: «Eidgenossenschaft kurzfristig»: JST (`bill_rate`) until 2020, from 2021 the annual average of SARON, without
/// a gap. The last JST year and the first SARON year recomputed by hand.
#[test]
fn issue_76_ak2_short_bonds_switch_from_jst_to_saron_without_a_gap() {
    let jst = JstData::builtin();
    let snb = engine::import::SnbData::builtin();
    let (first, v) = jst::ch_short_yearly(&jst, &snb).unwrap();
    assert_eq!(first, 1900);
    let at = |y: i32| v[(y - first) as usize];
    assert_eq!(jst.last_year(), 2020);
    // 1900 and 2020 from JST: 3.9 % and −0.85 %
    assert!((at(1900) - 0.039).abs() < 1e-6, "{}", at(1900));
    assert!((at(2020) - (-0.0085)).abs() < 1e-9, "{}", at(2020));
    // 2021: mean of the twelve SARON monthly values (percent) / 100
    let mean2021 = (1..=12).map(|m| snb.saron[&YearMonth::new(2021, m)]).sum::<f64>() / 12.0 / 100.0;
    assert!((at(2021) - mean2021).abs() < 1e-12);
    assert!(at(2021) < 0.0 && at(2021) > -0.01, "{}", at(2021));
    // Without gaps until the last whole SARON year; a started year does not count.
    let last_full = (2021..)
        .take_while(|y| (1..=12).all(|m| snb.saron.contains_key(&YearMonth::new(*y, m))))
        .last()
        .unwrap();
    assert_eq!(first + v.len() as i32 - 1, last_full);
    assert!(v.iter().all(|r| r.is_finite()));
}

/// #76/AK1, AK3: Choice «chShort» in the strategy, annual over the years in common with the stocks.
#[test]
fn issue_76_ak1_ak3_short_bonds_in_strategies() {
    let (first, short) = jst::ch_short_yearly(&JstData::builtin(), &engine::import::SnbData::builtin()).unwrap();
    // Name and storage form
    assert_eq!(strategy::bond_name(BondIndex::ChShort), "Eidgenossenschaft kurzfristig");
    assert_eq!(serde_json::to_value(BondIndex::ChShort).unwrap(), json!("chShort"));
    // World (JST): 1900–2020, stocks like «Welt (JST)», bonds = short-term series
    let w = ds(StockIndex::WorldJst, BondIndex::ChShort);
    let world = shipped(library::WORLD_FILE);
    assert_eq!(w.freq, Freq::Yearly);
    assert_eq!((w.first.year(), w.last().year()), (1900, 2020));
    assert!(short.len() >= w.len());
    for (k, &bond) in short.iter().enumerate().take(w.len()) {
        assert!((w.stocks[k] - world.stocks[k]).abs() < 1e-9);
        assert_eq!(w.bonds[k], bond);
    }
    // S&P 500: annual (not monthly), up to the last whole SARON year
    let s = ds(StockIndex::Sp500, BondIndex::ChShort);
    assert_eq!(s.freq, Freq::Yearly);
    assert_eq!(s.first.year(), 1922);
    assert_eq!(s.last().year(), (first + short.len() as i32 - 1).min(2025));
    let at = |v: &[f64], y: i32| v[(y - s.first.year()) as usize];
    assert_eq!(at(&s.bonds, 2021), short[(2021 - first) as usize]);
    // Source and notes name SARON and the JST licence
    assert!(s.meta.source.as_deref().unwrap().contains("SARON"));
    assert!(s.meta.notes.iter().any(|n| n.contains("Geldmarktsatz")));
    assert!(s.meta.notes.iter().any(|n| n.contains("CC BY-NC-SA")));
    // Short info via the reference of the user interface
    let r = format!("{}{}", strategy::PREFIX, json!({"stocks": "chJst", "bonds": "chShort"}));
    let p = strategy::from_reference(&r, None).unwrap().unwrap();
    assert_eq!((p.first.year(), p.last().year()), (1900, 2020));
    assert!(p.name.contains("Eidgenossenschaft kurzfristig"));
}

fn plan_struct(v: &Value) -> Plan {
    serde_json::from_value(v.clone()).unwrap()
}

/// #69/AK4, AK5: `returns_series` takes `series` if set (older plans, command line), otherwise the
/// strategy; without both there is no series. `load_series` only loads for return sources with a series.
#[test]
fn issue_69_returns_series_prefers_series_then_strategy_then_none() {
    let mut v = plan_with(None);
    v["returns"]["source"] = json!("historical");
    // Strategy only
    v["returns"]["strategy"] = json!({ "stocks": "sp500", "bonds": "ch" });
    let from_strategy = batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib()))
        .unwrap()
        .unwrap();
    assert_eq!(from_strategy.stocks, shipped(library::CH_BONDS_FILE).stocks);
    // Series and strategy: the series applies
    v["returns"]["series"] = json!("library:Welt (JST).csv");
    let from_series = batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib()))
        .unwrap()
        .unwrap();
    assert_eq!(from_series.stocks, shipped(library::WORLD_FILE).stocks);
    assert_ne!(from_series.stocks, from_strategy.stocks);
    // A series that cannot be loaded is an error, even if a strategy were set
    v["returns"]["series"] = json!("library:gibt-es-nicht.csv");
    assert!(batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib())).is_err());
    // Neither
    let o = v["returns"].as_object_mut().unwrap();
    o.remove("series");
    o.remove("strategy");
    assert!(batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib()))
        .unwrap()
        .is_none());
    // load_series: no series with fixed returns, even with a strategy
    v["returns"]["strategy"] = json!({ "stocks": "chJst", "bonds": "ch" });
    for (source, loaded) in [
        ("fixed", false),
        ("historical", true),
        ("rolling", true),
        ("bootstrap", true),
    ] {
        v["returns"]["source"] = json!(source);
        let r = batch::load_series(&plan_struct(&v), Path::new("."), Some(&lib())).unwrap();
        assert_eq!(r.is_some(), loaded, "{source}");
    }
}

/// #69/AK4: With a strategy the preview in the editor computes with the averages of the strategy series.
#[test]
fn issue_69_preview_uses_the_averages_of_the_strategy_series() {
    let mut v = plan_with(None);
    v["returns"]["source"] = json!("historical");
    v["returns"]["startDate"] = json!("1950-01");
    v["returns"]["strategy"] = json!({ "stocks": "chJst", "bonds": "ch" });
    let (s, b, i) = ds(StockIndex::ChJst, BondIndex::Ch).averages();
    let out = api::preview(&v, Path::new("."), Some(&lib()));
    let basis = out.basis.expect("Vorschau nennt ihre Grundlage");
    let pct = |x: f64| format!("{:.1} %", x * 100.0);
    assert!(
        basis.contains(&format!(
            "Aktien {}, Obligationen {}, Inflation {}",
            pct(s),
            pct(b),
            pct(i)
        )),
        "{basis}"
    );
    // Without series and strategy no basis (preview with the fixed values)
    v["returns"].as_object_mut().unwrap().remove("strategy");
    assert!(api::preview(&v, Path::new("."), Some(&lib())).basis.is_none());
}

/// #90/AK3: Plans with the former index «Schweiz (Pictet)» open without error. The stocks become
/// «Schweiz (JST)», the bonds «Eidgenossenschaft 10 Jahre»; a note names the change, and the
/// plan taken over computes.
#[test]
fn issue_90_ak3_pictet_strategies_are_replaced_with_a_note() {
    for (stocks, bonds, want_stocks, want_bonds, with_stocks, with_bonds) in [
        ("chPictet", "chPictet", "chJst", "ch", true, true),
        ("chPictet", "usa", "chJst", "usa", true, false),
        ("sp500", "chPictet", "sp500", "ch", false, true),
    ] {
        let mut v = plan_with(None);
        // Bootstrap: also the short monthly series S&P 500 with Swiss Confederation bonds (from 1988) gives runs.
        v["returns"]["source"] = json!("bootstrap");
        v["returns"]["bootstrap"] = json!({ "paths": 20, "block": 12, "seed": 1 });
        v["returns"]["strategy"] = json!({ "stocks": stocks, "bonds": bonds });
        let out = api::upgrade_plan(&v);
        assert_eq!(
            out.plan["returns"]["strategy"],
            json!({ "stocks": want_stocks, "bonds": want_bonds })
        );
        let notes: Vec<&String> = out.notes.iter().filter(|n| n.contains("«Schweiz (Pictet)»")).collect();
        assert_eq!(notes.len(), 1, "{:?}", out.notes);
        assert_eq!(
            notes[0].contains("für die Aktien «Schweiz (JST)»"),
            with_stocks,
            "{}",
            notes[0]
        );
        assert_eq!(
            notes[0].contains("für die Obligationen «Eidgenossenschaft 10 Jahre»"),
            with_bonds,
            "{}",
            notes[0]
        );
        assert!(!end_totals(&out.plan).is_empty(), "{stocks}/{bonds}");
    }
    // Without Pictet no note.
    let mut v = plan_with(None);
    v["returns"]["strategy"] = json!({ "stocks": "chJst", "bonds": "ch" });
    assert!(api::upgrade_plan(&v).notes.iter().all(|n| !n.contains("Pictet")));
    // Without the takeover the former key is no longer a valid index.
    assert!(serde_json::from_value::<StockIndex>(json!("chPictet")).is_err());
    assert!(serde_json::from_value::<BondIndex>(json!("chPictet")).is_err());
}

/// Library with a Shiller series only for the calendar year `year` (12 months, 1 % each, SARON 2 %).
fn lib_with_shiller_year(year: i32) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vivaplan-fea97-{}-{year}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut csv = String::from("# Titel: Test\n# Währung: CHF\nDate,Stocks,Bonds,Inflation,Saron\n");
    for m in 1..=12 {
        csv += &format!("{year}-{m:02},0.01,0.01,0.01,0.02\n");
    }
    std::fs::write(dir.join("Shiller CHF.csv"), csv).unwrap();
    dir
}

/// #124/AK8: Stocks of the world index (JST) with bonds USA (Shiller): a single common year suffices; SARON
/// also applies in the first and last year of its series; without a common year the strategy reports an error.
#[test]
fn issue_124_ak8_common_years_of_the_indices() {
    let s = Investment {
        stocks: StockIndex::WorldJst,
        bonds: BondIndex::Usa,
        weights: RegionWeights::default(),
    };
    let last = JstData::builtin().last_year();
    let one = lib_with_shiller_year(last);
    let r = strategy::dataset(&s, Some(&one));
    let late = lib_with_shiller_year(last + 5);
    let e = strategy::dataset(&s, Some(&late));
    let _ = std::fs::remove_dir_all(one);
    let _ = std::fs::remove_dir_all(late);
    let d = r.expect("ein gemeinsames Jahr");
    assert_eq!((d.first, d.stocks.len()), (YearMonth::new(last, 1), 1));
    assert_eq!(d.saron.as_deref().map(|s| s.len()), Some(1));
    assert!((d.saron.unwrap()[0] - 0.02).abs() < 1e-12, "SARON im einzigen Jahr");
    assert!(e.unwrap_err().to_string().contains("keine gemeinsamen Jahre"));
}
