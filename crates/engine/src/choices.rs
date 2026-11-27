//! #304: Choice fields of the plan for AIs (`describe choices`): every value of a field with fixed values, with its
//! label in the app and its meaning. People ask with the labels of the app («Monte Carlo · historisch»), the plan
//! stores internal values (`bootstrap`); without this table an AI guesses the value.
//!
//! The labels come from the text tables of the user interface (`crates/app/ui/i18n/<language>.json`), compiled in:
//! [`LABELS`] holds only the keys, no second copy of the texts. Meanings of the values come from the schema (doc
//! comment of an enum variant), missing ones from [`VALUE_MEANINGS`]. The tests check the tables against the schema
//! in both directions.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::describe::{schema_leaves, Item};

/// Languages of the user interface; the first one is the fallback.
pub const LANGUAGES: [&str; 4] = ["de", "en", "fr", "it"];

/// Text tables of the user interface, the same files the app compiles in.
const TABLES: [(&str, &str); 4] = [
    ("de", include_str!("../../app/ui/i18n/de.json")),
    ("en", include_str!("../../app/ui/i18n/en.json")),
    ("fr", include_str!("../../app/ui/i18n/fr.json")),
    ("it", include_str!("../../app/ui/i18n/it.json")),
];

/// Most characters of the meaning of a value.
pub const VALUE_MEANING_CHARS: usize = 120;

/// Labels of the values in the app: (field path, value, keys of the text tables). Several keys: the app shows the
/// value under several names (long in the selection, short in lists). `=Name`: a name the app shows the same in every
/// language, not in the text tables.
pub const LABELS: &[(&str, &str, &[&str])] = &[
    ("milestones.*.kind", "date", &["milestones.date"]),
    ("milestones.*.kind", "age", &["milestones.ageOf"]),
    ("optimization.goal", "consumption", &["optimize.goal1"]),
    ("optimization.goal", "keepProperty", &["optimize.goal2"]),
    ("optimization.property", "planned", &["optimize.propertyPlanned"]),
    ("optimization.property", "neverSell", &["optimize.propertyNeverSell"]),
    ("optimization.property", "both", &["optimize.propertyBoth"]),
    ("pillar3a.*.use", "repay", &["pillar3a.repayMortgage"]),
    ("pillar3a.*.use", "wealth", &["pillar3a.toWealth"]),
    ("pillar3a.*.payouts.*.kind", "date", &["pillar3a.date"]),
    ("pillar3a.*.payouts.*.kind", "age", &["pillar3a.age"]),
    ("pillar3a.*.payouts.*.use", "repay", &["pillar3a.repayMortgage"]),
    ("pillar3a.*.payouts.*.use", "wealth", &["pillar3a.toWealth"]),
    ("pillar3a.*.payouts.*.use", "home", &["pillar3a.homeEarly"]),
    ("realestate.properties.*.status", "owned", &["property.owned"]),
    ("realestate.properties.*.status", "planned", &["property.planned"]),
    ("realestate.properties.*.status", "rent", &["property.rent"]),
    (
        "realestate.properties.*.mortgage.amountMode",
        "fixed",
        &["mortgage.asEntered"],
    ),
    (
        "realestate.properties.*.mortgage.amountMode",
        "maxLoan",
        &["mortgage.eightyPct"],
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.after",
        "renew",
        &["mortgage.extend"],
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.after",
        "repay",
        &["mortgage.repay"],
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.amortMode",
        "direct",
        &["mortgage.direct"],
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.amortMode",
        "indirect",
        &["mortgage.indirect"],
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.type",
        "fixed",
        &["mortgage.fixed"],
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.type",
        "saron",
        &["=SARON"],
    ),
    (
        "realestate.properties.*.taxValue.kind",
        "amount",
        &["realestate.taxValueAmount"],
    ),
    (
        "realestate.properties.*.taxValue.kind",
        "rate",
        &["realestate.taxValueRate"],
    ),
    ("returns.source", "fixed", &["source.fixed"]),
    ("returns.source", "historical", &["source.historical"]),
    ("returns.source", "rolling", &["source.rolling"]),
    ("returns.source", "bootstrap", &["source.bootstrap"]),
    ("returns.source", "parametric", &["source.parametric"]),
    ("returns.extrapolation.method", "fixed", &["returns.fixedValues"]),
    ("returns.extrapolation.method", "average", &["returns.movingAvg"]),
    ("returns.extrapolation.method", "geometric", &["returns.movingGeoAvg"]),
    ("returns.strategy.bonds", "ch", &["index.confed10"]),
    ("returns.strategy.bonds", "usa", &["index.us10"]),
    ("returns.strategy.bonds", "chShort", &["index.confedShort"]),
    ("returns.strategy.stocks", "chJst", &["index.swiss"]),
    ("returns.strategy.stocks", "worldJst", &["index.world"]),
    ("returns.strategy.stocks", "worldCustom", &["index.worldCustom"]),
    ("returns.strategy.stocks", "sp500", &["index.sp500"]),
    ("taxes.mode", "own", &["taxes.modeOwn"]),
    ("taxes.mode", "location", &["taxes.modeLocation"]),
    ("taxes.deductions.*.kind", "canton", &["deductions.canton"]),
    ("taxes.deductions.*.kind", "own", &["deductions.own"]),
    ("taxes.location.church", "none", &["church.none"]),
    ("taxes.location.church", "reformed", &["church.reformed"]),
    ("taxes.location.church", "catholic", &["church.catholic"]),
    ("taxes.location.church", "christCatholic", &["church.christCatholic"]),
    ("taxes.relocations.*.church", "none", &["church.none"]),
    ("taxes.relocations.*.church", "reformed", &["church.reformed"]),
    ("taxes.relocations.*.church", "catholic", &["church.catholic"]),
    (
        "taxes.relocations.*.church",
        "christCatholic",
        &["church.christCatholic"],
    ),
    ("taxes.propertyGain.model", "zurich", &["gain.modelZh"]),
    ("taxes.propertyGain.model", "aargau", &["gain.modelAg"]),
    ("withdrawal.strategy", "standard", &["strat.standard"]),
    (
        "withdrawal.strategy",
        "three_buckets",
        &["strat.buckets", "strat.bucketsShort"],
    ),
    (
        "withdrawal.strategy",
        "three_buckets_guardrails",
        &["opt.bucketsGuardrails"],
    ),
    ("withdrawal.strategy", "guardrails", &["strat.guardrails"]),
    (
        "withdrawal.strategy",
        "fixed_rate",
        &["strat.fixedRate", "strat.fixedRateShort"],
    ),
    ("withdrawal.strategy", "vpw", &["strat.vpw"]),
    (
        "withdrawal.strategy",
        "endowment",
        &["strat.endowment", "strat.endowmentShort"],
    ),
    ("withdrawal.buckets.spending", "planned", &["withdrawal.asPlanned"]),
    (
        "withdrawal.buckets.spending",
        "guardrails",
        &["withdrawal.withGuardrails"],
    ),
];

/// Values without a choice in the app, so without a label: (field path, value or `*` for all, reason).
pub const WITHOUT_LABEL: &[(&str, &str, &str)] = &[
    (
        "realestate.properties.*.mortgage.pillar3a.use",
        "*",
        "pillar 3a on the mortgage of older plans (before #52), only read",
    ),
    (
        "pillar3a.*.use",
        "home",
        "#301: early withdrawal only as a single payout; for the remainder and the tranches an error",
    ),
    (
        "taxes.propertyGain.model",
        "curve",
        "set by the tax data of the canton (BL), no choice in the app",
    ),
    (
        "taxes.propertyGain.model",
        "return",
        "set by the tax data of the canton (ZG), no choice in the app",
    ),
];

/// Combinations the app shows as a value of their own, but which are not a value of the field: (field path, name).
pub const COMBINED: &[(&str, &str)] = &[("withdrawal.strategy", "three_buckets_guardrails")];

/// Meanings of values the schema does not explain (tagged variants, combinations): (field path, value, meaning).
pub const VALUE_MEANINGS: &[(&str, &str, &str)] = &[
    ("milestones.*.kind", "date", "Fixed date (`date`)."),
    ("milestones.*.kind", "age", "Age of a person (`person`, `years`)."),
    (
        "withdrawal.strategy",
        "three_buckets_guardrails",
        "Not a value of its own: three_buckets with withdrawal.buckets.spending = guardrails.",
    ),
];

fn tables() -> &'static BTreeMap<&'static str, Map<String, Value>> {
    static T: OnceLock<BTreeMap<&'static str, Map<String, Value>>> = OnceLock::new();
    T.get_or_init(|| {
        TABLES
            .iter()
            .map(|(lang, text)| (*lang, serde_json::from_str(text).expect("text table is valid JSON")))
            .collect()
    })
}

/// The language if the user interface has it, otherwise German.
pub fn known_language(language: &str) -> &'static str {
    LANGUAGES
        .iter()
        .copied()
        .find(|l| *l == language)
        .unwrap_or(LANGUAGES[0])
}

/// Language of the user interface: the choice saved in the settings of the app (`language`); with «wie System» or
/// without a choice the language of the locale (`de_CH.UTF-8` → `de`); German if there is no table for it.
pub fn ui_language(choice: Option<&str>, locale: Option<&str>) -> &'static str {
    let part = |s: &str| s.split(['-', '_', '.', '@']).next().unwrap_or_default().to_lowercase();
    [choice, locale]
        .into_iter()
        .flatten()
        .map(part)
        .find_map(|l| LANGUAGES.iter().copied().find(|x| *x == l))
        .unwrap_or(LANGUAGES[0])
}

/// Text of a key of the text tables in a language (`None`: missing or empty); `=Name` is the name itself.
pub fn label(language: &str, key: &str) -> Option<String> {
    if let Some(name) = key.strip_prefix('=') {
        return Some(name.to_string());
    }
    tables()
        .get(known_language(language))
        .and_then(|t| t.get(key))
        .and_then(Value::as_str)
        .filter(|t| !t.trim().is_empty())
        .map(str::to_string)
}

/// A value of a choice field.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub path: String,
    pub value: String,
    /// Labels in the app (empty: no choice in the app, see [`WITHOUT_LABEL`]).
    pub labels: Vec<String>,
    pub meaning: String,
}

/// Every value of every choice field of the plan (and the combinations of [`COMBINED`]), in the order of the schema,
/// with the labels in `language`.
pub fn values(language: &str) -> Vec<Choice> {
    let labels: BTreeMap<(&str, &str), &[&str]> = LABELS.iter().map(|&(p, v, k)| ((p, v), k)).collect();
    let extra: BTreeMap<(&str, &str), &str> = VALUE_MEANINGS.iter().map(|&(p, v, m)| ((p, v), m)).collect();
    let mut out = vec![];
    for leaf in schema_leaves().into_iter().filter(|l| l.kind == "enum") {
        let mut vals = leaf.values.clone();
        vals.extend(
            COMBINED
                .iter()
                .filter(|(p, _)| *p == leaf.path)
                .map(|(_, v)| v.to_string()),
        );
        for v in vals {
            let key = (leaf.path.as_str(), v.as_str());
            let meaning = extra
                .get(&key)
                .map(|m| m.to_string())
                .or_else(|| leaf.value_meanings.get(&v).map(|m| short(m)))
                .unwrap_or_default();
            let labels = labels
                .get(&key)
                .into_iter()
                .flat_map(|keys| keys.iter())
                .filter_map(|k| label(language, k))
                .collect();
            out.push(Choice {
                path: leaf.path.clone(),
                value: v,
                labels,
                meaning,
            });
        }
    }
    out
}

/// Meaning of a value in short: without issue numbers, at most [`VALUE_MEANING_CHARS`].
fn short(text: &str) -> String {
    let number = |n: &str| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit());
    let mut words: Vec<String> = vec![];
    for (i, w) in text.split_whitespace().enumerate() {
        let bare = w.trim_start_matches('(').trim_end_matches([')', ':', ';', ',', '.']);
        // `#65:` in the text, `65:` in front (the schema takes `#` at the start for a heading).
        let issue = bare.strip_prefix('#').is_some_and(number) || (i == 0 && w.ends_with(':') && number(bare));
        if !issue {
            words.push(w.to_string());
        } else if w.ends_with('.') {
            // "on (#196)." keeps the end of the sentence.
            if let Some(last) = words.last_mut() {
                last.push('.');
            }
        }
    }
    let mut t = words.join(" ");
    if let Some(first) = t.char_indices().next().map(|(_, c)| c) {
        t = first.to_uppercase().chain(t.chars().skip(1)).collect();
    }
    if t.chars().count() > VALUE_MEANING_CHARS {
        let cut: String = t.chars().take(VALUE_MEANING_CHARS - 1).collect();
        let at = cut.rfind(' ').unwrap_or(cut.len());
        t = format!("{}…", cut[..at].trim_end_matches([',', ';', ' ']));
    }
    t
}

/// Section `choices` of the catalog.
#[derive(Clone, Debug, Serialize)]
pub struct Choices {
    /// Language of the labels (the language of the user interface).
    pub language: &'static str,
    pub notes: Vec<&'static str>,
    /// Field path → value → `«label» – meaning`.
    #[serde(serialize_with = "nested")]
    pub fields: Vec<(String, Vec<Item>)>,
}

fn nested<S: serde::Serializer>(fields: &[(String, Vec<Item>)], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    struct Values<'a>(&'a [Item]);
    impl Serialize for Values<'_> {
        fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut m = s.serialize_map(Some(self.0.len()))?;
            for i in self.0 {
                m.serialize_entry(&i.name, &i.meaning)?;
            }
            m.end()
        }
    }
    let mut m = s.serialize_map(Some(fields.len()))?;
    for (path, items) in fields {
        m.serialize_entry(path, &Values(items))?;
    }
    m.end()
}

/// The section `choices` with the labels in `language`.
pub fn choices(language: &str) -> Choices {
    let mut fields: Vec<(String, Vec<Item>)> = vec![];
    for c in values(language) {
        let quoted: Vec<String> = c.labels.iter().map(|l| format!("«{l}»")).collect();
        let text = match (quoted.is_empty(), c.meaning.is_empty()) {
            (false, false) => format!("{} – {}", quoted.join(" / "), c.meaning),
            (false, true) => quoted.join(" / "),
            _ => c.meaning,
        };
        let item = Item {
            name: c.value,
            meaning: text,
        };
        match fields.last_mut() {
            Some((p, items)) if *p == c.path => items.push(item),
            _ => fields.push((c.path, vec![item])),
        }
    }
    Choices {
        language: known_language(language),
        notes: vec![
            "Value of the plan → «label in the app» – meaning. People name the label; set the value (plan_set).",
            "Several labels: the app shows the value under each (long and short name).",
            "Without a label: no choice in the app.",
        ],
        fields,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_304_ak2_ui_language_from_choice_or_locale() {
        assert_eq!(ui_language(Some("fr"), Some("de_CH.UTF-8")), "fr");
        assert_eq!(ui_language(Some("system"), Some("it_CH.UTF-8")), "it");
        assert_eq!(ui_language(None, Some("en-US")), "en");
        assert_eq!(ui_language(None, Some("es_ES")), "de");
        assert_eq!(ui_language(None, None), "de");
        assert_eq!(known_language("xx"), "de");
    }

    #[test]
    fn issue_304_ak1_short_meaning() {
        assert_eq!(short("#76: short-term paper."), "Short-term paper.");
        assert_eq!(short("65: Rate by the gain."), "Rate by the gain.");
        assert_eq!(short("With the switch on (#196)."), "With the switch on.");
        assert_eq!(short("Rules of the file (#63) only."), "Rules of the file only.");
        let long = "word ".repeat(60);
        let s = short(&long);
        assert!(s.chars().count() <= VALUE_MEANING_CHARS && s.ends_with('…'), "{s}");
        assert_eq!(label("de", "=SARON").as_deref(), Some("SARON"));
        assert_eq!(label("de", "no.such.key"), None);
    }
}
