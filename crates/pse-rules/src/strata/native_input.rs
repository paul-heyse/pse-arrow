// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable native construction outputs and exact located source correspondences.
//! Completion establishes execution and field construction, never producer semantics
//! by a digest, a callback replay, or an arbitrary result-wrapper constructor.

mod projection;
mod provenance;
mod union;
mod witness;

#[cfg(test)]
mod tests;

use super::{LocatedRuleInput, relational};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col};
use pse_catalog::session::{SnapshotSession, output::declare_relation_output, scalar};
use pse_ids::{CancellationToken, ReservationLease, SemanticId};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::{ColumnRole, RelationKey, RelationSpec};
use std::{collections::BTreeMap, sync::Arc};

/// An exact source binding and its key values in the completed native output.
#[derive(Clone, Debug)]
pub struct NativeWitness {
    /// Semantic input role retained in provenance.
    pub port: String,
    /// Actual immutable source member or completed native source.
    pub input: LocatedRuleInput,
    /// Completed-plan columns in the source declaration's primary-key order.
    /// `Some` requires actual row membership, including `Some(vec![])` for an
    /// explicitly declared singleton key. `None` records a complete read scope
    /// for absence/invalidation; it never substitutes for positive row support.
    pub key_columns: Option<Vec<String>>,
    /// Native condition selecting rows that carry this exact witness. Omitted
    /// witnesses are not inferred from nullable keys. A separate read-scope witness
    /// explicitly retains a complete source read scope for absence/invalidation.
    pub when: Option<Expr>,
}

/// Completed values and source correspondence. Arrow buffers own their allocation
/// claims; predecessor sessions, plans and graph objects are released after construction.
#[derive(Debug)]
pub struct NativeInput {
    relation: RelationKey,
    pass_id: SemanticId,
    checked: FieldCheckedBatch,
    support: FieldCheckedBatch,
    derivations: FieldCheckedBatch,
    registry: Arc<pse_schema::Registry>,
    _receipt: Arc<ReservationLease>,
}

impl NativeInput {
    /// Execute a native construction once and project its actual values and located
    /// witnesses. Column pairs map destination names to completed-plan column names.
    /// No row callback or arbitrary `RecordBatch` can mint this carrier.
    ///
    /// # Errors
    /// Invalid bindings, missing/conflicting output keys, false source witnesses,
    /// field obligations, engine errors, cancellation or resource exhaustion.
    pub async fn build(
        plan: LogicalPlan,
        output: RelationKey,
        pass_id: SemanticId,
        columns: Vec<(String, String)>,
        witnesses: Vec<NativeWitness>,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Arc<Self>, RuleError> {
        Self::build_inner(plan, output, pass_id, columns, witnesses, session, cancel)
            .await
            .map_err(|source| RuleError::Execution {
                rule: output.qualified_name(),
                phase: "native construction",
                source: Box::new(source),
            })
    }

    async fn build_inner(
        plan: LogicalPlan,
        output: RelationKey,
        pass_id: SemanticId,
        columns: Vec<(String, String)>,
        mut witnesses: Vec<NativeWitness>,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Arc<Self>, RuleError> {
        let registry = session.registry();
        let mut receipt = session.reserver().open("rules:native-input-receipt");
        receipt
            .try_grow(receipt_extent(&witnesses, &columns)?)
            .map_err(pse_catalog::CatalogError::from)?;
        let target = registry
            .relation(&output.qualified_name())
            .filter(|spec| spec.key == output)
            .ok_or_else(|| internal("native construction output declaration absent"))?;
        if columns.len() != target.columns.len()
            || columns
                .iter()
                .zip(&target.columns)
                .any(|((name, _), column)| name != column.name())
        {
            return Err(internal(
                "native output column projection differs from declared order",
            ));
        }
        for witness in &witnesses {
            witness.validate(session, cancel)?;
        }
        let plan = projection::inputs(plan, target, &columns, &mut witnesses)?;
        let complete = session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?;
        let mut ordinal = session.computation_roles().count();
        let role = loop {
            let role = format!("native-input:{}:{ordinal}", output.qualified_name());
            if !session.computation_roles().any(|actual| actual == role) {
                break role;
            }
            ordinal = ordinal
                .checked_add(1)
                .ok_or_else(|| internal("native role ordinal overflow"))?;
        };
        let owner =
            session.with_computation_roles(BTreeMap::from([(role.clone(), complete)]), cancel)?;
        let raw = owner.scan_computation_role(&role)?;
        let projected = LogicalPlanBuilder::from(raw.clone())
            .project(output_columns(target, &columns, pass_id, registry)?)
            .map_err(engine)?
            .distinct()
            .map_err(engine)?
            .build()
            .map_err(engine)?;
        let checked = materialize_with_constraint(
            projected,
            target,
            &owner,
            cancel,
            |values| witness::duplicate_keys(values, target),
            "native construction has conflicting rows for one declared key",
        )
        .await?;
        let support = witness::mapping(raw, target, &columns, &witnesses, &owner, cancel).await?;
        let derivations =
            provenance::materialize(&checked, &support, output, pass_id, &owner, cancel).await?;
        receipt.shrink(receipt.size().saturating_sub(size_of::<Self>()));
        Ok(Arc::new(Self {
            relation: output,
            pass_id,
            checked,
            support,
            derivations,
            registry: Arc::clone(registry),
            _receipt: ReservationLease::new(receipt),
        }))
    }

    /// Actual checked fields under the generated or declared output contract.
    pub const fn checked(&self) -> &FieldCheckedBatch {
        &self.checked
    }
    /// Actual rows, retaining only their Arrow buffer leases.
    pub fn batch(&self) -> &pse_relations::RecordBatch {
        self.checked.batch()
    }
    /// Materialized typed provenance, available after all producing plans are dropped.
    pub const fn derivations(&self) -> &FieldCheckedBatch {
        &self.derivations
    }
    /// Exact typed output-to-input keys and durable selections, independent of plans.
    pub const fn support_mapping(&self) -> &FieldCheckedBatch {
        &self.support
    }
    pub(super) fn validate(
        &self,
        relation: RelationKey,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(), RuleError> {
        self.validate_owner(relation, session, cancel)?;
        session.validate_workspace_input_values(&relation, self.batch())?;
        Ok(())
    }
    pub(super) fn validate_owner(
        &self,
        relation: RelationKey,
        session: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<(), RuleError> {
        if relation != self.relation {
            return Err(internal(
                "native input relation differs from its actual output",
            ));
        }
        cancel
            .checkpoint()
            .map_err(pse_catalog::CatalogError::from)?;
        if !Arc::ptr_eq(session.registry(), &self.registry) {
            return Err(internal(
                "native facts belong to a different registry declaration",
            ));
        }
        Ok(())
    }
}

fn output_columns(
    target: &RelationSpec,
    columns: &[(String, String)],
    pass: SemanticId,
    registry: &pse_schema::Registry,
) -> Result<Vec<Expr>, RuleError> {
    let key = scalar::key(
        target.id,
        target
            .primary_key
            .iter()
            .map(|name| {
                columns
                    .iter()
                    .find(|(target, _)| target == name)
                    .map(|(_, source)| (*name, col(source)))
                    .ok_or_else(|| internal("native output key projection absent"))
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    let derivation = derivation_id(key, pass, registry)?;
    Ok(columns
        .iter()
        .zip(&target.columns)
        .map(|((name, source), column)| {
            if column.name() == "derivation_id" && column.role() == ColumnRole::Provenance {
                derivation.clone().alias(name)
            } else if name == source {
                col(source)
            } else {
                col(source).alias(name)
            }
        })
        .collect())
}

/// One derivation identity for both embedded provenance and external support heads.
/// The key is calculated from the actual output, never from its execution position.
fn derivation_id(
    key: Expr,
    pass: SemanticId,
    registry: &pse_schema::Registry,
) -> Result<Expr, RuleError> {
    Ok(scalar::named_id(relational::id(pass, registry)?, key))
}

fn receipt_extent(
    witnesses: &[NativeWitness],
    columns: &[(String, String)],
) -> Result<usize, RuleError> {
    let mut bytes = size_of::<NativeInput>();
    for witness in witnesses {
        // This construction scratch claim is released once typed support is materialized.
        bytes = bytes
            .checked_add(size_of::<NativeWitness>())
            .and_then(|n| n.checked_add(witness.port.capacity()))
            .ok_or_else(|| internal("native witness metadata extent overflow"))?;
        for key in witness.key_columns.iter().flatten() {
            bytes = bytes
                .checked_add(size_of::<String>())
                .and_then(|n| n.checked_add(key.capacity()))
                .ok_or_else(|| internal("native witness key extent overflow"))?;
        }
    }
    for (name, source) in columns {
        bytes = bytes
            .checked_add(size_of::<(String, String)>())
            .and_then(|n| n.checked_add(name.capacity()))
            .and_then(|n| n.checked_add(source.capacity()))
            .ok_or_else(|| internal("native output metadata extent overflow"))?;
    }
    Ok(bytes)
}

/// Check the actual completed values through an immutable native provider. Checking
/// the producer and then running it again duplicates planning and can observe a
/// different execution. No checked result escapes until its relational check passes.
async fn materialize_with_constraint(
    plan: LogicalPlan,
    target: &RelationSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    violations: impl FnOnce(LogicalPlan) -> Result<LogicalPlan, RuleError>,
    reason: &str,
) -> Result<FieldCheckedBatch, RuleError> {
    let plan = declare_relation_output(plan, session.registry(), target).map_err(engine)?;
    let complete = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?;
    let checked = complete.into_checked_relation(session.registry(), target, cancel)?;
    let mut role = format!("__native_completed:{}", target.id);
    while session.input_roles().any(|(name, _)| name == role) {
        role.push('_');
    }
    let owner = session
        .with_checked_role_inputs(BTreeMap::from([(role.clone(), checked.clone())]), cancel)?;
    require_empty(violations(owner.scan_role(&role)?)?, &owner, cancel, reason).await?;
    Ok(checked)
}

async fn require_empty(
    plan: LogicalPlan,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    reason: &str,
) -> Result<(), RuleError> {
    // This contract observes existence only. Express that through native projection
    // and limit so column pruning can discard unrelated payload work and metadata;
    // every predicate/join key still participates in the zero-violation decision.
    let plan = LogicalPlan::Projection(
        datafusion_expr::Projection::try_new(vec![datafusion_expr::lit(true)], Arc::new(plan))
            .map_err(engine)?,
    );
    let plan = LogicalPlanBuilder::from(plan)
        .limit(0, Some(1))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let result = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?;
    if result.batches().iter().any(|batch| batch.num_rows() != 0) {
        return Err(internal(reason));
    }
    Ok(())
}
