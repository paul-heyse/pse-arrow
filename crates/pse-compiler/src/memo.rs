// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The artifact-hash memo (blueprint §14.3, ADR-0042).
//!
//! A key selects a lookup bucket. Reuse requires complete actual dependency equality and
//! current semantic admission, including source bytes and immutable parent bindings.
//! Fine-grained memoization with `salsa` is deferred under register row R-13.
//!
pub(crate) mod context;

use crate::{
    CompilerError,
    passes::{InputBundle, PassContext, PolicySet, StageKey},
};
use pse_catalog::{Snapshot, session::SessionSemantics};
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
#[derive(Debug)]
struct SourceInputs {
    entries: BTreeMap<(pse_ids::SemanticId, pse_ids::SemanticId, String), String>,
    engine: Option<SessionSemantics>,
    _lease: Arc<pse_ids::ReservationLease>,
}
impl Dependencies {
    /// Capture the complete declared input inventory and actual engine semantics.
    /// # Errors
    /// Retaining the complete source byte inventory exceeds the shared budget.
    pub fn capture(
        inputs: &InputBundle,
        ctx: &PassContext<'_>,
        executes_plans: bool,
    ) -> Result<Self, CompilerError> {
        let mut reservation = ctx.reserver.open("compiler:memo-source-inputs");
        let mut entries = BTreeMap::new();
        for bundle in ctx.documents.bundles() {
            for document in &bundle.documents {
                ctx.cancel.checkpoint()?;
                reservation
                    .try_grow(
                        document
                            .text
                            .len()
                            .saturating_add(document.path.len())
                            .saturating_add(256),
                    )
                    .map_err(|error| CompilerError::Catalog(error.into()))?;
                if entries
                    .insert(
                        (
                            bundle.package.package_id,
                            document.id,
                            document.path.clone(),
                        ),
                        document.text.clone(),
                    )
                    .is_some()
                {
                    return Err(crate::passes::dag::invalid("duplicate memo source input"));
                }
            }
        }
        let engine = if executes_plans {
            let session = ctx.session.ok_or_else(|| {
                crate::passes::dag::invalid(
                    "engine-dependent memo capture lacks actual sealed semantics",
                )
            })?;
            reservation
                .try_grow(session.semantic_inputs_extent()?)
                .map_err(|error| CompilerError::Catalog(error.into()))?;
            Some(session.semantic_inputs())
        } else {
            None
        };
        Ok(Self {
            registry: Arc::clone(ctx.registry),
            inputs: inputs.clone(),
            policies: ctx.policies.clone(),
            sources: Arc::new(SourceInputs {
                entries,
                engine,
                _lease: pse_ids::ReservationLease::new(reservation),
            }),
        })
    }
    /// Compare declarations, complete actual rows, parent context, policies and engine inputs.
    /// # Errors
    /// A retained actual relation no longer conforms to its declaration.
    pub fn equivalent(&self, other: &Self) -> Result<bool, CompilerError> {
        // Identical immutable declaration objects are direct authority. A different
        // registry object conservatively misses, even if its fingerprint collides.
        if !Arc::ptr_eq(&self.registry, &other.registry)
            || self.sources.entries != other.sources.entries
            || self.sources.engine != other.sources.engine
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
                        || !same_snapshot(left.snapshot(), right.snapshot(), &self.registry)?
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
                )?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn same_snapshot(
    left: &Arc<Snapshot>,
    right: &Arc<Snapshot>,
    registry: &Registry,
) -> Result<bool, CompilerError> {
    if Arc::ptr_eq(left, right) {
        return Ok(true);
    }
    let mut stack = vec![(Arc::clone(left), Arc::clone(right))];
    let mut visited = std::collections::BTreeSet::new();
    while let Some((left, right)) = stack.pop() {
        if Arc::ptr_eq(&left, &right) {
            continue;
        }
        if !visited.insert((Arc::as_ptr(&left), Arc::as_ptr(&right))) {
            continue;
        }
        if left.manifest_ref() != right.manifest_ref()
            || left.stage_pass() != right.stage_pass()
            || left.manifest().snapshot_kind != right.manifest().snapshot_kind
            || left.relations().keys().ne(right.relations().keys())
            || left.parents().keys().ne(right.parents().keys())
        {
            return Ok(false);
        }
        for (key, left) in left.relations() {
            let right = &right.relations()[key];
            if left.contract().canonical.schema != right.contract().canonical.schema
                || left.contract().canonical.relation_id != right.contract().canonical.relation_id
                || left.member().port != right.member().port
            {
                return Ok(false);
            }
            let spec = registry
                .relation_by_id(left.contract().canonical.relation_id)
                .ok_or_else(|| {
                    crate::passes::dag::invalid("memo dependency declaration missing")
                })?;
            left.contract().validate_against_registry(registry, spec)?;
            right.contract().validate_against_registry(registry, spec)?;
            // Arrow compares actual schema metadata and visible value buffers,
            // including signed-zero/NaN bits and recursive null masking.
            if left.batch() != right.batch() {
                return Ok(false);
            }
        }
        for (role, parent) in left.parents() {
            stack.push((Arc::clone(parent), Arc::clone(&right.parents()[role])));
        }
    }
    Ok(true)
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
    /// Retained dependency admission failed.
    pub fn lookup(
        &self,
        key: StageKey,
        dependencies: &Dependencies,
    ) -> Result<Option<Arc<Snapshot>>, CompilerError> {
        for entry in self.entries.get(&key).into_iter().flatten() {
            if entry.dependencies.equivalent(dependencies)? {
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
