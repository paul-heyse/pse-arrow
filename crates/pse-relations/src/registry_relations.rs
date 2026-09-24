// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Share the schema registry's native self-description buffers.
use crate::RelationError;
use arrow_array::RecordBatch;
use pse_schema::{Registry, model::RelationKey};
use std::collections::BTreeMap;

/// Share the completed native declaration projection without rebuilding its arrays.
/// # Errors
/// A known relation's schema or visible-value contract is invalid.
pub fn materialize(reg: &Registry) -> Result<BTreeMap<RelationKey, RecordBatch>, RelationError> {
    // The registry owns immutable reflection buffers. The retained proof applies
    // to those actual buffers only; arbitrary batches still require raw admission.
    let proof = reg.derived_implementation(|| Ok(ReflectionAdmission::default()))?;
    let mut admitted = proof
        .0
        .lock()
        .map_err(|_| crate::validate::mismatch("registry reflection", "admission lock poisoned"))?;
    if !*admitted {
        for (key, batch) in reg.schema_batches() {
            if let Some(spec) = reg.relation_by_key(*key) {
                crate::validate::validate_batch(reg, spec, batch)
                    .map_err(|errors| RelationError::Validation { errors })?;
            }
        }
        *admitted = true;
    }
    Ok(reg.schema_batches().iter().cloned().collect())
}

#[derive(Default)]
struct ReflectionAdmission(std::sync::Mutex<bool>);

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    #[test]
    fn unchanged_reflection_keeps_its_completed_local_admission() {
        let registry = pse_schema::registry().unwrap();
        let spec = registry.relation("reference.schema_relations").unwrap();
        let context = crate::validate::ValidationContext::for_registry(registry).unwrap();
        let prepared = context.relation(registry, spec).unwrap();
        let first = materialize(registry).unwrap();
        let calls = prepared.evaluation_count();
        let second = materialize(registry).unwrap();
        assert_eq!(prepared.evaluation_count(), calls);
        assert_eq!(first, second);
    }
}
