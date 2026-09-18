// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-admitted math relation adapters shared by normalization and physical typing.

pub(crate) mod domain;
pub(crate) mod sink;
mod source;
pub(crate) mod syntax;
pub use sink::{Family, RelationSink};
pub use source::{RelationSource, SourceFamily};

use pse_mathir::MathIrError;
pub(crate) fn malformed(detail: impl Into<String>) -> MathIrError {
    MathIrError::Malformed {
        node: None,
        detail: detail.into(),
    }
}

/// Enforce the existing resolution boundary independently of the Arrow payload shape.
pub(crate) fn check_payload_family(
    payload: &pse_mathir::Payload,
    normalized: bool,
    pending: bool,
) -> Result<(), MathIrError> {
    use pse_mathir::{DomainRef, GuardRef, Payload, ValueRef};
    if !normalized
        && (matches!(
            payload,
            Payload::SymbolRef {
                symbol: ValueRef::Template { .. } | ValueRef::Domain(_) | ValueRef::Index(_)
            } | Payload::PendingPath { .. }
                | Payload::PendingGather { .. }
        ) || matches!(payload.domain(), Some(DomainRef::Template { .. }))
            || matches!(payload.guard(), Some(GuardRef::Predicate { .. })))
    {
        return Err(malformed(
            "unresolved source reference in instantiated output",
        ));
    }
    if !pending
        && matches!(
            payload,
            Payload::PendingUnitConvert { .. } | Payload::PendingSmoothOp { .. }
        )
    {
        return Err(malformed("pending physical request in compiled output"));
    }
    Ok(())
}
