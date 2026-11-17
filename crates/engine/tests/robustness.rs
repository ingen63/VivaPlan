//! Robustheit: zufällig verfälschte Planungen dürfen den Rechenkern nicht zum Absturz bringen.
//! Erlaubt sind Prüfmeldungen und Fehlertexte, nicht aber Abstürze oder ungültige Zahlen.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use engine::api;
use engine::montecarlo::Rng;
use serde_json::{json, Value};

const MUTANTS: u64 = 600;

fn leaves(v: &Value, path: String, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                leaves(x, format!("{path}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                leaves(x, format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
    out.push(path);
}

/// Verfälscht 1 bis 4 Stellen der Planung. `destructive`: beliebige Stellen, auch Typwechsel und gelöschte Felder
/// (prüft das Lesen und die Eingabeprüfung). Sonst nur Zahlen durch Extremwerte (prüft die Rechnung selbst, #81).
fn mutate(rng: &mut Rng, plan: &mut Value, destructive: bool) {
    let mut paths = vec![];
    leaves(plan, String::new(), &mut paths);
    if !destructive {
        paths.retain(|p| plan.pointer(p).is_some_and(Value::is_number));
    }
    for _ in 0..1 + rng.below(4) {
        let p = &paths[rng.below(paths.len())];
        if p.is_empty() {
            continue;
        }
        let current = plan.pointer(p).cloned().unwrap_or(Value::Null);
        let numbers = [
            json!(0),
            json!(-1),
            json!(0.5),
            json!(1e15),
            json!(-1e15),
            json!(1e-300),
            json!(1e9),
            json!(-0.99),
        ];
        let strings = [
            json!("never"),
            json!("unbekannt"),
            json!("2026-13"),
            json!("1800-01"),
            json!("2300-01"),
            json!("start"),
            json!("end"),
            json!("afterSale"),
            json!("afterSale:xyz"),
            json!(""),
        ];
        let kind = if destructive { rng.below(10) } else { rng.below(7) };
        let replacement = match (kind, &current) {
            // meist typgerecht ersetzen, damit die Planung lesbar bleibt
            (0..=6, Value::Number(_)) => numbers[rng.below(numbers.len())].clone(),
            (0..=6, Value::String(_)) => strings[rng.below(strings.len())].clone(),
            (0..=6, Value::Array(a)) if !a.is_empty() => json!([]),
            (7, _) => [json!(null), json!([]), json!({}), json!(1), json!("x")][rng.below(5)].clone(),
            (8, _) | (9, _) => {
                // Feld löschen
                let (parent, key) = p.rsplit_once('/').unwrap();
                match plan.pointer_mut(parent) {
                    Some(Value::Object(o)) => {
                        o.remove(key);
                    }
                    Some(Value::Array(a)) => {
                        if let Ok(i) = key.parse::<usize>() {
                            if i < a.len() {
                                a.remove(i);
                            }
                        }
                    }
                    _ => {}
                }
                continue;
            }
            _ => continue,
        };
        if let Some(x) = plan.pointer_mut(p) {
            *x = replacement;
        }
    }
}

/// serde_json schreibt NaN als null; deshalb die Zahlen direkt prüfen (#129/AK4: auch das reale Endvermögen).
fn finite_result(r: &engine::sim::SimResult) -> bool {
    let s = &r.summary;
    [
        s.end_total,
        s.end_total_real,
        s.end_liquid,
        s.min_liquid.min(0.0),
        s.pension_fund_monthly,
        s.lumpsum_net,
        s.lumpsum_tax,
        s.taxes_total,
    ]
    .iter()
    .chain(s.total_at_retirement.iter())
    .all(|x| x.is_finite())
        && r.records.iter().all(|x| {
            [
                x.cash,
                x.stocks,
                x.bonds,
                x.liquid,
                x.property_equity,
                x.pension_fund,
                x.pillar3a,
                x.total,
                x.income,
                x.spending,
                x.taxes,
            ]
            .iter()
            .all(|v| v.is_finite())
        })
}

fn finite_run(out: &api::RunOut) -> bool {
    out.single.as_ref().is_none_or(finite_result)
        && out.batch.as_ref().is_none_or(|b| {
            b.details.iter().all(|(_, r)| finite_result(r))
                && b.summary
                    .percentiles
                    .iter()
                    .all(|p| p.end_total.is_finite() && p.end_total_real.is_some_and(f64::is_finite))
                && b.bands
                    .p50
                    .iter()
                    .chain(&b.bands.p5)
                    .chain(&b.bands.p95)
                    .all(|v| v.is_finite())
        })
        && out.scenarios.as_ref().is_none_or(|s| {
            s.results
                .iter()
                .all(|r| r.p5.is_finite() && r.p50.is_finite() && r.p95.is_finite())
        })
}

#[test]
fn mutated_plans_never_crash() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let mut base = api::example_plan();
    base["returns"]["bootstrap"] = json!({ "paths": 20, "block": 12, "seed": 1 });
    base["returns"]["parametric"] = json!({ "paths": 20, "mu": 0.07, "sigma": 0.15, "df": 6 });
    base["scenarios"]["sweeps"] = json!([{ "param": "wealth.stockShare", "from": 0.4, "to": 0.6, "step": 0.1 }]);
    let mut rng = Rng::new(99, 0);
    let sources = ["fixed", "historical", "rolling", "bootstrap", "parametric"];
    // gerechnet ohne Fehler: [nur Zahlen verfälscht, beliebig verfälscht]
    let mut ran = [0u64; 2];
    for n in 0..MUTANTS {
        let mut plan = base.clone();
        plan["returns"]["source"] = json!(sources[rng.below(sources.len())]);
        mutate(&mut rng, &mut plan, n % 2 == 1);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let issues = api::validate(&plan);
            let preview = api::preview(&plan, Path::new("/"), Some(&lib));
            let out = api::run(&plan, Path::new("/"), Some(&lib));
            (issues, preview, out)
        }));
        let Ok((_, preview, out)) = outcome else {
            panic!("Absturz bei Mutation {n}: {}", serde_json::to_string(&plan).unwrap());
        };
        if out.error.is_none() {
            ran[(n % 2) as usize] += 1;
        } else if std::env::var("ROBUST_DEBUG").is_ok() {
            eprintln!(
                "FEHLER {}",
                out.error.as_deref().unwrap_or("").lines().next().unwrap_or("")
            );
        }
        assert!(
            preview.result.as_ref().is_none_or(finite_result) && finite_run(&out),
            "ungültige Zahl bei Mutation {n}: {}",
            serde_json::to_string(&plan).unwrap()
        );
    }
    eprintln!("gerechnet: {ran:?} von je {}", MUTANTS / 2);
    // #81/AK2: Mindestanteile, gemessen am 14.11.2026 (nach #98): 97 und 69 von je 300.
    // Nur Zahlen verfälscht: Extremwerte weist die Eingabeprüfung oft ab (gewollt); ein Viertel gerechnet heisst, dass
    // die Rechnung mindestens 75 Planungen mit Extremwerten durchläuft. Sinkt der Anteil darunter, prüft der Test vor
    // allem noch die Eingabeprüfung und sollte andere Extremwerte wählen.
    // Beliebig verfälscht: Diese Hälfte prüft vor allem Lesen und Eingabeprüfung; ein Zehntel muss trotzdem rechnen,
    // damit auch Planungen mit falschen Verweisen und fehlenden Feldern bis in die Simulation kommen.
    let half = MUTANTS / 2;
    assert!(
        ran[0] * 4 >= half,
        "nur {} von {half} Mutationen mit Extremwerten gerechnet",
        ran[0]
    );
    assert!(
        ran[1] * 10 >= half,
        "nur {} von {half} beliebigen Mutationen gerechnet",
        ran[1]
    );
}
