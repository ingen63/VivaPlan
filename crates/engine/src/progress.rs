//! Fortschritt und Abbruch langer Berechnungen (Simulation, Szenarien, Optimierung).
//!
//! Es läuft immer höchstens eine Berechnung; die Oberfläche fragt den Stand ab und kann den
//! Abbruch verlangen. Gezählt werden Läufe (ein Lauf = ein Renditepfad durch die Planung).

use rayon::prelude::*;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

/// Meldung, an der die Oberfläche einen Abbruch erkennt.
pub const CANCELLED: &str = "Abgebrochen";

static ACTIVE: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);
static DONE: AtomicU64 = AtomicU64::new(0);
static TOTAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub active: bool,
    pub cancelled: bool,
    pub done: u64,
    /// Geschätzte Gesamtzahl der Läufe (bei der Optimierung nur grob).
    pub total: u64,
}

/// Beendet die Berechnung beim Verlassen des Gültigkeitsbereichs, auch bei Fehlern.
pub struct Job;

impl Drop for Job {
    fn drop(&mut self) {
        ACTIVE.store(false, Relaxed);
    }
}

/// Beginnt eine Berechnung mit `total` geschätzten Läufen und löscht einen früheren Abbruch.
pub fn start(total: u64) -> Job {
    CANCEL.store(false, Relaxed);
    DONE.store(0, Relaxed);
    TOTAL.store(total, Relaxed);
    ACTIVE.store(true, Relaxed);
    Job
}

pub fn set_total(n: u64) {
    TOTAL.store(n, Relaxed);
}

pub fn add_total(n: u64) {
    TOTAL.fetch_add(n, Relaxed);
}

pub fn tick(n: u64) {
    DONE.fetch_add(n, Relaxed);
}

/// Verlangt den Abbruch der laufenden Berechnung (wirkt nur, solange eine läuft).
pub fn cancel() {
    if ACTIVE.load(Relaxed) {
        CANCEL.store(true, Relaxed);
    }
}

pub fn cancelled() -> bool {
    CANCEL.load(Relaxed)
}

pub fn snapshot() -> Progress {
    Progress {
        active: ACTIVE.load(Relaxed),
        cancelled: CANCEL.load(Relaxed),
        done: DONE.load(Relaxed),
        total: TOTAL.load(Relaxed),
    }
}

/// `f(0..n)` parallel, in Blöcken: nach jedem Block wird der Fortschritt gezählt und ein Abbruch
/// geprüft. Nach einem Abbruch ist das Ergebnis unvollständig; der Aufrufer prüft [`cancelled`].
pub fn par_map<T, F>(n: usize, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync + Send,
{
    let chunk = (n / 100).max(rayon::current_num_threads() * 4).max(1);
    let mut out = Vec::with_capacity(n);
    let mut from = 0;
    while from < n {
        if cancelled() {
            break;
        }
        let to = (from + chunk).min(n);
        out.extend((from..to).into_par_iter().map(&f).collect::<Vec<_>>());
        tick((to - from) as u64);
        from = to;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // FEA-29: Zähler, Abbruch und Blockgrösse. Ein einziger Test, weil der Zustand global ist (nextest startet jeden Test
    // in einem eigenen Prozess).
    #[test]
    fn counters_cancel_and_blocks() {
        let job = start(10);
        assert_eq!(
            (
                snapshot().active,
                snapshot().cancelled,
                snapshot().done,
                snapshot().total
            ),
            (true, false, 0, 10)
        );
        set_total(20);
        add_total(5);
        tick(3);
        tick(4);
        assert_eq!((snapshot().done, snapshot().total), (7, 25));
        cancel();
        assert!(cancelled() && snapshot().cancelled);
        drop(job);
        assert!(!snapshot().active);

        // Ohne laufende Berechnung wirkt ein Abbruch nicht.
        let job = start(0);
        drop(job);
        cancel();
        assert!(!cancelled());

        // Vollständiger Lauf: Ergebnis in Reihenfolge, jeder Lauf genau einmal gezählt.
        let _job = start(0);
        let n = 1_000;
        let out = par_map(n, |i| i * 2);
        assert_eq!(out, (0..n).map(|i| i * 2).collect::<Vec<_>>());
        assert_eq!(snapshot().done, n as u64);

        // Abbruch im ersten Block: Der Block wird fertig gerechnet, danach endet die Schleife.
        // Blockgrösse = max(n / 100, Threads × 4, 1).
        let threads = rayon::current_num_threads();
        for n in [100usize, 100_000] {
            let _job = start(0);
            let out = par_map(n, |i| {
                if i == 0 {
                    cancel();
                }
                i
            });
            let chunk = (n / 100).max(threads * 4).max(1).min(n);
            assert_eq!(out.len(), chunk, "n = {n}");
            assert_eq!(snapshot().done, chunk as u64);
        }
    }
}
