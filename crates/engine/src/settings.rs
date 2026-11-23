//! Application-wide settings the calculation engine needs (#25).
//!
//! The user interface stores them locally (not in the plan file) and passes them with every call as `limits` in the
//! plan. If they are missing (command line, tests), the default values of this file apply. All places that need a limit
//! read it from [`Limits`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default of the milestones «AHV-Bezug» and «Bezug Pensionskasse» in years (#51).
pub const DEFAULT_AGE: f64 = 65.0;

/// Limits for the milestones «AHV-Bezug» and «Bezug Pensionskasse» and the pillar 3a maximum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Limits {
    /// AHV payout: lower and upper limit in years.
    pub ahv_min: f64,
    pub ahv_max: f64,
    /// Pension fund payout: lower and upper limit in years.
    pub pk_min: f64,
    pub pk_max: f64,
    /// Maximum pillar 3a contribution per year.
    pub pillar3a_max: f64,
    /// #152: Maximum pillar 3a contribution per year for persons without a pension fund («large» pillar 3a, at most
    /// 20 % of the earned income).
    pub pillar3a_max_large: f64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            ahv_min: 63.0,
            ahv_max: 70.0,
            pk_min: 58.0,
            pk_max: 65.0,
            pillar3a_max: 7258.0,
            pillar3a_max_large: 36288.0,
        }
    }
}

impl Limits {
    /// Errors in the settings themselves (user interface and validation). Empty = valid.
    pub fn problems(&self) -> Vec<crate::message::Message> {
        let mut out = vec![];
        let whole = |x: f64| x.fract() == 0.0 && (0.0..=130.0).contains(&x);
        for (what, lo, hi) in [
            (msg!(term_ahv_payout), self.ahv_min, self.ahv_max),
            (msg!(term_pk_payout), self.pk_min, self.pk_max),
        ] {
            if !whole(lo) || !whole(hi) {
                out.push(msg!(limits_not_whole_years, what = what));
            } else if lo > hi {
                out.push(msg!(limits_reversed, what = what, lo = lo, hi = hi));
            }
        }
        if !self.pillar3a_max.is_finite() || self.pillar3a_max < 0.0 {
            out.push(msg!(limits_pillar3a_max));
        }
        if !self.pillar3a_max_large.is_finite() || self.pillar3a_max_large < 0.0 {
            out.push(msg!(limits_pillar3a_max_large));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #25/AK1: Default values of the settings.
    #[test]
    fn defaults() {
        let l = Limits::default();
        assert_eq!(
            (l.ahv_min, l.ahv_max, l.pk_min, l.pk_max, l.pillar3a_max),
            (63.0, 70.0, 58.0, 65.0, 7258.0)
        );
        assert!(l.problems().is_empty());
        // Missing fields: default value.
        let p: Limits = serde_json::from_str(r#"{"pkMax": 67}"#).unwrap();
        assert_eq!((p.pk_max, p.ahv_min), (67.0, 63.0));
    }

    // #25/AK4: Validation of the input.
    #[test]
    fn problems() {
        let bad = |f: fn(&mut Limits)| {
            let mut l = Limits::default();
            f(&mut l);
            l.problems()
        };
        assert!(bad(|l| l.ahv_min = 71.0)[0].contains("lower limit (71) is above the upper limit (70)"));
        assert!(bad(|l| l.ahv_min = 70.0).is_empty(), "untere = obere Grenze ist gültig");
        assert!(bad(|l| l.pk_max = 64.5)[0].contains("whole years"));
        assert!(bad(|l| l.pk_min = -1.0)[0].contains("whole years"));
        assert!(bad(|l| l.pillar3a_max = -1.0)[0].contains("0 or more"));
        assert!(bad(|l| l.pillar3a_max = f64::NAN)[0].contains("0 or more"));
        assert!(bad(|l| l.pillar3a_max = 0.0).is_empty());
    }

    /// #152/AK2: Default of the large pillar 3a (BSV, as of 2025/26) and its check.
    #[test]
    fn issue_152_ak2_large_pillar3a_limit() {
        assert_eq!(Limits::default().pillar3a_max_large, 36288.0);
        let mut l = Limits {
            pillar3a_max_large: -1.0,
            ..Limits::default()
        };
        assert!(l.problems()[0].contains("Large pillar 3a"));
        l.pillar3a_max_large = f64::INFINITY;
        assert_eq!(l.problems().len(), 1);
        l.pillar3a_max_large = 0.0;
        assert!(l.problems().is_empty());
    }
}
