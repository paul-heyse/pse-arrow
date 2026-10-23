// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact admitted result reads. Metadata discovery does not fetch trajectory
//! blobs; each later read names one immutable member of the closed descriptor.

use crate::{
    canonical::{
        CanonicalError, CanonicalStore, PROTECTED_BEGIN, ProtectedSelection, bounded_query,
    },
    canonical_codec,
    canonical_execution::{ResultSetDescriptor, result_batch_key},
    generated::surreal as wire,
};
use pse_model::generated::runtime::{
    canonical_attempts::Row as Attempt, canonical_result_batches::Row as ResultBatch,
    canonical_result_block_outputs::Row as BlockOutput,
    canonical_result_blocks::Row as BlockMetadata, canonical_result_manifests::Row as Manifest,
    canonical_runs::Row as Run,
};
use std::{sync::Arc, time::Duration};
use surrealdb::types::{Object, Value};

/// Bound on independently read scientific blobs, including IPC schema and EOS.
pub const RESULT_BLOCK_BYTES: usize = wire::RESULT_BLOCK_BYTES;
pub(crate) const PAGE: usize = 64;
const IDENTITY_BYTES: usize = pse_schema::catalog::substrate::METADATA_ID_BYTES;

// Closing a client must join automatic release RPCs, and must refuse while
// returned owners still protect their exact selections. Fixture teardown uses
// the same production lifetime before disposing of its isolated database.
#[derive(Debug, Default)]
pub(crate) struct ResultReadDrain {
    live: std::sync::atomic::AtomicUsize,
    closing: std::sync::Mutex<bool>,
    releases: tokio_util::task::TaskTracker,
    error: std::sync::Mutex<Option<Arc<CanonicalError>>>,
    failed_protections: std::sync::Mutex<Vec<Arc<FailedProtection>>>,
}
#[derive(Debug)]
struct FailedProtection {
    store: CanonicalStore,
    revision: crate::canonical::Revision,
    key: String,
    error: Arc<CanonicalError>,
    retry: tokio::sync::Mutex<()>,
    done: std::sync::atomic::AtomicBool,
    _metadata_owner: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
}
impl ResultReadDrain {
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) fn live(&self) -> usize {
        self.live.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub(crate) fn retain(&self) -> Result<(), CanonicalError> {
        let closing = self
            .closing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *closing {
            return Err(invalid("canonical client protection admission is closed"));
        }
        self.live.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
    pub(crate) fn release(self: &Arc<Self>) -> ResultReleaseTask {
        let token = self.releases.token();
        self.live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        ResultReleaseTask {
            drain: self.clone(),
            completed: false,
            recovery: None,
            metadata_owner: None,
            _token: token,
        }
    }
    pub(crate) fn release_protection(
        self: &Arc<Self>,
        store: CanonicalStore,
        revision: crate::canonical::Revision,
        key: String,
        metadata_owner: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
    ) -> ResultReleaseTask {
        let mut release = self.release();
        release.recovery = Some((store, revision, key));
        release.metadata_owner = metadata_owner;
        release
    }
    pub(crate) async fn recover(self: &Arc<Self>) -> Result<(), CanonicalError> {
        let failed = self
            .failed_protections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for entry in failed {
            let drain = self.clone();
            let task = tokio::spawn(async move {
                let _retry = entry.retry.lock().await;
                if entry.done.load(std::sync::atomic::Ordering::Acquire) {
                    return Ok(());
                }
                crate::canonical::transport::optional_response_ownership_scope(
                    entry._metadata_owner.clone(),
                    entry.store.release_protection(&entry.revision, &entry.key),
                )
                .await?;
                entry.done.store(true, std::sync::atomic::Ordering::Release);
                drain
                    .failed_protections
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .retain(|entry| !entry.done.load(std::sync::atomic::Ordering::Acquire));
                Ok::<(), CanonicalError>(())
            });
            task.await
                .map_err(|error| invalid(&format!("protection recovery task failed: {error}")))??;
        }
        Ok(())
    }
    fn failure(&self, error: CanonicalError) {
        let mut saved = self
            .error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if saved.is_none() {
            *saved = Some(Arc::new(error));
        }
    }
    /// Seal new protection admission atomically with observing no live owners.
    /// A refusal for a live owner leaves admission open for that owner to finish.
    pub(crate) async fn close_and_drain(&self) -> Result<(), CanonicalError> {
        {
            let mut closing = self
                .closing
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if self.live.load(std::sync::atomic::Ordering::SeqCst) != 0 {
                return Err(invalid("canonical client still has live protected readers"));
            }
            *closing = true;
            self.releases.close();
        }
        self.wait_for_releases().await
    }
    #[cfg(any(test, feature = "test-support", feature = "canonical-tests"))]
    pub(crate) async fn drain(&self) -> Result<(), CanonicalError> {
        if self.live.load(std::sync::atomic::Ordering::SeqCst) != 0 {
            return Err(invalid("canonical client still has live protected readers"));
        }
        self.join_pending_releases().await
    }
    /// Join cleanup already enqueued by dropped readers while independent
    /// readers may remain live. This does not close protection admission or
    /// establish global reader drain; later drops still enqueue tracked releases.
    #[cfg(any(test, feature = "test-support", feature = "canonical-tests"))]
    pub(crate) async fn join_pending_releases(&self) -> Result<(), CanonicalError> {
        self.releases.close();
        self.wait_for_releases().await
    }
    async fn wait_for_releases(&self) -> Result<(), CanonicalError> {
        tokio::time::timeout(crate::canonical::REQUEST_TIMEOUT, self.releases.wait())
            .await
            .map_err(|_| CanonicalError::Timeout)?;
        if let Some(entry) = self
            .failed_protections
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
        {
            return Err(CanonicalError::ProtectionRelease {
                source: entry.error.clone(),
            });
        }
        if let Some(error) = self
            .error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            return Err(CanonicalError::ProtectionRelease {
                source: error.clone(),
            });
        }
        Ok(())
    }
}
pub(crate) struct ResultReleaseTask {
    drain: Arc<ResultReadDrain>,
    completed: bool,
    recovery: Option<(CanonicalStore, crate::canonical::Revision, String)>,
    metadata_owner: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
    _token: tokio_util::task::task_tracker::TaskTrackerToken,
}
impl ResultReleaseTask {
    fn failure(&mut self, error: CanonicalError) {
        if let Some((store, revision, key)) = self.recovery.take() {
            self.drain
                .failed_protections
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(Arc::new(FailedProtection {
                    store,
                    revision,
                    key,
                    error: Arc::new(error),
                    retry: tokio::sync::Mutex::new(()),
                    done: std::sync::atomic::AtomicBool::new(false),
                    _metadata_owner: self.metadata_owner.take(),
                }));
        } else {
            self.drain.failure(error);
        }
    }
    pub(crate) fn complete(mut self, result: Result<(), CanonicalError>) {
        if let Err(error) = result {
            self.failure(error);
        }
        self.completed = true;
    }
}
impl Drop for ResultReleaseTask {
    fn drop(&mut self) {
        if !self.completed {
            self.failure(invalid(
                "result protection release task ended before completion",
            ));
        }
        // The tracker token drops only after failure/completion has been recorded.
    }
}

/// Lifetime ownership shared by a selection and its returned block buffers.
/// Server leases prevent stale selections from resurrecting reclaimed sources.
#[derive(Debug)]
struct ReadOwner {
    store: CanonicalStore,
    selection: ProtectedSelection,
}

/// Live permission for only the original completion of an exact terminal attempt.
/// This owner carries neither a manifest inventory nor eligible seed discovery.
#[derive(Clone, Debug)]
pub struct CompletionRead {
    run: Run,
    attempt: Attempt,
    chunks: Option<pse_model::generated::runtime::canonical_result_sets::Row>,
    owner: Arc<ReadOwner>,
}
impl CompletionRead {
    /// Actual completion selection storage, without a manifest inventory.
    pub fn retained_bytes(&self) -> Result<usize, CanonicalError> {
        use pse_model::HeapUsage;
        [
            size_of::<Self>(),
            self.run.heap_bytes(),
            self.attempt.heap_bytes(),
            self.chunks.as_ref().map_or(0, HeapUsage::heap_bytes),
            size_of::<ReadOwner>(),
            2 * size_of::<usize>(),
            self.owner
                .store
                .handle_heap_bytes()
                .ok_or(CanonicalError::PayloadLimit)?,
            self.owner
                .selection
                .retained_bytes()
                .ok_or(CanonicalError::PayloadLimit)?,
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
        .ok_or(CanonicalError::PayloadLimit)
    }
    /// Carry this selection's actual metadata through its protected cleanup.
    pub fn retain_metadata_owner(
        &self,
        owner: Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<(), CanonicalError> {
        self.owner.selection.retain_metadata_owner(owner)
    }
    /// Exact admitted terminal receipt containing the completion header.
    pub fn attempt(&self) -> &Attempt {
        &self.attempt
    }
    /// Frozen completion chunk coverage, absent for an inline completion.
    pub fn chunk_coverage(&self) -> Option<(u64, u64)> {
        self.chunks
            .as_ref()
            .map(|set| (set.next_ordinal, set.row_count))
    }
}

/// An exact terminal attempt and its closed membership, pinned before paging.
/// Failed and partial outcomes retain their actual class rather than substituting
/// another run's values. Clone retains the same protected read owner.
#[derive(Clone, Debug)]
pub struct ResultRead {
    run: Run,
    attempt: Attempt,
    manifest: Manifest,
    sets: Vec<ResultSetDescriptor>,
    owner: Arc<ReadOwner>,
}
impl ResultRead {
    /// Retain allocation through file borrowers, tracked release and uncertain
    /// cleanup recovery, independently of the last public reader facade.
    pub fn retain_metadata_owner(
        &self,
        owner: Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<(), CanonicalError> {
        self.owner.selection.retain_metadata_owner(owner)
    }
    /// Owned selection metadata, including actual collection and string
    /// capacities. Shared physical clients remain with their deployment owner.
    /// This does not change clone authority or grant new read permission.
    pub fn retained_bytes(&self) -> Result<usize, CanonicalError> {
        use pse_model::HeapUsage;
        let descriptors = self.sets.iter().try_fold(
            self.sets
                .capacity()
                .checked_mul(size_of::<ResultSetDescriptor>())
                .ok_or(CanonicalError::PayloadLimit)?,
            |bytes, set| {
                [
                    set.key.capacity(),
                    set.name.capacity(),
                    set.batches_digest.capacity(),
                ]
                .into_iter()
                .try_fold(bytes, usize::checked_add)
                .ok_or(CanonicalError::PayloadLimit)
            },
        )?;
        [
            size_of::<Self>(),
            self.run.heap_bytes(),
            self.attempt.heap_bytes(),
            self.manifest.heap_bytes(),
            descriptors,
            size_of::<ReadOwner>(),
            2 * size_of::<usize>(),
            self.owner
                .store
                .handle_heap_bytes()
                .ok_or(CanonicalError::PayloadLimit)?,
            self.owner
                .selection
                .retained_bytes()
                .ok_or(CanonicalError::PayloadLimit)?,
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
        .ok_or(CanonicalError::PayloadLimit)
    }
    pub(crate) fn protected_selection(&self) -> &ProtectedSelection {
        &self.owner.selection
    }
    pub(crate) fn belongs_to(&self, store: &CanonicalStore) -> bool {
        store.db.same_driver(&self.owner.store.db)
    }
    /// Original immutable source protected by this exact result reader.
    pub fn source_revision(&self) -> &crate::canonical::Revision { self.owner.selection.revision() }
    /// Recheck and renew this original live grant before copied publication; expired grants never revive.
    pub async fn renew(&self, lifetime: Duration) -> Result<(), CanonicalError> { self.owner.selection.renew(lifetime).await }
    /// Exact semantic run and immutable source selection.
    pub fn run(&self) -> &Run {
        &self.run
    }
    /// Selected actual attempt, including recorded outcome and completion.
    pub fn attempt(&self) -> &Attempt {
        &self.attempt
    }
    /// Closed set descriptions, never a scan of late or abandoned staging.
    pub fn sets(&self) -> &[ResultSetDescriptor] {
        &self.sets
    }
    /// Exact immutable closure receipt admitted by the selected attempt.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn set(&self, key: &str) -> Result<&ResultSetDescriptor, CanonicalError> {
        self.sets
            .iter()
            .find(|s| s.key == key)
            .ok_or_else(|| invalid("result set outside admitted descriptor"))
    }
}
/// A catalog-only logical descriptor with independently verified file bytes.
#[derive(Clone, Debug)]
pub struct HydratedResultBatch {
    /// Selected immutable descriptor; no database bulk payload is retained.
    pub descriptor: ResultBatch,
    /// Actual file bytes retained under the caller's prepaid response allowance.
    pub payload: pse_model::Bytes,
    _allocation: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
}
impl std::ops::Deref for HydratedResultBatch {
    type Target = ResultBatch;
    fn deref(&self) -> &ResultBatch {
        &self.descriptor
    }
}

/// One exact admitted block with verified file bytes and continuing read ownership.
#[derive(Clone, Debug)]
pub struct ResultBlock {
    /// Narrow recorded shape, range and interpretation.
    pub metadata: BlockMetadata,
    /// Exact immutable batch; payload is not a queryable numeric projection.
    pub batch: HydratedResultBatch,
    owner: Arc<ReadOwner>,
}
impl ResultBlock {
    /// Protected immutable source revision attached to this retained buffer.
    pub fn source_revision(&self) -> &crate::canonical::Revision {
        self.owner.selection.revision()
    }
}

/// A non-Arrow scientific owner may decode its own admitted bounded payload.
/// This envelope carries no alternate scientific meaning or admission authority.
#[derive(Clone, Debug)]
pub struct ResultPayload {
    /// Exact immutable member of the closed result-set descriptor.
    pub batch: HydratedResultBatch,
    owner: Arc<ReadOwner>,
}
impl ResultPayload {
    /// Source protection remains owned while the encoded buffer is consumed.
    pub fn source_revision(&self) -> &crate::canonical::Revision {
        self.owner.selection.revision()
    }
}

/// One protected, complete prefix of original blocks and exact closed members.
/// Byte-limited short pages carry an explicit continuation and are not exhausted.
#[derive(Clone, Debug)]
pub struct JoinedResultBlockPage {
    /// Ordered original scientific blocks.
    pub blocks: Vec<BlockMetadata>,
    /// Exact descriptors in the same order; no block may disappear in a join.
    pub batches: Vec<ResultBatch>,
    /// Last returned original ordinal, when the page is nonempty.
    pub next: Option<u64>,
    /// Whether the selected inventory has no later member.
    pub exhausted: bool,
}

/// Narrow output indexes and their exact admitted original IPC block metadata.
#[derive(Clone, Debug)]
pub struct ResultOutputPage {
    /// Exact closed descriptors, one per distinct returned block.
    pub batches: Vec<ResultBatch>,
    /// Last returned original dense-index start coordinate.
    pub next: Option<u64>,
    /// True only when no later matching index remains, including byte-limited pages.
    pub exhausted: bool,
    /// Dense output coverage descriptors, without individual array elements.
    pub outputs: Vec<BlockOutput>,
    /// Existing original scientific blocks; each identity is manifest constrained.
    pub blocks: Vec<BlockMetadata>,
}

fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn identity(value: &str) -> Result<(), CanonicalError> {
    if value.is_empty() || value.len() > IDENTITY_BYTES {
        return Err(CanonicalError::PayloadLimit);
    }
    Ok(())
}

/// Check metadata against its exact batch, not a broad result-set prefix.
pub fn validate_result_block(
    metadata: &BlockMetadata,
    batch: &ResultBatch,
) -> Result<(), CanonicalError> {
    crate::canonical_execution::validate_result_batch(batch)?;
    validate_result_block_metadata(metadata, batch)
}

// The append owner has constructed the batch from ResultPayload; only that
// controlled owner can reuse its integrity work. Fresh reads use the full public
// verifier above before exposing any native bytes.
pub(crate) fn validate_result_block_metadata(
    metadata: &BlockMetadata,
    batch: &ResultBatch,
) -> Result<(), CanonicalError> {
    validate_result_block_interpretation(metadata, batch, wire::INTERPRETATION)
}
fn validate_result_block_interpretation(
    metadata: &BlockMetadata,
    batch: &ResultBatch,
    interpretation: &str,
) -> Result<(), CanonicalError> {
    if metadata.key != batch.key
        || metadata.batch != batch.key
        || metadata.result_set != batch.result_set
        || metadata.ordinal != batch.ordinal
        || metadata.rows != batch.row_count
        || metadata.end.checked_sub(metadata.start) != Some(metadata.rows)
        || metadata.payload_bytes != batch.payload_bytes
        || metadata.payload_bytes > RESULT_BLOCK_BYTES as u64
        || metadata.columns > 256
        || metadata.statistics.as_ref().is_some_and(|bytes| {
            bytes.len() > pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES
        })
        || metadata.interpretation != interpretation
    {
        return Err(invalid(
            "result block identity/shape/interpretation mismatch",
        ));
    }
    for value in [
        &metadata.key,
        &metadata.batch,
        &metadata.result_set,
        &metadata.output,
        &metadata.partition,
    ] {
        identity(value)?;
    }
    if metadata.coordinate_min.is_some() != metadata.coordinate_max.is_some()
        || metadata
            .coordinate_min
            .zip(metadata.coordinate_max)
            .is_some_and(|(min, max)| !min.is_finite() || !max.is_finite() || min > max)
    {
        return Err(invalid("result block coordinate projection"));
    }
    if metadata.payload_digest != batch.digest {
        return Err(invalid("result block payload digest"));
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the decoder verifies every independent admitted selection premise"
)]
fn decode_joined_result_page(
    mut envelope: Object,
    read: &ResultRead,
    set: &ResultSetDescriptor,
    output: &str,
    partition: &str,
    start: u64,
    end: u64,
    after: Option<u64>,
) -> Result<JoinedResultBlockPage, CanonicalError> {
    fn rows(value: Value) -> Result<Vec<Object>, CanonicalError> {
        match value {
            Value::Array(rows) if rows.len() <= PAGE => rows
                .into_iter()
                .map(|row| match row {
                    Value::Object(row) => Ok(row),
                    _ => Err(invalid("joined result member shape")),
                })
                .collect(),
            _ => Err(CanonicalError::PayloadLimit),
        }
    }
    let blocks = rows(canonical_codec::required(&mut envelope, "blocks")?)?
        .into_iter()
        .map(wire::decode_canonical_result_blocks)
        .collect::<Result<Vec<_>, _>>()?;
    let batches = rows(canonical_codec::required(&mut envelope, "batches")?)?
        .into_iter()
        .map(wire::decode_canonical_result_batches)
        .collect::<Result<Vec<_>, _>>()?;
    let next = match canonical_codec::required(&mut envelope, "next")? {
        Value::None | Value::Null => None,
        value => Some(canonical_codec::decode_uint(value)?),
    };
    let exhausted =
        canonical_codec::decode_boolean(canonical_codec::required(&mut envelope, "exhausted")?)?;
    if !envelope.is_empty()
        || blocks.len() != batches.len()
        || next != blocks.last().map(|block| block.ordinal)
        || (!exhausted && blocks.is_empty())
    {
        return Err(invalid(
            "joined result page coverage or continuation differs",
        ));
    }
    let mut previous = after;
    for (block, batch) in blocks.iter().zip(&batches) {
        validate_result_block(block, batch)?;
        if previous.is_some_and(|ordinal| block.ordinal <= ordinal)
            || block.key != result_batch_key(&read.attempt.key, &set.key, block.ordinal)
            || block.result_set != set.key
            || block.ordinal >= set.batch_count
            || block.output != output
            || block.partition != partition
            || block.end <= start
            || block.start >= end
            || block.interpretation != read.run.interpretation
            || batch.attempt != read.attempt.key
        {
            return Err(invalid(
                "joined result page outside exact admitted selection",
            ));
        }
        previous = Some(block.ordinal);
    }
    Ok(JoinedResultBlockPage {
        blocks,
        batches,
        next,
        exhausted,
    })
}

impl CanonicalStore {
    /// Select a bounded output index page natively, before fetching any original
    /// scientific payload. Dense coordinate predicates select recorded time bounds.
    #[allow(
        clippy::too_many_arguments,
        reason = "protected selection, output coordinates, cursor and finite coordinate bounds are explicit native read premises"
    )]
    pub async fn result_output_page(
        &self,
        read: &ResultRead,
        result_set: &str,
        output: &str,
        partition: &str,
        start: u64,
        end: u64,
        after: Option<u64>,
        minimum: Option<f64>,
        maximum: Option<f64>,
    ) -> Result<ResultOutputPage, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::OutputPage, async {
        let activity = read.owner.store.selection_activity(&read.owner.selection)?;
        activity.scope(async {
        if !self.db.same_driver(&read.owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        let set = read.set(result_set)?;
        identity(output)?;
        identity(partition)?;
        if start > end
            || minimum.is_some_and(|v| !v.is_finite())
            || maximum.is_some_and(|v| !v.is_finite())
            || minimum.zip(maximum).is_some_and(|(min, max)| min > max)
        {
            return Err(invalid("invalid output range/predicate"));
        }
        self.files.require_current()?;
        let query = format!("{PROTECTED_BEGIN}\nLET $pse_rpc_result=fn::pse_execution_v1::result_output_page($pse_rpc_expires_at,$result_set,$output,$partition,$start,$end,$after,$minimum,$maximum);\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection) {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $pse_rpc_result;\nCOMMIT;");
        let finite = |value: Option<f64>| {
            value
                .map(|v| Value::Number(surrealdb::types::Number::Float(v)))
                .unwrap_or(Value::None)
        };
        let mut response = self
            .protected_query(
                &read.run.problem,
                crate::canonical::QueryEffect::Read,
                "canonical_results::result_output_page",
                || {
                    Ok(self
                        .db
                        .metadata_query(crate::canonical::MetadataResponse::OutputPage, query.clone())
                        .bind(("problem", read.run.problem.clone()))
                        .bind(("revision", read.run.revision.clone()))
                        .bind((
                            "sequence",
                            canonical_codec::encode_uint(read.owner.selection.revision().sequence)?,
                        ))
                        .bind(("protection", read.owner.selection.key().to_owned()))
                        .bind(("result_set", set.key.clone()))
                        .bind(("output", output.to_owned()))
                        .bind(("partition", partition.to_owned()))
                        .bind(("start", canonical_codec::encode_uint(start)?))
                        .bind(("end", canonical_codec::encode_uint(end)?))
                        .bind((
                            "after",
                            after
                                .map(canonical_codec::encode_uint)
                                .transpose()?
                                .unwrap_or(Value::None),
                        ))
                        .bind(("minimum", finite(minimum)))
                        .bind(("maximum", finite(maximum))))
                },
            )
            .await?;
        let index = response.num_statements().saturating_sub(2);
        let mut object = response
            .take::<Option<Object>>(index)?
            .ok_or_else(|| invalid("output index response absent"))?;
        let rows = |value: Value| match value {
            Value::Array(rows) => rows
                .into_iter()
                .map(|v| match v {
                    Value::Object(row) => Ok(row),
                    _ => Err(invalid("output index row shape")),
                })
                .collect::<Result<Vec<_>, _>>(),
            _ => Err(invalid("output index array shape")),
        };
        let indexes = rows(canonical_codec::required(&mut object, "indexes")?)?;
        let blocks = rows(canonical_codec::required(&mut object, "blocks")?)?
            .into_iter()
            .map(wire::decode_canonical_result_blocks)
            .collect::<Result<Vec<_>, _>>()?;
        let batches = rows(canonical_codec::required(&mut object, "batches")?)?.into_iter()
            .map(wire::decode_canonical_result_batches).collect::<Result<Vec<_>,_>>()?;
        let next = match canonical_codec::required(&mut object, "next")? {
            Value::None | Value::Null => None,
            value => Some(canonical_codec::decode_uint(value)?),
        };
        let exhausted = canonical_codec::decode_boolean(canonical_codec::required(&mut object, "exhausted")?)?;
        if !object.is_empty() || indexes.len()>PAGE || blocks.len()>PAGE || batches.len()!=blocks.len()
            || (!exhausted && indexes.is_empty()) {
            return Err(invalid("joined output page extent or coverage differs"));
        }
        let mut identities=std::collections::BTreeSet::new();
        for (block,batch) in blocks.iter().zip(&batches) {
            validate_result_block(block,batch)?;
            if block.result_set!=set.key || block.ordinal>=set.batch_count
                || batch.attempt!=read.attempt.key || !identities.insert(block.key.clone()) {
                return Err(invalid("output index block outside exact manifest"));
            }
        }
        let mut page = ResultOutputPage { outputs: Vec::new(), blocks, batches, next, exhausted };
        let mut previous = after;
        for index in indexes {
            let item = wire::decode_canonical_result_block_outputs(index)?;
            let (key, batch, recorded_output, recorded_partition, lower, upper) = (
                item.result_set.clone(), item.batch.clone(), item.output.clone(),
                item.partition.clone(), item.start, item.end,
            );
            if item.interpretation != read.run.interpretation {
                return Err(invalid("output index interpretation"));
            }
            page.outputs.push(item);
            if key != set.key
                || recorded_output != output
                || recorded_partition != partition
                || previous.is_some_and(|row| lower <= row)
                || upper <= start
                || lower >= end
                || lower >= upper
                || !page
                    .blocks
                    .iter()
                    .any(|block| block.batch == batch && block.start <= lower && upper <= block.end)
            {
                return Err(invalid("output index outside exact recorded coverage"));
            }
            previous = Some(lower);
        }
        if page.next!=page.outputs.last().map(|item|item.start)
            || page.blocks.iter().any(|block| !page.outputs.iter().any(|item|item.batch==block.key)) {
            return Err(invalid("joined output page continuation or extra block differs"));
        }
        Ok(page)
            }).await
        }).await
    }
    /// Select the newest semantic run whose current terminal attempt has an
    /// explicitly requested recorded class. Empty classes include all outcomes.
    /// The problem/sequence native index orders discovery without reading blobs.
    pub async fn latest_results(
        &self,
        problem: &str,
        classes: &[crate::canonical_execution::TerminalClass],
    ) -> Result<ResultRead, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Selection, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        identity(problem)?;
        if classes.len() > 4 {
            return Err(invalid("terminal result class bound"));
        }
        let classes = classes
            .iter()
            .map(|class| class.as_str().to_owned())
            .collect::<Vec<_>>();
        let mut response=bounded_query(self.db.metadata_query(crate::canonical::MetadataResponse::Selection, "SELECT VALUE {run:key,attempt:terminal_attempt} FROM canonical_runs WHERE problem=$problem AND terminal_attempt != NONE AND terminal_class != NONE AND (array::len($classes)=0 OR terminal_class IN $classes) ORDER BY sequence DESC LIMIT 1;").bind(("problem",problem.to_owned())).bind(("classes",classes.clone()))).await?;
        let mut rows = response.take::<Vec<Object>>(0)?;
        let mut row = rows
            .pop()
            .ok_or_else(|| invalid("no admitted terminal result in requested classes"))?;
        if !rows.is_empty() {
            return Err(invalid("latest result selection cardinality"));
        }
        let run = canonical_codec::decode_string(canonical_codec::required(&mut row, "run")?)?;
        let attempt =
            canonical_codec::decode_string(canonical_codec::required(&mut row, "attempt")?)?;
        if !row.is_empty() {
            return Err(invalid("latest result selection fields"));
        }
        let read = self.read_results(&run, &attempt).await?;
        if read.run.problem != problem
            || (!classes.is_empty()
                && !read
                    .attempt
                    .outcome
                    .as_ref()
                    .is_some_and(|class| classes.iter().any(|selected| selected == class.as_str())))
        {
            return Err(invalid("latest result selection changed meaning"));
        }
        Ok(read)
            }).await
        }).await
    }

    async fn terminal_read_owner(
        &self,
        run_key: &str,
        attempt_key: &str,
        lifetime: Duration,
    ) -> Result<(Arc<ReadOwner>, Run, Attempt), CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Selection, async {
        identity(run_key)?;
        identity(attempt_key)?;
        let mut response=bounded_query(self.db.metadata_query(crate::canonical::MetadataResponse::Selection, "SELECT * FROM ONLY type::record('canonical_runs',$run); SELECT * FROM ONLY type::record('canonical_attempts',$attempt);")
            .bind(("run",run_key.to_owned())).bind(("attempt",attempt_key.to_owned()))).await?;
        let run = response
            .take::<Option<Object>>(0)?
            .map(wire::decode_canonical_runs)
            .transpose()?
            .ok_or_else(|| invalid("run unavailable"))?;
        let attempt = response
            .take::<Option<Object>>(1)?
            .map(wire::decode_canonical_attempts)
            .transpose()?
            .ok_or_else(|| invalid("attempt unavailable"))?;
        if run.key != run_key
            || attempt.key != attempt_key
            || attempt.run != run.key
            || !attempt.terminal()
            || !attempt.closed()
            || attempt.ingestion_open()
            || attempt.outcome.is_none()
            || run.interpretation != wire::INTERPRETATION
        {
            return Err(invalid("attempt is not an admitted terminal selection"));
        }
        let revision = self
            .revision(&run.revision)
            .await?
            .ok_or_else(|| invalid("run source revision unavailable"))?;
        if revision.problem != run.problem || revision.sequence != run.source_sequence {
            return Err(invalid("run/revision problem mismatch"));
        }
        let protection = self.protect(revision, lifetime).await?;
        let owner = Arc::new(ReadOwner {
            store: self.clone(),
            selection: protection,
        });
        Ok((owner, run, attempt))
        }).await
    }

    /// Resolve only an admitted terminal attempt, then pin its immutable revision
    /// before reading the exact manifest. Historical terminal attempts remain
    /// selectable even after the semantic run starts a new attempt.
    pub async fn read_results(
        &self,
        run_key: &str,
        attempt_key: &str,
    ) -> Result<ResultRead, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Selection, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let (owner, run, attempt) = self
            .terminal_read_owner(run_key, attempt_key, crate::canonical::REQUEST_TIMEOUT)
            .await?;
        let key = attempt
            .closed_manifest
            .as_deref()
            .ok_or_else(|| invalid("terminal attempt lacks closed manifest"))?;
        let mut response=self.protected_query(&format!("result-creators:{}",run.key), crate::canonical::QueryEffect::Mutation, "canonical_results::read_results", ||Ok(self.db.metadata_query(crate::canonical::MetadataResponse::Selection, format!("{PROTECTED_BEGIN}\n{RESULT_ADMISSION}\nSELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);\nCOMMIT;"))
            .bind(("problem",owner.selection.revision().problem.clone()))
            .bind(("revision",owner.selection.revision().key.clone()))
            .bind(("sequence",canonical_codec::encode_uint(owner.selection.revision().sequence)?))
            .bind(("protection",owner.selection.key().to_owned()))
            .bind(("manifest",key.to_owned())).bind(("attempt",attempt.key.clone())).bind(("run",run.key.clone())))).await?;
        let index = response.num_statements().saturating_sub(2);
        let manifest = response
            .take::<Option<Object>>(index)?
            .map(wire::decode_canonical_result_manifests)
            .transpose()?
            .ok_or_else(|| invalid("closed manifest unavailable"))?;
        if manifest.key != key
            || manifest.attempt != attempt.key
            || manifest.generation != attempt.generation
        {
            return Err(invalid("terminal manifest association"));
        }
        let sets = crate::canonical_execution::decode_result_descriptors(&manifest)?;
        self.admit_manifest_ticket("result",&manifest.key,Some((&run.key,&attempt.key,&manifest.key)),owner.selection.key()).await?;
        Ok(ResultRead {
            owner,
            run,
            attempt,
            manifest,
            sets,
        })
            }).await
        }).await
    }

    /// Pin one terminal completion without fetching its manifest descriptors or
    /// seed inventory. The same result-creator fence protects retirement.
    pub async fn read_completion(
        &self,
        run_key: &str,
        attempt_key: &str,
    ) -> Result<CompletionRead, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Selection, async {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let (owner, run, attempt) = self
            .terminal_read_owner(run_key, attempt_key, crate::canonical::REQUEST_TIMEOUT)
            .await?;
        let manifest = attempt
            .closed_manifest
            .as_deref()
            .ok_or_else(|| invalid("terminal attempt lacks closed manifest"))?;
        let set = crate::canonical_execution::result_set_key(&attempt.key, "__completion");
        let mut response = self.protected_query(
            &format!("result-creators:{}", run.key), crate::canonical::QueryEffect::Mutation,
            "canonical_results::read_completion",
            || Ok(self.db.query(format!("{PROTECTED_BEGIN}\n{RESULT_ADMISSION}\nIF $attempt_record.generation != $generation {{ THROW 'PSE_REFUSAL:terminal_completion_generation_changed'; }};\nSELECT * FROM ONLY type::record('canonical_result_sets',$set);\nCOMMIT;"))
                .bind(("problem", owner.selection.revision().problem.clone()))
                .bind(("revision", owner.selection.revision().key.clone()))
                .bind(("sequence", canonical_codec::encode_uint(owner.selection.revision().sequence)?))
                .bind(("protection", owner.selection.key().to_owned()))
                .bind(("manifest", manifest.to_owned()))
                .bind(("attempt", attempt.key.clone())).bind(("run", run.key.clone()))
                .bind(("generation", canonical_codec::encode_uint(attempt.generation)?))
                .bind(("set", set.clone()))),
        ).await?;
        let index = response.num_statements().saturating_sub(2);
        let chunks = response
            .take::<Option<Object>>(index)?
            .map(wire::decode_canonical_result_sets)
            .transpose()?;
        if chunks.as_ref().is_some_and(|row| {
            row.key != set
                || row.attempt != attempt.key
                || row.name != "__completion"
                || row.interpretation != run.interpretation
        }) {
            return Err(invalid("completion set association"));
        }
        self.admit_manifest_ticket("result",manifest,Some((&run.key,&attempt.key,manifest)),owner.selection.key()).await?;
        Ok(CompletionRead {
            owner,
            run,
            attempt,
            chunks,
        })
            }).await
        }).await
    }

    /// Fetch only an ordinal in the frozen original-completion set.
    pub async fn completion_payload(
        &self,
        read: &CompletionRead,
        ordinal: u64,
    ) -> Result<ResultPayload, CanonicalError> {
        let activity = read.owner.store.selection_activity(&read.owner.selection)?;
        activity
            .scope(async {
                let set = read
                    .chunks
                    .as_ref()
                    .ok_or_else(|| invalid("completion chunks absent"))?;
                self.read_payload(
                    &read.owner,
                    &read.run,
                    &read.attempt,
                    &set.key,
                    set.next_ordinal,
                    ordinal,
                )
                .await
            })
            .await
    }

    /// Read protected block coverage and its complete exact file descriptors in
    /// one bounded native transaction. Physical opening still reauthenticates the
    /// current file incarnation and placement under the same protection.
    #[allow(
        clippy::too_many_arguments,
        reason = "selection, scientific coordinates and original ordinal independently constrain a protected page"
    )]
    pub async fn result_joined_block_page(
        &self,
        read: &ResultRead,
        result_set: &str,
        output: &str,
        partition: &str,
        start: u64,
        end: u64,
        after: Option<u64>,
    ) -> Result<JoinedResultBlockPage, CanonicalError> {
        let layout = crate::canonical::MetadataResponse::JoinedBlockPage;
        crate::canonical::transport::metadata_scope(layout, async {
            let activity = self.selection_activity(&read.owner.selection)?;
            activity.scope(async {
                self.files.require_current()?;
                if !read.belongs_to(self) { return Err(invalid("result read belongs to another store")); }
                let set = read.set(result_set)?;
                identity(output)?;
                identity(partition)?;
                if start > end { return Err(CanonicalError::PayloadLimit); }
                let query = format!("{PROTECTED_BEGIN}\nLET $rpc_result=fn::pse_execution_v1::result_block_page($pse_rpc_expires_at,$result_set,$output,$partition,$start,$end,$after,$count);\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection) {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;");
                let mut response = self.protected_query(&read.run.problem, crate::canonical::QueryEffect::Read, "canonical_results::result_joined_block_page", || Ok(self.db.metadata_query(layout, query.clone())
                    .bind(("problem",read.run.problem.clone())).bind(("revision",read.run.revision.clone()))
                    .bind(("sequence",canonical_codec::encode_uint(read.owner.selection.revision().sequence)?))
                    .bind(("protection",read.owner.selection.key().to_owned()))
                    .bind(("result_set",set.key.clone())).bind(("output",output.to_owned())).bind(("partition",partition.to_owned()))
                    .bind(("count",canonical_codec::encode_uint(set.batch_count)?)).bind(("start",canonical_codec::encode_uint(start)?))
                    .bind(("end",canonical_codec::encode_uint(end)?)).bind(("after",after.map(canonical_codec::encode_uint).transpose()?.unwrap_or(Value::None)))
                )).await?;
                let index = response.num_statements().saturating_sub(2);
                let envelope = response.take::<Option<Object>>(index)?.ok_or_else(|| invalid("joined result page absent"))?;
                decode_joined_result_page(envelope, read, set, output, partition, start, end, after)
            }).await
        }).await
    }

    /// Read one exact closed-descriptor member for its declared scientific codec.
    /// The payload remains bounded and cannot be selected from an unsealed prefix.
    pub async fn result_payload(
        &self,
        read: &ResultRead,
        result_set: &str,
        ordinal: u64,
    ) -> Result<ResultPayload, CanonicalError> {
        let activity = read.owner.store.selection_activity(&read.owner.selection)?;
        activity
            .scope(async {
                let set = read.set(result_set)?;
                self.read_payload(
                    &read.owner,
                    &read.run,
                    &read.attempt,
                    &set.key,
                    set.batch_count,
                    ordinal,
                )
                .await
            })
            .await
    }

    /// Fetch only an explicitly demanded bounded set of exact logical members.
    /// Callers prepay each page before accumulating frozen snapshot metadata.
    pub async fn result_file_batch_descriptors(
        &self,
        read: &ResultRead,
        set_key: &str,
        ordinals: &[u64],
    ) -> Result<Vec<ResultBatch>, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Page, async {
        let activity = self.selection_activity(&read.owner.selection)?;
        activity.scope(async {
        self.files.require_current()?;
        if !read.belongs_to(self){return Err(invalid("result read belongs to another store"));}
        let set=read.set(set_key)?;
        if ordinals.len()>64||ordinals.windows(2).any(|pair|pair[0]>=pair[1])||ordinals.iter().any(|ordinal|*ordinal>=set.batch_count){return Err(invalid("demanded file descriptor page exceeds exact manifest scope"));}
        if ordinals.is_empty(){return Ok(Vec::new());}
        let selected=ordinals.iter().map(|value|canonical_codec::encode_uint(*value)).collect::<Result<Vec<_>,_>>()?;
        let mut response=self.protected_query(&read.run.problem,crate::canonical::QueryEffect::Read,"canonical_results::file_descriptors",||Ok(self.db.metadata_query(crate::canonical::MetadataResponse::Page, format!("{PROTECTED_BEGIN}\nRETURN SELECT * FROM canonical_result_batches WHERE result_set=$set AND ordinal IN $selected ORDER BY ordinal LIMIT 64; COMMIT;"))
        .bind(("problem",read.run.problem.clone())).bind(("revision",read.run.revision.clone())).bind(("sequence",canonical_codec::encode_uint(read.owner.selection.revision().sequence)?)).bind(("protection",read.owner.selection.key().to_owned())).bind(("set",set_key.to_owned())).bind(("selected",selected.clone())))).await?;
        let index=response.num_statements().saturating_sub(2);let page:Vec<Object>=response.take(index)?;
        if page.len()!=ordinals.len(){return Err(invalid("frozen demanded file inventory is incomplete"));}
        page.into_iter().zip(ordinals).map(|(row,ordinal)|{let batch=wire::decode_canonical_result_batches(row)?;crate::canonical_execution::validate_result_batch(&batch)?;if batch.ordinal!=*ordinal||batch.attempt!=read.attempt.key||batch.result_set!=set_key{return Err(invalid("frozen demanded file coordinate differs"));}Ok(batch)}).collect()
        }).await
        }).await
    }
    /// Authenticate an exact frozen logical member before exposing its native file.
    pub async fn open_result_file(
        &self,
        read: &ResultRead,
        batch: &ResultBatch,
    ) -> Result<crate::canonical_files::ProtectedFile, CanonicalError> {
        crate::canonical::transport::metadata_scope(
            crate::canonical::MetadataResponse::Header,
            async {
                let activity = self.selection_activity(&read.owner.selection)?;
                activity
                    .scope(async {
                        self.files.require_current()?;
                        if !read.belongs_to(self)
                            || batch.attempt != read.attempt.key
                            || batch.ordinal >= read.set(&batch.result_set)?.batch_count
                        {
                            return Err(invalid("file batch outside admitted manifest"));
                        }
                        let original = self
                            .result_batch_descriptor(&read.owner, &read.run, batch)
                            .await?;
                        if &original != batch {
                            return Err(invalid(
                                "file batch descriptor differs from admitted catalog",
                            ));
                        }
                        let descriptor = self.file_descriptor(batch).await?;
                        self.open_physical_file(descriptor, read.owner.clone())
                    })
                    .await
            },
        )
        .await
    }
    async fn result_batch_descriptor(
        &self,
        owner: &Arc<ReadOwner>,
        run: &Run,
        batch: &ResultBatch,
    ) -> Result<ResultBatch, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Header, async {
        let mut response=self.protected_query(&run.problem,crate::canonical::QueryEffect::Read,"canonical_results::file_descriptor",||Ok(self.db.metadata_query(crate::canonical::MetadataResponse::Header, format!("{PROTECTED_BEGIN}\nRETURN SELECT * FROM ONLY type::record('canonical_result_batches',$batch); COMMIT;"))
        .bind(("problem",run.problem.clone())).bind(("revision",run.revision.clone())).bind(("sequence",canonical_codec::encode_uint(owner.selection.revision().sequence)?)).bind(("protection",owner.selection.key().to_owned())).bind(("batch",batch.key.clone())))).await?;
        let index = response.num_statements().saturating_sub(2);
        Ok(wire::decode_canonical_result_batches(
            response
                .take::<Option<Object>>(index)?
                .ok_or_else(|| invalid("file batch unavailable"))?,
        )?)
        }).await
    }
    async fn hydrate_result_batch(
        &self,
        batch: ResultBatch,
        owner: Arc<ReadOwner>,
    ) -> Result<HydratedResultBatch, CanonicalError> {
        use std::io::Read;
        let descriptor = self.file_descriptor(&batch).await?;
        let mut file = self.open_physical_file(descriptor, owner)?;
        let mut bytes = Vec::with_capacity(batch.payload_bytes as usize);
        file.read_to_end(&mut bytes)
            .map_err(|e| invalid(&e.to_string()))?;
        Ok(HydratedResultBatch {
            descriptor: batch,
            payload: bytes.into(),
            _allocation: crate::canonical::transport::current_response_owner(),
        })
    }
    async fn read_payload(
        &self,
        owner: &Arc<ReadOwner>,
        run: &Run,
        attempt: &Attempt,
        result_set: &str,
        batch_count: u64,
        ordinal: u64,
    ) -> Result<ResultPayload, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Header, async {
        if !self.db.same_driver(&owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        if ordinal >= batch_count {
            return Err(invalid("payload outside admitted descriptor"));
        }
        let key = result_batch_key(&attempt.key, result_set, ordinal);
        let mut response=self.protected_query(&run.problem, crate::canonical::QueryEffect::Read, "canonical_results::result_payload", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM ONLY type::record('canonical_result_batches',$batch) TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection) {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
            .bind(("problem",run.problem.clone())).bind(("revision",run.revision.clone()))
            .bind(("sequence",canonical_codec::encode_uint(owner.selection.revision().sequence)?))
            .bind(("protection",owner.selection.key().to_owned())).bind(("batch",key.clone())))).await?;
        let index = response.num_statements().saturating_sub(2);
        let batch = response
            .take::<Option<Object>>(index)?
            .map(wire::decode_canonical_result_batches)
            .transpose()?
            .ok_or_else(|| invalid("admitted batch unavailable"))?;
        if batch.key != key
            || batch.attempt != attempt.key
            || batch.result_set != result_set
            || batch.ordinal != ordinal
        {
            return Err(invalid("payload admitted coordinate mismatch"));
        }
        crate::canonical_execution::validate_result_batch(&batch)?;
        let batch = self.hydrate_result_batch(batch, owner.clone()).await?;
        Ok(ResultPayload {
            batch,
            owner: owner.clone(),
        })
        }).await
    }

    /// Fetch one named admitted blob. Successful checked RPC completion precedes
    /// exposing any bytes; native gRPC completion is required before a result is exposed.
    pub async fn result_block(
        &self,
        read: &ResultRead,
        metadata: BlockMetadata,
    ) -> Result<ResultBlock, CanonicalError> {
        crate::canonical::transport::metadata_scope(crate::canonical::MetadataResponse::Header, async {
        let activity = read.owner.store.selection_activity(&read.owner.selection)?;
        activity.scope(async {
        if !self.db.same_driver(&read.owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        let set = read.set(&metadata.result_set)?;
        if metadata.ordinal >= set.batch_count
            || metadata.batch != result_batch_key(&read.attempt.key, &set.key, metadata.ordinal)
        {
            return Err(invalid("block outside admitted descriptor"));
        }
        let mut response=self.protected_query(&read.run.problem, crate::canonical::QueryEffect::Read, "canonical_results::result_block", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $pse_rpc_result = {{metadata:(SELECT * FROM ONLY type::record('canonical_result_blocks',$key)),batch:(SELECT * FROM ONLY type::record('canonical_result_batches',$batch))}};\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection) {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $pse_rpc_result;\nCOMMIT;"))
            .bind(("problem",read.run.problem.clone())).bind(("revision",read.run.revision.clone()))
            .bind(("sequence",canonical_codec::encode_uint(read.owner.selection.revision().sequence)?))
            .bind(("protection",read.owner.selection.key().to_owned())).bind(("key",metadata.key.clone())).bind(("batch",metadata.batch.clone())))).await?;
        let index = response.num_statements().saturating_sub(2);
        let mut envelope = response
            .take::<Option<Object>>(index)?
            .ok_or_else(|| invalid("admitted block envelope absent"))?;
        let object = |value: Value| match value {
            Value::Object(row) => Ok(row),
            _ => Err(invalid("admitted block envelope shape")),
        };
        let recorded = wire::decode_canonical_result_blocks(object(canonical_codec::required(
            &mut envelope,
            "metadata",
        )?)?)?;
        let batch = wire::decode_canonical_result_batches(object(canonical_codec::required(
            &mut envelope,
            "batch",
        )?)?)?;
        if recorded != metadata || !envelope.is_empty() || batch.attempt != read.attempt.key {
            return Err(invalid(
                "block differs from its immutable admitted coverage",
            ));
        }
        crate::canonical_execution::validate_result_batch(&batch)?;
        validate_result_block_interpretation(&metadata, &batch, &read.run.interpretation)?;
        let batch=self.hydrate_result_batch(batch,read.owner.clone()).await?;
        Ok(ResultBlock { metadata, batch, owner: read.owner.clone() })
            }).await
        }).await
    }
}

// Exact association transaction body shared with held-commit fixture controls.
pub(crate) const RESULT_ADMISSION: &str = r#"fn::pse_execution_v1::available($pse_rpc_expires_at, $run);
LET $attempt_record=SELECT * FROM ONLY type::record('canonical_attempts',$attempt);
LET $manifest_record=SELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);
IF $attempt_record=NONE OR $attempt_record.run!=$run OR ($attempt_record.phase NOT IN ['terminal','terminal_compacted']) OR $attempt_record.closed_manifest!=$manifest OR $manifest_record=NONE OR $manifest_record.attempt!=$attempt OR $manifest_record.generation!=$attempt_record.generation { THROW 'PSE_REFUSAL:terminal_result_selection_unavailable'; };
LET $selected_pin=SELECT * FROM ONLY type::record('canonical_result_protections',$protection) FOR UPDATE;
IF $selected_pin!=NONE AND ($selected_pin.run!=$run OR $selected_pin.attempt!=$attempt OR $selected_pin.manifest!=$manifest) { THROW 'PSE_REFUSAL:immutable_result_protection_identity_reused'; };
UPSERT type::record('canonical_result_protections',$protection) SET key=$protection,run=$run,attempt=$attempt,manifest=$manifest;
fn::pse_execution_v1::result_creator($pse_rpc_expires_at,$run);"#;

// Renewal preserves exact live pin, result manifest and retirement registration.
pub(crate) const RENEW_PROTECTION: &str = r#"UPDATE type::record('canonical_protections',$protection) SET expires_at=time::micros()+$lifetime;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $pin.expires_at <= time::micros() { THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; };"#;

#[cfg(test)]
mod protection_drain_unit {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "local lifetime controls inject failed and panicking release tasks"
    )]
    use super::*;

    enum ReleaseFault {
        Cancelled,
        Panicked,
        Failed,
    }

    async fn reports_release_fault(fault: ReleaseFault) {
        // Exercise the real completion guard without poisoning an unrelated
        // server fixture with a synthetic cleanup failure.
        let drain = Arc::new(ResultReadDrain::default());
        drain.retain().unwrap();
        let release = drain.release();
        let closing = drain.close_and_drain();
        tokio::pin!(closing);
        tokio::select! {
            biased;
            result = &mut closing => panic!("pending release did not block close: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
        assert!(
            drain.retain().is_err(),
            "closing cannot admit a new exact owner"
        );
        let message = match fault {
            ReleaseFault::Cancelled => {
                let task = tokio::spawn(async move {
                    let _release = release;
                    std::future::pending::<()>().await;
                });
                task.abort();
                assert!(task.await.unwrap_err().is_cancelled());
                "result protection release task ended before completion"
            }
            ReleaseFault::Panicked => {
                let task = tokio::spawn(async move {
                    let _release = release;
                    panic!("injected result protection release panic");
                });
                assert!(task.await.unwrap_err().is_panic());
                "result protection release task ended before completion"
            }
            ReleaseFault::Failed => {
                release.complete(Err(invalid("injected result protection release failure")));
                "injected result protection release failure"
            }
        };
        assert!(
            drain.releases.is_empty(),
            "failed tasks must physically leave the tracker"
        );
        let error = closing.await.unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
        let CanonicalError::ProtectionRelease { source } = error else {
            panic!("original source missing")
        };
        for retry in [drain.drain().await, drain.close_and_drain().await] {
            let CanonicalError::ProtectionRelease { source: repeated } = retry.unwrap_err() else {
                panic!("cleanup uncertainty was consumed")
            };
            assert!(
                Arc::ptr_eq(&source, &repeated),
                "observing failure must retain its exact original cause"
            );
        }
    }

    #[tokio::test]
    async fn canonical_protection_drain_reports_cancelled_release() {
        reports_release_fault(ReleaseFault::Cancelled).await;
    }
    #[tokio::test]
    async fn canonical_protection_drain_reports_panicking_release() {
        reports_release_fault(ReleaseFault::Panicked).await;
    }
    #[tokio::test]
    async fn canonical_protection_drain_reports_failed_release() {
        reports_release_fault(ReleaseFault::Failed).await;
    }
    #[tokio::test]
    async fn pending_release_join_preserves_independent_live_reader_refusal() {
        let drain = Arc::new(ResultReadDrain::default());
        drain.retain().unwrap(); // Independent artifact reader.
        drain.retain().unwrap(); // Result reader whose last follower is dropped.
        let cleanup = drain.release();
        let joining = drain.join_pending_releases();
        tokio::pin!(joining);
        tokio::select! {
            biased;
            result = &mut joining => panic!("pending cleanup did not block join: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
        cleanup.complete(Ok(()));
        joining.await.unwrap();
        assert!(drain.drain().await.is_err());
        assert!(drain.close_and_drain().await.is_err());
        let later_cleanup = drain.release();
        let final_drain = drain.close_and_drain();
        tokio::pin!(final_drain);
        tokio::select! {
            biased;
            result = &mut final_drain => panic!("later tracked cleanup did not block drain: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
        later_cleanup.complete(Ok(()));
        final_drain.await.unwrap();
    }
    #[tokio::test]
    async fn canonical_protection_close_refusal_and_observation_keep_live_admission() {
        let drain = Arc::new(ResultReadDrain::default());
        drain.retain().unwrap();
        assert!(drain.close_and_drain().await.is_err());
        // The retained-before-close order refuses close without stranding the
        // exact live owners or making an ordinary observation terminal.
        drain.retain().unwrap();
        drain.release().complete(Ok(()));
        drain.release().complete(Ok(()));
        drain.drain().await.unwrap();
        drain.retain().unwrap();
        let release = drain.release();
        let closing = drain.close_and_drain();
        tokio::pin!(closing);
        tokio::select! {
            biased;
            result = &mut closing => panic!("pending acknowledged release did not block close: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
        // The close-before-retain order refuses new establishment. Repeating
        // the same successfully drained close remains harmless.
        assert!(drain.retain().is_err());
        release.complete(Ok(()));
        closing.await.unwrap();
        drain.close_and_drain().await.unwrap();
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_results_server_unit {
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::panic,
        reason = "isolated result fixtures assert expected outcomes and inject release-task panics"
    )]
    #![allow(
        unsafe_code,
        reason = "isolated fixtures explicitly classify their synthetic partial or failed observations"
    )]
    use super::*;
    use crate::canonical::CanonicalOptions;
    use crate::canonical_execution::{RunRequest, TerminalClass, result_set_key};
    use std::path::Path;
    async fn fixture() -> (CanonicalStore, String, crate::canonical::Revision) {
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("explicit supervised server fixture required");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_test_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let revision = store.edit("problem", None, "source-1", &[]).await.unwrap();
        (store, options.database, revision)
    }

    #[tokio::test]
    async fn canonical_results_keep_historical_manifest_after_new_terminal_attempt() {
        let (store, _database, revision) = fixture().await;
        store
            .begin_run(&RunRequest {
                key: "run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let first = store
            .claim_run("run", "claim-first", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        store
            .append_result_batch(
                &first,
                "old-bytes",
                "observations",
                0,
                crate::canonical_execution::ResultPayload::new(vec![17]).unwrap(),
                1,
            )
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&first, "close-first")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: transport-only fixture explicitly records partial availability.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal-first",
                TerminalClass::Partial,
                &[1],
                crate::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let second = store
            .claim_run("run", "claim-second", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&second, "close-second")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: the empty fixture is failed, never usable scientific success.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal-second",
                TerminalClass::Failed,
                &[2],
                crate::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let historical = store.read_results("run", first.attempt()).await.unwrap();
        assert_eq!(historical.attempt().key, first.attempt());
        assert_eq!(historical.sets().len(), 1);
        let payload = store
            .result_payload(&historical, &historical.sets()[0].key, 0)
            .await
            .unwrap();
        assert_eq!(payload.batch.payload.as_slice(), [17]);
        assert!(
            store
                .result_payload(&historical, &historical.sets()[0].key, 1)
                .await
                .is_err()
        );
        let latest = store.latest_results("problem", &[]).await.unwrap();
        assert_eq!(latest.attempt().key, second.attempt());
        assert!(latest.sets().is_empty());
        assert!(
            store
                .latest_results("problem", &[TerminalClass::Succeeded])
                .await
                .is_err()
        );
        assert!(
            store
                .latest_results("problem", &[TerminalClass::Partial])
                .await
                .is_err()
        );
        drop((payload, historical, latest));
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn canonical_results_blocks_narrow_ranges_and_payload_receipts() {
        let (store, _database, revision) = fixture().await;
        store
            .begin_run(&RunRequest {
                key: "run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("run", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let set = result_set_key(fence.attempt(), "trajectory");
        for ordinal in 0..3u64 {
            let payload = vec![ordinal as u8; 32];
            let key = result_batch_key(fence.attempt(), &set, ordinal);
            let start = ordinal * 10 + if ordinal == 2 { 5 } else { 0 };
            let metadata = BlockMetadata {
                key: key.clone(),
                result_set: set.clone(),
                batch: key,
                output: "temperature".into(),
                partition: "time".into(),
                ordinal,
                start,
                end: start + 10,
                rows: 10,
                columns: 2,
                coordinate_min: Some(start as f64),
                coordinate_max: Some((start + 9) as f64),
                payload_bytes: payload.len() as u64,
                payload_digest: crate::canonical_execution::result_payload_digest(&payload),
                statistics: None,
                interpretation: wire::INTERPRETATION.into(),
            };
            let outputs = [BlockOutput {
                key: format!("dense-{ordinal}"),
                batch: metadata.batch.clone(),
                result_set: set.clone(),
                output: "temperature".into(),
                partition: "time".into(),
                start,
                end: start + 10,
                coordinate_min: metadata.coordinate_min,
                coordinate_max: metadata.coordinate_max,
                interpretation: wire::INTERPRETATION.into(),
            }];
            store
                .append_result_block_outputs(
                    &fence,
                    &format!("write-{ordinal}"),
                    "trajectory",
                    ordinal,
                    crate::canonical_execution::ResultPayload::new(payload).unwrap(),
                    10,
                    &metadata,
                    &outputs,
                )
                .await
                .unwrap();
        }
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this transport fixture is explicitly partial and asserts no scientific success.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal",
                TerminalClass::Partial,
                &[1],
                crate::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let read = store.read_results("run", fence.attempt()).await.unwrap();
        let joined = store
            .result_joined_block_page(&read, &set, "temperature", "time", 10, 20, None)
            .await
            .unwrap();
        assert_eq!(joined.blocks.len(), 1);
        assert_eq!(joined.next, Some(1));
        assert!(joined.exhausted);
        let encoded = |blocks: &[BlockMetadata],
                       batches: &[ResultBatch],
                       next: Option<u64>,
                       exhausted: bool| {
            let mut envelope = Object::new();
            envelope.insert(
                "blocks",
                Value::Array(
                    blocks
                        .iter()
                        .map(|block| {
                            Value::Object(wire::encode_canonical_result_blocks(block).unwrap())
                        })
                        .collect(),
                ),
            );
            envelope.insert(
                "batches",
                Value::Array(
                    batches
                        .iter()
                        .map(|batch| {
                            Value::Object(wire::encode_canonical_result_batches(batch).unwrap())
                        })
                        .collect(),
                ),
            );
            envelope.insert(
                "next",
                next.map(canonical_codec::encode_uint)
                    .transpose()
                    .unwrap()
                    .unwrap_or(Value::None),
            );
            envelope.insert("exhausted", Value::Bool(exhausted));
            envelope
        };
        let decode = |envelope| {
            decode_joined_result_page(
                envelope,
                &read,
                read.set(&set).unwrap(),
                "temperature",
                "time",
                10,
                20,
                None,
            )
        };
        assert!(decode(encoded(&joined.blocks, &[], Some(1), true)).is_err());
        assert!(
            decode(encoded(
                &joined.blocks,
                &[joined.batches[0].clone(), joined.batches[0].clone()],
                Some(1),
                true
            ))
            .is_err()
        );
        for changed in 0..5 {
            let mut batches = joined.batches.clone();
            match changed {
                0 => batches[0].attempt = "foreign-attempt".into(),
                1 => batches[0].result_set = "foreign-set".into(),
                2 => batches[0].ordinal = 2,
                3 => batches[0].digest = "0".repeat(64),
                _ => batches[0].key = "foreign-member".into(),
            }
            assert!(decode(encoded(&joined.blocks, &batches, Some(1), true)).is_err());
        }
        let mut blocks = joined.blocks.clone();
        blocks[0].interpretation = "foreign-interpretation".into();
        assert!(decode(encoded(&blocks, &joined.batches, Some(1), true)).is_err());
        assert!(decode(encoded(&joined.blocks, &joined.batches, None, false)).is_err());
        assert!(decode(encoded(&[], &[], None, false)).is_err());
        assert!(
            !decode(encoded(&joined.blocks, &joined.batches, Some(1), false))
                .unwrap()
                .exhausted,
            "a byte-limited short page remains resumable"
        );
        let page = joined.blocks;
        assert_eq!(page[0].ordinal, 1);
        let resumed = store
            .result_joined_block_page(&read, &set, "temperature", "time", 10, 20, Some(0))
            .await
            .unwrap();
        assert_eq!(resumed.blocks, page);
        assert_eq!(resumed.batches, joined.batches);
        assert_eq!(resumed.next, Some(1));
        assert!(resumed.exhausted);
        for (start, end, after) in [(10, 20, Some(1)), (20, 25, None), (0, 35, Some(2))] {
            let exhausted = store
                .result_joined_block_page(&read, &set, "temperature", "time", start, end, after)
                .await
                .unwrap();
            assert!(exhausted.blocks.is_empty());
            assert!(exhausted.batches.is_empty());
            assert_eq!(exhausted.next, None);
            assert!(exhausted.exhausted);
        }
        let tail = store
            .result_joined_block_page(&read, &set, "temperature", "time", 0, 35, Some(1))
            .await
            .unwrap();
        assert_eq!(
            tail.blocks.iter().map(|m| m.ordinal).collect::<Vec<_>>(),
            vec![2]
        );
        assert_eq!(tail.next, Some(2));
        assert!(tail.exhausted);

        for (start, end, after, expected) in [
            (10, 20, None, vec![10]),
            (10, 20, Some(9), vec![10]),
            (10, 20, Some(10), vec![]),
            (10, 20, Some(20), vec![]),
            (10, 20, Some(100), vec![]),
            (20, 25, None, vec![]),
            (0, 35, Some(10), vec![25]),
            (0, 35, Some(25), vec![]),
        ] {
            let output = store
                .result_output_page(
                    &read,
                    &set,
                    "temperature",
                    "time",
                    start,
                    end,
                    after,
                    None,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                output
                    .outputs
                    .iter()
                    .map(|index| index.start)
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(output.blocks.len(), output.outputs.len());
        }
        let blob = store.result_block(&read, page[0].clone()).await.unwrap();
        assert_eq!(blob.batch.payload.as_slice(), [1u8; 32]);
        assert!(
            store
                .result_joined_block_page(&read, &set, "other", "time", 0, 30, None)
                .await
                .unwrap()
                .blocks
                .is_empty()
        );
        let mut changed = page[0].clone();
        changed.payload_digest = "wrong".into();
        assert!(store.result_block(&read, changed).await.is_err());
        // Corrupt the isolated original file while leaving its catalog digest
        // and admitted logical locators unchanged. Fresh native IO must refuse.
        let descriptor = store
            .result_file_batch_descriptors(&read, &set, &[page[0].ordinal])
            .await
            .unwrap()
            .remove(0);
        let path = store.files.path(&descriptor.file_instance).unwrap();
        std::fs::write(&path, [2u8; 32]).unwrap();
        assert!(store.result_block(&read, page[0].clone()).await.is_err());
        assert_eq!(blob.batch.payload.as_slice(), [1u8; 32]);
        drop(read);
        assert_eq!(blob.source_revision().key, "source-1");
        assert!(
            store.remove_isolated_fixture().await.is_err(),
            "returned block owner must refuse fixture teardown"
        );
        drop(blob);
        store.remove_isolated_fixture().await.unwrap();
    }
}
