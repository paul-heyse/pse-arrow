// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Whole-release checkpoint receipts use the ordinary exact publication protocol.
use super::{ArtifactPlan, PublicationTarget, RelationOutput, invalid};
use pse_columnar::{AllocationLease, CancellationToken, MemoryConsumer};
use pse_engine::{EngineError, EngineSession, session::PreparedComputation};
use pse_model::HeapUsage;
use pse_relations::generated::{
    enums::PublicationKind,
    runtime::{publications, release_checkpoints},
};
use std::collections::BTreeMap;

/// Prepare one consumer's complete checkpoint under a stable admission attempt.
/// This function performs no input installation; runtime calls it only after admission.
/// # Errors
/// Invalid receipt/header, resource or policy refusal, incompatible native plans.
pub fn prepare_checkpoint(
    session: &EngineSession,
    control: PublicationTarget,
    output: PublicationTarget,
    header: publications::Row,
    checkpoint: release_checkpoints::Row,
    cancel: &CancellationToken,
) -> Result<PreparedComputation, EngineError> {
    if checkpoint.interpretation_version != 1
        || header.kind != PublicationKind::Relations
        || header.attempt_id != checkpoint.admission_id
        || header.publication_id != checkpoint.admission_id
    {
        return Err(invalid(
            "checkpoint publication must retain its whole-admission identity",
        ));
    }
    let reservation = MemoryConsumer::new("catalog:checkpoint-encoding").register(session.pool());
    reservation
        .try_grow(
            checkpoint
                .owned_bytes()
                .saturating_mul(16)
                .saturating_add(4096),
        )
        .map_err(pse_engine::session::engine)?;
    let admission = checkpoint.admission_id;
    let mut builder = release_checkpoints::Builder::with_registry(session.registry(), 1)?;
    builder.push(checkpoint)?;
    let mut session = session.with_checked_workspace(
        BTreeMap::from([(release_checkpoints::RELATION_KEY, builder.finish()?)]),
        cancel,
    )?;
    session.retain_owner(AllocationLease::new(reservation));
    let source = datafusion::common::ResolvedTableReference {
        catalog: "workspace".into(),
        schema: "runtime".into(),
        table: release_checkpoints::NAME.into(),
    };
    let plan = session.relation_plan(&source)?.plan().clone();
    let reference = output.reference;
    let mut artifact = ArtifactPlan::new(
        session,
        BTreeMap::from([(
            reference.clone(),
            RelationOutput {
                relation_id: release_checkpoints::RELATION_ID,
                plan,
            },
        )]),
        cancel,
    )?;
    artifact.operation_id = admission;
    artifact.prepare_publication(
        control,
        header,
        BTreeMap::from([(reference, output.location)]),
        vec![],
        cancel,
    )
}
