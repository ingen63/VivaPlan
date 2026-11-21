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

/// #191/AK9: An older plan with an AHV payout at 63 held the pension already reduced (2'177.28). When opening, the
/// amount is converted back to the pension at 65 (2'177.28 / (1 − 0.136) = 2'520.00) with a note once; the simulation
/// still computes with 2'177.28. The same for a deferral to 68 (2'950.92 / 1.171 = 2'520.00) and for the partner.
#[test]
fn issue_191_ak9_older_plan_converts_the_adjusted_ahv_pension_back_to_65() {
    use engine::model::Plan;
    let fixture: serde_json::Value = serde_json::from_str(include_str!("fixtures/planung-altformat.rplan")).unwrap();
    let with_payout = |years: f64, monthly: f64| {
        let mut old = fixture.clone();
        for ms in old["milestones"].as_array_mut().unwrap() {
            if ms["id"] == "ahv" {
                ms["years"] = json!(years);
            }
        }
        old["pension"]["ahv"]["monthly"] = json!(monthly);
        old
    };
    let simulated = |plan: &serde_json::Value| {
        let p: Plan = serde_json::from_value(plan.clone()).unwrap();
        engine::sim::ahv_view(&p)
    };
    let code = |n: &engine::message::Message| n.code().to_string();

    // Payout at 63
    let old = with_payout(63.0, 2_177.28);
    assert!(old["pension"]["ahv"].get("referenceAge").is_none());
    let first = api::upgrade_plan(&old);
    assert_eq!(first.plan["pension"]["ahv"]["monthly"], json!(2_520.0));
    assert_eq!(first.plan["pension"]["ahv"]["referenceAge"], true);
    let notes: Vec<_> = first.notes.iter().filter(|n| code(n) == "note_ahv_early").collect();
    assert_eq!(notes.len(), 1, "{:?}", first.notes);
    let text = notes[0].to_string();
    assert!(
        text.contains("13.6 %") && text.contains("2177.28") && text.contains("2520.00"),
        "{text}"
    );
    let view = simulated(&first.plan);
    assert_eq!(view[0].monthly, 2_177.28);
    assert_eq!(view[0].entered, 2_520.0);
    // once: the second opening changes nothing
    let second = api::upgrade_plan(&first.plan);
    assert_eq!(second.plan["pension"]["ahv"]["monthly"], json!(2_520.0));
    assert!(second.notes.iter().all(|n| !code(n).starts_with("note_ahv_early")));

    // Deferral to 68
    let first = api::upgrade_plan(&with_payout(68.0, 2_950.92));
    assert_eq!(first.plan["pension"]["ahv"]["monthly"], json!(2_520.0));
    assert!(first.notes.iter().any(|n| code(n) == "note_ahv_deferred"));
    assert_eq!(simulated(&first.plan)[0].monthly, 2_950.92);

    // Payout at 65 or no pension: unchanged, no note
    for old in [with_payout(65.0, 2_177.28), with_payout(63.0, 0.0)] {
        let out = api::upgrade_plan(&old);
        assert_eq!(out.plan["pension"]["ahv"]["monthly"], old["pension"]["ahv"]["monthly"]);
        assert!(out.notes.iter().all(|n| !code(n).starts_with("note_ahv_early")));
        assert_eq!(out.plan["pension"]["ahv"]["referenceAge"], true);
    }

    // Partner with a payout 15 months early (8.5 %): 1'830.00 → 2'000.00, note with the person
    let mut old = with_payout(65.0, 2_000.0);
    old["household"]["persons"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "B", "name": "B", "birth": "1972-01" }));
    old["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "ahv2", "kind": "age", "person": "B", "years": 63.75 }));
    old["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
        "ahv": { "monthly": 1_830.0, "at": "ahv2" } });
    let out = api::upgrade_plan(&old);
    assert_eq!(out.plan["partner"]["ahv"]["monthly"], json!(2_000.0));
    assert_eq!(out.plan["pension"]["ahv"]["monthly"], json!(2_000.0));
    let early: Vec<_> = out.notes.iter().filter(|n| code(n) == "note_ahv_early").collect();
    assert_eq!(early.len(), 1);
    assert!(early[0].to_string().contains("(person 2)"), "{}", early[0]);
    let view = simulated(&out.plan);
    assert_eq!((view[0].monthly, view[1].monthly), (2_000.0, 1_830.0));

    // Current plans (new plan, example) get no note and keep their pension
    for current in [
        include_str!("../assets/new.vplan"),
        include_str!("../assets/example.vplan"),
    ] {
        let v: serde_json::Value = serde_json::from_str(current).unwrap();
        let out = api::upgrade_plan(&v);
        assert!(
            out.notes.iter().all(|n| !code(n).starts_with("note_ahv_")),
            "{:?}",
            out.notes
        );
        assert_eq!(out.plan["pension"]["ahv"]["monthly"], v["pension"]["ahv"]["monthly"]);
    }
}
