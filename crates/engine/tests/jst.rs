//! FEA-51/FEA-52: Weltindex aus den JST-Daten und die Reihe «Welt (JST)».
//!
//! Auszug und Reihe neu erzeugen: `JST_UPDATE=1 cargo nextest run -p vivaplan-engine -E 'test(/^jst::/)'`.

use std::path::{Path, PathBuf};

use engine::import::SnbData;
use engine::jst::{self, Asset, JstData};
use engine::market::{DataSet, Freq};
use engine::time::YearMonth;

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn update() -> bool {
    std::env::var("JST_UPDATE").is_ok()
}

/// Der ausgelieferte Auszug entspricht der Rohdatei der Datenbank.
#[test]
fn extract_matches_the_database_file() {
    let raw = JstData::read_xlsx(&data_dir().join("quellen").join(jst::XLSX)).unwrap();
    let path = data_dir().join("quellen").join(jst::EXTRACT);
    if update() {
        std::fs::write(&path, raw.to_csv()).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        raw.to_csv(),
        "mit JST_UPDATE=1 neu erzeugen"
    );
    assert_eq!(JstData::builtin(), raw);
    assert_eq!(raw.last_year(), 2020);
}

/// FEA-52/AK5: Die ausgelieferte Reihe «Welt (JST)» entsteht genau so aus dem Auszug und den SNB-Daten.
#[test]
fn shipped_world_series_matches_the_sources() {
    let ds = jst::world_dataset(&JstData::builtin(), &SnbData::builtin()).unwrap();
    let path = data_dir().join("Welt (JST).csv");
    if update() {
        std::fs::write(&path, ds.to_csv()).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        ds.to_csv(),
        "mit JST_UPDATE=1 neu erzeugen"
    );
}

fn world() -> DataSet {
    DataSet::load_csv(&data_dir().join("Welt (JST).csv")).unwrap()
}

/// Wert der Reihe im Jahr `y`.
fn at(v: &[f64], y: i32) -> f64 {
    v[(y - jst::FIRST_YEAR) as usize]
}

/// FEA-51/AK3: Drei Jahre unabhängig nachgerechnet, direkt aus den Zeilen des Auszugs: Rendite je Land in CHF über die
/// Kurse Ende Jahr, Gewicht = BIP pro Kopf × Bevölkerung des Vorjahres, Länder ohne Rendite fallen weg.
/// Die Rohdaten und die Zwischenwerte der Jahre 1950, 1990 und 2008 stehen in `docs/spezifikation/FEA-51 FEA-52 Weltindex
/// JST.md` (Abschnitt «Stichjahre 1950, 1990, 2008»).
#[test]
fn fea51_ak3_three_years_by_hand() {
    let d = JstData::builtin();
    let countries = [
        "CHE", "USA", "DEU", "FRA", "ITA", "ESP", "NLD", "BEL", "GBR", "JPN", "AUS",
    ];
    let w = world();
    for year in [1950, 1990, 2008] {
        let (mut sum, mut weights) = (0.0, 0.0);
        let ch = |y: i32| d.row("CHE", y).unwrap().xrusd.unwrap();
        for c in countries {
            let (now, before) = (d.row(c, year).unwrap(), d.row(c, year - 1).unwrap());
            let (Some(r), Some(x1), Some(x0)) = (now.eq_tr, now.xrusd, before.xrusd) else {
                continue;
            };
            let in_chf = (1.0 + r) * (ch(year) / x1) / (ch(year - 1) / x0) - 1.0;
            let weight = before.rgdpmad.unwrap() * before.pop.unwrap();
            sum += weight * in_chf;
            weights += weight;
        }
        assert!((at(&w.stocks, year) - sum / weights).abs() < 1e-9, "{year}");
    }
    // Zur Kontrolle von Hand: 2008 verlor der Weltindex in CHF rund 41 % (Finanzkrise und starker Franken).
    assert!((at(&w.stocks, 2008) + 0.41).abs() < 0.01, "{}", at(&w.stocks, 2008));
}

/// FEA-51/AK1: Gewichte je Jahr: Summe 100 %, die USA ab 1950 rund die Hälfte; AK2: die fehlenden Länderjahre sind genau
/// die dokumentierten.
#[test]
fn fea51_ak1_ak2_weights_and_gaps() {
    let d = JstData::builtin();
    let all: Vec<&str> = jst::REGIONS.iter().flat_map(|r| r.countries.iter().copied()).collect();
    for year in jst::FIRST_YEAR..=2020 {
        let w = d.weights(&all, year, Asset::Stocks);
        assert!((w.iter().map(|(_, v)| v).sum::<f64>() - 1.0).abs() < 1e-12, "{year}");
    }
    let usa = |y| {
        d.weights(&all, y, Asset::Stocks)
            .into_iter()
            .find(|(c, _)| c == "USA")
            .unwrap()
            .1
    };
    assert!(
        (0.45..0.55).contains(&usa(1950)) && (0.25..0.35).contains(&usa(1900)),
        "{} {}",
        usa(1950),
        usa(1900)
    );
    let gaps = d.gaps(&all, jst::FIRST_YEAR, 2020, Asset::Stocks);
    let gaps: Vec<(&str, i32)> = gaps.iter().map(|(c, y)| (c.as_str(), *y)).collect();
    assert_eq!(
        gaps,
        [
            ("DEU", 1923),
            ("DEU", 1945),
            ("DEU", 1946),
            ("JPN", 1946),
            ("JPN", 1947)
        ]
    );
}

/// Geometrisches Mittel pro Jahr.
fn geo(v: impl Iterator<Item = f64>) -> f64 {
    let (mut log, mut n) = (0.0, 0.0);
    for r in v {
        log += (1.0 + r).ln();
        n += 1.0;
    }
    (log / n).exp() - 1.0
}

/// FEA-51/AK4: Kennzahlen für den Vergleich mit DMS (Werte in `docs/spezifikation/FEA-51 Weltindex JST.md`).
/// DMS (Global Investment Returns Yearbook 2021) nennt für 1900–2020 5,3 % reale Rendite pro Jahr in USD.
#[test]
fn fea51_ak4_figures_for_the_dms_comparison() {
    let d = JstData::builtin();
    let w = world();
    let ch = |y: i32| d.row("CHE", y).unwrap().xrusd.unwrap();
    let us_cpi = |y: i32| d.row("USA", y).unwrap().cpi.unwrap();
    // CHF → USD: × (USD je CHF heute / USD je CHF Vorjahr); real mit der US-Teuerung.
    let real_usd: Vec<f64> = (jst::FIRST_YEAR..=2020)
        .map(|y| (1.0 + at(&w.stocks, y)) * (ch(y - 1) / ch(y)) / (us_cpi(y) / us_cpi(y - 1)) - 1.0)
        .collect();
    let g = geo(real_usd.iter().copied());
    assert!((g - 0.0656).abs() < 0.0005, "real in USD: {g}");
    let worst = real_usd.iter().copied().fold(f64::INFINITY, f64::min);
    assert!((worst + 0.42).abs() < 0.02, "schlechtestes Jahr real in USD: {worst}");
    let mean = real_usd.iter().sum::<f64>() / real_usd.len() as f64;
    let sd = (real_usd.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / real_usd.len() as f64).sqrt();
    assert!((sd - 0.168).abs() < 0.005, "Schwankung: {sd}");
    // Nominal in CHF und real in CHF
    let nominal = geo(w.stocks.iter().copied());
    let real_chf = geo(w
        .stocks
        .iter()
        .zip(&w.inflation)
        .map(|(r, i)| (1.0 + r) / (1.0 + i) - 1.0));
    assert!(
        (nominal - 0.0796).abs() < 0.0005 && (real_chf - 0.0570).abs() < 0.0005,
        "{nominal} {real_chf}"
    );
}

/// FEA-52/AK2: Obligationen der Eidgenossenschaft: JST bis 1988, ab 1989 aus den SNB-Monatsrenditen; fortlaufend, ohne
/// Lücke beim Übergang; 1915 der Coupon.
#[test]
fn fea52_ak2_ch_bonds_switch_from_jst_to_snb_without_a_gap() {
    let d = JstData::builtin();
    let snb = SnbData::builtin();
    let (first, bonds) = jst::ch_bonds_yearly(&d, &snb).unwrap();
    assert_eq!(first, 1900);
    assert_eq!(first + bonds.len() as i32 - 1, 2024, "letztes ganzes SNB-Jahr");
    let jst_bond = |y| d.row("CHE", y).unwrap().bond_tr.unwrap();
    assert_eq!(at(&bonds, 1988), jst_bond(1988));
    assert_eq!(at(&bonds, 1915), d.row("CHE", 1915).unwrap().bond_rate.unwrap());
    let snb_year = |y: i32| {
        (1..=12)
            .map(|m| {
                let ym = YearMonth::new(y, m);
                let yld = |x: YearMonth| snb.ch_yield10[&x] / 100.0;
                1.0 + engine::import::bond_return(yld(ym.plus(-1)), yld(ym))
            })
            .product::<f64>()
            - 1.0
    };
    assert!((at(&bonds, 1989) - snb_year(1989)).abs() < 1e-12);
    assert!((at(&bonds, 2024) - snb_year(2024)).abs() < 1e-12);
    // Gleiche Grössenordnung beidseits des Übergangs: keine Einheit vertauscht (Prozent statt Dezimalzahl).
    for y in 1985..=1992 {
        assert!(at(&bonds, y).abs() < 0.2, "{y}: {}", at(&bonds, y));
    }
}

/// FEA-52/AK1, AK3, AK4: Reihe «Welt (JST)» 1900 bis 2020, jährlich, in CHF; Quelle, Zitate und Lizenz im Dateikopf.
#[test]
fn fea52_world_series_range_and_source() {
    let w = world();
    assert_eq!(w.freq, Freq::Yearly);
    assert_eq!((w.first, w.last()), (YearMonth::new(1900, 1), YearMonth::new(2020, 12)));
    assert_eq!(w.currency(), Some("CHF"));
    let src = w.meta.source.clone().unwrap();
    for part in [
        "Release 6",
        "CC BY-NC-SA 4.0",
        "Jordà, Schularick, Taylor (2017)",
        "(2019)",
        "abgerufen",
        "rendoblim",
    ] {
        assert!(src.contains(part), "Quelle ohne «{part}»");
    }
    assert!(w.meta.notes.iter().any(|n| n.contains("nicht kommerziell")));
    // Teuerung: Schweizer Konsumentenpreise der Datenbank
    let d = JstData::builtin();
    assert!((at(&w.inflation, 1975) - d.ch_inflation(1975).unwrap()).abs() < 1e-9);
}
