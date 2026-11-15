//! Monte Carlo: zufällige Renditepfade aus der Historie (stationärer Bootstrap) oder aus einem
//! parametrischen Aktienmodell (Student-t mit Kurssprüngen, wie das bisherige Programm).
//!
//! Die Pfade werden erst beim Rechnen erzeugt, Pfad `i` immer aus dem Startwert und `i`. Dadurch
//! braucht auch eine Simulation mit 50'000 Pfaden wenig Speicher, und jeder Pfad lässt sich für die
//! Detailansicht exakt wiederholen.

use crate::calibrate::Correlated;
use crate::market::{monthly, DataSet, FixedCurves, Freq, MarketPath};
use crate::model::{BootstrapParams, ParametricParams};
use crate::time::YearMonth;

// ---------- Zufallszahlen ----------

/// xoshiro256** mit splitmix64-Initialisierung: schnell, gute Qualität, reproduzierbar.
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
    /// Eigener Zufallsstrom pro (Startwert, Pfadnummer).
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

    /// Gleichverteilt in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Ganze Zahl in 0..n.
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_u64() as u128 * n as u128) >> 64) as usize
    }

    /// Standardnormalverteilt (Polar-Methode, das zweite Ergebnis wird aufbewahrt).
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

    /// Gammaverteilt mit Form `k` und Skala 1 (Marsaglia-Tsang).
    pub fn gamma(&mut self, k: f64) -> f64 {
        if k < 1.0 {
            let u = self.uniform().max(f64::MIN_POSITIVE);
            return self.gamma(k + 1.0) * u.powf(1.0 / k);
        }
        let d = k - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        loop {
            let x = self.normal();
            let v = (1.0 + c * x).powi(3);
            if v <= 0.0 {
                continue;
            }
            let u = self.uniform();
            if u < 1.0 - 0.0331 * x.powi(4) || u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
                return d * v;
            }
        }
    }

    /// Student-t mit `df` Freiheitsgraden.
    pub fn student_t(&mut self, df: f64) -> f64 {
        self.normal() / (2.0 * self.gamma(df / 2.0) / df).sqrt()
    }

    /// Poissonverteilt (für kleine Raten).
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

// ---------- Stationärer Bootstrap ----------

/// Zufällige Pfade aus Blöcken der historischen Reihe. Die Blocklänge ist geometrisch verteilt
/// (Mittelwert `block` Monate); Aktien, Obligationen, Inflation und SARON stammen immer aus demselben
/// Monat bzw. Jahr. Am Ende der Reihe geht es am Anfang weiter (zirkulär).
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

    /// Folge historischer Positionen (Monate bzw. Jahre der Reihe).
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
        let mut p = MarketPath::with_capacity(format!("Bootstrap-Pfad {}", index + 1), self.months);
        let saron = |i: usize, m: YearMonth| {
            ds.saron
                .as_ref()
                .map(|s| s[i])
                .filter(|v| !v.is_nan())
                .unwrap_or_else(|| self.fixed.saron.at(m))
        };
        match ds.freq {
            Freq::Monthly => {
                for (k, &i) in self
                    .positions(&mut rng, self.months, self.params.block)
                    .iter()
                    .enumerate()
                {
                    let m = self.start.plus(k as i32);
                    p.stocks.push(ds.stocks[i]);
                    p.bonds.push(ds.bonds[i]);
                    p.inflation.push(ds.inflation[i]);
                    p.saron.push(saron(i, m));
                }
            }
            Freq::Yearly => {
                // Ein Jahreswert pro Kalenderjahr der Simulation; Blocklänge in Jahren.
                let end = self.start.plus(self.months as i32 - 1);
                let years = (end.year() - self.start.year() + 1) as usize;
                let pos = self.positions(&mut rng, years, self.params.block / 12.0);
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

// ---------- Parametrisch ----------

/// Handelstage pro Monat wie im bisherigen Programm.
pub const TRADING_DAYS: usize = 21;

/// Aktien: geometrische Brownsche Bewegung mit Student-t-Schocks (auf Varianz 1 skaliert) und
/// Poisson-Kurssprüngen, in Tagesschritten gerechnet und zu Monatsrenditen zusammengefasst.
/// Obligationen und Inflation: mit `correlated` je Kalenderjahr zufällig und mit der Aktienrendite des Jahres
/// korreliert (FEA-79), aus einem eigenen Zufallsstrom, damit die Aktienpfade gleich bleiben; sonst wie der SARON
/// die fixen Werte.
pub struct Parametric<'a> {
    pub fixed: &'a FixedCurves,
    pub start: YearMonth,
    pub months: usize,
    pub params: ParametricParams,
    pub correlated: Option<Correlated>,
}

/// Unterscheidet den Zufallsstrom für Obligationen und Inflation von dem der Aktien.
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
        let mut p = MarketPath::with_capacity(format!("Zufallspfad {}", index + 1), self.months);
        for k in 0..self.months {
            let m = self.start.plus(k as i32);
            let mut log = 0.0;
            for _ in 0..TRADING_DAYS {
                let z = if q.df > 0.0 {
                    rng.student_t(q.df) * scale
                } else {
                    rng.normal()
                };
                log += drift + vol * z;
                if lambda > 0.0 {
                    let jumps = rng.poisson(lambda);
                    if jumps > 0 {
                        // Wie bisher: eine Sprunghöhe, mit der Anzahl Sprünge potenziert.
                        log += jumps as f64 * (q.jump_mu + q.jump_sigma * rng.normal());
                    }
                }
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
                // Monate dieses Kalenderjahres in der Simulation
                let m = self.start.plus(k as i32);
                let len = (13 - m.month() as usize).min(self.months - k);
                let log: f64 = p.stocks[k..k + len].iter().map(|r| (1.0 + r).ln()).sum();
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
        // Student-t mit df = 6: Varianz df/(df−2) = 1.5.
        let v: f64 = (0..n).map(|_| r.student_t(6.0).powi(2)).sum::<f64>() / n as f64;
        assert!((v - 1.5).abs() < 0.05, "{v}");
        let g: f64 = (0..n).map(|_| r.gamma(0.5)).sum::<f64>() / n as f64;
        assert!((g - 0.5).abs() < 0.01, "{g}");
        let p: f64 = (0..n).map(|_| r.poisson(0.3) as f64).sum::<f64>() / n as f64;
        assert!((p - 0.3).abs() < 0.01, "{p}");
        // Gleicher Startwert und Strom ⇒ gleiche Folge, anderer Strom ⇒ andere Folge.
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
        // Mittlere Blocklänge 12 Monate ⇒ etwa 50 neue Blöcke in 600 Monaten.
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
        };
        let g = Parametric {
            fixed: &f,
            start: YearMonth::new(2030, 1),
            months: 12,
            params: q,
            correlated: None,
        };
        // Jahres-Logrendite: Mittel (mu − σ²/2), Standardabweichung σ.
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

    // FEA-29: Gamma mit Form < 1 hat viel Masse nahe 0: P(X < 0.01) = P(|Z| < √0.02) ≈ 0.1125 bei Form 0.5.
    #[test]
    fn gamma_below_one_has_mass_near_zero() {
        let mut r = Rng::new(5, 0);
        let n = 200_000;
        let small = (0..n).filter(|_| r.gamma(0.5) < 0.01).count() as f64 / n as f64;
        assert!((small - 0.1125).abs() < 0.005, "{small}");
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

    // Jahresreihe: ein Wert pro Kalenderjahr, Blocklänge in Jahren (block / 12).
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
                // innerhalb eines Jahres derselbe Wert
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
        // Mittlere Blocklänge 10 Jahre ⇒ neuer Block mit Wahrscheinlichkeit 0.1; Folgejahr sonst 0.9 + 0.1/50.
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

    // Gleicher Startwert ⇒ gleiche Pfade, auch über Versionen. Eine bewusste Änderung des Zufallsgenerators oder
    // des Modells braucht wie die Golden-Dateien eine Begründung im Issue (dann die Werte neu setzen).
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
        // Toleranz für kleine Unterschiede der Mathematikbibliothek zwischen Plattformen.
        let expected = [0.06150036486902888, -0.3369147970351759, 0.008840613985451329];
        for (a, b) in p.stocks.iter().zip(expected) {
            assert!((a - b).abs() < 1e-9, "{:?}", p.stocks);
        }
        // Ohne Sprünge und mit Normalverteilung (es werden keine Zufallszahlen für Sprünge gezogen).
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
        let expected = [0.06842020080156996, 0.08498775592559737, 0.0484071107147952];
        for (a, b) in p.stocks.iter().zip(expected) {
            assert!((a - b).abs() < 1e-9, "{:?}", p.stocks);
        }
    }
}
