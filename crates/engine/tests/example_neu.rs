//! #4: The bundled example plan `crates/engine/assets/Neu.vplan` is readable, without errors and can be simulated.

use engine::api;
use engine::model::Plan;
use engine::sim::Simulator;
use engine::validate::Severity;
use std::path::Path;

#[test]
fn bundled_example_validates_and_runs() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/Neu.vplan");
    let value = api::read_plan(&path).expect("Beispielplanung lesbar");
    let errors: Vec<_> = api::validate(&value)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let plan: Plan = serde_json::from_value(value).expect("Beispielplanung gültig");
    assert!(Simulator::new(&plan).is_ok());
}
