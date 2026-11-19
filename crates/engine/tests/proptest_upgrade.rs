//! #137: Updating older plans (`api::upgrade_plan`, runs on every opening) is idempotent: a second run
//! changes nothing and reports nothing, also for corrupted or incomplete plans. Number of cases via
//! `PROPTEST_CASES` (nightly 10'000, `make proptest`).

use engine::api;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{json, Value};

const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/proptest_upgrade.txt"
);

/// Initial plans: example, new plan, a file in the old format and the ten golden plans (deliberately terse,
/// without fixed milestones per person).
fn bases() -> Vec<Value> {
    let mut v = vec![
        api::example_plan(),
        serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap(),
        serde_json::from_str(include_str!("fixtures/planung-altformat.rplan")).unwrap(),
    ];
    v.extend(crate::golden::cases().into_iter().map(|(_, p)| p));
    v
}

/// JSON pointers of all nodes and leaves.
fn pointers(v: &Value, path: String, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => o
            .iter()
            .for_each(|(k, x)| pointers(x, format!("{path}/{}", k.replace('~', "~0").replace('/', "~1")), out)),
        Value::Array(a) => a
            .iter()
            .enumerate()
            .for_each(|(i, x)| pointers(x, format!("{path}/{i}"), out)),
        _ => {}
    }
    if !path.is_empty() {
        out.push(path);
    }
}

/// Corrupts a field: delete, null, number, text, empty list or empty object.
fn mutate(plan: &mut Value, at: usize, kind: u8, n: f64) {
    let mut ps = vec![];
    pointers(plan, String::new(), &mut ps);
    // The limits (`limits`) come from the checked settings of the device, not from the file: do not corrupt them.
    ps.retain(|p| !p.starts_with("/limits"));
    let Some(p) = ps.get(at % ps.len().max(1)).cloned() else {
        return;
    };
    let (parent, key) = p.rsplit_once('/').unwrap();
    let key = key.replace("~1", "/").replace("~0", "~");
    let new = match kind % 6 {
        0 => None,
        1 => Some(Value::Null),
        2 => Some(json!(n)),
        3 => Some(json!(
            ["never", "start", "retire", "ahv", "65", "2030-01", "x"][(n.abs() as usize) % 7]
        )),
        4 => Some(json!([])),
        _ => Some(json!({})),
    };
    let target = if parent.is_empty() {
        Some(&mut *plan)
    } else {
        plan.pointer_mut(parent)
    };
    match (target, new) {
        (Some(Value::Object(o)), None) => {
            o.remove(&key);
        }
        (Some(Value::Object(o)), Some(x)) => {
            o.insert(key, x);
        }
        (Some(Value::Array(a)), x) => {
            if let Ok(i) = key.parse::<usize>() {
                match x {
                    None if i < a.len() => {
                        a.remove(i);
                    }
                    Some(x) if i < a.len() => a[i] = x,
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// The limits (`limits`, settings of the device) do not belong to the plan: the user interface sends them with every
/// call, `upgrade_plan` removes them from the result. Hence the second run gets the same limits as the
/// first (the golden plans carry wide limits via `any_age`).
fn with_limits_of(original: &Value, upgraded: &Value) -> Value {
    let mut v = upgraded.clone();
    if let (Some(l), Some(o)) = (original.get("limits"), v.as_object_mut()) {
        o.insert("limits".into(), l.clone());
    }
    v
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    })]

    /// #137/AK4: After the first update a second one changes nothing and reports nothing.
    #[test]
    fn proptest_upgrade_is_idempotent(
        base in 0..13usize,
        edits in prop::collection::vec((any::<usize>(), any::<u8>(), -1e6..1e6f64), 0..4)
    ) {
        let mut plan = bases()[base].clone();
        for (at, kind, n) in edits {
            mutate(&mut plan, at, kind, n);
        }
        let once = api::upgrade_plan(&plan);
        let twice = api::upgrade_plan(&with_limits_of(&plan, &once.plan));
        prop_assert!(twice.notes.is_empty(), "zweiter Lauf meldet {:?}", twice.notes);
        prop_assert_eq!(&twice.plan, &once.plan);
    }
}

/// #137/AK5: The bundled plans are up to date: updating changes nothing and reports nothing.
/// The golden plans are deliberately terse (milestones as dates, without fixed milestones per person); for them
/// idempotence applies (second run without a change), checked in the test below and in the property test above.
#[test]
fn issue_137_ak5_shipped_plans_need_no_upgrade() {
    for (name, plan) in [
        ("beispiel.vplan", api::example_plan()),
        (
            "Neu.vplan",
            serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap(),
        ),
    ] {
        let out = api::upgrade_plan(&plan);
        assert!(out.notes.is_empty(), "{name}: {:?}", out.notes);
        assert_eq!(out.plan, plan, "{name}");
    }
    for (name, plan) in crate::golden::cases() {
        let once = api::upgrade_plan(&plan);
        let twice = api::upgrade_plan(&with_limits_of(&plan, &once.plan));
        assert!(twice.notes.is_empty(), "{name}: {:?}", twice.notes);
        assert_eq!(twice.plan, once.plan, "{name}");
    }
}
