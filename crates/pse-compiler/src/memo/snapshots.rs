// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact dependency comparison follows admitted owners, including invocation inputs.
use crate::CompilerError;
use pse_catalog::{Snapshot, store::invocation::InvocationContext};
use pse_ids::{CancellationToken, MemoryReserver, Reservation};
use pse_schema::Registry;
use std::{collections::BTreeSet, sync::Arc};

type Pair = (Arc<Snapshot>, Arc<Snapshot>);
type Address = (*const Snapshot, *const Snapshot);

pub(crate) fn same_invocation(
    left: &InvocationContext,
    right: &InvocationContext,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<bool, CompilerError> {
    let mut comparison = Comparison::new(registry, reserver, cancel);
    if !comparison.invocation(left, right)? {
        return Ok(false);
    }
    comparison.run()
}

pub(crate) fn same_snapshot(
    left: &Arc<Snapshot>,
    right: &Arc<Snapshot>,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<bool, CompilerError> {
    let mut comparison = Comparison::new(registry, reserver, cancel);
    comparison.push(left, right)?;
    comparison.run()
}

struct Comparison<'a> {
    stack: Vec<Pair>,
    visited: BTreeSet<Address>,
    registry: &'a Registry,
    cancel: &'a CancellationToken,
    work: Box<dyn Reservation>,
}

impl<'a> Comparison<'a> {
    fn new(
        registry: &'a Registry,
        reserver: &dyn MemoryReserver,
        cancel: &'a CancellationToken,
    ) -> Self {
        Self {
            stack: Vec::new(),
            visited: BTreeSet::new(),
            registry,
            cancel,
            work: reserver.open("compiler:dependency-comparison"),
        }
    }

    fn push(&mut self, left: &Arc<Snapshot>, right: &Arc<Snapshot>) -> Result<(), CompilerError> {
        self.cancel.checkpoint()?;
        let key = (Arc::as_ptr(left), Arc::as_ptr(right));
        if Arc::ptr_eq(left, right) || self.visited.contains(&key) {
            return Ok(());
        }
        // Reserve a complete B-tree node for each newly visited pair and bound
        // Vec growth before either collection can allocate. Actual Arrow buffers
        // and snapshot owners are shared, never charged as new copies here.
        let extent = 11 * size_of::<Address>() + 16 * size_of::<usize>() + 4 * size_of::<Pair>();
        self.work
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        self.visited.insert(key);
        self.stack.push((Arc::clone(left), Arc::clone(right)));
        Ok(())
    }

    fn invocation(
        &mut self,
        left: &InvocationContext,
        right: &InvocationContext,
    ) -> Result<bool, CompilerError> {
        self.cancel.checkpoint()?;
        if std::ptr::eq(left, right) {
            return Ok(true);
        }
        if left.engine != right.engine || left.policies.keys().ne(right.policies.keys()) {
            return Ok(false);
        }
        match (&left.document_source, &right.document_source) {
            (None, None) => {}
            (Some(left), Some(right)) => self.push(left, right)?,
            _ => return Ok(false),
        }
        for (role, left) in &left.policies {
            let right = &right.policies[role];
            if left.policy_id != right.policy_id || left.port != right.port {
                return Ok(false);
            }
            self.push(&left.snapshot, &right.snapshot)?;
        }
        Ok(true)
    }

    fn run(mut self) -> Result<bool, CompilerError> {
        self.cancel.checkpoint()?;
        while let Some((left, right)) = self.stack.pop() {
            self.cancel.checkpoint()?;
            if left.manifest_ref() != right.manifest_ref()
                || left.stage_pass() != right.stage_pass()
                || left.manifest().snapshot_kind != right.manifest().snapshot_kind
                || left.parents().keys().ne(right.parents().keys())
                || !self.rows(&left, &right)?
            {
                return Ok(false);
            }
            match (left.invocation(), right.invocation()) {
                (None, None) => {}
                (Some(left), Some(right)) => {
                    if !self.invocation(left, right)? {
                        return Ok(false);
                    }
                }
                _ => return Ok(false),
            }
            for (role, parent) in left.parents() {
                self.push(parent, &right.parents()[role])?;
            }
        }
        Ok(true)
    }

    fn rows(&self, left: &Snapshot, right: &Snapshot) -> Result<bool, CompilerError> {
        if left.relations().keys().ne(right.relations().keys()) {
            return Ok(false);
        }
        for (key, left) in left.relations() {
            self.cancel.checkpoint()?;
            let right = &right.relations()[key];
            if left.contract().canonical.schema != right.contract().canonical.schema
                || left.contract().canonical.relation_id != right.contract().canonical.relation_id
                || left.member().port != right.member().port
            {
                return Ok(false);
            }
            let spec = self
                .registry
                .relation_by_id(left.contract().canonical.relation_id)
                .ok_or_else(|| {
                    crate::passes::dag::invalid("memo dependency declaration missing")
                })?;
            left.contract()
                .validate_against_registry(self.registry, spec)?;
            right
                .contract()
                .validate_against_registry(self.registry, spec)?;
            // Arrow compares actual schema metadata and visible value buffers,
            // including signed-zero/NaN bits and recursive null masking.
            if left.batch() != right.batch() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
