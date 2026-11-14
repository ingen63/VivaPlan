//! Eigenschaften der Parser und des CSV-Formats für beliebige Eingaben.
//! Fallzahl über `PROPTEST_CASES` steuerbar.

use engine::market::DataSet;
use proptest::prelude::*;

proptest! {
    /// Beliebiger Text darf den CSV-Parser nie zum Absturz bringen.
    #[test]
    fn proptest_parse_csv_never_panics(text in "\\PC{0,400}") {
        let _ = DataSet::parse_csv("x", &text);
    }

    /// Beliebiger Text darf den SNB-Parser nie zum Absturz bringen.
    #[test]
    fn proptest_parse_snb_never_panics(text in "\\PC{0,400}") {
        let _ = engine::import::parse_snb(&text);
    }

    /// Round-Trip: Was `to_csv` schreibt, liest `parse_csv` wieder ein (Jahreswerte).
    #[test]
    fn proptest_csv_round_trip(
        first in 1900i32..2100,
        rows in prop::collection::vec((-0.9f64..2.0, -0.9f64..2.0, -0.5f64..1.0), 1..30),
    ) {
        let mut text = String::from("Years,Stocks,Bonds,Inflation\n");
        for (i, (s, b, f)) in rows.iter().enumerate() {
            text.push_str(&format!("{},{s:.6},{b:.6},{f:.6}\n", first + i as i32));
        }
        let ds = DataSet::parse_csv("t", &text).unwrap();
        let again = DataSet::parse_csv("t", &ds.to_csv()).unwrap();
        prop_assert_eq!(ds.len(), rows.len());
        prop_assert_eq!(again.len(), ds.len());
        for i in 0..ds.len() {
            prop_assert!((ds.stocks[i] - again.stocks[i]).abs() < 1e-9);
            prop_assert!((ds.bonds[i] - again.bonds[i]).abs() < 1e-9);
            prop_assert!((ds.inflation[i] - again.inflation[i]).abs() < 1e-9);
        }
    }

    /// Der Jahres- in Monatssatz-Umrechner liefert nie eine ungültige Zahl und rechnet sich zurück.
    #[test]
    fn proptest_monthly_rate(y in -5.0f64..5.0) {
        let m = engine::market::monthly(y);
        prop_assert!(m.is_finite() && m >= -1.0);
        if y > -1.0 {
            prop_assert!(((1.0 + m).powi(12) - (1.0 + y)).abs() < 1e-9);
        }
    }
}
