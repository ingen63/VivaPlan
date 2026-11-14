//! Golden-Master-Tests: zehn feste Planungen, deren Ergebnisse in `tests/golden/*.json` liegen.
//! Jede Abweichung lässt den Test fehlschlagen. Gewollte Änderungen übernehmen mit
//! `GOLDEN_UPDATE=1 cargo nextest run -p vivaplan-engine -E 'test(/^golden::/)'` und die Differenz der JSON-Dateien prüfen.
//! Gespeichert sind die Monatswerte jedes Januars und Julis, der letzte Monat und je Zahlenfeld die Summe über alle
//! Monate (`recordSums`, FEA-64).

use std::path::{Path, PathBuf};

use engine::api;
use serde_json::{json, Value};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// 10 Jahre ab Januar 2030, Pensionierung am Start, keine Renditen, Teuerung und Steuern.
fn base() -> Value {
    crate::any_age(json!({
        "project": { "name": "Golden", "currency": "CHF" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "ahv", "name": "AHV", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2039-12" }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "never" }
        },
        "flows": { "income": 0, "spending": 5000 },
        "wealth": { "stocks": 300_000, "bonds": 200_000, "cash": 100_000, "stockShare": 0.6, "reserveMonths": 12 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.05, "bonds": 0.015, "inflation": 0.01, "saron": 0.01 } },
        "taxes": { "income": [[0, 0.0], [100_000, 0.1]], "wealth": [[0, 0.0], [2_000_000, 0.004]],
            "capital": [[0, 0.0], [1_000_000, 0.07]],
            "propertyGain": { "brackets": [], "reduction": 0.0 } }
    }))
}

fn long_horizon(mut v: Value, end: &str) -> Value {
    v["milestones"][3]["date"] = json!(end);
    v
}

fn house() -> Value {
    json!({ "id": "h", "name": "Haus", "status": "owned", "price": 800_000, "worth": 1_000_000, "sell": "never",
        "fixCosts": 800, "inflation": true,
        "mortgage": { "tranches": [
            { "name": "Fest", "amount": 300_000, "start": "2025-01", "after": "renew",
              "periods": [ { "type": "fixed", "term": 5, "rate": 0.015, "amort": 1500, "amortMode": "direct" },
                           { "type": "saron", "term": 5, "margin": 0.007 } ] },
            { "name": "Saron", "amount": 200_000, "start": "2026-01", "after": "repay",
              "periods": [ { "type": "saron", "term": 10, "margin": 0.006 } ] } ],
            "pillar3a": { "balance": 40_000, "rate": 0.01, "payout": "2034-06", "use": "repay" } } })
}

fn mc(source: &str, paths: usize) -> Value {
    let mut v = long_horizon(base(), "2059-12");
    v["returns"]["series"] = json!("library:Shiller CHF.csv");
    v["returns"]["source"] = json!(source);
    v["returns"][source] = if source == "bootstrap" {
        json!({ "paths": paths, "block": 12, "seed": 2026 })
    } else {
        json!({ "paths": paths, "mu": 0.06, "sigma": 0.16, "df": 5, "jumpLambda": 0.3, "jumpMu": -0.2, "jumpSigma": 0.1, "seed": 2026 })
    };
    v
}

fn cases() -> Vec<(&'static str, Value)> {
    let mut simple = base();
    simple["flows"]["income"] = json!({ "mode": "timed", "rows": [
        { "from": "start", "value": 6000 }, { "from": "retire", "value": 0 } ] });
    simple["milestones"][1]["date"] = json!("2033-01");

    let mut couple = long_horizon(base(), "2044-12");
    couple["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01" }, { "id": "B", "name": "B", "birth": "1967-06" } ]);
    couple["partner"] = json!({ "income": 2500,
        "pk": { "capital": 350_000, "at": "retire", "lumpsum": 0.3, "conversion": 0.05 },
        "ahv": { "monthly": 1800, "at": "ahv", "indexed": true } });
    couple["pension"]["ahv"] = json!({ "monthly": 2400, "at": "ahv", "indexed": true });
    couple["taxes"]["married"] = json!({ "income": [[0, 0.0], [150_000, 0.09]] });

    let mut pk_annuity = long_horizon(base(), "2054-12");
    pk_annuity["pension"]["pk"] = json!({ "capital": 900_000, "at": "retire", "lumpsum": 0, "conversion": 0.058,
        "interest": 0.0125 });
    pk_annuity["pension"]["ahv"] = json!({ "monthly": 2300, "at": "ahv", "indexed": true });

    let mut pk_lump = long_horizon(base(), "2054-12");
    pk_lump["pension"]["pk"] = json!({ "capital": 900_000, "at": "retire", "lumpsum": 1, "conversion": 0.058 });
    pk_lump["pension"]["ahv"] = json!({ "monthly": 2300, "at": "ahv", "indexed": true });

    let mut estate = long_horizon(base(), "2054-12");
    estate["realestate"] = json!({ "emergencyMonths": 6, "properties": [ house() ] });
    estate["flows"]["income"] = json!(12_000); // genug Einkommen, damit die Hypothek bei jeder Erneuerung tragbar bleibt

    let mut historical = long_horizon(base(), "2054-12");
    historical["returns"]["source"] = json!("historical");
    historical["returns"]["series"] = json!("library:Shiller CHF.csv");
    historical["returns"]["startDate"] = json!("1973-01");

    let mut scenarios = mc("bootstrap", 100);
    scenarios["pension"]["pk"] = json!({ "capital": 400_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 });
    scenarios["scenarios"] = json!({ "sweeps": [
        { "param": "wealth.stockShare", "from": 0.2, "to": 0.8, "step": 0.3 },
        { "param": "flows.spendingFactor", "from": 0.9, "to": 1.1, "step": 0.1 } ] });

    // Notverkauf: kaum Cash, hohe Ausgaben, Haus mit Hypothek
    let mut emergency = base();
    emergency["wealth"] = json!({ "stocks": 0, "bonds": 0, "cash": 25_000, "stockShare": 0.5, "reserveMonths": 0 });
    emergency["flows"]["spending"] = json!(4500);
    emergency["realestate"] = json!({ "emergencyMonths": 3, "properties": [ house() ] });

    vec![
        ("01-einzelperson", simple),
        ("02-ehepaar", couple),
        ("03-pk-rente", pk_annuity),
        ("04-pk-kapitalbezug", pk_lump),
        ("05-immobilie-hypothek", estate),
        ("06-historisch-1973", historical),
        ("07-bootstrap", mc("bootstrap", 200)),
        ("08-parametrisch", mc("parametric", 200)),
        ("09-szenarien", scenarios),
        ("10-notverkauf", emergency),
    ]
}

/// Ergebnis ohne Laufzeit; Monatswerte auf jeden Januar, jeden Juli und den letzten Monat verdichtet.
/// Neben jeder verdichteten Liste steht `recordSums`: je Zahlenfeld die Summe über alle Monate (FEA-64), damit auch
/// eine Änderung in einem nicht gespeicherten Monat auffällt.
fn snapshot(plan: &Value) -> Value {
    let out = api::run(plan, &library(), Some(&library()));
    assert!(out.error.is_none(), "{:?}", out.error);
    let mut v = serde_json::to_value(&out).unwrap();
    v.as_object_mut().unwrap().remove("elapsedMs");
    thin(&mut v);
    v
}

/// Summe je Zahlenfeld über alle Monate.
fn record_sums(records: &[Value]) -> Value {
    let mut sums = serde_json::Map::new();
    for r in records {
        for (k, x) in r.as_object().into_iter().flatten() {
            if let Some(x) = x.as_f64() {
                let s = sums.entry(k.clone()).or_insert(json!(0.0));
                *s = json!(s.as_f64().unwrap() + x);
            }
        }
    }
    Value::Object(sums)
}

fn thin(v: &mut Value) {
    match v {
        Value::Object(m) => {
            let mut sums = None;
            for (k, x) in m.iter_mut() {
                if k == "records" {
                    if let Value::Array(a) = x {
                        sums = Some(record_sums(a));
                        let n = a.len();
                        let keep: Vec<Value> = a
                            .iter()
                            .enumerate()
                            .filter(|(i, r)| {
                                *i + 1 == n
                                    || r["month"]
                                        .as_str()
                                        .is_some_and(|m| m.ends_with("-01") || m.ends_with("-07"))
                            })
                            .map(|(_, r)| r.clone())
                            .collect();
                        *a = keep;
                    }
                } else {
                    thin(x);
                }
            }
            if let Some(s) = sums {
                m.insert("recordSums".into(), s);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(thin),
        _ => {}
    }
}

/// Zahlen mit 1e-9 relativer Toleranz (und 1e-6 absolut), alles andere exakt.
fn diff(path: &str, a: &Value, b: &Value, out: &mut Vec<String>) {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            if (x - y).abs() > 1e-6_f64.max(1e-9 * x.abs().max(y.abs())) {
                out.push(format!("{path}: {x} statt {y}"));
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in x {
                match y.get(k) {
                    Some(w) => diff(&format!("{path}.{k}"), v, w, out),
                    None => out.push(format!("{path}.{k}: fehlt im Golden Master")),
                }
            }
            for k in y.keys().filter(|k| !x.contains_key(*k)) {
                out.push(format!("{path}.{k}: fehlt im Ergebnis"));
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                out.push(format!("{path}: {} Einträge statt {}", x.len(), y.len()));
            }
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                diff(&format!("{path}[{i}]"), v, w, out);
            }
        }
        _ if a == b => {}
        _ => out.push(format!("{path}: {a} statt {b}")),
    }
}

#[test]
fn golden_masters() {
    let update = std::env::var("GOLDEN_UPDATE").is_ok();
    std::fs::create_dir_all(dir()).unwrap();
    let mut failures = vec![];
    for (name, plan) in cases() {
        let now = snapshot(&plan);
        let file = dir().join(format!("{name}.json"));
        if update {
            std::fs::write(&file, serde_json::to_string_pretty(&now).unwrap() + "\n").unwrap();
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("{name}: Golden Master fehlt, mit GOLDEN_UPDATE=1 anlegen"));
        let expected: Value = serde_json::from_str(&text).unwrap();
        let mut d = vec![];
        diff("", &now, &expected, &mut d);
        if !d.is_empty() {
            failures.push(format!(
                "{name}: {} Abweichungen, z. B. {}",
                d.len(),
                d[..d.len().min(5)].join("; ")
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Die Golden Masters sind selbst plausibel: sie müssen alle Fälle abdecken und den jeweiligen Zweck zeigen.
#[test]
fn golden_cases_exercise_their_feature() {
    // FEA-64/AK2: das frisch gerechnete Ergebnis prüfen, nicht die eingecheckten Dateien
    // (sonst bestünde der Test auch, wenn GOLDEN_UPDATE=1 ein falsches Ergebnis übernommen hat).
    let all = cases();
    let get = |n: &str| -> Value { snapshot(&all.iter().find(|(name, _)| *name == n).unwrap().1) };
    let has_event = |v: &Value, kind: &str| {
        v["single"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == kind)
    };
    assert!(has_event(&get("03-pk-rente"), "retirement"));
    assert!(
        get("03-pk-rente")["single"]["summary"]["pensionFundMonthly"]
            .as_f64()
            .unwrap()
            > 4000.0
    );
    assert!(
        get("04-pk-kapitalbezug")["single"]["summary"]["lumpsumNet"]
            .as_f64()
            .unwrap()
            > 800_000.0
    );
    assert!(has_event(&get("05-immobilie-hypothek"), "pillar3aPayout"));
    assert!(has_event(&get("10-notverkauf"), "emergencySale"));
    assert_eq!(
        get("02-ehepaar")["single"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["kind"] == "retirement")
            .count(),
        2
    );
    assert_eq!(get("07-bootstrap")["batch"]["summary"]["runs"], 200);
    assert_eq!(get("08-parametrisch")["batch"]["summary"]["runs"], 200);
    assert_eq!(get("09-szenarien")["scenarios"]["results"].as_array().unwrap().len(), 9);
}

/// FEA-64/AK3: Eine Änderung in einem Monat, der nicht als Monatswert gespeichert ist (März), fällt über die
/// Summen auf.
#[test]
fn golden_detects_change_in_any_month() {
    let (_, plan) = cases().swap_remove(0);
    let out = api::run(&plan, &library(), Some(&library()));
    let mut full = serde_json::to_value(&out).unwrap();
    full.as_object_mut().unwrap().remove("elapsedMs");
    let mut changed = full.clone();
    let march = &mut changed["single"]["records"][2];
    assert_eq!(march["month"], "2030-03");
    march["cash"] = json!(march["cash"].as_f64().unwrap() + 1.0);
    thin(&mut full);
    thin(&mut changed);
    let mut d = vec![];
    diff("", &changed, &full, &mut d);
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].starts_with(".single.recordSums.cash:"), "{d:?}");
}
