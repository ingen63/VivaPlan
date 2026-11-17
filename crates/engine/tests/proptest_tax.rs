//! #134: Steuern sind nie negativ, und ein höherer Betrag ergibt nie eine tiefere Steuer. Fallzahl über `PROPTEST_CASES`
//! (nächtlich 10'000, `make proptest`).

use engine::model::{GainModel, PropertyGainTax};
use engine::tax::{falling_row, property_gain_tax_held, rate_at, tax};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/proptest_tax.txt"
);

/// Satztabelle wie in der Eingabeprüfung erlaubt: Beträge ≥ 0, Sätze 0–100 %, beliebige Reihenfolge, auch doppelte Beträge.
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
            },
        )
}

/// Steuer ohne Rundung und ohne Steuerfuss.
fn raw(t: &[(f64, f64)], x: f64) -> f64 {
    x * rate_at(t, x)
}

/// Fällt die Steuer irgendwo? Unabhängig von `falling_row` geprüft: an jeder Stufe gleich danach (Sprung) und je
/// Abschnitt zwischen dem Scheitel der Parabel `x × (a + s x)` (dort ist die Steuer am höchsten) und dem Abschnittsende.
fn really_falls(t: &[(f64, f64)]) -> bool {
    let mut xs: Vec<f64> = t.iter().map(|r| r.0).collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let jump = xs
        .iter()
        .any(|&x| x > 0.0 && raw(t, x * (1.0 + 1e-12)) < raw(t, x) - 1e-9 * x.max(1.0));
    let inner = xs.windows(2).any(|w| {
        let (x0, x1) = (w[0], w[1]);
        // knapp nach der Stufe: dort gilt schon der Satz der letzten Zeile mit diesem Betrag
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

    // #134/AK1: Für Tabellen ohne fallende Steuer ist die Steuer ≥ 0 und steigt mit dem Betrag (Rundung auf Rappen:
    // Toleranz 0.01). Zugleich prüft der Test die Erkennung `falling_row`: Meldet sie nichts, darf die Steuer nicht fallen.
    #[test]
    fn proptest_tax_never_falls_without_warning(
        t in table(), a in 0.0..3_000_000.0f64, d in 0.0..500_000.0f64, m in 0.0..=10.0f64
    ) {
        prop_assume!(falling_row(&t).is_none());
        let (lo, hi) = (tax(&t, a, m), tax(&t, a + d, m));
        prop_assert!(lo >= 0.0, "Steuer {lo} auf {a}");
        prop_assert!(hi + 0.01 >= lo, "Steuer fällt von {lo} ({a}) auf {hi} ({})", a + d);
    }

    // #134/AK4: Die Erkennung stimmt mit dem tatsächlichen Verlauf überein, in beide Richtungen.
    #[test]
    fn proptest_falling_row_matches_the_tax_curve(t in table()) {
        prop_assert_eq!(falling_row(&t).is_some(), really_falls(&t), "{:?}", t);
    }

    // #134/AK2: Grundstückgewinnsteuer ist für jeden Gewinn, jede Besitzdauer (auch unbekannt) und beide Modelle ≥ 0.
    #[test]
    fn proptest_property_gain_tax_never_negative(
        def in gain_def(), gain in -1_000_000.0..5_000_000.0f64, years in proptest::option::of(0..45i32), m in 0.0..=10.0f64
    ) {
        let t = property_gain_tax_held(Some(&def), gain, years, m);
        prop_assert!(t >= 0.0 && t.is_finite(), "{t}");
    }
}

// #134/AK4: Beispiele für die Erkennung. Durchschnittssätze, die mit dem Betrag steigen, sind in Ordnung; ein Satz, der
// schneller sinkt, als der Betrag steigt, lässt die Steuer fallen.
#[test]
fn issue_134_ak4_falling_tables_are_found() {
    assert_eq!(falling_row(&[]), None);
    assert_eq!(falling_row(&[(0.0, 0.0), (200_000.0, 0.15)]), None);
    // [[0, 8 %], [1 Mio, 2 %]] (erstes Beispiel des Issues): Steuer x × (0.08 − 0.06 x / 1 Mio) steigt bis 666'667 und fällt
    // danach (Ableitung bei 1 Mio: 0.02 − 0.06 = −0.04)
    assert_eq!(falling_row(&[(0.0, 0.08), (1_000_000.0, 0.02)]), Some(1));
    // Beispiel aus dem Kommentar zu #134: [[0, 10 %], [300'000, 1 %]] → 200'000: 8'000, 300'000: 3'000
    assert_eq!(
        falling_row(&[(300_000.0, 0.01), (0.0, 0.10)]),
        Some(0),
        "Zeile in erfasster Reihenfolge"
    );
    // leicht fallender Satz, Steuer steigt noch: [[100'000, 10 %], [200'000, 9 %]] → Ableitung bei 200'000: 0.09 − 0.02 = 0.07
    assert_eq!(falling_row(&[(100_000.0, 0.10), (200_000.0, 0.09)]), None);
    // Sprung: gleicher Betrag, tieferer Satz
    assert_eq!(falling_row(&[(0.0, 0.0), (50_000.0, 0.10), (50_000.0, 0.05)]), Some(2));
    assert_eq!(falling_row(&[(50_000.0, 0.05), (50_000.0, 0.10)]), None);
    // Gruppe gleicher Beträge: zählt erster gegen letzter Satz (rate_at), nicht die Zeilen dazwischen
    assert_eq!(
        falling_row(&[(100_000.0, 0.7), (100_000.0, 0.0), (100_000.0, 0.9)]),
        None
    );
    assert_eq!(
        falling_row(&[(100_000.0, 0.7), (100_000.0, 0.9), (100_000.0, 0.2)]),
        Some(2)
    );
}
