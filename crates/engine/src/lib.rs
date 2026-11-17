//! Rechenkern von VivaPlan.
//!
//! Ablauf: [`model::Plan`] laden → [`sim::Simulator::new`] (prüft und kompiliert die Planung) →
//! Renditepfade aus [`market`] erzeugen → [`sim::Simulator::run`] pro Pfad oder
//! [`batch::run_all`] für viele Pfade parallel.

pub mod api;
pub mod batch;
pub mod calibrate;
pub mod dispatch;
pub mod housing;
pub mod import;
pub mod jst;
pub mod legacy;
pub mod library;
pub mod market;
pub mod model;
pub mod montecarlo;
pub mod optimize;
pub mod progress;
pub mod resolve;
pub mod scenarios;
pub mod settings;
pub mod sim;
pub mod strategy;
pub mod tax;
#[doc(hidden)]
pub mod test_alloc;
pub mod time;
pub mod upgrade;
pub mod validate;

use std::fmt;

// #116: Speichergrenze je Testprozess (siehe `test_alloc`).
#[cfg(test)]
#[global_allocator]
static TEST_ALLOC: test_alloc::Limited = test_alloc::Limited;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Die Planung ist unvollständig oder widersprüchlich.
    Invalid(String),
    /// Eine Datei konnte nicht gelesen werden.
    Io(String),
    /// Funktion ist in dieser Etappe noch nicht umgesetzt.
    NotImplemented(String),
    /// Die Berechnung wurde abgebrochen.
    Cancelled,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Invalid(s) => write!(f, "{s}"),
            Error::Io(s) => write!(f, "{s}"),
            Error::NotImplemented(s) => write!(f, "Noch nicht umgesetzt: {s}"),
            Error::Cancelled => write!(f, "{}", crate::progress::CANCELLED),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// JSON-Schema der Projektdatei (für die Prüfung im Editor und als Dokumentation).
pub fn schema_json() -> String {
    serde_json::to_string_pretty(&schemars::schema_for!(model::Plan)).expect("Schema ist serialisierbar")
}

#[cfg(test)]
mod tests {
    // #40: Das Schema beschreibt die Projektdatei.
    #[test]
    fn schema_describes_the_plan() {
        let v: serde_json::Value = serde_json::from_str(&super::schema_json()).unwrap();
        assert_eq!(v["title"], "Plan");
        for key in ["format", "project", "household", "milestones", "returns", "taxes"] {
            assert!(v["properties"].get(key).is_some(), "{key}");
        }
    }
}
