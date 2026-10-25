# Execution fit

These findings concern necessary versus actual work for the supported workloads: repeated work,
crossings, units of publication, live state, exclusion scope and recovery. They cover the
simulator's edit/re-solve, study, dynamic and recycle workloads, under the maintainer premises of
canonical SurrealDB, 16 coordinated lanes, and durable work that outlives its caller without a
daemon.

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `b5bdac9da`. Every owner file of DA-034, DA-035 and DA-025 has an empty diff since `cbf45ccc9`.
S3 re-read their decisive sites at the new baseline. Unless a claim says *Measured*, costs are
structural observations.

## <a id="da-034"></a>DA-034 — The durable publication unit follows semantic and transport boundaries, and each unit carries the full file-intent lifecycle

**Principles:** AP-07, DP-10, DP-19, DP-20, PS-11; H2, H8, H16, H24, H26, H27, H28; G9 (the
edit → re-solve loop, PSE-S07).

**Refresh status:** new. It continues DA-025's concern with machinery scale in a specific
structural form. It is distinct from DA-026, whose per-row locators are verified removed.

**2026-10-25 refresh: unchanged.**
- These all still hold, and no templates changed:
  - the relation-boundary seal in `ResultAppendQueue::push`;
  - the separate `declare_empty_result_set` write;
  - the file per chunk in `append_execution_batch`;
  - the per-file protocol;
  - `aol=sync&sync=every`.
- No newer measurement than I30 exists.
- New in this window: with telemetry selected, each round trip also emits up to three SDK client
  spans:
  - `grpc_lifecycle.rs::observe`;
  - `RequestTrace::new("bounded")`;
  - `RequestTrace::new("grpc")`.

  The sampler defaults to `parentbased_always_on` with queue 2048 and batch 512. Per-request
  continuity is declared contract (Plan 52 O02/F02). Its volume therefore inherits this
  finding's per-unit multiplication. Export granularity and loss visibility are
  [DA-042](#da-042).

**Diagnosis.**
- `ResultAppendQueue` (`workflow/result_projection.rs`) packs batches into a file only within one
  (attempt, relation) pair:
  - It seals at every relation boundary (`push`, when `item.name != name`).
  - An empty relation forces a queue flush plus its own `declare_empty_result_set` write
    (`finish_deferred`).
- `canonical_execution.rs::append_execution_batch` gives every opaque chunk (completion, seeds,
  progress) its own file.
- Each file runs the complete crash-safe protocol (`pse-operations/src/canonical_files.rs`):
  1. `reserve_file`;
  2. two `intent_open` round trips around an instance lock;
  3. `publish_locked`, which fsyncs the file, hard-links it and fsyncs two directories;
  4. a `seal` write;
  5. a persistent `.locks/<instance>.lock` that is kept after collection.
- Metadata appends *are* grouped across relations. The per-unit cost is the file protocol.
- The store runs synchronous persistence (`aol=sync&sync=every`).
- The same unit serves:
  - every progress flush (at most 64 events or 200 ms, `durable.rs`);
  - every live-sample batch (`run.rs`);
  - every horizon step × relation (`durable.rs::publish_horizon_step`);
  - every 4-row study-summary page ([DA-035](#da-035)).
- Durable execution is the *default* Python runtime. An ordinary solve publishes every
  advertised relation under `StudyOutputDemand::Full`: up to 23 relation sets for modeling and
  19 for fit.

**Consequence.**
- **The edit → re-solve loop**, the profile's first workload, pays publication on its critical
  path.
  - The public `RunHandle` wait is terminal only after acknowledgement (Plan 48f).
  - ADR-0211's Running/Publishing overlap helps a worker refill. It does not shorten an
    interactive caller's wait.
  - **Measured** (Plan 48f I30, `docs/plans/48f-scheduling-resources-and-result-completion.md`):
    - five ordinary small KINSOL samples spent a median 1.47 ms in native work and a median
      0.85 s exporting through acknowledgement ("Publication dominated this small case");
    - conditions: a test build, a small payload, measured before Plan 49a moved the store to
      SurrealMX with synchronous persistence;
    - it is not a current-engine measurement and does not decompose round trips from fsyncs;
    - the probe still exists:
      `workflow/durable.rs::canonical_joined_publication_progress_and_ordinary_cost_probe`.
- **Other workloads.** Fixed publication machinery scales with semantic kinds, flushes and steps
  rather than useful bytes:
  - a value-only study occurrence publishes about ten summary sets plus completion, seed and
    progress files;
  - an MPC horizon publishes relations × steps;
  - live recording publishes one file per sample batch.
- **Every unit is permanent**, because collection never runs ([DA-025](#da-025)).

**Counterevidence.**
- Durable-by-default is deliberate: scope §0, §21 and ADR-0164 say "ordinary runtime construction
  is durable". `Runtime(..., ephemeral=True)` gives an explicit store-free route. The defect is
  the per-unit fixed cost, not durability.
- ADR-0187 moved bulk bytes into sealed private files, for bounded RPC envelopes and lazy
  protected reads. Its packing rule packs compatible batches from one
  attempt/relation/schema/interpretation, up to 16 MiB. It rejected per-writer logs as extra
  arbitration machinery.
- An Arrow IPC file holds one schema, so one file per relation is the natural container.
- Demanded acknowledgements flush immediately for observability.
- These justify the file tier and the per-relation container. They do not weigh the fixed cost
  per file for tiny, frequent units:
  - small control receipts already stay in the database;
  - Plan 45 groups physical-source publication by bytes.

**Evidence.**
- *Implemented* for the unit structure and the round-trip and fsync sequence. The coordinator
  re-read the relation-boundary seal and the Full default.
- *Measured* for the I30 small-case ratio, under its stated conditions.
- *Interface-checked* for the claim that the ratio persists on the current engine. Re-running
  the existing probe, and counting files, round trips and fsyncs for one Full solve, one Compact
  occurrence and a 100-step horizon, would settle it.

**Improvement direction.**
- Physical units chosen by flush economics and recovery need:
  - one sealed object per attempt or per publication-lane flush, with per-member descriptors;
  - small payloads inline in the metadata transaction;
  - empty sets declared in the close manifest;
  - progress and live cadence set by consumer demand.
- An interactive caller's critical path holds only what its result requires.
- Crash safety, exact bytes and per-run retirement semantics are kept.

Changing ADR-0187's packing rule is the rule impact a solution would carry.

## <a id="da-035"></a>DA-035 — Study population operations are realized as loops of per-occurrence item operations

**Principles:** AP-07, DP-10, DP-20, PS-11; H7, H8, H10, H11; G9 (PSE-S07, PSE-S03).

**Refresh status:** new.

**2026-10-25 refresh: unchanged.**
- These all still hold:
  - `create_study`'s single 90 s clock and its per-point `begin_run`;
  - `LIMIT 4`;
  - `MAXIMUM_STUDY_POINTS`;
  - the cohort cap of 128.
- Commit `80a889ac1` ("keep independent study claim custody separate") is a custody correctness
  fix. It adds a per-invocation nonce to the `study-claim:` key in `worker.rs::work_study_candidate`
  and does not change population handling.

**Diagnosis.**
1. **Creation.**
   - Point rows are batched at 64 rows or 2 MiB.
   - `pse-operations/src/canonical_studies.rs::create_study` still issues one guarded
     `begin_run` write per point. Each touches the execution, sequence and source-creator guards
     and creates roots and edges.
   - The whole creation runs inside one fixed `within_clock(ACTIVATION_REQUEST_TIMEOUT)` of 90 s,
     whatever the point count.
   - A retry restarts at point 0.
   - The admission bound is `MAXIMUM_STUDY_POINTS = 100_000` (`workflow/study.rs`).
2. **Finalization and observation.**
   - `study_outcome_page` and `STUDY_DEFINITION_SQL` use `LIMIT 4`, because each row carries the
     full descriptor (up to 512 KiB) under the 4 MiB message bound. The outcome row needs only
     the case id and binding hash from it.
   - `workflow/study.rs::finalize_canonical_study` turns each 4-row page into one sealed file,
     one `append_result_block` write and one `renew_attempt` write.
   - The joined caller re-pages every descriptor four at a time.
3. **Analysis.**
   - `workflow/study_workbench.rs::FrozenStudyCohort::aggregate` runs one full protected reader
     per member and holds them all live until one DataFusion `union`.
   - Cohorts refuse more than 64 studies or 128 occurrences.
   - There is no population-level reader over one relation across attempts.

**Consequence.** Take a value-only sweep near the admitted bound, the profile's canonical study
workload:
- **Creation** needs about 100k sequential guarded writes inside 90 s, under 1 ms each including
  synchronous persistence. A timeout repeats them (H11).
- **Finalization** produces about 25k four-row files serially (DA-034, DA-025).
- **Analysis.** Any sweep larger than 128 points needs caller-side paging and recombination.

The middle of the workload is sound: lane-parallel and structure-reusing. Its ends are not.

**Counterevidence.**
- Per-occurrence runs are the recovery and retry unit under the durability premise. Eager run
  creation gives every occurrence durable identity before activation.
- Bounded pages and cohorts keep every response and memory charge finite (DP-20). Decision I28
  retains bounded canonical pages and reopens streaming only for a named consumer.
- No architecture section declares 100k-point studies a supported workload; 100k is the
  admission bound. A clock that does not scale with N and a descriptor-sized page still do not
  follow from these reasons.

**Evidence:**
- *Implemented* for the loops, limits and clock scope. The coordinator re-read `create_study` and
  the `LIMIT 4` pages.
- Whether creation times out at large N is *Interface-checked*: no test exceeds about 132 points.
  Creating and finalizing 10k- and 100k-point studies would settle it.

**Improvement direction.**
- Population intents expressed at set level in the store: batched creation in bounded
  transactions with a resumable cursor, and a clock proportional to declared size.
- Server-side projection of only the fields outcome rows need (H7).
- The study summary published as one or a few objects.
- A population-level reader, or a study-level columnar view, under one protection.

## <a id="da-025"></a>DA-025 — Physical collection is a declared lifecycle transition with no production route

**Principles:** DP-19, DP-20, AP-07; H11, H25, H27.

**Refresh status:** reframed and narrowed.
- The original framing ("persistence machinery sized beyond actual participants") is withdrawn
  under the binding premises: 16 coordinated lanes, durability that outlives the caller across
  processes, and no daemon. The participants are real.
- Machinery scale moves to [DA-033](boundaries-and-composition.md#da-033) and [DA-034](#da-034).
- What survives is the part the prior account left unresolved: physical payload reclamation.
- **2026-10-25: diagnosis unchanged; consequence widened.**
  - `collect_file` still has only test callers.
  - Backup v4 inventories and re-reads every sealed instance, including logically released ones,
    under a fixed verifier clock ([DA-044](#da-044)). Backup cost and refusal risk therefore grow
    with payload that is never collected.

**Diagnosis.**
- §20.4 declares occurrence retirement, history-detail release and physical collection as
  separate transitions, with collection bounded and resumable.
- The only production code that unlinks sealed payloads is `CanonicalStore::collect_file`
  (`canonical_files.rs`). All its callers are in its test module. The coordinator re-checked this
  across `crates/`, `xtask/`, `scripts/` and `python/`.
- Logical reclamation releases borrowers and deletes block and batch rows, but never collects
  the file: `workflow/retention.rs::reclaim_run_results`, then
  `canonical_result_retention.rs::forget_run_results`, then `fn::pse_retention_v1::collect_run`.
- `canonical_lifecycle.rs` reports reclaimed physical bytes as "always unknown".
- Lock files are kept for the storage incarnation by design.

**Consequence.**
- Every DA-034 unit stays for the life of the storage incarnation: sealed file, lock file and
  instance row.
- The public `reclaim_run_results` ("reclaim one bounded page of scientific data") returns no
  disk space.
- A long-lived Plan 51 project store with history and backups grows with every solve,
  occurrence, progress flush and horizon step.
- Backup copies logically released files.

**Counterevidence.**
- The substrate guide deliberately selects no automatic collector and names "returned disk space"
  as a revisit trigger.
- ADR-0214 treats design-phase data as regenerable by fresh initialization.
- The maintainer recorded the unconsumed online collector as a decision.
- These held for a disposable design-phase store. They weaken once long-lived projects, backups
  and a public reclaim API exist, while the declared transition stays unimplemented rather than
  deliberately excluded from the contract.

**Evidence:**
- *Implemented* for the missing caller.
- Growth magnitude is unmeasured. Counting files before and after reclaiming a completed study
  would settle it.

**Improvement direction.**
- The declared collection transition gets an operator- or maintenance-owned route that composes
  with tickets, borrowers and backup freeze.
- Lock retention is proportional to live instances.
- Reclaimed bytes are reported truthfully.
- If collection stays out of scope, the contract says so and the public reclaim API says what it
  does not do. The decision's revisit trigger ("returned disk space") is arguably met by the
  Plan 51 long-lived stores.

## <a id="da-044"></a>DA-044 — Backup verification runs under a fixed clock and makes repeated full content passes over an ever-growing inventory

**Principles:** AP-07, DP-19, DP-20; H11, H20, H25, H27 (PSE-S07 and a long-lived Plan 51 store).

**Refresh status:** new at 2026-10-25. The backup v4 internals were a prior coverage gap. The
code dates from Plan 51 and was not changed in this window.

**Diagnosis** (`scripts/surreal_server.py`).
- **A fixed verifier clock.** `_backup` calls `freeze_payloads`, then `verify_payload_inventory`.
  The latter runs the native verifier (`pse-canonical-recovery --verify-payload-inventory`) over
  every sealed or attached instance under a fixed `timeout=120`, whatever the inventory size.
- **Repeated content passes.** By S3's count, each payload byte is read about six times:
  1. `freeze_payloads`;
  2. the native verifier;
  3. the `copy2` in `copy_payload_inventory`;
  4. `checked_file` on the target;
  5. `checked_file` on the source;
  6. the manifest `file_digest` over the whole destination tree.

  Restore validates the backup at least three times (`restore`, then twice in
  `complete_backup_copy`).
- **Asymmetric verification and durability.**
  - Payload files are copied, fsynced and compared against their source
    (`copy_payload_inventory`).
  - The `database` tree, `.generations` and `.contexts` are copied with `shutil.copytree`, never
    fsynced and never compared against the source. Only the destination copy is digested into
    the manifest. The database's fidelity is checked logically at restore, through the
    application inventory (§20).
  - Restore fsyncs every file and directory before it advances its phase.
- Backup is offline: it stops the store and requires drained contexts.

**Consequence.**
- Because nothing is collected ([DA-025](#da-025)), a long-lived store's backup time grows with
  every payload ever written.
- Past the verifier's throughput × 120 s, backup refuses. This is suspected, since it depends on
  throughput.
- The refusal is a raw `subprocess.TimeoutExpired`, reported by `main` as "TimeoutExpired:
  supervisor operation failed". By then `freeze_payloads` has closed admission and set
  `payload_frozen`. The store therefore stays frozen until an explicit reopen
  (`open_context_admission`). That fails closed, but undiagnosed. This is the challenger's
  reading of the control flow.
- The fixed clock has the same form as DA-035's 90 s creation clock.
- The missing database fsync sits inside a declared limit: §20 "does not … claim power-loss
  durability", and the substrate guide excludes "physical power-loss survival". What remains is
  an inconsistency with restore's own fsync discipline, not a broken contract. A restore detects a
  damaged copy and refuses.

**Counterevidence.**
- An offline full copy is the simplest sufficient protocol for a single operator (H25).
- The native verifier is an independent owner that checks more than the digest.
- Restore is truthful:
  - it refuses rather than repairs;
  - it reports `physical_reclaimed_bytes: None`;
  - it keeps unsettled effects;
  - it publishes a fresh closed identity before copying and can resume.

**Evidence:** *Implemented* for the fixed clock and the copy and fsync sequence; the coordinator
re-read `_backup`, the `timeout=120` call and `copy_payload_inventory`. The six-pass count is S3's
reading. The scale consequence is *Interface-checked*. A backup of a store holding a few hundred
thousand payload files and tens of GB would settle it.

**Improvement direction.**
- Verification bounded by declared inventory size, or resumable, with a diagnosed and
  self-thawing timeout.
- One content pass that both copies and digests.
- Backup durability consistent with restore's, within the declared limit.

**Related:** [DA-019](boundaries-and-composition.md#da-019) (backup lives in the tooling
supervisor), [DA-025](#da-025), [DA-034](#da-034).

## <a id="da-042"></a>DA-042 — Exported span granularity is selected by crate prefix, and the SDK's own drop count is compiled away

**Principles:** AP-05, DP-03, DP-21.

**Refresh status:** new at 2026-10-25 and narrowed before publication by challenge C10. The
draft also claimed that per-request SDK spans and possible span loss were defects. Both are
declared contract:
- per-request continuity is Plan 52 O02/F02, published at blueprint revision 268;
- §24.1 states that "final batches, or all spans from a short solve, may be absent", and that
  health reads "retain unavailable, unsupported, lost, sampled, expired and unknown
  observations".

What survives is narrow and bounded. It is not grouped under the five causes.

**Diagnosis.**
- **Granularity is a prefix convention.**
  - §24.1 says ordinary selected telemetry exports "coarse owner stages".
  - `tooling/telemetry.rs::exported_owner_span` exports every span whose target starts with
    `pse`, plus the vendored SDK lifecycle target.
  - Spans created before Plan 52 for finite capture are therefore exported once trace export is
    selected:
    - per-body admission and reuse spans in `pse-compiler/.../grouped.rs::semantic_body`;
    - per-construction spans in `pse-math/src/execution.rs`.
  - A future `info_span!` in an inner loop would be exported automatically.
- **The drop count that would make "lost" observable is discarded.**
  - `BatchSpanProcessor::on_end` drops a span on a full queue with `try_send`. The SDK keeps a
    private `dropped_spans_count`, reported only through `otel_warn!`.
  - The workspace and the vendored SDK both build `opentelemetry_sdk` with
    `default-features = false` and without `internal-logs`, which is in the SDK's default set, so
    the report compiles out.
  - `telemetry_process.py::local_status` then reports `dropped_records: "unavailable"`. That is
    truthful under §24.1, but it is unavailable by build selection, not by library limitation.

**Consequence.**
- The "coarse stages" property depends on authors never adding an `info_span!` in an inner loop.
- An operator who sees missing spans cannot tell queue loss from sampling or export failure.
  Span volume does follow DA-034's per-unit round trips (see [DA-034](#da-034)), which makes queue
  pressure plausible in long studies.
- Science is unaffected.

**Counterevidence.**
- Spans in `pse-runtime` are coarse.
- No inspected crate uses `trace_span!` or `debug_span!`.
- Per-body spans fire once per distinct body per preparation, not per iteration.
- §24.1 makes no completeness claim, and it says a processed count does not prove receipt.

**Evidence:** *Implemented* for the filter, the feature selection and the SDK's private counter.
S2, S3 and the challenger read these; S3 also read the installed `opentelemetry_sdk-0.33.1`
source. The coordinator re-read the filter and the workspace feature list. Counting spans emitted
against spans ingested for one Full solve and one study of about 1k points would show whether
queue loss occurs.

**Improvement direction.**
- Exported granularity selected by declared owner stage rather than crate prefix.
- The SDK's drop count surfaced through the existing pipeline-health read.

**Related:** [DA-034](#da-034), [DA-040](structure-and-invariants.md#da-040).

## Considered and not raised

**Thread and pool sizing (prior coverage gap, examined 2026-10-25).**
- *One owner inside each process.* `env.rs` sets `native_cores` to the smaller of available
  parallelism and the pool thread count. One `ReleaseNotifyingSemaphore` is shared by:
  - compiler jobs;
  - native math jobs;
  - prepared DataFusion query roots (`pse-engine/src/session/preparation.rs::query_admission`).
- *Native teams receive their admitted width.* MKL `Threads::enter`, the HiGHS `threads` option,
  and explicitly sized rayon pools inside `pounce::with_threads`. BLAS and OpenMP default to 1,
  with `MKL_DYNAMIC` authoritative.
- *Three cross-pool gaps are suspected, not established:*
  1. The `pse-worker` Tokio runtime uses Tokio's default worker count. Under `taskset` to 16
     CPUs, this coincides with the budget.
  2. DataFrame-route reads bypass CPU admission: result snapshot providers and the cohort union.
  3. The store receives thread counts sized to 32 logical CPUs, which overlap the worker's 16
     physical CPUs.
- Reading the existing tokio-metrics long-delay counters during a 16-lane study with concurrent
  workbench reads would settle them.

**Merge ancestry.**
- `project_comparison.rs::project_is_ancestor` walks history caller-side. It makes two RPCs per
  revision, refuses at 8192 visited, and runs twice per decision.
- That is DA-035's shape, but the cost is proportional to the distance to the base, and merges
  are rare.
- Withdrawn as a candidate.
- The merge semantics are a strength: three-way merge by logical identity with typed conflicts,
  and a compare-and-set head.

**Result inspection decode.**
- Block pruning applies before payload acquisition.
- Each block is then decoded at full width and fully admitted, with no IPC column projection.
- This is an H7/H19 lead, unmeasured.

**The per-problem creator guard.**
- Every protection acquisition and every creator writes `canonical_guards` /
  `source-creators:<problem>`, which serializes creator–creator decisions
  (`pse-operations/src/canonical.rs::ACQUIRE_PROTECTION`, `ProblemTurns`).
- Plan 49 decision I01 measured actual 16-lane mixed work. It kept `ProblemTurns` and the shared
  source-creator and sequence guards as necessary pacing, and found no safe excess-scope
  replacement.
- Plan 49a established that `FOR UPDATE` tracks observed records, not range phantoms. That rules
  out the obvious range-read alternative.
- A protection is a creator with respect to reclamation.
- What would reopen it: creators added since I01 (Plan 51 project roots and research rows), and
  per-member protections in study KPI analysis ([DA-035](#da-035)), contending measurably under
  concurrent 16-lane work.
- *2026-10-25.* No new kind of creator exists. Plan 55 adds a low-rate external population: each
  MCP `result_read` or `result_inspect` call acquires one protection and so touches
  `source-creators:<problem>`. Reading the existing `pacing_observation::conflict` counters
  during a 16-lane study with concurrent agent paging would show whether this matters.

## Closed by decision, resolved and merged identities

| ID | Refresh outcome | What establishes it |
|---|---|---|
| <a id="da-022"></a>DA-022 | Closed by decision (I16) | Memory admission rests on hand size models: about 328 estimator functions (275 before), with ad hoc constants such as `reads.len() * 128`. I16 investigated and found that existing `HeapUsage` and generated rows account their structures, and adopted no sizing crate. Revisit trigger: a concrete unsupported allocation or graph lifecycle with fallible admission, no double charge, last-alias custody and measured overhead. |
| <a id="da-026"></a>DA-026 | Resolved | `canonical_result_rows` is gone from the generated schema (`f1c12663e`). `workflow/result_snapshot.rs` builds a DataFusion `PruningPredicate` with `Inexact` pushdown, and scalar appends write no per-row outputs. |
| <a id="da-027"></a>DA-027 | Resolved | `pse-modeling/src/specialize/group.rs` and its group/re-expand/re-identify cycle are deleted (`4de4c68d6`). No rendering remains in compiler identity. |
| <a id="da-028"></a>DA-028 | Resolved structurally | Multi-output guarded regions share members across outputs (`pse-modeling/src/resolved/selection.rs::{RegionRequest, RegionOutput}`). The magnitude was never measured. |
| <a id="da-029"></a>DA-029 | Resolved in the compiler; residue merged | Compiler projection holds no per-item linear search. The runtime scans in `require_frozen_expression` and the dynamic row lookup are manifestations of [DA-006](semantic-model-and-authority.md#da-006). |
| <a id="da-030"></a>DA-030 | Closed by decision (I08–I10) | The cross-lane `MathService` cache and flights beside Salsa, and pointer-address keys, are retained. I08 keeps independent publication lanes, I09 keeps reached exact-key flights, and I10 adds no compiler persistence. The `selected()` firewall prevents unrelated edits from invalidating projection. No stale reuse has been demonstrated. Revisit when a stale reuse or a distinct cold-work need appears. |

The removal commit of the 2026-10-24 refresh preserves their 2026-10-21 text. DA-022's home
moved here from the local-reasoning page.

**Revisit triggers at 2026-10-25: none established as fired; one is a candidate.**
- *DA-022.* The window adds small `HeapUsage` impls and fixed inspection reservations. The OTel
  SDK batch queues and the blocking export clients sit outside the accounted pool, like the
  global-allocator exclusion ADR-0046 already states. No fallible-admission need is demonstrated.
- *DA-030.* `math.rs` adds only process counters, and no stale reuse is reported.
  - The trigger also names "a distinct cold-work need". Always-cold MCP workers
    ([DA-043](development-tooling.md#da-043)) are a **candidate** trigger: a new population that
    cannot reuse in-memory preparation.
  - Whether the need is material is unmeasured. DA-043's cold-call breakdown would decide it.
  - A remedy that keeps a native worker alive would not touch I10. A remedy that persists
    compiler products would reopen I10.
