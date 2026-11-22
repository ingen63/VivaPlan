//! #65/AK1, AK5, AK6: In the mode «Wohnort» each property pays the property gain tax of its canton; own rules stay
//! possible with `taxes.propertyGainOwn`; the validation then does not require own rules.

use engine::model::{Plan, PropertyGainTax, TaxLocation};
use engine::sim::Simulator;
use engine::tax::property_gain_tax_full;
use engine::taxdata;
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

fn plan(property_canton: Option<&str>) -> Value {
    let mut house = json!({ "id": "h", "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000,
        "bought": "2030-01", "sell": "2041-06", "fixCosts": 0, "inflation": false });
    if let Some(c) = property_canton {
        house["canton"] = json!(c);
    }
    json!({
        "project": { "name": "Gewinnsteuer" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1970-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2042-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 5000, "spending": 4000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "realestate": { "properties": [house] },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [[0, 0.0]], "capital": [[0, 0.0]],
                   "mode": "location", "location": { "canton": "ZH", "municipality": 261, "church": "none" } }
    })
}

fn sale_tax(v: &Value) -> f64 {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    let s = Simulator::new(&plan).unwrap();
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    let r = s.run(&path, true);
    r.records
        .iter()
        .find(|x| x.month == YearMonth::new(2041, 6))
        .unwrap()
        .tax_gain
}

fn loc() -> TaxLocation {
    TaxLocation {
        canton: "ZH".into(),
        municipality: Some(261),
        ..Default::default()
    }
}

fn expected(canton: &str) -> f64 {
    let (rule, m) = taxdata::builtin().property_gain(&loc(), Some(canton)).unwrap();
    // bought 2030-01, sold 2041-06: 137 months; gain 300 000 on 500 000
    property_gain_tax_full(Some(&rule), 300_000.0, 500_000.0, Some(137), m)
}

fn issues(v: &Value, severity: Severity) -> Vec<(&'static str, Option<String>)> {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    validate(&plan)
        .into_iter()
        .filter(|i| i.severity == severity)
        .map(|i| (i.message.code(), i.field))
        .collect()
}

/// #65/AK1: Without an entry the canton of residence applies, with an entry the canton of the property (Fribourg:
/// rate 12 % from 10 years, Art. 51 DStG: 300 000 × 12 % = 36 000).
#[test]
fn issue_65_ak1_property_uses_its_canton_rules() {
    let zh = sale_tax(&plan(None));
    assert!((zh - expected("ZH")).abs() < 0.01 && zh > 0.0, "{zh}");
    assert!((sale_tax(&plan(Some("ZH"))) - zh).abs() < 0.01);
    let fr = sale_tax(&plan(Some("FR")));
    assert!((fr - 36_000.0).abs() < 0.01, "{fr}");
    assert!((fr - expected("FR")).abs() < 0.01);
    assert!(
        (sale_tax(&plan(Some(""))) - zh).abs() < 0.01,
        "an empty canton counts as the canton of residence"
    );
}

/// #65/AK1: Cantons with a tax multiplier: in the canton of residence that of the municipality of residence (canton +
/// municipality), otherwise the median of the canton.
#[test]
fn issue_65_ak1_multiplier_of_municipality_or_median() {
    let data = taxdata::builtin();
    let bern = data.municipality(351).unwrap();
    let home = TaxLocation {
        canton: "BE".into(),
        municipality: Some(351),
        ..Default::default()
    };
    let (rule, m) = data.property_gain(&home, None).unwrap();
    assert!(rule.use_multiplier);
    assert!((m - (bern.3[0] + bern.3[1])).abs() < 1e-12, "{m}");
    // property in Bern, residence in Zurich: median of the canton
    let (_, m) = data.property_gain(&loc(), Some("BE")).unwrap();
    assert_eq!(m, data.canton("BE").unwrap().gain_multiplier);
    // the median lies between the lowest and the highest multiplier of the canton
    let rates: Vec<f64> = data
        .municipalities
        .iter()
        .filter(|x| x.2 == "BE")
        .map(|x| x.3[0] + x.3[1])
        .collect();
    let below = rates.iter().filter(|&&r| r < m).count();
    let above = rates.iter().filter(|&&r| r > m).count();
    assert!(below <= rates.len() / 2 && above <= rates.len() / 2, "{m}");
    // simulated sale in Bern with the residence in Zurich: tax with the median
    let v = plan(Some("BE"));
    assert!((sale_tax(&v) - expected("BE")).abs() < 0.01);
    // unknown canton
    let e = data.property_gain(&loc(), Some("XX")).unwrap_err();
    assert_eq!(e.message().code(), "gain_tax_canton_unknown");
}

/// #65/AK5: Own rules replace the rule of the canton only with `propertyGainOwn`; without the check box entered rules
/// stay unused. Plans in the mode «Eigene Tabellen» compute as before.
#[test]
fn issue_65_ak5_own_rules_need_the_flag() {
    let own = PropertyGainTax {
        brackets: vec![(1e12, 0.2)],
        ..Default::default()
    };
    let mut v = plan(Some("FR"));
    v["taxes"]["propertyGain"] = serde_json::to_value(&own).unwrap();
    assert!((sale_tax(&v) - 36_000.0).abs() < 0.01, "rule of the canton applies");
    v["taxes"]["propertyGainOwn"] = json!(true);
    assert!((sale_tax(&v) - 60_000.0).abs() < 0.01, "own rule: 300 000 × 20 %");
    let mut v = plan(Some("FR"));
    v["taxes"].as_object_mut().unwrap().remove("mode");
    v["taxes"]["propertyGain"] = serde_json::to_value(&own).unwrap();
    assert!((sale_tax(&v) - 60_000.0).abs() < 0.01, "mode «Eigene Tabellen»");
    // not written if not set
    let out = serde_json::to_value(serde_json::from_value::<Plan>(plan(None)).unwrap()).unwrap();
    assert!(out["taxes"].get("propertyGainOwn").is_none());
    assert!(out["realestate"]["properties"][0].get("canton").is_none());
    let out = serde_json::to_value(serde_json::from_value::<Plan>(plan(Some("FR"))).unwrap()).unwrap();
    assert_eq!(out["realestate"]["properties"][0]["canton"], "FR");
}

/// #65/AK5: Own rules count the holding period in full years as before (a rule from half a year applies only after a
/// full year); the rules of the canton count months.
#[test]
fn issue_65_ak5_own_rules_keep_full_years() {
    let own = PropertyGainTax {
        brackets: vec![(1e12, 0.1)],
        duration: vec![(0.0, 0.5), (11.5, 0.0)],
        ..Default::default()
    };
    // held 137 months = 11 full years: surcharge 50 % still applies with own rules
    let mut v = plan(None);
    v["taxes"]["propertyGain"] = serde_json::to_value(&own).unwrap();
    v["taxes"]["propertyGainOwn"] = json!(true);
    assert!((sale_tax(&v) - 45_000.0).abs() < 0.01, "{}", sale_tax(&v));
}

/// #65/AK6: In the mode «Wohnort» the validation requires no own rules, but it does with `propertyGainOwn`; an unknown
/// canton of the property is an error at the field; a missing purchase date is a warning.
#[test]
fn issue_65_ak6_validation_in_location_mode() {
    assert!(issues(&plan(None), Severity::Error).is_empty());
    assert!(issues(&plan(Some("VS")), Severity::Error).is_empty());
    let mut v = plan(None);
    v["taxes"]["propertyGainOwn"] = json!(true);
    assert_eq!(issues(&v, Severity::Error), vec![("gain_tax_missing", None)]);
    assert_eq!(
        issues(&plan(Some("XX")), Severity::Error),
        vec![("property_canton_unknown", Some("realestate.properties.0.canton".into()))]
    );
    // a rented property does not count, whatever its canton
    let mut v = plan(Some("XX"));
    v["realestate"]["properties"][0]["status"] = json!("rent");
    v["realestate"]["properties"][0]["from"] = json!("start");
    v["realestate"]["properties"][0]["rent"] = json!(1000);
    assert!(!issues(&v, Severity::Error)
        .iter()
        .any(|(c, _)| *c == "property_canton_unknown"));
    // unused own rules and the unused (hidden) tax multiplier are not checked
    let mut v = plan(None);
    v["taxes"]["propertyGain"] = json!({ "model": "aargau", "durationRates": [] });
    v["taxes"]["multiplier"] = json!(20.0);
    assert!(issues(&v, Severity::Error).is_empty());
    v["taxes"]["propertyGainOwn"] = json!(true);
    let codes: Vec<_> = issues(&v, Severity::Error).into_iter().map(|(c, _)| c).collect();
    assert!(
        codes.contains(&"tax_multiplier_range") && codes.contains(&"gain_tax_aargau_empty"),
        "{codes:?}"
    );
    // missing purchase date: warning
    let mut v = plan(None);
    v["realestate"]["properties"][0]
        .as_object_mut()
        .unwrap()
        .remove("bought");
    assert!(issues(&v, Severity::Warning)
        .iter()
        .any(|(c, _)| *c == "property_bought_missing_canton"));
    assert!(issues(&plan(None), Severity::Warning)
        .iter()
        .all(|(c, _)| *c != "property_bought_missing_canton"));
    // in the mode «Eigene Tabellen» the duty stays
    let mut v = plan(None);
    v["taxes"].as_object_mut().unwrap().remove("mode");
    assert!(issues(&v, Severity::Error)
        .iter()
        .any(|(c, _)| *c == "gain_tax_missing"));
}
