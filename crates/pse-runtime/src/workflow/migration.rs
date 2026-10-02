// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit artifact evolution uses the existing attempt, intent and catalog commit.
use super::{LeasedPublication, PublicationAttempt, Runtime, WorkflowError, Workspace};
use pse_catalog::artifact::migration::{ArtifactMigration, MemberMigration};
use pse_columnar::CancellationToken;
use pse_operations::{
    attempts::AttemptId,
    catalog::{MemberDescriptor, PublicationId},
};
use pse_relations::generated::enums::PublicationKind;

/// Prepare a stored migration of exact leased source members. The new immutable
/// members and typed lineage are one publication candidate; only the ordinary
/// catalog commit changes visibility. Preparation performs no write. Failed
/// transformation/admission leaves the selected source publication unchanged.
/// # Errors
/// Lapsed lease, incompatible source/target declaration, missing maps/reference
/// closure, invalid target destinations or ordinary publication admission failure.
#[expect(
    clippy::too_many_arguments,
    reason = "one explicit migration request binds source, transformation, durable attempt, workspace and expected publication parent"
)]
pub fn prepare_artifact_migration(
    runtime: &Runtime,
    source: &LeasedPublication,
    migrations: Vec<MemberMigration>,
    retained: Vec<MemberDescriptor>,
    attempt: AttemptId,
    workspace: &Workspace,
    parent: Option<PublicationId>,
    publication_id: Option<PublicationId>,
    cancel: &CancellationToken,
) -> Result<PublicationAttempt, WorkflowError> {
    source.guard().check()?;
    source
        .guard()
        .cancellation()
        .checkpoint()
        .map_err(pse_engine::EngineError::from)?;
    let migration = ArtifactMigration::new(source.publication(), migrations, retained, cancel)?;
    let (artifact, retained) = migration.into_publication();
    Ok(super::publication::prepare_artifact_publication(
        runtime,
        attempt,
        workspace,
        parent,
        publication_id,
        PublicationKind::Migration,
        artifact,
        retained,
        cancel,
    )?
    .retain_migration_source(source.guard().clone()))
}
