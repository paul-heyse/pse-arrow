# Analysis and offline publication capabilities

Supporting inquiry for the [principal review](../../reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md); the [evidence overview](README.md) records integration and execution limits.

Bounded source investigation, 2026-10-18, for the diagnostics/causal-pipeline design review. Baseline: HEAD `e9d24396c`; existing dirty documentation preserved. This is library evidence and conditional design advice, not a formal review or acceptance result. No builds, tests, probes, services, dependency changes or environment reproduction campaign were run.

## Versions and evidence boundary

`Cargo.toml` and `Cargo.lock` agree on DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2, datafusion-tracing 55.0.0, tracing 0.1.44, tracing-subscriber 0.3.23 and tracing-perfetto-sdk-schema 0.13.1. `.config/diagnostic-tools.toml` selects Trace Processor 58.2 with an archive digest. CPU capture invokes host `perf` and `samply`; no version pin was found for either in those declarations, `pyproject.toml`, or the capture implementation. Their exact installed versions remain for the native-tool investigation to establish. Current Context7 documentation was fetched for DataFusion, Perfetto and Samply; decisive contracts below use pinned source or explicitly identify current documentation as discovery only.

The DataFusion and DataFusion-tracing skills supplied pinned contracts and selected source. Existing skill probes retain their original small-fixture scope; they were not rerun and do not establish current assembled performance, RSS, export reliability or offline recipient usability.

## Shared live/offline meaning

**Implemented source boundaries.** `python/pse/diagnostics.py` exposes passive checkpoint/image inspection, original detail pages and Arrow projection. Rust owns interpretation. `crates/pse-runtime/src/inspection_comparison.rs` already implements pure bounded explanation/comparison over borrowed invocation documents: versioned policy, exact invocation references and source JSON pointers, typed values, clock scope, partial flags, omitted statements/source items and optional next observations. It refuses mismatched source identity and withholds normalized native costs when premises fail. `inspection_documents.rs` contains small native observation types, not a report/pack owner.

**Proposed fit.** A shared analysis function over immutable admitted source documents can serve a live snapshot, saved checkpoint and offline pack without a second Python interpretation algorithm. Source selection and physical query execution can differ; source identities, clock/population definitions, missing/nonfinite rules, coverage and interpretation policy must agree. A view over mutable live data is not a frozen snapshot. Do not turn the existing invocation comparison into a general critical-path model merely by adding elapsed-time arithmetic.

`pse-schema/src/catalog/diagnostic_evidence.rs` and runtime `capture/evidence.rs` provide an existing relational boundary: coherent event key `(operation, producer, generation, event_sequence)` plus field ordinal; family sequence; exact seconds/nanoseconds; units; separate UInt64, Int64, Float64, Boolean and text values; availability and partial state. Empty events survive as ordinal zero. Projection rejects duplicate event identities and untagged nonfinite numbers. Original pages/declarations remain authoritative. Preserve these keys when selecting/pivoting fields; joining by iteration or timestamp alone can combine unrelated producers or generations. Derive per-event rows before aggregation, or field expansion will multiply event counts.

## Tabular/window analysis: built-ins before another engine

**Interface-checked.** DataFusion 55.1 supports relational filters/projections/joins, aggregates and ordered window frames; existing Arrow batches can be registered through native providers, and Parquet has established file-provider routes. This is a credible realization for selected numerical histories, phase summaries and comparisons, without bespoke grouping/sorting machinery or adopting a new analytics platform. Exact contracts: [DataFrame](https://docs.rs/datafusion/55.1.0/datafusion/dataframe/struct.DataFrame.html), [TableProvider](https://docs.rs/datafusion-catalog/55.1.0/datafusion_catalog/trait.TableProvider.html), [window frames](https://github.com/apache/datafusion/blob/55.1.0/docs/source/user-guide/sql/window_functions.md), [Parquet reader selection](https://docs.rs/parquet/59.3.0/parquet/arrow/arrow_reader/struct.ArrowReaderBuilder.html).

Filters/windows need semantic selection, not just a SQL predicate: retain a predecessor for counter deltas; respect reset/generation boundaries; distinguish sample points from the interval they describe; preserve whole coherent records and the source declarations/coverage needed to interpret them. Existing `pse-diagnostics/src/observation.rs` already treats absolute counter baselines, resets, coalesced samples and crossing windows explicitly. Default ordered RANGE and explicit ROWS differ with duplicate ordering keys. Missing is not zero, nor does an empty filtered result establish full capture coverage. Provider pushdown must accurately advertise Exact/Inexact/Unsupported; filtering before required context can change the answer.

**Limits.** `execute_stream` avoids retaining all output but does not bound blocking sorts, joins, window state or caller-retained batches. The configured pool does not account for every allocation. Parquet pruning depends on useful row-group/page statistics and predicates; an output LIMIT is not an input-work bound. Inspect the physical plan and qualify representative sizes before claiming performance. Relevant pinned skill briefs: `df.consume`, `df.pushdown`, `df.aggregate-window`, `df.relations`, `parquet.selection`. No custom UDF/aggregate or extra engine is justified by the inspected needs alone; unresolved scientific interpretation remains application-owned.

## Trace/profile interpretation and interval caveats

Existing `scripts/diagnostic_sql/timeline.sql` sums completed inclusive durations and explicitly disclaims exclusive CPU cost. `numerical.sql` selects whole source-event fields; `coverage.sql` reports nonzero error/data-loss statistics and disclaims completeness. These are useful starting questions, not causal reconstruction or time-window selection. `scripts/diagnostic_trace.py` executes one selected SQL file against one raw trace, retains bounded stdout/stderr on failure and returns status. It does not publish a typed query-result manifest tying rows to input/SQL/window/status.

**Interface-checked.** Perfetto 58.2 has SQL window functions, argument extraction, ancestry operators and `SPAN_JOIN`. Its [pinned getting-started guide](https://github.com/google/perfetto/blob/v58.2/docs/analysis/perfetto-sql-getting-started.md) warns that overlapping spans within one input partition silently produce incorrect span-join rows; partitions must be integers. Nested application slices therefore cannot be fed wholesale into this operator. The release also supplies [interval intersection macros](https://github.com/google/perfetto/blob/v58.2/src/trace_processor/perfetto_sql/stdlib/intervals/intersect.sql); inspect their exact preconditions before selecting them. Current Context7 pipe-syntax examples are not established for this pin.

**Proposed semantics.** For finite positive-duration intervals, use explicit half-open overlap and clipping to the requested window; retain original endpoints and distinguish clipped duration. Define instant and unfinished-slice handling independently. Excluding `dur < 0`, as the current timeline query does, excludes open intervals rather than proving none existed. Inclusive sums, union coverage, exclusive ownership time and longest dependency path are different quantities. Critical-path claims need actual dependency/wakeup/flow relationships and boundary coverage; temporal overlap or parent-child nesting alone does not establish causality. Trace-local `utid`/`upid` prevent PID/TID reuse errors; cross-trace joins still need the capture/process identity and a justified clock mapping.

`capture/cpu.rs` records `cpu-clock:u` at 99 Hz with DWARF stack allowance 32768, monotonic clock and inherited task scope; converts through `samply import --save-only`; retains raw perf data and tool logs. Source comments correctly disclaim exclusive operation CPU. The sample population omits kernel CPU and does not establish blocked time, exact invocation cost or scheduling causality. The code separately decodes resolved/unresolved frames, with an explicit leaf-only fallback. A successful converted-file write is not proof that every sample is symbolicated or that a remote recipient has the symbols. [Current Samply source/docs](https://github.com/mstange/samply) describe Firefox processed-profile output and separate symbolication; transfer to the host tool version remains unqualified.

Datafusion-tracing 55.0 wraps the physical plan it sees; later rewrites can change coverage. Its metric recorder reads native metrics and aggregates by name when the shared recorder drops. Units/populations remain the native metric contract; completion, early abandonment, filtering and exporter lifetime affect observations. This instrumentation does not supply operation-exclusive CPU or a complete causal graph. Sources: pinned skill `content/corpus/source/datafusion-tracing/{exec_instrument_rule,metrics}.rs` and capability briefs `tracing.execution`, `tracing.metrics`, `tracing.context`, `tracing.export`.

## Offline recipient formats and interrupted publication

| Artifact | Suitable purpose | Boundary |
|---|---|---|
| Markdown plus fully local HTML | Read conclusions, source-linked tables, bounded charts and limitations without executing PSE | Embed assets/data or use relative local paths; no CDN, fetch, symbol server or hosted profiler dependency. Keep large integers as strings in browser data. |
| Original tagged JSON/JSONL and raw traces/profiles | Independent inspection of original observations and retained coverage | Include original metadata and readable-prefix/end-state information; absolute producer paths alone are not portable links. |
| Arrow IPC | Typed selected tables, schema/metadata and precise integer transport; existing result export uses it | File writer needs a completed footer; a partial file is not automatically readable. Stream messages have different recovery semantics. |
| Parquet | Large selected tabular evidence with compression and selective reads | Row-group buffering and finalization matter; no need to emit alongside IPC for every small pack. |
| CSV | Small visible query tables for broad tools | Companion schema/tags needed for exact integer, null/nonfinite and units; spreadsheet inference can lose meaning. |
| Perfetto SQLite or Arrow export | Optional external tooling convenience | Different scopes and reload behavior; avoid duplicating all tables by default. |

Arrow/Parquet retain typed integers but do not recover categories erased before encoding. Keep tagged nonfinite/unavailable/overflow states distinct from SQL null; generic JSON numeric output and browser Number cannot be the sole authority for UInt64. Sources: local diagnostic projection and pinned [IPC writer](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/writer/index.html), [Parquet ArrowWriter](https://docs.rs/parquet/59.3.0/parquet/arrow/struct.ArrowWriter.html).

The [Perfetto 58.2 export contract](https://github.com/google/perfetto/blob/v58.2/docs/analysis/trace-processor.md#subcommand-export) distinguishes reloadable version-coupled `perfetto`, standard static-table `arrow_tar`, and SQLite including views. Runtime-created tables are not exported. These are convenience derivatives; retain original raw evidence for interpretation. A hosted Perfetto/Firefox UI is not, by itself, an offline pack reader.

**Proposed publication fit.** Prepare a private destination; copy selected raw evidence and metadata; derive one useful typed table format plus readable analysis; finalize writers; publish a relative-path index only after success. On interruption preserve an explicitly incomplete artifact, original statuses, error and known readable prefixes rather than presenting the directory as complete. Record identities, selection/window, clock scope, interpretation version, omissions and artifact availability needed to assess the answer; an environment closure or rerun campaign is outside this request. A digest can identify bytes but does not validate conclusions.

Existing precedents differ in scope: `observation.rs` publishes a checkpoint through a pending-file rename; detail capture flushes complete JSONL frames and advances a readable-prefix count; CPU capture inventories partial files before propagating errors. `python/pse/_workflow.py::export_results` documents final-path-on-success and sibling `.incomplete` IPC. Reuse the appropriate semantics rather than conflating them. object_store 0.13.2 [put/PutMode contracts](https://docs.rs/object_store/0.13.2/object_store/trait.ObjectStore.html) provide atomic single-object writes, not an atomic multi-file review pack. Generic rename may use copy/delete on a store; local directory publication and crash durability require their own narrowly stated contract.

## Remaining decision work

The coordinator must choose the supported analysis questions and population/clock semantics, which source context a selected window retains, pack completion/readability behavior, and the recipient's minimum offline interaction. Representative large-artifact feasibility, actual host profiler versions, cross-artifact clock alignment and offline symbol completeness remain unmeasured/unqualified. No new platform, mandatory redundant format, provenance campaign or rerun environment is recommended by this investigation.

## Implementation inquiry disposition (2026-10-18)

The installed optional tool observations were `perf version 7.0.14`, `samply 0.13.1` and
Trace Processor `58.2-add693d8b` (RPC API 14). These identify the tools examined; they are not
an environment compatibility proof. Samply's matching source accepts the perf-import
presymbolication flag but does not consume that setting on the import path. No symbol
sidecar or correction is adopted. Keep existing raw perf data, actual resolved/unresolved,
unwind/loss and leaf-only limitations and the existing readable summary. A successful
conversion alone cannot establish portable symbol resolution.

**Interface-checked:** the maintained selected-window query uses an explicit ordered
running maximum and islands/union composition, rather than applying `SPAN_JOIN` to nested
intervals. Its half-open clipped union and original/clipped inclusive populations are
separate. Canonical text companions preserve native unsigned event identities in Perfetto;
earlier traces without those companions yield unavailable values. Native JSON/Arrow remains
the full-UInt64 authority.

No additional speedscope, CTF, OTLP or symbol format is required for the selected local
report. No Heaptrack, Callgrind, BPF/off-CPU platform, Tokio Console, SUNProfiler, Caliper or
uProf integration is adopted: there is no remaining selected question requiring their
instrumentation after the available owner facts and CPU/timeline routes. Reopen an individual
candidate for a concrete unresolved allocation, native-phase, async-scheduling, blocked-time
or stack question that existing observations cannot discriminate. This is a bounded
no-adoption decision, not a conclusion that those tools lack useful capabilities.

Directory publication and registry IPC stream export remain the selected boundary. Current
DataFusion/Arrow source supports the existing local stream/query path; remote IPC-stream
loading can buffer the full object and is not adopted as a purported bounded remote reader.
No new provider, analytics engine or diagnostic store is introduced. Larger skew/growth and
observer/export cost assessment remains Plan47f's assembled qualification scope.
