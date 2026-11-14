//! FEA-67: Bilanz-Invarianten an zufälligen Planungen über alle Renditequellen, alle sechs Entnahmestrategien (dazu
//! 3-Topf mit Guardrails) und Haushalte mit Partner. Der Generator ist eine proptest-Strategie: Ein Fehlschlag wird auf
//! eine möglichst kleine Planung verkleinert und in `tests/proptest-regressions/proptest_plans.txt` gespeichert; dort
//! gespeicherte Fälle laufen bei jedem Testlauf zuerst.
//!
//! Fallzahl über `PROPTEST_CASES` (Standard 256, nächtlich 10'000). Monte-Carlo-Fälle rechnen 20 Pfade. Die Fälle mit
//! Zeitreihen und Monte Carlo stehen in einem eigenen Test, der im Profil `mutants` nicht läuft (`.config/nextest.toml`).

use std::path::{Path, PathBuf};

use engine::batch;
use engine::model::Plan;
use engine::sim::{SimResult, Simulator};
use engine::validate::{self, Severity};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, TestCaseError};
use rayon::prelude::*;
use serde_json::{json, Value};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Renditequellen; die ersten beiden rechnen einen Pfad ohne Zeitreihe bzw. mit einer, die übrigen mehrere Pfade.
const LIGHT_SOURCES: [&str; 2] = ["fixed", "historical"];
const HEAVY_SOURCES: [&str; 3] = ["rolling", "bootstrap", "parametric"];
/// Alle sechs Strategien; `three_buckets_guardrails` ist 3-Topf mit Ausgaben nach Guardrails (FEA-05).
const STRATEGIES: [&str; 7] = [
    "standard",
    "three_buckets",
    "three_buckets_guardrails",
    "guardrails",
    "fixed_rate",
    "vpw",
    "endowment",
];

/// Zweite Person: Geburtsjahr, Erwerbseinkommen, PK-Kapital, Kapitalanteil, AHV-Rente, verheiratet.
type Partner = (i32, f64, f64, f64, f64, bool);
/// Immobilie: Besitz (sonst Miete), Preis, Hypothek in Prozent des Preises, Verkaufsjahr (0 = nie).
type Property = (bool, f64, f64, i32);

#[derive(Clone, Debug)]
struct Params {
    source: &'static str,
    strategy: &'static str,
    birth: i32,
    start: i32,
    retire_age: i32,
    end_age: i32,
    income: f64,
    spending: f64,
    stocks: f64,
    bonds: f64,
    cash: f64,
    stock_share: f64,
    stock_return: f64,
    inflation: f64,
    pk: f64,
    lumpsum: f64,
    ahv: f64,
    partner: Option<Partner>,
    properties: Vec<Property>,
    hist_start: i32,
    seed: u64,
}

fn partner() -> impl Strategy<Value = Partner> {
    (
        1955..1980i32,
        0.0..8_000.0f64,
        0.0..800_000.0f64,
        0.0..=1.0f64,
        0.0..2_400.0f64,
        any::<bool>(),
    )
}

fn property() -> impl Strategy<Value = Property> {
    (
        any::<bool>(),
        200_000.0..1_200_000.0f64,
        0.0..0.6f64,
        prop_oneof![Just(0), 2030..2050i32],
    )
}

fn params(sources: &'static [&'static str]) -> impl Strategy<Value = Params> {
    let who = (1955..1980i32, 2026..2030i32, 58..70i32, 80..100i32);
    let flows = (0.0..15_000.0f64, 2_000.0..12_000.0f64);
    let wealth = (0.0..2_000_000.0f64, 0.0..500_000.0f64, 0.0..300_000.0f64, 0.0..=1.0f64);
    let returns = (-0.05..0.1f64, 0.0..0.04f64);
    let pension = (0.0..1_500_000.0f64, 0.0..=1.0f64, 0.0..2_500.0f64);
    let rest = (
        prop::option::of(partner()),
        prop::collection::vec(property(), 0..3),
        1950..1990i32,
        any::<u64>(),
    );
    (
        prop::sample::select(sources),
        prop::sample::select(&STRATEGIES[..]),
        who,
        flows,
        wealth,
        returns,
        pension,
        rest,
    )
        .prop_map(
            |(source, strategy, (birth, start, retire_age, end_age), (income, spending), w, r, p, rest)| Params {
                source,
                strategy,
                birth,
                start,
                retire_age,
                end_age,
                income,
                spending,
                stocks: w.0,
                bonds: w.1,
                cash: w.2,
                stock_share: w.3,
                stock_return: r.0,
                inflation: r.1,
                pk: p.0,
                lumpsum: p.1,
                ahv: p.2,
                partner: rest.0,
                properties: rest.1,
                hist_start: rest.2,
                seed: rest.3,
            },
        )
}

/// Planung aus den Parametern. Steuern einfach, aber vorhanden; Monte Carlo mit 20 Pfaden.
fn plan(p: &Params) -> Value {
    let retire = (p.birth + p.retire_age).max(p.start);
    let (strategy, spending) = match p.strategy {
        "three_buckets_guardrails" => ("three_buckets", "guardrails"),
        s => (s, "planned"),
    };
    let mut v = crate::any_age(json!({
        "project": { "name": "Zufall", "currency": "CHF" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": format!("{}-06", p.birth) } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": format!("{}-01", p.start) },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": format!("{retire}-01") },
            { "id": "ahv", "name": "AHV", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "age", "person": "A", "years": p.end_age }
        ],
        "pension": {
            "pk": { "capital": p.pk, "at": "retire", "lumpsum": p.lumpsum, "conversion": 0.05, "interest": 0.01 },
            "ahv": { "monthly": p.ahv, "at": "ahv", "indexed": true }
        },
        "flows": {
            "income": { "mode": "timed", "rows": [ { "from": "start", "value": p.income }, { "from": "retire", "value": 0 } ] },
            "spending": p.spending
        },
        "wealth": { "stocks": p.stocks, "bonds": p.bonds, "cash": p.cash, "stockShare": p.stock_share, "reserveMonths": 6 },
        "returns": {
            "source": p.source,
            "fixed": { "stocks": p.stock_return, "bonds": 0.01, "inflation": p.inflation, "saron": 0.01 },
            "strategy": { "stocks": "sp500", "bonds": "usa" },
            "startDate": format!("{}-01", p.hist_start),
            "bootstrap": { "paths": 20, "block": 12, "seed": p.seed % 1000 },
            "parametric": { "paths": 20, "mu": 0.06, "sigma": 0.15, "df": 6, "seed": p.seed % 1000 }
        },
        "withdrawal": {
            "strategy": strategy,
            "buckets": { "cashYears": 2, "bondYears": 6, "refillThreshold": 0, "reserveMonths": 3, "spending": spending },
            "fixedRate": { "rate": 0.04 },
            "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 },
            "vpw": { "horizonAge": 100, "realReturn": 0.02 },
            "endowment": { "rate": 0.04, "smoothing": 0.3 }
        },
        "taxes": { "multiplier": 1.5, "income": [[0, 0.0], [200_000, 0.15]], "wealth": [[0, 0.0], [3_000_000, 0.005]],
                   "capital": [[0, 0.0], [1_000_000, 0.06]],
                   "propertyGain": { "brackets": [[4000, 0.1], [1e9, 0.3]], "reduction": 0.0 } },
        "realestate": { "emergencyMonths": 6, "properties": [] }
    }));
    if let Some((birth, income, pk, lump, ahv, married)) = p.partner {
        v["household"]["persons"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "id": "B", "name": "B", "birth": format!("{birth}-03") }));
        v["household"]["married"] = json!(married);
        v["partner"] = json!({ "income": income,
            "pk": { "capital": pk, "at": "retire", "lumpsum": lump, "conversion": 0.05 },
            "ahv": { "monthly": ahv, "at": "ahv", "indexed": true } });
    }
    let props: Vec<Value> = p
        .properties
        .iter()
        .enumerate()
        .map(|(k, &(owned, price, loan, sell))| {
            if !owned {
                return json!({ "id": format!("p{k}"), "name": format!("Miete {k}"), "status": "rent", "from": "start",
                    "rent": price / 400.0, "price": 0 });
            }
            let mut o = json!({ "id": format!("p{k}"), "name": format!("Objekt {k}"), "status": "owned",
                "price": price, "worth": price * 1.2, "fixCosts": 300, "inflation": true,
                "sell": if sell == 0 { json!("never") } else { json!(format!("{sell}-06")) } });
            if loan > 0.0 {
                o["mortgage"] = json!({ "tranches": [ { "name": "T", "amount": price * loan, "start": "2020-01",
                    "after": "renew", "periods": [ { "type": "fixed", "term": 10, "rate": 0.015 } ] } ] });
            }
            o
        })
        .collect();
    v["realestate"]["properties"] = json!(props);
    v
}

/// Rechnet alle Pfade der Planung; `None`, wenn die Eingabeprüfung einen Fehler meldet.
fn run(v: &Value) -> Option<(Simulator, Vec<SimResult>)> {
    let p: Plan = serde_json::from_value(v.clone()).expect("erzeugte Planung ist lesbar");
    if validate::validate(&p).iter().any(|i| i.severity == Severity::Error) {
        return None;
    }
    let sim = Simulator::new(&p).expect("ohne Prüffehler muss der Simulator starten");
    let data = batch::load_series(&p, Path::new("/"), Some(&library())).expect("Zeitreihe ist lesbar");
    let paths = batch::path_source(&p, &sim, data.as_ref()).expect("Pfade entstehen");
    // Rollierende Zeitfenster haben einen Pfad je historischem Startmonat (Hunderte); davon 20 gleichmässig verteilte.
    let step = paths.len().div_ceil(20).max(1);
    let picked: Vec<usize> = (0..paths.len()).step_by(step).collect();
    let results = picked.par_iter().map(|&i| sim.run(&paths.get(i), true)).collect();
    Some((sim, results))
}

/// Die Bilanz-Invarianten aus `invariants.rs` für jeden Pfad.
fn check(v: &Value, repeat: bool) -> Result<(), TestCaseError> {
    let Some((sim, results)) = run(v) else {
        // Der Generator soll gültige Planungen erzeugen; seltene Prüffehler (zum Beispiel Tragbarkeit) überspringen.
        return Err(TestCaseError::reject("Prüffehler"));
    };
    prop_assert!(!results.is_empty());
    for res in &results {
        prop_assert_eq!(res.records.len(), sim.months());
        for r in &res.records {
            for (name, x) in [
                ("cash", r.cash),
                ("stocks", r.stocks),
                ("bonds", r.bonds),
                ("equity", r.property_equity),
                ("pk", r.pension_fund),
                ("3a", r.pillar3a),
                ("total", r.total),
                ("income", r.income),
                ("spending", r.spending),
                ("taxes", r.taxes),
            ] {
                prop_assert!(x.is_finite(), "{} ungültig im {}", name, r.month);
            }
            let tol = 1e-6 * r.total.abs().max(1.0);
            prop_assert!(
                (r.liquid - (r.cash + r.stocks + r.bonds)).abs() < tol,
                "Liquid im {}",
                r.month
            );
            prop_assert!(
                (r.total - (r.liquid + r.property_equity + r.pension_fund + r.pillar3a)).abs() < tol,
                "Total im {}",
                r.month
            );
            prop_assert!(
                r.pension_fund >= -1e-9 && r.pillar3a >= -1e-9 && r.taxes >= -1e-9,
                "negativ im {}",
                r.month
            );
            if r.liquid > 1.0 {
                prop_assert!(r.stocks >= -1e-6 && r.bonds >= -1e-6, "negative Anlage im {}", r.month);
            }
        }
        let taxes: f64 = res.records.iter().map(|r| r.taxes).sum();
        prop_assert!(
            (res.summary.taxes_total - taxes).abs() < 1e-6 * taxes.max(1.0),
            "Steuern total"
        );
        if let Some(d) = res.summary.depleted {
            prop_assert!(d >= sim.start && d <= sim.end);
        }
    }
    // Gleiche Eingaben ⇒ gleiche Ergebnisse. Nur bei einem Pfad: Die Wiederholbarkeit von Monte Carlo prüft
    // `qa_matrix.rs`, und die zweite Rechnung verdoppelte hier die Laufzeit.
    if !repeat {
        return Ok(());
    }
    let (_, again) = run(v).unwrap();
    for (a, b) in results.iter().zip(&again) {
        prop_assert_eq!(a.summary.end_total.to_bits(), b.summary.end_total.to_bits());
    }
    Ok(())
}

/// FEA-67/AK1: Ablage der verkleinerten Fehlschläge. Der Standard (`SourceParallel`) sucht `lib.rs` oder `main.rs` und
/// findet bei Integrationstests keine, deshalb der Pfad ausdrücklich.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/proptest_plans.txt"
);

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    })]

    /// FEA-67/AK1, AK2: fixe Renditen und historischer Verlauf (ein Pfad), alle Strategien, mit und ohne Partner.
    #[test]
    fn proptest_plans_keep_books_balanced(p in params(&LIGHT_SOURCES)) {
        check(&plan(&p), true)?;
    }

    /// FEA-67/AK2, AK3: rollierende Zeitfenster, Bootstrap und parametrisches Monte Carlo (20 Pfade); nicht im Profil
    /// `mutants`.
    #[test]
    fn proptest_plans_monte_carlo_keep_books_balanced(p in params(&HEAVY_SOURCES)) {
        check(&plan(&p), false)?;
    }
}
