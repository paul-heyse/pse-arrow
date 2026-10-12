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
    canonical_result_rows::Row as ResultRow, canonical_runs::Row as Run,
};
use std::{sync::Arc, time::Duration};
use surrealdb::types::{Object, Value};

/// Bound on independently read scientific blobs, including IPC schema and EOS.
pub const RESULT_BLOCK_BYTES: usize = wire::RESULT_BLOCK_BYTES;
const PAGE: usize = 64;
const IDENTITY_BYTES: usize = 4096;

// Closing a client must join automatic release RPCs, and must refuse while
// returned owners still protect their exact selections. Fixture teardown uses
// the same production lifetime before disposing of its isolated database.
#[derive(Debug, Default)]
pub(crate) struct ResultReadDrain {
    live: std::sync::atomic::AtomicUsize,
    closing: std::sync::Mutex<bool>,
    releases: tokio_util::task::TaskTracker,
    error: std::sync::Mutex<Option<Arc<CanonicalError>>>,
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
            _token: token,
        }
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
        self.releases.close();
        self.wait_for_releases().await
    }
    async fn wait_for_releases(&self) -> Result<(), CanonicalError> {
        tokio::time::timeout(crate::canonical::REQUEST_TIMEOUT, self.releases.wait())
            .await
            .map_err(|_| CanonicalError::Timeout)?;
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
    _token: tokio_util::task::task_tracker::TaskTrackerToken,
}
impl ResultReleaseTask {
    pub(crate) fn complete(mut self, result: Result<(), CanonicalError>) {
        if let Err(error) = result {
            self.drain.failure(error);
        }
        self.completed = true;
    }
}
impl Drop for ResultReleaseTask {
    fn drop(&mut self) {
        if !self.completed {
            self.drain.failure(invalid(
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
    owner: Arc<ReadOwner>,
    run: Run,
    attempt: Attempt,
    chunks: Option<pse_model::generated::runtime::canonical_result_sets::Row>,
}
impl CompletionRead {
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
    /// Renew only this still-live exact permission; expired pins cannot revive.
    pub async fn renew(&self, lifetime: Duration) -> Result<(), CanonicalError> {
        self.owner.renew(lifetime).await
    }
}

/// An exact terminal attempt and its closed membership, pinned before paging.
/// Failed and partial outcomes retain their actual class rather than substituting
/// another run's values. Clone retains the same protected read owner.
#[derive(Clone, Debug)]
pub struct ResultRead {
    owner: Arc<ReadOwner>,
    run: Run,
    attempt: Attempt,
    manifest: Manifest,
    sets: Vec<ResultSetDescriptor>,
}
impl ResultRead {
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
        Arc::ptr_eq(&store.db, &self.owner.store.db)
    }
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
    /// Renew protection during a long paged selection. Expired leases cannot revive.
    pub async fn renew(&self, lifetime: Duration) -> Result<(), CanonicalError> {
        self.owner.renew(lifetime).await
    }
    fn set(&self, key: &str) -> Result<&ResultSetDescriptor, CanonicalError> {
        self.sets
            .iter()
            .find(|s| s.key == key)
            .ok_or_else(|| invalid("result set outside admitted descriptor"))
    }
}
impl ReadOwner {
    async fn renew(&self, lifetime: Duration) -> Result<(), CanonicalError> {
        let micros =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if micros <= 0 {
            return Err(invalid("positive read protection lifetime required"));
        }
        let selection = &self.selection;
        self.store
            .protected_query(
                &format!("pin:{}", selection.key()),
                crate::canonical::QueryEffect::Mutation,
                "canonical_results::renew",
                || {
                    Ok(self
                        .store
                        .db
                        .query(format!("{PROTECTED_BEGIN}\n{RENEW_PROTECTION}\nCOMMIT;"))
                        .bind(("problem", selection.revision().problem.clone()))
                        .bind(("revision", selection.revision().key.clone()))
                        .bind((
                            "sequence",
                            canonical_codec::encode_uint(selection.revision().sequence)?,
                        ))
                        .bind(("protection", selection.key().to_owned()))
                        .bind(("lifetime", micros)))
                },
            )
            .await?;
        Ok(())
    }
}

/// One exact bounded blob retains read ownership until its consumer releases it.
#[derive(Clone, Debug)]
pub struct ResultBlock {
    /// Narrow recorded shape, range and interpretation.
    pub metadata: BlockMetadata,
    /// Exact immutable batch; payload is not a queryable numeric projection.
    pub batch: ResultBatch,
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
    pub batch: ResultBatch,
    owner: Arc<ReadOwner>,
}
impl ResultPayload {
    /// Source protection remains owned while the encoded buffer is consumed.
    pub fn source_revision(&self) -> &crate::canonical::Revision {
        self.owner.selection.revision()
    }
}

/// Narrow output indexes and their exact admitted original IPC block metadata.
#[derive(Clone, Debug)]
pub struct ResultOutputPage {
    /// Complete typed scalar row locators; verified against IPC on decode.
    pub rows: Vec<ResultRow>,
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

/// Check the relation-owned complete typed scientific key and independently
/// derived access selectors. Original IPC comparison remains the read boundary.
pub fn validate_result_row(locator: &ResultRow) -> Result<(), CanonicalError> {
    if locator.interpretation != wire::INTERPRETATION {
        return Err(invalid("result row interpretation"));
    }
    for value in [
        &locator.key,
        &locator.result_set,
        &locator.attempt,
        &locator.batch,
        &locator.output,
        &locator.partition,
    ] {
        identity(value)?;
    }
    let registry =
        pse_schema::registry().map_err(|error| CanonicalError::Configuration(error.to_string()))?;
    let relation = registry
        .relation_by_id(locator.relation_id)
        .ok_or_else(|| invalid("unknown scalar row relation"))?;
    if relation.key.namespace != pse_schema::model::Namespace::Runtime {
        return Err(invalid("scalar row relation namespace"));
    }
    let partition = match relation.key.name {
        "solve_variables"
            if relation.primary_key == ["run_id", "step", "symbol_id"]
                && locator.experiment_id.is_none() =>
        {
            format!(
                "step:{}",
                locator
                    .step
                    .filter(|step| *step >= 0)
                    .ok_or_else(|| invalid("solve variable row lacks nonnegative step"))?
            )
        }
        "solve_constraints"
            if relation.primary_key == ["run_id", "step", "row_id"]
                && locator.experiment_id.is_none() =>
        {
            format!(
                "step:{}",
                locator
                    .step
                    .filter(|step| *step >= 0)
                    .ok_or_else(|| invalid("solve constraint row lacks nonnegative step"))?
            )
        }
        "fit_parameters"
            if relation.primary_key == ["run_id", "parameter_id"]
                && locator.step.is_none()
                && locator.experiment_id.is_none() =>
        {
            "0".into()
        }
        "fit_observations"
            if relation.primary_key == ["run_id", "observation_id"] && locator.step.is_none() =>
        {
            format!(
                "experiment:{}",
                locator
                    .experiment_id
                    .ok_or_else(|| invalid("fit observation row lacks experiment selector"))?
            )
        }
        _ => {
            return Err(invalid(
                "scalar row does not encode its complete registry key",
            ));
        }
    };
    if locator.output != format!("{}:{}", locator.relation_id, locator.owner)
        || locator.partition != partition
        || locator.key != format!("{}:{}", locator.batch, locator.row)
    {
        return Err(invalid(
            "scalar row access selector or identity differs from typed key",
        ));
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
        || metadata.payload_bytes != batch.payload.as_slice().len() as u64
        || metadata.payload_bytes > RESULT_BLOCK_BYTES as u64
        || metadata.columns > 256
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

impl CanonicalStore {
    /// Select a bounded output index page natively, before fetching any original
    /// scientific payload. Scalar predicates belong to decoded original columns;
    /// dense coordinate predicates still select recorded time bounds.
    #[allow(
        clippy::too_many_arguments,
        reason = "protected selection, output coordinates, cursor and finite scalar predicates are explicit native read premises"
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
        dense: bool,
        minimum: Option<f64>,
        maximum: Option<f64>,
        missing: Option<bool>,
    ) -> Result<ResultOutputPage, CanonicalError> {
        if !Arc::ptr_eq(&self.db, &read.owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        let set = read.set(result_set)?;
        identity(output)?;
        identity(partition)?;
        if start > end
            || minimum.is_some_and(|v| !v.is_finite())
            || maximum.is_some_and(|v| !v.is_finite())
            || minimum.zip(maximum).is_some_and(|(min, max)| min > max)
            || missing == Some(true) && (minimum.is_some() || maximum.is_some())
        {
            return Err(invalid("invalid output range/predicate"));
        }
        if !dense && (minimum.is_some() || maximum.is_some() || missing.is_some()) {
            return Err(invalid(
                "scalar predicates require decoded original columns",
            ));
        }
        let table = if dense {
            "canonical_result_block_outputs"
        } else {
            "canonical_result_rows"
        };
        // SurrealDB 3.3 selects the first range on a compound index column.
        // Resume from a cursor within the selected range; outside that window,
        // keep the selected range first and apply the exact cursor as a residual.
        let cursor_within_range = after.is_some_and(|cursor| cursor >= start && cursor < end);
        let cursor = if cursor_within_range {
            if dense {
                "start>$after AND "
            } else {
                "row>$after AND "
            }
        } else {
            ""
        };
        let residual_cursor = if after.is_some() && !cursor_within_range {
            if dense {
                " AND start>$after"
            } else {
                " AND row>$after"
            }
        } else {
            ""
        };
        let bounds = if dense {
            "end>$start AND start<$end AND ($minimum=NONE OR coordinate_max.projection>=$minimum) AND ($maximum=NONE OR coordinate_min.projection<=$maximum)"
        } else {
            "row>=$start AND row<$end"
        };
        let order = if dense { "start" } else { "row" };
        let query = format!(
            "{PROTECTED_BEGIN}\nLET $indexes=SELECT * FROM {table} WHERE result_set=$result_set AND output=$output AND partition=$partition AND {cursor}{bounds}{residual_cursor} ORDER BY {order} LIMIT {PAGE};\nLET $blocks=SELECT * FROM canonical_result_blocks WHERE result_set=$result_set AND key IN $indexes.batch ORDER BY ordinal LIMIT 65;\nLET $pse_rpc_result = {{indexes:$indexes,blocks:$blocks}};\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'immutable selection protection expired'; }};\nRETURN $pse_rpc_result;\nCOMMIT;"
        );
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
                        .query(query.clone())
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
                        .bind(("maximum", finite(maximum)))
                        .bind(("missing", missing.map(Value::Bool).unwrap_or(Value::None))))
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
        if !object.is_empty() || indexes.len() > PAGE || blocks.len() > PAGE {
            return Err(CanonicalError::PayloadLimit);
        }
        for block in &blocks {
            if block.result_set != set.key
                || block.ordinal >= set.batch_count
                || block.batch != result_batch_key(&read.attempt.key, &set.key, block.ordinal)
                || block.key != block.batch
            {
                return Err(invalid("output index block outside exact manifest"));
            }
        }
        let mut page = ResultOutputPage {
            rows: Vec::new(),
            outputs: Vec::new(),
            blocks,
        };
        let mut previous = after;
        for index in indexes {
            let (key, batch, recorded_output, recorded_partition, lower, upper) = if dense {
                let item = wire::decode_canonical_result_block_outputs(index)?;
                let result = (
                    item.result_set.clone(),
                    item.batch.clone(),
                    item.output.clone(),
                    item.partition.clone(),
                    item.start,
                    item.end,
                );
                if item.interpretation != read.run.interpretation {
                    return Err(invalid("output index interpretation"));
                }
                page.outputs.push(item);
                result
            } else {
                let item = wire::decode_canonical_result_rows(index)?;
                validate_result_row(&item)?;
                if item.attempt != read.attempt.key || item.relation_id.to_string() != set.name {
                    return Err(invalid("scalar row belongs to another attempt or relation"));
                }
                let result = (
                    item.result_set.clone(),
                    item.batch.clone(),
                    item.output.clone(),
                    item.partition.clone(),
                    item.row,
                    item.row
                        .checked_add(1)
                        .ok_or(CanonicalError::PayloadLimit)?,
                );
                if !page.blocks.iter().any(|block| {
                    block.batch == item.batch
                        && block.start.checked_add(item.offset) == Some(item.row)
                }) {
                    return Err(invalid("scalar row offset differs from original block"));
                }
                page.rows.push(item);
                result
            };
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
        Ok(page)
    }
    /// Select the newest semantic run whose current terminal attempt has an
    /// explicitly requested recorded class. Empty classes include all outcomes.
    /// The problem/sequence native index orders discovery without reading blobs.
    pub async fn latest_results(
        &self,
        problem: &str,
        classes: &[crate::canonical_execution::TerminalClass],
        lifetime: Duration,
    ) -> Result<ResultRead, CanonicalError> {
        identity(problem)?;
        if classes.len() > 4 {
            return Err(invalid("terminal result class bound"));
        }
        let classes = classes
            .iter()
            .map(|class| class.as_str().to_owned())
            .collect::<Vec<_>>();
        let mut response=bounded_query(self.db.query("SELECT VALUE {run:key,attempt:terminal_attempt} FROM canonical_runs WHERE problem=$problem AND terminal_attempt != NONE AND terminal_class != NONE AND (array::len($classes)=0 OR terminal_class IN $classes) ORDER BY sequence DESC LIMIT 1;").bind(("problem",problem.to_owned())).bind(("classes",classes.clone()))).await?;
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
        let read = self.read_results(&run, &attempt, lifetime).await?;
        if read.run.problem != problem
            || (!classes.is_empty()
                && !read
                    .attempt
                    .outcome
                    .as_ref()
                    .is_some_and(|class| classes.contains(class)))
        {
            return Err(invalid("latest result selection changed meaning"));
        }
        Ok(read)
    }

    async fn terminal_read_owner(
        &self,
        run_key: &str,
        attempt_key: &str,
        lifetime: Duration,
    ) -> Result<(Arc<ReadOwner>, Run, Attempt), CanonicalError> {
        identity(run_key)?;
        identity(attempt_key)?;
        let mut response=bounded_query(self.db.query("SELECT * FROM ONLY type::record('canonical_runs',$run); SELECT * FROM ONLY type::record('canonical_attempts',$attempt);")
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
            || !attempt.terminal
            || !attempt.closed
            || attempt.ingestion_open
            || !attempt.outcome.as_deref().is_some_and(|class| {
                matches!(class, "succeeded" | "failed" | "partial" | "cancelled")
            })
            || !crate::canonical::historical_product_interpretation_supported(&run.interpretation)
        {
            return Err(invalid("attempt is not an admitted terminal selection"));
        }
        if run.interpretation != wire::INTERPRETATION {
            let mut response = bounded_query(
                self.db
                    .query(
                        "SELECT * FROM ONLY type::record('canonical_historical_results',$attempt);",
                    )
                    .bind(("attempt", attempt.key.clone())),
            )
            .await?;
            let admission = response
                .take::<Option<Object>>(0)?
                .map(wire::decode_canonical_historical_results)
                .transpose()?
                .ok_or_else(|| {
                    invalid("historical result has no activated complete read admission")
                })?;
            if admission.key != attempt.key
                || admission.attempt != attempt.key
                || admission.run != run.key
                || admission.revision != run.revision
                || admission.sequence != run.source_sequence
                || admission.generation != attempt.generation
                || Some(admission.manifest.as_str()) != attempt.closed_manifest.as_deref()
                || Some(admission.outcome.as_str()) != attempt.outcome.as_deref()
                || admission.interpretation != run.interpretation
                || admission.issuing_store != self.issuing_store()?
                || admission.migration.is_empty()
                || admission.snapshot.len() != 64
                || admission.source_digest.len() != 64
            {
                return Err(invalid(
                    "historical result read admission differs from its exact original attempt",
                ));
            }
            let mut response = bounded_query(
                self.db
                    .query(
                        "SELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);",
                    )
                    .bind(("manifest", admission.manifest)),
            )
            .await?;
            let manifest = response
                .take::<Option<Object>>(0)?
                .map(wire::decode_canonical_result_manifests)
                .transpose()?
                .ok_or_else(|| invalid("historical admitted manifest absent"))?;
            if manifest.digest != admission.manifest_digest
                || manifest.attempt != attempt.key
                || manifest.generation != attempt.generation
            {
                return Err(invalid("historical admitted manifest differs"));
            }
            crate::canonical_execution::decode_result_descriptors(&manifest)?;
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
    }

    /// Resolve only an admitted terminal attempt, then pin its immutable revision
    /// before reading the exact manifest. Historical terminal attempts remain
    /// selectable even after the semantic run starts a new attempt.
    pub async fn read_results(
        &self,
        run_key: &str,
        attempt_key: &str,
        lifetime: Duration,
    ) -> Result<ResultRead, CanonicalError> {
        let (owner, run, attempt) = self
            .terminal_read_owner(run_key, attempt_key, lifetime)
            .await?;
        let key = attempt
            .closed_manifest
            .as_deref()
            .ok_or_else(|| invalid("terminal attempt lacks closed manifest"))?;
        let mut response=self.protected_query(&format!("result-creators:{}",run.key), crate::canonical::QueryEffect::Mutation, "canonical_results::read_results", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\n{RESULT_ADMISSION}\nSELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);\nCOMMIT;"))
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
        Ok(ResultRead {
            owner,
            run,
            attempt,
            manifest,
            sets,
        })
    }

    /// Pin one terminal completion without fetching its manifest descriptors or
    /// seed inventory. The same result-creator fence protects retirement.
    pub async fn read_completion(
        &self,
        run_key: &str,
        attempt_key: &str,
        lifetime: Duration,
    ) -> Result<CompletionRead, CanonicalError> {
        let (owner, run, attempt) = self
            .terminal_read_owner(run_key, attempt_key, lifetime)
            .await?;
        let manifest = attempt
            .closed_manifest
            .as_deref()
            .ok_or_else(|| invalid("terminal attempt lacks closed manifest"))?;
        let set = crate::canonical_execution::result_set_key(&attempt.key, "__completion");
        let mut response = self.protected_query(
            &format!("result-creators:{}", run.key), crate::canonical::QueryEffect::Mutation,
            "canonical_results::read_completion",
            || Ok(self.db.query(format!("{PROTECTED_BEGIN}\n{RESULT_ADMISSION}\nIF $attempt_record.generation != $generation {{ THROW 'terminal completion generation changed'; }};\nSELECT * FROM ONLY type::record('canonical_result_sets',$set);\nCOMMIT;"))
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
        Ok(CompletionRead {
            owner,
            run,
            attempt,
            chunks,
        })
    }

    /// Fetch only an ordinal in the frozen original-completion set.
    pub async fn completion_payload(
        &self,
        read: &CompletionRead,
        ordinal: u64,
    ) -> Result<ResultPayload, CanonicalError> {
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
    }

    /// Discover at most one narrow metadata page for an admitted output range.
    /// Decimal bounds retain the full unsigned domain; payload is never projected.
    #[allow(
        clippy::too_many_arguments,
        reason = "the protected selection, output identity, row window and page cursor independently bound this read"
    )]
    pub async fn result_block_page(
        &self,
        read: &ResultRead,
        result_set: &str,
        output: &str,
        partition: &str,
        start: u64,
        end: u64,
        after: Option<u64>,
    ) -> Result<Vec<BlockMetadata>, CanonicalError> {
        if !Arc::ptr_eq(&self.db, &read.owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        let set = read.set(result_set)?;
        identity(output)?;
        identity(partition)?;
        if start > end {
            return Err(CanonicalError::PayloadLimit);
        }
        // SurrealDB 3.3 chooses the first range on a compound index column.
        // Put the current cursor before the manifest's upper bound, preserving
        // the same residual extent checks and ordered 64-row page.
        let cursor = if after.is_some() {
            "ordinal>$after AND "
        } else {
            ""
        };
        let mut response=self.protected_query(&read.run.problem, crate::canonical::QueryEffect::Read, "canonical_results::result_block_page", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM canonical_result_blocks WHERE result_set=$result_set AND output=$output AND partition=$partition AND {cursor}ordinal<$count AND end>$start AND start<$end ORDER BY ordinal LIMIT {PAGE} TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'immutable selection protection expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
            .bind(("problem",read.run.problem.clone())).bind(("revision",read.run.revision.clone()))
            .bind(("sequence",canonical_codec::encode_uint(read.owner.selection.revision().sequence)?))
            .bind(("protection",read.owner.selection.key().to_owned()))
            .bind(("result_set",set.key.clone())).bind(("output",output.to_owned())).bind(("partition",partition.to_owned()))
            .bind(("count",canonical_codec::encode_uint(set.batch_count)?)).bind(("start",canonical_codec::encode_uint(start)?))
            .bind(("end",canonical_codec::encode_uint(end)?)).bind(("after",after.map(canonical_codec::encode_uint).transpose()?.unwrap_or(Value::None))))).await?;
        let index = response.num_statements().saturating_sub(2);
        let rows = response.take::<Vec<Object>>(index)?;
        if rows.len() > PAGE {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut previous = after;
        rows.into_iter()
            .map(|row| {
                let metadata = wire::decode_canonical_result_blocks(row)?;
                if previous.is_some_and(|ordinal| metadata.ordinal <= ordinal)
                    || metadata.key
                        != result_batch_key(&read.attempt.key, &set.key, metadata.ordinal)
                    || metadata.result_set != set.key
                    || metadata.ordinal >= set.batch_count
                    || metadata.output != output
                    || metadata.partition != partition
                    || metadata.end <= start
                    || metadata.start >= end
                    || metadata.interpretation != read.run.interpretation
                {
                    return Err(invalid("block metadata outside exact admitted selection"));
                }
                previous = Some(metadata.ordinal);
                Ok(metadata)
            })
            .collect()
    }

    /// Read one exact closed-descriptor member for its declared scientific codec.
    /// The payload remains bounded and cannot be selected from an unsealed prefix.
    pub async fn result_payload(
        &self,
        read: &ResultRead,
        result_set: &str,
        ordinal: u64,
    ) -> Result<ResultPayload, CanonicalError> {
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
        if !Arc::ptr_eq(&self.db, &owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        if ordinal >= batch_count {
            return Err(invalid("payload outside admitted descriptor"));
        }
        let key = result_batch_key(&attempt.key, result_set, ordinal);
        let mut response=self.protected_query(&run.problem, crate::canonical::QueryEffect::Read, "canonical_results::result_payload", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM ONLY type::record('canonical_result_batches',$batch) TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'immutable selection protection expired'; }};\nRETURN $rpc_result;\nCOMMIT;"))
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
        Ok(ResultPayload {
            batch,
            owner: owner.clone(),
        })
    }

    /// Fetch one named admitted blob. Successful checked RPC completion precedes
    /// exposing any bytes; native WebSocket completion is required before a result is exposed.
    pub async fn result_block(
        &self,
        read: &ResultRead,
        metadata: BlockMetadata,
    ) -> Result<ResultBlock, CanonicalError> {
        if !Arc::ptr_eq(&self.db, &read.owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        let set = read.set(&metadata.result_set)?;
        if metadata.ordinal >= set.batch_count
            || metadata.batch != result_batch_key(&read.attempt.key, &set.key, metadata.ordinal)
        {
            return Err(invalid("block outside admitted descriptor"));
        }
        let mut response=self.protected_query(&read.run.problem, crate::canonical::QueryEffect::Read, "canonical_results::result_block", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $pse_rpc_result = {{metadata:(SELECT * FROM ONLY type::record('canonical_result_blocks',$key)),batch:(SELECT * FROM ONLY type::record('canonical_result_batches',$batch))}};\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() {{ THROW 'immutable selection protection expired'; }};\nRETURN $pse_rpc_result;\nCOMMIT;"))
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
        Ok(ResultBlock {
            metadata,
            batch,
            owner: read.owner.clone(),
        })
    }
}

#[cfg(test)]
mod canonical_results_unit {
    use super::*;
    #[test]
    fn scalar_row_locator_requires_full_typed_registry_key_and_derived_selectors() {
        let registry = pse_schema::registry().unwrap();
        let relation = registry.relation("runtime.solve_variables").unwrap().id;
        let owner = pse_ids::SemanticId::from_bytes([1; 16]);
        let locator = ResultRow {
            key: "batch:9007199254740993".into(),
            result_set: "set".into(),
            attempt: "attempt".into(),
            batch: "batch".into(),
            relation_id: relation,
            run_id: pse_ids::SemanticId::from_bytes([2; 16]).into(),
            owner,
            step: Some(9007199254740993),
            experiment_id: None,
            output: format!("{relation}:{owner}"),
            partition: "step:9007199254740993".into(),
            row: 9007199254740993,
            offset: 1,
            interpretation: wire::INTERPRETATION.into(),
        };
        assert!(
            validate_result_row(&locator).is_ok(),
            "integer scientific keys never pass through Float64"
        );
        for invalid in [
            ResultRow {
                step: None,
                ..locator.clone()
            },
            ResultRow {
                step: Some(-1),
                ..locator.clone()
            },
            ResultRow {
                experiment_id: Some(owner),
                ..locator.clone()
            },
            ResultRow {
                output: format!("{relation}:{owner}:value"),
                ..locator.clone()
            },
            ResultRow {
                partition: "step:1".into(),
                ..locator.clone()
            },
            ResultRow {
                key: "alias".into(),
                ..locator.clone()
            },
        ] {
            assert!(validate_result_row(&invalid).is_err());
        }
        let fit = registry.relation("runtime.fit_parameters").unwrap().id;
        assert!(
            validate_result_row(&ResultRow {
                relation_id: fit,
                step: None,
                output: format!("{fit}:{owner}"),
                partition: "0".into(),
                ..locator.clone()
            })
            .is_ok()
        );
        let observation = registry.relation("runtime.fit_observations").unwrap().id;
        let observation = ResultRow {
            relation_id: observation,
            step: None,
            experiment_id: Some(owner),
            output: format!("{observation}:{owner}"),
            partition: format!("experiment:{owner}"),
            ..locator
        };
        assert!(validate_result_row(&observation).is_ok());
        assert!(
            validate_result_row(&ResultRow {
                experiment_id: None,
                ..observation.clone()
            })
            .is_err()
        );
        assert!(
            validate_result_row(&ResultRow {
                step: Some(0),
                ..observation
            })
            .is_err()
        );
    }
}

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
    async fn canonical_results_scalar_exact_admission_and_historical_attempts() {
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
            .claim_run("run", "claim-1", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let relation = pse_schema::registry()
            .unwrap()
            .relation("runtime.fit_parameters")
            .unwrap()
            .id;
        let name = relation.to_string();
        let set = result_set_key(first.attempt(), &name);
        let key = result_batch_key(first.attempt(), &set, 0);
        let owner = pse_ids::SemanticId::from_bytes([3; 16]);
        let output = format!("{relation}:{owner}");
        let mut row_indexes = (0..130u64)
            .map(|row| ResultRow {
                key: format!("{key}:{row}"),
                result_set: set.clone(),
                attempt: first.attempt().into(),
                batch: key.clone(),
                relation_id: relation,
                run_id: pse_ids::SemanticId::from_bytes([(row + 1) as u8; 16]).into(),
                owner,
                step: None,
                experiment_id: None,
                output: output.clone(),
                partition: "0".into(),
                row,
                offset: row,
                interpretation: wire::INTERPRETATION.into(),
            })
            .collect::<Vec<_>>();
        row_indexes.sort_by(|left, right| left.key.cmp(&right.key));
        let payload = vec![1, 2, 3];
        let block = BlockMetadata {
            key: key.clone(),
            batch: key,
            result_set: set.clone(),
            output: name.clone(),
            partition: "0".into(),
            ordinal: 0,
            start: 0,
            end: 130,
            rows: 130,
            columns: 1,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: payload.len() as u64,
            payload_digest: crate::canonical_execution::result_payload_digest(&payload),
            interpretation: wire::INTERPRETATION.into(),
        };
        // This mechanism fixture tests exact native indexing/receipts. The runtime
        // output control separately admits and verifies actual scientific IPC rows.
        for mutation in 0..4 {
            let mut invalid = row_indexes.clone();
            match mutation {
                0 => invalid[0].attempt = "foreign-attempt".into(),
                1 => invalid[0].offset = 130,
                2 => {
                    invalid.pop();
                }
                _ => invalid[1].run_id = invalid[0].run_id,
            }
            assert!(
                store
                    .append_result_block_rows(
                        &first,
                        &format!("invalid-locator-{mutation}"),
                        &name,
                        0,
                        crate::canonical_execution::ResultPayload::new(payload.clone()).unwrap(),
                        130,
                        &block,
                        &invalid,
                    )
                    .await
                    .is_err()
            );
        }
        store
            .append_result_block_rows(
                &first,
                "locator-write",
                &name,
                0,
                crate::canonical_execution::ResultPayload::new(payload.clone()).unwrap(),
                130,
                &block,
                &row_indexes,
            )
            .await
            .unwrap();
        let mut changed = row_indexes.clone();
        changed[0].owner = pse_ids::SemanticId::from_bytes([4; 16]);
        changed[0].output = format!("{relation}:{}", changed[0].owner);
        assert!(
            store
                .append_result_block_rows(
                    &first,
                    "changed-locator-replay",
                    &name,
                    0,
                    crate::canonical_execution::ResultPayload::new(payload).unwrap(),
                    130,
                    &block,
                    &changed,
                )
                .await
                .is_err(),
            "immutable batch replay cannot replace a scalar typed key"
        );
        assert!(
            store
                .read_results("run", first.attempt(), Duration::from_secs(30))
                .await
                .is_err()
        );
        let closed = store
            .close_result_ingestion(&first, "close-1")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture's retained scalar is explicitly partial, never scientific success.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal-1",
                TerminalClass::Partial,
                &[1],
                crate::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let read = store
            .read_results("run", first.attempt(), Duration::from_secs(30))
            .await
            .unwrap();
        let values = store
            .result_output_page(
                &read, &set, &output, "0", 0, 1, None, false, None, None, None,
            )
            .await
            .unwrap();
        assert_eq!(values.rows[0].row, 0);
        // Scalar pages retain the selected range when the cursor precedes it,
        // then resume exclusively within that range and exhaust after its end.
        for (after, expected) in [
            (Some(50), (80..90).collect::<Vec<_>>()),
            (Some(80), (81..90).collect()),
            (Some(85), (86..90).collect()),
            (Some(90), vec![]),
            (Some(95), vec![]),
            (Some(500), vec![]),
        ] {
            let page = store
                .result_output_page(
                    &read, &set, &output, "0", 80, 90, after, false, None, None, None,
                )
                .await
                .unwrap();
            assert_eq!(
                page.rows
                    .iter()
                    .map(|locator| locator.row)
                    .collect::<Vec<_>>(),
                expected
            );
        }
        for (after, expected) in [
            (None, (0..64).collect::<Vec<_>>()),
            (Some(63), (64..128).collect()),
            (Some(127), vec![128, 129]),
            (Some(129), vec![]),
        ] {
            let page = store
                .result_output_page(
                    &read, &set, &output, "0", 0, 130, after, false, None, None, None,
                )
                .await
                .unwrap();
            assert_eq!(
                page.rows
                    .iter()
                    .map(|locator| locator.row)
                    .collect::<Vec<_>>(),
                expected
            );
        }
        assert_eq!(read.attempt().outcome.as_deref(), Some("partial"));
        assert!(store.result_payload(&read, &set, 1).await.is_err());
        read.renew(Duration::from_secs(30)).await.unwrap();
        let second = store
            .claim_run("run", "claim-2", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&second, "close-2")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: the empty fixture outcome is explicitly failed, never usable scientific success.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal-2",
                TerminalClass::Failed,
                &[2],
                crate::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let historical = store
            .read_results("run", first.attempt(), Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(
            store
                .result_output_page(
                    &historical,
                    &set,
                    &output,
                    "0",
                    0,
                    1,
                    None,
                    false,
                    None,
                    None,
                    None
                )
                .await
                .unwrap()
                .rows
                .len(),
            1
        );
        let failed = store
            .read_results("run", second.attempt(), Duration::from_secs(30))
            .await
            .unwrap();
        assert!(failed.sets().is_empty());
        assert_eq!(failed.attempt().outcome.as_deref(), Some("failed"));
        let latest = store
            .latest_results("problem", &[], Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(latest.attempt().key, second.attempt());
        assert!(
            store
                .latest_results(
                    "problem",
                    &[TerminalClass::Succeeded],
                    Duration::from_secs(30)
                )
                .await
                .is_err()
        );
        assert!(
            store
                .latest_results(
                    "problem",
                    &[TerminalClass::Partial],
                    Duration::from_secs(30)
                )
                .await
                .is_err()
        );
        drop(latest);
        drop(values);
        drop(read);
        drop(historical);
        drop(failed);
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
                .append_result_block_indexes(
                    &fence,
                    &format!("write-{ordinal}"),
                    "trajectory",
                    ordinal,
                    crate::canonical_execution::ResultPayload::new(payload).unwrap(),
                    10,
                    &metadata,
                    &[],
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
        let read = store
            .read_results("run", fence.attempt(), Duration::from_secs(30))
            .await
            .unwrap();
        let page = store
            .result_block_page(&read, &set, "temperature", "time", 10, 20, None)
            .await
            .unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ordinal, 1);
        let resumed = store
            .result_block_page(&read, &set, "temperature", "time", 10, 20, Some(0))
            .await
            .unwrap();
        assert_eq!(resumed, page);
        assert!(
            store
                .result_block_page(&read, &set, "temperature", "time", 10, 20, Some(1))
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .result_block_page(&read, &set, "temperature", "time", 20, 25, None)
                .await
                .unwrap()
                .is_empty()
        );
        let tail = store
            .result_block_page(&read, &set, "temperature", "time", 0, 35, Some(1))
            .await
            .unwrap();
        assert_eq!(tail.iter().map(|m| m.ordinal).collect::<Vec<_>>(), vec![2]);
        assert!(
            store
                .result_block_page(&read, &set, "temperature", "time", 0, 35, Some(2))
                .await
                .unwrap()
                .is_empty()
        );

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
                    true,
                    None,
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
                .result_block_page(&read, &set, "other", "time", 0, 30, None)
                .await
                .unwrap()
                .is_empty()
        );
        let mut changed = page[0].clone();
        changed.payload_digest = "wrong".into();
        assert!(store.result_block(&read, changed).await.is_err());
        // Corrupt only this UUID-isolated fixture's stored payload, leaving the
        // acknowledged metadata and digest intact. A fresh read must verify the
        // bytes again, even though an earlier immutable owner remains valid.
        let tampered = bounded_query(
            store
                .db
                .query(
                    "UPDATE type::record('canonical_result_batches',$batch) SET payload=$payload;",
                )
                .bind(("batch", page[0].batch.clone()))
                .bind((
                    "payload",
                    canonical_codec::encode_bytes(vec![2u8; 32].into()).unwrap(),
                )),
        )
        .await
        .unwrap();
        tampered.check().unwrap();
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

// Exact association transaction body shared with held-commit fixture controls.
pub(crate) const RESULT_ADMISSION: &str = r#"fn::pse_execution_v1::available($pse_rpc_expires_at, $run);
LET $attempt_record=SELECT * FROM ONLY type::record('canonical_attempts',$attempt);
LET $manifest_record=SELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);
IF $attempt_record=NONE OR $attempt_record.run!=$run OR !$attempt_record.terminal OR !$attempt_record.closed OR $attempt_record.ingestion_open OR $attempt_record.closed_manifest!=$manifest OR $manifest_record=NONE OR $manifest_record.attempt!=$attempt OR $manifest_record.generation!=$attempt_record.generation { THROW 'terminal result selection unavailable'; };
LET $selected_pin=SELECT * FROM ONLY type::record('canonical_result_protections',$protection) FOR UPDATE;
IF $selected_pin!=NONE AND ($selected_pin.run!=$run OR $selected_pin.attempt!=$attempt OR $selected_pin.manifest!=$manifest) { THROW 'immutable result protection identity reused'; };
UPSERT type::record('canonical_result_protections',$protection) SET key=$protection,run=$run,attempt=$attempt,manifest=$manifest;
fn::pse_execution_v1::result_creator($pse_rpc_expires_at,$run);"#;

// Renewal preserves exact live pin, result manifest and retirement registration.
pub(crate) const RENEW_PROTECTION: &str = r#"UPDATE type::record('canonical_protections',$protection) SET expires_at=time::micros()+$lifetime;
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $pin.expires_at <= time::micros() { THROW 'immutable selection protection expired'; };"#;
