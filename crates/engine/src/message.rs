//! #171: Messages of the engine for the user interface. Every message has a stable code (English, snake_case) and
//! parameters (numbers, names, months and nested messages as values, never formatted text); the English text is
//! rendered from the template of the code (#176; the same template as `engine.<code>` in the table `en`). The user interface translates by the code (key `engine.<code>` in
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
//! A new message: add the code with its English template to the table at the end of this file, the same template as
//! `engine.<code>` to `crates/app/ui/i18n/en.json` and its translations to the other tables; then `MESSAGES_UPDATE=1 cargo nextest run -p vivaplan-engine -E
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

/// A message: code, parameters and the English text rendered from them. The text is rendered on first use (a run
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
    /// The English text (like `String::as_str`).
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
            /// English template (the same text as `engine.<code>` in the table `en`).
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

// The table: code => English template (= `en.json`). Sorted by area (comments); the user interface has the same keys under
// `engine.` in its text tables.
codes! {
    // General
    cancelled => "Cancelled",
    not_implemented => "Not yet implemented: {what}",
    // Names (parameters of other messages)
    person_n => "Person {n}",
    milestone_start => "Simulation start",
    milestone_end => "Simulation end",
    milestone_retire => "Pension fund payout",
    milestone_ahv => "AHV pension start",
    milestone_retire2 => "Pension fund payout person 2",
    milestone_ahv2 => "AHV pension start person 2",
    pillar3a => "Pillar 3a",
    pillar3a_of => "Pillar 3a {name}",
    term_pk_payout => "Pension fund payout",
    term_ahv_payout => "AHV pension start",
    term_pk_payout_2 => "Pension fund payout (person 2)",
    term_ahv_payout_2 => "AHV pension start (person 2)",
    term_pk_contribution => "Pension fund contributions",
    term_pk_interest => "Pension fund interest",
    term_pk_contribution_2 => "Pension fund contributions (person 2)",
    term_pk_interest_2 => "Pension fund interest (person 2)",
    term_income => "Income",
    term_income_2 => "Income (person 2)",
    term_spending => "Spending",
    term_savings => "Other inflows",
    term_stock_share => "Stock allocation",
    term_reserve => "Cash reserve",
    term_stock_return => "Stock return",
    term_bond_return => "Bond return",
    term_inflation => "Inflation",
    term_saron => "SARON",
    term_emergency_months => "Emergency sale threshold",
    term_income_tax => "Income tax",
    term_wealth_tax => "Wealth tax",
    term_capital_tax => "Lump-sum tax",
    term_income_tax_married => "Income tax (married)",
    term_wealth_tax_married => "Wealth tax (married)",
    term_capital_tax_married => "Lump-sum tax (married)",
    // Validation (validate.rs)
    labeled => "{what}: {problem}",
    guardrails_initial_rate => "The initial rate must be between 0 % and 100 %.",
    guardrails_bands => "The guardrails must be greater than or equal to 0 (lower less than 100 %).",
    guardrails_adjust => "The adjustment must be between 0 % and 100 %.",
    spending_limits_negative => "Spending floor and ceiling must not be negative.",
    spending_floor_above_ceiling => "The spending floor is above the ceiling.",
    persons_none => "At least one person is required.",
    persons_too_many => "At most two persons are possible.",
    partner_without_person => "The details for the second person belong to a second person in the household.",
    partner_missing => "The second person has neither income nor pension entered and is not taken into account.",
    person_name_missing => "Person {n}: the name is missing (required).",
    person_name_too_long => "Person {n}: the name may be at most {max} characters long.",
    ahv_cap_not_positive => "The cap on AHV pensions for couples must be greater than 0.",
    milestones_start_end_missing => "The milestones “start” and “end” must exist.",
    end_before_start => "The simulation end must be after the start.",
    period_too_long => "The simulation period is longer than 100 years.",
    paths_memory => "Too many paths for the available memory: {paths:int} paths × {months} months. With this planning period, at most {max:int} paths are possible.",
    birth_after_start => "{who}: the date of birth ({birth:ym}) is after the simulation start ({start:ym}).",
    person_too_old => "{who}: at the simulation end the person would be over {max} years old.",
    milestone_age_invalid => "“{name}”: age {years} is not possible.",
    milestone_before_start => "“{name}” ({month:ym}) is before the simulation start.",
    milestone_after_end => "“{name}” ({month:ym}) is after the simulation end.",
    series_empty => "{what}: no entries.",
    series_late_start => "{what}: no value from the simulation start – 0 applies until the first entry.",
    series_out_of_range => "{what}: values must be between {lo} and {hi}.",
    pk_lumpsum_range => "The lump sum must be between 0 % and 100 %.",
    pk_conversion_high => "Conversion rate of {rate:pct2} % is unusually high.",
    pk_lumpsum_range_2 => "Person 2: the lump sum must be between 0 % and 100 %.",
    pk_conversion_high_2 => "Person 2: conversion rate of {rate:pct2} % is unusually high.",
    settings_problem => "Settings: {problem}",
    limits_not_whole_years => "{what}: the limits must be whole years between 0 and 130.",
    limits_reversed => "{what}: the lower limit ({lo}) is above the upper limit ({hi}).",
    limits_pillar3a_max => "Pillar 3a: the maximum amount must be 0 or more.",
    milestone_age_limits => "“{name}”: age {age} is outside the limits of {lo} to {hi} years (settings).",
    income_after_pk => "After the pension fund payout ({at:ym}), earned income is still {amount:int} per month.",
    income_after_pk_indexed => "After the pension fund payout ({at:ym}), earned income is still {amount:int} per month in prices at the simulation start.",
    spending_below_housing => "Living expenses ({spending:int}) are lower than the housing costs at the simulation start ({housing:int}) that they include.",
    extra_after_end => "One-off amount “{label}” is after the simulation end.",
    weights_range => "Each regional weight must be between 0 % and 100 %.",
    weights_sum => "The regional weights add up to {sum} % instead of 100 %.",
    historical_series_missing => "Historical returns require a time series.",
    historical_start_missing => "The historical start month is missing.",
    bootstrap_series_missing => "Monte Carlo from history requires a time series.",
    paths_range => "The number of paths must be between 1 and 1'000'000.",
    block_length_min => "The mean block length must be at least 1 month.",
    volatility_negative => "Volatility and jump rate must not be negative.",
    degrees_of_freedom => "The degrees of freedom must be greater than 2 (0 = normal distribution).",
    buckets_negative => "The buckets must not be negative.",
    buckets_too_large => "Bucket 1 + bucket 2 cover more than 15 years – little stock allocation left.",
    buckets_without_retirement => "The 3-bucket strategy starts at retirement; without a payout date, “Standard” applies throughout.",
    withdrawal_rate_range => "The withdrawal rate must be between 0 % and 100 %.",
    withdrawal_rate_high => "A withdrawal rate of {rate:pct1} % is high; wealth shrinks quickly over a longer period.",
    vpw_invalid => "VPW: planning age and real return are invalid.",
    endowment_invalid => "Endowment: rate and smoothing must be between 0 % and 100 %.",
    withdrawal_without_retirement => "The withdrawal strategy starts at retirement; without a payout date, the planned spending applies throughout.",
    scenario_stock_share_buckets => "With the 3-bucket strategy, the stock allocation only applies until retirement; after that, the buckets determine the allocation.",
    scenario_start_date_source => "The historical start month only has an effect with the source “Historical from start date”.",
    optimize_max_ruin => "Optimization: the maximum probability of ruin must be between 0 % and 100 %.",
    optimize_paths => "Optimization: the number of paths must be greater than 0.",
    optimize_problem => "Optimization: {problem}",
    tax_multiplier_range => "Tax multiplier: the factor must be between 0 and 10.",
    tax_rate_falling => "{table}, bracket {row}: with this rate the tax falls although the amount rises. Please check the table (rate = tax divided by amount, not marginal tax rate).",
    tax_amount_negative => "{table}, bracket {row}: the amount must be 0 or greater.",
    tax_rate_range => "{table}, bracket {row}: the tax rate must be between 0 % and 100 %.",
    gain_tax_missing => "Property gains tax missing: the plan contains a property that is owned or planned for purchase. Please enter the Zurich template, the Aargau template or your own rules (even with a rate of 0 %) under “Taxes”.",
    gain_tax_duration_invalid => "Property gains tax: holding period and surcharge or reduction are invalid (years from 0, reduction less than 100 %).",
    gain_tax_aargau_empty => "Property gains tax (Aargau model): no tax rate by holding period has been entered.",
    gain_tax_aargau_invalid => "Property gains tax (Aargau model): holding period and tax rate are invalid (years from 0, rate between 0 and 100 %).",
    property_bought_missing => "{name}: the purchase date is missing. Without a purchase date, no surcharge or reduction (Zurich model) or the highest rate (Aargau model) applies to the property gains tax.",
    property_id_duplicate => "{name}: identifier “{id}” is used more than once.",
    property_after_sale_missing => "{name}: the property after whose sale this one is bought or rented no longer exists.",
    property_after_own_sale => "{name}: a purchase after its own sale is not possible.",
    property_after_rent_sale => "{name}: “{other}” is rented and is never sold.",
    property_sale_before_start => "{name}: the sale date is before the simulation start – the property is not sold at that date. The sale would have to be within the simulation period.",
    property_sale_before_bought => "{name}: the sale must be after the purchase date.",
    property_sale_before_buy => "{name}: the sale must be after “Purchase from”.",
    property_rent_from_missing => "{name}: “Rent from” is missing.",
    property_rent_missing => "{name}: rent is missing.",
    property_buy_before_start => "{name}: the purchase date is before the simulation start – the property is never bought. Enter properties already bought as “Owned”.",
    property_buy_missing => "{name}: no purchase date – the property is never bought.",
    mortgage_max_loan_not_planned => "{name}: “80 % of the purchase price” only applies to planned purchases; the amounts entered are used.",
    mortgage_above_price => "{name}: the mortgage is higher than the purchase price.",
    mortgage_above_80 => "{name}: loan-to-value above 80 % – banks usually require 20 % equity.",
    tranche_no_period => "{name}: no mortgage period entered.",
    tranche_start_missing => "{name}: start of the current period missing.",
    tranche_start_after_start => "{name}: the start is after the simulation start.",
    tranche_pillar3a_person_missing => "{name}: the person “{id}” for pillar 3a does not exist.",
    period_term_too_long => "{name}, period {period}: term over 100 years.",
    period_term_not_positive => "{name}, period {period}: the term must be greater than 0.",
    period_rate_high => "{name}, period {period}: interest rate of {rate:pct2} % is unusually high.",
    period_margin_high => "{name}, period {period}: SARON margin of {margin:pct2} % is unusually high.",
    pillar3a_person_missing => "Pillar 3a: the person “{person}” does not exist.",
    pillar3a_duplicate => "{who}: only one 3a account per person is possible.",
    pillar3a_balance_negative => "{who}: the balance must not be negative.",
    pillar3a_rate_range => "{who}: the interest rate must be between −50 % and 100 %.",
    pillar3a_contribution_range => "{who}: the contribution must be between 0 and the maximum amount of {max:int} per year (settings).",
    pillar3a_over_max => "{who}: contribution ({contribution:int}) and indirect amortization ({indirect:int}) together exceed the maximum amount of {max:int} per year.",
    pillar3a_repay_without_indirect => "{who}: “Repay mortgage” requires a tranche with indirect amortization; the remainder goes to liquid wealth.",
    pillar3a_payout_date_invalid => "{who}: the date of payout no. {n} is missing or invalid.",
    pillar3a_payout_age_invalid => "{who}: the age (0 to 150 years) of payout no. {n} is missing or invalid.",
    pillar3a_payout_too_early => "{who}: the payout in {date:ym} is more than 5 years before the start of the AHV pension ({ahv:ym}); at the earliest in {earliest:ym}.",
    pillar3a_payout_after_ahv => "{who}: the payout in {date:ym} is after the start of the AHV pension ({ahv:ym}); the balance will already have been paid out by then.",
    pillar3a_payout_amount => "{who}: the amount of the payout in {date:ym} must be greater than 0.",
    pillar3a_payout_repay_without_indirect => "{who}: “Repay mortgage” (payout in {date:ym}) requires a tranche with indirect amortization; it goes to liquid wealth.",
    pillar3a_payout_same_year_as_pk => "{who}: payout in {date:ym} and pension fund lump sum in the same year – they are added together for tax progression.",
    // Milestones and points in time (resolve.rs)
    milestone_person_unknown => "Milestone “{name}”: unknown person “{person}”",
    milestone_id_duplicate => "Milestone ID “{id}” is used more than once",
    milestone_missing => "Milestone “{id}” missing",
    time_ref_unknown => "Unknown time reference “{reference}”",
    // Files (model.rs, api.rs, library.rs)
    format_unknown => "The file has the format “{format}”. It requires a newer version of VivaPlan and will not be opened.",
    plan_file_invalid => "Project file: {detail}",
    // Variations (scenarios.rs)
    scenario_problem => "Variation: {problem}",
    scenario_param_missing => "Parameter missing",
    scenario_param_unknown => "unknown parameter “{param}”",
    scenario_step => "{param}: the step must be greater than 0",
    scenario_value_missing => "{param}: “{key}” missing",
    scenario_month_invalid => "{param}: “{value}” is not a valid month (expected YYYY-MM)",
    scenario_to_before_from => "{param}: “To” is before “From”",
    scenario_too_many => "The variations result in {count:int} combinations; at most {max:int} are technically possible.",
    scenario_too_many_overflow => "The variations result in more than 1'000'000 combinations; at most {max:int} are technically possible.",
    scenario_retire_missing => "Variation {param}: milestone “retire” missing in the plan",
    scenario_factor_missing => "Variation {param}: factor missing in the plan",
    file_error => "{path}: {detail}",
    series_too_short => "The time series {name} is shorter than the simulation period.",
    series_not_shipped => "Series “{file}” is not included",
    strategy_no_common_years => "The indices of the strategy “{strategy}” have no years in common.",
    strategy_unreadable => "Strategy cannot be read: {detail}",
    // Import of data series (import.rs, jst.rs)
    shiller_sheet_missing => "{path}: sheet “Data” missing ({detail})",
    shiller_header_missing => "Shiller file: header “Date” not found",
    shiller_column_missing => "Shiller file: column “{column}” not found",
    shiller_too_few => "Shiller file: too few monthly values",
    shiller_gaps => "Shiller file: no continuous monthly values",
    snb_line_mismatch => "SNB data: line “{line}” does not match the header",
    snb_month_invalid => "SNB data: “{value}” is not a valid month (expected YYYY-MM)",
    snb_not_a_number => "SNB data: “{value}” is not a number",
    snb_not_snb => "not an SNB file (CubeId or header missing)",
    snb_cube_unused_expected => "SNB cube “{id}” is not used (expected: devkum, plkopr, zimoma, rendoblim)",
    snb_cube_unused => "SNB cube “{id}” is not used",
    snb_series_missing => "SNB cube {cube}: series {key} missing",
    chf_monthly_only => "Conversion to CHF is only possible for monthly series.",
    series_not_usd => "The series {name} is not in USD.",
    fx_cpi_not_covering => "The USD/CHF exchange rate or the consumer price index does not cover the series.",
    fx_cpi_gaps => "The USD/CHF exchange rate or the consumer price index has gaps.",
    ch_bonds_monthly_only => "Swiss bonds can only be combined with a monthly CHF series.",
    ch_bonds_not_covering => "The Swiss Confederation bond yields do not cover the series.",
    ch_bonds_gaps => "The Swiss Confederation bond yields have gaps.",
    snb_bond_yields_missing => "SNB Swiss Confederation bond yields missing",
    jst_no_sheet => "{path}: no worksheet",
    jst_table_empty => "JST data: table is empty",
    jst_column_missing => "JST data: column “{column}” missing",
    jst_no_region_values => "JST data: no values for the countries of the regions",
    jst_extract_empty => "JST extract is empty",
    jst_extract_header => "JST extract: header “{header}” instead of “{expected}”",
    jst_extract_line => "JST extract, line {row}: “{line}”",
    jst_no_stock_return => "JST data: no stock return for {countries} in the year {year}",
    jst_ch_year_missing => "JST data: Switzerland {year} missing",
    jst_ch_bonds_missing => "JST data: Swiss bonds {year} missing",
    jst_ch_short_rates_missing => "JST data: Swiss short-term rates {year} missing",
    jst_ch_inflation_missing => "JST data: Swiss inflation {year} missing",
    ch_inflation_missing => "Swiss inflation {year} missing",
    library_unknown => "The time series “{file}” is in the data library, whose folder is not known here.",
    library_name_invalid => "Invalid name in the data library: “{file}”",
    import_chf_ends => "The CHF series ends in {month:ym} because the SNB data (exchange rate or consumer price index) do not extend further. Newer SNB data can also be imported.",
    import_snb_cube => "SNB cube {id} imported.",
    import_snb_cube_dated => "SNB cube {id} imported (as of {published}).",
    import_shiller_usd_missing => "“Shiller USD.csv” is missing in the library; the CHF series was not recalculated.",
    // Import of the old configuration (legacy.rs)
    legacy_variable_undefined => "Variable {name} is not defined",
    legacy_not_an_age => "“{value}” is not an age",
    legacy_not_a_number => "{path}: “{value}” is not a number",
    legacy_unreadable => "config.json: {detail}",
    legacy_general_missing => "config.json: section “General” missing – not a configuration of the previous program.",
    // Data series (market.rs)
    series_unreadable => "Time series {path} cannot be read: {detail}",
    series_file_empty => "Time series {name}: file is empty",
    series_column_missing => "Time series {name}: column “{column}” missing",
    series_line_problem => "Time series {name}, line {row}: {problem}",
    month_invalid => "“{value}” is not a valid month (expected YYYY-MM)",
    year_invalid => "“{value}” is not a year (1000 to 9999)",
    series_dates_not_continuous => "Date missing or not continuous",
    not_a_number => "“{value}” is not a number",
    series_no_data => "Time series {name}: no data",
    historical_start_outside => "Historical start {start:ym} is outside the time series {name} ({first:ym} – {last:ym})",
    raw_error => "{detail}",
    // Optimization (optimize.rs)
    percent_value => "{value} %",
    optimize_range_invalid => "{strategy}, {param}: the range from {lo} to {hi} is invalid (“from” must not be greater than “to”).",
    optimize_range_limits => "{strategy}, {param}: the range must be between {min} and {max}.",
    optimize_no_retirement => "The plan has no retirement.",
    optimize_strategy_unknown => "Unknown strategy “{strategy}” (standard, three_buckets, three_buckets_guardrails, guardrails, fixed_rate, vpw, endowment, all)",
    optimize_ruin_range => "The ruin limit must be between 0 % and 100 %.",
    optimize_paths_positive => "The number of paths must be greater than 0.",
    optimize_keep_property_without_property => "The goal “No emergency sale of real estate” requires a plan with at least one property.",
    optimize_min_consumption_range => "The minimum consumption must be between 0 % and 200 % of the planned spending.",
    optimize_no_pk_payout => "The plan has no retirement (pension fund “payout from”).",
    // Catalog of the optimization (#177; optimize.rs): strategies, parameters, units
    opt_strategy_standard => "Standard",
    opt_strategy_three_buckets => "3 buckets",
    opt_strategy_three_buckets_guardrails => "3 buckets + guardrails",
    opt_strategy_guardrails => "Guardrails",
    opt_strategy_fixed_rate => "Fixed rate",
    opt_strategy_vpw => "VPW",
    opt_strategy_endowment => "Endowment",
    opt_param_stock_share => "Stock allocation",
    opt_param_reserve_months => "Cash reserve",
    opt_param_rate => "Withdrawal rate",
    opt_param_horizon_age => "Planning age",
    opt_param_real_return => "Real return",
    opt_param_e_rate => "Target rate",
    opt_param_smoothing => "Smoothing",
    opt_param_initial_rate => "Initial rate",
    opt_param_upper => "Upper guardrail",
    opt_param_lower => "Lower guardrail",
    opt_param_adjust => "Adjustment",
    opt_param_floor => "Floor of living expenses",
    opt_param_ceiling => "Ceiling of living expenses",
    opt_param_cash_years => "Bucket 1: cash",
    opt_param_bond_years => "Bucket 2: bonds",
    opt_param_refill_threshold => "Refill from stocks at a previous-year return of at least",
    opt_param_with_unit => "{name} ({unit})",
    unit_percent => "%",
    unit_years => "years",
    unit_months => "months",
    unit_age => "age",
    // Investment strategy and indices (#177; strategy.rs)
    index_ch_jst => "Switzerland (JST)",
    index_world_jst => "World (JST)",
    index_world_custom => "World, custom (JST)",
    index_sp500 => "S&P 500 (Shiller)",
    index_confed_10 => "Swiss Confederation 10 years",
    index_us_10 => "USA 10 years (Shiller)",
    index_confed_short => "Swiss Confederation short-term",
    index_world_custom_weights => "World, custom (CH {ch} %, USA {usa} %, EU {eu} %, Asia-Pacific {apac} %)",
    investment_label => "Stocks {stocks}, bonds {bonds}",
    investment_part_stocks => "Stocks {name}",
    investment_part_bonds => "Bonds {name}",
    investment_info => "Stocks {stocks}, bonds {bonds}: annual {first}–{last}, in CHF without currency hedging.",
    investment_note_parts => "Stocks: {stocks}; bonds: {bonds}; annual (monthly values linked to calendar years), in CHF.",
    investment_note_saron => "SARON: annual average of the SNB monthly values, before 2000 the fixed value applies.",
    investment_note_confed_short => "Swiss Confederation short-term: JST (rate for about 3 months) until {year}, then annual average of SARON; the return practically equals the money market rate.",
    investment_note_jst_license => "JST data: license CC BY-NC-SA 4.0 (non-commercial).",
    // Descriptions of the bundled time series (#177; import.rs, jst.rs; German texts of older files: series_text.rs)
    series_title_shiller_usd => "Shiller: S&P Composite and 10-year US Treasury bonds (USD)",
    series_title_shiller_chf => "Shiller: S&P Composite and 10-year US Treasury bonds, in CHF",
    series_title_ch_bonds => "US stocks in CHF and Swiss federal bonds",
    series_title_world => "World (JST): stocks weighted by GDP, Swiss Confederation bonds, in CHF",
    series_info_shiller_usd => "Shiller: S&P Composite and 10-year US Treasury bonds in USD, total return incl. dividends. Inflation: US CPI.",
    series_info_shiller_chf => "Shiller: S&P Composite and 10-year US Treasury bonds, converted to CHF (SNB exchange rate), without currency hedging. Inflation: Swiss CPI, SARON from 06.1999.",
    series_info_ch_bonds => "Shiller: US stocks (S&P Composite) in CHF, without currency hedging. Bonds: 10-year Swiss federal bonds (SNB). Inflation: Swiss CPI, SARON from 06.1999.",
    series_info_world => "World stock index from the JST data (CH, USA, EU, Japan, Australia), weighted by GDP, in CHF without currency hedging. Bonds: Swiss Confederation 10 years. Inflation: CH.",
    series_note_total_return => "Stocks: total return incl. dividends; bonds: total return of 10-year government bonds.",
    series_note_usd_monthly => "Monthly values from monthly averages of prices; inflation: US CPI compared with the same month of the previous year.",
    series_note_chf_conversion => "Returns of the US assets converted to CHF (monthly average USD/CHF, without currency hedging).",
    series_note_chf_inflation => "Inflation: Swiss consumer price index compared with the same month of the previous year; SARON since its introduction (06.1999), before that the fixed value applies.",
    series_note_ch_bonds_stocks => "Stocks: S&P Composite in CHF (without currency hedging); bonds: 10-year Swiss Confederation bonds.",
    series_note_ch_bonds_return => "Bonds: total return of a 10-year bond at par from the monthly SNB yields (coupon plus price change), without costs.",
    series_note_world_stocks => "Stocks: total return of the countries in CHF, weighted by the real GDP of the previous year (Maddison); Germany 1923 excluded (hyperinflation), Germany 1945–1946 and Japan 1946–1947 without values.",
    series_note_world_bonds => "Bonds: JST until {year}, then from the monthly SNB yields (10 years); 1915 coupon instead of yield (stock exchange closed).",
    series_note_world_license => "License: CC BY-NC-SA 4.0 (non-commercial, share alike).",
    series_source_shiller => "Robert J. Shiller, “Irrational Exuberance”, online data (ie_data)",
    series_source_jst => "Jordà-Schularick-Taylor Macrohistory Database, Release 6 (macrohistory.net/database, retrieved 13.11.2026), license CC BY-NC-SA 4.0. Citations: Jordà, Schularick, Taylor (2017), “Macrofinancial History and the New Business Cycle Facts”, NBER Macroeconomics Annual 2016; Jordà, Knoll, Kuvshinov, Schularick, Taylor (2019), “The Rate of Return on Everything, 1870–2015”, Quarterly Journal of Economics",
    series_source_snb => "{source}; SNB (data.snb.ch): {cubes}",
    series_source_as_of => "{source} (as of {dates})",
    series_source_list => "{source}; {other}",
    plain_text => "{text}",
    // Interface of the engine (api.rs)
    plan_incomplete => "The project file is incomplete.",
    plan_incomplete_detail => "The project file is incomplete: {detail}",
    legacy_config_detected => "This is a configuration of the previous program – please use “Import old config.json”.",
    variation_selection_without_param => "Variation selected without a parameter",
    path_index_outside => "Path number {index} outside 0 to {max}",
    currency_mismatch => "The time series “{series}” is in {currency}, the plan in {plan_currency}. Returns and inflation then apply to {currency}.",
    preview_model_and_series => "Preview with the model's expected stock return ({stocks:pct1} % per year) and the averages of the time series: bonds {bonds:pct1} %, inflation {inflation:pct1} % per year.",
    preview_model => "Preview with the model's expected stock return ({stocks:pct1} % per year).",
    preview_series => "Preview with the averages of the time series: stocks {stocks:pct1} %, bonds {bonds:pct1} %, inflation {inflation:pct1} % per year.",
    // Simulation (sim.rs)
    plan_errors => "{errors:lines}",
    tranche_period_start_missing => "{name}: start of the current mortgage period missing",
    property_reference_missing => "{name}: property “{id}” does not exist",
    empty => "",
    reasons => "{items}",
    suffix_name => " ({name})",
    suffix_person => " (person {n})",
    event_sale => "Sale {name}: value {value:fix0} {currency}, mortgage {debt:fix0} {currency}, property gains tax {tax:fix0} {currency}",
    event_emergency_sale => "Emergency sale {name}: value {value:fix0} {currency}, mortgage {debt:fix0} {currency}, property gains tax {tax:fix0} {currency}",
    event_sale_reserved => "Sale proceeds {name} ({amount:fix0} {currency}) reserved for a purchase in the following month",
    note_sale_reserved => "Sale proceeds reserved for a purchase in the following month",
    note_withdrawal_applied => "Withdrawal strategy applied (no subsequent purchase)",
    event_purchase_dropped => "Purchase {name} dropped: follow-up rules (rent, further purchases “after sale”) apply from the following month",
    event_rent_start => "Renting from now on: {names}",
    event_mortgage_unaffordable_wealth => "Mortgage {name} no longer affordable: one-off payment of {lump:fix0} {currency} required, wealth of {wealth:fix0} {currency} is not enough; mortgage not renewed, emergency sale initiated",
    event_mortgage_unaffordable_criterion => "Mortgage {name} no longer affordable: one-off payment of {lump:fix0} {currency} required, after which the emergency sale criterion would trigger; mortgage not renewed, emergency sale initiated",
    event_mortgage_lump_sum => "One-off mortgage repayment {name} for affordability: {lump:fix0} {currency}, new mortgage {debt:fix0} {currency}",
    event_purchase_impossible => "Purchase {name} not possible: equity of {needed:fix0} {currency} required, {available:fix0} {currency} available",
    event_purchase => "Purchase {name}: price {price:fix0} {currency}, mortgage {mortgage:fix0} {currency}",
    event_purchase_lump_sum => "Purchase {name}: price {price:fix0} {currency}, mortgage {mortgage:fix0} {currency}, one-off repayment of {lump:fix0} {currency} for affordability",
    event_mortgage_period => "New mortgage period {name}{tranche}: {rate:pct2} %, {amount:fix0} {currency}",
    event_mortgage_repaid => "Mortgage repayment {name}: {amount:fix0} {currency}",
    event_amortization_direct => "Amortization of mortgage {name}{tranche} from the start of the AHV pension{who} directly instead of via pillar 3a: {amount:fix0} {currency} per year",
    event_pillar3a_short => "Pillar 3a{who}: planned {planned:fix0} {currency}, the balance only covers {available:fix0} {currency}",
    event_pillar3a_payout => "Pillar 3a{who}: {what} {amount:fix0} {currency}, tax {tax:fix0} {currency}, to mortgage {repaid:fix0} {currency}",
    payout => "Payout",
    payout_rest => "Payout of the remainder at the start of the AHV pension",
    event_extra => "{label}: {amount:fix0} {currency}",
    event_pk_payout => "Pension fund payout{who}: lump sum {lump:fix0} {currency} (tax {tax:fix0} {currency}), pension {pension:fix0} {currency} / month",
    event_ahv_start => "AHV pension{who} {pension:fix0} {currency} / month",
    event_withdrawal_start => "Withdrawal {amount:fix0} {currency} per year ({rate:pct1} % of {wealth:fix0} {currency})",
    event_guardrail_cut => "Guardrail: withdrawal cut by {adjust:pct0} % to {amount:fix0} {currency} per year (rate {rate:pct1} %)",
    event_guardrail_raise => "Guardrail: withdrawal raised by {adjust:pct0} % to {amount:fix0} {currency} per year (rate {rate:pct1} %)",
    event_strategy_applied => "Withdrawal strategy applied after {reasons}",
    event_depleted => "Wealth depleted",
    // Reasons (emergency sale, withdrawal strategy, inflows into the buckets)
    reason_emergency_criterion => "Emergency sale criterion",
    reason_wealth_negative => "Wealth < 0",
    reason_pots_insufficient => "Buckets 1, 2 and 3 are insufficient for the spending",
    reason_sale_no_follow_up => "Sale {name}, no subsequent purchase",
    reason_sale_surplus => "Surplus from the sale proceeds after the purchase",
    reason_extra_inflow => "Inflow “{label}”",
    reason_pk_payout => "Pension fund payout",
    reason_pk_lump_sum => "PK lump sum",
    reason_wealth_positive_again => "Wealth positive again (restart after phase 8)",
    reason_withdrawal_start => "Start of the withdrawal phase",
    reason_january => "January",
    why_sale => "Sale {name}",
    why_extra => "“{label}”",
    why_pillar3a => "Pillar 3a payout",
    why_reservation_surplus => "Surplus of the reservation after the purchase",
    // 3-bucket strategy
    event_buckets_reservation_inflow => "3 buckets inflow to the reservation ({why}): {amount:fix0} {currency}",
    reason_reservation_inflow => "Inflow to the reservation ({why})",
    event_buckets_inflow_external => "3 buckets external to bucket 3 ({why}): {amount:fix0} {currency}; buckets 1/2/3 after {pot1:fix0} {currency} / {pot2:fix0} {currency} / {pot3:fix0} {currency}",
    event_buckets_inflow_reservation => "3 buckets reservation to bucket 3 ({why}): {amount:fix0} {currency}; buckets 1/2/3 after {pot1:fix0} {currency} / {pot2:fix0} {currency} / {pot3:fix0} {currency}",
    reason_inflow_pot3 => "Inflow to bucket 3 ({why})",
    criterion_ignored => "Criterion ignored",
    criterion_met => "Stock return {stock_return:pct1} % ({basis}) ≥ threshold {threshold:pct1} %",
    criterion_not_met => "Stock return {stock_return:pct1} % ({basis}) < threshold {threshold:pct1} %",
    return_last_12_months => "last 12 months",
    return_since_start => "since start",
    event_buckets_refill => "3 buckets refilled ({reason}): net need {need:fix0} {currency} (spending {expenses:fix0} {currency} − income {receipts:fix0} {currency}, {source}); target bucket 1 {target1:fix0} {currency}, bucket 2 {target2:fix0} {currency}; {criterion}; buckets 1/2/3 before {before1:fix0} {currency} / {before2:fix0} {currency} / {before3:fix0} {currency}, after {after1:fix0} {currency} / {after2:fix0} {currency} / {after3:fix0} {currency}",
    reason_lookahead_met => "Look-ahead: bucket 1 does not cover the month's spending, return criterion met ({passed} months since January)",
    reason_lookahead_not_met => "Look-ahead: bucket 1 does not cover the month's spending, return criterion not met ({months} months until January)",
    event_buckets_c2 => "3 buckets C2 refill of bucket 1 before the payment (bucket 1 does not cover the spending of {payment:fix0} {currency}): buckets 1/2/3 before {before1:fix0} {currency} / {before2:fix0} {currency} / {before3:fix0} {currency}, after {after1:fix0} {currency} / {after2:fix0} {currency} / {after3:fix0} {currency}",
    event_buckets_empty => "3 buckets: all buckets empty, no property left – bucket 1 is negative ({cash:fix0} {currency}), buckets 2 and 3 are frozen",
    basis_january => "January, planned",
    basis_from => "from {month:ym}",
    basis_from_extrapolated => "from {month:ym}, extrapolated to 12 months",
    // Labels of the runs (market.rs, montecarlo.rs)
    path_fixed => "Fixed returns",
    path_historical => "Historical from {start:ym}",
    path_historical_year => "Historical from {year}",
    path_bootstrap => "Bootstrap path {n}",
    path_random => "Random path {n}",
    // Notes when updating an older plan (upgrade.rs)
    note_parametric => crate::upgrade::PARAMETRIC_NOTE,
    note_sampler => crate::upgrade::SAMPLER_NOTE,
    note_strategy_usd => crate::upgrade::STRATEGY_USD_NOTE,
    note_cash => crate::upgrade::CASH_NOTE,
    note_thirteenth_ahv => crate::upgrade::THIRTEENTH_AHV_NOTE,
    note_pictet_stocks => "“Switzerland (JST)” for the stocks",
    note_pictet_bonds => "“Swiss Confederation 10 years” for the bonds",
    note_pictet => "The index “Switzerland (Pictet)” is no longer included for licensing reasons. The investment strategy now uses {parts}; please check under “Wealth & investment”. The result changes.",
    note_pictet_both => "The index “Switzerland (Pictet)” is no longer included for licensing reasons. The investment strategy now uses {first} and {second}; please check under “Wealth & investment”. The result changes.",
    note_own_series => "The custom time series “{name}” is no longer supported. The investment strategy is now “World (JST)” with Swiss Confederation bonds; please check under “Wealth & investment”. The result changes.",
    note_spending_includes_housing => "Living expenses now include housing costs. The housing costs at the simulation start ({housing:fix0} per month) have been added to each value; the result stays the same.",
    note_milestone_added => "The milestone “{name}” was missing and is now set to {years} years.",
    note_milestone_renamed => "The milestone “{old}” is now called “{name}”.",
    note_milestone_age_limited => "“{name}” was at {age} years, outside the limits of {lo} to {hi} years; now {years} years.",
    note_milestone_own_person => "“{name}” now refers to the age of its own person ({years} years).",
    note_pk_reference => "The pension fund payout now starts at the milestone “{milestone}” (previously “{old}”).",
    note_ahv_reference => "The AHV pension now starts at the milestone “{milestone}” (previously “{old}”).",
    note_pillar3a_repay_to_wealth => "Pillar 3a of “{name}”: “Repay mortgage” now requires a tranche with indirect amortization; the payout goes to liquid wealth.",
    note_pillar3a_account_rest_at_ahv => "The pillar 3a of “{name}” is now a separate 3a account (section “Pillar 3a”); the remainder is paid out in the month the AHV pension starts.",
    note_pillar3a_account_moved => "The pillar 3a of “{name}” is now a separate 3a account (section “Pillar 3a”); the payout ({old:ym}) was more than 5 years before the start of the AHV pension and is now in {date:ym}.",
    note_pillar3a_account_payout => "The pillar 3a of “{name}” is now a separate 3a account (section “Pillar 3a”); the whole balance is paid out in {date:ym}.",
    note_pillar3a_account_no_date => "The pillar 3a of “{name}” is now a separate 3a account (section “Pillar 3a”); without a payout date, the balance is paid out in the month the AHV pension starts.",
    note_pillar3a_merged => "Several 3a balances have been combined into one account (interest of the first account: {rate:pct2} %).",
    // Import of the old configuration (legacy.rs)
    legacy_lumpsums_merged => "Multiple lump-sum withdrawals (LumpsumRatio) are combined into one withdrawal at retirement.",
    legacy_monte_carlo => "Monte Carlo imported (parametric stock model as before); bonds and inflation follow the fixed values.",
    legacy_iteration => "CalculationMode “Iteration” is not imported; variations can be entered as scenarios.",
    legacy_property_ignored => "Property “{name}” with status “{status}” is ignored.",
    legacy_properties_approximate => "Properties are imported approximately: mortgages are renewed on the same terms (affordability only checked for a purchase after a sale), amortization per quarter = 3/4 of the old value (same monthly expense).",
    // Commands (dispatch.rs)
    path_missing => "Path missing",
    command_desktop_only => "Command “{cmd}” is only available in the desktop app",
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
            "3 1'234'568 1234568 4.2 4.25 4 07.2031 Anna A, B 65.5 -2'500 Cancelled {missing_fmt:}"
        );
        // Braces that are no placeholder stay.
        assert_eq!(render("{ a } {", &params), "{ a } {");
    }

    /// #187/AK1: formats `fix1` and `fix2` round to one and two decimals (not the plain number).
    #[test]
    fn issue_187_ak1_fixed_decimals() {
        let params: Params = vec![("x", arg(&1234.5678))];
        assert_eq!(render("{x:fix1} {x:fix2} {x}", &params), "1234.6 1234.57 1234.5678");
    }

    /// #187/AK1: messages are equal only with equal code and equal parameters; comparison with texts and the
    /// conversion to a string use the rendered text.
    #[test]
    fn issue_187_ak1_equality_text_and_parameters() {
        assert_eq!(msg!(person_n, n = 1), msg!(person_n, n = 1));
        assert_ne!(msg!(person_n, n = 1), msg!(person_n, n = 2));
        assert_ne!(msg!(cancelled), msg!(plain_text, text = "Cancelled"));
        let m = msg!(cancelled);
        assert!(m == "Cancelled");
        assert!(m != "Nope");
        let (same, other) = (String::from("Cancelled"), String::from("Nope"));
        assert!(m == same);
        assert!(m != other);
        assert_eq!(String::from(msg!(cancelled)), "Cancelled");
        let m = Message::new(Code::person_n, vec![("a", Arg::Int(1)), ("n", Arg::Int(2))]);
        assert_eq!(m.param("n"), Some(&Arg::Int(2)));
        assert_eq!(m.param("a"), Some(&Arg::Int(1)));
        assert_eq!(m.param("x"), None);
        assert_eq!(arg(&true), Arg::Bool(true));
        assert_eq!(arg(&false), Arg::Bool(false));
    }

    #[test]
    fn issue_171_message_serializes_code_params_text() {
        let m = msg!(cancelled);
        assert_eq!(
            serde_json::to_value(&m).unwrap(),
            serde_json::json!({ "code": "cancelled", "params": {}, "text": "Cancelled" })
        );
        assert_eq!(m, "Cancelled");
        assert_eq!(m.to_string(), "Cancelled");
        assert_eq!(Code::parse("cancelled"), Some(Code::cancelled));
        assert_eq!(Code::parse("nope"), None);
    }

    /// #171/AK3: The examples of all codes for the test of the user interface (`e2e/unit/engine-messages.json`:
    /// code, English template, parameters, text). A new code changes the file: write it again with
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

    /// #182/AK5: the English templates corrected in the review of the translations (the same text as `en.json`).
    #[test]
    fn issue_182_ak5_english_templates_follow_the_review() {
        let cases = [
            (Code::event_amortization_direct, "directly instead of via pillar 3a"),
            (Code::import_chf_ends, "series ends in {month:ym}"),
            (Code::import_chf_ends, "do not extend further"),
            (Code::index_world_custom, "World, custom (JST)"),
            (Code::index_world_custom_weights, "World, custom (CH"),
            (Code::note_pillar3a_repay_to_wealth, "goes to liquid wealth"),
            (Code::paths_memory, "for the available memory"),
            (Code::pillar3a_payout_repay_without_indirect, "goes to liquid wealth"),
            (Code::pillar3a_repay_without_indirect, "goes to liquid wealth"),
        ];
        for (code, part) in cases {
            assert!(code.template().contains(part), "{}: {}", code.as_str(), code.template());
        }
        for &c in Code::ALL {
            assert!(!c.template().contains("free wealth"), "{}", c.as_str());
        }
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
            "“Pension fund payout” (07.2031) is before the simulation start."
        );
        assert_eq!(
            params_json(m.args()),
            serde_json::json!({ "name": { "code": "milestone_retire", "params": {}, "text": "Pension fund payout" }, "month": "2031-07" })
        );
        assert_eq!(render("{x}|{y}", &[("x", Arg::Null), ("y", Arg::Bool(true))]), "|true");
    }
}
