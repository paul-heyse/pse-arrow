//! Lexical allocation policy for exact-integer operations and their workers.
use std::cell::Cell;

thread_local! {
    static FRESH: Cell<bool> = const { Cell::new(false) };
}

/// A captured per-thread allocation policy, containing no buffer, callback or
/// caller control. Send the value to an explicitly owned worker and call
/// [`Self::run`] there to inherit the policy for that worker's actual operation.
/// Outside an explicit fresh scope, the native bounded integer cache is used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IntegerAllocationContext {
    fresh: bool,
}

impl IntegerAllocationContext {
    /// Construct exact integers without adopting old cleared-cache allocations.
    /// Temporaries dropped inside the call also bypass cache insertion. Existing
    /// cached values remain untouched; numerical algorithms are unchanged.
    pub const fn fresh() -> Self {
        Self { fresh: true }
    }

    /// Snapshot this thread's current allocation policy without allocating.
    pub fn capture() -> Self {
        Self { fresh: FRESH.get() }
    }

    /// Whether this policy requires backing independent of a previous operation.
    pub const fn is_fresh(self) -> bool {
        self.fresh
    }

    /// Apply this policy only for the current call. Nested calls and unwind
    /// restore the exact prior policy. Returned integers retain no policy or
    /// producer state; their eventual drop follows the then-current policy.
    pub fn run<T>(self, work: impl FnOnce() -> T) -> T {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                FRESH.set(self.0);
            }
        }
        let restore = Restore(FRESH.replace(self.fresh));
        let result = work();
        drop(restore);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_allocation_context_restores_nested_and_unwound_policy() {
        assert!(!IntegerAllocationContext::capture().is_fresh());
        IntegerAllocationContext::fresh().run(|| {
            assert!(IntegerAllocationContext::capture().is_fresh());
            IntegerAllocationContext::default().run(|| {
                assert!(!IntegerAllocationContext::capture().is_fresh());
            });
            assert!(IntegerAllocationContext::capture().is_fresh());
            let inherited = IntegerAllocationContext::capture();
            std::thread::scope(|workers| {
                workers
                    .spawn(move || {
                        inherited.run(|| {
                            assert!(IntegerAllocationContext::capture().is_fresh());
                        })
                    })
                    .join()
                    .unwrap();
            });
        });
        assert!(!IntegerAllocationContext::capture().is_fresh());
        let result = std::panic::catch_unwind(|| {
            IntegerAllocationContext::fresh().run(|| panic!("allocation-policy unwind"));
        });
        assert!(result.is_err());
        assert!(!IntegerAllocationContext::capture().is_fresh());
    }
}
