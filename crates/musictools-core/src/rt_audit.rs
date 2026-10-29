//! Realtime allocation auditing — the suite's replacement for truce's
//! `rt-paranoid`.
//!
//! The `alloc-audit` cargo feature (wired in by each plugin's `rt-audit`
//! feature, which workspace test runs enable via `--all-features`) installs
//! the counting global allocator in every binary that links this crate. Wrap
//! the code under test in [`audit`] and assert the returned count is zero.
//! The `warn_debug` mode counts and logs instead of aborting, so a regression
//! fails the test rather than killing the process.

/// The counting allocator, applied automatically to any binary that enables
/// the feature.
#[global_allocator]
static AUDIT: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;

/// Run `f` under the allocator's audit scope, returning its value plus the
/// number of allocations made while it ran.
#[must_use]
pub fn audit<R>(f: impl FnOnce() -> R) -> (R, u64) {
    let before = assert_no_alloc::violation_count();
    let result = assert_no_alloc::assert_no_alloc(f);
    let after = assert_no_alloc::violation_count();
    (result, u64::from(after.saturating_sub(before)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_counts_allocations_inside_the_closure() {
        let (_, allocations) = audit(|| {
            let consumed: Vec<u8> = vec![7; 1024];
            consumed.len()
        });
        assert!(allocations >= 1, "the Vec allocation should be counted");
    }

    #[test]
    fn audit_reports_zero_for_allocation_free_closures() {
        let (answer, allocations) = audit(|| 40 + 2);
        assert_eq!(answer, 42);
        assert_eq!(allocations, 0);
    }
}
