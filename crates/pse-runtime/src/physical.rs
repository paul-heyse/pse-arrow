// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One invocation-owned projection of actual physical source bindings.

mod decode;
mod material;
mod plans;

use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col, lit};
use pse_engine::session::{EngineSession, output::declare_relation_output};

use pse_columnar::{AllocationLease, CancellationToken};
use pse_quantity::{QuantityKindId, QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

/// The complete registered source selector, including explicitly absent families.
/// Invocation caches compare actual immutable owners for these keys, never hashes.
pub fn input_keys(registry: &Registry) -> Vec<RelationKey> {
    INPUTS
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
    preconditions: Arc<pse_quantity::PhysicalPreconditions>,
    batches: BTreeMap<RelationKey, FieldCheckedBatch>,
    _allocation: Arc<AllocationLease>,
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
        session: &EngineSession,
        registry: &Registry,
        cancel: &CancellationToken,
    ) -> Result<Self, PhysicalError> {
        if !std::ptr::eq(session.registry().as_ref(), registry) {
            return Err(invalid(
                "physical inventory and session use different registry authorities",
            ));
        }
        let mut batches = BTreeMap::new();
        let mut selections = Vec::new();
        let mut roots = Vec::new();
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
            selections.push((key, spec));
            roots.push(plan);
        }
        let prepared = session.prepare_many(&roots, cancel)?;
        let completed = pse_engine::session::PreparedComputation::execute_group(
            prepared.into_iter().map(Ok),
            cancel,
        )
        .await?;
        for ((key, spec), completed) in selections.into_iter().zip(completed) {
            batches.insert(
                key,
                completed.into_checked_relation(registry, spec, cancel)?,
            );
        }
        plans::units(session, registry, &mut batches, cancel).await?;
        let extent = batches
            .values()
            .try_fold(0usize, |total, batch| {
                total.checked_add(batch.batch().get_array_memory_size())
            })
            .and_then(|size| size.checked_mul(8))
            .ok_or_else(|| invalid("physical inventory allocation extent overflow"))?;
        let allocation = pse_columnar::MemoryConsumer::new("physical algorithm inventory")
            .register(session.pool());
        allocation
            .try_grow(extent)
            .map_err(pse_columnar::CanonError::from)?;
        let context = plans::context(session, registry, cancel).await?;
        let (quantities, boolean) = decode::inventory(&batches, registry, cancel, context)?;
        let elements = material::elements(&batches, registry, cancel)?;
        let preconditions = material::preconditions(&batches, registry, &quantities, cancel)?;
        let preconditions = Arc::new(pse_quantity::PhysicalPreconditions::new(preconditions)?);
        Ok(Self {
            elements,
            preconditions,
            quantities,
            boolean,
            batches,
            _allocation: AllocationLease::new(allocation),
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
    ) -> Result<Expr, PhysicalError> {
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
    /// Immutable compiler context from the same admitted rows.
    pub fn compiler_preconditions(&self) -> Arc<pse_quantity::PhysicalPreconditions> {
        self.preconditions.clone()
    }
    /// Actual retained source selections, for durable declaration publication.
    pub fn source_batches(&self) -> &BTreeMap<RelationKey, FieldCheckedBatch> {
        &self.batches
    }
    /// Check declared prerequisites against actual operation and operand contracts.
    pub fn precondition_checker(&self) -> &(dyn pse_quantity::infer::InvariantChecker + Sync) {
        self.preconditions.as_ref()
    }
    /// Retained checked input columns for generated physical adapters.
    pub fn batch(&self, key: &RelationKey) -> Option<&FieldCheckedBatch> {
        self.batches.get(key)
    }
}

const INPUTS: &[&str] = &[
    "reference.units",
    "normalized.units",
    "reference.unit_sets",
    "reference.quantity_kinds",
    "reference.bases",
    "reference.reference_states",
    "reference.quantity_types",
    "reference.conversion_rules",
    "reference.quantity_operations",
    "reference.quantity_operation_reductions",
    "reference.quantity_preconditions",
    "reference.math_context",
    "reference.elements",
];

fn invalid(detail: impl Into<String>) -> PhysicalError {
    pse_quantity::QuantityError::InferencePrecondition {
        rule: "quantity.relation_admission",
        detail: detail.into(),
    }
    .into()
}
/// Physical source admission preserves each originating diagnostic.
#[derive(Debug, thiserror::Error)]
pub enum PhysicalError {
    /// Original material failure.
    #[error(transparent)]
    Material(#[from] pse_material::MaterialError),
    /// Original quantity failure.
    #[error(transparent)]
    Quantity(#[from] pse_quantity::QuantityError),
    /// Original catalog failure.
    #[error(transparent)]
    Catalog(#[from] pse_engine::EngineError),
    /// Original relation failure.
    #[error(transparent)]
    Relation(#[from] pse_relations::RelationError),
    /// Original canon failure.
    #[error(transparent)]
    Canon(#[from] pse_columnar::CanonError),
    /// Original schema failure.
    #[error(transparent)]
    Schema(#[from] pse_schema::SchemaError),
}

pse_diagnostics::impl_diagnostic! {
    PhysicalError,
    code(_this){None},
    forward(this){match this {
        Self::Material(e)=>Some(e),Self::Quantity(e)=>Some(e),Self::Catalog(e)=>Some(e),
        Self::Relation(e)=>Some(e),Self::Canon(e)=>Some(e),Self::Schema(e)=>Some(e),
    }},
    help(_this){None},related(_this){None},source(_this){None}
}
