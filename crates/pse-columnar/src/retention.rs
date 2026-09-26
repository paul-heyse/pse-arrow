// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One clear/admit fence; each cache retains its own charging and eviction policy.
use std::sync::Mutex;

/// Clearing a retention owner fences every load that started in an older generation.
#[derive(Debug, Default)]
pub struct RetentionFence(Mutex<u64>);
impl RetentionFence {
    /// Capture before admitting a load, not after its asynchronous work begins.
    pub fn generation(&self) -> u64 {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// Advance the generation and clear retention atomically with respect to insertion.
    pub fn clear(&self, clear: impl FnOnce()) {
        let mut generation = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Exhaustion disables retention instead of wrapping into an older generation.
        *generation = generation.saturating_add(1);
        clear();
    }
    /// Publish only if no clear has intervened. Refusal does not invalidate the caller's value.
    pub fn admit<T>(&self, generation: u64, publish: impl FnOnce() -> T) -> Option<T> {
        let current = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (*current != u64::MAX && *current == generation).then(publish)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_fences_old_load_but_admits_new_generation() {
        let fence = RetentionFence::default();
        let old = fence.generation();
        fence.clear(|| {});
        assert!(fence.admit(old, || 9).is_none());
        assert_eq!(fence.admit(fence.generation(), || 7), Some(7));
    }
}

#[cfg(test)]
mod exhaustion {
    use super::*;
    #[test]
    fn exhausted_generation_refuses_all_retention_without_panicking() {
        let fence = RetentionFence(Mutex::new(u64::MAX - 1));
        let old = fence.generation();
        fence.clear(|| {});
        assert!(fence.admit(old, || 1).is_none());
        assert!(fence.admit(fence.generation(), || 1).is_none());
        fence.clear(|| {});
        assert!(fence.admit(old, || 1).is_none());
    }
}
