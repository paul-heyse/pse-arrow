// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native root selection and correspondence around the exact physical graph algorithm.

use super::invalid;
use crate::{
    AlgorithmContext, AlgorithmInputs, CompilerError,
    mathir_relations::{Family, RelationSink, RelationSource, SourceFamily},
    passes::native_rows::{AlgorithmInputs as NativeRows, column, engine, join, project, scan},
    quantity_relations::RelationSymbolSource,
};
use datafusion::logical_expr::{JoinType, LogicalPlan, LogicalPlanBuilder, col, lit};
use pse_catalog::session::{SnapshotSession, output::declare_relation_output};
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{
    NodeId,
    canonicalize::{CanonicalizeInput, Policy, RootEnvironment},
    infer::SymbolTypeSource,
};
use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, IndexSet, QuantityTypeId};
use pse_relations::{
    columnar::{Collection, FieldCheckedBatch, RelationRow},
    generated::{compiled, enums::ExpressionRootRole, inferred},
};
use pse_schema::model::{AlgorithmSpec, RelationKey, RelationSpec};
use std::collections::{BTreeMap, BTreeSet};

type Rows = BTreeMap<RelationKey, FieldCheckedBatch>;
type RootKey = (SemanticId, ExpressionRootRole, i64);

pub(super) async fn run(
    ctx: &AlgorithmContext<'_>,
    inputs: &AlgorithmInputs,
    pass: &AlgorithmSpec,
) -> Result<Rows, CompilerError> {
    let registry = ctx.registry.as_ref();
    let cancel = ctx.cancel;
    let checked = inputs.checked_rows(registry)?;
    let session = ctx
        .session
        .select_inputs(&BTreeSet::new(), cancel)?
        .with_checked_workspace(checked.clone(), cancel)?;
    let mut allocation = ctx.reserver.open("P10:physical-graph-workspace");
    let extent = checked.values().try_fold(0_usize, |sum, batch| {
        sum.checked_add(pse_ids::validation_extent(batch.batch())?)
            .ok_or_else(|| invalid("physical graph workspace extent overflow"))
    })?;
    allocation
        .try_grow(
            extent
                .checked_mul(6)
                .ok_or_else(|| invalid("physical graph workspace extent overflow"))?,
        )
        .map_err(pse_ids::CanonError::from)?;
    let physical = ctx.physical()?;
    if physical.neutral().is_none() || physical.boolean().is_none() {
        return Err(invalid(
            "P10 requires explicit unambiguous neutral and Boolean contracts",
        ));
    }
    let symbols = RelationSymbolSource::load(ctx, inputs).await?;
    let mut arguments = NativeRows::new(ctx.reserver, "P10:typed-graph-inputs");
    let roots = rows::<compiled::expression_roots::Row>(&mut arguments, &session, cancel).await?;
    let axes =
        rows::<compiled::expression_root_indices::Row>(&mut arguments, &session, cancel).await?;
    let expressions =
        rows::<compiled::symbol_expressions::Row>(&mut arguments, &session, cancel).await?;
    let links = rows::<inferred::math_dae_links::Row>(&mut arguments, &session, cancel).await?;
    let contributions = contribution_types(&mut arguments, &session, cancel).await?;
    let environments = root_environments(&roots, axes, &symbols, &contributions, cancel)?;
    let source = RelationSource::load_family(&session, SourceFamily::Inferred, cancel).await?;
    let source_roots = roots
        .iter()
        .map(|root| NodeId(root.node_id))
        .collect::<Vec<_>>();
    let loaded = pse_mathir::relations::load_untyped(&source, &source_roots)?;
    super::contracts::validate(
        &expressions,
        &links,
        &loaded,
        &symbols,
        physical.quantities(),
        cancel,
    )?;
    cancel.checkpoint()?;
    let graph = pse_mathir::canonicalize::canonicalize_with_environments(
        CanonicalizeInput {
            graph: &loaded.graph,
            equations: &loaded.equations,
            roots: &loaded.roots,
            symbols: &symbols,
            registry: physical.quantities(),
            kernel_bindings: &loaded.kernel_bindings,
            selections: &loaded.selections,
        },
        &environments,
        Policy::Strict,
    )?;
    canonical_rows(ctx, pass, &session, &checked, &graph, roots).await
}

fn root_environments(
    roots: &[compiled::expression_roots::Row],
    axes: Vec<compiled::expression_root_indices::Row>,
    symbols: &impl SymbolTypeSource,
    contributions: &BTreeMap<SemanticId, QuantityTypeId>,
    cancel: &CancellationToken,
) -> Result<Vec<RootEnvironment>, CompilerError> {
    let mut indices = BTreeMap::<RootKey, Vec<BoundIndexRef>>::new();
    // The native PK order supplies occurrence order. These are the quantity algorithm's
    // finite environments, not a parallel store or relation membership validator.
    for axis in axes {
        cancel.checkpoint()?;
        let domain = DomainId::from_id(axis.domain_id);
        let facts = symbols
            .domain(domain)
            .ok_or_else(|| invalid("root axis physical domain absent"))?;
        indices
            .entry((axis.owner_id, axis.role, axis.ordinal))
            .or_default()
            .push(BoundIndexRef::new(
                BoundIndexId::from_id(axis.bound_index_id),
                domain,
                facts.kind,
            ));
    }
    let mut environments = Vec::with_capacity(roots.len());
    for root in roots {
        cancel.checkpoint()?;
        let expected = match root.role {
            ExpressionRootRole::SymbolExpression | ExpressionRootRole::MethodOutput => Some(
                symbols
                    .symbol_type(root.owner_id)
                    .ok_or_else(|| invalid("root symbol physical type absent"))?,
            ),
            ExpressionRootRole::Contribution => Some(
                *contributions
                    .get(&root.owner_id)
                    .ok_or_else(|| invalid("root contribution physical type absent"))?,
            ),
            ExpressionRootRole::Display | ExpressionRootRole::Guard => None,
        };
        let indices = IndexSet::try_from_iter(indices.remove(&root_key(root)).unwrap_or_default())
            .map_err(|_| invalid("root occurrence has conflicting free-index identities"))?;
        environments.push(RootEnvironment { indices, expected });
    }
    Ok(environments)
}

async fn canonical_rows(
    ctx: &AlgorithmContext<'_>,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
    checked: &Rows,
    graph: &pse_mathir::CanonicalGraph,
    roots: Vec<compiled::expression_roots::Row>,
) -> Result<Rows, CompilerError> {
    let registry = ctx.registry.as_ref();
    let cancel = ctx.cancel;
    cancel.checkpoint()?;
    let mut sink = RelationSink::new(registry, Family::Compiled, ctx.reserver, cancel);
    pse_mathir::relations::emit(graph, &mut sink)?;
    let mut output = sink.into_batches()?;
    let canonical_roots = graph
        .roots()
        .get(..roots.len())
        .ok_or_else(|| invalid("physical algorithm omitted an input root occurrence"))?;
    let mut mapped = Collection::new(registry, ctx.reserver, cancel);
    mapped.ensure::<compiled::expression_roots::Row>()?;
    for (mut root, canonical) in roots.into_iter().zip(canonical_roots) {
        root.node_id = canonical.0;
        mapped.push(root)?;
    }
    let mapped = mapped.finish()?;
    let root_spec = compiled::expression_roots::spec(registry)?;
    let mapping = mapped
        .get(&root_spec.key)
        .ok_or_else(|| invalid("root algorithm output absent"))?
        .clone();
    let mapped_session = session.with_checked_role_inputs(
        BTreeMap::from([("canonical_roots".to_owned(), mapping.clone())]),
        cancel,
    )?;
    output.insert(root_spec.key, mapping);
    for port in &pass.outputs {
        let target = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("P10 output undeclared"))?;
        if matches!(target.key.name, "math_implicit_systems" | "math_dae_links") {
            let source = registry
                .relation(&format!("inferred.{}", target.key.name))
                .ok_or_else(|| invalid("P10 mathematical auxiliary source undeclared"))?;
            let plan = project(scan(session, source, "source")?, target, "source")?;
            output.insert(target.key, complete(plan, target, session, cancel).await?);
        } else if let std::collections::btree_map::Entry::Vacant(entry) = output.entry(target.key) {
            let batch = if let Some((field, owner, role)) = consumer(target.key.name) {
                let plan = remap(target, field, owner, role, &mapped_session)?;
                complete(plan, target, &mapped_session, cancel).await?
            } else if target.key.name.starts_with("math_") || target.key.name == "kernel_bindings" {
                FieldCheckedBatch::concat(registry, target, &[])?
            } else {
                checked
                    .get(&target.key)
                    .ok_or_else(|| invalid(format!("P10 preserved source {} absent", target.key)))?
                    .clone()
            };
            entry.insert(batch);
        }
    }
    Ok(output)
}

async fn rows<T: RelationRow>(
    arguments: &mut NativeRows,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Vec<T>, CompilerError> {
    let spec = T::relation(session.registry())?;
    let plan = LogicalPlanBuilder::from(scan(session, spec, "source")?)
        .sort(
            spec.primary_key
                .iter()
                .map(|name| column("source", name).sort(true, false)),
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    arguments
        .rows::<T>(plan, session, session.registry(), cancel)
        .await
}

async fn contribution_types(
    arguments: &mut NativeRows,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<SemanticId, QuantityTypeId>, CompilerError> {
    let registry = session.registry();
    let contributions = compiled::contributions::spec(registry)?;
    let roots = compiled::expression_roots::spec(registry)?;
    let selected = LogicalPlanBuilder::from(scan(session, roots, "root")?)
        .filter(column("root", "role").eq(lit("contribution")))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let plan = join(
        scan(session, contributions, "source")?,
        selected,
        JoinType::LeftSemi,
        &[("source.contribution_id", "root.owner_id")],
    )?;
    let plan = project(plan, contributions, "source")?;
    Ok(arguments
        .rows::<compiled::contributions::Row>(plan, session, registry, cancel)
        .await?
        .into_iter()
        .map(|row| {
            (
                row.contribution_id,
                QuantityTypeId::from_id(row.quantity_type_id),
            )
        })
        .collect())
}

fn root_key(row: &compiled::expression_roots::Row) -> RootKey {
    (row.owner_id, row.role, row.ordinal)
}
fn consumer(name: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match name {
        "symbol_expressions" | "predicate_mask_members" | "element_projection_coefficients" => {
            Some(("node_id", "symbol_id", "symbol_expression"))
        }
        "contributions" => Some(("expression_root", "contribution_id", "contribution")),
        _ => None,
    }
}
fn remap(
    target: &RelationSpec,
    field: &str,
    owner: &str,
    role: &str,
    session: &SnapshotSession,
) -> Result<LogicalPlan, CompilerError> {
    let mapping = LogicalPlanBuilder::from(session.scan_role("canonical_roots")?)
        .alias("mapping")
        .map_err(engine)?
        .filter(
            col("mapping.role")
                .eq(lit(role))
                .and(col("mapping.ordinal").eq(lit(0_u16))),
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let source = scan(session, target, "source")?;
    let joined = join(
        source,
        mapping,
        JoinType::Inner,
        &[(format!("source.{owner}").as_str(), "mapping.owner_id")],
    )?;
    LogicalPlanBuilder::from(joined)
        .project(target.columns.iter().map(|column_spec| {
            if column_spec.name() == field {
                column("mapping", "node_id").alias(field)
            } else {
                column("source", column_spec.name()).alias(column_spec.name())
            }
        }))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)
}
async fn complete(
    plan: LogicalPlan,
    target: &RelationSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, CompilerError> {
    let plan = declare_relation_output(plan, session.registry(), target).map_err(engine)?;
    Ok(session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?
        .into_checked_relation(session.registry(), target, cancel)?)
}
