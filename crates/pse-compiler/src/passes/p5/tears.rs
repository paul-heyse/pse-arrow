// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native edge/cost correspondence feeds the single finite graph heuristic.
use super::super::p4::invalid;
use crate::{
    CompilerError, PassContext,
    passes::{
        native_outputs::{OutputRows, Sources},
        native_rows::{AlgorithmInputs, column, engine, join},
    },
};
use datafusion::{
    arrow::array::{Array, Float64Array, StringArray},
    functions::core::expr_fn::coalesce,
    logical_expr::{JoinType, LogicalPlanBuilder, col, lit},
};
use petgraph::{
    algo::{greedy_feedback_arc_set, is_cyclic_directed},
    graph::DiGraph,
};
use pse_catalog::session::{SnapshotSession, output::declare_relation_projection, scalar};
use pse_ids::{Reservation, SemanticId};
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::{enums::TearMethod, inferred as i, normalized as n},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) async fn emit(
    ctx: &PassContext<'_>,
    sources: &Sources,
    session: &SnapshotSession,
    work: &mut dyn Reservation,
    output: &mut OutputRows<'_>,
) -> Result<(), CompilerError> {
    output.ensure::<i::tear_candidates::Row>()?;
    let edge_spec = i::topology_edges::Row::relation(ctx.registry)?;
    let connection_spec = n::connections::Row::relation(ctx.registry)?;
    let edges = LogicalPlanBuilder::from(session.scan_role(&edge_spec.key.qualified_name())?)
        .alias("edge")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let connections =
        LogicalPlanBuilder::from(session.scan_role(&connection_spec.key.qualified_name())?)
            .alias("connection")
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
    let joined = join(
        edges,
        connections,
        JoinType::Left,
        &[("edge.connection_id", "connection.connection_id")],
    )?;
    let mut arguments = AlgorithmInputs::new(ctx.reserver, "P5:topology-arguments");
    crate::passes::native_rows::reject(
        LogicalPlanBuilder::from(joined.clone())
            .filter(column("connection", "connection_id").is_null())
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?,
        session,
        ctx.cancel,
        "topology edge has no actual connection declaration",
    )
    .await?;
    let mut fields = edge_spec
        .columns
        .iter()
        .map(|field| column("edge", field.name()).alias(field.name()))
        .collect::<Vec<_>>();
    fields.extend([
        scalar::key(
            edge_spec
                .primary_key
                .iter()
                .map(|name| (*name, column("edge", name)))
                .collect(),
        )
        .alias("edge_key"),
        scalar::key(
            connection_spec
                .primary_key
                .iter()
                .map(|name| (*name, column("connection", name)))
                .collect(),
        )
        .alias("connection_key"),
        coalesce(vec![column("connection", "tear_cost"), lit(1.0_f64)]).alias("actual_cost"),
    ]);
    let plan = LogicalPlanBuilder::from(joined)
        .project(fields)
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let plan = declare_relation_projection(
        plan,
        ctx.registry,
        edge_spec,
        vec![col("edge_key"), col("connection_key"), col("actual_cost")],
    )
    .map_err(engine)?;
    let completed = arguments.execute(plan, session, ctx.cancel).await?;
    let mut actual = BTreeMap::new();
    let mut nodes = BTreeSet::new();
    let mut support = BTreeSet::new();
    let positions = (0..edge_spec.columns.len()).collect::<Vec<_>>();
    for batch in completed.batches() {
        let width = edge_spec.columns.len();
        let edge_keys = batch
            .column(width)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| invalid("topology source key is not Utf8"))?;
        let connection_keys = batch
            .column(width + 1)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| invalid("connection source key is not Utf8"))?;
        let costs = batch
            .column(width + 2)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| invalid("tear cost is not Float64"))?;
        let checked =
            FieldCheckedBatch::admit_owned_projection(ctx.registry, edge_spec, batch, &positions)?;
        for (position, edge) in i::topology_edges::Row::rows(&checked)?
            .into_iter()
            .enumerate()
        {
            ctx.cancel.checkpoint()?;
            if edge_keys.is_null(position)
                || connection_keys.is_null(position)
                || costs.is_null(position)
            {
                return Err(invalid("native topology occurrence is incomplete"));
            }
            let cost = costs.value(position);
            if !cost.is_finite() || cost < 0.0 {
                return Err(invalid("tear cost must be finite and nonnegative"));
            }
            if actual
                .insert(
                    edge.connection_id,
                    (edge.from_instance_id, edge.to_instance_id, cost),
                )
                .is_some()
            {
                return Err(invalid("one connection produced multiple unit graph edges"));
            }
            nodes.extend([edge.from_instance_id, edge.to_instance_id]);
            support.insert((edge_spec.key, edge_keys.value(position).to_owned()));
            support.insert((
                connection_spec.key,
                connection_keys.value(position).to_owned(),
            ));
        }
    }
    work.try_grow(
        actual
            .len()
            .checked_mul(8192)
            .ok_or_else(|| invalid("topology graph extent overflow"))?,
    )
    .map_err(pse_ids::CanonError::from)?;
    // petgraph 0.8.3 ignores weights. The admitted policy cost remains visible;
    // stable semantic node/edge order controls every heuristic tie. This is not
    // a minimum-cost or weighted-optimality claim.
    let mut graph =
        DiGraph::<SemanticId, SemanticId, usize>::with_capacity(nodes.len(), actual.len());
    let mut indices = BTreeMap::new();
    for id in nodes {
        indices.insert(
            id,
            graph
                .try_add_node(id)
                .map_err(|error| invalid(error.to_string()))?,
        );
    }
    for (id, (from, to, _)) in &actual {
        graph
            .try_add_edge(indices[from], indices[to], *id)
            .map_err(|error| invalid(error.to_string()))?;
    }
    ctx.cancel.checkpoint()?;
    let selected = greedy_feedback_arc_set(&graph)
        .map(|edge| *edge.weight())
        .collect::<BTreeSet<_>>();
    let groups = selected
        .iter()
        .map(|id| {
            let (from, to, _) = actual[id];
            (from, to)
        })
        .collect::<BTreeSet<_>>();
    let mut retained =
        DiGraph::<SemanticId, SemanticId, usize>::with_capacity(indices.len(), actual.len());
    let mut retained_indices = BTreeMap::new();
    for id in indices.keys() {
        retained_indices.insert(
            *id,
            retained
                .try_add_node(*id)
                .map_err(|error| invalid(error.to_string()))?,
        );
    }
    for (id, (from, to, _)) in &actual {
        if !groups.contains(&(*from, *to)) {
            retained
                .try_add_edge(retained_indices[from], retained_indices[to], *id)
                .map_err(|error| invalid(error.to_string()))?;
        }
    }
    ctx.cancel.checkpoint()?;
    if is_cyclic_directed(&retained) {
        return Err(invalid(
            "removing selected actual tear edge groups did not make the topology acyclic",
        ));
    }
    let support = sources.locate(support)?;
    for (ordinal, (id, (from, to, cost))) in actual.iter().enumerate() {
        ctx.cancel.checkpoint()?;
        let group = pse_ids::named_id(
            *from,
            &format!("pse:topology-edge-group:v1:{}", to.to_hex()),
        );
        output.push(
            i::tear_candidates::Row {
                connection_id: *id,
                edge_group_id: group,
                cost: *cost,
                chosen: groups.contains(&(*from, *to)),
                method: TearMethod::FeedbackArcSet,
                ordinal: u64::try_from(ordinal)
                    .map_err(|_| invalid("tear ordinal exceeds UInt64"))?,
                derivation_id: SemanticId::NIL,
            },
            &support,
        )?;
    }
    Ok(())
}
