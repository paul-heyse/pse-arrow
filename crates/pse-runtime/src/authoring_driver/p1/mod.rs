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
/// Complete headers include any immutable physical package supplied by the context owner.
/// # Errors
/// Missing dependencies, cycles, reference violations or incompatible generated field contracts.
pub fn source_batches(
    bundles: &[DocumentBundle],
    registry: &Registry,
    headers: &[authored::packages::Row],
    limits: pse_authoring::p0::GraphLimits,
) -> Result<crate::authoring_driver::document::Batches, DriverError> {
    use pse_relations::columnar::FieldCheckedBatch;
    admit_package_closure(bundles, headers, limits)?;
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
    let batches: crate::authoring_driver::document::Batches = parts
        .into_iter()
        .map(|(id, values)| {
            let spec = registry
                .relation_by_id(id)
                .ok_or_else(|| contract("source relation missing"))?;
            Ok((id, FieldCheckedBatch::concat(registry, spec, &values)?))
        })
        .collect::<Result<_, DriverError>>()?;
    admit_references(&batches, registry)?;
    Ok(batches)
}

/// Admit the final complete package closure under its retained owner's finite graph bounds.
/// Each submitted source header must agree with the explicit complete context.
/// Individual document loading remains usable before all dependency packages are present.
/// # Errors
/// Missing, duplicate, incompatible or cyclic package dependencies, or extent overflow.
pub fn admit_package_closure(
    bundles: &[DocumentBundle],
    headers: &[authored::packages::Row],
    limits: pse_authoring::p0::GraphLimits,
) -> Result<(), DriverError> {
    for bundle in bundles {
        if !headers.iter().any(|header| header == &bundle.package) {
            return Err(contract(
                "complete package context differs from submitted source header",
            ));
        }
    }
    pse_authoring::p0::resolve_rows(headers, limits)?;
    Ok(())
}

/// Check actual typed source facts independently of hydration or an earlier admission.
/// # Errors
/// Missing registration facts or shared reference/closure violations.
pub fn admit_references(
    batches: &crate::authoring_driver::document::Batches,
    registry: &Registry,
) -> Result<(), DriverError> {
    use pse_relations::columnar::{ArrowValue, RelationRow};
    let rows = batches
        .get(&authored::entities::RELATION_ID)
        .map(authored::entities::Row::rows)
        .transpose()?
        .unwrap_or_default();
    let mut mappings = BTreeMap::new();
    for document in registry.documents() {
        for section in &document.sections {
            if let (Some(identity), Some(kind)) = (section.identity_column, section.entity_kind) {
                mappings.insert(
                    section.relation,
                    (
                        identity,
                        kind,
                        section.name_column,
                        section.naming_scope_column,
                    ),
                );
            }
        }
    }
    let mut registrations = Vec::new();
    for (relation, (identity, kind, name, owner)) in mappings {
        let spec = registry
            .relation(relation)
            .ok_or_else(|| contract("source registration declaration missing"))?;
        let Some(batch) = batches.get(&spec.id) else {
            continue;
        };
        let column = |name: &str| {
            batch
                .batch()
                .column_by_name(name)
                .map(|array| array.as_ref())
                .ok_or_else(|| contract("source registration field missing"))
        };
        for row in 0..batch.batch().num_rows() {
            registrations.push(pse_rules::references::EntityRegistration {
                relation: relation.into(),
                entity: SemanticId::read(column(identity)?, row)?,
                kind: kind.into(),
                name: name
                    .map(|name| String::read(column(name)?, row).map_err(DriverError::from))
                    .transpose()?,
                parent: owner
                    .map(|name| {
                        Option::<SemanticId>::read(column(name)?, row).map_err(DriverError::from)
                    })
                    .transpose()?
                    .flatten(),
                package: batch
                    .batch()
                    .column_by_name("package_id")
                    .map(|array| SemanticId::read(array.as_ref(), row))
                    .transpose()?,
            });
        }
    }
    pse_rules::references::admit_entities(&rows, &registrations)?;
    Ok(())
}

fn contract(reason: &str) -> DriverError {
    DriverError::Authoring(pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    })
}
