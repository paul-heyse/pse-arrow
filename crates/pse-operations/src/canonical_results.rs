// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact admitted result reads. Metadata discovery does not fetch trajectory
//! blobs; each later read names one immutable member of the closed descriptor.

use crate::{
    canonical::{
        CanonicalError, CanonicalStore, PROTECTED_BEGIN, ProtectedSelection, bounded_query,
        protected_query,
    },
    canonical_codec,
    canonical_execution::{ResultSetDescriptor, result_batch_key},
    generated::surreal as wire,
};
use pse_model::generated::runtime::{
    canonical_attempts::Row as Attempt, canonical_result_batches::Row as ResultBatch,
    canonical_result_block_outputs::Row as BlockOutput,
    canonical_result_blocks::Row as BlockMetadata, canonical_result_cells::Row as ResultCell,
    canonical_result_manifests::Row as Manifest, canonical_runs::Row as Run,
};
use std::{sync::Arc, time::Duration};
use surrealdb::types::{Object, Value};

/// Bound on independently read scientific blobs, including IPC schema and EOS.
pub const RESULT_BLOCK_BYTES: usize = wire::RESULT_BLOCK_BYTES;
const PAGE: usize = 64;
const IDENTITY_BYTES: usize = 4096;

// Fixture teardown must wait for automatic release RPCs, and may not remove a
// database while any returned owner still protects its exact result selection.
#[cfg(any(feature = "test-support", feature = "canonical-tests"))]
#[derive(Debug, Default)]
pub(crate) struct ResultReadDrain {
    live: std::sync::atomic::AtomicUsize,
    pending: std::sync::atomic::AtomicUsize,
    changed: tokio::sync::Notify,
    error: std::sync::Mutex<Option<CanonicalError>>,
}
#[cfg(any(feature = "test-support", feature = "canonical-tests"))]
impl ResultReadDrain {
    fn retain(&self) {
        self.live.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    fn release(self: &Arc<Self>) -> ResultReleaseTask {
        self.pending
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        ResultReleaseTask {
            drain: self.clone(),
            completed: false,
        }
    }
    fn failure(&self, error: CanonicalError) {
        let mut saved = self
            .error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if saved.is_none() {
            *saved = Some(error);
        }
    }
    pub(crate) async fn drain(&self) -> Result<(), CanonicalError> {
        if self.live.load(std::sync::atomic::Ordering::SeqCst) != 0 {
            return Err(invalid("isolated fixture still has live result buffers"));
        }
        tokio::time::timeout(crate::canonical::REQUEST_TIMEOUT, async {
            loop {
                let notified = self.changed.notified();
                if self.pending.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                    break;
                }
                notified.await;
            }
        })
        .await
        .map_err(|_| CanonicalError::Timeout)?;
        if let Some(error) = self
            .error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            return Err(error);
        }
        Ok(())
    }
}
#[cfg(any(feature = "test-support", feature = "canonical-tests"))]
struct ResultReleaseTask {
    drain: Arc<ResultReadDrain>,
    completed: bool,
}
#[cfg(any(feature = "test-support", feature = "canonical-tests"))]
impl ResultReleaseTask {
    fn complete(mut self, result: Result<(), CanonicalError>) {
        if let Err(error) = result {
            self.drain.failure(error);
        }
        self.completed = true;
    }
}
#[cfg(any(feature = "test-support", feature = "canonical-tests"))]
impl Drop for ResultReleaseTask {
    fn drop(&mut self) {
        if !self.completed {
            self.drain.failure(invalid(
                "result protection release task ended before completion",
            ));
        }
        self.drain
            .pending
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        self.drain.changed.notify_waiters();
    }
}

/// Lifetime ownership shared by a selection and its returned block buffers.
/// Server leases prevent stale selections from resurrecting reclaimed sources.
#[derive(Debug)]
struct ReadOwner {
    store: CanonicalStore,
    selection: ProtectedSelection,
    executor: Option<tokio::runtime::Handle>,
}
impl Drop for ReadOwner {
    fn drop(&mut self) {
        #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
        let release = self.store.result_read_drain.release();
        if let Some(runtime) = self.executor.as_ref() {
            let store = self.store.clone();
            let selection = self.selection.clone();
            runtime.spawn(async move {
                let result = store.release(&selection).await;
                #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
                release.complete(result);
                #[cfg(not(any(feature = "test-support", feature = "canonical-tests")))]
                let _ = result;
            });
        }
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
        let micros =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if micros <= 0 {
            return Err(invalid("positive read protection lifetime required"));
        }
        let selection = &self.owner.selection;
        protected_query("canonical_results::renew", ||Ok(self.owner.store.db.query(format!("{PROTECTED_BEGIN}\nUPDATE type::record('canonical_protections',$protection) SET expires_at=time::micros()+$lifetime;\nUPSERT type::record('canonical_guards','retention:'+$problem) SET key='retention:'+$problem,generation=(generation ?? 0dec)+1dec;\nCOMMIT;"))
            .bind(("problem",selection.revision().problem.clone())).bind(("revision",selection.revision().key.clone()))
            .bind(("sequence",canonical_codec::encode_uint(selection.revision().sequence)?))
            .bind(("protection",selection.key().to_owned())).bind(("lifetime",micros)))).await?;
        Ok(())
    }
    fn set(&self, key: &str) -> Result<&ResultSetDescriptor, CanonicalError> {
        self.sets
            .iter()
            .find(|s| s.key == key)
            .ok_or_else(|| invalid("result set outside admitted descriptor"))
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
    /// Derived finite/missing scalar observations; verified against IPC on decode.
    pub cells: Vec<ResultCell>,
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

/// Validate a declared scalar cell through A2's codec. The row is the sole
/// declaration; tag checking distinguishes missing scientific values from raw
/// diagnostic bits and absent database values.
pub fn validate_result_cell(cell: &ResultCell) -> Result<(), CanonicalError> {
    if cell.interpretation != wire::INTERPRETATION {
        return Err(invalid("result cell interpretation"));
    }
    for value in [
        &cell.key,
        &cell.result_set,
        &cell.batch,
        &cell.output,
        &cell.partition,
        &cell.coordinate,
    ] {
        identity(value)?;
    }
    match cell.cell_kind.as_str() {
        "missing" if cell.bits.is_none() && cell.projection.is_none() => Ok(()),
        "finite" => {
            let bits = cell
                .bits
                .as_ref()
                .ok_or_else(|| invalid("finite cell lacks exact bits"))?;
            let projection = cell
                .projection
                .ok_or_else(|| invalid("finite cell lacks projection"))?;
            let mut object = Object::new();
            object.insert("bits", canonical_codec::encode_bytes(bits.clone())?);
            object.insert(
                "projection",
                Value::Number(surrealdb::types::Number::Float(projection)),
            );
            canonical_codec::decode_finite(Value::Object(object))?;
            Ok(())
        }
        "diagnostic" if cell.projection.is_none() => {
            let bits = cell
                .bits
                .as_ref()
                .ok_or_else(|| invalid("diagnostic cell lacks exact bits"))?;
            canonical_codec::decode_diagnostic_bits(canonical_codec::encode_bytes(bits.clone())?)?;
            Ok(())
        }
        _ => Err(invalid("unknown or inconsistent scientific cell kind")),
    }
}

/// Check metadata against its exact batch, not a broad result-set prefix.
pub fn validate_result_block(
    metadata: &BlockMetadata,
    batch: &ResultBatch,
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
        || metadata.interpretation != wire::INTERPRETATION
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
    crate::canonical_execution::validate_result_batch(batch)?;
    let digest = crate::canonical_execution::result_payload_digest(batch.payload.as_slice());
    if metadata.payload_digest != digest {
        return Err(invalid("result block payload digest"));
    }
    Ok(())
}

impl CanonicalStore {
    /// Select a bounded output index page natively, before fetching any original
    /// scientific payload. Numeric predicates use only finite projections.
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
        let table = if dense {
            "canonical_result_block_outputs"
        } else {
            "canonical_result_cells"
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
            "row>=$start AND row<$end AND ($minimum=NONE OR projection.projection>=$minimum) AND ($maximum=NONE OR projection.projection<=$maximum) AND ($missing=NONE OR ($missing AND cell_kind='missing') OR (!$missing AND cell_kind='finite'))"
        };
        let order = if dense { "start" } else { "row" };
        let query = format!(
            "{PROTECTED_BEGIN}\nLET $indexes=SELECT * FROM {table} WHERE result_set=$result_set AND output=$output AND partition=$partition AND {cursor}{bounds}{residual_cursor} ORDER BY {order} LIMIT {PAGE};\nLET $blocks=SELECT * FROM canonical_result_blocks WHERE result_set=$result_set AND key IN $indexes.batch ORDER BY ordinal LIMIT 65;\nRETURN {{indexes:$indexes,blocks:$blocks}};\nCOMMIT;"
        );
        let finite = |value: Option<f64>| {
            value
                .map(|v| Value::Number(surrealdb::types::Number::Float(v)))
                .unwrap_or(Value::None)
        };
        let mut response = protected_query("canonical_results::result_output_page", || {
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
        })
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
            cells: Vec::new(),
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
                if item.interpretation != wire::INTERPRETATION {
                    return Err(invalid("output index interpretation"));
                }
                page.outputs.push(item);
                result
            } else {
                let item = wire::decode_canonical_result_cells(index)?;
                validate_result_cell(&item)?;
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
                page.cells.push(item);
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

    /// Resolve only an admitted terminal attempt, then pin its immutable revision
    /// before reading the exact manifest. Historical terminal attempts remain
    /// selectable even after the semantic run starts a new attempt.
    pub async fn read_results(
        &self,
        run_key: &str,
        attempt_key: &str,
        lifetime: Duration,
    ) -> Result<ResultRead, CanonicalError> {
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
        #[cfg(any(feature = "test-support", feature = "canonical-tests"))]
        self.result_read_drain.retain();
        let owner = Arc::new(ReadOwner {
            store: self.clone(),
            selection: protection,
            executor: tokio::runtime::Handle::try_current().ok(),
        });
        let key = attempt
            .closed_manifest
            .as_deref()
            .ok_or_else(|| invalid("terminal attempt lacks closed manifest"))?;
        let mut response=protected_query("canonical_results::read_results", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nfn::pse_execution_v1::available($run);\nLET $attempt_record=SELECT * FROM ONLY type::record('canonical_attempts',$attempt);
LET $manifest_record=SELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);
IF $attempt_record=NONE OR $attempt_record.run!=$run OR !$attempt_record.terminal OR !$attempt_record.closed OR $attempt_record.ingestion_open OR $attempt_record.closed_manifest!=$manifest OR $manifest_record=NONE OR $manifest_record.attempt!=$attempt OR $manifest_record.generation!=$attempt_record.generation {{ THROW 'terminal result selection unavailable'; }};
UPSERT type::record('canonical_result_protections',$protection) SET key=$protection,run=$run,attempt=$attempt,manifest=$manifest;
UPSERT type::record('canonical_guards','retention:'+$problem) SET key='retention:'+$problem,generation=(generation ?? 0dec)+1dec;
SELECT * FROM ONLY type::record('canonical_result_manifests',$manifest);\nCOMMIT;"))
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
        let mut response=protected_query("canonical_results::result_block_page", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM canonical_result_blocks WHERE result_set=$result_set AND output=$output AND partition=$partition AND {cursor}ordinal<$count AND end>$start AND start<$end ORDER BY ordinal LIMIT {PAGE};\nCOMMIT;"))
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
                    || metadata.interpretation != wire::INTERPRETATION
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
        if !Arc::ptr_eq(&self.db, &read.owner.store.db) {
            return Err(invalid("result read belongs to another store"));
        }
        let set = read.set(result_set)?;
        if ordinal >= set.batch_count {
            return Err(invalid("payload outside admitted descriptor"));
        }
        let key = result_batch_key(&read.attempt.key, &set.key, ordinal);
        let mut response=protected_query("canonical_results::result_payload", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nSELECT * FROM ONLY type::record('canonical_result_batches',$batch);\nCOMMIT;"))
            .bind(("problem",read.run.problem.clone())).bind(("revision",read.run.revision.clone()))
            .bind(("sequence",canonical_codec::encode_uint(read.owner.selection.revision().sequence)?))
            .bind(("protection",read.owner.selection.key().to_owned())).bind(("batch",key.clone())))).await?;
        let index = response.num_statements().saturating_sub(2);
        let batch = response
            .take::<Option<Object>>(index)?
            .map(wire::decode_canonical_result_batches)
            .transpose()?
            .ok_or_else(|| invalid("admitted batch unavailable"))?;
        if batch.key != key
            || batch.attempt != read.attempt.key
            || batch.result_set != set.key
            || batch.ordinal != ordinal
        {
            return Err(invalid("payload admitted coordinate mismatch"));
        }
        crate::canonical_execution::validate_result_batch(&batch)?;
        Ok(ResultPayload {
            batch,
            owner: read.owner.clone(),
        })
    }

    /// Fetch one named admitted blob. Successful checked RPC completion precedes
    /// exposing any bytes; no provisional gRPC prefix is treated as a result.
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
        let mut response=protected_query("canonical_results::result_block", ||Ok(self.db.query(format!("{PROTECTED_BEGIN}\nRETURN {{metadata:(SELECT * FROM ONLY type::record('canonical_result_blocks',$key)),batch:(SELECT * FROM ONLY type::record('canonical_result_batches',$batch))}};\nCOMMIT;"))
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
        validate_result_block(&metadata, &batch)?;
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
    fn cell(kind: &str, bits: Option<u64>, projection: Option<f64>) -> ResultCell {
        ResultCell {
            key: "cell".into(),
            result_set: "set".into(),
            batch: "batch".into(),
            output: "x".into(),
            partition: "0".into(),
            row: 0,
            coordinate: "0".into(),
            cell_kind: kind.into(),
            bits: bits.map(|b| b.to_be_bytes().to_vec().into()),
            projection,
            interpretation: wire::INTERPRETATION.into(),
        }
    }
    #[test]
    fn scalar_domains_preserve_signed_zero_and_diagnostic_nan_without_fabricated_missing() {
        assert!(
            validate_result_cell(&cell("finite", Some((-0.0f64).to_bits()), Some(0.0))).is_ok()
        );
        assert!(
            validate_result_cell(&cell(
                "finite",
                Some(f64::INFINITY.to_bits()),
                Some(f64::INFINITY)
            ))
            .is_err()
        );
        assert!(
            validate_result_cell(&cell("diagnostic", Some(0x7ff8_0000_0000_0042), None)).is_ok()
        );
        assert!(validate_result_cell(&cell("diagnostic", Some(0), Some(0.0))).is_err());
        assert!(validate_result_cell(&cell("missing", None, None)).is_ok());
        assert!(validate_result_cell(&cell("missing", Some(0), None)).is_err());
        assert!(validate_result_cell(&cell("finite", None, Some(0.0))).is_err());
        assert!(validate_result_cell(&cell("null", None, None)).is_err());
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
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let revision = store.edit("problem", None, "source-1", &[]).await.unwrap();
        (store, options.database, revision)
    }

    enum ReleaseFault {
        Cancelled,
        Panicked,
        Failed,
    }

    async fn removal_reports_release_fault(fault: ReleaseFault) {
        let store = crate::testing::canonical_fixture_store().unwrap();
        // Exercise the same completion guard transferred by ReadOwner::drop to
        // its detached release RPC, while retaining the fixture in this task.
        store.result_read_drain.retain();
        let release = store.result_read_drain.release();
        let removal = store.remove_isolated_fixture();
        tokio::pin!(removal);
        // Poll teardown while the release is pending, so the control also covers
        // the notification that resumes its waiter after the task is destroyed.
        tokio::select! {
            biased;
            result = &mut removal => panic!("pending release did not block teardown: {result:?}"),
            () = tokio::task::yield_now() => {}
        }
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
                tokio::spawn(async move {
                    release.complete(Err(invalid("injected result protection release failure")));
                })
                .await
                .unwrap();
                "injected result protection release failure"
            }
        };
        assert_eq!(
            store
                .result_read_drain
                .pending
                .load(std::sync::atomic::Ordering::SeqCst),
            0,
            "the failed release must drain instead of waiting for the RPC timeout"
        );
        let error = removal.await.unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
        // Refusal precedes database removal. Once its recorded cause is observed,
        // explicit teardown can be retried and disarms automatic fixture cleanup.
        store.open().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn canonical_results_fixture_removal_reports_cancelled_release() {
        removal_reports_release_fault(ReleaseFault::Cancelled).await;
    }

    #[tokio::test]
    async fn canonical_results_fixture_removal_reports_panicking_release() {
        removal_reports_release_fault(ReleaseFault::Panicked).await;
    }

    #[tokio::test]
    async fn canonical_results_fixture_removal_reports_failed_release() {
        removal_reports_release_fault(ReleaseFault::Failed).await;
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
        let set = result_set_key(first.attempt(), "scalars");
        let key = result_batch_key(first.attempt(), &set, 0);
        let mut cells = vec![ResultCell {
            key: "negative-zero".into(),
            result_set: set.clone(),
            batch: key.clone(),
            output: "x".into(),
            partition: "0".into(),
            row: 0,
            coordinate: "0".into(),
            cell_kind: "finite".into(),
            bits: Some((-0.0f64).to_bits().to_be_bytes().to_vec().into()),
            projection: Some(0.0),
            interpretation: wire::INTERPRETATION.into(),
        }];
        cells.extend((1..130u64).map(|row| ResultCell {
            key: format!("scalar-{row:03}"),
            result_set: set.clone(),
            batch: key.clone(),
            output: "x".into(),
            partition: "0".into(),
            row,
            coordinate: row.to_string(),
            cell_kind: "finite".into(),
            bits: Some((row as f64).to_bits().to_be_bytes().to_vec().into()),
            projection: Some(row as f64),
            interpretation: wire::INTERPRETATION.into(),
        }));
        let payload = [1, 2, 3];
        let block = BlockMetadata {
            key: key.clone(),
            batch: key,
            result_set: set.clone(),
            output: "scalars".into(),
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
        store
            .append_result_block_cells(
                &first,
                "cell-write",
                "scalars",
                0,
                &payload,
                130,
                &block,
                &cells,
            )
            .await
            .unwrap();
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
        unsafe { store.seal_attempt(&manifest, "seal-1", TerminalClass::Partial, &[1]) }
            .await
            .unwrap();
        let read = store
            .read_results("run", first.attempt(), Duration::from_secs(30))
            .await
            .unwrap();
        let values = store
            .result_output_page(&read, &set, "x", "0", 0, 1, None, false, None, None, None)
            .await
            .unwrap();
        assert_eq!(
            values.cells[0].bits.as_ref().unwrap().as_slice(),
            (-0.0f64).to_bits().to_be_bytes()
        );
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
                    &read, &set, "x", "0", 80, 90, after, false, None, None, None,
                )
                .await
                .unwrap();
            assert_eq!(
                page.cells.iter().map(|cell| cell.row).collect::<Vec<_>>(),
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
                    &read, &set, "x", "0", 0, 130, after, false, None, None, None,
                )
                .await
                .unwrap();
            assert_eq!(
                page.cells.iter().map(|cell| cell.row).collect::<Vec<_>>(),
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
        unsafe { store.seal_attempt(&manifest, "seal-2", TerminalClass::Failed, &[2]) }
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
                    "x",
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
                .cells
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
                    &payload,
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
        unsafe { store.seal_attempt(&manifest, "seal", TerminalClass::Partial, &[1]) }
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
