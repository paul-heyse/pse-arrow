# Data, persistence, boundary and infrastructure library utilization: bounded supporting assessment

Inquiry **L2** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`L2-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `library-research` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

Baseline: `main` at b01a45d71a5f5ab29991ad193679f5588ac3662f (read-only; no builds, tests, services or probes run).
Labels are local (`L2-nn`). Evidence is source-traced unless stated otherwise. No speed or capacity claim below is *Measured*.

## Per-library table

| library@pin | integration owner(s) | capabilities used (path) | notable unused relevant capabilities | bespoke equivalents found | assessment |
|---|---|---|---|---|---|
| SurrealDB SDK 3.3.0 (vendored and patched, `protocol-ws`, RocksDB server) | `pse-operations` only (`canonical*.rs`, generated `surreal.surql`/`surreal.rs`) | 42 server-side `DEFINE FUNCTION`s (`fn::pse_execution_v1::*`, `pse_study_v1`, `pse_retention_v1`, `pse_analysis_v1`); multi-statement `BEGIN…COMMIT` in one RPC; `INSERT INTO t $rows RETURN NONE` bulk inserts; key-array `SELECT` (`$keys.map(type::record…)`); 101 indexes; `SELECT … FOR UPDATE` guards; native `Bytes`/`Decimal`/`RecordId` values (`canonical_codec.rs`); `TIMEOUT` clause | LIVE SELECT/change feeds (a 20 ms poll in `canonical_staging.rs::stage_edits`, which is rare); byte-sized rather than row-sized pages; no shared-lock mode exists at 3.3 | request deadline/cancellation lifecycle patch to the SDK (`vendor/surrealdb/PATCH.md`, about 2.8k changed lines ignoring whitespace, mostly `engine/remote/ws`); conflict retry loops (`canonical.rs::protected_query`, `conflict_backoff`, `canonical_staging.rs::retry_stage_mutation`) | **finding-candidate** (L2-02); patch is a candidate (L2-04) |
| Arrow 59.3 family (`arrow-ipc`, `-select`, `-cast`, `-ord`, `-row`) | `pse-columnar`, `pse-relations`, `pse-schema`, `pse-runtime` (result transport) | `RowConverter`+`take_record_batch` canonical ordering (`pse-columnar/src/canon/stages.rs::admit_and_order`); `take`/`FilterBuilder`/`concat_batches`/`cast_with_options` (`pse-relations/src/columnar.rs`); IPC `StreamWriter`/`StreamReader` per result block (`pse-runtime/src/workflow/result_blocks.rs`); C stream FFI export | `StreamDecoder` over an in-memory `Buffer` (zero-copy, slices instead of allocating from advertised sizes); `StreamWriter::try_new_buffered`; `Buffer::from(bytes::Bytes)` | ~400-line flatbuffer preflight (`result_blocks.rs::preflight_block`, `frame`, `layout`, `schema_field`) run on both encode and decode | candidate (L2-03) |
| parquet 59.3 (`arrow`,`async`,`object_store`) | `pse-runtime/src/authoring_driver/document/data.rs` | sync `ParquetRecordBatchReaderBuilder` over `bytes::Bytes` for authored data documents | `async`/`object_store` features have no non-test consumer | — | adequate (feature residue noted in L2-01) |
| DataFusion 55.1 (default features) | `pse-engine` (session/, operation/, provider/, cache_service/), `pse-relations`, `pse-rules`, `pse-runtime/src/physical.rs` | `LogicalPlanBuilder` plans, analyzer+optimizer+physical planner (`session/preparation.rs::prepare_scoped`), custom `TableProvider`s, ~15 `ScalarUDFImpl`/2 `AggregateUDFImpl`, `UserDefinedLogicalNodeCore`+`ExtensionPlanner` (cache, contract, operation), `ExtensionTypeRegistry` (`session/registry.rs`), memory pools | `MemTable::with_sort_order` (sorted sources are re-sorted in `physical.rs::PhysicalInventory::load`); DF file sources/caches (unused, see L2-01); `PREPARE`/param values only in tests | metadata re-derivation passes (`session/admission.rs::restore_semantic_fields`) and metadata-preserving UDFs (`preserve_field`, `retain_metadata`, `list_field`, …), which a library gap justifies (see §4); bespoke pools (`CancellingPool`, `ReleaseNotifyingPool`, `ScopedPool`) | used as an engine, not as a container; adequate at the library level, depth owned by F |
| object_store 0.13.2 + instrumented-object-store 55.0 | `pse-engine/src/stores.rs`, `stores/admission.rs::SharedIo` | local filesystem store wrapped in instrumentation plus a shared I/O semaphore | `LimitStore` (rejected with a stated reason: one gate across stores) | `SharedIo` gate | **finding-candidate** as residue (L2-01) |
| datafusion-tracing 55.0 | `pse-engine/src/session/assurance.rs::instrument` | physical-optimizer instrumentation rule, rule-phase spans, query planner restored; policy `Off` by default | — | — | adequate (proportionate) |
| pyo3 0.29.2, pyo3-arrow 0.19, pyo3-async-runtimes 0.29 | `pse-py` | `py.detach` around every native call; Arrow C stream export via `PyRecordBatchReader` (`inspection/stream.rs::TableStream`); one process tokio runtime shared with the async bridge (`inspection/runtime.rs::acquire`) | inbound Arrow/buffer-protocol inputs (not needed: inputs are small documents) | — | adequate (strength) |
| tokio 1.53 / tokio-util 0.7 | everywhere async | multi-thread runtime, `CancellationToken`, task-local deadlines (`canonical_transport.rs::OPERATION_DEADLINE`) | — | — | adequate |
| blake3 1.8.7 (`rayon`) | `pse-ids` (frame, preimage, derive, encoding) | `Hasher::new_derive_key`, incremental framing | `update_rayon` (feature compiled, never called); `update_mmap_rayon` | `ResourceBudget::hashing_may_use_pool` is exposed to Python but no code reads it | candidate (L2-05, low) |
| sha2 0.11 | `pse-buildinfo/identity.rs::FileObservation::capture_scoped`, `value_digest` | streamed SHA-256 (SHA-NI is detected at run time; host has `sha_ni`) | BLAKE3 for internal observation digests | — | adequate as a library; repetition is owned by H (L2-06) |
| serde_json 1.0.151 | many crates; Python document intake (`pse-py/src/documents.rs`) | receipts, documents, policies | — | warm seeds encode f64 as `u64` bits (`durable.rs::bits`), a strength | candidate (L2-07, preservation: `float_roundtrip` is enabled only through `clarabel`) |
| quick_cache 0.6.24 | `pse-runtime/src/math/retention/weighted.rs` | `unsync::Cache` with weighter inside a `Mutex` (hot 1.0, ghost 0) | `sync::Cache::get_value_or_guard_async` (coalescing) | `pse_columnar::flight::Flights` (single flight that owns completion, carries a native cancel flag, and has capacity, retiring and panic states) | adequate: semantics exceed the library's guard API |
| tracing 0.1.44 / tracing-subscriber | runtime, compiler, math, engine | coarse `info_span!` per phase (`pse-math/src/execution.rs`, `pse-compiler/src/workspace.rs`) | — | — | adequate |
| serde-saphyr, toml, schemars, winnow, sqlparser 0.62, uuid, rustc-hash, typed-index-collections, enum-map | authoring, codegen, schema fingerprint | cold-path parsing/schema generation | — | — | examined shallowly; adequate |
| pyarrow 25.0.1, numpy (extra), msgspec, attrs, cattrs | `python/pse` | `__arrow_c_stream__` consumption, `to_numpy(zero_copy_only=…)` (`_array.py`), msgspec JSON documents | — | `codec.structure_rows` (helper, not on a production path) | adequate |

## 1. Scope examined and coverage limits

**Examined (source-traced):**
- Workspace pins and resolved features (`cargo metadata --offline`).
- SurrealDB integration: `canonical.rs` (transport, `protected_query`, `PROTECTED_BEGIN`, open), `canonical_transport.rs`, `canonical_codec.rs`, `canonical_execution.rs::append_execution_batch`, the generated SurrealQL (`fn::pse_execution_v1::touch`/append/claim/seal and the membership indexes), and the paging queries in `canonical_selection.rs`/`canonical_staging.rs`.
- The vendored SDK patch, diffed against the registry `surrealdb-3.3.0`, plus `surrealdb-datastore-3.3.0::tx.rs::getu_key` and `surrealdb-kvs-rocksdb-3.3.0` (OptimisticTransactionDB).
- Result IPC: `result_blocks.rs`, `result_projection.rs`, `connected_results.rs::export_ipc`; arrow-ipc 59.3 reader source.
- DataFusion integration: `session/preparation.rs`, `cache_service/*`, `stores*`, `resources.rs`, `assurance.rs`, `physical.rs`, and the list of custom extension impls.
- pyo3 boundary: `inspection/stream.rs`, `inspection/runtime.rs`, `workflow.rs`.
- Hashing: blake3 and sha2 call sites. Also quick_cache and `Flights`.
- Python package imports and conversions.

**Not examined, or examined shallowly:**
- DataFusion plan-pass counts per edit→re-solve and validation repetition (owned by F/A).
- The SurrealDB RPC count on the critical path, retention scans and PE03's read-side composition (E).
- Connection topology under 16 cases (D).
- Hashing frequency and observation repetition (H).
- Build features (G).
- winnow/sqlparser/serde-saphyr/toml/schemars internals; Python `adapters/`, `parity/`, `authoring/`.
- SurrealDB query-planner behaviour, which needs an `EXPLAIN` against a server (proposed in §8).

Context7 was not needed: pinned source and the `datafusion`/`neo4j-surrealdb` skills answered the questions.

## 2. Complete operations examined (library lens)

- **Result publication (per relation).**
  - Steps: `result_projection.rs::store_result_table` → `ResultCursor::next_batch` → `encode_result_prefix` (`StreamWriter` into a `Vec`, then `preflight_block` of its own output) → `result_payload_digest` (BLAKE3) for the block descriptor → `CanonicalStore::append_result_block_cells` → `append_execution_batch` (a second digest of the same payload; `payload.to_vec()` then `encode_bytes` `to_vec()`) → one WS RPC that runs `fn::pse_execution_v1::append` (a transaction with bulk `INSERT … RETURN NONE`, compact acknowledgement).
  - Crossings: one RPC per block. Live state: the attempt fence. Window ≤ 1024 rows, block ≤ 512 KiB.
- **Result reopen (per block).** Protected read transaction (`PROTECTED_BEGIN`: `FOR UPDATE` on the `retention:<problem>` guard) → `decode_bytes` (`to_vec` copy out of `bytes::Bytes`) → digest check (`validate_result_batch`) → `decode_result_block` (`preflight_block`, then `StreamReader::try_new(Cursor)`, which copies again and validates) → `FieldCheckedBatch` → Python as an Arrow C stream (`TableStream`, zero-copy, `detach`).
- **Source selection (edit/prepare).** `selection_membership_page`: one protected transaction per 64 memberships (`ORDER BY key LIMIT 64`, with the interval predicate `from_sequence <= $s AND (to_sequence = NONE OR to_sequence > $s)` filtered after the `membership_pages(problem,key)` index).
- **Run lifecycle.** `begin_run`/`claim`/`seal` server functions each call `fn::pse_execution_v1::touch('retention:'+problem)`, which increments `generation` on the per-problem guard record.
- **Physical inventory load (cached by `physical_cache.rs`).** For each source relation: `LogicalPlanBuilder::scan(...).sort(primary_key)` → `prepare_many` → `PreparedComputation::execute_group` (`physical.rs::PhysicalInventory::load`).
- **Runtime construction.** `pse-engine/src/resources.rs`: `NativeCacheService::new(CacheBudget::for_memory(..))` eagerly reserves envelopes in the DataFusion pool; `RuntimeEnvBuilder::with_cache_manager(NativeCacheService::unbound_config())` and `with_object_store_registry(ObservedStores::new(concurrent_loads))`.

## 3. Candidate findings

### L2-01 — File-source cache and I/O machinery outlived the Delta/Parquet route it served

**Observed cause.**
- Plan 28 removed Delta. Commit 06302af77 deleted `pse-catalog` delta/cache_service and the only production caller of `pse_engine::cache_service::bind_state` (`-        pse_engine::cache_service::bind_state(location, state)`).
- What remains, with no non-test consumer:
  - `NativeCacheService::{bind_state, bind_state_with_generation, source_generation, bind_store, config}`; namespaced file metadata/statistics/listing `Envelope`s (`cache_service/{envelope,namespace}.rs`).
  - `admit_load`/`LoadGuard`/`inflight_bytes` (only `cache_service/tests.rs` calls `admit_load`).
  - The Parquet predicate-cache allowance (`load.rs::admit_predicates`, `operation/ownership.rs` `ReaderAllowance`).
  - `ObservedStores` + `stores/admission.rs::SharedIo` + `instrumented_object_store` wrapping a local filesystem that no provider reads; `pse-testkit/src/counting_store.rs`.
- Searches for `ListingTable`, `register_parquet`, `ParquetReadOptions`, `FileScanConfig`, `ParquetSource` and `bind_state(` callers outside `cache_service/` in `crates/` found none outside tests. The one `DataSourceExec` reference is a downcast in `session/round.rs`.
- Production construction makes the dead path observable:
  - `resources.rs` builds `NativeCacheService::new(caches, &pool)`, whose `Envelope::new` does `reservation.try_grow(bytes)` for `metadata_bytes` (`portion.min(50 MiB)`) and `statistics_bytes` (`portion.min(16 MiB)`) under the default `CacheBudget::for_memory` (`pse-runtime/src/settings.rs`).
  - It then hands DataFusion `NativeCacheService::unbound_config()` (all file-cache limits 0). The reserved envelopes therefore can never be filled.
  - `validate` also counts `inflight_bytes = 2·portion` and `predicate_total_bytes` against the pool, for loads and readers that never occur.

**Consequence.**
- Under the 16-case and sweep premises, each `SharedRuntime` pool starts with up to ~66 MiB of DataFusion pool capacity reserved for nothing, plus inflated budget-validation totals. The total can refuse a small explicit budget that would otherwise suffice.
- Development: about 2.4k lines (`cache_service/` + `stores*`) plus `object_store`/`parquet` async features and DataFusion's default `parquet`/compression features stay in every engine build and every review (H26/H27, DP-16; AGENTS "delete legacy code as soon as it is provably replaced").

**Simpler conforming alternative.**
- Delete the file-cache envelopes, store generation bindings, load gate, predicate allowance, `SharedIo` and the instrumented-store registry. Keep `syntax` and `model` caches and the query/output admission gates, which do have consumers.
- Use DataFusion's default `CacheManager`/`DefaultObjectStoreRegistry` if file sources return. Library-owned `LimitStore` and DataFusion's built-in file caches would serve a future file route without a pre-reserved envelope.
- Consider `default-features = false` on `datafusion` with the features the SQL/expression surface actually exposes (G owns build effects).

**Preservation.**
- Cache reports exposed to Python (`CacheReport` fields `metadata`/`statistics`/`listing`) would change shape. Decide whether the report contract keeps zero rows or drops them.
- Keep the `syntax`/`model` cache accounting and `admit_query`/`admit_output` gates.
- If authored invariants can call DataFusion regex/crypto/encoding functions, those features are part of the language surface and must stay.

**Uncertainty.** A test-only or Python inspection path may rely on `cache_service::inspection::bind` statistics rows. The exact reserved bytes depend on the configured `memory_limit_bytes`.

**Label.** *Interface-checked* (source-traced, plus git history).

**Verification.** `rg` for callers (done; reproducible). Inspect the pool `reserved()` immediately after `SharedRuntime::build` under the default budget (light probe P2).

### L2-02 — One per-problem guard record is both a lifecycle counter and the lock every protected read takes

**Observed cause.**
- Every protected immutable read starts with `PROTECTED_BEGIN` (`canonical.rs`): `SELECT * FROM type::record('canonical_guards','retention:'+$problem) FOR UPDATE`.
- In SurrealDB 3.3 on RocksDB, that is `getu_key` → `OptimisticTransaction::get_for_update(exclusive=true)` (`surrealdb-datastore-3.3.0/src/tx.rs::get_record_for_update`/`getu_key`; `surrealdb-kvs-rocksdb-3.3.0` `get_for_update_opt` on `OptimisticTransactionDB`). It registers the key for conflict validation at commit.
- The same record's `generation` is incremented by many ordinary lifecycle transitions:
  - `fn::pse_execution_v1::touch` (called from begin_run, claim, seal and per selected source in the generated `surreal.surql`);
  - pin renewal (`canonical_results.rs::renew`);
  - stage begin/close and source edit activation (`canonical_staging.rs` `BEGIN_STAGE`/`EDIT`);
  - product admission (`canonical.rs` `ADMIT_PRODUCT`).
- Any of these that commits during a reader's transaction aborts the reader with `TransactionConflict`. `protected_query` then retries up to `GUARDED_ATTEMPTS = 32` with `conflict_backoff` of 10–160 ms plus 0–63 ms jitter. The code comments name the scenario: "sixteen independent readers … contend on one problem's retention guard".
- Membership selection does this once per 64 rows (`canonical_selection.rs::selection_membership_page`, `membership_page`).

**Consequence.**
- Sixteen cases on one flowsheet, or a study's points sharing a problem: every run begin, claim and seal, plus every pin renewal, invalidates every in-flight protected read for that problem. Reads that are logically independent of each other and of run creation serialize through backoff sleeps.
- The amplification grows with concurrency and with selection size (pages of 64). That is H8/H23: the exclusion is broader than the effects it protects; AP-07/G9.

**Simpler conforming alternative.**
- Split the guard by the conflicts it must create. The reclaimer must conflict with readers and with root creation. Readers must not conflict with each other or with root creation.
  - Root creators and lifecycle steps touch a `roots:<problem>` key that only the reclaimer locks.
  - The reclaimer writes a `reclaim:<problem>` key that readers lock `FOR UPDATE`.
  - The reclaimer locks both.
- Alternatively, readers lock only their own pin record. The reclaimer locks the pins it inspects plus a pin-creation counter.
- SurrealQL 3.3 has no shared-lock mode. Write skew must be handled with `FOR UPDATE` (skill `SB091`), so key partitioning is the library-conforming tool.

**Preservation.**
- Reclamation must never delete rows visible to a live pin.
- Phantom protection between root creation and the reclaimer's range scan must remain. That is why the bump exists: SurrealDB OCC does not detect phantoms.
- Exact-identity replay and the uncertain-completion rules must be unchanged.

**Uncertainty.** Conflict frequency is not measured. Retries are not counted; only exhaustion is logged (`operation_failed`). If the reclaimer also relies on reader-side bumps, the split needs care. E owns the protocol; this is the library-semantics evidence.

**Label.** *Interface-checked* (SDK, datastore and RocksDB source traced). Effect not *Measured*.

**Verification.** Add a retry counter in a probe build, or run `canonical_execution_sixteen_retention_roundtrips_overlap_source_staging_and_private_pins` with a counter (P1).

### L2-03 — Result-block IPC framing is re-verified on the writer's own output and copied 3–4 times per crossing

**Observed cause.**
- Encode, `result_blocks.rs::encode_block`: bounded `StreamWriter` → `preflight_block` re-parses its own just-written flatbuffers (H19: re-proving what the writer established).
- Write crossing: `append_execution_batch` copies `payload.to_vec()`; `canonical_codec::encode_bytes` copies again (`Bytes::from(value.as_slice().to_vec())`).
- Read crossing: `decode_bytes` copies (`value.to_vec()`) out of SurrealDB's `bytes::Bytes` (`surrealdb-types-3.3.0/src/value/bytes.rs`). `decode_block` runs the bespoke `preflight_block` and then `StreamReader::try_new(Cursor::new(bytes))`, which allocates and copies metadata and body.
- The documented reason for the bespoke preflight is to refuse advertised allocations "before StreamReader can resize a metadata/body buffer". `arrow-ipc-59.3.0/src/reader/stream.rs::StreamDecoder::decode` over a complete in-memory `Buffer` slices messages (`buffer.slice_with_length`) rather than allocating from advertised sizes, and decodes zero-copy (`with_require_alignment(false)` copies only unaligned buffers).
- `result_payload_digest` runs twice on each written payload (`result_projection.rs` descriptor and `append_execution_batch`).

**Consequence.** Proportional work per ≤ 512 KiB block, not structural amplification. It matters for long trajectories and fitting outputs, where block counts grow, and for reopen of large retained results (H16 movement, H19).

**Simpler conforming alternative.**
- Decode with `StreamDecoder` over `Buffer::from(bytes::Bytes)` taken directly from the SurrealDB value (`Bytes::into_inner`). Keep only the semantic checks the decoder does not make: registry schema equality, one batch, row count, no compression or variadic buffers, end-of-stream.
- Skip the encode-side self-preflight, or keep it under `debug_assertions`.
- Carry `bytes::Bytes` through `pse_model::Bytes`.
- Hash once and reuse the digest for both descriptor and batch.

**Preservation.**
- Untrusted-input bounds (field/depth/row caps), exact IEEE bits, refusal of compression and cross-block dictionaries, and registry schema identity.
- Arrow validation stays on (no `with_skip_validation`).

**Uncertainty.** Whether `StreamDecoder` covers every refusal the bespoke `layout` walk makes (for example buffer overlap ordering). A differential fuzz against the existing refusal tests would settle it.

**Label.** *Interface-checked*.

**Verification.** Run the existing `canonical_result_blocks_unit` refusal tests against a `StreamDecoder` variant (proposal only).

### L2-04 — The vendored SurrealDB SDK carries a substantial private lifecycle patch

**Observed cause.**
- `vendor/surrealdb/PATCH.md` describes request-local deadlines and cancellation, bounded admissions (32+2), replay limits and a selection checkpoint.
- `diff -rwB` against the registry `surrealdb-3.3.0/src` shows 2,821 changed lines: `engine/remote/ws/mod.rs` 894, `ws/native.rs` 220, `method/query.rs` 60.
- Upstream 3.3.0 offers only `Config::query_timeout`/`transaction_timeout`. 3.3.0 is the newest release (`gh api …/releases`: v3.3.0 2026-09-28, v3.2.5 2026-10-06).
- `vendor/surrealdb/target` holds 1.4 GiB of build output (a G concern).

**Consequence.** Not runtime waste. It is integration burden: every SDK upgrade must re-port the patch, and the SDK's engine is coupled to `canonical_transport.rs`. It is justified by a real library gap (H27, DP-16).

**Alternative.** Upstream the request-context and bounded-route feature, or reduce the patch to a thin `RequestContext` hook. Until then, record a pins hold.

**Label.** *Interface-checked*. This is a candidate, not a finding.

### L2-05 — BLAKE3 parallel hashing is enabled and advertised but never used

**Observed cause.** The workspace pins `blake3` with `rayon`. `ResourceBudget::hashing_may_use_pool` (`pse-runtime/src/budget.rs`, also exposed through `settings.rs` to Python) is documented as permitting `blake3::update_rayon`. No production code reads the field and no `update_rayon`/`update_mmap*` call exists.

**Consequence.** A dead user-visible knob, plus a compiled-but-idle feature (H27). Negligible runtime effect.

**Alternative.** Remove the knob and the feature, or wire it where whole-artifact hashing is large (see L2-06).

**Label.** *Interface-checked*.

### L2-06 — Library option for whole-file observation hashing (H owns frequency)

**Observed cause.** `pse-buildinfo/identity.rs::FileObservation::capture_scoped` SHA-256-hashes complete files (artifact, executable, every loaded object in `LocalRuntimeObservation`) through 64 KiB reads. It is reached from `pse-runtime/src/math/portable.rs` local reconstruction. sha2 0.11 uses SHA-NI on this host.

**Library-owned options.**
- BLAKE3 for internal identities: multi-GB/s single-thread with AVX-512, and `update_rayon` is available. mmap+rayon carries a SIGBUS risk on truncation, which the replacement-detection contract must address.
- Better still, reuse by `(dev, ino, size, mtime, ctime)` within a process. That is H's remedy.

Only `value_digest`'s "external SHA256" naming suggests an external comparison. No external consumer was found in `crates/`.

**Label.** *Proposed*. Defer to H on whether repetition remains per body or per case.

### L2-07 — Exact float parsing of JSON documents depends on a feature enabled only by `clarabel`

**Observed cause.**
- Resolved `serde_json` features include `float_roundtrip`. `cargo tree -e features -i serde_json` shows it comes only from `clarabel v0.11.1`.
- Numerical policies with `f64` fields (`pse-model/src/numerics.rs`, for example `stationarity`, `complementarity`) are decoded from Python-sent JSON (`pse-py/src/documents.rs` `serde_json::from_slice`) and enter request identities.
- Without `float_roundtrip`, serde_json's parser may differ by 1 ULP from the shortest round-trip text msgspec emits.
- Warm seeds correctly avoid this by encoding bits (`durable.rs::bits`/`floats`).

**Consequence.** If the solver dependency set changes, identities and tolerances could change silently. This is a correctness and preservation risk, not efficiency.

**Alternative.** Declare `float_roundtrip` in `[workspace.dependencies] serde_json`. That is one declaration (prime directive 3).

**Label.** *Interface-checked*.

### L2-08 — Fixed 64-row protected pages with a post-filtered version interval (seam to E/A)

**Observed cause.**
- `canonical_selection.rs::selection_membership_page`/`membership_page`: `… WHERE problem = $problem AND key > $after AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) … ORDER BY key LIMIT 64`.
- The available index is `membership_pages (problem, key)` (generated `surreal.surql`). The interval is not part of it, so superseded memberships in the key range are scanned and discarded.
- Each page is a separate protected transaction (L2-02).
- The message limit is 4 MiB (`RESULT_MESSAGE_BYTES`). Membership rows are small, so 64 rows is far below what fits.

**Consequence.** Cold selection of a large flowsheet costs N/64 sequential protected RPCs. With retained history, pages scan dead rows (H25, H8).

**Alternative.** Byte-bounded pages. A `membership_active`-style compound index that includes the open-interval condition, for example storing `to_sequence = NONE` as a sentinel in an index `(problem, to_sequence, key)` for current-revision reads.

**Uncertainty.** Whether 3.3's planner pushes `ORDER BY key LIMIT` through the compound index is unverified (P3).

**Label.** *Proposed*.

## 4. Opportunities and uncertainties that are not findings

- **DataFusion metadata preservation** (`restore_semantic_fields`, `preserve_field`, `retain_metadata`, `list_field`, `typed_concat`, …).
  - At 55.1, `DFExtensionType` customizes only pretty-printing (skill `content/api/datafusion_common.types.extension.md`: "Currently, the following operations can be customized: Pretty-printing"). So these wrappers fill a real library gap, and the code already uses `ExtensionTypeRegistry`.
  - Track upstream extension-type semantics (55.2.0 is the latest tag). The number of passes per prepare is F's question.
- **Sorted sources re-sorted.** `PhysicalInventory::load` sorts each source by primary key. Providers declare no ordering (no `with_sort_order`/`output_ordering` on source providers). Declaring an ordering would let `EnforceSorting` elide the sort. Physical admission is cached (`physical_cache.rs`, 16 MiB), so this is cold.
- **IPC export** (`connected_results.rs::export_ipc`) writes `StreamWriter` to an unbuffered `File`. `try_new_buffered` exists. Cold path.
- **`SessionContext::state()` clone** per `bound_state` clones the function registries. Small; not material.
- **quick_cache vs `Flights`.** The bespoke single flight carries native cancel flags and completion ownership, which `get_value_or_guard_async` cannot express. Adequate.
- **LIVE SELECT** could replace the 20 ms poll waiting for a concurrent identical source edit (`stage_edits`). The path is rare; not material.
- **SurrealDB write path** already uses server functions, single-RPC transactions and bulk `INSERT … RETURN NONE`. PE03's append echo is gone (see §6).
- **`pse-py` production build** enables `pse-operations/test-support` (`crates/pse-py/Cargo.toml`). That is a seam for E/G, not a library question.
- **Arrow-ipc `lz4`/`zstd`** are compiled through feature unification while result blocks refuse compression. Compression could help growing retained history, at the cost of a decode CPU trade-off and changed digest semantics. Not proposed.

## 5. Strengths and preservation constraints

- **The Python boundary is zero-copy and releases the GIL.** It uses Arrow C stream export via pyo3-arrow (`TableStream::__arrow_c_stream__`), `py.detach` on every native call, one shared tokio runtime registered with pyo3-async-runtimes, and studies run natively rather than as per-point Python calls.
- **SurrealDB is used natively.** Server-side functions hold transition logic; one RPC carries a whole transaction; bulk inserts; key-array reads; typed exact codecs (unsigned values as `Decimal`, `RecordId` identity, IEEE bits as bytes); an interpretation/schema-digest marker check on open instead of a schema diff.
- **Arrow kernels are used where they fit.** `RowConverter`, `take`, `FilterBuilder`, `concat_batches`, `cast_with_options` (`CastOptions{safe:false}`).
- **Exactness by construction.** Warm seeds as `u64` bits; float bits in export metadata.
- **Tracing is proportionate.** It is off by default, uses coarse phase spans, and restores the planner to avoid plan-text spans.

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| PE02 (tiny IPC blocks from 64/width) | resolved (work removed), E to confirm | `result_projection.rs::store_result_chunk`: window = `RESULT_INDEX_RECORDS/width` capped at 1024 rows, prefix by metadata extent and 512 KiB; comment "scalar count is only a defensive extent ceiling" |
| PE03 (append echoes payload; two RPCs per read) | residual: append echo resolved; read composition not examined | generated `fn::pse_execution_v1::append` returns `{key,attempt,result_set,ordinal,digest,row_count,request}` |
| 10-09 F09 (complete Arrow map) | not examined (E) | — |
| EF02 (immutable admission discards native owner) | not examined (F) | — |
| EF04 (feature stabilization gives small roots heavy deps) | residual at library level; G owns | DataFusion default features (parquet, compression, crypto/regex), parquet `async`/`object_store`, arrow-ipc lz4/zstd resolved without production consumers (L2-01) |
| RC02 (durable fast hashing, unselected) | confirmed still present (dead knob) | `hashing_may_use_pool` unread; no `update_rayon` (L2-05) |
| EI05 (semantic BLAKE3, bucket prehash, interner) | not examined (H) | — |
| 10-09 evidence columnar notes | not examined (F) | — |
| Plan 28 Delta retirement (implied "removed") | residual | `cache_service`/`stores` file-source machinery remains reserved and wired (L2-01) |

## 7. Candidate rule impacts

- **AGENTS "delete legacy code as soon as it is provably replaced"** is already the governing rule for L2-01. No change is needed; the closing review of Plan 28 should have caught it.
- **Pins record for the SurrealDB vendor patch (L2-04).** `[workspace.metadata.pse.pins]` should carry the hold and its revisit trigger (upstream request contexts). If it already does, no change.
- **Prime directive 3** supports L2-07: declare `serde_json/float_roundtrip` once, where the dependency is declared.

## 8. Proposed light probes

- **P1 (L2-02).**
  - Question: how often do protected reads abort on the guard under 16 concurrent clients?
  - Method: run the existing `pse-operations` canonical test `canonical_execution_sixteen_retention_roundtrips_overlap_source_staging_and_private_pins` (feature `canonical-tests`) under `scripts/pse-env --resource-class light`, with a temporary counter in `protected_query` on the `TransactionConflict` arm, logged at test end.
  - Discriminating outcome: near zero retries means L2-02 is low priority; retries per read ≥ 1, or measurable backoff time, makes it a finding.
  - Requires a scratch code edit and the canonical server fixture, so the coordinator decides.
- **P2 (L2-01).**
  - Question: how much does a default runtime reserve for dead caches?
  - Method: in a scratch test, `SharedRuntime::build(ResourceBudget{..default with memory_limit_bytes = 4 GiB})`, then read the DataFusion pool `reserved()` and the `CacheReport` rows for `pse.cache.metadata`/`pse.cache.statistics`.
  - Expected: about 66 MiB (50 + 16) reserved with zero entries. Zero would refute the residue cost.
- **P3 (L2-08).**
  - Question: does the membership page query use the `membership_pages` index with limit pushdown?
  - Method: against the fixture database, `SELECT * FROM canonical_memberships WHERE problem = $p AND key > '' AND from_sequence <= $s AND (to_sequence = NONE OR to_sequence > $s) ORDER BY key LIMIT 64 EXPLAIN FULL;`
  - Discriminating outcome: an index range iterator with early limit versus a table or full-index scan, and the scanned versus returned counts after creating 10× superseded memberships.

