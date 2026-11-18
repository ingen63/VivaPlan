//! Die Monatssimulation.
//!
//! Reihenfolge pro Kalendermonat:
//! 1. Teuerung: jeden Monat mit (1 + Inflation)^(1/12) (Ausgaben, Mieten, Immobilienwerte); die AHV
//!    folgt im Januar mit der seit dem letzten Januar aufgelaufenen Teuerung.
//! 2. Ereignisse: einmalige Beträge, Pensionierung (Kapitalbezug, Rente), AHV-Beginn, Immobilien
//!    (Notverkauf aus dem Vormonat, Kauf oder Miete nach Verkauf, Kauf, Hypothekarperioden,
//!    Amortisation, Verkauf), Säule 3a.
//! 3. Geldflüsse: Einkommen, Renten, Zuflüsse minus Lebenshaltung und Wohnkosten → Cash.
//!    Die Lebenshaltung enthält weder Miete noch Hypothekarzinsen; Zinsen laufen monatlich auf und
//!    werden mit der Amortisation am Quartalsende (März, Juni, September, Dezember) bezahlt.
//! 4. Rendite auf Aktien und Obligationen (sofern das Portfolio positiv ist).
//! 5. Pensionskasse: Beiträge; im Dezember Verzinsung.
//! 6. Dezember: Einkommens- und Vermögenssteuer des Jahres veranlagen; bezahlt im Mai des Folgejahres
//!    (eine offene Steuer wird im letzten Planmonat beglichen).
//! 7. Notverkauf prüfen (wird im Folgemonat ausgeführt).
//! 8. Entnahmestrategie: «Standard» (Cash-Reserve, Rest nach Aktienanteil, monatlich) oder ab
//!    Pensionierung «3-Topf» (Cash, Obligationen, Aktien; Nachfüllen im Januar).
//!
//! Zweite Person: Einkommen, Pensionskasse und AHV laufen pro Person; Steuern, Vermögen und
//! Ausgaben gelten für den Haushalt. Die Entnahmestrategien (ausser «Standard») beginnen, sobald die erste
//! Person mit Bezugsdatum pensioniert ist. Sind beide Renten der AHV fällig, werden sie bei Verheirateten
//! auf `household.ahvCoupleCap` plafoniert.
//!
//! Fixe Entnahmerate, Guardrails, VPW und Endowment bestimmen ab der Pensionierung der ersten Person die
//! Lebenshaltung: jährliche Entnahme = Lebenshaltung + Wohnkosten − Einkommen und Renten (vor Steuern). Ein für einen
//! Kauf reservierter Verkaufserlös zählt nicht zum Vermögen der Entnahme. Bei «3-Topf» mit Guardrails (#6) steuern
//! die Guardrails die Ausgaben und die Töpfe die Anlage; der Netto-Bedarf der Töpfe rechnet dann mit der
//! Guardrails-Lebenshaltung.

use serde::Serialize;

use crate::market::MarketPath;
use crate::model::{
    Affordability, AfterLast, AmortMode, AmountMode, Buckets, PayoutUse, Plan, Property, PropertyStatus, RateKind,
    ReturnSource, Strategy, Tranche,
};
use crate::resolve::{Curve, Resolver};
use crate::tax;
use crate::time::YearMonth;
use crate::validate::{self, Severity};
use crate::{Error, Result};

/// Feste Cash-Reserve (Monate) für alle Strategien ausser «Standard».
pub const FIXED_RESERVE_MONTHS: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventKind {
    Extra,
    Retirement,
    AhvStart,
    Purchase,
    RentStart,
    Sale,
    EmergencySale,
    MortgageRenewal,
    MortgageRepayment,
    /// Wechsel von indirekter auf direkte Amortisation im Monat des AHV-Bezugs (#50).
    AmortizationSwitch,
    Pillar3aPayout,
    Buckets,
    Withdrawal,
    Depleted,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub month: YearMonth,
    pub kind: EventKind,
    pub text: String,
}

/// #133: Geldflüsse und Bewertungsänderungen eines Monats. Die Veränderung des Gesamtvermögens gegenüber dem Vormonat
/// ist genau [`Ledger::sum`]; alle übrigen Buchungen (Anlagen kaufen und verkaufen, Cash nachfüllen, Hypothek tilgen,
/// Säule 3a einzahlen und auszahlen, Liegenschaft kaufen und verkaufen) sind Umbuchungen und ändern das Total nicht.
/// Nur intern (Prüfung in den Tests), nicht im Ergebnis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ledger {
    /// Erwerbseinkommen, PK-Rente und AHV.
    pub income: f64,
    /// Einmalbeträge (positiv: Zufluss, negativ: Ausgabe).
    pub extras: f64,
    /// Sparbeträge aus der Planung.
    pub savings: f64,
    /// Lebenshaltung (Abfluss, positiv gezählt).
    pub living: f64,
    /// Wohnkosten wie bezahlt: Miete, Nebenkosten, Hypothekarzins, Amortisation (Abfluss, positiv gezählt).
    pub housing: f64,
    /// Amortisation in den Wohnkosten: senkt die Hypothek oder erhöht die Säule 3a, bleibt also im Vermögen.
    pub amortization: f64,
    /// Alle bezahlten Steuern (Abfluss, positiv gezählt).
    pub taxes: f64,
    /// Rendite von Aktien und Obligationen.
    pub market: f64,
    /// Beiträge an die Pensionskasse.
    pub pk_contributions: f64,
    /// Zinsgutschrift der Pensionskasse (im Dezember und beim Bezug).
    pub pk_interest: f64,
    /// Zins der Säule 3a.
    pub p3a_interest: f64,
    /// Wertänderung der Liegenschaften (Teuerung).
    pub revaluation: f64,
    /// In eine Rente umgewandeltes PK-Kapital (verlässt das Vermögen, positiv gezählt).
    pub annuity: f64,
}

impl Ledger {
    /// Veränderung des Gesamtvermögens nach dem Hauptbuch.
    pub fn sum(&self) -> f64 {
        self.income + self.extras + self.savings - self.living - self.housing + self.amortization - self.taxes
            + self.market
            + self.pk_contributions
            + self.pk_interest
            + self.p3a_interest
            + self.revaluation
            - self.annuity
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthRecord {
    pub month: YearMonth,
    pub age: f64,
    pub cash: f64,
    pub stocks: f64,
    pub bonds: f64,
    pub liquid: f64,
    pub property_equity: f64,
    pub pension_fund: f64,
    pub pillar3a: f64,
    pub total: f64,
    /// Einzahlung in die Säule 3a in diesem Monat (zählt zu den Ausgaben, #52).
    pub pillar3a_in: f64,
    /// Einkommen inkl. Renten.
    pub income: f64,
    /// Lebenshaltung inkl. Wohnkosten.
    pub spending: f64,
    pub taxes: f64,
    /// Aufschlüsselung der Steuern des Monats: Einkommen, Vermögen, Kapitalbezug (PK, 3a), Grundstückgewinn.
    pub tax_income: f64,
    pub tax_wealth: f64,
    pub tax_capital: f64,
    pub tax_gain: f64,
    /// #109: Im Dezember veranlagte Einkommens- und Vermögenssteuer des Steuerjahres (bezahlt wird sie im Mai des
    /// Folgejahres, siehe `tax_income`/`tax_wealth`). Die Jahreswerte zeigen sie in der Zeile des Steuerjahres.
    pub assessed_income: f64,
    pub assessed_wealth: f64,
    /// Geplante Lebenshaltung aus der Planung (nominal, ohne Wohnkosten).
    pub planned_spending: f64,
    /// Entnahme aus dem Vermögen: tatsächliche Lebenshaltung und Wohnkosten minus Einkommen (vor Steuern).
    /// Bei Guardrails & Co. legt die Strategie sie fest, sonst folgt sie den geplanten Ausgaben.
    pub withdrawal: f64,
    /// Aufschlüsselung: Erwerbseinkommen, PK-Rente, AHV, Wohnkosten (Zins, Amortisation, Nebenkosten, Miete).
    pub earned: f64,
    pub pk_pension: f64,
    pub ahv: f64,
    pub housing: f64,
    /// Marktwert der Immobilien und Hypotheken.
    pub property_value: f64,
    pub mortgage: f64,
    /// Teuerungsindex seit Start (1 = Preise des Startmonats); Wert / Index = heutige Kaufkraft.
    pub price_index: f64,
    /// Wertentwicklung von Aktien und Obligationen seit Start (1 = Startmonat), nominal.
    pub stock_index: f64,
    pub bond_index: f64,
    /// #133: Hauptbuch des Monats (nur intern, nicht im Ergebnis).
    #[serde(skip)]
    pub ledger: Ledger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEventKind {
    Purchase,
    Sale,
    EmergencySale,
    /// Erneuerung einer Hypothekarperiode (auch mit Einmalzahlung oder nicht erneuert wegen Notverkauf).
    Renewal,
}

/// Immobilienereignis mit den Beträgen: Kauf, Verkauf, Notverkauf und Hypothekenerneuerung in einer Liste.
/// Nicht zutreffende Beträge fehlen.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEvent {
    pub month: YearMonth,
    pub kind: PropertyEventKind,
    pub property: String,
    /// Kauf: Kaufpreis, Eigenmittel (Preis − Hypothek) und Hypothek.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mortgage: Option<f64>,
    /// Verkauf, Notverkauf: Verkaufserlös (Marktwert), abgelöste Hypothek, Grundstückgewinnsteuer, Nettoerlös.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proceeds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_repaid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gain_tax: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net: Option<f64>,
    /// Erneuerung: Tragbarkeit, Hypothek vorher und nachher, Einmalzahlung, Zinssatz der neuen Periode (Durchschnitt der Tranchen).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affordable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_after: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lump_sum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
    /// Notverkauf: Grund («Vermögen < 0» oder «Notverkaufskriterium»); bei einer Erneuerung der Grund, warum sie nicht erfolgt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Verkauf: was mit dem Erlös geschieht («Verkaufserlös reserviert für Kauf im Folgemonat» bzw. Entnahmestrategie sofort angewendet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl PropertyEvent {
    fn new(month: YearMonth, kind: PropertyEventKind, property: &str) -> Self {
        PropertyEvent {
            month,
            kind,
            property: property.to_string(),
            price: None,
            equity: None,
            mortgage: None,
            proceeds: None,
            debt_repaid: None,
            gain_tax: None,
            net: None,
            affordable: None,
            debt_before: None,
            debt_after: None,
            lump_sum: None,
            rate: None,
            reason: None,
            note: None,
        }
    }
}

/// Grundlage der Topfgrössen bei der 3-Topf-Strategie: Netto-Bedarf pro Jahr = Lebenshaltung + Wohnen +
/// Einkommenssteuer − regelmässige Einnahmen (AHV, PK-Rente, Erwerbseinkommen). Nicht enthalten: einmalige Ausgaben,
/// Amortisationen, Immobilienkäufe, Sondersteuern (Grundstückgewinn-, Kapitalbezugs-, Vermögenssteuer).
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeedBasis {
    /// Bezugsjahr der Werte.
    pub year: i32,
    /// «Januar, geplant» oder «ab <Monat>, auf 12 Monate hochgerechnet».
    pub source: String,
    /// Herkunft der Lebenshaltung: «geplant» oder «Guardrails» (#6).
    pub living_source: String,
    pub living: f64,
    pub housing: f64,
    /// Einkommenssteuer auf dem (hochgerechneten) Jahreseinkommen.
    pub income_tax: f64,
    /// AHV-Rente.
    pub ahv: f64,
    /// PK-Rente.
    pub pk: f64,
    /// Erwerbseinkommen.
    pub earned: f64,
    /// Einzahlungen in die Säule 3a (#52).
    pub pillar3a: f64,
    /// Anzahl Monate, aus denen die Werte hochgerechnet wurden (12 = volles Jahr).
    pub months: u32,
}

impl NeedBasis {
    pub fn expenses(&self) -> f64 {
        self.living + self.housing + self.income_tax + self.pillar3a
    }
    /// Regelmässige Einnahmen: Renten und Erwerbseinkommen.
    pub fn receipts(&self) -> f64 {
        self.ahv + self.pk + self.earned
    }
    /// Netto-Bedarf pro Jahr; nie negativ.
    pub fn need(&self) -> f64 {
        (self.expenses() - self.receipts()).max(0.0)
    }
}

/// Umbuchung zwischen den Töpfen (1 = Cash, 2 = Obligationen, 3 = Aktien; 0 = von ausserhalb, z. B. ein Zufluss).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketMove {
    pub from: u8,
    pub to: u8,
    pub amount: f64,
}

/// Eine Anwendung der 3-Topf-Strategie (Januar-Auffüllung, Sonderfall A, B oder C) mit Herleitung, Renditekriterium,
/// Umbuchungen und Topfständen.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketRefill {
    pub month: YearMonth,
    /// Beschreibung des Anlasses.
    pub reason: String,
    /// Ausgelöster Fall: «Januar», «A», «B» oder «C».
    pub trigger: String,
    /// Bei A: bereits vergangene Monate des Jahres, um die die Zielgrössen reduziert sind (sonst 0).
    pub passed_months: u32,
    pub basis: NeedBasis,
    pub need: f64,
    pub target1: f64,
    pub target2: f64,
    /// Aktienrendite für die Renditebedingung (letzte 12 Monate, mit dem Vorlauf vor dem Start) und ihre Grundlage.
    pub stock_return: f64,
    pub return_basis: String,
    pub threshold: f64,
    /// Aktienrendite ≥ Schwelle (Verkäufe aus Topf 3 sind erlaubt). Bei A und C wird das Kriterium ignoriert.
    pub condition_met: bool,
    /// Das Kriterium wurde ignoriert (A, B und C).
    pub condition_ignored: bool,
    pub moves: Vec<BucketMove>,
    /// Ausgaben dieses Monats (laufende und einmalige), die nach der Auffüllung aus Topf 1 bezahlt werden.
    pub payment: f64,
    /// Stand Topf 1 (Cash), Topf 2 (Obligationen), Topf 3 (Aktien) vor und nach der Auffüllung, jeweils vor den
    /// Ausgaben des Monats.
    pub before: [f64; 3],
    pub after: [f64; 3],
}

/// Prüfung der Tragbarkeit bei der Erneuerung einer Hypothekarperiode.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Renewal {
    pub month: YearMonth,
    pub property: String,
    /// Die Hypothek war tragbar (keine Massnahme nötig).
    pub affordable: bool,
    /// Einmalzahlung aus dem Vermögen (0, wenn keine nötig oder keine möglich war).
    pub lump_sum: f64,
    pub debt_before: f64,
    /// Hypothek nach der Erneuerung (bei Notverkauf unverändert, die Immobilie wird im Folgemonat verkauft).
    pub debt_after: f64,
    /// Grund des Notverkaufs: «Vermögen < 0» oder «Notverkaufskriterium»; `None`: kein Notverkauf.
    pub emergency_sale: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    /// Notverkäufe in diesem Lauf (aus beliebigem Grund).
    pub emergency_sales: usize,
    /// Einmalzahlungen wegen fehlender Tragbarkeit bei Erneuerungen.
    pub lump_sums: usize,
    /// Notverkäufe wegen fehlender Tragbarkeit bei Erneuerungen.
    pub affordability_sales: usize,
    pub end_total: f64,
    pub end_liquid: f64,
    /// Gesamtvermögen und liquides Vermögen zum Simulationsstart (vor dem ersten Monat), nominal.
    pub start_total: f64,
    pub start_liquid: f64,
    /// Erster Monat ohne liquides Vermögen und ohne Immobilien.
    pub depleted: Option<YearMonth>,
    pub total_at_retirement: Option<f64>,
    pub min_liquid: f64,
    pub min_liquid_month: Option<YearMonth>,
    pub pension_fund_monthly: f64,
    pub lumpsum_net: f64,
    pub lumpsum_tax: f64,
    pub taxes_total: f64,
    /// Gesamtvermögen am Ende in Preisen des Startmonats.
    pub end_total_real: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimResult {
    pub label: String,
    pub summary: Summary,
    pub events: Vec<Event>,
    /// Alle Erneuerungen von Hypothekarperioden mit dem Ergebnis der Tragbarkeitsprüfung.
    pub renewals: Vec<Renewal>,
    /// Alle Immobilienereignisse (Kauf, Verkauf, Notverkauf, Erneuerung) in zeitlicher Reihenfolge.
    pub property_events: Vec<PropertyEvent>,
    /// Anwendungen der 3-Topf-Strategie mit Herleitung (leer bei anderen Strategien).
    pub bucket_refills: Vec<BucketRefill>,
    /// Nur gefüllt, wenn mit `record = true` gerechnet wurde.
    pub records: Vec<MonthRecord>,
    /// Auszahlungsplan der Säule 3a je Konto (#55); nur mit `record = true`. Nicht Teil der gespeicherten
    /// Ergebnisse, die Vorschau gibt ihn eigens weiter.
    #[serde(skip)]
    pub pillar3a: Vec<Pillar3aView>,
}

/// Säule 3a eines Kontos in der Vorschau (#55).
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aView {
    pub person: String,
    /// Guthaben jedes Monats ab Simulationsstart vor den Auszahlungen dieses Monats, bis zum AHV-Bezug.
    pub balances: Vec<f64>,
    pub payouts: Vec<Pillar3aPayoutView>,
    /// Auszahlung des Rests im AHV-Bezug.
    pub rest: Option<Pillar3aRest>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aPayoutView {
    /// Zeile im Auszahlungsplan der Planung.
    pub index: usize,
    pub month: YearMonth,
    pub before: f64,
    pub paid: f64,
    pub after: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aRest {
    pub month: YearMonth,
    pub amount: f64,
}

// ---------------------------------------------------------------------------------------------
// Kompilierte Planung
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SPeriod {
    start: YearMonth,
    end: YearMonth,
    index: usize,
    kind: RateKind,
    rate: f64,
    margin: f64,
    amort_q: f64,
    direct: bool,
}

impl SPeriod {
    fn rate(&self, saron: f64) -> f64 {
        match self.kind {
            RateKind::Fixed => self.rate,
            RateKind::Saron => saron.max(0.0) + self.margin,
        }
    }
}

#[derive(Clone, Debug)]
struct CTranche {
    name: String,
    amount: f64,
    periods: Vec<SPeriod>,
    repay_at: Option<YearMonth>,
    /// Person (Index), in deren Säule 3a die indirekte Amortisation fliesst (#52).
    account: usize,
}

/// Zeitpunkt, der sich aus einem Verkauf während der Simulation ergibt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trigger {
    None,
    /// Nach dem Verkauf der letzten eigenen Immobilie.
    LastSale,
    /// Nach dem Verkauf einer bestimmten Immobilie (Index).
    SaleOf(usize),
}

/// Säule 3a einer Person (#52).
#[derive(Clone, Debug, Default)]
struct CAccount {
    /// Zusatz für Ereignistexte bei zwei Personen.
    who: String,
    balance: f64,
    rate_m: f64,
    /// Einzahlung pro Monat (ein Zwölftel der Jahreseinzahlung), bis vor dem AHV-Bezug.
    contribution_m: f64,
    /// Person (Id), für die Anzeige des Auszahlungsplans.
    person: String,
    /// Geplante Auszahlungen: Monat, Betrag (fehlt: ganzes Guthaben), Hypothek zurückzahlen, Zeile im Plan.
    payouts: Vec<(YearMonth, Option<f64>, bool, usize)>,
    /// AHV-Bezug: Ende der Einzahlungen und der indirekten Amortisation, Auszahlung des Rests.
    ahv: Option<YearMonth>,
    rest_repay: bool,
}

#[derive(Clone, Debug)]
struct CProp {
    name: String,
    status: PropertyStatus,
    price: f64,
    bought: Option<YearMonth>,
    worth: f64,
    infl_value: bool,
    infl_fix_costs: bool,
    infl_rent: bool,
    fix_costs: f64,
    buy: Option<YearMonth>,
    sell: Option<YearMonth>,
    rent_from: Option<YearMonth>,
    rent_until: Option<YearMonth>,
    rent: f64,
    /// Kauf bzw. Miete «nach Verkauf».
    buy_trigger: Trigger,
    rent_trigger: Trigger,
    /// Hypothek bei Kauf: höchste Belehnung statt erfasster Beträge.
    max_loan: bool,
    /// Perioden, wenn der Kaufzeitpunkt im Voraus feststeht (Besitz, geplanter Kauf mit Datum).
    tranches: Vec<CTranche>,
    /// Definition für Käufe «nach Verkauf»; die Perioden entstehen erst beim Kauf.
    tranche_defs: Vec<Tranche>,
}

/// Einkommen und Vorsorge einer Person.
struct PersonCfg {
    birth: YearMonth,
    income: Curve,
    /// #58: Einkommen in Preisen des Starts, mit dem Preisindex des Pfades fortgeschrieben.
    income_indexed: bool,
    pk_contribution: Curve,
    pk_interest: Curve,
    pk_capital: f64,
    pk_lumpsum: f64,
    pk_conversion: f64,
    pk_at: Option<YearMonth>,
    ahv_monthly: f64,
    ahv_at: Option<YearMonth>,
    ahv_indexed: bool,
}

#[derive(Clone, Copy, Debug)]
enum Rule {
    Fixed {
        rate: f64,
    },
    Guardrails {
        initial: f64,
        upper: f64,
        lower: f64,
        adjust: f64,
    },
    Vpw {
        horizon_age: f64,
        real_return: f64,
    },
    Endowment {
        rate: f64,
        smoothing: f64,
    },
}

/// Anteil des Vermögens, der es in `years` Jahren bei Realrendite `r` gleichmässig aufbraucht.
fn vpw_rate(r: f64, years: f64) -> f64 {
    let n = years.max(1.0);
    let rate = if r.abs() < 1e-9 {
        1.0 / n
    } else {
        r / (1.0 - (1.0 + r).powf(-n))
    };
    rate.clamp(0.0, 1.0)
}

/// Fixe Entnahmerate oder Guardrails.
#[derive(Clone, Copy, Debug)]
struct Dynamic {
    rule: Rule,
    floor: Option<f64>,
    ceiling: Option<f64>,
}

pub struct Simulator {
    currency: String,
    pub start: YearMonth,
    pub end: YearMonth,
    months: usize,
    /// Hauptperson, danach (falls vorhanden) die zweite Person.
    people: Vec<PersonCfg>,
    /// Plafonierung der AHV-Renten eines Ehepaars pro Monat.
    couple_cap: Option<f64>,
    spending: Curve,
    /// #53: Wohnanteil der erfassten Lebenshaltungskosten (Preise des Startmonats) und Faktor der Variation.
    housing_share: f64,
    spending_factor: f64,
    savings: Curve,
    stock_share: Curve,
    reserve: Curve,
    emergency: Curve,
    stocks0: f64,
    bonds0: f64,
    cash0: f64,
    extras: Vec<(YearMonth, f64, String)>,
    props: Vec<CProp>,
    /// Säule 3a je Person (Index wie `people`).
    accounts: Vec<CAccount>,
    /// Ids der Personen (für die Zuordnung der Tranchen zur Säule 3a).
    person_ids: Vec<String>,
    taxes: crate::model::Taxes,
    pub fixed: crate::market::FixedCurves,
    pub source: ReturnSource,
    afford: Affordability,
    /// 3-Topf-Strategie ab Pensionierung.
    buckets: Option<Buckets>,
    dynamic: Option<Dynamic>,
}

impl Simulator {
    pub fn new(plan: &Plan) -> Result<Self> {
        // Ältere Säule 3a an der Hypothek wie beim Laden übernehmen (#52).
        let normalized = crate::upgrade::normalized(plan);
        let plan: &Plan = &normalized;
        let errors: Vec<String> = validate::validate(plan)
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| i.message)
            .collect();
        if !errors.is_empty() {
            return Err(Error::Invalid(errors.join("\n")));
        }
        let w = &plan.withdrawal;
        let buckets = (w.strategy == Strategy::ThreeBuckets).then(|| w.buckets.clone().unwrap_or_default());
        let dynamic = match w.strategy {
            Strategy::FixedRate => Some(Rule::Fixed {
                rate: w.fixed_rate.clone().unwrap_or_default().rate,
            }),
            // #6: Bei 3-Topf mit Guardrails steuern die Guardrails die Ausgaben, die Töpfe die Anlage.
            _ if w.strategy == Strategy::Guardrails || w.buckets_with_guardrails() => {
                let g = w.guardrails.clone().unwrap_or_default();
                Some(Rule::Guardrails {
                    initial: g.initial_rate,
                    upper: g.upper,
                    lower: g.lower,
                    adjust: g.adjust,
                })
            }
            Strategy::Vpw => {
                let v = w.vpw.clone().unwrap_or_default();
                Some(Rule::Vpw {
                    horizon_age: v.horizon_age,
                    real_return: v.real_return,
                })
            }
            Strategy::Endowment => {
                let e = w.endowment.clone().unwrap_or_default();
                Some(Rule::Endowment {
                    rate: e.rate,
                    smoothing: e.smoothing,
                })
            }
            _ => None,
        }
        .map(|rule| Dynamic {
            rule,
            floor: w.floor.filter(|f| *f > 0.0),
            ceiling: w.ceiling.filter(|c| *c > 0.0),
        });
        let r = Resolver::new(plan)?;
        let start = r.milestone("start")?;
        let end = r.milestone("end")?;
        let curve = |s: &crate::model::Series| Curve::compile(s, &r);
        let person_cfg = |birth: YearMonth,
                          income: &crate::model::Series,
                          income_indexed: bool,
                          pk: &crate::model::PensionFund,
                          ahv: &crate::model::Ahv|
         -> Result<PersonCfg> {
            Ok(PersonCfg {
                birth,
                income: curve(income)?,
                income_indexed,
                pk_contribution: curve(&pk.contribution)?,
                pk_interest: curve(&pk.interest)?,
                pk_capital: pk.capital,
                pk_lumpsum: pk.lumpsum,
                pk_conversion: pk.conversion,
                pk_at: r.resolve(&pk.at)?,
                ahv_monthly: ahv.monthly,
                ahv_at: r.resolve(&ahv.at)?,
                ahv_indexed: ahv.indexed,
            })
        };
        let persons = &plan.household.persons;
        let mut people = vec![person_cfg(
            persons[0].birth,
            &plan.flows.income,
            plan.flows.income_indexed,
            &plan.pension.pk,
            &plan.pension.ahv,
        )?];
        if let (Some(q), Some(p2)) = (&plan.partner, persons.get(1)) {
            people.push(person_cfg(p2.birth, &q.income, q.income_indexed, &q.pk, &q.ahv)?);
        }
        let couple = people.len() > 1 && plan.household.married;
        let mut taxes = plan.taxes.clone();
        if let (true, Some(c)) = (couple, &plan.taxes.couple) {
            taxes.income = c.income.clone();
            taxes.wealth = c.wealth.clone();
            taxes.capital = c.capital.clone();
        }
        let props = plan
            .realestate
            .properties
            .iter()
            .map(|p| compile_property(p, plan, &r, end))
            .collect::<Result<Vec<_>>>()?;
        // Säule 3a: ein Konto je Person (ohne Angabe leer), damit die indirekte Amortisation immer ein Ziel hat.
        let accounts = persons
            .iter()
            .take(people.len())
            .enumerate()
            .map(|(i, person)| {
                let who = if people.len() > 1 {
                    format!(" ({})", person.name)
                } else {
                    String::new()
                };
                let ahv = r.milestone(crate::model::person_milestones(i).1).ok();
                match plan.pillar3a.iter().find(|a| a.person == person.id) {
                    None => CAccount {
                        who,
                        ahv,
                        person: person.id.clone(),
                        ..Default::default()
                    },
                    Some(a) => {
                        // #55: Zeitpunkt als Datum oder Alter der Kontoinhaberin bzw. des Kontoinhabers.
                        let mut payouts: Vec<_> = a
                            .payouts
                            .iter()
                            .enumerate()
                            .filter_map(|(j, x)| {
                                Some((
                                    x.month(Some(person.birth))?.max(start),
                                    x.amount,
                                    x.usage == PayoutUse::Repay,
                                    j,
                                ))
                            })
                            .collect();
                        payouts.sort_by_key(|x| (x.0, x.3));
                        CAccount {
                            who,
                            person: person.id.clone(),
                            balance: a.balance,
                            rate_m: crate::market::monthly(a.rate),
                            contribution_m: a.contribution / 12.0,
                            payouts,
                            ahv,
                            rest_repay: a.usage == PayoutUse::Repay,
                        }
                    }
                }
            })
            .collect();
        Ok(Simulator {
            currency: plan.project.currency.clone(),
            start,
            end,
            months: (start.until(end) + 1) as usize,
            people,
            couple_cap: couple.then_some(plan.household.ahv_couple_cap),
            spending: curve(&plan.flows.spending)?,
            housing_share: if plan.flows.spending_includes_housing {
                crate::housing::start_housing(plan, &r)
            } else {
                0.0
            },
            spending_factor: plan.flows.spending_factor.unwrap_or(1.0),
            savings: plan
                .flows
                .savings
                .as_ref()
                .map(curve)
                .transpose()?
                .unwrap_or(Curve::constant(0.0)),
            stock_share: curve(&plan.wealth.stock_share)?,
            // Einstellbar nur bei «Standard»; sonst feste Annahme (#18).
            reserve: if w.strategy == Strategy::Standard {
                curve(&plan.wealth.reserve_months)?
            } else {
                Curve::constant(FIXED_RESERVE_MONTHS)
            },
            emergency: curve(&plan.realestate.emergency_months)?,
            stocks0: plan.wealth.stocks,
            bonds0: plan.wealth.bonds,
            cash0: plan.wealth.cash,
            // Beträge vor dem Start werden im Startmonat gebucht.
            extras: plan
                .flows
                .extras
                .iter()
                .map(|x| (x.date.max(start), x.amount, x.label.clone()))
                .collect(),
            props,
            accounts,
            person_ids: persons.iter().map(|p| p.id.clone()).collect(),
            taxes,
            fixed: crate::market::FixedCurves {
                stocks: curve(&plan.returns.fixed.stocks)?,
                bonds: curve(&plan.returns.fixed.bonds)?,
                inflation: curve(&plan.returns.fixed.inflation)?,
                saron: curve(&plan.returns.fixed.saron)?,
            },
            source: plan.returns.source,
            afford: plan.realestate.affordability.clone(),
            buckets,
            dynamic,
        })
    }

    /// Geplante Lebenshaltung ohne Wohnkosten im Monat `m` in Preisen des Startmonats (#53): erfasster Betrag
    /// abzüglich Wohnanteil, mal Faktor der Variation.
    fn planned_living(&self, m: YearMonth) -> f64 {
        (self.spending.at(m) - self.housing_share) * self.spending_factor
    }

    /// Erwerbseinkommen des Haushalts in diesem Monat; `infl_idx` ist der Preisindex des Pfades seit dem Start. Steigt
    /// das Einkommen einer Person mit der Teuerung (#58), gilt ihr Betrag in Preisen des Starts.
    fn earned(&self, m: YearMonth, infl_idx: f64) -> f64 {
        self.people
            .iter()
            .map(|p| p.income.at(m) * if p.income_indexed { infl_idx } else { 1.0 })
            .sum()
    }

    /// Letzter Bezug einer Pensionskasse (Pensionierung des Haushalts).
    pub fn retire_at(&self) -> Option<YearMonth> {
        self.people.iter().filter_map(|p| p.pk_at).max()
    }

    pub fn months(&self) -> usize {
        self.months
    }

    /// Geplante Einzahlungen in die Säule 3a im Monat `m` (bis vor dem AHV-Bezug der Person).
    fn pillar3a_contribution(&self, m: YearMonth) -> f64 {
        self.accounts
            .iter()
            .filter(|a| a.ahv.is_none_or(|h| m < h))
            .map(|a| a.contribution_m)
            .sum()
    }

    pub fn age_at(&self, m: YearMonth) -> f64 {
        self.people[0].birth.until(m) as f64 / 12.0
    }

    pub fn run(&self, path: &MarketPath, record: bool) -> SimResult {
        assert!(
            path.stocks.len() >= self.months,
            "Renditepfad ist kürzer als der Simulationszeitraum"
        );
        Run::new(self, path, record).execute()
    }
}

fn compile_property(p: &Property, plan: &Plan, r: &Resolver, end: YearMonth) -> Result<CProp> {
    let res = |t: &Option<crate::time::TimeRef>| -> Result<Option<YearMonth>> {
        Ok(match t {
            Some(t) => r.resolve(t)?,
            None => None,
        })
    };
    let buy = res(&p.buy)?;
    let sell = res(&p.sell)?;
    let mut tranches = vec![];
    if p.status != PropertyStatus::Rent {
        if let Some(mg) = &p.mortgage {
            let limit = match p.status {
                PropertyStatus::Owned => sell.map_or(end.plus(1), |s| s.min(end.plus(1))),
                _ => end.plus(1),
            };
            for t in &mg.tranches {
                let first = match p.status {
                    PropertyStatus::Owned => t.start.ok_or_else(|| {
                        Error::Invalid(format!("{}: Beginn der laufenden Hypothekarperiode fehlt", p.name))
                    })?,
                    _ => match buy {
                        Some(b) => b,
                        None => continue,
                    },
                };
                let mut c = schedule(t, first, limit);
                c.account = account_of(t, plan);
                tranches.push(c);
            }
        }
    }
    let trigger = |t: &Option<crate::time::TimeRef>| -> Result<Trigger> {
        let Some(t) = t else { return Ok(Trigger::None) };
        if t.is_after_sale() {
            return Ok(Trigger::LastSale);
        }
        match t.sale_of() {
            None => Ok(Trigger::None),
            Some(id) => plan
                .realestate
                .properties
                .iter()
                .position(|q| q.id.as_deref() == Some(id))
                .map(Trigger::SaleOf)
                .ok_or_else(|| Error::Invalid(format!("{}: Immobilie «{id}» gibt es nicht", p.name))),
        }
    };
    Ok(CProp {
        buy_trigger: if p.status == PropertyStatus::Planned {
            trigger(&p.buy)?
        } else {
            Trigger::None
        },
        rent_trigger: if p.status == PropertyStatus::Rent {
            trigger(&p.from)?
        } else {
            Trigger::None
        },
        max_loan: p
            .mortgage
            .as_ref()
            .is_some_and(|m| m.amount_mode == AmountMode::MaxLoan),
        tranche_defs: p.mortgage.as_ref().map_or(vec![], |m| m.tranches.clone()),
        name: p.name.clone(),
        status: p.status,
        price: p.price,
        bought: p.bought,
        worth: p.worth.unwrap_or(p.price),
        infl_value: p.inflation_value.unwrap_or(p.inflation),
        infl_fix_costs: p.inflation_fix_costs.unwrap_or(p.inflation),
        infl_rent: p.inflation_rent.unwrap_or(true),
        fix_costs: p.fix_costs,
        buy,
        sell,
        rent_from: res(&p.from)?,
        rent_until: res(&p.until)?,
        rent: p.rent.unwrap_or(0.0),
        tranches,
    })
}

/// Person (Index), in deren Säule 3a die indirekte Amortisation einer Tranche fliesst; ohne Angabe die erste Person.
fn account_of(t: &Tranche, plan: &Plan) -> usize {
    t.pillar3a
        .as_deref()
        .and_then(|id| plan.household.persons.iter().position(|p| p.id == id))
        .unwrap_or(0)
        .min(1)
}

/// Perioden einer Tranche ab `first` bis vor `limit`; nach der letzten definierten Periode wird
/// verlängert (`renew`) oder die Restschuld am Ende zurückbezahlt (`repay`).
fn schedule(t: &crate::model::Tranche, first: YearMonth, limit: YearMonth) -> CTranche {
    let mut periods = vec![];
    let mut m = first;
    let mut i = 0;
    while m < limit && !t.periods.is_empty() && i < 1000 {
        if i >= t.periods.len() && t.after == AfterLast::Repay {
            break;
        }
        let d = &t.periods[i.min(t.periods.len() - 1)];
        let end = m.plus(((d.term * 12.0).round() as i32).max(1));
        periods.push(SPeriod {
            start: m,
            end,
            index: i,
            kind: d.kind,
            rate: d.rate,
            margin: d.margin,
            amort_q: d.amort,
            direct: d.amort_mode == AmortMode::Direct,
        });
        m = end;
        i += 1;
    }
    let repay_at = match (t.after, periods.last()) {
        (AfterLast::Repay, Some(last)) if last.end < limit => Some(last.end),
        _ => None,
    };
    CTranche {
        name: t.name.clone(),
        amount: t.amount,
        periods,
        repay_at,
        account: 0,
    }
}

// ---------------------------------------------------------------------------------------------
// Ein Lauf
// ---------------------------------------------------------------------------------------------

struct PropState {
    active: bool,
    done: bool,
    price_paid: f64,
    /// Kaufmonat (für die Besitzdauer); bei Besitz ohne Datum unbekannt.
    bought: Option<YearMonth>,
    balances: Vec<f64>,
    emergency_pending: bool,
    /// Grund des eingeleiteten Notverkaufs.
    emergency_reason: Option<String>,
    /// Perioden eines Kaufs «nach Verkauf».
    sched: Option<Vec<CTranche>>,
    /// Monat des Verkaufs.
    sold: Option<YearMonth>,
    /// Kauf «nach Verkauf von …» wurde versucht.
    attempted: bool,
    /// Aufgelaufene, noch nicht bezahlte Hypothekarzinsen (fällig am Quartalsende).
    int_accr: f64,
}

struct Run<'a> {
    s: &'a Simulator,
    path: &'a MarketPath,
    record: bool,
    cash: f64,
    stocks: f64,
    bonds: f64,
    people: Vec<PersonState>,
    /// AHV-Teuerung seit dem ersten Bezug (für die Plafonierung bei Ehepaaren).
    ahv_scale: f64,
    /// Steuern aus Ereignissen des Monats (3a-Auszahlung, Grundstückgewinn).
    event_tax: f64,
    infl_idx: f64,
    /// Teuerungsindex beim letzten Januar bzw. Start (Basis der jährlichen Anpassungen).
    infl_idx_jan: f64,
    /// Bei einem Verkauf mitten im Quartal fällig gewordene Hypothekarzinsen.
    interest_settled: f64,
    /// Veranlagte, noch nicht bezahlte Einkommens- und Vermögenssteuer.
    tax_pending: f64,
    /// Anteile von `tax_pending` (Einkommens- und Vermögenssteuer) und Steuern des laufenden Monats nach Art.
    pending_income: f64,
    pending_wealth: f64,
    month_capital_tax: f64,
    month_gain_tax: f64,
    year_income: f64,
    year_capital: f64,
    props: Vec<PropState>,
    /// Guthaben der Säule 3a je Person und ob der Rest schon ausbezahlt ist (#52).
    p3a: Vec<f64>,
    p3a_done: Vec<bool>,
    /// Auszahlungsplan der Säule 3a je Konto für die Anzeige (#55), nur mit `record`.
    p3a_view: Vec<Pillar3aView>,
    /// Einzahlung in die Säule 3a in diesem Monat.
    month_p3a_in: f64,
    /// Monat, in dem nach dem Verkauf der letzten Immobilie gekauft oder gemietet wird.
    after_sale: Option<YearMonth>,
    renting: bool,
    events: Vec<Event>,
    renewals: Vec<Renewal>,
    property_events: Vec<PropertyEvent>,
    bucket_refills: Vec<BucketRefill>,
    /// Grundlage der Topfgrössen des laufenden Jahres (im Januar bzw. beim Auslöser A festgelegt).
    basis: Option<NeedBasis>,
    /// Externe Zuflüsse dieses Monats, die nach den Renditen in Topf 3 gebucht werden (bis dahin in `cash`).
    inflow3: f64,
    /// Buchungsjournal der Zuflüsse dieses Monats (von, nach, Betrag, Anlass); 0 = extern, 4 = Reservierung. Wird mit
    /// `flush_inflow` ins Protokoll geschrieben.
    inflow_log: Vec<(u8, u8, f64, String)>,
    /// Einmalige Ausgaben dieses Monats ausserhalb der Ereignisse (Eigenmittel-Rest beim Kauf, Einmalzahlungen und
    /// Rückzahlungen der Hypothek). Sie gehören zu den Ausgaben des Monats und werden erst nach der Vorausschau bezahlt.
    month_out: f64,
    /// #133: Hauptbuch des laufenden Monats.
    ledger: Ledger,
    /// Gründe für den Sonderfall A in diesem Monat.
    a_reasons: Vec<String>,
    /// Reservierter Verkaufserlös für einen geplanten Kauf; liegt in `cash`, gehört aber zu keinem Topf.
    reserved: f64,
    /// Alle Töpfe leer, keine Immobilie mehr: Topf 1 darf negativ sein, Topf 2 und 3 sind stillgelegt.
    phase8: bool,
    /// Aktienindex am Ende jedes Monats (für die Rendite der letzten 12 Monate), beginnend mit dem Vorlauf des Pfads
    /// (#103): zuerst der Stand vor dem ersten Vorlaufmonat, dann je Vorlaufmonat; der Stand Ende Monat `k` liegt bei
    /// `hist_off + k`.
    stock_hist: Vec<f64>,
    hist_off: usize,
    /// Anteil der Amortisation an den laufenden Wohnkosten dieses Monats (gehört nicht zum Bedarf).
    month_amort: f64,
    /// Anlässe, bei denen die Entnahmestrategie in diesem Monat sofort angewendet wird (Zuflüsse, Verkauf ohne Folgekauf).
    apply_reasons: Vec<String>,
    /// Die Entnahmestrategie wird in diesem Monat zusätzlich zur Januar-Anwendung angewendet.
    apply_now: bool,
    /// Die Strategie hat in diesem Monat erstmals begonnen (Pensionierung); das steht schon im Protokoll.
    strategy_started: bool,
    /// Monat, in dem ein Verkaufserlös für einen Kauf im Folgemonat reserviert wurde.
    reserved_sale: Option<YearMonth>,
    /// Reservierter Verkaufserlös des laufenden Monats (bleibt liquide, wird nicht angelegt).
    reserved_cash: f64,
    records: Vec<MonthRecord>,
    summary: Summary,
    /// 3-Topf: Töpfe sind aufgeteilt (ab Pensionierung).
    buckets_on: bool,
    /// Aktienrendite im laufenden Kalenderjahr (Faktor) und im Vorjahr.
    stock_year: f64,
    stock_prev_year: f64,
    stock_idx: f64,
    bond_idx: f64,
    /// Laufende Wohnkosten dieses Monats (für die Lebenshaltung der Ausgabenregel im Netto-Bedarf, #6).
    month_running: f64,
    /// Fixe Rate bzw. Guardrails: jährliche Entnahme (nominal), Startrate und ob sie gilt.
    budget: f64,
    budget_rate0: f64,
    budget_on: bool,
}

/// Pensionskasse und AHV einer Person im Lauf.
#[derive(Clone, Debug, Default)]
struct PersonState {
    pk: f64,
    /// Aufgelaufener PK-Zins, gutgeschrieben im Dezember bzw. beim Bezug.
    pk_accrued: f64,
    retired: bool,
    pk_pension: f64,
    ahv: f64,
    ahv_started: bool,
}

impl<'a> Run<'a> {
    fn new(s: &'a Simulator, path: &'a MarketPath, record: bool) -> Self {
        let props = s
            .props
            .iter()
            .map(|p| {
                let owned = p.status == PropertyStatus::Owned;
                PropState {
                    active: owned,
                    done: false,
                    price_paid: p.price,
                    bought: if owned { p.bought } else { None },
                    balances: p.tranches.iter().map(|t| if owned { t.amount } else { 0.0 }).collect(),
                    emergency_pending: false,
                    emergency_reason: None,
                    sched: None,
                    sold: None,
                    attempted: false,
                    int_accr: 0.0,
                }
            })
            .collect();
        Run {
            s,
            path,
            record,
            cash: s.cash0,
            stocks: s.stocks0,
            bonds: s.bonds0,
            people: s
                .people
                .iter()
                .map(|p| PersonState {
                    pk: p.pk_capital,
                    ..Default::default()
                })
                .collect(),
            ahv_scale: 1.0,
            event_tax: 0.0,
            infl_idx: 1.0,
            infl_idx_jan: 1.0,
            interest_settled: 0.0,
            tax_pending: 0.0,
            pending_income: 0.0,
            pending_wealth: 0.0,
            month_capital_tax: 0.0,
            month_gain_tax: 0.0,
            year_income: 0.0,
            year_capital: 0.0,
            props,
            p3a: s.accounts.iter().map(|a| a.balance).collect(),
            p3a_done: vec![false; s.accounts.len()],
            p3a_view: s
                .accounts
                .iter()
                .map(|a| Pillar3aView {
                    person: a.person.clone(),
                    ..Default::default()
                })
                .collect(),
            month_p3a_in: 0.0,
            after_sale: None,
            renting: false,
            events: vec![],
            renewals: vec![],
            property_events: vec![],
            bucket_refills: vec![],
            basis: None,
            inflow3: 0.0,
            inflow_log: vec![],
            month_out: 0.0,
            ledger: Ledger::default(),
            a_reasons: vec![],
            reserved: 0.0,
            phase8: false,
            stock_hist: pre_levels(&path.pre_stocks, s.months),
            hist_off: path.pre_stocks.len() + 1,
            month_amort: 0.0,
            apply_reasons: vec![],
            apply_now: false,
            strategy_started: false,
            reserved_sale: None,
            reserved_cash: 0.0,
            records: if record { Vec::with_capacity(s.months) } else { vec![] },
            summary: Summary {
                min_liquid: f64::INFINITY,
                ..Default::default()
            },
            buckets_on: false,
            stock_year: 1.0,
            stock_prev_year: 0.0,
            stock_idx: 1.0,
            bond_idx: 1.0,
            month_running: 0.0,
            budget: 0.0,
            budget_rate0: 0.0,
            budget_on: false,
        }
    }

    fn pk_pension(&self) -> f64 {
        self.people.iter().map(|p| p.pk_pension).sum()
    }

    /// AHV-Renten des Haushalts; bei Ehepaaren zusammen höchstens die Plafonierung.
    fn ahv(&self) -> f64 {
        let sum: f64 = self.people.iter().map(|p| p.ahv).sum();
        match self.s.couple_cap {
            Some(cap) => sum.min(cap * self.ahv_scale),
            None => sum,
        }
    }

    /// Mindestens eine Person mit Bezugsdatum ist pensioniert (Beginn der Entnahmephase für alle Strategien ausser
    /// «Standard», #6/#47).
    fn any_retired(&self) -> bool {
        self.people
            .iter()
            .zip(&self.s.people)
            .any(|(p, c)| c.pk_at.is_some() && p.retired)
    }

    /// Betrag mit Währung für Ereignistexte.
    fn money(&self, v: f64) -> String {
        format!("{v:.0} {}", self.s.currency)
    }

    fn event(&mut self, month: YearMonth, kind: EventKind, text: String) {
        self.events.push(Event { month, kind, text });
    }

    fn worth(&self, i: usize) -> f64 {
        let p = &self.s.props[i];
        let base = if p.status == PropertyStatus::Owned {
            p.worth
        } else {
            p.price
        };
        if p.infl_value {
            base * self.infl_idx
        } else {
            base
        }
    }

    /// #133: Marktwert der Liegenschaften im Besitz.
    fn active_worth(&self) -> f64 {
        (0..self.props.len())
            .filter(|&i| self.props[i].active)
            .map(|i| self.worth(i))
            .sum()
    }

    fn equity(&self) -> f64 {
        (0..self.props.len())
            .filter(|&i| self.props[i].active)
            .map(|i| self.worth(i) - self.props[i].balances.iter().sum::<f64>())
            .sum()
    }

    /// Kapitalbezüge im selben Kalenderjahr werden für den Satz zusammengerechnet.
    fn capital_tax(&mut self, amount: f64) -> f64 {
        let t = &self.s.taxes;
        let before = tax::tax(&t.capital, self.year_capital, t.multiplier);
        self.year_capital += amount;
        // #134: Bei einer Tabelle mit fallender Steuer (Warnung der Prüfung) wäre der Zuwachs negativ.
        (tax::tax(&t.capital, self.year_capital, t.multiplier) - before).max(0.0)
    }

    fn sell(&mut self, i: usize, m: YearMonth, kind: EventKind) {
        let worth = self.worth(i);
        let debt: f64 = self.props[i].balances.iter().sum();
        // Besitzdauer in vollen Jahren; bei Besitz ohne Kaufdatum unbekannt (kein Zu- oder Abschlag).
        let years = self.props[i].bought.map(|b| b.until(m).max(0) / 12);
        let gain_tax = tax::property_gain_tax_held(
            self.s.taxes.property_gain.as_ref(),
            worth - self.props[i].price_paid,
            years,
            self.s.taxes.multiplier,
        );
        self.cash += worth - debt - gain_tax;
        self.ledger.taxes += gain_tax;
        self.interest_settled += std::mem::take(&mut self.props[i].int_accr);
        self.event_tax += gain_tax;
        self.month_gain_tax += gain_tax;
        let ps = &mut self.props[i];
        let reason = ps.emergency_reason.take();
        ps.active = false;
        ps.done = true;
        ps.emergency_pending = false;
        ps.sold = Some(m);
        ps.balances.iter_mut().for_each(|b| *b = 0.0);
        let label = if kind == EventKind::EmergencySale {
            "Notverkauf"
        } else {
            "Verkauf"
        };
        let name = self.s.props[i].name.clone();
        let mut pe = PropertyEvent::new(
            m,
            if kind == EventKind::EmergencySale {
                PropertyEventKind::EmergencySale
            } else {
                PropertyEventKind::Sale
            },
            &name,
        );
        pe.proceeds = Some(worth);
        pe.debt_repaid = Some(debt);
        pe.gain_tax = Some(gain_tax);
        pe.net = Some(worth - debt - gain_tax);
        if kind == EventKind::EmergencySale {
            self.summary.emergency_sales += 1;
            pe.reason = Some(reason.unwrap_or_else(|| "Notverkaufskriterium".into()));
        }
        // Folgekauf im nächsten Monat: Erlös reservieren, die Entnahmestrategie erst nach dem Kauf anwenden.
        // Sonst wird die Strategie im Verkaufsmonat sofort angewendet.
        let net = worth - debt - gain_tax;
        if net > 0.0 {
            if self.follow_up_purchase(i, m) {
                self.reserved_sale = Some(m);
                self.reserved_cash += net;
                self.reserved += net;
                self.mark_reservation(net, &format!("Verkauf {name}"));
                pe.note = Some("Verkaufserlös reserviert für Kauf im Folgemonat".into());
                let text = format!(
                    "Verkaufserlös {name} ({}) reserviert für Kauf im Folgemonat",
                    self.money(net)
                );
                self.event(m, EventKind::Withdrawal, text);
            } else {
                self.apply_reasons.push(format!("Verkauf {name}, kein Folgekauf"));
                pe.note = Some("Entnahmestrategie angewendet (kein Folgekauf)".into());
                self.mark_inflow(net, &format!("Verkauf {name}"));
            }
        }
        self.property_events.push(pe);
        self.event(
            m,
            kind,
            format!(
                "{label} {name}: Wert {}, Hypothek {}, Grundstückgewinnsteuer {}",
                self.money(worth),
                self.money(debt),
                self.money(gain_tax)
            ),
        );
        if !self.props.iter().any(|p| p.active) {
            self.after_sale = Some(m.plus(1));
        }
    }

    /// Ist im Monat nach dem Verkauf von Immobilie `sold` (Monat `m`) der Kauf einer anderen Immobilie geplant?
    /// Zählt der feste Kaufmonat M+1 und Käufe «nach Verkauf» dieser Immobilie bzw. der letzten Immobilie.
    fn follow_up_purchase(&self, sold: usize, m: YearMonth) -> bool {
        let other_active = self.props.iter().enumerate().any(|(j, p)| j != sold && p.active);
        self.s.props.iter().enumerate().any(|(j, p)| {
            if j == sold || p.status != PropertyStatus::Planned {
                return false;
            }
            let st = &self.props[j];
            if st.active || st.done || st.attempted {
                return false;
            }
            match p.buy_trigger {
                Trigger::None => p.buy == Some(m.plus(1)),
                Trigger::SaleOf(s) => s == sold,
                Trigger::LastSale => !other_active,
            }
        })
    }

    /// Käufe «nach Verkauf von …» im Monat nach dem jeweiligen Verkauf. Danach, falls keine
    /// Immobilie mehr im Besitz ist: die nächste Immobilie «nach Verkauf» kaufen, sofern tragbar und
    /// finanzierbar; sonst beginnen die Mieten «nach Verkauf».
    fn after_sales(&mut self, m: YearMonth) {
        let s = self.s;
        for i in 0..s.props.len() {
            let Trigger::SaleOf(j) = s.props[i].buy_trigger else {
                continue;
            };
            let ps = &self.props[i];
            if ps.active || ps.done || ps.attempted || !self.props[j].sold.is_some_and(|sm| m > sm) {
                continue;
            }
            self.props[i].attempted = true;
            if !self.try_buy(i, m) {
                // Kauf nicht finanzierbar: Der Kauf entfällt, die Folgeregeln greifen im Folgemonat
                // (Miete «nach Verkauf» dieser Immobilie, weitere Käufe «nach Verkauf» davon).
                self.props[i].done = true;
                self.props[i].sold = Some(m);
                let text = format!(
                    "Kauf {} entfällt: Folgeregeln (Miete, weitere Käufe «nach Verkauf») gelten ab dem Folgemonat",
                    s.props[i].name
                );
                self.event(m, EventKind::Purchase, text);
                if !self.props.iter().any(|p| p.active) {
                    self.after_sale.get_or_insert(m);
                }
            }
        }
        if !self.after_sale.is_some_and(|a| m >= a) {
            return;
        }
        self.after_sale = None;
        if self.props.iter().any(|p| p.active) {
            return;
        }
        for i in 0..s.props.len() {
            let p = &s.props[i];
            if p.buy_trigger == Trigger::LastSale && !self.props[i].active && !self.props[i].done && self.try_buy(i, m)
            {
                return;
            }
        }
        if !self.renting && s.props.iter().any(|p| p.rent_trigger == Trigger::LastSale) {
            self.renting = true;
            let names: Vec<&str> = s
                .props
                .iter()
                .filter(|p| p.rent_trigger == Trigger::LastSale)
                .map(|p| p.name.as_str())
                .collect();
            self.event(m, EventKind::RentStart, format!("Miete ab jetzt: {}", names.join(", ")));
        }
    }

    /// Laufende Wohnkosten pro Monat (Zins, Amortisation, Nebenkosten) aller aktiven Immobilien, um das
    /// Notverkaufskriterium nach einer Einmalzahlung zu schätzen. Die Salden der Immobilie `only` werden
    /// mit `keep` skaliert; ihre Tranchen kommen von `only_tranches` (sie sind während der Erneuerung herausgenommen).
    fn running_estimate(&self, m: YearMonth, k: usize, only: usize, only_tranches: &[CTranche], keep: f64) -> f64 {
        let s = self.s;
        let saron = self.path.saron[k];
        let mut total = 0.0;
        for (j, p) in s.props.iter().enumerate() {
            if !self.props[j].active {
                continue;
            }
            total += p.fix_costs * if p.infl_fix_costs { self.infl_idx } else { 1.0 };
            let tranches = if j == only {
                only_tranches
            } else {
                self.props[j].sched.as_deref().unwrap_or(&p.tranches)
            };
            for (t, tr) in tranches.iter().enumerate() {
                let b = self.props[j].balances[t] * if j == only { keep } else { 1.0 };
                let per = tr
                    .periods
                    .iter()
                    .find(|x| x.start <= m && m < x.end)
                    .or(tr.periods.last());
                if let Some(x) = per {
                    total += b * x.rate(saron) / 12.0;
                    if x.amort_q > 0.0 && b > 0.0 {
                        total += x.amort_q / 3.0;
                    }
                }
            }
        }
        total
    }

    /// Zinssatz der neuen Periode: Durchschnitt der Tranchen, gewichtet mit ihren Salden (mal `keep`).
    fn renewal_rate(&self, i: usize, m: YearMonth, k: usize, tranches: &[CTranche], keep: f64) -> f64 {
        let saron = self.path.saron[k];
        let (mut sum, mut weighted) = (0.0, 0.0);
        for (t, tr) in tranches.iter().enumerate() {
            let b = self.props[i].balances[t] * keep;
            if let Some(x) = tr
                .periods
                .iter()
                .find(|x| x.start <= m && m < x.end)
                .or(tr.periods.last())
            {
                sum += b;
                weighted += b * x.rate(saron);
            }
        }
        if sum > 0.0 {
            weighted / sum
        } else {
            0.0
        }
    }

    /// Hält eine Erneuerung im Protokoll fest (`renewals`) und in der gemeinsamen Liste der Immobilienereignisse.
    fn push_renewal(&mut self, record: Renewal, rate: f64) {
        let mut pe = PropertyEvent::new(record.month, PropertyEventKind::Renewal, &record.property);
        pe.affordable = Some(record.affordable);
        pe.debt_before = Some(record.debt_before);
        pe.debt_after = Some(record.debt_after);
        pe.lump_sum = (record.lump_sum > 0.0).then_some(record.lump_sum);
        pe.rate = Some(rate);
        pe.reason = record.emergency_sale.clone();
        self.property_events.push(pe);
        self.renewals.push(record);
    }

    /// Tragbarkeit bei der Erneuerung einer Hypothekarperiode (kalkulatorischer Zins, nur Einkommen).
    /// - Tragbar: nichts zu tun.
    /// - Nicht tragbar: minimale Einmalzahlung, damit sie es wieder ist. Sie wird aus dem Vermögen bezahlt
    ///   und die Hypothek reduziert erneuert, wenn das Vermögen danach >= 0 ist **und** das Notverkaufskriterium
    ///   nach der Zahlung nicht ausgelöst wird (geschätzt mit den geplanten Ausgaben und den Wohnkosten nach der Zahlung).
    /// - Sonst: keine Zahlung, die Hypothek wird nicht erneuert und ein Notverkauf eingeleitet (Folgemonat).
    ///
    /// Gibt `true` zurück, wenn die Hypothek erneuert wird.
    fn renewal_affordability(&mut self, i: usize, m: YearMonth, k: usize, tranches: &[CTranche]) -> bool {
        let s = self.s;
        let p = &s.props[i];
        let a = &s.afford;
        let debt: f64 = self.props[i].balances.iter().sum();
        if debt <= 0.0 {
            return true;
        }
        let name = p.name.clone();
        let income_m = s.earned(m, self.infl_idx) + self.pk_pension() + self.ahv();
        let fix = if p.fix_costs > 0.0 {
            p.fix_costs * 12.0 * if p.infl_fix_costs { self.infl_idx } else { 1.0 }
        } else {
            self.worth(i) * a.fix_costs
        };
        let max = ((a.sustainability * income_m * 12.0 - fix) / a.imputed_rate).max(0.0);
        let lump = debt - max;
        let mut record = Renewal {
            month: m,
            property: name.clone(),
            affordable: lump < 0.5,
            lump_sum: 0.0,
            debt_before: debt,
            debt_after: debt,
            emergency_sale: None,
        };
        if lump < 0.5 {
            let rate = self.renewal_rate(i, m, k, tranches, 1.0);
            self.push_renewal(record, rate);
            return true;
        }
        let liquid = self.cash + self.stocks + self.bonds;
        let keep = max / debt;
        // Nach der Zahlung: Vermögen >= 0 und Notverkaufskriterium nicht ausgelöst?
        let reason = if liquid - lump < 0.0 {
            Some("Vermögen < 0")
        } else {
            let shortfall =
                s.planned_living(m) * self.infl_idx + self.running_estimate(m, k, i, tranches, keep) - income_m;
            (liquid - lump - s.emergency.at(m) * shortfall < 0.0).then_some("Notverkaufskriterium")
        };
        if let Some(reason) = reason {
            let why = if reason == "Vermögen < 0" {
                format!("Vermögen {} reicht nicht", self.money(liquid))
            } else {
                "danach würde das Notverkaufskriterium auslösen".to_string()
            };
            let text = format!(
                "Hypothek {name} nicht mehr tragbar: Einmalzahlung {} nötig, {why}; Hypothek nicht erneuert, Notverkauf eingeleitet",
                self.money(lump)
            );
            self.event(m, EventKind::MortgageRenewal, text);
            self.props[i].emergency_pending = true;
            self.props[i].emergency_reason = Some(reason.to_string());
            self.summary.affordability_sales += 1;
            record.emergency_sale = Some(reason.to_string());
            let rate = self.renewal_rate(i, m, k, tranches, 1.0);
            self.push_renewal(record, rate);
            return false;
        }
        let rate = self.renewal_rate(i, m, k, tranches, keep);
        self.props[i].balances.iter_mut().for_each(|b| *b *= keep);
        self.cash -= lump;
        self.month_out += lump;
        self.summary.lump_sums += 1;
        let text = format!(
            "Einmalrückzahlung Hypothek {name} für die Tragbarkeit: {}, Hypothek neu {}",
            self.money(lump),
            self.money(max)
        );
        self.event(m, EventKind::MortgageRenewal, text);
        record.lump_sum = lump;
        record.debt_after = max;
        self.push_renewal(record, rate);
        true
    }

    /// Kauf mit Tragbarkeitsprüfung. Ist die gewünschte Hypothek nicht tragbar, wird sie gekürzt
    /// und die Differenz aus dem Vermögen bezahlt, sofern es reicht.
    fn try_buy(&mut self, i: usize, m: YearMonth) -> bool {
        let s = self.s;
        let p = &s.props[i];
        let a = &s.afford;
        let price = if p.infl_value { p.price * self.infl_idx } else { p.price };
        let entered: f64 = p.tranche_defs.iter().map(|t| t.amount).sum();
        let liquid = self.cash + self.stocks + self.bonds;
        // Tragbarkeit: nur Einkommen (Erwerb, Renten), keine Entnahme aus dem Vermögen.
        let income = (s.earned(m, self.infl_idx) + self.pk_pension() + self.ahv()) * 12.0;
        let fix = if p.fix_costs > 0.0 {
            p.fix_costs * 12.0
        } else {
            price * a.fix_costs
        };
        // Gewünschte Hypothek: höchste Belehnung oder die erfassten Beträge, höchstens die Belehnungsgrenze.
        let ltv = price * a.max_loan_to_value;
        let cap = if p.max_loan { ltv } else { entered.min(ltv) };
        let max = ((a.sustainability * income - fix) / a.imputed_rate).max(0.0);
        let mortgage = cap.min(max);
        // Einmalrückzahlung: Teil der gewünschten Hypothek, der für die Tragbarkeit aus dem Vermögen bezahlt wird.
        let lump = cap - mortgage;
        if liquid < price - mortgage {
            self.event(
                m,
                EventKind::Purchase,
                format!(
                    "Kauf {} nicht möglich: Eigenmittel {} nötig, vorhanden {}",
                    p.name,
                    self.money(price - mortgage),
                    self.money(liquid)
                ),
            );
            return false;
        }
        // Die Hypothek wird im Verhältnis der erfassten Beträge auf die Tranchen verteilt.
        let n = p.tranche_defs.len().max(1) as f64;
        let share = |t: &Tranche| {
            if entered > 0.0 {
                t.amount / entered
            } else {
                1.0 / n
            }
        };
        let sched: Vec<CTranche> = p
            .tranche_defs
            .iter()
            .map(|t| {
                let mut c = schedule(t, m, s.end.plus(1));
                c.amount = mortgage * share(t);
                c.account = t
                    .pillar3a
                    .as_deref()
                    .and_then(|id| s.person_ids.iter().position(|p| p == id))
                    .unwrap_or(0)
                    .min(1);
                c
            })
            .collect();
        let ps = &mut self.props[i];
        ps.balances = sched.iter().map(|t| t.amount).collect();
        ps.sched = Some(sched);
        ps.active = true;
        ps.price_paid = price;
        ps.bought = Some(m);
        self.pay_equity(price - mortgage);
        self.renting = false;
        let mut pe = PropertyEvent::new(m, PropertyEventKind::Purchase, &p.name);
        pe.price = Some(price);
        pe.equity = Some(price - mortgage);
        pe.mortgage = Some(mortgage);
        self.property_events.push(pe);
        self.event(
            m,
            EventKind::Purchase,
            format!(
                "Kauf {}: Preis {}, Hypothek {}{}",
                p.name,
                self.money(price),
                self.money(mortgage),
                if lump >= 0.5 {
                    format!(", Einmalrückzahlung {} für die Tragbarkeit", self.money(lump))
                } else {
                    String::new()
                }
            ),
        );
        true
    }

    /// Wohnkosten dieses Monats (inkl. Amortisation) und geglättete laufende Kosten für die Reserve.
    fn properties(&mut self, k: usize, m: YearMonth) -> (f64, f64) {
        let s = self.s;
        let saron = self.path.saron[k];
        let mut cost = 0.0;
        let mut running = 0.0;
        self.month_amort = 0.0;
        for i in 0..self.props.len() {
            if self.props[i].emergency_pending && self.props[i].active {
                self.sell(i, m, EventKind::EmergencySale);
            }
        }
        self.after_sales(m);
        for i in 0..self.props.len() {
            let p = &s.props[i];
            match p.status {
                PropertyStatus::Planned
                    if p.max_loan && !self.props[i].active && !self.props[i].done && p.buy == Some(m) =>
                {
                    self.try_buy(i, m);
                }
                PropertyStatus::Planned if !self.props[i].active && !self.props[i].done && p.buy == Some(m) => {
                    let price = if p.infl_value { p.price * self.infl_idx } else { p.price };
                    let ps = &mut self.props[i];
                    ps.active = true;
                    ps.price_paid = price;
                    ps.bought = Some(m);
                    for (b, t) in ps.balances.iter_mut().zip(&p.tranches) {
                        *b = t.amount;
                    }
                    let debt: f64 = ps.balances.iter().sum();
                    self.pay_equity(price - debt);
                    self.renting = false;
                    let mut pe = PropertyEvent::new(m, PropertyEventKind::Purchase, &p.name);
                    pe.price = Some(price);
                    pe.equity = Some(price - debt);
                    pe.mortgage = Some(debt);
                    self.property_events.push(pe);
                    self.event(
                        m,
                        EventKind::Purchase,
                        format!(
                            "Kauf {}: Preis {}, Hypothek {}",
                            p.name,
                            self.money(price),
                            self.money(debt)
                        ),
                    );
                }
                PropertyStatus::Rent => {
                    let from = match p.rent_trigger {
                        Trigger::None => p.rent_from.is_some_and(|f| m >= f),
                        Trigger::LastSale => self.renting,
                        Trigger::SaleOf(j) => self.props[j].sold.is_some_and(|sm| m > sm),
                    };
                    let until = p.rent_until.is_none_or(|u| m < u);
                    if from && until {
                        let rent = p.rent * if p.infl_rent { self.infl_idx } else { 1.0 };
                        cost += rent;
                        running += rent;
                    }
                }
                _ => {}
            }
            if !self.props[i].active {
                continue;
            }
            let fix = p.fix_costs * if p.infl_fix_costs { self.infl_idx } else { 1.0 };
            cost += fix;
            running += fix;
            // Perioden eines Kaufs «nach Verkauf» liegen im Laufzustand; kurz herausnehmen.
            let dynamic = self.props[i].sched.take();
            let tranches = dynamic.as_deref().unwrap_or(&p.tranches);
            // Erneuert eine Tranche ihre Periode, prüft die Bank die Tragbarkeit der ganzen Hypothek neu.
            let renews = k > 0
                && tranches.iter().enumerate().any(|(j, t)| {
                    self.props[i].balances[j] > 0.0 && t.periods.iter().any(|x| x.index > 0 && x.start == m)
                });
            // Nicht tragbar und nicht bezahlbar: Die Hypothek wird nicht erneuert (Notverkauf eingeleitet).
            let renewed = !renews || self.renewal_affordability(i, m, k, tranches);
            for (j, t) in tranches.iter().enumerate() {
                let per = t.periods.iter().find(|x| x.start <= m && m < x.end);
                let b = self.props[i].balances[j];
                if let Some(x) = per {
                    if x.index > 0 && x.start == m && b > 0.0 && k > 0 && renewed {
                        let tn = if tranches.len() > 1 {
                            format!(" ({})", t.name)
                        } else {
                            String::new()
                        };
                        self.event(
                            m,
                            EventKind::MortgageRenewal,
                            format!(
                                "Neue Hypothekarperiode {}{tn}: {:.2} %, {}",
                                p.name,
                                x.rate(saron) * 100.0,
                                self.money(b)
                            ),
                        );
                    }
                }
                if t.repay_at == Some(m) && b > 0.0 {
                    self.cash -= b;
                    self.month_out += b;
                    self.props[i].balances[j] = 0.0;
                    self.event(
                        m,
                        EventKind::MortgageRepayment,
                        format!("Rückzahlung Hypothek {}: {}", p.name, self.money(b)),
                    );
                    continue;
                }
                // #50: Im Monat des AHV-Bezugs wechselt eine indirekt amortisierte Tranche auf direkt; als Ereignis
                // zeigen, wenn vorher indirekt amortisiert wurde und ab jetzt weiter amortisiert wird.
                if k > 0 && b > 0.0 {
                    let a = t.account.min(s.accounts.len().saturating_sub(1));
                    if let Some(c) = s.accounts.get(a).filter(|c| c.ahv == Some(m)) {
                        let was_indirect = t.periods.iter().any(|x| !x.direct && x.amort_q > 0.0 && x.start < m);
                        if let Some(x) = per.filter(|x| x.amort_q > 0.0 && was_indirect) {
                            let tn = if tranches.len() > 1 {
                                format!(" ({})", t.name)
                            } else {
                                String::new()
                            };
                            self.event(
                                m,
                                EventKind::AmortizationSwitch,
                                format!(
                                    "Amortisation Hypothek {}{tn} ab AHV-Bezug{} direkt statt über die Säule 3a: {} pro Jahr",
                                    p.name,
                                    c.who,
                                    self.money(x.amort_q * 4.0)
                                ),
                            );
                        }
                    }
                }
                let rate = per.or(t.periods.last()).map_or(0.0, |x| x.rate(saron));
                let interest = b * rate / 12.0;
                self.props[i].int_accr += interest;
                running += interest;
                if let Some(x) = per {
                    if x.amort_q > 0.0 && b > 0.0 {
                        running += x.amort_q / 3.0;
                        self.month_amort += x.amort_q / 3.0;
                        if m.is_quarter_end() {
                            // #50: Indirekt nur bis vor dem AHV-Bezug der Person; danach direkt.
                            let a = t.account.min(s.accounts.len().saturating_sub(1));
                            let after_ahv = s.accounts.get(a).is_some_and(|c| c.ahv.is_some_and(|h| m >= h));
                            if x.direct || after_ahv || s.accounts.is_empty() {
                                let a = x.amort_q.min(b);
                                self.props[i].balances[j] -= a;
                                cost += a;
                                self.ledger.amortization += a;
                            } else {
                                self.p3a[a] += x.amort_q;
                                cost += x.amort_q;
                                self.ledger.amortization += x.amort_q;
                            }
                        }
                    }
                }
            }
            self.props[i].sched = dynamic;
            // Zinsen werden monatlich aufgelaufen und am Quartalsende bezahlt (am Planende der Rest).
            if m.is_quarter_end() || k + 1 == s.months {
                cost += std::mem::take(&mut self.props[i].int_accr);
            }
            if p.status != PropertyStatus::Rent && p.sell == Some(m) {
                self.sell(i, m, EventKind::Sale);
            }
        }
        cost += std::mem::take(&mut self.interest_settled);
        (cost, running)
    }

    /// Säule 3a (#52): Zins, Einzahlung bis vor dem AHV-Bezug, geplante Auszahlungen und Auszahlung des Rests im
    /// Monat des AHV-Bezugs. Liefert die Einzahlung des Monats (sie zählt zu den Ausgaben).
    fn pillar3a(&mut self, m: YearMonth) -> f64 {
        let s = self.s;
        let mut paid_in = 0.0;
        for a in 0..s.accounts.len() {
            let c = &s.accounts[a];
            if self.p3a_done[a] {
                continue;
            }
            self.ledger.p3a_interest += self.p3a[a] * c.rate_m;
            self.p3a[a] *= 1.0 + c.rate_m;
            if c.contribution_m > 0.0 && c.ahv.is_none_or(|h| m < h) {
                self.p3a[a] += c.contribution_m;
                paid_in += c.contribution_m;
            }
            if self.record {
                self.p3a_view[a].balances.push(self.p3a[a]);
            }
            for &(date, amount, repay, j) in &c.payouts {
                if date != m || c.ahv.is_some_and(|h| date > h) {
                    continue;
                }
                let available = self.p3a[a];
                let gross = amount.unwrap_or(available).min(available);
                if let Some(want) = amount.filter(|w| *w > available + 0.005) {
                    self.event(
                        m,
                        EventKind::Pillar3aPayout,
                        format!(
                            "Säule 3a{}: Geplant {}, das Guthaben reicht nur für {}",
                            c.who,
                            self.money(want),
                            self.money(available)
                        ),
                    );
                }
                self.pay_out_3a(a, m, gross, repay, "Auszahlung");
                if self.record {
                    let after = self.p3a[a];
                    self.p3a_view[a].payouts.push(Pillar3aPayoutView {
                        index: j,
                        month: m,
                        before: available,
                        paid: gross,
                        after,
                    });
                }
            }
            if c.ahv.is_some_and(|h| m >= h) {
                let rest = self.p3a[a];
                if rest > 0.005 {
                    self.pay_out_3a(a, m, rest, c.rest_repay, "Auszahlung des Rests beim AHV-Bezug");
                }
                if self.record {
                    self.p3a_view[a].rest = Some(Pillar3aRest {
                        month: m,
                        amount: rest.max(0.0),
                    });
                }
                self.p3a[a] = 0.0;
                self.p3a_done[a] = true;
            }
        }
        paid_in
    }

    /// Auszahlung aus der Säule 3a mit Kapitalbezugssteuer; auf Wunsch zuerst an die Tranchen mit indirekter
    /// Amortisation in dieses Konto, der Rest ins Vermögen.
    fn pay_out_3a(&mut self, a: usize, m: YearMonth, gross: f64, repay: bool, what: &str) {
        let s = self.s;
        if gross <= 0.0 {
            return;
        }
        self.p3a[a] -= gross;
        let t = self.capital_tax(gross);
        self.ledger.taxes += t;
        self.event_tax += t;
        self.month_capital_tax += t;
        let mut net = gross - t;
        let mut repaid = 0.0;
        if repay {
            for i in 0..self.props.len() {
                if !self.props[i].active {
                    continue;
                }
                let tr = self.props[i].sched.as_deref().unwrap_or(&s.props[i].tranches);
                let mut order: Vec<usize> = (0..tr.len()).filter(|&j| tr[j].account == a).collect();
                order.sort_by_key(|&j| tr[j].periods.iter().all(|x| x.direct));
                for j in order {
                    let r = net.min(self.props[i].balances[j]);
                    self.props[i].balances[j] -= r;
                    net -= r;
                    repaid += r;
                }
            }
        }
        self.inflow(net, "Auszahlung Säule 3a");
        let who = s.accounts[a].who.clone();
        self.event(
            m,
            EventKind::Pillar3aPayout,
            format!(
                "Säule 3a{who}: {what} {}, Steuer {}, an Hypothek {}",
                self.money(gross),
                self.money(t),
                self.money(repaid)
            ),
        );
    }

    fn rebalance(&mut self, reserve: f64, share: f64) {
        let total = self.cash + self.stocks + self.bonds;
        if total - reserve < 0.0 {
            self.cash = total;
            self.stocks = 0.0;
            self.bonds = 0.0;
        } else {
            let invest = total - reserve;
            self.cash = reserve;
            self.stocks = invest * share;
            self.bonds = invest * (1.0 - share);
        }
    }

    /// AHV- und PK-Rente im Monat `mm` für die Planung des laufenden Jahres: laufende Renten wie heute, noch nicht
    /// begonnene ab ihrem Datum (PK-Rente aus dem heutigen Guthaben geschätzt).
    fn pensions_at(&self, mm: YearMonth) -> (f64, f64) {
        let (mut ahv, mut pk) = (0.0, 0.0);
        for (st, c) in self.people.iter().zip(&self.s.people) {
            pk += if st.retired {
                st.pk_pension
            } else if c.pk_at.is_some_and(|a| mm >= a) {
                (st.pk + st.pk_accrued) * (1.0 - c.pk_lumpsum) * c.pk_conversion / 12.0
            } else {
                0.0
            };
            ahv += if st.ahv_started {
                st.ahv
            } else if c.ahv_at.is_some_and(|a| mm >= a) {
                c.ahv_monthly
            } else {
                0.0
            };
        }
        let ahv = match self.s.couple_cap {
            Some(cap) => ahv.min(cap * self.ahv_scale),
            None => ahv,
        };
        (ahv, pk)
    }

    /// Netto-Bedarf aus den geplanten Werten des laufenden Jahres ab Monat `m` bis Dezember (höchstens bis zum Planende),
    /// auf 12 Monate hochgerechnet. `housing_now` sind die laufenden Wohnkosten ohne Amortisation dieses Monats.
    fn plan_basis(&self, m: YearMonth, housing_now: f64, source: String) -> NeedBasis {
        let s = self.s;
        let last = s.start.plus(s.months as i32 - 1);
        let (mut living, mut housing, mut ahv, mut pk, mut earned, mut p3a) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut n = 0u32;
        // #6: Steuert eine Ausgabenregel (Guardrails) die Ausgaben, kommt die Lebenshaltung aus ihr.
        let rule = s.dynamic.filter(|_| self.budget_on);
        for i in 0..(13 - m.month() as i32) {
            let mm = m.plus(i);
            if mm > last {
                break;
            }
            let (a, p) = self.pensions_at(mm);
            living += match &rule {
                Some(d) => self.rule_living(
                    d,
                    mm,
                    a + p + s.earned(mm, self.infl_idx),
                    self.month_running + s.pillar3a_contribution(mm),
                ),
                None => s.planned_living(mm) * self.infl_idx,
            };
            housing += housing_now;
            p3a += s.pillar3a_contribution(mm);
            ahv += a;
            pk += p;
            earned += s.earned(mm, self.infl_idx);
            n += 1;
        }
        let f = if n > 0 { 12.0 / f64::from(n) } else { 0.0 };
        let t = &s.taxes;
        NeedBasis {
            year: m.year(),
            source,
            living_source: if rule.is_some() { "Guardrails" } else { "geplant" }.into(),
            living: living * f,
            housing: housing * f,
            income_tax: tax::tax(&t.income, (ahv + pk + earned) * f, t.multiplier),
            ahv: ahv * f,
            pk: pk * f,
            earned: earned * f,
            pillar3a: p3a * f,
            months: n,
        }
    }

    /// Aktienrendite für die Renditebedingung (#103): die letzten 12 Monate einschliesslich des laufenden, als Verhältnis
    /// der Indexstände; der Vorlauf des Pfads liefert die Monate vor dem Start. Fehlt er ganz oder teilweise (Reihe beginnt
    /// erst später), gilt die Rendite seit dem frühesten bekannten Stand («seit Start»).
    fn stock_return(&self, k: usize) -> (f64, &'static str) {
        let h = &self.stock_hist;
        // Vor dem Renditeschritt des Monats (Vorschau) fehlt der laufende Monat: dann gilt der letzte vorhandene
        let i = (self.hist_off + k).min(h.len() - 1);
        if i >= 12 {
            (h[i] / h[i - 12] - 1.0, "letzte 12 Monate")
        } else {
            (h[i] / h[0] - 1.0, "seit Start")
        }
    }

    /// Bucht `amount` von Topf `from` nach Topf `to` um und hält es fest (1 = Cash, 2 = Obligationen, 3 = Aktien).
    fn bucket_move(&mut self, from: u8, to: u8, amount: f64, moves: &mut Vec<BucketMove>) {
        if amount <= 1e-9 {
            return;
        }
        match from {
            1 => self.cash -= amount,
            2 => self.bonds -= amount,
            _ => self.stocks -= amount,
        }
        match to {
            1 => self.cash += amount,
            2 => self.bonds += amount,
            _ => self.stocks += amount,
        }
        moves.push(BucketMove { from, to, amount });
    }

    /// Zufluss in das Cash. Bei aktiver 3-Topf-Strategie gehört er in Topf 3 (Sonderfall B).
    fn inflow(&mut self, amount: f64, why: &str) {
        self.cash += amount;
        self.mark_inflow(amount, why);
    }

    /// Vermerkt einen bereits in `cash` gebuchten Zufluss für Topf 3; die Umbuchung folgt nach den Renditen.
    fn mark_inflow(&mut self, amount: f64, why: &str) {
        self.mark_inflow_from(0, amount, why);
    }

    /// Wie `mark_inflow`; `from` ist 0 (extern) oder 4 (Reservierung).
    fn mark_inflow_from(&mut self, from: u8, amount: f64, why: &str) {
        if amount > 0.0 && self.buckets_active() {
            self.inflow3 += amount;
            self.inflow_log.push((from, 3, amount, why.to_string()));
        }
    }

    /// Verkaufserlös, der für einen Kauf im Folgemonat reserviert wird (gehört zu keinem Topf).
    fn mark_reservation(&mut self, amount: f64, why: &str) {
        if amount > 0.0 && self.buckets_active() {
            self.inflow_log.push((0, 4, amount, why.to_string()));
        }
    }

    /// Die 3-Topf-Strategie läuft und Topf 2 und 3 sind nicht stillgelegt.
    fn buckets_active(&self) -> bool {
        self.s.buckets.is_some() && self.buckets_on && !self.phase8
    }

    /// Eigenmittel eines Kaufs: zuerst aus der Reservierung, der Rest belastet Topf 1.
    fn pay_equity(&mut self, equity: f64) {
        self.cash -= equity;
        let from_reservation = self.reserved.min(equity.max(0.0));
        self.reserved -= from_reservation;
        // Der Rest ist eine einmalige Ausgabe dieses Monats: vorgemerkt, nicht sofort aus Topf 1 bezahlt
        self.month_out += equity - from_reservation;
    }

    /// Protokolleintrag der 3-Topf-Strategie.
    #[allow(clippy::too_many_arguments)]
    fn log_buckets(
        &mut self,
        b: &Buckets,
        m: YearMonth,
        k: usize,
        trigger: &str,
        reason: String,
        passed: u32,
        ignored: bool,
        targets: (f64, f64),
        moves: Vec<BucketMove>,
        payment: f64,
        before: [f64; 3],
        after: [f64; 3],
        text: String,
    ) {
        let (ret, ret_basis) = self.stock_return(k);
        let basis = self.basis.clone().unwrap_or_default();
        let need = basis.need();
        self.event(m, EventKind::Buckets, text);
        self.bucket_refills.push(BucketRefill {
            month: m,
            reason,
            trigger: trigger.to_string(),
            passed_months: passed,
            basis,
            need,
            target1: targets.0,
            target2: targets.1,
            stock_return: ret,
            return_basis: ret_basis.to_string(),
            threshold: b.refill_threshold,
            condition_met: ret >= b.refill_threshold,
            condition_ignored: ignored,
            moves,
            payment,
            before,
            after,
        });
    }

    /// Sonderfall B: Zuflüsse ins Protokoll schreiben und die Beträge für Topf 3 von `cash` nach Topf 3 buchen (keine
    /// Auffüllung von Topf 1 oder 2). Das Protokoll zeigt Topf 1 ohne den Zufluss, der Zufluss erscheint als Buchung
    /// «extern → Topf 3», «extern → Reservierung» oder «Reservierung → Topf 3».
    /// `shift` rechnet den Stand von Topf 1 im Protokoll auf «vor den Ausgaben» um (0, wenn `cash` schon so geführt wird).
    fn flush_inflow(&mut self, b: &Buckets, m: YearMonth, k: usize, pay: f64, shift: f64) {
        let amount = std::mem::take(&mut self.inflow3);
        let log = std::mem::take(&mut self.inflow_log);
        if log.is_empty() {
            return;
        }
        let t1 = self.cash + shift - amount;
        let t = self.target_pair(b, 0, 0.0);
        // Herkunft eines Zuflusses: 0 = extern, 4 = Reservierung
        let name = |x: u8| if x == 0 { "extern" } else { "Reservierung" };
        // Reservierungen (extern → Reservierung): kein Topf ändert sich
        for (from, to, amt, why) in log.iter().filter(|x| x.1 == 4) {
            let st = [t1, self.bonds, self.stocks];
            let text = format!("3-Topf Zufluss in die Reservierung ({why}): {}", self.money(*amt));
            self.log_buckets(
                b,
                m,
                k,
                "B",
                format!("Zufluss in die Reservierung ({why})"),
                0,
                true,
                t,
                vec![BucketMove {
                    from: *from,
                    to: *to,
                    amount: *amt,
                }],
                pay,
                st,
                st,
                text,
            );
        }
        let to3: Vec<_> = log.iter().filter(|x| x.1 == 3).collect();
        if to3.is_empty() {
            return;
        }
        let why = to3.iter().map(|x| x.3.clone()).collect::<Vec<_>>().join(", ");
        let moves: Vec<BucketMove> = to3
            .iter()
            .map(|x| BucketMove {
                from: x.0,
                to: 3,
                amount: x.2,
            })
            .collect();
        let before = [t1, self.bonds, self.stocks];
        self.cash -= amount;
        self.stocks += amount;
        let after = [t1, self.bonds, self.stocks];
        let text = format!(
            "3-Topf {} nach Topf 3 ({why}): {}; Topf 1/2/3 nachher {} / {} / {}",
            if to3.iter().all(|x| x.0 == 4) { name(4) } else { name(0) },
            self.money(amount),
            self.money(after[0]),
            self.money(after[1]),
            self.money(after[2])
        );
        self.log_buckets(
            b,
            m,
            k,
            "B",
            format!("Zufluss nach Topf 3 ({why})"),
            0,
            true,
            t,
            moves,
            pay,
            before,
            after,
            text,
        );
    }

    /// Zielgrössen von Topf 1 und 2, um `passed` bereits vergangene Monate des Jahres reduziert.
    /// Das Ziel von Topf 1 enthält zusätzlich die einmaligen Ausgaben `one_off` dieses Monats.
    fn target_pair(&self, b: &Buckets, passed: u32, one_off: f64) -> (f64, f64) {
        let need = self.basis.as_ref().map_or(0.0, NeedBasis::need);
        let t = |years: f64| (years * 12.0 - f64::from(passed)).max(0.0) / 12.0 * need;
        (t(b.cash_years) + one_off, t(b.bond_years))
    }

    /// Gemeinsame Auffüllung der Töpfe für die Januar-Auffüllung, den Sonderfall A und C1. Nur in der Januar-Auffüllung
    /// (`surplus`) geht ein Überschuss in Topf 1 oder 2 in Topf 3. Topf 1 wird zuerst aus Topf 2 aufgefüllt. Verkäufe aus Topf 3 (Rest für Topf 1, dann Topf 2) nur,
    /// wenn das Renditekriterium erfüllt ist oder ignoriert wird (`ignore`).
    fn fill_pots(
        &mut self,
        b: &Buckets,
        m: YearMonth,
        k: usize,
        (trigger, reason, passed, ignore, pay): (&str, String, u32, bool, f64),
        (one_off, surplus): (f64, bool),
        (before, mut moves): ([f64; 3], Vec<BucketMove>),
    ) {
        let (t1, t2) = self.target_pair(b, passed, one_off);
        let (ret, ret_basis) = self.stock_return(k);
        let met = ret >= b.refill_threshold;
        let sell_ok = met || ignore;
        // Überschussbuchung in Topf 3 nur bei der Januar-Auffüllung
        if surplus && self.cash > t1 {
            self.bucket_move(1, 3, self.cash - t1, &mut moves);
        }
        let d1 = (t1 - self.cash).max(0.0);
        self.bucket_move(2, 1, d1.min(self.bonds.max(0.0)), &mut moves);
        if sell_ok {
            let d1 = (t1 - self.cash).max(0.0);
            self.bucket_move(3, 1, d1.min(self.stocks.max(0.0)), &mut moves);
            let d2 = (t2 - self.bonds).max(0.0);
            self.bucket_move(3, 2, d2.min(self.stocks.max(0.0)), &mut moves);
        }
        if surplus && self.bonds > t2 {
            self.bucket_move(2, 3, self.bonds - t2, &mut moves);
        }
        let basis = self.basis.clone().unwrap_or_default();
        let after = [self.cash, self.bonds, self.stocks];
        let crit = if ignore {
            "Kriterium ignoriert".to_string()
        } else {
            format!(
                "Aktienrendite {:.1} % ({}) {} Schwelle {:.1} %",
                ret * 100.0,
                ret_basis,
                if met { "≥" } else { "<" },
                b.refill_threshold * 100.0
            )
        };
        let text = format!(
            "3-Topf aufgefüllt ({reason}): Netto-Bedarf {} (Ausgaben {} − Einnahmen {}, {}); Ziel Topf 1 {}, Topf 2 {}; {crit}; Topf 1/2/3 vorher {} / {} / {}, nachher {} / {} / {}",
            self.money(basis.need()),
            self.money(basis.expenses()),
            self.money(basis.receipts()),
            basis.source,
            self.money(t1),
            self.money(t2),
            self.money(before[0]),
            self.money(before[1]),
            self.money(before[2]),
            self.money(after[0]),
            self.money(after[1]),
            self.money(after[2])
        );
        self.log_buckets(
            b,
            m,
            k,
            trigger,
            reason,
            passed,
            ignore,
            (t1, t2),
            moves,
            pay,
            before,
            after,
            text,
        );
    }

    /// Sonderfall C: Deckt Topf 1 die Ausgaben dieses Monats nicht (`pay` sind bereits abgebucht), hängt der Ablauf vom
    /// Renditekriterium ab.
    /// **C1** (erfüllt): wie die Januar-Auffüllung (Topf 1 aus Topf 2, Rest aus Topf 3, dann Topf 2 aus Topf 3), die Ziele um
    /// die seit Januar vergangenen Monate gekürzt, das Ziel von Topf 1 plus die einmaligen Ausgaben `one_off`; keine
    /// Überschussbuchung. Deckt Topf 1 danach die Ausgaben immer noch nicht, folgt C2.
    /// **C2** (nicht erfüllt): Topf 1 wird aus Topf 2, dann Topf 3 so aufgefüllt, dass er die Ausgaben deckt und danach
    /// noch (Min(Reserve, Monate bis Januar) − 1) Monatsbedarfe enthält; Topf 2 wird nicht aufgefüllt.
    /// Reicht das nicht, wird eine Immobilie notverkauft (Erlös in Topf 3, dann C erneut); ohne Immobilie beginnt die
    /// Phase 8 (Topf 1 darf negativ sein, Topf 2 und 3 sind stillgelegt).
    fn cover_payments(&mut self, b: &Buckets, m: YearMonth, k: usize, pay: f64, one_off: f64) {
        for _ in 0..=self.props.len() {
            if self.cash - self.reserved >= -0.005 {
                return;
            }
            let (ret, _) = self.stock_return(k);
            if ret >= b.refill_threshold {
                // C1: Ablauf der Januar-Auffüllung auf dem Stand vor den Ausgaben
                let passed = m.month() - 1;
                self.cash -= self.reserved;
                self.cash += pay;
                let before = [self.cash, self.bonds, self.stocks];
                let why = format!("Vorausschau: Topf 1 deckt die Ausgaben des Monats nicht, Renditekriterium erfüllt ({passed} Monate seit Januar)");
                self.fill_pots(
                    b,
                    m,
                    k,
                    ("C1", why, passed, false, pay),
                    (one_off, false),
                    (before, vec![]),
                );
                self.cash -= pay;
                self.cash += self.reserved;
                if self.cash - self.reserved >= -0.005 {
                    return;
                }
            }
            // C2 (auch als Rückfall, wenn C1 nicht reicht)
            let free = self.cash - self.reserved;
            let month_need = self.basis.as_ref().map_or(0.0, NeedBasis::need) / 12.0;
            let months_to_jan = f64::from(13 - m.month());
            let keep = (b.reserve_months.min(months_to_jan) - 1.0).max(0.0) * month_need;
            let want = keep - free;
            let before = [free + pay, self.bonds, self.stocks];
            let mut moves: Vec<BucketMove> = vec![];
            self.bucket_move(2, 1, want.min(self.bonds.max(0.0)), &mut moves);
            let rest = (keep - (self.cash - self.reserved)).max(0.0);
            self.bucket_move(3, 1, rest.min(self.stocks.max(0.0)), &mut moves);
            let free_after = self.cash - self.reserved;
            if !moves.is_empty() {
                let text = format!(
                    "3-Topf C2 Auffüllung Topf 1 vor der Zahlung (Topf 1 deckt die Ausgaben {} nicht): Topf 1/2/3 vorher {} / {} / {}, nachher {} / {} / {}",
                    self.money(pay),
                    self.money(before[0]),
                    self.money(before[1]),
                    self.money(before[2]),
                    self.money(free_after + pay),
                    self.money(self.bonds),
                    self.money(self.stocks)
                );
                let t = self.target_pair(b, 0, 0.0);
                let after = [free_after + pay, self.bonds, self.stocks];
                let why = format!("Vorausschau: Topf 1 deckt die Ausgaben des Monats nicht, Renditekriterium nicht erfüllt ({months_to_jan} Monate bis Januar)");
                self.log_buckets(b, m, k, "C2", why, 0, false, t, moves, pay, before, after, text);
            }
            if free_after >= -0.005 {
                return;
            }
            // Topf 2 und 3 reichen nicht: Notverkauf einer Immobilie, sonst Phase 8
            if let Some(i) = self.props.iter().position(|p| p.active) {
                self.props[i].emergency_reason = Some("Topf 1, 2 und 3 reichen für die Ausgaben nicht".into());
                self.sell(i, m, EventKind::EmergencySale);
                if self.reserved > 0.0 {
                    self.reserved = 0.0;
                    self.reserved_sale = None;
                }
                self.flush_inflow(b, m, k, pay, pay);
            } else {
                self.phase8 = true;
                let text = format!(
                    "3-Topf: alle Töpfe leer, keine Immobilie mehr – Topf 1 ist im Minus ({}), Topf 2 und 3 sind stillgelegt",
                    self.money(self.cash)
                );
                self.event(m, EventKind::Buckets, text);
                return;
            }
        }
    }

    /// 3-Topf-Strategie für diesen Monat, nach Renditen und Steuern. `pay` sind die Ausgaben des Monats (laufende und
    /// einmalige), die schon von `cash` abgezogen sind. Die Entscheidungen gelten für den Stand vor diesen Ausgaben
    /// (Reihenfolge: Einnahmen, Käufe, Zuflüsse B, Januar-Auffüllung oder A, Vorausschau C, Ausgaben); die Umbuchungen
    /// selbst folgen erst nach den Renditen.
    fn bucket_step(&mut self, b: &Buckets, m: YearMonth, k: usize, pay: f64, one_off: f64, housing_now: f64) {
        // Phase 8: nur Topf 1 zählt; sobald das Vermögen wieder positiv ist, beginnt A neu
        let mut collect = false;
        if self.phase8 {
            if self.cash - self.reserved < 0.0 {
                return;
            }
            self.phase8 = false;
            self.a_reasons
                .push("Vermögen wieder positiv (Neustart nach Phase 8)".into());
            collect = true;
        }
        if !self.buckets_on {
            collect = true;
            self.buckets_on = true;
            self.a_reasons.push("Start der Entnahmephase".into());
        }
        // Stand vor den Ausgaben des Monats; die Reservierung gehört zu keinem Topf
        self.cash -= self.reserved;
        self.cash += pay;
        self.flush_inflow(b, m, k, pay, 0.0);
        let a = !self.a_reasons.is_empty();
        if a || m.month() == 1 {
            let (label, passed, trigger) = if a {
                (format!("ab {:02}.{}", m.month(), m.year()), m.month() - 1, "A")
            } else {
                ("Januar, geplant".to_string(), 0, "Januar")
            };
            let source = if a && passed > 0 {
                format!("{label}, auf 12 Monate hochgerechnet")
            } else {
                label
            };
            self.basis = Some(self.plan_basis(m, housing_now, source));
            let before = [self.cash, self.bonds, self.stocks];
            let mut moves: Vec<BucketMove> = vec![];
            if collect {
                // Gesamtes liquides Vermögen zuerst in Topf 3
                self.bucket_move(1, 3, self.cash.max(0.0), &mut moves);
                self.bucket_move(2, 3, self.bonds.max(0.0), &mut moves);
            }
            let reason = if a {
                self.a_reasons.join(", ")
            } else {
                "Januar".to_string()
            };
            self.fill_pots(
                b,
                m,
                k,
                (trigger, reason, passed, a, pay),
                (one_off, m.month() == 1),
                (before, moves),
            );
        }
        self.cash -= pay;
        self.cash += self.reserved;
        self.cover_payments(b, m, k, pay, one_off);
    }

    /// Lebenshaltung eines Monats nach der Ausgabenregel: Entnahme / 12 + Sparen + Einkommen und Renten − Wohnkosten,
    /// begrenzt durch Unter- und Obergrenze (Anteile der geplanten Ausgaben).
    fn rule_living(&self, d: &Dynamic, m: YearMonth, income: f64, housing: f64) -> f64 {
        let s = self.s;
        let planned = s.planned_living(m) * self.infl_idx;
        let mut l = (self.budget / 12.0 + s.savings.at(m) + income - housing).max(0.0);
        if let Some(c) = d.ceiling {
            l = l.min(c * planned);
        }
        if let Some(f) = d.floor {
            l = l.max(f * planned);
        }
        l
    }

    /// Jährliche Entnahme bei Pensionierung festlegen und jeden Januar anpassen.
    fn update_budget(&mut self, d: &Dynamic, m: YearMonth, infl_factor: f64) {
        // #6: Ein für einen Kauf reservierter Verkaufserlös zählt nicht zum Vermögen der Ausgabenregel.
        let liquid = (self.cash + self.stocks + self.bonds - self.reserved).max(0.0);
        if !self.budget_on {
            self.budget_on = true;
            self.strategy_started = true;
            self.budget_rate0 = match d.rule {
                Rule::Fixed { rate } => rate,
                Rule::Guardrails { initial, .. } => initial,
                Rule::Vpw {
                    horizon_age,
                    real_return,
                } => vpw_rate(real_return, horizon_age - self.s.age_at(m)),
                Rule::Endowment { rate, .. } => rate,
            };
            self.budget = self.budget_rate0 * liquid;
            self.event(
                m,
                EventKind::Withdrawal,
                format!(
                    "Entnahme {} pro Jahr ({:.1} % von {})",
                    self.money(self.budget),
                    self.budget_rate0 * 100.0,
                    self.money(liquid)
                ),
            );
            return;
        }
        if m.month() != 1 && !self.apply_now {
            return;
        }
        match d.rule {
            Rule::Fixed { rate } => self.budget = rate * liquid,
            Rule::Vpw {
                horizon_age,
                real_return,
            } => {
                self.budget = vpw_rate(real_return, horizon_age - self.s.age_at(m)) * liquid;
            }
            Rule::Endowment { rate, smoothing } => {
                self.budget = (1.0 - smoothing) * self.budget * infl_factor + smoothing * rate * liquid;
            }
            Rule::Guardrails {
                upper, lower, adjust, ..
            } => {
                let current = |b: f64| if liquid > 0.0 { b / liquid } else { f64::INFINITY };
                // Nach einem Verlustjahr keine Teuerungsanpassung, solange die Rate über der Startrate liegt.
                if !(self.stock_prev_year < 0.0 && current(self.budget) > self.budget_rate0) {
                    self.budget *= infl_factor;
                }
                let rate = current(self.budget);
                if rate > self.budget_rate0 * (1.0 + upper) {
                    self.budget *= 1.0 - adjust;
                    self.event(
                        m,
                        EventKind::Withdrawal,
                        format!(
                            "Leitplanke: Entnahme um {:.0} % gekürzt auf {} pro Jahr (Rate {:.1} %)",
                            adjust * 100.0,
                            self.money(self.budget),
                            rate * 100.0
                        ),
                    );
                } else if rate < self.budget_rate0 * (1.0 - lower) {
                    self.budget *= 1.0 + adjust;
                    self.event(
                        m,
                        EventKind::Withdrawal,
                        format!(
                            "Leitplanke: Entnahme um {:.0} % erhöht auf {} pro Jahr (Rate {:.1} %)",
                            adjust * 100.0,
                            self.money(self.budget),
                            rate * 100.0
                        ),
                    );
                }
            }
        }
    }

    fn execute(mut self) -> SimResult {
        let s = self.s;
        // Startaufteilung mit den Werten des Startmonats.
        {
            let m = s.start;
            let mut running = 0.0;
            for (i, p) in s.props.iter().enumerate() {
                if !self.props[i].active {
                    continue;
                }
                running += p.fix_costs;
                for (j, t) in p.tranches.iter().enumerate() {
                    if let Some(x) = t.periods.iter().find(|x| x.start <= m && m < x.end) {
                        running += self.props[i].balances[j] * x.rate(self.path.saron[0]) / 12.0 + x.amort_q / 3.0;
                    }
                }
            }
            // #54: Vor der Pensionierung gilt keine Entnahmestrategie; die Bestände bei Simulationsstart werden
            // unverändert übernommen. Ist schon jemand pensioniert, teilt die Strategie wie bisher auf.
            if s.people.iter().any(|p| p.pk_at.is_some_and(|a| a <= m)) {
                let gap = (s.planned_living(m) + running - s.earned(m, self.infl_idx)).max(0.0);
                self.rebalance(gap * s.reserve.at(m), s.stock_share.at(m).clamp(0.0, 1.0));
            }
        }
        {
            let liquid = self.cash + self.stocks + self.bonds;
            let p3a: f64 = self.p3a.iter().sum();
            let pk: f64 = self.people.iter().map(|p| p.pk).sum();
            self.summary.start_liquid = liquid;
            self.summary.start_total = liquid + self.equity() + pk + p3a;
        }

        for k in 0..s.months {
            let m = s.start.plus(k as i32);
            let december = m.is_december();
            // Neuer Monat: Anlässe für die sofortige Anwendung der Entnahmestrategie sammeln; ein im Vormonat
            // reservierter Verkaufserlös wird nach dem Kauf (unten, danach Strategie) auf den Überschuss angewendet.
            self.apply_reasons.clear();
            self.a_reasons.clear();
            self.month_out = 0.0;
            self.ledger = Ledger::default();
            self.inflow_log.clear();
            self.strategy_started = false;
            self.reserved_cash = 0.0;
            let mut leftover = false;
            if self.reserved_sale == Some(m.plus(-1)) {
                self.apply_reasons
                    .push("Überschuss aus dem Verkaufserlös nach dem Kauf".into());
                self.reserved_sale = None;
                leftover = true;
            }
            let mut month_tax = 0.0;
            self.month_capital_tax = 0.0;
            self.month_gain_tax = 0.0;
            let mut infl_factor = 1.0;
            // Teuerung monatlich mit der Inflation des Vormonats. Beträge gelten in Preisen des
            // Startmonats. AHV und Entnahmebudget folgen im Januar mit der aufgelaufenen Teuerung.
            if k > 0 {
                let before = self.active_worth();
                self.infl_idx *= (1.0 + self.path.inflation[k - 1]).max(0.0).powf(1.0 / 12.0);
                self.ledger.revaluation += self.active_worth() - before;
            }
            if m.month() == 1 && k > 0 {
                let factor = self.infl_idx / self.infl_idx_jan;
                self.infl_idx_jan = self.infl_idx;
                infl_factor = factor;
                // #145: Der Plafond steigt einmal je Januar, sobald mindestens eine angepasste Rente läuft (nicht je Person).
                let mut indexed = false;
                for (st, c) in self.people.iter_mut().zip(&s.people) {
                    if st.ahv_started && c.ahv_indexed {
                        st.ahv *= factor;
                        indexed = true;
                    }
                }
                if indexed {
                    self.ahv_scale *= factor;
                }
            }

            // Ereignisse
            let mut extras_out = 0.0;
            for (em, amount, label) in &s.extras {
                if *em == m {
                    self.cash += amount;
                    self.ledger.extras += amount;
                    if *amount < 0.0 {
                        extras_out -= amount;
                    }
                    if *amount > 0.0 {
                        self.mark_inflow(*amount, &format!("«{label}»"));
                        self.apply_reasons.push(format!("Mittelzufluss «{label}»"));
                    }
                    let text = format!("{label}: {}", self.money(*amount));
                    self.event(m, EventKind::Extra, text);
                }
            }
            for pi in 0..self.people.len() {
                let c = &s.people[pi];
                if !self.people[pi].retired && c.pk_at.is_some_and(|a| m >= a) {
                    let (pk, accrued) = {
                        let st = &mut self.people[pi];
                        st.retired = true;
                        let accrued = std::mem::take(&mut st.pk_accrued);
                        st.pk += accrued;
                        (st.pk, accrued)
                    };
                    self.ledger.pk_interest += accrued;
                    let lump = pk * c.pk_lumpsum;
                    let t = self.capital_tax(lump);
                    self.ledger.taxes += t;
                    self.ledger.annuity += pk - lump;
                    month_tax += t;
                    self.month_capital_tax += t;
                    self.cash += lump - t;
                    if lump - t > 0.0 {
                        self.apply_reasons.push("Pensionskassenauszahlung".into());
                        self.a_reasons.push("PK-Kapitalbezug".into());
                        self.mark_inflow(lump - t, "PK-Kapitalbezug");
                    }
                    let pension = pk * (1.0 - c.pk_lumpsum) * c.pk_conversion / 12.0;
                    self.people[pi].pk_pension = pension;
                    self.people[pi].pk = 0.0;
                    self.summary.lumpsum_net += lump - t;
                    self.summary.lumpsum_tax += t;
                    self.summary.pension_fund_monthly += pension;
                    let who = if s.people.len() > 1 {
                        format!(" ({})", if pi == 0 { "Person 1" } else { "Person 2" })
                    } else {
                        String::new()
                    };
                    self.event(
                        m,
                        EventKind::Retirement,
                        format!(
                            "Bezug Pensionskasse{who}: Kapitalbezug {} (Steuer {}), Rente {} / Monat",
                            self.money(lump),
                            self.money(t),
                            self.money(pension)
                        ),
                    );
                }
                if !self.people[pi].ahv_started && c.ahv_at.is_some_and(|a| m >= a) {
                    self.people[pi].ahv_started = true;
                    self.people[pi].ahv = c.ahv_monthly;
                    let who = if s.people.len() > 1 {
                        format!(" ({})", if pi == 0 { "Person 1" } else { "Person 2" })
                    } else {
                        String::new()
                    };
                    self.event(
                        m,
                        EventKind::AhvStart,
                        format!("AHV-Rente{who} {} / Monat", self.money(c.ahv_monthly)),
                    );
                }
            }
            let (prop_cost, prop_running) = self.properties(k, m);
            // #52: Einzahlung in die Säule 3a zählt zu den Ausgaben (wie die Wohnkosten).
            let p3a_in = self.pillar3a(m);
            self.month_p3a_in = p3a_in;
            if leftover {
                // Überschuss der Reservierung nach dem Kauf: in Topf 3
                let rest = std::mem::take(&mut self.reserved);
                self.mark_inflow_from(4, rest, "Überschuss der Reservierung nach dem Kauf");
            }
            // Reihenfolge im Monat: Zuflüsse, Käufe/Verkäufe (oben), dann die Strategie; einmal pro Monat, auch im Januar.
            self.apply_now = !self.apply_reasons.is_empty();

            // Geldflüsse
            let earned = s.earned(m, self.infl_idx);
            let pk_pension = self.pk_pension();
            let ahv = self.ahv();
            let income = earned + pk_pension + ahv;
            let planned = s.planned_living(m) * self.infl_idx;
            // #6/#47: Die Ausgabenregel gilt ab der Pensionierung der ersten Person.
            let living = match s.dynamic {
                Some(d) if self.any_retired() => {
                    self.update_budget(&d, m, infl_factor);
                    self.rule_living(&d, m, income, prop_cost + p3a_in)
                }
                _ => planned,
            };
            self.month_running = prop_running;
            self.cash += s.savings.at(m) + income - living - prop_cost - p3a_in;
            self.ledger.income += income;
            self.ledger.savings += s.savings.at(m);
            self.ledger.living += living;
            self.ledger.housing += prop_cost;
            self.year_income += income;

            // Rendite
            let wealth = self.cash + self.stocks + self.bonds;
            if wealth > 0.0 {
                self.ledger.market += self.stocks * self.path.stocks[k] + self.bonds * self.path.bonds[k];
                self.stocks *= 1.0 + self.path.stocks[k];
                self.bonds *= 1.0 + self.path.bonds[k];
            }
            self.stock_idx *= 1.0 + self.path.stocks[k];
            self.stock_hist.push(self.stock_idx);
            self.bond_idx *= 1.0 + self.path.bonds[k];
            self.stock_year *= 1.0 + self.path.stocks[k];
            if december {
                self.stock_prev_year = self.stock_year - 1.0;
                self.stock_year = 1.0;
            }

            // Pensionskasse: Zins monatlich auf dem Guthaben zu Monatsbeginn, gutgeschrieben im Dezember
            for (st, c) in self.people.iter_mut().zip(&s.people) {
                if !st.retired {
                    st.pk_accrued += st.pk * c.pk_interest.at(m) / 12.0;
                    st.pk += c.pk_contribution.at(m);
                    self.ledger.pk_contributions += c.pk_contribution.at(m);
                    if december {
                        self.ledger.pk_interest += st.pk_accrued;
                        st.pk += st.pk_accrued;
                        st.pk_accrued = 0.0;
                    }
                }
            }

            // Steuern
            let mut paid = 0.0;
            let (mut paid_income, mut paid_wealth) = (0.0, 0.0);
            let (mut assessed_income, mut assessed_wealth) = (0.0, 0.0);
            if m.month() == 5 {
                paid += std::mem::take(&mut self.tax_pending);
                paid_income += std::mem::take(&mut self.pending_income);
                paid_wealth += std::mem::take(&mut self.pending_wealth);
            }
            if december {
                let t = &s.taxes;
                let taxable_wealth = wealth + self.equity();
                let (ti, tw) = (
                    tax::tax(&t.income, self.year_income, t.multiplier),
                    tax::tax(&t.wealth, taxable_wealth, t.multiplier),
                );
                self.tax_pending += ti + tw;
                self.pending_income += ti;
                self.pending_wealth += tw;
                (assessed_income, assessed_wealth) = (ti, tw);
                self.year_income = 0.0;
                self.year_capital = 0.0;
            }
            if k + 1 == s.months {
                paid += std::mem::take(&mut self.tax_pending);
                paid_income += std::mem::take(&mut self.pending_income);
                paid_wealth += std::mem::take(&mut self.pending_wealth);
            }
            self.cash -= paid;
            self.ledger.taxes += paid;
            month_tax += paid;
            month_tax += std::mem::take(&mut self.event_tax);
            self.summary.taxes_total += month_tax;

            // Notverkauf prüfen (mit den Renten nach einer allfälligen Indexierung)
            let liquid = self.cash + self.stocks + self.bonds;
            let shortfall = living + prop_running + p3a_in - (earned + self.pk_pension() + self.ahv());
            if liquid - s.emergency.at(m) * shortfall < 0.0 && !self.props.iter().any(|p| p.emergency_pending) {
                if let Some(ps) = self.props.iter_mut().find(|p| p.active) {
                    ps.emergency_pending = true;
                    ps.emergency_reason = Some("Notverkaufskriterium".into());
                }
            }

            // Entnahmestrategie: «3-Topf» ab Pensionierung, sonst «Standard»
            match &s.buckets {
                Some(b) if self.any_retired() => {
                    let one_off = extras_out + self.month_out;
                    let pay = living + prop_cost + p3a_in + paid + one_off;
                    let housing_now = (prop_running - self.month_amort).max(0.0);
                    self.bucket_step(b, m, k, pay, one_off, housing_now);
                }
                _ => {
                    // #54: Vor der Pensionierung hält die App den Cash-Betrag der Planung (mit der Teuerung), der
                    // Überschuss geht ins Portfolio, ein Fehlbetrag wird aus dem Portfolio nachgefüllt. Ab der
                    // Pensionierung: Reserve der Strategie («Standard» wie erfasst, sonst 3 Monate).
                    // Ein für den Kauf im Folgemonat reservierter Verkaufserlös bleibt liquide (nicht angelegt).
                    let target = if self.any_retired() {
                        s.reserve.at(m) * shortfall.max(0.0)
                    } else {
                        s.cash0 * self.infl_idx
                    };
                    let reserve = target + self.reserved_cash;
                    self.rebalance(reserve, s.stock_share.at(m).clamp(0.0, 1.0));
                }
            }
            // Zusätzliche Anwendung ausserhalb des Januars protokollieren (im Januar wendet die Strategie ohnehin an).
            if self.apply_now && m.month() != 1 && !self.strategy_started && s.dynamic.is_some() && self.any_retired() {
                let text = format!("Entnahmestrategie angewendet nach {}", self.apply_reasons.join(", "));
                self.event(m, EventKind::Withdrawal, text);
            }

            // Auswertung (ein Notverkauf in der 3-Topf-Strategie hat das Cash verändert)
            let liquid = self.cash + self.stocks + self.bonds;
            let equity = self.equity();
            let p3a: f64 = self.p3a.iter().sum();
            let pk: f64 = self.people.iter().map(|p| p.pk).sum();
            let total = liquid + equity + pk + p3a;
            if liquid < self.summary.min_liquid {
                self.summary.min_liquid = liquid;
                self.summary.min_liquid_month = Some(m);
            }
            // Aufgebraucht: Schulden oder keine Mittel mehr für ungedeckte Ausgaben.
            if (liquid < -0.005 || (liquid <= 0.005 && shortfall > 0.0))
                && self.summary.depleted.is_none()
                && !self.props.iter().any(|p| p.active)
            {
                self.summary.depleted = Some(m);
                self.event(m, EventKind::Depleted, "Vermögen aufgebraucht".into());
            }
            if s.retire_at() == Some(m) {
                self.summary.total_at_retirement = Some(total);
            }
            self.summary.end_total = total;
            self.summary.end_total_real = total / self.infl_idx;
            self.summary.end_liquid = liquid;
            if self.record {
                self.records.push(MonthRecord {
                    month: m,
                    age: s.age_at(m),
                    cash: self.cash,
                    stocks: self.stocks,
                    bonds: self.bonds,
                    liquid,
                    property_equity: equity,
                    pension_fund: pk,
                    pillar3a: p3a,
                    total,
                    pillar3a_in: p3a_in,
                    income,
                    spending: living + prop_cost + p3a_in,
                    taxes: month_tax,
                    tax_income: paid_income,
                    tax_wealth: paid_wealth,
                    tax_capital: self.month_capital_tax,
                    tax_gain: self.month_gain_tax,
                    assessed_income,
                    assessed_wealth,
                    planned_spending: planned,
                    withdrawal: living + prop_cost + p3a_in - income,
                    earned,
                    pk_pension,
                    ahv,
                    housing: prop_cost,
                    property_value: (0..self.props.len())
                        .filter(|&i| self.props[i].active)
                        .map(|i| self.worth(i))
                        .sum(),
                    mortgage: self
                        .props
                        .iter()
                        .filter(|p| p.active)
                        .map(|p| p.balances.iter().sum::<f64>())
                        .sum(),
                    price_index: self.infl_idx,
                    stock_index: self.stock_idx,
                    bond_index: self.bond_idx,
                    ledger: self.ledger,
                });
            }
        }
        SimResult {
            label: self.path.label.clone(),
            summary: self.summary,
            events: self.events,
            renewals: self.renewals,
            property_events: self.property_events,
            bucket_refills: self.bucket_refills,
            records: self.records,
            pillar3a: self.p3a_view,
        }
    }
}

/// Indexstände des Vorlaufs (#103): der Stand vor dem ersten Vorlaufmonat, dann je Vorlaufmonat; der letzte ist 1 (Stand
/// beim Start). Platz für `months` weitere Monate.
fn pre_levels(pre: &[f64], months: usize) -> Vec<f64> {
    let mut levels = Vec::with_capacity(pre.len() + 1 + months);
    levels.push(1.0);
    for r in pre.iter().rev() {
        let last = *levels.last().unwrap_or(&1.0);
        levels.push(last / (1.0 + r).max(1e-9));
    }
    levels.reverse();
    levels
}

#[cfg(test)]
mod tests_withdrawal {
    use super::vpw_rate;

    #[test]
    fn vpw_rate_is_annuity_factor() {
        assert!((vpw_rate(0.0, 25.0) - 0.04).abs() < 1e-12);
        // 30 Jahre bei 3 %: bekannter Annuitätenfaktor 0.0510
        assert!((vpw_rate(0.03, 30.0) - 0.051019).abs() < 1e-5);
        // im letzten Jahr wird alles entnommen, nie mehr als das Vermögen
        assert!(vpw_rate(0.02, 0.0) <= 1.0 && vpw_rate(0.02, 0.5) <= 1.0);
        assert!(vpw_rate(0.02, 20.0) > vpw_rate(0.02, 30.0));
    }
}

#[cfg(test)]
mod tests_schedule {
    use super::schedule;
    use crate::model::Tranche;
    use crate::time::YearMonth;

    fn tranche(periods: &[(f64, f64)], after: &str) -> Tranche {
        let periods: Vec<_> = periods
            .iter()
            .map(|(term, rate)| serde_json::json!({"type": "fixed", "term": term, "rate": rate}))
            .collect();
        serde_json::from_value(serde_json::json!({"name": "T", "amount": 1000, "after": after, "periods": periods}))
            .unwrap()
    }

    // #40: Perioden bis vor `limit`, Verlängerung bzw. Rückzahlung nach der letzten Periode.
    #[test]
    fn periods_until_the_limit() {
        let a = YearMonth::new(2030, 1);
        let s = schedule(
            &tranche(&[(1.0, 0.01), (2.0, 0.02)], "renew"),
            a,
            YearMonth::new(2036, 1),
        );
        let ends: Vec<_> = s.periods.iter().map(|p| (p.start, p.end, p.index, p.rate)).collect();
        assert_eq!(
            ends,
            vec![
                (a, YearMonth::new(2031, 1), 0, 0.01),
                (YearMonth::new(2031, 1), YearMonth::new(2033, 1), 1, 0.02),
                (YearMonth::new(2033, 1), YearMonth::new(2035, 1), 2, 0.02),
                (YearMonth::new(2035, 1), YearMonth::new(2037, 1), 3, 0.02),
            ]
        );
        assert_eq!(s.repay_at, None);
        // Eine Periode, die genau an der Grenze endet: keine weitere
        let s = schedule(&tranche(&[(1.0, 0.01)], "renew"), a, YearMonth::new(2032, 1));
        assert_eq!(s.periods.len(), 2);
        // Rückzahlung: am Ende der letzten Periode, wenn sie vor der Grenze liegt
        let s = schedule(
            &tranche(&[(1.0, 0.01), (2.0, 0.02)], "repay"),
            a,
            YearMonth::new(2036, 1),
        );
        assert_eq!((s.periods.len(), s.repay_at), (2, Some(YearMonth::new(2033, 1))));
        let s = schedule(
            &tranche(&[(1.0, 0.01), (2.0, 0.02)], "repay"),
            a,
            YearMonth::new(2033, 1),
        );
        assert_eq!(s.repay_at, None);
        // Höchstens 1000 Perioden
        let s = schedule(&tranche(&[(1.0 / 12.0, 0.01)], "renew"), a, YearMonth::new(2200, 1));
        assert_eq!(s.periods.len(), 1000);
        assert!(schedule(&tranche(&[], "renew"), a, YearMonth::new(2036, 1))
            .periods
            .is_empty());
    }
}

#[cfg(test)]
mod tests_pre_levels {
    use super::pre_levels;

    #[test]
    fn issue_103_pre_levels_walk_back_from_the_start_level() {
        // Vorlauf +10 %, dann +20 %: Start = 1, davor 1/1.2, davor 1/1.2/1.1.
        let l = pre_levels(&[0.10, 0.20], 3);
        assert_eq!(l.len(), 3);
        assert!((l[2] - 1.0).abs() < 1e-12);
        assert!((l[1] - 1.0 / 1.2).abs() < 1e-12);
        assert!((l[0] - 1.0 / 1.2 / 1.1).abs() < 1e-12);
        assert_eq!(pre_levels(&[], 3), vec![1.0]);
    }

    #[test]
    fn issue_103_pre_levels_total_loss_stays_finite() {
        // Rendite -100 % darf nicht durch 0 teilen.
        let l = pre_levels(&[-1.0], 1);
        assert_eq!(l.len(), 2);
        assert!(l[0].is_finite() && l[0] > 1e6);
        assert_eq!(l[1], 1.0);
    }
}
