// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact Delta publication opening. The control relation selects the entire catalog.
use datafusion::{
    catalog::{
        CatalogProvider, CatalogProviderList, MemoryCatalogProvider, MemoryCatalogProviderList,
        MemorySchemaProvider, SchemaProvider,
    },
    common::{DataFusionError, Result, ScalarValue},
    datasource::{ViewTable, provider_as_source},
    execution::session_state::{SessionState, SessionStateBuilder},
    logical_expr::{LogicalPlanBuilder, lit},
    physical_plan::collect,
};
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use pse_relations::generated::runtime::publications;
use pse_schema::Registry;
use std::sync::Arc;

/// Exact root selection; member versions are read from that immutable control version.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationRoot {
    /// Location of the workspace's Delta control table.
    pub location: url::Url,
    /// Exact Delta version of the publication record.
    pub version: i64,
}

/// Exact selected publication and its admitted native execution environment.
/// Reads retain policies, requirements, cancellation and Arrow ownership through
/// the same boundary as SQL and compiler plans.
#[derive(Debug)]
pub struct Publication {
    root: PublicationRoot,
    record: publications::Row,
    session: pse_engine::session::EngineSession,
}
impl Publication {
    /// Open exact control/member versions under the actual caller's native policy.
    /// Relation payloads remain lazy; opening verifies declaration and selection.
    /// # Errors
    /// Missing versions, incompatible contracts, policy refusal or cancellation.
    pub async fn open(
        root: PublicationRoot,
        registry: Arc<Registry>,
        factory: &pse_engine::session::EngineFactory,
        cancel: &pse_columnar::CancellationToken,
    ) -> std::result::Result<Self, crate::EngineError> {
        cancel.checkpoint()?;
        if root.version < 0 {
            return Err(pse_engine::session::engine(invalid(
                "publication version must be nonnegative",
            )));
        }
        let mut leases = Vec::new();
        if let Some(lease) = super::lease::read(&root.location, cancel)
            .await
            .map_err(pse_engine::session::engine)?
        {
            leases.push(lease);
        }
        let mut state = factory.native_state().clone();
        state.config_mut().set_extension(Arc::new(
            pse_engine::session::execution::AttemptScope::default(),
        ));
        let state = Arc::new(state);
        let record = cancel
            .until_cancelled(read_record(&root, &registry, Arc::clone(&state)))
            .await?
            .map_err(pse_engine::session::engine)?;
        super::admission::admit_profile(&record, &registry).map_err(pse_engine::session::engine)?;
        for location in record
            .members
            .iter()
            .map(|member| member.table_uri.as_str())
            .collect::<std::collections::BTreeSet<_>>()
        {
            let location = url::Url::parse(location)
                .map_err(|error| DataFusionError::External(Box::new(error)))
                .map_err(pse_engine::session::engine)?;
            if let Some(lease) = super::lease::read(&location, cancel)
                .await
                .map_err(pse_engine::session::engine)?
            {
                leases.push(lease);
            }
        }
        let state = cancel
            .until_cancelled(bind_members(&record, &registry, state))
            .await?
            .map_err(pse_engine::session::engine)?;
        let mut session =
            crate::selection::bind_publication(&record, &state, registry, factory, cancel).await?;
        for owner in leases {
            session.retain_owner(owner);
        }
        // Binding does not certify requirements, but an open cannot bypass them.
        session.check_requirements(cancel).await?;
        let publication = Self {
            root,
            record,
            session,
        };
        if publication.record.kind != pse_relations::generated::enums::PublicationKind::Relations {
            publication.artifact_descriptor(cancel).await?;
        }
        Ok(publication)
    }
    /// Exact root retained by this handle.
    pub fn root(&self) -> &PublicationRoot {
        &self.root
    }
    /// Complete generated control record; no parallel manifest is retained.
    pub fn record(&self) -> &publications::Row {
        &self.record
    }
    /// Actual immutable execution environment over the selected native hierarchy.
    pub fn session(&self) -> &pse_engine::session::EngineSession {
        &self.session
    }
    /// Move the selected provider owners into an invocation without retaining this handle.
    pub fn into_session(self) -> pse_engine::session::EngineSession {
        self.session
    }
    /// Exact generated member selected by the control transaction.
    /// # Errors
    /// The qualified name is outside this publication.
    pub fn member(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
    ) -> std::result::Result<publications::RuntimePublicationsFieldMembersItem, crate::EngineError>
    {
        crate::selection::selected_member(&self.session, reference)
    }
    /// Begin an owned native relation stream with common admission and requirements.
    /// Dropping the publication does not invalidate its stream or exported batches.
    /// # Errors
    /// Missing selection, policy refusal, planning, resource or cancellation failure.
    pub async fn relation_stream(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
        cancel: &pse_columnar::CancellationToken,
    ) -> std::result::Result<pse_engine::session::OwnedComputationStream, crate::EngineError> {
        self.member(reference)?;
        self.session.relation_stream(reference, cancel).await
    }
}
/// Read only the bounded control relation; relation payloads remain lazy providers.
pub(super) async fn read_record(
    root: &PublicationRoot,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<publications::Row> {
    read_optional_record(root, registry, state)
        .await?
        .ok_or_else(|| invalid("publication control has no published row"))
}

/// An empty declared control table is initialized storage, not a publication root.
pub(super) async fn read_optional_record(
    root: &PublicationRoot,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<Option<publications::Row>> {
    read_control(root, registry, state, None).await
}

pub(super) async fn read_maintained_record(
    root: &PublicationRoot,
    registry: &Registry,
    state: Arc<SessionState>,
    lease: &super::lease::MaintenanceLease,
) -> Result<publications::Row> {
    read_control(root, registry, state, Some(lease))
        .await?
        .ok_or_else(|| invalid("publication control has no published row"))
}

async fn read_control(
    root: &PublicationRoot,
    registry: &Registry,
    state: Arc<SessionState>,
    lease: Option<&super::lease::MaintenanceLease>,
) -> Result<Option<publications::Row>> {
    let contract = super::contract::DeclaredCheck::new(
        registry,
        publications::spec(registry).map_err(external)?.id,
    )?;
    let control = if let Some(lease) = lease {
        super::provider::open_maintained_view(
            root.location.clone(),
            root.version,
            &contract,
            Arc::clone(&state),
            lease,
        )
        .await?
    } else {
        super::provider::open_declared_view(
            root.location.clone(),
            root.version,
            &contract,
            Arc::clone(&state),
        )
        .await?
    };
    let plan = LogicalPlanBuilder::scan(
        "publication_control",
        provider_as_source(Arc::new(control)),
        None,
    )?
    .limit(0, Some(2))?
    .build()?;
    let batches = collect(state.create_physical_plan(&plan).await?, state.task_ctx()).await?;
    let batch =
        datafusion::arrow::compute::concat_batches(contract.layout().execution_schema(), &batches)?;
    if batch.num_rows() == 0 {
        return Ok(None);
    }
    if batch.num_rows() > 1 {
        return Err(invalid(
            "publication control must contain exactly one workspace row",
        ));
    }
    let record = publications::View::try_from_batch_with_registry(registry, &batch)
        .map_err(external)?
        .row(0)
        .map_err(external)?;
    Ok(Some(record))
}
pub(super) async fn bind_members(
    record: &publications::Row,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<Arc<SessionState>> {
    let catalogs = Arc::new(MemoryCatalogProviderList::new());
    let limit = state
        .config()
        .get_extension::<pse_engine::cache_service::NativeCacheService>()
        .map_or(1, |service| service.policy().concurrent_loads.get());
    let slots = pse_columnar::MemoryConsumer::new("publication:member-open-slots")
        .register(&state.runtime_env().memory_pool);
    slots.try_grow(
        record
            .members
            .len()
            .checked_mul(256)
            .ok_or_else(|| invalid("member open inventory overflows"))?,
    )?;
    let opens = futures_util::stream::iter(0..record.members.len()).map(|ordinal| {
        let member = &record.members[ordinal];
        let state = state.clone();
        async move {
            let view = selected_provider(member, registry, state)
                .await
                .map_err(|error| {
                    error.context(format!(
                        "open publication member {}.{}",
                        member.schema_name, member.table_name
                    ))
                })?;
            Ok::<_, DataFusionError>((ordinal, view))
        }
        .boxed()
    });
    let mut opened = opens
        .buffer_unordered(limit)
        .try_collect::<Vec<_>>()
        .await?;
    opened.sort_unstable_by_key(|(ordinal, _)| *ordinal);
    for (ordinal, view) in opened {
        let member = &record.members[ordinal];
        let catalog = if let Some(catalog) = catalogs.catalog(&member.catalog_name) {
            catalog
        } else {
            let catalog: Arc<dyn CatalogProvider> = Arc::new(MemoryCatalogProvider::new());
            catalogs.register_catalog(member.catalog_name.clone(), Arc::clone(&catalog));
            catalog
        };
        let schema = if let Some(schema) = catalog.schema(&member.schema_name) {
            schema
        } else {
            let schema: Arc<dyn SchemaProvider> = Arc::new(MemorySchemaProvider::new());
            catalog.register_schema(&member.schema_name, Arc::clone(&schema))?;
            schema
        };
        if schema
            .register_table(member.table_name.clone(), view)?
            .is_some()
        {
            return Err(invalid(
                "publication has duplicate qualified table bindings",
            ));
        }
    }
    let state = Arc::new(
        SessionStateBuilder::new_from_existing(state.as_ref().clone())
            .with_catalog_list(catalogs)
            .build(),
    );
    Ok(state)
}

/// Open exactly the declared member slice through the common native provider.
pub(crate) async fn selected_provider(
    member: &publications::RuntimePublicationsFieldMembersItem,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<Arc<dyn datafusion::catalog::TableProvider>> {
    let relation = registry
        .relation_by_id(member.relation_id)
        .ok_or_else(|| invalid("publication references an unknown relation contract"))?;
    if i64::from(relation.key.version) != member.relation_version
        || relation.fingerprint != member.contract_fingerprint
    {
        return Err(invalid(
            "publication relation version or fingerprint differs from its declaration",
        ));
    }
    let contract = super::contract::DeclaredCheck::new(registry, relation.id)?;
    let location = url::Url::parse(&member.table_uri)
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
    let view = super::provider::open_declared_view(
        location,
        member.delta_version,
        &contract,
        Arc::clone(&state),
    )
    .await?;
    let view = match member
        .selection
        .selected()
        .map_err(pse_columnar::external)?
    {
        publications::RuntimePublicationsFieldMembersItemSelectionSelected::Full => view,
        publications::RuntimePublicationsFieldMembersItemSelectionSelected::Revision(selection) => {
            let column = relation
                .column(&selection.column)
                .ok_or_else(|| invalid("revision selection column is undeclared"))?;
            if column.extension() != Some(pse_schema::model::ExtensionUse::SemanticId) {
                return Err(invalid(
                    "revision selection column is not a semantic identity",
                ));
            }
            let value =
                ScalarValue::FixedSizeBinary(16, Some(selection.revision_id.as_bytes().to_vec()));
            let plan = LogicalPlanBuilder::from(view.logical_plan().clone())
                .filter(
                    datafusion::logical_expr::Expr::Column(datafusion::common::Column::from_name(
                        &selection.column,
                    ))
                    .eq(lit(value)),
                )?
                .build()?;
            ViewTable::new(plan, None)
        }
    };
    crate::cache_service::resident::selected(super::provider::selected_view(view), member, &state)
}

pub(super) async fn verify_inputs(
    record: &publications::Row,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<()> {
    // Inputs and members project the same declared nested Arrow value. Transfer
    // the column directly; no generic scalar reconstruction or second schema.
    let mut builder = publications::Builder::with_registry(registry, 1).map_err(external)?;
    builder.push(record.clone()).map_err(external)?;
    let batch = builder.finish().map_err(external)?.into_batch();
    let schema = batch.schema();
    let mut columns = batch.columns().to_vec();
    columns[schema.index_of("members")?] = Arc::clone(batch.column(schema.index_of("inputs")?));
    let batch = datafusion::arrow::array::RecordBatch::try_new(schema, columns)?;
    let inputs = publications::View::try_from_batch_with_registry(registry, &batch)
        .map_err(external)?
        .row(0)
        .map_err(external)?;
    bind_members(&inputs, registry, state).await?;
    Ok(())
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}
