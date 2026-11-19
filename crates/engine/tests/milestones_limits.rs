//! #51 (fixed milestones «AHV-Bezug» and «Bezug Pensionskasse» with limits) and #25 (limits from the
//! settings). Made-up data: birth 1980-06, start 2030-01.

use engine::api;
use engine::model::Plan;
use engine::resolve::Resolver;
use engine::time::YearMonth;
use engine::validate::{Issue, Severity};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Grenzen" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-06" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "age", "person": "A", "years": 90, "fixed": true }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "ahv" }
        },
        "flows": { "income": { "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "retire", "value": 0 } ] },
            "spending": 5000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 100_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn issues(v: &Value) -> Vec<Issue> {
    api::validate(v)
}

fn errors(v: &Value) -> Vec<String> {
    issues(v)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| i.message.to_string())
        .collect()
}

fn warnings(v: &Value) -> Vec<String> {
    issues(v)
        .into_iter()
        .filter(|i| i.severity == Severity::Warning)
        .map(|i| i.message.to_string())
        .collect()
}

fn with_age(id: usize, years: f64) -> Value {
    let mut v = base();
    v["milestones"][id]["years"] = json!(years);
    v
}

/// #51/AK2: The limits themselves are valid; values outside are rejected with a clear message.
#[test]
fn issue_51_ak2_ages_within_limits() {
    assert!(errors(&base()).is_empty(), "{:?}", errors(&base()));
    for ok in [63.0, 65.0, 70.0] {
        assert!(errors(&with_age(2, ok)).is_empty(), "AHV {ok}");
    }
    for ok in [58.0, 60.5, 65.0] {
        assert!(errors(&with_age(1, ok)).is_empty(), "PK {ok}");
    }
    let e = errors(&with_age(2, 62.0));
    assert_eq!(
        e,
        vec!["«AHV-Bezug»: Alter 62 liegt ausserhalb der Grenzen 63 bis 70 Jahre (Einstellungen).".to_string()]
    );
    assert!(errors(&with_age(2, 71.0))[0].contains("Alter 71 liegt ausserhalb der Grenzen 63 bis 70"));
    assert!(errors(&with_age(1, 57.0))[0]
        .contains("«Bezug Pensionskasse»: Alter 57 liegt ausserhalb der Grenzen 58 bis 65"));
    assert!(errors(&with_age(1, 65.5))[0].contains("Alter 65.5 liegt ausserhalb"));
    // The message is at the field of the milestone.
    let i = issues(&with_age(1, 66.0))
        .into_iter()
        .find(|i| i.severity == Severity::Error)
        .unwrap();
    assert_eq!(
        (i.section, i.field.as_deref()),
        ("milestones", Some("milestones.1.years"))
    );
    // Entered as a date: the same limits.
    let mut v = base();
    v["milestones"][1] = json!({ "id": "retire", "name": "Bezug Pensionskasse", "kind": "date", "date": "2038-05" });
    assert!(errors(&v)[0].contains("Alter 57.9 liegt ausserhalb"));
    v["milestones"][1]["date"] = json!("2038-06");
    assert!(errors(&v).is_empty());
}

/// #51/AK4: Birth 1980-06: AHV payout at 65, 63 and 70 years; pension fund payout at 58.
#[test]
fn issue_51_ak4_months_of_the_milestones() {
    let month = |id: usize, years: f64, which: &str| {
        let p: Plan = serde_json::from_value(with_age(id, years)).unwrap();
        Resolver::new(&p).unwrap().milestone(which).unwrap()
    };
    assert_eq!(month(2, 65.0, "ahv"), YearMonth::new(2045, 6));
    assert_eq!(month(2, 63.0, "ahv"), YearMonth::new(2043, 6));
    assert_eq!(month(2, 70.0, "ahv"), YearMonth::new(2050, 6));
    assert_eq!(month(1, 58.0, "retire"), YearMonth::new(2038, 6));
}

/// #51/AK1, AK6: If the fixed milestones are missing, they apply in the calculation engine at 65 years (also for person 2).
#[test]
fn issue_51_ak1_milestones_always_exist() {
    let mut v = base();
    v["milestones"] = json!([v["milestones"][0], v["milestones"][3]]);
    v["household"]["persons"] =
        json!([{ "id": "A", "name": "A", "birth": "1980-06" }, { "id": "B", "name": "B", "birth": "1982-01" }]);
    let p: Plan = serde_json::from_value(v).unwrap();
    let r = Resolver::new(&p).unwrap();
    assert_eq!(r.milestone("retire").unwrap(), YearMonth::new(2045, 6));
    assert_eq!(r.milestone("ahv").unwrap(), YearMonth::new(2045, 6));
    assert_eq!(r.milestone("ahv2").unwrap(), YearMonth::new(2047, 1));
    assert_eq!(r.milestone_name("retire2"), "Bezug Pensionskasse Person 2");
}

/// #51/AK8, AK9: Warning about earned income after the pension fund payout, with the amount.
#[test]
fn issue_51_ak8_ak9_income_after_the_pension_fund() {
    // Pension fund payout 2040-06 (60 years), income 12'500 from 2035-02 without a later row with 0.
    let income = |rows: Value| {
        let mut v = with_age(1, 60.0);
        v["flows"]["income"] = json!({ "mode": "timed", "rows": rows });
        warnings(&v)
    };
    const TEXT: &str = "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (06.2040) noch";
    let w = income(json!([{ "from": "start", "value": 8000 }, { "from": "2035-02", "value": 12_500 }]));
    assert!(w.contains(&format!("{TEXT} 12'500 pro Monat.")), "{w:?}");
    let w = income(json!([{ "from": "2035-02", "value": 12_500 }, { "from": "retire", "value": 0 }]));
    assert!(!w.iter().any(|m| m.contains(TEXT)), "{w:?}");
    let w = income(json!([{ "from": "2035-02", "value": 12_500 }, { "from": "2040-06", "value": 3000 }]));
    assert!(w.contains(&format!("{TEXT} 3'000 pro Monat.")), "{w:?}");
    // Only a warning, no error
    let mut v = with_age(1, 60.0);
    v["flows"]["income"] = json!(12_500);
    assert!(errors(&v).is_empty());
    // Two persons: the warning names the person
    v["household"]["persons"] =
        json!([{ "id": "A", "name": "Anna", "birth": "1980-06" }, { "id": "B", "name": "Ben", "birth": "1982-01" }]);
    v["partner"] = json!({ "income": 2000, "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 },
        "ahv": { "monthly": 0, "at": "ahv2" } });
    let w = warnings(&v);
    assert!(
        w.iter().any(|m| m
            .starts_with("Ben: Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (01.2047) noch 2'000")),
        "{w:?}"
    );
}

/// #51: The warning does not go month by month until the end; an absurdly distant end does not hang the validation.
#[test]
fn issue_51_income_warning_with_distant_end() {
    let mut v = with_age(1, 60.0);
    v["milestones"][3]["years"] = json!(100_000_000);
    v["flows"]["income"] = json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "retire", "value": 0 },
        { "from": "2100-01", "value": 700 } ] });
    // #83: The validation looks for the amount via the grid points and takes milliseconds. If it went month by month
    // until the end (1.2 billion months), it would need many minutes on any machine. The limit of 60 s only catches
    // this hang and also holds on a 2-core runner under coverage measurement.
    let t = std::time::Instant::now();
    let w = warnings(&v);
    assert!(t.elapsed().as_secs() < 60, "Prüfung dauert zu lange");
    assert!(w.iter().any(|m| m.ends_with("noch 700 pro Monat.")), "{w:?}");
}

/// #25/AK5, AK6: The limits come as parameters from the settings; if not given, the default values apply.
#[test]
fn issue_25_ak6_limits_from_the_settings() {
    let mut v = with_age(1, 67.0);
    assert!(errors(&v)[0].contains("58 bis 65"));
    v["limits"] = json!({ "pkMax": 67 });
    assert!(errors(&v).is_empty(), "67 gültig mit oberer Grenze 67");
    let mut v68 = with_age(1, 68.0);
    v68["limits"] = json!({ "pkMax": 67 });
    assert_eq!(
        errors(&v68),
        vec![
            "«Bezug Pensionskasse»: Alter 68 liegt ausserhalb der Grenzen 58 bis 67 Jahre (Einstellungen).".to_string()
        ]
    );
    // Invalid settings are reported
    let mut bad = base();
    bad["limits"] = json!({ "ahvMin": 71 });
    assert!(errors(&bad)
        .iter()
        .any(|m| m.contains("Einstellungen: AHV-Bezug: Die untere Grenze (71) liegt über der oberen Grenze (70).")));
    // The limits do not belong to the plan: they are missing when writing.
    let p: Plan = serde_json::from_value(v.clone()).unwrap();
    assert!(serde_json::to_value(&p).unwrap().get("limits").is_none());
}

/// #170/AK8: the fixed milestones store no name; the engine uses the German default name by the id, so that results,
/// messages and event texts stay the same as with the names stored (golden files unchanged until #176).
#[test]
fn issue_170_ak8_fixed_milestones_without_name_give_the_same_results_and_texts() {
    let mut named = api::example_plan();
    named["returns"]["source"] = json!("fixed");
    named["scenarios"]["sweeps"] = json!([]);
    // the shipped example stores no names for the fixed milestones
    for ms in named["milestones"].as_array().unwrap() {
        let id = ms["id"].as_str().unwrap();
        assert_eq!(
            ms.get("name").is_none(),
            engine::model::fixed_milestone_name(id).is_some(),
            "{id}"
        );
    }
    let unnamed = named.clone();
    for ms in named["milestones"].as_array_mut().unwrap() {
        if let Some(n) = engine::model::fixed_milestone_name(ms["id"].as_str().unwrap()) {
            ms["name"] = json!(n);
        }
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let library = dir.join("data");
    let run = |p: &Value| serde_json::to_value(api::run(p, dir, Some(&library))).unwrap();
    let (a, b) = (run(&unnamed), run(&named));
    assert!(a["single"]["records"].as_array().is_some_and(|r| !r.is_empty()));
    assert_eq!(a["single"], b["single"]);
    assert_eq!(a["issues"], b["issues"]);
    // messages with a milestone name: payout of the pension fund before the start
    let early = |mut v: Value| {
        v["milestones"][1] =
            json!({ "id": "retire", "kind": "date", "date": "1990-01", "name": v["milestones"][1]["name"] });
        if v["milestones"][1]["name"].is_null() {
            v["milestones"][1].as_object_mut().unwrap().remove("name");
        }
        v
    };
    let texts = |v: &Value| {
        api::validate(v)
            .into_iter()
            .map(|i| i.message.to_string())
            .collect::<Vec<_>>()
    };
    let (a, b) = (texts(&early(unnamed.clone())), texts(&early(named.clone())));
    assert_eq!(a, b);
    assert!(a.iter().any(|t| t.contains("«Bezug Pensionskasse»")), "{a:?}");
    // German default names of all fixed milestones
    let names: Vec<_> = ["start", "end", "retire", "ahv", "retire2", "ahv2", "sale"]
        .iter()
        .map(|id| engine::model::fixed_milestone_name(id))
        .collect();
    assert_eq!(
        names,
        [
            Some("Simulationsstart".to_string()),
            Some("Simulationsende".to_string()),
            Some("Bezug Pensionskasse".to_string()),
            Some("AHV-Bezug".to_string()),
            Some("Bezug Pensionskasse Person 2".to_string()),
            Some("AHV-Bezug Person 2".to_string()),
            None
        ]
    );
}
