//! Das Datenmodell einer Planung (Projektdatei `.vplan`, JSON).
//!
//! Beträge sind monatlich, Sätze jährlich als Dezimalzahl (0.05 = 5 %), sofern nicht anders vermerkt.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::time::{TimeRef, YearMonth};

pub const FORMAT: &str = "vivaplan/1";
/// legacy: RetirementPlanner – Kennung älterer Dateien; beim Lesen gültig, geschrieben wird [`FORMAT`].
pub const LEGACY_FORMAT: &str = "pensionsplaner/1";

fn default_format() -> String {
    FORMAT.to_string()
}

/// #144: Meldung bei einer Formatkennung, die diese Version nicht kennt (derselbe Text wie `unknownFormatText` in
/// `crates/app/ui/util.js`).
pub fn unknown_format(format: &str) -> String {
    format!(
        "Die Datei hat das Format «{format}». Sie braucht eine neuere Version von VivaPlan und wird nicht geöffnet."
    )
}

/// #144: Prüft die Formatkennung einer Planung. Gültig sind [`FORMAT`], [`LEGACY_FORMAT`] und keine Kennung; jede andere
/// stammt aus einer neueren Version (oder ist keine Planung) und wird abgelehnt.
pub fn check_format(plan: &serde_json::Value) -> Result<(), crate::Error> {
    match plan.get("format") {
        None => Ok(()),
        Some(serde_json::Value::String(f)) if f == FORMAT || f == LEGACY_FORMAT => Ok(()),
        Some(serde_json::Value::String(f)) => Err(crate::Error::Invalid(unknown_format(f))),
        Some(other) => Err(crate::Error::Invalid(unknown_format(&other.to_string()))),
    }
}
fn one() -> f64 {
    1.0
}
fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    #[serde(default = "default_format")]
    pub format: String,
    pub project: Project,
    pub household: Household,
    pub milestones: Vec<Milestone>,
    pub pension: Pension,
    /// Zweite Person (Double Income): eigenes Einkommen, Pensionskasse und AHV.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner: Option<Partner>,
    pub flows: Flows,
    pub wealth: Wealth,
    pub returns: Returns,
    #[serde(default)]
    pub withdrawal: Withdrawal,
    pub taxes: Taxes,
    #[serde(default)]
    pub realestate: RealEstate,
    #[serde(default)]
    pub scenarios: Scenarios,
    /// Gespeicherte Einstellungen der Optimierung (Oberfläche und `optimize`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optimization: Option<OptimizerSettings>,
    /// Säule 3a, höchstens ein Konto je Person (#52).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pillar3a: Vec<Pillar3aAccount>,
    /// Anwendungsweite Grenzen aus den Einstellungen (#25). Die Oberfläche gibt sie bei jedem Aufruf mit; sie
    /// gehören nicht zur Planung und werden nie gespeichert. Fehlen sie, gelten die Standardwerte.
    #[serde(default, skip_serializing)]
    #[schemars(skip)]
    pub limits: crate::settings::Limits,
}

/// Ziel der Optimierung.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Goal {
    /// Ziel 1: Konsum bei begrenztem Ruinrisiko.
    #[default]
    Consumption,
    /// Ziel 2: Kein Notverkauf von Immobilien, also möglichst wenige ungeplante Verkäufe (Notverkäufe).
    KeepProperty,
}

/// Einstellungen der Optimierung. Anteile als Zahl (0.05 = 5 %); `strategies` leer = alle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct OptimizerSettings {
    /// Höchste erlaubte Ruinwahrscheinlichkeit.
    pub max_ruin: f64,
    /// Ziel der Optimierung (Standard: Konsum).
    pub goal: Goal,
    /// Ziel «Kein Notverkauf von Immobilien»: Der Median des Jahreskonsums beträgt mindestens diesen Anteil der geplanten Ausgaben.
    pub min_consumption: f64,
    /// Ruin zählt nur bis zu diesem Alter der Hauptperson (`None` = bis zum Simulationsende).
    pub ruin_age: Option<f64>,
    /// Gewicht des Medians im Konsumziel (0 bis 1); der Rest geht auf das 10-%-Perzentil.
    pub weight_median: f64,
    /// Abzug für die Schwankung des Konsums.
    pub vol_weight: f64,
    /// Höchste erlaubte Schwankung (`None` = keine Grenze).
    pub max_vol: Option<f64>,
    /// Pfade der Suche und der Prüfung, zufällige Startpunkte je Strategie.
    pub paths: usize,
    pub check_paths: usize,
    pub samples: usize,
    /// Schlüssel der Strategien (`guardrails`, `vpw` …).
    pub strategies: Vec<String>,
    /// Bandbreiten der Parameter je Strategie (Schlüssel der Strategie, dann des Parameters).
    /// Ohne Eintrag gelten die Standard-Bandbreiten.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub ranges: std::collections::BTreeMap<String, std::collections::BTreeMap<String, ParamRange>>,
}

/// Bandbreite eines Parameters in der Optimierung, in der Einheit des Parameters
/// (Jahre in Jahren, Anteile als Zahl: 0.05 = 5 %).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ParamRange {
    /// Der Parameter bleibt auf dem Wert der Planung und wird nicht variiert.
    pub fixed: Option<bool>,
    /// Untere und obere Grenze der Suche; ohne Angabe die Standard-Bandbreite.
    pub lo: Option<f64>,
    pub hi: Option<f64>,
}

impl Default for OptimizerSettings {
    fn default() -> Self {
        OptimizerSettings {
            max_ruin: 0.05,
            goal: Goal::Consumption,
            min_consumption: 0.9,
            ruin_age: None,
            weight_median: 0.5,
            vol_weight: 1.0,
            max_vol: None,
            paths: 5000,
            check_paths: 10_000,
            samples: 30,
            strategies: vec![],
            ranges: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    #[serde(default = "default_currency")]
    pub currency: String,
}
fn default_currency() -> String {
    "CHF".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Household {
    /// Erste Person = Hauptperson (`pension`, `flows.income`), zweite Person = `partner`.
    pub persons: Vec<Person>,
    /// Verheiratet: Ehepaar-Regeln (AHV-Plafonierung, Steuertabellen für Verheiratete).
    #[serde(default = "yes")]
    pub married: bool,
    /// Plafonierung der beiden AHV-Renten zusammen pro Monat (150 % der Maximalrente).
    #[serde(default = "default_couple_cap")]
    pub ahv_couple_cap: f64,
}

fn yes() -> bool {
    true
}
/// 150 % der AHV-Maximalrente von 2 520 CHF (Stand 2025).
fn default_couple_cap() -> f64 {
    3780.0
}

/// Einkommen und Vorsorge der zweiten Person; Altersangaben beziehen sich auf `household.persons[1]`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    /// Erwerbseinkommen netto pro Monat (steuerbar).
    #[serde(default)]
    pub income: Series,
    /// #58: Das Erwerbseinkommen steigt mit der Teuerung (Beträge in Preisen des Simulationsstarts); fehlt: nein.
    #[serde(default, skip_serializing_if = "is_false")]
    pub income_indexed: bool,
    pub pk: PensionFund,
    pub ahv: Ahv,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: String,
    pub name: String,
    pub birth: YearMonth,
}

/// Name des Meilensteins «Bezug Pensionskasse» (bisher «Pensionierung», #51).
pub const PK_NAME: &str = "Bezug Pensionskasse";
/// Name des Meilensteins «AHV-Bezug».
pub const AHV_NAME: &str = "AHV-Bezug";

/// Feste Meilensteine einer Person (#51): Ids für «Bezug Pensionskasse» und «AHV-Bezug». Erste Person `retire`
/// und `ahv`, zweite Person `retire2` und `ahv2`. Sie sind immer vorhanden und nicht löschbar.
pub fn person_milestones(index: usize) -> (&'static str, &'static str) {
    if index == 0 {
        ("retire", "ahv")
    } else {
        ("retire2", "ahv2")
    }
}

/// Anzeigename eines festen Meilensteins der Person `index` (zweite Person mit Zusatz).
pub fn person_milestone_name(base: &str, index: usize) -> String {
    if index == 0 {
        base.to_string()
    } else {
        format!("{base} Person 2")
    }
}

/// Benannter Zeitpunkt. `start` und `end` müssen vorhanden sein.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub kind: MilestoneKind,
    #[serde(default, skip_serializing_if = "is_false")]
    pub fixed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MilestoneKind {
    /// Fixes Datum.
    Date { date: YearMonth },
    /// Alter einer Person (Dezimaljahre, 64.5 = 64 Jahre 6 Monate).
    Age { person: String, years: f64 },
}

/// Ein Wert, der konstant ist oder sich ab bestimmten Zeitpunkten ändert.
/// Eine reine Zahl gilt als konstanter Wert.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Series {
    Number(f64),
    Def(SeriesDef),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum SeriesDef {
    Const { value: f64 },
    Timed { rows: Vec<SeriesRow> },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SeriesRow {
    pub from: TimeRef,
    pub value: f64,
}

impl Default for Series {
    fn default() -> Self {
        Series::Number(0.0)
    }
}

impl Series {
    pub fn constant(v: f64) -> Self {
        Series::Def(SeriesDef::Const { value: v })
    }
    pub fn timed(rows: Vec<(TimeRef, f64)>) -> Self {
        Series::Def(SeriesDef::Timed {
            rows: rows
                .into_iter()
                .map(|(from, value)| SeriesRow { from, value })
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pension {
    pub pk: PensionFund,
    pub ahv: Ahv,
}

/// Pensionskasse (2. Säule).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PensionFund {
    /// Altersguthaben bei Simulationsstart.
    pub capital: f64,
    /// Zeitpunkt des Bezugs (Rente und Kapital).
    pub at: TimeRef,
    /// Anteil Kapitalbezug 0..1.
    pub lumpsum: f64,
    /// Umwandlungssatz für den Rentenanteil.
    pub conversion: f64,
    /// Beiträge pro Monat bis zum Bezug.
    #[serde(default)]
    pub contribution: Series,
    /// Verzinsung pro Jahr, gutgeschrieben im Dezember.
    #[serde(default)]
    pub interest: Series,
}

/// AHV (1. Säule).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Ahv {
    /// Monatsrente bei Bezugsbeginn.
    pub monthly: f64,
    pub at: TimeRef,
    /// Jährliche Anpassung an die Inflation (im Dezember).
    #[serde(default)]
    pub indexed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Flows {
    /// Erwerbseinkommen netto pro Monat (steuerbar).
    #[serde(default)]
    pub income: Series,
    /// #58: Das Erwerbseinkommen steigt mit der Teuerung (Beträge in Preisen des Simulationsstarts); fehlt: nein.
    #[serde(default, skip_serializing_if = "is_false")]
    pub income_indexed: bool,
    /// Lebenshaltungskosten pro Monat in Preisen bei Simulationsstart.
    pub spending: Series,
    /// #53: Die Lebenshaltungskosten enthalten Miete bzw. Wohnkosten. Die Simulation zieht den Wohnanteil
    /// (Wohnkosten im Startmonat, siehe [`crate::housing`]) ab und rechnet mit den simulierten Wohnkosten. Fehlt das
    /// Feld (ältere Planungen), gelten die Lebenshaltungskosten ohne Wohnkosten.
    #[serde(default, skip_serializing_if = "is_false")]
    pub spending_includes_housing: bool,
    /// Faktor auf die Lebenshaltung ohne Wohnen (Variation «Ausgaben-Faktor»); fehlt: 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spending_factor: Option<f64>,
    /// Weitere Zuflüsse pro Monat (nicht steuerbar).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savings: Option<Series>,
    #[serde(default)]
    pub extras: Vec<Extra>,
}

/// Einmaliger Betrag; positiv = Zufluss.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub date: YearMonth,
    pub amount: f64,
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Wealth {
    pub stocks: f64,
    pub bonds: f64,
    pub cash: f64,
    /// Aktienanteil am investierten Vermögen 0..1.
    pub stock_share: Series,
    /// Cash-Reserve in Monaten der ungedeckten Ausgaben.
    #[serde(default)]
    pub reserve_months: Series,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReturnSource {
    Fixed,
    Historical,
    Rolling,
    Bootstrap,
    Parametric,
}

/// #69: Index für die Aktien der Anlagestrategie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StockIndex {
    /// Schweizer Aktien (JST), jährlich ab 1900.
    ChJst,
    /// Welt-Aktienindex (JST, nach BIP gewichtet), jährlich ab 1900.
    WorldJst,
    /// Welt mit eigenen Gewichten der Regionen (JST), jährlich ab 1900.
    WorldCustom,
    /// S&P 500 (Shiller) in CHF, monatlich ab 1922.
    Sp500,
}

/// #69: Index für die Obligationen der Anlagestrategie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BondIndex {
    /// 10-jährige Obligationen der Eidgenossenschaft (JST bis 1988, danach SNB).
    Ch,
    /// 10-jährige US-Staatsanleihen (Shiller) in CHF.
    Usa,
    /// #76: kurzfristige Papiere der Eidgenossenschaft (etwa 3 Monate): JST bis 2020, danach SARON.
    ChShort,
}

/// #69: Gewichte der Regionen für «Welt individuell» (Anteile, zusammen 1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegionWeights {
    pub ch: f64,
    pub usa: f64,
    /// Deutschland, Frankreich, Italien, Spanien, Niederlande, Belgien, Grossbritannien (unter sich nach BIP gewichtet).
    pub eu: f64,
    /// Japan und Australien (unter sich nach BIP gewichtet).
    pub apac: f64,
}

impl Default for RegionWeights {
    fn default() -> Self {
        RegionWeights {
            ch: 0.2,
            usa: 0.4,
            eu: 0.3,
            apac: 0.1,
        }
    }
}

/// #69: Anlagestrategie: woher die Renditen von Aktien und Obligationen stammen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Investment {
    pub stocks: StockIndex,
    pub bonds: BondIndex,
    /// Nur für [`StockIndex::WorldCustom`].
    #[serde(default)]
    pub weights: RegionWeights,
}

impl Default for Investment {
    fn default() -> Self {
        Investment {
            stocks: StockIndex::WorldJst,
            bonds: BondIndex::Ch,
            weights: RegionWeights::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Returns {
    pub source: ReturnSource,
    pub fixed: FixedReturns,
    /// #69: Anlagestrategie; gilt, wenn `series` fehlt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<Investment>,
    /// Zeitreihe (Pfad zur CSV-Datei, relativ zur Projektdatei); ältere Planungen (vor #69).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saron_series: Option<String>,
    /// Historischer Startmonat (Quelle `historical`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<YearMonth>,
    /// Fortschreibung, wenn die Zeitreihe vor dem Simulationsende endet.
    #[serde(default)]
    pub extrapolation: Extrapolation,
    /// Monte Carlo aus der Historie (Quelle `bootstrap`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<BootstrapParams>,
    /// Monte Carlo mit Aktienmodell (Quelle `parametric`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parametric: Option<ParametricParams>,
}

fn default_paths() -> usize {
    10_000
}
fn default_seed() -> u64 {
    42
}
fn default_block() -> f64 {
    12.0
}

/// Stationärer Bootstrap: Blöcke zufälliger Länge aus der Zeitreihe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapParams {
    #[serde(default = "default_paths")]
    pub paths: usize,
    /// Mittlere Blocklänge in Monaten.
    #[serde(default = "default_block")]
    pub block: f64,
    /// Startwert des Zufallsgenerators (gleicher Wert ⇒ gleiche Pfade).
    #[serde(default = "default_seed")]
    pub seed: u64,
}

impl Default for BootstrapParams {
    fn default() -> Self {
        BootstrapParams {
            paths: default_paths(),
            block: default_block(),
            seed: default_seed(),
        }
    }
}

/// Aktienmodell des bisherigen Programms: Student-t-Schocks und Kurssprünge, Tagesschritte. Mit einer Zeitreihe
/// (Anlagestrategie) sind Obligationen und Inflation zufällig und mit den Aktien korreliert (#101), sonst folgen sie
/// wie der SARON den fixen Werten.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParametricParams {
    #[serde(default = "default_paths")]
    pub paths: usize,
    /// Erwartete Jahresrendite (Drift).
    pub mu: f64,
    /// Jahresvolatilität.
    pub sigma: f64,
    /// Freiheitsgrade der Student-t-Verteilung (0 = Normalverteilung).
    pub df: f64,
    /// Erwartete Anzahl Kurssprünge pro Jahr.
    #[serde(default)]
    pub jump_lambda: f64,
    /// Mittlere logarithmische Sprunghöhe (−0.3 ≈ −26 %).
    #[serde(default)]
    pub jump_mu: f64,
    #[serde(default)]
    pub jump_sigma: f64,
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// #100: Die Modellparameter folgen den Vorschlagswerten des Aktienindex (nicht von Hand geändert). Fehlt das
    /// Merkmal (ältere Planungen), gelten die gespeicherten Werte.
    #[serde(default)]
    pub auto: bool,
    /// #132: Verfahren der Zufallsziehung, mit dem die Planung zuletzt gespeichert wurde. Fehlt es (0), stammt die Planung
    /// von vor #132, und `upgrade` nennt beim Öffnen die geänderte Rechnung. Gerechnet wird immer mit dem aktuellen
    /// Verfahren [`crate::montecarlo::SAMPLER`].
    #[serde(default)]
    pub sampler: u32,
}

impl Default for ParametricParams {
    fn default() -> Self {
        ParametricParams {
            paths: default_paths(),
            mu: 0.07,
            sigma: 0.15,
            df: 6.0,
            jump_lambda: 0.05,
            jump_mu: -0.3,
            jump_sigma: 0.15,
            seed: default_seed(),
            auto: false,
            sampler: crate::montecarlo::SAMPLER,
        }
    }
}

/// Jahreswerte. Im Modus `fixed` die Renditen; sonst Ersatzwerte (z. B. SARON ohne Zeitreihe).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FixedReturns {
    pub stocks: Series,
    pub bonds: Series,
    pub inflation: Series,
    #[serde(default)]
    pub saron: Series,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Extrapolation {
    pub method: ExtrapolationMethod,
    /// Anzahl Jahre für Durchschnitte.
    pub range: usize,
}

impl Default for Extrapolation {
    fn default() -> Self {
        Extrapolation {
            method: ExtrapolationMethod::Fixed,
            range: 30,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExtrapolationMethod {
    /// Fixe Werte aus `returns.fixed`.
    Fixed,
    /// Gleitender arithmetischer Durchschnitt.
    Average,
    /// Gleitender geometrischer Durchschnitt.
    Geometric,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    #[serde(default)]
    pub strategy: Strategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buckets: Option<Buckets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_rate: Option<FixedRate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrails: Option<Guardrails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vpw: Option<Vpw>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endowment: Option<Endowment>,
    /// Untergrenze der Lebenshaltung (Anteil der geplanten Ausgaben, 0 = keine); nur Guardrails und fixe Rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<f64>,
    /// Obergrenze der Lebenshaltung (Anteil der geplanten Ausgaben, 0 = keine).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling: Option<f64>,
}

/// Fixe Entnahmerate: ab Pensionierung wird jedes Jahr im Januar dieser Anteil des liquiden Vermögens
/// ausgegeben (Lebenshaltung + Wohnkosten − Einkommen und Renten, vor Steuern).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FixedRate {
    pub rate: f64,
}

impl Default for FixedRate {
    fn default() -> Self {
        FixedRate { rate: 0.04 }
    }
}

/// Guardrails nach Guyton-Klinger: Startentnahme in Prozent des Vermögens bei Pensionierung, danach
/// jährlich mit der Teuerung angepasst. Übersteigt die aktuelle Rate die Startrate um `upper`, wird um
/// `adjust` gekürzt; unterschreitet sie die Startrate um `lower`, wird um `adjust` erhöht.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Guardrails {
    pub initial_rate: f64,
    /// Obere Leitplanke: relative Abweichung der Rate nach oben (0.2 = 20 %).
    pub upper: f64,
    pub lower: f64,
    /// Anpassung der Ausgaben beim Erreichen einer Leitplanke (0.1 = 10 %).
    pub adjust: f64,
}

impl Default for Guardrails {
    fn default() -> Self {
        Guardrails {
            initial_rate: 0.05,
            upper: 0.2,
            lower: 0.2,
            adjust: 0.1,
        }
    }
}

/// Variable Prozententnahme (VPW): jedes Jahr im Januar wird der Anteil des liquiden Vermögens entnommen,
/// der es bis zum Planungsalter gleichmässig aufbraucht (Annuität mit der angenommenen Realrendite).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Vpw {
    /// Alter der Hauptperson, bis zu dem das Vermögen reichen soll.
    pub horizon_age: f64,
    /// Angenommene reale Jahresrendite des Vermögens (0.02 = 2 %).
    pub real_return: f64,
}

impl Default for Vpw {
    fn default() -> Self {
        Vpw {
            horizon_age: 100.0,
            real_return: 0.02,
        }
    }
}

/// Endowment-Regel (Yale/Tobin): Entnahme = (1 − `smoothing`) × Vorjahresentnahme mit Teuerung
/// + `smoothing` × `rate` × aktuelles Vermögen. Glättet Schwankungen des Vermögens.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Endowment {
    pub rate: f64,
    /// Gewicht des aktuellen Vermögens (0 = nur Vorjahr mit Teuerung, 1 = fixe Rate).
    pub smoothing: f64,
}

impl Default for Endowment {
    fn default() -> Self {
        Endowment {
            rate: 0.04,
            smoothing: 0.3,
        }
    }
}

/// 3-Topf-Strategie ab Pensionierung. Ziele in Jahren der ungedeckten Ausgaben (Ausgaben − Einkommen).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Buckets {
    /// Topf 1: Cash.
    pub cash_years: f64,
    /// Topf 2: Obligationen.
    pub bond_years: f64,
    /// Renditekriterium: Aus Topf 3 wird in der Januar-Auffüllung nur verkauft, wenn die Aktienrendite der letzten 12 Monate mindestens so hoch ist.
    #[serde(default)]
    pub refill_threshold: f64,
    /// Unterjährige Auffüllung von Topf 1 (Fall C): Nach den Ausgaben des Monats bleiben mindestens so viele Monatsbedarfe
    /// minus einer in Topf 1 (höchstens die Monate bis Januar).
    #[serde(default = "default_reserve_months")]
    pub reserve_months: f64,
    /// #6: Ausgaben ab Pensionierung wie geplant oder mit Guardrails gesteuert (Parameter aus `withdrawal.guardrails`,
    /// Grenzen aus `withdrawal.floor` und `withdrawal.ceiling`).
    #[serde(default, skip_serializing_if = "BucketSpending::is_planned")]
    pub spending: BucketSpending,
}

/// Ausgabenregel der 3-Topf-Strategie (#6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BucketSpending {
    /// Geplante Ausgaben.
    #[default]
    Planned,
    /// Guardrails nach Guyton-Klinger.
    Guardrails,
}

impl BucketSpending {
    pub fn is_planned(&self) -> bool {
        *self == BucketSpending::Planned
    }
}

fn default_reserve_months() -> f64 {
    3.0
}

impl Default for Buckets {
    fn default() -> Self {
        Buckets {
            cash_years: 2.0,
            bond_years: 6.0,
            refill_threshold: 0.0,
            reserve_months: 3.0,
            spending: BucketSpending::Planned,
        }
    }
}

impl Withdrawal {
    /// #6: 3-Topf-Strategie, deren Ausgaben die Guardrails steuern.
    pub fn buckets_with_guardrails(&self) -> bool {
        self.strategy == Strategy::ThreeBuckets
            && self
                .buckets
                .as_ref()
                .is_some_and(|b| b.spending == BucketSpending::Guardrails)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Cash-Reserve, Rest nach Aktienanteil investiert, monatliches Rebalancing.
    #[default]
    Standard,
    ThreeBuckets,
    Guardrails,
    FixedRate,
    Vpw,
    Endowment,
}

/// Steuertabellen: Paare `[ab Betrag, Satz]`, linear interpoliert, multipliziert mit `multiplier`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Taxes {
    /// Steuerfuss (Kanton + Gemeinde).
    #[serde(default = "one")]
    pub multiplier: f64,
    pub income: Vec<(f64, f64)>,
    pub wealth: Vec<(f64, f64)>,
    /// Kapitalbezüge (PK, 3a); Bezüge im selben Jahr werden zusammengerechnet.
    pub capital: Vec<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_gain: Option<PropertyGainTax>,
    /// Tabellen für Verheiratete; gelten, wenn die Planung eine zweite Person hat und `household.married`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub couple: Option<TaxTables>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxTables {
    pub income: Vec<(f64, f64)>,
    pub wealth: Vec<(f64, f64)>,
    pub capital: Vec<(f64, f64)>,
}

/// Modell der Grundstückgewinnsteuer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GainModel {
    /// Stufentarif auf den Gewinn, dann Zu- oder Abschlag nach Besitzdauer (Kanton Zürich).
    #[default]
    Zurich,
    /// Direkter Steuersatz auf den Gewinn nach Besitzdauer (Kanton Aargau, § 109 StG).
    Aargau,
}

/// Grundstückgewinnsteuer. Modell Zürich: Stufen `[Breite der Stufe, Satz]`, danach Zu- oder
/// Abschlag nach Besitzdauer. Modell Aargau: Satz je Besitzdauer. Die Regeln unterscheiden sich je Kanton.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropertyGainTax {
    #[serde(default)]
    pub model: GainModel,
    /// Modell Aargau: Paare `[ab vollen Jahren, Steuersatz auf den Gewinn]`. Es gilt die letzte Zeile,
    /// deren Jahre erreicht sind. Ohne Kaufdatum gilt der höchste Satz.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duration_rates: Vec<(f64, f64)>,
    /// Die Steuer zusätzlich mit dem Steuerfuss (`taxes.multiplier`) multiplizieren.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub use_multiplier: bool,
    #[serde(default)]
    pub brackets: Vec<(f64, f64)>,
    /// Feste Ermässigung der Steuer, unabhängig von der Besitzdauer (0.5 = 50 %).
    #[serde(default)]
    pub reduction: f64,
    /// Zu- und Abschläge nach Besitzdauer: Paare `[ab vollen Jahren, Änderung der Steuer]`
    /// (0.5 = Zuschlag von 50 %, −0.2 = Abzug von 20 %). Es gilt die letzte Zeile, deren Jahre
    /// erreicht sind. Ohne Kaufdatum der Immobilie wird nichts angepasst.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duration: Vec<(f64, f64)>,
    /// Gewinne unter diesem Betrag bleiben steuerfrei (0 = keine Freigrenze).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub exempt_below: f64,
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RealEstate {
    /// Notverkauf, wenn das liquide Vermögen die ungedeckten Ausgaben dieser Anzahl Monate nicht mehr deckt.
    #[serde(default)]
    pub emergency_months: Series,
    #[serde(default)]
    pub properties: Vec<Property>,
    /// Tragbarkeit für Käufe «nach Verkauf».
    #[serde(default)]
    pub affordability: Affordability,
}

/// Tragbarkeitsrechnung der Banken: (kalkulatorischer Zins × Hypothek + Nebenkosten) höchstens
/// `sustainability` × Jahreseinkommen (Erwerb und Renten, ohne Entnahme aus dem Vermögen); Hypothek höchstens 80 %.
/// Reicht das nicht, wird die Hypothek gekürzt (Einmalrückzahlung aus dem Vermögen).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Affordability {
    pub sustainability: f64,
    pub imputed_rate: f64,
    /// Nebenkosten pro Jahr in Prozent des Kaufpreises.
    pub fix_costs: f64,
    /// Nicht mehr verwendet (die Tragbarkeit rechnet nur mit dem Einkommen); bleibt für ältere Dateien erhalten.
    pub wealth_contribution: f64,
    pub max_loan_to_value: f64,
}

impl Default for Affordability {
    fn default() -> Self {
        Affordability {
            sustainability: 1.0 / 3.0,
            imputed_rate: 0.05,
            fix_costs: 0.01,
            wealth_contribution: 0.03,
            max_loan_to_value: 0.8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PropertyStatus {
    Owned,
    Planned,
    Rent,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    /// Eindeutige Kennung, damit andere Objekte darauf verweisen können («nach Verkauf von …»).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub status: PropertyStatus,
    /// Kaufpreis (Besitz: bezahlter Preis, für die Grundstückgewinnsteuer).
    #[serde(default)]
    pub price: f64,
    /// Marktwert heute (nur Besitz).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worth: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy: Option<TimeRef>,
    /// Kaufdatum (Besitz): bestimmt die Besitzdauer bei der Grundstückgewinnsteuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bought: Option<YearMonth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell: Option<TimeRef>,
    /// Miete ab (Status `rent`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<TimeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<TimeRef>,
    /// Miete pro Monat in Preisen bei Simulationsstart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rent: Option<f64>,
    #[serde(default)]
    pub fix_costs: f64,
    /// Wert und Nebenkosten mit der Inflation anpassen (älteres Feld; gilt, wo die Einzelwahl fehlt).
    #[serde(default)]
    pub inflation: bool,
    /// Wert mit der Inflation anpassen (fehlt: `inflation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflation_value: Option<bool>,
    /// Nebenkosten mit der Inflation anpassen (fehlt: `inflation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflation_fix_costs: Option<bool>,
    /// Miete mit der Inflation anpassen (fehlt: ja).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflation_rent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mortgage: Option<Mortgage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Mortgage {
    /// Hypothek bei Kauf: erfasste Beträge oder maximale Belehnung (Minimum Eigenmittel).
    #[serde(default)]
    pub amount_mode: AmountMode,
    pub tranches: Vec<Tranche>,
    /// Veraltet (vor #52): Säule 3a an der Hypothek. Wird beim Laden in [`Plan::pillar3a`] übernommen
    /// (siehe [`crate::upgrade`]) und danach nicht mehr verwendet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pillar3a: Option<Pillar3a>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AmountMode {
    /// Beträge der Tranchen wie erfasst.
    #[default]
    Fixed,
    /// Höchste Belehnung (`affordability.maxLoanToValue`, also Minimum an Eigenmitteln), gekürzt,
    /// falls nicht tragbar. Die Beträge der Tranchen gelten dann als Anteile.
    MaxLoan,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AfterLast {
    /// Letzte Periode zu gleichen Konditionen verlängern.
    #[default]
    Renew,
    /// Restschuld aus dem Vermögen zurückzahlen.
    Repay,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tranche {
    #[serde(default)]
    pub name: String,
    /// Betrag heute (Besitz) bzw. beim Kauf (geplant).
    pub amount: f64,
    /// Beginn der laufenden Periode (nur Besitz).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<YearMonth>,
    #[serde(default)]
    pub after: AfterLast,
    pub periods: Vec<Period>,
    /// Person (Id), in deren Säule 3a die indirekte Amortisation fliesst (#52); fehlt: erste Person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pillar3a: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RateKind {
    #[default]
    Fixed,
    Saron,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AmortMode {
    #[default]
    Direct,
    /// Einzahlung in die Säule 3a statt Reduktion der Schuld.
    Indirect,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    #[serde(rename = "type", default)]
    pub kind: RateKind,
    /// Laufzeit in Jahren.
    pub term: f64,
    /// Fester Zins p.a. (Art `fixed`).
    #[serde(default)]
    pub rate: f64,
    /// Marge p.a. auf den SARON (Art `saron`).
    #[serde(default)]
    pub margin: f64,
    /// Amortisation pro Quartal (fällig März, Juni, September, Dezember).
    #[serde(default)]
    pub amort: f64,
    #[serde(default)]
    pub amort_mode: AmortMode,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PayoutUse {
    #[default]
    Repay,
    Wealth,
}

/// Veraltete Säule 3a an der Hypothek (vor #52); nur noch zum Laden älterer Dateien.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3a {
    pub balance: f64,
    pub rate: f64,
    pub payout: TimeRef,
    #[serde(rename = "use", default)]
    pub usage: PayoutUse,
}

fn wealth_use() -> PayoutUse {
    PayoutUse::Wealth
}

/// Säule 3a einer Person (#52): Guthaben bei Simulationsstart, Einzahlung und Auszahlungsplan. Der Rest wird
/// spätestens im Monat des Meilensteins «AHV-Bezug» der Person ausbezahlt.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aAccount {
    /// Person (Id aus `household.persons`).
    pub person: String,
    /// Guthaben bei Simulationsstart.
    #[serde(default)]
    pub balance: f64,
    /// Verzinsung bzw. Rendite pro Jahr.
    #[serde(default)]
    pub rate: f64,
    /// Einzahlung pro Jahr (0 bis Maximalbetrag), monatlich ein Zwölftel, bis vor dem AHV-Bezug. Zählt zu den Ausgaben.
    #[serde(default)]
    pub contribution: f64,
    /// Geplante Auszahlungen, frühestens 5 Jahre vor dem AHV-Bezug.
    #[serde(default)]
    pub payouts: Vec<Pillar3aPayout>,
    /// Verwendung des Rests im Monat des AHV-Bezugs.
    #[serde(rename = "use", default = "wealth_use")]
    pub usage: PayoutUse,
}

/// Zeitpunkt einer 3a-Auszahlung (#55): fixes Datum oder Alter der Kontoinhaberin bzw. des Kontoinhabers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PayoutAt {
    #[default]
    Date,
    Age,
}

impl PayoutAt {
    fn is_date(&self) -> bool {
        *self == PayoutAt::Date
    }
}

/// Geplante Auszahlung aus der Säule 3a.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aPayout {
    /// «date» (Vorgabe, auch für ältere Planungen) oder «age».
    #[serde(default, skip_serializing_if = "PayoutAt::is_date")]
    pub kind: PayoutAt,
    /// Monat der Auszahlung bei `kind = date`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<YearMonth>,
    /// Alter in Dezimaljahren bei `kind = age` (61.5 = 61 Jahre 6 Monate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub years: Option<f64>,
    /// Betrag; fehlt: das ganze Guthaben.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(rename = "use", default = "wealth_use")]
    pub usage: PayoutUse,
}

impl Pillar3aPayout {
    /// Monat der Auszahlung; `None`, wenn Datum bzw. Alter fehlt oder das Alter nicht zwischen 0 und 150 Jahren liegt.
    pub fn month(&self, birth: Option<YearMonth>) -> Option<YearMonth> {
        match self.kind {
            PayoutAt::Date => self.date,
            PayoutAt::Age => {
                let y = self.years.filter(|y| (0.0..=150.0).contains(y))?;
                Some(birth?.plus((y * 12.0).round() as i32))
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Scenarios {
    #[serde(default)]
    pub sweeps: Vec<serde_json::Value>,
}

impl Plan {
    pub fn from_json(text: &str) -> Result<Plan, crate::Error> {
        let v: serde_json::Value =
            serde_json::from_str(text).map_err(|e| crate::Error::Invalid(format!("Projektdatei: {e}")))?;
        check_format(&v)?;
        serde_json::from_value(v).map_err(|e| crate::Error::Invalid(format!("Projektdatei: {e}")))
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Plan ist immer serialisierbar")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const EXAMPLE: &str = include_str!("../assets/Neu.vplan");

    // #40: Vorgabewerte beim Lesen und knappe Ausgabe beim Schreiben.
    #[test]
    fn defaults_when_reading() {
        let mut v: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
        v.as_object_mut().unwrap().remove("format");
        let plan = Plan::from_json(&v.to_string()).unwrap();
        assert_eq!(plan.format, FORMAT);

        let b: BootstrapParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(
            b,
            BootstrapParams {
                paths: 10_000,
                block: 12.0,
                seed: 42
            }
        );
        assert_eq!(BootstrapParams::default(), b);
        let p: ParametricParams = serde_json::from_value(json!({"mu": 0.07, "sigma": 0.15, "df": 6.0})).unwrap();
        assert_eq!((p.paths, p.seed), (10_000, 42));
        let d = ParametricParams::default();
        assert_eq!((d.paths, d.seed), (10_000, 42));
    }

    #[test]
    fn fixed_milestone_is_written_only_when_set() {
        let mut m = Milestone {
            id: "m".into(),
            name: "M".into(),
            kind: MilestoneKind::Date {
                date: YearMonth::new(2030, 1),
            },
            fixed: false,
        };
        assert!(serde_json::to_value(&m).unwrap().get("fixed").is_none());
        m.fixed = true;
        assert_eq!(serde_json::to_value(&m).unwrap()["fixed"], json!(true));
    }

    #[test]
    fn series_constructors() {
        assert_eq!(
            serde_json::to_value(Series::constant(2.5)).unwrap(),
            json!({"mode": "const", "value": 2.5})
        );
        assert_eq!(
            serde_json::to_value(Series::timed(vec![(TimeRef("2030-01".into()), 1.5)])).unwrap(),
            json!({"mode": "timed", "rows": [{"from": "2030-01", "value": 1.5}]})
        );
    }

    // #124/AK7: Eine Auszahlung nach Alter behält beim Speichern ihre Art; die Art «date» (Vorgabe) wird weggelassen.
    #[test]
    fn issue_124_ak7_payout_kind_is_saved_only_for_age() {
        let age: Pillar3aPayout = serde_json::from_value(json!({"kind": "age", "years": 61.5})).unwrap();
        let date: Pillar3aPayout = serde_json::from_value(json!({"date": "2040-06"})).unwrap();
        assert_eq!(serde_json::to_value(&age).unwrap()["kind"], json!("age"));
        assert!(serde_json::to_value(&date).unwrap().get("kind").is_none());
    }
}
