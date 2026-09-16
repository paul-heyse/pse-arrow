// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The artifact-hash memo (blueprint §14.3, ADR-0042).
//!
//! A key selects a lookup bucket. Reuse requires complete actual dependency equality and
//! current semantic admission, including source bytes and immutable parent bindings.
//! Fine-grained memoization with `salsa` is deferred under register row R-13.
//!

pub(crate) mod snapshots;
use snapshots::same_snapshot;

use crate::{
    CompilerError,
    passes::{InputBundle, PassContext, PolicySet, StageKey},
};
use pse_catalog::{Snapshot, session::SnapshotSession};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::Registry;
use std::{collections::BTreeMap, sync::Arc};

/// Immutable actual dependencies retained for direct comparison after a bucket lookup.
#[derive(Clone, Debug)]
pub struct Dependencies {
    registry: Arc<Registry>,
    inputs: InputBundle,
    policies: PolicySet,
    sources: Arc<SourceInputs>,
}
struct SourceInputs {
    documents: pse_authoring::document::OwnedDocumentSet,
    engine: SnapshotSession,
    _lease: Arc<pse_ids::ReservationLease>,
}
impl std::fmt::Debug for SourceInputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceInputs")
            .field("documents", &self.documents)
            .field("engine_profile", &self.engine.profile_hash())
            .finish_non_exhaustive()
    }
}
impl Dependencies {
    /// Actual execution effects can refuse reuse even when all retained inputs agree.
    /// # Errors
    /// The attempt observation collector is unavailable.
    pub fn reusable(&self) -> Result<bool, CompilerError> {
        Ok(!self.sources.engine.requires_fresh_execution()?)
    }
    /// Capture the complete declared input inventory and actual engine semantics.
    /// # Errors
    /// Retaining the complete source byte inventory exceeds the shared budget.
    pub fn capture(inputs: &InputBundle, ctx: &PassContext<'_>) -> Result<Self, CompilerError> {
        let mut reservation = ctx.reserver.open("compiler:memo-source-inputs");
        ctx.cancel.checkpoint()?;
        reservation
            .try_grow(4096 + inputs.ports.len() * 256 + ctx.policies.0.len() * 512)
            .map_err(|error| CompilerError::Catalog(error.into()))?;
        let engine = ctx.session.clone();
        Ok(Self {
            registry: Arc::clone(ctx.registry),
            inputs: inputs.clone(),
            policies: ctx.policies.clone(),
            sources: Arc::new(SourceInputs {
                documents: ctx.documents.clone(),
                engine,
                _lease: pse_ids::ReservationLease::new(reservation),
            }),
        })
    }
    /// Compare declarations, complete actual rows, parent context, policies and engine inputs.
    /// # Errors
    /// A retained relation no longer conforms, comparison exceeds its reservation,
    /// or the request is cancelled.
    pub fn equivalent(
        &self,
        other: &Self,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<bool, CompilerError> {
        cancel.checkpoint()?;
        if !self.reusable()? || !other.reusable()? {
            return Ok(false);
        }
        // Identical immutable declaration objects are direct authority. A different
        // registry object conservatively misses, even if its fingerprint collides.
        if !Arc::ptr_eq(&self.registry, &other.registry)
            || !same_documents(&self.sources.documents, &other.sources.documents)
            || self
                .sources
                .engine
                .validate_execution_environment(&other.sources.engine)
                .is_err()
            || self.inputs.ports.keys().ne(other.inputs.ports.keys())
            || self.policies.0.keys().ne(other.policies.0.keys())
        {
            return Ok(false);
        }
        for (name, left) in &self.inputs.ports {
            match (left, &other.inputs.ports[name]) {
                (None, None) => {}
                (Some(left), Some(right)) => {
                    if left.relation_id() != right.relation_id()
                        || left.relation().member().port != right.relation().member().port
                        || !same_snapshot(
                            left.snapshot(),
                            right.snapshot(),
                            &self.registry,
                            reserver,
                            cancel,
                        )?
                    {
                        return Ok(false);
                    }
                }
                _ => return Ok(false),
            }
        }
        for (name, left) in &self.policies.0 {
            let right = &other.policies.0[name];
            if left.policy_id != right.policy_id
                || left.input.relation_id() != right.input.relation_id()
                || left.input.relation().member().port != right.input.relation().member().port
                || !same_snapshot(
                    left.input.snapshot(),
                    right.input.snapshot(),
                    &self.registry,
                    reserver,
                    cancel,
                )?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn same_documents(
    left: &pse_authoring::document::OwnedDocumentSet,
    right: &pse_authoring::document::OwnedDocumentSet,
) -> bool {
    if left.same_owner(right) {
        return true;
    }
    let left = left.bundles();
    let right = right.bundles();
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.package.package_id == right.package.package_id
                && left.documents.len() == right.documents.len()
                && left
                    .documents
                    .iter()
                    .zip(&right.documents)
                    .all(|(left, right)| {
                        left.id == right.id && left.path == right.path && left.text == right.text
                    })
        })
}

#[derive(Debug)]
struct Entry {
    dependencies: Dependencies,
    output: Arc<Snapshot>,
}
/// A bounded in-process stage memo. Stored identities are indexes, never admission proofs.
#[derive(Debug)]
pub struct Memo {
    entries: BTreeMap<StageKey, Vec<Entry>>,
    capacity: usize,
}
impl Memo {
    /// Limit retained entries; zero explicitly disables reuse.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            capacity,
        }
    }
    /// Resolve a bucket and compare complete actual dependencies before returning a result.
    /// # Errors
    /// Retained dependency admission, resource reservation or cancellation failed.
    pub fn lookup(
        &self,
        key: StageKey,
        dependencies: &Dependencies,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<Option<Arc<Snapshot>>, CompilerError> {
        cancel.checkpoint()?;
        for entry in self.entries.get(&key).into_iter().flatten() {
            if entry
                .dependencies
                .equivalent(dependencies, reserver, cancel)?
            {
                return Ok(Some(Arc::clone(&entry.output)));
            }
        }
        Ok(None)
    }
    /// Record an output admitted by the catalog. Deterministically evict at capacity.
    pub fn insert(&mut self, key: StageKey, dependencies: Dependencies, output: Arc<Snapshot>) {
        if self.capacity == 0 {
            return;
        }
        while self.entries.values().map(Vec::len).sum::<usize>() >= self.capacity {
            self.entries.pop_first();
        }
        self.entries.entry(key).or_default().push(Entry {
            dependencies,
            output,
        });
    }
    /// Clear dependencies after an external validity boundary changes.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
