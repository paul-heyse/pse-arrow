// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registered producer invocation and private, single-use publication results.
//!
//! The catalog invokes the actual implementation retained by its admission context.
//! Opaque arguments are immutable Rust control objects interpreted only by that
//! implementation; relation inputs and outputs remain declared Arrow relations.

use std::{any::Any, collections::BTreeMap, sync::Arc};
mod ports;

use pse_ids::{CancellationToken, SnapshotKind};
use pse_relations::{RecordBatch, columnar::FieldCheckedBatch};
use pse_schema::model::PassSpec;

use crate::{
    BoxFut, Catalog, CatalogError, Snapshot,
    store::{
        membership::{self, AdmissionContext},
        publish::{BundleDraft, RelationDraft},
    },
};

/// The actual implementation bound to one authoritative producing pass.
pub trait StageProducer: Send + Sync + std::fmt::Debug {
    /// Exact registered declaration, checked before any execution.
    fn spec(&self) -> &PassSpec;
    /// Bind exact declared input ports and the actual captured engine before planning.
    /// # Errors
    /// Missing inputs, incompatible arguments or an invalid native environment.
    fn bind_session(
        &self,
        catalog: &Catalog,
        inputs: &AdmissionContext,
        arguments: Option<&(dyn Any + Send + Sync)>,
        cancel: &CancellationToken,
    ) -> Result<crate::session::SnapshotSession, CatalogError>;
    /// Execute the selected implementation once. `None` requests current-format
    /// import reconstruction from the actual admitted parents.
    fn execute<'a>(
        &'a self,
        catalog: &'a Catalog,
        inputs: &'a AdmissionContext,
        arguments: Option<&'a (dyn Any + Send + Sync)>,
        session: &'a crate::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, Result<ProducedStage, CatalogError>>;
}

/// Output of the registered implementation. This is not a publication capability.
/// Only the catalog executor can construct a `CompletedProduction` after all
/// declared output obligations have completed.
#[derive(Clone, Debug)]
pub struct ProducedStage {
    /// Every declared output port, including typed empty relations.
    pub outputs: BTreeMap<String, FieldCheckedBatch>,
    /// Actual diagnostic relations from this execution.
    pub findings: Vec<RecordBatch>,
    /// Actual support relations from this execution.
    pub derivations: Vec<RecordBatch>,
    /// Native plan observations from this execution.
    pub plans: Vec<crate::session::PlanObservation>,
}

/// One invocation retaining the exact catalog, producer, input roles and arguments.
pub struct PreparedProduction {
    native: crate::session::PreparedComputation,
    operation: Arc<ProductionOperation>,
}
struct ProductionOperation {
    catalog: Catalog,
    producer: Arc<dyn StageProducer>,
    context: AdmissionContext,
    arguments: Option<Arc<dyn Any + Send + Sync>>,
    output: std::sync::Mutex<Option<Arc<ProducedStage>>>,
}
impl std::fmt::Debug for PreparedProduction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedProduction")
            .field("operation", &self.operation)
            .finish_non_exhaustive()
    }
}

/// Single-use result of a successful registered producer and its residual program.
/// No public constructor or mutable output access exists.
#[derive(Debug)]
pub struct CompletedProduction {
    prepared: PreparedProduction,
    produced: Arc<ProducedStage>,
    completion: crate::session::CompletedComputation,
}
impl CompletedProduction {
    /// The actual native outcome stream completion and logical/physical observations.
    pub const fn computation(&self) -> &crate::session::CompletedComputation {
        &self.completion
    }
    /// Inspect exact completed ports through native providers; lookup cannot execute again.
    pub fn output_session(&self) -> crate::session::SnapshotSession {
        self.prepared
            .native
            .bound_session()
            .with_purpose(pse_schema::model::provider::OperationPurpose::Inspect)
    }
    /// Immutable complete values for read-only correspondence at an external boundary.
    pub fn outputs(&self) -> &BTreeMap<String, FieldCheckedBatch> {
        &self.produced.outputs
    }
}

/// Published stage and its retained execution observations.
#[derive(Debug)]
pub struct PublishedProduction {
    /// Complete admitted immutable output.
    pub snapshot: Arc<Snapshot>,
    /// Findings from the producer's single execution.
    pub findings: Vec<RecordBatch>,
    /// Support from the same execution.
    pub derivations: Vec<RecordBatch>,
    /// Native plan observations from the same execution.
    pub plans: Vec<crate::session::PlanObservation>,
}

impl Catalog {
    /// Bind a producer from this immutable catalog's implementation inventory.
    /// Arguments cannot replace the selected implementation or supply its outputs.
    /// # Errors
    /// Missing/mismatched producer declaration or invalid parent roles.
    pub fn prepare_production(
        &self,
        mut context: AdmissionContext,
        arguments: Option<Arc<dyn Any + Send + Sync>>,
        cancel: &CancellationToken,
    ) -> Result<PreparedProduction, CatalogError> {
        let id = context
            .stage_pass
            .ok_or_else(|| membership::refused("producing pass absent"))?;
        let producer = self
            .validator
            .as_ref()
            .and_then(|validator| validator.stage_producer(id))
            .ok_or_else(|| membership::refused("producing implementation is not registered"))?;
        if self.registry.pass(&producer.spec().qualified_name()) != Some(producer.spec())
            || producer.spec().id != id
        {
            return Err(membership::refused(
                "registered producer differs from the actual declaration",
            ));
        }
        let manifest = self.manifest_template(SnapshotKind::Stage, &context)?;
        membership::inventory(&self.registry, &manifest, &context, &self.admission)?;
        let session = producer.bind_session(self, &context, arguments.as_deref(), cancel)?;
        if let Some(engine) = context
            .invocation
            .as_ref()
            .and_then(|invocation| invocation.engine.as_ref())
        {
            if !session.matches_semantic_inputs(engine) {
                return Err(membership::refused(
                    "operation engine differs from its captured invocation",
                ));
            }
        } else {
            let mut allocation = session.reserver().open("operation:engine-capture");
            allocation.try_grow(session.semantic_inputs_extent()?)?;
            let engine = session.semantic_inputs();
            let invocation = context.invocation.as_ref();
            context.invocation = Some(crate::store::invocation::InvocationContext::capture(
                invocation.into_iter().flat_map(|invocation| {
                    invocation.policies.iter().map(|(role, policy)| {
                        (
                            role.as_str(),
                            policy.policy_id,
                            &policy.snapshot,
                            policy.port.as_str(),
                        )
                    })
                }),
                invocation.and_then(|invocation| invocation.document_source.as_ref()),
                Some(&engine),
                session.reserver(),
            )?);
        }
        let operation = Arc::new(ProductionOperation {
            catalog: self.clone(),
            producer,
            context,
            arguments,
            output: std::sync::Mutex::new(None),
        });
        let session = ports::bind(session, &operation)?;
        let native_operation: Arc<dyn crate::session::operation::NativeOperation> =
            operation.clone();
        let native = session.prepare_operation(native_operation, cancel)?;
        Ok(PreparedProduction { native, operation })
    }

    /// Publish a completed local computation without replaying its producer or
    /// repeating the residual program. Immutable artifacts finish before visibility.
    /// # Errors
    /// A foreign context, encoding/storage failure, cancellation or resource exhaustion.
    pub async fn publish_production(
        &self,
        completed: CompletedProduction,
        cancel: &CancellationToken,
    ) -> Result<PublishedProduction, CatalogError> {
        let completed = self
            .prepare_production_publication(completed, cancel)?
            .execute(cancel)
            .await?;
        let observation = completed.computation().observation().clone();
        let mut result = completed.into_value();
        result.plans.push(observation);
        Ok(result)
    }

    /// Bind an actually completed multi-output computation to its native publication.
    /// Output providers are inputs to this operation; scanning them never reruns production.
    /// # Errors
    /// A foreign completion, conflicting binding or inadmissible publication policy.
    pub fn prepare_production_publication(
        &self,
        completed: CompletedProduction,
        cancel: &CancellationToken,
    ) -> Result<crate::store::operation::PreparedStoreOperation<PublishedProduction>, CatalogError>
    {
        use crate::provider::binding::BindingKey;
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let mut session = completed.prepared.native.bound_session();
        let outputs = session
            .bindings
            .iter()
            .filter(|(key, _)| matches!(key, BindingKey::Output(_)))
            .map(|(_, binding)| binding.clone())
            .collect::<Vec<_>>();
        for binding in outputs {
            session
                .bindings
                .insert(BindingKey::Native(binding.reference.clone()), binding)
                .map_err(crate::session::engine)?;
        }
        let producer = completed.prepared.operation.producer.spec().id.to_string();
        self.prepare_store_operation_in(
            &session,
            crate::store::operation::StoreCommand {
                name: "store.publish_production",
                scope: ProviderScope::Schema("store".into(), "manifests".into()),
                purpose: OperationPurpose::Publish,
                arguments: vec![datafusion::logical_expr::lit(producer)],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let result = catalog.publish_production_inner(completed, &cancel).await?;
                    let count = u64::try_from(result.snapshot.relations().len())
                        .map_err(|_| membership::refused("output count overflows"))?;
                    Ok((result, count))
                })
            }),
            cancel,
        )
    }

    async fn publish_production_inner(
        &self,
        completed: CompletedProduction,
        cancel: &CancellationToken,
    ) -> Result<PublishedProduction, CatalogError> {
        if !Arc::ptr_eq(
            &self.admission,
            &completed.prepared.operation.catalog.admission,
        ) {
            return Err(membership::refused(
                "completed producer belongs to a different catalog context",
            ));
        }
        let context = completed.prepared.operation.context.clone();
        let manifest = self.manifest_template(SnapshotKind::Stage, &context)?;
        let spec = completed.prepared.operation.producer.spec();
        let mut relations = BTreeMap::new();
        let mut candidates = BTreeMap::new();
        for port in &spec.outputs {
            let relation = self
                .registry
                .relation(&port.relation)
                .ok_or_else(|| membership::refused("output declaration absent"))?;
            let checked = completed.produced.outputs[port.port].clone();
            let batch = checked.batch().clone();
            candidates.insert(port.port.to_owned(), checked);
            relations.insert(
                port.port.to_owned(),
                RelationDraft {
                    contract: Arc::new(crate::RelationContract::from_spec(
                        &self.registry,
                        relation,
                        crate::EncodingPolicy::IpcFile,
                    )?),
                    batches: vec![batch],
                },
            );
        }
        let snapshot = self
            .publish_admitted_bundle(
                BundleDraft {
                    manifest,
                    relations,
                    context,
                },
                candidates,
                cancel,
            )
            .await?;
        Ok(PublishedProduction {
            snapshot,
            findings: completed.produced.findings.clone(),
            derivations: completed.produced.derivations.clone(),
            plans: completed
                .produced
                .plans
                .iter()
                .cloned()
                .chain(std::iter::once(completed.completion.observation().clone()))
                .collect(),
        })
    }
}

impl PreparedProduction {
    /// Exact selected producer declaration.
    pub fn spec(&self) -> &PassSpec {
        self.operation.producer.spec()
    }
    /// Actual bound parent roles, retained before any optimizer can eliminate a read.
    pub fn inputs(&self) -> &AdmissionContext {
        &self.operation.context
    }
    /// Run the registered producer once and finish all residual output obligations.
    /// # Errors
    /// Producer failure, missing output, unresolved predicate, cancellation or resources.
    pub async fn execute(
        self,
        cancel: &CancellationToken,
    ) -> Result<CompletedProduction, CatalogError> {
        let completion = self.native.clone().execute(cancel).await?;
        let produced = self
            .operation
            .output
            .lock()
            .map_err(|_| membership::refused("operation output lock poisoned"))?
            .as_ref()
            .cloned()
            .ok_or_else(|| {
                membership::refused("native operation did not complete its output catalog")
            })?;
        Ok(CompletedProduction {
            prepared: self,
            produced,
            completion,
        })
    }
}
impl std::fmt::Debug for ProductionOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductionOperation")
            .field("producer", &self.producer)
            .field("input_roles", &self.context.parents.keys())
            .finish_non_exhaustive()
    }
}
impl crate::session::operation::NativeOperation for ProductionOperation {
    fn name(&self) -> &str {
        self.producer.spec().name
    }
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        Arc::new(datafusion::arrow::datatypes::Schema::new(vec![
            datafusion::arrow::datatypes::Field::new(
                "count",
                datafusion::arrow::datatypes::DataType::UInt64,
                false,
            ),
        ]))
    }
    fn effects(&self) -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
        self.producer.spec().effects.clone()
    }
    fn execute<'a>(
        &'a self,
        session: &'a crate::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> BoxFut<'a, Result<RecordBatch, CatalogError>> {
        Box::pin(async move {
            cancel.checkpoint()?;
            let produced = self
                .producer
                .execute(
                    &self.catalog,
                    &self.context,
                    self.arguments.as_deref(),
                    session,
                    cancel,
                )
                .await?;
            let manifest = self
                .catalog
                .manifest_template(SnapshotKind::Stage, &self.context)?;
            let inventory = membership::inventory(
                &self.catalog.registry,
                &manifest,
                &self.context,
                &self.catalog.admission,
            )?;
            if produced.outputs.keys().ne(inventory.keys()) {
                return Err(membership::refused(
                    "producer did not complete every declared output port",
                ));
            }
            for (port, checked) in &produced.outputs {
                checked.check_declaration(&self.catalog.registry, inventory[port])?;
            }
            self.catalog
                .admit_ports(
                    SnapshotKind::Stage,
                    &produced.outputs,
                    &inventory,
                    &self.context,
                    cancel,
                )
                .await?;
            cancel.checkpoint()?;
            let count = u64::try_from(produced.outputs.len())
                .map_err(|_| membership::refused("output count exceeds native outcome range"))?;
            *self
                .output
                .lock()
                .map_err(|_| membership::refused("operation output lock poisoned"))? =
                Some(Arc::new(produced));
            RecordBatch::try_new(
                self.schema(),
                vec![Arc::new(datafusion::arrow::array::UInt64Array::from(vec![
                    count,
                ]))],
            )
            .map_err(pse_relations::RelationError::from)
            .map_err(CatalogError::from)
        })
    }
}
