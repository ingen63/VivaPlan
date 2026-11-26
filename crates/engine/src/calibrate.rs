//! #100 and #101: Suggested values for «Monte Carlo (parametrisch)» from the annual returns of the investment strategy.
//!
//! - **Stocks (#100):** From the whole annual series of the stock index (not only the years in common with the
//!   bonds). Degrees of freedom (6) and the spread of the jump size (15 %) stay fixed. For every combination of
//!   jump rate and jump size of a grid, μ and σ follow in closed form from the mean and spread of the
//!   annual returns; the combination whose 5 % quantile and geometric mean come closest to the series is
//!   chosen. The model's annual return is a Poisson mixture of lognormal distributions (the 252
//!   Student-t daily steps of a year are practically normally distributed together).
//! - **Bonds and inflation (#101):** Mean, spread and the 3×3 correlation (stocks, bonds,
//!   inflation) of the annual values from the same series the historical bootstrap uses. The years are
//!   independent of each other.
//! - **Inflation period (#279):** Mean and spread of the inflation and its correlation with stocks and bonds only
//!   from the years from `limits.inflationFrom` (default 1950); the war years drive the spread from 2.3 % to 5 %.

use serde::Serialize;

use crate::market::{DataSet, Freq, StockSeries};
use crate::model::ParametricParams;
use crate::montecarlo::Rng;

/// Degrees of freedom of the Student-t shocks in the suggested values.
pub const DF: f64 = 6.0;
/// Spread of the jump size in the suggested values.
pub const JUMP_SIGMA: f64 = 0.15;
/// Minimum number of years for suggested values.
pub const MIN_YEARS: usize = 10;

// ---------- Metrics of an annual series ----------

/// Metrics of annual returns; the spread as in the bootstrap (drawing with replacement, denominator n).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Moments {
    pub mean: f64,
    pub sd: f64,
    /// Geometric mean exp(mean of ln(1 + r)) − 1.
    pub geo: f64,
    /// 5 % quantile (linearly interpolated).
    pub q5: f64,
}

pub fn mean(x: &[f64]) -> f64 {
    x.iter().sum::<f64>() / x.len() as f64
}

/// Standard deviation with denominator n.
pub fn sd(x: &[f64]) -> f64 {
    let m = mean(x);
    (x.iter().map(|v| (v - m).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

/// Pearson correlation; 0 for a constant series.
pub fn corr(a: &[f64], b: &[f64]) -> f64 {
    let (ma, mb) = (mean(a), mean(b));
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum();
    let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum();
    if va <= 0.0 || vb <= 0.0 {
        0.0
    } else {
        cov / (va * vb).sqrt()
    }
}

/// Quantile with linear interpolation between the sorted values (like numpy).
pub fn quantile(x: &[f64], p: f64) -> f64 {
    let mut s = x.to_vec();
    s.sort_by(f64::total_cmp);
    let h = (s.len() - 1) as f64 * p;
    let (lo, hi) = (h.floor() as usize, h.ceil() as usize);
    s[lo] + (h - lo as f64) * (s[hi] - s[lo])
}

pub fn moments(x: &[f64]) -> Moments {
    Moments {
        mean: mean(x),
        sd: sd(x),
        geo: (x.iter().map(|r| (1.0 + r).ln()).sum::<f64>() / x.len() as f64).exp() - 1.0,
        q5: quantile(x, 0.05),
    }
}

// ---------- Annual return of the stock model ----------

/// Standard normal distribution (cumulative distribution function), relative accuracy about 1e-7.
fn phi(x: f64) -> f64 {
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * z);
    let erfc = t
        * (-z * z - 1.265_512_23
            + t * (1.000_023_68
                + t * (0.374_091_96
                    + t * (0.096_784_18
                        + t * (-0.186_288_06
                            + t * (0.278_868_07
                                + t * (-1.135_203_98
                                    + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77)))))))))
            .exp();
    if x >= 0.0 {
        1.0 - 0.5 * erfc
    } else {
        0.5 * erfc
    }
}

/// Log annual return L of the stock model as a mixture: with k jumps (weight Poisson(λ)) normally distributed with
/// mean μ − σ²/2 + k·μJ and variance σ² + k·σJ².
#[derive(Clone, Debug)]
pub struct YearModel {
    comps: Vec<(f64, f64, f64)>,
}

impl YearModel {
    pub fn new(q: &ParametricParams) -> YearModel {
        let base = q.mu - 0.5 * q.sigma * q.sigma;
        let lam = q.jump_lambda.max(0.0);
        let mut comps = vec![];
        let mut w = (-lam).exp();
        for k in 0..=12 {
            if k > 0 {
                w *= lam / k as f64;
            }
            comps.push((
                w,
                base + k as f64 * q.jump_mu,
                q.sigma * q.sigma + k as f64 * q.jump_sigma.powi(2),
            ));
        }
        YearModel { comps }
    }

    /// Expectation and spread of L.
    pub fn log_moments(&self) -> (f64, f64) {
        let m: f64 = self.comps.iter().map(|(w, m, _)| w * m).sum();
        let e2: f64 = self.comps.iter().map(|(w, m, v)| w * (v + m * m)).sum();
        (m, (e2 - m * m).max(0.0).sqrt())
    }

    /// Metrics of the simple annual return exp(L) − 1.
    pub fn moments(&self) -> Moments {
        let e1: f64 = self.comps.iter().map(|(w, m, v)| w * (m + v / 2.0).exp()).sum();
        let e2: f64 = self.comps.iter().map(|(w, m, v)| w * (2.0 * m + 2.0 * v).exp()).sum();
        let (lm, _) = self.log_moments();
        Moments {
            mean: e1 - 1.0,
            sd: (e2 - e1 * e1).max(0.0).sqrt(),
            geo: lm.exp() - 1.0,
            q5: self.quantile(0.05),
        }
    }

    fn cdf_log(&self, l: f64) -> f64 {
        self.comps
            .iter()
            .map(|(w, m, v)| {
                if *v > 0.0 {
                    w * phi((l - m) / v.sqrt())
                } else if l >= *m {
                    *w
                } else {
                    0.0
                }
            })
            .sum()
    }

    /// Quantile of the simple annual return.
    pub fn quantile(&self, p: f64) -> f64 {
        let (mut lo, mut hi) = (-10.0, 10.0);
        for _ in 0..100 {
            let mid = 0.5 * (lo + hi);
            if self.cdf_log(mid) < p {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (0.5 * (lo + hi)).exp() - 1.0
    }

    /// Correlation between the simple return exp(L) − 1 and L.
    pub fn corr_simple_log(&self) -> f64 {
        let (lm, ls) = self.log_moments();
        let e1: f64 = self.comps.iter().map(|(w, m, v)| w * (m + v / 2.0).exp()).sum();
        let e2: f64 = self.comps.iter().map(|(w, m, v)| w * (2.0 * m + 2.0 * v).exp()).sum();
        let el: f64 = self
            .comps
            .iter()
            .map(|(w, m, v)| w * (m + v) * (m + v / 2.0).exp())
            .sum();
        let sd = (e2 - e1 * e1).max(0.0).sqrt();
        if sd <= 0.0 || ls <= 0.0 {
            1.0
        } else {
            ((el - lm * e1) / (sd * ls)).clamp(-1.0, 1.0)
        }
    }
}

// ---------- #100: Stock parameters ----------

/// Suggested values for the stock model.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockFit {
    pub mu: f64,
    pub sigma: f64,
    pub df: f64,
    pub jump_lambda: f64,
    pub jump_mu: f64,
    pub jump_sigma: f64,
}

impl StockFit {
    /// Sets the model parameters (number of paths and seed stay).
    pub fn apply(&self, q: &mut ParametricParams) {
        q.mu = self.mu;
        q.sigma = self.sigma;
        q.df = self.df;
        q.jump_lambda = self.jump_lambda;
        q.jump_mu = self.jump_mu;
        q.jump_sigma = self.jump_sigma;
    }

    pub fn params(&self) -> ParametricParams {
        let mut q = ParametricParams::default();
        self.apply(&mut q);
        q
    }
}

fn round_to(x: f64, step: f64) -> f64 {
    (x / step).round() * step
}

/// Suggested values from annual returns; `None` with fewer than [`MIN_YEARS`] years or without a fitting combination.
pub fn fit_stocks(years: &[f64]) -> Option<StockFit> {
    if years.len() < MIN_YEARS || years.iter().any(|r| !r.is_finite() || *r <= -1.0) {
        return None;
    }
    fit_moments(&moments(years))
}

/// Suggested values for target metrics of the annual return.
pub fn fit_moments(t: &Moments) -> Option<StockFit> {
    let js = JUMP_SIGMA;
    let mut best: Option<(f64, StockFit)> = None;
    for l in 0..=30 {
        let lam = l as f64 * 0.01;
        for j in 1..=10 {
            let jm = -0.05 * j as f64;
            if l == 0 && j > 1 {
                break; // without jumps the jump size does not matter
            }
            // E[1 + R] = e^μ · Φ and E[(1 + R)²] = e^(2μ + σ²) · Ψ (Φ, Ψ: contribution of the jumps)
            let phi1 = (lam * ((jm + js * js / 2.0).exp() - 1.0)).exp();
            let psi = (lam * ((2.0 * jm + 2.0 * js * js).exp() - 1.0)).exp();
            let emu = (1.0 + t.mean) / phi1;
            let s2 = ((t.sd * t.sd + (1.0 + t.mean).powi(2)) / (emu * emu) / psi).ln();
            if s2.is_nan() || s2 <= 0.02 * 0.02 {
                continue;
            }
            let fit = StockFit {
                mu: round_to(emu.ln(), 0.0005),
                sigma: round_to(s2.sqrt(), 0.0005),
                df: DF,
                jump_lambda: round_to(lam, 0.01),
                jump_mu: round_to(if l == 0 { 0.0 } else { jm }, 0.01),
                jump_sigma: if l == 0 { 0.0 } else { js },
            };
            let m = YearModel::new(&fit.params()).moments();
            let err = ((m.q5 - t.q5) / 0.01).powi(2)
                + ((m.geo - t.geo) / 0.002).powi(2)
                + ((m.mean - t.mean) / 0.002).powi(2)
                + ((m.sd - t.sd) / 0.002).powi(2);
            if best.as_ref().is_none_or(|(e, _)| err < *e) {
                best = Some((err, fit));
            }
        }
    }
    best.map(|(_, f)| f)
}

// ---------- #101: Bonds and inflation ----------

/// Annual values of a series: stocks and bonds linked to calendar years, inflation in December (annual rate).
/// Annual series stay as they are.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct YearlyTriples {
    pub first: i32,
    pub stocks: Vec<f64>,
    pub bonds: Vec<f64>,
    pub inflation: Vec<f64>,
}

pub fn yearly(ds: &DataSet) -> YearlyTriples {
    match ds.freq {
        Freq::Yearly => YearlyTriples {
            first: ds.first.year(),
            stocks: ds.stocks.clone(),
            bonds: ds.bonds.clone(),
            inflation: ds.inflation.clone(),
        },
        Freq::Monthly => {
            let s = crate::strategy::compound_years(ds.first, &ds.stocks);
            let b = crate::strategy::compound_years(ds.first, &ds.bonds);
            let i = crate::strategy::yearly_rates(ds.first, &ds.inflation, true);
            YearlyTriples {
                first: s.first,
                stocks: s.values,
                bonds: b.values,
                inflation: i.values,
            }
        }
    }
}

/// Mean, spread and correlation of stocks, bonds and inflation (annual values).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joint {
    pub first: i32,
    pub last: i32,
    /// #279: Years the inflation (mean, spread, correlations) is estimated from.
    pub inflation_first: i32,
    pub inflation_last: i32,
    pub bonds_mean: f64,
    pub bonds_sd: f64,
    pub inflation_mean: f64,
    pub inflation_sd: f64,
    /// Order stocks, bonds, inflation.
    pub corr: [[f64; 3]; 3],
}

/// Joint metrics of the annual values. Stocks and bonds from the whole series; inflation and its correlations
/// only from the years from `inflation_from` (#279; a year before the series uses the whole series). `None` with
/// fewer than [`MIN_YEARS`] years in the whole series or from `inflation_from`.
pub fn joint(y: &YearlyTriples, inflation_from: i32) -> Option<Joint> {
    let n = y.stocks.len();
    if n < MIN_YEARS || y.bonds.len() != n || y.inflation.len() != n {
        return None;
    }
    let all = [&y.stocks, &y.bonds, &y.inflation];
    if all.iter().any(|c| c.iter().any(|v| !v.is_finite())) {
        return None;
    }
    let skip = (inflation_from.saturating_sub(y.first)).clamp(0, n as i32) as usize;
    if n - skip < MIN_YEARS {
        return None;
    }
    let tail = [&y.stocks[skip..], &y.bonds[skip..], &y.inflation[skip..]];
    let mut corr = [[1.0; 3]; 3];
    for a in 0..3 {
        for b in 0..3 {
            if a != b {
                // Pairs with the inflation only from its years
                corr[a][b] = if a == 2 || b == 2 {
                    self::corr(tail[a], tail[b])
                } else {
                    self::corr(all[a], all[b])
                };
            }
        }
    }
    let last = y.first + n as i32 - 1;
    Some(Joint {
        first: y.first,
        last,
        inflation_first: y.first + skip as i32,
        inflation_last: last,
        bonds_mean: mean(&y.bonds),
        bonds_sd: sd(&y.bonds),
        inflation_mean: mean(tail[2]),
        inflation_sd: sd(tail[2]),
        corr,
    })
}

/// Bonds and inflation of a year, conditional on the standardized log annual return of the stocks z:
/// (zB, zI) = a·z + L·(e1, e2) with independent standard normals e1, e2. The factors a are chosen so that the
/// correlation matches the simple stock return of the series (divided by corr(exp(L) − 1, L)).
#[derive(Clone, Debug, PartialEq)]
pub struct Correlated {
    /// Expectation and spread of the log annual return of the stocks.
    pub log_mean: f64,
    pub log_sd: f64,
    pub bonds: (f64, f64),
    pub inflation: (f64, f64),
    pub a_bonds: f64,
    pub a_inflation: f64,
    /// Lower triangular matrix of the residual covariance: l11, l21, l22.
    pub l: (f64, f64, f64),
}

impl Correlated {
    pub fn new(j: &Joint, q: &ParametricParams) -> Correlated {
        let m = YearModel::new(q);
        let (log_mean, log_sd) = m.log_moments();
        let c = m.corr_simple_log().max(0.05);
        let lim = 0.99;
        let a_b = (j.corr[0][1] / c).clamp(-lim, lim);
        let a_i = (j.corr[0][2] / c).clamp(-lim, lim);
        let v_b = 1.0 - a_b * a_b;
        let v_i = 1.0 - a_i * a_i;
        let bound = 0.99 * (v_b * v_i).sqrt();
        let cov = (j.corr[1][2] - a_b * a_i).clamp(-bound, bound);
        let l11 = v_b.sqrt();
        let l21 = if l11 > 0.0 { cov / l11 } else { 0.0 };
        let l22 = (v_i - l21 * l21).max(0.0).sqrt();
        Correlated {
            log_mean,
            log_sd,
            bonds: (j.bonds_mean, j.bonds_sd),
            inflation: (j.inflation_mean, j.inflation_sd),
            a_bonds: a_b,
            a_inflation: a_i,
            l: (l11, l21, l22),
        }
    }

    /// Annual values (bonds, inflation) for the log return of the stocks `log` over `months` months of the year.
    pub fn draw(&self, log: f64, months: usize, e1: f64, e2: f64) -> (f64, f64) {
        let f = months as f64 / 12.0;
        let z = if self.log_sd > 0.0 {
            (log - self.log_mean * f) / (self.log_sd * f.sqrt())
        } else {
            0.0
        };
        let zb = self.a_bonds * z + self.l.0 * e1;
        let zi = self.a_inflation * z + self.l.1 * e1 + self.l.2 * e2;
        (
            (self.bonds.0 + self.bonds.1 * zb).max(-0.95),
            (self.inflation.0 + self.inflation.1 * zi).max(-0.95),
        )
    }
}

// ---------- Suggestion for a series ----------

/// Suggested values for a series (displayed in the tab «Simulation»).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub stocks: Option<StockFit>,
    /// Years of the stock series the stock parameters come from.
    pub stock_years: Option<(i32, i32)>,
    pub joint: Option<Joint>,
}

/// Number of years of the sample used to determine the annual distribution of the bootstrap on monthly data.
pub const BOOTSTRAP_YEARS: usize = 100_000;

/// Annual returns as the historical bootstrap draws them from a stock series. Annual series: the years themselves (the
/// bootstrap draws whole years). Monthly series: a fixed sample of [`BOOTSTRAP_YEARS`] years from blocks with
/// a mean length of 12 months (default of the bootstrap); months from different times yield milder
/// extreme years than the calendar years.
pub fn bootstrap_years(s: &StockSeries) -> Vec<f64> {
    match s.freq {
        Freq::Yearly => s.values.clone(),
        Freq::Monthly => {
            let n = s.values.len();
            if n < 12 * MIN_YEARS {
                return vec![];
            }
            let mut rng = Rng::new(0x4645_412d_3738, 0);
            (0..BOOTSTRAP_YEARS)
                .map(|_| {
                    let mut i = rng.below(n);
                    let mut g = 1.0 + s.values[i];
                    for _ in 1..12 {
                        i = if rng.uniform() < 1.0 / 12.0 {
                            rng.below(n)
                        } else {
                            (i + 1) % n
                        };
                        g *= 1.0 + s.values[i];
                    }
                    g - 1.0
                })
                .collect()
        }
    }
}

/// Whole stock series of the index, otherwise the stocks of the series itself.
fn stock_series(ds: &DataSet) -> StockSeries {
    ds.stock_series.clone().unwrap_or_else(|| StockSeries {
        freq: ds.freq,
        first: ds.first,
        values: ds.stocks.clone(),
    })
}

/// Suggested values of a series; the inflation from the years from `inflation_from` (#279).
pub fn suggestion(ds: &DataSet, inflation_from: i32) -> Suggestion {
    let y = yearly(ds);
    let s = stock_series(ds);
    let stocks = fit_stocks(&bootstrap_years(&s));
    let last = match s.freq {
        Freq::Yearly => s.first.year() + s.values.len() as i32 - 1,
        Freq::Monthly => s.first.plus(s.values.len() as i32 - 1).year(),
    };
    Suggestion {
        stock_years: stocks.map(|_| (s.first.year(), last)),
        stocks,
        joint: joint(&y, inflation_from),
    }
}

/// Parameters the model computes with: with `auto` the suggested values of the series (#100), otherwise the
/// saved ones; plus the correlation with bonds and inflation, if there is a series (#101), the inflation from the years
/// from `inflation_from` (#279).
pub fn resolve(
    q: &ParametricParams,
    data: Option<&DataSet>,
    inflation_from: i32,
) -> (ParametricParams, Option<Correlated>) {
    let mut q = q.clone();
    let Some(ds) = data else {
        return (q, None);
    };
    // Determine the stock parameters only with `auto` (with monthly series the sample costs a few milliseconds)
    if q.auto {
        if let Some(f) = fit_stocks(&bootstrap_years(&stock_series(ds))) {
            f.apply(&mut q);
        }
    }
    let c = joint(&yearly(ds), inflation_from).map(|j| Correlated::new(&j, &q));
    (q, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_matches_known_values() {
        assert!((phi(0.0) - 0.5).abs() < 1e-7);
        assert!((phi(1.0) - 0.841_344_746).abs() < 1e-6);
        assert!((phi(-1.644_853_6) - 0.05).abs() < 1e-6);
        assert!((phi(3.0) - 0.998_650_102).abs() < 1e-6);
    }

    #[test]
    fn moments_of_a_small_series() {
        let m = moments(&[0.1, -0.1, 0.2, 0.0]);
        assert!((m.mean - 0.05).abs() < 1e-12);
        assert!((m.sd - (0.0125f64).sqrt()).abs() < 1e-12);
        // Linear quantile: (n − 1)·0.05 = 0.15 between −0.1 and 0.0
        assert!((m.q5 - (-0.085)).abs() < 1e-12);
        assert!((quantile(&[3.0, 1.0, 2.0], 0.5) - 2.0).abs() < 1e-12);
        assert_eq!(corr(&[1.0, 1.0], &[1.0, 2.0]), 0.0);
        assert!((corr(&[1.0, 2.0, 3.0], &[2.0, 4.0, 7.0]) - 0.993_399).abs() < 1e-6);
    }

    #[test]
    fn year_model_without_jumps_is_lognormal() {
        let q = ParametricParams {
            mu: 0.08,
            sigma: 0.2,
            jump_lambda: 0.0,
            ..Default::default()
        };
        let m = YearModel::new(&q).moments();
        assert!((m.mean - (0.08f64.exp() - 1.0)).abs() < 1e-12);
        assert!((m.geo - ((0.08 - 0.02f64).exp() - 1.0)).abs() < 1e-12);
        let q5 = (0.06 - 1.644_853_6 * 0.2f64).exp() - 1.0;
        assert!((m.q5 - q5).abs() < 1e-6, "{}", m.q5);
        let sd = (0.08f64.exp()) * (0.04f64.exp() - 1.0).sqrt();
        assert!((m.sd - sd).abs() < 1e-12);
        let c = YearModel::new(&q).corr_simple_log();
        assert!(c > 0.95 && c < 1.0, "{c}");
    }

    #[test]
    fn fit_reproduces_mean_sd_geo_and_q5() {
        // Made-up series with crash years
        let years: Vec<f64> = (0..100)
            .map(|i| match i % 20 {
                0 => -0.35,
                7 => -0.15,
                _ => 0.05 + 0.25 * (((i * 37) % 17) as f64 / 16.0 - 0.5),
            })
            .collect();
        let t = moments(&years);
        let f = fit_stocks(&years).unwrap();
        let m = YearModel::new(&f.params()).moments();
        assert!((m.mean - t.mean).abs() < 0.002, "{m:?} {t:?}");
        assert!((m.sd - t.sd).abs() < 0.002, "{m:?} {t:?}");
        assert!((m.geo - t.geo).abs() < 0.005, "{m:?} {t:?}");
        assert!((m.q5 - t.q5).abs() < 0.03, "{m:?} {t:?}");
        assert_eq!((f.df, f.jump_sigma > 0.0 || f.jump_lambda == 0.0), (6.0, true));
        assert!(fit_stocks(&years[..5]).is_none(), "zu kurz");
        assert!(fit_stocks(&[0.05; 20]).is_none(), "ohne Streuung");
    }

    #[test]
    fn correlated_reproduces_the_target_correlation() {
        let j = Joint {
            first: 1900,
            last: 2000,
            inflation_first: 1900,
            inflation_last: 2000,
            bonds_mean: 0.04,
            bonds_sd: 0.06,
            inflation_mean: 0.02,
            inflation_sd: 0.03,
            corr: [[1.0, 0.3, -0.2], [0.3, 1.0, -0.5], [-0.2, -0.5, 1.0]],
        };
        let q = ParametricParams::default();
        let c = Correlated::new(&j, &q);
        let ym = YearModel::new(&q);
        // Residual variance plus explained part gives 1
        assert!((c.a_bonds.powi(2) + c.l.0.powi(2) - 1.0).abs() < 1e-12);
        assert!((c.a_inflation.powi(2) + c.l.1.powi(2) + c.l.2.powi(2) - 1.0).abs() < 1e-12);
        let k = ym.corr_simple_log();
        assert!((c.a_bonds * k - 0.3).abs() < 1e-12);
        assert!((c.a_inflation * k + 0.2).abs() < 1e-12);
        assert!((c.a_bonds * c.a_inflation + c.l.0 * c.l.1 - (-0.5)).abs() < 1e-12);
        // Means at z = 0
        let (b, i) = c.draw(c.log_mean, 12, 0.0, 0.0);
        assert!((b - 0.04).abs() < 1e-12 && (i - 0.02).abs() < 1e-12);
        // Half a year: expectation halved, spread with √½
        let (b, _) = c.draw(c.log_mean * 0.5 + c.log_sd * 0.5f64.sqrt(), 6, 0.0, 0.0);
        assert!((b - (0.04 + 0.06 * c.a_bonds)).abs() < 1e-12);
    }

    #[test]
    fn impossible_correlations_are_bounded() {
        let j = Joint {
            first: 1900,
            last: 2000,
            inflation_first: 1900,
            inflation_last: 2000,
            bonds_mean: 0.0,
            bonds_sd: 0.1,
            inflation_mean: 0.0,
            inflation_sd: 0.1,
            corr: [[1.0, 0.99, -0.99], [0.99, 1.0, 0.99], [-0.99, 0.99, 1.0]],
        };
        let c = Correlated::new(&j, &ParametricParams::default());
        assert!(c.a_bonds <= 0.99 && c.a_inflation >= -0.99);
        assert!(c.l.0 >= 0.0 && c.l.2 >= 0.0 && c.l.2.is_finite());
    }

    #[test]
    fn resolve_uses_the_suggestion_only_with_auto() {
        let n = 60;
        let ds = DataSet {
            name: "t".into(),
            freq: Freq::Yearly,
            first: crate::time::YearMonth::new(1950, 1),
            stocks: (0..n)
                .map(|i| if i % 9 == 0 { -0.3 } else { 0.06 + 0.02 * (i % 5) as f64 })
                .collect(),
            bonds: (0..n).map(|i| 0.03 + 0.01 * (i % 3) as f64).collect(),
            inflation: (0..n).map(|i| 0.02 + 0.005 * (i % 4) as f64).collect(),
            ..Default::default()
        };
        let own = ParametricParams::default();
        let (q, c) = resolve(&own, Some(&ds), 0);
        assert_eq!(q, own);
        assert!(c.is_some());
        let auto = ParametricParams {
            auto: true,
            ..Default::default()
        };
        let (q, _) = resolve(&auto, Some(&ds), 0);
        assert_ne!(q.mu, own.mu);
        assert_eq!((q.paths, q.seed, q.auto), (auto.paths, auto.seed, true));
        let (q, c) = resolve(&auto, None, 0);
        assert_eq!((q, c), (auto, None));
        // Whole stock series of the index before the common years
        let mut ds2 = ds.clone();
        ds2.stock_series = Some(StockSeries {
            freq: Freq::Yearly,
            first: crate::time::YearMonth::new(1900, 1),
            values: vec![0.2; 50].into_iter().chain(ds.stocks.clone()).collect(),
        });
        let s = suggestion(&ds2, 0);
        assert_eq!(s.stock_years, Some((1900, 2009)));
        assert_ne!(s.stocks, suggestion(&ds, 0).stocks);
    }
}

#[cfg(test)]
mod bootstrap_tests {
    use super::*;
    use crate::time::YearMonth;

    #[test]
    fn bootstrap_years_of_a_monthly_series() {
        let s = StockSeries {
            freq: Freq::Monthly,
            first: YearMonth::new(1950, 1),
            values: (0..240).map(|i| if i % 2 == 0 { 0.02 } else { -0.01 }).collect(),
        };
        let y = bootstrap_years(&s);
        assert_eq!(y.len(), BOOTSTRAP_YEARS);
        assert_eq!(y, bootstrap_years(&s), "feste Stichprobe");
        // Every year is a product of 12 months of 1.02 and 0.99
        assert!(y
            .iter()
            .all(|r| *r > 0.99f64.powi(12) - 1.0 - 1e-12 && *r < 1.02f64.powi(12) - 1.0 + 1e-12));
        let m = mean(&y);
        assert!((m - ((1.02f64 * 0.99).powf(6.0) - 1.0)).abs() < 0.01, "{m}");
        let short = StockSeries {
            values: vec![0.01; 100],
            ..s.clone()
        };
        assert!(bootstrap_years(&short).is_empty());
        let yearly = StockSeries {
            freq: Freq::Yearly,
            ..s
        };
        assert_eq!(bootstrap_years(&yearly).len(), 240);
    }

    // #124/AK5: Limits of the inputs. Exactly MIN_YEARS years (or 12 × MIN_YEARS months) suffice; a return of
    // −100 % or an incomplete series gives no suggestion; `last` is the last year of the series.
    #[test]
    fn issue_124_ak5_input_limits() {
        let years: Vec<f64> = (0..MIN_YEARS).map(|i| if i % 3 == 0 { -0.1 } else { 0.12 }).collect();
        assert!(fit_stocks(&years).is_some(), "genau {MIN_YEARS} Jahre");
        assert!(fit_stocks(&years[1..]).is_none(), "ein Jahr zu wenig");
        let mut total_loss = years.repeat(2);
        total_loss[3] = -1.0;
        assert!(fit_stocks(&total_loss).is_none(), "Rendite −100 %");

        let y = YearlyTriples {
            first: 1990,
            stocks: years.clone(),
            bonds: years.iter().map(|r| r / 3.0).collect(),
            inflation: years.iter().map(|r| 0.02 + r / 10.0).collect(),
        };
        let j = joint(&y, 0).expect("genau MIN_YEARS Jahre");
        assert_eq!((j.first, j.last), (1990, 1990 + MIN_YEARS as i32 - 1));
        let short = YearlyTriples {
            stocks: y.stocks[1..].to_vec(),
            bonds: y.bonds[1..].to_vec(),
            inflation: y.inflation[1..].to_vec(),
            ..y.clone()
        };
        assert!(joint(&short, 0).is_none(), "ein Jahr zu wenig");
        let mut bonds_short = y.clone();
        bonds_short.bonds.pop();
        assert!(joint(&bonds_short, 0).is_none(), "Obligationen unvollständig");
        let mut inflation_short = y.clone();
        inflation_short.inflation.pop();
        assert!(joint(&inflation_short, 0).is_none(), "Inflation unvollständig");

        let monthly = StockSeries {
            freq: Freq::Monthly,
            first: crate::time::YearMonth::new(1950, 1),
            values: vec![0.01; 12 * MIN_YEARS],
        };
        assert_eq!(
            bootstrap_years(&monthly).len(),
            BOOTSTRAP_YEARS,
            "genau 12 × MIN_YEARS Monate"
        );
    }

    // #124/AK5: Without spread the annual return is fixed: the quantile is e^μ − 1. Without spread of the stocks
    // bonds and inflation stay at the mean of the series (z = 0).
    #[test]
    fn issue_124_ak5_model_without_spread() {
        let q = ParametricParams {
            mu: 0.05,
            sigma: 0.0,
            jump_lambda: 0.0,
            ..ParametricParams::default()
        };
        let m = YearModel::new(&q);
        for p in [0.05, 0.5, 0.95] {
            assert!(
                (m.quantile(p) - (0.05f64.exp() - 1.0)).abs() < 1e-9,
                "p = {p}: {}",
                m.quantile(p)
            );
        }
        let c = Correlated {
            log_mean: 0.05,
            log_sd: 0.0,
            bonds: (0.03, 0.05),
            inflation: (0.01, 0.02),
            a_bonds: 0.3,
            a_inflation: 0.2,
            l: (0.9, 0.1, 0.9),
        };
        assert_eq!(c.draw(0.4, 12, 0.0, 0.0), (0.03, 0.01));
    }

    // #124/AK5: Choice of the grid. A series without outliers fits best without jumps (λ = 0); for series with
    // crash years the error function weights the 5 % quantile (1 %), the means and the spread (0.2 %): result as a
    // regression.
    #[test]
    fn issue_124_ak5_grid_choice() {
        let calm = YearModel::new(
            &StockFit {
                mu: 0.06,
                sigma: 0.12,
                df: DF,
                jump_lambda: 0.0,
                jump_mu: 0.0,
                jump_sigma: 0.0,
            }
            .params(),
        )
        .moments();
        let f = fit_moments(&calm).expect("Vorschlag");
        assert_eq!((f.jump_lambda, f.jump_mu, f.jump_sigma), (0.0, 0.0, 0.0), "{f:?}");
        let crash = Moments {
            mean: 0.085,
            sd: 0.19,
            geo: 0.067,
            q5: -0.27,
        };
        let f = fit_moments(&crash).expect("Vorschlag");
        let got = [f.mu, f.sigma, f.jump_lambda, f.jump_mu, f.jump_sigma];
        let want = [0.139, 0.119, 0.27, -0.25, JUMP_SIGMA];
        assert!(got.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-9), "{f:?}");
        // Here the deviation of the spread decides (μ and σ are rounded to 0.0005 and do not hit it exactly).
        let steep = Moments {
            mean: 0.12,
            sd: 0.16,
            geo: 0.12 - 0.16 * 0.16 / 2.0,
            q5: -0.22,
        };
        let f = fit_moments(&steep).expect("Vorschlag");
        let got = [f.mu, f.sigma, f.jump_lambda, f.jump_mu];
        let want = [0.1665, 0.0735, 0.25, -0.25];
        assert!(got.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-9), "{f:?}");
    }
}
