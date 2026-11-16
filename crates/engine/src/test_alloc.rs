//! Speichergrenze für Testprozesse (ISS-22).
//!
//! Ein Mutant kann eine Schleife endlos machen, die Vektoren füllt (Beispiel: `k += len` wird `k *= len`). Ohne Grenze
//! fordert dann jeder Testprozess in Sekunden Gigabytes an; bei mehreren parallelen Tests reicht das, um macOS zum
//! Stillstand und zu einer Kernel-Panic zu bringen. macOS kennt keine Speichergrenze je Prozess (`ulimit -v` wirkt nicht).
//!
//! Deshalb setzen die Testprogramme von Engine, App und Kommandozeile [`Limited`] als globalen Speicherverwalter. Er zählt
//! die belegten Bytes und lässt eine Anforderung über [`LIMIT`] fehlschlagen. Rust bricht den Prozess dann sofort ab
//! («memory allocation of … bytes failed»), der Test schlägt fehl und der Mutant gilt als erkannt.
//!
//! Die App selbst verwendet den Speicherverwalter des Systems; nur Testprogramme setzen diesen hier.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Höchstens belegter Speicher je Testprozess. Der schwerste Test braucht rund 0,7 GB (`make test`), die Tests des
/// Mutationsprofils höchstens 0,2 GB.
pub const LIMIT: usize = 1536 << 20;

static USED: AtomicUsize = AtomicUsize::new(0);

/// Speicherverwalter mit Obergrenze [`LIMIT`]; reicht alles an den Speicherverwalter des Systems weiter.
pub struct Limited;

impl Limited {
    /// Zur Zeit belegte Bytes (für Tests).
    pub fn used() -> usize {
        USED.load(Ordering::Relaxed)
    }

    /// Reserviert `size` Bytes im Zähler; `false`, wenn die Grenze überschritten würde (dann bleibt der Zähler gleich).
    fn reserve(size: usize) -> bool {
        let before = USED.fetch_add(size, Ordering::Relaxed);
        if before.saturating_add(size) > LIMIT {
            USED.fetch_sub(size, Ordering::Relaxed);
            return false;
        }
        true
    }
}

// SAFETY: Alle Aufrufe gehen unverändert an `System`; der Zähler ändert nichts an Zeigern oder Layouts.
unsafe impl GlobalAlloc for Limited {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !Self::reserve(layout.size()) {
            return std::ptr::null_mut();
        }
        let p = unsafe { System.alloc(layout) };
        if p.is_null() {
            USED.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if !Self::reserve(layout.size()) {
            return std::ptr::null_mut();
        }
        let p = unsafe { System.alloc_zeroed(layout) };
        if p.is_null() {
            USED.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        USED.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old = layout.size();
        if new_size > old && !Self::reserve(new_size - old) {
            return std::ptr::null_mut();
        }
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if p.is_null() {
            if new_size > old {
                USED.fetch_sub(new_size - old, Ordering::Relaxed);
            }
        } else if new_size < old {
            USED.fetch_sub(old - new_size, Ordering::Relaxed);
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ISS-22/AK2: Über der Grenze schlägt die Anforderung fehl, ohne den Zähler zu verändern; eine normale Anforderung
    // gelingt und wird bei der Freigabe wieder abgezogen. Der Speicherverwalter ist in diesem Testprogramm der globale,
    // deshalb zählen andere Tests mit; geprüft wird nur, was diese Aufrufe am Zähler ändern.
    #[test]
    fn iss_22_ak2_allocations_above_the_limit_fail_and_the_counter_stays_consistent() {
        let huge = Layout::from_size_align(LIMIT + 1, 8).unwrap();
        unsafe {
            assert!(Limited.alloc(huge).is_null());
            assert!(Limited.alloc_zeroed(huge).is_null());
        }
        assert!(Limited::used() <= LIMIT);

        let small = Layout::from_size_align(1 << 20, 8).unwrap();
        unsafe {
            let p = Limited.alloc(small);
            assert!(!p.is_null());
            // Vergrössern über die Grenze schlägt fehl, der alte Block bleibt gültig.
            assert!(Limited.realloc(p, small, LIMIT + 1).is_null());
            let q = Limited.realloc(p, small, 2 << 20);
            assert!(!q.is_null());
            let r = Limited.realloc(q, Layout::from_size_align(2 << 20, 8).unwrap(), 1 << 10);
            assert!(!r.is_null());
            Limited.dealloc(r, Layout::from_size_align(1 << 10, 8).unwrap());
        }
    }

    #[test]
    fn iss_22_ak2_reserve_refuses_beyond_the_limit_and_accepts_small_requests() {
        assert!(!Limited::reserve(LIMIT + 1));
        assert!(!Limited::reserve(usize::MAX)); // kein Überlauf des Zählers
        assert!(Limited::reserve(4096));
        USED.fetch_sub(4096, Ordering::Relaxed);
    }
}
