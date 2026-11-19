//! Calculation engine of VivaPlan.
//!
//! Flow: load [`model::Plan`] → [`sim::Simulator::new`] (validates and compiles the plan) →
//! generate return paths from [`market`] → [`sim::Simulator::run`] per path or
//! [`batch::run_all`] for many paths in parallel.

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

// #116: Memory limit per test process (see `test_alloc`).
#[cfg(test)]
#[global_allocator]
static TEST_ALLOC: test_alloc::Limited = test_alloc::Limited;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The plan is incomplete or contradictory.
    Invalid(String),
    /// A file could not be read.
    Io(String),
    /// Function is not yet implemented in this stage.
    NotImplemented(String),
    /// The computation was cancelled.
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

/// JSON schema of the project file (for validation in the editor and as documentation).
pub fn schema_json() -> String {
    serde_json::to_string_pretty(&schemars::schema_for!(model::Plan)).expect("Schema ist serialisierbar")
}

#[cfg(test)]
mod tests {
    // #40: The schema describes the project file.
    #[test]
    fn schema_describes_the_plan() {
        let v: serde_json::Value = serde_json::from_str(&super::schema_json()).unwrap();
        assert_eq!(v["title"], "Plan");
        for key in ["format", "project", "household", "milestones", "returns", "taxes"] {
            assert!(v["properties"].get(key).is_some(), "{key}");
        }
    }
}
