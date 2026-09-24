# Preview rows and choose formatting

A positive preview limit enables the recorder. An omitted formatter selects the default pretty formatter; a custom callback changes presentation.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| limit zero | No row preview is needed | Recorder disabled. |
| positive limit with None formatter | Default table rendering is sufficient | No custom callback required. |
| custom formatter | A different rendering is needed | Formatting can fail and adds work to completion. |

## Contract

**Accumulation.** Each partition stream retains slices up to its limit. On recorder drop, available partition previews are concatenated and the final batch is capped before formatting.
Claim `tracing.preview.1`; source_observation; evidence: source.

**Failure.** Formatter Err emits a warning; preview concatenation failure in the final recorder omits output. Concatenation in poll_next can return an error through the stream.
Claim `tracing.preview.2`; source_observation; evidence: source.

**Resource scope.** A displayed row cap is not a byte or peak-memory bound: partitions, concatenation, and shared sliced buffers matter.
Claim `tracing.preview.3`; source_observation; evidence: source.

**Executed scope.** Positive limit with no callback produces default formatted rows, limited to the requested display count; a callback changes formatting, zero disables preview, and a formatter error emits a warning without failing the query.
Claim `tracing.preview.observed`; runtime_observation; evidence: consumer.

## Implementation

- Set preview_limit; optionally supply Arc<dyn Fn(&RecordBatch) -> Result<String, ArrowError> + Send + Sync> through inference.
- Relate recording to recorder/stream lifetime and inspect the declared preview field.

## Limits and unknowns

- Preview is neither a complete result nor a random sample.
- No measured allocation/overhead bound across arbitrary schemas or partition counts.

## Exact contracts

- [`datafusion_tracing::options::InstrumentationOptions::preview_fn`](../operations/datafusion_tracing.options.InstrumentationOptions.md#op-9a19b737593814d4c01ca326) — `preview_fn: Option<std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>`
- [`datafusion_tracing::options::InstrumentationOptions::preview_fn`](../operations/datafusion_tracing.options.InstrumentationOptions.md#op-a7e49b396c5d69f07388e5ff) — `preview_fn: Option<std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>`
- [`datafusion_tracing::options::InstrumentationOptionsBuilder::preview_limit`](../operations/datafusion_tracing.options.InstrumentationOptionsBuilder.md#op-10115deab09736db979aa497) — `preview_limit: usize`
- [`datafusion_tracing::options::InstrumentationOptionsBuilder::preview_limit`](../operations/datafusion_tracing.options.InstrumentationOptionsBuilder.md#op-d4b2d47b13c9aa255d85e495) — `fn preview_limit(self, limit: usize) -> Self`
- [`datafusion_tracing::preview_utils::pretty_format_compact_batch`](../operations/datafusion_tracing.preview_utils.pretty_format_compact_batch.md#op-5a8fa15e2bb92e83eed1b7b8) — `fn pretty_format_compact_batch(batch: &datafusion::arrow::array::RecordBatch, max_width: usize, max_row_height: usize, min_compacted_col_width: usize) -> Result<impl Display, datafusion::arrow::error::ArrowError>`
- [`datafusion_tracing::preview_utils::pretty_format_compact_batch`](../operations/datafusion_tracing.preview_utils.pretty_format_compact_batch.md#op-cc2a4e58ce947d96244e03f2) — `fn pretty_format_compact_batch(batch: &datafusion::arrow::array::RecordBatch, max_width: usize, max_row_height: usize, min_compacted_col_width: usize) -> Result<impl Display, datafusion::arrow::error::ArrowError>`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/preview.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): Positive limit with no callback produces default formatted rows, limited to the requested display count; a callback changes formatting, zero disables preview, and a formatter error emits a warning without failing the query.
  Tests: preview_default_custom_disabled_and_error, partition_preview_cap_and_live_stream_lifetime
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
