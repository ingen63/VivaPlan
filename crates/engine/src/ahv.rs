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

use crate::time::YearMonth;

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
pub fn offset(birth: YearMonth, at: YearMonth) -> i32 {
    birth.plus(REFERENCE_AGE_MONTHS).until(at)
}

/// Reduction (negative) or supplement in whole francs for the pension `monthly` and the rate `rate` (leaflet 3.04,
/// examples 24–26: the amounts are rounded to francs).
pub fn amount(monthly: f64, rate: f64) -> f64 {
    (monthly * rate).round()
}

/// Monthly pension of a single person (no cap) with a payout `offset` months from 65: the entered pension at 65 plus the
/// reduction or supplement in whole francs ([`amount`]).
pub fn adjusted(monthly: f64, offset: i32) -> f64 {
    monthly + amount(monthly, rate(offset))
}

/// #191 (upgrade of older plans): pension at the reference age from an amount already adjusted for the payout,
/// rounded to the centime.
pub fn at_reference_age(adjusted: f64, offset: i32) -> f64 {
    (adjusted / (1.0 + rate(offset)) * 100.0).round() / 100.0
}

/// AHV of one person for [`Calc`].
#[derive(Clone, Debug)]
pub struct Person {
    /// Entered monthly pension at the reference age (65).
    pub entered: f64,
    /// Payout (`None` = never).
    pub at: Option<YearMonth>,
    /// Month in which the person reaches the reference age.
    pub reference: YearMonth,
    /// Annual adjustment to inflation.
    pub indexed: bool,
}

impl Person {
    pub fn new(birth: YearMonth, entered: f64, at: Option<YearMonth>, indexed: bool) -> Self {
        Person {
            entered,
            at,
            reference: birth.plus(REFERENCE_AGE_MONTHS),
            indexed,
        }
    }
    /// Months of the payout after the reference age (negative = early payout).
    pub fn offset(&self) -> i32 {
        self.at.map_or(0, |a| self.reference.until(a))
    }
    pub fn rate(&self) -> f64 {
        rate(self.offset())
    }
    /// From this month the person counts for the cap of married couples: payout (also early) or reference age
    /// (also during a deferral). Never without a payout.
    pub fn entitled_from(&self) -> Option<YearMonth> {
        self.at.map(|a| a.min(self.reference))
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    entitled: bool,
    started: bool,
    /// Pension at 65 (nominal, adjusted to inflation from the entitlement).
    base: f64,
    /// Fixed reduction (negative) or supplement from the reference age or the end of the deferral (nominal).
    fixed: Option<f64>,
    /// Sum and number of the (capped) pensions during the early payout or deferral.
    acc_sum: f64,
    acc_n: f64,
    paid: f64,
}

/// #191: AHV of the household month by month, as the AHV computes it (leaflet 3.04, items 4, 11–14, 24–26):
/// - The cap of married couples acts on the pensions at 65 (not reduced, not increased) as soon as both are entitled
///   ([`Person::entitled_from`]); both pensions are cut proportionally.
/// - During an early payout the (capped) pension is reduced by the rate each month. From the reference age a fixed
///   reduction applies: the rate times the average of the (capped) pensions during the early payout.
/// - A deferral of at least one year gets a fixed supplement from the payout: the rate times the average of the
///   (capped) deferred pension. Reduction and supplement lie outside the cap.
/// - Amounts in whole francs; pension, cap, reduction and supplement follow inflation in January alike.
#[derive(Clone, Debug)]
pub struct Calc {
    people: Vec<Person>,
    cap: Option<f64>,
    /// Inflation of the cap since the first entitlement (#145: once per January, not per person).
    cap_scale: f64,
    st: Vec<State>,
}

impl Calc {
    /// `cap`: cap of married couples (`None` for one person or unmarried couples).
    pub fn new(people: Vec<Person>, cap: Option<f64>) -> Self {
        let st = vec![State::default(); people.len()];
        Calc {
            people,
            cap: cap.filter(|_| st.len() > 1),
            cap_scale: 1.0,
            st,
        }
    }

    /// January: pensions (and their reduction or supplement) of entitled indexed persons rise by `factor`; the cap
    /// rises once if at least one of them does.
    pub fn index(&mut self, factor: f64) {
        let mut any = false;
        for (st, p) in self.st.iter_mut().zip(&self.people) {
            if st.entitled && p.indexed {
                st.base *= factor;
                if let Some(f) = st.fixed.as_mut() {
                    *f *= factor;
                }
                any = true;
            }
        }
        if any {
            self.cap_scale *= factor;
        }
    }

    /// Pensions at 65 after the cap, for the entitlements `entitled` and the pensions `base`.
    fn capped(&self, entitled: &[bool], base: &[f64]) -> Vec<f64> {
        let b: Vec<f64> = base
            .iter()
            .zip(entitled)
            .map(|(b, e)| if *e { *b } else { 0.0 })
            .collect();
        let sum: f64 = b.iter().sum();
        match self.cap {
            Some(cap) if entitled.iter().all(|e| *e) && sum > cap * self.cap_scale => {
                b.iter().map(|x| x * cap * self.cap_scale / sum).collect()
            }
            _ => b,
        }
    }

    /// Month `m`: updates entitlement, payout, the cap and the reduction or supplement; returns the persons whose
    /// payout starts in `m`. `first`: first month of the simulation; the months of an early payout or deferral before
    /// it count with the pensions of `m` (#191/AK22).
    pub fn step(&mut self, m: YearMonth, first: bool) -> Vec<usize> {
        let mut started = vec![];
        for (i, (st, p)) in self.st.iter_mut().zip(&self.people).enumerate() {
            if !st.entitled && p.entitled_from().is_some_and(|e| m >= e) {
                st.entitled = true;
                st.base = p.entered;
            }
            if !st.started && p.at.is_some_and(|a| m >= a) {
                st.started = true;
                started.push(i);
            }
        }
        let entitled: Vec<bool> = self.st.iter().map(|s| s.entitled).collect();
        let base: Vec<f64> = self.st.iter().map(|s| s.base).collect();
        let b = self.capped(&entitled, &base);
        for ((st, p), b) in self.st.iter_mut().zip(&self.people).zip(b) {
            st.paid = 0.0;
            let Some(at) = p.at.filter(|_| st.entitled) else {
                continue;
            };
            let rate = p.rate();
            // phase of the early payout (payout .. reference age) or of the deferral (reference age .. payout)
            let (from, to) = if at < p.reference {
                (at, p.reference)
            } else {
                (p.reference, at)
            };
            if first && from < m {
                let before = from.until(m.min(to)).max(0) as f64;
                st.acc_sum += before * b;
                st.acc_n += before;
            }
            if m < to {
                st.acc_sum += b;
                st.acc_n += 1.0;
                if st.started {
                    st.paid = b + amount(b, rate); // early payout: reduced each month
                }
                continue;
            }
            let fixed = *st.fixed.get_or_insert_with(|| {
                let avg = if st.acc_n > 0.0 { st.acc_sum / st.acc_n } else { b };
                amount(avg, rate)
            });
            st.paid = b + fixed;
        }
        started
    }

    /// Pension of person `i` in the last month of [`Calc::step`].
    pub fn paid(&self, i: usize) -> f64 {
        self.st[i].paid
    }

    /// Household AHV in the last month of [`Calc::step`].
    pub fn total(&self) -> f64 {
        self.st.iter().map(|s| s.paid).sum()
    }

    /// Estimated household AHV in a later month `m` (planning of the current year): today's amounts, pensions not yet
    /// due from their entitlement with the entered pension, the cap and the reduction or supplement as known today.
    pub fn projected(&self, m: YearMonth) -> f64 {
        let entitled: Vec<bool> = self
            .st
            .iter()
            .zip(&self.people)
            .map(|(s, p)| s.entitled || p.entitled_from().is_some_and(|e| m >= e))
            .collect();
        let base: Vec<f64> = self
            .st
            .iter()
            .zip(&self.people)
            .map(|(s, p)| if s.entitled { s.base } else { p.entered })
            .collect();
        let b = self.capped(&entitled, &base);
        self.st
            .iter()
            .zip(&self.people)
            .zip(b)
            .map(|((s, p), b)| match p.at {
                Some(at) if m >= at => {
                    if at < p.reference && m < p.reference {
                        b + amount(b, p.rate())
                    } else {
                        b + s.fixed.unwrap_or_else(|| amount(b, p.rate()))
                    }
                }
                _ => 0.0,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // amounts in whole francs: 2'520 × 13.6 % = 342.72 → 343; 2'000 × 8.5 % = 170; 2'520 × 17.1 % = 430.92 → 431
        assert_eq!(adjusted(2520.0, -24), 2177.0);
        assert_eq!(adjusted(2000.0, -15), 1830.0);
        assert_eq!(adjusted(2520.0, 36), 2951.0);
        assert_eq!(adjusted(2520.0, 0), 2520.0);
        assert_eq!(at_reference_age(2177.28, -24), 2520.0);
        assert_eq!(at_reference_age(2950.92, 36), 2520.0);
        let birth = YearMonth::new(1967, 3);
        assert_eq!(offset(birth, YearMonth::new(2030, 3)), -24);
        assert_eq!(offset(birth, YearMonth::new(2035, 6)), 39);
    }

    fn ym(y: i32, m: u32) -> YearMonth {
        YearMonth::new(y, m)
    }

    /// Person with the pension `entered` at 65, reference age in `reference`, payout `at`.
    fn person(entered: f64, reference: YearMonth, at: Option<YearMonth>) -> Person {
        Person {
            entered,
            at,
            reference,
            indexed: true,
        }
    }

    /// Steps from `from` to `to` (inclusive); returns the pensions per person in each month.
    fn run(calc: &mut Calc, from: YearMonth, to: YearMonth) -> Vec<(YearMonth, Vec<f64>)> {
        let mut out = vec![];
        let mut m = from;
        while m <= to {
            calc.step(m, m == from);
            out.push((m, (0..calc.st.len()).map(|i| calc.paid(i)).collect()));
            m = m.plus(1);
        }
        out
    }

    fn at(out: &[(YearMonth, Vec<f64>)], m: YearMonth) -> Vec<f64> {
        out.iter().find(|(x, _)| *x == m).unwrap().1.clone()
    }

    fn couple(a: Person, b: Person) -> Calc {
        Calc::new(vec![a, b], Some(3780.0))
    }

    const CAP: f64 = 3780.0;

    /// #191/AK11: Both at 65: capped from the month the second reaches 65, 1'890 each.
    #[test]
    fn issue_191_ak11_both_at_65_capped_from_the_second_reference_age() {
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2032, 1), Some(ym(2032, 1))),
        );
        let out = run(&mut c, ym(2029, 1), ym(2033, 12));
        assert_eq!(at(&out, ym(2029, 12)), vec![0.0, 0.0]);
        assert_eq!(at(&out, ym(2031, 12)), vec![2520.0, 0.0]);
        assert_eq!(at(&out, ym(2032, 1)), vec![1890.0, 1890.0]);
        assert_eq!(at(&out, ym(2033, 12)), vec![1890.0, 1890.0]);
    }

    /// #191/AK12: Sum below the cap: no cap; A 2'000 at 63 → 2'000 − 272 = 1'728, B 1'500.
    #[test]
    fn issue_191_ak12_sum_below_the_cap() {
        let mut c = couple(
            person(2000.0, ym(2032, 1), Some(ym(2030, 1))),
            person(1500.0, ym(2031, 1), Some(ym(2031, 1))),
        );
        let out = run(&mut c, ym(2030, 1), ym(2033, 12));
        assert_eq!(at(&out, ym(2030, 6)), vec![1728.0, 0.0]);
        assert_eq!(at(&out, ym(2031, 6)), vec![1728.0, 1500.0]);
        assert_eq!(at(&out, ym(2033, 6)), vec![1728.0, 1500.0]);
    }

    /// #191/AK13: Unmarried (no cap): A at 63 → 2'177, B at 65 → 2'520.
    #[test]
    fn issue_191_ak13_unmarried_no_cap() {
        let mut c = Calc::new(
            vec![
                person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
                person(2520.0, ym(2031, 1), Some(ym(2031, 1))),
            ],
            None,
        );
        let out = run(&mut c, ym(2030, 1), ym(2033, 12));
        assert_eq!(at(&out, ym(2033, 6)), vec![2177.0, 2520.0]);
        // a cap for one person is ignored
        let mut one = Calc::new(vec![person(2520.0, ym(2030, 1), Some(ym(2030, 1)))], Some(1000.0));
        assert_eq!(at(&run(&mut one, ym(2030, 1), ym(2030, 2)), ym(2030, 2)), vec![2520.0]);
    }

    /// #191/AK14: A early, B without entitlement during the whole early payout: reduction from the full pension
    /// (343); once capped A 1'890 − 343 = 1'547, B 1'890.
    #[test]
    fn issue_191_ak14_early_payout_before_the_spouse_is_entitled() {
        let mut c = couple(
            person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2035, 1), Some(ym(2035, 1))),
        );
        let out = run(&mut c, ym(2030, 1), ym(2036, 12));
        assert_eq!(at(&out, ym(2030, 6)), vec![2177.0, 0.0]);
        assert_eq!(at(&out, ym(2033, 6)), vec![2177.0, 0.0]);
        assert_eq!(at(&out, ym(2035, 1)), vec![1547.0, 1890.0]);
        assert!(at(&out, ym(2035, 1)).iter().sum::<f64>() < CAP);
    }

    /// #191/AK15: A 24 months early, B reaches 65 after 12 months: first 2'177, then 1'890 − 257 = 1'633; from A's 65
    /// the reduction from the average 13.6 % × 2'205 = 299.88 → 300, A 1'590.
    #[test]
    fn issue_191_ak15_spouse_reaches_65_during_the_early_payout() {
        let mut c = couple(
            person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2031, 1), Some(ym(2031, 1))),
        );
        let out = run(&mut c, ym(2030, 1), ym(2033, 12));
        assert_eq!(at(&out, ym(2030, 12)), vec![2177.0, 0.0]);
        assert_eq!(at(&out, ym(2031, 1)), vec![1633.0, 1890.0]);
        assert_eq!(at(&out, ym(2031, 12)), vec![1633.0, 1890.0]);
        assert_eq!(at(&out, ym(2032, 1)), vec![1590.0, 1890.0]);
        assert_eq!(at(&out, ym(2033, 12)), vec![1590.0, 1890.0]);
    }

    /// #191/AK16: Both early at 63 in the same month: capped at once, 1'890 − 257 = 1'633 each, also after 65.
    #[test]
    fn issue_191_ak16_both_early_in_the_same_month() {
        let mut c = couple(
            person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
        );
        let out = run(&mut c, ym(2030, 1), ym(2033, 12));
        assert_eq!(at(&out, ym(2030, 1)), vec![1633.0, 1633.0]);
        assert_eq!(at(&out, ym(2033, 12)), vec![1633.0, 1633.0]);
    }

    /// #191/AK17: A defers 3 years, B already draws: B capped at 1'890 from A's 65 although A draws nothing; from 68
    /// A 1'890 + 323 = 2'213. The supplement lies outside the cap (sum 4'103 > 3'780).
    #[test]
    fn issue_191_ak17_deferral_while_the_spouse_draws() {
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2033, 1))),
            person(2520.0, ym(2028, 1), Some(ym(2028, 1))),
        );
        let out = run(&mut c, ym(2028, 1), ym(2034, 12));
        assert_eq!(at(&out, ym(2029, 12)), vec![0.0, 2520.0]);
        assert_eq!(at(&out, ym(2030, 1)), vec![0.0, 1890.0]);
        assert_eq!(at(&out, ym(2032, 12)), vec![0.0, 1890.0]);
        assert_eq!(at(&out, ym(2033, 1)), vec![2213.0, 1890.0]);
        assert_eq!(at(&out, ym(2034, 12)).iter().sum::<f64>(), 4103.0);
    }

    /// #191/AK18: A defers 1 year, B without entitlement meanwhile: supplement from the full pension 131, A 2'651;
    /// once capped A 1'890 + 131 = 2'021, B 1'890.
    #[test]
    fn issue_191_ak18_deferral_before_the_spouse_is_entitled() {
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2031, 1))),
            person(2520.0, ym(2033, 1), Some(ym(2033, 1))),
        );
        let out = run(&mut c, ym(2030, 1), ym(2034, 12));
        assert_eq!(at(&out, ym(2030, 6)), vec![0.0, 0.0]);
        assert_eq!(at(&out, ym(2031, 1)), vec![2651.0, 0.0]);
        assert_eq!(at(&out, ym(2033, 1)), vec![2021.0, 1890.0]);
    }

    /// #191/AK19: A payout less than one year after 65: no supplement, no pension before.
    #[test]
    fn issue_191_ak19_deferral_under_one_year_gets_no_supplement() {
        let mut c = Calc::new(vec![person(2520.0, ym(2030, 1), Some(ym(2030, 7)))], None);
        let out = run(&mut c, ym(2030, 1), ym(2031, 12));
        assert_eq!(at(&out, ym(2030, 6)), vec![0.0]);
        assert_eq!(at(&out, ym(2030, 7)), vec![2520.0]);
    }

    /// #191/AK21: Inflation 10 %: pension, cap, reduction and supplement rise alike in January. Case AK14 from 2035:
    /// (1'890 − 343) × 1.1 = 1'701.70 and 1'890 × 1.1 = 2'079; case AK17 after 68: (1'890 + 323) × 1.1 = 2'434.30.
    #[test]
    fn issue_191_ak21_inflation_raises_pension_cap_reduction_and_supplement_alike() {
        let mut c = couple(
            person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2035, 1), Some(ym(2035, 1))),
        );
        run(&mut c, ym(2030, 1), ym(2035, 12));
        c.index(1.1);
        c.step(ym(2036, 1), false);
        assert!((c.paid(0) - 1547.0 * 1.1).abs() < 1e-9, "{}", c.paid(0));
        assert!((c.paid(1) - 1890.0 * 1.1).abs() < 1e-9, "{}", c.paid(1));
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2033, 1))),
            person(2520.0, ym(2028, 1), Some(ym(2028, 1))),
        );
        run(&mut c, ym(2028, 1), ym(2033, 12));
        c.index(1.1);
        c.step(ym(2034, 1), false);
        assert!((c.paid(0) - 2213.0 * 1.1).abs() < 1e-9, "{}", c.paid(0));
        // a pension that is not indexed stays; the cap rises with the other
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2030, 1))),
            Person {
                indexed: false,
                ..person(2520.0, ym(2030, 1), Some(ym(2030, 1)))
            },
        );
        c.step(ym(2030, 1), true);
        c.index(1.1);
        c.step(ym(2031, 1), false);
        assert!((c.total() - CAP * 1.1).abs() < 1e-9, "{}", c.total());
    }

    /// #191/AK22: The simulation starts during or after the early payout: the months before count with the amounts at
    /// the start. Case AK15 from 06.2031 (capped 1'890): reduction 13.6 % × 1'890 = 257, A 1'633 from 01.2032.
    #[test]
    fn issue_191_ak22_months_before_the_start_count_with_the_amounts_at_the_start() {
        let make = || {
            couple(
                person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
                person(2520.0, ym(2031, 1), Some(ym(2031, 1))),
            )
        };
        let out = run(&mut make(), ym(2031, 6), ym(2032, 6));
        assert_eq!(at(&out, ym(2031, 6)), vec![1633.0, 1890.0]);
        assert_eq!(at(&out, ym(2032, 1)), vec![1633.0, 1890.0]);
        // start after the early payout: the same
        let out = run(&mut make(), ym(2033, 1), ym(2033, 2));
        assert_eq!(at(&out, ym(2033, 1)), vec![1633.0, 1890.0]);
        // deferral before the start (AK18 from 2032): supplement 5.2 % × 2'520 = 131
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2031, 1))),
            person(2520.0, ym(2033, 1), Some(ym(2033, 1))),
        );
        let out = run(&mut c, ym(2032, 1), ym(2033, 1));
        assert_eq!(at(&out, ym(2032, 1)), vec![2651.0, 0.0]);
        assert_eq!(at(&out, ym(2033, 1)), vec![2021.0, 1890.0]);
    }

    /// #191/AK23: One person never draws AHV: the other is never capped.
    #[test]
    fn issue_191_ak23_spouse_without_ahv_never_caps() {
        let mut c = couple(
            person(2520.0, ym(2030, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2031, 1), None),
        );
        let out = run(&mut c, ym(2030, 1), ym(2040, 12));
        assert!(out.iter().all(|(_, v)| v == &vec![2520.0, 0.0]));
        assert_eq!(c.projected(ym(2045, 1)), 2520.0);
    }

    /// #191: The projection for the planning of the year: pensions not yet due from their payout, with the cap and
    /// the reduction as known.
    #[test]
    fn issue_191_projection_of_later_months() {
        let mut c = couple(
            person(2520.0, ym(2032, 1), Some(ym(2030, 1))),
            person(2520.0, ym(2035, 1), Some(ym(2035, 1))),
        );
        assert_eq!(c.projected(ym(2029, 12)), 0.0);
        assert_eq!(c.projected(ym(2030, 1)), 2177.0);
        run(&mut c, ym(2030, 1), ym(2033, 1));
        assert_eq!(c.projected(ym(2034, 6)), 2177.0);
        assert_eq!(c.projected(ym(2035, 1)), 1547.0 + 1890.0);
    }
}
