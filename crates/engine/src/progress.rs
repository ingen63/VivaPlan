//! Fortschritt und Abbruch langer Berechnungen (Simulation, Szenarien, Optimierung).
//!
//! Jede Berechnung ist ein [`Job`] mit eigener Kennung, eigenem Abbruch und eigenem Zähler (#107). Der Job der
//! laufenden Berechnung hängt am Thread, der sie aufgerufen hat; [`par_map`], [`tick`] und [`cancelled`] wirken auf ihn.
//! Ohne Job (zum Beispiel bei der Messung der Rechenzeit) gibt es weder Abbruch noch Zähler (#104). Gezählt werden
//! Läufe (ein Lauf = ein Renditepfad durch die Planung).

use rayon::prelude::*;
use serde::Serialize;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

/// Meldung, an der die Oberfläche einen Abbruch erkennt.
pub const CANCELLED: &str = "Abgebrochen";

/// Kennung einer Berechnung. Die Oberfläche kann sie selbst wählen und beim Start mitgeben; sonst vergibt die Engine eine.
pub type JobId = u64;

struct State {
    id: JobId,
    active: AtomicBool,
    cancel: AtomicBool,
    done: AtomicU64,
    total: AtomicU64,
}

impl State {
    fn progress(&self) -> Progress {
        Progress {
            id: self.id,
            active: self.active.load(Relaxed),
            cancelled: self.cancel.load(Relaxed),
            done: self.done.load(Relaxed),
            total: self.total.load(Relaxed),
        }
    }
}

/// Vom Menschen oder der Oberfläche nicht vergebene Kennungen zählen ab 1 hoch; die Oberfläche nimmt grosse Zahlen.
static NEXT: AtomicU64 = AtomicU64::new(1);
/// Laufende Jobs und der zuletzt beendete (für die letzte Abfrage des Stands).
static JOBS: Mutex<Vec<Arc<State>>> = Mutex::new(Vec::new());

thread_local! {
    static CURRENT: RefCell<Option<Arc<State>>> = const { RefCell::new(None) };
}

fn jobs() -> std::sync::MutexGuard<'static, Vec<Arc<State>>> {
    JOBS.lock().unwrap_or_else(|e| e.into_inner())
}

fn current() -> Option<Arc<State>> {
    CURRENT.with(|c| c.borrow().clone())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub id: JobId,
    pub active: bool,
    pub cancelled: bool,
    pub done: u64,
    /// Geschätzte Gesamtzahl der Läufe (bei der Optimierung nur grob).
    pub total: u64,
}

/// Beendet die Berechnung beim Verlassen des Gültigkeitsbereichs, auch bei Fehlern. Der Job gehört dem Thread, der ihn
/// gestartet hat (deshalb nicht `Send`).
pub struct Job {
    state: Arc<State>,
    previous: Option<Arc<State>>,
    _same_thread: PhantomData<*const ()>,
}

impl Job {
    pub fn id(&self) -> JobId {
        self.state.id
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.state.active.store(false, Relaxed);
        let previous = self.previous.take();
        CURRENT.with(|c| *c.borrow_mut() = previous);
    }
}

/// Beginnt eine Berechnung mit `total` geschätzten Läufen und eigener Kennung.
pub fn start(total: u64) -> Job {
    start_as(None, total)
}

/// Wie [`start`], mit einer von der Oberfläche gewählten Kennung. Eine noch laufende Berechnung mit derselben Kennung
/// bleibt unberührt, die neue bekommt dann eine eigene.
pub fn start_as(id: Option<JobId>, total: u64) -> Job {
    let mut list = jobs();
    list.retain(|j| j.active.load(Relaxed));
    let id = match id {
        Some(id) if !list.iter().any(|j| j.id == id) => id,
        _ => NEXT.fetch_add(1, Relaxed),
    };
    let state = Arc::new(State {
        id,
        active: AtomicBool::new(true),
        cancel: AtomicBool::new(false),
        done: AtomicU64::new(0),
        total: AtomicU64::new(total),
    });
    list.push(state.clone());
    drop(list);
    let previous = CURRENT.with(|c| c.borrow_mut().replace(state.clone()));
    Job {
        state,
        previous,
        _same_thread: PhantomData,
    }
}

pub fn set_total(n: u64) {
    if let Some(j) = current() {
        j.total.store(n, Relaxed);
    }
}

pub fn add_total(n: u64) {
    if let Some(j) = current() {
        j.total.fetch_add(n, Relaxed);
    }
}

pub fn tick(n: u64) {
    if let Some(j) = current() {
        j.done.fetch_add(n, Relaxed);
    }
}

/// Verlangt den Abbruch aller laufenden Berechnungen (wirkt nur, solange eine läuft).
pub fn cancel() {
    for j in jobs().iter().filter(|j| j.active.load(Relaxed)) {
        j.cancel.store(true, Relaxed);
    }
}

/// Verlangt den Abbruch der Berechnung mit dieser Kennung; andere Berechnungen laufen weiter. `false`, wenn keine
/// Berechnung mit dieser Kennung läuft.
pub fn cancel_job(id: JobId) -> bool {
    match jobs().iter().find(|j| j.id == id && j.active.load(Relaxed)) {
        Some(j) => {
            j.cancel.store(true, Relaxed);
            true
        }
        None => false,
    }
}

/// Ist für die Berechnung dieses Threads ein Abbruch verlangt? Ohne Job und nach dessen Ende nie (#104).
pub fn cancelled() -> bool {
    current().is_some_and(|j| j.active.load(Relaxed) && j.cancel.load(Relaxed))
}

/// Stand der zuletzt gestarteten Berechnung (laufend oder beendet).
pub fn snapshot() -> Progress {
    jobs().last().map_or(
        Progress {
            id: 0,
            active: false,
            cancelled: false,
            done: 0,
            total: 0,
        },
        |j| j.progress(),
    )
}

/// Stand der Berechnung mit dieser Kennung; unbekannt oder noch nicht gestartet: nicht aktiv, null.
pub fn snapshot_of(id: JobId) -> Progress {
    jobs().iter().find(|j| j.id == id).map_or(
        Progress {
            id,
            active: false,
            cancelled: false,
            done: 0,
            total: 0,
        },
        |j| j.progress(),
    )
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

    // #107/AK5: Jeder Test benutzt eigene Kennungen und liest nur den Stand seiner Berechnung (`snapshot_of`); der
    // Abbruch gilt je Kennung (`cancel_job`) und der Thread-Job ist threadlokal. Die Tests stören sich daher nicht, auch
    // wenn sie im selben Prozess laufen. Die prozessweiten Aufrufe `cancel()` (alle Berechnungen) und `snapshot()`
    // (zuletzt gestartete) prüft `crates/engine/tests/cancel.rs` mit einer echten Berechnung.

    // #40: Zähler und Gesamtzahl einer Berechnung.
    #[test]
    fn counters_follow_the_job() {
        let job = start_as(Some(3001), 10);
        let id = job.id();
        let p = snapshot_of(id);
        assert_eq!((p.active, p.cancelled, p.done, p.total), (true, false, 0, 10));
        set_total(20);
        add_total(5);
        tick(3);
        tick(4);
        let p = snapshot_of(id);
        assert_eq!((p.done, p.total), (7, 25));
        drop(job);
        assert!(!snapshot_of(id).active);
    }

    // #104: Nach dem Ende der Berechnung wirkt ein Abbruch nicht mehr; eine Rechnung ohne Job läuft ganz durch.
    #[test]
    fn a_cancel_does_not_outlive_its_job() {
        let job = start_as(Some(3002), 10);
        let id = job.id();
        assert!(cancel_job(id));
        assert!(cancelled() && snapshot_of(id).cancelled);
        drop(job);
        // Die Marke wirkt nicht mehr (der Stand bleibt nur bis zum Start der nächsten Berechnung abfragbar).
        assert!(!cancelled());
        assert_eq!(par_map(1_000, |i| i).len(), 1_000);
        assert!(!cancel_job(id), "beendet: kein Abbruch mehr möglich");
    }

    // Vollständiger Lauf: Ergebnis in Reihenfolge, jeder Lauf genau einmal gezählt.
    #[test]
    fn par_map_keeps_order_and_counts_every_run_once() {
        let job = start_as(Some(3003), 0);
        let n = 1_000;
        let out = par_map(n, |i| i * 2);
        assert_eq!(out, (0..n).map(|i| i * 2).collect::<Vec<_>>());
        assert_eq!(snapshot_of(job.id()).done, n as u64);
    }

    // Abbruch im ersten Block: Der Block wird fertig gerechnet, danach endet die Schleife.
    // Blockgrösse = max(n / 100, Threads × 4, 1).
    #[test]
    fn a_cancel_in_the_first_block_ends_after_that_block() {
        let threads = rayon::current_num_threads();
        for (k, n) in [100usize, 100_000].into_iter().enumerate() {
            let job = start_as(Some(3010 + k as u64), 0);
            let id = job.id();
            let out = par_map(n, |i| {
                if i == 0 {
                    cancel_job(id);
                }
                i
            });
            let chunk = (n / 100).max(threads * 4).max(1).min(n);
            assert_eq!(out.len(), chunk, "n = {n}");
            assert_eq!(snapshot_of(id).done, chunk as u64);
        }
    }

    // #107/AK2, #107/AK3: Zwei Berechnungen gleichzeitig beeinflussen sich nicht. Dieser Test braucht nur Kennungen
    // und `cancel_job`, nie das globale `cancel()`.
    #[test]
    fn two_jobs_run_side_by_side_and_cancel_one() {
        use std::sync::mpsc::channel;
        use std::time::Duration;
        let (a_ready_tx, a_ready_rx) = channel();
        let (b_ready_tx, b_ready_rx) = channel();
        let (go_tx, go_rx) = channel::<()>();
        let a = std::thread::spawn(move || {
            let _job = start_as(Some(1001), 100);
            par_map(10, |i| i); // zählt mit, bevor der Abbruch kommt
            a_ready_tx.send(()).unwrap();
            while !cancelled() {
                par_map(1_000, |i| i);
                std::thread::sleep(Duration::from_millis(1));
            }
            snapshot_of(1001)
        });
        let b = std::thread::spawn(move || {
            let _job = start_as(Some(1002), 5_000);
            b_ready_tx.send(()).unwrap();
            go_rx.recv().unwrap();
            let out = par_map(5_000, |i| i);
            (out.len(), cancelled(), snapshot_of(1002))
        });
        a_ready_rx.recv().unwrap();
        b_ready_rx.recv().unwrap();
        assert!(!cancel_job(9_999), "unbekannte Kennung");
        assert!(cancel_job(1001));
        let seen_a = a.join().unwrap();
        assert!(
            (seen_a.id, seen_a.active, seen_a.cancelled) == (1001, true, true),
            "{seen_a:?}"
        );
        assert!(seen_a.done > 0 && seen_a.total == 100);
        // B ist unberührt und rechnet vollständig
        assert!(!snapshot_of(1002).cancelled);
        go_tx.send(()).unwrap();
        let (len, b_cancelled, seen_b) = b.join().unwrap();
        assert_eq!((len, b_cancelled), (5_000, false));
        assert_eq!(
            (seen_b.id, seen_b.cancelled, seen_b.done, seen_b.total),
            (1002, false, 5_000, 5_000)
        );
        // Nach dem Ende wirkt cancel_job nicht mehr
        assert!(!cancel_job(1001) && !cancel_job(1002));
    }

    // #107/AK1: Eine bereits vergebene Kennung wird nicht doppelt benutzt; verschachtelte Jobs stellen den äusseren wieder her.
    #[test]
    fn ids_are_unique_and_jobs_nest() {
        let outer = start_as(Some(2001), 10);
        let inner = start_as(Some(2001), 20);
        assert_ne!(outer.id(), inner.id());
        tick(1);
        assert_eq!(snapshot_of(inner.id()).done, 1);
        assert_eq!(snapshot_of(2001).done, 0);
        drop(inner);
        tick(2);
        assert_eq!(snapshot_of(2001).done, 2);
        drop(outer);
        tick(5); // ohne Job: wirkungslos (und kein Absturz)
        assert!(!cancelled());
    }
}
