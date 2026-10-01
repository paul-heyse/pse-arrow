// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Publication through the operational catalog (ADR-0114 Outcomes 1–9; Plan 22 X9–X12).
//!
//! A durable attempt's immutable results are published in its registered workspace:
//! 1. the publication intent (identity, durable attempt, member prefix) is registered in
//!    the catalog before the first member write;
//! 2. the candidate executes: members are written to immutable Delta tables under the
//!    intent's prefix, and the complete record is admitted with every actual version;
//! 3. one catalog transaction makes the record visible if the workspace head is still the
//!    expected parent. A lost race is a conflict; the publisher re-prepares against the
//!    new head with the same intent and recovers the members it already wrote.
//!
//! The runtime composes the two crates: `pse-catalog` performs Delta member I/O and never
//! sees PostgreSQL, `pse-operations` owns the catalog statements. A lost commit
//! acknowledgement is reported as unresolved and never retried implicitly; settling the
//! ticket queries the catalog.
use super::{RunResult, WorkflowError, contract};
use datafusion::common::ResolvedTableReference;
use pse_catalog::artifact::{ArtifactPlan, RelationOutput};
pub use pse_catalog::delta::ticket::PublicationTicket;
use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_operations::{
    OperationsError,
    attempts::AttemptId,
    catalog::{
        Committed, MemberDescriptor, NewIntent, NewWorkspace, PublicationCommit, PublicationId,
        SettleRequest, Settlement, WorkspaceId,
    },
};
use pse_relations::generated::{
    enums::{ArtifactReconstruction, PublicationKind},
    runtime::{artifact_descriptors as descriptor, publication_manifests},
};
use std::collections::BTreeMap;

/// A registered publication workspace: one publication history with one head, whose
/// members are written under `root`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    /// The workspace identity.
    pub workspace_id: WorkspaceId,
    /// Its unique name.
    pub name: String,
    /// The directory its members are written under.
    #[serde(rename = "root_uri")]
    pub root: url::Url,
}

impl Workspace {
    fn of(
        row: pse_operations::catalog::RuntimeOperationalWorkspacesRow,
    ) -> Result<Self, WorkflowError> {
        Ok(Self {
            workspace_id: row.workspace_id,
            name: row.name,
            root: url::Url::parse(&row.root_uri).map_err(|error| contract(error.to_string()))?,
        })
    }
    /// The prefix every member of one publication of `attempt` is written under.
    /// # Errors
    /// The root cannot be joined.
    pub fn member_prefix(
        &self,
        attempt: AttemptId,
        publication: PublicationId,
    ) -> Result<url::Url, WorkflowError> {
        self.root
            .join(&format!("members/{attempt}/{publication}/"))
            .map_err(|error| contract(error.to_string()))
    }
}

/// A committed publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Published {
    /// The publication, now the head of its workspace.
    pub publication_id: PublicationId,
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// The parent it was committed on.
    pub parent: Option<PublicationId>,
    /// The durable attempt it publishes.
    pub attempt_id: AttemptId,
}

/// The settlement of a publication ticket whose commit outcome is unknown.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum PublicationSettlement {
    /// The ticket's publication is visible.
    Committed {
        /// The publication.
        publication_id: PublicationId,
    },
    /// Nothing was committed and nothing is in flight: the same ticket may commit again.
    ProvedNoncommit,
    /// The ticket can never commit as prepared: re-prepare against the head.
    Conflict {
        /// Why.
        reason: String,
        /// The workspace head at settlement.
        head: Option<PublicationId>,
    },
    /// The catalog could not be reached; nothing may be concluded. Settle again later.
    Unresolved {
        /// Why.
        reason: String,
    },
}

/// A prepared publication: the ticket exists, nothing is written. [`Self::commit`]
/// registers the intent, executes the candidate and commits it once.
#[derive(Debug)]
pub struct PublicationAttempt {
    command: pse_engine::session::PreparedComputation,
    /// Recoverable before commit consumes this command.
    pub ticket: PublicationTicket,
    operations: super::Operations,
    registry: std::sync::Arc<pse_schema::Registry>,
    /// The workspace published in.
    pub workspace: Workspace,
    /// The exact expected parent; never rebased.
    pub parent: Option<PublicationId>,
    /// The durable attempt published: the publication attempt itself (Plan 22 X9).
    pub attempt_id: AttemptId,
    /// The publication identity, separate from the run identity.
    pub publication_id: PublicationId,
    /// The member prefix of the publication's intent.
    pub member_prefix: url::Url,
    /// The complete product descriptor, for an artifact publication.
    pub descriptor: Option<pse_model::artifact::ArtifactDescriptor>,
}

impl PublicationAttempt {
    /// Consume once: register the intent, write and admit the candidate, commit it in the
    /// catalog. Never retried implicitly.
    /// # Errors
    /// [`OperationsError::PublicationConflict`] (through [`WorkflowError::Operations`])
    /// when the head moved: re-prepare with the same publication identity against the
    /// head; [`WorkflowError::PublicationUnresolved`] when the commit acknowledgement was
    /// lost or the catalog became unreachable after the members were written: settle the
    /// ticket; member write, admission or catalog refusals.
    pub async fn commit(self, cancel: &CancellationToken) -> Result<Published, WorkflowError> {
        let catalog = self.operations.store().catalog();
        catalog
            .register_intent(&NewIntent {
                publication_id: self.publication_id,
                workspace_id: self.workspace.workspace_id,
                attempt_id: self.attempt_id,
                member_prefix: self.member_prefix.to_string(),
            })
            .await?;
        let completed = self.command.execute(cancel).await?;
        let record = candidate_record(&completed, &self.registry)?;
        let request = PublicationCommit {
            publication_id: self.publication_id,
            workspace_id: self.workspace.workspace_id,
            attempt_id: self.attempt_id,
            expected_parent: self.parent,
            kind: record.kind,
            members: record.members,
            inputs: record.inputs,
            windows: record.windows,
        };
        match catalog.commit(&request).await {
            Ok(Committed::Advanced { .. } | Committed::AlreadyCommitted { .. }) => Ok(Published {
                publication_id: self.publication_id,
                workspace_id: self.workspace.workspace_id,
                parent: self.parent,
                attempt_id: self.attempt_id,
            }),
            Err(error @ OperationsError::Unavailable { .. }) => {
                Err(WorkflowError::PublicationUnresolved {
                    publication: self.publication_id,
                    reason: error.to_string(),
                })
            }
            Err(error) => Err(error.into()),
        }
    }
}

/// The one admitted record a candidate execution returns.
pub(super) fn candidate_record(
    completed: &pse_engine::session::CompletedComputation,
    registry: &pse_schema::Registry,
) -> Result<publication_manifests::Row, WorkflowError> {
    let batches = completed.batches();
    let [batch] = batches else {
        return Err(contract("a publication candidate returns one record"));
    };
    if batch.num_rows() != 1 {
        return Err(contract("a publication candidate returns one record"));
    }
    publication_manifests::View::try_from_batch_with_registry(registry, batch)
        .and_then(|view| view.row(0))
        .map_err(super::relation)
}

/// Prepare a publication of a composed artifact for a durable attempt in `workspace`:
/// members go under the intent's prefix, `{root}/members/{attempt}/{publication}/`, one
/// table per output (`{schema}/{table}/`). The attempt identifies the composition, so a
/// re-preparation of the same publication recovers the members already written. A
/// fresh publication identity is minted unless one is given (re-preparing after a
/// conflict reuses it). Performs no write.
/// # Errors
/// An invalid root, declarations, destinations, retained members or product obligations.
#[expect(
    clippy::too_many_arguments,
    reason = "one publication request: attempt, workspace, parent, identity, kind, artifact and retained members"
)]
pub fn prepare_artifact_publication(
    runtime: &super::Runtime,
    attempt: AttemptId,
    workspace: &Workspace,
    parent: Option<PublicationId>,
    publication_id: Option<PublicationId>,
    kind: PublicationKind,
    artifact: ArtifactPlan,
    retained: Vec<MemberDescriptor>,
    cancel: &CancellationToken,
) -> Result<PublicationAttempt, WorkflowError> {
    let super::Durability::Durable(operations) = runtime.durability() else {
        return Err(contract(
            "publication needs a durable runtime registered in the operational store (ADR-0114 Outcome 16)",
        ));
    };
    if workspace.root.cannot_be_a_base() || !workspace.root.path().ends_with('/') {
        return Err(contract(
            "a workspace root is an absolute directory URL ending in /",
        ));
    }
    let publication_id = publication_id.unwrap_or_else(pse_operations::mint_id);
    let member_prefix = workspace.member_prefix(attempt, publication_id)?;
    let artifact = artifact.with_operation(attempt.into())?;
    let descriptor = None;
    let destinations = artifact
        .outputs()
        .keys()
        .map(|name| {
            Ok((
                name.clone(),
                member_prefix
                    .join(&format!("{}/{}/", name.schema, name.table))
                    .map_err(|error| contract(error.to_string()))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
    // The exact inputs are the selections the outputs read (opened publications'
    // members); the candidate refuses any other vector.
    let mut inputs = BTreeMap::new();
    for output in artifact.outputs().values() {
        for member in
            pse_catalog::selection::selected_dependencies(artifact.session(), &output.plan, cancel)?
        {
            inputs.insert(
                (
                    member.catalog_name.clone(),
                    member.schema_name.clone(),
                    member.table_name.clone(),
                ),
                member,
            );
        }
    }
    let header = publication_manifests::Row {
        publication_id,
        workspace_id: workspace.workspace_id,
        parent_publication_id: parent,
        attempt_id: attempt,
        kind,
        inputs: inputs.into_values().collect(),
        members: vec![],
        windows: vec![],
        exported_at: None,
        export_lease_id: None,
        export_expires_at: None,
        maintenance_epoch: None,
        store_fingerprint: None,
    };
    let (command, ticket) = artifact.prepare_publication(header, destinations, retained, cancel)?;
    Ok(PublicationAttempt {
        command,
        ticket,
        operations: operations.clone(),
        registry: std::sync::Arc::clone(runtime.registry()),
        workspace: workspace.clone(),
        parent,
        attempt_id: attempt,
        publication_id,
        member_prefix,
        descriptor,
    })
}

impl RunResult {
    /// Prepare the publication of this durable attempt's immutable results and
    /// declarations in `workspace`, against the exact expected `parent`. The publication
    /// attempt is the run's durable attempt (Plan 22 X9). Performs no write and never
    /// reruns science.
    /// # Errors
    /// An ephemeral run, a durable run whose attempt was not recorded, or invalid
    /// declarations, destinations or product obligations.
    pub fn prepare_publication(
        &self,
        workspace: &Workspace,
        parent: Option<PublicationId>,
        publication_id: Option<PublicationId>,
        cancel: &CancellationToken,
    ) -> Result<PublicationAttempt, WorkflowError> {
        let attempt = match &self.durability {
            super::RunDurability::Ephemeral => {
                return Err(WorkflowError::EphemeralPublication {
                    run_id: self.run_id,
                });
            }
            super::RunDurability::Durable(record) => {
                record
                    .attempt
                    .as_ref()
                    .map_err(|error| WorkflowError::Shared(error.clone()))?;
                record.attempt_id
            }
        };
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
        let descriptor = self.descriptor(tables)?;
        let artifact = ArtifactPlan::new(session, outputs, cancel)?
            .with_product(descriptor.clone(), cancel)?;
        let mut prepared = prepare_artifact_publication(
            &self.runtime,
            attempt,
            workspace,
            parent,
            publication_id,
            PublicationKind::Run,
            artifact,
            vec![],
            cancel,
        )?;
        prepared.descriptor = Some(descriptor);
        Ok(prepared)
    }

    fn descriptor(
        &self,
        tables: &BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>,
    ) -> Result<pse_model::artifact::ArtifactDescriptor, WorkflowError> {
        let mut source = FramedHasher::new(pse_ids::Frame::RunSourceV1);
        let mut algorithms = FramedHasher::new(pse_ids::Frame::RunAlgorithmsV1);
        let mut target = FramedHasher::new(pse_ids::Frame::RunTargetV1);
        match &self.request {
            super::run::RunRequest::Modeling(steps) => {
                for p in steps {
                    source.hash(&p.source.revision.identity());
                    algorithms
                        .hash(&p.model.case.compiled().plan.structure().key())
                        .hash(&p.model.case.compiled().presolve.key);
                    target.hash(
                        &p.solve
                            .request_identity()
                            .map_err(crate::math::MathRuntimeError::from)?,
                    );
                }
            }
            super::run::RunRequest::Fit(f) => {
                source.hash(&f.problem.source_identity);
                algorithms.hash(&f.problem.profile_key);
                target.hash(&f.problem.key);
            }
            #[cfg(feature = "solver-diffsol")]
            super::run::RunRequest::Shooting { problem, initial } => {
                source.hash(&problem.simulation.source.revision.identity());
                algorithms.hash(&problem.profile_key);
                target.hash(&problem.request_identity(initial.as_deref()));
            }
            super::run::RunRequest::Simulation(s) => {
                source.hash(&s.source.revision.identity());
                algorithms.hash(&s.contract().identity);
                target.hash(&s.identity());
            }
        }
        let semantic_identity = source.finish_hash();
        pse_model::artifact::ArtifactDescriptor::create(descriptor::Row {
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
        .map_err(|e| contract(e.to_string()))
    }
}

impl super::Runtime {
    /// The operations of a durable runtime.
    pub(crate) fn operations(&self) -> Result<&super::Operations, WorkflowError> {
        match &self.durability {
            super::Durability::Durable(operations) => Ok(operations),
            super::Durability::Ephemeral => Err(contract(
                "the publication catalog needs a durable runtime (ADR-0114 Outcome 16)",
            )),
        }
    }

    /// Register a publication workspace with its member root, or return the one of that
    /// name with the same root. A root holding a Delta publication control table (the
    /// former format) is refused: its publications are regenerated by rerunning.
    /// # Errors
    /// An ephemeral runtime; an invalid root; a name registered with another root or a
    /// root registered under another name; a legacy root; store failures.
    pub async fn register_workspace(
        &self,
        name: &str,
        root: url::Url,
    ) -> Result<Workspace, WorkflowError> {
        let catalog = self.operations()?.store().catalog();
        if root.cannot_be_a_base() || !root.path().ends_with('/') {
            return Err(contract(
                "a workspace root is an absolute directory URL ending in /",
            ));
        }
        if let Some(existing) = catalog.workspace_by_name(name).await? {
            let existing = Workspace::of(existing)?;
            return if existing.root == root {
                Ok(existing)
            } else {
                Err(contract(format!(
                    "workspace {name} is registered with root {}",
                    existing.root
                )))
            };
        }
        let control = root
            .join("control/_delta_log/")
            .map_err(|error| contract(error.to_string()))?;
        if self.holds_objects(&control).await? {
            return Err(WorkflowError::LegacyWorkspace { root });
        }
        let row = catalog
            .register_workspace(&NewWorkspace {
                workspace_id: pse_operations::mint_id(),
                name: name.to_owned(),
                root_uri: root.to_string(),
            })
            .await?;
        Workspace::of(row)
    }

    /// Whether any object exists under `prefix` in the deployment's object stores.
    async fn holds_objects(&self, prefix: &url::Url) -> Result<bool, WorkflowError> {
        use futures_util::StreamExt;
        let state = self.sessions.native_state();
        let store = state
            .runtime_env()
            .object_store_registry
            .get_store(prefix)
            .map_err(|error| WorkflowError::Engine(pse_engine::session::engine(error)))?;
        let path = object_store::path::Path::from_url_path(prefix.path())
            .map_err(|error| contract(error.to_string()))?;
        Ok(store.list(Some(&path)).next().await.is_some())
    }

    /// A registered workspace by name.
    /// # Errors
    /// An ephemeral runtime; an unknown name; store failures.
    pub async fn workspace(&self, name: &str) -> Result<Workspace, WorkflowError> {
        let row = self
            .operations()?
            .store()
            .catalog()
            .workspace_by_name(name)
            .await?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "workspace",
                id: name.to_owned(),
            })?;
        Workspace::of(row)
    }

    /// The head of a workspace; `None` before its first publication.
    /// # Errors
    /// An ephemeral runtime; an unknown workspace; store failures.
    pub async fn head(
        &self,
        workspace: WorkspaceId,
    ) -> Result<Option<PublicationId>, WorkflowError> {
        Ok(self.operations()?.store().catalog().head(workspace).await?)
    }

    /// Settle a ticket whose commit outcome is unknown, by querying the catalog. Read-only
    /// except for the recorded settlement; nothing is prepared or written again.
    pub async fn settle_publication(&self, ticket: &PublicationTicket) -> PublicationSettlement {
        let operations = match self.operations() {
            Ok(operations) => operations,
            Err(error) => {
                return PublicationSettlement::Unresolved {
                    reason: error.to_string(),
                };
            }
        };
        let request = SettleRequest {
            settlement_id: pse_operations::mint_id(),
            attempt_id: ticket.attempt_id(),
            publication_id: ticket.publication_id(),
            workspace_id: ticket.workspace_id(),
            expected_parent: ticket.parent_publication_id(),
        };
        match operations.store().catalog().settle(&request).await {
            Ok(Settlement::Committed { publication_id }) => {
                PublicationSettlement::Committed { publication_id }
            }
            Ok(Settlement::ProvedNoncommit) => PublicationSettlement::ProvedNoncommit,
            Ok(Settlement::Conflict { reason, head }) => {
                PublicationSettlement::Conflict { reason, head }
            }
            Err(error) => PublicationSettlement::Unresolved {
                reason: error.to_string(),
            },
        }
    }
}
