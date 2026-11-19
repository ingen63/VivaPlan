//! Progress and cancellation of long computations (simulation, scenarios, optimization).
//!
//! Every computation is a [`Job`] with its own id, its own cancellation and its own counter (#107). The job of the
//! running computation is attached to the thread that called it; [`par_map`], [`tick`] and [`cancelled`] act on it.
//! Without a job (for example when measuring the computing time) there is neither cancellation nor a counter (#104).
//! Runs are counted (one run = one return path through the plan).

use rayon::prelude::*;
use serde::Serialize;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

/// Message by which the user interface recognizes a cancellation.
pub const CANCELLED: &str = "Cancelled";

/// Id of a computation. The user interface can choose it itself and pass it at the start; otherwise the engine assigns one.
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

/// Ids not assigned by a person or the user interface count up from 1; the user interface uses large numbers.
static NEXT: AtomicU64 = AtomicU64::new(1);
/// Running jobs and the one finished last (for the last query of the status).
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
    /// Estimated total number of runs (only rough for the optimization).
    pub total: u64,
}

/// Finishes the computation when leaving the scope, also on errors. The job belongs to the thread that
/// started it (hence not `Send`).
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

/// Starts a computation with `total` estimated runs and its own id.
pub fn start(total: u64) -> Job {
    start_as(None, total)
}

/// Like [`start`], with an id chosen by the user interface. A still running computation with the same id
/// stays untouched; the new one then gets an id of its own.
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

/// Requests the cancellation of all running computations (only has an effect while one is running).
pub fn cancel() {
    for j in jobs().iter().filter(|j| j.active.load(Relaxed)) {
        j.cancel.store(true, Relaxed);
    }
}

/// Requests the cancellation of the computation with this id; other computations continue. `false` if no
/// computation with this id is running.
pub fn cancel_job(id: JobId) -> bool {
    match jobs().iter().find(|j| j.id == id && j.active.load(Relaxed)) {
        Some(j) => {
            j.cancel.store(true, Relaxed);
            true
        }
        None => false,
    }
}

/// Has a cancellation been requested for this thread's computation? Never without a job or after its end (#104).
pub fn cancelled() -> bool {
    current().is_some_and(|j| j.active.load(Relaxed) && j.cancel.load(Relaxed))
}

/// Status of the computation started last (running or finished).
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

/// Status of the computation with this id; unknown or not yet started: not active, zero.
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

/// `f(0..n)` in parallel, in blocks: after every block the progress is counted and a cancellation
/// checked. After a cancellation the result is incomplete; the caller checks [`cancelled`].
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

    // #107/AK5: Every test uses its own ids and reads only the status of its computation (`snapshot_of`); the
    // cancellation applies per id (`cancel_job`) and the thread job is thread-local. So the tests do not interfere, even
    // when they run in the same process. The process-wide calls `cancel()` (all computations) and `snapshot()`
    // (started last) are checked by `crates/engine/tests/cancel.rs` with a real computation.

    // #40: Counter and total of a computation.
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

    // #104: After the end of the computation a cancellation has no effect; a computation without a job runs to the end.
    #[test]
    fn a_cancel_does_not_outlive_its_job() {
        let job = start_as(Some(3002), 10);
        let id = job.id();
        assert!(cancel_job(id));
        assert!(cancelled() && snapshot_of(id).cancelled);
        drop(job);
        // The flag no longer has an effect (the status can only be queried until the next computation starts).
        assert!(!cancelled());
        assert_eq!(par_map(1_000, |i| i).len(), 1_000);
        assert!(!cancel_job(id), "beendet: kein Abbruch mehr möglich");
    }

    // Complete run: result in order, every run counted exactly once.
    #[test]
    fn par_map_keeps_order_and_counts_every_run_once() {
        let job = start_as(Some(3003), 0);
        let n = 1_000;
        let out = par_map(n, |i| i * 2);
        assert_eq!(out, (0..n).map(|i| i * 2).collect::<Vec<_>>());
        assert_eq!(snapshot_of(job.id()).done, n as u64);
    }

    // Cancellation in the first block: the block is computed to the end, then the loop ends.
    // Block size = max(n / 100, threads × 4, 1).
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

    // #107/AK2, #107/AK3: Two simultaneous computations do not affect each other. This test only needs ids
    // and `cancel_job`, never the global `cancel()`.
    #[test]
    fn two_jobs_run_side_by_side_and_cancel_one() {
        use std::sync::mpsc::channel;
        use std::time::Duration;
        let (a_ready_tx, a_ready_rx) = channel();
        let (b_ready_tx, b_ready_rx) = channel();
        let (go_tx, go_rx) = channel::<()>();
        let a = std::thread::spawn(move || {
            let _job = start_as(Some(1001), 100);
            par_map(10, |i| i); // counts along before the cancellation comes
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
        // B is untouched and computes completely
        assert!(!snapshot_of(1002).cancelled);
        go_tx.send(()).unwrap();
        let (len, b_cancelled, seen_b) = b.join().unwrap();
        assert_eq!((len, b_cancelled), (5_000, false));
        assert_eq!(
            (seen_b.id, seen_b.cancelled, seen_b.done, seen_b.total),
            (1002, false, 5_000, 5_000)
        );
        // After the end cancel_job has no effect any more
        assert!(!cancel_job(1001) && !cancel_job(1002));
    }

    // #107/AK1: An id already assigned is not used twice; nested jobs restore the outer one.
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
        tick(5); // without a job: no effect (and no crash)
        assert!(!cancelled());
    }
}
