//! #25/#52/#51: `api::upgrade_plan` brings an older plan up to date and does not pass on the
//! settings (`limits`).

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

/// #190/AK12: An older plan file (before the 13th AHV pension) gets the note once; the plan stays the same apart from
/// the marker. Plans from now on (new plan, example) and plans without an AHV pension get no note.
#[test]
fn issue_190_ak12_note_on_the_thirteenth_ahv_pension_once_for_an_older_plan() {
    let note = engine::upgrade::THIRTEENTH_AHV_NOTE;
    let old: serde_json::Value = serde_json::from_str(include_str!("fixtures/planung-altformat.rplan")).unwrap();
    assert!(old["pension"]["ahv"].get("thirteenthNoted").is_none());
    let first = api::upgrade_plan(&old);
    assert_eq!(
        first.notes.iter().filter(|n| *n == note).count(),
        1,
        "{:?}",
        first.notes
    );
    assert_eq!(first.plan["pension"]["ahv"]["thirteenthNoted"], true);
    assert_eq!(
        first.plan["pension"]["ahv"]["monthly"],
        old["pension"]["ahv"]["monthly"]
    );
    let second = api::upgrade_plan(&first.plan);
    assert!(second.notes.iter().all(|n| n != note), "einmalig: {:?}", second.notes);

    for current in [
        include_str!("../assets/new.vplan"),
        include_str!("../assets/example.vplan"),
    ] {
        let v: serde_json::Value = serde_json::from_str(current).unwrap();
        assert!(api::upgrade_plan(&v).notes.iter().all(|n| n != note));
    }

    // without an AHV pension: marker, but no note; with a pension of the partner only: note
    let mut none = old.clone();
    none["pension"]["ahv"]["monthly"] = json!(0);
    let out = api::upgrade_plan(&none);
    assert!(out.notes.iter().all(|n| n != note));
    assert_eq!(out.plan["pension"]["ahv"]["thirteenthNoted"], true);
    none["partner"] = json!({ "pk": { "at": "retire2" }, "ahv": { "monthly": 1_500, "at": "ahv2" } });
    assert!(api::upgrade_plan(&none).notes.iter().any(|n| n == note));
}
