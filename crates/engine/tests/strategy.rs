//! FEA-53: Anlagestrategie statt Wahl der Zeitreihe.

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

/// FEA-53/AK3: 100 % USA in «Welt individuell» ergibt die USA-Reihe des Weltindexes; 100 % Schweiz die Schweizer.
#[test]
fn fea53_ak3_hundred_percent_usa_is_the_usa_series() {
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

/// FEA-53/AK2: Ändert sich das Gewicht der EU, ändern sich die Anteile aller EU-Länder im gleichen Verhältnis: Die
/// Rendite ist die Summe der Regionen mal ihr Gewicht; innerhalb der EU bleibt die Verteilung nach BIP.
#[test]
fn fea53_ak2_eu_weight_scales_all_eu_countries_alike() {
    let d = JstData::builtin();
    let eu = d.region_stocks(jst::EU.countries).unwrap();
    let usa = d.region_stocks(jst::USA.countries).unwrap();
    let a = custom(0.0, 0.6, 0.4, 0.0);
    let b = custom(0.0, 0.8, 0.2, 0.0);
    for i in 0..eu.len() {
        assert!((a.stocks[i] - (0.6 * usa[i] + 0.4 * eu[i])).abs() < 1e-12);
        // EU von 40 % auf 20 %: der EU-Teil halbiert sich (jedes EU-Land halbiert)
        assert!((a.stocks[i] - b.stocks[i] - 0.2 * (eu[i] - usa[i])).abs() < 1e-12);
    }
    // Innerhalb der EU nach BIP: Deutschland und Frankreich zusammen ergeben nicht das einfache Mittel.
    let y = 2000;
    let w = d.weights(jst::EU.countries, y, jst::Asset::Stocks);
    let de = w.iter().find(|(c, _)| c == "DEU").unwrap().1;
    let gb = w.iter().find(|(c, _)| c == "GBR").unwrap().1;
    assert!(de > gb && (w.iter().map(|x| x.1).sum::<f64>() - 1.0).abs() < 1e-12);
}

/// FEA-53/AK2: Gewichte zusammen nicht 100 % oder ausserhalb 0–100 %: Fehler beim Feld, kein Lauf.
#[test]
fn fea53_ak2_invalid_weights_are_reported_at_the_field() {
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
    // Andere Aktienwahl: die Gewichte spielen keine Rolle
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

/// Kombinationen, die es als ausgelieferte Reihe gibt, liefern genau diese (gleiche Ergebnisse für ältere Planungen).
#[test]
fn fea53_shipped_combinations_are_the_shipped_series() {
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

/// Entscheid zu FEA-53: jährlich, sobald ein Teil jährlich ist; Zeitraum = gemeinsame Jahre; Monatswerte zu
/// Kalenderjahren verknüpft, Teuerung aus der Quelle der Aktien.
#[test]
fn fea53_mixed_strategies_are_yearly_over_the_common_years() {
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
    // SARON: Jahresmittel ab dem ersten ganzen Jahr (2000), vorher fixer Wert
    let saron = s.saron.as_ref().unwrap();
    assert!(at(saron, 1999).is_nan());
    let mean2000 = (1..=12)
        .map(|m| shiller.saron.as_ref().unwrap()[i(2000, m)])
        .sum::<f64>()
        / 12.0;
    assert!((at(saron, 2000) - mean2000).abs() < 1e-12);

    // JST-Aktien mit Obligationen der Eidgenossenschaft: 1900–2020, gleich wie die Reihe «Welt (JST)»
    let w = ds(StockIndex::WorldJst, BondIndex::Ch);
    let world = shipped(library::WORLD_FILE);
    assert_eq!((w.first, w.last()), (world.first, world.last()));
    for k in 0..w.len() {
        assert!((w.stocks[k] - world.stocks[k]).abs() < 1e-9 && (w.bonds[k] - world.bonds[k]).abs() < 1e-9);
        assert!((w.inflation[k] - world.inflation[k]).abs() < 1e-9);
    }
    // JST-Aktien mit US-Obligationen: Teuerung aus der Quelle der Aktien (JST)
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

/// FEA-53/AK5: Ältere Planungen werden beim Öffnen auf die Strategie abgebildet.
#[test]
fn fea53_ak5_old_series_become_strategies() {
    for (series, stocks, bonds, note) in [
        (Some("library:Shiller CHF.csv"), "sp500", "usa", false),
        (Some("library:Shiller CHF + CH-Obligationen.csv"), "sp500", "ch", false),
        // FEA-72/AK4: Pictet wird nicht mehr mitgeliefert: Schweiz (JST) mit Eidgenossenschaft, mit Hinweis.
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
    // Eine vorhandene Strategie bleibt unverändert
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

/// FEA-53/AK5: Die übernommene Planung rechnet genau wie vorher (alle historischen Startzeitpunkte).
#[test]
fn fea53_ak5_upgraded_plans_give_the_same_results() {
    let series = "library:Shiller CHF.csv";
    let mut old = plan_with(Some(series));
    old["returns"]["source"] = json!("rolling");
    let new = api::upgrade_plan(&old).plan;
    assert!(new["returns"].get("series").is_none());
    let (a, b) = (end_totals(&old), end_totals(&new));
    assert!(a.len() > 5);
    assert_eq!(a, b, "{series}");
}

/// FEA-53/AK4: Fixe Renditen funktionieren wie bisher; die Strategie spielt dann keine Rolle.
#[test]
fn fea53_ak4_fixed_returns_ignore_the_strategy() {
    let mut a = plan_with(None);
    a["returns"]["source"] = json!("fixed");
    let mut b = a.clone();
    b["returns"]["strategy"] = json!({ "stocks": "chJst", "bonds": "ch" });
    assert_eq!(end_totals(&a), end_totals(&b));
}

/// Kurzinfo der Strategie für die Oberfläche (`strategy:<JSON>`) und neue Planungen mit «Welt (JST)».
#[test]
fn fea53_strategy_info_and_new_plans() {
    let r = format!("{}{}", strategy::PREFIX, json!({ "stocks": "chJst", "bonds": "usa" }));
    let i = api::series_info(&r, Path::new("."), None).unwrap();
    assert!(!i.monthly);
    assert_eq!((i.first, i.last), (YearMonth::new(1922, 1), YearMonth::new(2020, 12)));
    assert_eq!(i.currency.as_deref(), Some("CHF"));
    assert!(i.title.unwrap().contains("Schweiz (JST)"));
    assert!(api::series_info("strategy:{kaputt", Path::new("."), None).is_err());
    let neu: Plan = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
    assert_eq!(neu.returns.strategy, Some(Investment::default()));
    assert!(neu.returns.series.is_none());
    // Mit Strategie braucht «rolling» bzw. «bootstrap» keine Zeitreihe mehr; ohne beides ist es ein Fehler.
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

/// FEA-53/AK8: Die Kurzinfo nennt die Teile der Strategie mit ihren eigenen Jahren. Die Mittelwerte gelten für den
/// gemeinsamen Zeitraum: Aktien JST (1900–2020) mit Obligationen USA (ab 1922) sind auf 1922–2020 gekürzt, das
/// Mittel der Obligationen ist das dieser Jahre. Ohne Kürzung reichen beide Teile genau über die Reihe.
#[test]
fn fea53_ak8_parts_and_common_period_of_the_averages() {
    let info = |stocks: &str, bonds: &str| {
        let r = format!("{}{}", strategy::PREFIX, json!({ "stocks": stocks, "bonds": bonds }));
        api::series_info(&r, Path::new("."), None).unwrap()
    };
    let years = |p: &library::SeriesPart| (p.label.clone(), p.first, p.last);

    // Aktien JST (1900–2020), Obligationen USA (Shiller, ab 1922): gemeinsam 1922–2020, beide Teile sind gekürzt.
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
    // Das Mittel der Obligationen gilt für den gemeinsamen Zeitraum.
    let geo = |v: &[f64]| (v.iter().map(|r| (1.0 + r).ln()).sum::<f64>() / v.len() as f64).exp() - 1.0;
    let common = ds(StockIndex::ChJst, BondIndex::Usa);
    assert_eq!(common.len(), (2020 - 1922 + 1) as usize);
    assert!((cut.avg_bonds - geo(&common.bonds)).abs() < 1e-12, "{}", cut.avg_bonds);

    // S&P 500 mit Obligationen der Eidgenossenschaft (ausgelieferte Monatsreihe ab 1988): die Aktien sind gekürzt.
    let sp = info("sp500", "ch");
    assert!(sp.parts[0].first < sp.first.year(), "{:?}", sp.parts[0]);
    assert_eq!((sp.parts[1].first, sp.parts[1].last), (sp.first.year(), sp.last.year()));

    // Eine einzelne Zeitreihe hat keine Teile.
    let plain = api::series_info("library:Welt (JST).csv", &lib(), Some(&lib())).unwrap();
    assert!(plain.parts.is_empty());
}

/// FEA-53/AK8: S&P 500 mit US-Obligationen ist die ausgelieferte Reihe; kein Teil ist gekürzt. Ein unlesbarer
/// Verweis gibt einen Fehler, ein fremder Verweis `None`.
#[test]
fn fea53_ak8_parts_without_cut_and_reference_errors() {
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

/// FEA-59/AK2: «Eidgenossenschaft kurzfristig»: JST (`bill_rate`) bis 2020, ab 2021 das Jahresmittel des SARON, ohne
/// Lücke. Das letzte JST-Jahr und das erste SARON-Jahr von Hand nachgerechnet.
#[test]
fn fea59_ak2_short_bonds_switch_from_jst_to_saron_without_a_gap() {
    let jst = JstData::builtin();
    let snb = engine::import::SnbData::builtin();
    let (first, v) = jst::ch_short_yearly(&jst, &snb).unwrap();
    assert_eq!(first, 1900);
    let at = |y: i32| v[(y - first) as usize];
    assert_eq!(jst.last_year(), 2020);
    // 1900 und 2020 aus JST: 3.9 % und −0.85 %
    assert!((at(1900) - 0.039).abs() < 1e-6, "{}", at(1900));
    assert!((at(2020) - (-0.0085)).abs() < 1e-9, "{}", at(2020));
    // 2021: Mittel der zwölf SARON-Monatswerte (Prozent) / 100
    let mean2021 = (1..=12).map(|m| snb.saron[&YearMonth::new(2021, m)]).sum::<f64>() / 12.0 / 100.0;
    assert!((at(2021) - mean2021).abs() < 1e-12);
    assert!(at(2021) < 0.0 && at(2021) > -0.01, "{}", at(2021));
    // Lückenlos bis zum letzten ganzen SARON-Jahr; ein angefangenes Jahr zählt nicht.
    let last_full = (2021..)
        .take_while(|y| (1..=12).all(|m| snb.saron.contains_key(&YearMonth::new(*y, m))))
        .last()
        .unwrap();
    assert_eq!(first + v.len() as i32 - 1, last_full);
    assert!(v.iter().all(|r| r.is_finite()));
}

/// FEA-59/AK1, AK3: Wahl «chShort» in der Strategie, jährlich über die gemeinsamen Jahre mit den Aktien.
#[test]
fn fea59_ak1_ak3_short_bonds_in_strategies() {
    let (first, short) = jst::ch_short_yearly(&JstData::builtin(), &engine::import::SnbData::builtin()).unwrap();
    // Name und Speicherform
    assert_eq!(strategy::bond_name(BondIndex::ChShort), "Eidgenossenschaft kurzfristig");
    assert_eq!(serde_json::to_value(BondIndex::ChShort).unwrap(), json!("chShort"));
    // Welt (JST): 1900–2020, Aktien wie «Welt (JST)», Obligationen = kurzfristige Reihe
    let w = ds(StockIndex::WorldJst, BondIndex::ChShort);
    let world = shipped(library::WORLD_FILE);
    assert_eq!(w.freq, Freq::Yearly);
    assert_eq!((w.first.year(), w.last().year()), (1900, 2020));
    assert!(short.len() >= w.len());
    for (k, &bond) in short.iter().enumerate().take(w.len()) {
        assert!((w.stocks[k] - world.stocks[k]).abs() < 1e-9);
        assert_eq!(w.bonds[k], bond);
    }
    // S&P 500: jährlich (nicht monatlich), bis ins letzte ganze SARON-Jahr
    let s = ds(StockIndex::Sp500, BondIndex::ChShort);
    assert_eq!(s.freq, Freq::Yearly);
    assert_eq!(s.first.year(), 1922);
    assert_eq!(s.last().year(), (first + short.len() as i32 - 1).min(2025));
    let at = |v: &[f64], y: i32| v[(y - s.first.year()) as usize];
    assert_eq!(at(&s.bonds, 2021), short[(2021 - first) as usize]);
    // Quelle und Hinweise nennen SARON und die JST-Lizenz
    assert!(s.meta.source.as_deref().unwrap().contains("SARON"));
    assert!(s.meta.notes.iter().any(|n| n.contains("Geldmarktsatz")));
    assert!(s.meta.notes.iter().any(|n| n.contains("CC BY-NC-SA")));
    // Kurzinfo über den Verweis der Oberfläche
    let r = format!("{}{}", strategy::PREFIX, json!({"stocks": "chJst", "bonds": "chShort"}));
    let p = strategy::from_reference(&r, None).unwrap().unwrap();
    assert_eq!((p.first.year(), p.last().year()), (1900, 2020));
    assert!(p.name.contains("Eidgenossenschaft kurzfristig"));
}

fn plan_struct(v: &Value) -> Plan {
    serde_json::from_value(v.clone()).unwrap()
}

/// FEA-53/AK4, AK5: `returns_series` nimmt `series`, falls gesetzt (ältere Planungen, Kommandozeile), sonst die
/// Strategie; ohne beides gibt es keine Reihe. `load_series` lädt nur bei Renditequellen mit Zeitreihe.
#[test]
fn fea53_returns_series_prefers_series_then_strategy_then_none() {
    let mut v = plan_with(None);
    v["returns"]["source"] = json!("historical");
    // Nur Strategie
    v["returns"]["strategy"] = json!({ "stocks": "sp500", "bonds": "ch" });
    let from_strategy = batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib()))
        .unwrap()
        .unwrap();
    assert_eq!(from_strategy.stocks, shipped(library::CH_BONDS_FILE).stocks);
    // Reihe und Strategie: die Reihe gilt
    v["returns"]["series"] = json!("library:Welt (JST).csv");
    let from_series = batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib()))
        .unwrap()
        .unwrap();
    assert_eq!(from_series.stocks, shipped(library::WORLD_FILE).stocks);
    assert_ne!(from_series.stocks, from_strategy.stocks);
    // Eine nicht ladbare Reihe ist ein Fehler, auch wenn eine Strategie gesetzt wäre
    v["returns"]["series"] = json!("library:gibt-es-nicht.csv");
    assert!(batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib())).is_err());
    // Weder noch
    let o = v["returns"].as_object_mut().unwrap();
    o.remove("series");
    o.remove("strategy");
    assert!(batch::returns_series(&plan_struct(&v), Path::new("."), Some(&lib()))
        .unwrap()
        .is_none());
    // load_series: bei fixen Renditen keine Reihe, auch mit Strategie
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

/// FEA-53/AK4: Die Vorschau im Editor rechnet bei einer Strategie mit den Durchschnitten der Strategie-Reihe.
#[test]
fn fea53_preview_uses_the_averages_of_the_strategy_series() {
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
    // Ohne Reihe und Strategie keine Grundlage (Vorschau mit den fixen Werten)
    v["returns"].as_object_mut().unwrap().remove("strategy");
    assert!(api::preview(&v, Path::new("."), Some(&lib())).basis.is_none());
}

/// FEA-72/AK3: Planungen mit dem früheren Index «Schweiz (Pictet)» öffnen ohne Fehler. Die Aktien werden
/// «Schweiz (JST)», die Obligationen «Eidgenossenschaft 10 Jahre»; ein Hinweis nennt die Umstellung, und die
/// übernommene Planung rechnet.
#[test]
fn fea72_ak3_pictet_strategies_are_replaced_with_a_note() {
    for (stocks, bonds, want_stocks, want_bonds, with_stocks, with_bonds) in [
        ("chPictet", "chPictet", "chJst", "ch", true, true),
        ("chPictet", "usa", "chJst", "usa", true, false),
        ("sp500", "chPictet", "sp500", "ch", false, true),
    ] {
        let mut v = plan_with(None);
        // Bootstrap: auch die kurze Monatsreihe S&P 500 mit Eidgenossenschaft (ab 1988) liefert Läufe.
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
    // Ohne Pictet kein Hinweis.
    let mut v = plan_with(None);
    v["returns"]["strategy"] = json!({ "stocks": "chJst", "bonds": "ch" });
    assert!(api::upgrade_plan(&v).notes.iter().all(|n| !n.contains("Pictet")));
    // Ohne Übernahme ist der frühere Schlüssel kein gültiger Index mehr.
    assert!(serde_json::from_value::<StockIndex>(json!("chPictet")).is_err());
    assert!(serde_json::from_value::<BondIndex>(json!("chPictet")).is_err());
}

/// Bibliothek mit einer Shiller-Reihe nur für das Kalenderjahr `year` (12 Monate, je 1 %, SARON 2 %).
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

/// FEA-97/AK8: Aktien des Weltindex (JST) mit Obligationen USA (Shiller): Ein einziges gemeinsames Jahr genügt, der SARON
/// gilt auch im ersten und letzten Jahr seiner Reihe; ohne gemeinsames Jahr meldet die Strategie einen Fehler.
#[test]
fn fea97_ak8_common_years_of_the_indices() {
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
