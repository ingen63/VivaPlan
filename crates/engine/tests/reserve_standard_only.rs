//! #18: The cash reserve (`wealth.reserveMonths`) only takes effect with the strategy «Standard».
//! All other strategies compute with a fixed reserve of 3 months.

use engine::batch;
use engine::model::Plan;
use engine::sim::Simulator;
use serde_json::{json, Value};

fn base(strategy: Value, reserve: f64) -> Value {
    json!({
        "project": { "name": "Reserve" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1970-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            // #54: Strategy and reserve apply from retirement; here it starts at the start.
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2036-12" }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "never" }
        },
        "flows": { "income": 60_000, "spending": 90_000 },
        "wealth": { "stocks": 400_000, "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": reserve },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.06, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] },
        "withdrawal": strategy
    })
}

fn totals(v: Value) -> Vec<f64> {
    let p: Plan = serde_json::from_value(v).expect("Planung gültig");
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true).records.iter().map(|r| r.total).collect()
}

#[test]
fn standard_uses_the_configured_reserve() {
    let std = json!({ "strategy": "standard" });
    assert_ne!(totals(base(std.clone(), 3.0)), totals(base(std, 6.0)));
}

#[test]
fn other_strategies_ignore_the_configured_reserve() {
    let others = [
        json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } }),
        json!({ "strategy": "guardrails", "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 } }),
        json!({ "strategy": "three_buckets" }),
    ];
    for w in others {
        let fixed = totals(base(w.clone(), 3.0));
        for reserve in [0.0, 6.0, 24.0] {
            assert_eq!(
                fixed,
                totals(base(w.clone(), reserve)),
                "{w}: Reserve {reserve} muss ohne Wirkung sein"
            );
        }
    }
}
