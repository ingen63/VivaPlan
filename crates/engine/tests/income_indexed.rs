//! FEA-43: Erwerbseinkommen optional mit der Teuerung.
//!
//! Erfundene Planung: Geburt 1980-01, Start 2040-01, Bezug Pensionskasse und AHV-Bezug mit 65 (2045-01), keine
//! Renditen, keine Steuern; Teuerung wo vermerkt.

use engine::api;
use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base(inflation: f64) -> Value {
    json!({
        "project": { "name": "Teuerung" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 8000, "spending": 4000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 100_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": inflation, "saron": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn sim(v: &Value) -> Simulator {
    let plan: Plan = serde_json::from_value(v.clone()).expect("Planung lesbar");
    Simulator::new(&plan).expect("Planung gültig")
}

fn run(v: &Value) -> SimResult {
    let s = sim(v);
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records
        .iter()
        .find(|x| x.month == YearMonth::new(y, m))
        .expect("Monat im Zeitraum")
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: {a} statt {b}");
}

/// FEA-43/AK1, AK3: 8'000 pro Monat, Teuerung 2 %: mit «ja» nach 12 Monaten 8'160, nach 5 Jahren 8'832.64; mit «nein»
/// (auch ohne Angabe) bleiben es 8'000.
#[test]
fn fea43_ak1_ak3_income_follows_inflation() {
    let mut v = base(0.02);
    let r = run(&v);
    close(rec(&r, 2041, 1).earned, 8000.0, "ohne Angabe: nominal gleich");
    close(rec(&r, 2045, 1).earned, 8000.0, "ohne Angabe nach 5 Jahren");

    v["flows"]["incomeIndexed"] = json!(true);
    let r = run(&v);
    close(rec(&r, 2040, 1).earned, 8000.0, "Startmonat");
    close(rec(&r, 2041, 1).earned, 8160.0, "nach 12 Monaten");
    close(rec(&r, 2045, 1).earned, 8000.0 * 1.02f64.powi(5), "nach 5 Jahren");
    assert!((rec(&r, 2045, 1).earned - 8830.0).abs() < 5.0, "rund 8'830");

    v["flows"]["incomeIndexed"] = json!(false);
    close(rec(&run(&v), 2045, 1).earned, 8000.0, "«nein»");
}

/// FEA-43/AK2: Jede Zeile eines zeitabhängigen Einkommens gilt in Preisen des Starts.
#[test]
fn fea43_ak2_timed_rows_are_in_start_prices() {
    let mut v = base(0.02);
    v["flows"]["income"] = json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "2043-01", "value": 10_000 } ] });
    v["flows"]["incomeIndexed"] = json!(true);
    let r = run(&v);
    close(rec(&r, 2042, 1).earned, 8000.0 * 1.02f64.powi(2), "erste Zeile");
    close(
        rec(&r, 2043, 1).earned,
        10_000.0 * 1.02f64.powi(3),
        "zweite Zeile ab 2043, Teuerung seit dem Start",
    );
}

/// FEA-43/AK4: Auf jedem Pfad wächst das Einkommen mit dem Preisindex dieses Pfades, genau wie die Lebenshaltung.
#[test]
fn fea43_ak4_follows_the_price_index_of_any_path() {
    let mut v = base(0.0);
    v["flows"]["incomeIndexed"] = json!(true);
    let s = sim(&v);
    let mut path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    // Teuerung wie auf einem historischen oder zufälligen Pfad: jeden Monat anders, auch negativ.
    for (k, x) in path.inflation.iter_mut().enumerate() {
        *x = [0.03, -0.01, 0.08, 0.0, 0.015][k % 5];
    }
    let r = s.run(&path, true);
    let mut grew = false;
    for x in r.records.iter().filter(|x| x.month < YearMonth::new(2045, 1)) {
        assert!(
            (x.earned / 8000.0 - x.planned_spending / 4000.0).abs() < 1e-9,
            "{}: {} / {}",
            x.month,
            x.earned,
            x.planned_spending
        );
        grew |= x.earned > 8001.0;
    }
    assert!(grew, "Teuerung wirkt");
}

/// FEA-43: Die Auswahl gilt pro Person; die zweite Person hat ihre eigene.
#[test]
fn fea43_one_choice_per_person() {
    let mut v = base(0.02);
    v["household"]["persons"] =
        json!([ { "id": "A", "name": "A", "birth": "1980-01" }, { "id": "B", "name": "B", "birth": "1981-01" } ]);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "name": "Bezug PK 2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "name": "AHV-Bezug 2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "income": 3000, "incomeIndexed": true,
        "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv2" } });
    close(
        rec(&run(&v), 2041, 1).earned,
        8000.0 + 3060.0,
        "nur Person 2 mit der Teuerung",
    );
    v["flows"]["incomeIndexed"] = json!(true);
    v["partner"]["incomeIndexed"] = json!(false);
    close(
        rec(&run(&v), 2041, 1).earned,
        8160.0 + 3000.0,
        "nur Person 1 mit der Teuerung",
    );
}

/// FEA-43/AK5: Einkommenssteuer auf dem angepassten Einkommen; die Warnung nach dem Bezug der Pensionskasse nennt
/// den Betrag in Preisen des Starts.
#[test]
fn fea43_ak5_tax_and_warning_use_the_indexed_amount() {
    let mut v = base(0.02);
    v["taxes"]["income"] = json!([[0, 0.1], [10_000_000, 0.1]]);
    let plain = run(&v);
    v["flows"]["incomeIndexed"] = json!(true);
    let indexed = run(&v);
    let year = |r: &SimResult, y: i32| -> (f64, f64) {
        r.records
            .iter()
            .filter(|x| x.month.year() == y)
            .fold((0.0, 0.0), |a, x| (a.0 + x.earned, a.1 + x.tax_income))
    };
    let (e0, t0) = year(&plain, 2042);
    let (e1, t1) = year(&indexed, 2042);
    assert!(e1 > e0 + 1000.0, "{e1} / {e0}");
    // Die Steuer eines Jahres wird im Folgejahr bezahlt: 10 % des Einkommens 2041 bzw. 2042.
    let (p0, _) = year(&plain, 2041);
    let (p1, _) = year(&indexed, 2041);
    // Steuern auf Rappen gerundet
    assert!(
        (t0 - p0 * 0.1).abs() < 0.01,
        "Steuer ohne Teuerung: {t0} statt {}",
        p0 * 0.1
    );
    assert!(
        (t1 - p1 * 0.1).abs() < 0.01,
        "Steuer auf dem angepassten Einkommen: {t1} statt {}",
        p1 * 0.1
    );

    let warn = |v: &Value| -> Vec<String> {
        api::validate(v)
            .into_iter()
            .map(|i| i.message)
            .filter(|m| m.contains("Erwerbseinkommen"))
            .collect()
    };
    assert_eq!(warn(&v), vec!["Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (01.2045) noch 8'000 pro Monat in Preisen des Simulationsstarts.".to_string()]);
    v["flows"]["incomeIndexed"] = json!(false);
    assert_eq!(
        warn(&v),
        vec![
            "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (01.2045) noch 8'000 pro Monat.".to_string()
        ]
    );
}

/// FEA-43/AK1: «nein» wird nicht geschrieben (ältere Dateien bleiben gleich), «ja» schon.
#[test]
fn fea43_ak1_written_only_when_set() {
    let mut v = base(0.0);
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    assert!(!plan.flows.income_indexed);
    assert!(serde_json::to_value(&plan).unwrap()["flows"]
        .get("incomeIndexed")
        .is_none());
    v["flows"]["incomeIndexed"] = json!(true);
    let plan: Plan = serde_json::from_value(v).unwrap();
    assert_eq!(
        serde_json::to_value(&plan).unwrap()["flows"]["incomeIndexed"],
        json!(true)
    );
    // Neue Planungen: «ja» (Entscheid zu FEA-43)
    let neu: Plan = serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap();
    assert!(neu.flows.income_indexed);
}
