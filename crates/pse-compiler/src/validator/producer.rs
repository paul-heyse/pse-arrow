// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The sole adapter from declared compiler operations to catalog-owned completion.

use crate::{
    BoundInput, CompilerError, InputBundle, Pass, PassContext, PolicySet, passes::dag::invalid,
};
use pse_catalog::{
    Catalog, CatalogError,
    computation::{ProducedStage, StageProducer},
    session::SessionFactory,
    store::membership::AdmissionContext,
};
use pse_ids::CancellationToken;
use pse_schema::{
    Registry,
    model::{PassSpec, PortSource},
};
use std::{any::Any, sync::Arc};

/// Immutable local control arguments minted only by compiler request construction.
/// Source owners and actual policy handles are shared, never reconstructed here.
#[derive(Clone)]
pub(crate) struct InvocationArguments {
    pub(crate) physical: Arc<super::physical::PhysicalInventories>,
    pub(crate) documents: pse_authoring::document::OwnedDocumentSet,
    pub(crate) policies: PolicySet,
    pub(crate) session: pse_catalog::session::SnapshotSession,
}
pub(super) struct RegisteredPass {
    pub(super) pass: Arc<dyn Pass>,
    pub(super) sessions: Arc<SessionFactory>,
}
impl std::fmt::Debug for RegisteredPass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredPass")
            .field("declaration", self.pass.spec())
            .finish_non_exhaustive()
    }
}
impl StageProducer for RegisteredPass {
    fn spec(&self) -> &PassSpec {
        self.pass.spec()
    }
    fn bind_session(
        &self,
        catalog: &Catalog,
        parents: &AdmissionContext,
        arguments: Option<&(dyn Any + Send + Sync)>,
        cancel: &CancellationToken,
    ) -> Result<pse_catalog::session::SnapshotSession, CatalogError> {
        let build = || -> Result<_, CompilerError> {
            if let Some(arguments) = arguments {
                let arguments = arguments
                    .downcast_ref::<InvocationArguments>()
                    .ok_or_else(|| invalid("different typed invocation context"))?;
                return Ok(arguments.session.clone());
            }
            let inputs = bind(self.pass.spec(), catalog.registry(), parents)?;
            let session = self.sessions.candidate_checked_ports(
                inputs.checked_ports(),
                Arc::clone(catalog.registry()),
                cancel,
            )?;
            if let Some(engine) = parents
                .invocation
                .as_ref()
                .and_then(|invocation| invocation.engine.as_ref())
            {
                Ok(session.restore_engine(engine)?)
            } else {
                Ok(session)
            }
        };
        build()
            .map(|session| {
                session.with_purpose(pse_schema::model::provider::OperationPurpose::Construct)
            })
            .map_err(|error| CatalogError::Semantic(Arc::new(error)))
    }
    fn execute<'a>(
        &'a self,
        catalog: &'a Catalog,
        parents: &'a AdmissionContext,
        arguments: Option<&'a (dyn Any + Send + Sync)>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<ProducedStage, CatalogError>> {
        Box::pin(async move {
            self.run(catalog, parents, arguments, session, cancel)
                .await
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))
        })
    }
}
impl RegisteredPass {
    async fn restore_arguments(
        &self,
        catalog: &Catalog,
        parents: &AdmissionContext,
        session: &pse_catalog::session::SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<InvocationArguments, CompilerError> {
        let registry = catalog.registry();
        let invocation = parents.invocation.as_ref().ok_or_else(|| {
            invalid("compiler stage lacks its current durable invocation binding")
        })?;
        if invocation.engine.is_none() {
            return Err(invalid(
                "stored engine binding disagrees with the producing pass",
            ));
        }
        let policies = invocation
            .policies
            .iter()
            .map(|(role, policy)| {
                let relation = policy
                    .snapshot
                    .relations()
                    .get(&policy.port)
                    .ok_or_else(|| invalid("admitted policy member is absent"))?;
                let spec = registry
                    .relation_by_id(relation.contract().canonical.relation_id)
                    .ok_or_else(|| invalid("policy relation declaration is absent"))?;
                Ok((
                    role.clone(),
                    crate::passes::PolicyBinding {
                        policy_id: policy.policy_id,
                        input: BoundInput::bind_port(
                            Arc::clone(&policy.snapshot),
                            spec.key,
                            &policy.port,
                            registry,
                        )?,
                    },
                ))
            })
            .collect::<Result<_, CompilerError>>()?;
        let documents = parents
            .traversal
            .owner::<super::traversal::Documents>()?
            .get(catalog, invocation.document_source.as_ref(), cancel)
            .await?;
        Ok(InvocationArguments {
            physical: parents
                .traversal
                .owner::<super::physical::PhysicalInventories>()?,
            documents,
            policies: PolicySet(policies),
            session: session.clone(),
        })
    }

    async fn run(
        &self,
        catalog: &Catalog,
        parents: &AdmissionContext,
        arguments: Option<&(dyn Any + Send + Sync)>,
        bound_session: &pse_catalog::session::SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<ProducedStage, CompilerError> {
        let registry = catalog.registry();
        let spec = self.pass.spec();
        let started = std::time::Instant::now();
        tracing::info!(
            pass = spec.name,
            phase = "binding",
            "registered stage started"
        );
        let inputs = bind(spec, registry, parents)?;
        let reopening = arguments.is_none();
        let imported;
        let arguments = if let Some(arguments) = arguments {
            arguments
                .downcast_ref::<InvocationArguments>()
                .ok_or_else(|| invalid("producer received a different typed invocation context"))?
        } else {
            imported = self
                .restore_arguments(catalog, parents, bound_session, cancel)
                .await?;
            &imported
        };
        let physical_keys = crate::quantity_relations::inventory::input_keys(registry);
        let physical = if inputs.ports.values().flatten().any(|input| {
            physical_keys.iter().any(|key| {
                registry
                    .relation_by_key(*key)
                    .is_some_and(|spec| spec.id == input.relation_id())
            })
        }) {
            Some(
                arguments
                    .physical
                    .get(&inputs, bound_session, registry, cancel)
                    .await?,
            )
        } else {
            None
        };
        let ctx = PassContext {
            physical: physical.as_ref(),
            registry,
            documents: &arguments.documents,
            policies: &arguments.policies,
            cancel,
            reserver: bound_session.reserver(),
            session: bound_session,
        };
        if reopening {
            // Boundary admission checks actual selected keys and the same complete
            // dependency framing used before local execution.
            crate::passes::key::stage_key(spec, &inputs, &ctx)?;
        }
        tracing::info!(
            pass = spec.name,
            phase = "execution",
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "registered stage context ready"
        );
        let mut output = self.pass.run(&ctx, &inputs).await?;
        tracing::info!(
            pass = spec.name,
            phase = "output",
            elapsed_seconds = started.elapsed().as_secs_f64(),
            "registered stage body completed"
        );
        output.plans.extend(bound_session.execution_observations()?);
        Ok(output)
    }
}

fn bind(
    spec: &PassSpec,
    registry: &Registry,
    context: &AdmissionContext,
) -> Result<InputBundle, CompilerError> {
    let mut inputs = InputBundle::new();
    for port in &spec.inputs {
        let relation = registry
            .relation(&port.relation)
            .ok_or_else(|| invalid("stage input declaration missing"))?;
        let binding = context
            .parents
            .get(port.port)
            .map(|snapshot| match port.source {
                PortSource::Pinned => {
                    BoundInput::bind(Arc::clone(snapshot), relation.key, registry)
                }
                PortSource::Derived { port, .. } => {
                    BoundInput::bind_port(Arc::clone(snapshot), relation.key, port, registry)
                }
            })
            .transpose()?;
        inputs.ports.insert(port.port, binding);
    }
    inputs.validate(spec, registry)?;
    Ok(inputs)
}
