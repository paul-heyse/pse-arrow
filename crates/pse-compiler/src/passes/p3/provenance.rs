// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P3 retains actual construction outputs and their explicit source occurrences.
use super::invalid;
use crate::{
    CompilerError, InputBundle,
    passes::native_outputs::{self, GeneratedOutputs, SourceKey, SourceRole, Sources},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver, ReservationLease};
use pse_relations::{
    columnar::{Collection, FieldCheckedBatch},
    generated::provenance::{algorithm_source_occurrences, derivations},
};
use pse_rules::strata::{LocatedRuleInput, RuleInputLocation, native_input::NativeInput};
use pse_schema::{
    Registry,
    model::{PassSpec, RelationKey},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(super) struct Construction {
    pub(super) sources: Sources,
    pub(super) output: BTreeMap<RelationKey, Arc<NativeInput>>,
}
impl Construction {
    pub(super) fn new(
        checked: &BTreeMap<RelationKey, FieldCheckedBatch>,
        inputs: &InputBundle,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let sources = Sources::from_inputs(inputs, registry)?;
        if checked.keys().any(|key| !sources.contains_key(key)) {
            return Err(invalid(
                "P3 requires one explicit immutable role per source relation",
            ));
        }
        Ok(Self {
            sources,
            output: BTreeMap::new(),
        })
    }
    pub(super) fn replace(&mut self, values: BTreeMap<RelationKey, Arc<NativeInput>>) {
        for (key, value) in values {
            self.sources.insert(
                key,
                (
                    key.qualified_name(),
                    LocatedRuleInput {
                        relation: key,
                        location: RuleInputLocation::Native(Arc::clone(&value)),
                    },
                ),
            );
            self.output.insert(key, value);
        }
    }
    pub(super) fn checked(&self) -> BTreeMap<RelationKey, FieldCheckedBatch> {
        self.output
            .iter()
            .map(|(key, value)| (*key, value.checked().clone()))
            .collect()
    }
    pub(super) fn session(
        &self,
        base: &SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CompilerError> {
        let original = base
            .input_keys()
            .filter(|key| !self.output.contains_key(key))
            .collect();
        Ok(base
            .select_inputs(&original, cancel)?
            .with_checked_workspace(self.checked(), cancel)?)
    }
    pub(super) fn evidence(
        &self,
        registry: &Registry,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<FieldCheckedBatch, CompilerError> {
        let mut pieces = Vec::new();
        for value in self.output.values() {
            pieces.push(value.derivations().clone());
        }
        let spec = derivations::spec(registry)?;
        Ok(FieldCheckedBatch::concat_reserved(
            registry, spec, &pieces, reserver, cancel,
        )?)
    }
}

/// Couple already generated fields to exact source rows while their actual buffers
/// and algorithm argument owners are still available. Row ordinals refer to these
/// immutable arrays, and are never derived from native execution order.
pub(super) fn capture(
    columns: BTreeMap<RelationKey, FieldCheckedBatch>,
    mut source: impl FnMut(RelationKey, usize) -> Result<BTreeSet<SourceKey>, CompilerError>,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<GeneratedOutputs, CompilerError> {
    let mut occurrences = Collection::new(registry, reserver, cancel);
    occurrences.ensure::<algorithm_source_occurrences::Row>()?;
    let mut positive_sources = BTreeMap::<RelationKey, BTreeSet<SourceRole>>::new();
    let mut work = reserver.open("P3:algorithm-source-occurrences");
    for (key, batch) in &columns {
        let target = registry
            .relation_by_id(batch.relation_id())
            .ok_or_else(|| invalid("P3 algorithm target undeclared"))?;
        for ordinal in 0..batch.batch().num_rows() {
            cancel.checkpoint()?;
            let sources = source(*key, ordinal)?;
            if sources.is_empty() {
                return Err(invalid("P3 algorithm row has no actual source"));
            }
            for source in sources {
                work.try_grow(
                    source
                        .key
                        .len()
                        .checked_add(256)
                        .ok_or_else(|| invalid("P3 occurrence allocation overflow"))?,
                )
                .map_err(pse_ids::CanonError::from)?;
                let original = registry
                    .relation(&source.relation.qualified_name())
                    .ok_or_else(|| invalid("P3 algorithm source undeclared"))?;
                positive_sources
                    .entry(*key)
                    .or_default()
                    .insert(source.role());
                occurrences.push(algorithm_source_occurrences::Row {
                    output_relation_id: target.id,
                    constructed_row_ordinal: u64::try_from(ordinal)
                        .map_err(|_| invalid("P3 ordinal exceeds UInt64"))?,
                    source_port: source.port,
                    source_relation_id: original.id,
                    source_key: source.key,
                })?;
            }
        }
    }
    Ok(GeneratedOutputs {
        columns,
        positive_sources,
        read_scopes: BTreeMap::new(),
        occurrences: occurrences
            .finish()?
            .remove(&algorithm_source_occurrences::RELATION_KEY)
            .ok_or_else(|| invalid("P3 occurrence columns absent"))?,
        _work: ReservationLease::new(work),
    })
}

pub(super) async fn materialize(
    generated: GeneratedOutputs,
    construction: &Construction,
    pass: &PassSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    native_outputs::materialize(generated, &construction.sources, pass, session, cancel).await
}
