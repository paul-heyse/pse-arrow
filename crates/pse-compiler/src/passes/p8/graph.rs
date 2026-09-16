// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Consume the current indexed graph and transform the roots used by this pass.
use super::{expansion::invalid, outputs::Output};
use crate::{
    CompilerError, PassContext,
    mathir_relations::{RelationSource, SourceFamily},
    passes::{
        native_construction::{append, c, error, join, prefix, project},
        native_graph::{identity, node_mapping, ordinal, root_field},
        native_outputs::Sources,
        native_rows::AlgorithmInputs,
    },
};
use datafusion::arrow::array::FixedSizeBinaryArray;
use datafusion::{
    arrow::array::Array,
    logical_expr::{JoinType, col},
};
use pse_catalog::session::{SnapshotSession, output::declare_relation_projection, scalar};
use pse_mathir::{NodeId, relations::LoadedMath};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::{Namespace, PassSpec, RelationKey};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Batches = BTreeMap<RelationKey, FieldCheckedBatch>;
pub(super) fn load(inputs: &Batches, ctx: &PassContext<'_>) -> Result<LoadedMath, CompilerError> {
    let mut graph = inputs.clone();
    for name in [
        "math_dae_links",
        "math_implicit_systems",
        "math_equations",
        "math_objectives",
        "math_complementarity",
    ] {
        let spec = ctx
            .registry
            .relation(&format!("inferred.{name}"))
            .ok_or_else(|| invalid("graph side-table undeclared"))?;
        let batch = inputs
            .get(&spec.key)
            .ok_or_else(|| invalid("current indexed graph side-table omitted"))?;
        if !matches!(name, "math_dae_links" | "math_implicit_systems")
            && batch.batch().num_rows() != 0
        {
            return Err(invalid("scalar solver objects precede their owning pass"));
        }
        // DAE/system relations have semantic IDs and are carried by their own
        // declared ports; the indexed expression loader consumes its graph tables.
        graph.insert(
            spec.key,
            FieldCheckedBatch::concat_reserved(ctx.registry, spec, &[], ctx.reserver, ctx.cancel)?,
        );
    }
    Ok(pse_mathir::relations::load_untyped(
        &RelationSource::from_checked(&graph, ctx.registry, SourceFamily::Inferred)?,
        &[],
    )?)
}

pub(super) async fn inputs(
    batches: &Batches,
    loaded: &LoadedMath,
    output: &mut Output<'_>,
    arguments: &mut AlgorithmInputs,
    sources: &Sources,
    session: &SnapshotSession,
    ctx: &PassContext<'_>,
    pass: &PassSpec,
) -> Result<(), CompilerError> {
    arguments.reserve(
        loaded
            .node_mapping
            .len()
            .checked_mul(2048)
            .ok_or_else(|| invalid("law graph mapping extent overflow"))?,
    )?;
    let session = arguments
        .retain_plan(
            node_mapping(&loaded.node_mapping)?,
            session,
            "P8:node-map",
            ctx.cancel,
        )
        .await?;
    for port in &pass.outputs {
        let spec = ctx
            .registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("law output declaration absent"))?;
        let graph = spec.key.namespace == Namespace::Inferred
            && (spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings")
            && !matches!(spec.key.name, "math_dae_links" | "math_implicit_systems");
        // Only graph nodes and their roots change in this construction. Other
        // completed relations keep their existing owners and source derivations.
        if !graph && root_field(&spec.key).is_none() {
            continue;
        }
        let Some(batch) = batches.get(&spec.key) else {
            continue;
        };
        let (role, _) = sources
            .get(&spec.key)
            .ok_or_else(|| invalid("current graph source role absent"))?;
        let session = session.with_checked_role_inputs(
            BTreeMap::from([("P8:graph-input".to_owned(), batch.clone())]),
            ctx.cancel,
        )?;
        let mut plan = append(
            session.scan_role("P8:graph-input")?,
            [scalar::key(
                spec.id,
                spec.primary_key
                    .iter()
                    .map(|name| (*name, col(*name)))
                    .collect(),
            )
            .alias("__graph_source_key")],
        )?;
        if let Some(field) = root_field(&spec.key) {
            plan = join(
                plan,
                prefix(session.scan_computation_role("P8:node-map")?, "mapped")?,
                JoinType::Left,
                [col(field).eq(c("mapped", "old_node"))],
            )?;
            let required = session
                .scalar_function("pse_require_nonnull")?
                .call(vec![c("mapped", "new_node")]);
            plan = project(
                plan,
                spec.columns
                    .iter()
                    .map(|column| {
                        if column.name() == field {
                            required.clone().alias(field)
                        } else {
                            col(column.name())
                        }
                    })
                    .chain([col("__graph_source_key")]),
            )?;
        }
        let plan =
            declare_relation_projection(plan, ctx.registry, spec, vec![col("__graph_source_key")])
                .map_err(error)?;
        let complete = arguments.execute(plan, &session, ctx.cancel).await?;
        for batch in complete.batches() {
            let values = FieldCheckedBatch::admit_owned_projection(
                ctx.registry,
                spec,
                batch,
                &(0..spec.columns.len()).collect::<Vec<_>>(),
            )?;
            let keys = batch
                .batch()
                .column(spec.columns.len())
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| invalid("graph source key storage differs"))?;
            let mut support = Vec::new();
            for row in 0..values.batch().num_rows() {
                if keys.is_null(row) {
                    return Err(invalid("graph source key is null"));
                }
                let source = crate::passes::native_outputs::SourceKey {
                    relation: spec.key,
                    port: role.clone(),
                    key: crate::passes::native_rows::key_value(keys.value(row))?,
                };
                if graph {
                    for field in ["node_id", "parent_node_id"] {
                        if let Some(old) = ordinal(&values, field, row)? {
                            let node = loaded.node_mapping.get(&NodeId(old)).ok_or_else(|| {
                                invalid("input graph node is outside its actual mapping")
                            })?;
                            output
                                .nodes
                                .entry(*node)
                                .or_default()
                                .insert(source.clone());
                        }
                    }
                    for (field, map) in [
                        ("indexed_equation_id", &mut output.equations),
                        ("kernel_binding_id", &mut output.kernels),
                    ] {
                        if let Some(id) = identity(&values, field, row)? {
                            map.entry(id).or_default().insert(source.clone());
                        }
                    }
                }
                support.push(BTreeSet::from([source]));
            }
            if !graph {
                output.columns.append_checked(&values, |_, row| {
                    support
                        .get(row)
                        .cloned()
                        .ok_or_else(|| invalid("graph source/result correspondence absent"))
                })?;
            }
        }
    }
    Ok(())
}
