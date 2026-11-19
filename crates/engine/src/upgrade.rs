//! Takeover of older plans when loading (#52, #51).
//!
//! [`upgrade`] works on the plan's JSON as the user interface holds it and returns a note for every change,
//! so that nothing is lost silently:
//! - Every person gets the fixed milestones «Bezug Pensionskasse» and «AHV-Bezug» (default 65 years); an age
//!   outside the limits from the settings is set to the nearest limit.
//! - Pension fund and AHV pension refer to these milestones.
//! - The pillar 3a on a mortgage becomes a 3a account of its own of the first person with a payout plan.
//! - #69: The chosen series becomes an investment strategy (index for stocks and bonds).
//! - #100/#101: The parametric model gets the flag `auto` (parameters follow the stock index). If the
//!   plan computes parametrically, its parameters stay (`auto` = false) and a note names the changed computation.
//! - #132: The parametric model gets the flag `sampler` (sampling method). If the plan computes
//!   parametrically, a note names the different random paths.

use std::borrow::Cow;

use serde_json::{json, Map, Value};

use crate::message::Message;
use crate::model::{fixed_milestone_name, person_milestone_name, person_milestones, Plan, AHV_NAME, PK_NAME};
use crate::settings::{Limits, DEFAULT_AGE};
use crate::time::YearMonth;

fn ym(v: &Value) -> Option<YearMonth> {
    serde_json::from_value(v.clone()).ok()
}

/// #171: Age with at most one decimal as a parameter (65, 62.5).
fn age_value(y: f64) -> f64 {
    (y * 10.0).round() / 10.0
}

/// Month of a milestone from the JSON (date or age of a person).
fn milestone_month(ms: &Value, births: &[(String, YearMonth)]) -> Option<YearMonth> {
    match ms.get("kind").and_then(Value::as_str) {
        Some("date") => ms.get("date").and_then(ym),
        Some("age") => {
            let person = ms.get("person").and_then(Value::as_str)?;
            let years = ms.get("years").and_then(Value::as_f64)?;
            let birth = births.iter().find(|(id, _)| id == person)?.1;
            Some(birth.plus((years * 12.0).round() as i32))
        }
        _ => None,
    }
}

/// Brings a plan up to date; the notes describe every change.
pub fn upgrade(v: &mut Value) -> Vec<Message> {
    let mut notes = vec![];
    if !v.is_object() {
        return notes;
    }
    let limits: Limits = v
        .get("limits")
        .and_then(|l| serde_json::from_value(l.clone()).ok())
        .unwrap_or_default();
    let births: Vec<(String, YearMonth)> = v
        .pointer("/household/persons")
        .and_then(Value::as_array)
        .map(|ps| {
            ps.iter()
                .filter_map(|p| Some((p.get("id")?.as_str()?.to_string(), p.get("birth").and_then(ym)?)))
                .collect()
        })
        .unwrap_or_default();
    format(v);
    milestones(v, &births, &limits, &mut notes);
    pension_refs(v, &births, &mut notes);
    pillar3a(v, &births, &mut notes);
    strategy(v, &mut notes);
    pictet(v, &mut notes);
    parametric(v, &mut notes);
    sampler(v, &mut notes);
    // #154: last, because `housing` needs a readable plan, and the steps before can only make it readable
    // (e.g. an invalid `returns.series`). Otherwise only the next opening would catch up on the adjustment.
    housing(v, &mut notes);
    notes
}

/// #101: Note when opening a plan that computed parametrically before #100/#101.
pub const PARAMETRIC_NOTE: &str = "Monte Carlo parametrisch rechnet neu auch Obligationen und Inflation zufällig, korreliert mit den Aktien gemäss der Anlagestrategie (bisher fixe Werte). Ihre Parameter des Aktienmodells bleiben; die Vorschlagswerte für den Aktienindex stehen im Reiter «Simulation». Das Ergebnis ändert sich.";

/// #100/#101: Flag `auto` for the parameters of the stock model. Plans that compute parametrically keep their
/// values (with a note, #101/AK6); for all others the values were not in use and from now on follow the
/// stock index.
fn parametric(v: &mut Value, notes: &mut Vec<Message>) {
    let Some(ret) = v.get_mut("returns").and_then(Value::as_object_mut) else {
        return;
    };
    let active = ret.get("source").and_then(Value::as_str) == Some("parametric");
    match ret.get_mut("parametric").and_then(Value::as_object_mut) {
        Some(q) if q.contains_key("auto") => return,
        Some(q) => {
            q.insert("auto".into(), json!(!active));
        }
        // without saved values it computed with the previous default values; these stay
        None if active => {
            let q = serde_json::to_value(crate::model::ParametricParams::default()).unwrap_or_default();
            ret.insert("parametric".into(), q);
        }
        None => {}
    }
    if active {
        notes.push(msg!(note_parametric));
    }
}

/// #132/AK7: Note when opening a plan that computed parametrically before #132.
pub const SAMPLER_NOTE: &str = "Monte Carlo parametrisch erzeugt die Zufallspfade mit einem schnelleren Verfahren. Die Verteilung der Renditen bleibt gleich, die einzelnen Pfade sind aber andere. Das Ergebnis ändert sich deshalb leicht; mit demselben Zufallsstartwert bleibt es wiederholbar.";

/// #132: Flag `sampler` for the parametric model. If it is missing, the plan dates from before #132; if it computes
/// parametrically, it gets a note (not in addition to [`PARAMETRIC_NOTE`], which already names the change).
fn sampler(v: &mut Value, notes: &mut Vec<Message>) {
    let active = v.pointer("/returns/source").and_then(Value::as_str) == Some("parametric");
    let Some(q) = v.pointer_mut("/returns/parametric").and_then(Value::as_object_mut) else {
        return;
    };
    if q.contains_key("sampler") {
        return;
    }
    q.insert("sampler".into(), json!(crate::montecarlo::SAMPLER));
    if active && !notes.iter().any(|n| n == PARAMETRIC_NOTE) {
        notes.push(msg!(note_sampler));
    }
}

/// #90: The index «Schweiz (Pictet)» is no longer bundled. Stocks «Schweiz (Pictet)» become
/// «Schweiz (JST)», bonds «Schweiz (Pictet)» become «Eidgenossenschaft 10 Jahre», with a note.
fn pictet(v: &mut Value, notes: &mut Vec<Message>) {
    let Some(s) = v.pointer_mut("/returns/strategy").and_then(Value::as_object_mut) else {
        return;
    };
    let mut parts = vec![];
    if s.get("stocks").and_then(Value::as_str) == Some(PICTET_KEY) {
        s.insert("stocks".into(), json!("chJst"));
        parts.push(msg!(note_pictet_stocks));
    }
    if s.get("bonds").and_then(Value::as_str) == Some(PICTET_KEY) {
        s.insert("bonds".into(), json!("ch"));
        parts.push(msg!(note_pictet_bonds));
    }
    if !parts.is_empty() {
        notes.push(if parts.len() > 1 {
            msg!(note_pictet_both, first = parts[0], second = parts[1])
        } else {
            msg!(note_pictet, parts = parts[0])
        });
    }
}

/// #90: Key of the former index «Schweiz (Pictet)» in the plan file.
const PICTET_KEY: &str = "chPictet";

/// #69: The series of an older plan becomes the investment strategy with the same values. Series without
/// a matching strategy (in USD or own files) are replaced, with a note.
fn strategy(v: &mut Value, notes: &mut Vec<Message>) {
    let Some(ret) = v.get_mut("returns").and_then(Value::as_object_mut) else {
        return;
    };
    if ret.contains_key("strategy") {
        return;
    }
    let series = ret.remove("series").and_then(|s| s.as_str().map(str::to_string));
    let (stocks, bonds) = match series.as_deref() {
        None | Some("library:Shiller CHF.csv") => ("sp500", "usa"),
        Some("library:Shiller CHF + CH-Obligationen.csv") => ("sp500", "ch"),
        // #90: replaced afterwards in `pictet` (with a note).
        Some("library:Pictet CH.csv") => (PICTET_KEY, PICTET_KEY),
        Some("library:Welt (JST).csv") => ("worldJst", "ch"),
        Some("library:Shiller USD.csv") => {
            notes.push(msg!(note_strategy_usd));
            ("sp500", "usa")
        }
        Some(other) => {
            let name = other.strip_prefix(crate::library::PREFIX).unwrap_or(other);
            notes.push(msg!(note_own_series, name = name));
            ("worldJst", "ch")
        }
    };
    ret.insert("strategy".into(), json!({ "stocks": stocks, "bonds": bonds }));
}

/// #69: Note if a plan used the series in USD.
pub const STRATEGY_USD_NOTE: &str = "Die Zeitreihe «Shiller USD» (in USD) wird durch die Anlagestrategie S&P 500 mit US-Obligationen in CHF ersetzt; alle Strategien rechnen in CHF. Das Ergebnis ändert sich.";

/// #44: The old format identifier (legacy: RetirementPlanner) is replaced by the new one; the content stays the same,
/// hence without a note.
fn format(v: &mut Value) {
    if v.get("format").and_then(Value::as_str) == Some(crate::model::LEGACY_FORMAT) {
        v["format"] = Value::String(crate::model::FORMAT.into());
    }
}

/// #54: Note when opening an older plan; the result may change.
pub const CASH_NOTE: &str = "Cash wird neu bis zur Pensionierung (Bezug Pensionskasse) auf dem erfassten Betrag gehalten, mit der Teuerung fortgeschrieben; ein Überschuss geht ins Portfolio. Die Entnahmestrategie, auch «Standard» mit ihrer Cash-Reserve, gilt erst ab der Pensionierung. Das Ergebnis kann sich deshalb ändern.";

/// #53: Older plans entered the living costs without housing costs. The housing share at the simulation start is added to
/// every value, so that the result stays the same.
fn housing(v: &mut Value, notes: &mut Vec<Message>) {
    if v.pointer("/flows/spendingIncludesHousing").and_then(Value::as_bool) == Some(true) || v.get("flows").is_none() {
        return;
    }
    let Ok(plan) = serde_json::from_value::<Plan>(v.clone()) else {
        return;
    };
    let Ok(r) = crate::resolve::Resolver::new(&plan) else {
        return;
    };
    let w = crate::housing::start_housing(&plan, &r);
    v["flows"]["spendingIncludesHousing"] = json!(true);
    // #54/AK8: same marking of an older plan (before #53/#54).
    notes.push(msg!(note_cash));
    if w <= 0.0 {
        return;
    }
    let add = |x: &mut Value| {
        if let Some(n) = x.as_f64() {
            *x = json!(n + w);
        }
    };
    let s = &mut v["flows"]["spending"];
    if s.is_number() {
        add(s);
    } else {
        if let Some(x) = s.get_mut("value") {
            add(x);
        }
        if let Some(rows) = s.get_mut("rows").and_then(Value::as_array_mut) {
            for row in rows {
                if let Some(x) = row.get_mut("value") {
                    add(x);
                }
            }
        }
    }
    notes.push(msg!(note_spending_includes_housing, housing = w));
}

/// #51: fixed milestones per person, names and limits. #170: the fixed milestones store no name; the German
/// default names of `start` and `end` are removed, a name of its own stays.
fn milestones(v: &mut Value, births: &[(String, YearMonth)], limits: &Limits, notes: &mut Vec<Message>) {
    if !v.get("milestones").is_some_and(Value::is_array) {
        v["milestones"] = json!([]);
    }
    for ms in v["milestones"].as_array_mut().expect("Liste") {
        let id = ms.get("id").and_then(Value::as_str).unwrap_or("");
        if !matches!(id, "start" | "end") {
            continue;
        }
        let default = fixed_milestone_name(id);
        if ms.get("name").and_then(Value::as_str) == default.as_deref() {
            if let Some(o) = ms.as_object_mut() {
                o.remove("name");
            }
        }
    }
    for (i, (pid, birth)) in births.iter().take(2).enumerate() {
        let (pk, ahv) = person_milestones(i);
        for (id, base, lo, hi) in [
            (pk, PK_NAME, limits.pk_min, limits.pk_max),
            (ahv, AHV_NAME, limits.ahv_min, limits.ahv_max),
        ] {
            let name = person_milestone_name(base, i);
            let name_msg = crate::model::fixed_milestone_message(id);
            let list = v["milestones"].as_array_mut().expect("Liste");
            let Some(ms) = list
                .iter_mut()
                .find(|m| m.get("id").and_then(Value::as_str) == Some(id))
            else {
                // Insert before the end of the simulation, otherwise at the end.
                let at = list
                    .iter()
                    .position(|m| m.get("id").and_then(Value::as_str) == Some("end"))
                    .unwrap_or(list.len());
                list.insert(
                    at,
                    json!({ "id": id, "kind": "age", "person": pid, "years": DEFAULT_AGE }),
                );
                notes.push(msg!(
                    note_milestone_added,
                    name = name_msg,
                    years = age_value(DEFAULT_AGE)
                ));
                continue;
            };
            // #170: no stored name; the name comes from the id (#51: an older name is replaced with a note)
            let old_name = ms.get("name").and_then(Value::as_str).unwrap_or("").to_string();
            if !old_name.is_empty() && old_name != name {
                notes.push(msg!(note_milestone_renamed, old = old_name, name = name_msg));
            }
            if let Some(o) = ms.as_object_mut() {
                o.remove("name");
            }
            let other_person = ms.get("kind").and_then(Value::as_str) == Some("age")
                && ms.get("person").and_then(Value::as_str) != Some(pid.as_str());
            let age = if other_person {
                None
            } else {
                milestone_month(ms, births).map(|m| f64::from(birth.until(m)) / 12.0)
            };
            let fixed = match age {
                None => Some(DEFAULT_AGE),
                Some(a) if a < lo - 1e-9 => Some(lo),
                Some(a) if a > hi + 1e-9 => Some(hi),
                Some(_) => None,
            };
            if let Some(years) = fixed {
                let o = ms.as_object_mut().expect("Objekt");
                o.remove("date");
                o.insert("kind".into(), json!("age"));
                o.insert("person".into(), json!(pid));
                o.insert("years".into(), json!(years));
                notes.push(match age {
                    Some(a) => msg!(
                        note_milestone_age_limited,
                        name = name_msg,
                        age = age_value(a),
                        lo = age_value(lo),
                        hi = age_value(hi),
                        years = age_value(years)
                    ),
                    None => msg!(note_milestone_own_person, name = name_msg, years = age_value(years)),
                });
            }
        }
    }
}

/// #51: Pension fund and AHV pension refer to the fixed milestones of the person.
fn pension_refs(v: &mut Value, births: &[(String, YearMonth)], notes: &mut Vec<Message>) {
    for (i, base) in [(0usize, "/pension"), (1usize, "/partner")] {
        if i >= births.len() {
            continue;
        }
        let (pk, ahv) = person_milestones(i);
        for (key, id) in [("pk", pk), ("ahv", ahv)] {
            let Some(o) = v.pointer_mut(&format!("{base}/{key}")).and_then(Value::as_object_mut) else {
                continue;
            };
            let old = o.get("at").and_then(Value::as_str).unwrap_or("").to_string();
            if old != id {
                o.insert("at".into(), json!(id));
                let ms = crate::model::fixed_milestone_message(id);
                notes.push(if key == "pk" {
                    msg!(note_pk_reference, milestone = ms, old = old)
                } else {
                    msg!(note_ahv_reference, milestone = ms, old = old)
                });
            }
        }
    }
}

/// #52: Pillar 3a on a mortgage becomes a 3a account of its own of the first person.
fn pillar3a(v: &mut Value, births: &[(String, YearMonth)], notes: &mut Vec<Message>) {
    let Some((pid, _)) = births.first().cloned() else {
        return;
    };
    let month_of = |v: &Value, r: &str| -> Option<YearMonth> {
        if let Ok(m) = r.parse::<YearMonth>() {
            return Some(m);
        }
        let list = v.get("milestones").and_then(Value::as_array);
        match list.and_then(|l| l.iter().find(|m| m.get("id").and_then(Value::as_str) == Some(r))) {
            Some(ms) => milestone_month(ms, births),
            // Fixed milestone of the first person missing in the file: default age.
            None if r == person_milestones(0).0 || r == person_milestones(0).1 => {
                births.first().map(|b| b.1.plus((DEFAULT_AGE * 12.0) as i32))
            }
            None => None,
        }
    };
    let ahv_at = month_of(v, person_milestones(0).1);
    let mut found: Vec<(String, Map<String, Value>, bool)> = vec![];
    if let Some(props) = v.pointer_mut("/realestate/properties").and_then(Value::as_array_mut) {
        for p in props {
            let name = p.get("name").and_then(Value::as_str).unwrap_or("Immobilie").to_string();
            let Some(mg) = p.get_mut("mortgage").and_then(Value::as_object_mut) else {
                continue;
            };
            let indirect = mg.get("tranches").and_then(Value::as_array).is_some_and(|ts| {
                ts.iter()
                    .flat_map(|t| t.get("periods").and_then(Value::as_array).into_iter().flatten())
                    .any(|d| {
                        d.get("amortMode").and_then(Value::as_str) == Some("indirect")
                            && d.get("amort").and_then(Value::as_f64).unwrap_or(0.0) > 0.0
                    })
            });
            if let Some(Value::Object(pa)) = mg.remove("pillar3a") {
                // An empty account without indirect amortization contributes nothing (the user interface used to create it).
                if !indirect && pa.get("balance").and_then(Value::as_f64).unwrap_or(0.0) <= 0.0 {
                    continue;
                }
                found.push((name, pa, indirect));
            }
        }
    }
    if found.is_empty() {
        return;
    }
    if !v.get("pillar3a").is_some_and(Value::is_array) {
        v["pillar3a"] = json!([]);
    }
    for (name, pa, indirect) in found {
        let balance = pa.get("balance").and_then(Value::as_f64).unwrap_or(0.0);
        let rate = pa.get("rate").and_then(Value::as_f64).unwrap_or(0.0);
        let mut usage = pa.get("use").and_then(Value::as_str).unwrap_or("repay").to_string();
        if usage == "repay" && !indirect {
            usage = "wealth".into();
            notes.push(msg!(note_pillar3a_repay_to_wealth, name = name));
        }
        let payout = pa.get("payout").and_then(Value::as_str).unwrap_or("never").to_string();
        let mut entry = None;
        let text;
        match (month_of(v, &payout), ahv_at) {
            (Some(m), Some(a)) if m >= a => {
                text = msg!(note_pillar3a_account_rest_at_ahv, name = name);
            }
            (Some(m), Some(a)) => {
                let earliest = a.plus(-60);
                let date = m.max(earliest);
                if date > m {
                    text = msg!(note_pillar3a_account_moved, name = name, old = m, date = date);
                } else {
                    text = msg!(note_pillar3a_account_payout, name = name, date = date);
                }
                entry = Some(json!({ "date": date, "use": usage }));
            }
            _ => text = msg!(note_pillar3a_account_no_date, name = name),
        }
        notes.push(text);
        let list = v["pillar3a"].as_array_mut().expect("Liste");
        match list
            .iter_mut()
            .find(|a| a.get("person").and_then(Value::as_str) == Some(pid.as_str()))
        {
            Some(acc) => {
                let b = acc.get("balance").and_then(Value::as_f64).unwrap_or(0.0);
                acc["balance"] = json!(b + balance);
                if let Some(e) = entry {
                    if !acc.get("payouts").is_some_and(Value::is_array) {
                        acc["payouts"] = json!([]);
                    }
                    acc["payouts"].as_array_mut().expect("Liste").push(e);
                }
                notes.push(msg!(
                    note_pillar3a_merged,
                    rate = acc.get("rate").and_then(Value::as_f64).unwrap_or(0.0)
                ));
            }
            None => list.push(json!({
                "person": pid, "balance": balance, "rate": rate, "contribution": 0,
                "payouts": entry.map_or(vec![], |e| vec![e]), "use": usage
            })),
        }
    }
}

/// Plan with deprecated pillar 3a on the mortgage: taken over as when loading (for calls with finished plans,
/// e.g. from tests). Otherwise unchanged.
pub fn normalized(plan: &Plan) -> Cow<'_, Plan> {
    let legacy = plan
        .realestate
        .properties
        .iter()
        .any(|p| p.mortgage.as_ref().is_some_and(|m| m.pillar3a.is_some()));
    if !legacy {
        return Cow::Borrowed(plan);
    }
    let mut v = serde_json::to_value(plan).expect("serialisierbar");
    let births: Vec<(String, YearMonth)> = plan.household.persons.iter().map(|p| (p.id.clone(), p.birth)).collect();
    pillar3a(&mut v, &births, &mut vec![]);
    // Impossible values (such as an age of a billion years) cannot be taken over; the validation reports them.
    match serde_json::from_value::<Plan>(v) {
        Ok(mut p) => {
            p.limits = plan.limits;
            Cow::Owned(p)
        }
        Err(_) => Cow::Borrowed(plan),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> Value {
        json!({
            "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-06" } ] },
            "milestones": [
                { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
                { "id": "retire", "name": "Pensionierung", "kind": "age", "person": "A", "years": 60 },
                { "id": "end", "name": "Ende", "kind": "date", "date": "2070-12" }
            ],
            "pension": { "pk": { "at": "retire" }, "ahv": { "at": "never" } }
        })
    }

    fn ms<'a>(v: &'a Value, id: &str) -> &'a Value {
        v["milestones"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == id)
            .unwrap()
    }

    // #51/AK5, AK6: missing AHV payout at 65 before the end, «Pensionierung» is called «Bezug Pensionskasse».
    #[test]
    fn issue_51_ak6_missing_milestones_and_names() {
        let mut v = plan();
        let notes = upgrade(&mut v);
        assert_eq!(ms(&v, "ahv")["years"], 65.0);
        assert_eq!(v["milestones"][2]["id"], "ahv", "vor dem Ende eingefügt");
        // #170: the fixed milestone keeps no name (the user interface takes it from the text table)
        assert!(ms(&v, "retire").get("name").is_none());
        assert!(ms(&v, "ahv").get("name").is_none());
        assert_eq!(ms(&v, "retire")["years"], 60, "Alter innerhalb der Grenzen bleibt");
        assert_eq!(v["pension"]["ahv"]["at"], "ahv");
        assert!(
            notes
                .iter()
                .any(|n| n == "Der Meilenstein «AHV-Bezug» fehlte und steht neu auf 65 Jahren."),
            "{notes:?}"
        );
        assert!(notes
            .iter()
            .any(|n| n == "Der Meilenstein «Pensionierung» heisst neu «Bezug Pensionskasse»."));
        assert!(notes
            .iter()
            .any(|n| n == "Die AHV-Rente beginnt neu am Meilenstein «AHV-Bezug» (bisher «never»)."));
        // Second run: nothing left to do
        assert!(upgrade(&mut v).is_empty());
    }

    // #51/AK6: An age outside the limits is set to the nearest limit, with a note.
    #[test]
    fn issue_51_ak6_ages_outside_the_limits() {
        let mut v = plan();
        v["milestones"][1]["years"] = json!(55);
        v["milestones"].as_array_mut().unwrap().insert(
            2,
            json!({ "id": "ahv", "name": "AHV-Bezug", "kind": "date", "date": "2052-06" }),
        );
        let notes = upgrade(&mut v);
        assert_eq!(
            (ms(&v, "retire")["years"].as_f64(), ms(&v, "ahv")["years"].as_f64()),
            (Some(58.0), Some(70.0))
        );
        assert_eq!(ms(&v, "ahv")["kind"], "age");
        assert!(ms(&v, "ahv").get("date").is_none());
        assert!(
            notes.iter().any(|n| n
                == "«Bezug Pensionskasse» lag bei 55 Jahren, ausserhalb der Grenzen 58 bis 65 Jahre; neu 58 Jahre."),
            "{notes:?}"
        );
        assert!(
            notes
                .iter()
                .any(|n| n == "«AHV-Bezug» lag bei 72 Jahren, ausserhalb der Grenzen 63 bis 70 Jahre; neu 70 Jahre."),
            "{notes:?}"
        );
        // #25: Limits from the settings
        let mut v = plan();
        v["milestones"][1]["years"] = json!(55);
        v["limits"] = json!({ "pkMin": 50 });
        upgrade(&mut v);
        assert_eq!(ms(&v, "retire")["years"], 55);
    }

    // #170/AK8: opening an older file removes the German default names of the fixed milestones; a name of its own
    // stays; the second run changes nothing. The engine uses the German default name by the id.
    #[test]
    fn issue_170_ak8_default_names_of_fixed_milestones_are_removed() {
        let mut v = plan();
        v["household"]["persons"] =
            json!([{ "id": "A", "name": "A", "birth": "1980-06" }, { "id": "B", "name": "B", "birth": "1982-01" }]);
        v["partner"] = json!({ "pk": { "at": "retire2" }, "ahv": { "at": "ahv2" } });
        v["milestones"] = json!([
            { "id": "start", "name": "Simulationsstart", "kind": "date", "date": "2026-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "retire2", "name": "Bezug Pensionskasse Person 2", "kind": "age", "person": "B", "years": 65 },
            { "id": "ahv2", "name": "AHV-Bezug Person 2", "kind": "age", "person": "B", "years": 65 },
            { "id": "sale", "name": "Hausverkauf", "kind": "date", "date": "2040-01" },
            { "id": "end", "name": "Planende", "kind": "date", "date": "2060-12", "fixed": true }
        ]);
        let notes = upgrade(&mut v);
        assert!(notes.iter().all(|n| !n.contains("heisst neu")), "{notes:?}");
        for id in ["start", "retire", "ahv", "retire2", "ahv2"] {
            assert!(ms(&v, id).get("name").is_none(), "{id}");
        }
        assert_eq!(ms(&v, "end")["name"], "Planende", "eigener Name bleibt");
        assert_eq!(ms(&v, "sale")["name"], "Hausverkauf");
        let before = v.clone();
        assert!(upgrade(&mut v).is_empty());
        assert_eq!(v, before);
        // The engine reads the plan and names the fixed milestones in German (texts unchanged until #176)
        let milestones: Vec<crate::model::Milestone> = serde_json::from_value(v["milestones"].clone()).unwrap();
        let labels: Vec<String> = milestones.iter().map(|m| m.label().into_owned()).collect();
        assert_eq!(
            labels,
            [
                "Simulationsstart",
                "Bezug Pensionskasse",
                "AHV-Bezug",
                "Bezug Pensionskasse Person 2",
                "AHV-Bezug Person 2",
                "Hausverkauf",
                "Planende"
            ]
        );
    }

    // #51/AK1: the second person gets milestones of their own; the payouts of person 2 point to them.
    #[test]
    fn issue_51_second_person() {
        let mut v = plan();
        v["household"]["persons"] = json!([{ "id": "A", "birth": "1980-06" }, { "id": "B", "birth": "1982-01" }]);
        v["partner"] = json!({ "pk": { "at": "retire" }, "ahv": { "at": "ahv" } });
        upgrade(&mut v);
        assert!(ms(&v, "retire2").get("name").is_none());
        assert_eq!(ms(&v, "ahv2")["person"], "B");
        assert_eq!(
            (v["partner"]["pk"]["at"].as_str(), v["partner"]["ahv"]["at"].as_str()),
            (Some("retire2"), Some("ahv2"))
        );
        // Milestone of the wrong person: moved to the own person at 65 years
        let mut v = plan();
        v["household"]["persons"] = json!([{ "id": "A", "birth": "1980-06" }, { "id": "B", "birth": "1982-01" }]);
        v["milestones"].as_array_mut().unwrap().insert(
            2,
            json!({ "id": "ahv2", "name": "AHV-Bezug Person 2", "kind": "age", "person": "A", "years": 64 }),
        );
        let notes = upgrade(&mut v);
        assert_eq!(
            (ms(&v, "ahv2")["person"].as_str(), ms(&v, "ahv2")["years"].as_f64()),
            (Some("B"), Some(65.0))
        );
        assert!(notes
            .iter()
            .any(|n| n.contains("bezieht sich neu auf das Alter der eigenen Person")));
    }

    fn with_legacy(pa: Value, amort: &str) -> Value {
        let mut v = plan();
        v["realestate"] = json!({ "properties": [ { "name": "Haus", "mortgage": { "tranches": [ { "periods": [
            { "amort": 500, "amortMode": amort } ] } ], "pillar3a": pa } } ] });
        v
    }

    // #52/AK13: Pillar 3a on the mortgage becomes an account of the first person with a payout plan.
    #[test]
    fn issue_52_ak13_legacy_pillar3a() {
        // AHV payout 2045-06 (65); earliest payout 2040-06.
        let mut v = with_legacy(
            json!({ "balance": 40_000, "rate": 0.01, "payout": "retire", "use": "repay" }),
            "indirect",
        );
        let notes = upgrade(&mut v);
        assert!(v["realestate"]["properties"][0]["mortgage"].get("pillar3a").is_none());
        assert_eq!(
            v["pillar3a"],
            json!([{ "person": "A", "balance": 40_000.0, "rate": 0.01, "contribution": 0,
            "payouts": [{ "date": "2040-06", "use": "repay" }], "use": "repay" }])
        );
        assert!(
            notes
                .iter()
                .any(|n| n.contains("Auszahlung des ganzen Guthabens im 06.2040")),
            "{notes:?}"
        );
        // Too early: limited to the earliest point in time
        let mut v = with_legacy(
            json!({ "balance": 1, "rate": 0, "payout": "2035-01", "use": "wealth" }),
            "indirect",
        );
        let notes = upgrade(&mut v);
        assert_eq!(v["pillar3a"][0]["payouts"][0]["date"], "2040-06");
        assert!(notes
            .iter()
            .any(|n| n.contains("lag mehr als 5 Jahre vor dem AHV-Bezug und ist neu im 06.2040")));
        // After the AHV payout or «nie»: no plan entry, remainder at the AHV payout
        for payout in ["2050-01", "never"] {
            let mut v = with_legacy(
                json!({ "balance": 1, "rate": 0, "payout": payout, "use": "wealth" }),
                "indirect",
            );
            upgrade(&mut v);
            assert_eq!(v["pillar3a"][0]["payouts"], json!([]), "{payout}");
        }
        // «Hypothek zurückzahlen» without indirect amortization: into free wealth, with a note
        let mut v = with_legacy(
            json!({ "balance": 5, "rate": 0, "payout": "retire", "use": "repay" }),
            "direct",
        );
        let notes = upgrade(&mut v);
        assert_eq!(v["pillar3a"][0]["use"], "wealth");
        assert!(notes
            .iter()
            .any(|n| n.contains("setzt neu eine Tranche mit indirekter Amortisation voraus")));
        // An empty account without indirect amortization is dropped silently
        let mut v = with_legacy(
            json!({ "balance": 0, "rate": 0, "payout": "retire", "use": "repay" }),
            "direct",
        );
        assert!(upgrade(&mut v).iter().all(|n| !n.contains("3a")));
        assert!(v.get("pillar3a").is_none());
    }

    // #52/AK13: several properties with 3a are combined into one account.
    #[test]
    fn issue_52_ak13_two_legacy_accounts_are_merged() {
        let mut v = with_legacy(
            json!({ "balance": 10_000, "rate": 0.01, "payout": "retire", "use": "wealth" }),
            "indirect",
        );
        let second = v["realestate"]["properties"][0].clone();
        v["realestate"]["properties"].as_array_mut().unwrap().push(second);
        v["realestate"]["properties"][1]["mortgage"]["pillar3a"]["balance"] = json!(5000);
        let notes = upgrade(&mut v);
        assert_eq!(v["pillar3a"].as_array().unwrap().len(), 1);
        assert_eq!(v["pillar3a"][0]["balance"], 15_000.0);
        assert_eq!(v["pillar3a"][0]["payouts"].as_array().unwrap().len(), 2);
        assert!(notes.iter().any(|n| n.contains("zu einem Konto zusammengefasst")));
    }

    // Unusable data does not lead to a crash.
    #[test]
    fn odd_input() {
        assert!(upgrade(&mut json!(null)).is_empty());
        let mut v = json!({ "household": { "persons": [] } });
        assert!(upgrade(&mut v).is_empty());
    }

    // #124/AK6: An age exactly at the limit stays (no note). A pillar 3a on the mortgage without a
    // payout date (or with an unknown milestone) is paid out at the AHV payout; «Hypothek zurückzahlen» requires
    // an indirect amortization above 0. A plan that does not compute parametrically gets no parameters.
    #[test]
    fn issue_124_ak6_limits_pillar3a_and_parametric() {
        let mut v = plan();
        v["milestones"][1]["years"] = json!(58);
        let notes = upgrade(&mut v);
        assert_eq!(ms(&v, "retire")["years"], 58);
        assert!(!notes.iter().any(|n| n.contains("ausserhalb der Grenzen")), "{notes:?}");

        for payout in [None, Some("never"), Some("unbekannt")] {
            let mut v = plan();
            let mut pa = json!({ "balance": 10_000, "rate": 0, "use": "repay" });
            if let Some(p) = payout {
                pa["payout"] = json!(p);
            }
            v["realestate"] = json!({ "properties": [ { "name": "Haus", "mortgage": { "pillar3a": pa,
                "tranches": [ { "amount": 100_000, "periods": [ { "amort": 0, "amortMode": "indirect" } ] } ] } } ] });
            let notes = upgrade(&mut v);
            assert!(
                notes.iter().any(|n| n == "Die Säule 3a von «Haus» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»); ohne Auszahlungszeitpunkt wird das Guthaben im Monat des AHV-Bezugs ausbezahlt."),
                "{payout:?}: {notes:?}"
            );
            assert_eq!(v["pillar3a"][0]["use"], "wealth", "Amortisation 0: nicht indirekt");
            assert!(notes
                .iter()
                .any(|n| n.contains("setzt neu eine Tranche mit indirekter Amortisation voraus")));
        }

        let mut v = plan();
        v["returns"] = json!({ "source": "fixed" });
        assert!(upgrade(&mut v).iter().all(|n| n != PARAMETRIC_NOTE));
        assert!(v["returns"].get("parametric").is_none());
    }

    // #154: An invalid series (`returns.series` not text) makes the plan readable only after the step `strategy`.
    // The adjustment of the housing costs (#53/#54) still happens in the first run; the second changes nothing.
    #[test]
    fn issue_154_housing_runs_after_the_plan_became_readable() {
        let mut v = crate::api::example_plan();
        let o = v.as_object_mut().unwrap();
        o["returns"].as_object_mut().unwrap().remove("strategy");
        o["returns"]["series"] = json!([]);
        o["flows"].as_object_mut().unwrap().remove("spendingIncludesHousing");
        let notes = upgrade(&mut v);
        assert!(notes.iter().any(|n| n == CASH_NOTE), "{notes:?}");
        assert_eq!(v["flows"]["spendingIncludesHousing"], true);
        let once = v.clone();
        assert!(upgrade(&mut v).is_empty());
        assert_eq!(v, once);
    }
}
