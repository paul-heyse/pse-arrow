// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bind actual immutable producer outputs by their declared input roles (blueprint §14.1).
//!
use super::{AlgorithmInputs, invalid};
use crate::CompilerError;
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, RelationKey},
};
use std::collections::{BTreeMap, BTreeSet};

impl AlgorithmInputs {
    /// Select a relation-keyed inventory for algorithms requiring one input per schema.
    /// General native execution accesses declared arguments through [`Self::port`].
    /// # Errors
    /// One declaration is bound ambiguously or absent from the registry.
    pub fn checked_rows(
        &self,
        registry: &Registry,
    ) -> Result<BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>, CompilerError>
    {
        let mut rows = BTreeMap::new();
        for input in self.ports.values().flatten() {
            let spec = registry
                .relation_by_id(input.relation_id())
                .ok_or_else(|| invalid("input relation absent"))?;
            if rows
                .insert(spec.key, input.relation()?.checked().clone())
                .is_some()
            {
                return Err(invalid(
                    "one relation is bound through multiple input ports",
                ));
            }
        }
        Ok(rows)
    }
    /// Check the complete port set, actual relation fields.
    /// # Errors
    /// Missing/extra ports, required absence, foreign schema or incompatible fields.
    pub fn validate(&self, spec: &AlgorithmSpec, registry: &Registry) -> Result<(), CompilerError> {
        let expected = spec
            .inputs
            .iter()
            .map(|port| port.port.as_str())
            .collect::<BTreeSet<_>>();
        if self
            .ports
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != expected
        {
            return Err(invalid("input port inventory differs from declaration"));
        }
        for port in &spec.inputs {
            let Some(input) = self.ports[&port.port].as_ref() else {
                if port.required {
                    return Err(invalid(format!("required input {} absent", port.port)));
                }
                continue;
            };
            input.check(port, registry)?;
        }
        Ok(())
    }
}
