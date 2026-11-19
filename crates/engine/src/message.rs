//! #171: Messages of the engine for the user interface. Every message has a stable code (English, snake_case) and
//! parameters (numbers, names, months and nested messages as values, never formatted text); the German text is
//! rendered from the template of the code. The user interface translates by the code (key `engine.<code>` in
//! `crates/app/ui/i18n/<language>.json`, formatted by `engineText` in `util.js`) and shows the text for an unknown
//! code; the command line and the golden files use the text.
//!
//! Placeholders in a template: `{name}` or `{name:format}`.
//! - without a format: a text as is, a number as Rust and JavaScript print it, a list joined by ", ", a nested
//!   message rendered from its own template;
//! - `int`: rounded (half away from zero), thousands separator `'` (12'500);
//! - `fix0`, `fix1`, `fix2`: fixed number of decimals (as `{:.1}`);
//! - `pct0`, `pct1`, `pct2`: a fraction × 100 with a fixed number of decimals, without « %»;
//! - `ym`: a month `YYYY-MM` shown as `MM.YYYY`;
//! - `lines`: a list, one entry per line.
//!
//! A new message: add the code with its German template to the table at the end of this file and the same key
//! `engine.<code>` to `crates/app/ui/i18n/de.json`; then `MESSAGES_UPDATE=1 cargo nextest run -p vivaplan-engine -E
//! 'test(issue_171_ak3)'` writes the examples for the test of the user interface (`e2e/unit/engine-messages.json`).

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::fmt;
use std::sync::OnceLock;

use crate::time::YearMonth;

/// Value of a parameter. Lighter than `serde_json::Value` (no map per message): a run with many paths creates many
/// events.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Arg {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Num(f64),
    Text(String),
    Msg(Box<Message>),
    List(Vec<Arg>),
}

/// Serialized as JSON value; a nested message as `{code, params, text}`.
impl Serialize for Arg {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Arg::Null => s.serialize_none(),
            Arg::Bool(b) => s.serialize_bool(*b),
            Arg::Int(i) => s.serialize_i64(*i),
            Arg::Num(x) => s.serialize_f64(*x),
            Arg::Text(t) => s.serialize_str(t),
            Arg::Msg(m) => m.serialize(s),
            Arg::List(l) => {
                let mut q = s.serialize_seq(Some(l.len()))?;
                for x in l {
                    q.serialize_element(x)?;
                }
                q.end()
            }
        }
    }
}

/// Conversion of a value into a parameter (used by [`msg!`] through [`arg`]).
pub trait ToArg {
    fn to_arg(&self) -> Arg;
}

macro_rules! int_args {
    ($($t:ty),*) => {$(
        impl ToArg for $t {
            fn to_arg(&self) -> Arg {
                Arg::Int(*self as i64)
            }
        }
    )*};
}
int_args!(i32, i64, u8, u32, u64, usize);

impl ToArg for f64 {
    fn to_arg(&self) -> Arg {
        Arg::Num(*self)
    }
}
impl ToArg for bool {
    fn to_arg(&self) -> Arg {
        Arg::Bool(*self)
    }
}
impl ToArg for str {
    fn to_arg(&self) -> Arg {
        Arg::Text(self.to_string())
    }
}
impl ToArg for String {
    fn to_arg(&self) -> Arg {
        Arg::Text(self.clone())
    }
}
/// A month as `YYYY-MM` (format `ym` shows `MM.YYYY`).
impl ToArg for YearMonth {
    fn to_arg(&self) -> Arg {
        Arg::Text(self.iso())
    }
}
impl ToArg for Message {
    fn to_arg(&self) -> Arg {
        Arg::Msg(Box::new(self.clone()))
    }
}
impl ToArg for Arg {
    fn to_arg(&self) -> Arg {
        self.clone()
    }
}
impl<T: ToArg> ToArg for Option<T> {
    fn to_arg(&self) -> Arg {
        self.as_ref().map_or(Arg::Null, ToArg::to_arg)
    }
}
impl<T: ToArg> ToArg for [T] {
    fn to_arg(&self) -> Arg {
        Arg::List(self.iter().map(ToArg::to_arg).collect())
    }
}
impl<T: ToArg, const N: usize> ToArg for [T; N] {
    fn to_arg(&self) -> Arg {
        self[..].to_arg()
    }
}
impl<T: ToArg> ToArg for Vec<T> {
    fn to_arg(&self) -> Arg {
        self[..].to_arg()
    }
}
impl<T: ToArg + ?Sized> ToArg for &T {
    fn to_arg(&self) -> Arg {
        (**self).to_arg()
    }
}

/// Parameters of a message (name → value), in the order given.
pub type Params = Vec<(&'static str, Arg)>;

/// A message: code, parameters and the German text rendered from them. The text is rendered on first use (a run
/// with many paths creates many events whose texts are never shown).
#[derive(Clone, Debug)]
pub struct Message {
    code: Code,
    params: Params,
    text: OnceLock<String>,
}

/// Equal code and parameters (the text follows from them).
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.params == other.params
    }
}

impl Message {
    pub fn new(code: Code, params: Params) -> Self {
        Message {
            code,
            params,
            text: OnceLock::new(),
        }
    }
    pub fn code(&self) -> &'static str {
        self.code.as_str()
    }
    /// The parameters as a JSON object when serialized (`{name: value}`).
    pub fn params(&self) -> ParamsMap<'_> {
        ParamsMap(&self.params)
    }
    /// The parameters in the order given.
    pub fn args(&self) -> &[(&'static str, Arg)] {
        &self.params
    }
    /// Value of a parameter.
    pub fn param(&self, name: &str) -> Option<&Arg> {
        self.params.iter().find(|(k, _)| *k == name).map(|(_, v)| v)
    }
    pub fn text(&self) -> &str {
        self.text.get_or_init(|| render(self.code.template(), &self.params))
    }
    /// The German text (like `String::as_str`).
    pub fn as_str(&self) -> &str {
        self.text()
    }
}

/// Parameters, serialized as a JSON object.
#[derive(Clone, Copy, Debug)]
pub struct ParamsMap<'a>(pub &'a [(&'static str, Arg)]);

impl Serialize for ParamsMap<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0 {
            m.serialize_entry(k, v)?;
        }
        m.end()
    }
}

/// Serialized as `{ "code", "params", "text" }`.
impl Serialize for Message {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(Some(3))?;
        m.serialize_entry("code", self.code.as_str())?;
        m.serialize_entry("params", &ParamsMap(&self.params))?;
        m.serialize_entry("text", self.text())?;
        m.end()
    }
}

/// The parameters of a message as a JSON object (for tests and the user interface).
pub fn params_json(params: &[(&'static str, Arg)]) -> serde_json::Value {
    serde_json::to_value(ParamsMap(params)).unwrap_or_default()
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.text())
    }
}

impl std::ops::Deref for Message {
    type Target = str;
    fn deref(&self) -> &str {
        self.text()
    }
}

impl PartialEq<str> for Message {
    fn eq(&self, other: &str) -> bool {
        self.text() == other
    }
}

impl PartialEq<&str> for Message {
    fn eq(&self, other: &&str) -> bool {
        self.text() == *other
    }
}

impl PartialEq<String> for Message {
    fn eq(&self, other: &String) -> bool {
        self.text() == other
    }
}

impl From<Message> for String {
    fn from(m: Message) -> String {
        m.text().to_string()
    }
}

/// An empty message (default of result fields that always get a text).
impl Default for Message {
    fn default() -> Self {
        crate::msg!(empty)
    }
}

/// Value of a parameter (used by [`msg!`]).
pub fn arg<T: ToArg + ?Sized>(v: &T) -> Arg {
    v.to_arg()
}

/// `msg!(code, name = value, …)`: message with the code from the table and the parameters as values.
#[macro_export]
macro_rules! msg {
    ($code:ident $(, $k:ident = $v:expr)* $(,)?) => {{
        $crate::message::Message::new(
            $crate::message::Code::$code,
            vec![$((stringify!($k), $crate::message::arg(&$v))),*],
        )
    }};
}

/// Renders a template with the parameters (see the module documentation). A placeholder without a parameter stays.
pub fn render(template: &str, params: &[(&'static str, Arg)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let end = after.find('}');
        let inner = end.map(|j| &after[..j]);
        match inner.and_then(placeholder) {
            Some((name, format)) => {
                match params.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => value(v, format, &mut out),
                    None => {
                        debug_assert!(false, "parameter {name} missing in «{template}»");
                        out.push_str(&rest[i..i + inner.map_or(0, str::len) + 2]);
                    }
                }
                rest = &after[end.unwrap_or(0) + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// `name` or `name:format` (word characters as `\w` in JavaScript).
fn placeholder(s: &str) -> Option<(&str, &str)> {
    let word = |w: &str| !w.is_empty() && w.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    match s.split_once(':') {
        Some((name, format)) => (word(name) && word(format)).then_some((name, format)),
        None => word(s).then_some((s, "")),
    }
}

fn value(v: &Arg, format: &str, out: &mut String) {
    use std::fmt::Write;
    match v {
        Arg::Msg(m) => out.push_str(m.text()),
        Arg::List(l) => {
            let sep = if format == "lines" { "\n" } else { ", " };
            for (i, x) in l.iter().enumerate() {
                if i > 0 {
                    out.push_str(sep);
                }
                value(x, format, out);
            }
        }
        Arg::Text(s) if format == "ym" => match s.split_once('-') {
            Some((y, m)) => {
                let _ = write!(out, "{m}.{y}");
            }
            None => out.push_str(s),
        },
        Arg::Text(s) => out.push_str(s),
        Arg::Int(i) => number(*i as f64, format, &i.to_string(), out),
        Arg::Num(x) => number(*x, format, &x.to_string(), out),
        Arg::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Arg::Null => {}
    }
}

fn number(x: f64, format: &str, plain: &str, out: &mut String) {
    use std::fmt::Write;
    let _ = match format {
        "int" => write!(out, "{}", group(x)),
        "fix0" => write!(out, "{x:.0}"),
        "fix1" => write!(out, "{x:.1}"),
        "fix2" => write!(out, "{x:.2}"),
        "pct0" => write!(out, "{:.0}", x * 100.0),
        "pct1" => write!(out, "{:.1}", x * 100.0),
        "pct2" => write!(out, "{:.2}", x * 100.0),
        _ => write!(out, "{plain}"),
    };
}

/// Placeholders of a template: (name, format), in order of appearance.
pub fn placeholders(template: &str) -> Vec<(&str, &str)> {
    let mut out = vec![];
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        let after = &rest[i + 1..];
        match after.find('}').and_then(|j| placeholder(&after[..j]).map(|p| (j, p))) {
            Some((j, p)) => {
                out.push(p);
                rest = &after[j + 1..];
            }
            None => rest = after,
        }
    }
    out
}

/// #171/AK3: Example of a message with a value for every placeholder, chosen by its format (numbers for `int`,
/// `fix…` and `pct…`, a month for `ym`, a list for `lines`, otherwise a nested message). The test of the user
/// interface (`e2e/unit/engine-messages.test.js`) renders the examples with the text table and compares the text.
pub fn example(code: Code) -> Message {
    let mut params: Params = vec![];
    for (name, format) in placeholders(code.template()) {
        if params.iter().any(|(k, _)| *k == name) {
            continue;
        }
        let v = match format {
            "int" | "fix0" | "fix1" | "fix2" => Arg::Num(12345.678),
            "pct0" | "pct1" | "pct2" => Arg::Num(0.0371),
            "ym" => Arg::Text("2031-07".into()),
            "lines" => arg(&[msg!(person_n, n = 1), msg!(person_n, n = 2)]),
            _ => arg(&msg!(person_n, n = 2)),
        };
        params.push((name, v));
    }
    Message::new(code, params)
}

/// Rounded amount with thousands separator `'` («12'500», «-3'000»).
pub fn group(v: f64) -> String {
    let n = v.round() as i64;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('\'');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

macro_rules! codes {
    ($($code:ident => $de:expr,)*) => {
        /// Codes of the messages; the name is the code (`engine.<code>` in the text tables).
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Code {
            $($code,)*
        }
        impl Code {
            /// All codes (for the tests of the text tables).
            pub const ALL: &'static [Code] = &[$(Code::$code,)*];
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Code::$code => stringify!($code),)*
                }
            }
            /// German template.
            pub fn template(self) -> &'static str {
                match self {
                    $(Code::$code => $de,)*
                }
            }
            pub fn parse(s: &str) -> Option<Code> {
                match s {
                    $(stringify!($code) => Some(Code::$code),)*
                    _ => None,
                }
            }
        }
    };
}

// The table: code => German template. Sorted by area (comments); the user interface has the same keys under
// `engine.` in its text tables.
codes! {
    // General
    cancelled => "Abgebrochen",
    not_implemented => "Noch nicht umgesetzt: {what}",
    // Names (parameters of other messages)
    person_n => "Person {n}",
    milestone_start => "Simulationsstart",
    milestone_end => "Simulationsende",
    milestone_retire => "Bezug Pensionskasse",
    milestone_ahv => "AHV-Bezug",
    milestone_retire2 => "Bezug Pensionskasse Person 2",
    milestone_ahv2 => "AHV-Bezug Person 2",
    pillar3a => "Säule 3a",
    pillar3a_of => "Säule 3a {name}",
    term_pk_payout => "Bezug Pensionskasse",
    term_ahv_payout => "AHV-Bezug",
    term_pk_payout_2 => "Bezug Pensionskasse (Person 2)",
    term_ahv_payout_2 => "AHV-Bezug (Person 2)",
    term_pk_contribution => "Beiträge Pensionskasse",
    term_pk_interest => "Verzinsung Pensionskasse",
    term_pk_contribution_2 => "Beiträge Pensionskasse (Person 2)",
    term_pk_interest_2 => "Verzinsung Pensionskasse (Person 2)",
    term_income => "Einkommen",
    term_income_2 => "Einkommen (Person 2)",
    term_spending => "Ausgaben",
    term_savings => "Weitere Zuflüsse",
    term_stock_share => "Aktienanteil",
    term_reserve => "Cash-Reserve",
    term_stock_return => "Aktienrendite",
    term_bond_return => "Obligationenrendite",
    term_inflation => "Inflation",
    term_saron => "SARON",
    term_emergency_months => "Notverkauf-Schwelle",
    term_income_tax => "Einkommenssteuer",
    term_wealth_tax => "Vermögenssteuer",
    term_capital_tax => "Kapitalbezugssteuer",
    term_income_tax_married => "Einkommenssteuer (Verheiratete)",
    term_wealth_tax_married => "Vermögenssteuer (Verheiratete)",
    term_capital_tax_married => "Kapitalbezugssteuer (Verheiratete)",
    // Validation (validate.rs)
    labeled => "{what}: {problem}",
    guardrails_initial_rate => "Die Startrate muss zwischen 0 % und 100 % liegen.",
    guardrails_bands => "Die Leitplanken müssen grösser oder gleich 0 sein (unten kleiner als 100 %).",
    guardrails_adjust => "Die Anpassung muss zwischen 0 % und 100 % liegen.",
    spending_limits_negative => "Unter- und Obergrenze der Ausgaben dürfen nicht negativ sein.",
    spending_floor_above_ceiling => "Die Untergrenze der Ausgaben liegt über der Obergrenze.",
    persons_none => "Mindestens eine Person ist nötig.",
    persons_too_many => "Es sind höchstens zwei Personen möglich.",
    partner_without_person => "Die Angaben zur zweiten Person gehören zu einer zweiten Person im Haushalt.",
    partner_missing => "Die zweite Person hat weder Einkommen noch Vorsorge erfasst und bleibt unberücksichtigt.",
    person_name_missing => "Person {n}: Der Name fehlt (Pflichtangabe).",
    person_name_too_long => "Person {n}: Der Name darf höchstens {max} Zeichen lang sein.",
    ahv_cap_not_positive => "Die Plafonierung der AHV-Renten muss grösser als 0 sein.",
    milestones_start_end_missing => "Die Meilensteine «start» und «end» müssen vorhanden sein.",
    end_before_start => "Das Simulationsende muss nach dem Start liegen.",
    period_too_long => "Der Simulationszeitraum ist länger als 100 Jahre.",
    paths_memory => "Zu viele Pfade für den Speicher: {paths:int} Pfade × {months} Monate. Bei dieser Planungsdauer sind höchstens {max:int} Pfade möglich.",
    birth_after_start => "{who}: Das Geburtsdatum ({birth:ym}) liegt nach dem Simulationsstart ({start:ym}).",
    person_too_old => "{who}: Am Simulationsende wäre die Person über {max} Jahre alt.",
    milestone_age_invalid => "«{name}»: Alter {years} ist nicht möglich.",
    milestone_before_start => "«{name}» ({month:ym}) liegt vor dem Simulationsstart.",
    milestone_after_end => "«{name}» ({month:ym}) liegt nach dem Simulationsende.",
    series_empty => "{what}: keine Einträge.",
    series_late_start => "{what}: kein Wert ab Simulationsstart – bis zum ersten Eintrag gilt 0.",
    series_out_of_range => "{what}: Werte müssen zwischen {lo} und {hi} liegen.",
    pk_lumpsum_range => "Kapitalbezug muss zwischen 0 % und 100 % liegen.",
    pk_conversion_high => "Umwandlungssatz {rate:pct2} % ist ungewöhnlich hoch.",
    pk_lumpsum_range_2 => "Person 2: Kapitalbezug muss zwischen 0 % und 100 % liegen.",
    pk_conversion_high_2 => "Person 2: Umwandlungssatz {rate:pct2} % ist ungewöhnlich hoch.",
    settings_problem => "Einstellungen: {problem}",
    limits_not_whole_years => "{what}: Die Grenzen müssen ganze Jahre zwischen 0 und 130 sein.",
    limits_reversed => "{what}: Die untere Grenze ({lo}) liegt über der oberen Grenze ({hi}).",
    limits_pillar3a_max => "Säule 3a: Der Maximalbetrag muss 0 oder mehr sein.",
    milestone_age_limits => "«{name}»: Alter {age} liegt ausserhalb der Grenzen {lo} bis {hi} Jahre (Einstellungen).",
    income_after_pk => "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse ({at:ym}) noch {amount:int} pro Monat.",
    income_after_pk_indexed => "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse ({at:ym}) noch {amount:int} pro Monat in Preisen des Simulationsstarts.",
    spending_below_housing => "Die Lebenshaltungskosten ({spending:int}) sind kleiner als die Wohnkosten zum Simulationsstart ({housing:int}), die sie enthalten.",
    extra_after_end => "Einmaliger Betrag «{label}» liegt nach dem Simulationsende.",
    weights_range => "Jedes Gewicht der Regionen muss zwischen 0 % und 100 % liegen.",
    weights_sum => "Die Gewichte der Regionen ergeben {sum} % statt 100 %.",
    historical_series_missing => "Für historische Renditen muss eine Zeitreihe gewählt sein.",
    historical_start_missing => "Der historische Startmonat fehlt.",
    bootstrap_series_missing => "Für Monte Carlo aus der Historie muss eine Zeitreihe gewählt sein.",
    paths_range => "Die Anzahl Pfade muss zwischen 1 und 1'000'000 liegen.",
    block_length_min => "Die mittlere Blocklänge muss mindestens 1 Monat betragen.",
    volatility_negative => "Volatilität und Sprungrate dürfen nicht negativ sein.",
    degrees_of_freedom => "Die Freiheitsgrade müssen grösser als 2 sein (0 = Normalverteilung).",
    buckets_negative => "Die Töpfe dürfen nicht negativ sein.",
    buckets_too_large => "Topf 1 + Topf 2 decken mehr als 15 Jahre – wenig Aktienanteil übrig.",
    buckets_without_retirement => "Die 3-Topf-Strategie beginnt mit der Pensionierung; ohne Bezugsdatum gilt durchgehend «Standard».",
    withdrawal_rate_range => "Die Entnahmerate muss zwischen 0 % und 100 % liegen.",
    withdrawal_rate_high => "Eine Entnahmerate von {rate:pct1} % ist hoch; das Vermögen schrumpft bei längerer Dauer rasch.",
    vpw_invalid => "VPW: Planungsalter und Realrendite sind ungültig.",
    endowment_invalid => "Endowment: Rate und Glättung müssen zwischen 0 % und 100 % liegen.",
    withdrawal_without_retirement => "Die Entnahmestrategie beginnt mit der Pensionierung; ohne Bezugsdatum gelten die geplanten Ausgaben durchgehend.",
    scenario_stock_share_buckets => "Mit der 3-Topf-Strategie wirkt der Aktienanteil nur bis zur Pensionierung; danach bestimmen die Töpfe die Aufteilung.",
    scenario_start_date_source => "Der historische Startmonat wirkt nur mit der Quelle «Historisch ab Startzeitpunkt».",
    optimize_max_ruin => "Optimierung: Die höchste Ruinwahrscheinlichkeit muss zwischen 0 % und 100 % liegen.",
    optimize_paths => "Optimierung: Die Anzahl Pfade muss grösser als 0 sein.",
    optimize_problem => "Optimierung: {problem}",
    tax_multiplier_range => "Steuerfuss: Der Faktor muss zwischen 0 und 10 liegen.",
    tax_rate_falling => "{table}, Stufe {row}: Mit diesem Satz sinkt die Steuer, obwohl der Betrag steigt. Bitte prüfen Sie die Tabelle (Satz = Steuer geteilt durch Betrag, nicht Grenzsteuersatz).",
    tax_amount_negative => "{table}, Stufe {row}: Der Betrag muss 0 oder grösser sein.",
    tax_rate_range => "{table}, Stufe {row}: Der Steuersatz muss zwischen 0 % und 100 % liegen.",
    gain_tax_missing => "Grundstückgewinnsteuer fehlt: Die Planung enthält eine Immobilie im Besitz oder mit geplantem Kauf. Bitte unter «Steuern» die Vorlage Zürich, die Vorlage Aargau oder eigene Regeln (auch mit Satz 0 %) erfassen.",
    gain_tax_duration_invalid => "Grundstückgewinnsteuer: Besitzdauer und Zu- oder Abschlag sind ungültig (Jahre ab 0, Abschlag kleiner als 100 %).",
    gain_tax_aargau_empty => "Grundstückgewinnsteuer (Modell Aargau): Es ist kein Steuersatz nach Besitzdauer erfasst.",
    gain_tax_aargau_invalid => "Grundstückgewinnsteuer (Modell Aargau): Besitzdauer und Steuersatz sind ungültig (Jahre ab 0, Satz zwischen 0 und 100 %).",
    property_bought_missing => "{name}: Das Kaufdatum fehlt. Ohne Kaufdatum gilt kein Zu- oder Abschlag (Modell Zürich) bzw. der höchste Satz (Modell Aargau) bei der Grundstückgewinnsteuer.",
    property_id_duplicate => "{name}: Kennung «{id}» ist doppelt vergeben.",
    property_after_sale_missing => "{name}: Die Immobilie, nach deren Verkauf gekauft bzw. gemietet wird, gibt es nicht mehr.",
    property_after_own_sale => "{name}: Kauf nach dem eigenen Verkauf ist nicht möglich.",
    property_after_rent_sale => "{name}: «{other}» ist eine Miete und wird nie verkauft.",
    property_sale_before_start => "{name}: Verkaufszeitpunkt liegt vor dem Simulationsstart – das Objekt wird nicht zu diesem Zeitpunkt verkauft. Der Verkauf müsste im Simulationszeitraum liegen.",
    property_sale_before_bought => "{name}: Der Verkauf muss nach dem Kaufdatum liegen.",
    property_sale_before_buy => "{name}: Der Verkauf muss nach «Kauf ab» liegen.",
    property_rent_from_missing => "{name}: «Miete ab» fehlt.",
    property_rent_missing => "{name}: Miete fehlt.",
    property_buy_before_start => "{name}: Kaufzeitpunkt liegt vor dem Simulationsstart – das Objekt wird nie gekauft. Bereits gekaufte Objekte als «Besitz» erfassen.",
    property_buy_missing => "{name}: kein Kaufzeitpunkt – das Objekt wird nie gekauft.",
    mortgage_max_loan_not_planned => "{name}: «80 % des Kaufpreises» gilt nur für geplante Käufe; es zählen die erfassten Beträge.",
    mortgage_above_price => "{name}: Hypothek ist höher als der Kaufpreis.",
    mortgage_above_80 => "{name}: Belehnung über 80 % – Banken verlangen meist 20 % Eigenmittel.",
    tranche_no_period => "{name}: keine Hypothekarperiode erfasst.",
    tranche_start_missing => "{name}: Beginn der laufenden Periode fehlt.",
    tranche_start_after_start => "{name}: Beginn liegt nach dem Simulationsstart.",
    tranche_pillar3a_person_missing => "{name}: Die Person «{id}» für die Säule 3a gibt es nicht.",
    period_term_too_long => "{name}, Periode {period}: Laufzeit über 100 Jahre.",
    period_term_not_positive => "{name}, Periode {period}: Laufzeit muss grösser als 0 sein.",
    period_rate_high => "{name}, Periode {period}: Zins {rate:pct2} % ist ungewöhnlich hoch.",
    period_margin_high => "{name}, Periode {period}: SARON-Marge {margin:pct2} % ist ungewöhnlich hoch.",
    pillar3a_person_missing => "Säule 3a: Die Person «{person}» gibt es nicht.",
    pillar3a_duplicate => "{who}: Pro Person ist nur ein 3a-Konto möglich.",
    pillar3a_balance_negative => "{who}: Das Guthaben darf nicht negativ sein.",
    pillar3a_rate_range => "{who}: Der Zins muss zwischen −50 % und 100 % liegen.",
    pillar3a_contribution_range => "{who}: Die Einzahlung muss zwischen 0 und dem Maximalbetrag von {max:int} pro Jahr liegen (Einstellungen).",
    pillar3a_over_max => "{who}: Einzahlung ({contribution:int}) und indirekte Amortisation ({indirect:int}) übersteigen zusammen den Maximalbetrag von {max:int} pro Jahr.",
    pillar3a_repay_without_indirect => "{who}: «Hypothek zurückzahlen» setzt eine Tranche mit indirekter Amortisation voraus; der Rest geht ins freie Vermögen.",
    pillar3a_payout_date_invalid => "{who}: Das Datum der {n}. Auszahlung fehlt oder ist ungültig.",
    pillar3a_payout_age_invalid => "{who}: Das Alter (0 bis 150 Jahre) der {n}. Auszahlung fehlt oder ist ungültig.",
    pillar3a_payout_too_early => "{who}: Die Auszahlung im {date:ym} liegt mehr als 5 Jahre vor dem AHV-Bezug ({ahv:ym}); frühestens im {earliest:ym}.",
    pillar3a_payout_after_ahv => "{who}: Die Auszahlung im {date:ym} liegt nach dem AHV-Bezug ({ahv:ym}); das Guthaben ist dann schon ausbezahlt.",
    pillar3a_payout_amount => "{who}: Der Betrag der Auszahlung im {date:ym} muss grösser als 0 sein.",
    pillar3a_payout_repay_without_indirect => "{who}: «Hypothek zurückzahlen» (Auszahlung im {date:ym}) setzt eine Tranche mit indirekter Amortisation voraus; sie geht ins freie Vermögen.",
    pillar3a_payout_same_year_as_pk => "{who}: Auszahlung im {date:ym} und PK-Kapitalbezug im selben Jahr – sie werden für die Steuerprogression zusammengerechnet.",
    // Milestones and points in time (resolve.rs)
    milestone_person_unknown => "Meilenstein «{name}»: unbekannte Person «{person}»",
    milestone_id_duplicate => "Meilenstein-ID «{id}» ist doppelt vergeben",
    milestone_missing => "Meilenstein «{id}» fehlt",
    time_ref_unknown => "Unbekannter Zeitbezug «{reference}»",
    // Files (model.rs, api.rs, library.rs)
    format_unknown => "Die Datei hat das Format «{format}». Sie braucht eine neuere Version von VivaPlan und wird nicht geöffnet.",
    plan_file_invalid => "Projektdatei: {detail}",
    // Variations (scenarios.rs)
    scenario_problem => "Variation: {problem}",
    scenario_param_missing => "Parameter fehlt",
    scenario_param_unknown => "unbekannter Parameter «{param}»",
    scenario_step => "{param}: Schritt muss grösser als 0 sein",
    scenario_value_missing => "{param}: «{key}» fehlt",
    scenario_month_invalid => "{param}: «{value}» ist kein gültiger Monat (erwartet JJJJ-MM)",
    scenario_to_before_from => "{param}: «Bis» liegt vor «Von»",
    scenario_too_many => "Die Variationen ergeben {count:int} Kombinationen; höchstens {max:int} sind technisch möglich.",
    scenario_too_many_overflow => "Die Variationen ergeben mehr als 1'000'000 Kombinationen; höchstens {max:int} sind technisch möglich.",
    scenario_retire_missing => "Variation {param}: Meilenstein «retire» fehlt in der Planung",
    scenario_factor_missing => "Variation {param}: Faktor fehlt in der Planung",
    file_error => "{path}: {detail}",
    series_too_short => "Die Zeitreihe {name} ist kürzer als der Simulationszeitraum.",
    series_not_shipped => "Reihe «{file}» ist nicht ausgeliefert",
    strategy_no_common_years => "Die Indizes der Strategie «{strategy}» haben keine gemeinsamen Jahre.",
    strategy_unreadable => "Strategie nicht lesbar: {detail}",
    // Import of data series (import.rs, jst.rs)
    shiller_sheet_missing => "{path}: Blatt «Data» fehlt ({detail})",
    shiller_header_missing => "Shiller-Datei: Kopfzeile «Date» nicht gefunden",
    shiller_column_missing => "Shiller-Datei: Spalte «{column}» nicht gefunden",
    shiller_too_few => "Shiller-Datei: zu wenige Monatswerte",
    shiller_gaps => "Shiller-Datei: keine durchgehenden Monatswerte",
    snb_line_mismatch => "SNB-Daten: Zeile «{line}» passt nicht zur Kopfzeile",
    snb_month_invalid => "SNB-Daten: «{value}» ist kein gültiger Monat (erwartet JJJJ-MM)",
    snb_not_a_number => "SNB-Daten: «{value}» ist keine Zahl",
    snb_not_snb => "keine SNB-Datei (CubeId bzw. Kopfzeile fehlt)",
    snb_cube_unused_expected => "SNB-Würfel «{id}» wird nicht verwendet (erwartet: devkum, plkopr, zimoma, rendoblim)",
    snb_cube_unused => "SNB-Würfel «{id}» wird nicht verwendet",
    snb_series_missing => "SNB-Würfel {cube}: Reihe {key} fehlt",
    chf_monthly_only => "Die Umrechnung in CHF ist nur für Monatsreihen möglich.",
    series_not_usd => "Die Reihe {name} ist nicht in USD.",
    fx_cpi_not_covering => "Kurs USD/CHF bzw. Landesindex decken die Reihe nicht ab.",
    fx_cpi_gaps => "Kurs USD/CHF bzw. Landesindex haben Lücken.",
    ch_bonds_monthly_only => "Schweizer Obligationen lassen sich nur mit einer CHF-Monatsreihe kombinieren.",
    ch_bonds_not_covering => "Die Renditen der Eidgenossenschaft decken die Reihe nicht ab.",
    ch_bonds_gaps => "Die Renditen der Eidgenossenschaft haben Lücken.",
    snb_bond_yields_missing => "SNB-Renditen der Eidgenossenschaft fehlen",
    jst_no_sheet => "{path}: kein Tabellenblatt",
    jst_table_empty => "JST-Daten: Tabelle ist leer",
    jst_column_missing => "JST-Daten: Spalte «{column}» fehlt",
    jst_no_region_values => "JST-Daten: keine Werte für die Länder der Regionen",
    jst_extract_empty => "JST-Auszug ist leer",
    jst_extract_header => "JST-Auszug: Kopfzeile «{header}» statt «{expected}»",
    jst_extract_line => "JST-Auszug, Zeile {row}: «{line}»",
    jst_no_stock_return => "JST-Daten: keine Aktienrendite für {countries} im Jahr {year}",
    jst_ch_year_missing => "JST-Daten: Schweiz {year} fehlt",
    jst_ch_bonds_missing => "JST-Daten: Obligationen Schweiz {year} fehlen",
    jst_ch_short_rates_missing => "JST-Daten: kurzfristige Zinsen Schweiz {year} fehlen",
    jst_ch_inflation_missing => "JST-Daten: Teuerung Schweiz {year} fehlt",
    ch_inflation_missing => "Teuerung Schweiz {year} fehlt",
    library_unknown => "Die Zeitreihe «{file}» liegt in der Datenbibliothek, deren Ordner hier nicht bekannt ist.",
    library_name_invalid => "Ungültiger Name in der Datenbibliothek: «{file}»",
    import_chf_ends => "Die CHF-Reihe endet {month:ym}, weil die SNB-Daten (Kurs bzw. Landesindex) nicht weiter reichen. Neuere SNB-Daten lassen sich ebenfalls importieren.",
    import_snb_cube => "SNB-Würfel {id} übernommen.",
    import_snb_cube_dated => "SNB-Würfel {id} übernommen (Stand {published}).",
    import_shiller_usd_missing => "«Shiller USD.csv» fehlt in der Bibliothek; die CHF-Reihe wurde nicht neu berechnet.",
    // Import of the old configuration (legacy.rs)
    legacy_variable_undefined => "Variable {name} ist nicht definiert",
    legacy_not_an_age => "«{value}» ist kein Alter",
    legacy_not_a_number => "{path}: «{value}» ist keine Zahl",
    legacy_unreadable => "config.json: {detail}",
    legacy_general_missing => "config.json: Abschnitt «General» fehlt – keine Konfiguration des früheren Programms.",
    // Data series (market.rs)
    series_unreadable => "Zeitreihe {path} nicht lesbar: {detail}",
    series_file_empty => "Zeitreihe {name}: Datei ist leer",
    series_column_missing => "Zeitreihe {name}: Spalte «{column}» fehlt",
    series_line_problem => "Zeitreihe {name}, Zeile {row}: {problem}",
    month_invalid => "«{value}» ist kein gültiger Monat (erwartet JJJJ-MM)",
    year_invalid => "«{value}» ist kein Jahr (1000 bis 9999)",
    series_dates_not_continuous => "Datum fehlt oder ist nicht fortlaufend",
    not_a_number => "«{value}» ist keine Zahl",
    series_no_data => "Zeitreihe {name}: keine Daten",
    historical_start_outside => "Historischer Start {start:ym} liegt ausserhalb der Zeitreihe {name} ({first:ym} – {last:ym})",
    raw_error => "{detail}",
    // Optimization (optimize.rs)
    percent_value => "{value} %",
    optimize_range_invalid => "{strategy}, {param}: Die Bandbreite von {lo} bis {hi} ist ungültig («von» darf nicht grösser als «bis» sein).",
    optimize_range_limits => "{strategy}, {param}: Die Bandbreite muss zwischen {min} und {max} liegen.",
    optimize_no_retirement => "Die Planung hat keine Pensionierung.",
    optimize_strategy_unknown => "Unbekannte Strategie «{strategy}» (standard, three_buckets, three_buckets_guardrails, guardrails, fixed_rate, vpw, endowment, all)",
    optimize_ruin_range => "Die Ruingrenze muss zwischen 0 % und 100 % liegen.",
    optimize_paths_positive => "Die Anzahl Pfade muss grösser als 0 sein.",
    optimize_keep_property_without_property => "Das Ziel «Kein Notverkauf von Immobilien» braucht eine Planung mit mindestens einer Immobilie.",
    optimize_min_consumption_range => "Der Mindestkonsum muss zwischen 0 % und 200 % der geplanten Ausgaben liegen.",
    optimize_no_pk_payout => "Die Planung hat keine Pensionierung (Pensionskasse «Bezug ab»).",
    // Interface of the engine (api.rs)
    plan_incomplete => "Die Projektdatei ist unvollständig.",
    plan_incomplete_detail => "Die Projektdatei ist unvollständig: {detail}",
    legacy_config_detected => "Das ist eine Konfiguration des bisherigen Programms – bitte «Alte config.json importieren» verwenden.",
    variation_selection_without_param => "Auswahl der Variation ohne Parameter",
    path_index_outside => "Pfadnummer {index} ausserhalb von 0 bis {max}",
    currency_mismatch => "Die Zeitreihe «{series}» ist in {currency}, die Planung in {plan_currency}. Renditen und Inflation gelten dann für {currency}.",
    preview_model_and_series => "Vorschau mit der erwarteten Aktienrendite des Modells ({stocks:pct1} % pro Jahr) und den Durchschnitten der Zeitreihe: Obligationen {bonds:pct1} %, Inflation {inflation:pct1} % pro Jahr.",
    preview_model => "Vorschau mit der erwarteten Aktienrendite des Modells ({stocks:pct1} % pro Jahr).",
    preview_series => "Vorschau mit den Durchschnitten der Zeitreihe: Aktien {stocks:pct1} %, Obligationen {bonds:pct1} %, Inflation {inflation:pct1} % pro Jahr.",
    // Simulation (sim.rs)
    plan_errors => "{errors:lines}",
    tranche_period_start_missing => "{name}: Beginn der laufenden Hypothekarperiode fehlt",
    property_reference_missing => "{name}: Immobilie «{id}» gibt es nicht",
    empty => "",
    reasons => "{items}",
    suffix_name => " ({name})",
    suffix_person => " (Person {n})",
    event_sale => "Verkauf {name}: Wert {value:fix0} {currency}, Hypothek {debt:fix0} {currency}, Grundstückgewinnsteuer {tax:fix0} {currency}",
    event_emergency_sale => "Notverkauf {name}: Wert {value:fix0} {currency}, Hypothek {debt:fix0} {currency}, Grundstückgewinnsteuer {tax:fix0} {currency}",
    event_sale_reserved => "Verkaufserlös {name} ({amount:fix0} {currency}) reserviert für Kauf im Folgemonat",
    note_sale_reserved => "Verkaufserlös reserviert für Kauf im Folgemonat",
    note_withdrawal_applied => "Entnahmestrategie angewendet (kein Folgekauf)",
    event_purchase_dropped => "Kauf {name} entfällt: Folgeregeln (Miete, weitere Käufe «nach Verkauf») gelten ab dem Folgemonat",
    event_rent_start => "Miete ab jetzt: {names}",
    event_mortgage_unaffordable_wealth => "Hypothek {name} nicht mehr tragbar: Einmalzahlung {lump:fix0} {currency} nötig, Vermögen {wealth:fix0} {currency} reicht nicht; Hypothek nicht erneuert, Notverkauf eingeleitet",
    event_mortgage_unaffordable_criterion => "Hypothek {name} nicht mehr tragbar: Einmalzahlung {lump:fix0} {currency} nötig, danach würde das Notverkaufskriterium auslösen; Hypothek nicht erneuert, Notverkauf eingeleitet",
    event_mortgage_lump_sum => "Einmalrückzahlung Hypothek {name} für die Tragbarkeit: {lump:fix0} {currency}, Hypothek neu {debt:fix0} {currency}",
    event_purchase_impossible => "Kauf {name} nicht möglich: Eigenmittel {needed:fix0} {currency} nötig, vorhanden {available:fix0} {currency}",
    event_purchase => "Kauf {name}: Preis {price:fix0} {currency}, Hypothek {mortgage:fix0} {currency}",
    event_purchase_lump_sum => "Kauf {name}: Preis {price:fix0} {currency}, Hypothek {mortgage:fix0} {currency}, Einmalrückzahlung {lump:fix0} {currency} für die Tragbarkeit",
    event_mortgage_period => "Neue Hypothekarperiode {name}{tranche}: {rate:pct2} %, {amount:fix0} {currency}",
    event_mortgage_repaid => "Rückzahlung Hypothek {name}: {amount:fix0} {currency}",
    event_amortization_direct => "Amortisation Hypothek {name}{tranche} ab AHV-Bezug{who} direkt statt über die Säule 3a: {amount:fix0} {currency} pro Jahr",
    event_pillar3a_short => "Säule 3a{who}: Geplant {planned:fix0} {currency}, das Guthaben reicht nur für {available:fix0} {currency}",
    event_pillar3a_payout => "Säule 3a{who}: {what} {amount:fix0} {currency}, Steuer {tax:fix0} {currency}, an Hypothek {repaid:fix0} {currency}",
    payout => "Auszahlung",
    payout_rest => "Auszahlung des Rests beim AHV-Bezug",
    event_extra => "{label}: {amount:fix0} {currency}",
    event_pk_payout => "Bezug Pensionskasse{who}: Kapitalbezug {lump:fix0} {currency} (Steuer {tax:fix0} {currency}), Rente {pension:fix0} {currency} / Monat",
    event_ahv_start => "AHV-Rente{who} {pension:fix0} {currency} / Monat",
    event_withdrawal_start => "Entnahme {amount:fix0} {currency} pro Jahr ({rate:pct1} % von {wealth:fix0} {currency})",
    event_guardrail_cut => "Leitplanke: Entnahme um {adjust:pct0} % gekürzt auf {amount:fix0} {currency} pro Jahr (Rate {rate:pct1} %)",
    event_guardrail_raise => "Leitplanke: Entnahme um {adjust:pct0} % erhöht auf {amount:fix0} {currency} pro Jahr (Rate {rate:pct1} %)",
    event_strategy_applied => "Entnahmestrategie angewendet nach {reasons}",
    event_depleted => "Vermögen aufgebraucht",
    // Reasons (emergency sale, withdrawal strategy, inflows into the buckets)
    reason_emergency_criterion => "Notverkaufskriterium",
    reason_wealth_negative => "Vermögen < 0",
    reason_pots_insufficient => "Topf 1, 2 und 3 reichen für die Ausgaben nicht",
    reason_sale_no_follow_up => "Verkauf {name}, kein Folgekauf",
    reason_sale_surplus => "Überschuss aus dem Verkaufserlös nach dem Kauf",
    reason_extra_inflow => "Mittelzufluss «{label}»",
    reason_pk_payout => "Pensionskassenauszahlung",
    reason_pk_lump_sum => "PK-Kapitalbezug",
    reason_wealth_positive_again => "Vermögen wieder positiv (Neustart nach Phase 8)",
    reason_withdrawal_start => "Start der Entnahmephase",
    reason_january => "Januar",
    why_sale => "Verkauf {name}",
    why_extra => "«{label}»",
    why_pillar3a => "Auszahlung Säule 3a",
    why_reservation_surplus => "Überschuss der Reservierung nach dem Kauf",
    // 3-bucket strategy
    event_buckets_reservation_inflow => "3-Topf Zufluss in die Reservierung ({why}): {amount:fix0} {currency}",
    reason_reservation_inflow => "Zufluss in die Reservierung ({why})",
    event_buckets_inflow_external => "3-Topf extern nach Topf 3 ({why}): {amount:fix0} {currency}; Topf 1/2/3 nachher {pot1:fix0} {currency} / {pot2:fix0} {currency} / {pot3:fix0} {currency}",
    event_buckets_inflow_reservation => "3-Topf Reservierung nach Topf 3 ({why}): {amount:fix0} {currency}; Topf 1/2/3 nachher {pot1:fix0} {currency} / {pot2:fix0} {currency} / {pot3:fix0} {currency}",
    reason_inflow_pot3 => "Zufluss nach Topf 3 ({why})",
    criterion_ignored => "Kriterium ignoriert",
    criterion_met => "Aktienrendite {stock_return:pct1} % ({basis}) ≥ Schwelle {threshold:pct1} %",
    criterion_not_met => "Aktienrendite {stock_return:pct1} % ({basis}) < Schwelle {threshold:pct1} %",
    return_last_12_months => "letzte 12 Monate",
    return_since_start => "seit Start",
    event_buckets_refill => "3-Topf aufgefüllt ({reason}): Netto-Bedarf {need:fix0} {currency} (Ausgaben {expenses:fix0} {currency} − Einnahmen {receipts:fix0} {currency}, {source}); Ziel Topf 1 {target1:fix0} {currency}, Topf 2 {target2:fix0} {currency}; {criterion}; Topf 1/2/3 vorher {before1:fix0} {currency} / {before2:fix0} {currency} / {before3:fix0} {currency}, nachher {after1:fix0} {currency} / {after2:fix0} {currency} / {after3:fix0} {currency}",
    reason_lookahead_met => "Vorausschau: Topf 1 deckt die Ausgaben des Monats nicht, Renditekriterium erfüllt ({passed} Monate seit Januar)",
    reason_lookahead_not_met => "Vorausschau: Topf 1 deckt die Ausgaben des Monats nicht, Renditekriterium nicht erfüllt ({months} Monate bis Januar)",
    event_buckets_c2 => "3-Topf C2 Auffüllung Topf 1 vor der Zahlung (Topf 1 deckt die Ausgaben {payment:fix0} {currency} nicht): Topf 1/2/3 vorher {before1:fix0} {currency} / {before2:fix0} {currency} / {before3:fix0} {currency}, nachher {after1:fix0} {currency} / {after2:fix0} {currency} / {after3:fix0} {currency}",
    event_buckets_empty => "3-Topf: alle Töpfe leer, keine Immobilie mehr – Topf 1 ist im Minus ({cash:fix0} {currency}), Topf 2 und 3 sind stillgelegt",
    basis_january => "Januar, geplant",
    basis_from => "ab {month:ym}",
    basis_from_extrapolated => "ab {month:ym}, auf 12 Monate hochgerechnet",
    // Labels of the runs (market.rs, montecarlo.rs)
    path_fixed => "Fixe Renditen",
    path_historical => "Historisch ab {start:ym}",
    path_historical_year => "Historisch ab {year}",
    path_bootstrap => "Bootstrap-Pfad {n}",
    path_random => "Zufallspfad {n}",
    // Notes when updating an older plan (upgrade.rs)
    note_parametric => crate::upgrade::PARAMETRIC_NOTE,
    note_sampler => crate::upgrade::SAMPLER_NOTE,
    note_strategy_usd => crate::upgrade::STRATEGY_USD_NOTE,
    note_cash => crate::upgrade::CASH_NOTE,
    note_pictet_stocks => "für die Aktien «Schweiz (JST)»",
    note_pictet_bonds => "für die Obligationen «Eidgenossenschaft 10 Jahre»",
    note_pictet => "Der Index «Schweiz (Pictet)» wird aus Lizenzgründen nicht mehr mitgeliefert. Die Anlagestrategie verwendet jetzt {parts}; bitte unter «Vermögen & Anlage» prüfen. Das Ergebnis ändert sich.",
    note_pictet_both => "Der Index «Schweiz (Pictet)» wird aus Lizenzgründen nicht mehr mitgeliefert. Die Anlagestrategie verwendet jetzt {first} und {second}; bitte unter «Vermögen & Anlage» prüfen. Das Ergebnis ändert sich.",
    note_own_series => "Die eigene Zeitreihe «{name}» wird nicht mehr unterstützt. Die Anlagestrategie ist jetzt «Welt (JST)» mit Obligationen der Eidgenossenschaft; bitte unter «Vermögen & Anlage» prüfen. Das Ergebnis ändert sich.",
    note_spending_includes_housing => "Die Lebenshaltungskosten enthalten neu die Wohnkosten. Die Wohnkosten zum Simulationsstart ({housing:fix0} pro Monat) sind zu jedem Wert addiert; das Ergebnis bleibt gleich.",
    note_milestone_added => "Der Meilenstein «{name}» fehlte und steht neu auf {years} Jahren.",
    note_milestone_renamed => "Der Meilenstein «{old}» heisst neu «{name}».",
    note_milestone_age_limited => "«{name}» lag bei {age} Jahren, ausserhalb der Grenzen {lo} bis {hi} Jahre; neu {years} Jahre.",
    note_milestone_own_person => "«{name}» bezieht sich neu auf das Alter der eigenen Person ({years} Jahre).",
    note_pk_reference => "Der Bezug der Pensionskasse beginnt neu am Meilenstein «{milestone}» (bisher «{old}»).",
    note_ahv_reference => "Die AHV-Rente beginnt neu am Meilenstein «{milestone}» (bisher «{old}»).",
    note_pillar3a_repay_to_wealth => "Säule 3a von «{name}»: «Hypothek zurückzahlen» setzt neu eine Tranche mit indirekter Amortisation voraus; die Auszahlung geht ins freie Vermögen.",
    note_pillar3a_account_rest_at_ahv => "Die Säule 3a von «{name}» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»); der Rest wird im Monat des AHV-Bezugs ausbezahlt.",
    note_pillar3a_account_moved => "Die Säule 3a von «{name}» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»); die Auszahlung ({old:ym}) lag mehr als 5 Jahre vor dem AHV-Bezug und ist neu im {date:ym}.",
    note_pillar3a_account_payout => "Die Säule 3a von «{name}» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»); Auszahlung des ganzen Guthabens im {date:ym}.",
    note_pillar3a_account_no_date => "Die Säule 3a von «{name}» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»); ohne Auszahlungszeitpunkt wird das Guthaben im Monat des AHV-Bezugs ausbezahlt.",
    note_pillar3a_merged => "Mehrere 3a-Guthaben sind zu einem Konto zusammengefasst (Zins des ersten Kontos: {rate:pct2} %).",
    // Import of the old configuration (legacy.rs)
    legacy_lumpsums_merged => "Mehrere Kapitalbezüge (LumpsumRatio) werden zu einem Bezug bei der Pensionierung zusammengefasst.",
    legacy_monte_carlo => "Monte Carlo übernommen (parametrisches Aktienmodell wie bisher); Obligationen und Inflation folgen den fixen Werten.",
    legacy_iteration => "CalculationMode «Iteration» wird nicht übernommen; Variationen lassen sich als Szenarien erfassen.",
    legacy_property_ignored => "Immobilie «{name}» mit Status «{status}» wird ignoriert.",
    legacy_properties_approximate => "Immobilien werden näherungsweise übernommen: Hypotheken werden zu gleichen Konditionen verlängert (Tragbarkeit nur noch beim Kauf nach Verkauf), Amortisation pro Quartal = 3/4 des alten Werts (gleicher monatlicher Aufwand).",
    // Commands (dispatch.rs)
    path_missing => "Pfad fehlt",
    command_desktop_only => "Befehl «{cmd}» gibt es nur in der Desktop-App",
    // END OF TABLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_171_render_formats() {
        let params: Params = vec![
            ("n", arg(&3)),
            ("x", arg(&1234567.5)),
            ("r", arg(&0.0425)),
            ("m", arg("2031-07")),
            ("s", arg("Anna")),
            ("l", arg(&["A", "B"])),
            ("f", arg(&65.5)),
            ("neg", arg(&-2500.4)),
            ("nested", arg(&msg!(cancelled))),
        ];
        let t = "{n} {x:int} {x:fix0} {r:pct1} {r:pct2} {r:pct0} {m:ym} {s} {l} {f} {neg:int} {nested} {missing_fmt:}";
        assert_eq!(
            render(t, &params),
            "3 1'234'568 1234568 4.2 4.25 4 07.2031 Anna A, B 65.5 -2'500 Abgebrochen {missing_fmt:}"
        );
        // Braces that are no placeholder stay.
        assert_eq!(render("{ a } {", &params), "{ a } {");
    }

    #[test]
    fn issue_171_message_serializes_code_params_text() {
        let m = msg!(cancelled);
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({ "code": "cancelled", "params": {}, "text": "Abgebrochen" })
        );
        assert_eq!(m, "Abgebrochen");
        assert_eq!(m.to_string(), "Abgebrochen");
        assert_eq!(Code::parse("cancelled"), Some(Code::cancelled));
        assert_eq!(Code::parse("nope"), None);
    }

    /// #171/AK3: The examples of all codes for the test of the user interface (`e2e/unit/engine-messages.json`:
    /// code, German template, parameters, text). A new code changes the file: write it again with
    /// `MESSAGES_UPDATE=1 cargo nextest run -p vivaplan-engine -E 'test(issue_171_ak3)'`.
    #[test]
    fn issue_171_ak3_examples_file_is_current() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../e2e/unit/engine-messages.json");
        let examples: Vec<serde_json::Value> = Code::ALL
            .iter()
            .map(|&c| {
                let m = example(c);
                serde_json::json!({ "code": c.as_str(), "template": c.template(), "params": params_json(m.args()), "text": m.text() })
            })
            .collect();
        let now = serde_json::to_string_pretty(&examples).unwrap() + "\n";
        if std::env::var("MESSAGES_UPDATE").is_ok() {
            std::fs::write(&file, &now).unwrap();
        }
        let saved = std::fs::read_to_string(&file).unwrap_or_default();
        assert!(
            saved == now,
            "e2e/unit/engine-messages.json is outdated: write it again with MESSAGES_UPDATE=1"
        );
    }

    /// #171: Codes are unique snake_case words, every template has valid placeholders, and every example renders
    /// without a placeholder left.
    #[test]
    fn issue_171_codes_and_templates_are_valid() {
        let mut seen = std::collections::HashSet::new();
        for &c in Code::ALL {
            let s = c.as_str();
            assert!(seen.insert(s), "{s} twice");
            assert!(
                s.chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
                "{s}"
            );
            for (name, format) in placeholders(c.template()) {
                assert!(
                    matches!(
                        format,
                        "" | "int" | "fix0" | "fix1" | "fix2" | "pct0" | "pct1" | "pct2" | "ym" | "lines"
                    ),
                    "{s}: {name}:{format}"
                );
            }
            let text = example(c).text().to_string();
            assert!(
                placeholders(&text).is_empty() || c.template().contains("{missing"),
                "{s}: {text}"
            );
        }
    }

    #[test]
    fn issue_171_nested_message_and_month() {
        let m = msg!(
            milestone_before_start,
            name = msg!(milestone_retire),
            month = YearMonth::new(2031, 7)
        );
        assert_eq!(
            m.text(),
            "«Bezug Pensionskasse» (07.2031) liegt vor dem Simulationsstart."
        );
        assert_eq!(
            params_json(m.args()),
            serde_json::json!({ "name": { "code": "milestone_retire", "params": {}, "text": "Bezug Pensionskasse" }, "month": "2031-07" })
        );
        assert_eq!(render("{x}|{y}", &[("x", Arg::Null), ("y", Arg::Bool(true))]), "|true");
    }
}
