// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Store operations execute inside the common native attempt and resource scope.

use crate::{
    BoxFut, Catalog, CatalogError,
    session::{
        CompletedComputation, PreparedComputation, SnapshotSession, operation::NativeOperation,
    },
};
use datafusion::{
    arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema, SchemaRef},
    },
    logical_expr::Expr,
};
use pse_ids::CancellationToken;
use pse_schema::model::provider::{OperationEffect, OperationPurpose, ProviderScope};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

type Work<T> = Box<
    dyn FnOnce(
            Catalog,
            SnapshotSession,
            CancellationToken,
        ) -> BoxFut<'static, Result<(T, u64), CatalogError>>
        + Send,
>;

/// Captured command metadata projected into provider policy and native EXPLAIN.
pub(crate) struct StoreCommand {
    pub(crate) name: &'static str,
    pub(crate) scope: ProviderScope,
    pub(crate) purpose: OperationPurpose,
    pub(crate) arguments: Vec<Expr>,
}

/// Inspectable prepared store operation. Its work and typed result cannot be supplied
/// by a consumer after preparation, and native execution owns its once-only guard.
pub struct PreparedStoreOperation<T: Send + 'static> {
    native: PreparedComputation,
    task: Arc<StoreTask<T>>,
}
impl<T: Send + 'static> std::fmt::Debug for PreparedStoreOperation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedStoreOperation")
            .field("native", &self.native)
            .finish_non_exhaustive()
    }
}
impl<T: Send + 'static> PreparedStoreOperation<T> {
    /// Original/analyzed/optimized plans, captured effects and retained source bindings.
    pub const fn computation(&self) -> &PreparedComputation {
        &self.native
    }
    /// Exact write outcomes remain inspectable if a native stream is dropped.
    pub fn publication_journal(&self) -> super::publication::PublicationJournal {
        self.task.journal.clone()
    }
    /// Execute the native operation and retain its actual typed result and count stream.
    /// # Errors
    /// Policy, source resolution, admission, publication, cancellation or resource failure.
    pub async fn execute(
        self,
        cancel: &CancellationToken,
    ) -> Result<CompletedStoreOperation<T>, CatalogError> {
        let completion = self.native.execute(cancel).await?;
        let value = self
            .task
            .output
            .lock()
            .map_err(|_| invalid("result lock poisoned"))?
            .take()
            .ok_or_else(|| invalid("native operation completed without its typed result"))?;
        Ok(CompletedStoreOperation { value, completion })
    }
}
/// Typed store result paired with its real native execution completion.
#[derive(Debug)]
pub struct CompletedStoreOperation<T> {
    value: T,
    completion: CompletedComputation,
}
impl<T> CompletedStoreOperation<T> {
    /// Native execution observation and completed outcome batches.
    pub const fn computation(&self) -> &CompletedComputation {
        &self.completion
    }
    /// Read the actual completed value.
    pub const fn value(&self) -> &T {
        &self.value
    }
    /// Consume the completed handle and transfer its typed result.
    pub fn into_value(self) -> T {
        self.value
    }
}
struct StoreTask<T> {
    name: &'static str,
    effects: BTreeSet<OperationEffect>,
    arguments: Vec<Expr>,
    catalog: Catalog,
    journal: super::publication::PublicationJournal,
    work: Mutex<Option<Work<T>>>,
    output: Mutex<Option<T>>,
}
impl<T> std::fmt::Debug for StoreTask<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTask")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}
impl<T: Send + 'static> NativeOperation for StoreTask<T> {
    fn handles_cancellation(&self) -> bool {
        true
    }
    fn execution_failure(&self, error: CatalogError) -> CatalogError {
        self.journal.failure(error)
    }
    fn irreversible_completion(&self) -> bool {
        self.output.lock().is_ok_and(|output| output.is_some()) && self.journal.has_visible()
    }
    fn name(&self) -> &str {
        self.name
    }
    fn effects(&self) -> BTreeSet<OperationEffect> {
        self.effects.clone()
    }
    fn arguments(&self) -> Vec<Expr> {
        self.arguments.clone()
    }
    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![Field::new(
            "count",
            DataType::UInt64,
            false,
        )]))
    }
    fn execute<'a>(
        &'a self,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, Result<RecordBatch, CatalogError>> {
        Box::pin(async move {
            let work = self
                .work
                .lock()
                .map_err(|_| invalid("work lock poisoned"))?
                .take()
                .ok_or_else(|| invalid("store work was already consumed"))?;
            let mut catalog = self.catalog.clone();
            catalog.reserver = Arc::clone(&session.reserver);
            catalog.execution = Some(session.clone());
            let (value, count) = work(catalog, session.clone(), cancel.clone()).await?;
            *self
                .output
                .lock()
                .map_err(|_| invalid("result lock poisoned"))? = Some(value);
            RecordBatch::try_new(
                self.schema(),
                vec![Arc::new(UInt64Array::from(vec![count]))],
            )
            .map_err(|error| CatalogError::Infrastructure {
                op: "native store outcome".into(),
                source: Box::new(error),
            })
        })
    }
}
impl Catalog {
    pub(crate) fn bind_snapshot(
        &self,
        session: &mut SnapshotSession,
        catalog: &str,
        schema: &str,
        snapshot: &Arc<crate::Snapshot>,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        use crate::provider::{
            binding::{BindingKey, TableBinding},
            table::RelationTable,
        };
        for (port, relation) in snapshot.relations() {
            cancel.checkpoint()?;
            let reference = datafusion::common::TableReference::full(catalog, schema, port.clone());
            let key = BindingKey::Native(reference.clone());
            if let Some(existing) = session.bindings.get(&key) {
                if existing
                    .provider
                    .downcast_ref::<RelationTable>()
                    .is_some_and(|table| {
                        std::ptr::eq::<crate::Manifest>(
                            table.snapshot().manifest().as_ref(),
                            snapshot.manifest().as_ref(),
                        ) && Arc::ptr_eq(table.relation(), relation)
                    })
                {
                    continue;
                }
                return Err(invalid(
                    "snapshot port conflicts with an existing operation binding",
                ));
            }
            let spec = self
                .registry
                .relation_by_id(relation.contract().canonical.relation_id)
                .ok_or_else(|| invalid("snapshot declaration absent"))?;
            session
                .bindings
                .insert(
                    key,
                    TableBinding::new(
                        reference,
                        Arc::new(RelationTable::from_port(
                            Arc::clone(snapshot),
                            port,
                            &self.registry,
                        )?),
                        Some(spec.key),
                        Some(relation.checked().clone()),
                    ),
                )
                .map_err(crate::session::engine)?;
        }
        Ok(())
    }

    pub(crate) fn bind_draft(
        &self,
        session: &mut SnapshotSession,
        reference: datafusion::common::TableReference,
        draft: &super::publish::RelationDraft,
    ) -> Result<(), CatalogError> {
        let spec = self
            .registry
            .relation_by_id(draft.contract.canonical.relation_id)
            .ok_or_else(|| invalid("operation input declaration absent"))?;
        draft
            .contract
            .validate_against_registry(&self.registry, spec)?;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(&self.registry, spec)
                .map_err(|error| invalid(&error.to_string()))?,
        );
        let provider = Arc::new(
            crate::provider::batches::BatchInput::try_new(schema, draft.batches.clone())
                .map_err(crate::session::engine)?,
        );
        session
            .bindings
            .insert(
                crate::provider::binding::BindingKey::Native(reference.clone()),
                crate::provider::binding::TableBinding::new(
                    reference,
                    provider,
                    Some(spec.key),
                    None,
                ),
            )
            .map_err(crate::session::engine)
    }

    pub(crate) fn context_session(
        &self,
        context: &super::membership::AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        let mut session = self.validation_session(cancel)?;
        for (role, snapshot) in &context.parents {
            self.bind_snapshot(&mut session, "parents", role, snapshot, cancel)?;
        }
        Ok(session)
    }
    pub(crate) fn prepare_store_operation<T: Send + 'static>(
        &self,
        name: &'static str,
        scope: ProviderScope,
        purpose: OperationPurpose,
        arguments: Vec<Expr>,
        work: Work<T>,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<T>, CatalogError> {
        let session = self.validation_session(cancel)?;
        self.prepare_store_operation_in(
            &session,
            StoreCommand {
                name,
                scope,
                purpose,
                arguments,
            },
            work,
            cancel,
        )
    }

    pub(crate) fn prepare_store_operation_in<T: Send + 'static>(
        &self,
        session: &SnapshotSession,
        command: StoreCommand,
        work: Work<T>,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<T>, CatalogError> {
        let mut session = session.with_purpose(command.purpose);
        session.bindings.target(command.scope);
        let mut catalog = self.clone();
        let journal = self.publication.clone().unwrap_or_default();
        catalog.publication = Some(journal.clone());
        let task = Arc::new(StoreTask {
            name: command.name,
            effects: match command.purpose {
                OperationPurpose::Resolve => [OperationEffect::Read, OperationEffect::Observe]
                    .into_iter()
                    .collect(),
                OperationPurpose::Publish => [
                    OperationEffect::Read,
                    OperationEffect::Observe,
                    OperationEffect::Publish,
                ]
                .into_iter()
                .collect(),
                _ => command.purpose.effects(),
            },
            arguments: command.arguments,
            catalog,
            journal,
            work: Mutex::new(Some(work)),
            output: Mutex::new(None),
        });
        let operation: Arc<dyn NativeOperation> = task.clone();
        let native = session.prepare_operation(operation, cancel)?;
        Ok(PreparedStoreOperation { native, task })
    }
}
fn invalid(reason: &str) -> CatalogError {
    super::verify::admission("store operation", reason)
}
