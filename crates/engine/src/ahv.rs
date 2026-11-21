//! #191: Flexible AHV payout: reduction for an early payout, supplement for a deferred payout.
//!
//! The plan holds the expected monthly AHV pension at the reference age (65), as the pension forecast of the
//! compensation office states it. If the person's AHV payout lies before or after the reference age, the simulation
//! computes with the pension adjusted by the rates of this file, for the whole payout duration (the AHV fixes the
//! reduction or supplement as an amount and adjusts it with the pensions; with the same inflation this equals the
//! percentage over the whole payout).
//!
//! Source: leaflet 3.04 «Flexibler Rentenbezug» of the AHV/IV information office, edition November 2025, valid from
//! 1 January 2026 (order number 3.04/d, 3.04-26/01): item 4 (reduction table for an early payout) and item 14
//! (supplement table for a deferral). Not included: partial early payout or deferral (20–80 %), the reduced rates and
//! the early payout from 62 for women of the transitional generation (born 1961–1969) and their lower reference age.

/// Year from which the rates of this file apply (leaflet 3.04, edition November 2025, valid from 1 January 2026).
pub const RATES_VALID_FROM: i32 = 2026;

/// Reference age in months (65 years).
pub const REFERENCE_AGE_MONTHS: i32 = 65 * 12;

/// Reduction in percent for an early payout of 0 to 24 months before the reference age (leaflet 3.04, item 4:
/// 6.8 % per year, month by month; the earliest payout is at 63, i.e. at most 24 months, 13.6 %).
pub const EARLY_REDUCTION_PCT: [f64; 25] = [
    0.0, 0.6, 1.1, 1.7, 2.3, 2.8, 3.4, 4.0, 4.5, 5.1, 5.7, 6.2, // 0 years, 0–11 months
    6.8, 7.4, 7.9, 8.5, 9.1, 9.6, 10.2, 10.8, 11.3, 11.9, 12.5, 13.0, // 1 year, 0–11 months
    13.6, // 2 years
];

/// Supplement in percent for a deferral of 1 year to 5 years in quarter steps (leaflet 3.04, item 14): index 0 =
/// 1 year (12–14 months), index 1 = 1 year 3 months (15–17 months), …, index 16 = 5 years.
pub const DEFERRAL_SUPPLEMENT_PCT: [f64; 17] = [
    5.2, 6.6, 8.0, 9.4, // 1 year, 0–2, 3–5, 6–8, 9–11 months
    10.8, 12.3, 13.9, 15.5, // 2 years
    17.1, 18.8, 20.5, 22.2, // 3 years
    24.0, 25.8, 27.7, 29.6, // 4 years
    31.5, // 5 years
];

/// Shortest deferral in months (1 year); a later payout less than one year after the reference age gets no supplement.
pub const MIN_DEFERRAL_MONTHS: i32 = 12;

/// Adjustment rate for a payout `offset` months after the reference age (negative = early payout): −0.136 for 24
/// months early, +0.171 for 36 months deferral, 0 at the reference age. An early payout of more than 24 months counts
/// as 24 months (13.6 %), a deferral of more than 5 years as 5 years (31.5 %); a deferral under one year gets nothing.
pub fn rate(offset: i32) -> f64 {
    if offset < 0 {
        let months = offset.unsigned_abs().min(24) as usize;
        -EARLY_REDUCTION_PCT[months] / 100.0
    } else if offset < MIN_DEFERRAL_MONTHS {
        0.0
    } else {
        let quarters = ((offset - MIN_DEFERRAL_MONTHS) / 3).min(16) as usize;
        DEFERRAL_SUPPLEMENT_PCT[quarters] / 100.0
    }
}

/// Months from the reference age of a person born in `birth` to the payout `at` (negative = early payout).
pub fn offset(birth: crate::time::YearMonth, at: crate::time::YearMonth) -> i32 {
    birth.plus(REFERENCE_AGE_MONTHS).until(at)
}

/// Monthly pension with which the simulation computes: the entered pension at the reference age, adjusted by
/// [`rate`], rounded to the centime.
pub fn adjusted(monthly: f64, offset: i32) -> f64 {
    (monthly * (1.0 + rate(offset)) * 100.0).round() / 100.0
}

/// #191 (upgrade of older plans): pension at the reference age from an amount already adjusted for the payout,
/// rounded to the centime; inverse of [`adjusted`].
pub fn at_reference_age(adjusted: f64, offset: i32) -> f64 {
    (adjusted / (1.0 + rate(offset)) * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::YearMonth;

    #[test]
    fn issue_191_ak2_rate_tables_match_leaflet_3_04() {
        // reduction: 6.8 % per year, month by month (spot checks of the table) and at most 13.6 %
        let early = [
            (1, 0.6),
            (2, 1.1),
            (3, 1.7),
            (6, 3.4),
            (11, 6.2),
            (12, 6.8),
            (15, 8.5),
            (18, 10.2),
            (23, 13.0),
            (24, 13.6),
        ];
        for (m, pct) in early {
            assert_eq!(EARLY_REDUCTION_PCT[m], pct, "{m} Monate");
            assert!((rate(-(m as i32)) + pct / 100.0).abs() < 1e-12);
        }
        for (m, pct) in EARLY_REDUCTION_PCT.iter().enumerate() {
            let linear = 6.8 * m as f64 / 12.0;
            assert!((pct - linear).abs() <= 0.05 + 1e-9, "{m}");
        }
        assert_eq!(rate(-30), -0.136);
        // supplement: whole years and quarter steps
        let late = [
            (12, 5.2),
            (14, 5.2),
            (15, 6.6),
            (18, 8.0),
            (21, 9.4),
            (24, 10.8),
            (27, 12.3),
            (30, 13.9),
            (33, 15.5),
            (36, 17.1),
            (39, 18.8),
            (42, 20.5),
            (45, 22.2),
            (48, 24.0),
            (51, 25.8),
            (54, 27.7),
            (57, 29.6),
            (59, 29.6),
            (60, 31.5),
            (70, 31.5),
        ];
        for (m, pct) in late {
            assert!((rate(m) - pct / 100.0).abs() < 1e-12, "{m} Monate: {}", rate(m));
        }
        assert_eq!(rate(0), 0.0);
        assert_eq!(rate(11), 0.0);
        assert_eq!(RATES_VALID_FROM, 2026);
        assert_eq!(REFERENCE_AGE_MONTHS, 780);
    }

    #[test]
    fn issue_191_ak3_ak4_ak5_adjusted_pensions() {
        assert_eq!(adjusted(2520.0, -24), 2177.28);
        assert_eq!(adjusted(2000.0, -15), 1830.0);
        assert_eq!(adjusted(2520.0, 36), 2950.92);
        assert_eq!(adjusted(2520.0, 0), 2520.0);
        assert_eq!(at_reference_age(2177.28, -24), 2520.0);
        assert_eq!(at_reference_age(2950.92, 36), 2520.0);
        let birth = YearMonth::new(1967, 3);
        assert_eq!(offset(birth, YearMonth::new(2030, 3)), -24);
        assert_eq!(offset(birth, YearMonth::new(2035, 6)), 39);
    }
}
