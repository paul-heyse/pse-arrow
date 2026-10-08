// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact canonical result selections streamed one independently decoded blob at
//! a time. Row coordinates select recorded coverage; missing ranges stay missing.

use super::{Runtime, WorkflowError, contract, result_blocks};
use datafusion::arrow::{datatypes::SchemaRef, record_batch::RecordBatch};
use pse_columnar::{
    CancellationToken, MemoryConsumer, MemoryReservation, owned_buffer::AllocationScope,
};
use pse_model::generated::runtime::canonical_result_blocks::Row as BlockMetadata;
use pse_operations::canonical_results::ResultRead;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
    time::Duration,
};

const READ_LIFETIME: Duration = Duration::from_secs(60);
// One checked RPC is limited to 4 MiB, plus its native-value/typed-row copies,
// the verifier's 32 MiB apparent metadata bound, decoder scratch and decoded
// block. No trajectory is
// materialized here; returned arrays keep their independent actual reservation.
const READ_BYTES: usize = 48 * 1024 * 1024;

#[derive(Debug)]
struct OutputFilter {
    output: String,
    partition: String,
    dense: bool,
    minimum: Option<f64>,
    maximum: Option<f64>,
    missing: Option<bool>,
}
#[derive(Debug)]
enum OutputRows {
    Scalar(Vec<pse_model::generated::runtime::canonical_result_cells::Row>),
    Dense(pse_model::generated::runtime::canonical_result_block_outputs::Row),
}

/// A protected terminal attempt, exact relation and requested row coordinates.
#[derive(Debug)]
pub struct CanonicalResultReader {
    runtime: Runtime,
    read: ResultRead,
    relation: pse_ids::SemanticId,
    set: String,
    schema: SchemaRef,
    start: u64,
    end: u64,
    after: Option<u64>,
    last_end: Option<u64>,
    pending: VecDeque<BlockMetadata>,
    metadata_owner: Option<MemoryReservation>,
    exhausted: bool,
    consumed: bool,
    cancel: CancellationToken,
    output: Option<OutputFilter>,
    indexes: BTreeMap<String, OutputRows>,
    last_receipt: Option<(String, u64)>,
}
impl CanonicalResultReader {
    /// Exact registry schema of the selected scientific relation.
    pub fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    /// Actual recorded terminal attempt, including failure/partiality class.
    pub fn selection(&self) -> &ResultRead {
        &self.read
    }
    /// Cancellation shared with the Arrow boundary and its consumer.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }
    /// Stop unread database work; returned arrays own their decoded buffers.
    pub fn close(&mut self) {
        self.cancel.cancel();
        self.pending.clear();
        self.indexes.clear();
        self.metadata_owner = None;
        self.exhausted = true;
    }

    /// Publish a local Arrow IPC export only after every selected database read
    /// and final IPC write succeeds. Interrupted staging remains `.incomplete`;
    /// creating the final hard link refuses an existing destination atomically.
    pub async fn export_ipc(&mut self, destination: &std::path::Path) -> Result<(), WorkflowError> {
        if self.consumed {
            return Err(contract("export requires an unread complete selection"));
        }
        let mut staging = destination.as_os_str().to_os_string();
        staging.push(".incomplete");
        let staging = std::path::PathBuf::from(staging);
        let io = |source| pse_engine::EngineError::Infrastructure {
            op: "publish canonical result IPC export".into(),
            source: Box::new(source),
        };
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(io)?;
        let mut metadata = self.schema.metadata().clone();
        metadata.insert(
            "pse.canonical.problem".into(),
            self.read.run().problem.clone(),
        );
        metadata.insert(
            "pse.canonical.revision".into(),
            self.read.run().revision.clone(),
        );
        metadata.insert("pse.canonical.run".into(), self.read.run().key.clone());
        metadata.insert(
            "pse.canonical.attempt".into(),
            self.read.attempt().key.clone(),
        );
        metadata.insert(
            "pse.canonical.terminal-class".into(),
            self.read
                .attempt()
                .outcome
                .clone()
                .ok_or_else(|| contract("terminal result class absent"))?,
        );
        metadata.insert(
            "pse.canonical.manifest".into(),
            self.read.manifest().digest.clone(),
        );
        metadata.insert(
            "pse.canonical.interpretation".into(),
            self.read.run().interpretation.clone(),
        );
        metadata.insert("pse.canonical.row-start".into(), self.start.to_string());
        metadata.insert("pse.canonical.row-end".into(), self.end.to_string());
        if let Some(filter) = &self.output {
            metadata.insert("pse.canonical.output".into(), filter.output.clone());
            metadata.insert("pse.canonical.partition".into(), filter.partition.clone());
            if let Some(value) = filter.minimum {
                metadata.insert(
                    "pse.canonical.minimum-bits".into(),
                    value.to_bits().to_string(),
                );
            }
            if let Some(value) = filter.maximum {
                metadata.insert(
                    "pse.canonical.maximum-bits".into(),
                    value.to_bits().to_string(),
                );
            }
            if let Some(value) = filter.missing {
                metadata.insert("pse.canonical.missing".into(), value.to_string());
            }
        }
        let schema = Arc::new(self.schema.as_ref().clone().with_metadata(metadata));
        let mut writer =
            datafusion::arrow::ipc::writer::StreamWriter::try_new(file, schema.as_ref())
                .map_err(result_blocks::ResultBlockError::from)?;
        while let Some(batch) = self.next_batch().await? {
            let batch = batch
                .with_schema(schema.clone())
                .map_err(result_blocks::ResultBlockError::from)?;
            writer
                .write(&batch)
                .map_err(result_blocks::ResultBlockError::from)?;
        }
        self.cancel
            .checkpoint()
            .map_err(pse_engine::EngineError::from)?;
        writer
            .finish()
            .map_err(result_blocks::ResultBlockError::from)?;
        writer.get_ref().sync_all().map_err(io)?;
        drop(writer);
        std::fs::hard_link(&staging, destination).map_err(io)?;
        std::fs::remove_file(staging).map_err(io)?;
        Ok(())
    }

    /// Decode one admitted member only after successful checked RPC completion.
    /// A late failure is surfaced to the stream; it cannot become normal exhaustion.
    pub async fn next_batch(&mut self) -> Result<Option<RecordBatch>, WorkflowError> {
        self.next_relation_batch()
            .await?
            .map(|batch| {
                batch
                    .checked_export(&self.cancel)
                    .map_err(pse_engine::EngineError::from)
                    .map_err(Into::into)
            })
            .transpose()
    }
    pub(super) fn last_receipt(&self) -> Option<&(String, u64)> {
        self.last_receipt.as_ref()
    }
    /// Return one admitted registry relation batch for native columnar algorithms.
    /// The reader holds renewable protection for further database reads; copied
    /// decoded buffers retain their allocation owners independently of the reader.
    pub async fn next_relation_batch(
        &mut self,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        if let Err(error) = self.cancel.checkpoint() {
            self.close();
            return Err(pse_engine::EngineError::from(error).into());
        }
        self.consumed = true;
        let outcome = self.next_checked().await;
        if outcome.is_err() {
            self.close();
        }
        outcome
    }
    async fn next_checked(
        &mut self,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        loop {
            if self.pending.is_empty() {
                self.metadata_owner = None;
                if self.exhausted {
                    return Ok(None);
                }
                self.read.renew(READ_LIFETIME).await?;
                let owner = MemoryConsumer::new("canonical:result-metadata")
                    .register(&self.runtime.shared.pool());
                owner
                    .try_grow(READ_BYTES)
                    .map_err(pse_engine::EngineError::from)?;
                let page = if let Some(filter) = &self.output {
                    let page = self
                        .runtime
                        .canonical_store()
                        .result_output_page(
                            &self.read,
                            &self.set,
                            &filter.output,
                            &filter.partition,
                            self.start,
                            self.end,
                            self.after,
                            filter.dense,
                            filter.minimum,
                            filter.maximum,
                            filter.missing,
                        )
                        .await?;
                    self.exhausted = page.cells.len() + page.outputs.len() < 64;
                    self.after = page
                        .cells
                        .last()
                        .map(|cell| cell.row)
                        .or_else(|| page.outputs.last().map(|output| output.start))
                        .or(self.after);
                    for cell in page.cells {
                        match self
                            .indexes
                            .entry(cell.batch.clone())
                            .or_insert_with(|| OutputRows::Scalar(Vec::new()))
                        {
                            OutputRows::Scalar(rows) => rows.push(cell),
                            OutputRows::Dense(_) => {
                                return Err(contract("mixed scientific output index kinds"));
                            }
                        }
                    }
                    for output in page.outputs {
                        if self
                            .indexes
                            .insert(output.batch.clone(), OutputRows::Dense(output))
                            .is_some()
                        {
                            return Err(contract("duplicate dense output index"));
                        }
                    }
                    page.blocks
                } else {
                    let page = self
                        .runtime
                        .canonical_store()
                        .result_block_page(
                            &self.read,
                            &self.set,
                            &self.relation.to_string(),
                            "0",
                            self.start,
                            self.end,
                            self.after,
                        )
                        .await?;
                    self.exhausted = page.len() < 64;
                    self.after = page.last().map(|m| m.ordinal).or(self.after);
                    page
                };
                self.pending = page.into();
                self.metadata_owner = Some(owner);
                if self.pending.is_empty() {
                    continue;
                }
            }
            let metadata = self
                .pending
                .pop_front()
                .ok_or_else(|| contract("result metadata page empty"))?;
            self.cancel
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            let owner =
                MemoryConsumer::new("canonical:result-block").register(&self.runtime.shared.pool());
            owner
                .try_grow(READ_BYTES)
                .map_err(pse_engine::EngineError::from)?;
            let block = self
                .runtime
                .canonical_store()
                .result_block(&self.read, metadata)
                .await?;
            if block.metadata.columns != self.schema.fields().len() as u64
                || self.output.is_none()
                    && self.last_end.is_some_and(|end| block.metadata.start < end)
            {
                return Err(contract(
                    "recorded result block shape/order/coverage overlaps",
                ));
            }
            self.last_end = Some(block.metadata.end);
            let rows = usize::try_from(block.metadata.rows)
                .map_err(|_| contract("result row extent exceeds platform"))?;
            let batch = result_blocks::decode_result_block(
                block.batch.payload.as_slice(),
                self.schema.clone(),
                rows,
            )?;
            let lower = self.start.max(block.metadata.start);
            let upper = self.end.min(block.metadata.end);
            if lower >= upper {
                continue;
            }
            let offset = usize::try_from(lower - block.metadata.start)
                .map_err(|_| contract("result coordinate exceeds platform"))?;
            let length = usize::try_from(upper - lower)
                .map_err(|_| contract("result coordinate exceeds platform"))?;
            let batch = if self.output.is_some() {
                batch
            } else {
                batch.slice(offset, length)
            };
            let scope = AllocationScope::default();
            let owned = scope
                .attach_reserved(batch, owner)
                .map_err(pse_engine::EngineError::from)?;
            let spec = self
                .runtime
                .registry
                .relation_by_id(self.relation)
                .ok_or_else(|| contract("result relation unavailable"))?;
            let validation = self.runtime.validation_context()?;
            let checked = pse_relations::columnar::FieldCheckedBatch::admit_owned(
                &self.runtime.registry,
                spec,
                owned,
                &validation,
                &self.cancel,
            )
            .map_err(pse_engine::EngineError::from)?;
            let checked = if let Some(filter) = &self.output {
                let indexes = self
                    .indexes
                    .remove(&block.batch.key)
                    .ok_or_else(|| contract("selected output block lacks index"))?;
                let indices = match indexes {
                    OutputRows::Scalar(cells) => {
                        let expected = super::result_projection::scalar_cells_at(
                            &checked,
                            &self.set,
                            &block.batch.key,
                            0,
                            checked.batch().num_rows(),
                            block.metadata.start,
                        )?;
                        for cell in &cells {
                            if !expected.iter().any(|value| value == cell) {
                                return Err(contract(
                                    "native scalar index differs from original scientific row",
                                ));
                            }
                        }
                        cells
                            .into_iter()
                            .map(|cell| {
                                u32::try_from(cell.row - block.metadata.start)
                                    .map_err(|_| contract("selected row exceeds Arrow take extent"))
                            })
                            .collect::<Result<Vec<_>, _>>()?
                    }
                    OutputRows::Dense(output) => {
                        let view=pse_relations::generated::runtime::simulation_samples::View::from_checked(&checked).map_err(super::relation)?;
                        let mut indices = Vec::new();
                        let mut min = f64::INFINITY;
                        let mut max = f64::NEG_INFINITY;
                        for row in 0..view.len() {
                            let value = view.row(row).map_err(super::relation)?;
                            if format!("{}:{}:value", self.relation, value.symbol_id)
                                != filter.output
                                || output.start != block.metadata.start
                                || output.end != block.metadata.end
                            {
                                return Err(contract(
                                    "dense output group differs from original scientific rows",
                                ));
                            }
                            min = min.min(value.time);
                            max = max.max(value.time);
                            let coordinate = block.metadata.start + row as u64;
                            if coordinate >= self.start
                                && coordinate < self.end
                                && filter.minimum.is_none_or(|lower| value.time >= lower)
                                && filter.maximum.is_none_or(|upper| value.time <= upper)
                            {
                                indices.push(u32::try_from(row).map_err(|_| {
                                    contract("selected trajectory row exceeds Arrow take extent")
                                })?);
                            }
                        }
                        if output.coordinate_min != Some(min) || output.coordinate_max != Some(max)
                        {
                            return Err(contract(
                                "dense coordinate projection differs from exact recorded time",
                            ));
                        }
                        indices
                    }
                };
                if indices.is_empty() {
                    continue;
                }
                checked
                    .take_reserved(
                        &datafusion::arrow::array::UInt32Array::from(indices),
                        &self.runtime.shared.pool(),
                        &self.cancel,
                    )
                    .map_err(super::relation)?
            } else {
                checked
            };
            self.last_receipt = Some((block.batch.key.clone(), lower));
            return Ok(Some(checked));
        }
    }
}
impl Runtime {
    /// Read the exact canonical run header as its registry-declared Arrow row.
    pub async fn run_record(
        &self,
        run: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::canonical_runs as rows;
        let extent = MemoryConsumer::new("canonical:result-header").register(&self.shared.pool());
        extent
            .try_grow(READ_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let row = self
            .canonical_store()
            .canonical_run(run)
            .await?
            .ok_or_else(|| contract("canonical run absent"))?;
        let validation = self.validation_context()?;
        let mut builder = rows::Builder::with_registry(&self.registry, 1, &validation)
            .map_err(super::relation)?;
        builder.push(row).map_err(super::relation)?;
        builder
            .finish()
            .map_err(super::relation)?
            .retained(&self.shared.pool(), &CancellationToken::new())
            .map_err(super::relation)
    }
    /// Read the actual attempt's recorded lifecycle and completion without substitution.
    pub async fn attempt_record(
        &self,
        attempt: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::canonical_attempts as rows;
        let extent = MemoryConsumer::new("canonical:result-header").register(&self.shared.pool());
        extent
            .try_grow(READ_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let row = self
            .canonical_store()
            .canonical_attempt(attempt)
            .await?
            .ok_or_else(|| contract("canonical attempt absent"))?;
        let validation = self.validation_context()?;
        let mut builder = rows::Builder::with_registry(&self.registry, 1, &validation)
            .map_err(super::relation)?;
        builder.push(row).map_err(super::relation)?;
        builder
            .finish()
            .map_err(super::relation)?
            .retained(&self.shared.pool(), &CancellationToken::new())
            .map_err(super::relation)
    }
    /// Read one admitted terminal attempt's exact closed manifest as a typed row.
    pub async fn result_manifest(
        &self,
        attempt: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::canonical_result_manifests as rows;
        let extent = MemoryConsumer::new("canonical:result-header").register(&self.shared.pool());
        extent
            .try_grow(READ_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let store = self.canonical_store();
        let selected = store
            .canonical_attempt(attempt)
            .await?
            .ok_or_else(|| contract("canonical attempt absent"))?;
        let read = store
            .read_results(&selected.run, attempt, READ_LIFETIME)
            .await?;
        let validation = self.validation_context()?;
        let mut builder = rows::Builder::with_registry(&self.registry, 1, &validation)
            .map_err(super::relation)?;
        builder
            .push(read.manifest().clone())
            .map_err(super::relation)?;
        builder
            .finish()
            .map_err(super::relation)?
            .retained(&self.shared.pool(), &CancellationToken::new())
            .map_err(super::relation)
    }
    /// Select a problem's newest admitted terminal run under explicit outcome classes.
    pub async fn latest_results(
        &self,
        problem: &str,
        classes: &[pse_operations::canonical_execution::TerminalClass],
        relation: &str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
    ) -> Result<CanonicalResultReader, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let read = self
            .canonical_store()
            .latest_results(problem, classes, READ_LIFETIME)
            .await?;
        self.result_selection(read, relation, start, end, cancel)
    }
    /// Reopen one exact terminal run/attempt and registry relation after restart.
    /// `start..end` are half-open recorded row coordinates; partial/failure
    /// outcomes keep their own observations and are never substituted by another run.
    pub async fn results(
        &self,
        run: &str,
        attempt: &str,
        relation: &str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
    ) -> Result<CanonicalResultReader, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        if start > end {
            return Err(contract("result row range is reversed"));
        }
        let read = self
            .canonical_store()
            .read_results(run, attempt, READ_LIFETIME)
            .await?;
        self.result_selection(read, relation, start, end, cancel)
    }
    fn result_selection(
        &self,
        read: ResultRead,
        relation: &str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
    ) -> Result<CanonicalResultReader, WorkflowError> {
        if start > end {
            return Err(contract("result row range is reversed"));
        }
        let spec = self
            .registry
            .relation(relation)
            .ok_or_else(|| contract("unknown result relation"))?;
        let name = spec.id.to_string();
        let schema = pse_schema::arrow::relation_schema_ref(&self.registry, spec)
            .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
        let set = read
            .sets()
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| contract("relation is not part of selected attempt"))?
            .key
            .clone();
        Ok(CanonicalResultReader {
            runtime: self.clone(),
            read,
            relation: spec.id,
            set,
            schema,
            start,
            end,
            after: None,
            last_end: None,
            pending: VecDeque::new(),
            metadata_owner: None,
            exhausted: start == end,
            consumed: false,
            cancel,
            output: None,
            indexes: BTreeMap::new(),
            last_receipt: None,
        })
    }
    /// Select one explicitly declared scientific output. Scalar numeric bounds
    /// filter finite values; dense bounds select time. Original rows retain their
    /// units, basis and quality fields, and every index is checked against IPC.
    #[allow(
        clippy::too_many_arguments,
        reason = "one output-read operation exposes exact attempt, scientific output, stored row range and finite/null predicate coordinates"
    )]
    pub async fn output_results(
        &self,
        run: &str,
        attempt: &str,
        relation: &str,
        owner: pse_ids::SemanticId,
        field: &str,
        partition: &str,
        start: u64,
        end: u64,
        minimum: Option<f64>,
        maximum: Option<f64>,
        missing: Option<bool>,
        cancel: CancellationToken,
    ) -> Result<CanonicalResultReader, WorkflowError> {
        if minimum.is_some_and(|value| !value.is_finite())
            || maximum.is_some_and(|value| !value.is_finite())
            || minimum.zip(maximum).is_some_and(|(min, max)| min > max)
            || missing == Some(true) && (minimum.is_some() || maximum.is_some())
        {
            return Err(contract("invalid finite output predicate"));
        }
        let mut reader = self
            .results(run, attempt, relation, start, end, cancel)
            .await?;
        super::result_projection::check_output_field(reader.relation, field)?;
        let dense =
            reader.relation == pse_relations::generated::runtime::simulation_samples::RELATION_ID;
        if dense && missing.is_some() {
            return Err(contract(
                "finite dense outputs have no missing-value predicate",
            ));
        }
        reader.output = Some(OutputFilter {
            output: format!("{}:{owner}:{field}", reader.relation),
            partition: partition.into(),
            dense,
            minimum,
            maximum,
            missing,
        });
        Ok(reader)
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_connected_results_server_unit {
    #![allow(
        unsafe_code,
        reason = "isolated generated finite-field fixtures explicitly assert a partial terminal class"
    )]
    use super::*;
    use datafusion::arrow::array::{Array, Float64Array};
    use pse_operations::canonical_execution::{
        RunRequest, TerminalClass, result_batch_key, result_payload_digest, result_set_key,
    };
    use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
    #[tokio::test]
    async fn canonical_decoded_arrow_survives_reader_drop_and_result_reclamation() {
        let runtime = super::super::tests::runtime();
        let store = runtime.canonical_store().clone();
        let pool = runtime.shared.pool();
        let source = store
            .edit("decoded-owner", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&RunRequest {
                key: "decoded-owner".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("decoded-owner", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let run_id = pse_operations::mint_id();
        let expected = [Some(-0.0), None, Some(273.15)]
            .into_iter()
            .enumerate()
            .map(|(step, real)| solve_metrics::Row {
                run_id,
                step: step as i64,
                namespace: "original".into(),
                name: "temperature".into(),
                kind: if real.is_some() {
                    NativeMetricKind::Real
                } else {
                    NativeMetricKind::Unavailable
                },
                real,
                integer: None,
                boolean: None,
                text: None,
                unavailable: if real.is_none() {
                    Some(pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite)
                } else {
                    None
                },
            })
            .collect::<Vec<_>>();
        let validation = runtime.validation_context().unwrap();
        let mut builder =
            solve_metrics::Builder::with_registry(&runtime.registry, expected.len(), &validation)
                .unwrap();
        for row in &expected {
            builder.push(row.clone()).unwrap();
        }
        let original = builder.finish().unwrap();
        super::super::result_projection::store_result_table(
            &store,
            &fence,
            solve_metrics::RELATION_ID,
            &original,
            &pool,
        )
        .await
        .unwrap();
        drop(original);
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: independently authored observations assert partial availability only.
        unsafe { store.seal_attempt(&manifest, "seal", TerminalClass::Partial, &[1]) }
            .await
            .unwrap();
        let baseline = pool.reserved();
        let mut reader = runtime
            .results(
                "decoded-owner",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                3,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let batch = reader.next_batch().await.unwrap().unwrap();
        {
            let view = solve_metrics::View::try_from_batch_with_registry(
                &runtime.registry,
                &batch,
                &validation,
            )
            .unwrap();
            assert_eq!(
                (0..view.len())
                    .map(|index| view.row(index).unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
        let escaped = batch.column_by_name("real").unwrap().clone();
        assert!(store.forget_run_results("decoded-owner").await.is_err());
        drop(reader);
        drop(batch);
        assert!(pool.reserved() > baseline);
        // Fully decoded buffers require no storage access. The reader's final
        // protection release can settle while these accounted arrays remain live.
        let mut retired = false;
        for _ in 0..32 {
            if store.forget_run_results("decoded-owner").await.is_ok() {
                retired = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(retired);
        let mut reclaimed = 0;
        let mut complete = false;
        for _ in 0..8 {
            let page = store.reclaim_result_page("decoded-owner").await.unwrap();
            reclaimed += page.batches;
            if page.complete {
                complete = true;
                break;
            }
        }
        assert!(complete);
        assert_eq!(reclaimed, 1);
        assert!(
            runtime
                .results(
                    "decoded-owner",
                    fence.attempt(),
                    "runtime.solve_metrics",
                    0,
                    3,
                    CancellationToken::new()
                )
                .await
                .is_err()
        );
        store.remove_isolated_fixture().await.unwrap();
        drop(store);
        drop(runtime);
        let values = escaped.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert_eq!(values.value(2), 273.15);
        assert!(pool.reserved() > baseline);
        drop(escaped);
        assert_eq!(pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn canonical_output_indexes_read_original_rows_and_skip_unrelated_blocks() {
        use pse_relations::generated::runtime::{fit_parameters, simulation_samples};
        let runtime = super::super::tests::runtime();
        let store = runtime.canonical_store();
        let source = store.edit("projection", None, "source", &[]).await.unwrap();
        store
            .begin_run(&RunRequest {
                key: "projection".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("projection", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let validation = runtime.validation_context().unwrap();
        let first: pse_ids::SemanticId = pse_operations::mint_id();
        let second: pse_ids::SemanticId = pse_operations::mint_id();
        let absent: pse_ids::SemanticId = pse_operations::mint_id();
        let unit: pse_ids::SemanticId = pse_operations::mint_id();
        let run_id = pse_operations::mint_id();
        let mut parameters =
            fit_parameters::Builder::with_registry(&runtime.registry, 800, &validation).unwrap();
        for index in 0..800 {
            parameters
                .push(fit_parameters::Row {
                    run_id,
                    parameter_id: if index == 0 {
                        first
                    } else if index == 41 {
                        second
                    } else if index == 42 {
                        absent
                    } else {
                        pse_operations::mint_id()
                    },
                    fixed: false,
                    value: if index == 42 {
                        None
                    } else if index == 41 {
                        Some(-0.0)
                    } else {
                        Some(index as f64)
                    },
                    unit_id: unit,
                    scale: 1.0,
                    at_bound: None,
                })
                .unwrap();
        }
        let parameters = parameters.finish().unwrap();
        super::super::result_projection::store_result_table(
            store,
            &fence,
            fit_parameters::RELATION_ID,
            &parameters,
            &runtime.shared.pool(),
        )
        .await
        .unwrap();
        let mut samples =
            simulation_samples::Builder::with_registry(&runtime.registry, 200, &validation)
                .unwrap();
        for sample in (0..100).rev() {
            for symbol in [first, second] {
                samples
                    .push(simulation_samples::Row {
                        run_id,
                        sample,
                        time: sample as f64,
                        symbol_id: symbol,
                        quantity_id: unit,
                        unit_id: unit,
                        value: if sample == 0 { -0.0 } else { sample as f64 },
                    })
                    .unwrap();
            }
        }
        let samples = samples.finish().unwrap();
        super::super::result_projection::store_result_table(
            store,
            &fence,
            simulation_samples::RELATION_ID,
            &samples,
            &runtime.shared.pool(),
        )
        .await
        .unwrap();
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: fixture observations deliberately assert partial availability, not usable success.
        unsafe { store.seal_attempt(&manifest, "seal", TerminalClass::Partial, &[1]) }
            .await
            .unwrap();
        let read = store
            .read_results("projection", fence.attempt(), Duration::from_secs(60))
            .await
            .unwrap();
        let set = result_set_key(fence.attempt(), &fit_parameters::RELATION_ID.to_string());
        let page = store
            .result_output_page(
                &read,
                &set,
                &format!("{}:{absent}:value", fit_parameters::RELATION_ID),
                "0",
                0,
                u64::MAX,
                None,
                false,
                None,
                None,
                Some(true),
            )
            .await
            .unwrap();
        assert_eq!(page.cells.len(), 1);
        assert_eq!(page.blocks.len(), 1);
        assert!(
            read.sets()
                .iter()
                .find(|descriptor| descriptor.key == set)
                .unwrap()
                .batch_count
                > page.blocks.len() as u64
        );
        let mut missing = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.fit_parameters",
                absent,
                "value",
                "0",
                0,
                u64::MAX,
                None,
                None,
                Some(true),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let batch = missing.next_batch().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert!(batch.column_by_name("value").unwrap().is_null(0));
        assert!(missing.next_batch().await.unwrap().is_none());
        let mut zero = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.fit_parameters",
                second,
                "value",
                "0",
                0,
                u64::MAX,
                Some(0.0),
                Some(0.0),
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let batch = zero.next_batch().await.unwrap().unwrap();
        assert_eq!(
            batch
                .column_by_name("value")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .value(0)
                .to_bits(),
            (-0.0f64).to_bits()
        );
        let trajectory_set = result_set_key(
            fence.attempt(),
            &simulation_samples::RELATION_ID.to_string(),
        );
        let page = store
            .result_output_page(
                &read,
                &trajectory_set,
                &format!("{}:{second}:value", simulation_samples::RELATION_ID),
                "0",
                0,
                u64::MAX,
                None,
                true,
                Some(20.0),
                Some(22.0),
                None,
            )
            .await
            .unwrap();
        assert_eq!(page.outputs.len(), 1);
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(
            read.sets()
                .iter()
                .find(|descriptor| descriptor.key == trajectory_set)
                .unwrap()
                .batch_count,
            2
        );
        let mut trajectory = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.simulation_samples",
                second,
                "value",
                "0",
                0,
                u64::MAX,
                Some(20.0),
                Some(22.0),
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let batch = trajectory.next_batch().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), 3);
        let view = simulation_samples::View::try_from_batch_with_registry(
            &runtime.registry,
            &batch,
            &validation,
        )
        .unwrap();
        for index in 0..3 {
            let row = view.row(index).unwrap();
            assert_eq!(row.symbol_id, second);
            assert_eq!(row.unit_id, unit);
            assert_eq!(row.sample, 20 + index as i64);
            assert_eq!(row.time, (20 + index) as f64);
        }
        assert!(trajectory.next_batch().await.unwrap().is_none());
        drop(trajectory);
        drop(zero);
        drop(missing);
        drop(read);
    }
    #[tokio::test]
    async fn canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation() {
        let runtime = super::super::tests::runtime();
        let store = runtime.canonical_store();
        let source = store.edit("problem", None, "source", &[]).await.unwrap();
        store
            .begin_run(&RunRequest {
                key: "run".into(),
                revision: source.clone(),
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
        let validation = runtime.validation_context().unwrap();
        let mut builder =
            solve_metrics::Builder::with_registry(&runtime.registry, 4, &validation).unwrap();
        for (step, value) in [Some(1.0), Some(-0.0), None, Some(4.0)]
            .into_iter()
            .enumerate()
        {
            builder
                .push(solve_metrics::Row {
                    run_id: pse_operations::mint_id(),
                    step: step as i64,
                    namespace: "fixture".into(),
                    name: "temperature".into(),
                    kind: if value.is_some() {
                        NativeMetricKind::Real
                    } else {
                        NativeMetricKind::Unavailable
                    },
                    real: value,
                    integer: None,
                    boolean: None,
                    text: None,
                    unavailable: if value.is_none() {
                        Some(pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite)
                    } else {
                        None
                    },
                })
                .unwrap();
        }
        let checked = builder.finish().unwrap();
        let payload = result_blocks::encode_result_block(checked.batch()).unwrap();
        let name = solve_metrics::RELATION_ID.to_string();
        let set = result_set_key(fence.attempt(), &name);
        let key = result_batch_key(fence.attempt(), &set, 0);
        let metadata = BlockMetadata {
            key: key.clone(),
            result_set: set,
            batch: key,
            output: name.clone(),
            partition: "0".into(),
            ordinal: 0,
            start: 0,
            end: 4,
            rows: 4,
            columns: checked.batch().num_columns() as u64,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: payload.len() as u64,
            payload_digest: result_payload_digest(&payload),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        store
            .append_result_block(&fence, "write", &name, 0, &payload, 4, &metadata)
            .await
            .unwrap();
        let progress = super::super::ProgressEventDocument {
            phase: "fixture".into(),
            elapsed_seconds: 0.25,
            values: [(
                "exact_counter".into(),
                super::super::ProgressMetricDocument::Integer((1i64 << 53) + 1),
            )]
            .into(),
            incumbent: None,
            step: Some(0),
            sequence: Some(0),
            at: Some(1),
        };
        let progress_payload = serde_json::to_vec(&vec![progress]).unwrap();
        store
            .append_result_batch(&fence, "progress", "__progress", 0, &progress_payload, 1)
            .await
            .unwrap();
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: finite generated rows are explicitly classified as a partial fixture, not usable success.
        unsafe { store.seal_attempt(&manifest, "seal", TerminalClass::Partial, &[1]) }
            .await
            .unwrap();
        assert_eq!(
            runtime.run_record("run").await.unwrap().batch().num_rows(),
            1
        );
        assert_eq!(
            runtime
                .attempt_record(fence.attempt())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
        assert_eq!(
            runtime
                .result_manifest(fence.attempt())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
        let mut latest = runtime
            .latest_results(
                "problem",
                &[TerminalClass::Partial],
                "runtime.solve_metrics",
                0,
                4,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(latest.selection().attempt().key, fence.attempt());
        assert_eq!(latest.next_batch().await.unwrap().unwrap().num_rows(), 4);
        let mut history = runtime
            .progress("run", fence.attempt(), CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(history.run(), "run");
        assert_eq!(history.attempt(), fence.attempt());
        let events = history.next_page().await.unwrap().unwrap();
        assert_eq!(events.len(), 1);
        assert!(
            matches!(events[0].values["exact_counter"],super::super::ProgressMetricDocument::Integer(value) if value==(1i64<<53)+1)
        );
        assert!(history.next_page().await.unwrap().is_none());
        let cancel = CancellationToken::new();
        let mut reader = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                1,
                3,
                cancel.clone(),
            )
            .await
            .unwrap();
        assert_eq!(
            reader.selection().attempt().outcome.as_deref(),
            Some("partial")
        );
        let batch = reader.next_batch().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), 2);
        let values = batch
            .column_by_name("real")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert!(reader.next_batch().await.unwrap().is_none());
        let mut cancelled = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                cancel.clone(),
            )
            .await
            .unwrap();
        cancel.cancel();
        assert!(cancelled.next_batch().await.is_err());
        // Already yielded Arrow buffers remain independent of stream cancellation.
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("results.arrow");
        let mut export = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        export.export_ipc(&destination).await.unwrap();
        assert!(destination.is_file());
        assert!(!directory.path().join("results.arrow.incomplete").exists());
        let mut file = datafusion::arrow::ipc::reader::StreamReader::try_new(
            std::fs::File::open(&destination).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(
            file.schema().metadata().get("pse.canonical.attempt"),
            Some(&fence.attempt().to_owned())
        );
        assert_eq!(file.next().unwrap().unwrap().num_rows(), 4);
        assert!(file.next().is_none());
        let token = CancellationToken::new();
        let mut interrupted = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                token.clone(),
            )
            .await
            .unwrap();
        token.cancel();
        let missing = directory.path().join("cancelled.arrow");
        assert!(interrupted.export_ipc(&missing).await.is_err());
        assert!(!missing.exists());
        assert!(
            directory
                .path()
                .join("cancelled.arrow.incomplete")
                .is_file()
        );
        let original = std::fs::read(&destination).unwrap();
        let mut overwrite = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(overwrite.export_ipc(&destination).await.is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), original);
        // An admitted transport receipt can still be corrupt scientific IPC.
        // Failure after an already yielded prefix must not publish a complete file.
        store
            .begin_run(&RunRequest {
                key: "malformed".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let broken = store
            .claim_run(
                "malformed",
                "malformed-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let broken_set = result_set_key(broken.attempt(), &name);
        for (ordinal, start, end, bytes) in [(0, 0, 4, payload.clone()), (1, 4, 5, vec![8])] {
            let key = result_batch_key(broken.attempt(), &broken_set, ordinal);
            let block = BlockMetadata {
                key: key.clone(),
                result_set: broken_set.clone(),
                batch: key,
                output: name.clone(),
                partition: "0".into(),
                ordinal,
                start,
                end,
                rows: end - start,
                columns: checked.batch().num_columns() as u64,
                coordinate_min: None,
                coordinate_max: None,
                payload_bytes: bytes.len() as u64,
                payload_digest: result_payload_digest(&bytes),
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            store
                .append_result_block(
                    &broken,
                    &format!("malformed-write-{ordinal}"),
                    &name,
                    ordinal,
                    &bytes,
                    end - start,
                    &block,
                )
                .await
                .unwrap();
        }
        let closed = store
            .close_result_ingestion(&broken, "malformed-close")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: deliberately corrupt transport evidence is explicitly partial.
        unsafe { store.seal_attempt(&manifest, "malformed-seal", TerminalClass::Partial, &[1]) }
            .await
            .unwrap();
        let mut late = runtime
            .results(
                "malformed",
                broken.attempt(),
                "runtime.solve_metrics",
                0,
                5,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(late.next_batch().await.unwrap().unwrap().num_rows(), 4);
        assert!(late.next_batch().await.is_err());
        let mut late_export = runtime
            .results(
                "malformed",
                broken.attempt(),
                "runtime.solve_metrics",
                0,
                5,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let partial = directory.path().join("partial.arrow");
        assert!(late_export.export_ipc(&partial).await.is_err());
        assert!(!partial.exists());
        assert!(directory.path().join("partial.arrow.incomplete").is_file());
    }
}
