//! #272: Taxes with the place of residence as the default: defaults of new plans, the missing municipality and the
//! two estimated taxes of the card «Steuern» (first full calendar year, first full calendar year after the
//! retirement). The plans are made up.

use std::path::Path;

use engine::api;
use engine::digest::{self, StoredRun};
use engine::time::YearMonth;
use engine::validate::Severity;
use serde_json::{json, Value};

fn ym(s: &str) -> YearMonth {
    s.parse().unwrap()
}

/// New plan starting in `start` in Zurich with income and wealth, so that there is a tax.
fn plan(start: &str) -> Value {
    let mut v = api::blank_plan(ym(start));
    v["household"]["persons"][0]["name"] = json!("Anna");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    v["flows"]["income"] = json!({ "mode": "const", "value": 120_000 });
    v["flows"]["spending"] = json!({ "mode": "const", "value": 60_000 });
    for (k, x) in [("stocks", 400_000), ("bonds", 200_000), ("cash", 100_000)] {
        v["wealth"][k] = json!(x);
    }
    v
}

fn preview(v: &Value) -> Value {
    serde_json::to_value(api::preview(v, Path::new("."), None)).unwrap()
}

fn estimates(v: &Value) -> Vec<(String, i64, f64)> {
    let p = preview(v);
    assert!(p["result"].is_object(), "{}", p["issues"]);
    p["taxEstimates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["kind"].as_str().unwrap().to_string(),
                e["year"].as_i64().unwrap(),
                e["amount"].as_f64().unwrap(),
            )
        })
        .collect()
}

/// #272/AK2, AK4: new plans start in the mode «Wohnort» without a municipality, with the proposal of the canton until
/// and from the retirement and the pillar 3a deduction; the validation reports only the missing municipality.
#[test]
fn issue_272_ak4_new_plans_in_the_mode_location_without_municipality() {
    let v = api::blank_plan(ym("2026-07"));
    let t = &v["taxes"];
    assert_eq!(t["mode"], "location");
    assert_eq!(t["location"], json!({ "canton": "", "church": "none" }));
    assert_eq!(
        t["deductions"],
        json!([{ "from": "start", "kind": "canton" }, { "from": "retire", "kind": "canton" }])
    );
    assert_eq!(t["deductPillar3a"], true);
    assert_eq!(t["indexed"], true);
    let errors: Vec<_> = api::validate(&v)
        .into_iter()
        .filter(|i| i.severity == Severity::Error && i.section == "taxes")
        .map(|i| (i.message.code(), i.field))
        .collect();
    assert_eq!(
        errors,
        vec![(
            "tax_municipality_missing",
            Some("taxes.location.municipality".to_string())
        )]
    );
    // with the municipality the plan has no error in the section «Steuern»
    assert!(!api::validate(&plan("2026-07"))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "taxes"));
}

/// #272/AK3: two estimates in prices of the start, rounded to 100: first full calendar year and first full calendar
/// year after the retirement; the amount is income and wealth tax of the yearly values (real) of the tax year.
#[test]
fn issue_272_ak3_two_estimates_from_the_preview() {
    // start July 2026, person born July 1966, retirement at 65 in July 2031
    let v = plan("2026-07");
    let got = estimates(&v);
    assert_eq!(
        got.iter().map(|(k, y, _)| (k.as_str(), *y)).collect::<Vec<_>>(),
        vec![("first", 2027), ("retired", 2032)]
    );
    let p = preview(&v);
    let run: StoredRun = serde_json::from_value(p["result"].clone()).unwrap();
    let real = digest::yearly(&run).real;
    for (_, year, amount) in &got {
        let row = real.iter().find(|r| r.year as i64 == *year).unwrap();
        assert_eq!(*amount, ((row.tax_income + row.tax_wealth) / 100.0).round() * 100.0);
        assert!(*amount > 0.0 && amount % 100.0 == 0.0, "{amount}");
    }
    // earned income until the retirement: the tax before is higher than after
    assert!(got[0].2 > got[1].2, "{got:?}");
}

/// #272/AK3: a start in January makes the start year the first full year; a retirement in January the year of the
/// retirement the first full year after it.
#[test]
fn issue_272_ak3_full_years_at_january() {
    let mut v = plan("2026-01");
    v["household"]["persons"][0]["birth"] = json!("1966-01");
    let got = estimates(&v);
    assert_eq!(
        got.iter().map(|(k, y, _)| (k.as_str(), *y)).collect::<Vec<_>>(),
        vec![("first", 2026), ("retired", 2031)]
    );
}

/// #272/AK3: with the retirement before the start (or in the first full year) only one estimate.
#[test]
fn issue_272_ak3_only_one_estimate_if_retired() {
    let mut v = plan("2026-07");
    v["household"]["persons"][0]["birth"] = json!("1955-03");
    v["flows"]["income"] = json!({ "mode": "const", "value": 0 });
    let got = estimates(&v);
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!((got[0].0.as_str(), got[0].1), ("first", 2027));
    // retirement in January 2027: the first full year after it is the first full year
    let mut v = plan("2026-07");
    v["household"]["persons"][0]["birth"] = json!("1962-01");
    assert_eq!(estimates(&v).len(), 1);
}

/// #272/AK3: without a full calendar year no estimate; the function directly with an empty run.
#[test]
fn issue_272_ak3_no_full_year() {
    assert!(digest::tax_estimates(&[], Some(ym("2030-01"))).is_empty());
    let mut v = plan("2026-07");
    v["milestones"][3] = json!({ "id": "end", "kind": "date", "date": "2027-06", "fixed": true });
    assert!(estimates(&v).is_empty());
}
