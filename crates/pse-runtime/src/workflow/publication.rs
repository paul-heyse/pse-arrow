// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable results use the existing native Delta publication and settlement boundary.
use super::{RunResult, WorkflowError, contract};
use datafusion::{
    arrow::array::{Array, Int64Array},
    common::ResolvedTableReference,
};
pub use pse_catalog::delta::ticket::{PublicationSettlement, PublicationTicket};
use pse_catalog::{
    artifact::{ArtifactPlan, PublicationTarget, RelationOutput},
    delta::publication::PublicationRoot,
};
use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_relations::generated::{
    enums::{ArtifactReconstruction, PublicationKind},
    runtime::{artifact_descriptors as descriptor, publications},
};
use std::{collections::BTreeMap, sync::Arc};
/// Explicit immutable destination and identities for a single publication request.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationRequest {
    /// Directory holding control and immutable attempt members.
    pub base: url::Url,
    /// Workspace control identity.
    pub workspace_id: SemanticId,
    /// Exact expected parent; never rebased.
    pub parent: Option<SemanticId>,
    /// Caller-stable publication identity.
    pub publication_id: SemanticId,
    /// Caller-stable attempt identity.
    pub attempt_id: SemanticId,
}
impl PublicationRequest {
    /// Mint fresh identities before preparation; retain this value with the ticket.
    pub fn new(base: url::Url, workspace_id: SemanticId, parent: Option<SemanticId>) -> Self {
        Self {
            base,
            workspace_id,
            parent,
            publication_id: pse_authoring::ids::uuid_v7(),
            attempt_id: pse_authoring::ids::uuid_v7(),
        }
    }
}
/// A fully prepared single-consumption command. No solve callback is retained.
#[derive(Debug)]
pub struct PublicationAttempt {
    command: pse_engine::session::PreparedComputation,
    /// Recoverable before commit consumes this command.
    pub ticket: PublicationTicket,
    root: url::Url,
    /// Stable attempt identity for native effect inspection after unresolved errors.
    pub attempt_id: SemanticId,
    /// Publication identity, separate from the run identity.
    pub publication_id: SemanticId,
    /// Complete descriptor for exact reopen and admission.
    pub descriptor: pse_model::artifact::ArtifactDescriptor,
}
impl PublicationAttempt {
    /// Consume once. Preserve native settlement errors; never blindly retry a write.
    pub async fn commit(
        self,
        cancel: &CancellationToken,
    ) -> Result<PublicationRoot, WorkflowError> {
        let completed = self.command.execute(cancel).await?;
        let batches = completed.batches();
        if batches.len() != 1 || batches[0].num_rows() != 1 {
            return Err(contract(
                "publication completed without one control version receipt",
            ));
        }
        let versions = batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| contract("publication receipt version type"))?;
        if versions.is_null(0) || versions.value(0) < 0 {
            return Err(contract(
                "publication receipt missing exact committed version",
            ));
        }
        Ok(PublicationRoot {
            location: self.root,
            version: versions.value(0),
        })
    }
}
impl RunResult {
    /// Prepare a complete control-last publication of immutable results and declarations.
    /// Locations and conditional parent are explicit. This method performs no writes.
    pub fn prepare_publication(
        &self,
        base: url::Url,
        workspace_id: SemanticId,
        parent: Option<SemanticId>,
        cancel: &CancellationToken,
    ) -> Result<PublicationAttempt, WorkflowError> {
        self.prepare_publication_request(
            PublicationRequest::new(base, workspace_id, parent),
            cancel,
        )
    }
    /// Prepare the explicit request and recovery ticket without writes or rerunning science.
    /// # Errors
    /// Invalid declarations, destinations or product obligations.
    pub fn prepare_publication_request(
        &self,
        request: PublicationRequest,
        cancel: &CancellationToken,
    ) -> Result<PublicationAttempt, WorkflowError> {
        let PublicationRequest {
            base,
            workspace_id,
            parent,
            publication_id,
            attempt_id,
        } = request;
        if base.cannot_be_a_base() || !base.path().ends_with('/') {
            return Err(contract(
                "publication base must be an absolute directory URL ending in /",
            ));
        }
        let tables = self.tables().map_err(WorkflowError::Shared)?;
        let mut rows = BTreeMap::new();
        for (id, batch) in tables {
            let spec = self
                .runtime
                .registry
                .relation_by_id(*id)
                .ok_or_else(|| contract("result declaration missing"))?;
            rows.insert(spec.key, batch.clone());
        }
        let session =
            self.runtime
                .sessions
                .candidate_checked(rows, self.runtime.registry.clone(), cancel)?;
        let mut outputs = BTreeMap::new();
        for id in tables.keys() {
            let spec = self
                .runtime
                .registry
                .relation_by_id(*id)
                .ok_or_else(|| contract("result declaration missing"))?;
            let source = ResolvedTableReference {
                catalog: "workspace".into(),
                schema: spec.key.namespace.as_str().into(),
                table: spec.key.name.into(),
            };
            let output = ResolvedTableReference {
                catalog: "artifact".into(),
                schema: source.schema.clone(),
                table: source.table.clone(),
            };
            outputs.insert(
                output,
                RelationOutput {
                    relation_id: *id,
                    plan: session.relation_plan(&source)?.plan().clone(),
                },
            );
        }
        let mut source = FramedHasher::new("pse.run.source.v1");
        let mut algorithms = FramedHasher::new("pse.run.algorithms.v1");
        let mut target = FramedHasher::new("pse.run.target.v1");
        match &self.request {
            super::run::RunRequest::Solves(steps) => {
                for step in steps {
                    source.hash(&step.revision.identity());
                    algorithms
                        .hash(&step.compiled().plan.structure().key())
                        .hash(&step.compiled().presolve.key);
                    if let Some(c) = step.solve.compatibility() {
                        target.hash(&c.layout).hash(&c.data);
                    }
                }
            }
            super::run::RunRequest::Fit(f) => {
                source.hash(&f.problem.revision.identity());
                algorithms.hash(&f.problem.profile_key);
                target.hash(&f.problem.key);
            }
            super::run::RunRequest::Simulation(s) => {
                source.hash(&s.revision.identity());
                algorithms.hash(&s.contract.identity);
                target.hash(&s.key);
            }
        }
        let semantic_identity = source.finish_hash();
        let descriptor = pse_model::artifact::ArtifactDescriptor::create(descriptor::Row {
            artifact_id: ContentHash::from_bytes([0; 32]),
            descriptor_version: 2,
            profile: PublicationKind::Run,
            profile_contract: pse_schema::fingerprint::semantic_profile(
                &self.runtime.registry,
                "run",
                &tables.keys().copied().collect(),
            )
            .map_err(|e| contract(e.to_string()))?,
            requested_relations: tables.keys().copied().collect(),
            release_id: semantic_identity,
            release_members: vec![],
            semantic_identity,
            implementation: descriptor::RuntimeArtifactDescriptorsFieldImplementation {
                source: pse_buildinfo::SOURCE_IDENTITY,
                build: pse_buildinfo::BUILD_IDENTITY,
                registry: self.runtime.registry.fingerprint(),
                algorithms: algorithms.finish_hash(),
            },
            target_contract: target.finish_hash(),
            value_assumptions: vec![
                descriptor::RuntimeArtifactDescriptorsFieldValueAssumptionsItem {
                    name: "run_id".into(),
                    canonical_value: self.run_id.as_bytes().to_vec(),
                },
            ],
            reconstruction: ArtifactReconstruction::None,
        })
        .map_err(|e| contract(e.to_string()))?;
        let artifact = ArtifactPlan::new(session, outputs, cancel)?
            .with_product(descriptor.clone(), cancel)?;
        let destinations = artifact
            .outputs()
            .keys()
            .map(|name| {
                Ok((
                    name.clone(),
                    base.join(&format!(
                        "members/{publication_id}/{attempt_id}/{}/{}/",
                        name.schema, name.table
                    ))
                    .map_err(|e| contract(e.to_string()))?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
        let root = base.join("control/").map_err(|e| contract(e.to_string()))?;
        let (command, ticket) = artifact.prepare_publication(
            PublicationTarget {
                reference: ResolvedTableReference {
                    catalog: "artifact".into(),
                    schema: "runtime".into(),
                    table: "publications".into(),
                },
                location: root.clone(),
            },
            publications::Row {
                workspace_id,
                publication_id,
                parent_publication_id: parent,
                attempt_id,
                kind: PublicationKind::Run,
                inputs: vec![],
                members: vec![],
            },
            destinations,
            vec![],
            cancel,
        )?;
        Ok(PublicationAttempt {
            command,
            ticket,
            root,
            publication_id,
            attempt_id,
            descriptor,
        })
    }
}
impl super::Runtime {
    /// Read-only recovery from a saved ticket; no preparation or materialization is retried.
    pub async fn settle_publication(
        &self,
        ticket: &PublicationTicket,
        cancel: &CancellationToken,
    ) -> PublicationSettlement {
        ticket
            .settle(
                self.registry.clone(),
                Arc::new(self.sessions.native_state().clone()),
                cancel,
            )
            .await
    }

    /// Reopen an exact control version; no latest lookup or solver replay.
    pub async fn open(
        &self,
        root: PublicationRoot,
        cancel: &CancellationToken,
    ) -> Result<Arc<pse_catalog::delta::publication::Publication>, WorkflowError> {
        Ok(Arc::new(
            pse_catalog::delta::publication::Publication::open(
                root,
                self.registry.clone(),
                &self.sessions,
                cancel,
            )
            .await?,
        ))
    }
}
