//! Fortschritt und Abbruch langer Berechnungen. Fortschritt und Abbruch gelten prozessweit: Jeder Test braucht einen
//! eigenen Prozess (nextest); parallel im selben Prozess würden sich die Tests gegenseitig abbrechen.

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

/// Wartet, bis die Berechnung läuft und gezählt hat, und verlangt dann den Abbruch.
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

    // normaler Lauf: Fortschritt ist danach nicht mehr aktiv, alle Läufe sind gezählt
    let r = api::run(&plan(400), Path::new("/"), Some(&lib));
    assert!(r.error.is_none(), "{:?}", r.error);
    let p = progress::snapshot();
    assert!(!p.active && !p.cancelled && p.done == 400 && p.total == 400, "{p:?}");

    // mit Variationen zählen die Läufe aller Szenarien dazu
    let mut sw = plan(100);
    sw["scenarios"]["sweeps"] = json!([{ "param": "flows.spendingFactor", "from": 0.9, "to": 1.1, "step": 0.1 }]);
    let r = api::run(&sw, Path::new("/"), Some(&lib));
    assert!(r.error.is_none(), "{:?}", r.error);
    let p = progress::snapshot();
    assert!(p.done == 400 && p.total == 400, "{p:?}");

    // Abbruch der Simulation
    let w = cancel_when_running();
    let t0 = Instant::now();
    let r = api::run(&plan(300_000), Path::new("/"), Some(&lib));
    let seen = w.join().unwrap();
    assert_eq!(r.error.as_deref(), Some("Abgebrochen"));
    assert!(seen.total == 300_000 && seen.done < seen.total);
    // #83: Dass abgebrochen wurde, zeigen Fehlertext und Zähler. Die Zeitgrenze fängt nur ein Hängen ab; der Abbruch
    // selbst dauert Millisekunden, 60 s halten auch auf einem 2-Kern-Runner unter Abdeckungsmessung.
    assert!(
        t0.elapsed() < Duration::from_secs(60),
        "Abbruch kam zu spät: {:?}",
        t0.elapsed()
    );
    assert!(!progress::snapshot().active);

    // #104: Der Abbruch gilt nur für die laufende Berechnung. Die Messung der Rechenzeit (`bench`) läuft ohne eigene
    // Berechnung; nach einem Abbruch muss sie trotzdem alle Läufe rechnen (vorher: 0,0006 µs je Lauf statt einiger µs).
    let b = api::bench(&plan(300), Path::new("/"), Some(&lib)).unwrap();
    assert!(b.us_per_run > 0.1, "{b:?}");

    // danach läuft die nächste Berechnung wieder normal (der Abbruch gilt nur einmal)
    let r = api::run(&plan(200), Path::new("/"), Some(&lib));
    assert!(r.error.is_none(), "{:?}", r.error);

    // Abbruch der Optimierung
    let w = cancel_when_running();
    let t0 = Instant::now();
    let req = json!({ "strategies": ["standard", "guardrails"], "paths": 20000, "checkPaths": 20000, "samples": 30, "seed": 1 });
    let e = api::optimize(&plan(200), &req, Path::new("/"), Some(&lib)).unwrap_err();
    w.join().unwrap();
    assert_eq!(e.to_string(), "Abgebrochen");
    // #83: wie oben, nur gegen ein Hängen.
    assert!(
        t0.elapsed() < Duration::from_secs(60),
        "Abbruch kam zu spät: {:?}",
        t0.elapsed()
    );
    assert!(!progress::snapshot().active);
}

/// #124/AK4: `cancel()` bricht jede laufende Berechnung ab, eine beendete nicht. Kurz, damit der Abbruch auch im
/// Mutationslauf geprüft wird (der Test oben ist dort zu lang). Wie dieser braucht er einen eigenen Prozess (nextest).
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
