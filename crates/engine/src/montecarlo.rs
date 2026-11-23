//! Monte Carlo: random return paths from history (stationary bootstrap) or from a
//! parametric stock model (Student-t with price jumps, like the previous program).
//!
//! The paths are generated only while computing, path `i` always from the seed and `i`. Thus
//! even a simulation with 50'000 paths needs little memory, and every path can be repeated exactly
//! for the detail view.

use crate::calibrate::Correlated;
use crate::market::{monthly, DataSet, FixedCurves, Freq, MarketPath, PRE_MONTHS};
use crate::model::{BootstrapParams, ParametricParams};
use crate::time::YearMonth;

// ---------- Random numbers ----------

/// xoshiro256** with splitmix64 initialization: fast, good quality, reproducible.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
    spare: Option<f64>,
}

fn splitmix(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl Rng {
    /// Own random stream per (seed, path number).
    pub fn new(seed: u64, stream: u64) -> Rng {
        let mut x = seed ^ stream.wrapping_mul(0xd134_2543_de82_ef95);
        Rng {
            s: [splitmix(&mut x), splitmix(&mut x), splitmix(&mut x), splitmix(&mut x)],
            spare: None,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniformly distributed in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Integer in 0..n.
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_u64() as u128 * n as u128) >> 64) as usize
    }

    /// Standard normally distributed (polar method, the second result is kept).
    pub fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare.take() {
            return z;
        }
        loop {
            let u = 2.0 * self.uniform() - 1.0;
            let v = 2.0 * self.uniform() - 1.0;
            let q = u * u + v * v;
            if q > 0.0 && q < 1.0 {
                let f = (-2.0 * q.ln() / q).sqrt();
                self.spare = Some(v * f);
                return u * f;
            }
        }
    }

    /// Student-t with `df` > 0 degrees of freedom (#132: polar method of Bailey, 1994). A point (u, v) uniformly distributed in the
    /// unit circle, w = u² + v²; then u·√(df·(w^(−2/df) − 1)/w) is exactly t-distributed. Whether the point is accepted is
    /// decided by plain arithmetic (w < 1), without functions of the math library. For the default `df` = 6,
    /// w^(−1/3) = 1/∛w, faster than the general power.
    pub fn student_t(&mut self, df: f64) -> f64 {
        // u and v from the upper and lower 32 bits of the same random number (middle of the 2³² steps in (−1, 1)): half as
        // many calls of the generator; the resolution 2⁻³¹ suffices for daily returns.
        const STEP: f64 = 1.0 / (1u64 << 31) as f64;
        loop {
            let x = self.next_u64();
            let u = ((x >> 32) as f64 + 0.5) * STEP - 1.0;
            let v = ((x & 0xffff_ffff) as f64 + 0.5) * STEP - 1.0;
            let w = u * u + v * v;
            if w < 1.0 {
                // df·(w^(−2/df) − 1)/w; with df = 6 and c = ∛w as 6·(1 − c)/(c·w), with one division
                let r = if df == 6.0 {
                    let c = w.cbrt();
                    6.0 * (1.0 - c) / (c * w)
                } else {
                    df * (w.powf(-2.0 / df) - 1.0) / w
                };
                return u * r.sqrt();
            }
        }
    }

    /// Poisson distributed (for small rates).
    pub fn poisson(&mut self, lambda: f64) -> u32 {
        let l = (-lambda).exp();
        let mut k = 0;
        let mut p = self.uniform();
        while p > l {
            k += 1;
            p *= self.uniform();
        }
        k
    }
}

// ---------- Stationary bootstrap ----------

/// Random paths from blocks of the historical series. The block length is geometrically distributed
/// (mean `block` months); stocks, bonds, inflation and SARON always come from the same
/// month or year. At the end of the series it continues at the beginning (circular).
pub struct Bootstrap<'a> {
    pub data: &'a DataSet,
    pub fixed: &'a FixedCurves,
    pub start: YearMonth,
    pub months: usize,
    pub params: BootstrapParams,
}

impl Bootstrap<'_> {
    pub fn len(&self) -> usize {
        self.params.paths
    }

    pub fn is_empty(&self) -> bool {
        self.params.paths == 0
    }

    /// Sequence of historical positions (months or years of the series).
    fn positions(&self, rng: &mut Rng, count: usize, mean_len: f64) -> Vec<usize> {
        let n = self.data.len();
        let p_new = 1.0 / mean_len.max(1.0);
        let mut out = Vec::with_capacity(count);
        let mut i = rng.below(n);
        for j in 0..count {
            if j > 0 {
                i = if rng.uniform() < p_new {
                    rng.below(n)
                } else {
                    (i + 1) % n
                };
            }
            out.push(i);
        }
        out
    }

    pub fn path(&self, index: usize) -> MarketPath {
        let ds = self.data;
        let mut rng = Rng::new(self.params.seed, index as u64);
        let mut p = MarketPath::with_capacity(msg!(path_bootstrap, n = index + 1), self.months);
        let saron = |i: usize, m: YearMonth| {
            ds.saron
                .as_ref()
                .map(|s| s[i])
                .filter(|v| !v.is_nan())
                .unwrap_or_else(|| self.fixed.saron.at(m))
        };
        match ds.freq {
            Freq::Monthly => {
                let pos = self.positions(&mut rng, self.months, self.params.block);
                // Lead-in: the real months before the first block (circular like the bootstrap)
                let n = ds.len();
                p.pre_stocks = (1..=PRE_MONTHS)
                    .rev()
                    .map(|j| ds.stocks[(pos[0] + n * PRE_MONTHS - j) % n])
                    .collect();
                for (k, &i) in pos.iter().enumerate() {
                    let m = self.start.plus(k as i32);
                    p.stocks.push(ds.stocks[i]);
                    p.bonds.push(ds.bonds[i]);
                    p.inflation.push(ds.inflation[i]);
                    p.saron.push(saron(i, m));
                }
            }
            Freq::Yearly => {
                // One annual value per calendar year of the simulation; block length in years.
                let end = self.start.plus(self.months as i32 - 1);
                let years = (end.year() - self.start.year() + 1) as usize;
                let pos = self.positions(&mut rng, years, self.params.block / 12.0);
                // Lead-in: months in the start year before the start from the first year, the others from the year before (circular)
                let n = ds.len();
                p.pre_stocks = (1..=PRE_MONTHS as i32)
                    .rev()
                    .map(|j| {
                        let back = (self.start.year() - self.start.plus(-j).year()) as usize;
                        monthly(ds.stocks[(pos[0] + n - back % n) % n])
                    })
                    .collect();
                for k in 0..self.months {
                    let m = self.start.plus(k as i32);
                    let i = pos[(m.year() - self.start.year()) as usize];
                    p.stocks.push(monthly(ds.stocks[i]));
                    p.bonds.push(monthly(ds.bonds[i]));
                    p.inflation.push(ds.inflation[i]);
                    p.saron.push(saron(i, m));
                }
            }
        }
        p
    }
}

// ---------- Parametric ----------

/// #132: Sampling method of the parametric model (Student-t after Bailey, jumps via the distance to the
/// next jump day). New plans store it in `returns.parametric.sampler`; increase it on a change that changes the
/// paths and add a note in `upgrade`.
pub const SAMPLER: u32 = 1;

/// Trading days per month as in the previous program.
pub const TRADING_DAYS: usize = 21;

/// Stocks: geometric Brownian motion with Student-t shocks (scaled to variance 1) and
/// Poisson price jumps, computed in daily steps and combined into monthly returns.
/// Bonds and inflation: with `correlated` random per calendar year and correlated with the year's stock return
/// (#101), from a random stream of their own, so the stock paths stay the same; otherwise, like SARON,
/// the fixed values.
pub struct Parametric<'a> {
    pub fixed: &'a FixedCurves,
    pub start: YearMonth,
    pub months: usize,
    pub params: ParametricParams,
    pub correlated: Option<Correlated>,
}

/// #132: Price jumps, Poisson distributed with rate `lambda` per day. Instead of drawing a Poisson number every day (with the
/// default there is none on about 99.98 % of the days), the distance to the next day with a jump is drawn: it is
/// geometrically distributed with P(no jump) = exp(−λ), i.e. ⌊−ln U / λ⌋ with U uniformly distributed in (0, 1]. On that day
/// the count follows from the Poisson distribution conditional on «at least one jump». The distribution of the jumps
/// per day is the same as with one Poisson number per day.
struct Jumps {
    lambda: f64,
    /// P(exactly one jump | at least one) = λ·e^(−λ) / (1 − e^(−λ)).
    first: f64,
    /// Days without a jump until the next day with a jump.
    gap: u64,
}

impl Jumps {
    fn new(lambda: f64, rng: &mut Rng) -> Jumps {
        let mut j = Jumps {
            lambda,
            first: if lambda > 0.0 { lambda / lambda.exp_m1() } else { 0.0 },
            gap: u64::MAX,
        };
        j.next_gap(rng);
        j
    }

    fn next_gap(&mut self, rng: &mut Rng) {
        if self.lambda > 0.0 {
            let u = 1.0 - rng.uniform();
            // `as` clamps very large values to u64::MAX (then no further jump in the path)
            self.gap = (-u.ln() / self.lambda) as u64;
        }
    }

    /// Number of jumps on this day.
    fn today(&mut self, rng: &mut Rng) -> u32 {
        if self.gap > 0 {
            // without jumps (λ = 0) the distance stays u64::MAX and never reaches 0
            self.gap -= 1;
            return 0;
        }
        // Count ≥ 1 by inversion of the conditional Poisson distribution
        let u = rng.uniform();
        let (mut k, mut pk) = (1u32, self.first);
        let mut cdf = pk;
        while u >= cdf && pk > 0.0 {
            k += 1;
            pk *= self.lambda / k as f64;
            cdf += pk;
        }
        self.next_gap(rng);
        k
    }
}

/// Distinguishes the random stream for bonds and inflation from that of the stocks.
const JOINT_STREAM: u64 = 0x6a09_e667_f3bc_c908;

impl Parametric<'_> {
    pub fn len(&self) -> usize {
        self.params.paths
    }

    pub fn is_empty(&self) -> bool {
        self.params.paths == 0
    }

    pub fn path(&self, index: usize) -> MarketPath {
        let q = &self.params;
        let mut rng = Rng::new(q.seed, index as u64);
        let dt = 1.0 / (12.0 * TRADING_DAYS as f64);
        let scale = if q.df > 2.0 { ((q.df - 2.0) / q.df).sqrt() } else { 1.0 };
        let drift = (q.mu - 0.5 * q.sigma * q.sigma) * dt;
        let vol = q.sigma * dt.sqrt();
        let lambda = q.jump_lambda * dt;
        let mut p = MarketPath::with_capacity(msg!(path_random, n = index + 1), self.months);
        let mut jumps = Jumps::new(lambda, &mut rng);
        let mut month = |rng: &mut Rng| {
            let mut log = 0.0;
            for _ in 0..TRADING_DAYS {
                let z = if q.df > 0.0 {
                    rng.student_t(q.df) * scale
                } else {
                    rng.normal()
                };
                log += drift + vol * z;
                let n = jumps.today(rng);
                if n > 0 {
                    // As before: one jump size, raised to the power of the number of jumps.
                    log += n as f64 * (q.jump_mu + q.jump_sigma * rng.normal());
                }
            }
            log
        };
        // Lead-in (#103): 12 months from the same random stream, drawn before the path
        p.pre_stocks = (0..PRE_MONTHS).map(|_| month(&mut rng).exp() - 1.0).collect();
        // Log returns of the months, for the correlation with bonds and inflation (#132: not recomputed from the
        // return)
        let mut logs = Vec::with_capacity(if self.correlated.is_some() { self.months } else { 0 });
        for k in 0..self.months {
            let m = self.start.plus(k as i32);
            let log = month(&mut rng);
            if self.correlated.is_some() {
                logs.push(log);
            }
            p.stocks.push(log.exp() - 1.0);
            p.saron.push(self.fixed.saron.at(m));
            if self.correlated.is_none() {
                p.bonds.push(monthly(self.fixed.bonds.at(m)));
                p.inflation.push(self.fixed.inflation.at(m));
            }
        }
        if let Some(c) = &self.correlated {
            let mut rng = Rng::new(q.seed ^ JOINT_STREAM, index as u64);
            let mut k = 0;
            while k < self.months {
                // Months of this calendar year in the simulation
                let m = self.start.plus(k as i32);
                let len = (13 - m.month() as usize).min(self.months - k);
                // #116: Every pass makes progress, and `k` counts the months already written. Otherwise an
                // error (or a mutant like `k *= len`) would make the loop endless and fill the memory.
                debug_assert!(
                    len > 0 && k == p.bonds.len(),
                    "Parametric::path: Schleife kommt nicht voran"
                );
                let log: f64 = logs[k..k + len].iter().sum();
                let (e1, e2) = (rng.normal(), rng.normal());
                let (b, i) = c.draw(log, len, e1, e2);
                for _ in 0..len {
                    p.bonds.push(monthly(b));
                    p.inflation.push(i);
                }
                k += len;
            }
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::Curve;

    fn fixed() -> FixedCurves {
        FixedCurves {
            stocks: Curve::constant(0.0),
            bonds: Curve::constant(0.02),
            inflation: Curve::constant(0.01),
            saron: Curve::constant(0.005),
        }
    }

    #[test]
    fn rng_moments() {
        let mut r = Rng::new(1, 0);
        let n = 200_000;
        let (mut s1, mut s2) = (0.0, 0.0);
        for _ in 0..n {
            let z = r.normal();
            s1 += z;
            s2 += z * z;
        }
        assert!((s1 / n as f64).abs() < 0.01);
        assert!((s2 / n as f64 - 1.0).abs() < 0.01);
        // Student-t with df = 6: variance df/(df−2) = 1.5.
        let v: f64 = (0..n).map(|_| r.student_t(6.0).powi(2)).sum::<f64>() / n as f64;
        assert!((v - 1.5).abs() < 0.05, "{v}");
        let p: f64 = (0..n).map(|_| r.poisson(0.3) as f64).sum::<f64>() / n as f64;
        assert!((p - 0.3).abs() < 0.01, "{p}");
        // Same seed and stream ⇒ same sequence, different stream ⇒ different sequence.
        assert_eq!(Rng::new(7, 3).next_u64(), Rng::new(7, 3).next_u64());
        assert_ne!(Rng::new(7, 3).next_u64(), Rng::new(7, 4).next_u64());
    }

    #[test]
    fn bootstrap_keeps_months_together() {
        let n = 50;
        let ds = DataSet {
            name: "t".into(),
            freq: Freq::Monthly,
            first: YearMonth::new(2000, 1),
            stocks: (0..n).map(|i| i as f64).collect(),
            bonds: (0..n).map(|i| 1000.0 + i as f64).collect(),
            inflation: (0..n).map(|i| 2000.0 + i as f64).collect(),
            saron: Some(
                (0..n)
                    .map(|i| if i < 10 { f64::NAN } else { 3000.0 + i as f64 })
                    .collect(),
            ),
            ..Default::default()
        };
        let f = fixed();
        let b = Bootstrap {
            data: &ds,
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 600,
            params: BootstrapParams {
                paths: 10,
                block: 12.0,
                seed: 42,
            },
        };
        let p = b.path(3);
        assert_eq!(p.stocks.len(), 600);
        // #103: Lead-in = the 12 real months before the first block (circular)
        let i0 = p.stocks[0] as usize;
        let pre: Vec<f64> = (1..=12).rev().map(|j| ((i0 + n - j) % n) as f64).collect();
        assert_eq!(p.pre_stocks, pre);
        let mut jumps = 0;
        for k in 0..600 {
            let i = p.stocks[k] as usize;
            assert_eq!(p.bonds[k], 1000.0 + i as f64);
            assert_eq!(p.inflation[k], 2000.0 + i as f64);
            assert_eq!(p.saron[k], if i < 10 { 0.005 } else { 3000.0 + i as f64 });
            if k > 0 && i != (p.stocks[k - 1] as usize + 1) % n {
                jumps += 1;
            }
        }
        // Mean block length 12 months ⇒ about 50 new blocks in 600 months.
        assert!((30..75).contains(&jumps), "{jumps}");
        assert_eq!(b.path(3).stocks, p.stocks, "reproduzierbar");
        assert_ne!(b.path(4).stocks, p.stocks);
    }

    #[test]
    fn parametric_matches_model() {
        let f = fixed();
        let q = ParametricParams {
            paths: 2000,
            mu: 0.07,
            sigma: 0.15,
            df: 6.0,
            jump_lambda: 0.0,
            jump_mu: 0.0,
            jump_sigma: 0.0,
            seed: 1,
            auto: false,
            ..Default::default()
        };
        let g = Parametric {
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 12,
            params: q,
            correlated: None,
        };
        // Annual log return: mean (mu − σ²/2), standard deviation σ.
        let logs: Vec<f64> = (0..g.len())
            .map(|i| g.path(i).stocks.iter().map(|r| (1.0 + r).ln()).sum())
            .collect();
        let mean = logs.iter().sum::<f64>() / logs.len() as f64;
        let sd = (logs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / logs.len() as f64).sqrt();
        assert!((mean - (0.07 - 0.5 * 0.0225)).abs() < 0.01, "{mean}");
        assert!((sd - 0.15).abs() < 0.01, "{sd}");
        let p = g.path(0);
        assert!((p.bonds[0] - monthly(0.02)).abs() < 1e-15);
        assert_eq!(p.inflation[0], 0.01);
    }

    #[test]
    fn empty_sources() {
        let ds = DataSet::default();
        let f = fixed();
        let start = YearMonth::new(2030, 1);
        let b = |paths| Bootstrap {
            data: &ds,
            fixed: &f,
            start,
            months: 1,
            params: BootstrapParams {
                paths,
                block: 12.0,
                seed: 1,
            },
        };
        assert!(b(0).is_empty() && !b(3).is_empty());
        let p = |paths| Parametric {
            fixed: &f,
            start,
            months: 1,
            params: ParametricParams {
                paths,
                ..Default::default()
            },
            correlated: None,
        };
        assert!(p(0).is_empty() && !p(3).is_empty());
    }

    /// #103: Annual series, start in July: January to June of the start year from the first year drawn, July to
    /// December of the previous year from the year before (circular).
    #[test]
    fn issue_103_yearly_bootstrap_pre_run_by_calendar_year() {
        let n = 20;
        let ds = DataSet {
            name: "j".into(),
            freq: Freq::Yearly,
            first: YearMonth::new(1950, 1),
            stocks: (0..n).map(|i| i as f64 * 0.01).collect(),
            bonds: vec![0.0; n],
            inflation: vec![0.0; n],
            ..Default::default()
        };
        let f = fixed();
        let b = Bootstrap {
            data: &ds,
            fixed: &f,
            start: YearMonth::new(2030, 7),
            months: 24,
            params: BootstrapParams {
                paths: 50,
                block: 12.0,
                seed: 3,
            },
        };
        let mut wrapped = false;
        for k in 0..b.len() {
            let p = b.path(k);
            let i0 = (((1.0 + p.stocks[0]).powi(12) - 1.0) / 0.01).round() as usize;
            let prev = (i0 + n - 1) % n;
            wrapped |= i0 == 0;
            assert_eq!(p.pre_stocks.len(), PRE_MONTHS);
            for r in &p.pre_stocks[..6] {
                assert!((r - monthly(ds.stocks[prev])).abs() < 1e-12, "Vorjahr");
            }
            for r in &p.pre_stocks[6..] {
                assert_eq!(*r, p.stocks[0], "Startjahr");
            }
        }
        assert!(wrapped, "kein Pfad beginnt am Anfang der Reihe");
    }

    // Annual series: one value per calendar year, block length in years (block / 12).
    #[test]
    fn yearly_bootstrap_block_length_in_years() {
        let n = 50;
        let ds = DataSet {
            name: "j".into(),
            freq: Freq::Yearly,
            first: YearMonth::new(1950, 1),
            stocks: (0..n).map(|i| i as f64 * 0.01).collect(),
            bonds: vec![0.0; n],
            inflation: vec![0.0; n],
            ..Default::default()
        };
        let f = fixed();
        let b = Bootstrap {
            data: &ds,
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 30 * 12,
            params: BootstrapParams {
                paths: 200,
                block: 120.0,
                seed: 9,
            },
        };
        let index = |r: f64| ((1.0 + r).powi(12) - 1.0) / 0.01;
        let (mut steps, mut follow) = (0, 0);
        for k in 0..b.len() {
            let p = b.path(k);
            for y in 0..30 {
                // within a year the same value
                assert_eq!(p.stocks[12 * y], p.stocks[12 * y + 11]);
            }
            for y in 1..30 {
                let (a, c) = (
                    index(p.stocks[12 * (y - 1)]).round() as usize,
                    index(p.stocks[12 * y]).round() as usize,
                );
                steps += 1;
                if c == (a + 1) % n {
                    follow += 1;
                }
            }
        }
        // Mean block length 10 years ⇒ new block with probability 0.1; otherwise the following year 0.9 + 0.1/50.
        let share = follow as f64 / steps as f64;
        assert!((share - 0.902).abs() < 0.03, "{share}");
    }

    #[test]
    fn parametric_with_two_degrees_of_freedom_is_random() {
        let f = fixed();
        let q = ParametricParams {
            paths: 1,
            df: 2.0,
            jump_lambda: 0.0,
            ..Default::default()
        };
        let g = Parametric {
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 24,
            params: q,
            correlated: None,
        };
        let p = g.path(0);
        assert!(p.stocks.windows(2).any(|w| w[0] != w[1]));
    }

    /// Cumulative distribution function of the Student-t distribution with `df` ≥ 1, numerically: with x = √df·tan θ the density is
    /// proportional to cos^(df−1) θ on (−π/2, π/2) (Simpson's rule).
    fn t_cdf(df: f64, t: f64) -> f64 {
        let integral = |b: f64| {
            let (a, n) = (-std::f64::consts::FRAC_PI_2, 20_000);
            let h = (b - a) / n as f64;
            let f = |x: f64| x.cos().max(0.0).powf(df - 1.0);
            let inner: f64 = (1..n)
                .map(|i| f(a + i as f64 * h) * if i % 2 == 1 { 4.0 } else { 2.0 })
                .sum();
            (f(a) + inner + f(b)) * h / 3.0
        };
        integral((t / df.sqrt()).atan()) / integral(std::f64::consts::FRAC_PI_2)
    }

    // #132/AK4: The Student-t draw (Bailey) has the distribution function of the t-distribution, for the default df = 6
    // (separate computation with ∛w), integer and non-integer df and df ≤ 2 (infinite variance).
    // slow: 200'000 draws for each of six degrees of freedom
    #[test]
    fn issue_132_ak4_student_t_matches_distribution() {
        assert!((t_cdf(1.0, 1.0) - 0.75).abs() < 1e-9, "Cauchy");
        assert!((t_cdf(2.0, 1.0) - (0.5 + 0.5 / 3f64.sqrt())).abs() < 1e-9);
        let n = 200_000;
        for (k, df) in [1.0, 2.0, 2.5, 4.5, 6.0, 30.0].into_iter().enumerate() {
            let mut r = Rng::new(132, k as u64);
            let xs: Vec<f64> = (0..n).map(|_| r.student_t(df)).collect();
            assert!(xs.iter().all(|x| x.is_finite()), "df {df}");
            for t in [-3.0, -1.5, -0.5, 0.0, 0.3, 1.0, 2.0, 4.0] {
                let share = xs.iter().filter(|&&x| x <= t).count() as f64 / n as f64;
                let exact = t_cdf(df, t);
                assert!((share - exact).abs() < 0.005, "df {df}, t {t}: {share} statt {exact}");
            }
        }
    }

    // #132/AK4: The jumps per day are Poisson distributed, even though only the distance to the next jump day is drawn;
    // without jumps (λ = 0) no random number is used.
    #[test]
    fn issue_132_ak4_jumps_are_poisson_per_day() {
        for (k, lambda) in [0.3, 0.02, 0.001].into_iter().enumerate() {
            let mut r = Rng::new(132, 100 + k as u64);
            let mut j = Jumps::new(lambda, &mut r);
            let days = 2_000_000;
            let mut count = [0usize; 4];
            for _ in 0..days {
                count[(j.today(&mut r) as usize).min(3)] += 1;
            }
            let pois = |m: i32| (-lambda).exp() * lambda.powi(m) / (1..=m).product::<i32>().max(1) as f64;
            for (m, &c) in count.iter().enumerate().take(3) {
                let exact = pois(m as i32);
                let sd = (exact * (1.0 - exact) / days as f64).sqrt();
                assert!(
                    (c as f64 / days as f64 - exact).abs() < 5.0 * sd + 1e-7,
                    "λ {lambda}, {m} Sprünge: {} statt {exact}",
                    c as f64 / days as f64
                );
            }
        }
        let mut r = Rng::new(1, 0);
        let mut j = Jumps::new(0.0, &mut r);
        assert!((0..10_000).all(|_| j.today(&mut r) == 0));
        assert_eq!(r.next_u64(), Rng::new(1, 0).next_u64(), "keine Zufallszahl verbraucht");
    }

    // #132/AK4: Annual log return of a path with price jumps: mean (μ − σ²/2) + λ·μJ, variance σ² plus that of the jumps
    // (per day N·(μJ + σJ·Z) with N Poisson(λ·dt): E[N²]·(μJ² + σJ²) − (E[N]·μJ)²).
    // slow: 20'000 parametric paths with price jumps
    #[test]
    fn issue_132_ak4_path_with_jumps_matches_model() {
        let f = fixed();
        let (mu, sigma, lam, jm, js) = (0.07, 0.15, 2.0, -0.1, 0.05);
        let g = Parametric {
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 12,
            params: ParametricParams {
                paths: 20_000,
                mu,
                sigma,
                df: 4.5,
                jump_lambda: lam,
                jump_mu: jm,
                jump_sigma: js,
                seed: 3,
                auto: false,
                ..Default::default()
            },
            correlated: None,
        };
        let logs: Vec<f64> = (0..g.len())
            .map(|i| g.path(i).stocks.iter().map(|r| (1.0 + r).ln()).sum())
            .collect();
        let n = logs.len() as f64;
        let mean = logs.iter().sum::<f64>() / n;
        let var = logs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let days = (12 * TRADING_DAYS) as f64;
        let l = lam / days;
        let jump_var = days * ((l + l * l) * (jm * jm + js * js) - (l * jm).powi(2));
        let (m0, v0) = (mu - 0.5 * sigma * sigma + lam * jm, sigma * sigma + jump_var);
        assert!((mean - m0).abs() < 4.0 * (v0 / n).sqrt(), "{mean} statt {m0}");
        assert!((var / v0 - 1.0).abs() < 0.05, "{var} statt {v0}");
    }

    // Same seed ⇒ same paths, also across versions. A deliberate change of the random number generator or
    // of the model needs, like the golden files, a justification in the issue (then set the values anew).
    #[test]
    fn parametric_paths_are_stable_across_versions() {
        let f = fixed();
        let q = ParametricParams {
            paths: 1,
            df: 2.0,
            jump_lambda: 3.0,
            ..Default::default()
        };
        let g = Parametric {
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 3,
            params: q,
            correlated: None,
        };
        let p = g.path(0);
        // Tolerance for small differences of the math library between platforms.
        // #103: Since the lead-in (12 months before the path from the same stream) the path starts with the 13th month.
        // #132: new values (Student-t after Bailey, jumps via the distance to the next jump day).
        let expected = [-0.028948303776431428, 0.022424257414504822, 0.004603269924865483];
        for (a, b) in p.stocks.iter().zip(expected) {
            assert!((a - b).abs() < 1e-9, "{:?}", p.stocks);
        }
        // Without jumps and with a normal distribution (no random numbers are drawn for jumps).
        let q = ParametricParams {
            paths: 1,
            df: 0.0,
            jump_lambda: 0.0,
            ..Default::default()
        };
        let p = Parametric {
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 3,
            params: q,
            correlated: None,
        }
        .path(0);
        let expected = [0.02816194106109493, 0.04537789657227864, 0.08217147870817776];
        for (a, b) in p.stocks.iter().zip(expected) {
            assert!((a - b).abs() < 1e-9, "{:?}", p.stocks);
        }
        // #103: The lead-in is the 12 months before from the same stream; its first months are the earlier values
        // of the path (before #103 the path itself).
        assert_eq!(p.pre_stocks.len(), PRE_MONTHS);
        let before = [0.06842020080156996, 0.08498775592559737, 0.0484071107147952];
        for (a, b) in p.pre_stocks.iter().zip(before) {
            assert!((a - b).abs() < 1e-9, "{:?}", p.pre_stocks);
        }
    }
}
