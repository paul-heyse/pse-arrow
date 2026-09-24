// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parse complete document inventories into authoritative typed source relations.

use crate::authoring_driver::document::DocumentBundle;
use crate::authoring_driver::{DriverError, SourceSpan};
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

/// Parse and bind a complete desired document inventory to typed source relations.
/// This is the parser boundary used by the native source operator; it retains no
/// stage envelope, base snapshot, publication controller or predecessor graph.
/// # Errors
/// Source syntax, ambiguous bindings, target resolution, cancellation or resources.
pub async fn project(
    bundles: &crate::authoring_driver::document::OwnedDocumentSet,
    session: &pse_engine::session::EngineSession,
    cancel: &pse_columnar::CancellationToken,
) -> Result<crate::authoring_driver::document::Batches, DriverError> {
    bundles.validate_registry(session.registry())?;
    let mut work = pse_columnar::MemoryConsumer::new("authoring:native-source-projection")
        .register(session.pool());
    work.try_grow(crate::authoring_driver::work::sources(bundles.bundles())?)?;
    let (batches, _, _) = project_sources(bundles, session, cancel, &mut work).await?;
    Ok(batches)
}

pub(crate) async fn project_sources(
    bundles: &crate::authoring_driver::document::OwnedDocumentSet,
    session: &pse_engine::session::EngineSession,
    cancel: &pse_columnar::CancellationToken,
    work: &mut pse_columnar::MemoryReservation,
) -> Result<
    (
        crate::authoring_driver::document::Batches,
        crate::authoring_driver::document::binding::OwnedSourceBindings,
        crate::authoring_driver::native_relations::plans::Completions,
    ),
    DriverError,
> {
    let registry = session.registry();
    let mut batches = source_batches(bundles.bundles(), registry)?;
    for batch in batches.values_mut() {
        *batch = batch.retained(session.pool(), cancel)?;
    }
    let bindings = crate::authoring_driver::document::binding::bind_sources_owned(
        bundles, &batches, session, cancel,
    )
    .await?;
    let mut completed = Vec::new();
    bind_targets(&mut batches, session, work, &mut completed, cancel).await?;
    Ok((batches, bindings, completed))
}

/// Bind target syntax against its actual declared instance/domain inventory.
/// # Errors
/// Ambiguous declarations, invalid domain selectors or unavailable resources.
async fn bind_targets(
    batches: &mut crate::authoring_driver::document::Batches,
    session: &pse_engine::session::EngineSession,
    work: &mut pse_columnar::MemoryReservation,
    completed: &mut crate::authoring_driver::native_relations::plans::Completions,
    cancel: &pse_columnar::CancellationToken,
) -> Result<(), DriverError> {
    use pse_relations::columnar::ArrowValue;
    macro_rules! targets {
        ($source:ident, $destination:ident, $key:ident) => {{
            let mut output = authored::$destination::Builder::with_registry(session.registry(), 0)?;
            if let Some(source) = batches.get(&authored::$source::RELATION_ID) {
                let view = authored::$source::View::from_checked(source)?;
                for ordinal in 0..source.batch().num_rows() {
                    let row = view.row(ordinal)?;
                    let at = SourceSpan::try_from(row.source_span)?;
                    let path = pse_authoring::targets::parse(&row.target, at)?;
                    for target in crate::authoring_driver::targets::resolve_native(
                        &path, batches, session, row.$key, work, completed, cancel,
                    )
                    .await?
                    {
                        output.push(authored::$destination::Row {
                            $key: row.$key,
                            ordinal: target.ordinal,
                            instance_id: target.instance_id,
                            member: ArrowValue::read(
                                target
                                    .member
                                    .to_array(
                                        &authored::$destination::spec(session.registry())?
                                            .column("member")
                                            .ok_or_else(|| contract("target member field absent"))?
                                            .data_type(),
                                    )?
                                    .as_ref(),
                                0,
                            )?,
                        })?;
                    }
                }
            }
            batches.insert(
                authored::$destination::RELATION_ID,
                output.finish()?.retained(session.pool(), cancel)?,
            );
        }};
    }
    targets!(case_specs, case_spec_targets, spec_id);
    targets!(case_activations, case_activation_targets, activation_id);
    targets!(observations, observation_targets, observation_id);
    Ok(())
}
fn contract(reason: &str) -> DriverError {
    DriverError::Authoring(pse_authoring::AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    })
}
