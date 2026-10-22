// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Boundary inputs share the actual native cache and execution policy.
use super::CacheBudget;
use crate::EngineError;
use std::num::NonZeroUsize;

/// Native names, widths, defaults and projections for cache settings.
#[macro_export]
macro_rules! cache_settings_fields {
    ($emit:ident, $budget:ident) => { $emit! { $budget;
        working_bytes: usize, "int" => usize, $budget.working_bytes;
        concurrent_loads: usize, "int", 1 => usize, $budget.concurrent_loads.get();
        inflight_bytes: usize, "int", 0 => usize, $budget.inflight_bytes;
        inspection_bytes: usize, "int", 0 => usize, $budget.inspection_bytes;
        predicate_total_bytes: usize, "int", 0 => usize, $budget.predicate_total_bytes;
        predicate_cache_bytes: usize, "int", 0 => usize, $budget.predicate_cache_bytes;
        syntax_bytes: usize, "int", 0 => usize, $budget.syntax_bytes;
    } };
}
macro_rules! input {
    ($budget:ident; $($name:ident: $ty:ty, $hint:literal $(, $default:expr)? => $out:ty, $read:expr;)*) => {
        /// Raw cache inputs resolved by the native capacity validator.
        #[derive(Debug)]
        pub struct CacheSettingsInput { $(#[doc = concat!("Declared ", stringify!($name), " input.")] pub $name: $ty,)* }
    };
}
cache_settings_fields!(input, budget);
impl CacheSettingsInput {
    /// Construct the budget; runtime also checks the aggregate deployment limit.
    /// # Errors
    /// Zero concurrency, overflow or missing working capacity.
    pub fn resolve(self) -> Result<CacheBudget, EngineError> {
        let concurrent_loads = NonZeroUsize::new(self.concurrent_loads)
            .ok_or_else(|| invalid("concurrent_loads must be positive"))?;
        let budget = CacheBudget {
            working_bytes: self.working_bytes,
            concurrent_loads,
            inflight_bytes: self.inflight_bytes,
            inspection_bytes: self.inspection_bytes,
            predicate_cache_bytes: self.predicate_cache_bytes,
            predicate_total_bytes: self.predicate_total_bytes,
            syntax_bytes: self.syntax_bytes,
            ..CacheBudget::disabled(self.working_bytes)
        };
        budget.validate(usize::MAX)?;
        Ok(budget)
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::ConfigInvalid {
        key: "pse.runtime.cache".into(),
        reason: reason.into(),
    }
}

