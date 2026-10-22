// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current receiving-source bindings, paired with an already admitted numerical body.
use super::{CompiledBody, Evaluation, PreparedBody, PreparedBodyData, TrialEvaluation, Worker};
use crate::{MathError, jets::ProductValues};
use pse_ids::{FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, ExecutionScope, ProviderBindings, ProviderError};
use pse_model::diagnostic::SourceLocation;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak, atomic::AtomicBool},
};

/// Complete current instance attribution for exactly one admitted original body.
/// Maps are never used to establish reusable arithmetic identity.
#[derive(Clone, Debug)]
pub struct SourceBindings {
    instance: SemanticId,
    checked_members: Arc<BTreeMap<SemanticId, SemanticId>>,
    checked_sources: Arc<BTreeMap<SemanticId, SemanticId>>,
    occurrences: Arc<BTreeMap<SemanticId, SourceLocation>>,
    source: Weak<PreparedBodyData>,
}
impl PartialEq for SourceBindings {
    fn eq(&self, other: &Self) -> bool {
        self.instance == other.instance
            && self.checked_members == other.checked_members
            && self.checked_sources == other.checked_sources
            && self.occurrences == other.occurrences
    }
}
impl SourceBindings {
    /// Validate actual token inventories before any compiled body can acquire these bindings.
    pub fn new(
        instance: SemanticId,
        body: &PreparedBody,
        checked_members: BTreeMap<SemanticId, SemanticId>,
        checked_sources: BTreeMap<SemanticId, SemanticId>,
        occurrences: BTreeMap<SemanticId, SourceLocation>,
    ) -> Result<Self, MathError> {
        Self::check_inventory(
            instance,
            body,
            &checked_members,
            &checked_sources,
            &occurrences,
        )?;
        Ok(Self {
            instance,
            checked_members: Arc::new(checked_members),
            checked_sources: Arc::new(checked_sources),
            occurrences: Arc::new(occurrences),
            source: Arc::downgrade(&body.data),
        })
    }
    pub(crate) fn check_inventory(
        instance: SemanticId,
        body: &PreparedBody,
        checked_members: &BTreeMap<SemanticId, SemanticId>,
        checked_sources: &BTreeMap<SemanticId, SemanticId>,
        occurrences: &BTreeMap<SemanticId, SourceLocation>,
    ) -> Result<(), MathError> {
        let maps_valid = checked_members
            .iter()
            .chain(checked_sources)
            .all(|(token, target)| *token != SemanticId::NIL && *target != SemanticId::NIL);
        let locations_valid = occurrences.iter().all(|(token, location)| {
            *token != SemanticId::NIL
                && location.source != SemanticId::NIL
                && location.document != Some(SemanticId::NIL)
                && match (location.start, location.end) {
                    (None, None) => true,
                    (Some(start), Some(end)) => start <= end && location.document.is_some(),
                    _ => false,
                }
        });
        if instance == SemanticId::NIL
            || !maps_valid
            || !locations_valid
            || !checked_members.keys().eq(body.checked_members().iter())
            || !checked_sources.keys().eq(body.checked_sources().iter())
            || !occurrences.keys().eq(body.occurrence_tokens().iter())
        {
            return Err(MathError::Contract(
                "receiving source bindings do not cover the admitted body".into(),
            ));
        }
        Ok(())
    }
    /// Actual current instance, never an anonymous default.
    pub fn instance(&self) -> SemanticId {
        self.instance
    }
    /// Actual current checked-member mapping, without transferring admission authority.
    pub fn checked_members(&self) -> &BTreeMap<SemanticId, SemanticId> {
        &self.checked_members
    }
    /// Actual current closure-source mapping, separate from checked members.
    pub fn checked_sources(&self) -> &BTreeMap<SemanticId, SemanticId> {
        &self.checked_sources
    }
    /// Complete current occurrence DTOs for explicit transport and receiving validation.
    pub fn occurrences(&self) -> &BTreeMap<SemanticId, SourceLocation> {
        &self.occurrences
    }
    /// Refuse bindings issued for an unrelated admitted body.
    pub fn check(&self, body: &PreparedBody) -> Result<(), MathError> {
        let same = self.source.ptr_eq(&Arc::downgrade(&body.data))
            || self
                .source
                .upgrade()
                .is_some_and(|data| PreparedBody { data, owner: None } == *body);
        if same {
            Ok(())
        } else {
            Err(MathError::Contract(
                "source bindings belong to another admitted body".into(),
            ))
        }
    }
    /// Bind a demanded derivative program to its original body without retaining that body.
    pub fn check_compiled(&self, body: &CompiledBody) -> Result<(), MathError> {
        let source = &body.prepared_support().data.source;
        let same = self.source.ptr_eq(source)
            || self
                .source
                .upgrade()
                .zip(source.upgrade())
                .is_some_and(|(a, b)| {
                    PreparedBody {
                        data: a,
                        owner: None,
                    } == PreparedBody {
                        data: b,
                        owner: None,
                    }
                });
        if same {
            Ok(())
        } else {
            Err(MathError::Contract(
                "source bindings belong to another compiled source".into(),
            ))
        }
    }
    /// Own the current diagnostic projection without changing or globally rewriting its cause.
    pub fn project(&self, cause: MathError) -> MathError {
        MathError::Instance {
            instance: self.instance,
            checked_members: self.checked_members.as_ref().clone(),
            checked_sources: self.checked_sources.as_ref().clone(),
            occurrences: self.occurrences.as_ref().clone(),
            cause: Box::new(cause),
        }
    }
    /// Conservative retained extent of these current-source maps, excluding a second body.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.checked_members
                    .len()
                    .saturating_add(self.checked_sources.len())
                    .saturating_mul(size_of::<(SemanticId, SemanticId)>() + 96),
            )
            .saturating_add(
                self.occurrences
                    .len()
                    .saturating_mul(size_of::<(SemanticId, SourceLocation)>() + 96),
            )
            .saturating_add(
                self.occurrences
                    .values()
                    .map(SourceLocation::heap_bytes)
                    .fold(0usize, usize::saturating_add),
            )
            .saturating_add(6 * size_of::<usize>())
    }
    /// Frame complete current attribution only when a caller's identity includes source context.
    pub fn frame(&self, hash: &mut FramedHasher) {
        hash.id(&self.instance);
        for map in [&self.checked_members, &self.checked_sources] {
            hash.u64(map.len() as u64);
            for (local, current) in map.iter() {
                hash.id(local).id(current);
            }
        }
        hash.u64(self.occurrences.len() as u64);
        for (local, current) in self.occurrences.iter() {
            hash.id(local);
            current.frame(hash);
        }
    }
    pub(crate) fn evaluation_error(&self, error: MathError) -> MathError {
        // These are control/refusal classes consumed directly by the provider boundary.
        match error {
            MathError::Cancelled
            | MathError::Limit(_)
            | MathError::ByteLimit { .. }
            | MathError::SlotLimit { .. }
            | MathError::WorkLimit { .. }
            | MathError::Scope(_) => error,
            MathError::Provider {
                cause: ProviderError::Cancelled | ProviderError::Deadline | ProviderError::Limit(_),
                ..
            } => error,
            error => self.project(error),
        }
    }
}

/// One compiled product paired with its validated receiving-source context.
#[derive(Clone, Debug)]
pub struct SourceBody {
    compiled: Arc<CompiledBody>,
    bindings: SourceBindings,
}
impl SourceBody {
    /// Refuse crossed body/context pairs before worker creation.
    pub fn new(compiled: Arc<CompiledBody>, bindings: SourceBindings) -> Result<Self, MathError> {
        bindings.check_compiled(&compiled)?;
        Ok(Self { compiled, bindings })
    }
    /// Retained admitted numerical product, without mutable access to its paired slot.
    pub fn compiled(&self) -> &Arc<CompiledBody> {
        &self.compiled
    }
    /// Source context for new demanded products of this same original body.
    pub fn bindings(&self) -> &SourceBindings {
        &self.bindings
    }
    /// Retain admission through the product; this preserves the original source allocation identity.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.compiled = Arc::new(self.compiled.as_ref().clone().with_owner(owner));
        self
    }
    /// Conservative body and current attribution extent.
    pub fn retained_bytes(&self) -> usize {
        self.compiled
            .retained_bytes()
            .saturating_add(self.bindings.retained_bytes())
    }
    pub(crate) fn worker(&self) -> SourceWorker {
        SourceWorker {
            worker: self.compiled.worker(),
            bindings: self.bindings.clone(),
        }
    }
    pub(crate) fn worker_scoped(&self, scope: ExecutionScope) -> SourceWorker {
        SourceWorker {
            worker: self.compiled.worker_scoped(scope),
            bindings: self.bindings.clone(),
        }
    }
}
impl Deref for SourceBody {
    type Target = Arc<CompiledBody>;
    fn deref(&self) -> &Self::Target {
        &self.compiled
    }
}

/// Worker-local numerical scratch with immutable, body-specific diagnostic attribution.
#[derive(Debug)]
pub(crate) struct SourceWorker {
    worker: Worker,
    bindings: SourceBindings,
}
impl Deref for SourceWorker {
    type Target = Worker;
    fn deref(&self) -> &Self::Target {
        &self.worker
    }
}
impl DerefMut for SourceWorker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.worker
    }
}
impl SourceWorker {
    pub(crate) fn prepare_providers(
        &mut self,
        providers: &mut ProviderBindings,
    ) -> Result<(), MathError> {
        self.worker
            .prepare_providers(providers)
            .map_err(|error| self.bindings.evaluation_error(error))
    }
    pub(crate) fn evaluate(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        self.worker
            .evaluate(inputs, order, providers, cancel)
            .map_err(|error| self.bindings.evaluation_error(error))
    }
    pub(crate) fn evaluate_trial_into(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
        result: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        self.worker
            .evaluate_trial_into(inputs, order, providers, cancel, result)
            .map_err(|error| self.bindings.evaluation_error(error))
    }
    pub(crate) fn evaluate_product_trial_into(
        &mut self,
        inputs: &[f64],
        values: ProductValues<'_>,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
        output: &mut [f64],
        trial: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        self.worker
            .evaluate_product_trial_into(inputs, values, providers, cancel, output, trial)
            .map_err(|error| self.bindings.evaluation_error(error))
    }
}

#[cfg(test)]
#[path = "source_bindings_tests.rs"]
mod tests;
