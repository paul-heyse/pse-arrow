// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Export manifests (Plan 22 X12): one publication record for offline readers.
//!
//! An export writes the record the catalog granted — with the export's reader lease, its
//! expiry, the workspace maintenance epoch and the operational store fingerprint — as a
//! one-row Delta table of `runtime.publication_manifests`: version 0 creates the table
//! with its contract, version 1 holds the row, and nothing else. An offline reader opens
//! exactly version 1 and then exactly the members the row names. A table written under
//! the former Delta control relation is an unsupported historical format.
use datafusion::{
    common::{DataFusionError, ResolvedTableReference, Result},
    execution::session_state::SessionState,
    logical_expr::LogicalPlanBuilder,
    physical_plan::collect,
};
use deltalake::{kernel::transaction::CommitProperties, protocol::SaveMode};
use pse_columnar::CancellationToken;
use pse_engine::{
    EngineError,
    session::{EngineSession, PreparedComputation},
};
use pse_relations::generated::runtime::publication_manifests;
use pse_schema::{compatibility::CompatibilityError, model::provider::OperationPurpose};
use std::{collections::BTreeMap, sync::Arc};

/// The only version of an export manifest that holds its row.
pub const MANIFEST_VERSION: i64 = 1;

/// The publication a record describes, as the operational catalog's identity.
///
/// The record's other references carry their identities in the registry; its key cannot,
/// because a single-column key that declares an identity owns it and the catalog's
/// publication intents own `publication`. This is the one place the key becomes a
/// [`PublicationId`](pse_model::generated::identities::PublicationId).
#[must_use]
pub const fn publication_of(
    record: &publication_manifests::Row,
) -> pse_model::generated::identities::PublicationId {
    pse_model::generated::identities::PublicationId::from_id(record.publication_id)
}

/// The identity of the former Delta control relation `runtime.publications@2`, whose
/// tables are refused rather than interpreted.
fn legacy_control() -> pse_ids::SemanticId {
    pse_schema::builder::registry_id("relation:runtime.publications@2")
}

/// Prepare writing an export manifest at `destination`, which must not exist. The
/// record must carry its export fields.
/// # Errors
/// A record without export fields, an existing destination, policy or planning failure.
pub fn prepare_manifest(
    session: &EngineSession,
    destination: url::Url,
    record: publication_manifests::Row,
    cancel: &CancellationToken,
) -> std::result::Result<PreparedComputation, EngineError> {
    if record.exported_at.is_none() {
        return Err(EngineError::Admission {
            path: "publication.manifest".into(),
            reason: "an export manifest records its export".into(),
        });
    }
    let registry = Arc::clone(session.registry());
    pse_engine::validation::bind_defaults(&registry)?;
    let mut builder = publication_manifests::Builder::with_registry(&registry, 1)?;
    builder.push(record)?;
    let session = session.with_checked_workspace(
        BTreeMap::from([(publication_manifests::RELATION_KEY, builder.finish()?)]),
        cancel,
    )?;
    let source = ResolvedTableReference {
        catalog: "workspace".into(),
        schema: "runtime".into(),
        table: publication_manifests::NAME.into(),
    };
    let input = pse_engine::session::output::declare_relation_output(
        session.relation_plan(&source)?.plan().clone(),
        &registry,
        publication_manifests::spec(&registry)?,
    )
    .map_err(pse_engine::session::engine)?;
    let mut session = session.with_purpose(OperationPurpose::Publish);
    session.bind_target(pse_schema::model::provider::ProviderScope::Table(
        "export".into(),
        "runtime".into(),
        publication_manifests::NAME.into(),
    ));
    let table = super::provider::table_builder(destination, &session.bound_state()?)
        .map_err(pse_engine::session::engine)?
        .build()
        .map_err(|error| pse_engine::session::engine(DataFusionError::External(Box::new(error))))?;
    let plan = super::write::DeltaWrite::declared(
        table,
        input,
        SaveMode::ErrorIfExists,
        CommitProperties::default(),
        super::contract::DeclaredCheck::new(&registry, publication_manifests::RELATION_ID)
            .map_err(pse_engine::session::engine)?,
    )
    .map_err(pse_engine::session::engine)?;
    session.prepare(plan, cancel)
}

/// Read the record of an export manifest: exactly version 1 of a table of the manifest
/// contract.
/// # Errors
/// [`CompatibilityError::MigrationRequired`] for a table of the former Delta control
/// relation; a table of another contract; a missing table; any version but 1.
pub async fn read_manifest(
    location: url::Url,
    registry: &pse_schema::Registry,
    state: Arc<SessionState>,
) -> Result<publication_manifests::Row> {
    let opened = super::provider::open_native(
        location.clone(),
        None,
        crate::cache_service::snapshot::LoadRequirement::Metadata,
        &state,
    )
    .await?;
    let snapshot = opened.table.snapshot().map_err(external)?;
    let configuration = snapshot.metadata().configuration();
    if configuration
        .get(pse_schema::arrow::KEY_CONTRACT_ID)
        .is_some_and(|id| *id == legacy_control().to_hex())
    {
        return Err(external(CompatibilityError::MigrationRequired(format!(
            "{location} is a Delta publication control table, an unsupported historical format; regenerate the publication by rerunning it"
        ))));
    }
    if opened.table.version() != Some(1) {
        return Err(invalid(
            "an export manifest holds exactly one row, at version 1; later versions are refused",
        ));
    }
    let contract =
        super::contract::DeclaredCheck::new(registry, publication_manifests::RELATION_ID)?;
    let view = super::provider::open_declared_view(
        location,
        MANIFEST_VERSION,
        &contract,
        Arc::clone(&state),
    )
    .await?;
    let plan = LogicalPlanBuilder::scan(
        "publication_manifest",
        datafusion::datasource::provider_as_source(Arc::new(view)),
        None,
    )?
    .limit(0, Some(2))?
    .build()?;
    let batches = collect(state.create_physical_plan(&plan).await?, state.task_ctx()).await?;
    let batch =
        datafusion::arrow::compute::concat_batches(contract.layout().execution_schema(), &batches)?;
    if batch.num_rows() != 1 {
        return Err(invalid("an export manifest holds exactly one row"));
    }
    // The row's SQL checks run on the engine's validation planner.
    pse_engine::validation::bind_defaults(registry).map_err(external)?;
    publication_manifests::View::try_from_batch_with_registry(registry, &batch)
        .map_err(external)?
        .row(0)
        .map_err(external)
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn legacy_control_identity_is_the_former_relation() {
        // The identity the removed `runtime.publications@2` declaration generated.
        assert_eq!(
            super::legacy_control().to_hex(),
            "5f310f54c9b3ebfac003c02ef1044244"
        );
    }
}
