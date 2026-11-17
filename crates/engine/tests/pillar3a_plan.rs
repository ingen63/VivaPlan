//! #55: Auszahlungsplan der Säule 3a mit Datum oder Alter, Anzeige des Guthabens aus der Vorschau.
//!
//! Erfundene Planung: Geburt 1980-06, Start 2040-01, AHV-Bezug mit 65 Jahren (2045-06), frühestens 5 Jahre davor
//! (2040-06); keine Renditen, keine Teuerung, keine Steuern.

use engine::api;
use engine::model::{PayoutAt, Plan};
use engine::time::YearMonth;
use engine::validate::Severity;
use serde_json::{json, Value};
use std::path::Path;

fn base(payouts: Value) -> Value {
    json!({
        "project": { "name": "3a-Plan" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-06" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] },
        "pillar3a": [ { "person": "A", "balance": 120_000, "rate": 0, "payouts": payouts } ]
    })
}

fn ym(y: i32, m: u32) -> YearMonth {
    YearMonth::new(y, m)
}

fn errors(v: &Value) -> Vec<(Option<String>, String)> {
    api::validate(v)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| (i.field, i.message))
        .collect()
}

/// #55/AK2: Geburt 1980-06, Alter 61.5 ergibt 12.2041.
#[test]
fn issue_55_ak2_age_gives_the_month() {
    let plan: Plan = serde_json::from_value(base(json!([{ "kind": "age", "years": 61.5, "amount": 50_000 }]))).unwrap();
    let x = &plan.pillar3a[0].payouts[0];
    assert_eq!(x.kind, PayoutAt::Age);
    assert_eq!(x.month(Some(ym(1980, 6))), Some(ym(2041, 12)));
    // ohne Geburt, ohne Alter oder mit absurdem Alter: kein Monat
    assert_eq!(x.month(None), None);
    let bad: Plan = serde_json::from_value(base(
        json!([{ "kind": "age", "years": 1e9 }, { "kind": "age" }, { "kind": "date" }]),
    ))
    .unwrap();
    assert!(bad.pillar3a[0]
        .payouts
        .iter()
        .all(|x| x.month(Some(ym(1980, 6))).is_none()));
}

/// #55/AK3: «frühestens 5 Jahre vor dem AHV-Bezug» gilt auch nach Alter; die Meldung steht am Feld «years».
#[test]
fn issue_55_ak3_earliest_rule_for_age_rows() {
    assert!(
        errors(&base(json!([{ "kind": "age", "years": 60, "amount": 1000 }]))).is_empty(),
        "60 Jahre = 2040-06 ist erlaubt"
    );
    let e = errors(&base(json!([{ "kind": "age", "years": 59.9, "amount": 1000 }])));
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.payouts.0.years"));
    assert_eq!(e[0].1, "Säule 3a: Die Auszahlung im 05.2040 liegt mehr als 5 Jahre vor dem AHV-Bezug (06.2045); frühestens im 06.2040.");
    // Als Datum: dieselbe Regel am Feld «date»
    let e = errors(&base(json!([{ "date": "2040-05", "amount": 1000 }])));
    assert_eq!(e[0].0.as_deref(), Some("pillar3a.0.payouts.0.date"));
    // Fehlendes Alter bzw. Datum ist ein Fehler am Feld
    let e = errors(&base(json!([{ "kind": "age" }, { "kind": "date" }])));
    assert_eq!(
        e,
        vec![
            (
                Some("pillar3a.0.payouts.0.years".into()),
                "Säule 3a: Das Alter (0 bis 150 Jahre) der 1. Auszahlung fehlt oder ist ungültig.".into()
            ),
            (
                Some("pillar3a.0.payouts.1.date".into()),
                "Säule 3a: Das Datum der 2. Auszahlung fehlt oder ist ungültig.".into()
            ),
        ]
    );
}

/// #55/AK4, AK5: Guthaben 120'000, Zeile 2041 (Alter 61.5) mit 50'000: vor 120'000, Rest 70'000; im AHV-Bezug 70'000.
/// Die Vorschau liefert ausserdem das Guthaben jedes Monats (für das Vorabfüllen, AK6/AK7: 2042-12 → 70'000).
#[test]
fn issue_55_ak4_ak5_preview_shows_balances() {
    let v = base(json!([{ "kind": "age", "years": 61.5, "amount": 50_000 }]));
    let out = api::preview(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let view = &out.pillar3a[0];
    assert_eq!(view.person, "A");
    let p = &view.payouts[0];
    assert_eq!((p.index, p.month), (0, ym(2041, 12)));
    assert_eq!((p.before, p.paid, p.after), (120_000.0, 50_000.0, 70_000.0));
    let rest = view.rest.as_ref().unwrap();
    assert_eq!((rest.month, rest.amount), (ym(2045, 6), 70_000.0));
    // Guthaben vor den Auszahlungen jedes Monats ab Start bis zum AHV-Bezug
    let at = |m: YearMonth| view.balances[ym(2040, 1).until(m) as usize];
    assert_eq!(at(ym(2040, 1)), 120_000.0);
    assert_eq!(at(ym(2041, 12)), 120_000.0);
    assert_eq!(at(ym(2042, 12)), 70_000.0);
    assert_eq!(view.balances.len(), ym(2040, 1).until(ym(2045, 6)) as usize + 1);
    // In gespeicherten Ergebnissen erscheint der Plan nicht (Golden-Dateien unverändert).
    let saved = serde_json::to_value(out.result.as_ref().unwrap()).unwrap();
    assert!(saved.get("pillar3a").is_none());
}

/// #55: Die Simulation zahlt nach Alter wie nach Datum aus; gleiche Zeitpunkte ergeben gleiche Ergebnisse.
#[test]
fn issue_55_age_and_date_rows_give_the_same_result() {
    let run = |v: Value| api::preview(&v, Path::new("/"), None).result.unwrap();
    let a = run(base(json!([{ "kind": "age", "years": 61.5, "amount": 50_000 }])));
    let d = run(base(json!([{ "date": "2041-12", "amount": 50_000 }])));
    for (x, y) in a.records.iter().zip(&d.records) {
        assert_eq!((x.pillar3a, x.total), (y.pillar3a, y.total), "{}", x.month);
    }
}

/// #55/AK8: Ältere Zeilen nur mit Datum bleiben gültig, gelten als «Datum» und werden ohne «kind» geschrieben.
#[test]
fn issue_55_ak8_older_rows_stay_dates() {
    let v = base(json!([{ "date": "2041-01", "amount": 10_000, "use": "wealth" }]));
    assert!(errors(&v).is_empty());
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(plan.pillar3a[0].payouts[0].kind, PayoutAt::Date);
    let back = serde_json::to_value(&plan).unwrap();
    assert_eq!(
        back["pillar3a"][0]["payouts"][0],
        json!({ "date": "2041-01", "amount": 10_000.0, "use": "wealth" })
    );
    assert!(api::upgrade_plan(&v).plan["pillar3a"][0]["payouts"][0]
        .get("kind")
        .is_none());
}

/// #55: Auszahlung vor dem Start wird auf den Start gelegt; mehrere Zeilen im selben Monat laufen in Planreihenfolge.
#[test]
fn issue_55_before_start_and_same_month_rows() {
    let v = base(json!([
        { "date": "2040-06", "amount": 30_000 },
        { "date": "2040-06", "amount": 20_000 },
        { "date": "2040-06", "amount": 100_000 },
        { "date": "2040-06", "amount": 1_000 }
    ]));
    let out = api::preview(&v, Path::new("/"), None);
    let view = &out.pillar3a[0];
    let p = &view.payouts;
    assert_eq!(p.iter().map(|x| x.index).collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    assert_eq!((p[0].before, p[0].paid, p[0].after), (120_000.0, 30_000.0, 90_000.0));
    assert_eq!((p[1].before, p[1].paid, p[1].after), (90_000.0, 20_000.0, 70_000.0));
    // Mehr als das Guthaben wird auf das Guthaben begrenzt
    assert_eq!((p[2].before, p[2].paid, p[2].after), (70_000.0, 70_000.0, 0.0));
    assert_eq!(view.rest.as_ref().unwrap().amount, 0.0);

    // Ein Datum vor dem Start (2040-06, Start 2040-09) wird auf den Start gelegt
    let mut v = base(json!([{ "date": "2040-06", "amount": 40_000 }]));
    v["milestones"][0]["date"] = json!("2040-09");
    let out = api::preview(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    let p = &out.pillar3a[0].payouts[0];
    assert_eq!(
        (p.month, p.before, p.paid, p.after),
        (ym(2040, 9), 120_000.0, 40_000.0, 80_000.0)
    );
}

/// #55: Zwei Personen haben je eine eigene Ansicht, Alter zählt ab der jeweiligen Geburt.
#[test]
fn issue_55_two_people_each_get_a_view() {
    let mut v = base(json!([{ "kind": "age", "years": 61, "amount": 50_000 }]));
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1980-06" },
        { "id": "B", "name": "B", "birth": "1981-06" }
    ]);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "name": "Bezug PK 2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "name": "AHV-Bezug 2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv2" } });
    v["pillar3a"] = json!([
        { "person": "A", "balance": 120_000, "rate": 0, "payouts": [{ "kind": "age", "years": 61, "amount": 50_000 }] },
        { "person": "B", "balance": 80_000, "rate": 0, "payouts": [{ "kind": "age", "years": 61, "amount": 30_000 }] }
    ]);
    let out = api::preview(&v, Path::new("/"), None);
    assert!(out.error.is_none(), "{:?}", out.error);
    assert_eq!(out.pillar3a.len(), 2, "{:?} {:?}", out.error, out.issues);
    let (a, b) = (&out.pillar3a[0], &out.pillar3a[1]);
    assert_eq!((a.person.as_str(), b.person.as_str()), ("A", "B"));
    assert_eq!((a.payouts[0].month, a.payouts[0].after), (ym(2041, 6), 70_000.0));
    assert_eq!(
        (b.payouts[0].month, b.payouts[0].before, b.payouts[0].after),
        (ym(2042, 6), 80_000.0, 50_000.0)
    );
    assert_eq!(a.rest.as_ref().unwrap().month, ym(2045, 6));
    assert_eq!(b.rest.as_ref().unwrap().month, ym(2046, 6));
    assert_eq!(b.rest.as_ref().unwrap().amount, 50_000.0);
}
