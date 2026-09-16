// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native selection of admitted stage outputs for the existing physical algorithms.
mod columns;
mod kernels;
mod plans;
pub(crate) mod preconditions;

use super::invalid;
use crate::{CompilerError, InputBundle, PassContext};
use datafusion::logical_expr::{LogicalPlan, LogicalPlanBuilder, col};
use pse_catalog::session::{
    CompletedComputation, SnapshotSession, output::declare_relation_output,
};
use pse_ids::{CancellationToken, Reservation, ReservationLease, SemanticId};
use pse_mathir::{
    NodeId,
    index::DomainFacts,
    infer::{GroupFacts, KernelContract, SymbolTypeSource},
};
use pse_quantity::{ConversionId, DomainId, DomainKind, QuantityKindId, QuantityTypeId, UnitId};
use pse_relations::{columnar::FieldCheckedBatch, generated::normalized};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Algorithm-only indexes derived from the actual admitted P10 input bindings.
/// Construction is private to the registered pass route: local field checking alone
/// cannot establish the carrier correspondences established by the input producers.
#[derive(Debug)]
pub struct RelationSymbolSource {
    symbols: BTreeMap<SemanticId, (QuantityTypeId, UnitId)>,
    domains: BTreeMap<DomainId, DomainFacts>,
    groups: BTreeMap<SemanticId, GroupFacts>,
    kernels: BTreeMap<SemanticId, KernelContract>,
    unknowns: BTreeMap<(SemanticId, u16), QuantityTypeId>,
    boolean: Option<QuantityKindId>,
    preconditions: Arc<preconditions::PhysicalPreconditions>,
    _allocation: Arc<ReservationLease>,
}
impl SymbolTypeSource for RelationSymbolSource {
    fn invariant_checker(&self, _node: NodeId) -> &dyn pse_quantity::infer::InvariantChecker {
        self.preconditions.as_ref()
    }
    fn symbol_type(&self, symbol: SemanticId) -> Option<QuantityTypeId> {
        self.symbols.get(&symbol).map(|value| value.0)
    }
    fn symbol_unit(&self, symbol: SemanticId) -> Option<UnitId> {
        self.symbols.get(&symbol).map(|value| value.1)
    }
    fn boolean_kind(&self) -> Option<QuantityKindId> {
        self.boolean
    }
    fn group(&self, group: SemanticId) -> Option<&GroupFacts> {
        self.groups.get(&group)
    }
    fn domain(&self, domain: DomainId) -> Option<&DomainFacts> {
        self.domains.get(&domain)
    }
    fn kernel_contract(&self, kernel: SemanticId) -> Option<&KernelContract> {
        self.kernels.get(&kernel)
    }
    fn implicit_unknown(&self, system: SemanticId, ordinal: u16) -> Option<QuantityTypeId> {
        self.unknowns.get(&(system, ordinal)).copied()
    }
    fn conversion_binding(
        &self,
        _node: NodeId,
        _operand: u16,
        _conversion: ConversionId,
    ) -> Option<SemanticId> {
        None
    }
}
impl RelationSymbolSource {
    /// Bind admitted input owners to the same immutable engine, then execute
    /// native joins/order/recursive carrier selection. Quantity operations and
    /// occurrence typing remain the leaf algorithms' responsibility.
    pub(crate) async fn load(
        ctx: &PassContext<'_>,
        inputs: &InputBundle,
    ) -> Result<Self, CompilerError> {
        let spec = ctx
            .registry
            .pass("P10@1")
            .ok_or_else(|| invalid("physical source requires declared P10"))?;
        inputs.validate(spec, ctx.registry)?;
        let session = ctx
            .session
            .select_inputs(&BTreeSet::new(), ctx.cancel)?
            .with_checked_workspace(inputs.checked_rows(ctx.registry)?, ctx.cancel)?;
        let physical = ctx.physical()?;
        let quantities = physical.quantities();
        let mut loader = Loader {
            session: &session,
            cancel: ctx.cancel,
            allocation: ctx.reserver.open("physical graph algorithm indexes"),
        };
        let mut symbols = BTreeMap::new();
        let plan = plans::project(
            plans::scan(&session, "compiled.symbols", "symbol")?,
            [
                col("symbol.symbol_id").alias("symbol_id"),
                col("symbol.quantity_type_id").alias("quantity_type_id"),
                col("symbol.unit_id").alias("unit_id"),
            ],
        )?;
        let result = loader.execute(plan).await?;
        for batch in result.batches() {
            for row in 0..batch.num_rows() {
                ctx.cancel.checkpoint()?;
                let id = columns::id(batch, "symbol_id", row)?;
                let quantity =
                    QuantityTypeId::from_id(columns::id(batch, "quantity_type_id", row)?);
                let unit = UnitId::from_id(columns::id(batch, "unit_id", row)?);
                let contract = quantities.quantity_type(quantity)?;
                if !contract.key.shape.is_empty() {
                    return Err(invalid("scalar symbol has an indexed quantity type"));
                }
                pse_quantity::convert_spec_for_type(
                    quantities.unit(unit)?,
                    quantities.unit(contract.canonical_unit)?,
                    &contract.key,
                )?;
                symbols.insert(id, (quantity, unit));
            }
        }
        let domain_batch = loader.declared("normalized.domains").await?;
        let view = normalized::domains::View::from_checked(&domain_batch)?;
        let mut domains = BTreeMap::new();
        for position in 0..view.len() {
            ctx.cancel.checkpoint()?;
            let row = view.row(position)?;
            let unit = row.unit_id.map(UnitId::from_id);
            if let Some(unit) = unit {
                quantities.unit(unit)?;
            }
            domains.insert(
                DomainId::from_id(row.domain_id),
                DomainFacts {
                    kind: DomainKind::parse(row.kind.as_str())
                        .ok_or_else(|| invalid("unknown physical domain kind"))?,
                    continuous: row.continuous,
                    unit,
                    members: Vec::new(),
                },
            );
        }
        let result = loader.execute(plans::domain_members(&session)?).await?;
        for batch in result.batches() {
            for row in 0..batch.num_rows() {
                ctx.cancel.checkpoint()?;
                let domain = DomainId::from_id(columns::id(batch, "domain_id", row)?);
                domains
                    .get_mut(&domain)
                    .ok_or_else(|| invalid("native domain join lacks its selected owner"))?
                    .members
                    .push(columns::id(batch, "member_id", row)?);
            }
        }
        let mut groups = BTreeMap::new();
        let result = loader.execute(plans::groups(&session, ctx.cancel)?).await?;
        for batch in result.batches() {
            for row in 0..batch.num_rows() {
                ctx.cancel.checkpoint()?;
                let id = columns::id(batch, "group_id", row)?;
                let factors = columns::ids(batch, "domain_ids", row)?
                    .into_iter()
                    .map(DomainId::from_id)
                    .collect::<Vec<_>>();
                let quantity =
                    QuantityTypeId::from_id(columns::id(batch, "quantity_type_id", row)?);
                let mut key = quantities.quantity_type(quantity)?.key.clone();
                let shape = factors
                    .iter()
                    .map(|domain| {
                        domains
                            .get(domain)
                            .map(|facts| facts.kind)
                            .ok_or_else(|| invalid("group factor has no actual domain"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if columns::id(batch, "origin_group_id", row)? == id && key.shape != shape {
                    return Err(invalid(
                        "declared group shape differs from actual ordered domains",
                    ));
                }
                key.shape = shape;
                let facts = GroupFacts {
                    quantity_type: quantities.resolve_key(&key)?,
                    domains: factors,
                    valid_tuples: Vec::new(),
                    members: BTreeMap::new(),
                };
                if groups.insert(id, facts).is_some() {
                    return Err(invalid(
                        "native group carrier has multiple physical meanings",
                    ));
                }
            }
        }
        let result = loader.execute(plans::group_members(&session)?).await?;
        for batch in result.batches() {
            for row in 0..batch.num_rows() {
                ctx.cancel.checkpoint()?;
                let group = columns::id(batch, "group_id", row)?;
                let facts = groups
                    .get_mut(&group)
                    .ok_or_else(|| invalid("native member has no physical group"))?;
                let member_type =
                    QuantityTypeId::from_id(columns::id(batch, "quantity_type_id", row)?);
                let mut scalar = quantities.quantity_type(facts.quantity_type)?.key.clone();
                scalar.shape.clear();
                if quantities.quantity_type(member_type)?.key != scalar {
                    return Err(invalid(
                        "group member differs from its complete scalar physical contract",
                    ));
                }
                let tuple = columns::ids(batch, "tuple", row)?;
                facts.valid_tuples.push(tuple.clone());
                facts
                    .members
                    .insert(tuple, columns::id(batch, "symbol_id", row)?);
            }
        }
        let descriptor_batch = loader.declared("reference.kernel_specs").await?;
        let kernels = kernels::decode(&descriptor_batch, quantities, ctx.cancel)?;
        let mut unknowns = BTreeMap::new();
        let result = loader.execute(plans::unknowns(&session)?).await?;
        for batch in result.batches() {
            for row in 0..batch.num_rows() {
                ctx.cancel.checkpoint()?;
                let system = columns::id(batch, "implicit_system_id", row)?;
                let ordinal = u16::try_from(columns::integer(batch, "ordinal", row)?)
                    .map_err(|_| invalid("implicit unknown ordinal exceeds u16"))?;
                unknowns.insert(
                    (system, ordinal),
                    QuantityTypeId::from_id(columns::id(batch, "quantity_type_id", row)?),
                );
            }
        }
        Ok(Self {
            symbols,
            domains,
            groups,
            kernels,
            unknowns,
            boolean: physical.boolean(),
            preconditions: physical.checker_owner(),
            _allocation: ReservationLease::new(loader.allocation),
        })
    }
}
struct Loader<'a> {
    session: &'a SnapshotSession,
    cancel: &'a CancellationToken,
    allocation: Box<dyn Reservation>,
}
impl Loader<'_> {
    async fn execute(&mut self, plan: LogicalPlan) -> Result<CompletedComputation, CompilerError> {
        let result = self
            .session
            .prepare_rule_plan(plan, self.cancel)?
            .execute(self.cancel)
            .await?;
        let extent = result
            .batches()
            .iter()
            .try_fold(0usize, |sum, batch| {
                sum.checked_add(batch.get_array_memory_size())?
                    .checked_add(batch.num_rows().checked_mul(256)?)
            })
            .and_then(|bytes| bytes.checked_mul(8))
            .ok_or_else(|| invalid("physical algorithm result extent overflow"))?;
        self.allocation
            .try_grow(extent)
            .map_err(pse_ids::CanonError::from)?;
        Ok(result)
    }
    async fn declared(&mut self, name: &str) -> Result<FieldCheckedBatch, CompilerError> {
        let spec = self
            .session
            .registry()
            .relation(name)
            .ok_or_else(|| invalid("physical input declaration absent"))?;
        let plan = LogicalPlanBuilder::from(plans::scan(self.session, name, "input")?)
            .sort(
                spec.primary_key
                    .iter()
                    .map(|key| col(*key).sort(true, false)),
            )
            .and_then(LogicalPlanBuilder::build)
            .map_err(plans::engine)?;
        let plan =
            declare_relation_output(plan, self.session.registry(), spec).map_err(plans::engine)?;
        let result = self.execute(plan).await?;
        let batch = result.checked_relation(self.session.registry(), spec, self.cancel)?;
        Ok(batch)
    }
}
