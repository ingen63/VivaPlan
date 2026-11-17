//! Kalendermonate und Zeitbezüge («ab Pensionierung», «2027-05», «nie»).

use std::fmt;
use std::str::FromStr;

use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject, StringValidation};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Ein Kalendermonat, intern als fortlaufender Index `Jahr * 12 + (Monat - 1)`.
/// In Dateien als `"JJJJ-MM"`, in Meldungen als `MM.JJJJ`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct YearMonth(pub i32);

impl YearMonth {
    /// Überlaufsicher (#60): extreme Jahre ergeben einen sehr fernen Monat statt eines Absturzes.
    pub fn new(year: i32, month: u32) -> Self {
        YearMonth(year.saturating_mul(12).saturating_add(month as i32 - 1))
    }
    pub fn year(self) -> i32 {
        self.0.div_euclid(12)
    }
    pub fn month(self) -> u32 {
        (self.0.rem_euclid(12) + 1) as u32
    }
    /// Überlaufsicher: extreme Eingaben ergeben einen sehr fernen Monat statt eines Absturzes.
    pub fn plus(self, months: i32) -> Self {
        YearMonth(self.0.saturating_add(months))
    }
    /// Anzahl Monate von `self` bis `other` (positiv, wenn `other` später liegt).
    pub fn until(self, other: YearMonth) -> i32 {
        other.0.saturating_sub(self.0)
    }
    pub fn is_december(self) -> bool {
        self.month() == 12
    }
    /// Ende eines Quartals (März, Juni, September, Dezember).
    pub fn is_quarter_end(self) -> bool {
        self.month().is_multiple_of(3)
    }
    pub fn iso(self) -> String {
        format!("{:04}-{:02}", self.year(), self.month())
    }
}

impl fmt::Display for YearMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}.{:04}", self.month(), self.year())
    }
}

impl FromStr for YearMonth {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || format!("«{s}» ist kein gültiger Monat (erwartet JJJJ-MM)");
        let (y, m) = s.trim().split_once('-').ok_or_else(err)?;
        let year: i32 = y.parse().map_err(|_| err())?;
        let month: u32 = m.parse().map_err(|_| err())?;
        if y.len() != 4 || !(1..=12).contains(&month) {
            return Err(err());
        }
        Ok(YearMonth::new(year, month))
    }
}

impl Serialize for YearMonth {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.iso())
    }
}

impl<'de> Deserialize<'de> for YearMonth {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for YearMonth {
    fn schema_name() -> String {
        "YearMonth".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                pattern: Some(r"^\d{4}-(0[1-9]|1[0-2])$".into()),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

/// Verweis auf einen Zeitpunkt: Meilenstein-ID, fixes Datum `"JJJJ-MM"` oder `"never"`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct TimeRef(pub String);

impl TimeRef {
    pub const NEVER: &'static str = "never";
    /// Kauf bzw. Miete, sobald die letzte eigene Immobilie verkauft ist.
    pub const AFTER_SALE: &'static str = "afterSale";

    pub fn milestone(id: &str) -> Self {
        TimeRef(id.to_string())
    }
    pub fn date(m: YearMonth) -> Self {
        TimeRef(m.iso())
    }
    pub fn never() -> Self {
        TimeRef(Self::NEVER.to_string())
    }
    pub fn is_never(&self) -> bool {
        self.0 == Self::NEVER
    }
    pub fn after_sale() -> Self {
        TimeRef(Self::AFTER_SALE.to_string())
    }
    pub fn is_after_sale(&self) -> bool {
        self.0 == Self::AFTER_SALE
    }
    /// «Nach Verkauf von …»: `"afterSale:<Immobilien-ID>"`.
    pub fn after_sale_of(id: &str) -> Self {
        TimeRef(format!("{}:{id}", Self::AFTER_SALE))
    }
    /// Die Immobilien-ID bei `"afterSale:<id>"`.
    pub fn sale_of(&self) -> Option<&str> {
        self.0.strip_prefix("afterSale:")
    }
    /// Zeitpunkt, der erst während der Simulation feststeht.
    pub fn is_dynamic(&self) -> bool {
        self.is_after_sale() || self.sale_of().is_some()
    }
    /// Das fixe Datum, falls der Verweis eines ist.
    pub fn as_date(&self) -> Option<YearMonth> {
        if self.0.len() == 7 && self.0.as_bytes()[4] == b'-' {
            self.0.parse().ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_format() {
        let m: YearMonth = "2027-05".parse().unwrap();
        assert_eq!(m.year(), 2027);
        assert_eq!(m.month(), 5);
        assert_eq!(m.to_string(), "05.2027");
        assert_eq!(m.iso(), "2027-05");
        assert_eq!(m.plus(8).iso(), "2028-01");
        assert!("2027-13".parse::<YearMonth>().is_err());
        assert!("27-01".parse::<YearMonth>().is_err());
    }

    #[test]
    fn calendar_helpers() {
        assert!(YearMonth::new(2026, 12).is_december());
        assert!(YearMonth::new(2026, 6).is_quarter_end());
        assert!(!YearMonth::new(2026, 5).is_quarter_end());
        assert_eq!(YearMonth::new(2026, 3).until(YearMonth::new(2027, 5)), 14);
    }

    #[test]
    fn time_ref_kinds() {
        assert_eq!(TimeRef("2030-01".into()).as_date(), Some(YearMonth::new(2030, 1)));
        assert_eq!(TimeRef("retire".into()).as_date(), None);
        assert!(TimeRef::never().is_never());
    }

    #[test]
    fn schema_describes_a_year_month_string() {
        let schema = serde_json::to_value(schemars::schema_for!(YearMonth)).unwrap();
        assert_eq!(schema["title"], "YearMonth");
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["pattern"], r"^\d{4}-(0[1-9]|1[0-2])$");
    }
}
