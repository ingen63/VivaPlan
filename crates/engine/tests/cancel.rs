//! Progress and cancellation of long computations. Progress and cancellation apply process-wide: every test needs a
//! process of its own (nextest); in parallel in the same process the tests would cancel each other.

use std::path::Path;
use std::time::{Duration, Instant};

use engine::{api, progress};
use serde_json::json;

fn plan(paths: u64) -> serde_json::Value {
    let mut p = api::example_plan();
    p["returns"]["source"] = json!("bootstrap");
    p["returns"]["series"] = json!("library:Shiller CHF.csv");
    p["returns"]["bootstrap"]["paths"] = json!(paths);
    p["scenarios"]["sweeps"] = json!([]);
    p
}

/// Waits until the computation runs and has counted, then requests the cancellation.
fn cancel_when_running() -> std::thread::JoinHandle<progress::Progress> {
    std::thread::spawn(|| {
        let t0 = Instant::now();
        loop {
            let p = progress::snapshot();
            if p.active && p.done > 0 {
                progress::cancel();
                return progress::snapshot();
            }
            assert!(t0.elapsed() < Duration::from_secs(60), "Berechnung kam nicht in Gang");
            std::thread::sleep(Duration::from_millis(5));
        }
    })
}

#[test]
fn progress_counts_and_cancel_stops_simulation_and_optimization() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");

    // normal run: progress is no longer active afterwards, all runs are counted
    let r = api::run(&plan(400), Path::new("/"), Some(&lib));
    assert!(r.error.is_none(), "{:?}", r.error);
    let p = progress::snapshot();
    assert!(!p.active && !p.cancelled && p.done == 400 && p.total == 400, "{p:?}");

    // with variations the runs of all scenarios are counted too
    let mut sw = plan(100);
    sw["scenarios"]["sweeps"] = json!([{ "param": "flows.spendingFactor", "from": 0.9, "to": 1.1, "step": 0.1 }]);
    let r = api::run(&sw, Path::new("/"), Some(&lib));
    assert!(r.error.is_none(), "{:?}", r.error);
    let p = progress::snapshot();
    assert!(p.done == 400 && p.total == 400, "{p:?}");

    // Cancellation of the simulation
    let w = cancel_when_running();
    let t0 = Instant::now();
    // #139: 100'000 paths × 472 months = 47 million path-months, below the memory limit of 50 million.
    let r = api::run(&plan(100_000), Path::new("/"), Some(&lib));
    let seen = w.join().unwrap();
    assert_eq!(r.error.as_deref(), Some("Abgebrochen"));
    assert!(seen.total == 100_000 && seen.done < seen.total);
    // #83: Error text and counter show that it was cancelled. The time limit only catches a hang; the cancellation
    // itself takes milliseconds, 60 s also hold on a 2-core runner under coverage measurement.
    assert!(
        t0.elapsed() < Duration::from_secs(60),
        "Abbruch kam zu spät: {:?}",
        t0.elapsed()
    );
    assert!(!progress::snapshot().active);

    // #104: The cancellation only applies to the running computation. The measurement of the computing time (`bench`) runs without a
    // computation of its own; after a cancellation it must still compute all runs (previously: 0.0006 µs per run instead of a few µs).
    let b = api::bench(&plan(300), Path::new("/"), Some(&lib)).unwrap();
    assert!(b.us_per_run > 0.1, "{b:?}");

    // afterwards the next computation runs normally again (the cancellation applies only once)
    let r = api::run(&plan(200), Path::new("/"), Some(&lib));
    assert!(r.error.is_none(), "{:?}", r.error);

    // Cancellation of the optimization
    let w = cancel_when_running();
    let t0 = Instant::now();
    let req = json!({ "strategies": ["standard", "guardrails"], "paths": 20000, "checkPaths": 20000, "samples": 30, "seed": 1 });
    let e = api::optimize(&plan(200), &req, Path::new("/"), Some(&lib)).unwrap_err();
    w.join().unwrap();
    assert_eq!(e.to_string(), "Abgebrochen");
    // #83: as above, only against a hang.
    assert!(
        t0.elapsed() < Duration::from_secs(60),
        "Abbruch kam zu spät: {:?}",
        t0.elapsed()
    );
    assert!(!progress::snapshot().active);
}

/// #124/AK4: `cancel()` cancels every running computation, not a finished one. Short, so that the cancellation is also
/// checked in the mutation run (the test above is too long there). Like that one it needs a process of its own (nextest).
#[test]
fn issue_124_ak4_cancel_stops_every_running_job() {
    let running = progress::start_as(None, 10);
    let finished = progress::start_as(None, 10);
    let finished_id = finished.id();
    drop(finished);
    assert!(!progress::cancelled());
    progress::cancel();
    assert!(progress::cancelled());
    assert!(progress::snapshot_of(running.id()).cancelled);
    assert!(!progress::snapshot_of(finished_id).cancelled);
}

/// #135/AK2: Cancellation before the first block computes. A computation can only be cancelled once its job runs
/// (`api::run_as` creates it itself); hence the test's job and `batch::run_with_bands` are used directly here, exactly
/// as in `api::run_as` after the last cancellation check. `progress::par_map` cancels before the first block and
/// returns no runs; the bands stay without a panic (previously `len() - 1` on an empty column).
#[test]
fn issue_135_ak2_cancel_before_the_first_block_does_not_panic() {
    let mut v = api::example_plan();
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 400, "mu": 0.07, "sigma": 0.15, "df": 6, "seed": 1 });
    v["scenarios"]["sweeps"] = json!([]);
    let plan: engine::model::Plan = serde_json::from_value(v).unwrap();
    let sim = engine::sim::Simulator::new(&plan).unwrap();
    let paths = engine::batch::path_source(&plan, &sim, None).unwrap();
    assert_eq!(paths.len(), 400);

    let job = progress::start(paths.len() as u64);
    assert!(progress::cancel_job(job.id()));
    let (results, bands) = engine::batch::run_with_bands(&sim, &paths);
    assert!(results.is_empty(), "kein Block gerechnet");
    assert_eq!(bands.p50.len(), sim.months());
    assert!(bands.p50.iter().chain(&bands.min).all(|&x| x == 0.0));
    assert!(bands.real.as_ref().is_some_and(|r| r.p99.iter().all(|&x| x == 0.0)));
    let s = engine::batch::summarize(&results);
    assert_eq!((s.runs, s.percentiles.len()), (0, 0));
    assert_eq!(progress::snapshot_of(job.id()).done, 0);
}
