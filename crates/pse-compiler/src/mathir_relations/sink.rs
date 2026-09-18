// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Existing mathematical callbacks append directly to generated native Arrow builders.

use super::malformed;
use pse_authoring::SourceSpan;
use pse_ids::{CancellationToken, MemoryReserver, Reservation, SemanticId};
use pse_mathir::{MathIrError, NodeId};
use pse_relations::{
    RelationError,
    columnar::{Collection, FieldCheckedBatch},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

/// A mathematical storage family and its explicit normalization provenance.
#[derive(Clone, Debug)]
pub enum Family {
    /// Fully resolved compiled mathematical relations.
    Compiled,
    /// Instantiated references with deferred physical requests for P10.
    Inferred,
    /// Parsed source family with graph-local ordinal offset and exact source location.
    Normalized {
        /// Declared expression family.
        prefix: &'static str,
        /// Global ordinal offset.
        offset: i64,
        /// Source derivation identity.
        derivation: SemanticId,
        /// Original authored expression range.
        source_span: SourceSpan,
    },
}

/// Typed callback adapter; its implementation is generated from the actual `MathIR` API
/// and exact declared relation families. It has no dynamic row representation.
#[derive(Debug)]
pub struct RelationSink<'a> {
    pub(crate) family: Family,
    pub(crate) columns: Collection<'a>,
    reserver: &'a dyn MemoryReserver,
    cancel: &'a CancellationToken,
}
impl<'a> RelationSink<'a> {
    /// Start an empty projection under the caller's shared allocator.
    pub fn new(
        registry: &'a Registry,
        family: Family,
        reserver: &'a dyn MemoryReserver,
        cancel: &'a CancellationToken,
    ) -> Self {
        Self {
            family,
            columns: Collection::new(registry, reserver, cancel),
            reserver,
            cancel,
        }
    }
    /// Complete actual generated Arrow columns; no key or graph admission is implied.
    /// # Errors
    /// Cancellation, shared allocation failure or Arrow construction failure.
    pub fn into_batches(self) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, RelationError> {
        self.columns.finish()
    }
    pub(crate) fn node(&self, node: NodeId) -> Result<i64, MathIrError> {
        let offset = match self.family {
            Family::Compiled | Family::Inferred => 0,
            Family::Normalized { offset, .. } => offset,
        };
        node.0
            .checked_add(offset)
            .ok_or_else(|| malformed("math ordinal overflow"))
            .and_then(|value| {
                if value < 0 {
                    Err(malformed("negative node ordinal"))
                } else {
                    Ok(value)
                }
            })
    }
    pub(crate) fn optional_node(&self, node: Option<NodeId>) -> Result<Option<i64>, MathIrError> {
        node.map(|node| self.node(node)).transpose()
    }
    pub(crate) fn provenance(&self) -> Result<(SemanticId, SourceSpan), MathIrError> {
        match self.family {
            Family::Normalized {
                derivation,
                source_span,
                ..
            } => Ok((derivation, source_span)),
            Family::Compiled | Family::Inferred => Err(malformed(
                "source provenance requested outside normalized family",
            )),
        }
    }
    pub(crate) fn reserve_row(&self, payload: usize) -> Result<Box<dyn Reservation>, MathIrError> {
        self.cancel.checkpoint()?;
        let mut work = self.reserver.open("math:typed-callback-row");
        let extent = payload
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| malformed("mathematical callback allocation extent overflow"))?;
        work.try_grow(extent).map_err(pse_ids::CanonError::from)?;
        Ok(work)
    }
}

pub(crate) fn adapter_error(error: RelationError) -> MathIrError {
    match error {
        RelationError::Canon(error) => MathIrError::Canon(error),
        other => malformed(other.to_string()),
    }
}
