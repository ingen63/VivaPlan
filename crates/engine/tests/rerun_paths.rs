//! #24: Läufe einer Variationsstufe werden aus den Pfadnummern nachgerechnet und stimmen mit der Szenarientabelle überein.

use engine::api;
use serde_json::{json, Value};
use std::path::Path;

fn plan() -> Value {
    crate::any_age(json!({
        "project": { "name": "Nachrechnen" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2032-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2045-12" }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "never" } },
        "flows": { "income": 6_000, "spending": 5_000 },
        "wealth": { "stocks": 400_000, "bonds": 100_000, "cash": 50_000, "stockShare": 0.6, "reserveMonths": 3 },
        "returns": { "source": "parametric", "fixed": { "stocks": 0.04, "bonds": 0.01, "inflation": 0.01 },
            "parametric": { "paths": 60, "mu": 0.06, "sigma": 0.15, "df": 0, "jumpLambda": 0, "jumpMu": 0, "jumpSigma": 0, "seed": 7 } },
        "taxes": { "income": [], "wealth": [], "capital": [] },
        "scenarios": { "sweeps": [ { "param": "wealth.stockShare", "from": 0.3, "to": 0.9, "step": 0.3 } ] }
    }))
}

fn picks_of(run: &api::RunOut, level: usize) -> Vec<(u32, usize, f64)> {
    let sc = run.scenarios.as_ref().expect("Szenarien");
    sc.results[level]
        .percentiles
        .iter()
        .filter(|p| matches!(p.percent, 0 | 1 | 5 | 50 | 95 | 99))
        .map(|p| (p.percent, p.index, p.end_total))
        .collect()
}

#[test]
fn rerun_matches_the_scenario_table_for_every_level() {
    let v = plan();
    let run = api::run(&v, Path::new("."), None);
    assert!(
        run.error.is_none(),
        "{:?} {:?}",
        run.error,
        run.issues.iter().map(|i| i.message.clone()).collect::<Vec<_>>()
    );
    let sc = run.scenarios.as_ref().unwrap();
    for (level, r) in sc.results.iter().enumerate() {
        let picks = picks_of(&run, level);
        assert_eq!(picks.len(), 6);
        let mut q = v.clone();
        q["scenarios"] = json!({ "sweeps": [], "select": [ { "param": sc.params[0], "value": r.values[0] } ] });
        let out = api::rerun_paths(
            &q,
            &picks.iter().map(|&(p, i, _)| (p, i)).collect::<Vec<_>>(),
            Path::new("."),
            None,
        );
        assert!(out.error.is_none(), "{:?}", out.error);
        assert_eq!(out.details.len(), 6);
        for ((percent, sim), (p, _, end)) in out.details.iter().zip(&picks) {
            assert_eq!(percent, p);
            assert!(
                (sim.summary.end_total - end).abs() < 1e-6 * end.abs().max(1.0),
                "Stufe {level}, Perzentil {p}: {} statt {end}",
                sim.summary.end_total
            );
        }
    }
}

#[test]
fn rerun_rejects_unknown_paths_and_bad_selection() {
    let mut q = plan();
    q["scenarios"] = json!({ "sweeps": [], "select": [ { "param": "wealth.stockShare", "value": 0.3 } ] });
    let out = api::rerun_paths(&q, &[(50, 9_999)], Path::new("."), None);
    assert!(out.error.unwrap().contains("Pfadnummer"));
    q["scenarios"]["select"] = json!([ { "param": "gibt.es.nicht", "value": 1 } ]);
    assert!(api::rerun_paths(&q, &[(50, 0)], Path::new("."), None).error.is_some());
    q["scenarios"]["select"] = json!([ { "value": 1 } ]);
    assert!(api::rerun_paths(&q, &[(50, 0)], Path::new("."), None).error.is_some());
}

// #132/AK3: Mit Student-t-Schocks und Kurssprüngen (die mit #132 schneller gezogen werden) ergeben gleiche Einstellungen
// dieselben Ergebnisse, und jeder Lauf der Detailansicht ist der Pfad, der das Perzentil bestimmt hat: Sein Endvermögen
// ist das des Perzentils, im Gesamtlauf wie in jeder Variationsstufe.
#[test]
fn issue_132_ak3_same_seed_same_results_and_detail_path() {
    let mut v = plan();
    v["returns"]["parametric"] = json!({ "paths": 200, "mu": 0.06, "sigma": 0.15, "df": 6, "jumpLambda": 2.0,
        "jumpMu": -0.2, "jumpSigma": 0.1, "seed": 11 });
    let ends = |run: &api::RunOut| -> Vec<(u32, usize, f64)> {
        (0..run.scenarios.as_ref().unwrap().results.len())
            .flat_map(|level| picks_of(run, level))
            .collect()
    };
    let a = api::run(&v, Path::new("."), None);
    assert!(a.error.is_none(), "{:?}", a.error);
    let b = api::run(&v, Path::new("."), None);
    assert_eq!(ends(&a), ends(&b), "gleicher Startwert, gleiche Ergebnisse");
    let mut other = v.clone();
    other["returns"]["parametric"]["seed"] = json!(12);
    assert_ne!(ends(&a), ends(&api::run(&other, Path::new("."), None)));

    let sc = a.scenarios.as_ref().unwrap();
    for (level, r) in sc.results.iter().enumerate() {
        let picks = picks_of(&a, level);
        let mut q = v.clone();
        q["scenarios"] = json!({ "sweeps": [], "select": [ { "param": sc.params[0], "value": r.values[0] } ] });
        let out = api::rerun_paths(
            &q,
            &picks.iter().map(|&(p, i, _)| (p, i)).collect::<Vec<_>>(),
            Path::new("."),
            None,
        );
        assert!(out.error.is_none(), "{:?}", out.error);
        for ((percent, sim), (p, _, end)) in out.details.iter().zip(&picks) {
            assert_eq!(percent, p);
            assert_eq!(sim.summary.end_total, *end, "Stufe {level}, Perzentil {p}");
        }
    }

    // ohne Variationen: Detailläufe des Gesamtlaufs
    v["scenarios"] = json!({ "sweeps": [] });
    let run = api::run(&v, Path::new("."), None);
    let batch = run.batch.as_ref().expect("mehrere Läufe");
    for (percent, sim) in &batch.details {
        let pc = batch
            .summary
            .percentiles
            .iter()
            .find(|p| p.percent == *percent)
            .expect("Perzentil");
        assert_eq!(sim.summary.end_total, pc.end_total, "Perzentil {percent}");
    }
    assert!(batch.details.len() >= 5);
}
