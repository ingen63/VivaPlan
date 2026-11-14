//! FEA-24/FEA-37/FEA-38: `api::upgrade_plan` bringt eine ältere Planung auf den aktuellen Stand und gibt die
//! Einstellungen (`limits`) nicht weiter.

use engine::api;
use serde_json::json;

#[test]
fn upgrade_plan_adds_missing_milestones_and_drops_limits() {
    let plan = json!({
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-06" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "age", "person": "A", "years": 60 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2070-12" }
        ],
        "pension": { "pk": { "at": "retire" }, "ahv": { "at": "never" } },
        "limits": { "ahvMin": 0, "ahvMax": 130 }
    });
    let out = api::upgrade_plan(&plan);
    assert!(out.plan.get("limits").is_none(), "Grenzen werden nicht übernommen");
    assert!(!out.notes.is_empty(), "jede Änderung wird gemeldet");
    let ids: Vec<&str> = out.plan["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"ahv"), "AHV-Bezug ergänzt: {ids:?}");
    assert!(plan.get("limits").is_some(), "Eingabe bleibt unverändert");
}
