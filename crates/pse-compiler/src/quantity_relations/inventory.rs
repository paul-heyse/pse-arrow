// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One invocation-owned projection of actual physical source bindings.

mod decode;
mod material;
mod plans;

use super::invalid;
use crate::CompilerError;
use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col, lit};
use pse_catalog::session::{SnapshotSession, output::declare_relation_output};
use pse_ids::{CancellationToken, ReservationLease};
use pse_quantity::{QuantityKindId, QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

/// The complete registered source selector, including explicitly absent families.
/// Invocation caches compare actual immutable owners for these keys, never hashes.
pub fn input_keys(registry: &Registry) -> Vec<RelationKey> {
    pse_schema::catalog::s14_semantic_passes::PHYSICAL_INPUTS
        .iter()
        .filter_map(|name| registry.relation(name).map(|spec| spec.key))
        .collect()
}

/// Immutable algorithm inputs projected once from the actual bound Arrow sources.
/// It supplies no independent maps for a caller to reconcile with those sources.
#[derive(Debug)]
pub struct PhysicalInventory {
    quantities: QuantityRegistry,
    boolean: Option<QuantityKindId>,
    elements: pse_material::ElementTable,
    preconditions: Arc<super::source::preconditions::PhysicalPreconditions>,
    batches: BTreeMap<RelationKey, FieldCheckedBatch>,
    _allocation: Arc<ReservationLease>,
}

impl PhysicalInventory {
    /// Select and order the actual physical inputs through the common native boundary,
    /// then construct the immutable exact rational/affine algorithm inventory once.
    /// Missing optional families stay absent; this never creates a neutral declaration.
    ///
    /// # Errors
    /// Native execution, incompatible actual definitions, missing physical references,
    /// conflicting context choices, cancellation or shared resource exhaustion.
    pub async fn load(
        session: &SnapshotSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<Self, CompilerError> {
        if !std::ptr::eq(session.registry().as_ref(), registry) {
            return Err(invalid(
                "physical inventory and session use different registry authorities",
            ));
        }
        let mut batches = BTreeMap::new();
        for key in input_keys(registry) {
            cancel.checkpoint()?;
            if session.table_provider(&key).is_none() {
                continue;
            }
            let spec = registry
                .relation(&key.qualified_name())
                .ok_or_else(|| invalid("physical source declaration disappeared"))?;
            let scan = LogicalPlanBuilder::scan(
                session.table_reference(&key)?,
                session.table_source(&key)?,
                None,
            )
            .map_err(plans::engine)?
            .sort(
                spec.primary_key
                    .iter()
                    .map(|name| col(*name).sort(true, false)),
            )
            .map_err(plans::engine)?
            .build()
            .map_err(plans::engine)?;
            let plan = declare_relation_output(scan, registry, spec).map_err(plans::engine)?;
            let batch = session
                .prepare_rule_plan(plan, cancel)?
                .execute(cancel)
                .await?
                .into_checked_relation(registry, spec, cancel)?;
            batches.insert(key, batch);
        }
        plans::units(session, registry, &mut batches, cancel).await?;
        let extent = batches
            .values()
            .try_fold(0usize, |total, batch| {
                total.checked_add(batch.batch().get_array_memory_size())
            })
            .and_then(|size| size.checked_mul(8))
            .ok_or_else(|| invalid("physical inventory allocation extent overflow"))?;
        let mut allocation = session.reserver().open("physical algorithm inventory");
        allocation
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        let context = plans::context(session, registry, cancel).await?;
        let (quantities, boolean) = decode::inventory(&batches, registry, cancel, context)?;
        let elements = material::elements(&batches, registry, cancel)?;
        let preconditions = material::preconditions(&batches, registry, &quantities, cancel)?;
        let preconditions = Arc::new(super::source::preconditions::PhysicalPreconditions::new(
            preconditions,
        )?);
        Ok(Self {
            elements,
            preconditions,
            quantities,
            boolean,
            batches,
            _allocation: ReservationLease::new(allocation),
        })
    }

    /// Actual immutable quantity declarations used by exact physical algorithms.
    pub const fn quantities(&self) -> &QuantityRegistry {
        &self.quantities
    }
    /// Construct native representation arithmetic for an actual joined unit/type pair.
    /// Callers bind the value's unit and complete quantity context before calling this;
    /// this method does not infer either from a number or a dimension. Multiplication
    /// precedes addition, matching the leaf algorithm's two-rounding contract.
    ///
    /// # Errors
    /// An absent unit/type, incompatible dimension or datum, or invalid coefficients.
    pub fn conversion_expr(
        &self,
        value: Expr,
        from: UnitId,
        to: UnitId,
        quantity: QuantityTypeId,
    ) -> Result<Expr, CompilerError> {
        let quantity = self.quantities.quantity_type(quantity)?;
        let conversion = pse_quantity::convert_spec_for_type(
            self.quantities.unit(from)?,
            self.quantities.unit(to)?,
            &quantity.key,
        )?;
        if self.quantities.unit(from)?.dimension
            != self.quantities.unit(quantity.canonical_unit)?.dimension
        {
            return Err(invalid(
                "unit representation differs from its bound quantity dimension",
            ));
        }
        Ok((value * lit(conversion.scale)) + lit(conversion.offset))
    }
    /// Explicit context choice from actual `reference.math_context` rows.
    pub fn neutral(&self) -> Option<QuantityTypeId> {
        self.quantities.neutral_dimensionless()
    }
    /// Explicit Boolean kind choice; dimensions never select it.
    pub const fn boolean(&self) -> Option<QuantityKindId> {
        self.boolean
    }
    /// Actual declared element masses for composition algorithms.
    pub const fn elements(&self) -> &pse_material::ElementTable {
        &self.elements
    }
    /// Actual selected operand predicates for physical operation algorithms.
    pub fn preconditions(&self) -> &[pse_quantity::PhysicalPrecondition] {
        self.preconditions.declarations()
    }
    /// Check declared prerequisites against actual operation and operand contracts.
    pub fn precondition_checker(&self) -> &(dyn pse_quantity::infer::InvariantChecker + Sync) {
        self.preconditions.as_ref()
    }
    pub(crate) fn checker_owner(&self) -> Arc<super::source::preconditions::PhysicalPreconditions> {
        Arc::clone(&self.preconditions)
    }
    /// Retained checked input columns for generated physical adapters.
    pub fn batch(&self, key: &RelationKey) -> Option<&FieldCheckedBatch> {
        self.batches.get(key)
    }
}
