//! FEA-78: Aktienparameter des parametrischen Modells je Aktienindex. FEA-79: Obligationen und Inflation mit
//! Korrelation je Kombination aus Aktien- und Obligationenindex.

use std::path::{Path, PathBuf};

use engine::api;
use engine::calibrate::{self, Correlated, Moments};
use engine::market::{DataSet, FixedCurves, Freq, MarketPath};
use engine::model::{BondIndex, BootstrapParams, Investment, ParametricParams, RegionWeights, StockIndex};
use engine::montecarlo::{Bootstrap, Parametric};
use engine::resolve::Curve;
use engine::strategy;
use engine::time::YearMonth;
use rayon::prelude::*;
use serde_json::{json, Value};

fn lib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn inv(stocks: StockIndex, bonds: BondIndex) -> Investment {
    Investment {
        stocks,
        bonds,
        weights: RegionWeights::default(),
    }
}

fn ds(stocks: StockIndex, bonds: BondIndex) -> DataSet {
    strategy::dataset(&inv(stocks, bonds), Some(&lib())).unwrap()
}

fn fixed() -> FixedCurves {
    FixedCurves {
        stocks: Curve::constant(0.0),
        bonds: Curve::constant(0.0),
        inflation: Curve::constant(0.0),
        saron: Curve::constant(0.0),
    }
}

const YEARS: usize = 30;
const PATHS: usize = 3000;

fn start() -> YearMonth {
    YearMonth::new(2030, 1)
}

/// Parametrische Pfade mit den Vorschlagswerten der Reihe (wie eine Planung mit `auto`).
fn parametric_paths(d: &DataSet, paths: usize, correlated: bool) -> Vec<MarketPath> {
    let f = fixed();
    let auto = ParametricParams {
        paths,
        seed: 7,
        auto: true,
        ..Default::default()
    };
    let (params, c) = calibrate::resolve(&auto, Some(d));
    let g = Parametric {
        fixed: &f,
        start: start(),
        months: YEARS * 12,
        params,
        correlated: if correlated { c } else { None },
    };
    (0..paths).into_par_iter().map(|i| g.path(i)).collect()
}

/// Historischer Bootstrap der Reihe mit den Standardwerten (Blocklänge 12 Monate).
fn bootstrap_paths(d: &DataSet, paths: usize) -> Vec<MarketPath> {
    let f = fixed();
    let b = Bootstrap {
        data: d,
        fixed: &f,
        start: start(),
        months: YEARS * 12,
        params: BootstrapParams {
            paths,
            block: 12.0,
            seed: 7,
        },
    };
    (0..paths).into_par_iter().map(|i| b.path(i)).collect()
}

/// Jahreswerte eines Pfades: Aktien und Obligationen verknüpft, Inflation (Jahressatz) des Dezembers.
fn years_of(p: &MarketPath) -> Vec<(f64, f64, f64)> {
    (0..YEARS)
        .map(|y| {
            let r = |v: &[f64]| v[12 * y..12 * y + 12].iter().map(|x| 1.0 + x).product::<f64>() - 1.0;
            (r(&p.stocks), r(&p.bonds), p.inflation[12 * y + 11])
        })
        .collect()
}

fn all_years(paths: &[MarketPath]) -> Vec<(f64, f64, f64)> {
    paths.iter().flat_map(years_of).collect()
}

/// 5., 50. und 95. Perzentil der geometrischen Aktienrendite über 30 Jahre.
fn geo30(paths: &[MarketPath]) -> [f64; 3] {
    let g: Vec<f64> = paths
        .iter()
        .map(|p| {
            let l: f64 = p.stocks.iter().map(|r| (1.0 + r).ln()).sum();
            (l / YEARS as f64).exp() - 1.0
        })
        .collect();
    [
        calibrate::quantile(&g, 0.05),
        calibrate::quantile(&g, 0.5),
        calibrate::quantile(&g, 0.95),
    ]
}

const INDICES: [(StockIndex, BondIndex, &str); 3] = [
    (StockIndex::WorldJst, BondIndex::Ch, "Welt (JST)"),
    (StockIndex::ChJst, BondIndex::Ch, "Schweiz (JST)"),
    (StockIndex::Sp500, BondIndex::Usa, "S&P 500"),
];

/// FEA-78/AK1, AK2: Mittelwert, Streuung und geometrisches Mittel der Jahresrenditen höchstens 0,5 Prozentpunkte vom
/// Bootstrap desselben Index entfernt, das 5 %-Quantil höchstens 2 Prozentpunkte, die geometrische Rendite über
/// 30 Jahre (5., 50., 95. Perzentil) höchstens 0,5 Prozentpunkte.
#[test]
fn fea78_ak1_ak2_parametric_matches_the_bootstrap_of_each_index() {
    let mut report = vec![];
    for (s, b, name) in INDICES {
        let d = ds(s, b);
        let par = parametric_paths(&d, PATHS, false);
        let boot = bootstrap_paths(&d, PATHS);
        let stocks = |p: &[MarketPath]| all_years(p).iter().map(|t| t.0).collect::<Vec<_>>();
        let (mp, mb): (Moments, Moments) = (calibrate::moments(&stocks(&par)), calibrate::moments(&stocks(&boot)));
        let (gp, gb) = (geo30(&par), geo30(&boot));
        report.push(format!("{name}: parametrisch {mp:?} {gp:?}, Bootstrap {mb:?} {gb:?}"));
        assert!((mp.mean - mb.mean).abs() <= 0.005, "{name} Mittelwert: {report:?}");
        assert!((mp.sd - mb.sd).abs() <= 0.005, "{name} Streuung: {report:?}");
        assert!((mp.geo - mb.geo).abs() <= 0.005, "{name} geometrisch: {report:?}");
        assert!((mp.q5 - mb.q5).abs() <= 0.02, "{name} 5 %-Quantil: {report:?}");
        for k in 0..3 {
            assert!((gp[k] - gb[k]).abs() <= 0.005, "{name} 30 Jahre: {report:?}");
        }
    }
    println!("{}", report.join("\n"));
}

/// FEA-78/AK1 (Referenz aus der Analyse): Kennzahlen der Jahresreihen der drei Indizes.
#[test]
fn fea78_ak1_reference_values_of_the_series() {
    for ((s, b, name), (mean, sd, geo)) in
        INDICES
            .into_iter()
            .zip([(0.094, 0.170, 0.080), (0.087, 0.183, 0.071), (0.111, 0.221, 0.088)])
    {
        let d = ds(s, b);
        let st = d.stock_series.clone().unwrap();
        // Kalenderjahre (Monatsreihe ab Januar zu Jahren verknüpft)
        let years: Vec<f64> = match st.freq {
            Freq::Yearly => st.values.clone(),
            Freq::Monthly => st
                .values
                .as_chunks::<12>()
                .0
                .iter()
                .map(|m| m.iter().map(|r| 1.0 + r).product::<f64>() - 1.0)
                .collect(),
        };
        let m = calibrate::moments(&years);
        // Streuung der Analyse mit Nenner n − 1, hier n: Abweichung unter 0,1 Prozentpunkten
        assert!((m.mean - mean).abs() < 0.001, "{name} {m:?}");
        assert!((m.sd - sd).abs() < 0.002, "{name} {m:?}");
        assert!((m.geo - geo).abs() < 0.001, "{name} {m:?}");
    }
}

/// FEA-78/AK3: Die Vorschlagswerte entstehen zur Laufzeit aus den ausgelieferten Reihen.
#[test]
fn fea78_ak3_suggestions_for_the_shipped_series() {
    let fit = |s, b| calibrate::suggestion(&ds(s, b)).stocks.unwrap();
    let expected = [
        (StockIndex::WorldJst, BondIndex::Ch, [0.1215, 0.1225, 0.15, -0.25]),
        (StockIndex::ChJst, BondIndex::Ch, [0.1055, 0.145, 0.26, -0.1]),
        (StockIndex::Sp500, BondIndex::Usa, [0.1505, 0.165, 0.26, -0.2]),
    ];
    let mut got = vec![];
    for (s, b, _) in expected {
        let f = fit(s, b);
        got.push([f.mu, f.sigma, f.jump_lambda, f.jump_mu]);
        assert_eq!((f.df, f.jump_sigma), (6.0, 0.15));
    }
    for ((_, _, e), g) in expected.iter().zip(&got) {
        for (a, b) in e.iter().zip(g) {
            assert!((a - b).abs() < 1e-9, "erwartet {expected:?}, erhalten {got:?}");
        }
    }
    // Die Aktienparameter hängen nur vom Aktienindex ab, nicht von den Obligationen
    assert_eq!(
        fit(StockIndex::Sp500, BondIndex::Ch),
        fit(StockIndex::Sp500, BondIndex::Usa)
    );
    assert_eq!(
        fit(StockIndex::WorldJst, BondIndex::ChShort),
        fit(StockIndex::WorldJst, BondIndex::Usa)
    );
    // S&P 500 mit Eidgenossen: ganze Aktienreihe ab 1922, nicht nur die gemeinsamen Jahre ab 1988
    let s = calibrate::suggestion(&ds(StockIndex::Sp500, BondIndex::Ch));
    assert_eq!(s.stock_years.unwrap().0, 1922);
}

/// FEA-78/AK3: Nach einem Update der Shiller-Daten ändern sich die Vorschlagswerte für «S&P 500».
#[test]
fn fea78_ak3_new_shiller_data_changes_the_suggestion() {
    let dir = std::env::temp_dir().join(format!("vivaplan-fea78-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let text = std::fs::read_to_string(lib().join("Shiller CHF.csv")).unwrap();
    // Gekürzte Reihe (bis 1999) als «neue» Daten
    let cut: String = text
        .lines()
        .filter(|l| !l.starts_with(|c: char| c.is_ascii_digit()) || l.get(..4).is_some_and(|y| y < "2000"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(dir.join("Shiller CHF.csv"), cut).unwrap();
    let i = inv(StockIndex::Sp500, BondIndex::Usa);
    let before = calibrate::suggestion(&strategy::dataset(&i, Some(&lib())).unwrap());
    let after = calibrate::suggestion(&strategy::dataset(&i, Some(&dir)).unwrap());
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(after.stock_years.unwrap().1, 1999);
    assert_ne!(before.stocks, after.stocks);
}

/// FEA-78/AK7: «Welt individuell» erhält Vorschlagswerte aus seiner eigenen Reihe.
#[test]
fn fea78_ak7_custom_world_uses_its_own_series() {
    let custom = |weights| {
        calibrate::suggestion(
            &strategy::dataset(
                &Investment {
                    stocks: StockIndex::WorldCustom,
                    bonds: BondIndex::Ch,
                    weights,
                },
                Some(&lib()),
            )
            .unwrap(),
        )
        .stocks
        .unwrap()
    };
    let only_ch = custom(RegionWeights {
        ch: 1.0,
        usa: 0.0,
        eu: 0.0,
        apac: 0.0,
    });
    let ch = calibrate::suggestion(&ds(StockIndex::ChJst, BondIndex::Ch))
        .stocks
        .unwrap();
    assert_eq!(only_ch, ch, "nur Schweiz = Schweiz (JST)");
    let mixed = custom(RegionWeights::default());
    assert_ne!(mixed, ch);
    assert_ne!(
        mixed,
        calibrate::suggestion(&ds(StockIndex::WorldJst, BondIndex::Ch))
            .stocks
            .unwrap()
    );
}

fn plan_json(source: &str, parametric: Value) -> Value {
    let mut v = api::example_plan();
    v["returns"]["source"] = json!(source);
    v["returns"]["parametric"] = parametric;
    v
}

/// FEA-78/AK6, FEA-79/AK6: Ältere Planungen behalten ihre Parameter; wer parametrisch rechnet, erhält einen Hinweis.
#[test]
fn fea78_ak6_fea79_ak6_older_plans_keep_their_parameters() {
    let old = json!({"paths": 100, "mu": 0.05, "sigma": 0.12, "df": 4, "jumpLambda": 0.1, "jumpMu": -0.2, "jumpSigma": 0.1, "seed": 3});
    let out = api::upgrade_plan(&plan_json("parametric", old.clone()));
    assert_eq!(out.plan["returns"]["parametric"]["auto"], json!(false));
    assert_eq!(out.plan["returns"]["parametric"]["mu"], json!(0.05));
    assert!(
        out.notes.iter().any(|n| n == engine::upgrade::PARAMETRIC_NOTE),
        "{:?}",
        out.notes
    );
    // Einmalig: nach dem Speichern (Merkmal vorhanden) kein Hinweis mehr
    let again = api::upgrade_plan(&out.plan);
    assert!(!again.notes.iter().any(|n| n == engine::upgrade::PARAMETRIC_NOTE));
    // Andere Quelle: die Werte waren nicht in Gebrauch und folgen künftig dem Index, ohne Hinweis
    let out = api::upgrade_plan(&plan_json("bootstrap", old.clone()));
    assert_eq!(out.plan["returns"]["parametric"]["auto"], json!(true));
    assert!(!out.notes.iter().any(|n| n == engine::upgrade::PARAMETRIC_NOTE));
    // Parametrisch ohne gespeicherte Werte: die bisherigen Standardwerte bleiben, mit Hinweis
    let mut v = plan_json("parametric", json!(null));
    v["returns"].as_object_mut().unwrap().remove("parametric");
    let out = api::upgrade_plan(&v);
    let q: ParametricParams = serde_json::from_value(out.plan["returns"]["parametric"].clone()).unwrap();
    assert_eq!(q, ParametricParams::default());
    assert!(out.notes.iter().any(|n| n == engine::upgrade::PARAMETRIC_NOTE));
    // Rechnung: Die gespeicherten Aktienparameter gelten unverändert
    let q: ParametricParams = serde_json::from_value(old).unwrap();
    let (r, _) = calibrate::resolve(&q, Some(&ds(StockIndex::WorldJst, BondIndex::Ch)));
    assert_eq!(r, q);
}

/// FEA-79/AK1, AK2: Für alle 9 Kombinationen stimmen die Korrelationen der Jahreswerte auf 0,05 mit der Reihe, Mittel
/// und Streuung von Obligationen und Inflation auf 0,3 Prozentpunkte.
#[test]
fn fea79_ak1_ak2_correlation_mean_and_sd_for_all_nine_combinations() {
    let stocks = [StockIndex::WorldJst, StockIndex::Sp500, StockIndex::ChJst];
    let bonds = [BondIndex::ChShort, BondIndex::Ch, BondIndex::Usa];
    let mut report = vec![];
    for s in stocks {
        for b in bonds {
            let d = ds(s, b);
            let j = calibrate::suggestion(&d).joint.unwrap();
            let y = all_years(&parametric_paths(&d, 2000, true));
            let col = |k: usize| -> Vec<f64> {
                y.iter()
                    .map(|t| match k {
                        0 => t.0,
                        1 => t.1,
                        _ => t.2,
                    })
                    .collect()
            };
            let cols = [col(0), col(1), col(2)];
            let line = format!(
                "{s:?}/{b:?}: Reihe {:.2} {:.2} {:.2}, Simulation {:.2} {:.2} {:.2}",
                j.corr[0][1],
                j.corr[0][2],
                j.corr[1][2],
                calibrate::corr(&cols[0], &cols[1]),
                calibrate::corr(&cols[0], &cols[2]),
                calibrate::corr(&cols[1], &cols[2])
            );
            for (a, c) in [(0, 1), (0, 2), (1, 2)] {
                let sim = calibrate::corr(&cols[a], &cols[c]);
                assert!((sim - j.corr[a][c]).abs() <= 0.05, "{line}");
            }
            assert!((calibrate::mean(&cols[1]) - j.bonds_mean).abs() <= 0.003, "{line}");
            assert!((calibrate::sd(&cols[1]) - j.bonds_sd).abs() <= 0.003, "{line}");
            assert!((calibrate::mean(&cols[2]) - j.inflation_mean).abs() <= 0.003, "{line}");
            assert!((calibrate::sd(&cols[2]) - j.inflation_sd).abs() <= 0.003, "{line}");
            report.push(line);
        }
    }
    println!("{}", report.join("\n"));
    // Referenz Aktien/Obligationen aus der Analyse (Welt, S&P 500, Schweiz; kurzfristig, 10 Jahre, USA)
    let reference = [[-0.07, -0.01, 0.24], [-0.02, -0.05, 0.45], [-0.07, 0.19, 0.27]];
    for (s, row) in stocks.iter().zip(reference) {
        for (b, r) in bonds.iter().zip(row) {
            let c = calibrate::suggestion(&ds(*s, *b)).joint.unwrap().corr[0][1];
            assert!((c - r).abs() < 0.03, "{s:?}/{b:?}: {c}");
        }
    }
}

/// Ruinwahrscheinlichkeit: 60 % Aktien, 40 % Obligationen, jährlich ausgeglichen, Entnahme 4 % des Startvermögens zu
/// Jahresbeginn, mit der Teuerung angepasst, 30 Jahre.
fn ruin(paths: &[MarketPath]) -> f64 {
    let ruined = paths
        .iter()
        .filter(|p| {
            let (mut w, mut price) = (1.0, 1.0);
            for (s, b, i) in years_of(p) {
                w -= 0.04 * price;
                if w <= 0.0 {
                    return true;
                }
                w *= 1.0 + 0.6 * s + 0.4 * b;
                price *= 1.0 + i;
            }
            false
        })
        .count();
    ruined as f64 / paths.len() as f64
}

/// Wie [`ruin`], aber monatlich ausgeglichen und monatlich entnommen (wie die Standardstrategie der App).
fn ruin_monthly(paths: &[MarketPath]) -> f64 {
    let ruined = paths
        .iter()
        .filter(|p| {
            let (mut w, mut price) = (1.0, 1.0);
            for k in 0..YEARS * 12 {
                w -= 0.04 / 12.0 * price;
                if w <= 0.0 {
                    return true;
                }
                w *= 1.0 + 0.6 * p.stocks[k] + 0.4 * p.bonds[k];
                price *= (1.0 + p.inflation[k]).powf(1.0 / 12.0);
            }
            false
        })
        .count();
    ruined as f64 / paths.len() as f64
}

/// FEA-79/AK3: Ruinwahrscheinlichkeit (60/40, 4 % real, 30 Jahre) höchstens 2 Prozentpunkte neben dem Bootstrap für
/// «Welt (JST)» und «Schweiz (JST)» mit «Eidgenossenschaft 10 Jahre» und «kurzfristig». Gerechnet auf den
/// Renditepfaden (ohne Steuern und Ausgabenplanung), jährlich ausgeglichen: Die Jahresreihen des Bootstraps haben
/// innerhalb eines Jahres gleiche Monatsrenditen, ein monatlicher Ausgleich brächte dem parametrischen Modell
/// (Schwankung auch innerhalb des Jahres) einen Ausgleichsgewinn, den der Bootstrap nicht hat.
#[test]
fn fea79_ak3_ruin_probability_close_to_the_bootstrap() {
    let mut report = vec![];
    for s in [StockIndex::WorldJst, StockIndex::ChJst] {
        for b in [BondIndex::Ch, BondIndex::ChShort] {
            let d = ds(s, b);
            let (boot, par) = (bootstrap_paths(&d, 6000), parametric_paths(&d, 6000, true));
            let (rb, rp) = (ruin(&boot), ruin(&par));
            report.push(format!(
                "{s:?}/{b:?}: Bootstrap {rb:.3}, parametrisch {rp:.3}; monatlich ausgeglichen {:.3} / {:.3}",
                ruin_monthly(&boot),
                ruin_monthly(&par)
            ));
            assert!((rp - rb).abs() <= 0.02, "{report:?}");
        }
    }
    println!("{}", report.join("\n"));
}

/// FEA-79/AK4: Ein Wechsel der Obligationen oder der Aktien ändert die Matrix.
#[test]
fn fea79_ak4_matrix_depends_on_the_combination() {
    let m = |s, b| calibrate::suggestion(&ds(s, b)).joint.unwrap();
    let a = m(StockIndex::WorldJst, BondIndex::Ch);
    assert_ne!(a.corr, m(StockIndex::WorldJst, BondIndex::Usa).corr);
    assert_ne!(a.corr, m(StockIndex::ChJst, BondIndex::Ch).corr);
    assert_ne!(a.bonds_mean, m(StockIndex::WorldJst, BondIndex::ChShort).bonds_mean);
    // Die Kurzinfo zur Planung enthält die Werte (Anzeige im Reiter «Simulation»)
    let i = inv(StockIndex::WorldJst, BondIndex::Usa);
    let reference = format!("strategy:{}", serde_json::to_string(&i).unwrap());
    let info = api::series_info(&reference, Path::new("."), Some(&lib())).unwrap();
    let p = info.parametric.unwrap();
    assert_eq!(p.joint.unwrap(), m(StockIndex::WorldJst, BondIndex::Usa));
    assert!(p.stocks.is_some());
}

/// FEA-79/AK5: Gleicher Startwert ergibt gleiche Pfade; die Aktienpfade bleiben mit und ohne Korrelation gleich.
#[test]
fn fea79_ak5_same_seed_same_paths_and_unchanged_stocks() {
    let d = ds(StockIndex::WorldJst, BondIndex::Ch);
    let f = fixed();
    let q = ParametricParams {
        paths: 20,
        seed: 11,
        ..Default::default()
    };
    let (params, c) = calibrate::resolve(&q, Some(&d));
    let g = |correlated: Option<Correlated>| Parametric {
        fixed: &f,
        start: YearMonth::new(2030, 7),
        months: 40,
        params: params.clone(),
        correlated,
    };
    let (with, without) = (g(c.clone()), g(None));
    for i in 0..20 {
        let (a, b) = (with.path(i), without.path(i));
        assert_eq!(a.stocks, b.stocks, "Aktienpfad {i}");
        assert_eq!(a.stocks, with.path(i).stocks);
        assert_eq!(a.bonds, with.path(i).bonds);
        assert_eq!(a.inflation, with.path(i).inflation);
        assert_ne!(a.bonds, b.bonds);
        // Juli bis Dezember (6 Monate) ein Wert, danach je Kalenderjahr einer
        assert!(a.inflation[..6].windows(2).all(|w| w[0] == w[1]));
        assert_ne!(a.inflation[5], a.inflation[6]);
        assert!(a.inflation[6..18].windows(2).all(|w| w[0] == w[1]));
    }
    assert_ne!(with.path(0).bonds, with.path(1).bonds);
}
