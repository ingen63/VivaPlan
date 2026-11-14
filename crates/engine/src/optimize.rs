//! Optimierung der Entnahmestrategie für ein festes Basisszenario.
//!
//! Ziel: Ruinwahrscheinlichkeit bis zu einem Alter unter einer Grenze halten und dabei den realen
//! Konsum (Median und 10-%-Perzentil über alle Pfade und Jahre ab der Pensionierung) möglichst hoch,
//! seine Schwankung von Jahr zu Jahr möglichst klein machen.
//!
//! Ablauf: Parameter einer Strategie per Zufallssuche und anschliessender Mustersuche (Koordinaten-
//! suche mit schrumpfender Schrittweite) auf einer Menge Renditepfade verbessern, dann das Ergebnis
//! auf einer zweiten, unabhängigen Pfadmenge (anderer Startwert) prüfen. Alle Kandidaten sehen
//! dieselben Pfade, damit Unterschiede von den Parametern stammen.

use serde::Serialize;
use serde_json::{json, Value};

use crate::batch::Paths;
use crate::model::{Goal, Plan};
use crate::sim::{SimResult, Simulator};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Zielvorgaben.
#[derive(Clone, Debug)]
pub struct Objective {
    /// Höchste erlaubte Ruinwahrscheinlichkeit (0.05 = 5 %).
    pub max_ruin: f64,
    pub goal: Goal,
    /// Ziel «Kein Notverkauf von Immobilien»: Mindestanteil der geplanten Ausgaben, den der Median des Konsums erreichen muss.
    pub min_consumption: f64,
    /// Ruin zählt nur bis zu diesem Alter der Hauptperson (`None` = bis zum Simulationsende).
    pub ruin_age: Option<f64>,
    /// Gewicht des Medians im Konsumziel; der Rest geht auf das 10-%-Perzentil.
    pub weight_median: f64,
    /// Abzug für die Schwankung: Ziel × (1 − `vol_weight` × Schwankung).
    pub vol_weight: f64,
    /// Höchste erlaubte Schwankung (mittlere absolute Jahresänderung, 0.1 = 10 %).
    pub max_vol: Option<f64>,
}

impl Default for Objective {
    fn default() -> Self {
        Objective {
            max_ruin: 0.05,
            goal: Goal::Consumption,
            min_consumption: 0.9,
            ruin_age: None,
            weight_median: 0.5,
            vol_weight: 1.0,
            max_vol: None,
        }
    }
}

/// Kennzahlen einer Strategie über alle Pfade. Konsum in Preisen des Startmonats pro Jahr.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub runs: usize,
    /// Anteil der Pfade, in denen das Vermögen bis zum Ziel-Alter aufgebraucht ist.
    pub ruin: f64,
    /// Median des Jahreskonsums (alle Pfade und Jahre ab Pensionierung).
    pub median: f64,
    /// 10-%-Perzentil des Jahreskonsums.
    pub p10: f64,
    /// Mittlere absolute Änderung des Jahreskonsums gegenüber dem Vorjahr, Mittel über die Pfade.
    pub volatility: f64,
    /// Entnahmerate im ersten Jahr der Pensionierung: Entnahme aus dem Vermögen (Lebenshaltung und Wohnkosten
    /// minus Einnahmen, vor Steuern) im Verhältnis zum liquiden Vermögen bei Beginn; Median über die Pfade.
    pub draw_rate: f64,
    /// Anteil der Pfade mit mindestens einem ungeplanten Immobilienverkauf (Notverkauf).
    pub emergency_share: f64,
    /// Median der geplanten Ausgaben pro Jahr ab Pensionierung (inkl. Wohnkosten, Preise des Startmonats).
    pub planned: f64,
}

impl Metrics {
    /// Zielwert (höher ist besser), ohne Nebenbedingungen. Ziel 1: Konsum pro Jahr. Ziel 2: weniger ungeplante
    /// Verkäufe vor geringerer Ruinwahrscheinlichkeit (der Anteil wiegt so viel, dass die Ruin nur bei Gleichstand zählt).
    pub fn score(&self, o: &Objective) -> f64 {
        if o.goal == Goal::KeepProperty {
            return -(self.emergency_share * 1e6 + self.ruin);
        }
        let w = o.weight_median.clamp(0.0, 1.0);
        (w * self.median + (1.0 - w) * self.p10) * (1.0 - o.vol_weight * self.volatility).max(0.0)
    }

    /// Verletzung der Nebenbedingungen (0 = erfüllt).
    pub fn violation(&self, o: &Objective) -> f64 {
        let ruin = (self.ruin - o.max_ruin).max(0.0);
        if o.goal == Goal::KeepProperty {
            let short = if self.planned > 0.0 {
                (o.min_consumption - self.median / self.planned).max(0.0)
            } else {
                0.0
            };
            return ruin + short;
        }
        ruin + o.max_vol.map_or(0.0, |v| (self.volatility - v).max(0.0))
    }

    pub fn feasible(&self, o: &Objective) -> bool {
        self.violation(o) <= 1e-12
    }

    /// Vergleichswert: zulässige Kandidaten nach Ziel, unzulässige darunter nach Verletzung.
    /// Bei gleichem Ziel (z. B. fixe Ausgaben) gewinnt die kleinere Ruinwahrscheinlichkeit.
    fn fitness(&self, o: &Objective) -> f64 {
        let v = self.violation(o);
        if v > 1e-12 {
            -1e9 * (1.0 + v)
        } else if o.goal == Goal::KeepProperty {
            self.score(o)
        } else {
            self.score(o) * (1.0 - 0.05 * self.ruin)
        }
    }
}

/// Ein einstellbarer Parameter mit Standard-Bandbreite der Suche.
#[derive(Clone, Copy, Debug)]
pub struct ParamDef {
    pub key: &'static str,
    pub label: &'static str,
    pub lo: f64,
    pub hi: f64,
    /// Ausgabe als Prozent.
    pub percent: bool,
    /// Einheit der Anzeige: `%`, `Jahre`, `Monate` oder `Alter`.
    pub unit: &'static str,
    /// Äusserste zulässige Grenzen für selbst gewählte Bandbreiten.
    pub min: f64,
    pub max: f64,
    /// Ohne Angabe in der Planung bleibt der Parameter fix.
    pub fixed: bool,
}

const fn p(
    key: &'static str,
    label: &'static str,
    lo: f64,
    hi: f64,
    unit: &'static str,
    min: f64,
    max: f64,
) -> ParamDef {
    ParamDef {
        key,
        label,
        lo,
        hi,
        percent: unit.len() == 1,
        unit,
        min,
        max,
        fixed: false,
    }
}

impl ParamDef {
    /// Anzeigename mit Einheit, soweit sie nicht Prozent ist.
    fn shown(&self) -> String {
        if self.percent {
            self.label.into()
        } else {
            format!("{} ({})", self.label, self.unit)
        }
    }

    const fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
}

const STOCK_SHARE: ParamDef = p("stockShare", "Aktienanteil", 0.1, 1.0, "%", 0.0, 1.0);

/// Bandbreiten je Strategie und Parameter (Schlüssel der Strategie, dann des Parameters).
pub type Ranges = std::collections::BTreeMap<String, std::collections::BTreeMap<String, crate::model::ParamRange>>;

/// Strategie der Optimierung: die Entnahmestrategien der Planung und dazu 3-Topf mit Guardrails (FEA-05), die in der
/// Planung «3-Topf» mit `buckets.spending = "guardrails"` ist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptStrategy {
    Standard,
    ThreeBuckets,
    ThreeBucketsGuardrails,
    Guardrails,
    FixedRate,
    Vpw,
    Endowment,
}

impl OptStrategy {
    /// Strategie einer Planung.
    pub fn of(w: &crate::model::Withdrawal) -> Self {
        use crate::model::Strategy;
        match w.strategy {
            Strategy::Standard => OptStrategy::Standard,
            Strategy::ThreeBuckets if w.buckets_with_guardrails() => OptStrategy::ThreeBucketsGuardrails,
            Strategy::ThreeBuckets => OptStrategy::ThreeBuckets,
            Strategy::Guardrails => OptStrategy::Guardrails,
            Strategy::FixedRate => OptStrategy::FixedRate,
            Strategy::Vpw => OptStrategy::Vpw,
            Strategy::Endowment => OptStrategy::Endowment,
        }
    }
}

pub fn strategy_name(s: OptStrategy) -> &'static str {
    match s {
        OptStrategy::Standard => "Standard",
        OptStrategy::ThreeBuckets => "3-Topf",
        OptStrategy::ThreeBucketsGuardrails => "3-Topf + Guardrails",
        OptStrategy::Guardrails => "Guardrails",
        OptStrategy::FixedRate => "Fixe Rate",
        OptStrategy::Vpw => "VPW",
        OptStrategy::Endowment => "Endowment",
    }
}

pub fn strategy_key(s: OptStrategy) -> &'static str {
    match s {
        OptStrategy::Standard => "standard",
        OptStrategy::ThreeBuckets => "three_buckets",
        OptStrategy::ThreeBucketsGuardrails => "three_buckets_guardrails",
        OptStrategy::Guardrails => "guardrails",
        OptStrategy::FixedRate => "fixed_rate",
        OptStrategy::Vpw => "vpw",
        OptStrategy::Endowment => "endowment",
    }
}

pub fn all_strategies() -> [OptStrategy; 7] {
    [
        OptStrategy::Standard,
        OptStrategy::ThreeBuckets,
        OptStrategy::ThreeBucketsGuardrails,
        OptStrategy::Guardrails,
        OptStrategy::FixedRate,
        OptStrategy::Vpw,
        OptStrategy::Endowment,
    ]
}

/// Parameter, die für eine Strategie optimiert werden. Floor und Ceiling bleiben wie in der Planung.
pub fn param_defs(s: OptStrategy) -> Vec<ParamDef> {
    let floor = p("floor", "Untergrenze der Lebenshaltung", 0.5, 1.0, "%", 0.0, 2.0).fixed();
    let ceiling = p("ceiling", "Obergrenze der Lebenshaltung", 1.0, 1.5, "%", 0.0, 5.0).fixed();
    match s {
        OptStrategy::Standard => vec![
            STOCK_SHARE,
            p("reserveMonths", "Reserve", 3.0, 36.0, "Monate", 0.0, 120.0),
        ],
        OptStrategy::FixedRate => vec![
            p("rate", "Entnahmerate", 0.02, 0.08, "%", 0.0, 0.5),
            STOCK_SHARE,
            floor,
            ceiling,
        ],
        OptStrategy::Vpw => vec![
            p("horizonAge", "Planungsalter", 85.0, 105.0, "Alter", 60.0, 120.0),
            p("realReturn", "Realrendite", -0.01, 0.05, "%", -0.1, 0.2),
            STOCK_SHARE,
        ],
        OptStrategy::Endowment => vec![
            p("eRate", "Zielrate", 0.02, 0.08, "%", 0.0, 0.5),
            p("smoothing", "Glättung", 0.05, 1.0, "%", 0.0, 1.0),
            STOCK_SHARE,
        ],
        OptStrategy::Guardrails => vec![
            p("initialRate", "Startrate", 0.02, 0.08, "%", 0.0, 0.5),
            p("upper", "Obere Leitplanke", 0.05, 0.5, "%", 0.0, 2.0),
            p("lower", "Untere Leitplanke", 0.05, 0.5, "%", 0.0, 2.0),
            p("adjust", "Anpassung", 0.02, 0.3, "%", 0.0, 1.0),
            STOCK_SHARE,
            floor,
            ceiling,
        ],
        OptStrategy::ThreeBuckets => bucket_defs().to_vec(),
        // FEA-05: Töpfe und Guardrails; untere Leitplanke und Anpassung sind standardmässig fix.
        OptStrategy::ThreeBucketsGuardrails => {
            let mut v = bucket_defs().to_vec();
            v.extend([
                p("initialRate", "Startrate", 0.02, 0.08, "%", 0.0, 0.5),
                p("upper", "Obere Leitplanke", 0.05, 0.5, "%", 0.0, 2.0),
                p("lower", "Untere Leitplanke", 0.05, 0.5, "%", 0.0, 2.0).fixed(),
                p("adjust", "Anpassung", 0.02, 0.3, "%", 0.0, 1.0).fixed(),
                floor,
                ceiling,
            ]);
            v
        }
    }
}

const fn bucket_defs() -> [ParamDef; 3] {
    [
        p("cashYears", "Topf 1: Cash", 0.5, 4.0, "Jahre", 0.0, 20.0),
        p("bondYears", "Topf 2: Obligationen", 0.0, 10.0, "Jahre", 0.0, 30.0),
        p(
            "refillThreshold",
            "Nachfüllen aus Aktien ab Vorjahresrendite",
            -0.1,
            0.2,
            "%",
            -1.0,
            1.0,
        ),
    ]
}

/// Parameter einer Strategie nach den Einstellungen: `(variiert, fix)`. Variierte Parameter tragen
/// die gewählte Bandbreite; fixe behalten den Wert der Planung.
pub fn split_defs(s: OptStrategy, ranges: &Ranges) -> Result<(Vec<ParamDef>, Vec<ParamDef>)> {
    let (mut varied, mut fixed) = (vec![], vec![]);
    for mut d in param_defs(s) {
        let r = ranges.get(strategy_key(s)).and_then(|m| m.get(d.key));
        if r.and_then(|r| r.fixed).unwrap_or(d.fixed) {
            fixed.push(d);
            continue;
        }
        let (lo, hi) = (
            r.and_then(|r| r.lo).unwrap_or(d.lo),
            r.and_then(|r| r.hi).unwrap_or(d.hi),
        );
        let show = |v: f64| {
            if d.percent {
                format!("{} %", (v * 1e4).round() / 100.0)
            } else {
                format!("{v}")
            }
        };
        if !lo.is_finite() || !hi.is_finite() || lo > hi {
            return Err(Error::Invalid(format!(
                "{}, {}: Die Bandbreite von {} bis {} ist ungültig («von» darf nicht grösser als «bis» sein).",
                strategy_name(s),
                d.label,
                show(lo),
                show(hi)
            )));
        }
        if lo < d.min || hi > d.max {
            return Err(Error::Invalid(format!(
                "{}, {}: Die Bandbreite muss zwischen {} und {} liegen.",
                strategy_name(s),
                d.label,
                show(d.min),
                show(d.max)
            )));
        }
        (d.lo, d.hi) = (lo, hi);
        varied.push(d);
    }
    Ok((varied, fixed))
}

/// Angaben zu einem Parameter für die Oberfläche.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogParam {
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub percent: bool,
    pub lo: f64,
    pub hi: f64,
    pub min: f64,
    pub max: f64,
    /// Ohne eigene Einstellung bleibt der Parameter fix.
    pub fixed: bool,
    /// Wert in der Planung (`None`: nicht gesetzt, die Strategie nimmt ihren Standardwert).
    pub current: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStrategy {
    pub key: &'static str,
    pub name: &'static str,
    pub params: Vec<CatalogParam>,
}

/// Alle Strategien mit ihren Parametern, Standard-Bandbreiten und den Werten der Planung.
pub fn catalog(plan: &Value) -> Vec<CatalogStrategy> {
    all_strategies()
        .into_iter()
        .map(|s| CatalogStrategy {
            key: strategy_key(s),
            name: strategy_name(s),
            params: param_defs(s)
                .into_iter()
                .map(|d| CatalogParam {
                    key: d.key,
                    label: d.label,
                    unit: d.unit,
                    percent: d.percent,
                    lo: d.lo,
                    hi: d.hi,
                    min: d.min,
                    max: d.max,
                    fixed: d.fixed,
                    current: current(plan, d.key).or_else(|| model_default(d.key)),
                })
                .collect(),
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Planung (JSON) anpassen
// ---------------------------------------------------------------------------------------------

fn last_value(series: &Value) -> Option<f64> {
    match series {
        Value::Number(n) => n.as_f64(),
        Value::Object(o) => match o.get("rows") {
            Some(Value::Array(rows)) => rows.last().and_then(|r| r["value"].as_f64()),
            _ => o.get("value").and_then(Value::as_f64),
        },
        _ => None,
    }
}

/// Setzt den Wert ab der Pensionierung; frühere Einträge bleiben.
fn set_from_retirement(series: &mut Value, retire: &str, v: f64) {
    let mut rows: Vec<Value> = match &*series {
        Value::Object(o) => match o.get("rows") {
            Some(Value::Array(rows)) => rows
                .iter()
                .filter(|r| match r["from"].as_str() {
                    Some(f) if f.len() == 7 && f.as_bytes()[4] == b'-' => f < retire,
                    Some("retire") => false,
                    _ => true,
                })
                .cloned()
                .collect(),
            _ => vec![json!({"from": "start", "value": last_value(series).unwrap_or(v)})],
        },
        Value::Number(n) => vec![json!({"from": "start", "value": n})],
        _ => vec![],
    };
    rows.push(json!({"from": retire, "value": v}));
    *series = json!({"mode": "timed", "rows": rows});
}

fn current(plan: &Value, key: &str) -> Option<f64> {
    match key {
        "stockShare" => last_value(&plan["wealth"]["stockShare"]),
        "reserveMonths" => last_value(&plan["wealth"]["reserveMonths"]),
        "rate" => plan["withdrawal"]["fixedRate"]["rate"].as_f64(),
        "floor" | "ceiling" => plan["withdrawal"][key].as_f64(),
        "initialRate" | "upper" | "lower" | "adjust" => plan["withdrawal"]["guardrails"][key].as_f64(),
        "cashYears" | "bondYears" | "refillThreshold" => plan["withdrawal"]["buckets"][key].as_f64(),
        "horizonAge" | "realReturn" => plan["withdrawal"]["vpw"][key].as_f64(),
        "eRate" => plan["withdrawal"]["endowment"]["rate"].as_f64(),
        "smoothing" => plan["withdrawal"]["endowment"]["smoothing"].as_f64(),
        _ => None,
    }
}

/// Standardwert eines Parameters, wenn die Planung für die Strategie nichts festlegt.
fn model_default(key: &str) -> Option<f64> {
    use crate::model::{Buckets, Endowment, FixedRate, Guardrails, Vpw};
    let (b, g, v, e) = (
        Buckets::default(),
        Guardrails::default(),
        Vpw::default(),
        Endowment::default(),
    );
    Some(match key {
        "cashYears" => b.cash_years,
        "bondYears" => b.bond_years,
        "refillThreshold" => b.refill_threshold,
        "initialRate" => g.initial_rate,
        "upper" => g.upper,
        "lower" => g.lower,
        "adjust" => g.adjust,
        "horizonAge" => v.horizon_age,
        "realReturn" => v.real_return,
        "eRate" => e.rate,
        "smoothing" => e.smoothing,
        "rate" => FixedRate::default().rate,
        _ => return None,
    })
}

fn set(plan: &mut Value, key: &str, v: f64, retire: &str) {
    match key {
        "stockShare" => set_from_retirement(&mut plan["wealth"]["stockShare"], retire, v),
        "reserveMonths" => set_from_retirement(&mut plan["wealth"]["reserveMonths"], retire, v),
        "rate" => plan["withdrawal"]["fixedRate"] = json!({ "rate": v }),
        "floor" | "ceiling" => plan["withdrawal"][key] = json!(v),
        "initialRate" | "upper" | "lower" | "adjust" => {
            if !plan["withdrawal"]["guardrails"].is_object() {
                plan["withdrawal"]["guardrails"] =
                    json!({"initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1});
            }
            plan["withdrawal"]["guardrails"][key] = json!(v);
        }
        "cashYears" | "bondYears" | "refillThreshold" => {
            if !plan["withdrawal"]["buckets"].is_object() {
                plan["withdrawal"]["buckets"] = json!({"cashYears": 2.0, "bondYears": 6.0, "refillThreshold": 0.0});
            }
            plan["withdrawal"]["buckets"][key] = json!(v);
        }
        "horizonAge" | "realReturn" => {
            if !plan["withdrawal"]["vpw"].is_object() {
                plan["withdrawal"]["vpw"] = json!({"horizonAge": 100.0, "realReturn": 0.02});
            }
            plan["withdrawal"]["vpw"][key] = json!(v);
        }
        "eRate" | "smoothing" => {
            if !plan["withdrawal"]["endowment"].is_object() {
                plan["withdrawal"]["endowment"] = json!({"rate": 0.04, "smoothing": 0.3});
            }
            let k = if key == "eRate" { "rate" } else { key };
            plan["withdrawal"]["endowment"][k] = json!(v);
        }
        _ => {}
    }
}

/// Planung mit Strategie und Parametern.
pub fn with_params(base: &Value, strategy: OptStrategy, defs: &[ParamDef], x: &[f64], retire: YearMonth) -> Value {
    let mut v = base.clone();
    let combined = strategy == OptStrategy::ThreeBucketsGuardrails;
    v["withdrawal"]["strategy"] = json!(if combined {
        "three_buckets"
    } else {
        strategy_key(strategy)
    });
    // FEA-05: Die Ausgabenregel der Töpfe folgt der Strategie, nicht der Planung.
    if v["withdrawal"]["buckets"].is_object() || combined {
        if !v["withdrawal"]["buckets"].is_object() {
            v["withdrawal"]["buckets"] = json!({"cashYears": 2.0, "bondYears": 6.0, "refillThreshold": 0.0});
        }
        v["withdrawal"]["buckets"]["spending"] = json!(if combined { "guardrails" } else { "planned" });
    }
    v["scenarios"]["sweeps"] = json!([]);
    let r = retire.iso();
    for (d, x) in defs.iter().zip(x) {
        set(&mut v, d.key, *x, &r);
    }
    v
}

// ---------------------------------------------------------------------------------------------
// Kennzahlen
// ---------------------------------------------------------------------------------------------

struct PathStats {
    ruined: bool,
    /// Realer Konsum pro Jahr (12-Monats-Blöcke ab Pensionierung).
    annual: Vec<f64>,
    vol: f64,
    /// Entnahmerate im ersten Jahr ab Pensionierung.
    draw_rate: f64,
    /// Mindestens ein Notverkauf in diesem Lauf.
    emergency: bool,
    /// Geplante Ausgaben pro Jahr (real, inkl. Wohnkosten), gleiche Blöcke wie `annual`.
    planned: Vec<f64>,
}

fn path_stats(sim: &Simulator, res: &SimResult, retire: YearMonth, o: &Objective) -> PathStats {
    let ruined = res.summary.depleted.is_some_and(|m| match o.ruin_age {
        Some(age) => sim.age_at(m) <= age,
        None => true,
    });
    // Nach der Erschöpfung zählt nur noch, was das Einkommen deckt.
    let depleted = res.summary.depleted;
    let mut annual = Vec::new();
    let mut planned = Vec::new();
    let (mut sum, mut plan_sum, mut n) = (0.0, 0.0, 0);
    for r in res.records.iter().filter(|r| r.month >= retire) {
        let spent = if depleted.is_some_and(|d| r.month >= d) {
            r.spending.min(r.income).max(0.0)
        } else {
            r.spending
        };
        sum += spent / r.price_index;
        plan_sum += (r.planned_spending + r.housing) / r.price_index;
        n += 1;
        if n == 12 {
            annual.push(sum);
            planned.push(plan_sum);
            sum = 0.0;
            plan_sum = 0.0;
            n = 0;
        }
    }
    let changes: Vec<f64> = annual
        .windows(2)
        .filter(|w| w[0] > 1e-9)
        .map(|w| ((w[1] - w[0]) / w[0]).abs())
        .collect();
    let vol = if changes.is_empty() {
        0.0
    } else {
        changes.iter().sum::<f64>() / changes.len() as f64
    };
    // Entnahme der ersten 12 Monate ab Pensionierung im Verhältnis zum liquiden Vermögen davor.
    let first: Vec<_> = res.records.iter().filter(|r| r.month >= retire).take(12).collect();
    let draw_rate = first.first().map_or(0.0, |r| {
        let start = r.liquid + r.spending - r.income + r.taxes;
        let draw: f64 = first.iter().map(|r| r.spending - r.income).sum();
        if start > 0.0 {
            draw.max(0.0) / start
        } else {
            0.0
        }
    });
    PathStats {
        ruined,
        annual,
        vol,
        draw_rate,
        emergency: res.summary.emergency_sales > 0,
        planned,
    }
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() - 1) as f64 * q).round() as usize]
}

/// Rechnet eine Planung mit allen Pfaden und fasst zusammen.
pub fn evaluate(plan: &Plan, paths: &Paths, o: &Objective) -> Result<Metrics> {
    let sim = Simulator::new(plan)?;
    let retire = sim
        .retire_at()
        .ok_or_else(|| Error::Invalid("Die Planung hat keine Pensionierung.".into()))?;
    let stats: Vec<PathStats> = crate::progress::par_map(paths.len(), |i| {
        path_stats(&sim, &sim.run(&paths.get(i), true), retire, o)
    });
    if crate::progress::cancelled() {
        return Err(Error::Cancelled);
    }
    let n = stats.len().max(1);
    let mut all: Vec<f64> = stats.iter().flat_map(|s| s.annual.iter().copied()).collect();
    all.sort_by(f64::total_cmp);
    Ok(Metrics {
        runs: stats.len(),
        ruin: stats.iter().filter(|s| s.ruined).count() as f64 / n as f64,
        median: quantile(&all, 0.5),
        p10: quantile(&all, 0.1),
        volatility: stats.iter().map(|s| s.vol).sum::<f64>() / n as f64,
        draw_rate: {
            let mut r: Vec<f64> = stats.iter().map(|s| s.draw_rate).collect();
            r.sort_by(f64::total_cmp);
            quantile(&r, 0.5)
        },
        emergency_share: stats.iter().filter(|s| s.emergency).count() as f64 / n as f64,
        planned: {
            let mut p: Vec<f64> = stats.iter().flat_map(|s| s.planned.iter().copied()).collect();
            p.sort_by(f64::total_cmp);
            quantile(&p, 0.5)
        },
    })
}

// ---------------------------------------------------------------------------------------------
// Suche
// ---------------------------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        // splitmix64
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Ergebnis der Optimierung einer Strategie.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub strategy: String,
    /// Parameter (Schlüssel, Anzeigename, Wert, als Prozent).
    pub params: Vec<(String, String, f64, bool)>,
    /// Fixe Parameter mit ihrem Wert aus der Planung (gleiche Form wie `params`).
    pub fixed: Vec<(String, String, f64, bool)>,
    /// Kennzahlen auf den Pfaden der Optimierung.
    pub fitted: Metrics,
    /// Kennzahlen auf unabhängigen Pfaden.
    pub validated: Metrics,
    pub feasible: bool,
    pub evaluations: usize,
    /// Strategie und Anlageparameter der Planung (`withdrawal`, `wealth.stockShare`, `wealth.reserveMonths`),
    /// zum Übernehmen in die Planung.
    pub patch: Value,
}

pub struct Settings {
    /// Zufällige Startpunkte.
    pub samples: usize,
    pub seed: u64,
}

fn parse_plan(v: &Value) -> Result<Plan> {
    serde_json::from_value(v.clone()).map_err(|e| Error::Invalid(e.to_string()))
}

/// Pfade der Optimierung: `fit` für die Suche, `check` für die unabhängige Prüfung.
pub struct PathSets<'a, 'p> {
    pub fit: &'a Paths<'p>,
    pub check: &'a Paths<'p>,
}

/// Zielvorgaben der Suche auf `n` Pfaden. Die Suche nutzt die Ruingrenze sonst bis zum Anschlag aus und trifft auf
/// neuen Pfaden darüber (Zufallsfehler der Schätzung): Sicherheitsabstand von 1,5 Standardfehlern.
fn fit_objective(o: &Objective, n: usize) -> Objective {
    let n = n.max(1) as f64;
    let margin = 1.5 * (o.max_ruin * (1.0 - o.max_ruin) / n).sqrt();
    Objective {
        max_ruin: (o.max_ruin - margin).max(0.0),
        ..o.clone()
    }
}

/// Startpunkte der Suche: Werte der Planung (soweit vorhanden), Mitte des Bereichs, Zufall.
/// Der Wert der Planung zählt nur, wenn er in der gewählten Bandbreite liegt (sonst Mitte).
fn start_points(base: &Value, defs: &[ParamDef], st: &Settings) -> Vec<Vec<f64>> {
    let mid = |d: &ParamDef| (d.lo + d.hi) / 2.0;
    let mut starts: Vec<Vec<f64>> = vec![defs
        .iter()
        .map(|d| {
            current(base, d.key)
                .filter(|v| (d.lo..=d.hi).contains(v))
                .unwrap_or(mid(d))
        })
        .collect()];
    // Ohne variierte Parameter gibt es nur die Planung selbst zu prüfen.
    if defs.is_empty() {
        return starts;
    }
    starts.push(defs.iter().map(mid).collect());
    let mut rng = Rng(st.seed ^ 0xA5A5_5A5A);
    for _ in 0..st.samples {
        starts.push(defs.iter().map(|d| d.lo + (d.hi - d.lo) * rng.next()).collect());
    }
    starts
}

/// Sucht das Maximum von `eval` innerhalb der Grenzen `bounds` (je Parameter unten, oben): Der beste Startpunkt ist
/// der Ausgangswert, danach folgt eine Mustersuche (je Achse ± Schritt, bei Misserfolg Schritt halbieren).
/// Ergebnis: Parameter, Zusatzwert von `eval` und Fitness des besten Punkts.
fn search<T>(
    starts: Vec<Vec<f64>>,
    bounds: &[(f64, f64)],
    mut eval: impl FnMut(&[f64]) -> Result<(T, f64)>,
) -> Result<(Vec<f64>, T, f64)> {
    let clamp = |x: &mut Vec<f64>| {
        for (v, &(lo, hi)) in x.iter_mut().zip(bounds) {
            *v = v.clamp(lo, hi);
        }
    };

    let mut best: Option<(Vec<f64>, T, f64)> = None;
    for mut x in starts {
        clamp(&mut x);
        let (m, f) = eval(&x)?;
        if best.as_ref().is_none_or(|b| f > b.2) {
            best = Some((x, m, f));
        }
    }
    let (mut bx, mut bm, mut bf) = best.expect("mindestens ein Startpunkt");

    let mut step = 0.2;
    while step > 0.01 {
        let mut improved = false;
        for (i, &(lo, hi)) in bounds.iter().enumerate() {
            for dir in [1.0, -1.0] {
                let mut x = bx.clone();
                x[i] += dir * step * (hi - lo);
                clamp(&mut x);
                if x[i] == bx[i] {
                    continue;
                }
                let (m, f) = eval(&x)?;
                if f > bf {
                    (bx, bm, bf) = (x, m, f);
                    improved = true;
                    break;
                }
            }
        }
        if !improved {
            step /= 2.0;
        }
    }
    Ok((bx, bm, bf))
}

/// Optimiert eine Strategie mit den Pfaden `paths`.
pub fn optimize_strategy(
    base: &Value,
    strategy: OptStrategy,
    retire: YearMonth,
    paths: PathSets<'_, '_>,
    o: &Objective,
    st: &Settings,
    ranges: &Ranges,
) -> Result<Outcome> {
    let PathSets { fit, check } = paths;
    let (defs, fixed_defs) = split_defs(strategy, ranges)?;
    let fit_obj = fit_objective(o, fit.len());
    let o_fit = &fit_obj;
    let mut evals = 0usize;
    let eval = |x: &[f64]| -> Result<(Metrics, f64)> {
        evals += 1;
        let plan = parse_plan(&with_params(base, strategy, &defs, x, retire))?;
        let m = evaluate(&plan, fit, o_fit)?;
        let f = m.fitness(o_fit);
        Ok((m, f))
    };
    let bounds: Vec<(f64, f64)> = defs.iter().map(|d| (d.lo, d.hi)).collect();
    let (bx, bm, _) = search(start_points(base, &defs, st), &bounds, eval)?;

    let best_plan = with_params(base, strategy, &defs, &bx, retire);
    let validated = evaluate(&parse_plan(&best_plan)?, check, o)?;
    Ok(Outcome {
        patch: json!({
            "withdrawal": best_plan["withdrawal"],
            "wealth": { "stockShare": best_plan["wealth"]["stockShare"], "reserveMonths": best_plan["wealth"]["reserveMonths"] },
        }),
        strategy: strategy_name(strategy).into(),
        params: defs
            .iter()
            .zip(&bx)
            .map(|(d, v)| (d.key.to_string(), d.shown(), *v, d.percent))
            .collect(),
        fixed: fixed_defs
            .iter()
            .filter_map(|d| {
                let v = current(&best_plan, d.key).or_else(|| model_default(d.key))?;
                Some((d.key.to_string(), d.shown(), v, d.percent))
            })
            .collect(),
        fitted: bm,
        validated,
        feasible: validated.feasible(o),
        evaluations: evals,
    })
}

// ---------------------------------------------------------------------------------------------
// Auftrag von CLI und Oberfläche
// ---------------------------------------------------------------------------------------------

fn d_ruin() -> f64 {
    0.05
}
fn d_weight() -> f64 {
    0.5
}
fn d_one() -> f64 {
    1.0
}
fn d_fit() -> usize {
    5000
}
fn d_check() -> usize {
    10_000
}
fn d_samples() -> usize {
    30
}
fn d_seed() -> u64 {
    1
}

/// Einstellungen einer Optimierung (JSON der Oberfläche; Anteile als Zahl, 0.05 = 5 %).
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Request {
    /// Schlüssel der Strategien (`guardrails`, `vpw` …); leer = alle.
    pub strategies: Vec<String>,
    pub max_ruin: f64,
    /// Ziel der Optimierung (Standard: Konsum).
    pub goal: Goal,
    /// Ziel «Kein Notverkauf von Immobilien»: Mindestanteil der geplanten Ausgaben für den Median des Konsums.
    pub min_consumption: f64,
    pub ruin_age: Option<f64>,
    pub weight_median: f64,
    pub vol_weight: f64,
    pub max_vol: Option<f64>,
    pub paths: usize,
    pub check_paths: usize,
    pub samples: usize,
    pub seed: u64,
    /// Bandbreiten und fixe Parameter je Strategie (siehe [`split_defs`]).
    pub ranges: Ranges,
}

impl Default for Request {
    fn default() -> Self {
        Request {
            strategies: vec![],
            max_ruin: d_ruin(),
            goal: Goal::Consumption,
            min_consumption: 0.9,
            ruin_age: None,
            weight_median: d_weight(),
            vol_weight: d_one(),
            max_vol: None,
            paths: d_fit(),
            check_paths: d_check(),
            samples: d_samples(),
            seed: d_seed(),
            ranges: Ranges::new(),
        }
    }
}

impl From<&crate::model::OptimizerSettings> for Request {
    fn from(s: &crate::model::OptimizerSettings) -> Self {
        Request {
            strategies: s.strategies.clone(),
            max_ruin: s.max_ruin,
            goal: s.goal,
            min_consumption: s.min_consumption,
            ruin_age: s.ruin_age,
            weight_median: s.weight_median,
            vol_weight: s.vol_weight,
            max_vol: s.max_vol,
            paths: s.paths,
            check_paths: s.check_paths,
            samples: s.samples,
            seed: d_seed(),
            ranges: s.ranges.clone(),
        }
    }
}

impl Request {
    pub fn objective(&self) -> Objective {
        Objective {
            max_ruin: self.max_ruin,
            goal: self.goal,
            min_consumption: self.min_consumption,
            ruin_age: self.ruin_age,
            weight_median: self.weight_median,
            vol_weight: self.vol_weight,
            max_vol: self.max_vol,
        }
    }

    pub fn strategy_list(&self) -> Result<Vec<OptStrategy>> {
        if self.strategies.is_empty() || self.strategies.iter().any(|k| k == "all") {
            return Ok(all_strategies().to_vec());
        }
        self.strategies
            .iter()
            .map(|k| {
                all_strategies()
                    .into_iter()
                    .find(|s| strategy_key(*s) == k.trim())
                    .ok_or_else(|| {
                        Error::Invalid(format!(
                            "Unbekannte Strategie «{k}» (standard, three_buckets, three_buckets_guardrails, guardrails, fixed_rate, vpw, endowment, all)"
                        ))
                    })
            })
            .collect()
    }
}

/// Die Planung, wie sie ist, auf den Prüfpfaden.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub strategy: String,
    pub metrics: Metrics,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub goal: Goal,
    pub min_consumption: f64,
    pub retire: String,
    pub fit_paths: usize,
    pub check_paths: usize,
    pub source: String,
    pub reference: Reference,
    /// Beste zuerst: zulässige nach Ziel, unzulässige nach geringster Verletzung.
    pub outcomes: Vec<Outcome>,
    pub elapsed_ms: f64,
}

fn with_paths(base: &Value, n: usize, seed_offset: u64) -> Value {
    let mut v = base.clone();
    for key in ["bootstrap", "parametric"] {
        if v["returns"][key].is_object() {
            v["returns"][key]["paths"] = json!(n);
            let seed = v["returns"][key]["seed"].as_u64().unwrap_or(42);
            v["returns"][key]["seed"] = json!(seed + seed_offset);
        }
    }
    v
}

/// Typische Zahl der Auswertungen der Mustersuche (nur für die Fortschrittsanzeige).
fn search_evals(s: OptStrategy) -> u64 {
    match strategy_key(s) {
        "standard" => 22,
        "fixed_rate" => 14,
        "three_buckets" => 46,
        "endowment" => 40,
        "vpw" => 74,
        "three_buckets_guardrails" => 110,
        _ => 90,
    }
}

/// Geschätzte Zahl der Läufe für die Fortschrittsanzeige: je Strategie Startpunkte plus Mustersuche
/// (Erfahrungswerte, anteilig zu den variierten Parametern) auf den Suchpfaden, dazu die Prüfpfade für jede Strategie
/// und für die Planung selbst.
fn progress_total(req: &Request, strategies: &[OptStrategy]) -> Result<u64> {
    let mut search = 0;
    for s in strategies {
        let (varied, _) = split_defs(*s, &req.ranges)?;
        let usual = param_defs(*s).iter().filter(|d| !d.fixed).count().max(1) as u64;
        search += if varied.is_empty() {
            1
        } else {
            req.samples as u64 + 2 + search_evals(*s) * varied.len() as u64 / usual
        };
    }
    Ok(search * req.paths as u64 + (strategies.len() as u64 + 1) * req.check_paths as u64)
}

/// Rangfolge der Ergebnisse: zulässige nach Ziel auf den Prüfpfaden, unzulässige darunter nach geringster Verletzung.
fn rank(o: &Outcome, obj: &Objective) -> f64 {
    if o.feasible {
        o.validated.score(obj)
    } else {
        -1e9 * (1.0 + o.validated.violation(obj))
    }
}

/// Sucht für die gewählten Strategien die besten Parameter. `base_dir`: Ordner der Projektdatei,
/// `library`: Datenbibliothek.
pub fn run_request(
    base: &Value,
    base_dir: &std::path::Path,
    library: Option<&std::path::Path>,
    req: &Request,
) -> Result<Report> {
    use crate::batch;
    let t0 = std::time::Instant::now();
    let obj = req.objective();
    if !(0.0..=1.0).contains(&obj.max_ruin) {
        return Err(Error::Invalid(
            "Die Ruingrenze muss zwischen 0 % und 100 % liegen.".into(),
        ));
    }
    if req.paths == 0 || req.check_paths == 0 {
        return Err(Error::Invalid("Die Anzahl Pfade muss grösser als 0 sein.".into()));
    }
    if req.goal == Goal::KeepProperty {
        let has_property = base["realestate"]["properties"]
            .as_array()
            .is_some_and(|p| p.iter().any(|x| x["status"] != "rent"));
        if !has_property {
            return Err(Error::Invalid(
                "Das Ziel «Kein Notverkauf von Immobilien» braucht eine Planung mit mindestens einer Immobilie.".into(),
            ));
        }
        if !(0.0..=2.0).contains(&req.min_consumption) {
            return Err(Error::Invalid(
                "Der Mindestkonsum muss zwischen 0 % und 200 % der geplanten Ausgaben liegen.".into(),
            ));
        }
    }
    let strategies = req.strategy_list()?;
    let _job = crate::progress::start(progress_total(req, &strategies)?);
    let plan = parse_plan(base)?;
    let sim = Simulator::new(&plan)?;
    let retire = sim
        .retire_at()
        .ok_or_else(|| Error::Invalid("Die Planung hat keine Pensionierung (Pensionskasse «Bezug ab»).".into()))?;
    let p_fit = parse_plan(&with_paths(base, req.paths, 0))?;
    let p_check = parse_plan(&with_paths(base, req.check_paths, 7919))?;
    let data = batch::load_series(&plan, base_dir, library)?;
    // Pfade der Suche einmal erzeugen, damit nicht jeder Kandidat sie neu berechnet.
    let fit = match batch::path_source(&p_fit, &sim, data.as_ref())? {
        Paths::List(v) => Paths::List(v),
        other => Paths::List(other.iter_owned().collect()),
    };
    let check = batch::path_source(&p_check, &sim, data.as_ref())?;
    let reference = Reference {
        strategy: strategy_name(OptStrategy::of(&plan.withdrawal)).into(),
        metrics: evaluate(&plan, &check, &obj)?,
    };
    let settings = Settings {
        samples: req.samples,
        seed: req.seed,
    };
    let mut outcomes = Vec::new();
    for s in strategies {
        outcomes.push(optimize_strategy(
            base,
            s,
            retire,
            PathSets {
                fit: &fit,
                check: &check,
            },
            &obj,
            &settings,
            &req.ranges,
        )?);
    }
    outcomes.sort_by(|a, b| rank(b, &obj).total_cmp(&rank(a, &obj)));
    Ok(Report {
        goal: req.goal,
        min_consumption: req.min_consumption,
        retire: retire.iso(),
        fit_paths: fit.len(),
        check_paths: check.len(),
        source: format!("{:?}", plan.returns.source).to_lowercase(),
        reference,
        outcomes,
        elapsed_ms: t0.elapsed().as_secs_f64() * 1e3,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_share_keeps_earlier_rows() {
        let mut s = json!({"mode": "timed", "rows": [
            {"from": "2026-03", "value": 0.3}, {"from": "2028-02", "value": 0.8}, {"from": "2030-01", "value": 0.5}]});
        set_from_retirement(&mut s, "2028-02", 0.6);
        assert_eq!(s["rows"].as_array().unwrap().len(), 2);
        assert_eq!(last_value(&s), Some(0.6));
        let mut n = json!(0.4);
        set_from_retirement(&mut n, "2030-01", 0.7);
        assert_eq!(n["rows"][0]["from"], "start");
        assert_eq!(last_value(&n), Some(0.7));
    }

    #[test]
    fn quantiles_and_fitness() {
        let v = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(quantile(&v, 0.5), 3.0);
        let o = Objective::default();
        let ok = Metrics {
            runs: 1,
            ruin: 0.01,
            median: 100.0,
            p10: 80.0,
            volatility: 0.0,
            draw_rate: 0.0,
            emergency_share: 0.0,
            planned: 0.0,
        };
        let bad = Metrics { ruin: 0.2, ..ok };
        assert!(ok.fitness(&o) > bad.fitness(&o));
        assert!(!bad.feasible(&o));
        assert!((ok.score(&o) - 90.0).abs() < 1e-9);
    }

    // ---- FEA-29 ----

    /// Erfundene Planung ohne Steuern, Teuerung und Renditen: Start 2026-11, Pensionierung 2031-11, Ende 2061-11,
    /// Ausgaben 1000 pro Monat, Vermögen nur als Bargeld.
    fn calm(cash: f64) -> Value {
        let mut v: Value = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
        // Bezug der Pensionskasse auch nach 65 (der Test verschiebt ihn); die Grenzen prüft FEA-38 anderswo.
        v["limits"] = json!({"pkMax": 130, "ahvMax": 130});
        v["returns"]["fixed"] = json!({"stocks": 0, "bonds": 0, "inflation": 0, "saron": 0});
        v["taxes"] = json!({"multiplier": 1, "income": [[0, 0]], "wealth": [[0, 0]], "capital": [[0, 0]]});
        v["flows"]["spending"] = json!(1000);
        v["wealth"] = json!({"stocks": 0, "bonds": 0, "cash": cash, "stockShare": 0, "reserveMonths": 0});
        v
    }

    fn run_calm(cash: f64) -> (Simulator, SimResult) {
        let plan = parse_plan(&calm(cash)).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let res = sim.run(&path, true);
        (sim, res)
    }

    fn metrics() -> Metrics {
        Metrics {
            runs: 1,
            ruin: 0.02,
            median: 100.0,
            p10: 80.0,
            volatility: 0.1,
            draw_rate: 0.0,
            emergency_share: 0.1,
            planned: 120.0,
        }
    }

    #[test]
    fn score_violation_and_fitness() {
        let m = metrics();
        let o = Objective {
            weight_median: 0.7,
            vol_weight: 2.0,
            ..Default::default()
        };
        // (0.7 × 100 + 0.3 × 80) × (1 − 2 × 0.1) = 94 × 0.8
        assert!((m.score(&o) - 75.2).abs() < 1e-9);
        assert!((m.fitness(&o) - 75.2 * (1.0 - 0.05 * 0.02)).abs() < 1e-9);
        let keep = Objective {
            goal: Goal::KeepProperty,
            min_consumption: 0.9,
            ..Default::default()
        };
        assert_eq!(m.score(&keep), -(0.1 * 1e6 + 0.02));
        let enough = Metrics { planned: 100.0, ..m };
        assert_eq!(enough.fitness(&keep), enough.score(&keep));
        // Median 100 von geplanten 120 ⇒ 0.9 − 0.8333 fehlen
        assert!((m.violation(&keep) - (0.9 - 100.0 / 120.0)).abs() < 1e-12);
        // Ruin 0.08 bei Grenze 0.05, Schwankung 0.15 bei Grenze 0.1
        let bad = Metrics {
            ruin: 0.08,
            volatility: 0.15,
            ..m
        };
        let o = Objective {
            max_ruin: 0.05,
            max_vol: Some(0.1),
            ..Default::default()
        };
        assert!((bad.violation(&o) - 0.08).abs() < 1e-12);
        assert!((bad.fitness(&o) - -1e9 * 1.08).abs() < 1e-3);
        // Eine Verletzung von genau 1e-12 gilt noch als erfüllt.
        let tiny = Metrics {
            ruin: 1e-12,
            volatility: 0.0,
            ..m
        };
        let o = Objective {
            max_ruin: 0.0,
            ..Default::default()
        };
        assert!(tiny.feasible(&o) && tiny.fitness(&o) > 0.0);
    }

    #[test]
    fn names_and_param_display() {
        let names: Vec<_> = all_strategies().into_iter().map(strategy_name).collect();
        assert_eq!(
            names,
            [
                "Standard",
                "3-Topf",
                "3-Topf + Guardrails",
                "Guardrails",
                "Fixe Rate",
                "VPW",
                "Endowment"
            ]
        );
        let defs = param_defs(OptStrategy::Standard);
        assert_eq!(defs[0].shown(), "Aktienanteil");
        assert_eq!(defs[1].shown(), "Reserve (Monate)");
    }

    #[test]
    fn ranges_are_checked() {
        let r = |lo: f64, hi: f64| -> Ranges {
            [(
                "fixed_rate".to_string(),
                [(
                    "rate".to_string(),
                    crate::model::ParamRange {
                        fixed: None,
                        lo: Some(lo),
                        hi: Some(hi),
                    },
                )]
                .into(),
            )]
            .into()
        };
        let (varied, _) = split_defs(OptStrategy::FixedRate, &r(0.04, 0.04)).unwrap();
        assert_eq!((varied[0].lo, varied[0].hi), (0.04, 0.04));
        assert!(split_defs(OptStrategy::FixedRate, &r(0.05, 0.04)).is_err());
        assert!(split_defs(OptStrategy::FixedRate, &r(0.04, f64::INFINITY)).is_err());
        assert!(split_defs(OptStrategy::FixedRate, &r(f64::NAN, 0.04)).is_err());
    }

    #[test]
    fn retirement_rows_and_current_values() {
        // Datumseinträge vor der Pensionierung und andere Verweise bleiben; «retire» und spätere Daten fallen weg.
        let mut s = json!({"mode": "timed", "rows": [
            {"from": "start", "value": 0.1}, {"from": "end", "value": 0.15}, {"from": "pension", "value": 0.2},
            {"from": "2026-03", "value": 0.3}, {"from": "retire", "value": 0.4}, {"from": "2030-01", "value": 0.5}]});
        set_from_retirement(&mut s, "2028-02", 0.6);
        let from: Vec<_> = s["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["from"].as_str().unwrap())
            .collect();
        assert_eq!(from, ["start", "end", "pension", "2026-03", "2028-02"]);

        let plan = json!({
            "wealth": {"stockShare": 0.45, "reserveMonths": {"mode": "timed", "rows": [{"from": "start", "value": 7}]}},
            "withdrawal": {"fixedRate": {"rate": 0.031}, "floor": 0.6, "ceiling": 1.3,
                "guardrails": {"initialRate": 0.041, "upper": 0.21, "lower": 0.22, "adjust": 0.11},
                "buckets": {"cashYears": 2.5, "bondYears": 5.5, "refillThreshold": 0.03},
                "vpw": {"horizonAge": 98, "realReturn": 0.015}, "endowment": {"rate": 0.035, "smoothing": 0.4}}
        });
        let expected = [
            ("stockShare", 0.45),
            ("reserveMonths", 7.0),
            ("rate", 0.031),
            ("floor", 0.6),
            ("ceiling", 1.3),
            ("initialRate", 0.041),
            ("upper", 0.21),
            ("lower", 0.22),
            ("adjust", 0.11),
            ("cashYears", 2.5),
            ("bondYears", 5.5),
            ("refillThreshold", 0.03),
            ("horizonAge", 98.0),
            ("realReturn", 0.015),
            ("eRate", 0.035),
            ("smoothing", 0.4),
        ];
        for (k, x) in expected {
            assert_eq!(current(&plan, k), Some(x), "{k}");
        }
        assert_eq!(current(&plan, "unbekannt"), None);

        use crate::model::{Buckets, Endowment, FixedRate, Guardrails, Vpw};
        let (b, g, v, e) = (
            Buckets::default(),
            Guardrails::default(),
            Vpw::default(),
            Endowment::default(),
        );
        let defaults = [
            ("cashYears", b.cash_years),
            ("bondYears", b.bond_years),
            ("refillThreshold", b.refill_threshold),
            ("initialRate", g.initial_rate),
            ("upper", g.upper),
            ("lower", g.lower),
            ("adjust", g.adjust),
            ("horizonAge", v.horizon_age),
            ("realReturn", v.real_return),
            ("eRate", e.rate),
            ("smoothing", e.smoothing),
            ("rate", FixedRate::default().rate),
        ];
        for (k, x) in defaults {
            assert_eq!(model_default(k), Some(x), "{k}");
        }
        assert_eq!(model_default("stockShare"), None);
    }

    #[test]
    fn set_writes_each_parameter() {
        let retire = "2031-11";
        // Fehlende Abschnitte entstehen mit Vorgabewerten.
        let mut p = json!({"wealth": {"stockShare": 0.5, "reserveMonths": 6}, "withdrawal": {}});
        for (k, x) in [
            ("stockShare", 0.7),
            ("reserveMonths", 9.0),
            ("rate", 0.03),
            ("floor", 0.8),
            ("ceiling", 1.2),
            ("upper", 0.3),
            ("bondYears", 4.0),
            ("realReturn", 0.01),
            ("eRate", 0.05),
        ] {
            set(&mut p, k, x, retire);
            if !["stockShare", "reserveMonths"].contains(&k) {
                assert_eq!(current(&p, k), Some(x), "{k}");
            }
        }
        assert_eq!(
            p["wealth"]["stockShare"]["rows"][1],
            json!({"from": retire, "value": 0.7})
        );
        assert_eq!(
            p["wealth"]["reserveMonths"]["rows"][1],
            json!({"from": retire, "value": 9.0})
        );
        assert_eq!(
            p["withdrawal"]["guardrails"],
            json!({"initialRate": 0.05, "upper": 0.3, "lower": 0.2, "adjust": 0.1})
        );
        assert_eq!(
            p["withdrawal"]["buckets"],
            json!({"cashYears": 2.0, "bondYears": 4.0, "refillThreshold": 0.0})
        );
        assert_eq!(p["withdrawal"]["vpw"], json!({"horizonAge": 100.0, "realReturn": 0.01}));
        assert_eq!(p["withdrawal"]["endowment"], json!({"rate": 0.05, "smoothing": 0.3}));
        // Vorhandene Abschnitte behalten ihre übrigen Werte.
        set(&mut p, "lower", 0.25, retire);
        set(&mut p, "cashYears", 3.0, retire);
        set(&mut p, "horizonAge", 95.0, retire);
        set(&mut p, "smoothing", 0.6, retire);
        assert_eq!(p["withdrawal"]["guardrails"]["upper"], json!(0.3));
        assert_eq!(p["withdrawal"]["buckets"]["bondYears"], json!(4.0));
        assert_eq!(p["withdrawal"]["vpw"]["realReturn"], json!(0.01));
        assert_eq!(p["withdrawal"]["endowment"], json!({"rate": 0.05, "smoothing": 0.6}));
        let before = p.clone();
        set(&mut p, "unbekannt", 1.0, retire);
        assert_eq!(p, before);
    }

    #[test]
    fn path_statistics() {
        // Viel Vermögen: jedes Jahr 12'000 Konsum, keine Schwankung; Entnahmerate 12'000 / Vermögen vor der Pensionierung.
        let (sim, res) = run_calm(1_000_000.0);
        let retire = sim.retire_at().unwrap();
        let st = path_stats(&sim, &res, retire, &Objective::default());
        assert!(!st.ruined && !st.emergency);
        assert_eq!(st.annual, vec![12_000.0; 30]);
        assert_eq!(st.planned, vec![12_000.0; 30]);
        assert_eq!(st.vol, 0.0);
        assert!((st.draw_rate - 12_000.0 / 940_000.0).abs() < 1e-12);

        // 100'000: ab Februar 2035 (Alter 68.25) aufgebraucht, danach zählt nur, was das Einkommen (0) deckt.
        let (sim, res) = run_calm(100_000.0);
        let st = path_stats(&sim, &res, retire, &Objective::default());
        assert!(st.ruined);
        assert_eq!(&st.annual[..5], &[12_000.0, 12_000.0, 12_000.0, 3_000.0, 0.0]);
        // Änderungen 0, 0, 0.75, 1 (Jahre mit 0 zählen nicht als Ausgangswert)
        assert!((st.vol - 1.75 / 4.0).abs() < 1e-12);
        assert!((st.draw_rate - 12_000.0 / 40_000.0).abs() < 1e-12);
        let ruin_until = |age: f64| {
            path_stats(
                &sim,
                &res,
                retire,
                &Objective {
                    ruin_age: Some(age),
                    ..Default::default()
                },
            )
            .ruined
        };
        assert!(ruin_until(68.25) && !ruin_until(68.0));

        // Ohne Vermögen bei der Pensionierung: Entnahmerate 0
        let (sim, res) = run_calm(60_000.0);
        assert_eq!(path_stats(&sim, &res, retire, &Objective::default()).draw_rate, 0.0);

        // Mit AHV (500) und Steuern im ersten Monat (Pensionierung im Mai 2032): Vermögen vor dem Monat aus dem Vormonat.
        let mut v = calm(500_000.0);
        v["pension"]["ahv"]["monthly"] = json!(500);
        v["taxes"]["income"] = json!([[0, 0.1]]);
        v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2032-05"});
        let sim = Simulator::new(&parse_plan(&v).unwrap()).unwrap();
        let res = sim.run(&crate::market::fixed_path(&sim.fixed, sim.start, sim.months()), true);
        let retire = sim.retire_at().unwrap();
        let k = res.records.iter().position(|r| r.month == retire).unwrap();
        assert!(res.records[k].taxes > 0.0 && res.records[k].income > 0.0);
        let st = path_stats(&sim, &res, retire, &Objective::default());
        assert!((st.draw_rate - 12.0 * 500.0 / res.records[k - 1].liquid).abs() < 1e-12);
    }

    #[test]
    fn path_statistics_in_real_terms_with_rent() {
        // Teuerung 2 %, Miete 500 (mit der Teuerung): Konsum und Plan jedes Jahr real 12 × (1000 + 500).
        let mut v = calm(1_000_000.0);
        v["returns"]["fixed"]["inflation"] = json!(0.02);
        v["flows"]["spendingIncludesHousing"] = json!(false);
        v["realestate"]["properties"] = json!([{"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0,
            "inflation": true, "from": "start", "rent": 500}]);
        let sim = Simulator::new(&parse_plan(&v).unwrap()).unwrap();
        let res = sim.run(&crate::market::fixed_path(&sim.fixed, sim.start, sim.months()), true);
        assert!(res.records.last().unwrap().price_index > 1.5);
        let st = path_stats(&sim, &res, sim.retire_at().unwrap(), &Objective::default());
        assert_eq!(st.annual.len(), 30);
        for (a, p) in st.annual.iter().zip(&st.planned) {
            assert!(
                (a / 18_000.0 - 1.0).abs() < 0.01 && (p / 18_000.0 - 1.0).abs() < 0.01,
                "{:?} {:?}",
                st.annual,
                st.planned
            );
        }
    }

    /// Zielfunktion, die die geprüften Punkte aufzeichnet.
    fn traced<'a>(f: impl Fn(f64) -> f64 + 'a, seen: &'a mut Vec<f64>) -> impl FnMut(&[f64]) -> Result<((), f64)> + 'a {
        move |x: &[f64]| {
            seen.push(x[0]);
            Ok(((), f(x[0])))
        }
    }

    #[test]
    fn search_takes_the_best_start_point() {
        // Plateaus ohne Steigung: Von einem Startpunkt aus kommt die Suche nicht weiter, der beste Start entscheidet.
        let plateau = |x: f64| {
            if x > 0.85 {
                2.0
            } else if x < 0.15 {
                1.0
            } else {
                0.0
            }
        };
        let mut seen = Vec::new();
        let starts = vec![vec![0.5], vec![0.9], vec![0.1], vec![0.95], vec![-3.0]];
        let (x, _, f) = search(starts, &[(0.0, 1.0)], traced(plateau, &mut seen)).unwrap();
        // Bei gleicher Fitness gilt der frühere Startpunkt; Startpunkte ausserhalb werden begrenzt.
        assert_eq!((x, f), (vec![0.9], 2.0));
        assert_eq!(&seen[..5], &[0.5, 0.9, 0.1, 0.95, 0.0]);
    }

    #[test]
    fn search_steps_and_directions() {
        // Ohne Verbesserung: je Achse +20 %, dann −20 % der Bandbreite (hier 0.5 bis 2.5), danach halbe Schritte bis 1.25 %.
        let bounds = [(0.5, 2.5)];
        let mut seen = Vec::new();
        let (x, _, _) = search(vec![vec![1.0]], &bounds, traced(|_| 0.0, &mut seen)).unwrap();
        assert_eq!(x, vec![1.0]);
        let expected = [0.4, -0.4, 0.2, -0.2, 0.1, -0.1, 0.05, -0.05, 0.025, -0.025];
        assert_eq!(seen.len(), 1 + expected.len(), "{seen:?}");
        for (v, d) in seen[1..].iter().zip(expected) {
            assert!((v - 1.0 - d).abs() < 1e-12, "{seen:?}");
        }

        // Die erste Verbesserung wird genommen: Zuerst geht es nach oben bis an die Grenze.
        let (x, _, _) = search(vec![vec![1.0]], &bounds, |x: &[f64]| Ok(((), (x[0] - 1.0).powi(2)))).unwrap();
        assert_eq!(x, vec![2.5]);

        // Ein Maximum innerhalb wird auf einen Schritt genau gefunden (in beide Richtungen).
        for peak in [0.87, 2.11] {
            let (x, _, _) = search(vec![vec![1.0]], &bounds, |x: &[f64]| Ok(((), -(x[0] - peak).powi(2)))).unwrap();
            assert!((x[0] - peak).abs() <= 0.025, "{x:?} statt {peak}");
        }
    }

    #[test]
    fn progress_estimate_of_the_search() {
        let req = Request {
            samples: 10,
            paths: 100,
            check_paths: 1000,
            ..Default::default()
        };
        // Fixe Rate: 10 Stichproben + Planung + Mitte + 14 Schritte, dazu Prüfung von Strategie und Planung.
        assert_eq!(
            progress_total(&req, &[OptStrategy::FixedRate]).unwrap(),
            26 * 100 + 2 * 1000
        );
        // Alle: 7 × 12 Startpunkte + 22 + 14 + 46 + 110 + 40 + 74 + 90 Schritte (FEA-05: 3-Topf + Guardrails)
        assert_eq!(
            progress_total(&req, &all_strategies()).unwrap(),
            (84 + 396) * 100 + 8 * 1000
        );
        // Ein festgehaltener Parameter von zweien: halbe Schrittzahl; alle fest: ein Lauf.
        let fix = |keys: &[&str]| -> Ranges {
            [(
                "fixed_rate".to_string(),
                keys.iter()
                    .map(|k| {
                        (
                            k.to_string(),
                            crate::model::ParamRange {
                                fixed: Some(true),
                                lo: None,
                                hi: None,
                            },
                        )
                    })
                    .collect(),
            )]
            .into()
        };
        let one = Request {
            ranges: fix(&["stockShare"]),
            ..req.clone()
        };
        assert_eq!(
            progress_total(&one, &[OptStrategy::FixedRate]).unwrap(),
            19 * 100 + 2 * 1000
        );
        let all = Request {
            ranges: fix(&["stockShare", "rate"]),
            ..req.clone()
        };
        assert_eq!(progress_total(&all, &[OptStrategy::FixedRate]).unwrap(), 100 + 2 * 1000);
    }

    fn emergency_plan() -> Value {
        let mut e = calm(20_000.0);
        e["realestate"]["emergencyMonths"] = json!(6);
        e["realestate"]["properties"] = json!([{"name": "Haus", "status": "owned", "price": 300000, "worth": 300000,
            "sell": "never", "fixCosts": 0, "inflation": false}]);
        e["taxes"]["propertyGain"] = json!({"brackets": [[0, 0]]});
        e
    }

    #[test]
    fn evaluate_averages_over_paths() {
        let plan = parse_plan(&calm(100_000.0)).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let paths = Paths::List(vec![path.clone(), path.clone(), path]);
        let m = evaluate(&plan, &paths, &Objective::default()).unwrap();
        assert_eq!((m.runs, m.ruin, m.emergency_share), (3, 1.0, 0.0));
        assert!((m.volatility - 0.4375).abs() < 1e-12);
        assert!((m.draw_rate - 0.3).abs() < 1e-12);
        assert_eq!(m.planned, 12_000.0);

        let plan = parse_plan(&emergency_plan()).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let res = sim.run(&path, true);
        assert!(path_stats(&sim, &res, sim.retire_at().unwrap(), &Objective::default()).emergency);
        let m = evaluate(
            &plan,
            &Paths::List(vec![path.clone(), path.clone(), path]),
            &Objective::default(),
        )
        .unwrap();
        assert_eq!(m.emergency_share, 1.0);
    }

    #[test]
    fn splitmix_values() {
        let mut r = Rng(0);
        assert_eq!(r.next(), 0.8833108082136426);
        assert_eq!(r.next(), 0.43152799704850997);
    }

    #[test]
    fn search_margin_and_start_points() {
        // 1,5 Standardfehler bei 100 Pfaden: 1.5 × √(0.05 × 0.95 / 100)
        let o = Objective::default();
        let f = fit_objective(&o, 100);
        assert!((f.max_ruin - (0.05 - 1.5 * (0.05f64 * 0.95 / 100.0).sqrt())).abs() < 1e-15);
        assert_eq!(fit_objective(&o, 1).max_ruin, 0.0);
        assert_eq!(fit_objective(&o, 0).max_ruin, 0.0);

        let defs = param_defs(OptStrategy::FixedRate)[..2].to_vec(); // Rate 2–8 %, Aktienanteil 10–100 %
        let plan = json!({"withdrawal": {"fixedRate": {"rate": 0.03}}, "wealth": {"stockShare": 1.5}});
        let st = Settings { samples: 20, seed: 4 };
        let s = start_points(&plan, &defs, &st);
        assert_eq!(s.len(), 22);
        // Wert der Planung (Aktienanteil ausserhalb ⇒ Mitte), dann die Mitte
        assert_eq!(s[0], vec![0.03, 0.55]);
        assert_eq!(s[1], vec![0.05, 0.55]);
        for x in &s[2..] {
            assert!((0.02..=0.08).contains(&x[0]) && (0.1..=1.0).contains(&x[1]), "{x:?}");
        }
        let spread = |i: usize| {
            s[2..].iter().map(|x| x[i]).fold(f64::INFINITY, f64::min) < s[2..].iter().map(|x| x[i]).fold(0.0, f64::max)
        };
        assert!(spread(0) && spread(1));
        assert_eq!(start_points(&plan, &[], &st).len(), 1);
        // Zufällige Startpunkte aus Rng(Saat XOR 0xA5A5_5A5A); Saat 10 überschneidet die Bits der Konstante.
        let s = start_points(&plan, &defs, &Settings { samples: 1, seed: 10 });
        let mut r = Rng(10 ^ 0xA5A5_5A5A);
        let at = |d: &ParamDef, u: f64| d.lo + (d.hi - d.lo) * u;
        assert_eq!(s[2], vec![at(&defs[0], r.next()), at(&defs[1], r.next())]);
    }

    // Die Mustersuche findet das Optimum der Entnahmerate (feines Raster zum Vergleich).
    #[test]
    fn search_finds_the_optimum() {
        let mut f = calm(400_000.0);
        f["withdrawal"]["strategy"] = json!("fixed_rate");
        let plan = parse_plan(&f).unwrap();
        let sim = Simulator::new(&plan).unwrap();
        let retire = sim.retire_at().unwrap();
        let path = crate::market::fixed_path(&sim.fixed, sim.start, sim.months());
        let paths = Paths::List(vec![path.clone(), path]);
        let ranges: Ranges = [(
            "fixed_rate".to_string(),
            [(
                "stockShare".to_string(),
                crate::model::ParamRange {
                    fixed: Some(true),
                    lo: None,
                    hi: None,
                },
            )]
            .into(),
        )]
        .into();
        let o = Objective::default();
        let out = optimize_strategy(
            &f,
            OptStrategy::FixedRate,
            retire,
            PathSets {
                fit: &paths,
                check: &paths,
            },
            &o,
            &Settings { samples: 3, seed: 1 },
            &ranges,
        )
        .unwrap();
        assert!(out.feasible);
        assert_eq!(out.params.len(), 1);
        assert_eq!(
            out.fixed.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(),
            ["stockShare", "floor", "ceiling"]
        );
        let rate = out.params[0].2;
        let fit = |r: f64| {
            let p = parse_plan(&with_params(
                &f,
                OptStrategy::FixedRate,
                &param_defs(OptStrategy::FixedRate)[..1],
                &[r],
                retire,
            ))
            .unwrap();
            evaluate(&p, &paths, &o).unwrap().fitness(&o)
        };
        let (best_r, best_f) = (0..=120)
            .map(|i| 0.02 + i as f64 * 0.0005)
            .map(|r| (r, fit(r)))
            .fold((0.0, f64::MIN), |a, b| if b.1 > a.1 { b } else { a });
        assert!((rate - best_r).abs() < 0.002, "Suche {rate}, Raster {best_r}");
        assert!(fit(rate) >= best_f - 1.0, "{} < {best_f}", fit(rate));
        assert_eq!(out.patch["withdrawal"]["fixedRate"]["rate"], json!(rate));
    }

    #[test]
    fn request_defaults_and_checks() {
        let r: Request = serde_json::from_value(json!({})).unwrap();
        assert_eq!((r.max_ruin, r.weight_median, r.vol_weight), (0.05, 0.5, 1.0));
        assert_eq!((r.paths, r.check_paths, r.samples, r.seed), (5000, 10_000, 30, 1));
        let r = Request {
            max_ruin: 0.1,
            weight_median: 0.6,
            vol_weight: 2.0,
            min_consumption: 0.8,
            ruin_age: Some(90.0),
            max_vol: Some(0.2),
            goal: Goal::KeepProperty,
            ..Default::default()
        };
        let o = r.objective();
        assert_eq!(
            (
                o.max_ruin,
                o.weight_median,
                o.vol_weight,
                o.min_consumption,
                o.ruin_age,
                o.max_vol,
                o.goal
            ),
            (0.1, 0.6, 2.0, 0.8, Some(90.0), Some(0.2), Goal::KeepProperty)
        );
        let list = |s: &[&str]| {
            Request {
                strategies: s.iter().map(|x| x.to_string()).collect(),
                ..Default::default()
            }
            .strategy_list()
            .unwrap()
            .len()
        };
        assert_eq!(
            (list(&[]), list(&["all"]), list(&["vpw", "all"]), list(&["vpw"])),
            (7, 7, 7, 1)
        );

        let dir = std::path::Path::new(".");
        let err = |base: &Value, r: Request| run_request(base, dir, None, &r).unwrap_err().to_string();
        let plain = calm(100_000.0);
        assert!(err(
            &plain,
            Request {
                check_paths: 0,
                ..Default::default()
            }
        )
        .contains("Anzahl Pfade"));
        assert!(err(
            &plain,
            Request {
                paths: 0,
                ..Default::default()
            }
        )
        .contains("Anzahl Pfade"));
        // Ziel 1 braucht keine Immobilie (die unbekannte Strategie beendet den Auftrag danach).
        let unknown = || Request {
            strategies: vec!["x".into()],
            ..Default::default()
        };
        assert!(err(&plain, unknown()).contains("Unbekannte Strategie"));
        // Ziel 2 braucht eine Immobilie, die keine Miete ist, und einen Mindestkonsum zwischen 0 und 200 %.
        let keep = |min: f64| Request {
            goal: Goal::KeepProperty,
            min_consumption: min,
            ..unknown()
        };
        assert!(err(&plain, keep(0.9)).contains("mindestens einer Immobilie"));
        let mut rent = calm(100_000.0);
        rent["realestate"]["properties"] = json!([{"name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "start", "rent": 1000}]);
        assert!(err(&rent, keep(0.9)).contains("mindestens einer Immobilie"));
        let owned = emergency_plan();
        assert!(err(&owned, keep(2.5)).contains("Mindestkonsum"));
        assert!(err(&owned, keep(0.9)).contains("Unbekannte Strategie"));
    }

    fn outcome(feasible: bool, ruin: f64) -> Outcome {
        let m = Metrics {
            ruin,
            volatility: 0.0,
            ..metrics()
        };
        Outcome {
            strategy: String::new(),
            params: vec![],
            fixed: vec![],
            fitted: m,
            validated: m,
            feasible,
            evaluations: 0,
            patch: Value::Null,
        }
    }

    #[test]
    fn ranking_of_outcomes() {
        let o = Objective::default();
        assert!((rank(&outcome(true, 0.01), &o) - outcome(true, 0.01).validated.score(&o)).abs() < 1e-12);
        // Unzulässig: −1e9 × (1 + Verletzung)
        assert!((rank(&outcome(false, 0.15), &o) - -1e9 * 1.1).abs() < 1e-3);
        assert!(rank(&outcome(false, 0.10), &o) > rank(&outcome(false, 0.15), &o));
    }

    #[test]
    fn report_time_in_milliseconds() {
        let mut v = calm(400_000.0);
        v["returns"]["source"] = json!("parametric");
        v["returns"]["parametric"] = json!({"paths": 2, "mu": 0.05, "sigma": 0.1, "df": 0});
        let r = Request {
            strategies: vec!["standard".into()],
            paths: 2,
            check_paths: 2,
            samples: 0,
            ranges: [(
                "standard".to_string(),
                [(
                    "reserveMonths".to_string(),
                    crate::model::ParamRange {
                        fixed: Some(true),
                        lo: None,
                        hi: None,
                    },
                )]
                .into(),
            )]
            .into(),
            ..Default::default()
        };
        let t = std::time::Instant::now();
        let rep = run_request(&v, std::path::Path::new("."), None, &r).unwrap();
        let outer_ms = t.elapsed().as_secs_f64() * 1e3;
        assert_eq!((rep.fit_paths, rep.check_paths, rep.outcomes.len()), (2, 2, 1));
        // Gemessene Zeit in Millisekunden, höchstens so lang wie der ganze Aufruf.
        assert!(
            rep.elapsed_ms > 0.01 && rep.elapsed_ms <= outer_ms,
            "{} > {outer_ms}",
            rep.elapsed_ms
        );
    }
}
