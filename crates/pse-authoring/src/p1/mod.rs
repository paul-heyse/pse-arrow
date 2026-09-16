// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P1 parses a complete package inventory and stages typed candidate changes.

use crate::change_set::AuthoredReader;
use crate::document::DocumentBundle;
use crate::{AuthoringError, SourceSpan};
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
) -> Result<crate::document::Batches, AuthoringError> {
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

/// Construct source rows, bindings and their native before/after difference once.
/// The immutable result retains the actual source projection for P2/commit consumption.
/// # Errors
/// Source binding, stale base, duplicate keys, cancellation or native execution.
pub async fn stage_owned(
    bundles: &crate::document::OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) -> Result<crate::change_set::OwnedChangeSet, AuthoringError> {
    let registry = session.registry();
    bundles.validate_registry(registry)?;
    let mut work = session.reserver().open("authoring:source-construction");
    work.try_grow(crate::work::sources(bundles.bundles())?)?;
    let (batches, bindings, completed) =
        project_sources(bundles, session, cancel, work.as_mut()).await?;
    let prior = crate::change_set::base::checked(base, session)?;
    let staged = crate::change_set::stage::stage_checked(
        base.revision_id(),
        header,
        &batches,
        &prior,
        session,
        cancel,
    )
    .await?;
    let mut changes = (*staged).clone();
    let mut candidate = prior.into_owned();
    candidate.extend(batches);
    changes.attach_source(crate::change_set::SourceConstruction {
        documents: bundles.clone(),
        candidate,
        bindings,
        completed,
        edits: Vec::new(),
        renamed: std::collections::BTreeSet::new(),
    });
    Ok(crate::change_set::OwnedChangeSet::new(changes, work))
}

pub(crate) async fn project_sources(
    bundles: &crate::document::OwnedDocumentSet,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
    work: &mut dyn pse_ids::Reservation,
) -> Result<
    (
        crate::document::Batches,
        crate::document::binding::OwnedSourceBindings,
        crate::change_set::plans::Completions,
    ),
    AuthoringError,
> {
    let registry = session.registry();
    let mut batches = source_batches(bundles.bundles(), registry)?;
    for batch in batches.values_mut() {
        *batch = batch.retained(session.reserver(), cancel)?;
    }
    let bindings =
        crate::document::binding::bind_sources_owned(bundles, &batches, session, cancel).await?;
    let mut completed = Vec::new();
    bind_targets(&mut batches, session, work, &mut completed, cancel).await?;
    Ok((batches, bindings, completed))
}

/// Establish source/result correspondence at the external change admission boundary.
/// Local source constructors retain this coupling and do not invoke this comparison.
/// # Errors
/// Supplied rows differ from actual parsed source projection/target resolution.
pub async fn admit_source_candidate(
    bundles: &crate::document::OwnedDocumentSet,
    candidate: &crate::change_set::OwnedCandidateSnapshot,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) -> Result<crate::change_set::OwnedCandidateSnapshot, AuthoringError> {
    bundles.validate_registry(session.registry())?;
    let mut work = session
        .reserver()
        .open("authoring:external-source-correspondence");
    work.try_grow(crate::work::sources(bundles.bundles())?)?;
    let (batches, bindings, mut completed) =
        project_sources(bundles, session, cancel, work.as_mut()).await?;
    let execution = crate::change_set::plans::session(session)?;
    for (id, expected) in batches {
        let spec = session
            .registry()
            .relation_by_id(id)
            .ok_or_else(|| contract("source relation absent"))?;
        let actual = candidate
            .checked_relations()
            .get(&id)
            .ok_or_else(|| contract("candidate omits source relation"))?
            .clone();
        let bound = crate::change_set::plans::roles(&execution, actual, expected, cancel)?;
        let plan = crate::change_set::plans::difference(&bound, spec)?;
        if crate::change_set::plans::execute_recorded(&bound, plan, cancel, &mut completed)
            .await?
            .iter()
            .any(|batch| batch.num_rows() != 0)
        {
            return Err(contract(
                "external candidate differs from complete source projection",
            ));
        }
    }
    let mut changes = candidate.changes.clone();
    changes.attach_source(crate::change_set::SourceConstruction {
        documents: bundles.clone(),
        candidate: candidate.checked_relations().clone(),
        bindings,
        completed,
        edits: Vec::new(),
        renamed: std::collections::BTreeSet::new(),
    });
    Ok(crate::change_set::OwnedCandidateSnapshot::new(
        crate::change_set::CandidateSnapshot {
            relations: candidate.relations.clone(),
            base_revision_id: candidate.base_revision_id,
            changes,
        },
        candidate.checked_relations().clone(),
        work,
    ))
}

/// Consume locally constructed source rows without replaying stage/application.
/// P2 still establishes the complete relational/domain obligations.
/// # Errors
/// The same source construction failures as [`stage_owned`].
pub async fn construct(
    bundles: &crate::document::OwnedDocumentSet,
    base: &dyn AuthoredReader,
    header: authored::change_sets::Row,
    session: &pse_catalog::session::SnapshotSession,
    cancel: &pse_ids::CancellationToken,
) -> Result<crate::change_set::OwnedCandidateSnapshot, AuthoringError> {
    let changes = stage_owned(bundles, base, header, session, cancel).await?;
    let source = changes
        .source()
        .ok_or_else(|| contract("source constructor output missing"))?;
    let mut work = session.reserver().open("authoring:source-candidate");
    work.try_grow(crate::work::mul(source.candidate.len(), 2048)?)?;
    Ok(crate::change_set::OwnedCandidateSnapshot::new(
        crate::change_set::CandidateSnapshot {
            relations: raw_batches(&source.candidate),
            base_revision_id: base.revision_id(),
            changes: (*changes).clone(),
        },
        source.candidate.clone(),
        work,
    ))
}

/// Bind target syntax against its actual declared instance/domain inventory.
/// # Errors
/// Ambiguous declarations, invalid domain selectors or unavailable resources.
async fn bind_targets(
    batches: &mut crate::document::Batches,
    session: &pse_catalog::session::SnapshotSession,
    work: &mut dyn pse_ids::Reservation,
    completed: &mut crate::change_set::plans::Completions,
    cancel: &pse_ids::CancellationToken,
) -> Result<(), AuthoringError> {
    macro_rules! targets {
        ($source:ident, $destination:ident, $key:ident) => {{
            let mut output = authored::$destination::Builder::with_registry(session.registry(), 0)?;
            if let Some(source) = batches.get(&authored::$source::RELATION_ID) {
                let view = authored::$source::View::from_checked(source)?;
                for ordinal in 0..source.batch().num_rows() {
                    let row = view.row(ordinal)?;
                    let at = SourceSpan::try_from(row.source_span)?;
                    let path = crate::targets::parse(&row.target, at)?;
                    for target in crate::targets::resolve_native(
                        &path, batches, session, row.$key, work, completed, cancel,
                    )
                    .await?
                    {
                        output.push(authored::$destination::Row {
                            $key: row.$key,
                            ordinal: target.ordinal,
                            instance_id: target.instance_id,
                            member_kind: target.member_kind,
                            symbol_decl_id: target.symbol_decl_id,
                            equation_decl_id: target.equation_decl_id,
                            port_template_id: target.port_template_id,
                            port_name: target.port_name,
                            index: target.index,
                            wildcard: target.wildcard,
                        })?;
                    }
                }
            }
            batches.insert(
                authored::$destination::RELATION_ID,
                output.finish()?.retained(session.reserver(), cancel)?,
            );
        }};
    }
    targets!(case_specs, case_spec_targets, spec_id);
    targets!(case_activations, case_activation_targets, activation_id);
    targets!(observations, observation_targets, observation_id);
    Ok(())
}
fn contract(reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
}

pub(crate) fn raw_batches(
    rows: &crate::document::Batches,
) -> BTreeMap<SemanticId, datafusion::arrow::array::RecordBatch> {
    rows.iter()
        .map(|(id, batch)| (*id, batch.batch().clone()))
        .collect()
}
