// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generated law columns retain the selected arguments and graph correspondence.
use super::{expansion::invalid, inventory::Located};
use crate::{
    AlgorithmContext, CompilerError,
    mathir_relations::{Family, RelationSink},
    passes::{
        native_graph::{identity, ordinal},
        native_outputs::{self, OutputRows, SourceKey, Sources},
    },
};
use pse_ids::SemanticId;
use pse_mathir::{ExprGraph, NodeId, relations::LoadedMath};
use pse_relations::{
    RecordBatch,
    columnar::{FieldCheckedBatch, RelationRow},
};
use pse_schema::model::{AlgorithmSpec, RelationKey};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Support = BTreeSet<SourceKey>;
pub(super) struct Output<'a> {
    pub columns: OutputRows<'a>,
    pub active: Support,
    pub nodes: BTreeMap<NodeId, Support>,
    pub equations: BTreeMap<SemanticId, Support>,
    pub kernels: BTreeMap<SemanticId, Support>,
}
impl<'a> Output<'a> {
    pub(super) fn new(ctx: &AlgorithmContext<'a>) -> Result<Self, CompilerError> {
        Ok(Self {
            columns: OutputRows::new(ctx.registry, ctx.reserver, ctx.cancel)?,
            active: Support::new(),
            nodes: BTreeMap::new(),
            equations: BTreeMap::new(),
            kernels: BTreeMap::new(),
        })
    }
    pub(super) fn use_row<T>(&mut self, row: &Located<T>) {
        row.support(&mut self.active);
    }
    pub(super) fn push<T: RelationRow>(&mut self, row: T) -> Result<(), CompilerError> {
        self.columns.push(row, &self.active)
    }
    pub(super) fn rows<T: RelationRow>(&mut self) -> Result<Vec<T>, CompilerError> {
        self.columns.rows::<T>()
    }
    pub(super) fn source_graph(
        &mut self,
        graph: &ExprGraph,
        root: NodeId,
    ) -> Result<(), CompilerError> {
        for id in reachable(graph, [root])? {
            self.active.extend(
                self.nodes
                    .get(&id)
                    .ok_or_else(|| invalid("law operand has no actual graph source"))?
                    .iter()
                    .cloned(),
            );
        }
        Ok(())
    }
    pub(super) fn record_graph(
        &mut self,
        graph: &ExprGraph,
        roots: impl IntoIterator<Item = NodeId>,
    ) -> Result<(), CompilerError> {
        for id in reachable(graph, roots)? {
            self.nodes
                .entry(id)
                .or_default()
                .extend(self.active.iter().cloned());
        }
        Ok(())
    }
    pub(super) async fn finish(
        mut self,
        loaded: &LoadedMath,
        retained: &super::graph::Batches,
        ctx: &AlgorithmContext<'_>,
        pass: &AlgorithmSpec,
        sources: &Sources,
        session: &pse_catalog::session::SnapshotSession,
    ) -> Result<(BTreeMap<RelationKey, FieldCheckedBatch>, Vec<RecordBatch>), CompilerError> {
        let mut sink = RelationSink::new(ctx.registry, Family::Inferred, ctx.reserver, ctx.cancel);
        pse_mathir::relations::emit_loaded(loaded, &mut sink)?;
        for batch in sink.into_batches()?.into_values() {
            self.columns.append_checked(&batch, |batch, row| {
                let mut support = Support::new();
                for field in ["node_id", "parent_node_id"] {
                    if let Some(id) = ordinal(batch, field, row)? {
                        support.extend(self.nodes.get(&NodeId(id)).into_iter().flatten().cloned());
                    }
                }
                for (field, values) in [
                    ("indexed_equation_id", &self.equations),
                    ("binding_id", &self.kernels),
                ] {
                    if let Some(id) = identity(batch, field, row)? {
                        support.extend(values.get(&id).into_iter().flatten().cloned());
                    }
                }
                Ok(support)
            })?;
        }
        let mut rows = BTreeMap::new();
        let mut derivations = Vec::new();
        for (key, input) in
            native_outputs::materialize(self.columns.finish()?, sources, pass, session, ctx.cancel)
                .await?
        {
            rows.insert(key, input.checked().clone());
            derivations.push(input.derivations().clone().into_batch());
        }
        for port in &pass.outputs {
            let relation = ctx
                .registry
                .relation(&port.relation)
                .ok_or_else(|| invalid("law output declaration absent"))?;
            let produced = rows.remove(&relation.key);
            let rebuilt = super::graph::rebuilds_graph(relation.key)
                || crate::passes::native_graph::root_field(&relation.key).is_some();
            let prior = retained.get(&relation.key);
            let inputs = match (produced, prior, rebuilt) {
                (Some(new), Some(prior), false) => vec![prior.clone(), new],
                (Some(new), _, true) | (Some(new), None, false) => vec![new],
                (None, Some(prior), _) => vec![prior.clone()],
                (None, None, _) => vec![],
            };
            // Existing rows retain their existing derivations. Only the new rows
            // above generated support; native Arrow concatenation keeps the full
            // relation without reminting provenance for unchanged facts.
            rows.insert(
                relation.key,
                FieldCheckedBatch::concat_reserved(
                    ctx.registry,
                    relation,
                    &inputs,
                    ctx.reserver,
                    ctx.cancel,
                )?,
            );
        }
        Ok((rows, derivations))
    }
}
fn reachable(
    graph: &ExprGraph,
    roots: impl IntoIterator<Item = NodeId>,
) -> Result<BTreeSet<NodeId>, CompilerError> {
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if seen.insert(id) {
            pending.extend(graph.node(id)?.children.iter().copied());
        }
    }
    Ok(seen)
}
