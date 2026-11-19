//! #148: Statistical tests of the return paths with confidence limits instead of fixed percentages. Every limit is computed
//! from the sample size (4 standard errors or 99 % interval); fixed seeds, therefore reproducible.
//! Every test also shows that a stock return shifted by 1 percentage point per year would violate the limit.

use std::path::{Path, PathBuf};

use engine::calibrate;
use engine::market::{DataSet, FixedCurves, MarketPath};
use engine::model::{BondIndex, BootstrapParams, Investment, ParametricParams, RegionWeights, StockIndex};
use engine::montecarlo::{Bootstrap, Parametric};
use engine::resolve::Curve;
use engine::strategy;
use engine::time::YearMonth;
use rayon::prelude::*;

const YEARS: usize = 30;
const PATHS: usize = 2000;

fn dataset() -> DataSet {
    let lib: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let inv = Investment {
        stocks: StockIndex::Sp500,
        bonds: BondIndex::Usa,
        weights: RegionWeights::default(),
    };
    strategy::dataset(&inv, Some(&lib)).unwrap()
}

fn fixed() -> FixedCurves {
    FixedCurves {
        stocks: Curve::constant(0.0),
        bonds: Curve::constant(0.0),
        inflation: Curve::constant(0.0),
        saron: Curve::constant(0.0),
    }
}

fn start() -> YearMonth {
    YearMonth::new(2030, 1)
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// Sample variance (n − 1).
fn var(v: &[f64]) -> f64 {
    let m = mean(v);
    v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64
}

/// Is `est` within 4 standard errors of `expected`?
fn within(est: f64, expected: f64, se: f64) -> bool {
    (est - expected).abs() < 4.0 * se
}

/// Annual log returns of the stocks, annual values of bonds and inflation per calendar year of all paths.
fn years(paths: &[MarketPath]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let (mut s, mut b, mut i) = (vec![], vec![], vec![]);
    for p in paths {
        for y in 0..YEARS {
            let r = 12 * y..12 * y + 12;
            s.push(p.stocks[r.clone()].iter().map(|x| (1.0 + x).ln()).sum());
            b.push(p.bonds[r.clone()].iter().map(|x| 1.0 + x).product::<f64>() - 1.0);
            i.push(p.inflation[12 * y]);
        }
    }
    (s, b, i)
}

/// Name, values of the source and values of a path.
type Series = (&'static str, Vec<f64>, fn(&MarketPath) -> Vec<f64>);

/// #148/AK1: Bootstrap (circular blocks, mean length 12 months). Every month of the source is equally likely,
/// so for the monthly values: mean and spread (around the mean of the source) match the source. Checked
/// per series (stocks as log return, bonds, inflation) with the mean over the paths; the paths are independent,
/// the standard error comes from the spread of the path means. The mean annual log return of the stocks is 12 times
/// the monthly one and thus checked too. The spread of the annual return depends on the autocorrelation, which the
/// block bootstrap only keeps within the blocks; it is deliberately not compared with the source here.
#[test]
fn issue_148_ak1_bootstrap_keeps_mean_and_spread_of_the_source() {
    let d = dataset();
    let f = fixed();
    let b = Bootstrap {
        data: &d,
        fixed: &f,
        start: start(),
        months: YEARS * 12,
        params: BootstrapParams {
            paths: PATHS,
            block: 12.0,
            seed: 11,
        },
    };
    let paths: Vec<MarketPath> = (0..PATHS).into_par_iter().map(|i| b.path(i)).collect();
    let log = |v: &[f64]| v.iter().map(|x| (1.0 + x).ln()).collect::<Vec<f64>>();
    let series: [Series; 3] = [
        ("Aktien", log(&d.stocks), |p| {
            p.stocks.iter().map(|x| (1.0 + x).ln()).collect()
        }),
        ("Obligationen", d.bonds.clone(), |p| p.bonds.clone()),
        ("Inflation", d.inflation.clone(), |p| p.inflation.clone()),
    ];
    for (name, src, of) in series {
        let (mu, n) = (mean(&src), src.len() as f64);
        let pop_var = src.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / n;
        let per_path: Vec<Vec<f64>> = paths.iter().map(of).collect();
        let means: Vec<f64> = per_path.iter().map(|v| mean(v)).collect();
        let sq: Vec<f64> = per_path
            .iter()
            .map(|v| v.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / v.len() as f64)
            .collect();
        let (se_m, se_v) = ((var(&means) / PATHS as f64).sqrt(), (var(&sq) / PATHS as f64).sqrt());
        let (m, v) = (mean(&means), mean(&sq));
        assert!(within(m, mu, se_m), "{name}: Mittel {m} statt {mu} (SE {se_m})");
        assert!(
            within(v, pop_var, se_v),
            "{name}: Varianz {v} statt {pop_var} (SE {se_v})"
        );
        if name == "Aktien" {
            // Annual log return: 12 × monthly mean; 1 percentage point per year ≈ ln(1.01) would lie outside the limit
            assert!(within(12.0 * m, 12.0 * mu, 12.0 * se_m));
            assert!(
                !within(12.0 * m + 0.01_f64.ln_1p(), 12.0 * mu, 12.0 * se_m),
                "Test zu schwach: SE {se_m}"
            );
        }
    }
}

/// #148/AK2: Parametric (Student-t with 6 degrees of freedom, price jumps). Annual log return L = sum of 252
/// daily steps with drift (μ − σ²/2)/252, shock σ/√252 · t (variance 1) and jumps n · (μ_J + σ_J · Z),
/// n ~ Poisson(λ/252). Hence E[L] = μ − σ²/2 + λ · μ_J and
/// Var[L] = σ² + 252 · [(a + a²)(σ_J² + μ_J²) − a² μ_J²] with a = λ/252.
/// Standard error: mean √(s²/N), variance √((m₄ − s⁴)/N) with the fourth central moment of the sample.
#[test]
fn issue_148_ak2_parametric_stock_returns_match_the_model() {
    let (mu, sigma, lambda, jm, js) = (0.07, 0.15, 0.3, -0.10, 0.10);
    let q = ParametricParams {
        paths: PATHS,
        seed: 13,
        mu,
        sigma,
        df: 6.0,
        jump_lambda: lambda,
        jump_mu: jm,
        jump_sigma: js,
        auto: false,
        ..Default::default()
    };
    let f = fixed();
    let g = Parametric {
        fixed: &f,
        start: start(),
        months: YEARS * 12,
        params: q,
        correlated: None,
    };
    let paths: Vec<MarketPath> = (0..PATHS).into_par_iter().map(|i| g.path(i)).collect();
    let (l, _, _) = years(&paths);
    let n = l.len() as f64;
    let a = lambda / 252.0;
    let e = mu - sigma * sigma / 2.0 + lambda * jm;
    let v_model = sigma * sigma + 252.0 * ((a + a * a) * (js * js + jm * jm) - a * a * jm * jm);
    let (m, s2) = (mean(&l), var(&l));
    let m4 = l.iter().map(|x| (x - m).powi(4)).sum::<f64>() / n;
    let (se_m, se_v) = ((s2 / n).sqrt(), ((m4 - s2 * s2) / n).sqrt());
    assert!(within(m, e, se_m), "Mittel {m} statt {e} (SE {se_m})");
    assert!(within(s2, v_model, se_v), "Varianz {s2} statt {v_model} (SE {se_v})");
    assert!(!within(m + 0.01_f64.ln_1p(), e, se_m), "Test zu schwach: SE {se_m}");
}

/// Fisher z: is the correlation `r` from `n` pairs within the 99 % interval around `rho`?
fn corr_ok(r: f64, rho: f64, n: usize) -> bool {
    (r.atanh() - rho.atanh()).abs() < 2.576 / ((n - 3) as f64).sqrt()
}

/// #148/AK3: Parametric with correlated bonds and inflation (#101, suggested values of the series). The annual values
/// depend linearly on the standardized annual log return of the stocks: corr(bonds, L) = a_b,
/// corr(inflation, L) = a_i, corr(bonds, inflation) = a_b · a_i + l11 · l21 (`calibrate::Correlated`).
#[test]
fn issue_148_ak3_parametric_correlations_match_the_model() {
    let d = dataset();
    let auto = ParametricParams {
        paths: PATHS,
        seed: 17,
        auto: true,
        ..Default::default()
    };
    let (params, c) = calibrate::resolve(&auto, Some(&d));
    let c = c.expect("korreliert");
    let f = fixed();
    let g = Parametric {
        fixed: &f,
        start: start(),
        months: YEARS * 12,
        params,
        correlated: Some(c.clone()),
    };
    let paths: Vec<MarketPath> = (0..PATHS).into_par_iter().map(|i| g.path(i)).collect();
    let (l, b, i) = years(&paths);
    let n = l.len();
    let expected = [
        ("Aktien/Obligationen", calibrate::corr(&l, &b), c.a_bonds),
        ("Aktien/Inflation", calibrate::corr(&l, &i), c.a_inflation),
        (
            "Obligationen/Inflation",
            calibrate::corr(&b, &i),
            c.a_bonds * c.a_inflation + c.l.0 * c.l.1,
        ),
    ];
    for (name, r, rho) in expected {
        assert!(corr_ok(r, rho, n), "{name}: {r:.4} statt {rho:.4}");
    }
    // Mean and spread of bonds and inflation as in the series (4 standard errors)
    for (name, v, (m0, s0)) in [("Obligationen", &b, c.bonds), ("Inflation", &i, c.inflation)] {
        let (m, s2) = (mean(v), var(v));
        assert!(within(m, m0, (s2 / n as f64).sqrt()), "{name}: Mittel {m} statt {m0}");
        let m4 = v.iter().map(|x| (x - m).powi(4)).sum::<f64>() / n as f64;
        assert!(
            within(s2, s0 * s0, ((m4 - s2 * s2) / n as f64).sqrt()),
            "{name}: Varianz {s2} statt {}",
            s0 * s0
        );
    }
}
