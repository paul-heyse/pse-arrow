# Publication, persistence and results serving: bounded supporting assessment

Inquiry **E** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`E-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

This is a bounded supporting assessment for the efficiency-alignment review. The baseline is `main` at b01a45d71a5f5ab29991ad193679f5588ac3662f. The standard applied is Core 3.4 with H1–H28, Process Simulator 1.5 and the pse-arrow binding. Labels E-nn are local; the principal reviewer owns the combined finding IDs.

## 1. Scope examined and coverage limits

**I read these in full or at the decisive symbols:**
- `crates/pse-runtime/src/workflow/durable.rs`: `DurableAttempt::{register,finish,store_tables,terminate}`, `Operations::{record,settle,recover,put_physical_rows}`, `Streamer`, `write_chunks`, `read_chunks`, `store_seeds`, `Heartbeat`.
- `result_projection.rs`: `scalar_cells_at`, `store_result_table`, `store_result_chunk`, `metadata_prefix`, `store_trajectory`, `selected_scalar_cell`.
- `result_blocks.rs`: encode, visit and decode.
- `results.rs`: `table_ids`, `cursor_by_id`.
- `result_export.rs`: `SelectedCollection::push_ref` and `Rows`.
- `connected_results.rs`: `CanonicalResultReader::next_checked`, `Runtime::{results,output_results,result_manifest}`.
- `retention.rs`.
- `pse-operations/src/canonical.rs`: `ProblemTurns`, `CanonicalStore::{protected_query,ensure_writes,protect,acquire_protection,release_protection}`, the free `protected_query`, `PROTECTED_BEGIN`.
- `canonical_execution.rs`: `protected_execution_query`, `append_execution_batch`, `close_result_ingestion`, `closed_response`, `reconcile_closed_attempt`, `AttemptFence`, the key and digest helpers.
- `canonical_results.rs`: `read_results`, `result_block_page`, `result_output_page`, `result_block`, `result_payload`, `validate_result_block`, `validate_result_cell`, `ResultRead::renew`.
- `canonical_result_retention.rs`, and the `RECLAIM` text in `canonical_retention.rs`.
- The generated `surreal.surql` schema and indexes, and the generators `pse-codegen/src/codegen/surreal_execution.rs` (append, close, writer, touch, seal, renew, `study_cancelled`) and `surreal_retention.rs` (`forget_run`, `collect_run`, `forget_analysis`). I also read the study functions `cancel`, `observe` and `conclude`.
- `scripts/surreal_server.py`: `ensure_primary`, `_ensure_primary`, `recorded_primary`, `checked_primary`, `primary_ready`, `verify_generation`, `file_digest`.
- The Rust caller `worker.rs::Runtime::ensure_managed_primary`, `native_deployment.rs::NativeExecutionSelection`, and `pse-buildinfo/identity.rs::FileObservation`.
- `pse-py`: `inspection/stream.rs::TableStream` and `workflow.rs::NativeStoredResult`.
- Python: `_runs.py`, the `_workflow.py` results APIs and `_modeling.py` `result(index)`.
- Blueprint §19.2 ("Retained scientific rows") and §20.4/§20.6.
- The vendored SurrealDB 3.3.0 `SELECT … FOR UPDATE` semantics (`surrealdb-core-3.3.0/src/kvs/tests/select_for_update_test.rs`; skill probes SB091/SB092).

**Not read, or only skimmed:**
- The `canonical_staging`/`canonical_selection` source paths, which belong to A.
- Study scheduling (`worker.rs` loop, `canonical_studies.rs`), which belongs to D.
- Checked take and `admit_owned` internals, which belong to F.
- Most of the 7,291-line supervisor beyond ensure-primary.
- The test oracles in `scripts/tests` (state F06).
- Fitting and trajectory encoders beyond `store_trajectory`.

No probe was run. All counts below are source-derived, not measured.

## 2. Complete operations examined

**(a) Durable completion of one modeling run.** This is the default runtime: `RunResult::finished` → `DurableAttempt::finish` → `sender.send_replace`. The waiter sees nothing until every step below completes.

1. Progress drain: `Streamer` sends one `append_result_batch` per ≤64 events per 200 ms window.
2. `store_tables` runs one relation cursor per `RunResult::table_ids()` entry (22 for modeling). Each yields ≥1 block through `append_result_block[_cells|_indexes]` → `append_execution_batch`. Each append costs:
   - one `ensure_writes` config.json read;
   - two RPCs: `protected_execution_query` first looks up the run's problem, then calls `fn::pse_execution_v1::append` under the problem's `ProblemTurns` write gate.
3. `store_seeds`: one `canonical_run` read, then per step a JSON seed document of u64 bits, chunk appends and `register_result_seed`.
4. `terminate`:
   - `canonical_attempt` and `canonical_run` reads;
   - completion `write_chunks`;
   - `close_result_ingestion`: two RPCs plus an unconditional `closure_operation` control query.
5. `Operations::settle`:
   - `reconcile_closed_attempt`: `canonical_attempt`, `canonical_result_manifest`, a sets query, ≥1 batch-page query per result set (about 25 sets), then the manifest function (two RPCs);
   - `seal_attempt` (two RPCs).
6. `Operations::record` reopens the sealed attempt:
   - `read_results`: four RPCs including `protect` and a guard-bumping protected transaction;
   - `read_chunks` for the completion: renew plus payload per chunk;
   - the seed page loop: renew plus page per page;
   - a release when the read is dropped.

Live state: the full in-memory `RunResult`, plus a second deserialized copy of the completion inside `DurableRecord`.

Rough sequential tally, with *n* = rows of `solve_variables` and *m* = rows of `solve_constraints`: about 60–80 fixed RPCs plus 2·(n/≈26 + m/≈33) for the scalar relations (§E-01). All of them run after the solve and before `wait()` returns.

**(b) Study point N+1 publication.** The publication is identical to (a). The point's run shares the study's `problem` (generated `fn::pse_study_v1::append` checks `$run.problem = $s.problem`). In-process lanes therefore share one `ProblemTurns` gate, one `retention:<problem>` guard and one `study:<id>` guard.

**(c) Reopening a subset.**
- `Runtime::results`: `read_results` (four RPCs), then per page of 64 blocks a renew (Mutation) plus `result_block_page` (Read).
- Per block, `result_block` is one fused RPC. It is followed by:
  - `validate_result_block`, which computes the BLAKE3 digest twice;
  - `decode_result_block`, which preflights the IPC and compares the full schema;
  - `FieldCheckedBatch::admit_owned`;
  - `checked_export`;
  - one FFI batch to pyarrow.
- `output_results` replaces the page query with `result_output_page` over scalar cells. It then fetches and decodes each referenced block whole, re-derives every selected cell (`selected_scalar_cell`) and runs `take_reserved`.

**(d) Inspecting stored results.** A `StoredResult.usable`, `.completion` or `.diagnostics()` access calls `Runtime::stored_completion` → `Operations::record` (step 6 of (a)) on every access (§E-05).

**(e) Reclamation.**
- `reclaim_run_results` makes one `forget_run` call and one `collect_run` page per call.
- A `collect_run` page deletes at most 64 cells and 64 outputs from one batch.
- Analysis retirement pages 64 edges, nodes, inputs and roots per call.

**(f) Starting a durable study (`pkg.study(defn)`).**
- `ensure_managed_primary` runs `NativeExecutionSelection::from_store`, which SHA-256-hashes the supervisor script and the worker executable.
- It then spawns `surreal_server.py ensure-primary` (≈7.3k-line Python supervisor) and waits for its JSON reply.
- Finally it calls `require_current`, which hashes the same files again (§E-06).

## 3. Candidate findings

### E-01: A derived per-field scalar-cell index sets the physical publication unit and multiplies writes for a single-row lookup

**Observed cause.**

`result_projection.rs::scalar_cells_at` emits one `canonical_result_cells` record per declared scalar field per row:
- 10 per `solve_variables` row;
- 8 per `solve_constraints` row;
- 4 per `fit_observations` row;
- 2 per `fit_parameters` row.

Each record carries a 64-hex batch key, a formatted `key`/`output`/`partition`, exact bits and a projection.

`store_result_chunk` sizes each IPC block by `metadata_prefix(extents)` against `EXECUTION_METADATA_BYTES` (`generated::surreal::RESULT_INDEX_BYTES` = 128 KiB). The cell metadata, not the 512 KiB payload bound, therefore decides the rows per block:
- My source estimate is about 480 B per encoded cell, giving about 26 rows per `solve_variables` block and about 33 per `solve_constraints` block.
- The unit test `result_metadata_prefix_uses_encoded_extent_and_keeps_exact_cells` asserts only `6 < rows < 100`.

`surreal.surql` defines five indexes on `canonical_result_cells`: `canonical_result_cells_key`, `result_cell_rows`, `result_cell_numeric`, `result_cell_selection` and `result_cell_batch`. Each cell is also validated, framed and hashed per cell in `append_execution_batch`, and encoded to a SurrealDB `Value` twice: once by `result_cell_metadata_extent` and once in the append.

`store_trajectory` has the same shape for dense outputs. It cuts at least one block per `symbol_id` group, although `append_result_block_indexes` and `canonical_result_block_outputs` already accept several output descriptors per block.

**The only consumer** is `Runtime::output_results` (`connected_results.rs`, Python `Runtime.output_results`). No Python or other Rust production caller was found (search: `crates/`, `python/pse`, `tests/`). It selects exactly one `(relation, owner, field, partition)`. The `solve_variables` primary key is `(run_id, step, symbol_id)` (`docs/generated/relations/runtime.md`), so each scalar output names at most one row. The numeric predicate and `result_cell_numeric` therefore filter a set of one value. After the index lookup, the reader still fetches the whole block, decodes it and re-derives each selected cell, so the authority stays with the IPC row.

**Consequences.**

*Write side*, under edit → re-solve, every study point and larger structures: each durable solve with *n* variables writes 10n cell records and about 50n index entries, plus 8m and 40m for constraints. It also makes about 2(n/26 + m/33) sequential append RPCs, each a server transaction with guard writes (§E-02) and permanent receipts (§E-07). A 10k-variable, 8k-constraint result means about 164k cell records and about 1,300 append RPCs.

*Read side*: a full `results(...)` read of such a relation costs one RPC, one schema decode and comparison, one registry admission and one FFI batch per ~26 rows.

*Reclamation*: `collect_run` deletes 64 cells per RPC, so the same run needs about 2,600 pages.

*Dynamics*: thousands of outputs with short sample series produce thousands of small blocks.

The heuristics involved are:
- H2: the semantic field set dictates the transaction and frame unit.
- H6/H27: index write burden without matching demand.
- H8: item-level index records.
- H16: repeated schema per tiny block.

This is AP-07/G9 under PSE-S03/S07.

**Simpler alternative.**
- Store result relations in primary-key order and select blocks by key range. Canonical storage order already differs from public order for trajectories.
- Record per-block key bounds in the existing block descriptors (`coordinate_min`/`coordinate_max`, `result_block_range`), or at most one locator record per row rather than one per field.
- Evaluate the scalar predicate with Arrow `filter`/`take` on the decoded candidate block, which the reader already decodes.
- Let blocks fill to the 512 KiB payload bound. That is about 5k `solve_variables` rows, so most relations need one append.
- For trajectories, group many symbols per block with one `block_outputs` row each; the schema already supports this.
- If a predicate across outputs is ever needed (for example, all variables with `upper_violation > tol`), DataFusion or Arrow kernels over the decoded set fit better than an entity-attribute-value index.

Library support: Arrow `filter`/`take` (arrow-select 59.3); SurrealDB `INSERT … $rows` already bulk-inserts.

**Preservation constraints.**
- Exact IEEE bits, signed zero, missing versus finite, and diagnostic bits remain in the original rows.
- Readers verify selection against decoded rows.
- Selection never crosses attempts.
- The 512 KiB block bound and preflight remain.
- Explicit refusal remains for unsupported fields.

**Uncertainty.** Actual rows per block are unmeasured (probe P1). A future consumer could need cross-output numeric predicates; that does not justify the current per-field index for single-row lookups.

**Evidence.** Interface-checked: source-traced writer, reader, schema and consumer search.

**Verification.** Exercise the same output reads (owner/field/partition, missing, finite bounds, signed zero) against key-ordered large blocks. Count appends, cells and blocks per run before and after.

### E-02: Publication and reads register conflicts by writing problem-wide and study-wide guards, which serializes independent points

**Observed cause.**

On every `append`, `renew`, `close`, `seal` and `seed`:
- the generated `fn::pse_execution_v1::writer` calls `available` (touches `execution-run:<run>`), touches it again, and calls `study_cancelled`;
- `study_cancelled` writes `canonical_guards:study:<study>` through `touch` (`UPDATE … generation += 1`).

`fn::pse_study_v1::cancel` touches the same guard. Every point writer of a study therefore writes one shared record.

The same pattern applies to `retention:<problem>`:
- `acquire_protection`, `release_protection` and `ResultRead::renew` UPSERT-bump it;
- the `read_results` protected transaction bumps it;
- `seal` touches it;
- every `PROTECTED_BEGIN` read selects it `FOR UPDATE`.

In SurrealDB 3.3, `FOR UPDATE` registers the key for commit-time conflict with any committed write to it (`select_for_update_test.rs`; skill SB091). Every bump therefore invalidates every concurrent protected read or mutation of that problem across processes.

In-process, `ProblemTurns::run` serializes all `QueryEffect::Mutation` work per problem behind a `tokio::sync::RwLock`. That includes every append, renew, claim, protect, release and `read_results`. The conflict retry loop with `conflict_backoff`, up to 32 attempts (`canonical.rs::protected_query`), runs inside that gate. Readers also renew at fine grain:
- `read_chunks` renews per 512 KiB chunk;
- `Operations::record` renews per seed page;
- `CanonicalResultReader` renews per page.

`docs/plans/28e…` records a 1,000-point managed sweep that failed on a definite result-ingestion conflict. The correction placed ingestion and study mutations under the same per-problem pacing, which is serialization; it did not narrow the conflict domain.

**Consequence.** Sixteen concurrent cases of one study (PSE-S07, or the workload premise) funnel all of their roughly 60–1,000+ publication transactions per point through one in-process gate. Unrelated preparation reads of the same problem also wait behind them. Across processes (managed primary, Python reader, recovery), each guard bump forces whole-transaction retries of unrelated reads and writes.

The heuristics involved are:
- H22: shared ownership forces independent work to wait.
- H23: the exclusion domain is broader than the effects it protects.
- H24: retries run while holding the gate.

This is AP-07/G9 and DP-20. It overlaps D's lead parallel F01. The E-specific additions are the study-guard write on every append and renew, and the per-chunk and per-page reader renewals.

**Simpler alternative.** Register the conflict on the rare side:
- Frequent writers `SELECT` the study header or cancel record `FOR UPDATE` without writing it.
- Cancellation writes it.
- Two appends then no longer conflict; an append conflicts only with a concurrent cancel.

For retention, new acquisitions (phantoms) and reclamation still need a shared registration. Renewal of an existing live protection and result-read admission could instead conflict through the protection row that reclamation must lock. Narrower domains are also possible, for example per revision interval or per run for result reads.

Size renewals by remaining lifetime, not per chunk or page. Release the local gate between retries. All of this uses native SurrealDB capability (`FOR UPDATE` on record ids; SB092 notes it does not cover predicates) and needs no new machinery.

**Preservation constraints.**
- Cancellation must still win against late appends.
- Reclamation must not race new protections.
- Named-absence and set premises (US04) keep coverage.
- Original request clocks and uncertain-response settlement remain.

**Uncertainty.** Contention magnitude and its attribution are unmeasured. The exact reclamation premise that renewal must register needs a short design check.

**Evidence.** Interface-checked from generated SurrealQL, the Rust pacing and upstream `FOR UPDATE` semantics. The failure itself is recorded in the 28e evidence, not re-run.

**Verification.** Run sixteen same-study points with concurrent cancel, reclaim and reader activity. Count conflicts and retries per operation; cancellation and reclamation must remain safe.

### E-03: Durable finish re-establishes facts the writer already holds, all on the edit → re-solve critical path

**Observed cause.** Four separate mechanisms:
- `AttemptFence` (`canonical_execution.rs`) carries `run`, `attempt` and `generation` but not `problem`. `protected_execution_query` (17 call sites) therefore spends one RPC per mutation re-reading the immutable `canonical_runs.problem`, used only to select the local pacing gate.
- `reconcile_closed_attempt` pages back every batch's `(key, digest, row_count)` per result set and rebuilds the batch-selection digest. The writer already received each of those values in the compact append acknowledgment, and the server maintains `canonical_result_sets.{next_ordinal,row_count}` atomically per append.
- `Operations::settle` ends with `self.record(..)`. That call reprotects the attempt, reruns the manifest transaction, re-reads the completion chunks the same process just wrote, and pages the seeds it just registered, all to build `DurableRecord`.
- `closed_response` issues `closure_operation` even when the close succeeded.

**Consequence.** For an interactive value edit on the default durable runtime, the user's `wait()` returns only after this re-proof: one extra RPC per append, about 25 reconcile pages, about 10 reopen RPCs, and two guard bumps (§E-02). The whole route is serialized with other work on the problem. A second deserialized completion copy stays live beside the in-memory one (H17).

The heuristics involved are H19 (established properties not carried), H16 (persist and reload without purpose) and H1 (mechanism mistaken for guarantee). This is AP-07 and DP-10.

**Simpler alternative.**
- Carry `problem` in the fence. `DurableAttempt::register` already holds the `canonical_run` row, which includes it.
- Compute descriptors from the writer's acknowledged appends. The server manifest function then checks counts and rows against `canonical_result_sets` in one transaction. Keep the paged read-back for the recovery path (`Operations::recover`), where no writer memory exists.
- Build `DurableRecord` from the seal acknowledgment (the returned attempt row) and the exact completion bytes whose digest the append receipts and seal acknowledged.

A further option, with an explicit contract: return the in-memory scientific result to the waiter once the completion is captured, and expose durability as its own state. One premise must be checked first. Warm starts across runs read stored seeds (`run.rs::with_stored_start` → `latest_seed`), so durability is on that dependency path.

**Preservation constraints.**
- Truthful terminal class.
- Exact manifest and seal fencing.
- Lost-acknowledgment settlement through operation receipts.
- Recovery still reconciles from server state.
- §20.6's "acknowledged stored completion" is preserved semantically; see the rule impacts.

**Uncertainty.** End-to-end latency share is unmeasured.

**Evidence.** Interface-checked.

**Verification.** Inject a lost append or seal acknowledgment and a crash before seal. Records must be byte-identical to the read-back variant. Count RPCs per durable run.

### E-04: Each relation is its own physical transaction, including relations with no rows

**Observed cause.** `store_tables` iterates all 22 advertised relations. For an empty relation, `store_result_chunk` reserves 2 MiB, encodes a schema-only IPC block and appends it. `store_result_table` refuses `ordinal == 0`, so every relation needs at least one block. Each append creates:
- a set, a batch and a block;
- an operation receipt and an operation guard;
- three guard writes;
- later, a reconcile page.

For a typical feasible square solve, several relations are normally empty: `solution_pool`, `incumbents`, `infeasibility_certificates`, and the sensitivity, Hessian and covariance relations unless requested.

**Consequence.** A fixed cost of roughly 22 append transactions per run, each two RPCs, plus permanent residue. For an interactive solve or a 1,000-point study this cost is paid regardless of result size. The heuristics are H2 and H6; this is AP-07.

**Simpler alternative.**
- Declare empty relations in the close transaction: the close lists the advertised relations with zero rows and creates empty sets, and reconciliation verifies them. Readers already take the schema from the registry.
- Append small non-empty relations together in one transaction: one RPC calling `append` per block, or an array form, within the 4 MiB `RESULT_MESSAGE_BYTES` envelope.

**Preservation constraint.** A crash before close must never make a partially exported relation look empty. Emptiness is therefore declared only at close, and the manifest still covers every advertised relation.

**Uncertainty.** The typical count of empty relations is unmeasured (probe P1).

**Evidence.** Interface-checked.

### E-05: A stored-result handle re-reads the whole durable record on every property access

**Observed cause.** `pse-py workflow.rs::NativeStoredResult::{usable,completion,diagnostics}` each call `Runtime::stored_completion` → `Operations::record`. That is:
- `read_results`: four RPCs, including `protect` and a guard bump;
- completion chunks: renew plus payload each;
- the full seed page loop: renew plus page each;
- a release on drop.

`StoredResult` in `_runs.py` is a frozen key-only handle, and Python re-decodes the JSON each time.

**Consequence.** A user scanning a 1,000-point study with `result(i).usable` and `.completion` performs about 2,000 complete reopenings, about 20k RPCs and about 10k retention-guard bumps on the study's problem. Those bumps contend with any concurrent work (§E-02). The seed inventory is read although the caller wants only the completion. The heuristics are H12, H6 and H14.

**Simpler alternative.** The completion is immutable after the seal. Have the handle (or the study report that creates it) hold the acknowledged `StoredCompletion` read once, or have `stored_completion` read only the attempt row and its completion, without the seed inventory. Use one bulk read for a terminal study page.

**Preservation constraint.** Retired or reclaimed runs must still refuse. The answer is exact for the selected attempt.

**Evidence.** Interface-checked.

### E-06: Managed-primary admission re-hashes the ~720 MB worker executable at least six times per durable study start

**Observed cause.** On a warm call (primary already running):
- `worker.rs::ensure_managed_primary` → `NativeExecutionSelection::from_store` captures SHA-256 of `supervisor_script` and `worker_executable` (`pse_buildinfo::FileObservation::capture`): pass 1.
- The spawned `surreal_server.py ensure-primary` → `_ensure_primary` calls `checked_primary`. That call runs `recorded_primary` → `verify_generation`, which hashes every generation file including `bin/pse-worker` (pass 2), then `file_digest(worker_executable)` again (pass 3).
- `primary_ready` calls `checked_primary` again: passes 4 and 5.
- `require_current` re-captures: pass 6.

On a cold start, the post-launch loop calls `primary_ready` every ≤50 ms, so each poll performs two full passes. The worker binary in the qualification state is 719,686,088 bytes (Measured: file size from `ls` of `.generations/*/bin/pse-worker`, 2026-10-11).

**Consequence.** About 4.3 GB of SHA-256 per durable `pkg.study` start before any point runs. That is seconds of CPU on the interactive "run a small study after an edit" path. The cold startup loop is dominated by hashing rather than readiness observation.

The heuristics are H18 (check where uncertainty changes), H19, H26 and H14. This is AP-07. It is the same class as the 2026-10-08 SHA-256 incident; seam with H (PA01/PE05/PE06).

**Simpler alternative.**
- Verify generation bytes once, when `publish_generation` publishes them; generations are immutable, private and 0700.
- On use, check an identity record: (dev, inode, size, mtime, ctime, mode) plus the stored digest. ctime cannot be set by an unprivileged process, so content replacement or modification is still detected.
- At minimum, hash once per `ensure-primary` invocation, and let Rust pass its observation to the supervisor rather than both recomputing.
- BLAKE3 with mmap and rayon (pinned blake3 1.8.7) would be faster for whatever full hashing remains, but removing repetition comes first (H28).

**Preservation constraint.** Refuse replaced or modified executables and changed configuration. Hostile-root tampering is not a stated requirement (DP-19, "without … hostile-operator assumptions").

**Uncertainty.** Actual per-pass time is unmeasured (probe P2).

**Evidence.** Interface-checked call graph; Measured file size.

### E-07: Lifecycle residue grows with every operation and is scanned by reclamation

**Observed cause.**
- Every `append`, `claim`, `close`, `seal`, `seed`, `study-start` and `study-claim` creates a permanent `canonical_execution_operations` record (two indexes).
- `fn::pse_execution_v1::operation` → `touch('execution-operation:'+key)` creates a permanent `canonical_guards` record with a unique `incarnation` (two indexes).
- `collect_run` only blanks the seal operation's result bytes.
- `release_protection` UPSERTs `released = true` tombstones.
- No path deletes `canonical_protections` or `canonical_result_protections`. `scripts/tests/test_surreal_server.py` asserts the absence of `DELETE canonical_protections`.
- `RECLAIM` checks live pins with a `protection_interval (problem, sequence)` range scan that filters `released = false`, so it walks released history.

**Consequence.** Under the growing-retained-history premise:
- receipts and guards grow by about one pair per append (§E-01, §E-04), surviving run reclamation;
- protections grow by about 6–8 per study point (preparation and record reads), forever;
- reclamation and `forget_run` checks scan this history.

The heuristics are H25 and H27. This is AP-07.

The tombstone only needs to outlive a possible late acquisition, and acquisitions are refused after their RPC deadline (`fn::pse_execution_v1::deadline`). Append receipts serve replay settlement, but replay after retirement is refused by `available` regardless.

**Simpler alternative.**
- Collect append receipts and operation guards with the run's batches.
- Collect released or expired protections after the acquisition deadline has passed.
- Keep run-level lifecycle receipts (claim, seal, retirement).

**Uncertainty.** Growth rate is unmeasured (probe P4).

**Evidence.** Interface-checked.

## 4. Opportunities and uncertainties that are not findings

- **Digest repetition (H20/H26).** BLAKE3 runs four times per published block: the runtime descriptor, `append_execution_batch`, and twice in `validate_result_block`/`validate_result_batch`. It runs twice per read block. This is cheap at ≤512 KiB, so it is a cleanup only.
- **`ensure_writes` per mutation.** A synchronous `std::fs::read` plus JSON decode of `state/config.json` on Tokio threads. The cost is small. It is a client-side advisory check, and quiesce safety must come from supervisor drain; that is Plan 35's domain.
- **Seeds.** `store_seeds` writes JSON arrays of u64 bits (about 2–3× the binary size). In-process continuation re-reads the predecessor's seed from the store (`study.rs` `latest_seed`, `with_stored_start`). Seam with D.
- **Read coalescing.** Python receives one batch per stored block. `arrow_select::coalesce::BatchCoalescer` (present in 59.3) could merge them, but E-01's larger blocks remove most of the need.
- **Per-row accounting.** `result_export.rs::SelectedCollection::push_ref` registers a `MemoryConsumer` per row (seam with F). The dense output reader formats a `String` per row.
- **Read-side `admit_owned`.** Revalidation of digest-protected immutable blocks is defensible at the database trust boundary (H18); left to F.
- **Reconciliation clock.** `reconcile_closed_attempt` runs all pages under one 30 s `REQUEST_TIMEOUT`. A very long trajectory (many blocks) could refuse; whether this is predictable needs a workload premise (H11).
- **`latest_results` plan.** `ORDER BY sequence DESC LIMIT 1` with an `IN` predicate may or may not use `run_terminal_selection`. Study point runs share the problem, so a scan would grow with history (probe P3).
- **Repeated `read_results` in analysis.** `result_analysis` calls `self.results(...)`, a fresh `read_results`, per relation although it holds `input: ResultRead`. This is a minor cleanup.
- **Ephemeral path.** It still persists canonical source revisions on edit. That belongs to A.

## 5. Strengths and preservation constraints

- **Compact acknowledgment (PE03 fixed).** The append function returns batch coordinates plus the request hash, and Rust compares without payload. Uncertain completion settles through `settle_operation`/`closure_operation` receipts rather than blind redispatch. Preserve this.
- **Fused read.** `result_block` reads metadata and payload in one protected transaction; selective block discovery filters by row coverage before any payload fetch.
- **Relation-local transport (F09 fixed).** Name discovery uses `table_ids` without encoding, and publication streams relation cursors.
- **Coherent publication.** Fenced ingestion, close, reconcile-from-server-state recovery and a short seal; readers admit only closed manifests and never see late staging (DP-19). Recovery after a crash needs no science rerun.
- **Bounded blocks.** Self-contained IPC blocks with preflight before decoder allocation; compression and shared-dictionary refusal; registry schema authority.
- **Cancellation settlement (state F09 fixed).** An incomplete export after acknowledged cancellation now settles Cancelled with completion `None`.
- **Protection ownership (state F04).** `ProtectionOwner::drop` releases through the captured executor, and the acquisition task owns cleanup after caller abandonment.
- **Analysis retirement (F11 fixed).** Full paged deletion with creation-window settlement.

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| PE02 | Residual. The unit is now the 128 KiB index metadata (about 26 or 33 rows by estimate), not 64 cells; it is still index-dictated (E-01). | `result_projection.rs::metadata_prefix`, `store_result_chunk` |
| PE03 | Resolved (append echo and two-RPC read removed). Minor residue: double digest on read. | generated `append` return; `canonical_results.rs::result_block` |
| 10-09 F09 | Resolved. | `results.rs::table_ids`, `cursor_by_id`; `durable.rs::store_tables` |
| 10-09 F11 | Resolved. | `surreal_retention.rs::forget_analysis`, `settle_analysis` |
| EI01 | Not examined (F). | — |
| EI02 | Resolved as adopted; the index itself is questioned in E-01. | `selected_scalar_cell` |
| EI03 | Confirmed still present as a deliberate limit; no workload premise. | `analyses.rs::result_analysis` (`row_count > 4096` refuses) |
| EF01 | Resolved (work removed). | `authoring_driver/document/owned.rs::BundleOwner` has no `_parent` |
| US01 | Resolved. | single store; `Runtime::results` by run and attempt |
| US02 | Resolved. | `DurableAttempt::finish` stores every relation, seeds and completion automatically |
| US03 | Implemented; not re-tested. | `validate_result_cell`, `canonical_codec` |
| US04 | Implemented. Its realization causes E-02 (moved: correctness is now paid as contention). | `touch`, `writer`, `PROTECTED_BEGIN` |
| State F04 | Resolved (source). | `canonical.rs::ProtectionOwner::drop`, `protect` |
| State F06 | Not examined. | — |
| State F08 | Not examined (Plan 35 SD01 claims resolved). | — |
| State F09 | Resolved (source). | `durable.rs::finish`, `terminate` |
| Parallel F01 (D) | Confirmed still present, with additional E instances (study guard on every append and renew; per-chunk renews). The 28e correction serialized rather than narrowed. | E-02 |
| Plan 35 | Read coordination by effect is implemented. The ensure-primary hashing (E-06) is outside its changes. | `ProblemTurns`; `surreal_server.py` |

## 7. Candidate rule impacts

1. **Blueprint §19.2, "Retained scientific rows" and "Derived scalar cells…".**
   - Change: replace per-field scalar cells with key-ordered blocks plus block key bounds (or one row locator), with predicates evaluated on decoded rows. Allow multi-output dense blocks.
   - Depends: E-01.
   - If the rule is kept: decouple index staging from IPC framing so blocks can be larger. The write and reclaim amplification remains.
2. **Blueprint §20.6, "Durable records and study facts consume the acknowledged stored completion" (`Operations::settle` → `record`).**
   - Change: state that the acknowledged bytes (digest-acknowledged appends plus the seal receipt) satisfy this without read-back; read back only after an uncertain acknowledgment.
   - Depends: E-03.
   - If the rule is kept: only the fence/problem and reconcile parts of E-03 apply.
3. **Blueprint §20.4, "Read acquisition, renewal, admission and reclamation share the retention guard", and §20.6, "Study cancellation participates through indexed point/header association in renewal and append fences".**
   - Change: register on the rare side or on narrower records, and exclude renewal and admission of existing protections where reclamation locks protection rows.
   - Depends: E-02.
   - If the rule is kept: only local pacing granularity and retry-outside-gate improvements apply.
4. **Blueprint §20.4, "preserving lifecycle and lineage receipts", plus the protection-tombstone policy (test assertion in `scripts/tests/test_surreal_server.py`).**
   - Change: collect append receipts and guards and post-deadline tombstones; keep run-level receipts.
   - Depends: E-07.
5. **Plan 35 and RC01 executable-association admission ("not weakened to … digest-shaped strings").**
   - Change: full-byte verification at generation publication; stat identity plus stored digest on use.
   - Depends: E-06.
   - If the rule is kept: hash once per invocation and share the Rust observation.
6. **Every advertised relation needs ≥1 stored block (code: `store_result_table`).**
   - Change: allow close-declared empty sets.
   - Depends: E-04.
   - I did not locate a blueprint sentence for this, so it may be only a code rule.

## 8. Proposed light probes

These are read-only and run against an existing qualification store through the authenticated read-only route (VIEWER client or bounded MCP), under `scripts/pse-env --resource-class light`.

**P1 (E-01, E-04).** Take one completed modeling attempt with ≥1,000 variables:
- `SELECT result_set, count() AS blocks, math::sum(rows) AS rows, math::max(rows) AS max_rows, math::sum(payload_bytes) AS bytes FROM canonical_result_blocks WHERE result_set IN (SELECT VALUE key FROM canonical_result_sets WHERE attempt=$a) GROUP BY result_set;`
- the same query with `rows = 0`;
- `SELECT count() FROM canonical_result_cells WHERE result_set IN (…) GROUP ALL;`
- `SELECT kind, count() FROM canonical_execution_operations WHERE run=$r GROUP BY kind;`

Discriminating outcome: `solve_variables` max_rows of about 20–40 with many zero-row blocks confirms E-01 and E-04; hundreds of rows refutes the E-01 magnitude.

**P2 (E-06).** Run `scripts/pse-env --resource-class light -- python3 -I -c "import hashlib,sys,time;t=time.perf_counter();hashlib.file_digest(open(sys.argv[1],'rb'),'sha256');print(time.perf_counter()-t)" <generation>/bin/pse-worker`. A time of ≥0.3 s per pass makes six passes ≥2 s per study start.

**P3 (§4).** Append the `EXPLAIN FULL` clause (SurrealQL places it at the end of the statement) to `latest_results`' `SELECT … ORDER BY sequence DESC LIMIT 1` on a problem with many study runs. An index scan with limit clears the concern; a range or table scan confirms history growth on ordinary work.

**P4 (E-07).** Run:
- `SELECT released, count() FROM canonical_protections GROUP BY released;`
- `SELECT count() FROM canonical_guards WHERE string::starts_with(key,'execution-operation:') GROUP ALL;`
- `SELECT count() FROM canonical_runs GROUP ALL;`

The discriminating outcome is residue per live run.
