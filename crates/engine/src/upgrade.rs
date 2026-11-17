//! Übernahme älterer Planungen beim Laden (#52, #51).
//!
//! [`upgrade`] arbeitet auf dem JSON der Planung, wie die Oberfläche es hält, und liefert Hinweise für jede Änderung,
//! damit nichts still verloren geht:
//! - Jede Person erhält die festen Meilensteine «Bezug Pensionskasse» und «AHV-Bezug» (Standard 65 Jahre); ein Alter
//!   ausserhalb der Grenzen aus den Einstellungen wird auf die nächste Grenze gesetzt.
//! - Pensionskasse und AHV-Rente beziehen sich auf diese Meilensteine.
//! - Die Säule 3a an einer Hypothek wird ein eigenes 3a-Konto der ersten Person mit einem Auszahlungsplan.
//! - #69: Die gewählte Zeitreihe wird eine Anlagestrategie (Index für Aktien und Obligationen).
//! - #100/#101: Das parametrische Modell erhält das Merkmal `auto` (Parameter folgen dem Aktienindex). Rechnet die
//!   Planung parametrisch, bleiben ihre Parameter (`auto` = falsch) und ein Hinweis nennt die geänderte Rechnung.

use std::borrow::Cow;

use serde_json::{json, Map, Value};

use crate::model::{person_milestone_name, person_milestones, Plan, AHV_NAME, PK_NAME};
use crate::settings::{Limits, DEFAULT_AGE};
use crate::time::YearMonth;

fn ym(v: &Value) -> Option<YearMonth> {
    serde_json::from_value(v.clone()).ok()
}

fn fmt_age(y: f64) -> String {
    let s = format!("{y:.1}");
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

/// Monat eines Meilensteins aus dem JSON (Datum oder Alter einer Person).
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

/// Bringt eine Planung auf den aktuellen Stand; die Hinweise beschreiben jede Änderung.
pub fn upgrade(v: &mut Value) -> Vec<String> {
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
    // #154: zuletzt, denn `housing` braucht eine lesbare Planung, und die Schritte davor können sie erst lesbar machen
    // (z. B. eine ungültige `returns.series`). Sonst holte erst das nächste Öffnen die Anpassung nach.
    housing(v, &mut notes);
    notes
}

/// #101: Hinweis beim Öffnen einer Planung, die vor #100/#101 parametrisch rechnete.
pub const PARAMETRIC_NOTE: &str = "Monte Carlo parametrisch rechnet neu auch Obligationen und Inflation zufällig, korreliert mit den Aktien gemäss der Anlagestrategie (bisher fixe Werte). Ihre Parameter des Aktienmodells bleiben; die Vorschlagswerte für den Aktienindex stehen im Reiter «Simulation». Das Ergebnis ändert sich.";

/// #100/#101: Merkmal `auto` für die Parameter des Aktienmodells. Planungen, die parametrisch rechnen, behalten ihre
/// Werte (mit Hinweis, #101/AK6); bei allen anderen waren die Werte nicht in Gebrauch und folgen künftig dem
/// Aktienindex.
fn parametric(v: &mut Value, notes: &mut Vec<String>) {
    let Some(ret) = v.get_mut("returns").and_then(Value::as_object_mut) else {
        return;
    };
    let active = ret.get("source").and_then(Value::as_str) == Some("parametric");
    match ret.get_mut("parametric").and_then(Value::as_object_mut) {
        Some(q) if q.contains_key("auto") => return,
        Some(q) => {
            q.insert("auto".into(), json!(!active));
        }
        // ohne gespeicherte Werte rechnete sie mit den bisherigen Standardwerten; diese bleiben
        None if active => {
            let q = serde_json::to_value(crate::model::ParametricParams::default()).unwrap_or_default();
            ret.insert("parametric".into(), q);
        }
        None => {}
    }
    if active {
        notes.push(PARAMETRIC_NOTE.into());
    }
}

/// #90: Der Index «Schweiz (Pictet)» wird nicht mehr mitgeliefert. Aktien «Schweiz (Pictet)» werden
/// «Schweiz (JST)», Obligationen «Schweiz (Pictet)» werden «Eidgenossenschaft 10 Jahre», mit Hinweis.
fn pictet(v: &mut Value, notes: &mut Vec<String>) {
    let Some(s) = v.pointer_mut("/returns/strategy").and_then(Value::as_object_mut) else {
        return;
    };
    let mut parts = vec![];
    if s.get("stocks").and_then(Value::as_str) == Some(PICTET_KEY) {
        s.insert("stocks".into(), json!("chJst"));
        parts.push("für die Aktien «Schweiz (JST)»");
    }
    if s.get("bonds").and_then(Value::as_str) == Some(PICTET_KEY) {
        s.insert("bonds".into(), json!("ch"));
        parts.push("für die Obligationen «Eidgenossenschaft 10 Jahre»");
    }
    if !parts.is_empty() {
        notes.push(format!(
            "Der Index «Schweiz (Pictet)» wird aus Lizenzgründen nicht mehr mitgeliefert. Die Anlagestrategie verwendet jetzt {}; bitte unter «Vermögen & Anlage» prüfen. Das Ergebnis ändert sich.",
            parts.join(" und ")
        ));
    }
}

/// #90: Schlüssel des früheren Index «Schweiz (Pictet)» in der Plandatei.
const PICTET_KEY: &str = "chPictet";

/// #69: Die Zeitreihe einer älteren Planung wird die Anlagestrategie mit denselben Werten. Reihen ohne
/// entsprechende Strategie (in USD oder eigene Dateien) werden ersetzt, mit Hinweis.
fn strategy(v: &mut Value, notes: &mut Vec<String>) {
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
        // #90: wird anschliessend in `pictet` ersetzt (mit Hinweis).
        Some("library:Pictet CH.csv") => (PICTET_KEY, PICTET_KEY),
        Some("library:Welt (JST).csv") => ("worldJst", "ch"),
        Some("library:Shiller USD.csv") => {
            notes.push(STRATEGY_USD_NOTE.into());
            ("sp500", "usa")
        }
        Some(other) => {
            let name = other.strip_prefix(crate::library::PREFIX).unwrap_or(other);
            notes.push(format!(
                "Die eigene Zeitreihe «{name}» wird nicht mehr unterstützt. Die Anlagestrategie ist jetzt «Welt (JST)» mit Obligationen der Eidgenossenschaft; bitte unter «Vermögen & Anlage» prüfen. Das Ergebnis ändert sich."
            ));
            ("worldJst", "ch")
        }
    };
    ret.insert("strategy".into(), json!({ "stocks": stocks, "bonds": bonds }));
}

/// #69: Hinweis, wenn eine Planung die Reihe in USD verwendete.
pub const STRATEGY_USD_NOTE: &str = "Die Zeitreihe «Shiller USD» (in USD) wird durch die Anlagestrategie S&P 500 mit US-Obligationen in CHF ersetzt; alle Strategien rechnen in CHF. Das Ergebnis ändert sich.";

/// #44: Die alte Format-Kennung (legacy: RetirementPlanner) wird durch die neue ersetzt; der Inhalt bleibt gleich,
/// deshalb ohne Hinweis.
fn format(v: &mut Value) {
    if v.get("format").and_then(Value::as_str) == Some(crate::model::LEGACY_FORMAT) {
        v["format"] = Value::String(crate::model::FORMAT.into());
    }
}

/// #54: Hinweis beim Öffnen einer älteren Planung; das Ergebnis kann sich ändern.
pub const CASH_NOTE: &str = "Cash wird neu bis zur Pensionierung (Bezug Pensionskasse) auf dem erfassten Betrag gehalten, mit der Teuerung fortgeschrieben; ein Überschuss geht ins Portfolio. Die Entnahmestrategie, auch «Standard» mit ihrer Cash-Reserve, gilt erst ab der Pensionierung. Das Ergebnis kann sich deshalb ändern.";

/// #53: Ältere Planungen erfassten die Lebenshaltung ohne Wohnkosten. Der Wohnanteil zum Simulationsstart wird zu
/// jedem Wert addiert, damit das Ergebnis gleich bleibt.
fn housing(v: &mut Value, notes: &mut Vec<String>) {
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
    // #54/AK8: gleiche Kennzeichnung einer älteren Planung (vor #53/#54).
    notes.push(CASH_NOTE.to_string());
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
    notes.push(format!(
        "Die Lebenshaltungskosten enthalten neu die Wohnkosten. Die Wohnkosten zum Simulationsstart ({:.0} pro Monat) sind zu jedem Wert addiert; das Ergebnis bleibt gleich.",
        w
    ));
}

/// #51: feste Meilensteine je Person, Namen und Grenzen.
fn milestones(v: &mut Value, births: &[(String, YearMonth)], limits: &Limits, notes: &mut Vec<String>) {
    if !v.get("milestones").is_some_and(Value::is_array) {
        v["milestones"] = json!([]);
    }
    for (i, (pid, birth)) in births.iter().take(2).enumerate() {
        let (pk, ahv) = person_milestones(i);
        for (id, base, lo, hi) in [
            (pk, PK_NAME, limits.pk_min, limits.pk_max),
            (ahv, AHV_NAME, limits.ahv_min, limits.ahv_max),
        ] {
            let name = person_milestone_name(base, i);
            let list = v["milestones"].as_array_mut().expect("Liste");
            let Some(ms) = list
                .iter_mut()
                .find(|m| m.get("id").and_then(Value::as_str) == Some(id))
            else {
                // Vor dem Simulationsende einfügen, sonst am Schluss.
                let at = list
                    .iter()
                    .position(|m| m.get("id").and_then(Value::as_str) == Some("end"))
                    .unwrap_or(list.len());
                list.insert(
                    at,
                    json!({ "id": id, "name": name, "kind": "age", "person": pid, "years": DEFAULT_AGE }),
                );
                notes.push(format!(
                    "Der Meilenstein «{name}» fehlte und steht neu auf {} Jahren.",
                    fmt_age(DEFAULT_AGE)
                ));
                continue;
            };
            let old_name = ms.get("name").and_then(Value::as_str).unwrap_or("").to_string();
            if old_name != name {
                ms["name"] = json!(name);
                notes.push(format!("Der Meilenstein «{old_name}» heisst neu «{name}»."));
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
                    Some(a) => format!(
                        "«{name}» lag bei {} Jahren, ausserhalb der Grenzen {} bis {} Jahre; neu {} Jahre.",
                        fmt_age(a),
                        fmt_age(lo),
                        fmt_age(hi),
                        fmt_age(years)
                    ),
                    None => format!(
                        "«{name}» bezieht sich neu auf das Alter der eigenen Person ({} Jahre).",
                        fmt_age(years)
                    ),
                });
            }
        }
    }
}

/// #51: Pensionskasse und AHV-Rente beziehen sich auf die festen Meilensteine der Person.
fn pension_refs(v: &mut Value, births: &[(String, YearMonth)], notes: &mut Vec<String>) {
    for (i, base) in [(0usize, "/pension"), (1usize, "/partner")] {
        if i >= births.len() {
            continue;
        }
        let (pk, ahv) = person_milestones(i);
        for (key, id, what) in [("pk", pk, "Der Bezug der Pensionskasse"), ("ahv", ahv, "Die AHV-Rente")] {
            let Some(o) = v.pointer_mut(&format!("{base}/{key}")).and_then(Value::as_object_mut) else {
                continue;
            };
            let old = o.get("at").and_then(Value::as_str).unwrap_or("").to_string();
            if old != id {
                o.insert("at".into(), json!(id));
                let ms = person_milestone_name(if key == "pk" { PK_NAME } else { AHV_NAME }, i);
                notes.push(format!("{what} beginnt neu am Meilenstein «{ms}» (bisher «{old}»)."));
            }
        }
    }
}

/// #52: Säule 3a an einer Hypothek wird ein eigenes 3a-Konto der ersten Person.
fn pillar3a(v: &mut Value, births: &[(String, YearMonth)], notes: &mut Vec<String>) {
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
            // Fester Meilenstein der ersten Person, der in der Datei fehlt: Standardalter.
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
                // Ein leeres Konto ohne indirekte Amortisation trägt nichts bei (die Oberfläche legte es früher an).
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
            notes.push(format!(
                "Säule 3a von «{name}»: «Hypothek zurückzahlen» setzt neu eine Tranche mit indirekter Amortisation voraus; die Auszahlung geht ins freie Vermögen."
            ));
        }
        let payout = pa.get("payout").and_then(Value::as_str).unwrap_or("never").to_string();
        let mut entry = None;
        let mut text = format!("Die Säule 3a von «{name}» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»)");
        match (month_of(v, &payout), ahv_at) {
            (Some(m), Some(a)) if m >= a => {
                text += "; der Rest wird im Monat des AHV-Bezugs ausbezahlt";
            }
            (Some(m), Some(a)) => {
                let earliest = a.plus(-60);
                let date = m.max(earliest);
                if date > m {
                    text +=
                        &format!("; die Auszahlung ({m}) lag mehr als 5 Jahre vor dem AHV-Bezug und ist neu im {date}");
                } else {
                    text += &format!("; Auszahlung des ganzen Guthabens im {date}");
                }
                entry = Some(json!({ "date": date, "use": usage }));
            }
            _ => text += "; ohne Auszahlungszeitpunkt wird das Guthaben im Monat des AHV-Bezugs ausbezahlt",
        }
        notes.push(text + ".");
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
                notes.push(format!(
                    "Mehrere 3a-Guthaben sind zu einem Konto zusammengefasst (Zins des ersten Kontos: {:.2} %).",
                    acc.get("rate").and_then(Value::as_f64).unwrap_or(0.0) * 100.0
                ));
            }
            None => list.push(json!({
                "person": pid, "balance": balance, "rate": rate, "contribution": 0,
                "payouts": entry.map_or(vec![], |e| vec![e]), "use": usage
            })),
        }
    }
}

/// Planung mit veralteter Säule 3a an der Hypothek: übernommen wie beim Laden (für Aufrufe mit fertigen Planungen,
/// etwa aus Tests). Sonst unverändert.
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
    // Unmögliche Werte (etwa ein Alter von einer Milliarde Jahren) lassen sich nicht übernehmen; die Prüfung meldet sie.
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

    // #51/AK5, AK6: fehlender AHV-Bezug mit 65 vor dem Ende, «Pensionierung» heisst «Bezug Pensionskasse».
    #[test]
    fn issue_51_ak6_missing_milestones_and_names() {
        let mut v = plan();
        let notes = upgrade(&mut v);
        assert_eq!(ms(&v, "ahv")["years"], 65.0);
        assert_eq!(v["milestones"][2]["id"], "ahv", "vor dem Ende eingefügt");
        assert_eq!(ms(&v, "retire")["name"], "Bezug Pensionskasse");
        assert_eq!(ms(&v, "retire")["years"], 60, "Alter innerhalb der Grenzen bleibt");
        assert_eq!(v["pension"]["ahv"]["at"], "ahv");
        assert!(
            notes.contains(&"Der Meilenstein «AHV-Bezug» fehlte und steht neu auf 65 Jahren.".to_string()),
            "{notes:?}"
        );
        assert!(notes.contains(&"Der Meilenstein «Pensionierung» heisst neu «Bezug Pensionskasse».".to_string()));
        assert!(notes.contains(&"Die AHV-Rente beginnt neu am Meilenstein «AHV-Bezug» (bisher «never»).".to_string()));
        // Zweiter Lauf: nichts mehr zu tun
        assert!(upgrade(&mut v).is_empty());
    }

    // #51/AK6: Alter ausserhalb der Grenzen wird auf die nächste Grenze gesetzt, mit Hinweis.
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
            notes.contains(
                &"«Bezug Pensionskasse» lag bei 55 Jahren, ausserhalb der Grenzen 58 bis 65 Jahre; neu 58 Jahre."
                    .to_string()
            ),
            "{notes:?}"
        );
        assert!(
            notes.contains(
                &"«AHV-Bezug» lag bei 72 Jahren, ausserhalb der Grenzen 63 bis 70 Jahre; neu 70 Jahre.".to_string()
            ),
            "{notes:?}"
        );
        // #25: Grenzen aus den Einstellungen
        let mut v = plan();
        v["milestones"][1]["years"] = json!(55);
        v["limits"] = json!({ "pkMin": 50 });
        upgrade(&mut v);
        assert_eq!(ms(&v, "retire")["years"], 55);
    }

    // #51/AK1: zweite Person erhält eigene Meilensteine; Bezüge von Person 2 zeigen darauf.
    #[test]
    fn issue_51_second_person() {
        let mut v = plan();
        v["household"]["persons"] = json!([{ "id": "A", "birth": "1980-06" }, { "id": "B", "birth": "1982-01" }]);
        v["partner"] = json!({ "pk": { "at": "retire" }, "ahv": { "at": "ahv" } });
        upgrade(&mut v);
        assert_eq!(ms(&v, "retire2")["name"], "Bezug Pensionskasse Person 2");
        assert_eq!(ms(&v, "ahv2")["person"], "B");
        assert_eq!(
            (v["partner"]["pk"]["at"].as_str(), v["partner"]["ahv"]["at"].as_str()),
            (Some("retire2"), Some("ahv2"))
        );
        // Meilenstein der falschen Person: auf die eigene Person mit 65 Jahren
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

    // #52/AK13: Säule 3a an der Hypothek wird ein Konto der ersten Person mit Auszahlungsplan.
    #[test]
    fn issue_52_ak13_legacy_pillar3a() {
        // AHV-Bezug 2045-06 (65); frühester Bezug 2040-06.
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
        // Zu früh: auf den frühesten Zeitpunkt begrenzt
        let mut v = with_legacy(
            json!({ "balance": 1, "rate": 0, "payout": "2035-01", "use": "wealth" }),
            "indirect",
        );
        let notes = upgrade(&mut v);
        assert_eq!(v["pillar3a"][0]["payouts"][0]["date"], "2040-06");
        assert!(notes
            .iter()
            .any(|n| n.contains("lag mehr als 5 Jahre vor dem AHV-Bezug und ist neu im 06.2040")));
        // Nach dem AHV-Bezug oder «nie»: kein Plan-Eintrag, Rest im AHV-Bezug
        for payout in ["2050-01", "never"] {
            let mut v = with_legacy(
                json!({ "balance": 1, "rate": 0, "payout": payout, "use": "wealth" }),
                "indirect",
            );
            upgrade(&mut v);
            assert_eq!(v["pillar3a"][0]["payouts"], json!([]), "{payout}");
        }
        // «Hypothek zurückzahlen» ohne indirekte Amortisation: ins freie Vermögen, mit Hinweis
        let mut v = with_legacy(
            json!({ "balance": 5, "rate": 0, "payout": "retire", "use": "repay" }),
            "direct",
        );
        let notes = upgrade(&mut v);
        assert_eq!(v["pillar3a"][0]["use"], "wealth");
        assert!(notes
            .iter()
            .any(|n| n.contains("setzt neu eine Tranche mit indirekter Amortisation voraus")));
        // Leeres Konto ohne indirekte Amortisation entfällt still
        let mut v = with_legacy(
            json!({ "balance": 0, "rate": 0, "payout": "retire", "use": "repay" }),
            "direct",
        );
        assert!(upgrade(&mut v).iter().all(|n| !n.contains("3a")));
        assert!(v.get("pillar3a").is_none());
    }

    // #52/AK13: mehrere Immobilien mit 3a werden zu einem Konto zusammengefasst.
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

    // Unbrauchbare Daten führen nicht zum Absturz.
    #[test]
    fn odd_input() {
        assert!(upgrade(&mut json!(null)).is_empty());
        let mut v = json!({ "household": { "persons": [] } });
        assert!(upgrade(&mut v).is_empty());
    }

    // #124/AK6: Ein Alter genau auf der Grenze bleibt (kein Hinweis). Eine Säule 3a an der Hypothek ohne
    // Auszahlungszeitpunkt (oder mit unbekanntem Meilenstein) wird im AHV-Bezug ausbezahlt; «Hypothek zurückzahlen» setzt
    // eine indirekte Amortisation über 0 voraus. Eine Planung, die nicht parametrisch rechnet, bekommt keine Parameter.
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
                notes.contains(&"Die Säule 3a von «Haus» ist neu ein eigenes 3a-Konto (Abschnitt «Säule 3a»); ohne Auszahlungszeitpunkt wird das Guthaben im Monat des AHV-Bezugs ausbezahlt.".to_string()),
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

    // #154: Eine ungültige Zeitreihe (`returns.series` kein Text) macht die Planung erst nach dem Schritt `strategy`
    // lesbar. Die Anpassung der Wohnkosten (#53/#54) geschieht trotzdem im ersten Lauf; der zweite ändert nichts.
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
