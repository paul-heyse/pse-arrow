// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native preparation handoff for bounded canonical study progression.
use super::*;

#[cfg(any(test, feature = "test-support"))]
tokio::task_local! {
    static PREPARED_ACK_LOST: bool;
    static PREPARED_READBACK_FAILED: bool;
}

#[cfg(feature = "test-support")]
#[path = "progression_controls.rs"]
/// Controls that require an explicitly supplied privately owned canonical store.
pub mod controls;

/// Exact bounded traversal position. Only the issuing database can consume it.
#[derive(Clone, Debug)]
pub struct StudyContinuation {
    issuer: String,
    module: String,
    coordinates: Object,
}
/// Canonically selected preparation, never an execution claim.
#[derive(Clone, Debug)]
pub struct StudyPreparation {
    issuer: String,
    module: String,
    scope: StudyScope,
    action: pse_model::study::PointAction,
}
impl StudyPreparation {
    /// Complete original immediate premise set used by native source/seed preparation.
    pub fn scope(&self) -> &StudyScope {
        &self.scope
    }
    /// Shared-policy action returned by the canonical owner, without native reinterpretation.
    pub fn action(&self) -> &pse_model::study::PointAction {
        &self.action
    }
}
/// Applicable bounded domain action selected and, where effect-free, applied canonically.
#[derive(Clone, Debug)]
pub enum StudyProgressAction {
    /// Prepare the exact selected source/seed outside the transaction.
    Prepare(Box<StudyPreparation>),
    /// All occurrences settled; assemble the existing native parent summary.
    Finalize(StudySummary),
    /// One refusal or cancellation was settled without a native attempt.
    Settled {
        /// Exact occurrence.
        point: String,
    },
    /// A durable readiness repair page was resumed using its original operation.
    Maintenance {
        /// Exact owning study.
        study: String,
    },
    /// No action on this examined population. Follow continuation before declaring idle.
    Waiting,
}
/// One bounded canonical traversal result. An empty examined page is not exhaustive idle.
#[derive(Clone, Debug)]
pub struct StudyProgress {
    /// Selected domain action.
    pub action: StudyProgressAction,
    /// Continue bounded discovery; only absence establishes end of this pass.
    pub continuation: Option<StudyContinuation>,
    /// Actual candidate or readiness-page entries examined.
    pub examined: u64,
}
/// Canonical disposition after native preparation and actual launch admission.
#[derive(Debug)]
pub enum PreparedStudyClaim {
    /// Exact original execution claim; no native start has happened yet.
    Claimed(Box<StudyClaim>),
    /// Seed policy refused and the occurrence was settled without native execution.
    Settled {
        /// Exact occurrence.
        point: String,
    },
    /// No start was authorized; release unused native admission.
    Waiting,
}

fn scope_from_object(mut object: Object) -> Result<StudyScope, CanonicalError> {
    let mut study = readiness_object(codec::required(&mut object, "study")?)?;
    let key = codec::decode_string(codec::required(&mut study, "key")?)?;
    let generation = codec::decode_uint(codec::required(&mut study, "generation")?)?;
    let cancelled = codec::decode_boolean(codec::required(&mut study, "cancelled")?)?;
    let point = decode_scope_point(readiness_object(codec::required(&mut object, "point")?)?)?;
    let predecessors = readiness_array(codec::required(&mut object, "predecessors")?)?
        .into_iter()
        .map(|row| decode_scope_point(readiness_object(row)?))
        .collect::<Result<Vec<_>, _>>()?;
    if predecessors.len() > 64
        || point.study != key
        || predecessors.iter().any(|row| row.study != key)
    {
        return Err(CanonicalError::IncompleteResponse);
    }
    Ok(StudyScope {
        study: key,
        generation,
        cancelled,
        point,
        predecessors,
    })
}
impl CanonicalStore {
    /// Progress one bounded canonical population. Native code only resumes the
    /// existing interrupted-page protocol; traversal and policy remain in SurrealQL.
    pub async fn progress_studies(
        &self,
        continuation: Option<&StudyContinuation>,
    ) -> Result<StudyProgress, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let issuer = self.issuing_store()?.to_owned();
            let module = self.study_module_hash()?.to_owned();
            if continuation.is_some_and(|cursor| cursor.issuer != issuer || cursor.module != module) {
                return Err(CanonicalError::Configuration("study continuation belongs to a different issuing store or policy module".into()));
            }
            let cursor = continuation.map(|cursor| Value::Object(cursor.coordinates.clone())).unwrap_or(Value::None);
            let operation = format!("study-readiness:{}", uuid::Uuid::new_v4());
            let request = encode(&(&issuer, &module, cursor.to_sql()))?;
            let response = protected_query("canonical_studies::progress_studies", || Ok(self.db.query(
                "RETURN fn::pse_study_v1::progress($pse_rpc_expires_at,$issuer,$module,$cursor,$operation,$request);")
                .bind(("issuer", issuer.clone())).bind(("module", module.clone())).bind(("cursor", cursor.clone())).bind(("operation", operation.clone())).bind(("request", Bytes::from(request.clone()))))).await
                .and_then(|mut response| response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse));
            let mut row = match response {
                Ok(row) => row,
                Err(error) => crate::canonical::settlement::settle_exact("study-progression", &operation, &request, Err(error), async {
                    let mut response = protected_query("canonical_studies::progress_result", || Ok(self.db.query(
                        "RETURN fn::pse_study_v1::progress_result($pse_rpc_expires_at,$operation,$request);")
                        .bind(("operation", operation.clone())).bind(("request", Bytes::from(request.clone()))))).await?;
                    Ok(response.take::<Option<Object>>(0)?)
                }, |_| true, CanonicalError::source_transaction_refused).await?,
            };
            let kind = codec::decode_string(codec::required(&mut row, "kind")?)?;
            let mut examined = codec::decode_uint(codec::required(&mut row, "examined")?)?;
            let continuation = row.remove("continuation").filter(|value| !matches!(value, Value::None | Value::Null))
                .map(|value| Ok::<_, CanonicalError>(StudyContinuation { issuer: issuer.clone(), module: module.clone(), coordinates: readiness_object(value)? })).transpose()?;
            let action = match kind.as_str() {
                "prepare" => {
                    let scope = scope_from_object(readiness_object(codec::required(&mut row, "scope")?)?)?;
                    let action: pse_model::study::PointAction = decode(codec::decode_bytes(codec::required(&mut row, "action")?)?.as_slice())?;
                    if u64::from(action.occurrence.0) != scope.point.occurrence || action.expected_revision != scope.point.revision {
                        return Err(CanonicalError::IncompleteResponse);
                    }
                    StudyProgressAction::Prepare(Box::new(StudyPreparation { issuer, module, scope, action }))
                },
                "settled" => StudyProgressAction::Settled { point: codec::decode_string(codec::required(&mut row, "point")?)? },
                "finalize" => StudyProgressAction::Finalize(decode_study_summary(readiness_object(codec::required(&mut row, "study")?)?)?),
                "repair" => {
                    let (pending, created) = decode_pending_readiness(readiness_object(codec::required(&mut row, "pending")?)?)?;
                    let study = pending.study.clone();
                    examined = self.resume_readiness_page(pending, !created).await?;
                    StudyProgressAction::Maintenance { study }
                },
                "waiting" => StudyProgressAction::Waiting,
                _ => return Err(CanonicalError::IncompleteResponse),
            };
            if !row.is_empty() { return Err(codec::CodecError::UnknownFields.into()); }
            Ok(StudyProgress { action, continuation, examined })
        })).await
    }

    /// Resolve prepared seed evidence canonically under the original exact read
    /// set, then claim. Caller retains actual launch resources through this call.
    pub async fn claim_prepared_study_point(
        &self,
        preparation: &StudyPreparation,
        seed: Option<SeedFact>,
        operation: &str,
        worker: &str,
        lifetime: Duration,
    ) -> Result<PreparedStudyClaim, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            if preparation.issuer != self.issuing_store()? || preparation.module != self.study_module_hash()? {
                return Err(CanonicalError::Configuration("study preparation belongs to a different issuing store or policy module".into()));
            }
            let scope = &preparation.scope;
            let lifetime = i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
            if !(1..=86_400_000_000).contains(&lifetime) { return Err(CanonicalError::PayloadLimit); }
            let expected = scope_expected(scope)?;
            let seed = encode(&seed)?;
            bounded(&seed, 16 * 1024)?;
            let request = encode(&(operation, &preparation.issuer, &preparation.module, &scope.study, scope.generation, &scope.point.key, &expected.iter().map(|row| row.to_sql()).collect::<Vec<_>>(), &seed, worker, lifetime))?;
            let claim_operation = format!("{operation}:attempt");
            let attempt = execution_attempt_key(&scope.point.run, &claim_operation);
            let claim_request = encode(&(&scope.point.run, &claim_operation, worker, lifetime, wire::INTERPRETATION))?;
            let response = self.protected_execution_query(&scope.point.run, "canonical_studies::claim_prepared_study_point", || Ok(self.db.query(
                "RETURN fn::pse_study_v1::prepared_claim($pse_rpc_expires_at,$issuer,$module,$study,$generation,$point,$expected,$seed,$operation,$request,$claim_request,$attempt,$worker,$lifetime);")
                .bind(("issuer", preparation.issuer.clone())).bind(("module", preparation.module.clone())).bind(("study", scope.study.clone())).bind(("generation", codec::encode_uint(scope.generation)?))
                .bind(("point", scope.point.key.clone())).bind(("expected", expected.clone())).bind(("seed", Bytes::from(seed.clone()))).bind(("operation", operation.to_owned())).bind(("request", Bytes::from(request.clone())))
                .bind(("claim_request", Bytes::from(claim_request.clone()))).bind(("attempt", attempt.clone())).bind(("worker", worker.to_owned())).bind(("lifetime", lifetime)))).await
                .and_then(|mut response| response.take::<Option<Object>>(0)?.ok_or(CanonicalError::IncompleteResponse));
            #[cfg(any(test, feature = "test-support"))]
            let response = if response.is_ok() && PREPARED_ACK_LOST.try_with(|lost| *lost).unwrap_or(false) {
                Err(CanonicalError::IncompleteResponse)
            } else { response };
            let mut result = match response {
                Ok(result) => result,
                Err(error) => crate::canonical::settlement::settle_exact("study-preparation", operation, &request, Err(error), async {
                    #[cfg(any(test, feature = "test-support"))]
                    if PREPARED_READBACK_FAILED.try_with(|failed| *failed).unwrap_or(false) {
                        return Err(CanonicalError::IncompleteResponse);
                    }
                    let mut response = self.protected_execution_query(&scope.point.run, "canonical_studies::prepared_result", || Ok(self.db.query(
                        "RETURN fn::pse_study_v1::prepared_result($pse_rpc_expires_at,$operation,$request);")
                        .bind(("operation", operation.to_owned())).bind(("request", Bytes::from(request.clone()))))).await?;
                    Ok(response.take::<Option<Object>>(0)?)
                }, |_| true, CanonicalError::source_transaction_refused).await?,
            };
            match codec::decode_string(codec::required(&mut result, "kind")?)?.as_str() {
                "claimed" => {
                    let row = wire::decode_canonical_attempts(readiness_object(codec::required(&mut result, "attempt")?)?)?;
                    Ok(PreparedStudyClaim::Claimed(Box::new(self.finish_study_claim(scope, None, operation, &request, &claim_request, Ok(row)).await?)))
                },
                "settled" => {
                    let point = codec::decode_string(codec::required(&mut result, "point")?)?;
                    if point != scope.point.key { return Err(CanonicalError::IncompleteResponse); }
                    Ok(PreparedStudyClaim::Settled { point })
                },
                "waiting" => Ok(PreparedStudyClaim::Waiting),
                _ => Err(CanonicalError::IncompleteResponse),
            }
        })).await
    }
}
