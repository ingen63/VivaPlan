//! FEA-13: Jede Zeile der Liste der Pflichteingaben (`docs/Pflichteingaben.md`) hat einen Test, der den
//! Fehler auslöst und die Meldung prüft.

use engine::api;
use engine::validate::{Issue, Severity};
use serde_json::{json, Value};

/// Gültige Minimalplanung: 2 Jahre ab Januar 2030, ein Haushalt mit einer Person.
fn base() -> Value {
    crate::any_age(json!({
        "project": { "name": "Test" },
        "household": { "persons": [ { "id": "A", "name": "Anna", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2040-01" },
            { "id": "ahv", "name": "AHV", "kind": "date", "date": "2040-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2031-12" }
        ],
        "pension": {
            "pk": { "capital": 100_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 },
            "ahv": { "monthly": 2_000, "at": "ahv" }
        },
        "flows": { "income": 5_000, "spending": 4_000 },
        "wealth": { "stocks": 1_000, "bonds": 1_000, "cash": 1_000, "stockShare": 0.5, "reserveMonths": 3 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.04, "bonds": 0.01, "inflation": 0.01 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    }))
}

fn issues(v: &Value) -> Vec<Issue> {
    api::validate(v)
}

/// Die Planung hat einen Fehler im Abschnitt `section`, dessen Meldung `text` enthält (und am Feld `field`, falls angegeben).
fn assert_error(v: &Value, section: &str, text: &str, field: Option<&str>) {
    let all = issues(v);
    let hit = all.iter().find(|i| {
        i.severity == Severity::Error
            && i.section == section
            && i.message.contains(text)
            && (field.is_none() || i.field.as_deref() == field)
    });
    assert!(hit.is_some(), "kein Fehler «{text}» im Abschnitt {section}: {all:?}");
}

#[test]
fn valid_base_plan_has_no_errors() {
    assert!(
        !issues(&base()).iter().any(|i| i.severity == Severity::Error),
        "{:?}",
        issues(&base())
    );
}

#[test]
fn household_needs_one_or_two_persons() {
    let mut v = base();
    v["household"]["persons"] = json!([]);
    assert_error(&v, "general", "Mindestens eine Person", None);
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01" }, { "id": "B", "name": "B", "birth": "1965-01" }, { "id": "C", "name": "C", "birth": "1965-01" }]);
    assert_error(&v, "general", "höchstens zwei Personen", None);
}

#[test]
fn person_needs_a_valid_birth_date() {
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("2031-01");
    assert_error(&v, "general", "Geburtsdatum", Some("household.persons.0.birth"));
    let mut v = base();
    v["household"]["persons"][0]["birth"] = json!("1900-01");
    assert_error(&v, "general", "über 120 Jahre", Some("household.persons.0.birth"));
    // Fehlt das Datum, lässt sich die Planung nicht laden
    let mut v = base();
    v["household"]["persons"][0].as_object_mut().unwrap().remove("birth");
    assert_error(&v, "general", "unvollständig", None);
}

#[test]
fn person_needs_a_name_of_at_most_sixty_characters() {
    for name in ["", "   "] {
        let mut v = base();
        v["household"]["persons"][0]["name"] = json!(name);
        assert_error(&v, "general", "Der Name fehlt", Some("household.persons.0.name"));
    }
    let mut v = base();
    v["household"]["persons"][0]["name"] = json!("x".repeat(61));
    assert_error(&v, "general", "höchstens 60 Zeichen", Some("household.persons.0.name"));
    let mut ok = base();
    ok["household"]["persons"][0]["name"] = json!("x".repeat(60));
    assert!(!issues(&ok)
        .iter()
        .any(|i| i.field.as_deref() == Some("household.persons.0.name")));
    // Die zweite Person wird ebenso geprüft
    let mut v = base();
    v["household"]["persons"] =
        json!([{ "id": "A", "name": "A", "birth": "1965-01" }, { "id": "B", "name": "", "birth": "1965-01" }]);
    v["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0.05 }, "ahv": { "monthly": 0, "at": "ahv" } });
    assert_error(
        &v,
        "general",
        "Person 2: Der Name fehlt",
        Some("household.persons.1.name"),
    );
}

#[test]
fn ahv_cap_must_be_positive() {
    let mut v = base();
    v["household"]["ahvCoupleCap"] = json!(0);
    assert_error(&v, "pension", "Plafonierung der AHV-Renten", None);
}

#[test]
fn milestones_start_and_end_are_required_and_ordered() {
    let mut v = base();
    v["milestones"] = json!([{ "id": "retire", "name": "P", "kind": "date", "date": "2040-01" }]);
    assert_error(&v, "milestones", "start", None);
    let mut v = base();
    v["milestones"][3]["date"] = json!("2029-12");
    assert_error(&v, "milestones", "Simulationsende muss nach dem Start", None);
    let mut v = base();
    v["milestones"][3]["date"] = json!("2131-01");
    assert_error(&v, "milestones", "länger als 100 Jahre", None);
}

#[test]
fn age_milestones_must_be_between_zero_and_130() {
    let mut v = base();
    v["milestones"][1] = json!({ "id": "retire", "name": "Pensionierung", "kind": "age", "person": "A", "years": 131 });
    assert_error(&v, "milestones", "Alter 131 ist nicht möglich", None);
}

#[test]
fn pension_fund_lump_sum_is_between_zero_and_one_hundred_percent() {
    let mut v = base();
    v["pension"]["pk"]["lumpsum"] = json!(1.5);
    assert_error(&v, "pension", "Kapitalbezug muss zwischen 0 % und 100 %", None);
}

#[test]
fn required_sections_must_be_present() {
    // AHV-Rente, Vermögen und Steuern «müssen vorhanden sein»: sonst lässt sich die Planung nicht laden.
    for (path, key) in [("pension", "ahv"), ("wealth", "stocks"), ("", "taxes")] {
        let mut v = base();
        if path.is_empty() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[path].as_object_mut().unwrap().remove(key);
        }
        assert_error(&v, "general", "unvollständig", None);
    }
}

#[test]
fn income_and_spending_series_need_valid_entries() {
    let mut v = base();
    v["flows"]["income"] = json!({ "mode": "timed", "rows": [] });
    assert_error(&v, "flows", "Einkommen: keine Einträge", None);
    let mut v = base();
    v["flows"]["spending"] = json!({ "mode": "timed", "rows": [ { "from": "gibtsnicht", "value": 100 } ] });
    assert_error(&v, "flows", "Ausgaben", None);
    let mut v = base();
    v["wealth"]["stockShare"] = json!(2.0);
    assert_error(&v, "wealth", "Aktienanteil: Werte müssen zwischen", None);
}

#[test]
fn return_source_needs_its_inputs() {
    let mut v = base();
    v["returns"]["source"] = json!("historical");
    assert_error(&v, "returns", "Zeitreihe gewählt sein", None);
    let mut v = base();
    v["returns"]["source"] = json!("historical");
    v["returns"]["series"] = json!("library:x.csv");
    assert_error(&v, "returns", "Startmonat fehlt", None);
    let mut v = base();
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["series"] = json!("library:x.csv");
    v["returns"]["bootstrap"] = json!({ "paths": 0, "block": 12, "seed": 1 });
    assert_error(&v, "returns", "Anzahl Pfade", None);
    v["returns"]["bootstrap"] = json!({ "paths": 10, "block": 0, "seed": 1 });
    assert_error(&v, "returns", "Blocklänge", None);
    let mut v = base();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 10, "mu": 0.07, "sigma": 0.15, "df": 1, "jumpLambda": 0.1, "jumpMu": 0, "jumpSigma": 0, "seed": 1 });
    assert_error(&v, "returns", "Freiheitsgrade", None);
    v["returns"]["parametric"]["df"] = json!(6);
    v["returns"]["parametric"]["sigma"] = json!(-1);
    assert_error(&v, "returns", "Volatilität", None);
}

#[test]
fn withdrawal_strategies_need_valid_parameters() {
    let cases = [
        (
            json!({ "strategy": "three_buckets", "buckets": { "cashYears": -1, "bondYears": 2 } }),
            "Töpfe dürfen nicht negativ",
        ),
        (
            json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 1.5 } }),
            "Entnahmerate muss zwischen",
        ),
        (
            json!({ "strategy": "vpw", "vpw": { "horizonAge": 0, "realReturn": 0.02 } }),
            "VPW",
        ),
        (
            json!({ "strategy": "endowment", "endowment": { "rate": 1.5, "smoothing": 0.3 } }),
            "Endowment",
        ),
        (
            json!({ "strategy": "guardrails", "guardrails": { "initialRate": 1.5, "upper": 0.2, "lower": 0.2, "adjust": 0.1 } }),
            "Startrate",
        ),
        (
            json!({ "strategy": "guardrails", "guardrails": { "initialRate": 0.05, "upper": -0.2, "lower": 0.2, "adjust": 0.1 } }),
            "Leitplanken",
        ),
        (
            json!({ "strategy": "guardrails", "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 1.5 } }),
            "Anpassung",
        ),
        (
            json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 }, "floor": 0.9, "ceiling": 0.5 }),
            "Untergrenze der Ausgaben liegt über der Obergrenze",
        ),
        (
            json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 }, "floor": -0.1 }),
            "dürfen nicht negativ sein",
        ),
    ];
    for (w, text) in cases {
        let mut v = base();
        v["withdrawal"] = w;
        assert_error(&v, "withdrawal", text, None);
    }
}

fn house(extra: Value) -> Value {
    let mut p = json!({ "id": "haus", "name": "Haus", "status": "owned", "price": 500_000, "worth": 600_000, "sell": "never",
        "mortgage": { "tranches": [ { "name": "T1", "amount": 300_000, "periods": [ { "term": 10, "rate": 0.01 } ] } ] } });
    for (k, val) in extra.as_object().unwrap() {
        p[k] = val.clone();
    }
    p
}

#[test]
fn properties_need_their_inputs() {
    let with = |p: Value| {
        let mut v = base();
        v["realestate"] = json!({ "properties": [p] });
        v
    };
    // Hypothekentranche ohne Periode
    assert_error(
        &with(house(
            json!({ "mortgage": { "tranches": [ { "name": "T1", "amount": 300_000, "periods": [] } ] } }),
        )),
        "realestate",
        "keine Hypothekarperiode",
        None,
    );
    // Hypothek höher als der Kaufpreis (geplante Immobilie)
    let planned = json!({ "id": "w", "name": "Wohnung", "status": "planned", "buy": "2030-06", "price": 100_000,
        "mortgage": { "tranches": [ { "name": "T1", "amount": 300_000, "periods": [ { "term": 10, "rate": 0.01 } ] } ] } });
    assert_error(
        &with(planned),
        "realestate",
        "Hypothek ist höher als der Kaufpreis",
        None,
    );
    // Kauf nach Verkauf einer Immobilie, die es nicht gibt
    let after =
        json!({ "id": "w", "name": "Wohnung", "status": "planned", "buy": "afterSale:gibtsnicht", "price": 500_000 });
    assert_error(&with(after), "realestate", "gibt es nicht mehr", None);
    // Kennung doppelt
    let mut v = base();
    v["realestate"] = json!({ "properties": [house(json!({})), house(json!({ "name": "Haus 2" }))] });
    assert_error(&v, "realestate", "doppelt vergeben", None);
    // Miete ohne Beginn und ohne Betrag
    let rent = json!({ "id": "m", "name": "Miete", "status": "rent" });
    assert_error(&with(rent.clone()), "realestate", "«Miete ab» fehlt", None);
    assert_error(&with(rent), "realestate", "Miete fehlt", None);
}

#[test]
fn pillar_3a_is_checked() {
    // FEA-37: Die Säule 3a ist ein eigenes Konto (Abschnitt «Säule 3a»).
    let mut v = base();
    v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 2.0 }]);
    assert_error(&v, "pillar3a", "Der Zins muss zwischen", Some("pillar3a.0.rate"));
}

#[test]
fn property_gain_tax_rules_must_be_valid() {
    let mut v = base();
    v["taxes"]["propertyGain"] = json!({ "model": "aargau", "durationRates": [], "brackets": [], "reduction": 0 });
    assert_error(&v, "taxes", "kein Steuersatz nach Besitzdauer", None);
    let mut v = base();
    v["taxes"]["propertyGain"] = json!({ "model": "zurich", "brackets": [], "reduction": 0, "duration": [[-1, 0.1]] });
    assert_error(&v, "taxes", "Besitzdauer", None);
}

#[test]
fn variations_need_a_valid_parameter() {
    let mut v = base();
    v["scenarios"] = json!({ "sweeps": [ { "param": "gibtsnicht", "from": 0, "to": 1, "step": 0.1 } ] });
    assert_error(&v, "scenarios", "Variation", None);
}

fn with_property(status: &str) -> Value {
    let mut v = base();
    v["realestate"] = json!({ "emergencyMonths": 3, "properties": [
        { "id": "p", "name": "Objekt", "status": status, "price": 500_000, "worth": 600_000, "buy": "2030-06", "fixCosts": 0, "rent": 1_000 } ] });
    v
}

#[test]
fn property_gain_tax_is_mandatory_for_owned_and_planned_properties() {
    for status in ["owned", "planned"] {
        let v = with_property(status);
        assert_error(&v, "taxes", "Grundstückgewinnsteuer fehlt", None);
        // Auch die Simulation weist die Planung ab (Start blockiert)
        let out = api::run(&v, std::path::Path::new("."), None);
        assert!(out.error.is_some(), "{status}: Simulation müsste abgewiesen werden");
    }
}

#[test]
fn rented_properties_and_no_property_do_not_need_property_gain_tax() {
    assert!(!issues(&with_property("rent"))
        .iter()
        .any(|i| i.message.contains("Grundstückgewinnsteuer fehlt")));
    assert!(!issues(&base())
        .iter()
        .any(|i| i.message.contains("Grundstückgewinnsteuer fehlt")));
}

#[test]
fn any_property_gain_rules_satisfy_the_requirement_even_with_zero_rate() {
    // Eigene Regeln mit Satz 0 %
    let mut v = with_property("owned");
    v["taxes"]["propertyGain"] = json!({ "brackets": [], "reduction": 0.0 });
    assert!(!issues(&v)
        .iter()
        .any(|i| i.message.contains("Grundstückgewinnsteuer fehlt")));
    // Vorlage Aargau
    let mut v = with_property("planned");
    v["taxes"]["propertyGain"] = json!({ "model": "aargau", "durationRates": [[0, 0.4], [5, 0.2]] });
    assert!(!issues(&v)
        .iter()
        .any(|i| i.message.contains("Grundstückgewinnsteuer fehlt")));
}

fn has_sell_error(v: &Value, field: &str) -> bool {
    issues(v)
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "realestate" && i.field.as_deref() == Some(field))
}

fn with_gain(mut v: Value) -> Value {
    v["taxes"]["propertyGain"] = json!({ "brackets": [], "reduction": 0.0 });
    v
}

#[test]
fn sale_must_come_after_the_purchase() {
    let plan = |p: Value| {
        let mut v = base();
        v["realestate"] = json!({ "properties": [p] });
        with_gain(v)
    };
    let planned = |buy: &str, sell: Value| json!({ "id": "w", "name": "Wohnung", "status": "planned", "buy": buy, "sell": sell, "price": 500_000 });
    let f = "realestate.properties.0.sell";
    // Kauf geplant: Verkauf davor, im selben Monat und danach
    assert!(has_sell_error(&plan(planned("2030-06", json!("2030-03"))), f));
    assert!(has_sell_error(&plan(planned("2030-06", json!("2030-06"))), f));
    assert_error(
        &plan(planned("2030-06", json!("2030-06"))),
        "realestate",
        "nach «Kauf ab»",
        Some(f),
    );
    assert!(!has_sell_error(&plan(planned("2030-06", json!("2030-07"))), f));
    // Vorgabe «nie» und fehlender Verkauf: kein Fehler
    assert!(!has_sell_error(&plan(planned("2030-06", json!("never"))), f));
    let mut no_sell = planned("2030-06", json!(null));
    no_sell.as_object_mut().unwrap().remove("sell");
    assert!(!has_sell_error(&plan(no_sell), f));
    // Meilenstein als Verkaufszeitpunkt (Pensionierung 2040-01 liegt nach dem Kauf; «start» 2030-01 davor)
    assert!(!has_sell_error(&plan(planned("2030-06", json!("retire"))), f));
    assert!(has_sell_error(&plan(planned("2030-06", json!("start"))), f));
    // Kauf «nach Verkauf von …»: keine Prüfung des Datums
    let mut after = planned("afterSale:haus", json!("2030-03"));
    after["id"] = json!("w");
    let mut v = base();
    v["realestate"] = json!({ "properties": [house(json!({ "sell": "2030-02" })), after] });
    assert!(!has_sell_error(&with_gain(v), "realestate.properties.1.sell"));
    // Besitz: Verkauf gegenüber dem Kaufdatum
    let owned = |bought: &str, sell: &str| house(json!({ "bought": bought, "sell": sell }));
    assert!(has_sell_error(&plan(owned("2020-05", "2020-05")), f));
    assert_error(
        &plan(owned("2020-05", "2019-01")),
        "realestate",
        "nach dem Kaufdatum",
        Some(f),
    );
    assert!(!has_sell_error(&plan(owned("2020-05", "2031-01")), f));
    assert!(!has_sell_error(&plan(owned("2020-05", "never")), f));
    // Miete verkauft nie
    let rent =
        json!({ "id": "m", "name": "Miete", "status": "rent", "from": "2030-06", "rent": 1000, "sell": "2030-01" });
    assert!(!has_sell_error(&plan(rent), f));
}

#[test]
fn planned_property_is_sold_at_the_chosen_date() {
    // Die Engine verkauft ein geplantes Objekt zum gewählten Zeitpunkt (Ereignis «Verkauf»).
    let mut v = base();
    v["milestones"][3]["date"] = json!("2033-12");
    v["wealth"]["cash"] = json!(1_000_000);
    v["realestate"] = json!({ "properties": [
        { "id": "w", "name": "Wohnung", "status": "planned", "buy": "2030-06", "sell": "2032-03", "price": 400_000, "worth": 400_000,
          "mortgage": { "tranches": [] } } ] });
    let v = with_gain(v);
    let out = api::run(&v, std::path::Path::new("."), None);
    let single = out.single.expect("Einzellauf");
    assert!(
        single
            .property_events
            .iter()
            .any(|e| matches!(e.kind, engine::sim::PropertyEventKind::Sale) && e.month.iso() == "2032-03"),
        "{:?}",
        out.issues
    );
}

/// ISS-18/AK1–AK3: Steuerfuss und Satztabellen (auch Verheiratete) werden geprüft; Grenzwerte gelten als gültig.
#[test]
fn iss18_tax_tables_and_multiplier_must_be_in_range() {
    let errors = |v: &Value| -> Vec<String> {
        issues(v)
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| i.message)
            .collect()
    };
    // Grenzwerte: Satz 0 % und 100 %, Betrag 0, Steuerfuss 0 und 10
    for m in [0.0, 10.0] {
        let mut v = base();
        v["taxes"] = json!({ "multiplier": m, "income": [[0, 0.0], [1e6, 1.0]], "wealth": [[0, 0.0]], "capital": [] });
        assert!(errors(&v).is_empty(), "{m}: {:?}", errors(&v));
    }
    // AK1: Satz unter 0 % oder über 100 %, am Feld
    for (rate, key) in [(-0.01, "wealth"), (1.01, "income"), (-1e15, "capital")] {
        let mut v = base();
        v["taxes"][key] = json!([[0, 0.0], [100_000, rate]]);
        assert_error(
            &v,
            "taxes",
            "Stufe 2: Der Steuersatz muss zwischen 0 % und 100 % liegen.",
            Some(&format!("taxes.{key}.1.1")),
        );
    }
    // AK2: negativer Betrag, am Feld
    let mut v = base();
    v["taxes"]["income"] = json!([[-1, 0.1]]);
    assert_error(
        &v,
        "taxes",
        "Einkommenssteuer, Stufe 1: Der Betrag muss 0 oder grösser sein.",
        Some("taxes.income.0.0"),
    );
    // AK1, AK2: Tabellen für Verheiratete
    let mut v = base();
    v["taxes"]["couple"] = json!({ "income": [], "wealth": [[0, -0.001]], "capital": [[-5, 0.0]] });
    assert_error(
        &v,
        "taxes",
        "Vermögenssteuer (Verheiratete), Stufe 1: Der Steuersatz",
        Some("taxes.couple.wealth.0.1"),
    );
    assert_error(
        &v,
        "taxes",
        "Kapitalbezugssteuer (Verheiratete), Stufe 1: Der Betrag",
        Some("taxes.couple.capital.0.0"),
    );
    // AK3: Steuerfuss unter 0 oder über 10, am Feld
    for m in [-0.1, 10.5] {
        let mut v = base();
        v["taxes"]["multiplier"] = json!(m);
        assert_error(
            &v,
            "taxes",
            "Steuerfuss: Der Faktor muss zwischen 0 und 10 liegen.",
            Some("taxes.multiplier"),
        );
    }
}
