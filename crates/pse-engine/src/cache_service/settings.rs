// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Boundary inputs share native policy validation and checked duration conversion.
use super::CacheBudget;
use crate::EngineError;
use std::{num::NonZeroUsize, time::Duration};

/// Native names, widths, defaults and projections for cache settings.
#[macro_export]
macro_rules! cache_settings_fields {
    ($emit:ident, $budget:ident) => { $emit! { $budget;
        working_bytes: usize, "int" => usize, $budget.working_bytes;
        metadata_bytes: usize, "int", 0 => usize, $budget.metadata_bytes;
        statistics_bytes: usize, "int", 0 => usize, $budget.statistics_bytes;
        listing_bytes: usize, "int", 0 => usize, $budget.listing_bytes;
        listing_ttl_ms: Option<u64>, "int | None", Some(0) => Option<u64>, $budget.listing_ttl.map($crate::cache_service::settings::duration_millis).transpose()?;
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
    /// Zero concurrency, invalid TTL, overflow or missing working capacity.
    pub fn resolve(self) -> Result<CacheBudget, EngineError> {
        let concurrent_loads = NonZeroUsize::new(self.concurrent_loads)
            .ok_or_else(|| invalid("concurrent_loads must be positive"))?;
        let budget = CacheBudget {
                working_bytes: self.working_bytes,
                metadata_bytes: self.metadata_bytes,
                statistics_bytes: self.statistics_bytes,
                listing_bytes: self.listing_bytes,
                listing_ttl: self.listing_ttl_ms.map(Duration::from_millis),
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
/// Exact unsigned milliseconds; never truncate native precision.
/// # Errors
/// Fractional milliseconds or values wider than the boundary contract.
pub fn duration_millis(value: Duration) -> Result<u64, EngineError> {
    if !value.subsec_nanos().is_multiple_of(1_000_000) {
        return Err(invalid("duration is not an exact millisecond count"));
    }
    u64::try_from(value.as_millis())
        .map_err(|_| invalid("duration exceeds unsigned 64-bit milliseconds"))
}
fn invalid(reason: &str) -> EngineError {
    EngineError::ConfigInvalid {
        key: "pse.runtime.cache".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod cache_settings_unit {
    use super::*;
    #[test]
    fn millisecond_boundary_is_exact_and_checked() {
        for millis in [0, 1, u64::MAX] {
            assert_eq!(
                duration_millis(Duration::from_millis(millis)).unwrap(),
                millis
            );
        }
        assert!(duration_millis(Duration::from_nanos(1)).is_err());
        assert!(duration_millis(Duration::MAX).is_err());
    }
}
