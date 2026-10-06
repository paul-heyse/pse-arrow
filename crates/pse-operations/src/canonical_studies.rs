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

impl CanonicalStore {
    /// Store admitted definitions in bounded batches, then atomically activate complete
    /// membership. Retrying exact identities settles ingestion without executing science.
    pub async fn create_study(
        &self,
        key: &str,
        run: &RunRequest,
        metadata: &[u8],
        points: &[NewOccurrence],
    ) -> Result<Study, CanonicalError> {
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
        self.begin_run(run).await?;
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
        let object = wire::encode_canonical_studies(&row)?;
        self.ensure_writes()?;
        protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::create($row);")
                .bind(("row", object.clone())))
        })
        .await?;
        // Bulk native insertion amortizes round trips while each transaction remains
        // bounded by both occurrence count and encoded descriptor/edge extent.
        let mut batch = Vec::new();
        let mut batch_edges = Vec::new();
        let mut extent = 0_usize;
        for (ordinal, point) in points.iter().enumerate() {
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
                self.append_study_batch(&row.key, &batch, &batch_edges)
                    .await?;
                batch.clear();
                batch_edges.clear();
                extent = 0;
            }
            extent += point_extent;
            batch.push(wire::encode_canonical_study_points(&native)?);
            batch_edges.extend(edges);
        }
        if !batch.is_empty() {
            self.append_study_batch(&row.key, &batch, &batch_edges)
                .await?;
        }
        let mut response = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::activate($study);")
                .bind(("study", row.key.clone())))
        })
        .await?;
        Ok(wire::decode_canonical_studies(
            response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
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
        let mut response=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::claim($study,$generation,$point,$expected,$start,$operation,$request,$claim_request,$attempt,$worker,$lifetime,$facts);").bind(("study",scope.study.clone())).bind(("generation",codec::encode_uint(scope.generation)?)).bind(("point",scope.point.key.clone())).bind(("expected",expected.clone())).bind(("start",Bytes::from(start_bytes.clone()))).bind(("operation",operation.to_owned())).bind(("request",Bytes::from(request.clone()))).bind(("claim_request",Bytes::from(claim_request.clone()))).bind(("attempt",attempt.clone())).bind(("worker",worker.to_owned())).bind(("lifetime",lifetime)).bind(("facts",Bytes::from(facts.clone()))))).await?;
        let row = wire::decode_canonical_attempts(
            response
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?;
        let fence = AttemptFence::from_row(row)?;
        let point = self
            .canonical_study_point(&scope.point.key)
            .await?
            .ok_or(CanonicalError::IncompleteResponse)?;
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
    /// authority. A claim alone is never reported as a native start.
    pub async fn mark_study_started(
        &self,
        claim: &StudyClaim,
    ) -> Result<StudyPoint, CanonicalError> {
        let mut facts = point_facts(&claim.point)?;
        facts.revision = 0;
        facts.native_started = true;
        let facts = encode(&facts)?;
        self.ensure_writes()?;
        let mut result=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::started($study,$point,$revision,$run,$attempt,$generation,$facts);").bind(("study",claim.point.study.clone())).bind(("point",claim.point.key.clone())).bind(("revision",codec::encode_uint(claim.point.revision)?)).bind(("run",claim.fence.run().to_owned())).bind(("attempt",claim.fence.attempt().to_owned())).bind(("generation",codec::encode_uint(claim.fence.generation())?)).bind(("facts",Bytes::from(facts.clone()))))).await?;
        Ok(wire::decode_canonical_study_points(
            result
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
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
        let mut result=protected_query(||Ok(self.db.query("RETURN fn::pse_study_v1::finalize($study,$operation,$request,$attempt,$worker,$lifetime);").bind(("study",study.key.clone())).bind(("operation",operation.clone())).bind(("request",Bytes::from(request.clone()))).bind(("attempt",attempt.clone())).bind(("worker",worker.to_owned())).bind(("lifetime",lifetime)))).await?;
        result
            .take::<Option<Object>>(0)?
            .map(|row| AttemptFence::from_row(wire::decode_canonical_attempts(row)?))
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
        let mut result = protected_query(|| {
            Ok(self
                .db
                .query("RETURN fn::pse_study_v1::cancel($study);")
                .bind(("study", key.to_owned())))
        })
        .await?;
        Ok(wire::decode_canonical_studies(
            result
                .take::<Option<Object>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)?,
        )?)
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
        unsafe {
            store
                .seal_attempt(&manifest, "seal-predecessor", TerminalClass::Failed, &[])
                .await
        }
        .unwrap();
        let (facts, outcome) = observation(&claim.point, StudyPointState::Failed);
        unsafe {
            store
                .observe_study_point(&claim.point, &facts, &outcome, true)
                .await
        }
        .unwrap()
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
            .create_study("study", &request(&revision, "study-run"), &[9], &points)
            .await
            .unwrap();
        let replay = store
            .create_study("study", &request(&revision, "study-run"), &[9], &points)
            .await
            .unwrap();
        assert!(replay.active);
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
}
