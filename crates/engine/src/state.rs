//! #218: Characteristic curves: a parameter given as support points over a state variable of the run (for example the
//! stock share over the withdrawal rate), linear in between. The simulation evaluates the curve when it rebalances.

use crate::model::{CurveEval, StateCurve, StateInput};
use crate::{Error, Result};

/// A compiled characteristic curve: support points sorted by their input value.
#[derive(Clone, Debug, PartialEq)]
pub struct StateLine {
    pub input: StateInput,
    pub monthly: bool,
    points: Vec<(f64, f64)>,
}

impl StateLine {
    pub fn compile(c: &StateCurve) -> Result<Self> {
        if c.points.is_empty() {
            return Err(Error::Invalid(msg!(curve_points_empty)));
        }
        let mut points = c.points.clone();
        // Sort stably: for the same input value the later point applies above it.
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        Ok(StateLine {
            input: c.input,
            monthly: c.eval == CurveEval::Monthly,
            points,
        })
    }

    /// Value at input `x`: the first value below the first point, the last above the last, linear in between. An
    /// input that cannot be computed (NaN) gets the first value.
    pub fn at(&self, x: f64) -> f64 {
        let p = &self.points;
        if x.is_nan() {
            return p[0].1;
        }
        match p.partition_point(|q| q.0 <= x) {
            0 => p[0].1,
            i if i == p.len() => p[i - 1].1,
            i => {
                let ((x1, a), (x2, b)) = (p[i - 1], p[i]);
                a + (b - a) * (x - x1) / (x2 - x1)
            }
        }
    }

    /// The values of the support points (for the range check).
    pub fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.points.iter().map(|p| p.1)
    }
}

/// Ratio `a / b` of two amounts; with `b <= 0` infinite if `a > 0` and otherwise 0.
pub fn ratio(a: f64, b: f64) -> f64 {
    if b > 0.0 {
        a / b
    } else if a > 0.0 {
        f64::INFINITY
    } else {
        0.0
    }
}

/// State of a run that only plans with a characteristic curve keep: the withdrawals of the last 12 months, the highest
/// stock index so far and the value of the curve until its next evaluation.
#[derive(Clone, Debug, Default)]
pub struct Tracker {
    withdrawals: [f64; 12],
    months: usize,
    peak: f64,
    peak_upto: usize,
    pub value: f64,
    year: Option<i32>,
}

impl Tracker {
    /// Records the withdrawal of a month (expenses minus income; negative when the household saves).
    pub fn push_withdrawal(&mut self, amount: f64) {
        self.withdrawals[self.months % 12] = amount;
        self.months += 1;
    }

    /// Withdrawal of the last 12 months; in the first year the months so far extrapolated to 12, before the first
    /// month `first` × 12. Never negative.
    pub fn withdrawal_year(&self, first: f64) -> f64 {
        let n = self.months.min(12);
        let sum = if n == 0 {
            first * 12.0
        } else {
            self.withdrawals[..n].iter().sum::<f64>() * 12.0 / n as f64
        };
        sum.max(0.0)
    }

    /// Distance of the level `hist[i]` below the highest level of `hist[..=i]` (0 at the peak).
    pub fn drawdown(&mut self, hist: &[f64], i: usize) -> f64 {
        while self.peak_upto <= i && self.peak_upto < hist.len() {
            self.peak = self.peak.max(hist[self.peak_upto]);
            self.peak_upto += 1;
        }
        if self.peak > 0.0 {
            1.0 - hist[i] / self.peak
        } else {
            0.0
        }
    }

    /// Whether the curve is evaluated now: every time with `monthly`, otherwise at the first rebalancing of a year.
    pub fn due(&mut self, year: i32, monthly: bool) -> bool {
        let due = monthly || self.year != Some(year);
        self.year = Some(year);
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(points: &[(f64, f64)]) -> StateLine {
        StateLine::compile(&StateCurve {
            mode: crate::model::CurveMode::Curve,
            input: StateInput::WithdrawalRate,
            points: points.to_vec(),
            eval: CurveEval::Yearly,
        })
        .unwrap()
    }

    // #218/AK2: below the first point the first value, above the last the last value, linear in between.
    #[test]
    fn issue_218_ak2_curve_before_between_after() {
        let c = line(&[(0.06, 0.4), (0.02, 0.8)]);
        assert_eq!(c.at(0.0), 0.8);
        assert_eq!(c.at(0.02), 0.8);
        assert!((c.at(0.04) - 0.6).abs() < 1e-12);
        assert_eq!(c.at(0.06), 0.4);
        assert_eq!(c.at(f64::INFINITY), 0.4);
        assert_eq!(c.at(f64::NEG_INFINITY), 0.8);
        assert_eq!(c.at(f64::NAN), 0.8);
        assert_eq!(c.values().collect::<Vec<_>>(), vec![0.8, 0.4]);
        // A single point is constant; the same input twice is a jump (the later point applies from there on).
        assert_eq!(line(&[(1.0, 0.5)]).at(-3.0), 0.5);
        let jump = line(&[(0.0, 0.2), (1.0, 0.4), (1.0, 0.9), (2.0, 1.0)]);
        assert!((jump.at(0.5) - 0.3).abs() < 1e-12);
        assert_eq!(jump.at(1.0), 0.9);
        assert!((jump.at(1.5) - 0.95).abs() < 1e-12);
        let empty = StateLine::compile(&StateCurve {
            mode: crate::model::CurveMode::Curve,
            input: StateInput::Age,
            points: vec![],
            eval: CurveEval::Monthly,
        });
        assert_eq!(empty.unwrap_err().message().code(), "curve_points_empty");
    }

    // #218/AK1: ratio, withdrawals of the last 12 months, drawdown and the yearly evaluation.
    #[test]
    fn issue_218_ak1_tracker_withdrawal_drawdown_and_due() {
        assert_eq!(ratio(4.0, 100.0), 0.04);
        assert_eq!(ratio(4.0, 0.0), f64::INFINITY);
        assert_eq!(ratio(-4.0, -1.0), 0.0);
        assert_eq!(ratio(0.0, 0.0), 0.0);
        assert_eq!(ratio(0.0, -1.0), 0.0);
        let mut t = Tracker::default();
        assert_eq!(t.withdrawal_year(1000.0), 12_000.0);
        assert_eq!(t.withdrawal_year(-1000.0), 0.0);
        t.push_withdrawal(1000.0);
        t.push_withdrawal(3000.0);
        assert_eq!(t.withdrawal_year(0.0), 24_000.0);
        for _ in 0..12 {
            t.push_withdrawal(500.0);
        }
        assert_eq!(t.withdrawal_year(0.0), 6000.0);
        t.push_withdrawal(-100_000.0);
        assert_eq!(t.withdrawal_year(0.0), 0.0);

        let hist = [1.0, 2.0, 1.5, 1.0, 3.0];
        assert_eq!(t.drawdown(&hist, 0), 0.0);
        assert_eq!(t.drawdown(&hist, 2), 0.25);
        assert_eq!(t.drawdown(&hist, 3), 0.5);
        assert_eq!(t.drawdown(&hist, 4), 0.0);
        assert_eq!(Tracker::default().drawdown(&[0.0], 0), 0.0);

        assert!(t.due(2030, false));
        assert!(!t.due(2030, false));
        assert!(t.due(2030, true));
        assert!(t.due(2031, false));
    }
}
