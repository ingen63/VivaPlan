//! Tests that need a resource (#224): Windows, Unix, the local original data.
//!
//! Locally such a test may be skipped, but only with a reason in the test output. In the CI job that provides the
//! resource, the job sets the variable of the resource to `1`; there a missing resource makes the test fail instead of
//! being skipped silently. Rule per variable: empty or not set = skip, `1` = required, any other value = error that
//! names the variable and the allowed value. The variables and the jobs that set them are listed in `docs/TESTS.md`.

/// A resource that a test needs, with the environment variable that makes it required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resource {
    /// Environment variable, e.g. `VIVAPLAN_WINDOWS_REQUIRED`.
    pub var: &'static str,
    /// What is missing when the test is skipped (German, appears in the test output).
    pub what: &'static str,
}

/// Windows (path tests with drive letters and backslashes, #141).
pub const WINDOWS: Resource = Resource {
    var: "VIVAPLAN_WINDOWS_REQUIRED",
    what: "Windows",
};

/// Unix (macOS, Linux): tests with file permissions and symbolic links (#136).
pub const UNIX: Resource = Resource {
    var: "VIVAPLAN_UNIX_REQUIRED",
    what: "Unix (macOS, Linux)",
};

/// Shiller's original spreadsheet `crates/engine/data/quellen/shiller-index.ods`; it only exists locally (#94).
pub const DATA: Resource = Resource {
    var: "VIVAPLAN_DATA_REQUIRED",
    what: "Shiller-Originaltabelle crates/engine/data/quellen/shiller-index.ods (#94)",
};

/// What a test does with a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The resource is there: run the test.
    Run,
    /// The resource is missing and not required: skip with this reason.
    Skip(String),
}

impl Resource {
    /// Decision for the value of the variable (`None` = not set) and whether the resource is there. Error if the value
    /// is not allowed, or if the resource is required (`1`) but missing.
    pub fn decide(&self, value: Option<&str>, present: bool) -> Result<Decision, String> {
        let required = match value {
            None | Some("") => false,
            Some("1") => true,
            Some(v) => {
                return Err(format!(
                    "{}={v:?} ist nicht erlaubt: «1» (Pflicht) oder leer bzw. nicht gesetzt (überspringen) (#224)",
                    self.var
                ))
            }
        };
        match (present, required) {
            (true, _) => Ok(Decision::Run),
            (false, true) => Err(format!(
                "{}=1 verlangt diesen Test, aber es fehlt: {} (#224)",
                self.var, self.what
            )),
            (false, false) => Ok(Decision::Skip(format!(
                "übersprungen: es fehlt {} (Pflicht mit {}=1, #224)",
                self.what, self.var
            ))),
        }
    }

    /// For a test: `true` = run it; `false` = skip it, the reason is printed to stderr. Panics if the variable has a
    /// value that is not allowed, or if it is `1` and the resource is missing.
    pub fn available(&self, present: bool) -> bool {
        let value = std::env::var_os(self.var).map(|v| v.to_string_lossy().into_owned());
        match self.decide(value.as_deref(), present) {
            Ok(Decision::Run) => true,
            Ok(Decision::Skip(reason)) => {
                eprintln!("{reason}");
                false
            }
            Err(e) => panic!("{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: Resource = Resource {
        var: "VIVAPLAN_TEST_RESOURCE_REQUIRED",
        what: "Testressource",
    };

    /// #224/AK2: empty or not set = skip with a reason, `1` = required, a present resource always runs.
    #[test]
    fn issue_224_ak2_empty_or_unset_skips_and_present_runs() {
        for value in [None, Some("")] {
            let Ok(Decision::Skip(reason)) = TEST.decide(value, false) else {
                panic!("{value:?} must skip");
            };
            assert!(
                reason.contains("Testressource") && reason.contains(TEST.var),
                "{reason}"
            );
            assert_eq!(TEST.decide(value, true), Ok(Decision::Run));
        }
        assert_eq!(TEST.decide(Some("1"), true), Ok(Decision::Run));
    }

    /// #224/AK2: any value other than empty or `1` is an error naming the variable and the allowed value.
    #[test]
    fn issue_224_ak2_other_values_are_errors_naming_variable_and_value() {
        for v in ["0", "true", "yes", " 1", "1 ", "2"] {
            for present in [true, false] {
                let e = TEST.decide(Some(v), present).unwrap_err();
                assert!(e.contains(TEST.var) && e.contains("«1»"), "{v:?}: {e}");
            }
        }
    }

    /// #224/AK4: required (`1`) and the resource missing gives an error, not a skip.
    #[test]
    fn issue_224_ak4_required_but_missing_is_an_error() {
        let e = TEST.decide(Some("1"), false).unwrap_err();
        assert!(e.contains(TEST.var) && e.contains("Testressource"), "{e}");
    }

    /// #224/AK4: `available` panics if the resource is required but missing (the test fails in CI).
    #[test]
    #[should_panic(expected = "VIVAPLAN_TEST_PANIC_REQUIRED=1 verlangt diesen Test")]
    fn issue_224_ak4_available_panics_when_required_and_missing() {
        // A variable of its own, so that no other test sees it (also under `cargo test`, one process for all tests).
        let r = Resource {
            var: "VIVAPLAN_TEST_PANIC_REQUIRED",
            what: "Testressource",
        };
        std::env::set_var(r.var, "1");
        r.available(false);
    }

    /// #224/AK1: unset, a missing resource is skipped (reason on stderr), a present one runs.
    #[test]
    fn issue_224_ak1_available_without_variable() {
        assert!(TEST.available(true));
        assert!(!TEST.available(false));
    }

    /// The variables are distinct and follow the pattern `VIVAPLAN_*_REQUIRED`.
    #[test]
    fn variables_are_distinct_and_named_consistently() {
        let all = [WINDOWS, UNIX, DATA];
        for (i, r) in all.iter().enumerate() {
            assert!(
                r.var.starts_with("VIVAPLAN_") && r.var.ends_with("_REQUIRED"),
                "{}",
                r.var
            );
            assert!(all[i + 1..].iter().all(|o| o.var != r.var));
        }
    }
}
