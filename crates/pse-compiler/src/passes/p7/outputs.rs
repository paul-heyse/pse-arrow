// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed callback columns and their actual graph/source correspondence.
use super::{RealizationOutput, Realizer, Support, invalid};
use crate::{
    CompilerError,
    mathir_relations::{Family, RelationSink},
    passes::{
        native_graph::{identity, ordinal},
        native_outputs::{self, SourceKey, Sources},
    },
};
use pse_mathir::{NodeId, relations::LoadedMath};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationKey;
use std::collections::{BTreeMap, BTreeSet};

impl Realizer<'_> {
    pub(super) fn source_keys(
        &self,
        support: &BTreeSet<Support>,
    ) -> Result<BTreeSet<SourceKey>, CompilerError> {
        located(&self.inventory.source_keys, &self.sources, support)
    }
    pub(super) async fn finish(mut self) -> Result<RealizationOutput, CompilerError> {
        let mut symbols = self.symbols.values().cloned().collect::<Vec<_>>();
        symbols.sort_by_key(|symbol| symbol.symbol_id);
        for (ordinal, mut row) in symbols.into_iter().enumerate() {
            row.ordinal = u64::try_from(ordinal).map_err(|_| invalid("symbol ordinal overflow"))?;
            self.active_support = self
                .symbol_support
                .get(&row.symbol_id)
                .cloned()
                .ok_or_else(|| invalid("realized symbol has no actual source support"))?;
            self.append("compiled.symbols", row)?;
        }
        let loaded = LoadedMath {
            graph: std::mem::take(&mut self.graph),
            equations: std::mem::take(&mut self.equations),
            roots: Vec::new(),
            selections: std::mem::take(&mut self.selections),
            kernel_bindings: std::mem::take(&mut self.kernels),
            node_mapping: BTreeMap::new(),
        };
        let mut sink =
            RelationSink::new(self.registry, Family::Inferred, self.reserver, self.cancel);
        pse_mathir::relations::emit_loaded(&loaded, &mut sink)?;
        for input in sink.into_batches()?.into_values() {
            let keys = &self.inventory.source_keys;
            let sources = &self.sources;
            let nodes = &self.node_support;
            let equations = &self.equation_support;
            let kernels = &self.kernel_support;
            self.output.append_checked(&input, |input, row| {
                let mut support = BTreeSet::new();
                for name in ["node_id", "parent_node_id"] {
                    if let Some(node) = ordinal(input, name, row)? {
                        support.extend(nodes.get(&NodeId(node)).into_iter().flatten().copied());
                    }
                }
                for (name, values) in [
                    ("indexed_equation_id", equations),
                    ("kernel_binding_id", kernels),
                ] {
                    if let Some(id) = identity(input, name, row)? {
                        support.extend(values.get(&id).into_iter().flatten().copied());
                    }
                }
                located(keys, sources, &support)
            })?;
        }
        // Empty outputs are actual declared Arrow constructions. They carry no row
        // witness and cannot stand in for an algorithm that failed to run.
        for port in &self.spec.outputs {
            let spec = self
                .registry
                .relation(&port.relation)
                .ok_or_else(|| invalid("P7 output contract absent"))?;
            let empty = FieldCheckedBatch::concat_reserved(
                self.registry,
                spec,
                &[],
                self.reserver,
                self.cancel,
            )?;
            self.output.append_checked(&empty, |_, _| {
                Err(invalid("declared empty output unexpectedly has a row"))
            })?;
        }
        let mut rows = BTreeMap::new();
        let mut derivations = Vec::new();
        for (key, input) in native_outputs::materialize(
            self.output.finish()?,
            &self.sources,
            self.spec,
            &self.session,
            self.cancel,
        )
        .await?
        {
            rows.insert(key, input.checked().clone());
            derivations.push(input.derivations().clone().into_batch());
            self.sources.replace_native(key, input)?;
        }
        Ok(RealizationOutput {
            rows,
            derivations,
            sources: self.sources,
        })
    }
}

fn located(
    keys: &BTreeMap<RelationKey, Vec<pse_ids::ContentHash>>,
    sources: &Sources,
    support: &BTreeSet<Support>,
) -> Result<BTreeSet<SourceKey>, CompilerError> {
    sources.locate(
        support
            .iter()
            .map(|(relation, index)| {
                keys.get(relation)
                    .and_then(|rows| rows.get(*index))
                    .copied()
                    .map(|key| (*relation, key))
                    .ok_or_else(|| {
                        invalid("algorithm source ordinal is outside its actual coupled projection")
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
    )
}
