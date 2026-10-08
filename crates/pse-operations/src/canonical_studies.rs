// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded occurrence storage and scoped decisions over the canonical substrate.

use crate::{
    canonical::{CanonicalError, CanonicalStore, bounded_query, protected_query},
    canonical_codec as codec,
    canonical_execution::{AttemptFence, RunRequest, execution_attempt_key},
    generated::surreal as wire,
};
use pse_model::generated::enums::StudyPointState;
pub use pse_model::generated::runtime::{
    canonical_studies::Row as Study, canonical_study_points::Row as StudyPoint,
};
use pse_model::study::{
    ActionKind, EffectState, OccurrenceGraph, OccurrenceKey, PointFacts, PointOutcome, PointPolicy,
    ScientificFacts, SeedFact, StartPolicy, StartProvenance,
};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};
use surrealdb::types::{Bytes, Object, Value};

/// Actual bounded study discovery fields, without scientific metadata or occurrence descriptors.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudySummary {
    /// Exact retained study key.
    pub key: String,
    /// Exact parent scientific execution.
    pub run: String,
    /// Selected scientific problem.
    pub problem: String,
    /// Exact primary source revision.
    pub revision: String,
    /// Admitted implementation interpretation.
    pub interpretation: String,
    /// Requested occurrence count.
    pub point_count: u64,
    /// Exact immutable ingestion cursor.
    pub next_ordinal: u64,
    /// Complete membership has been activated.
    pub active: bool,
    /// Persistent cancellation authority.
    pub cancelled: bool,
    /// Current scoped study authority generation.
    pub generation: u64,
    /// The owning parent summary has concluded the study.
    pub terminal: bool,
}
impl From<&Study> for StudySummary {
    fn from(study: &Study) -> Self {
        Self {
            key: study.key.clone(),
            run: study.run.clone(),
            problem: study.problem.clone(),
            revision: study.revision.clone(),
            interpretation: study.interpretation.clone(),
            point_count: study.point_count,
            next_ordinal: study.next_ordinal,
            active: study.active,
            cancelled: study.cancelled,
            generation: study.generation,
            terminal: study.terminal,
        }
    }
}
fn decode_study_summary(mut row: Object) -> Result<StudySummary, CanonicalError> {
    use codec::{decode_boolean, decode_string, decode_uint, required};
    let key = decode_string(required(&mut row, "key")?)?;
    let run = decode_string(required(&mut row, "run")?)?;
    let problem = decode_string(required(&mut row, "problem")?)?;
    let revision = decode_string(required(&mut row, "revision")?)?;
    let interpretation = decode_string(required(&mut row, "interpretation")?)?;
    let point_count = decode_uint(required(&mut row, "point_count")?)?;
    let next_ordinal = decode_uint(required(&mut row, "next_ordinal")?)?;
    let active = decode_boolean(required(&mut row, "active")?)?;
    let cancelled = decode_boolean(required(&mut row, "cancelled")?)?;
    let generation = decode_uint(required(&mut row, "generation")?)?;
    let terminal = decode_boolean(required(&mut row, "terminal")?)?;
    if !row.is_empty() {
        return Err(codec::CodecError::UnknownFields.into());
    }
    Ok(StudySummary {
        key,
        run,
        problem,
        revision,
        interpretation,
        point_count,
        next_ordinal,
        active,
        cancelled,
        generation,
        terminal,
    })
}

/// Actual occurrence discovery and decision premises, without immutable operation bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedStudyPoint {
    /// Exact occurrence key.
    pub key: String,
    /// Owning study key.
    pub study: String,
    /// Actual immutable membership position.
    pub ordinal: u64,
    /// Authored occurrence identity.
    pub occurrence: u64,
    /// Original scientific policy bytes.
    pub policy: pse_model::Bytes,
    /// Current scientific facts bytes.
    pub facts: pse_model::Bytes,
    /// Actual retained terminal observation, when present.
    pub outcome: Option<pse_model::Bytes>,
    /// Exact scientific execution key.
    pub run: String,
    /// Monotone concurrency revision.
    pub revision: u64,
    /// An attempt has been assigned.
    pub assigned: bool,
    /// The occurrence has settled.
    pub settled: bool,
    /// Exact current attempt key, when present.
    pub attempt: Option<String>,
    /// Actual admitted start, when present.
    pub start: Option<pse_model::Bytes>,
}
impl ScopedStudyPoint {
    /// Decode the one stored scientific policy.
    pub fn policy(&self) -> Result<PointPolicy, CanonicalError> {
        decode_point_policy(&self.policy, self.occurrence)
    }
    /// Decode facts with their native concurrency revision.
    pub fn facts(&self) -> Result<PointFacts, CanonicalError> {
        decode_point_facts(&self.facts, self.occurrence, self.revision)
    }
    /// Decode the actual retained terminal observation.
    pub fn outcome(&self) -> Result<Option<PointOutcome>, CanonicalError> {
        decode_point_outcome(self.outcome.as_ref())
    }
}
impl From<&StudyPoint> for ScopedStudyPoint {
    fn from(point: &StudyPoint) -> Self {
        Self {
            key: point.key.clone(),
            study: point.study.clone(),
            ordinal: point.ordinal,
            occurrence: point.occurrence,
            policy: point.policy.clone(),
            facts: point.facts.clone(),
            outcome: point.outcome.clone(),
            run: point.run.clone(),
            revision: point.revision,
            assigned: point.assigned,
            settled: point.settled,
            attempt: point.attempt.clone(),
            start: point.start.clone(),
        }
    }
}

/// One bounded occurrence supplied after complete graph/scientific admission.
#[derive(Clone, Debug)]
pub struct NewOccurrence {
    /// Policy is the sole stored authority for dependency and seed meaning.
    pub policy: PointPolicy,
    /// Existing scientific owner encodes the operation and binding, excluding policy.
    pub descriptor: Vec<u8>,
    /// Exact scientific request and provenance recorded before claim.
    pub run: RunRequest,
}

/// Scoped immutable policies and current point premises; private fields prevent
/// substituting revisions between discovery and guarded application.
#[derive(Clone, Debug)]
pub struct StudyScope {
    study: String,
    generation: u64,
    cancelled: bool,
    point: ScopedStudyPoint,
    predecessors: Vec<ScopedStudyPoint>,
}
impl StudyScope {
    /// Requested occurrence metadata; operation bytes are fetched separately.
    pub fn point(&self) -> &ScopedStudyPoint {
        &self.point
    }
    /// Immediate predecessor records, excluding every unrelated occurrence.
    pub fn predecessors(&self) -> &[ScopedStudyPoint] {
        &self.predecessors
    }
    /// Study cancellation is a fenced decision premise.
    pub fn cancelled(&self) -> bool {
        self.cancelled
    }
    /// Evaluate the shared scientific policy with the resolved candidate seed.
    pub fn action(
        &self,
        seed: Option<SeedFact>,
    ) -> Result<pse_model::study::PointAction, CanonicalError> {
        let policy: PointPolicy = decode(self.point.policy.as_slice())?;
        let mut candidate = self.point.facts()?;
        candidate.seed = seed;
        let mut facts = vec![candidate];
        let mut policies = Vec::with_capacity(self.predecessors.len());
        for predecessor in &self.predecessors {
            policies.push(decode(predecessor.policy.as_slice())?);
            facts.push(predecessor.facts()?);
        }
        crate::study_policy::candidate_action(&policy, &policies, &facts, self.cancelled)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))
    }
}

/// Canonical claim and the actual start that the scientific owner must consume.
#[derive(Clone, Debug)]
pub struct StudyClaim {
    /// Acknowledged native attempt fence, before any numerical effect.
    pub fence: AttemptFence,
    /// Point receipt after the guarded decision.
    pub point: StudyPoint,
    /// Actual selected start, not an inferred success label.
    pub start: StartProvenance,
}
impl StudyClaim {
    fn begin_dispatch(&self) -> Result<StudyDispatchAdmission<'_>, CanonicalError> {
        let mut facts = point_facts(&self.point)?;
        if facts.native_started {
            return Err(CanonicalError::Configuration(
                "study occurrence already crossed its native dispatch boundary".into(),
            ));
        }
        facts.revision = 0;
        facts.native_started = true;
        let facts = encode(&facts)?;
        let nonce: pse_ids::SemanticId = crate::mint_id();
        let operation = format!("study-start:{nonce}");
        let request = encode(&(
            &self.point.study,
            &self.point.key,
            self.point.revision,
            self.fence.run(),
            self.fence.attempt(),
            self.fence.generation(),
            &facts,
            &self.start,
        ))?;
        Ok(StudyDispatchAdmission {
            claim: self,
            operation,
            request,
            facts,
        })
    }
}

// Private and noncloneable: only this start invocation knows its fresh operation
// identity. A historical start from another invocation cannot settle this one.
struct StudyDispatchAdmission<'a> {
    claim: &'a StudyClaim,
    operation: String,
    request: Vec<u8>,
    facts: Vec<u8>,
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json::to_vec(value).map_err(|error| CanonicalError::Configuration(error.to_string()))
}
fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CanonicalError> {
    serde_json::from_slice(bytes).map_err(|error| CanonicalError::Configuration(error.to_string()))
}
fn bounded(bytes: &[u8], limit: usize) -> Result<(), CanonicalError> {
    if bytes.len() > limit {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}
/// Exact occurrence key includes study identity, never binding equality.
pub fn point_key(study: &str, key: OccurrenceKey) -> String {
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
    hash.str("pse.study.occurrence.v1")
        .str(study)
        .u64(u64::from(key.0));
    hash.finish_hash().to_hex()
}
/// Decode the policy facts with the native monotone concurrency revision.
pub fn point_facts(point: &StudyPoint) -> Result<PointFacts, CanonicalError> {
    decode_point_facts(&point.facts, point.occurrence, point.revision)
}
fn decode_point_facts(
    bytes: &pse_model::Bytes,
    occurrence: u64,
    revision: u64,
) -> Result<PointFacts, CanonicalError> {
    let mut facts: PointFacts = decode(bytes.as_slice())?;
    facts.revision = revision;
    if u64::from(facts.key.0) != occurrence {
        return Err(CanonicalError::Configuration(
            "study occurrence facts differ".into(),
        ));
    }
    Ok(facts)
}
/// Decode the one stored scientific policy.
pub fn point_policy(point: &StudyPoint) -> Result<PointPolicy, CanonicalError> {
    decode_point_policy(&point.policy, point.occurrence)
}
fn decode_point_policy(
    bytes: &pse_model::Bytes,
    occurrence: u64,
) -> Result<PointPolicy, CanonicalError> {
    let policy: PointPolicy = decode(bytes.as_slice())?;
    if u64::from(policy.key.0) != occurrence {
        return Err(CanonicalError::Configuration(
            "study policy occurrence differs".into(),
        ));
    }
    Ok(policy)
}
/// Decode an explicitly retained terminal observation.
pub fn point_outcome(point: &StudyPoint) -> Result<Option<PointOutcome>, CanonicalError> {
    decode_point_outcome(point.outcome.as_ref())
}
fn decode_point_outcome(
    bytes: Option<&pse_model::Bytes>,
) -> Result<Option<PointOutcome>, CanonicalError> {
    bytes.map(|bytes| decode(bytes.as_slice())).transpose()
}

fn check_created_study(expected: &Study, observed: &Study) -> Result<(), CanonicalError> {
    if expected.key != observed.key
        || expected.run != observed.run
        || expected.problem != observed.problem
        || expected.revision != observed.revision
        || expected.metadata != observed.metadata
        || expected.interpretation != observed.interpretation
        || expected.point_count != observed.point_count
    {
        return Err(CanonicalError::OperationReused);
    }
    Ok(())
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CreationBoundary {
    BatchAppended,
    BeforeActivation,
    ActivationAcknowledged,
}
#[cfg(test)]
tokio::task_local! {
    static CREATION_OBSERVER: Box<dyn Fn(CreationBoundary) + Send + Sync>;
}
#[cfg(test)]
fn creation_observed(boundary: CreationBoundary) {
    let _ = CREATION_OBSERVER.try_with(|observer| observer(boundary));
}

impl CanonicalStore {
    /// Store admitted definitions in bounded batches, then atomically activate complete
    /// membership. Retrying exact identities settles ingestion without executing science.
    /// The narrow caller probe introduces no runtime or data-boundary dependency.
    /// `None` means cancellation stopped inactive ingestion. An activated study is
    /// returned only after exact settlement and durable cancellation when requested.
    pub async fn create_study(
        &self,
        key: &str,
        run: &RunRequest,
        metadata: &[u8],
        points: &[NewOccurrence],
        cancelled: &(impl Fn() -> bool + Sync),
    ) -> Result<Option<Study>, CanonicalError> {
        let row = Study {
            key: key.into(),
            run: run.key.clone(),
            problem: run.revision.problem.clone(),
            revision: run.revision.key.clone(),
            metadata: metadata.to_vec().into(),
            interpretation: wire::INTERPRETATION.into(),
            point_count: points.len() as u64,
            next_ordinal: 0,
            active: false,
            cancelled: false,
            generation: 0,
            terminal: false,
        };
        if cancelled() {
            return self.stop_study_creation(&row, run, points).await;
        }
        if key.is_empty() || points.is_empty() || points.len() > 100_000 {
            return Err(CanonicalError::PayloadLimit);
        }
        bounded(metadata, 128 * 1024)?;
        let graph = OccurrenceGraph {
            points: points.iter().map(|point| point.policy.clone()).collect(),
        };
        crate::study_policy::admit(&graph)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        for point in points {
            if cancelled() {
                return self.stop_study_creation(&row, run, points).await;
            }
            bounded(&point.descriptor, 512 * 1024)?;
            bounded(&encode(&point.policy)?, 16 * 1024)?;
            let mut predecessors: BTreeSet<_> = point
                .policy
                .dependencies
                .iter()
                .map(|edge| edge.predecessor())
                .collect();
            if let StartPolicy::Continuation(edge) = &point.policy.start {
                predecessors.insert(edge.predecessor);
            }
            if predecessors.len() > 64 {
                return Err(CanonicalError::PayloadLimit);
            }
            if point.run.revision != run.revision {
                return Err(CanonicalError::Configuration(
                    "study occurrence revision differs".into(),
                ));
            }
        }
        if cancelled() {
            return self.stop_study_creation(&row, run, points).await;
        }
        self.begin_run(run).await?;
        if cancelled() {
            return self.stop_study_creation(&row, run, points).await;
        }
        let object = wire::encode_canonical_studies(&row)?;
        self.ensure_writes()?;
        protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::create($row);")
                .bind(("row", object.clone())))
        })
        .await?;
        if cancelled() {
            return self.stop_study_creation(&row, run, points).await;
        }
        // Bulk native insertion amortizes round trips while each transaction remains
        // bounded by both occurrence count and encoded descriptor/edge extent.
        let mut batch = Vec::new();
        let mut batch_edges = Vec::new();
        let mut extent = 0_usize;
        for (ordinal, point) in points.iter().enumerate() {
            if cancelled() {
                return self.stop_study_creation(&row, run, points).await;
            }
            let key = point_key(&row.key, point.policy.key);
            let facts = PointFacts {
                key: point.policy.key,
                revision: 0,
                native_started: false,
                lifecycle: StudyPointState::Pending,
                scientific: ScientificFacts::default(),
                attempt_count: 0,
                retry_failure: None,
                effect: EffectState::Absent,
                seed: None,
            };
            let native = StudyPoint {
                key: key.clone(),
                study: row.key.clone(),
                ordinal: ordinal as u64,
                occurrence: u64::from(point.policy.key.0),
                policy: encode(&point.policy)?.into(),
                descriptor: point.descriptor.clone().into(),
                facts: encode(&facts)?.into(),
                outcome: None,
                run: point.run.key.clone(),
                revision: 0,
                assigned: false,
                settled: false,
                attempt: None,
                start: None,
            };
            self.begin_run(&point.run).await?;
            if cancelled() {
                return self.stop_study_creation(&row, run, points).await;
            }
            let mut edges = BTreeMap::new();
            for edge in &point.policy.dependencies {
                edges.insert(
                    (
                        edge.predecessor(),
                        if matches!(edge, pse_model::study::Dependency::UsableResult(_)) {
                            "usable"
                        } else {
                            "ordering"
                        },
                    ),
                    (),
                );
            }
            if let StartPolicy::Continuation(edge) = &point.policy.start {
                edges.insert((edge.predecessor, "seed"), ());
            }
            let edges = edges
                .into_keys()
                .map(|(predecessor, kind)| {
                    let target = point_key(&row.key, predecessor);
                    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
                    hash.str("pse.study.edge.v1")
                        .str(&key)
                        .str(&target)
                        .str(kind);
                    wire::encode_canonical_study_dependencies(
                        &pse_model::generated::runtime::canonical_study_dependencies::Row {
                            key: hash.finish_hash().to_hex(),
                            study: row.key.clone(),
                            dependent: key.clone(),
                            predecessor: target,
                            kind: kind.into(),
                        },
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let point_extent = native.descriptor.len()
                + native.policy.len()
                + native.facts.len()
                + 4096
                + edges.len() * 1024;
            if !batch.is_empty() && (batch.len() == 64 || extent + point_extent > 2 * 1024 * 1024) {
                if cancelled() {
                    return self.stop_study_creation(&row, run, points).await;
                }
                self.append_study_batch(&row.key, &batch, &batch_edges)
                    .await?;
                #[cfg(test)]
                creation_observed(CreationBoundary::BatchAppended);
                if cancelled() {
                    return self.stop_study_creation(&row, run, points).await;
                }
                batch.clear();
                batch_edges.clear();
                extent = 0;
            }
            extent += point_extent;
            batch.push(wire::encode_canonical_study_points(&native)?);
            batch_edges.extend(edges);
        }
        if !batch.is_empty() {
            if cancelled() {
                return self.stop_study_creation(&row, run, points).await;
            }
            self.append_study_batch(&row.key, &batch, &batch_edges)
                .await?;
            #[cfg(test)]
            creation_observed(CreationBoundary::BatchAppended);
        }
        #[cfg(test)]
        creation_observed(CreationBoundary::BeforeActivation);
        if cancelled() {
            return self.stop_study_creation(&row, run, points).await;
        }
        // Do not race this future against cancellation. Once issued, activation
        // must be settled by exact immutable identity before the caller can leave.
        let response = self.send_study_activation(&row.key).await;
        self.finish_study_activation(&row, response, cancelled)
            .await
    }

    async fn stop_study_creation(
        &self,
        expected: &Study,
        run: &RunRequest,
        points: &[NewOccurrence],
    ) -> Result<Option<Study>, CanonicalError> {
        let Some(study) = self.canonical_study(&expected.key).await? else {
            return Ok(None);
        };
        check_created_study(expected, &study)?;
        if !study.active {
            return Ok(None);
        }
        if points.is_empty() || points.len() > 100_000 {
            return Err(CanonicalError::PayloadLimit);
        }
        if study.next_ordinal != study.point_count {
            return Err(CanonicalError::IncompleteResponse);
        }
        // Prove every immutable member with bounded point/run reads before
        // revoking this active study; checking a retry never publishes a run.
        self.check_execution_run_identity(run).await?;
        for (ordinal, point) in points.iter().enumerate() {
            bounded(&point.descriptor, 512 * 1024)?;
            let policy = encode(&point.policy)?;
            bounded(&policy, 16 * 1024)?;
            let key = point_key(&expected.key, point.policy.key);
            let saved = self
                .canonical_study_point(&key)
                .await?
                .ok_or(CanonicalError::OperationReused)?;
            if saved.key != key
                || saved.study != expected.key
                || saved.ordinal != ordinal as u64
                || saved.occurrence != u64::from(point.policy.key.0)
                || saved.descriptor.as_slice() != point.descriptor
                || saved.policy.as_slice() != policy
                || saved.run != point.run.key
            {
                return Err(CanonicalError::OperationReused);
            }
            self.check_execution_run_identity(&point.run).await?;
        }
        self.cancel_study(&expected.key).await.map(Some)
    }

    async fn send_study_activation(&self, key: &str) -> Result<Study, CanonicalError> {
        let mut response = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::activate($study);")
                .bind(("study", key.to_owned())))
        })
        .await?;
        Ok(wire::decode_canonical_studies(
            response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }

    // Activation has no fresh scientific operation: exact immutable membership
    // and the native study guard are its authority. Cancellation touches that same
    // guard and refuses a subsequently arriving activation of an inactive header.
    async fn finish_study_activation(
        &self,
        expected: &Study,
        response: Result<Study, CanonicalError>,
        cancelled: &(impl Fn() -> bool + Sync),
    ) -> Result<Option<Study>, CanonicalError> {
        #[cfg(test)]
        creation_observed(CreationBoundary::ActivationAcknowledged);
        let study =
            match response {
                Ok(study) => study,
                Err(error) => {
                    if cancelled() {
                        let study = self.cancel_study(&expected.key).await?;
                        check_created_study(expected, &study)?;
                        return Ok(study.active.then_some(study));
                    }
                    let study = self.canonical_study(&expected.key).await.map_err(|settlement| {
                    CanonicalError::Configuration(format!(
                        "study {} activation requires exact settlement: {error}; {settlement}",
                        expected.key))
                })?.ok_or(CanonicalError::IncompleteResponse)?;
                    check_created_study(expected, &study)?;
                    if !study.active {
                        // An inactive read cannot prove an issued request did not commit.
                        // Fence it before returning its original failure.
                        self.cancel_study(&expected.key).await?;
                        return Err(error);
                    }
                    study
                }
            };
        check_created_study(expected, &study)?;
        if !study.active || study.next_ordinal != study.point_count {
            return Err(CanonicalError::IncompleteResponse);
        }
        if cancelled() {
            return self.cancel_study(&expected.key).await.map(Some);
        }
        Ok(Some(study))
    }

    async fn append_study_batch(
        &self,
        study: &str,
        points: &[Object],
        edges: &[Object],
    ) -> Result<(), CanonicalError> {
        protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::append($study,$points,$edges);")
                .bind(("study", study.to_owned()))
                .bind(("points", points.to_vec()))
                .bind(("edges", edges.to_vec())))
        })
        .await?;
        Ok(())
    }
    /// Read a study header without its occurrence graph.
    pub async fn canonical_study(&self, key: &str) -> Result<Option<Study>, CanonicalError> {
        let mut result = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_studies',$key);")
                .bind(("key", key.to_owned())),
        )
        .await?;
        result
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_studies)
            .transpose()
            .map_err(Into::into)
    }
    /// Read one requested occurrence's immutable operation and current observation.
    pub async fn canonical_study_point(
        &self,
        key: &str,
    ) -> Result<Option<StudyPoint>, CanonicalError> {
        let mut result = bounded_query(
            self.db
                .query("SELECT * FROM ONLY type::record('canonical_study_points',$key);")
                .bind(("key", key.to_owned())),
        )
        .await?;
        result
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_study_points)
            .transpose()
            .map_err(Into::into)
    }
    /// Narrow ordered discovery. Discovery never grants a claim.
    pub async fn study_candidates(
        &self,
        study: &str,
        after: Option<u64>,
    ) -> Result<Vec<ScopedStudyPoint>, CanonicalError> {
        let mut result=bounded_query(self.db.query("SELECT key,study,ordinal,occurrence,policy,facts,outcome,run,revision,assigned,settled,attempt,start FROM canonical_study_points WHERE study=$study AND settled=false AND assigned=false AND ordinal > $after ORDER BY ordinal LIMIT 64;").bind(("study",study.to_owned())).bind(("after",after.map(codec::encode_uint).transpose()?.unwrap_or(codec::encode_int(-1)?)))).await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(decode_scope_point)
            .collect()
    }
    /// Bounded explicit occurrence page, including history for user summaries.
    pub async fn study_point_page(
        &self,
        study: &str,
        after: Option<u64>,
    ) -> Result<Vec<ScopedStudyPoint>, CanonicalError> {
        let mut result=bounded_query(self.db.query("SELECT key,study,ordinal,occurrence,policy,facts,outcome,run,revision,assigned,settled,attempt,start FROM canonical_study_points WHERE study=$study AND ordinal > $after ORDER BY ordinal LIMIT 64;").bind(("study",study.to_owned())).bind(("after",after.map(codec::encode_uint).transpose()?.unwrap_or(codec::encode_int(-1)?)))).await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(decode_scope_point)
            .collect()
    }
    /// One snapshot supplies only the candidate and immediate decision premises.
    pub async fn study_scope(&self, point: &str) -> Result<StudyScope, CanonicalError> {
        let mut result = bounded_query(
            self.db
                .query("RETURN fn::pse_study_v1::scope($point);")
                .bind(("point", point.to_owned())),
        )
        .await?;
        let mut object = result
            .take::<Option<Object>>(0)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        let Value::Object(mut study) = codec::required(&mut object, "study")? else {
            return Err(CanonicalError::IncompleteResponse);
        };
        let key = codec::decode_string(codec::required(&mut study, "key")?)?;
        let generation = codec::decode_uint(codec::required(&mut study, "generation")?)?;
        let cancelled = codec::decode_boolean(codec::required(&mut study, "cancelled")?)?;
        let Value::Object(point) = codec::required(&mut object, "point")? else {
            return Err(CanonicalError::IncompleteResponse);
        };
        let point = decode_scope_point(point)?;
        let Value::Array(rows) = codec::required(&mut object, "predecessors")? else {
            return Err(CanonicalError::IncompleteResponse);
        };
        let predecessors = rows
            .into_iter()
            .map(|row| {
                let Value::Object(row) = row else {
                    return Err(CanonicalError::IncompleteResponse);
                };
                decode_scope_point(row)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if predecessors.len() > 64 {
            return Err(CanonicalError::PayloadLimit);
        }
        Ok(StudyScope {
            study: key,
            generation,
            cancelled,
            point,
            predecessors,
        })
    }

    /// Recompute shared policy then atomically fence every consumed premise and claim.
    pub async fn claim_study_point(
        &self,
        scope: &StudyScope,
        seed: Option<SeedFact>,
        operation: &str,
        worker: &str,
        lifetime: Duration,
    ) -> Result<StudyClaim, CanonicalError> {
        let action = scope.action(seed.clone())?;
        let ActionKind::Start(start) = action.kind else {
            return Err(CanonicalError::Configuration(
                "study policy did not authorize a start".into(),
            ));
        };
        let lifetime =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if !(1..=86_400_000_000).contains(&lifetime) {
            return Err(CanonicalError::PayloadLimit);
        }
        let attempt = execution_attempt_key(&scope.point.run, &format!("{operation}:attempt"));
        let mut facts = scope.point.facts()?;
        facts.revision = 0;
        facts.seed = seed;
        facts.attempt_count = facts
            .attempt_count
            .checked_add(1)
            .ok_or(CanonicalError::PayloadLimit)?;
        facts.lifecycle = StudyPointState::Assigned;
        // Native-start is a fact of this actual attempt. A policy-authorized retry
        // keeps prior outcomes but has not crossed its own dispatch boundary yet.
        facts.native_started = false;
        let facts = encode(&facts)?;
        let expected = std::iter::once(&scope.point)
            .chain(scope.predecessors.iter())
            .map(|point| {
                let mut object = Object::new();
                object.insert("key", codec::encode_string(point.key.clone())?);
                object.insert("revision", codec::encode_uint(point.revision)?);
                Ok(object)
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        let start_bytes = encode(&start)?;
        let request = encode(&(
            operation,
            &scope.study,
            scope.generation,
            &scope.point.key,
            scope.point.revision,
            &start,
            worker,
            lifetime,
        ))?;
        let claim_operation = format!("{operation}:attempt");
        let claim_request = encode(&(
            &scope.point.run,
            &claim_operation,
            worker,
            lifetime,
            wire::INTERPRETATION,
        ))?;
        self.ensure_writes()?;
        let response=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::claim($study,$generation,$point,$expected,$start,$operation,$request,$claim_request,$attempt,$worker,$lifetime,$facts);").bind(("study",scope.study.clone())).bind(("generation",codec::encode_uint(scope.generation)?)).bind(("point",scope.point.key.clone())).bind(("expected",expected.clone())).bind(("start",Bytes::from(start_bytes.clone()))).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("claim_request",Bytes::from(claim_request.clone()))).bind(("attempt",attempt.clone())).bind(("worker",worker.to_owned())).bind(("lifetime",lifetime)).bind(("facts",Bytes::from(facts.clone()))))).await;
        let response = response.and_then(|mut response| {
            Ok(wire::decode_canonical_attempts(
                response
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?)
        });
        self.finish_study_claim(scope, start, operation, &request, &claim_request, response)
            .await
    }

    // The operation receipt is the sole acknowledgment authority. A readable point
    // alone cannot establish that this exact decision, worker and lease committed.
    async fn finish_study_claim(
        &self,
        scope: &StudyScope,
        start: StartProvenance,
        operation: &str,
        request: &[u8],
        claim_request: &[u8],
        response: Result<crate::canonical_execution::CanonicalAttempt, CanonicalError>,
    ) -> Result<StudyClaim, CanonicalError> {
        let row = match response {
            Ok(row) => row,
            Err(error) => match self
                .settle_operation(operation, "study-claim", request)
                .await?
            {
                Some(key) => self.canonical_attempt(&key).await?.ok_or(error)?,
                None => return Err(error),
            },
        };
        let expected_attempt =
            execution_attempt_key(&scope.point.run, &format!("{operation}:attempt"));
        if row.key != expected_attempt
            || row.run != scope.point.run
            || row.claim_operation != format!("{operation}:attempt")
            || row.request.as_slice() != claim_request
        {
            return Err(CanonicalError::OperationReused);
        }
        let fence = AttemptFence::from_row(row)?;
        let point = self
            .canonical_study_point(&scope.point.key)
            .await?
            .ok_or(CanonicalError::IncompleteResponse)?;
        if point.study != scope.study
            || point.run != scope.point.run
            || point.occurrence != scope.point.occurrence
            || !point.assigned
            || point.settled
            || point.revision
                != scope
                    .point
                    .revision
                    .checked_add(1)
                    .ok_or(CanonicalError::PayloadLimit)?
            || point.attempt.as_deref() != Some(fence.attempt())
            || point.start.as_ref().map(|bytes| bytes.as_slice())
                != Some(encode(&start)?.as_slice())
            || point_facts(&point)?.native_started
        {
            return Err(CanonicalError::Configuration(
                "study claim acknowledgment no longer names the pre-dispatch occurrence".into(),
            ));
        }
        Ok(StudyClaim {
            fence,
            point,
            start,
        })
    }
    /// Effect-free refusal/cancellation fences the same complete immediate read set
    /// as a claim. It cannot assert native execution or scientific usability.
    pub async fn refuse_study_point(
        &self,
        scope: &StudyScope,
        action: &pse_model::study::PointAction,
        diagnostic: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    ) -> Result<StudyPoint, CanonicalError> {
        if action.occurrence.0 as u64 != scope.point.occurrence
            || action.expected_revision != scope.point.revision
            || !matches!(action.kind, ActionKind::Refuse(_) | ActionKind::Cancel)
        {
            return Err(CanonicalError::Configuration(
                "effect-free settlement must name a scoped refusal or cancellation".into(),
            ));
        }
        let mut facts = scope.point.facts()?;
        if facts.native_started || scope.point.assigned {
            return Err(CanonicalError::Configuration(
                "native attempt needs terminal observation".into(),
            ));
        }
        facts.revision = 0;
        facts.lifecycle = if matches!(action.kind, ActionKind::Cancel) {
            StudyPointState::Cancelled
        } else {
            StudyPointState::Failed
        };
        facts.scientific = ScientificFacts::default();
        facts.effect = EffectState::Absent;
        facts.retry_failure = Some(pse_model::study::RetryFailure::Deterministic);
        let mut outcome = scope.point.outcome()?.unwrap_or(PointOutcome {
            key: facts.key,
            lifecycle: facts.lifecycle,
            scientific: facts.scientific.clone(),
            diagnostic: None,
            start: None,
            effect: facts.effect,
            attempts: vec![],
        });
        outcome.lifecycle = facts.lifecycle;
        outcome.scientific = facts.scientific.clone();
        outcome.effect = facts.effect;
        outcome.diagnostic = diagnostic;
        let expected = scope_expected(scope)?;
        let facts = encode(&facts)?;
        let outcome = encode(&outcome)?;
        self.ensure_writes()?;
        let mut result=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::settle($study,$generation,$cancelled,$point,$expected,$facts,$outcome);").bind(("study",scope.study.clone())).bind(("generation",codec::encode_uint(scope.generation)?)).bind(("cancelled",scope.cancelled)).bind(("point",scope.point.key.clone())).bind(("expected",expected.clone())).bind(("facts",Bytes::from(facts.clone()))).bind(("outcome",Bytes::from(outcome.clone()))))).await?;
        Ok(wire::decode_canonical_study_points(
            result
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }
    /// Record the exact pre-dispatch boundary under current cancellation and attempt
    /// authority. Each invocation owns a fresh exact receipt; only that receipt may
    /// settle an uncertain response. A claim alone is never reported as a native start.
    pub async fn mark_study_started(
        &self,
        claim: &StudyClaim,
    ) -> Result<StudyPoint, CanonicalError> {
        self.ensure_writes()?;
        let admission = claim.begin_dispatch()?;
        let result = self.send_study_started(&admission).await;
        self.finish_study_started(admission, result).await
    }

    async fn send_study_started(
        &self,
        admission: &StudyDispatchAdmission<'_>,
    ) -> Result<StudyPoint, CanonicalError> {
        let claim = admission.claim;
        let result=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::started($study,$point,$revision,$run,$attempt,$generation,$operation,$request,$facts);").bind(("study",claim.point.study.clone())).bind(("point",claim.point.key.clone())).bind(("revision",codec::encode_uint(claim.point.revision)?)).bind(("run",claim.fence.run().to_owned())).bind(("attempt",claim.fence.attempt().to_owned())).bind(("generation",codec::encode_uint(claim.fence.generation())?)).bind(("operation",admission.operation.clone())).bind(("request",Bytes::from(admission.request.clone()))).bind(("facts",Bytes::from(admission.facts.clone()))))).await;
        let mut result = result?;
        Ok(wire::decode_canonical_study_points(
            result
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }

    async fn finish_study_started(
        &self,
        admission: StudyDispatchAdmission<'_>,
        response: Result<StudyPoint, CanonicalError>,
    ) -> Result<StudyPoint, CanonicalError> {
        let claim = admission.claim;
        let point = match response {
            Ok(point) => point,
            Err(error) => {
                // A definite rejection of a repeated start must not authorize a
                // second dispatch merely because the first boundary is readable.
                let uncertain = matches!(&error, CanonicalError::Timeout | CanonicalError::IncompleteResponse)
                    // gRPC DeadlineExceeded has an unstructured query category;
                    // only this invocation's committed receipt can resolve it.
                    || matches!(&error, CanonicalError::Driver(error) if error.is_connection()
                        || (error.is_query() && error.query_details().is_none()));
                if !uncertain {
                    return Err(error);
                }
                match self
                    .settle_operation(&admission.operation, "study-start", &admission.request)
                    .await?
                {
                    Some(attempt) if attempt == claim.fence.attempt() => (),
                    Some(_) => return Err(CanonicalError::OperationReused),
                    None => return Err(error),
                }
                // Fence the readback against cancellation, replacement and expiry;
                // do not repeat the start transition or authorize another dispatch.
                let mut response = protected_query(|| Ok(self.db.query(
                    "BEGIN; fn::pse_execution_v1::fence($run,$attempt,$generation); SELECT * FROM ONLY type::record('canonical_study_points',$point); COMMIT;"
                ).bind(("run", claim.fence.run().to_owned()))
                    .bind(("attempt", claim.fence.attempt().to_owned()))
                    .bind(("generation", codec::encode_uint(claim.fence.generation())?))
                    .bind(("point", claim.point.key.clone())))).await?;
                let index = response.num_statements().saturating_sub(2);
                let Some(row) = response.take::<Option<Object>>(index)? else {
                    return Err(error);
                };
                wire::decode_canonical_study_points(row)?
            }
        };
        if point.study != claim.point.study
            || point.run != claim.fence.run()
            || point.revision
                != claim
                    .point
                    .revision
                    .checked_add(1)
                    .ok_or(CanonicalError::PayloadLimit)?
            || point.attempt.as_deref() != Some(claim.fence.attempt())
            || !point.assigned
            || point.settled
            || point.start != claim.point.start
            || point.facts.as_slice() != admission.facts.as_slice()
        {
            return Err(CanonicalError::Configuration(
                "study start acknowledgment does not match the exact admitted dispatch".into(),
            ));
        }
        Ok(point)
    }
    /// Exact indexed study parent association used by effect-free summary recovery.
    pub async fn canonical_study_for_run(
        &self,
        run: &str,
    ) -> Result<Option<Study>, CanonicalError> {
        let mut result = bounded_query(
            self.db
                .query("SELECT * FROM canonical_studies WHERE run=$run LIMIT 1;")
                .bind(("run", run.to_owned())),
        )
        .await?;
        let mut rows = result.take::<Vec<Object>>(0)?;
        rows.pop()
            .map(wire::decode_canonical_studies)
            .transpose()
            .map_err(Into::into)
    }
    /// Assign the effect-free parent summary only after all occurrences are settled.
    pub async fn begin_study_finalization(
        &self,
        study: &StudySummary,
        worker: &str,
        lifetime: Duration,
    ) -> Result<Option<AttemptFence>, CanonicalError> {
        let parent = self
            .canonical_run(&study.run)
            .await?
            .ok_or(CanonicalError::IncompleteResponse)?;
        let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
        identity
            .str("pse.study.summary-claim.v1")
            .str(&study.key)
            .str(&study.run)
            .str(worker)
            .u64(parent.current_generation);
        let operation = identity.finish_hash().to_hex();
        let attempt = execution_attempt_key(&study.run, &operation);
        let lifetime =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if !(1..=86_400_000_000).contains(&lifetime) {
            return Err(CanonicalError::PayloadLimit);
        }
        let request = encode(&(
            &study.run,
            &operation,
            worker,
            lifetime,
            wire::INTERPRETATION,
        ))?;
        self.ensure_writes()?;
        let result=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::finalize($study,$operation,$request,$attempt,$worker,$lifetime);").bind(("study",study.key.clone())).bind(("operation",operation.clone())).bind(("request",Bytes::from(request.clone()))).bind(("attempt",attempt.clone())).bind(("worker",worker.to_owned())).bind(("lifetime",lifetime)))).await;
        let result = result.and_then(|mut result| {
            result
                .take::<Option<Object>>(0)?
                .map(wire::decode_canonical_attempts)
                .transpose()
                .map_err(Into::into)
        });
        self.finish_study_finalization(&study.run, &operation, &request, result)
            .await
    }

    async fn finish_study_finalization(
        &self,
        run: &str,
        operation: &str,
        request: &[u8],
        response: Result<Option<crate::canonical_execution::CanonicalAttempt>, CanonicalError>,
    ) -> Result<Option<AttemptFence>, CanonicalError> {
        let row = match response {
            Ok(row) => row,
            Err(error) => match self.settle_operation(operation, "claim", request).await? {
                Some(key) => Some(self.canonical_attempt(&key).await?.ok_or(error)?),
                None => return Err(error),
            },
        };
        row.map(|row| {
            if row.run != run
                || row.key != execution_attempt_key(run, operation)
                || row.claim_operation != operation
                || row.request.as_slice() != request
            {
                return Err(CanonicalError::OperationReused);
            }
            AttemptFence::from_row(row)
        })
        .transpose()
    }
    /// Assigned occurrences are independently discoverable for explicit lost-worker
    /// recovery; no complete study graph or immutable descriptors are returned.
    pub async fn study_assigned_page(
        &self,
        study: &str,
        after: Option<u64>,
    ) -> Result<Vec<ScopedStudyPoint>, CanonicalError> {
        let mut result=bounded_query(self.db.query("SELECT key,study,ordinal,occurrence,policy,facts,outcome,run,revision,assigned,settled,attempt,start FROM canonical_study_points WHERE study=$study AND settled=false AND assigned=true AND ordinal>$after ORDER BY ordinal LIMIT 64;").bind(("study",study.to_owned())).bind(("after",after.map(codec::encode_uint).transpose()?.unwrap_or(codec::encode_int(-1)?)))).await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(decode_scope_point)
            .collect()
    }
    /// Record only observations of the exact admitted attempt; no raw successful flag.
    /// # Safety
    /// Facts and outcome must be the owning scientific kernel's actual admitted
    /// observations of this exact terminal attempt, or its effect-free policy refusal.
    /// Chosen DTO bytes do not grant scientific predecessor permission.
    #[allow(
        unsafe_code,
        reason = "explicit scientific observation authority, without pointer or ABI operations"
    )]
    pub async unsafe fn observe_study_point(
        &self,
        point: &StudyPoint,
        facts: &PointFacts,
        outcome: &PointOutcome,
        settled: bool,
    ) -> Result<StudyPoint, CanonicalError> {
        if facts.key != outcome.key || u64::from(facts.key.0) != point.occurrence {
            return Err(CanonicalError::Configuration(
                "study observation occurrence differs".into(),
            ));
        }
        let mut facts = facts.clone();
        facts.revision = 0;
        let facts = encode(&facts)?;
        bounded(&facts, 16 * 1024)?;
        let outcome = encode(outcome)?;
        bounded(&outcome, 128 * 1024)?;
        self.ensure_writes()?;
        let mut response=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::observe($point,$revision,$attempt,$facts,$outcome,$settled);").bind(("point",point.key.clone())).bind(("revision",codec::encode_uint(point.revision)?)).bind(("attempt",point.attempt.clone())).bind(("facts",Bytes::from(facts.clone()))).bind(("outcome",Bytes::from(outcome.clone()))).bind(("settled",settled)))).await?;
        Ok(wire::decode_canonical_study_points(
            response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }
    /// Revoke future claims; supervisor still cancels/drains each assigned native attempt.
    pub async fn cancel_study(&self, key: &str) -> Result<Study, CanonicalError> {
        self.ensure_writes()?;
        let result = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::cancel($study);")
                .bind(("study", key.to_owned())))
        })
        .await;
        let response = result.and_then(|mut result| {
            Ok(wire::decode_canonical_studies(
                result
                    .take::<Option<Object>>(0)?
                    .ok_or(CanonicalError::IncompleteResponse)?,
            )?)
        });
        self.finish_study_cancel(key, response).await
    }
    async fn finish_study_cancel(
        &self,
        key: &str,
        response: Result<Study, CanonicalError>,
    ) -> Result<Study, CanonicalError> {
        let study = match response {
            Ok(study) => study,
            Err(error) => self
                .canonical_study(key)
                .await
                .map_err(|settlement| {
                    CanonicalError::Configuration(format!(
                        "study {key} cancellation requires exact settlement: {error}; {settlement}"
                    ))
                })?
                .filter(|study| study.cancelled)
                .ok_or(error)?,
        };
        if study.key != key || !study.cancelled {
            return Err(CanonicalError::IncompleteResponse);
        }
        Ok(study)
    }
    /// Conclude only after native scoped observations settle every admitted occurrence.
    pub async fn conclude_study(&self, key: &str) -> Result<Study, CanonicalError> {
        self.ensure_writes()?;
        let mut result = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::conclude($study);")
                .bind(("study", key.to_owned())))
        })
        .await?;
        Ok(wire::decode_canonical_studies(
            result
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
    }

    /// Active metadata pages for managed workers; no definition/result hydration.
    pub async fn study_page(
        &self,
        after: Option<&str>,
    ) -> Result<Vec<StudySummary>, CanonicalError> {
        let mut result=bounded_query(self.db.query("SELECT key,run,problem,revision,interpretation,point_count,next_ordinal,active,cancelled,generation,terminal FROM canonical_studies WHERE active=true AND terminal=false AND key>$after ORDER BY key LIMIT 64;").bind(("after",after.unwrap_or("").to_owned()))).await?;
        result
            .take::<Vec<Object>>(0)?
            .into_iter()
            .map(decode_study_summary)
            .collect()
    }
}

fn scope_expected(scope: &StudyScope) -> Result<Vec<Object>, CanonicalError> {
    std::iter::once(&scope.point)
        .chain(scope.predecessors.iter())
        .map(|point| {
            let mut object = Object::new();
            object.insert("key", codec::encode_string(point.key.clone())?);
            object.insert("revision", codec::encode_uint(point.revision)?);
            Ok(object)
        })
        .collect()
}

fn decode_scope_point(mut row: Object) -> Result<ScopedStudyPoint, CanonicalError> {
    use codec::{decode_boolean, decode_bytes, decode_string, decode_uint, required};
    let point = ScopedStudyPoint {
        key: decode_string(required(&mut row, "key")?)?,
        study: decode_string(required(&mut row, "study")?)?,
        ordinal: decode_uint(required(&mut row, "ordinal")?)?,
        occurrence: decode_uint(required(&mut row, "occurrence")?)?,
        policy: decode_bytes(required(&mut row, "policy")?)?,
        facts: decode_bytes(required(&mut row, "facts")?)?,
        outcome: match row.remove("outcome") {
            None | Some(Value::None) => None,
            Some(value) => Some(decode_bytes(value)?),
        },
        run: decode_string(required(&mut row, "run")?)?,
        revision: decode_uint(required(&mut row, "revision")?)?,
        assigned: decode_boolean(required(&mut row, "assigned")?)?,
        settled: decode_boolean(required(&mut row, "settled")?)?,
        attempt: match row.remove("attempt") {
            None | Some(Value::None) => None,
            Some(value) => Some(decode_string(value)?),
        },
        start: match row.remove("start") {
            None | Some(Value::None) => None,
            Some(value) => Some(decode_bytes(value)?),
        },
    };
    if !row.is_empty() {
        return Err(codec::CodecError::UnknownFields.into());
    }
    Ok(point)
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    unsafe_code,
    reason = "controlled scientific fixture admission without pointer or ABI operations"
)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unreachable,
    reason = "isolated study fixtures and exhaustive test oracles fail on unexpected results"
)]
mod canonical_studies_server_unit {
    use super::*;
    use crate::canonical::{CanonicalOptions, checked};
    use crate::canonical_execution::TerminalClass;
    use pse_model::study::{Dependency, SeedNeed};

    async fn fixture() -> (CanonicalStore, String, crate::canonical::Revision) {
        let state = std::env::var("PSE_SURREAL_STATE").expect("explicit isolated server fixture");
        let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_studies_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let revision = store.edit("problem", None, "source", &[]).await.unwrap();
        (store, options.database, revision)
    }
    fn request(revision: &crate::canonical::Revision, key: &str) -> RunRequest {
        RunRequest {
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        }
    }
    fn occurrence(
        revision: &crate::canonical::Revision,
        key: u32,
        dependencies: Vec<Dependency>,
    ) -> NewOccurrence {
        NewOccurrence {
            policy: PointPolicy {
                key: OccurrenceKey(key),
                dependencies,
                start: StartPolicy::Fresh,
                seed_need: SeedNeed::NotNeeded,
                attempt_limit: 1,
            },
            descriptor: vec![42],
            run: request(revision, &format!("point-run-{key}")),
        }
    }
    fn observation(point: &StudyPoint, state: StudyPointState) -> (PointFacts, PointOutcome) {
        let mut facts = point_facts(point).unwrap();
        facts.lifecycle = state;
        facts.native_started = point.assigned;
        let outcome = PointOutcome {
            key: facts.key,
            lifecycle: state,
            scientific: facts.scientific.clone(),
            diagnostic: None,
            start: point
                .start
                .as_ref()
                .map(|bytes| decode(bytes.as_slice()).unwrap()),
            effect: EffectState::Absent,
            attempts: vec![],
        };
        (facts, outcome)
    }
    async fn fail(store: &CanonicalStore, claim: &StudyClaim) -> StudyPoint {
        let closed = store
            .close_result_ingestion(&claim.fence, "close-predecessor")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture owns the exact empty attempt and admits a failed outcome with no effects.
        unsafe {
            store
                .seal_attempt(&manifest, "seal-predecessor", TerminalClass::Failed, &[])
                .await
        }
        .unwrap();
        let (facts, outcome) = observation(&claim.point, StudyPointState::Failed);
        // SAFETY: the just-sealed failed receipt backs these exact fixture-owned observations and absent effect.
        unsafe {
            store
                .observe_study_point(&claim.point, &facts, &outcome, true)
                .await
        }
        .unwrap()
    }

    #[tokio::test]
    async fn study_committed_claim_start_and_summary_lost_ack_settle_exact_identity() {
        let (store, database, revision) = fixture().await;
        store
            .create_study(
                "lost-ack",
                &request(&revision, "parent"),
                &[9],
                &[occurrence(&revision, 0, vec![])],
                &|| false,
            )
            .await
            .unwrap();
        let scope = store
            .study_scope(&point_key("lost-ack", OccurrenceKey(0)))
            .await
            .unwrap();
        let duplicate_scope = store
            .study_scope(&point_key("lost-ack", OccurrenceKey(0)))
            .await
            .unwrap();
        let operation = "claim-lost-ack";
        let lifetime = Duration::from_secs(60);
        let mut claim = store
            .claim_study_point(&scope, None, operation, "worker", lifetime)
            .await
            .unwrap();
        let request = encode(&(
            operation,
            &scope.study,
            scope.generation,
            &scope.point.key,
            scope.point.revision,
            &claim.start,
            "worker",
            60_000_000_i64,
        ))
        .unwrap();
        let attempt = store
            .canonical_attempt(claim.fence.attempt())
            .await
            .unwrap()
            .unwrap();
        let recovered = store
            .finish_study_claim(
                &scope,
                claim.start.clone(),
                operation,
                &request,
                attempt.request.as_slice(),
                Err(CanonicalError::Timeout),
            )
            .await
            .unwrap();
        assert_eq!(recovered.fence.attempt(), claim.fence.attempt());
        assert_eq!(recovered.fence.generation(), claim.fence.generation());
        assert_eq!(recovered.point, claim.point);
        assert_eq!(recovered.start, claim.start);
        assert!(matches!(
            store
                .finish_study_claim(
                    &scope,
                    claim.start.clone(),
                    operation,
                    b"another request",
                    attempt.request.as_slice(),
                    Err(CanonicalError::Timeout)
                )
                .await,
            Err(CanonicalError::OperationReused)
        ));
        assert!(matches!(
            store
                .finish_study_claim(
                    &scope,
                    claim.start.clone(),
                    "not-committed",
                    &request,
                    attempt.request.as_slice(),
                    Err(CanonicalError::Timeout)
                )
                .await,
            Err(CanonicalError::Timeout)
        ));

        // Independently replay the same admitted claim before either dispatch.
        // These are distinct objects, not merely clones sharing local state.
        let duplicate = store
            .claim_study_point(&duplicate_scope, None, operation, "worker", lifetime)
            .await
            .unwrap();
        let admission = claim.begin_dispatch().unwrap();
        let started = store.send_study_started(&admission).await.unwrap();
        let duplicate_admission = duplicate.begin_dispatch().unwrap();
        assert_ne!(admission.operation, duplicate_admission.operation);
        assert!(
            store
                .send_study_started(&duplicate_admission)
                .await
                .is_err()
        );
        assert!(
            matches!(
                store
                    .finish_study_started(duplicate_admission, Err(CanonicalError::Timeout))
                    .await,
                Err(CanonicalError::Timeout)
            ),
            "losing the second rejection cannot settle the first committed start"
        );
        // A caller can refresh the public revision while keeping old pre-start
        // facts. The server's per-attempt receipt must independently refuse it.
        let mut false_refresh = duplicate.clone();
        false_refresh.point.revision = started.revision;
        let false_admission = false_refresh.begin_dispatch().unwrap();
        assert!(store.send_study_started(&false_admission).await.is_err());
        assert!(matches!(
            store
                .finish_study_started(false_admission, Err(CanonicalError::Timeout))
                .await,
            Err(CanonicalError::Timeout)
        ));
        assert_eq!(
            store
                .finish_study_started(admission, Err(CanonicalError::Timeout))
                .await
                .unwrap(),
            started
        );
        assert!(store.mark_study_started(&claim.clone()).await.is_err());
        assert!(
            store
                .finish_study_claim(
                    &scope,
                    claim.start.clone(),
                    operation,
                    &request,
                    attempt.request.as_slice(),
                    Err(CanonicalError::Timeout)
                )
                .await
                .is_err(),
            "a committed start cannot be recovered as a second pre-dispatch claim"
        );
        claim.point = started;
        assert!(
            store.mark_study_started(&claim).await.is_err(),
            "refreshing to the exact returned point cannot admit a second dispatch"
        );
        assert!(
            claim.begin_dispatch().is_err(),
            "a recorded native start is refused before minting another nonce"
        );
        fail(&store, &claim).await;
        let summary =
            StudySummary::from(&store.canonical_study("lost-ack").await.unwrap().unwrap());
        let fence = store
            .begin_study_finalization(&summary, "summary-worker", lifetime)
            .await
            .unwrap()
            .unwrap();
        let attempt = store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        let recovered = store
            .finish_study_finalization(
                &summary.run,
                &attempt.claim_operation,
                attempt.request.as_slice(),
                Err(CanonicalError::Timeout),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(recovered.attempt(), fence.attempt());
        assert_eq!(recovered.generation(), fence.generation());
        assert!(matches!(
            store
                .finish_study_finalization(
                    &summary.run,
                    &attempt.claim_operation,
                    b"different worker request",
                    Err(CanonicalError::Timeout)
                )
                .await,
            Err(CanonicalError::OperationReused)
        ));
        let mut response = bounded_query(
            store
                .db
                .query("SELECT key FROM canonical_attempts WHERE run=$run;")
                .bind(("run", summary.run)),
        )
        .await
        .unwrap();
        assert_eq!(
            response.take::<Vec<Object>>(0).unwrap().len(),
            1,
            "lost summary acknowledgment must not mint a second generation"
        );

        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_start_lost_ack_cannot_redispatch_after_cancellation() {
        let (store, database, revision) = fixture().await;
        store
            .create_study(
                "cancel-start",
                &request(&revision, "parent"),
                &[9],
                &[occurrence(&revision, 0, vec![])],
                &|| false,
            )
            .await
            .unwrap();
        let scope = store
            .study_scope(&point_key("cancel-start", OccurrenceKey(0)))
            .await
            .unwrap();
        let claim = store
            .claim_study_point(&scope, None, "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let admission = claim.begin_dispatch().unwrap();
        store.send_study_started(&admission).await.unwrap();
        store.cancel_study("cancel-start").await.unwrap();
        assert!(
            store
                .finish_study_started(admission, Err(CanonicalError::Timeout))
                .await
                .is_err()
        );
        assert!(
            point_facts(
                &store
                    .canonical_study_point(&claim.point.key)
                    .await
                    .unwrap()
                    .unwrap()
            )
            .unwrap()
            .native_started,
            "the committed boundary remains historical fact"
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_retry_new_attempt_has_its_own_native_start_boundary() {
        let (store, database, revision) = fixture().await;
        let mut point = occurrence(&revision, 0, vec![]);
        point.policy.attempt_limit = 2;
        store
            .create_study(
                "retry-start",
                &request(&revision, "parent"),
                &[9],
                &[point],
                &|| false,
            )
            .await
            .unwrap();
        let scope = store
            .study_scope(&point_key("retry-start", OccurrenceKey(0)))
            .await
            .unwrap();
        let mut first = store
            .claim_study_point(&scope, None, "first", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        first.point = store.mark_study_started(&first).await.unwrap();
        let closed = store
            .close_result_ingestion(&first.fence, "close-first")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this controlled lifecycle fixture asserts failure and no effect.
        unsafe { store.seal_attempt(&manifest, "seal-first", TerminalClass::Failed, &[]) }
            .await
            .unwrap();
        let (mut facts, outcome) = observation(&first.point, StudyPointState::Failed);
        facts.retry_failure = Some(pse_model::study::RetryFailure::Transient);
        // SAFETY: an actual failed terminal receipt backs this effect-free retry fact.
        unsafe { store.observe_study_point(&first.point, &facts, &outcome, false) }
            .await
            .unwrap();
        let scope = store.study_scope(&first.point.key).await.unwrap();
        assert!(matches!(
            scope.action(None).unwrap().kind,
            ActionKind::Start(_)
        ));
        let retry = store
            .claim_study_point(&scope, None, "retry", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        assert_ne!(retry.fence.attempt(), first.fence.attempt());
        assert!(retry.fence.generation() > first.fence.generation());
        assert!(!point_facts(&retry.point).unwrap().native_started);
        let started = store.mark_study_started(&retry).await.unwrap();
        let facts = point_facts(&started).unwrap();
        assert!(facts.native_started);
        assert_eq!(facts.attempt_count, 2);
        assert_eq!(
            store
                .canonical_attempt(first.fence.attempt())
                .await
                .unwrap()
                .unwrap()
                .outcome
                .as_deref(),
            Some("failed")
        );
        let mut response = bounded_query(store.db.query(
            "SELECT key FROM canonical_execution_operations WHERE run=$run AND kind='study-start';"
        ).bind(("run", first.fence.run().to_owned()))).await.unwrap();
        assert_eq!(
            response.take::<Vec<Object>>(0).unwrap().len(),
            2,
            "each genuinely new attempt owns one dispatch boundary"
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn canonical_studies_scoped_claim_premises_and_distinct_occurrences() {
        let (store, database, revision) = fixture().await;
        let points = vec![
            occurrence(&revision, 0, vec![]),
            occurrence(&revision, 1, vec![Dependency::Ordering(OccurrenceKey(0))]),
            occurrence(
                &revision,
                2,
                vec![Dependency::UsableResult(OccurrenceKey(0))],
            ),
            occurrence(&revision, 3, vec![]),
        ];
        store
            .create_study(
                "study",
                &request(&revision, "study-run"),
                &[9],
                &points,
                &|| false,
            )
            .await
            .unwrap();
        let replay = store
            .create_study(
                "study",
                &request(&revision, "study-run"),
                &[9],
                &points,
                &|| false,
            )
            .await
            .unwrap();
        assert!(replay.unwrap().active);
        let page = store.study_candidates("study", None).await.unwrap();
        assert_eq!(page.len(), 4);
        for (ordinal, point) in page.iter().enumerate() {
            let full = store
                .canonical_study_point(&point.key)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(point, &ScopedStudyPoint::from(&full));
            assert_eq!(point.ordinal, ordinal as u64);
            assert_eq!(full.descriptor.as_slice(), &[42]);
        }
        let waiting = store
            .study_scope(&point_key("study", OccurrenceKey(1)))
            .await
            .unwrap();
        assert_eq!(waiting.predecessors().len(), 1);
        assert!(matches!(
            waiting.action(None).unwrap().kind,
            ActionKind::Wait(_)
        ));
        let first = store
            .study_scope(&point_key("study", OccurrenceKey(0)))
            .await
            .unwrap();
        assert!(first.predecessors().is_empty());
        let (left, right) = tokio::join!(
            store.claim_study_point(&first, None, "left", "worker", Duration::from_secs(60)),
            store.claim_study_point(&first, None, "right", "worker", Duration::from_secs(60))
        );
        assert_ne!(left.is_ok(), right.is_ok());
        let claimed = left.or(right).unwrap();
        fail(&store, &claimed).await;
        assert!(
            store
                .claim_study_point(
                    &waiting,
                    None,
                    "stale-premise",
                    "worker",
                    Duration::from_secs(60)
                )
                .await
                .is_err()
        );
        let order = store
            .study_scope(&point_key("study", OccurrenceKey(1)))
            .await
            .unwrap();
        assert!(matches!(
            order.action(None).unwrap().kind,
            ActionKind::Start(StartProvenance::NotNeeded)
        ));
        let usable = store
            .study_scope(&point_key("study", OccurrenceKey(2)))
            .await
            .unwrap();
        assert!(matches!(
            usable.action(None).unwrap().kind,
            ActionKind::Refuse(_)
        ));
        assert!(
            store
                .claim_study_point(&usable, None, "unusable", "worker", Duration::from_secs(60))
                .await
                .is_err()
        );
        let disjoint = store
            .study_scope(&point_key("study", OccurrenceKey(3)))
            .await
            .unwrap();
        let claim = store
            .claim_study_point(
                &disjoint,
                None,
                "independent",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        assert_ne!(claim.fence.run(), claimed.fence.run());
        assert!(store.conclude_study("study").await.is_err());
        let cancel_scope = store
            .study_scope(&point_key("study", OccurrenceKey(1)))
            .await
            .unwrap();
        store.cancel_study("study").await.unwrap();
        assert!(
            store
                .claim_study_point(
                    &cancel_scope,
                    None,
                    "after-cancel",
                    "worker",
                    Duration::from_secs(60)
                )
                .await
                .is_err()
        );
        assert!(
            store
                .study_scope(&point_key("study", OccurrenceKey(1)))
                .await
                .unwrap()
                .cancelled()
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    async fn inactive_complete_study(
        store: &CanonicalStore,
        revision: &crate::canonical::Revision,
        key: &str,
    ) -> Study {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        let result = CREATION_OBSERVER
            .scope(
                Box::new(move |boundary| {
                    if boundary == CreationBoundary::BeforeActivation {
                        signal.store(true, Ordering::SeqCst);
                    }
                }),
                store.create_study(
                    key,
                    &request(revision, &format!("parent-{key}")),
                    &[9],
                    &[occurrence(revision, 0, vec![])],
                    &|| cancelled.load(Ordering::SeqCst),
                ),
            )
            .await
            .unwrap();
        assert!(result.is_none());
        let study = store.canonical_study(key).await.unwrap().unwrap();
        assert!(!study.active && !study.cancelled);
        assert_eq!(study.next_ordinal, study.point_count);
        study
    }

    #[tokio::test]
    async fn study_creation_precancelled_has_no_published_header() {
        let (store, database, revision) = fixture().await;
        assert!(
            store
                .create_study(
                    "precancelled",
                    &request(&revision, "parent"),
                    &[9],
                    &[occurrence(&revision, 0, vec![])],
                    &|| true
                )
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .canonical_study("precancelled")
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.canonical_run("parent").await.unwrap().is_none());
        assert!(store.canonical_run("point-run-0").await.unwrap().is_none());
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_cancellation_between_batches_stays_inactive() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let (store, database, revision) = fixture().await;
        let points = (0..100)
            .map(|key| occurrence(&revision, key, vec![]))
            .collect::<Vec<_>>();
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        let result = CREATION_OBSERVER
            .scope(
                Box::new(move |boundary| {
                    if boundary == CreationBoundary::BatchAppended {
                        signal.store(true, Ordering::SeqCst);
                    }
                }),
                store.create_study(
                    "partial",
                    &request(&revision, "parent"),
                    &[9],
                    &points,
                    &|| cancelled.load(Ordering::SeqCst),
                ),
            )
            .await
            .unwrap();
        assert!(result.is_none());
        let header = store.canonical_study("partial").await.unwrap().unwrap();
        assert!(!header.active && !header.cancelled);
        assert_eq!(header.next_ordinal, 64);
        assert_eq!(header.point_count, 100);
        assert!(
            store
                .study_scope(&point_key("partial", OccurrenceKey(0)))
                .await
                .is_err()
        );
        assert!(
            store
                .canonical_study_point(&point_key("partial", OccurrenceKey(64)))
                .await
                .unwrap()
                .is_none()
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_cancellation_immediately_before_activation_stays_inactive() {
        let (store, database, revision) = fixture().await;
        let study = inactive_complete_study(&store, &revision, "before-activation").await;
        assert!(!study.active && !study.cancelled);
        assert!(
            store
                .study_scope(&point_key(&study.key, OccurrenceKey(0)))
                .await
                .is_err()
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_cancellation_after_acknowledged_activation_is_durable() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let (store, database, revision) = fixture().await;
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        let study = CREATION_OBSERVER
            .scope(
                Box::new(move |boundary| {
                    if boundary == CreationBoundary::ActivationAcknowledged {
                        signal.store(true, Ordering::SeqCst);
                    }
                }),
                store.create_study(
                    "after-activation",
                    &request(&revision, "parent"),
                    &[9],
                    &[occurrence(&revision, 0, vec![])],
                    &|| cancelled.load(Ordering::SeqCst),
                ),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(study.active && study.cancelled);
        assert_eq!(study.generation, 1);
        let scope = store
            .study_scope(&point_key(&study.key, OccurrenceKey(0)))
            .await
            .unwrap();
        assert!(scope.cancelled());
        assert!(
            store
                .claim_study_point(
                    &scope,
                    None,
                    "never-start",
                    "worker",
                    Duration::from_secs(60)
                )
                .await
                .is_err()
        );
        assert!(
            !store
                .canonical_study_point(&scope.point().key)
                .await
                .unwrap()
                .unwrap()
                .assigned
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_issued_activation_lost_ack_settles_and_cancels_exact_identity() {
        let (store, database, revision) = fixture().await;
        let expected = inactive_complete_study(&store, &revision, "issued-activation").await;
        let committed = store.send_study_activation(&expected.key).await.unwrap();
        assert!(committed.active);
        let cancelled = store
            .finish_study_activation(&expected, Err(CanonicalError::Timeout), &|| true)
            .await
            .unwrap()
            .unwrap();
        assert!(cancelled.active && cancelled.cancelled);
        assert_eq!(cancelled.key, expected.key);
        assert_eq!(cancelled.run, expected.run);
        assert_eq!(cancelled.next_ordinal, 1);
        assert_eq!(cancelled.generation, 1);
        assert!(store.send_study_activation(&expected.key).await.is_err());
        // A lost cancel acknowledgement settles the same revoked generation.
        let settled = store
            .finish_study_cancel(&expected.key, Err(CanonicalError::Timeout))
            .await
            .unwrap();
        assert_eq!(settled.generation, cancelled.generation);
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_uncertain_inactive_activation_is_fenced_before_return() {
        let (store, database, revision) = fixture().await;
        let expected = inactive_complete_study(&store, &revision, "uncertain-activation").await;
        let result = store
            .finish_study_activation(&expected, Err(CanonicalError::Timeout), &|| true)
            .await
            .unwrap();
        assert!(result.is_none());
        let fenced = store.canonical_study(&expected.key).await.unwrap().unwrap();
        assert!(!fenced.active && fenced.cancelled);
        // An issued request that arrives after this observation cannot expose work.
        assert!(store.send_study_activation(&expected.key).await.is_err());
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_lost_ack_recovers_exact_active_membership_without_reingestion() {
        let (store, database, revision) = fixture().await;
        let expected = inactive_complete_study(&store, &revision, "lost-activation").await;
        store.send_study_activation(&expected.key).await.unwrap();
        let recovered = store
            .finish_study_activation(&expected, Err(CanonicalError::Timeout), &|| false)
            .await
            .unwrap()
            .unwrap();
        assert!(recovered.active && !recovered.cancelled);
        assert_eq!(recovered.generation, 0);
        assert_eq!(recovered.next_ordinal, recovered.point_count);
        let mut different = expected.clone();
        different.metadata = vec![8].into();
        assert!(matches!(
            store
                .finish_study_activation(&different, Err(CanonicalError::Timeout), &|| false)
                .await,
            Err(CanonicalError::OperationReused)
        ));
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_cancelled_retry_settles_existing_active_identity() {
        let (store, database, revision) = fixture().await;
        let points = [occurrence(&revision, 0, vec![])];
        let run = request(&revision, "parent-retry");
        store
            .create_study("active-retry", &run, &[9], &points, &|| false)
            .await
            .unwrap()
            .unwrap();
        let cancelled = store
            .create_study("active-retry", &run, &[9], &points, &|| true)
            .await
            .unwrap()
            .unwrap();
        assert!(cancelled.active && cancelled.cancelled);
        assert_eq!(cancelled.generation, 1);
        let replay = store
            .create_study("active-retry", &run, &[9], &points, &|| true)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(replay.generation, 1);
        let other_run = request(&revision, "other-parent-retry");
        let mut other_points = points.clone();
        other_points[0].run.key = "other-point-retry".into();
        store
            .create_study("different-retry", &other_run, &[9], &other_points, &|| {
                false
            })
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            store
                .create_study("different-retry", &other_run, &[8], &other_points, &|| true)
                .await,
            Err(CanonicalError::OperationReused)
        ));
        assert!(
            !store
                .canonical_study("different-retry")
                .await
                .unwrap()
                .unwrap()
                .cancelled
        );
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    async fn cancelled_retry_preserves_active_study(
        store: &CanonicalStore,
        key: &str,
        run: &RunRequest,
        points: &[NewOccurrence],
    ) {
        assert!(matches!(
            store.create_study(key, run, &[9], points, &|| true).await,
            Err(CanonicalError::OperationReused)
        ));
        let retained = store.canonical_study(key).await.unwrap().unwrap();
        assert!(retained.active && !retained.cancelled);
        assert_eq!(retained.generation, 0);
        assert_eq!(retained.next_ordinal, retained.point_count);
    }

    #[tokio::test]
    async fn study_creation_cancelled_retry_refuses_changed_later_descriptor() {
        let (store, database, revision) = fixture().await;
        let run = request(&revision, "descriptor-parent");
        let points = [
            occurrence(&revision, 0, vec![]),
            occurrence(&revision, 1, vec![]),
        ];
        store
            .create_study("descriptor-retry", &run, &[9], &points, &|| false)
            .await
            .unwrap()
            .unwrap();
        let mut changed = points.clone();
        changed[1].descriptor.push(43);
        cancelled_retry_preserves_active_study(&store, "descriptor-retry", &run, &changed).await;
        let retained = store
            .canonical_study_point(&point_key("descriptor-retry", OccurrenceKey(1)))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retained.descriptor.as_slice(), points[1].descriptor);
        let exact = store
            .create_study("descriptor-retry", &run, &[9], &points, &|| true)
            .await
            .unwrap()
            .unwrap();
        assert!(exact.active && exact.cancelled);
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_cancelled_retry_refuses_changed_policy_and_membership() {
        let (store, database, revision) = fixture().await;
        let run = request(&revision, "policy-parent");
        let points = [
            occurrence(&revision, 0, vec![]),
            occurrence(&revision, 1, vec![Dependency::Ordering(OccurrenceKey(0))]),
        ];
        store
            .create_study("policy-retry", &run, &[9], &points, &|| false)
            .await
            .unwrap()
            .unwrap();
        for field in 0..6 {
            let mut changed = points.clone();
            match field {
                0 => {
                    changed[1].policy.dependencies =
                        vec![Dependency::UsableResult(OccurrenceKey(0))]
                }
                1 => changed[1].policy.seed_need = SeedNeed::Required,
                2 => {
                    changed[1].policy.start =
                        StartPolicy::Continuation(pse_model::study::SeedEdge {
                            predecessor: OccurrenceKey(0),
                            role: pse_model::study::SeedRole::PrimalSolution,
                            permission: Default::default(),
                            unavailable: Default::default(),
                        })
                }
                3 => changed[1].policy.attempt_limit = 2,
                4 => changed[1].policy.key = OccurrenceKey(2),
                5 => changed.swap(0, 1),
                _ => unreachable!(),
            }
            cancelled_retry_preserves_active_study(&store, "policy-retry", &run, &changed).await;
        }
        let retained = store
            .canonical_study_point(&point_key("policy-retry", OccurrenceKey(1)))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(point_policy(&retained).unwrap(), points[1].policy);
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn study_creation_cancelled_retry_refuses_changed_parent_and_point_run_receipts() {
        let (store, database, revision) = fixture().await;
        let run = request(&revision, "run-parent");
        let points = [
            occurrence(&revision, 0, vec![]),
            occurrence(&revision, 1, vec![]),
        ];
        store
            .create_study("run-retry", &run, &[9], &points, &|| false)
            .await
            .unwrap()
            .unwrap();
        for parent in [true, false] {
            for field in 0..5 {
                let mut changed_run = run.clone();
                let mut changed_points = points.clone();
                let request = if parent {
                    &mut changed_run
                } else {
                    &mut changed_points[1].run
                };
                match field {
                    0 => request.request.push(4),
                    1 => request.source_selection.push(4),
                    2 => request.attestation.push(4),
                    3 => request.sources.push(revision.clone()),
                    4 => request.revision.sequence += 1,
                    _ => unreachable!(),
                }
                cancelled_retry_preserves_active_study(
                    &store,
                    "run-retry",
                    &changed_run,
                    &changed_points,
                )
                .await;
            }
        }
        let mut changed = points.clone();
        changed[1].run.key = "unpublished-retry-run".into();
        cancelled_retry_preserves_active_study(&store, "run-retry", &run, &changed).await;
        assert!(
            store
                .canonical_run("unpublished-retry-run")
                .await
                .unwrap()
                .is_none()
        );
        let retained = store
            .canonical_run(&points[1].run.key)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retained.request.as_slice(), points[1].run.request);
        assert_eq!(retained.attestation.as_slice(), points[1].run.attestation);
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }
}
