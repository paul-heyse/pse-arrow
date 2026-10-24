# Execution fit

These findings concern necessary versus actual work for the supported workloads: repeated work,
crossings, units of publication, live state, exclusion scope and recovery. They cover the
simulator's edit/re-solve, study, dynamic and recycle workloads, under the maintainer premises of
canonical SurrealDB, 16 coordinated lanes, and durable work that outlives its caller without a
daemon.

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `cbf45ccc9`. Unless a claim says *Measured*, costs are structural observations.

## <a id="da-034"></a>DA-034 — The durable publication unit follows semantic and transport boundaries, and each unit carries the full file-intent lifecycle

**Principles:** AP-07, DP-10, DP-19, DP-20, PS-11; H2, H8, H16, H24, H26, H27, H28; G9 (the
edit → re-solve loop, PSE-S07).

**Refresh status:** new. It continues DA-025's concern with machinery scale in a specific
structural form. It is distinct from DA-026, whose per-row locators are verified removed.

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

## Considered and not raised

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

## Closed by decision, resolved and merged identities

| ID | Refresh outcome | What establishes it |
|---|---|---|
| <a id="da-022"></a>DA-022 | Closed by decision (I16) | Memory admission rests on hand size models: about 328 estimator functions (275 before), with ad hoc constants such as `reads.len() * 128`. I16 investigated and found that existing `HeapUsage` and generated rows account their structures, and adopted no sizing crate. Revisit trigger: a concrete unsupported allocation or graph lifecycle with fallible admission, no double charge, last-alias custody and measured overhead. |
| <a id="da-026"></a>DA-026 | Resolved | `canonical_result_rows` is gone from the generated schema (`f1c12663e`). `workflow/result_snapshot.rs` builds a DataFusion `PruningPredicate` with `Inexact` pushdown, and scalar appends write no per-row outputs. |
| <a id="da-027"></a>DA-027 | Resolved | `pse-modeling/src/specialize/group.rs` and its group/re-expand/re-identify cycle are deleted (`4de4c68d6`). No rendering remains in compiler identity. |
| <a id="da-028"></a>DA-028 | Resolved structurally | Multi-output guarded regions share members across outputs (`pse-modeling/src/resolved/selection.rs::{RegionRequest, RegionOutput}`). The magnitude was never measured. |
| <a id="da-029"></a>DA-029 | Resolved in the compiler; residue merged | Compiler projection holds no per-item linear search. The runtime scans in `require_frozen_expression` and the dynamic row lookup are manifestations of [DA-006](semantic-model-and-authority.md#da-006). |
| <a id="da-030"></a>DA-030 | Closed by decision (I08–I10) | The cross-lane `MathService` cache and flights beside Salsa, and pointer-address keys, are retained. I08 keeps independent publication lanes, I09 keeps reached exact-key flights, and I10 adds no compiler persistence. The `selected()` firewall prevents unrelated edits from invalidating projection. No stale reuse has been demonstrated. Revisit when a stale reuse or a distinct cold-work need appears. |

The removal commit of this refresh preserves their 2026-10-21 text. DA-022's home has moved here
from the local-reasoning page.
