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
use pse_columnar::PlanOrigin;

/// Classify all engine failures through the diagnostic leaf's sole mapping authority.
pub fn classify(error: DataFusionError, origin: PlanOrigin) -> RuleError {
    pse_columnar::classify(error, origin).into()
}

pub(crate) fn engine(error: DataFusionError) -> RuleError {
    classify(error, PlanOrigin::RuleCompiler)
}

pub(crate) fn internal(what: impl Into<String>) -> RuleError {
    RuleError::Internal { what: what.into() }
}
