// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parse complete document inventories into authoritative typed source relations.

use crate::authoring_driver::DriverError;
use crate::authoring_driver::document::DocumentBundle;
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use pse_schema::Registry;
use std::collections::BTreeMap;

/// Assemble a complete desired source inventory using generated Arrow concatenation.
/// Empty declared sections remain explicit input bindings.
/// # Errors
/// Missing declarations or incompatible generated field contracts.
pub fn source_batches(
    bundles: &[DocumentBundle],
    registry: &Registry,
) -> Result<crate::authoring_driver::document::Batches, DriverError> {
    use pse_relations::columnar::FieldCheckedBatch;
    let mut parts = BTreeMap::<SemanticId, Vec<FieldCheckedBatch>>::new();
    for document in registry.documents() {
        for section in &document.sections {
            let spec = registry
                .relation(section.relation)
                .ok_or_else(|| contract("source declaration missing"))?;
            parts.entry(spec.id).or_default();
        }
    }
    for id in [
        authored::packages::RELATION_ID,
        authored::documents::RELATION_ID,
        authored::entities::RELATION_ID,
    ] {
        parts.entry(id).or_default();
    }
    for bundle in bundles {
        for (id, batch) in &bundle.batches {
            parts.entry(*id).or_default().push(batch.clone());
        }
    }
    parts
        .into_iter()
        .map(|(id, values)| {
            let spec = registry
                .relation_by_id(id)
                .ok_or_else(|| contract("source relation missing"))?;
            Ok((id, FieldCheckedBatch::concat(registry, spec, &values)?))
        })
        .collect()
}

fn contract(reason: &str) -> DriverError {
    DriverError::Authoring(pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    })
}
