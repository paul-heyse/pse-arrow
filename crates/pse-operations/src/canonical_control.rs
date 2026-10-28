// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable control documents; construction confers no scientific admission.
use crate::canonical_study_workbench::{StudyWorkbenchFilter, StudyWorkbenchHeader};
use pse_model::document::Version;

/// Authority-bound continuation for live browsing, never a frozen analysis population.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchCursor {
    /// Canonical issuing database identity.
    pub issuing_store: String,
    /// Exact original predicates.
    pub filter: StudyWorkbenchFilter,
    /// Last displayed study key.
    pub after: String,
}
/// Bounded live header selection; continuation pages can observe newer metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchRequest {
    /// Exact scientific and engineering predicates.
    pub filter: StudyWorkbenchFilter,
    /// Maximum headers, from one through 64.
    pub maximum: usize,
    /// Issuer- and predicate-bound continuation.
    pub cursor: Option<StudyWorkbenchCursor>,
}
/// Live browsing page, explicitly independent from frozen completeness.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchPage {
    /// Compact actual metadata; payloads are unopened.
    pub headers: Vec<StudyWorkbenchHeader>,
    /// More headers under this predicate, if present.
    pub next: Option<StudyWorkbenchCursor>,
}

/// Versioned application request for compact live study headers.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlBrowseRequest {
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Existing indexed predicates; never arbitrary SQL.
    pub filter: StudyWorkbenchFilter,
    /// Maximum compact rows, from one through 64.
    pub maximum: u32,
    /// Original issuing-store and predicate binding.
    pub cursor: Option<StudyWorkbenchCursor>,
}
/// Compact HTTP projection with exact decimal strings rather than JSON floating point.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlStudyHeader {
    /// Exact study identity.
    pub study: String,
    /// Exact parent execution.
    pub run: String,
    /// Original source owner.
    pub problem: String,
    /// Exact original modeling revision.
    pub source: String,
    /// Optional original physical revision.
    pub physical_revision: Option<String>,
    /// Optional original physical identity.
    pub physical_identity: Option<String>,
    /// Optional original engineering context.
    pub context: Option<String>,
    /// Exact nonnegative u64 decimal encoding of the population count.
    pub point_count: String,
    /// Exact nonnegative u64 decimal encoding of the cancellation generation.
    pub generation: String,
    /// Original membership is activated.
    pub active: bool,
    /// Actual persistent cancellation flag.
    pub cancelled: bool,
    /// Parent summary is concluded.
    pub terminal: bool,
    /// Exact terminal attempt, when present.
    pub result_attempt: Option<String>,
    /// Original operational outcome; never scientific usability.
    pub outcome: Option<pse_model::generated::enums::CanonicalTerminalOutcome>,
}
/// Bounded live headers, not a frozen complete population.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlBrowseResponse {
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Compact headers with exact integer wire values.
    pub headers: Vec<ControlStudyHeader>,
    /// Original issuer- and predicate-bound continuation.
    pub next: Option<StudyWorkbenchCursor>,
}
/// Stable cancellation identity and immutable intent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlCancelRequest {
    /// Original issuing database; never rebound after restore.
    pub issuing_store: String,
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Caller-retained operation identity, scoped to the authenticated actor and store.
    pub operation_id: String,
    /// Exact study whose future claims are revoked.
    pub study: String,
}
impl ControlCancelRequest {
    /// Issue a fresh retained command identity; callers keep this value across uncertain delivery.
    pub fn issue(issuing_store: String, study: String) -> Self {
        Self {
            document_version: Default::default(),
            issuing_store,
            study,
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
/// Read the original cancellation decision without issuing another effect.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlReconcileRequest {
    /// Original issuing database; never rebound after restore.
    pub issuing_store: String,
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Original caller-retained operation identity.
    pub operation_id: String,
}
/// Persisted command disposition, distinct from an unresolved transport outcome.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ControlCommandStatus {
    /// The command was accepted, including an already concluded no-op.
    Accepted,
    /// The original decision refused the request without cancelling anything.
    Refused,
}
/// One exact durable cancellation decision; not a native completion witness.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlCommandReceipt {
    /// Original issuing database; never rebound after restore.
    pub issuing_store: String,
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Original operation identity.
    pub operation_id: String,
    /// Original immutable target.
    pub study: String,
    /// Recorded command disposition.
    pub status: ControlCommandStatus,
    /// Actual generation as exact decimal text; absent when the study was absent.
    pub generation: Option<String>,
    /// The study was already terminal at the original decision.
    pub already_concluded: bool,
    /// Actual cancellation state at the original decision.
    pub cancelled: bool,
    /// Typed declared refusal identity, when refused.
    pub reason: Option<String>,
}
/// Definitive pre-effect refusal; unlike a command receipt this response is not persisted.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlCommandRefusal {
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Original issuing database.
    pub issuing_store: String,
    /// Reused operation identity.
    pub operation_id: String,
    /// Closed declared refusal identity.
    pub reason: String,
}
/// Missing means unobserved, never proof that an earlier request cannot still commit.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlReconcileResponse {
    /// Exact control wire interpretation.
    pub document_version: Version<1>,
    /// Original immutable decision if observed under this actor and issuing store.
    pub receipt: Option<ControlCommandReceipt>,
}

/// Safe administrative acknowledgement; no password or token is returned.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlActorIssue {
    /// Exact server actor record identity.
    pub actor_id: String,
    /// Original canonical database identity.
    pub issuing_store: String,
}
impl crate::canonical::CanonicalStore {
    /// Privileged explicit provisioning on an already initialized store.
    /// Existing actor identities refuse rather than overwrite credentials or grants.
    pub async fn issue_control_actor(
        &self,
        key: &str,
        password: &str,
        browse: bool,
        cancel: bool,
    ) -> Result<ControlActorIssue, crate::canonical::CanonicalError> {
        use crate::canonical::transport::{original_deadline, within_clock};
        use crate::canonical::{CanonicalError, REQUEST_TIMEOUT, protected_query};
        if key.is_empty() || key.len() > 128 || password.len() < 16 || password.len() > 4096 {
            return Err(CanonicalError::PayloadLimit);
        }
        self.ensure_writes()?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut reply = protected_query("canonical_control::issue_actor", || Ok(self.db.script(
                crate::canonical_script::GuardedScript::new(crate::canonical_script::Envelope::DeadlineOnly).returning::<Option<String>>(
                    "IF (SELECT id FROM ONLY type::record('canonical_control_actors',$key))!=NONE {THROW 'PSE_REFUSAL:control_actor_already_issued';}; LET $issued=CREATE ONLY type::record('canonical_control_actors',$key) SET key=$key,password=crypto::argon2::generate($password),enabled=true,browse=$browse,cancel=$cancel;",
                    "encoding::json::encode({actor_id:<string>$issued.id,issuing_store:(SELECT * FROM ONLY canonical_interpretations:current).issuing_store})",
                )?
            ).bind(("key",key.to_owned())).bind(("password",password.to_owned())).bind(("browse",browse)).bind(("cancel",cancel)))).await?;
            let encoded = crate::canonical_script::ResultSlot::<Option<String>>::returned().take(&mut reply)?.ok_or(CanonicalError::IncompleteResponse)?;
            serde_json::from_str(&encoded).map_err(|error| CanonicalError::Configuration(error.to_string()))
        })).await
    }
    /// Compact browsing through the neutral operational owner; no scientific root.
    pub async fn control_study_workbench_page(
        &self,
        request: StudyWorkbenchRequest,
    ) -> Result<StudyWorkbenchPage, crate::canonical::CanonicalError> {
        use crate::canonical::CanonicalError;
        let issuer = self.issuing_store()?.to_owned();
        if request.cursor.as_ref().is_some_and(|cursor| {
            cursor.issuing_store != issuer
                || cursor.filter != request.filter
                || cursor.after.is_empty()
                || cursor.after.len() > 4096
        }) {
            return Err(CanonicalError::Configuration(
                "workbench cursor identity, predicates or extent are invalid".into(),
            ));
        }
        let page = self
            .select_study_workbench(
                &request.filter,
                request.cursor.as_ref().map(|cursor| cursor.after.as_str()),
                request.maximum,
            )
            .await?;
        let next = if page.has_more {
            page.headers.last().map(|header| StudyWorkbenchCursor {
                issuing_store: issuer,
                filter: request.filter,
                after: header.study.clone(),
            })
        } else {
            None
        };
        Ok(StudyWorkbenchPage {
            headers: page.headers,
            next,
        })
    }
    /// Observe one native command receipt; missing does not authorize replay.
    pub async fn reconcile_study_cancel(
        &self,
        request: &ControlReconcileRequest,
    ) -> Result<ControlReconcileResponse, crate::canonical::CanonicalError> {
        use crate::canonical::transport::{original_deadline, within_clock};
        use crate::canonical::{CanonicalError, REQUEST_TIMEOUT, protected_query};
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let mut reply=protected_query("canonical_control::reconcile",||Ok(self.db.query("RETURN encoding::json::encode(fn::pse_control_v1::reconcile($pse_rpc_expires_at,'native-system',{document_version:1,issuing_store:$issuing_store,operation_id:$operation_id}));").bind(("issuing_store",request.issuing_store.clone())).bind(("operation_id",request.operation_id.clone())))).await?;
            let encoded:Option<String>=reply.take(0)?;
            let encoded=encoded.ok_or(CanonicalError::IncompleteResponse)?;
            serde_json::from_str(&encoded).map_err(|error|CanonicalError::Configuration(error.to_string()))
        })).await
    }
    /// Settle only the original command; an unobserved receipt never permits another mutation.
    pub(crate) async fn finish_control_cancel(
        &self,
        request: &ControlCancelRequest,
        response: Result<ControlCommandReceipt, crate::canonical::CanonicalError>,
    ) -> Result<ControlCommandReceipt, crate::canonical::CanonicalError> {
        use crate::canonical::transport::{original_deadline, within_clock};
        use crate::canonical::{CanonicalError, REQUEST_TIMEOUT};
        within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let intent = serde_json::to_vec(request)
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
            crate::canonical::settlement::settle_exact(
                "control-study-cancel",
                &request.operation_id,
                &intent,
                response,
                async {
                    Ok(self
                        .reconcile_study_cancel(&ControlReconcileRequest {
                            document_version: Default::default(),
                            issuing_store: request.issuing_store.clone(),
                            operation_id: request.operation_id.clone(),
                        })
                        .await?
                        .receipt)
                },
                |receipt| {
                    receipt.issuing_store == request.issuing_store
                        && receipt.operation_id == request.operation_id
                        && receipt.study == request.study
                },
                CanonicalError::source_transaction_refused,
            )
            .await
        })
        .await
    }
    /// Issue one privileged native cancellation using the same recorded command owner.
    pub async fn cancel_study_command(
        &self,
        request: &ControlCancelRequest,
    ) -> Result<ControlCommandReceipt, crate::canonical::CanonicalError> {
        use crate::canonical::transport::{original_deadline, within_clock};
        use crate::canonical::{CanonicalError, REQUEST_TIMEOUT, protected_query};
        if request.operation_id.is_empty()
            || request.operation_id.len() > 128
            || request.study.is_empty()
            || request.study.len() > 4096
            || request.issuing_store.len() > 4096
        {
            return Err(CanonicalError::PayloadLimit);
        }
        self.ensure_writes()?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let response=async {
                let mut reply=protected_query("canonical_control::cancel",||Ok(self.db.query("RETURN encoding::json::encode(fn::pse_control_v1::cancel($pse_rpc_expires_at,'native-system',{document_version:1,issuing_store:$issuing_store,operation_id:$operation_id,study:$study}));").bind(("issuing_store",request.issuing_store.clone())).bind(("operation_id",request.operation_id.clone())).bind(("study",request.study.clone())))).await?;
                let encoded:Option<String>=reply.take(0)?;
                let encoded=encoded.ok_or(CanonicalError::IncompleteResponse)?;
                if let Ok(refusal)=serde_json::from_str::<ControlCommandRefusal>(&encoded) {
                    if refusal.reason=="control_command_identity_reused" && refusal.issuing_store==request.issuing_store && refusal.operation_id==request.operation_id {return Err(CanonicalError::OperationReused);}
                    return Err(CanonicalError::IncompleteResponse);
                }
                serde_json::from_str(&encoded).map_err(|error|CanonicalError::Configuration(error.to_string()))
            }.await;
            self.finish_control_cancel(request,response).await
        })).await
    }
}
