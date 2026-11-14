//! Steuerberechnung mit einfachen Satztabellen.

use crate::model::{GainModel, PropertyGainTax};

/// Satz für einen Betrag: linear zwischen den Stufen interpoliert, ausserhalb konstant fortgesetzt.
pub fn rate_at(table: &[(f64, f64)], x: f64) -> f64 {
    if table.is_empty() {
        return 0.0;
    }
    let mut t = table.to_vec();
    t.sort_by(|a, b| a.0.total_cmp(&b.0));
    if x <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        let ((x0, r0), (x1, r1)) = (w[0], w[1]);
        if x <= x1 {
            if x1 == x0 {
                return r1;
            }
            return r0 + (r1 - r0) * (x - x0) / (x1 - x0);
        }
    }
    t[t.len() - 1].1
}

/// Steuer = Betrag × Satz × Steuerfuss, auf Rappen gerundet.
pub fn tax(table: &[(f64, f64)], x: f64, multiplier: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    round2(x * rate_at(table, x) * multiplier)
}

/// Grundstückgewinnsteuer: Gewinn wird stufenweise besteuert (Breite und Satz je Stufe), dann reduziert.
pub fn property_gain_tax(def: Option<&PropertyGainTax>, gain: f64) -> f64 {
    property_gain_tax_held(def, gain, None, 1.0)
}

/// Wie [`property_gain_tax`], mit Zu- oder Abschlag nach der Besitzdauer in vollen Jahren
/// (`None`: unbekannt, dann gibt es keinen Zu- oder Abschlag).
/// `multiplier` ist der Steuerfuss; er wirkt nur, wenn die Regeln `use_multiplier` setzen.
pub fn property_gain_tax_held(def: Option<&PropertyGainTax>, gain: f64, years: Option<i32>, multiplier: f64) -> f64 {
    let Some(def) = def else { return 0.0 };
    if gain <= 0.0 || gain < def.exempt_below {
        return 0.0;
    }
    let factor = if def.use_multiplier { multiplier } else { 1.0 };
    if def.model == GainModel::Aargau {
        // Direkter Satz nach Besitzdauer; ohne Kaufdatum der höchste Satz.
        let rate = match years {
            Some(y) => def
                .duration_rates
                .iter()
                .filter(|&&(from, _)| from <= f64::from(y))
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .map(|&(_, r)| r),
            None => def.duration_rates.iter().map(|&(_, r)| r).max_by(f64::total_cmp),
        }
        .unwrap_or(0.0);
        return round2(gain * rate * factor * (1.0 - def.reduction));
    }
    // Stufen in der erfassten Reihenfolge; über der letzten Stufe gilt deren Satz.
    let mut rest = gain;
    let mut tax = 0.0;
    for &(width, rate) in &def.brackets {
        let part = rest.min(width.max(0.0));
        tax += rate * part;
        rest -= part;
        if rest <= 0.0 {
            break;
        }
    }
    if let Some(&(_, last)) = def.brackets.last() {
        tax += last * rest.max(0.0);
    }
    let adjust = years.map_or(0.0, |y| {
        def.duration
            .iter()
            .filter(|&&(from, _)| from <= f64::from(y))
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(0.0, |&(_, change)| change)
    });
    round2(tax * (1.0 + adjust) * factor * (1.0 - def.reduction))
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round_ties_even() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation() {
        let t = [(0.0, 0.0), (50_000.0, 0.04), (100_000.0, 0.06)];
        assert_eq!(rate_at(&t, -5.0), 0.0);
        assert!((rate_at(&t, 75_000.0) - 0.05).abs() < 1e-12);
        assert_eq!(rate_at(&t, 1e9), 0.06);
        assert!((tax(&t, 75_000.0, 2.0) - 7_500.0).abs() < 1e-9);
    }

    #[test]
    fn gain_tax_brackets() {
        let def = PropertyGainTax {
            brackets: vec![(4000.0, 0.1), (6000.0, 0.15)],
            reduction: 0.5,
            ..Default::default()
        };
        // 4000 × 10 % + 1000 × 15 % = 550, halbiert = 275
        assert!((property_gain_tax(Some(&def), 5000.0) - 275.0).abs() < 1e-9);
    }

    #[test]
    fn aargau_rate_with_multiplier_and_reduction() {
        let def = PropertyGainTax {
            model: GainModel::Aargau,
            duration_rates: vec![(0.0, 0.3), (5.0, 0.2)],
            use_multiplier: true,
            reduction: 0.1,
            ..Default::default()
        };
        // 10 000 × 20 % (ab 5 Jahren) × Steuerfuss 1.5 × (1 − 10 %) = 2700
        assert!((property_gain_tax_held(Some(&def), 10_000.0, Some(7), 1.5) - 2700.0).abs() < 1e-9);
        // 10 000 × 30 % × 1.5 × 0.9 = 4050
        assert!((property_gain_tax_held(Some(&def), 10_000.0, Some(2), 1.5) - 4050.0).abs() < 1e-9);
    }
}
