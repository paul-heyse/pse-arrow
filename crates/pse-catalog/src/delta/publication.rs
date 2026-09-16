// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact Delta publication opening. The control relation selects the entire catalog.
use super::{layout::DurableLayout, provider::open_view};
use datafusion::{
    catalog::{
        CatalogProvider, CatalogProviderList, MemoryCatalogProvider, MemoryCatalogProviderList,
        MemorySchemaProvider, SchemaProvider,
    },
    common::{DataFusionError, Result, ScalarValue},
    datasource::{ViewTable, provider_as_source},
    execution::session_state::{SessionState, SessionStateBuilder},
    logical_expr::{LogicalPlanBuilder, col, lit},
    physical_plan::collect,
};
use pse_relations::generated::runtime::publications;
use pse_schema::Registry;
use std::sync::Arc;

/// Exact root selection; member versions are read from that immutable control version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationRoot {
    /// Location of the workspace's Delta control table.
    pub location: url::Url,
    /// Exact Delta version of the publication record.
    pub version: u64,
}

/// Opened target facts and native providers. This contains no materialized model or
/// predecessor graph; scans load selected relation columns as queries require them.
#[derive(Debug)]
pub struct Publication {
    root: PublicationRoot,
    record: publications::Row,
    state: Arc<SessionState>,
}
impl Publication {
    /// Open one exact root and bind all its exact members into native catalogs.
    /// Required model invariants belong to candidate admission; opening verifies the
    /// control contract, relation identities/layouts, version availability and binding.
    /// # Errors
    /// Invalid control rows, duplicate bindings, unknown contracts or unavailable versions.
    pub async fn open(
        root: PublicationRoot,
        registry: &Registry,
        state: Arc<SessionState>,
    ) -> Result<Self> {
        let record = read_record(&root, registry, Arc::clone(&state)).await?;
        let state = bind_members(&record, registry, state).await?;
        Ok(Self {
            root,
            record,
            state,
        })
    }
    /// Exact root retained by this handle.
    pub fn root(&self) -> &PublicationRoot {
        &self.root
    }
    /// The generated typed control record, retained without a parallel schema.
    pub fn record(&self) -> &publications::Row {
        &self.record
    }
    /// Actual selected provider, including its exact Delta version and revision filter.
    /// Resolving this provider performs no relation scan.
    /// # Errors
    /// The full name is outside this publication or its immutable binding is absent.
    pub async fn member_provider(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
    ) -> Result<Arc<dyn datafusion::catalog::TableProvider>> {
        self.member(reference)?;
        self.state
            .catalog_list()
            .catalog(&reference.catalog)
            .and_then(|catalog| catalog.schema(&reference.schema))
            .ok_or_else(|| invalid("selected member namespace is absent"))?
            .table(&reference.table)
            .await?
            .ok_or_else(|| invalid("selected member provider is absent"))
    }
    /// Exact generated member declaration selected by the control transaction.
    /// # Errors
    /// The fully qualified name is not selected.
    pub fn member(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
    ) -> Result<&publications::RuntimePublicationsFieldMembersItem> {
        self.record
            .members
            .iter()
            .find(|member| {
                member.catalog_name == reference.catalog.as_ref()
                    && member.schema_name == reference.schema.as_ref()
                    && member.table_name == reference.table.as_ref()
            })
            .ok_or_else(|| invalid("relation is not selected by this publication"))
    }
    /// Stream one complete selected relation with its generated Arrow contract.
    /// Unlike an arbitrary SQL projection this boundary denotes a declared relation;
    /// schema annotations are restored after native optimization, with exact field
    /// checks. The stream owns its native providers/runtime independently of this handle.
    /// # Errors
    /// The qualified relation is absent from this publication, planning fails or
    /// native rewrites change declared fields. I/O errors remain stream errors.
    pub async fn relation_stream(
        &self,
        reference: &datafusion::common::ResolvedTableReference,
    ) -> Result<datafusion::physical_plan::SendableRecordBatchStream> {
        if !self.record.members.iter().any(|member| {
            member.catalog_name == reference.catalog.as_ref()
                && member.schema_name == reference.schema.as_ref()
                && member.table_name == reference.table.as_ref()
        }) {
            return Err(invalid("relation is not selected by this publication"));
        }
        let context = datafusion::execution::context::SessionContext::new_with_state(
            self.session_state().await?,
        );
        let plan = context
            .table(datafusion::common::TableReference::full(
                reference.catalog.clone(),
                reference.schema.clone(),
                reference.table.clone(),
            ))
            .await?
            .into_unoptimized_plan();
        let schema = Arc::new(plan.schema().as_arrow().clone());
        let state = context.state();
        let physical =
            super::layout::declared_output(state.create_physical_plan(&plan).await?, &schema)?;
        datafusion::physical_plan::execute_stream(physical, state.task_ctx())
    }
    /// A private native namespace for one invocation, sharing exact providers and resources.
    /// Registering or replacing names cannot change this publication or another invocation.
    /// # Errors
    /// Native catalog lookup or registration fails.
    pub async fn session_state(&self) -> Result<SessionState> {
        let catalogs = Arc::new(MemoryCatalogProviderList::new());
        for name in self.state.catalog_list().catalog_names() {
            let source = self
                .state
                .catalog_list()
                .catalog(&name)
                .ok_or_else(|| invalid("publication catalog disappeared"))?;
            let catalog = Arc::new(MemoryCatalogProvider::new());
            for name in source.schema_names() {
                let source = source
                    .schema(&name)
                    .ok_or_else(|| invalid("publication schema disappeared"))?;
                let schema = Arc::new(MemorySchemaProvider::new());
                for table in source.table_names() {
                    let provider = source
                        .table(&table)
                        .await?
                        .ok_or_else(|| invalid("publication table disappeared"))?;
                    schema.register_table(table, provider)?;
                }
                catalog.register_schema(&name, schema)?;
            }
            catalogs.register_catalog(name, catalog);
        }
        Ok(
            SessionStateBuilder::new_from_existing(self.state.as_ref().clone())
                .with_catalog_list(catalogs)
                .build(),
        )
    }
}
/// Read only the bounded control relation; relation payloads remain lazy providers.
pub(super) async fn read_record(
    root: &PublicationRoot,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<publications::Row> {
    let layout = DurableLayout::new(publications::schema().map_err(external)?)?;
    let control = open_view(
        root.location.clone(),
        root.version,
        &layout,
        Arc::clone(&state),
    )
    .await?;
    let plan = LogicalPlanBuilder::scan(
        "publication_control",
        provider_as_source(Arc::new(control)),
        None,
    )?
    .limit(0, Some(2))?
    .build()?;
    let batches = collect(state.create_physical_plan(&plan).await?, state.task_ctx()).await?;
    let batch = datafusion::arrow::compute::concat_batches(layout.execution_schema(), &batches)?;
    if batch.num_rows() != 1 {
        return Err(invalid(
            "publication control must contain exactly one workspace row",
        ));
    }
    let record = publications::View::try_from_batch_with_registry(registry, &batch)
        .map_err(external)?
        .row(0)
        .map_err(external)?;
    Ok(record)
}
pub(super) async fn bind_members(
    record: &publications::Row,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<Arc<SessionState>> {
    let catalogs = Arc::new(MemoryCatalogProviderList::new());
    for member in &record.members {
        let relation = registry
            .relation_by_id(member.relation_id)
            .ok_or_else(|| invalid("publication references an unknown relation contract"))?;
        if relation.key.version != member.relation_version
            || relation.fingerprint != member.contract_fingerprint
        {
            return Err(invalid(
                "publication relation version or fingerprint differs from its declaration",
            ));
        }
        let execution =
            Arc::new(pse_schema::arrow::relation_schema(registry, relation).map_err(external)?);
        let layout = DurableLayout::new(execution)?;
        let location = url::Url::parse(&member.table_uri).map_err(external)?;
        let view = open_view(location, member.delta_version, &layout, Arc::clone(&state)).await?;
        let view = match (&member.revision_column, member.revision_id) {
            (None, None) => view,
            (Some(column), Some(id)) => {
                let value = ScalarValue::FixedSizeBinary(16, Some(id.as_bytes().to_vec()));
                let plan = LogicalPlanBuilder::from(view.logical_plan().clone())
                    .filter(col(column).eq(lit(value)))?
                    .build()?;
                ViewTable::new(plan, None)
            }
            _ => {
                return Err(invalid(
                    "revision selection requires both column and identity",
                ));
            }
        };
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
            .register_table(member.table_name.clone(), Arc::new(view))?
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

pub(super) async fn verify_inputs(
    record: &publications::Row,
    registry: &Registry,
    state: Arc<SessionState>,
) -> Result<()> {
    use pse_relations::typed::CellCodec;
    // Both vectors use the same declared nested Arrow shape. The generated codecs
    // transfer that shape without a second handwritten member declaration.
    let members = record
        .inputs
        .iter()
        .cloned()
        .map(|input| {
            publications::RuntimePublicationsFieldMembersItem::from_cell(input.into_cell())
                .map_err(external)
        })
        .collect::<Result<Vec<_>>>()?;
    let inputs = publications::Row {
        members,
        ..record.clone()
    };
    bind_members(&inputs, registry, state).await?;
    Ok(())
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
