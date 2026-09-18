// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit remote metadata resolution into retained native provider generations.

mod provider;

use super::{PreparedComputation, SnapshotSession};
use crate::{BoxFut, CatalogError};
use datafusion::{
    arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    catalog::{AsyncCatalogProviderList, CatalogProviderList},
    common::TableReference,
};
use pse_ids::CancellationToken;
use pse_schema::model::provider::{OperationEffect, OperationPurpose, ProviderScope};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

/// Requested consistency of the actual backend handle, not a validity assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionConsistency {
    /// The backend must retain one coherent metadata revision for this handle's lifetime.
    Snapshot,
    /// Explicitly observe changing metadata; no coherent-snapshot guarantee is inferred.
    Observation,
}

/// A backend handle implementing the native asynchronous catalog hierarchy.
/// Snapshot handles must bind every lookup and listed name to the same actual revision.
/// Table data still requires independent version binding or explicit value capture.
pub trait CatalogRevision: AsyncCatalogProviderList + std::fmt::Debug {
    /// Complete references in this revision, if exhaustive enumeration is supported.
    /// Absence means unsupported enumeration, never an empty catalog.
    fn all_references(&self) -> Option<&[TableReference]> {
        None
    }
}

/// Actual remote implementation. Opening happens only during an admitted operation.
pub trait CatalogSource: std::fmt::Debug + Send + Sync {
    /// Open the requested backend handle. Refuse snapshot mode when the backend cannot
    /// bind a coherent revision; a timestamp or digest alone does not implement it.
    fn open<'a>(
        &'a self,
        consistency: ResolutionConsistency,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, Result<Arc<dyn CatalogRevision>, CatalogError>>;
}

/// Exact discovery request. Names use the invoking session's native normalization.
#[derive(Clone, Debug)]
pub enum ResolutionRequest {
    /// Resolve just these references, including negative lookups, once each.
    References(Vec<TableReference>),
    /// Resolve the backend's complete revision inventory or fail explicitly.
    All,
}

/// A prepared native metadata operation. Constructing or dropping it performs no I/O.
#[derive(Debug)]
pub struct PreparedCatalogResolution {
    native: PreparedComputation,
    operation: Arc<Resolve>,
}
impl PreparedCatalogResolution {
    /// The actual operation and captured scope before any backend access.
    pub const fn computation(&self) -> &PreparedComputation {
        &self.native
    }
    /// Resolve the requested metadata with one in-flight backend request at a time.
    /// # Errors
    /// Backend failure, unsupported consistency/enumeration, cancellation or admission.
    pub async fn execute(
        self,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        self.native.execute(cancel).await?;
        self.operation
            .output
            .lock()
            .map_err(|_| invalid("resolution lock unavailable"))?
            .take()
            .ok_or_else(|| invalid("resolution completed without its captured generation"))
    }
}

/// Actual completed metadata owner and its precise lookup coverage.
#[derive(Debug)]
pub(crate) struct ResolutionGeneration {
    /// Keeps the backend transaction/revision alive for all bound table providers.
    _revision: Arc<dyn CatalogRevision>,
    _reservation: Mutex<Box<dyn pse_ids::Reservation>>,
    pub(crate) references: BTreeSet<TableReference>,
    pub(crate) consistency: ResolutionConsistency,
    pub(crate) exhaustive: bool,
}
impl ResolutionGeneration {
    pub(crate) fn covers(&self, reference: &TableReference) -> bool {
        self.exhaustive || self.references.contains(reference)
    }
}

#[derive(Debug)]
struct Resolve {
    source: Arc<dyn CatalogSource>,
    request: ResolutionRequest,
    consistency: ResolutionConsistency,
    output: Mutex<Option<SnapshotSession>>,
    started: std::sync::atomic::AtomicBool,
}
impl SnapshotSession {
    /// Prepare native asynchronous catalog discovery under the selected provider scope.
    /// # Errors
    /// Conflicting scope policies, disallowed observation or malformed input names.
    pub fn prepare_catalog_resolution(
        &self,
        source: Arc<dyn CatalogSource>,
        request: ResolutionRequest,
        consistency: ResolutionConsistency,
        cancel: &CancellationToken,
    ) -> Result<PreparedCatalogResolution, CatalogError> {
        let mut session = self.with_purpose(OperationPurpose::Resolve);
        if let ResolutionRequest::References(references) = &request {
            let config = session.bound_state()?.config_options().catalog.clone();
            for reference in references {
                let reference = reference
                    .clone()
                    .resolve(&config.default_catalog, &config.default_schema);
                session.bindings.target(ProviderScope::Table(
                    reference.catalog.to_string(),
                    reference.schema.to_string(),
                    reference.table.to_string(),
                ));
            }
        }
        let operation = Arc::new(Resolve {
            source,
            request,
            consistency,
            output: Mutex::default(),
            started: std::sync::atomic::AtomicBool::new(false),
        });
        let reference = TableReference::full(
            "__pse_metadata",
            "invocation",
            format!("resolution_{:p}", Arc::as_ptr(&operation)),
        );
        let source: Arc<dyn datafusion::catalog::TableProvider> =
            Arc::new(provider::ResolutionProvider {
                operation: Arc::clone(&operation),
                session: session.clone(),
            });
        let bound = session.with_provider(reference.clone(), Arc::clone(&source), cancel)?;
        let plan = datafusion::logical_expr::LogicalPlanBuilder::scan(
            reference,
            datafusion::datasource::provider_as_source(source),
            None,
        )
        .and_then(datafusion::logical_expr::LogicalPlanBuilder::build)
        .map_err(super::engine)?;
        let native = bound.prepare(plan, cancel)?;
        Ok(PreparedCatalogResolution { native, operation })
    }
}
impl Resolve {
    fn schema() -> SchemaRef {
        Arc::new(Schema::new(vec![Field::new(
            "count",
            DataType::UInt64,
            false,
        )]))
    }
    fn effects() -> BTreeSet<OperationEffect> {
        [OperationEffect::Read, OperationEffect::Observe]
            .into_iter()
            .collect()
    }
    fn execute<'a>(
        &'a self,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, Result<RecordBatch, CatalogError>> {
        Box::pin(async move {
            let revision = cancel
                .until_cancelled(self.source.open(self.consistency, session, cancel))
                .await??;
            let state = session.bound_state()?;
            let config = state.config();
            let requested = match &self.request {
                ResolutionRequest::References(references) => references.as_slice(),
                ResolutionRequest::All => revision
                    .all_references()
                    .ok_or_else(|| invalid("backend does not support exhaustive enumeration"))?,
            };
            let mut reservation = session.reserver().open("provider:resolution-names");
            let bytes = requested.iter().try_fold(1024_usize, |total, reference| {
                let text = reference
                    .catalog()
                    .map_or(0, str::len)
                    .checked_add(reference.schema().map_or(0, str::len))
                    .and_then(|n| n.checked_add(reference.table().len()))
                    .and_then(|n| n.checked_mul(8))
                    .and_then(|n| n.checked_add(4096))
                    .ok_or_else(|| invalid("resolution name allocation overflows"))?;
                total
                    .checked_add(text)
                    .ok_or_else(|| invalid("resolution inventory allocation overflows"))
            })?;
            reservation.try_grow(bytes)?;
            let references: BTreeSet<_> = requested
                .iter()
                .map(|reference| {
                    let reference = reference.clone().resolve(
                        &config.options().catalog.default_catalog,
                        &config.options().catalog.default_schema,
                    );
                    TableReference::full(reference.catalog, reference.schema, reference.table)
                })
                .collect();
            // The pinned native resolver deduplicates catalog/schema/table requests,
            // including absent entries. Its serial traversal bounds concurrency at one.
            let mut scoped = session.clone();
            for reference in &references {
                scoped.bindings.target(ProviderScope::Table(
                    reference.catalog().unwrap_or_default().to_owned(),
                    reference.schema().unwrap_or_default().to_owned(),
                    reference.table().to_owned(),
                ));
            }
            scoped.effective_policy()?.admit(&Self::effects())?;
            let state = scoped.bound_state()?;
            let native = cancel
                .until_cancelled(revision.resolve(
                    &references.iter().cloned().collect::<Vec<_>>(),
                    state.config(),
                ))
                .await?
                .map_err(super::engine)?;
            cancel.checkpoint()?;
            let generation = Arc::new(ResolutionGeneration {
                _revision: revision,
                _reservation: Mutex::new(reservation),
                references,
                consistency: self.consistency,
                exhaustive: matches!(self.request, ResolutionRequest::All),
            });
            let (result, count) = bind_resolved(&scoped, &native, generation, cancel).await?;
            result.check_requirements(cancel).await?;
            *self
                .output
                .lock()
                .map_err(|_| invalid("resolution lock unavailable"))? = Some(result);
            RecordBatch::try_new(
                Self::schema(),
                vec![Arc::new(UInt64Array::from(vec![count]))],
            )
            .map_err(|e| super::engine(e.into()))
        })
    }
}

async fn bind_resolved(
    session: &SnapshotSession,
    native: &Arc<dyn CatalogProviderList>,
    generation: Arc<ResolutionGeneration>,
    cancel: &CancellationToken,
) -> Result<(SnapshotSession, u64), CatalogError> {
    let mut result = session.clone();
    let mut count = 0;
    if generation.exhaustive {
        result
            .bindings
            .resolution("", Arc::clone(&generation))
            .map_err(super::engine)?;
    }
    for reference in &generation.references {
        cancel.checkpoint()?;
        let catalog_name = reference
            .catalog()
            .ok_or_else(|| invalid("unresolved catalog name"))?;
        let schema_name = reference
            .schema()
            .ok_or_else(|| invalid("unresolved schema name"))?;
        result
            .bindings
            .resolution(catalog_name, Arc::clone(&generation))
            .map_err(super::engine)?;
        let Some(catalog) = native.catalog(catalog_name) else {
            continue;
        };
        result
            .bindings
            .namespace(catalog_name, None)
            .map_err(super::engine)?;
        let Some(schema) = catalog.schema(schema_name) else {
            continue;
        };
        result
            .bindings
            .namespace(catalog_name, Some(schema_name))
            .map_err(super::engine)?;
        let Some(provider) = schema
            .table(reference.table())
            .await
            .map_err(super::engine)?
        else {
            continue;
        };
        // Discovery cannot establish row facts. This same ordinary native admission
        // rejects advertised constraints until an explicit value capture establishes them.
        result = result.with_provider(reference.clone(), provider, cancel)?;
        count += 1;
    }
    Ok((result, count))
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.resolution".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
