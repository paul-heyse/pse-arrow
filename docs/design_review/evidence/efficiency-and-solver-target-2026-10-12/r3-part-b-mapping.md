> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R3 (library-research), Part B: obligations mapped onto the R3 libraries. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R3 Part B: mapping the W-A/W-B obligations onto observation, results and analytics

Inputs: `w-a-interactive-obligations.md`, `w-b-multicase-obligations.md` (both at `6c368c88a`),
and my Part A (`r3-observation-results-analytics.md`). Static reading only; nothing executed.
Library facts carry their Part A labels (*Interface-checked* unless a skill probe is named).
Benefits are *Proposed*. Ledger stage references are written `A1.17` (W-A journey 1, stage 17)
and `B1b.13` (W-B journey 1b, stage 13). Interpretation is marked **[I]**.

---

## 0. Settled premise from Part A: are canonical IPC bytes a hashing input for results?

Resolved statically. Answer: **yes for integrity and acknowledgment, no for scientific identity.**

- **Per-block integrity.** `pse_operations::canonical_execution::result_payload_digest`
  (`crates/pse-operations/src/canonical_execution.rs:2871`) hashes the exact stored IPC bytes
  under `pse.execution.result-payload.v1`. Its doc (l.2874-2876) says: "transport integrity, not
  Arrow, registry or scientific admission". `canonical_results.rs:567` refuses a block whose
  `payload_digest` disagrees.
- **Chain up to the seal.** Batch digests fold into a per-set `FramedHasher`
  (`Frame::CanonicalPayloadV1`, `"pse.execution.batch-selection.v1"`, l.2596). Set descriptors fold
  into the manifest digest that seal compares (§20.2). An "identical retry can share; conflicting
  content refuses" (§20.2), so exact-retry idempotency depends on the writer producing
  deterministic bytes.
- **Physical source rows.** Here the bytes *are* identity. `durable.rs::put_physical_rows`
  (around l.3160-3172) frames `"physical.rows.v1"` plus the relation and the IPC payload bytes,
  and keys the problem `physical:rows:<hash>`. Deduplication therefore depends on the encoding.
- **Logical content identity.** `pse.canon.v2` (blueprint §5.3) is defined over a normalized
  two-stream IPC preimage. The section itself says that "no current production publication or
  reuse path consumes relation logical hashes".
- **Consequence for the allocations.** A Parquet payload (S1) would change the integrity frames,
  not any scientific identity. The manifest, seal and retry-equality chain would hash Parquet
  file bytes instead. Parquet output is deterministic for a fixed writer version and properties,
  but the footer carries a `created_by` version string, so the bytes change across parquet
  upgrades. That is acceptable for integrity and unacceptable for a cross-version dedup key.
  The physical-rows key, which is the encoding-dependent one, would need a versioned frame or a
  move to `pse.canon.v2` logical hashes.

---

## 1. Obligations in scope, grouped by kind

In scope are the obligations that produce, carry, store, select, reopen, validate or observe
results and records. Durable-lifecycle obligations (leases, fences, claims, heartbeats) belong to
R1. They appear here only where results depend on them.

### 1.1 Result encoding and export (A1.17, B1b.13, B3.7, B4a.4, B4b.3, A4.1)

The ledgers record this:

- every solve encodes every advertised relation (about 20 for modeling, `results.rs::table_ids`)
  into bounded IPC blocks with row and output indexes, plus seeds and the manifest;
- encoding runs once for export and again for every `.table()` read, because `EncodingState`
  memoizes only failures (A4.1);
- a long trajectory is appended only at `finish` (B4a.4);
- horizon runs append a prefix per step (B4b.3).

| Obligation | Library ownership | Necessity | Mapping consequence |
|---|---|---|---|
| Columnar encoding of each relation | **Wholesale** by Arrow builders, which is already the case. The IPC `StreamWriter` and Parquet `ArrowWriter` both own serialization. | Functional target: results must persist exactly | none |
| Bounding the payload per transport unit (512 KiB blocks, 128 KiB index prefix) | **Not** library-owned. It exists because payloads travel inside SurrealDB RPCs (§20.1; the I3/I4 choices). Parquet row groups and pages bound units natively. | **Imposed by a rule**, not by the workload: §20.1 "independent uncompressed IPC blocks of at most 512 KiB", which follows from ADR-0164 putting payloads in the store | Payloads in files (S1) remove block chunking and the RPC per batch. In-store payloads (S3) keep it. |
| Exporting all advertised relations on every solve | Not a library question | Rule: §19 "Ordinary execution durably retains its actual scientific outcome", narrowed by ADR-0180 demand-selected detail (B1b.13). Interactive value edits (A2.7) also export everything. **[I]** For an edit → re-solve loop, the target needs the result available, not necessarily persisted in full each time. | Out of R3's library scope. It decides how much S1/S3 write volume matters. |
| Re-encoding on each Python read (A4.1) | **Wholesale** by Arrow: the export path already holds the encoded `RecordBatch`es. Retaining them and handing `Arc` clones through the C stream (`FFI_ArrowArrayStream`, pyo3-arrow) is zero-copy by construction. | Not necessary: the completion is immutable (A1.16) | Removes repeated encoding and validation (VAL) per read. Loses nothing, except that retained batches must be charged to the pool. `ArrowMemoryPool` exists for that. |
| Mid-run durability of trajectories (B4a) | **Partly**: both formats stream. An IPC stream or a Parquet file can be written incrementally (row group by row group), but neither is readable as a whole until finished: IPC needs its end marker, Parquet its footer. | Workload premise: "Interruption before publication" and "Long trajectories" | S1 can flush closed row-group files per window. A prefix becomes a set of complete small files, which is the horizon-prefix pattern (B4b.3) already. Peak memory then stops growing with the horizon **[I]**. |

### 1.2 Result storage, indexing and selective reopen (A1.17, A4.2-A4.5)

The ledgers record:

- reads are store mutations: `read_results` takes a protected read with a 60 s lifetime (A4.2);
- each block costs one RPC, a decode and a VAL re-run (A4.3);
- output and latest selectors check `canonical_result_rows` index rows against the IPC rows (A4.4).

| Obligation | Library ownership | Necessity | Mapping consequence |
|---|---|---|---|
| Locating rows and outputs (row index, block coordinate min/max) | **Wholesale** by Parquet: column statistics at row-group and page level, page index, `RowSelection`, `RowFilter`, `StatisticsConverter`. DataFusion `ListingTable`/`ParquetSource` adds pruning from predicates (skill brief `parquet.selection`, fixture-*Tested*). | Functional target: selective inspection of large results | S1 removes the hand-built `canonical_result_rows` and `canonical_result_block_outputs` index tables and their per-cell verification (A4.4). It loses the ability to ask SurrealQL to index into payload rows. **[I]** No consumer outside these readers was found in the ledgers. |
| Exact verification that an index entry agrees with its payload (A4.4 `verify_scalar_row`) | **By construction** in S1: statistics are derived from the same pages by the writer, so no second copy exists to verify. Under S3 it stays the project's. | Needed only because the index is a second representation (§20.1 "verified against authoritative values") | Removed by S1 |
| Read protection against concurrent reclaim | **Not** library-owned. Files have no MVCC. A filesystem keeps an open file readable on Linux after unlink, but a ranged `object_store` reader re-opens the file per range. | Needed whenever retention and readers run concurrently (W-B (a): workers, clients, reclaim) | Remains the project's in every allocation. S1 would protect a manifest, not blocks, which means one protected read per result set rather than per page **[I]**. |
| Decode admission (bounds before allocation) | **Partly**. arrow-ipc 60.0 "bound[s] IPC message allocations by the bytes actually read". Parquet bounds through metadata, and 60.0 rejects oversized Thrift lists. At 59.3 the project-owned preflight remains. | Needed for untrusted bytes. **[I]** Bytes this process wrote and digest-checked are not untrusted in the sense of an external boundary. | Moving to 60.0 narrows the bespoke preflight. That move is blocked by the one-type-universe rule until DataFusion moves to Arrow 60 (Part A §1.8). |
| Re-validation (VAL) of decoded blocks (A4.3) | Arrow validates structure on decode by default. Registry domain validation is project-owned (VAL through `PhysicalExpr`). | Write-time VAL plus a digest match already establish the facts. The re-run comes from the rule "Raw ingress (IPC, FFI and external batches) still admits" (prior F "Preserve") and `FieldCheckedBatch::admit_owned`. | **[I]** A digest-verified block can carry its write-time admission, which removes the second VAL. An unsafe skip of Arrow structural validation is documented only for trusted same-process input (`reader.rs:1066-1076`). Store-returned bytes may reasonably stay structurally validated. |

### 1.3 Observation records: progress, metrics, strategy events, lineage (A1.14, B1b.12, B4a.3, A4.6, A4.10)

The ledgers record:

- the progress `Tap` feeds a `Streamer` (64 events per batch, 200 ms flush) that writes store RPCs
  during the run;
- `Progress::new(256)` bounds dynamics events;
- `resource_usage` reads DataFusion pool metrics;
- `PreparationCounts` is in-process, task-local evidence (B1a.9).

| Obligation | Library ownership | Necessity | Mapping consequence |
|---|---|---|---|
| Bounded, non-interfering capture of in-solve events | **Partly** by tracing: no-cost-when-disabled callsites, per-layer filtering, span timing. Delivery and boundedness are not library-guaranteed. A project `Layer` and channel must own drop accounting (tracing-appender would be lossy, and it is absent anyway). | Target: diagnostics and measurement are product capability. The tier rule (principal §3.3): never back-pressure a solve | Under S2, ad hoc `Instant`/`elapsed`/string-metric timing goes away. Completeness is lost unless the folding layer is the only path, with drop counts recorded. |
| Phase timing across admission, preparation, routing, rung, native, assessment and publication (not recorded today, per A2) | **Wholesale for capture** by tracing span trees, including async through `Instrument`. Native threads need explicit span entry. datafusion-tracing already wraps the engine path (`pse-engine/src/session/assurance.rs`). | Target: measurement and the history layer | S2 supplies timing without bespoke clocks. The ledgers show **no production subscriber**, so today the spans reach nothing. |
| Durable progress during long runs (A1.14, B1b.12) | **Not** library-owned in S2 (tracing is not durable). S1: an append-only progress file per attempt is possible but unreadable until closed. A rolling closed-file pattern works. | Product: `RunHandle.progress()` and durable progress after process loss (§19, `progress_documents.rs` coverage) | Remains the project's. The store stream is the right carrier for live cross-process visibility. S1 suits only closed history. |
| Typed per-attempt record (solver statistics, rung ladder, timing, identities) | **Wholesale for representation** by Arrow: struct, list and union columns, and extension types. Parquet and DataFusion store and query it (C3 aggregates). | Target: three-layer selection needs typed, comparable history (maintainer requirement 3) | One schema serves telemetry, measurement, history and diagnosis (Part A §4.3). The ledgers add no counter-evidence. |
| Cross-point and cross-run aggregation (none today; A2 §3) | **Wholesale** by DataFusion: `percentile_cont`, `median`, `stddev`, `corr`, windows, joins (registry `df55-sql-parquet-nested-string`). statrs 0.19 adds `OnlineMoments` in-process. | Target: studies, the history layer and convergence-effort regression | Removes the need for bespoke summary code in `StudyReport` and similar places. Nothing is lost. |
| Process resource observation (A4.10) | **Wholesale** by the DataFusion pool wrappers (`TrackConsumersPool`, `PeakRecordingPool`), as today | Product capability | Unchanged. Per-attempt attribution stays the project's (A2 §8). |

### 1.4 Validation of authored and scientific relations (VAL in A1.4-A1.7, A3.3, A4.1, A4.3, A4.5, A4.8)

W-A (b) records three things. pse-engine supplies resources, VAL through `PhysicalExpr`, and
physical admission, which is the only query execution. VAL runs at least eight times on one
journey (load, re-validate, build-to-validate-then-drop, delta, export, re-read, record rows,
source export). Batch admission "does not certify keys or foreign keys".

| Obligation | Library ownership | Necessity | Mapping consequence |
|---|---|---|---|
| Per-row declared domains (ranges, enums, nullability, nested) | **Partly** by DataFusion expressions: `create_physical_expr` compiled once and reused (skill brief `df.expressions`). Arrow kernels alone could also evaluate them. Neither needs a `SessionState` per owner. | Needed once per admitted batch (H19/H20 "establish once") | **[I]** Holding one compiled `PhysicalExpr` set per registry-contract identity removes repeated preparation. Carrying admission through the immutable product, from write to export to read, removes the six of the eight VAL sites that re-prove an unchanged premise (A1.6, A1.7, A3.3, A4.1, A4.3, A4.5). The ledger rows themselves describe those six as rechecks or build-only-to-validate. |
| Key and reference obligations in physical admission (A1.4, `BoundObligation{plan: LogicalPlan}` "rows violating") | **Wholesale-capable** by DataFusion: anti-joins, `GROUP BY ... HAVING count>1`, sort. Today this is the only query execution. | Needed once per physical document set | Library-owned already. The question is proportion, not ownership: about 13 scans through the full preparation pipeline (prior F §2a). |
| Build-to-validate-then-drop (A1.7, A3.3) | Not a library obligation | **Imposed by a rule**: generated builders are the only admission route, so validation requires materialization | **[I]** Validating the parsed rows once at load (A1.5) and carrying admission removes these steps. |

### 1.5 Seeds and starts as stored results (B1b.8, B1b.13, A2.7)

The ledgers record that durable points re-read and decode seeds (`latest_seed`, `artifact_seed`,
an Arrow payload decode), and that Python value edits get no warm start unless the caller uses
`with_stored_start` or `with_primal_start`.

| Obligation | Library ownership | Necessity | Mapping consequence |
|---|---|---|---|
| Seed payload encoding and decoding | Wholesale by Arrow IPC (as today) | Product: durable continuation and seed reuse (ADR-0173) | The library adds nothing. The seed volume is small; the cost is the RPC chain plus eligibility (R1). |
| In-process start reuse across value edits | Not a library question. Retaining the previous completion's batches in memory (1.1 row 4) would let an in-process session offer them as a start with no store read. | Target: interactive edit → re-solve | **[I]** This sits with W-A (a): session state already holds `Arc<ModelingResult>`. |

---

## 2. Refined allocations

**S1 — Parquet + DataFusion as the results and observation plane.**

Supported by the ledgers:
- full-relation export on every solve (A1.17, A2.7, B1b.13), which is file-shaped bulk;
- index rows verified against the payload (A4.4), which is a second representation S1 eliminates;
- per-block RPC and decode on read (A4.2-A4.3);
- no relational route over results (W-A (b): "no DataFrame or listing tables over results");
- no cross-point aggregation (A2);
- trajectories appended only at finish (B4a.4), where row-group files bound memory.

Undermined by the ledgers:
- Durable progress needs live cross-process visibility (A1.14, B1b.12; the waiter poll in B1b.6),
  which closed files cannot provide.
- Read protection against reclaim stays (W-B (a) concurrency).
- Payload and manifest visibility would no longer be atomic within one store transaction (§20.2).
- The physical-rows key is encoding-dependent (§0).

Refined shape: the store keeps authority, manifests, progress streams, seeds and small typed
records. Files hold bulk relations and closed observation history. Integrity frames are
re-versioned for Parquet bytes. Physical-rows identity moves to `pse.canon.v2` or keeps IPC
bytes.

**S2 — tracing as the observation carrier.**

Supported by the ledgers:
- phase timing is absent while timing sources are scattered (A2 summary);
- `PreparationCounts` is task-local and in-process only (B1a.9);
- datafusion-tracing is already wired on the engine path;
- no subscriber exists, so instrumentation reaches nothing today.

Undermined by the ledgers:
- Native work runs on fresh `pse-math` OS threads and FFI callbacks (A1.15, B1a.8, B3.4b). Span
  context does not follow automatically: the code must enter it in each thread and carry span
  handles into callback state.
- Durable progress cannot come from tracing.

Refined shape: tracing carries phase spans and timing. The typed solver statistics stay in
`SolveReport` (decision-readable, in memory). One folding `Layer` joins both into the per-attempt
Arrow record at attempt end, and that record is written through S1 or S3.

**S3 — store authority plus a DataFusion provider over the existing blocks.**

Supported by the ledgers:
- the store remains required by current contracts on every interactive step (W-A (a) items 1-7);
- the relational gap (W-A (b)).

Undermined by the ledgers:
- It keeps the RPC per block, read protections as mutations, index verification and chunking,
  so it removes none of the A4.2-A4.4 costs.
- It adds a provider that must be differentially tested.

Refined view **[I]**: S3 serves inspection breadth, not efficiency. It is the minimal option if the
§20.1 rule stays.

**Common to all three.** "Retain encoded batches per completion" (1.1) and "carry admission
through immutable products" (1.4) are library-native in Arrow and DataFusion. They are
independent of where payloads live.

---

## 3. Repeated or reconstructed work removed by construction

| Ledger case | Removed by | Why "by construction" |
|---|---|---|
| `.table()` re-encodes after export (A4.1; W-A (c) "Result relations") | Arrow `RecordBatch` immutability plus `Arc` sharing; C stream export of retained batches | The batches already exist. Sharing an immutable buffer has no invalidation question. |
| VAL re-run on decoded stored blocks (A4.3) | Payload digest equality (already computed) carrying the write-time admission | The digest proves the bytes are the ones admitted. No new mechanism is needed beyond keeping the association. |
| Index rows verified against IPC rows (A4.4) | Parquet statistics and page index written from the same pages (S1) | There is no second representation to reconcile. |
| Per-block RPC paging and the chunking of bulk results (A1.17, A4.2) | Parquet row groups and pages with ranged reads (S1) | The transport unit becomes the file format's unit, not a store message bound. |
| Bespoke cross-point and cross-run summaries (absent today; A2 §3) | DataFusion aggregates and windows over observation records | Expressed as queries, not code. |
| Repeated `PhysicalExpr` preparation per validation owner (W-A (b) VAL; prior F-01) | One compiled `PhysicalExpr` set per registry-contract identity (DataFusion physical expressions are reusable across batches) | Preparation is a pure function of the contract identity. |
| Ad hoc timing per adapter (A2 §1) | tracing span open and close times | The timing is a property of the span. |

Not removable by R3's libraries: the seed decode per durable point (B1b.8), which is small and
dominated by the R1 RPC chain; read protections (any allocation); export of all relations per
edit (a policy question).

---

## 4. Open premises that would change this mapping

1. **Whether §20.1/ADR-0164's "results in the store as bounded IPC blocks" stays a rule.** If it
   stays, S1 is out and S3 plus the two common changes are the ceiling. Settled by: the
   coordinator's rule-impact decision. The evidence is the A4.2-A4.4 costs against the atomicity
   that the store transaction provides (§20.2).
2. **Whether any consumer needs SurrealQL over result rows** (for example `latest_results`
   predicates or analysis lineage edges into rows). If one does, the S1 index removal is partial.
   Partly settled by a search for `canonical_result_rows|canonical_result_block_outputs` over
   `crates`, `scripts` and `python`, excluding generated paths. The hits are writers
   (`result_projection.rs`, `canonical_execution.rs`), the reader and selectors
   (`canonical_results.rs`, `connected_results.rs`), retention codegen (`surreal_retention.rs`,
   `surreal_execution.rs`), the schema catalog (`pse-schema/src/catalog/substrate.rs`) and one
   fixture script. I found no analysis or lineage consumer, but a lexical search does not
   establish absence.
3. **Whether "raw ingress always admits" covers bytes this deployment wrote and digest-verified.**
   This decides whether removing A4.3's VAL re-run is a rule change. Settled by the owner text of
   the `FieldCheckedBatch` contract (`pse-relations/src/columnar.rs`) and the blueprint trust
   boundary.
4. **Whether physical-rows deduplication must survive encoder changes.** If it must, the key moves
   to `pse.canon.v2` whatever the payload format. Settled by §20.3/§20.4 reuse expectations for
   physical inputs.
5. **Whether native threads can carry tracing context cheaply.** This decides whether S2 covers
   native phases or only Rust-side phases. Settled by reading the `pse-math` job spawn
   (`rt/math/jobs.rs` ~l.1077) and the callback state, and whether a span handle can be moved in
   (static reading suffices).
6. **Whether durable progress must be readable mid-run by other processes.** If it must, the
   store stream stays and S1 covers only closed history. Settled by the §19 progress contract and
   the `RunHandle.progress` consumers.
7. **The Arrow/Parquet 60 adoption timing.** It requires a DataFusion release on Arrow 60 and a
   family-major ADR. It changes decode-bound and NaN-statistics facts (Part A §1.8), not the
   allocations.

Files written: this file only.
