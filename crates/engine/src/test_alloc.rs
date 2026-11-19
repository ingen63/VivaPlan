//! Memory limit for test processes (#116).
//!
//! A mutant can make a loop that fills vectors endless (example: `k += len` becomes `k *= len`). Without a limit
//! every test process then requests gigabytes within seconds; with several parallel tests that is enough to bring macOS to a
//! standstill and a kernel panic. macOS has no memory limit per process (`ulimit -v` has no effect).
//!
//! Therefore the test programs of engine, app and command line set [`Limited`] as the global allocator. It counts
//! the allocated bytes and lets a request above [`LIMIT`] fail. Rust then aborts the process at once
//! («memory allocation of … bytes failed»), the test fails and the mutant counts as caught.
//!
//! The app itself uses the system allocator; only test programs set this one.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Maximum allocated memory per test process. The heaviest test needs about 0.7 GB (`make test`), the tests of the
/// mutation profile at most 0.2 GB.
pub const LIMIT: usize = 1536 << 20;

static USED: AtomicUsize = AtomicUsize::new(0);
/// #139: Peak of [`USED`] since the last [`Limited::reset_peak`].
static PEAK: AtomicUsize = AtomicUsize::new(0);

/// Allocator with the upper limit [`LIMIT`]; passes everything on to the system allocator.
pub struct Limited;

impl Limited {
    /// Currently allocated bytes (for tests).
    pub fn used() -> usize {
        USED.load(Ordering::Relaxed)
    }

    /// #139: Peak allocated memory since the last [`Limited::reset_peak`] (for memory tests).
    pub fn peak() -> usize {
        PEAK.load(Ordering::Relaxed)
    }

    /// #139: Resets the peak to the currently allocated memory.
    pub fn reset_peak() {
        PEAK.store(USED.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    /// Reserves `size` bytes in the counter; `false` if the limit would be exceeded (then the counter stays the same).
    fn reserve(size: usize) -> bool {
        let before = USED.fetch_add(size, Ordering::Relaxed);
        if before.saturating_add(size) > LIMIT {
            USED.fetch_sub(size, Ordering::Relaxed);
            return false;
        }
        PEAK.fetch_max(before + size, Ordering::Relaxed);
        true
    }
}

// SAFETY: All calls go unchanged to `System`; the counter changes nothing about pointers or layouts.
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

    // #116/AK2: Above the limit the request fails without changing the counter; a normal request
    // succeeds and is subtracted again on release. The allocator is the global one in this test program,
    // so other tests count along; only what these calls change in the counter is checked.
    #[test]
    fn issue_116_ak2_allocations_above_the_limit_fail_and_the_counter_stays_consistent() {
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
            // Growing beyond the limit fails, the old block stays valid.
            assert!(Limited.realloc(p, small, LIMIT + 1).is_null());
            let q = Limited.realloc(p, small, 2 << 20);
            assert!(!q.is_null());
            let r = Limited.realloc(q, Layout::from_size_align(2 << 20, 8).unwrap(), 1 << 10);
            assert!(!r.is_null());
            Limited.dealloc(r, Layout::from_size_align(1 << 10, 8).unwrap());
        }
    }

    #[test]
    fn issue_116_ak2_reserve_refuses_beyond_the_limit_and_accepts_small_requests() {
        assert!(!Limited::reserve(LIMIT + 1));
        assert!(!Limited::reserve(usize::MAX)); // no overflow of the counter
        assert!(Limited::reserve(4096));
        USED.fetch_sub(4096, Ordering::Relaxed);
    }
}
