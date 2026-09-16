// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P3 retains actual construction outputs and their explicit source occurrences.
use super::invalid;
use crate::{
    CompilerError, InputBundle,
    passes::native_outputs::{self, GeneratedOutputs, Sources},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver};
use pse_relations::{columnar::FieldCheckedBatch, generated::provenance::derivations};
use pse_rules::strata::{LocatedRuleInput, RuleInputLocation, native_input::NativeInput};
use pse_schema::{
    Registry,
    model::{PassSpec, RelationKey},
};
use std::{collections::BTreeMap, sync::Arc};

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

pub(super) async fn materialize(
    generated: GeneratedOutputs,
    construction: &Construction,
    pass: &PassSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    native_outputs::materialize(generated, &construction.sources, pass, session, cancel).await
}
