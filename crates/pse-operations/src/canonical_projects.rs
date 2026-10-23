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
    canonical_project_branches::Row as Branch, canonical_project_drafts::Row as Draft,
    canonical_project_operations::Row as ProjectAcknowledgement,
    canonical_projects::Row as Project, canonical_research_references::Row as ResearchReference,
};
use surrealdb::types::{Object, Value};

#[cfg(all(test, feature = "canonical-tests"))]
tokio::task_local! { static PROJECT_REPLY_LOSS: bool; }
#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) fn simulate_reply_loss() -> bool {
    PROJECT_REPLY_LOSS.try_with(|value| *value).unwrap_or(false)
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
}
impl ProjectPublication {
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
        let expected = match self.kind.as_str() {
            "fork" => {
                if self.expected.is_some()
                    || self.draft.is_some()
                    || self.draft_generation.is_some()
                    || self.branch_generation != 0
                {
                    return Err(CanonicalError::OperationReused);
                }
                let origin = self
                    .origin_revision
                    .as_deref()
                    .ok_or(CanonicalError::OperationReused)?;
                identity(origin)?;
                request(&(
                    "fork",
                    &self.issuing_store,
                    &self.project,
                    self.project_generation,
                    &self.branch,
                    &self.problem,
                    origin,
                    &self.operation,
                ))?
            }
            "publish" => {
                if self.origin_revision.is_some()
                    || self.draft.is_some() != self.draft_generation.is_some()
                {
                    return Err(CanonicalError::OperationReused);
                }
                let parent = self
                    .expected
                    .as_deref()
                    .ok_or(CanonicalError::OperationReused)?;
                identity(parent)?;
                if let Some(draft) = &self.draft {
                    identity(draft)?;
                }
                request(&(
                    "publish",
                    &self.issuing_store,
                    &self.project,
                    self.project_generation,
                    &self.branch,
                    self.branch_generation,
                    &self.problem,
                    parent,
                    self.draft.as_deref().zip(self.draft_generation),
                    &self.operation,
                ))?
            }
            _ => return Err(CanonicalError::OperationReused),
        };
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
        row.insert("kind", self.kind.clone());
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
                        "SELECT * FROM ONLY type::record('canonical_projects',$key);",
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
            let mut response=protected_query("canonical_projects::branches",||Ok(self.project_query("BEGIN; LET $project=SELECT * FROM ONLY type::record('canonical_projects',$key); IF $project=NONE OR ($generation!=NONE AND $project.generation!=$generation) { THROW 'PSE_REFUSAL:project_generation_changed'; }; LET $rows=SELECT * FROM canonical_project_branches WITH INDEX project_branch_order WHERE project=$key AND key>$after ORDER BY key LIMIT 65; RETURN {project:$project,branches:$rows}; COMMIT;")?.bind(("key",project.to_owned())).bind(("generation",cursor.map(|c|codec::encode_uint(c.generation)).transpose()?.unwrap_or(Value::None))).bind(("after",cursor.map_or("",|c|c.after.as_str()).to_owned())))).await?;
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
                        "SELECT * FROM ONLY type::record('canonical_project_branches',$key);",
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
            let mut response=bounded_query(self.project_query("LET $branch=SELECT * FROM ONLY type::record('canonical_project_branches',$key); LET $problem=IF $branch=NONE {NONE} ELSE {SELECT * FROM ONLY type::record('canonical_problems',$branch.problem)}; RETURN IF $problem=NONE {NONE} ELSE {SELECT * FROM ONLY type::record('canonical_revisions',$problem.head)};")?.bind(("key",key.to_owned()))).await?;
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
        Ok(ProjectPublication {
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
        })
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
            || draft.is_some_and(|d| d.branch != branch.key || d.base_revision != expected)
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
        Ok(ProjectPublication {
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
        })
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
        let row = Draft {
            key: key.into(),
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
            &row.key,
            &row.project,
            &row.branch,
            &row.base_revision,
            project.generation,
            expected,
            crate::canonical_execution::result_payload_digest(content),
            artifact,
        ))?;
        let encoded = wire::encode_canonical_project_drafts(&row)?;
        let obligation = obligation(artifact, key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async{self.ensure_writes()?;let result=protected_query("canonical_projects::draft",||Ok(self.project_query("BEGIN; RETURN fn::pse_project_v1::draft($pse_rpc_expires_at,$operation,$request,$row,$expected,$project_generation,$obligation); COMMIT;")?.bind(("operation",operation.to_owned())).bind(("request",surrealdb::types::Bytes::from(request.clone()))).bind(("row",encoded.clone())).bind(("expected",expected.map(codec::encode_uint).transpose()?.unwrap_or(Value::None))).bind(("project_generation",codec::encode_uint(project.generation)?)).bind(("obligation",obligation.clone())))).await;self.settle_project_effect(operation,&request,result).await})).await
    }
    /// Read raw draft custody; source admission is deliberately a separate command.
    pub async fn project_draft(&self, key: &str) -> Result<Option<Draft>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.project_query(
                        "SELECT * FROM ONLY type::record('canonical_project_drafts',$key);",
                    )?
                    .bind(("key", key.to_owned())),
                )
                .await?;
                response
                    .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                    .map(wire::decode_canonical_project_drafts)
                    .transpose()
                    .map_err(Into::into)
            }))
            .await
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

#[cfg(all(test, feature = "canonical-tests"))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "isolated native project fixtures assert exact owned effects"
    )]
    use super::*;
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
        assert!(draft.content.is_empty());
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
