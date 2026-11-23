//! #134: Taxes are never negative, and a higher amount never gives a lower tax. Number of cases via `PROPTEST_CASES`
//! (nightly 10'000, `make proptest`).

use engine::model::{GainModel, PropertyGainTax};
use engine::tax::{falling_row, property_gain_tax_held, rate_at, tax};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/proptest_tax.txt"
);

/// Rate table as allowed by the input validation: amounts ≥ 0, rates 0–100 %, any order, also duplicate amounts.
fn table() -> impl Strategy<Value = Vec<(f64, f64)>> {
    prop::collection::vec(
        (
            prop_oneof![Just(0.0), 0.0..2_000_000.0f64, Just(100_000.0)],
            0.0..=1.0f64,
        ),
        0..6,
    )
}

fn gain_def() -> impl Strategy<Value = PropertyGainTax> {
    (
        any::<bool>(),
        prop::collection::vec((0.0..40.0f64, 0.0..=1.0f64), 0..5),
        any::<bool>(),
        prop::collection::vec((-10_000.0..100_000.0f64, -0.5..=1.0f64), 0..6),
        -0.5..1.5f64,
        prop::collection::vec((0.0..30.0f64, -0.99..1.0f64), 0..5),
        0.0..10_000.0f64,
    )
        .prop_map(
            |(aargau, duration_rates, use_multiplier, brackets, reduction, duration, exempt_below)| PropertyGainTax {
                model: if aargau {
                    GainModel::Aargau
                } else {
                    GainModel::default()
                },
                duration_rates,
                use_multiplier,
                brackets,
                reduction,
                duration,
                exempt_below,
                ..Default::default()
            },
        )
}

/// Tax without rounding and without tax multiplier.
fn raw(t: &[(f64, f64)], x: f64) -> f64 {
    x * rate_at(t, x)
}

/// Does the tax fall anywhere? Checked independently of `falling_row`: at every bracket right after it (jump) and per
/// section between the vertex of the parabola `x × (a + s x)` (where the tax is highest) and the end of the section.
fn really_falls(t: &[(f64, f64)]) -> bool {
    let mut xs: Vec<f64> = t.iter().map(|r| r.0).collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let jump = xs
        .iter()
        .any(|&x| x > 0.0 && raw(t, x * (1.0 + 1e-12)) < raw(t, x) - 1e-9 * x.max(1.0));
    let inner = xs.windows(2).any(|w| {
        let (x0, x1) = (w[0], w[1]);
        // just after the bracket: there the rate of the last row with this amount already applies
        let after = x0 + (x1 - x0) * 1e-9;
        let (r0, r1) = (rate_at(t, after), rate_at(t, x1));
        let s = (r1 - r0) / (x1 - after);
        let a = r0 - s * after;
        let top = if s < 0.0 { (-a / (2.0 * s)).clamp(after, x1) } else { x1 };
        raw(t, x1) < raw(t, top) - 1e-12 * x1.max(1.0)
    });
    jump || inner
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        max_global_rejects: 1_000_000,
        ..ProptestConfig::default()
    })]

    // #134/AK1: For tables without a falling tax the tax is ≥ 0 and rises with the amount (rounding to centimes:
    // tolerance 0.01). At the same time the test checks the detection `falling_row`: if it reports nothing, the tax must not fall.
    #[test]
    fn proptest_tax_never_falls_without_warning(
        t in table(), a in 0.0..3_000_000.0f64, d in 0.0..500_000.0f64, m in 0.0..=10.0f64
    ) {
        prop_assume!(falling_row(&t).is_none());
        let (lo, hi) = (tax(&t, a, m), tax(&t, a + d, m));
        prop_assert!(lo >= 0.0, "Steuer {lo} auf {a}");
        prop_assert!(hi + 0.01 >= lo, "Steuer fällt von {lo} ({a}) auf {hi} ({})", a + d);
    }

    // #134/AK4: The detection matches the actual course, in both directions.
    #[test]
    fn proptest_falling_row_matches_the_tax_curve(t in table()) {
        prop_assert_eq!(falling_row(&t).is_some(), really_falls(&t), "{:?}", t);
    }

    // #134/AK2: Property gain tax is ≥ 0 for every gain, every holding period (also unknown) and both models.
    #[test]
    fn proptest_property_gain_tax_never_negative(
        def in gain_def(), gain in -1_000_000.0..5_000_000.0f64, years in proptest::option::of(0..45i32), m in 0.0..=10.0f64
    ) {
        let t = property_gain_tax_held(Some(&def), gain, years, m);
        prop_assert!(t >= 0.0 && t.is_finite(), "{t}");
    }
}

// #134/AK4: Examples for the detection. Average rates that rise with the amount are fine; a rate that
// falls faster than the amount rises makes the tax fall.
#[test]
fn issue_134_ak4_falling_tables_are_found() {
    assert_eq!(falling_row(&[]), None);
    assert_eq!(falling_row(&[(0.0, 0.0), (200_000.0, 0.15)]), None);
    // [[0, 8 %], [1 million, 2 %]] (first example of the issue): tax x × (0.08 − 0.06 x / 1 million) rises up to 666'667 and falls
    // afterwards (derivative at 1 million: 0.02 − 0.06 = −0.04)
    assert_eq!(falling_row(&[(0.0, 0.08), (1_000_000.0, 0.02)]), Some(1));
    // Example from the comment on #134: [[0, 10 %], [300'000, 1 %]] → 200'000: 8'000, 300'000: 3'000
    assert_eq!(
        falling_row(&[(300_000.0, 0.01), (0.0, 0.10)]),
        Some(0),
        "Zeile in erfasster Reihenfolge"
    );
    // slightly falling rate, tax still rises: [[100'000, 10 %], [200'000, 9 %]] → derivative at 200'000: 0.09 − 0.02 = 0.07
    assert_eq!(falling_row(&[(100_000.0, 0.10), (200_000.0, 0.09)]), None);
    // Jump: same amount, lower rate
    assert_eq!(falling_row(&[(0.0, 0.0), (50_000.0, 0.10), (50_000.0, 0.05)]), Some(2));
    assert_eq!(falling_row(&[(50_000.0, 0.05), (50_000.0, 0.10)]), None);
    // Group of equal amounts: first against last rate counts (rate_at), not the rows in between
    assert_eq!(
        falling_row(&[(100_000.0, 0.7), (100_000.0, 0.0), (100_000.0, 0.9)]),
        None
    );
    assert_eq!(
        falling_row(&[(100_000.0, 0.7), (100_000.0, 0.9), (100_000.0, 0.2)]),
        Some(2)
    );
}

/// #162/AK1: Limits of the detection not covered by the examples of #134 (mutants in `falling_row`).
#[test]
fn issue_162_ak1_falling_row_edges() {
    // Jump at amount 0: the tax is 0 there; a lower rate afterwards does not make it fall.
    assert_eq!(falling_row(&[(0.0, 0.10), (0.0, 0.05), (100_000.0, 0.05)]), None);
    // The same amount with the same rate is not a jump.
    assert_eq!(falling_row(&[(100_000.0, 0.1), (100_000.0, 0.1)]), None);
    // Rate 0 everywhere: derivative 0 at the end of the section is no fall.
    assert_eq!(falling_row(&[(0.0, 0.0), (100_000.0, 0.0)]), None);
    // The tax falls only at the start of the section (negative rate, rising derivative): the check at the start
    // finds it, the one at the end does not. The input validation requires rates ≥ 0; `falling_row` applies to every table.
    assert_eq!(falling_row(&[(1.0, -0.5), (11.0, 0.5)]), Some(1));
}

/// #223/AK2: The only entry ever saved in `proptest-regressions/proptest_tax.txt` (shrunk by
/// `proptest_falling_row_matches_the_tax_curve` during #134) was no bug in the engine: `falling_row` was right, the
/// oracle `really_falls` was not. It took the start of a section from the bracket itself, where `rate_at` still applies
/// the rate of the first of several rows with the same amount (0 %), instead of the rate just after it (79 %). From
/// 100'000 the rate falls from 79 % to 0 % at 109'822, so the tax falls in that section. The entry was removed; this
/// test keeps the case readable and fails with the former oracle.
#[test]
fn issue_223_ak2_oracle_sees_a_fall_after_a_group_of_equal_amounts() {
    let t = [
        (100_000.0, 0.0),
        (100_000.0, 0.790_172_171_361_065_8),
        (109_821.611_465_208_95, 0.0),
    ];
    // at the bracket the first row applies, just after it the last one
    assert_eq!(rate_at(&t, 100_000.0), 0.0);
    assert!((rate_at(&t, 100_000.001) - 0.790_172).abs() < 1e-5);
    // tax ≈ 79'017 just after the bracket, 0 at the end of the section
    assert!(raw(&t, 100_000.001) > 79_000.0);
    assert_eq!(raw(&t, 109_821.611_465_208_95), 0.0);
    assert_eq!(falling_row(&t), Some(2));
    assert!(really_falls(&t));
}
