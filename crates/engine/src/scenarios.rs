//! Szenario-Variationen: einzelne Parameter der Planung systematisch variieren und jede Kombination
//! mit allen Renditepfaden rechnen.

use serde::Serialize;
use serde_json::{json, Value};

use crate::time::YearMonth;
use crate::{Error, Result};

/// Technische Obergrenze der Kombinationen pro Simulation (Speicherschutz). Darunter gibt es keine feste Grenze;
/// die Oberfläche warnt vor langen Rechenzeiten.
pub const MAX_COMBINATIONS: usize = 1_000_000;

/// Zahl mit Hochkomma als Tausendertrennzeichen (1'000'000).
fn grouped(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('\'');
        }
        out.push(c);
    }
    out
}

/// Ein variierter Parameter mit seinen Werten (Zahl oder Monat `JJJJ-MM`).
#[derive(Clone, Debug, PartialEq)]
pub struct Sweep {
    pub param: String,
    pub values: Vec<Value>,
}

const PARAMS: [&str; 6] = [
    "wealth.stockShare",
    "withdrawal.rate",
    "pension.pk.lumpsum",
    "returns.startDate",
    "milestone.retire",
    "flows.spendingFactor",
];

fn is_month(param: &str) -> bool {
    matches!(param, "returns.startDate" | "milestone.retire")
}

/// Liest `scenarios.sweeps` (`{param, from, to, step}`; bei Monaten `step` in Monaten).
pub fn parse(sweeps: &[Value]) -> Result<Vec<Sweep>> {
    let bad = |s: String| Error::Invalid(format!("Variation: {s}"));
    sweeps
        .iter()
        .map(|s| {
            let param = s
                .get("param")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("Parameter fehlt".into()))?
                .to_string();
            if !PARAMS.contains(&param.as_str()) {
                return Err(bad(format!("unbekannter Parameter «{param}»")));
            }
            let step = s.get("step").and_then(Value::as_f64).unwrap_or(0.0);
            if step <= 0.0 {
                return Err(bad(format!("{param}: Schritt muss grösser als 0 sein")));
            }
            // Ein Wert mehr als erlaubt, damit `count_combinations` die Überschreitung meldet.
            let cap = MAX_COMBINATIONS + 1;
            let values: Vec<Value> = if is_month(&param) {
                let ym = |k: &str| -> Result<YearMonth> {
                    s.get(k)
                        .and_then(Value::as_str)
                        .ok_or_else(|| bad(format!("{param}: «{k}» fehlt")))?
                        .parse::<YearMonth>()
                        .map_err(|e| bad(format!("{param}: {e}")))
                };
                let (a, b) = (ym("from")?, ym("to")?);
                let step = step.round().max(1.0) as i32;
                (0..)
                    .map(|i| a.plus(i * step))
                    .take_while(|m| *m <= b)
                    .take(cap)
                    .map(|m| json!(m.iso()))
                    .collect()
            } else {
                let num = |k: &str| {
                    s.get(k)
                        .and_then(Value::as_f64)
                        .ok_or_else(|| bad(format!("{param}: «{k}» fehlt")))
                };
                let (a, b) = (num("from")?, num("to")?);
                (0..)
                    .map(|i| a + i as f64 * step)
                    .take_while(|v| *v <= b + step * 1e-9)
                    .take(cap)
                    // Rundung, damit 0.1-Schritte als 0.3, 0.4 … erscheinen.
                    .map(|v| json!((v * 1e9).round() / 1e9))
                    .collect()
            };
            if values.is_empty() {
                return Err(bad(format!("{param}: «Bis» liegt vor «Von»")));
            }
            Ok(Sweep { param, values })
        })
        .collect()
}

/// Zahl der Kombinationen, ohne sie anzulegen; mehr als [`MAX_COMBINATIONS`] sind ein Fehler.
pub fn count_combinations(sweeps: &[Sweep]) -> Result<usize> {
    let mut n: usize = 1;
    for s in sweeps {
        n = n.saturating_mul(s.values.len());
    }
    if n > MAX_COMBINATIONS {
        let shown = if n == usize::MAX {
            "mehr als 1'000'000".to_string()
        } else {
            grouped(n)
        };
        return Err(Error::Invalid(format!(
            "Die Variationen ergeben {shown} Kombinationen; höchstens {} sind technisch möglich.",
            grouped(MAX_COMBINATIONS)
        )));
    }
    Ok(n)
}

/// Alle Kombinationen (kartesisches Produkt), jede als Liste der Werte in der Reihenfolge der Variationen.
pub fn combinations(sweeps: &[Sweep]) -> Result<Vec<Vec<Value>>> {
    count_combinations(sweeps)?;
    let mut out = vec![vec![]];
    for s in sweeps {
        out = out
            .into_iter()
            .flat_map(|c| {
                s.values.iter().map(move |v| {
                    let mut c = c.clone();
                    c.push(v.clone());
                    c
                })
            })
            .collect();
    }
    Ok(out)
}

/// Setzt einen variierten Wert in der Planung (JSON).
pub fn apply(plan: &mut Value, param: &str, value: &Value) -> Result<()> {
    let missing = |what: &str| Error::Invalid(format!("Variation {param}: {what} fehlt in der Planung"));
    match param {
        "wealth.stockShare" => plan["wealth"]["stockShare"] = value.clone(),
        "pension.pk.lumpsum" => plan["pension"]["pk"]["lumpsum"] = value.clone(),
        "returns.startDate" => plan["returns"]["startDate"] = value.clone(),
        // Entnahmerate der fixen Rate bzw. Startrate der Guardrails
        "withdrawal.rate" => {
            plan["withdrawal"]["fixedRate"] = json!({ "rate": value });
            if plan["withdrawal"]["guardrails"].is_object() {
                plan["withdrawal"]["guardrails"]["initialRate"] = value.clone();
            }
        }
        "milestone.retire" => {
            let ms = plan["milestones"]
                .as_array_mut()
                .and_then(|a| a.iter_mut().find(|m| m["id"] == "retire"))
                .ok_or_else(|| missing("Meilenstein «retire»"))?;
            let name = ms["name"].clone();
            *ms = json!({ "id": "retire", "name": name, "kind": "date", "date": value });
        }
        "flows.spendingFactor" => {
            let f = value.as_f64().ok_or_else(|| missing("Faktor"))?;
            // FEA-39: Der Faktor gilt für die Lebenshaltung ohne Wohnen (bei älteren Planungen dasselbe wie bisher).
            plan["flows"]["spendingFactor"] = json!(f);
        }
        _ => return Err(Error::Invalid(format!("unbekannter Parameter «{param}»"))),
    }
    Ok(())
}

/// Kennzahlen eines Szenarios.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioResult {
    /// Werte der Variationen (Reihenfolge wie `params`).
    pub values: Vec<Value>,
    pub runs: usize,
    pub success_rate: f64,
    /// Gesamtvermögen am Ende: 5., 50. und 95. Perzentil.
    pub p5: f64,
    pub p50: f64,
    pub p95: f64,
    /// Median des Endvermögens in Preisen des Startmonats.
    pub p50_real: f64,
    /// Median des Gesamtvermögens bei Pensionierung.
    pub at_retirement: Option<f64>,
    pub earliest_depletion: Option<YearMonth>,
    /// Endvermögen und Erschöpfung der Läufe an den Perzentilen (0 = schlechtester Lauf); Grundlage für das Optimum je Perzentil.
    pub percentiles: Vec<crate::batch::Percentile>,
    /// Fehler, falls das Szenario nicht gerechnet werden konnte (z. B. Startmonat ausserhalb der Reihe).
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_values_and_combinations() {
        let s = parse(&[
            json!({"param": "wealth.stockShare", "from": 0.3, "to": 1.0, "step": 0.1}),
            json!({"param": "milestone.retire", "from": "2026-09", "to": "2027-03", "step": 3}),
        ])
        .unwrap();
        assert_eq!(s[0].values.len(), 8);
        assert_eq!(s[0].values[1], json!(0.4));
        assert_eq!(s[1].values, vec![json!("2026-09"), json!("2026-12"), json!("2027-03")]);
        let c = combinations(&s).unwrap();
        assert_eq!(c.len(), 24);
        assert_eq!(c[1], vec![json!(0.3), json!("2026-12")]);
        assert!(parse(&[json!({"param": "x", "from": 0, "to": 1, "step": 1})]).is_err());
    }

    #[test]
    fn combinations_are_limited_only_technically_to_one_million() {
        let sweep =
            |param: &str, from: f64, to: f64, step: f64| json!({"param": param, "from": from, "to": to, "step": step});
        // 11 × 10 × 10 = 1100: früher abgelehnt, jetzt erlaubt
        let many = parse(&[
            sweep("wealth.stockShare", 0.0, 1.0, 0.1),
            sweep("pension.pk.lumpsum", 0.0, 0.9, 0.1),
            sweep("flows.spendingFactor", 1.0, 1.9, 0.1),
        ])
        .unwrap();
        assert_eq!(count_combinations(&many).unwrap(), 1100);
        assert_eq!(combinations(&many).unwrap().len(), 1100);
        // 101 × 100 × 100 = 1'010'000: über der technischen Grenze, ohne die Kombinationen anzulegen
        let too_many = parse(&[
            sweep("wealth.stockShare", 0.0, 1.0, 0.01),
            sweep("pension.pk.lumpsum", 0.0, 0.99, 0.01),
            sweep("flows.spendingFactor", 1.0, 1.99, 0.01),
        ])
        .unwrap();
        let err = count_combinations(&too_many).unwrap_err().to_string();
        assert_eq!(
            err,
            "Die Variationen ergeben 1'010'000 Kombinationen; höchstens 1'000'000 sind technisch möglich."
        );
        assert!(combinations(&too_many).is_err());
        // genau 1'000'000 sind erlaubt
        assert_eq!(grouped(1_000_000), "1'000'000");
        assert_eq!(grouped(999), "999");
    }

    #[test]
    fn apply_scales_spending_and_moves_retirement() {
        let mut p = json!({
            "milestones": [{"id": "retire", "name": "Pensionierung", "kind": "age", "person": "A", "years": 64}],
            "flows": {"spending": {"mode": "timed", "rows": [{"from": "start", "value": 1000}, {"from": "retire", "value": 800}]}}
        });
        // FEA-39: Der Faktor steht als eigenes Feld (gilt für die Lebenshaltung ohne Wohnen); die Reihe bleibt.
        apply(&mut p, "flows.spendingFactor", &json!(1.1)).unwrap();
        assert_eq!(p["flows"]["spendingFactor"], 1.1);
        assert_eq!(p["flows"]["spending"]["rows"][1]["value"], 800);
        apply(&mut p, "milestone.retire", &json!("2027-01")).unwrap();
        assert_eq!(p["milestones"][0]["date"], "2027-01");
        assert_eq!(p["milestones"][0]["kind"], "date");
    }

    // FEA-29: Startmonat und Entnahmerate (fixe Rate und Startrate der Guardrails).
    #[test]
    fn apply_start_date_and_withdrawal_rate() {
        let mut p = json!({"returns": {}, "withdrawal": {"guardrails": {"initialRate": 0.04}}});
        apply(&mut p, "returns.startDate", &json!("1990-01")).unwrap();
        assert_eq!(p["returns"]["startDate"], "1990-01");
        apply(&mut p, "withdrawal.rate", &json!(0.035)).unwrap();
        assert_eq!(p["withdrawal"]["fixedRate"], json!({"rate": 0.035}));
        assert_eq!(p["withdrawal"]["guardrails"]["initialRate"], json!(0.035));
        let mut p = json!({"withdrawal": {}});
        apply(&mut p, "withdrawal.rate", &json!(0.03)).unwrap();
        assert!(p["withdrawal"].get("guardrails").is_none());
        assert!(apply(&mut p, "x", &json!(1)).is_err());
    }

    // FEA-29: Genau 1'000'000 Werte sind erlaubt, ein Wert mehr nicht; längere Bereiche werden beim Anlegen gekappt.
    #[test]
    fn one_million_values_is_the_limit() {
        let sweep =
            |to: f64| parse(&[json!({"param": "wealth.stockShare", "from": 1.0, "to": to, "step": 1.0})]).unwrap();
        assert_eq!(count_combinations(&sweep(1_000_000.0)).unwrap(), 1_000_000);
        assert!(count_combinations(&sweep(1_000_001.0)).is_err());
        let capped = sweep(5_000_000.0);
        assert_eq!(capped[0].values.len(), 1_000_001);
        assert!(count_combinations(&capped).is_err());
    }
}
