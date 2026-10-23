// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Project and research operations over the canonical source owner, never a second model store.
use crate::canonical::REQUEST_TIMEOUT;
use crate::canonical::transport::{original_deadline, within_clock};
use crate::{
    canonical::{
        CanonicalError, CanonicalStore, ObjectEdit, Revision, bounded_query, protected_query,
    },
    canonical_codec as codec,
    generated::surreal as wire,
};
pub use pse_model::generated::runtime::{
    canonical_draft_diagnostics::Row as DraftDiagnostic,
    canonical_project_bookmarks::Row as Bookmark, canonical_project_branches::Row as Branch,
    canonical_project_draft_versions::Row as DraftVersion,
    canonical_project_drafts::Row as DraftHead,
    canonical_project_operations::Row as ProjectAcknowledgement,
    canonical_project_savepoints::Row as Savepoint, canonical_projects::Row as Project,
    canonical_research_references::Row as ResearchReference,
};
use surrealdb::types::{Object, Value};

/// One atomic draft-head and immutable-version selection.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[doc = "Selected mutable editing head."]
    pub head: DraftHead,
    #[doc = "Original immutable raw version."]
    pub version: DraftVersion,
}
impl std::ops::Deref for Draft {
    type Target = DraftHead;
    fn deref(&self) -> &DraftHead {
        &self.head
    }
}
#[cfg(feature = "canonical-tests")]
tokio::task_local! { pub(crate) static PROJECT_REPLY_LOSS: bool; }
#[cfg(feature = "canonical-tests")]
pub(crate) fn simulate_reply_loss() -> bool {
    PROJECT_REPLY_LOSS.try_with(|value| *value).unwrap_or(false)
}

/// Exact original protected fit evidence and explicit authored target mapping.
/// This binding stores identities and decisions, never a second copy of fitted values.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchPublicationBinding {
    /// Frozen original research selection.
    pub selection: String,
    /// Actual original run.
    pub run: String,
    /// Actual original attempt.
    pub attempt: String,
    /// Original qualified export set coordinate.
    pub result_set: String,
    /// Exact closed manifest digest selected by protected read.
    pub manifest_digest: String,
    /// Original run's immutable consumed source receipt, independently compared in activation.
    pub source_selection: Vec<u8>,
    /// Explicit cell/target identities and physical/bound premises.
    pub mapping: Vec<u8>,
}
/// Original source-publication intent. Retain it unchanged through uncertain completion.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectPublication {
    issuing_store: String,
    project: String,
    project_generation: u64,
    branch: String,
    branch_generation: u64,
    problem: String,
    expected: Option<String>,
    origin_revision: Option<String>,
    draft: Option<String>,
    draft_generation: Option<u64>,
    operation: String,
    kind: String,
    request: Vec<u8>,
    branch_name: String,
    extra_parents: Vec<String>,
    context_commitment: Option<String>,
    research: Option<ResearchPublicationBinding>,
}
impl ProjectPublication {
    /// Bind exact original scientific evidence into the ordinary source activation intent.
    pub fn with_research_publication(
        mut self,
        research: ResearchPublicationBinding,
    ) -> Result<Self, CanonicalError> {
        if self.kind != "publish"
            || self.draft.is_some()
            || research.mapping.len() > 131072
            || research.source_selection.len() > 131072
        {
            return Err(CanonicalError::PayloadLimit);
        }
        self.research = Some(research);
        self.refresh_request()?;
        Ok(self)
    }
    /// Original scientific binding, if this is a research publication.
    pub fn research(&self) -> Option<&ResearchPublicationBinding> {
        self.research.as_ref()
    }

    /// Bind the complete caller decision before effect issuance.
    pub fn with_context_commitment(mut self, commitment: &str) -> Result<Self, CanonicalError> {
        identity(commitment)?;
        self.context_commitment = Some(commitment.into());
        self.refresh_request()?;
        Ok(self)
    }
    /// Original caller decision digest, independent of the source content identity.
    pub fn context_commitment(&self) -> Option<&str> {
        self.context_commitment.as_deref()
    }
    /// Retain exact additional merge ancestry through the canonical owner.
    pub fn with_merge_ancestry(
        mut self,
        base: &str,
        proposal: &str,
    ) -> Result<Self, CanonicalError> {
        identity(base)?;
        identity(proposal)?;
        if self.kind != "publish" {
            return Err(CanonicalError::OperationReused);
        }
        self.extra_parents = vec![base.into(), proposal.into()];
        self.extra_parents.sort();
        self.extra_parents.dedup();
        self.refresh_request()?;
        Ok(self)
    }
    /// Fork presentation name; identity remains the caller-selected branch key.
    pub fn with_branch_name(mut self, name: &str) -> Result<Self, CanonicalError> {
        identity(name)?;
        if self.kind != "fork" {
            return Err(CanonicalError::OperationReused);
        }
        self.branch_name = name.into();
        self.refresh_request()?;
        Ok(self)
    }
    fn refresh_request(&mut self) -> Result<(), CanonicalError> {
        self.request = self.canonical_request()?;
        Ok(())
    }
    fn canonical_request(&self) -> Result<Vec<u8>, CanonicalError> {
        request(&(
            3u8,
            &self.issuing_store,
            &self.project,
            self.project_generation,
            &self.branch,
            self.branch_generation,
            &self.problem,
            &self.expected,
            &self.origin_revision,
            &self.draft,
            self.draft_generation,
            &self.operation,
            &self.kind,
            &self.branch_name,
            &self.extra_parents,
            (&self.context_commitment, &self.research),
        ))
    }

    /// Exact independently owned source problem.
    pub fn problem(&self) -> &str {
        &self.problem
    }
    /// Caller-chosen immutable operation, usable for restart settlement.
    pub fn operation(&self) -> &str {
        &self.operation
    }
    /// Saved draft identity and generation required by this original intent.
    pub fn draft(&self) -> Option<(&str, u64)> {
        self.draft.as_deref().zip(self.draft_generation)
    }
    /// Original exact parent; no later head is substituted.
    pub fn expected(&self) -> Option<&str> {
        self.expected.as_deref()
    }
    /// Validate all kind-specific coordinates against the immutable original request.
    /// Deserialization alone never makes an opaque retained receipt authoritative.
    pub fn validate(&self) -> Result<(), CanonicalError> {
        for key in [
            &self.issuing_store,
            &self.project,
            &self.branch,
            &self.problem,
            &self.operation,
        ] {
            identity(key)?;
        }
        identity(&self.branch_name)?;
        if self.extra_parents.len() > 2 {
            return Err(CanonicalError::PayloadLimit);
        }
        for parent in &self.extra_parents {
            identity(parent)?;
        }
        if let Some(commitment) = &self.context_commitment {
            identity(commitment)?;
        }
        match self.kind.as_str() {
            "fork"
                if self.expected.is_none()
                    && self.origin_revision.is_some()
                    && self.draft.is_none()
                    && self.draft_generation.is_none()
                    && self.branch_generation == 0
                    && self.extra_parents.is_empty() => {}
            "publish"
                if self.origin_revision.is_none()
                    && self.expected.is_some()
                    && self.draft.is_some() == self.draft_generation.is_some() => {}
            _ => return Err(CanonicalError::OperationReused),
        }
        let expected = self.canonical_request()?;
        if self.request != expected {
            return Err(CanonicalError::OperationReused);
        }
        Ok(())
    }
    pub(crate) fn request(&self) -> &[u8] {
        &self.request
    }
    pub(crate) fn encode(&self) -> Result<Value, CanonicalError> {
        self.validate()?;
        let mut row = Object::new();
        row.insert("issuing_store", self.issuing_store.clone());
        row.insert("project", self.project.clone());
        row.insert(
            "project_generation",
            codec::encode_uint(self.project_generation)?,
        );
        row.insert("branch", self.branch.clone());
        row.insert(
            "branch_generation",
            codec::encode_uint(self.branch_generation)?,
        );
        let research = if let Some(binding) = &self.research {
            let mut value = Object::new();
            for (key, text) in [
                ("selection", &binding.selection),
                ("run", &binding.run),
                ("attempt", &binding.attempt),
                ("result_set", &binding.result_set),
                ("manifest_digest", &binding.manifest_digest),
            ] {
                value.insert(key, text.clone());
            }
            value.insert(
                "source_selection",
                surrealdb::types::Bytes::from(binding.source_selection.clone()),
            );
            value.insert(
                "mapping",
                surrealdb::types::Bytes::from(binding.mapping.clone()),
            );
            Value::Object(value)
        } else {
            Value::None
        };
        row.insert("research", research);
        row.insert("kind", self.kind.clone());
        row.insert("branch_name", self.branch_name.clone());
        row.insert(
            "extra_parents",
            Value::Array(
                self.extra_parents
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
        row.insert(
            "request",
            surrealdb::types::Bytes::from(self.request.clone()),
        );
        for (key, value) in [
            ("expected", &self.expected),
            ("origin_revision", &self.origin_revision),
            ("draft", &self.draft),
        ] {
            row.insert(key, value.clone().map(Value::String).unwrap_or(Value::None));
        }
        row.insert(
            "draft_generation",
            self.draft_generation
                .map(codec::encode_uint)
                .transpose()?
                .unwrap_or(Value::None),
        );
        Ok(Value::Object(row))
    }
}
/// Bounded branch continuation tied to its issuing authority and project membership generation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectCursor {
    issuing_store: String,
    project: String,
    generation: u64,
    after: String,
}
/// Compact metadata selection. It carries no protected payload lifetime.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BranchPage {
    /// Exact selected project metadata.
    pub project: Project,
    /// Ordered branch headers, with heads obtainable only from their canonical source owner.
    pub branches: Vec<Branch>,
    /// More branch membership remains; it is fenced against generation/authority changes.
    pub next: Option<ProjectCursor>,
}
fn identity(value: &str) -> Result<(), CanonicalError> {
    if value.is_empty() || value.len() > 4096 {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}
fn request<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json::to_vec(value).map_err(|e| CanonicalError::Configuration(e.to_string()))
}
fn object(value: Value) -> Result<Object, codec::CodecError> {
    match value {
        Value::Object(row) => Ok(row),
        _ => Err(codec::CodecError::Domain("object")),
    }
}
fn final_object(response: &mut surrealdb::IndexedResults) -> Result<Object, CanonicalError> {
    let index = response
        .num_statements()
        .checked_sub(2)
        .ok_or(CanonicalError::IncompleteResponse)?;
    response
        .take::<Option<Object>>(index)?
        .ok_or(CanonicalError::IncompleteResponse)
}
fn obligation(artifact: Option<&str>, owner: &str) -> Result<Value, CanonicalError> {
    artifact
        .map(|key| {
            wire::encode_canonical_artifact_obligations(
                &crate::canonical_artifacts::artifact_obligation(key, "project", owner),
            )
            .map(Value::Object)
        })
        .transpose()
        .map(|v| v.unwrap_or(Value::None))
        .map_err(Into::into)
}
impl CanonicalStore {
    fn project_query(
        &self,
        sql: &str,
    ) -> Result<surrealdb::method::Query<'_, surrealdb::engine::remote::grpc::Client>, CanonicalError>
    {
        let guard = "fn::pse_project_v1::authority($pse_rpc_expires_at,$project_issuing_store);";
        let sql = if let Some(body) = sql.strip_prefix("BEGIN;") {
            format!("BEGIN; {guard} {body}")
        } else {
            format!("BEGIN; {guard} {sql} COMMIT;")
        };
        Ok(self
            .db
            .query(sql)
            .bind(("project_issuing_store", self.issuing_store()?.to_owned())))
    }
    /// Read immutable exact-operation attribution; absence is not a rollback proof.
    pub async fn project_acknowledgement(
        &self,
        operation: &str,
    ) -> Result<Option<ProjectAcknowledgement>, CanonicalError> {
        identity(operation)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async { let mut response=bounded_query(self.project_query("SELECT * FROM ONLY type::record('canonical_project_operations',$operation);")?.bind(("operation",operation.to_owned()))).await?; response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?.map(wire::decode_canonical_project_operations).transpose().map_err(Into::into) })).await
    }
    /// Explicit project creation with caller-issued immutable identity.
    pub async fn create_project(
        &self,
        operation: &str,
        project: &str,
        title: &str,
    ) -> Result<ProjectAcknowledgement, CanonicalError> {
        identity(operation)?;
        identity(project)?;
        if title.len() > 4096 {
            return Err(CanonicalError::PayloadLimit);
        }
        let request = request(&("create", self.issuing_store()?, project, title))?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {self.ensure_writes()?;
            let result=protected_query("canonical_projects::create",||Ok(self.project_query("BEGIN; RETURN fn::pse_project_v1::create($pse_rpc_expires_at,$operation,$request,$project,$title); COMMIT;")?.bind(("operation",operation.to_owned())).bind(("request",surrealdb::types::Bytes::from(request.clone()))).bind(("project",project.to_owned())).bind(("title",title.to_owned())))).await;
            self.settle_project_effect(operation,&request,result).await
        })).await
    }
    async fn settle_project_effect(
        &self,
        operation: &str,
        request: &[u8],
        result: Result<surrealdb::IndexedResults, CanonicalError>,
    ) -> Result<ProjectAcknowledgement, CanonicalError> {
        #[cfg(all(test, feature = "canonical-tests"))]
        let result = if result.is_ok() && simulate_reply_loss() {
            Err(CanonicalError::IncompleteResponse)
        } else {
            result
        };
        match result {
            Ok(mut response) => {
                let ack = wire::decode_canonical_project_operations(final_object(&mut response)?)?;
                if ack.request.as_slice() != request {
                    return Err(CanonicalError::OperationReused);
                }
                Ok(ack)
            }
            Err(source) => match self.project_acknowledgement(operation).await {
                Ok(Some(ack)) => {
                    if ack.request.as_slice() == request {
                        Ok(ack)
                    } else {
                        Err(CanonicalError::OperationReused)
                    }
                }
                Ok(None) if source.source_transaction_refused() => Err(source),
                _ => Err(CanonicalError::ProjectUnsettled {
                    operation: operation.into(),
                    source: Box::new(source),
                }),
            },
        }
    }
    /// Select one compact project header without admitting or opening source payloads.
    pub async fn project(&self, key: &str) -> Result<Option<Project>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.project_query(
                        "SELECT * FROM ONLY type::record('canonical_projects',$key) WHERE fn::pse_project_import_v1::available(key);",
                    )?
                    .bind(("key", key.to_owned())),
                )
                .await?;
                response
                    .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_projects)
                    .transpose()
                    .map_err(Into::into)
            }))
            .await
    }
    /// Bounded native branch query. Continuations never cross project generations or stores.
    pub async fn project_branches(
        &self,
        project: &str,
        cursor: Option<&ProjectCursor>,
    ) -> Result<BranchPage, CanonicalError> {
        identity(project)?;
        let issuing = self.issuing_store()?;
        if cursor.is_some_and(|c| c.issuing_store != issuing || c.project != project) {
            return Err(CanonicalError::Configuration(
                "project cursor issuing context differs".into(),
            ));
        }
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async{
            let mut response=protected_query("canonical_projects::branches",||Ok(self.project_query("BEGIN; LET $project=SELECT * FROM ONLY type::record('canonical_projects',$key); IF $project=NONE OR !fn::pse_project_import_v1::available($key) OR ($generation!=NONE AND $project.generation!=$generation) { THROW 'PSE_REFUSAL:project_generation_changed'; }; LET $rows=SELECT * FROM canonical_project_branches WITH INDEX project_branch_order WHERE project=$key AND key>$after ORDER BY key LIMIT 65; RETURN {project:$project,branches:$rows}; COMMIT;")?.bind(("key",project.to_owned())).bind(("generation",cursor.map(|c|codec::encode_uint(c.generation)).transpose()?.unwrap_or(Value::None))).bind(("after",cursor.map_or("",|c|c.after.as_str()).to_owned())))).await?;
            let mut row=final_object(&mut response)?;let project=wire::decode_canonical_projects(object(codec::required(&mut row,"project")?)?)?;
            let Value::Array(rows)=codec::required(&mut row,"branches")? else{return Err(CanonicalError::IncompleteResponse);};
            let mut branches=rows.into_iter().map(|v|wire::decode_canonical_project_branches(object(v)?)).collect::<Result<Vec<_>,_>>()?;
            let next=if branches.len()>64{branches.truncate(64);Some(ProjectCursor{issuing_store:self.issuing_store()?.into(),project:project.key.clone(),generation:project.generation,after:branches.last().ok_or(CanonicalError::IncompleteResponse)?.key.clone()})}else{None};
            Ok(BranchPage{project,branches,next})
        })).await
    }
    /// Read one branch; its source head is deliberately not duplicated in this row.
    pub async fn project_branch(&self, key: &str) -> Result<Option<Branch>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.project_query(
                        "SELECT * FROM ONLY type::record('canonical_project_branches',$key) WHERE fn::pse_project_import_v1::available(project);",
                    )?
                    .bind(("key", key.to_owned())),
                )
                .await?;
                response
                    .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_project_branches)
                    .transpose()
                    .map_err(Into::into)
            }))
            .await
    }
    /// Resolve the current head from its single source authority in one snapshot.
    pub async fn project_branch_revision(
        &self,
        key: &str,
    ) -> Result<Option<Revision>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async{
            let mut response=bounded_query(self.project_query("LET $branch=SELECT * FROM ONLY type::record('canonical_project_branches',$key) WHERE fn::pse_project_import_v1::available(project); LET $problem=IF $branch=NONE {NONE} ELSE {SELECT * FROM ONLY type::record('canonical_problems',$branch.problem)}; RETURN IF $problem=NONE {NONE} ELSE {SELECT * FROM ONLY type::record('canonical_revisions',$problem.head)};")?.bind(("key",key.to_owned()))).await?;
            response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?.map(wire::decode_canonical_revisions).transpose().map_err(Into::into)
        })).await
    }
    /// Capture an original independent-branch fork intent; source closure is checked atomically.
    pub fn project_fork_intent(
        &self,
        project: &Project,
        branch: &str,
        problem: &str,
        origin: &Revision,
        operation: &str,
    ) -> Result<ProjectPublication, CanonicalError> {
        for key in [&project.key, branch, problem, &origin.key, operation] {
            identity(key)?;
        }
        let request = request(&(
            "fork",
            self.issuing_store()?,
            &project.key,
            project.generation,
            branch,
            problem,
            &origin.key,
            operation,
        ))?;
        let mut intent = ProjectPublication {
            issuing_store: self.issuing_store()?.into(),
            project: project.key.clone(),
            project_generation: project.generation,
            branch: branch.into(),
            branch_generation: 0,
            problem: problem.into(),
            expected: None,
            origin_revision: Some(origin.key.clone()),
            draft: None,
            draft_generation: None,
            operation: operation.into(),
            kind: "fork".into(),
            request,
            branch_name: branch.into(),
            extra_parents: Vec::new(),
            context_commitment: None,
            research: None,
        };
        intent.refresh_request()?;
        Ok(intent)
    }
    /// Capture one exact branch publication, optionally requiring a saved draft generation.
    pub fn project_publication_intent(
        &self,
        project: &Project,
        branch: &Branch,
        expected: &str,
        draft: Option<&Draft>,
        operation: &str,
    ) -> Result<ProjectPublication, CanonicalError> {
        if branch.project != project.key
            || draft.is_some_and(|d| d.branch != branch.key || d.version.base_revision != expected)
        {
            return Err(CanonicalError::Configuration(
                "project publication context differs".into(),
            ));
        }
        identity(expected)?;
        identity(operation)?;
        let request = request(&(
            "publish",
            self.issuing_store()?,
            &project.key,
            project.generation,
            &branch.key,
            branch.generation,
            &branch.problem,
            expected,
            draft.map(|d| (&d.key, d.generation)),
            operation,
        ))?;
        let mut intent = ProjectPublication {
            issuing_store: self.issuing_store()?.into(),
            project: project.key.clone(),
            project_generation: project.generation,
            branch: branch.key.clone(),
            branch_generation: branch.generation,
            problem: branch.problem.clone(),
            expected: Some(expected.into()),
            origin_revision: None,
            draft: draft.map(|d| d.key.clone()),
            draft_generation: draft.map(|d| d.generation),
            operation: operation.into(),
            kind: "publish".into(),
            request,
            branch_name: branch.name.clone(),
            extra_parents: Vec::new(),
            context_commitment: None,
            research: None,
        };
        intent.refresh_request()?;
        Ok(intent)
    }
    pub(crate) async fn verify_project_publication_ack(
        &self,
        intent: &ProjectPublication,
    ) -> Result<(), CanonicalError> {
        intent.validate()?;
        let ack = self
            .project_acknowledgement(&intent.operation)
            .await?
            .ok_or(CanonicalError::OperationReused)?;
        if ack.request.as_slice() != intent.request
            || ack.revision.as_deref() != Some(intent.operation.as_str())
            || ack.project != intent.project
            || ack.branch.as_deref() != Some(intent.branch.as_str())
        {
            return Err(CanonicalError::OperationReused);
        }
        Ok(())
    }
    /// Observe this exact accepted publication before considering mutable current draft state.
    /// A missing acknowledgement is only an observation, never permission to replace the intent.
    pub async fn settled_project_publication(
        &self,
        intent: &ProjectPublication,
    ) -> Result<Option<Revision>, CanonicalError> {
        intent.validate()?;
        if intent.issuing_store != self.issuing_store()? {
            return Err(CanonicalError::OperationReused);
        }
        if self
            .project_acknowledgement(&intent.operation)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        self.verify_project_publication_ack(intent).await?;
        let revision = self
            .revision(&intent.operation)
            .await?
            .ok_or(CanonicalError::IncompleteResponse)?;
        if revision.problem != intent.problem
            || revision.parent != intent.expected
            || revision.operation != intent.operation
            || revision.key != intent.operation
            || revision.interpretation != wire::INTERPRETATION
        {
            return Err(CanonicalError::OperationReused);
        }
        Ok(Some(revision))
    }
    /// Publish through the existing source activation with project guards in that transaction.
    pub async fn edit_project(
        &self,
        intent: &ProjectPublication,
        edits: &[ObjectEdit],
    ) -> Result<Revision, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(
                original_deadline(crate::canonical::ACTIVATION_REQUEST_TIMEOUT),
                async {
                    intent.validate()?;
                    if intent.issuing_store != self.issuing_store()? {
                        return Err(CanonicalError::Configuration(
                            "project publication issuing authority differs".into(),
                        ));
                    }
                    if let Some(ack) = self.project_acknowledgement(&intent.operation).await? {
                        if ack.request.as_slice() != intent.request {
                            return Err(CanonicalError::OperationReused);
                        }
                        // The source owner's original request must also match, even after a lost response.
                        let stage = self
                            .stage_edits(
                                &intent.problem,
                                intent.expected.as_deref(),
                                &intent.operation,
                                edits,
                            )
                            .await?;
                        return match stage {
                            crate::canonical_staging::StageOutcome::Acknowledged(revision) => {
                                Ok(revision)
                            }
                            _ => Err(CanonicalError::IncompleteResponse),
                        };
                    }
                    self.edit_project_authorized(intent, edits).await
                },
            ))
            .await
    }
    /// Save original malformed/empty raw bytes under exact generation; this never admits science.
    pub async fn save_project_draft(
        &self,
        operation: &str,
        project: &Project,
        branch: &Branch,
        key: &str,
        base: &str,
        expected: Option<u64>,
        content: &[u8],
        artifact: Option<&str>,
    ) -> Result<ProjectAcknowledgement, CanonicalError> {
        for id in [operation, &project.key, &branch.key, key, base] {
            identity(id)?;
        }
        if content.len() > 128 * 1024
            || branch.project != project.key
            || (artifact.is_some() && !content.is_empty())
        {
            return Err(CanonicalError::PayloadLimit);
        }
        let row = DraftVersion {
            retired: false,
            key: operation.into(),
            draft: key.into(),
            project: project.key.clone(),
            branch: branch.key.clone(),
            base_revision: base.into(),
            generation: expected
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(CanonicalError::PayloadLimit)?,
            content: content.to_vec().into(),
            artifact: artifact.map(str::to_owned),
        };
        let request = request(&(
            "draft",
            self.issuing_store()?,
            operation,
            &row.draft,
            &row.project,
            &row.branch,
            &row.base_revision,
            project.generation,
            expected,
            crate::canonical_execution::result_payload_digest(content),
            artifact,
        ))?;
        let encoded = wire::encode_canonical_project_draft_versions(&row)?;
        let obligation = obligation(artifact, key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async{self.ensure_writes()?;let result=protected_query("canonical_projects::draft",||Ok(self.project_query("BEGIN; RETURN fn::pse_project_v1::draft($pse_rpc_expires_at,$operation,$request,$row,$expected,$project_generation,$obligation); COMMIT;")?.bind(("operation",operation.to_owned())).bind(("request",surrealdb::types::Bytes::from(request.clone()))).bind(("row",encoded.clone())).bind(("expected",expected.map(codec::encode_uint).transpose()?.unwrap_or(Value::None))).bind(("project_generation",codec::encode_uint(project.generation)?)).bind(("obligation",obligation.clone())))).await;self.settle_project_effect(operation,&request,result).await})).await
    }
    /// Read the head and exact immutable version in one short snapshot.
    pub async fn project_draft(&self, key: &str) -> Result<Option<Draft>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            let mut response=bounded_query(self.project_query("LET $head=SELECT * FROM ONLY type::record('canonical_project_drafts',$key) WHERE fn::pse_project_import_v1::available(project); RETURN IF $head=NONE {NONE} ELSE {{head:$head,version:(SELECT * FROM ONLY type::record('canonical_project_draft_versions',$head.version) WHERE project=$head.project AND fn::pse_project_import_v1::available(project))}};")?.bind(("key",key.to_owned()))).await?;
            let value=response.take::<Option<Object>>(response.num_statements().saturating_sub(2))?;
            value.map(|mut row|Ok(Draft{head:wire::decode_canonical_project_drafts(object(codec::required(&mut row,"head")?)?)?,version:wire::decode_canonical_project_draft_versions(object(codec::required(&mut row,"version")?)?)?})).transpose()
        })).await
    }
    /// Read one exact original draft version, independent of later head movement.
    pub async fn project_draft_version(
        &self,
        key: &str,
    ) -> Result<Option<DraftVersion>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.project_query(
                        "SELECT * FROM ONLY type::record('canonical_project_draft_versions',$key) WHERE fn::pse_project_import_v1::available(project);",
                    )?
                    .bind(("key", key.to_owned())),
                )
                .await?;
                response
                    .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_project_draft_versions)
                    .transpose()
                    .map_err(Into::into)
            }))
            .await
    }
    /// Open one immutable bookmark, without hydrating scientific payloads.
    pub async fn project_bookmark(&self, key: &str) -> Result<Option<Bookmark>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.project_query(
                        "SELECT * FROM ONLY type::record('canonical_project_bookmarks',$key) WHERE fn::pse_project_import_v1::available(project);",
                    )?
                    .bind(("key", key.to_owned())),
                )
                .await?;
                response
                    .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_project_bookmarks)
                    .transpose()
                    .map_err(Into::into)
            }))
            .await
    }
    /// Bounded exact ancestry: canonical parent, branch origin and additional accepted merge parents.
    pub async fn project_revision_parents(
        &self,
        revision: &Revision,
    ) -> Result<Vec<String>, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            let mut response=bounded_query(self.project_query("LET $actual=SELECT * FROM ONLY type::record('canonical_revisions',$key); IF $actual=NONE OR $actual.problem!=$problem {THROW 'PSE_REFUSAL:project_source_unavailable';}; LET $branch=SELECT * FROM canonical_project_branches WITH INDEX project_branch_problem WHERE problem=$problem LIMIT 2; FOR $owner IN $branch {IF !fn::pse_project_import_v1::available($owner.project) {THROW 'PSE_REFUSAL:project_source_unavailable';};}; LET $extra=SELECT VALUE parent FROM canonical_project_ancestry WITH INDEX project_ancestry WHERE revision=$key LIMIT 3; RETURN {parent:$actual.parent,origin:IF $actual.parent=NONE AND array::len($branch)=1 {$branch[0].origin_revision} ELSE {NONE},extra:$extra};")?.bind(("key",revision.key.clone())).bind(("problem",revision.problem.clone()))).await?;
            let mut row=final_object(&mut response)?;let mut result=Vec::new();for field in ["parent","origin"]{if let Some(value)=row.remove(field){if value!=Value::None && value!=Value::Null {result.push(codec::decode_string(value)?);}}}
            let extra=codec::required(&mut row,"extra")?;if let Value::Array(items)=extra {for item in items{if let Value::String(v)=item {result.push(v);}}}result.sort();result.dedup();Ok(result)
        })).await
    }
    /// Retain exact original source/result lineage without copying scientific values.
    pub async fn retain_research_reference(
        &self,
        operation: &str,
        project: &Project,
        key: &str,
        revision: &str,
        result: Option<(&str, &str)>,
        artifact: Option<&str>,
    ) -> Result<ProjectAcknowledgement, CanonicalError> {
        for id in [operation, &project.key, key, revision] {
            identity(id)?;
        }
        let request = request(&(
            "research",
            self.issuing_store()?,
            operation,
            &project.key,
            project.generation,
            key,
            revision,
            result,
            artifact,
        ))?;
        let row = ResearchReference {
            key: key.into(),
            project: project.key.clone(),
            revision: revision.into(),
            run: result.map(|r| r.0.into()),
            attempt: result.map(|r| r.1.into()),
            artifact: artifact.map(str::to_owned),
            request: request.clone().into(),
        };
        let row = wire::encode_canonical_research_references(&row)?;
        let obligation = obligation(artifact, key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async{self.ensure_writes()?;let result=protected_query("canonical_projects::research",||Ok(self.project_query("BEGIN; RETURN fn::pse_project_v1::research($pse_rpc_expires_at,$operation,$request,$row,$project_generation,$obligation); COMMIT;")?.bind(("operation",operation.to_owned())).bind(("request",surrealdb::types::Bytes::from(request.clone()))).bind(("row",row.clone())).bind(("project_generation",codec::encode_uint(project.generation)?)).bind(("obligation",obligation.clone())))).await;self.settle_project_effect(operation,&request,result).await})).await
    }
}

/// Presentation and draft visibility commands, bound to an exact project generation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectNavigation {
    /// Rename a project label.
    ProjectTitle {
        #[doc = "Presentation name within its owner namespace."]
        name: String,
    },
    /// Soft deletion never releases scientific closure.
    ProjectVisibility {
        #[doc = "Visibility flag; closure custody is unchanged."]
        deleted: bool,
    },
    /// Rename a branch within its project namespace.
    BranchName {
        #[doc = "Stable record identity."]
        key: String,
        #[doc = "Presentation name within its owner namespace."]
        name: String,
    },
    /// Retain an exact admitted revision under a name.
    Bookmark {
        #[doc = "Stable record identity."]
        key: String,
        #[doc = "Presentation name within its owner namespace."]
        name: String,
        #[doc = "Exact admitted source revision."]
        revision: String,
    },
    /// Pin the selected immutable raw version.
    Savepoint {
        #[doc = "Stable record identity."]
        key: String,
        #[doc = "Presentation name within its owner namespace."]
        name: String,
        #[doc = "Stable draft identity."]
        draft: String,
        #[doc = "Exact immutable raw version."]
        version: String,
        #[doc = "Expected mutable head generation."]
        generation: u64,
    },
    /// Change draft visibility under its expected head generation.
    DraftVisibility {
        #[doc = "Stable record identity."]
        key: String,
        #[doc = "Expected mutable head generation."]
        generation: u64,
        #[doc = "Visibility flag; closure custody is unchanged."]
        deleted: bool,
    },
    /// Change savepoint visibility.
    SavepointVisibility {
        #[doc = "Stable record identity."]
        key: String,
        #[doc = "Visibility flag; closure custody is unchanged."]
        deleted: bool,
    },
    /// Restore an exact savepoint as a new head generation.
    Restore {
        #[doc = "Stable record identity."]
        key: String,
        #[doc = "Stable draft identity."]
        draft: String,
        #[doc = "Expected mutable head generation."]
        generation: u64,
    },
}
/// Versioned public command transport; the action carries no scientific authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectNavigationRequest {
    /// Command codec version.
    pub version: pse_model::document::Version<1>,
    /// Explicit presentation or visibility change.
    pub action: ProjectNavigation,
}
/// Finite metadata page selector; cursor is scoped to the issuing store and owner.
pub use pse_model::generated::enums::ProjectMetadataKind;
/// Continuation of a live metadata listing, not a historical snapshot.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMetadataCursor {
    issuing_store: String,
    kind: ProjectMetadataKind,
    owner: String,
    after: String,
}
/// Exact owner and optional continuation for a bounded live metadata listing.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMetadataRequest {
    /// Header collection to inspect.
    pub kind: ProjectMetadataKind,
    /// Project, branch, draft or version according to kind; empty for all projects.
    pub owner: String,
    /// Original continuation from the same store and collection.
    pub cursor: Option<ProjectMetadataCursor>,
}
/// Bounded metadata only; selected raw bodies are fetched separately.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectMetadataPage {
    #[doc = "Project metadata."]
    pub projects: Vec<Project>,
    #[doc = "Draft head metadata without raw bodies."]
    pub drafts: Vec<DraftHead>,
    #[doc = "Exact source bookmark metadata."]
    pub bookmarks: Vec<Bookmark>,
    #[doc = "Exact saved draft-version references."]
    pub savepoints: Vec<Savepoint>,
    #[doc = "Version-linked admission outcomes."]
    pub diagnostics: Vec<DraftDiagnostic>,
    #[doc = "Continuation of this live metadata listing."]
    pub next: Option<ProjectMetadataCursor>,
}
impl CanonicalStore {
    /// Execute one original presentation/editing command; settle its exact receipt on retry.
    pub async fn project_navigate(
        &self,
        operation: &str,
        project: &Project,
        action: &ProjectNavigation,
    ) -> Result<ProjectAcknowledgement, CanonicalError> {
        identity(operation)?;
        identity(&project.key)?;
        // Use the same typed codec as generated native integers, never lossy JSON numbers.
        let mut encoded = Object::new();
        let (kind, name) = match action {
            ProjectNavigation::ProjectTitle { name } => ("project_title", Some(name)),
            ProjectNavigation::ProjectVisibility { deleted } => {
                encoded.insert("deleted", *deleted);
                ("project_visibility", None)
            }
            ProjectNavigation::BranchName { key, name } => {
                identity(key)?;
                encoded.insert("key", key.clone());
                ("branch_name", Some(name))
            }
            ProjectNavigation::Bookmark {
                key,
                name,
                revision,
            } => {
                identity(key)?;
                identity(revision)?;
                encoded.insert("key", key.clone());
                encoded.insert("revision", revision.clone());
                ("bookmark", Some(name))
            }
            ProjectNavigation::Savepoint {
                key,
                name,
                draft,
                version,
                generation,
            } => {
                for id in [key, draft, version] {
                    identity(id)?;
                }
                encoded.insert("key", key.clone());
                encoded.insert("draft", draft.clone());
                encoded.insert("version", version.clone());
                encoded.insert("generation", codec::encode_uint(*generation)?);
                ("savepoint", Some(name))
            }
            ProjectNavigation::DraftVisibility {
                key,
                generation,
                deleted,
            } => {
                identity(key)?;
                encoded.insert("key", key.clone());
                encoded.insert("generation", codec::encode_uint(*generation)?);
                encoded.insert("deleted", *deleted);
                ("draft_visibility", None)
            }
            ProjectNavigation::SavepointVisibility { key, deleted } => {
                identity(key)?;
                encoded.insert("key", key.clone());
                encoded.insert("deleted", *deleted);
                ("savepoint_visibility", None)
            }
            ProjectNavigation::Restore {
                key,
                draft,
                generation,
            } => {
                identity(key)?;
                identity(draft)?;
                encoded.insert("key", key.clone());
                encoded.insert("draft", draft.clone());
                encoded.insert("generation", codec::encode_uint(*generation)?);
                ("restore", None)
            }
        };
        encoded.insert("kind", kind);
        if let Some(name) = name {
            identity(name)?;
            encoded.insert("name", name.clone());
        }
        let request = request(&(
            "navigation-v1",
            self.issuing_store()?,
            operation,
            &project.key,
            project.generation,
            action,
        ))?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            self.ensure_writes()?;let result=protected_query("canonical_projects::navigate",||Ok(self.project_query("RETURN fn::pse_project_v1::navigate($pse_rpc_expires_at,$operation,$request,$project,$generation,$action);")?.bind(("operation",operation.to_owned())).bind(("request",surrealdb::types::Bytes::from(request.clone()))).bind(("project",project.key.clone())).bind(("generation",codec::encode_uint(project.generation)?)).bind(("action",encoded.clone())))).await;
            self.settle_project_effect(operation,&request,result).await
        })).await
    }
    /// Inspect bounded indexed headers. Deleted entries remain explicit and restorable.
    pub async fn project_metadata(
        &self,
        kind: ProjectMetadataKind,
        owner: &str,
        cursor: Option<&ProjectMetadataCursor>,
    ) -> Result<ProjectMetadataPage, CanonicalError> {
        if kind != ProjectMetadataKind::Projects {
            identity(owner)?;
        }
        if cursor.is_some_and(|c| {
            c.issuing_store != self.issuing_store().unwrap_or("")
                || c.kind != kind
                || c.owner != owner
        }) {
            return Err(CanonicalError::OperationReused);
        }
        let (table, index, field) = match kind {
            ProjectMetadataKind::Projects => ("canonical_projects", "", ""),
            ProjectMetadataKind::Drafts => {
                ("canonical_project_drafts", "project_draft_order", "branch")
            }
            ProjectMetadataKind::Bookmarks => (
                "canonical_project_bookmarks",
                "project_bookmark_order",
                "project",
            ),
            ProjectMetadataKind::Savepoints => (
                "canonical_project_savepoints",
                "project_savepoint_order",
                "draft",
            ),
            ProjectMetadataKind::Diagnostics => (
                "canonical_draft_diagnostics",
                "draft_diagnostic_order",
                "version",
            ),
        };
        let clause = if field.is_empty() {
            "".into()
        } else {
            format!("{field}=$owner AND ")
        };
        let index = if index.is_empty() {
            String::new()
        } else {
            format!(" WITH INDEX {index}")
        };
        // Scope checks and row filtering share this same snapshot. A forged
        // branch/draft/version coordinate cannot open a pending imported project.
        let (guard, availability) = match kind {
            ProjectMetadataKind::Projects => (String::new(), "fn::pse_project_import_v1::available(key)"),
            ProjectMetadataKind::Bookmarks => (
                "IF !fn::pse_project_import_v1::available($owner) {THROW 'PSE_REFUSAL:project_source_unavailable';};".into(),
                "fn::pse_project_import_v1::available(project)",
            ),
            ProjectMetadataKind::Drafts | ProjectMetadataKind::Savepoints | ProjectMetadataKind::Diagnostics => {
                let owner_table=match kind {
                    ProjectMetadataKind::Drafts => "canonical_project_branches",
                    ProjectMetadataKind::Savepoints => "canonical_project_drafts",
                    _ => "canonical_project_draft_versions",
                };
                (format!("LET $scope=SELECT project FROM ONLY type::record('{owner_table}',$owner); IF $scope=NONE OR !fn::pse_project_import_v1::available($scope.project) {{THROW 'PSE_REFUSAL:project_source_unavailable';}};"),
                 if kind==ProjectMetadataKind::Diagnostics {"fn::pse_project_import_v1::available(type::record('canonical_project_draft_versions',version).project)"} else {"fn::pse_project_import_v1::available(project)"})
            },
        };
        let sql = format!("{guard} SELECT * FROM {table}{index} WHERE {clause}{availability} AND key>$after ORDER BY key LIMIT 65;");
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.project_query(&sql)?
                        .bind(("owner", owner.to_owned()))
                        .bind(("after", cursor.map_or("", |c| c.after.as_str()).to_owned())),
                )
                .await?;
                let mut rows =
                    response.take::<Vec<Object>>(response.num_statements().saturating_sub(2))?;
                let more = rows.len() > 64;
                rows.truncate(64);
                let after = rows
                    .last()
                    .and_then(|r| r.get("key"))
                    .cloned()
                    .map(codec::decode_string)
                    .transpose()?;
                let mut result = ProjectMetadataPage {
                    projects: Vec::new(),
                    drafts: Vec::new(),
                    bookmarks: Vec::new(),
                    savepoints: Vec::new(),
                    diagnostics: Vec::new(),
                    next: if more {
                        Some(ProjectMetadataCursor {
                            issuing_store: self.issuing_store()?.into(),
                            kind,
                            owner: owner.into(),
                            after: after.ok_or(CanonicalError::IncompleteResponse)?,
                        })
                    } else {
                        None
                    },
                };
                for row in rows {
                    match kind {
                        ProjectMetadataKind::Projects => {
                            result.projects.push(wire::decode_canonical_projects(row)?)
                        }
                        ProjectMetadataKind::Drafts => result
                            .drafts
                            .push(wire::decode_canonical_project_drafts(row)?),
                        ProjectMetadataKind::Bookmarks => result
                            .bookmarks
                            .push(wire::decode_canonical_project_bookmarks(row)?),
                        ProjectMetadataKind::Savepoints => result
                            .savepoints
                            .push(wire::decode_canonical_project_savepoints(row)?),
                        ProjectMetadataKind::Diagnostics => result
                            .diagnostics
                            .push(wire::decode_canonical_draft_diagnostics(row)?),
                    }
                }
                Ok(result)
            }))
            .await
    }
    /// Retain the outcome of explicit admission against the immutable raw version.
    pub async fn project_record_diagnostic(
        &self,
        operation: &str,
        version: &str,
        message: &str,
        admitted: bool,
    ) -> Result<DraftDiagnostic, CanonicalError> {
        identity(operation)?;
        identity(version)?;
        if message.len() > 65536 {
            return Err(CanonicalError::PayloadLimit);
        }
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async{
            self.ensure_writes()?;let mut response=protected_query("canonical_projects::diagnostic",||Ok(self.project_query("RETURN fn::pse_project_v1::diagnostic($pse_rpc_expires_at,$operation,$version,$message,$admitted);")?.bind(("operation",operation.to_owned())).bind(("version",version.to_owned())).bind(("message",message.to_owned())).bind(("admitted",admitted)))).await?;
            wire::decode_canonical_draft_diagnostics(final_object(&mut response)?).map_err(Into::into)
        })).await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "isolated native project fixtures assert exact owned effects"
    )]
    use super::*;
    #[tokio::test]
    async fn pending_import_is_unavailable_through_public_project_research_and_context_reads() {
        let store=fixture().await;
        let origin=store.edit("pending-origin",None,"pending-origin-revision",&[]).await.unwrap();
        store.create_project("pending-create","pending-project","Pending fixture").await.unwrap();
        let project=store.project("pending-project").await.unwrap().unwrap();
        let intent=store.project_fork_intent(&project,"pending-branch","pending-problem",&origin,"pending-fork").unwrap();
        let head=store.edit_project(&intent,&[]).await.unwrap();
        let project=store.project("pending-project").await.unwrap().unwrap();
        let branch=store.project_branch("pending-branch").await.unwrap().unwrap();
        store.save_project_draft("pending-version",&project,&branch,"pending-draft",&head.key,None,b"raw draft",None).await.unwrap();
        store.project_navigate("pending-bookmark-op",&project,&ProjectNavigation::Bookmark{key:"pending-bookmark".into(),name:"bookmark".into(),revision:head.key.clone()}).await.unwrap();
        let project=store.project("pending-project").await.unwrap().unwrap();
        store.project_navigate("pending-savepoint-op",&project,&ProjectNavigation::Savepoint{key:"pending-savepoint".into(),name:"saved".into(),draft:"pending-draft".into(),version:"pending-version".into(),generation:1}).await.unwrap();
        store.project_record_diagnostic("pending-diagnostic","pending-version","fixture diagnostic",false).await.unwrap();
        // These deliberately synthetic metadata rows test disclosure boundaries,
        // not scientific admission or validity. Every row belongs to this fixture.
        bounded_query(store.db.query("BEGIN; CREATE type::record('canonical_context_journals','pending-project') SET key='pending-project',sequence=1dec; CREATE canonical_project_contexts:pending_context SET key='pending_context',project='pending-project',scope='fixture',revision=$revision,source_identity='fixture',physical_context='fixture',physical_revision=$revision,physical_source_identity='fixture',effective_from_us=0,effective_to_us=NONE,recorded_sequence=1dec,corrects=NONE,deleted=false,request=$bytes; CREATE canonical_research_datasets:pending_dataset SET key='pending_dataset',project='pending-project',revision=$revision,physical_context='fixture',context='pending_context',parent=NONE,role='raw',members=$bytes,request=$bytes; CREATE canonical_research_selections:pending_selection SET key='pending_selection',project='pending-project',dataset='pending_dataset',revision=$revision,physical_context='fixture',context='pending_context',fit='fixture',selection=$bytes,request=$bytes; CREATE canonical_research_publications:pending_publication SET key='pending_publication',project='pending-project',selection='pending_selection',run='fixture',attempt='fixture',revision=$revision,parent=$revision,mapping=$bytes,request=$bytes; COMMIT;")
            .bind(("revision",head.key.clone())).bind(("bytes",surrealdb::types::Bytes::from(vec![1_u8])))).await.unwrap();
        let cut=store.project_context_cut("pending-project").await.unwrap();
        assert!(store.research_dataset("pending_dataset").await.unwrap().is_some());
        assert!(store.project_context_record("pending_context").await.unwrap().is_some());
        // Install an original incomplete-import checkpoint only after creating
        // adversarial pre-existing references, so forged known coordinates are covered.
        bounded_query(store.db.query("CREATE canonical_project_imports:pending_import SET key='pending_import',issuing_store=$issuer,bundle_digest='fixture',project='pending-project',branch='pending-branch',problem='pending-problem',revision=$revision,origin=$bytes,request=$bytes,artifact=NONE,complete=false;")
            .bind(("issuer",store.issuing_store().unwrap().to_owned())).bind(("revision",head.key.clone())).bind(("bytes",surrealdb::types::Bytes::from(vec![1_u8])))).await.unwrap();
        assert!(store.project("pending-project").await.unwrap().is_none());
        assert!(store.project_branches("pending-project",None).await.is_err());
        assert!(store.project_branch("pending-branch").await.unwrap().is_none());
        assert!(store.project_branch_revision("pending-branch").await.unwrap().is_none());
        assert!(store.project_bookmark("pending-bookmark").await.unwrap().is_none());
        assert!(store.project_draft("pending-draft").await.unwrap().is_none());
        assert!(store.project_draft_version("pending-version").await.unwrap().is_none());
        assert!(store.project_revision_parents(&head).await.is_err());
        let projects=store.project_metadata(ProjectMetadataKind::Projects,"",None).await.unwrap();
        assert!(projects.projects.iter().all(|p|p.key!="pending-project"));
        for (kind,owner) in [(ProjectMetadataKind::Drafts,"pending-branch"),(ProjectMetadataKind::Bookmarks,"pending-project"),(ProjectMetadataKind::Savepoints,"pending-draft"),(ProjectMetadataKind::Diagnostics,"pending-version")] {
            assert!(store.project_metadata(kind,owner,None).await.is_err());
        }
        assert!(store.research_dataset("pending_dataset").await.unwrap().is_none());
        assert!(store.research_selection("pending_selection").await.unwrap().is_none());
        assert!(store.research_publication("pending_publication").await.unwrap().is_none());
        assert!(store.research_datasets("pending-project","",64).await.is_err());
        assert!(store.project_context_cut("pending-project").await.is_err());
        assert!(store.project_context_record("pending_context").await.unwrap().is_none());
        assert!(store.select_project_context(&cut,"fixture",0).await.is_err());
        assert!(store.project_acknowledgement("pending-fork").await.unwrap().is_some(),"original operation settlement remains observable");
        assert!(store.revision(&head.key).await.unwrap().is_some(),"original source primitives remain valid for artifact registration");
        bounded_query(store.db.query("UPDATE canonical_project_imports:pending_import SET complete=true;")).await.unwrap();
        assert!(store.project("pending-project").await.unwrap().is_some());
        assert!(store.project_branch("pending-branch").await.unwrap().is_some());
        assert!(store.project_draft_version("pending-version").await.unwrap().is_some());
        assert!(store.research_dataset("pending_dataset").await.unwrap().is_some());
        assert!(store.project_context_record("pending_context").await.unwrap().is_some());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_navigation_versions_savepoints_and_visibility_are_exact() {
        let store = fixture().await;
        let source = store.edit("origin", None, "origin-rev", &[]).await.unwrap();
        store
            .create_project("create-nav", "project-nav", "Before")
            .await
            .unwrap();
        let project = store.project("project-nav").await.unwrap().unwrap();
        let fork = store
            .project_fork_intent(&project, "branch-id", "branch-owner", &source, "fork-nav")
            .unwrap()
            .with_branch_name("main")
            .unwrap();
        let head = store.edit_project(&fork, &[]).await.unwrap();
        let project = store.project("project-nav").await.unwrap().unwrap();
        let branch = store.project_branch("branch-id").await.unwrap().unwrap();
        assert_eq!(branch.name, "main");
        store
            .save_project_draft(
                "save-one",
                &project,
                &branch,
                "draft-nav",
                &head.key,
                None,
                b"[[malformed",
                None,
            )
            .await
            .unwrap();
        let first = store.project_draft("draft-nav").await.unwrap().unwrap();
        assert_eq!(first.version.key, "save-one");
        store
            .project_navigate(
                "point-one",
                &project,
                &ProjectNavigation::Savepoint {
                    key: "point-id".into(),
                    name: "first".into(),
                    draft: first.key.clone(),
                    version: first.version.key.clone(),
                    generation: first.generation,
                },
            )
            .await
            .unwrap();
        let project = store.project("project-nav").await.unwrap().unwrap();
        store
            .save_project_draft(
                "save-two",
                &project,
                &branch,
                "draft-nav",
                &head.key,
                Some(1),
                b"new malformed",
                None,
            )
            .await
            .unwrap();
        assert!(
            store
                .save_project_draft(
                    "save-race",
                    &project,
                    &branch,
                    "draft-nav",
                    &head.key,
                    Some(1),
                    b"loser",
                    None
                )
                .await
                .is_err()
        );
        assert_eq!(
            store
                .project_draft_version("save-one")
                .await
                .unwrap()
                .unwrap(),
            first.version
        );
        store
            .project_navigate(
                "restore-one",
                &project,
                &ProjectNavigation::Restore {
                    key: "point-id".into(),
                    draft: "draft-nav".into(),
                    generation: 2,
                },
            )
            .await
            .unwrap();
        let restored = store.project_draft("draft-nav").await.unwrap().unwrap();
        assert_eq!(restored.generation, 3);
        assert_eq!(restored.version.key, "save-one");
        let project = store.project("project-nav").await.unwrap().unwrap();
        store
            .project_navigate(
                "bookmark-one",
                &project,
                &ProjectNavigation::Bookmark {
                    key: "bookmark-id".into(),
                    name: "baseline".into(),
                    revision: head.key.clone(),
                },
            )
            .await
            .unwrap();
        let project = store.project("project-nav").await.unwrap().unwrap();
        store
            .project_navigate(
                "rename-branch",
                &project,
                &ProjectNavigation::BranchName {
                    key: "branch-id".into(),
                    name: "renamed".into(),
                },
            )
            .await
            .unwrap();
        let page = store
            .project_metadata(ProjectMetadataKind::Bookmarks, "project-nav", None)
            .await
            .unwrap();
        assert_eq!(page.bookmarks[0].revision, head.key);
        let project = store.project("project-nav").await.unwrap().unwrap();
        let delete = ProjectNavigation::ProjectVisibility { deleted: true };
        store
            .project_navigate("delete-project", &project, &delete)
            .await
            .unwrap();
        assert!(store.project("project-nav").await.unwrap().unwrap().deleted);
        assert!(
            store
                .save_project_draft(
                    "deleted-save",
                    &project,
                    &branch,
                    "draft-nav",
                    &head.key,
                    Some(3),
                    b"no",
                    None
                )
                .await
                .is_err()
        );
        // Lost-response settlement uses original coordinates even after later membership changes.
        assert_eq!(
            store
                .project_navigate("delete-project", &project, &delete)
                .await
                .unwrap()
                .key,
            "delete-project"
        );
        let project = store.project("project-nav").await.unwrap().unwrap();
        store
            .project_navigate(
                "restore-project",
                &project,
                &ProjectNavigation::ProjectVisibility { deleted: false },
            )
            .await
            .unwrap();
        assert!(!store.project("project-nav").await.unwrap().unwrap().deleted);
        assert_eq!(
            store
                .project_metadata(ProjectMetadataKind::Drafts, "branch-id", None)
                .await
                .unwrap()
                .drafts[0]
                .version,
            "save-one"
        );
        store.disconnect().await.unwrap();
    }
    async fn fixture() -> CanonicalStore {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options =
            crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_projects_{}", uuid::Uuid::new_v4().simple());
        crate::testing::canonical_fixture_with_options(&options, true).unwrap()
    }
    fn edit(version: &str) -> ObjectEdit {
        ObjectEdit {
            logical: "object".into(),
            scope: "root".into(),
            name: "value".into(),
            version: Some(crate::canonical::ObjectVersion {
                key: version.into(),
                logical: "object".into(),
                kind: "test".into(),
                payload: version.as_bytes().to_vec().into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
            references: vec![],
        }
    }
    async fn fork(
        store: &CanonicalStore,
        branch: &str,
        origin: &Revision,
        edits: &[ObjectEdit],
    ) -> Revision {
        let project = store.project("project").await.unwrap().unwrap();
        let intent = store
            .project_fork_intent(
                &project,
                branch,
                &format!("problem-{branch}"),
                origin,
                &format!("fork-{branch}"),
            )
            .unwrap();
        store.edit_project(&intent, edits).await.unwrap()
    }
    #[tokio::test]
    async fn project_branches_are_independent_and_exact_operations_settle() {
        let store = fixture().await;
        let edits = vec![edit("v1")];
        let origin = store.edit("origin", None, "source", &edits).await.unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let a = fork(&store, "a", &origin, &edits).await;
        let b = fork(&store, "b", &origin, &edits).await;
        let project = store.project("project").await.unwrap().unwrap();
        let ba = store.project_branch("a").await.unwrap().unwrap();
        let bb = store.project_branch("b").await.unwrap().unwrap();
        let ia = store
            .project_publication_intent(&project, &ba, &a.key, None, "publish-a")
            .unwrap();
        let ib = store
            .project_publication_intent(&project, &bb, &b.key, None, "publish-b")
            .unwrap();
        let ea = vec![edit("va")];
        let eb = vec![edit("vb")];
        let (a, b) = tokio::join!(store.edit_project(&ia, &ea), store.edit_project(&ib, &eb));
        let a = a.unwrap();
        let b = b.unwrap();
        assert_ne!(a.problem, b.problem);
        assert_eq!(store.edit_project(&ia, &ea).await.unwrap(), a);
        assert!(matches!(
            store.edit_project(&ia, &eb).await,
            Err(CanonicalError::OperationReused)
        ));
        assert_eq!(
            store.project_branch_revision("a").await.unwrap().unwrap(),
            a
        );
        assert_eq!(
            store.project_branch_revision("b").await.unwrap().unwrap(),
            b
        );
        assert!(
            store
                .edit(&a.problem, Some(&a.key), "unguarded", &[edit("bad")])
                .await
                .is_err()
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_draft_custody_accepts_empty_and_malformed_but_fences_stale_publish() {
        let store = fixture().await;
        let edits = vec![edit("v1")];
        let origin = store.edit("origin", None, "source", &edits).await.unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let revision = fork(&store, "a", &origin, &edits).await;
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch("a").await.unwrap().unwrap();
        store
            .save_project_draft(
                "save-empty",
                &project,
                &branch,
                "draft",
                &revision.key,
                None,
                &[],
                None,
            )
            .await
            .unwrap();
        let draft = store.project_draft("draft").await.unwrap().unwrap();
        assert!(draft.version.content.is_empty());
        let publication = store
            .project_publication_intent(&project, &branch, &revision.key, Some(&draft), "publish")
            .unwrap();
        store
            .save_project_draft(
                "save-other",
                &project,
                &branch,
                "other-draft",
                &revision.key,
                None,
                b"other",
                None,
            )
            .await
            .unwrap();
        for (field, value) in [
            ("draft", "other-draft"),
            ("project", "other-project"),
            ("expected", "other-base"),
            ("kind", "fork"),
        ] {
            let mut document = serde_json::to_value(&publication).unwrap();
            document[field] = serde_json::Value::String(value.into());
            let forged: ProjectPublication = serde_json::from_value(document).unwrap();
            assert!(matches!(
                store.edit_project(&forged, &[edit("forged")]).await,
                Err(CanonicalError::OperationReused)
            ));
        }

        store
            .save_project_draft(
                "save-invalid",
                &project,
                &branch,
                "draft",
                &revision.key,
                Some(1),
                b"malformed [[",
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            store
                .project_draft("draft")
                .await
                .unwrap()
                .unwrap()
                .version
                .content
                .as_slice(),
            b"malformed [["
        );
        assert!(
            store
                .edit_project(&publication, &[edit("v2")])
                .await
                .is_err()
        );
        assert!(
            store
                .project_acknowledgement("publish")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store.project_branch_revision("a").await.unwrap().unwrap(),
            revision
        );
        assert!(matches!(
            store
                .save_project_draft(
                    "save-invalid",
                    &project,
                    &branch,
                    "draft",
                    &revision.key,
                    Some(1),
                    b"different",
                    None
                )
                .await,
            Err(CanonicalError::OperationReused)
                | Err(CanonicalError::Refused {
                    code: crate::canonical::RefusalCode::ProjectOperationIdentityChanged,
                    ..
                })
        ));
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_same_branch_cas_has_only_one_winner() {
        let store = fixture().await;
        let edits = vec![edit("v1")];
        let origin = store.edit("origin", None, "source", &edits).await.unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let revision = fork(&store, "a", &origin, &edits).await;
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch("a").await.unwrap().unwrap();
        let a = store
            .project_publication_intent(&project, &branch, &revision.key, None, "publish-a")
            .unwrap();
        let b = store
            .project_publication_intent(&project, &branch, &revision.key, None, "publish-b")
            .unwrap();
        let ea = vec![edit("va")];
        let eb = vec![edit("vb")];
        let (a, b) = tokio::join!(store.edit_project(&a, &ea), store.edit_project(&b, &eb));
        assert_ne!(a.is_ok(), b.is_ok());
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_fork_rejects_incomplete_closure_and_foreign_cursor() {
        let store = fixture().await;
        let origin = store
            .edit("origin", None, "source", &[edit("v1")])
            .await
            .unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let intent = store
            .project_fork_intent(&project, "a", "problem-a", &origin, "fork")
            .unwrap();
        assert!(store.edit_project(&intent, &[edit("other")]).await.is_err());
        assert!(store.project_branch("a").await.unwrap().is_none());
        let cursor = ProjectCursor {
            issuing_store: uuid::Uuid::new_v4().to_string(),
            project: "project".into(),
            generation: 0,
            after: String::new(),
        };
        assert!(
            store
                .project_branches("project", Some(&cursor))
                .await
                .is_err()
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_research_requires_actual_source_and_exact_receipts_are_immutable() {
        let store = fixture().await;
        let source = store
            .edit("origin", None, "source", &[edit("v1")])
            .await
            .unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        store
            .retain_research_reference("reference", &project, "research", &source.key, None, None)
            .await
            .unwrap();
        assert!(
            store
                .retain_research_reference("missing", &project, "invalid", "absent", None, None)
                .await
                .is_err()
        );
        assert!(
            bounded_query(
                store
                    .db
                    .query("UPDATE canonical_project_operations:create SET kind='mutated';")
            )
            .await
            .is_err()
        );
        assert_eq!(
            store
                .project_acknowledgement("create")
                .await
                .unwrap()
                .unwrap()
                .kind,
            "create"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_lost_acknowledgements_settle_original_atomic_effects() {
        let store = fixture().await;
        let edits = vec![edit("v1")];
        let origin = store.edit("origin", None, "source", &edits).await.unwrap();
        PROJECT_REPLY_LOSS
            .scope(true, async {
                store
                    .create_project("create", "project", "Study")
                    .await
                    .unwrap();
                let forked = fork(&store, "a", &origin, &edits).await;
                assert_eq!(
                    store
                        .project_acknowledgement("fork-a")
                        .await
                        .unwrap()
                        .unwrap()
                        .revision
                        .as_deref(),
                    Some(forked.key.as_str())
                );
                let project = store.project("project").await.unwrap().unwrap();
                let branch = store.project_branch("a").await.unwrap().unwrap();
                let intent = store
                    .project_publication_intent(&project, &branch, &forked.key, None, "publication")
                    .unwrap();
                let revision = store.edit_project(&intent, &[edit("v2")]).await.unwrap();
                assert_eq!(revision.key, "publication");
                assert_eq!(
                    store.project_branch_revision("a").await.unwrap().unwrap(),
                    revision
                );
            })
            .await;
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_concurrent_same_source_operation_different_intents_do_not_false_settle() {
        let store = fixture().await;
        let edits = vec![edit("v1")];
        let origin = store.edit("origin", None, "source", &edits).await.unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let a = store
            .project_fork_intent(&project, "a", "target", &origin, "fork")
            .unwrap();
        let b = store
            .project_fork_intent(&project, "b", "target", &origin, "fork")
            .unwrap();
        let (a, b) = tokio::join!(
            store.edit_project(&a, &edits),
            store.edit_project(&b, &edits)
        );
        assert_ne!(a.is_ok(), b.is_ok());
        let branches = store.project_branches("project", None).await.unwrap();
        assert_eq!(branches.branches.len(), 1);
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn project_native_selection_and_publication_refuse_changed_issuing_authority() {
        let store = fixture().await;
        let edits = vec![edit("v1")];
        let origin = store.edit("origin", None, "source", &edits).await.unwrap();
        store
            .create_project("create", "project", "Study")
            .await
            .unwrap();
        let revision = fork(&store, "a", &origin, &edits).await;
        let project = store.project("project").await.unwrap().unwrap();
        let branch = store.project_branch("a").await.unwrap().unwrap();
        let intent = store
            .project_publication_intent(&project, &branch, &revision.key, None, "publication")
            .unwrap();
        let cursor = ProjectCursor {
            issuing_store: store.issuing_store().unwrap().into(),
            project: "project".into(),
            generation: project.generation,
            after: String::new(),
        };
        let old = store.issuing_store().unwrap().to_owned();
        bounded_query(
            store
                .db
                .query("UPDATE ONLY canonical_interpretations:current SET issuing_store=$issuer;")
                .bind(("issuer", uuid::Uuid::new_v4().to_string())),
        )
        .await
        .unwrap();
        let selection = store.project_branches("project", Some(&cursor)).await;
        let publication = store.edit_project(&intent, &[edit("v2")]).await;
        let native=bounded_query(store.db.query("BEGIN; RETURN fn::pse_project_v1::source_guard($pse_rpc_expires_at,$problem,$operation,$intent); COMMIT;").bind(("problem",intent.problem.clone())).bind(("operation",intent.operation.clone())).bind(("intent",intent.encode().unwrap()))).await;
        // Restore the isolated fixture authority before any assertion can unwind teardown.
        bounded_query(
            store
                .db
                .query("UPDATE ONLY canonical_interpretations:current SET issuing_store=$issuer;")
                .bind(("issuer", old)),
        )
        .await
        .unwrap();
        for result in [
            selection.map(|_| ()),
            publication.map(|_| ()),
            native.map(|_| ()),
        ] {
            assert!(matches!(
                result,
                Err(CanonicalError::Refused {
                    code: crate::canonical::RefusalCode::ProjectIssuingAuthorityChanged,
                    ..
                })
            ));
        }
        assert!(store.revision("publication").await.unwrap().is_none());
        assert_eq!(
            store.project_branch_revision("a").await.unwrap().unwrap(),
            revision
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}
