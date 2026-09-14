// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Output bundles: complete, immutable, with every declared port (blueprint §14.1).
//!
//! A bundle contains all declared output ports, including explicit empty relations. Each
//! stage publishes a complete replacement rather than overwriting another stage's artifact.
//!
use super::{BoundInput, InputBundle, PassOutput, dag::invalid};
use crate::CompilerError;
use pse_catalog::{EncodingPolicy, RelationContract, Snapshot};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{PassSpec, PortSource, RelationKey},
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl BoundInput {
    /// Resolve a relation from an admitted immutable snapshot and validate its full contract.
    /// # Errors
    /// The snapshot lacks the exact relation or its declaration differs.
    pub fn bind(
        snapshot: Arc<Snapshot>,
        key: RelationKey,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let spec = registry
            .relation(&key.qualified_name())
            .filter(|spec| spec.key == key)
            .ok_or_else(|| invalid("bound relation version is undeclared"))?;
        let mut candidates = snapshot
            .relations()
            .values()
            .filter(|relation| relation.contract().canonical.relation_id == spec.id);
        let relation = Arc::clone(
            candidates
                .next()
                .ok_or_else(|| invalid("bound snapshot lacks declared relation"))?,
        );
        if candidates.next().is_some() {
            return Err(invalid(
                "relation has multiple output ports; bind its declared port explicitly",
            ));
        }
        relation
            .contract()
            .validate_against_registry(registry, spec)?;
        Ok(Self { snapshot, relation })
    }

    /// Resolve the exact declared output port, even when a stage has repeated schemas.
    /// # Errors
    /// Missing port, different relation version or incompatible actual contract.
    pub fn bind_port(
        snapshot: Arc<Snapshot>,
        key: RelationKey,
        port: &str,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        let spec = registry
            .relation(&key.qualified_name())
            .filter(|spec| spec.key == key)
            .ok_or_else(|| invalid("bound relation version is undeclared"))?;
        let relation = Arc::clone(
            snapshot
                .relations()
                .get(port)
                .filter(|relation| relation.contract().canonical.relation_id == spec.id)
                .ok_or_else(|| invalid("bound snapshot lacks declared output port"))?,
        );
        relation
            .contract()
            .validate_against_registry(registry, spec)?;
        Ok(Self { snapshot, relation })
    }
}
impl InputBundle {
    /// Check the complete port set, actual relation fields and producer ownership.
    /// # Errors
    /// Missing/extra ports, required absence, foreign schema or a mismatched producer.
    pub fn validate(&self, spec: &PassSpec, registry: &Registry) -> Result<(), CompilerError> {
        let expected = spec
            .inputs
            .iter()
            .map(|port| port.port)
            .collect::<BTreeSet<_>>();
        if self.ports.keys().copied().collect::<BTreeSet<_>>() != expected {
            return Err(invalid("input port inventory differs from declaration"));
        }
        for port in &spec.inputs {
            let Some(input) = self.ports[port.port].as_ref() else {
                if port.required {
                    return Err(invalid(format!("required input {} absent", port.port)));
                }
                continue;
            };
            let relation = registry
                .relation(&port.relation)
                .ok_or_else(|| invalid("input relation missing"))?;
            input
                .relation
                .contract()
                .validate_against_registry(registry, relation)?;
            pse_relations::validate::validate_batch(registry, relation, input.relation.batch())
                .map_err(|errors| pse_relations::RelationError::Validation { errors })?;
            if let PortSource::Derived { pass, port: source } = port.source {
                let producer = registry
                    .pass(pass)
                    .ok_or_else(|| invalid("producer absent"))?;
                if input.snapshot.stage_pass() != Some(producer.id)
                    || input.relation.member().port != source
                {
                    return Err(invalid(
                        "derived input has a different producing pass or output port",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Actual rows exposed to a pass, with ambiguity refused.
    /// # Errors
    /// Conflicting bindings for the same declared relation.
    pub fn rows(
        &self,
        registry: &Registry,
    ) -> Result<BTreeMap<RelationKey, RecordBatch>, CompilerError> {
        let mut rows = BTreeMap::new();
        for input in self.ports.values().flatten() {
            let spec = registry
                .relation_by_id(input.relation_id())
                .ok_or_else(|| invalid("input relation missing"))?;
            if rows
                .insert(spec.key, input.relation.batch().clone())
                .is_some()
            {
                return Err(invalid(
                    "one relation is bound through multiple input ports",
                ));
            }
        }
        Ok(rows)
    }
}
/// Admit the full output port inventory and every actual batch before publication.
/// # Errors
/// Omitted/extra ports, changed pass attribution, malformed schema or row values.
pub fn validate_output(
    output: &PassOutput,
    spec: &PassSpec,
    registry: &Registry,
) -> Result<(), CompilerError> {
    if output.record.pass_id != spec.id || output.record.version != spec.version {
        return Err(invalid("output pass attribution differs"));
    }
    if output.ports.keys().copied().collect::<BTreeSet<_>>()
        != spec.outputs.iter().map(|port| port.port).collect()
    {
        return Err(invalid("output port inventory differs"));
    }
    for port in &spec.outputs {
        let relation = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("output relation missing"))?;
        RelationContract::from_spec(registry, relation, EncodingPolicy::IpcFile)?;
        for batch in &output.ports[port.port] {
            pse_relations::validate::validate_batch(registry, relation, batch)
                .map_err(|errors| pse_relations::RelationError::Validation { errors })?;
        }
    }
    Ok(())
}
