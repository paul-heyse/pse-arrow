// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable project engineering contexts and exact, authority-bound recorded cuts.
use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, REQUEST_TIMEOUT, bounded_query, protected_query},
    canonical_codec as codec,
    generated::surreal as wire,
};
pub use pse_model::generated::runtime::canonical_project_contexts::Row as ProjectContext;
use surrealdb::types::Object;
/// Exact project journal position; never a source revision sequence or wall clock.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ContextCut {
    /// Issuing canonical database identity.
    pub issuing_store: String,
    /// Stable project identity.
    pub project: String,
    /// Last included journal position.
    pub sequence: u64,
}
/// Complete frozen selection. Deserializing this document grants no source authority.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextSelection {
    /// Original recorded cut.
    pub cut: ContextCut,
    /// UTC instant in signed microseconds since the Unix epoch.
    pub effective_at_us: i64,
    /// Original immutable selected source descriptor.
    pub context: ProjectContext,
}
impl Eq for ContextSelection {}
fn identity(value: &str) -> Result<(), CanonicalError> {
    if value.is_empty() || value.len() > 4096 {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}
fn final_object(
    response: &mut surrealdb::IndexedResults,
) -> Result<Option<Object>, CanonicalError> {
    let index = response
        .num_statements()
        .checked_sub(2)
        .ok_or(CanonicalError::IncompleteResponse)?;
    response.take::<Option<Object>>(index).map_err(Into::into)
}
fn encoded<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json::to_vec(value).map_err(|e| CanonicalError::Configuration(e.to_string()))
}
impl CanonicalStore {
    fn context_query(
        &self,
        sql: &str,
    ) -> Result<surrealdb::method::Query<'_, surrealdb::engine::remote::grpc::Client>, CanonicalError>
    {
        Ok(self.db.query(format!("BEGIN; fn::pse_project_v1::authority($pse_rpc_expires_at,$context_issuer); {sql} COMMIT;")).bind(("context_issuer",self.issuing_store()?.to_owned())))
    }
    /// Capture a project journal position without loading scientific payloads.
    pub async fn project_context_cut(&self, project: &str) -> Result<ContextCut, CanonicalError> {
        identity(project)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let mut response=bounded_query(self.context_query("LET $project=SELECT * FROM ONLY type::record('canonical_projects',$project); IF $project=NONE OR !fn::pse_project_import_v1::available($project.key) {THROW 'PSE_REFUSAL:project_context_unavailable';}; RETURN SELECT * FROM ONLY type::record('canonical_context_journals',$project.key);")?.bind(("project",project.to_owned()))).await?;
            let sequence=final_object(&mut response)?.map(wire::decode_canonical_context_journals).transpose()?.map_or(0,|r|r.sequence);
            Ok(ContextCut{issuing_store:self.issuing_store()?.into(),project:project.into(),sequence})
        })).await
    }
    /// Append one immutable record. Retry the original operation and all original coordinates.
    #[allow(
        clippy::too_many_arguments,
        reason = "immutable context append binds complete original source, effective interval and ancestry"
    )]
    pub async fn append_project_context(
        &self,
        operation: &str,
        project: &crate::canonical_projects::Project,
        scope: &str,
        revision: &str,
        source_identity: &str,
        physical_context: &str,
        physical_revision: &str,
        physical_source_identity: &str,
        effective_from_us: i64,
        effective_to_us: Option<i64>,
        corrects: Option<&str>,
        deleted: bool,
    ) -> Result<ProjectContext, CanonicalError> {
        for value in [
            operation,
            &project.key,
            scope,
            revision,
            source_identity,
            physical_context,
            physical_revision,
            physical_source_identity,
        ] {
            identity(value)?;
        }
        if let Some(parent) = corrects {
            identity(parent)?;
        }
        if effective_to_us.is_some_and(|to| to <= effective_from_us)
            || deleted && corrects.is_none()
        {
            return Err(CanonicalError::Configuration(
                "invalid context interval or tombstone ancestry".into(),
            ));
        }
        let request = encoded(&(
            "context-v1",
            self.issuing_store()?,
            operation,
            &project.key,
            project.generation,
            scope,
            revision,
            source_identity,
            physical_context,
            physical_revision,
            physical_source_identity,
            effective_from_us,
            effective_to_us,
            corrects,
            deleted,
        ))?;
        let row = ProjectContext {
            key: operation.into(),
            project: project.key.clone(),
            scope: scope.into(),
            revision: revision.into(),
            source_identity: source_identity.into(),
            physical_context: physical_context.into(),
            physical_revision: physical_revision.into(),
            physical_source_identity: physical_source_identity.into(),
            effective_from_us,
            effective_to_us,
            recorded_sequence: 0,
            corrects: corrects.map(str::to_owned),
            deleted,
            request: request.clone().into(),
        };
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            self.ensure_writes()?;
            let response=protected_query("canonical_contexts::append",||Ok(self.context_query("RETURN fn::pse_context_v1::append($pse_rpc_expires_at,$row,$generation);")?.bind(("row",wire::encode_canonical_project_contexts(&row)?)).bind(("generation",codec::encode_uint(project.generation)?)))).await;
            #[cfg(all(test,feature="canonical-tests"))]
            let response=if response.is_ok()&&crate::canonical_projects::simulate_reply_loss(){Err(CanonicalError::IncompleteResponse)}else{response};
            let write = response.and_then(|mut response| {
                wire::decode_canonical_project_contexts(final_object(&mut response)?.ok_or(CanonicalError::IncompleteResponse)?).map_err(Into::into)
            });
            crate::canonical::settlement::settle_exact(
                "project-context", operation, &request, write,
                self.project_context_record(operation),
                |got| got.request.as_slice() == request,
                CanonicalError::source_transaction_refused,
            ).await
        })).await
    }
    /// Exact immutable metadata, including tombstones and hidden project membership.
    /// This does not assert that scientific source payloads remain available.
    pub async fn project_context_record(
        &self,
        key: &str,
    ) -> Result<Option<ProjectContext>, CanonicalError> {
        identity(key)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {let mut response=bounded_query(self.context_query("RETURN SELECT * FROM ONLY type::record('canonical_project_contexts',$key) WHERE fn::pse_project_import_v1::available(project);")?.bind(("key",key.to_owned()))).await?;final_object(&mut response)?.map(wire::decode_canonical_project_contexts).transpose().map_err(Into::into)})).await
    }
    fn check_cut(&self, cut: &ContextCut) -> Result<(), CanonicalError> {
        identity(&cut.project)?;
        if cut.issuing_store != self.issuing_store()? {
            return Err(CanonicalError::Configuration(
                "context cut issuing authority differs".into(),
            ));
        }
        Ok(())
    }
    /// Resolve a UTC effective instant at the original recorded cut. Unrelated overlaps refuse.
    pub async fn select_project_context(
        &self,
        cut: &ContextCut,
        scope: &str,
        effective_at_us: i64,
    ) -> Result<Option<ContextSelection>, CanonicalError> {
        self.check_cut(cut)?;
        identity(scope)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let mut response=bounded_query(self.context_query("IF !fn::pse_project_import_v1::available($project) {THROW 'PSE_REFUSAL:project_context_unavailable';}; LET $selected=fn::pse_context_v1::select($pse_rpc_expires_at,$project,$scope,$effective,$cut); IF $selected!=NONE {fn::pse_context_v1::available($pse_rpc_expires_at,$selected.revision); fn::pse_context_v1::available($pse_rpc_expires_at,$selected.physical_revision);}; RETURN $selected;")?.bind(("project",cut.project.clone())).bind(("scope",scope.to_owned())).bind(("effective",effective_at_us)).bind(("cut",codec::encode_uint(cut.sequence)?))).await?;
            Ok(final_object(&mut response)?.map(wire::decode_canonical_project_contexts).transpose()?.map(|context|ContextSelection{cut:cut.clone(),effective_at_us,context}))
        })).await
    }
    /// Validate all frozen coordinates against original immutable facts, never today's selection.
    pub async fn validate_project_context(
        &self,
        selection: &ContextSelection,
    ) -> Result<(), CanonicalError> {
        self.check_cut(&selection.cut)?;
        if selection.context.project != selection.cut.project {
            return Err(CanonicalError::OperationReused);
        }
        let selected = self
            .select_project_context(
                &selection.cut,
                &selection.context.scope,
                selection.effective_at_us,
            )
            .await?
            .ok_or(CanonicalError::OperationReused)?;
        if encoded(&selected)? != encoded(selection)? {
            return Err(CanonicalError::OperationReused);
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "isolated history fixtures assert original effects and exact selection"
    )]
    use super::*;
    async fn fixture() -> CanonicalStore {
        let state = std::env::var("PSE_SURREAL_STATE").expect("canonical fixture service");
        let mut options =
            crate::canonical::CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_contexts_{}", uuid::Uuid::new_v4().simple());
        crate::testing::canonical_fixture_with_options(&options, true).unwrap()
    }
    async fn source(store: &CanonicalStore, key: &str) -> crate::canonical::Revision {
        store.edit("science", None, key, &[]).await.unwrap()
    }
    #[tokio::test]
    async fn project_context_corrections_preserve_original_cuts_and_half_open_intervals() {
        let store = fixture().await;
        let source = source(&store, "source").await;
        store
            .create_project("create", "project", "History")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        store
            .append_project_context(
                "original",
                &project,
                "calibration",
                &source.key,
                "source-hash",
                "physical",
                &source.key,
                "physical-source-hash",
                0,
                None,
                None,
                false,
            )
            .await
            .unwrap();
        let old = store.project_context_cut("project").await.unwrap();
        let retained = store
            .select_project_context(&old, "calibration", 26)
            .await
            .unwrap()
            .unwrap();
        store
            .append_project_context(
                "correction",
                &project,
                "calibration",
                &source.key,
                "corrected-hash",
                "physical",
                &source.key,
                "physical-source-hash",
                20,
                Some(40),
                Some("original"),
                false,
            )
            .await
            .unwrap();
        let corrected = store.project_context_cut("project").await.unwrap();
        store
            .append_project_context(
                "deletion",
                &project,
                "calibration",
                &source.key,
                "corrected-hash",
                "physical",
                &source.key,
                "physical-source-hash",
                25,
                Some(30),
                Some("correction"),
                true,
            )
            .await
            .unwrap();
        let deleted = store.project_context_cut("project").await.unwrap();
        assert_eq!(
            store
                .select_project_context(&old, "calibration", 26)
                .await
                .unwrap()
                .unwrap()
                .context
                .key,
            "original"
        );
        assert_eq!(
            store
                .select_project_context(&corrected, "calibration", 26)
                .await
                .unwrap()
                .unwrap()
                .context
                .key,
            "correction"
        );
        assert!(
            store
                .select_project_context(&deleted, "calibration", 26)
                .await
                .unwrap()
                .is_none()
        );
        for (at, key) in [
            (19, "original"),
            (20, "correction"),
            (30, "correction"),
            (39, "correction"),
            (40, "original"),
        ] {
            assert_eq!(
                store
                    .select_project_context(&deleted, "calibration", at)
                    .await
                    .unwrap()
                    .unwrap()
                    .context
                    .key,
                key
            );
        }
        store.validate_project_context(&retained).await.unwrap();
        assert_eq!(retained.context.key, "original");
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_context_refuses_unrelated_overlap_foreign_cut_and_forged_record() {
        let store = fixture().await;
        let source = source(&store, "source").await;
        store
            .create_project("create", "project", "History")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        store
            .append_project_context(
                "a",
                &project,
                "scope",
                &source.key,
                "hash",
                "physical",
                &source.key,
                "physical-source-hash",
                -100,
                None,
                None,
                false,
            )
            .await
            .unwrap();
        let cut = store.project_context_cut("project").await.unwrap();
        let original = store
            .select_project_context(&cut, "scope", 0)
            .await
            .unwrap()
            .unwrap();
        let mut forged = original.clone();
        forged.context.source_identity = "forged".into();
        assert!(store.validate_project_context(&forged).await.is_err());
        let mut foreign = cut.clone();
        foreign.issuing_store = "foreign".into();
        assert!(
            store
                .select_project_context(&foreign, "scope", 0)
                .await
                .is_err()
        );
        store
            .append_project_context(
                "b",
                &project,
                "scope",
                &source.key,
                "other",
                "physical",
                &source.key,
                "physical-source-hash",
                0,
                None,
                None,
                false,
            )
            .await
            .unwrap();
        let cut = store.project_context_cut("project").await.unwrap();
        assert!(matches!(
            store.select_project_context(&cut, "scope", 0).await,
            Err(CanonicalError::Refused {
                code: crate::canonical::RefusalCode::ProjectContextAmbiguous,
                ..
            })
        ));
        assert!(
            store
                .append_project_context(
                    "bad",
                    &project,
                    "other",
                    &source.key,
                    "hash",
                    "physical",
                    &source.key,
                    "physical-source-hash",
                    0,
                    None,
                    Some("a"),
                    false
                )
                .await
                .is_err()
        );
        store.validate_project_context(&original).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn project_context_exact_replay_settles_once_and_released_source_is_unavailable() {
        let store = fixture().await;
        let source = source(&store, "source").await;
        let physical = store
            .edit("physical", None, "physical-source", &[])
            .await
            .unwrap();
        store
            .create_project("create", "project", "History")
            .await
            .unwrap();
        let project = store.project("project").await.unwrap().unwrap();
        let row = crate::canonical_projects::PROJECT_REPLY_LOSS
            .scope(
                true,
                store.append_project_context(
                    "a",
                    &project,
                    "scope",
                    &source.key,
                    "hash",
                    "physical",
                    &physical.key,
                    "physical-source-hash",
                    0,
                    None,
                    None,
                    false,
                ),
            )
            .await
            .unwrap();
        let replay = store
            .append_project_context(
                "a",
                &project,
                "scope",
                &source.key,
                "hash",
                "physical",
                &physical.key,
                "physical-source-hash",
                0,
                None,
                None,
                false,
            )
            .await
            .unwrap();
        assert_eq!(row, replay);
        assert_eq!(
            store.project_context_cut("project").await.unwrap().sequence,
            1
        );
        let (b, c) = tokio::join!(
            store.append_project_context(
                "b",
                &project,
                "scope-b",
                &source.key,
                "hash",
                "physical",
                &physical.key,
                "physical-source-hash",
                0,
                None,
                None,
                false
            ),
            store.append_project_context(
                "c",
                &project,
                "scope-c",
                &source.key,
                "hash",
                "physical",
                &physical.key,
                "physical-source-hash",
                0,
                None,
                None,
                false
            ),
        );
        let b = b.unwrap();
        let c = c.unwrap();
        assert_ne!(b.recorded_sequence, c.recorded_sequence);
        assert_eq!(
            store.project_context_cut("project").await.unwrap().sequence,
            3
        );
        assert!(
            bounded_query(
                store
                    .db
                    .query("UPDATE canonical_project_contexts:a SET source_identity='mutated';")
            )
            .await
            .is_err()
        );
        assert!(
            store
                .append_project_context(
                    "a",
                    &project,
                    "scope",
                    &source.key,
                    "forged",
                    "physical",
                    &physical.key,
                    "physical-source-hash",
                    0,
                    None,
                    None,
                    false
                )
                .await
                .is_err()
        );
        let cut = store.project_context_cut("project").await.unwrap();
        let frozen = store
            .select_project_context(&cut, "scope", 0)
            .await
            .unwrap()
            .unwrap();
        // Exercise the source owner's declared unavailable-range gate, not current-value fallback.
        bounded_query(store.db.query("CREATE canonical_reclaimed_ranges:released CONTENT {key:'released',problem:$problem,from_sequence:$from,to_sequence:$to};").bind(("problem",source.problem.clone())).bind(("from",codec::encode_uint(source.sequence).unwrap())).bind(("to",codec::encode_uint(source.sequence+1).unwrap()))).await.unwrap();
        assert!(store.validate_project_context(&frozen).await.is_err());
        bounded_query(store.db.query("DELETE canonical_reclaimed_ranges:released; CREATE canonical_reclaimed_ranges:physical_released CONTENT {key:'physical_released',problem:$problem,from_sequence:$from,to_sequence:$to};").bind(("problem",physical.problem.clone())).bind(("from",codec::encode_uint(physical.sequence).unwrap())).bind(("to",codec::encode_uint(physical.sequence+1).unwrap()))).await.unwrap();
        assert!(
            store.validate_project_context(&frozen).await.is_err(),
            "physical closure release is not replaced by current physical inputs"
        );
        assert!(store.project_context_record("a").await.unwrap().is_some());
        store.remove_isolated_fixture().await.unwrap();
    }
}
