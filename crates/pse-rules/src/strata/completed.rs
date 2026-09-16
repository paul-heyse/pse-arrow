// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completed typed facts; construction releases execution ancestry.
use crate::{RuleError, errmap::internal};
use datafusion::arrow::array::RecordBatch;
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::{RelationKey, RuleSpec};
use std::{collections::BTreeMap, sync::Arc};

/// A whole admitted program's immutable successful facts and typed derivations.
pub struct CompletedProgram {
    relations: BTreeMap<RelationKey, FieldCheckedBatch>,
    derivations: Vec<RecordBatch>,
    rules: Vec<SemanticId>,
    registry: Arc<pse_schema::Registry>,
    _lease: Arc<pse_ids::ReservationLease>,
}
impl std::fmt::Debug for CompletedProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompletedProgram")
            .field("relations", &self.relations.keys())
            .field("rules", &self.rules)
            .finish_non_exhaustive()
    }
}
impl CompletedProgram {
    // Called exclusively after successful fixed point, full head and conflict admission.
    pub(super) fn seal(
        relations: &BTreeMap<RelationKey, FieldCheckedBatch>,
        derivations: &[RecordBatch],
        rules: &[RuleSpec],
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Arc<Self>, RuleError> {
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        let count = relations
            .len()
            .checked_add(derivations.len())
            .and_then(|n| n.checked_add(rules.len()))
            .ok_or_else(|| internal("completed receipt extent overflow"))?;
        let mut reservation = session.reserver().open("rules:completed-receipt");
        reservation
            .try_grow(
                count
                    .checked_mul(8192)
                    .ok_or_else(|| internal("completed receipt extent overflow"))?,
            )
            .map_err(pse_catalog::CatalogError::from)?;
        Ok(Arc::new(Self {
            relations: relations.clone(),
            derivations: derivations.to_vec(),
            rules: rules.iter().map(|rule| rule.id).collect(),
            registry: Arc::clone(session.registry()),
            _lease: pse_ids::ReservationLease::new(reservation),
        }))
    }
    /// The complete relation inventory retained by the completed program.
    pub fn keys(&self) -> impl Iterator<Item = RelationKey> + '_ {
        self.relations.keys().copied()
    }
    /// Retain a real member of this completed program; no arbitrary row wrapping exists.
    /// # Errors
    /// The relation is not one of the completed outputs.
    pub fn relation(&self, key: RelationKey) -> Result<Arc<CompletedRelation>, RuleError> {
        let checked = self
            .relations
            .get(&key)
            .ok_or_else(|| internal("relation is outside completed program"))?;
        Ok(Arc::new(CompletedRelation {
            checked: checked.clone(),
            registry: Arc::clone(&self.registry),
            relation: key,
        }))
    }
    /// Exact derivation rows produced by the successful executor.
    pub fn derivations(&self) -> &[RecordBatch] {
        &self.derivations
    }
}
/// A completed member owns its checked Arrow buffers, without retaining its producer.
#[derive(Debug)]
pub struct CompletedRelation {
    checked: FieldCheckedBatch,
    registry: Arc<pse_schema::Registry>,
    relation: RelationKey,
}
impl CompletedRelation {
    /// Actual immutable completed rows.
    pub fn batch(&self) -> &RecordBatch {
        self.checked().batch()
    }
    /// Retained native field construction from the successful program. Binding this
    /// value to another workspace preserves the established field obligations.
    pub fn checked(&self) -> &FieldCheckedBatch {
        &self.checked
    }
    pub(super) fn validate(
        &self,
        source: RelationKey,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(), RuleError> {
        self.validate_owner(source, session, cancel)?;
        session.validate_workspace_input_values(&source, self.batch())?;
        Ok(())
    }
    pub(super) fn validate_owner(
        &self,
        source: RelationKey,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(), RuleError> {
        if source != self.relation {
            return Err(internal("completed relation membership differs"));
        }
        if !Arc::ptr_eq(session.registry(), &self.registry) {
            return Err(internal(
                "completed facts belong to a different registry declaration",
            ));
        }
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        // Arrow buffer leases retain values and accounting independently of the producer.
        Ok(())
    }
}
