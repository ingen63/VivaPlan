//! Anwendungsweite Einstellungen, die der Rechenkern braucht (FEA-24).
//!
//! Die Oberfläche speichert sie lokal (nicht in der Planungsdatei) und gibt sie bei jedem Aufruf als `limits` in der
//! Planung mit. Fehlen sie (Kommandozeile, Tests), gelten die Standardwerte dieser Datei. Alle Stellen, die eine Grenze
//! brauchen, lesen sie aus [`Limits`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Standard des Meilensteins «AHV-Bezug» und «Bezug Pensionskasse» in Jahren (FEA-38).
pub const DEFAULT_AGE: f64 = 65.0;

/// Grenzen für die Meilensteine «AHV-Bezug» und «Bezug Pensionskasse» und Maximalbetrag der Säule 3a.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Limits {
    /// AHV-Bezug: untere und obere Grenze in Jahren.
    pub ahv_min: f64,
    pub ahv_max: f64,
    /// Bezug Pensionskasse: untere und obere Grenze in Jahren.
    pub pk_min: f64,
    pub pk_max: f64,
    /// Maximalbetrag der Einzahlung in die Säule 3a pro Jahr.
    pub pillar3a_max: f64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            ahv_min: 63.0,
            ahv_max: 70.0,
            pk_min: 58.0,
            pk_max: 65.0,
            pillar3a_max: 7258.0,
        }
    }
}

impl Limits {
    /// Fehler in den Einstellungen selbst (Oberfläche und Prüfung). Leer = gültig.
    pub fn problems(&self) -> Vec<String> {
        let mut out = vec![];
        let whole = |x: f64| x.fract() == 0.0 && (0.0..=130.0).contains(&x);
        for (what, lo, hi) in [
            ("AHV-Bezug", self.ahv_min, self.ahv_max),
            ("Bezug Pensionskasse", self.pk_min, self.pk_max),
        ] {
            if !whole(lo) || !whole(hi) {
                out.push(format!(
                    "{what}: Die Grenzen müssen ganze Jahre zwischen 0 und 130 sein."
                ));
            } else if lo > hi {
                out.push(format!(
                    "{what}: Die untere Grenze ({lo}) liegt über der oberen Grenze ({hi})."
                ));
            }
        }
        if !self.pillar3a_max.is_finite() || self.pillar3a_max < 0.0 {
            out.push("Säule 3a: Der Maximalbetrag muss 0 oder mehr sein.".into());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FEA-24/AK1: Standardwerte der Einstellungen.
    #[test]
    fn defaults() {
        let l = Limits::default();
        assert_eq!(
            (l.ahv_min, l.ahv_max, l.pk_min, l.pk_max, l.pillar3a_max),
            (63.0, 70.0, 58.0, 65.0, 7258.0)
        );
        assert!(l.problems().is_empty());
        // Fehlende Felder: Standardwert.
        let p: Limits = serde_json::from_str(r#"{"pkMax": 67}"#).unwrap();
        assert_eq!((p.pk_max, p.ahv_min), (67.0, 63.0));
    }

    // FEA-24/AK4: Prüfung der Eingabe.
    #[test]
    fn problems() {
        let bad = |f: fn(&mut Limits)| {
            let mut l = Limits::default();
            f(&mut l);
            l.problems()
        };
        assert!(bad(|l| l.ahv_min = 71.0)[0].contains("untere Grenze (71) liegt über der oberen Grenze (70)"));
        assert!(bad(|l| l.ahv_min = 70.0).is_empty(), "untere = obere Grenze ist gültig");
        assert!(bad(|l| l.pk_max = 64.5)[0].contains("ganze Jahre"));
        assert!(bad(|l| l.pk_min = -1.0)[0].contains("ganze Jahre"));
        assert!(bad(|l| l.pillar3a_max = -1.0)[0].contains("0 oder mehr"));
        assert!(bad(|l| l.pillar3a_max = f64::NAN)[0].contains("0 oder mehr"));
        assert!(bad(|l| l.pillar3a_max = 0.0).is_empty());
    }
}
