//! #137: Nachführen älterer Planungen (`api::upgrade_plan`, läuft bei jedem Öffnen) ist idempotent: Ein zweiter Lauf
//! ändert nichts und meldet nichts, auch bei verfälschten oder unvollständigen Planungen. Fallzahl über
//! `PROPTEST_CASES` (nächtlich 10'000, `make proptest`).

use engine::api;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{json, Value};

const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/proptest_upgrade.txt"
);

/// Ausgangsplanungen: Beispiel, neue Planung, eine Datei im alten Format und die zehn Golden-Planungen (bewusst knapp,
/// ohne feste Meilensteine je Person).
fn bases() -> Vec<Value> {
    let mut v = vec![
        api::example_plan(),
        serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap(),
        serde_json::from_str(include_str!("fixtures/planung-altformat.rplan")).unwrap(),
    ];
    v.extend(crate::golden::cases().into_iter().map(|(_, p)| p));
    v
}

/// JSON-Pointer aller Knoten und Blätter.
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

/// Verfälscht ein Feld: löschen, null, Zahl, Text, leere Liste oder leeres Objekt.
fn mutate(plan: &mut Value, at: usize, kind: u8, n: f64) {
    let mut ps = vec![];
    pointers(plan, String::new(), &mut ps);
    // Die Grenzen (`limits`) stammen aus den geprüften Einstellungen des Geräts, nicht aus der Datei: nicht verfälschen.
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

/// Die Grenzen (`limits`, Einstellungen des Geräts) gehören nicht zur Planung: Die Oberfläche schickt sie bei jedem
/// Aufruf mit, `upgrade_plan` entfernt sie aus dem Ergebnis. Der zweite Lauf bekommt deshalb dieselben Grenzen wie der
/// erste (die Golden-Planungen tragen über `any_age` weite Grenzen).
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

    /// #137/AK4: Nach dem ersten Nachführen ändert ein zweites nichts und meldet nichts.
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

/// #137/AK5: Die mitgelieferten Planungen sind auf dem aktuellen Stand: Nachführen ändert nichts und meldet nichts.
/// Die Golden-Planungen sind bewusst knapp (Meilensteine als Datum, ohne feste Meilensteine je Person); für sie gilt die
/// Idempotenz (zweiter Lauf ohne Änderung), geprüft im Test unten und im Eigenschaftstest oben.
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
