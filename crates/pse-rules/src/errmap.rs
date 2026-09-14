// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `DataFusionError` into the platform taxonomy, at one place (blueprint §23.2).
//!
//! Per-call-site classification is how a `ResourcesExhausted` becomes an
//! `internal.invariant` in one path and a `user.model` in another. The mapping table is
//! in §23.2 and the origin - rule compiler, analytics or kernel UDF - is what selects the
//! row.
//!
use crate::RuleError;
use datafusion_common::DataFusionError;
use pse_catalog::PlanOrigin;

/// Classify all engine failures through the catalog's sole mapping authority.
pub fn classify(error: DataFusionError, origin: PlanOrigin) -> RuleError {
    let mut errors: Vec<_> = pse_catalog::classify(error, origin)
        .into_iter()
        .map(RuleError::Catalog)
        .collect();
    if errors.len() == 1 {
        errors.remove(0)
    } else {
        RuleError::Collection { errors }
    }
}

pub(crate) fn engine(error: DataFusionError) -> RuleError {
    classify(error, PlanOrigin::RuleCompiler)
}

pub(crate) fn internal(what: impl Into<String>) -> RuleError {
    RuleError::Internal { what: what.into() }
}
