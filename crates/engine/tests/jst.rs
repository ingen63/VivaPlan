//! #67/#68: World index from the JST data and the series «Welt (JST)».
//!
//! Regenerate extract and series: `JST_UPDATE=1 cargo nextest run -p vivaplan-engine -E 'test(/^jst::/)'`.

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

/// The bundled extract matches the raw file of the database.
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

/// #68/AK5: The bundled series «Welt (JST)» is created exactly like this from the extract and the SNB data.
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

/// Value of the series in year `y`.
fn at(v: &[f64], y: i32) -> f64 {
    v[(y - jst::FIRST_YEAR) as usize]
}

/// #67/AK3: Three years recomputed independently, directly from the rows of the extract: return per country in CHF via the
/// rates at the end of the year, weight = GDP per capita × population of the previous year, countries without a return drop out.
/// The raw data and the intermediate values of the years 1950, 1990 and 2008 are in `docs/Weltindex
/// JST.md` (section «Stichjahre 1950, 1990, 2008»).
#[test]
fn issue_67_ak3_three_years_by_hand() {
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
    // For a check by hand: in 2008 the world index in CHF lost about 41 % (financial crisis and strong franc).
    assert!((at(&w.stocks, 2008) + 0.41).abs() < 0.01, "{}", at(&w.stocks, 2008));
}

/// #67/AK1: Weights per year: sum 100 %, the USA from 1950 about half; AK2: the missing country-years are exactly
/// the documented ones.
#[test]
fn issue_67_ak1_ak2_weights_and_gaps() {
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

/// Geometric mean per year.
fn geo(v: impl Iterator<Item = f64>) -> f64 {
    let (mut log, mut n) = (0.0, 0.0);
    for r in v {
        log += (1.0 + r).ln();
        n += 1.0;
    }
    (log / n).exp() - 1.0
}

/// #67/AK4: Metrics for the comparison with DMS (values in `docs/Weltindex JST.md`).
/// DMS (Global Investment Returns Yearbook 2021) gives 5.3 % real return per year in USD for 1900–2020.
#[test]
fn issue_67_ak4_figures_for_the_dms_comparison() {
    let d = JstData::builtin();
    let w = world();
    let ch = |y: i32| d.row("CHE", y).unwrap().xrusd.unwrap();
    let us_cpi = |y: i32| d.row("USA", y).unwrap().cpi.unwrap();
    // CHF → USD: × (USD per CHF today / USD per CHF previous year); real with US inflation.
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
    // Nominal in CHF and real in CHF
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

/// #68/AK2: Swiss Confederation bonds: JST until 1988, from 1989 from the SNB monthly returns; continuous, without
/// a gap at the transition; 1915 the coupon.
#[test]
fn issue_68_ak2_ch_bonds_switch_from_jst_to_snb_without_a_gap() {
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
    // The same order of magnitude on both sides of the transition: no unit swapped (percent instead of decimal).
    for y in 1985..=1992 {
        assert!(at(&bonds, y).abs() < 0.2, "{y}: {}", at(&bonds, y));
    }
}

/// #68/AK1, AK3, AK4: Series «Welt (JST)» 1900 to 2020, annual, in CHF; source, citations and licence in the file header.
#[test]
fn issue_68_world_series_range_and_source() {
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
        "retrieved",
        "rendoblim",
    ] {
        assert!(src.contains(part), "Quelle ohne «{part}»");
    }
    assert!(w.meta.notes.iter().any(|n| n.contains("non-commercial")));
    // Inflation: Swiss consumer prices of the database
    let d = JstData::builtin();
    assert!((at(&w.inflation, 1975) - d.ch_inflation(1975).unwrap()).abs() < 1e-9);
}
