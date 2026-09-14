// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The stage DAG, derived from the declared ports (blueprint §14.1).
//!
//! The driver derives its graph from named stage and port edges; reading a relation schema
//! without an input binding is invalid. That is what stops a pass from consuming whatever
//! artifact happened to be latest.
//!
use std::collections::{BTreeMap, BTreeSet};

use crate::CompilerError;
use pse_schema::{
    Registry,
    model::{Authority, PassSpec, PortSource},
};

/// A closed dependency graph derived exclusively from declared pass ports.
#[derive(Clone, Debug)]
pub struct StageDag {
    specs: BTreeMap<&'static str, PassSpec>,
    order: Vec<&'static str>,
}
impl StageDag {
    /// Admit all registered passes, including every producer edge.
    /// # Errors
    /// Duplicate producers, missing ports, incompatible schemas or cycles.
    pub fn build(registry: &Registry) -> Result<Self, CompilerError> {
        let specs = registry
            .passes()
            .iter()
            .map(|spec| (spec.name, spec.clone()))
            .collect::<BTreeMap<_, _>>();
        if specs.len() != registry.passes().len() {
            return Err(invalid("ambiguous pass versions"));
        }
        let mut pending = BTreeMap::new();
        for (name, spec) in &specs {
            let outputs = spec
                .outputs
                .iter()
                .map(|port| port.port)
                .collect::<BTreeSet<_>>();
            let inputs = spec
                .inputs
                .iter()
                .map(|port| port.port)
                .collect::<BTreeSet<_>>();
            if outputs.len() != spec.outputs.len() || inputs.len() != spec.inputs.len() {
                return Err(invalid("a pass repeats an input or output port"));
            }
            for output in &spec.outputs {
                let relation = registry
                    .relation(&output.relation)
                    .ok_or_else(|| invalid("output relation undeclared"))?;
                if !matches!(*name, "P0" | "P1" | "P2")
                    && matches!(
                        relation.authority,
                        Authority::Authored | Authority::Reference
                    )
                {
                    return Err(invalid("a derived pass writes primitive authority"));
                }
            }
            let mut dependencies = BTreeSet::new();
            for input in &spec.inputs {
                let relation = registry
                    .relation(&input.relation)
                    .ok_or_else(|| invalid("input relation undeclared"))?;
                if *name == "P2"
                    && !matches!(
                        relation.authority,
                        Authority::Authored | Authority::Reference
                    )
                {
                    return Err(invalid("P2 reads nonprimitive facts"));
                }
                if let PortSource::Derived { pass, port } = input.source {
                    let output = specs
                        .get(pass)
                        .and_then(|spec| spec.output(port))
                        .ok_or_else(|| invalid("missing declared producer port"))?;
                    if output.relation != input.relation {
                        return Err(invalid("producer and consumer relation contracts differ"));
                    }
                    dependencies.insert(pass);
                }
            }
            pending.insert(*name, dependencies);
        }
        let mut order = Vec::new();
        while !pending.is_empty() {
            let ready = pending
                .iter()
                .find(|(_, parents)| parents.is_empty())
                .map(|(name, _)| *name)
                .ok_or_else(|| invalid("cyclic stage dependencies"))?;
            pending.remove(ready);
            for parents in pending.values_mut() {
                parents.remove(ready);
            }
            order.push(ready);
        }
        Ok(Self { specs, order })
    }
    /// Validate against a fresh registry view, including complete declarations.
    /// # Errors
    /// Any graph error or changed declared pass contract.
    pub fn validate(&self, registry: &Registry) -> Result<(), CompilerError> {
        let current = Self::build(registry)?;
        if self.specs != current.specs || self.order != current.order {
            return Err(invalid("stage declarations changed"));
        }
        Ok(())
    }
    /// Deterministic dependency-first stage order.
    pub fn topo_order(&self) -> &[&'static str] {
        &self.order
    }
    /// One exact registered declaration.
    pub fn spec(&self, name: &str) -> Option<&PassSpec> {
        self.specs.get(name)
    }
    /// Dependency closure needed for a selected output stage.
    /// # Errors
    /// Requested stage is unavailable in this closed registry.
    pub fn through(&self, name: &str) -> Result<Vec<&PassSpec>, CompilerError> {
        let root = self
            .specs
            .get(name)
            .ok_or_else(|| invalid("requested pass is unavailable"))?;
        let mut wanted = BTreeSet::new();
        let mut stack = vec![root.name];
        while let Some(name) = stack.pop() {
            if wanted.insert(name) {
                for input in &self.specs[name].inputs {
                    if let PortSource::Derived { pass, .. } = input.source {
                        stack.push(pass);
                    }
                }
            }
        }
        Ok(self
            .order
            .iter()
            .filter(|name| wanted.contains(**name))
            .map(|name| &self.specs[name])
            .collect())
    }
}
pub(crate) fn invalid(reason: impl Into<String>) -> CompilerError {
    CompilerError::StageGraph {
        reason: reason.into(),
    }
}
