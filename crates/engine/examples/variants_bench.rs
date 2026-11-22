//! #215/AK4: Measures the computing time of 8 variants on parametric return paths on one processor core.
//!
//! Run with `cargo run --release -p vivaplan-engine --example variants_bench [-- <paths> <repeats>]`
//! (default 1000 paths, best of 3 repeats). Two measurements, each with 8 variants of the example plan:
//!
//! - `variations`: a variation of the spending factor with 8 steps (`run` with `scenarios.sweeps`);
//! - `compare`: the optimization in comparison mode (plan as it is plus 7 strategies, 8 evaluations).
//!
//! Besides the time, the program prints a fingerprint of the results: it must be the same before and after a change
//! that only affects the computing time (bit-identical results).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::Instant;

use engine::api;
use serde_json::{json, Value};

fn plan(paths: usize) -> Value {
    let mut p = api::example_plan();
    p["returns"]["source"] = json!("parametric");
    p["returns"]["series"] = json!("library:Shiller CHF.csv");
    p["returns"]["parametric"]["paths"] = json!(paths);
    p
}

fn fingerprint(v: &impl serde::Serialize) -> u64 {
    let mut h = DefaultHasher::new();
    serde_json::to_string(v).unwrap().hash(&mut h);
    h.finish()
}

/// Best time of `repeats` runs of `f` in milliseconds and the fingerprint of the last result.
fn measure(repeats: usize, f: impl Fn() -> u64) -> (f64, u64) {
    let mut best = f64::INFINITY;
    let mut print = 0;
    for _ in 0..repeats {
        let t0 = Instant::now();
        print = f();
        best = best.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    (best, print)
}

fn main() {
    let args: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let paths = args.first().copied().unwrap_or(1000);
    let repeats = args.get(1).copied().unwrap_or(3).max(1);
    rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

    let mut variations = plan(paths);
    variations["scenarios"]["sweeps"] =
        json!([{ "param": "flows.spendingFactor", "from": 0.9, "to": 1.25, "step": 0.05 }]);
    let (ms, print) = measure(repeats, || {
        let out = api::run(&variations, Path::new("/"), Some(&lib));
        assert!(out.error.is_none(), "{:?}", out.error);
        let scenarios = out.scenarios.as_ref().unwrap();
        assert_eq!(scenarios.results.len(), 8);
        fingerprint(&(scenarios, out.batch.as_ref().unwrap()))
    });
    println!("variations: {paths} paths, 8 variants, 1 core: {ms:.0} ms, fingerprint {print:016x}");

    let compare = plan(paths);
    let request = json!({ "compare": true });
    let (ms, print) = measure(repeats, || {
        let r = api::optimize(&compare, &request, Path::new("/"), Some(&lib)).unwrap();
        assert_eq!(r.outcomes.len(), 7);
        fingerprint(&(&r.outcomes, &r.reference))
    });
    println!("compare:    {paths} paths, 8 evaluations, 1 core: {ms:.0} ms, fingerprint {print:016x}");
}
