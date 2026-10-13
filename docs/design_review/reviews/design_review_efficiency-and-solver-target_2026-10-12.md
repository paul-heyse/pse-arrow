---
title: Efficiency alignment and solver target, consolidated
date: 2026-10-12
tier: design
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: Process Simulator 1.5
baseline: 6c368c88acda548d421b44e8fc4f457c95bcec7e
---

# Efficiency alignment and solver target, consolidated

## 1. Synthesis: decision, scope and what to keep

**Decision: Revise.**

**What is sound.** The scientific core is sound and already runs in the user's process:
- the authored model and typed physical admission;
- Salsa preparation and Symbolica evaluators;
- class-specific native solvers;
- the independent original-space assessment.

**What has never been decided.** Where the **working authority** lives, and how many processes may **write** durable state. Today the durable canonical store is the authority for every operation. It is reached over WebSocket by peer writer processes: Python clients, managed workers and a separately supervised server. The cost does not come from durability. It comes from four structural choices.

1. **The store sits on every solve's critical path.** On the default durable runtime, a value edit followed by a re-solve does all of the following before and after its native work ([F01](#f01)):
   - takes and releases two protections;
   - re-reads the root declaration;
   - rechecks the selected admission against the store;
   - registers, claims and heartbeats a run;
   - exports about twenty result relations.

   No cross-process coordination obligation is consumed by any of this. Durability itself is consumed only by the cross-session history and artifact features, which an explicit publication can serve.
2. **Lifecycle state is shared and mutable where immutable facts would do** ([F02](#f02)). Writers are peer processes that talk to a server ([F03](#f03)). Together these are the largest identifiable share of the code and of recent qualification failures, though not most of either:
   - about a quarter of hand-written non-test Rust;
   - most of the supervisor and transport code;
   - 16 of 34 failed or interrupted acceptance attempts.
3. **Results are opaque bytes in the transactional store,** with hand-built indexes, no analytics route and no local tier for unpublished work ([F04](#f04)).
4. **Native work runs at the grain of a call.** Allocations and sessions are discarded at each call and point boundary ([F07](#f07)). Parallelism exists only between cases. Library concurrency constraints are hidden inside adapters ([F08](#f08)).

The 2026-10-12 solver-methods review still describes the code: selection, transition, observation and bridge code are unchanged. This review restates its target on the corrected foundation, which gives method choice structure facts with declared lifetimes, deterministic session reuse and one typed observation record. Two independent challenges corrected material numerical premises along the way (§11). The detailed solver target is in the [solver companion](../evidence/efficiency-and-solver-target-2026-10-12/solver-target.md).

**Two judgments, kept separate.**

- **Architectural fitness fails G9.**
  - AP-07 is violated for edit → re-solve, sweeps, fitting and dynamics.
  - AP-01, AP-05 and AP-06 are violated: lifecycle, placement and capacity responsibilities are spread across six layers, and no runtime exists without the store.
  - AP-03 and AP-04 remain violated for method choice.
  - AP-02 is satisfied at the adapter seam.
- **Behavioural adequacy fails narrowly.**
  - PS-08 (MUST): recycle iteration history is not recorded.
  - G2: typed iteration counts are missing.
  - G7: several capability claims are untrue.
  - G1, G3–G6 and PS-G2/PS-G3 pass on the inspected paths.

### What the design gets right, and must keep

1. **A library-owned, in-process scientific pipeline.** No Newton step, line search or factorization is reimplemented (PS-09).
2. **Independent original-space assessment decides usability (PS-10).** No reuse, warm start or history mechanism may bypass it.
3. **Pure admission with typed refusals,** and a pure strategy walker behind an effect boundary.
4. **Value rebinding over a prepared structural view** (`bound_case`, lane-held study preparation).
5. **§17.6's separation of native allocation reuse from numerical start policy.** Seeds flow only along declared study edges (§19.3). The target extends this separation; it does not erase it.
6. **Immutable revisions; occurrence identities kept separate from content identities; the scientific artifact model** (Plan 37, ADR-0182). These are the raw material of an immutable record.
7. **Truthful partial, failed and unknown outcomes; honest drain; capacity released only after native join.**

### The causes and the findings

| Cause | Findings |
|---|---|
| **A. The durable store is the working authority, written by peer processes** | [F01](#f01) interactive and ephemeral work carries coordination it does not consume. [F02](#f02) mutable lifecycle state is shared where immutable facts suffice. [F03](#f03) separate writer processes and a supervised server carry their own machinery. [F04](#f04) results and observations live as opaque bytes in the store, with no analytics route and no local tier |
| **B. Reuse has several authorities, and structure is not a shared product** | [F05](#f05) Salsa is subordinated to persisted receipts beside several caches. [F06](#f06) structure facts are recomputed or ignored |
| **C. Native work runs at the grain of a call** | [F07](#f07) allocations and sessions are discarded at each call and point, while retained numeric state is thread-history-dependent. [F08](#f08) parallelism is case-only, and shared native resources are implicit |
| **D. Method choice has no governing model, and observation is not decision-grade** (restated solver review) | [F09](#f09) selection models admissibility, not suitability. [F10](#f10) recycle convergence records no history (PS-08). [F11](#f11) observation, measurement and records are thin or untrue; bridges stop by throwing. [F12](#f12) diagnosis runs only on request, with no replay. [F13](#f13) library-owned methods are unreachable |
| **E. Assurance is re-established where nothing changed** | [F14](#f14) validation and admission repeat at crossings that change no premise, using a query engine where compiled expressions suffice |

### Recommended target

The details are in §6. Everything here is *Proposed*.

1. **A live model, a local journal and an immutable record, with one writer per durable log.**
   - **The live model is the working authority.** The owning process holds the Salsa database, prepared bundles and lanes. They serve edit → re-solve and in-process studies without store traffic on the critical path.
   - **A local journal per project** keeps three things:
     - authored revisions, eagerly, so API edits survive a native crash;
     - attempt records and failure replay bundles, always;
     - spilled large results.
   - **The durable record holds immutable facts.** Occurrence identities are minted and tombstoned; content keys only deduplicate payloads. Publishing *seals* journal files with one catalog commit.
   - **Each durable log has one writer.** A durable study is owned by one runner process, and child worker processes return Arrow over pipes. Separate processes are kept for crash isolation and process-exclusive libraries; separate *writers* are not.
   - **A shared multi-writer store remains only if** a product feature needs several independent processes to mutate one project (premise 1, §9). It would then use store-native constraints, not bespoke protocols.
   - **In-process lifetimes** use tokio-util.
2. **Salsa is the single reuse authority for pure products,** with one structure plane:
   - pattern facts keyed by pattern;
   - form facts keyed by body;
   - orderings handed to libraries;
   - symbolic factorizations only for the project's own factorizations.

   Byte-bounded retention is keyed by Salsa identities. Several live revisions coexist, one per lineage.
3. **Lanes own allocation reuse that does not change results.** Numerical warm state flows only along **declared** edges: study continuation chains derived from the study definition, never from scheduling, and an explicit session start policy. Every study and solve declares a **determinism class**, and one owner enforces it.
4. **Concurrency is placement, not method.** Libraries declare the **shared native resources** they use, with access modes:
   - `mumps`: call-serialized;
   - `petsc-process`: exclusive;
   - `highs-scheduler`: shared or exclusive;
   - OpenMP thread settings.

   The plan chooses methods from problem features. Capacity places them on lanes, serial lanes or non-durable process lanes. Inner library widths are fixed per profile and recorded.
5. **Results and observations are Arrow in memory first.** They go to journal files, are sealed into Parquet with a catalog manifest, and are queried with DataFusion. One typed attempt record serves layer-2 decisions (in memory), measurement, history, diagnosis and efficiency telemetry.
6. **Method choice follows the restated solver target.**

### Scope and decisions

| Field | Assessment |
|---|---|
| Subject and boundary | The whole simulator for the target workloads: authoring and admission; preparation and reuse; evaluation; native execution and method choice; studies, fitting, dynamics and recycles; persistence, publication and observation; capacity and parallelism; the Python boundary. The development loop only where product-shaped. Agent tooling and host admission only where the product drives them |
| Standard | Core principles and template 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5, pse-arrow binding |
| Tier / purpose | DESIGN / TARGET |
| Reviewer / date | Coordinator as principal author, 2026-10-12. Supporting inquiries are listed below. Independent omission/preservation (X1) and numerical-methods (X2) challenges are reconciled in §11 |
| Baseline | `main` at `6c368c88a`, clean tree, after Plan 37 closed. Plan 36 is stopped at P-G1; Plans 28 and 32 are open |
| Architectural fitness | **Fails G9.** AP-01, AP-03, AP-04, AP-05, AP-06 and AP-07 violated; AP-02 satisfied |
| Behavioural adequacy | **Revise (narrow).** PS-08 not met; G2 and G7 narrow failures |
| Overall decision | **Revise** |
| Disposition owner | **Not yet scheduled.** The proposed owner is a revised plan series created through `plan-creation` (§10). Rule impacts take effect only after operator confirmation |

**Maintainer inputs, 2026-10-12:**
- Parallel execution is in scope and is to be designed.
- No HSL.
- The development loop is in scope only where product-shaped.
- The solver requirements stand: three-layer selection with advisory history; diagnostics and measurement as product; no benchmark gating; free libraries.

**Supporting documents** in [`../evidence/efficiency-and-solver-target-2026-10-12/`](../evidence/efficiency-and-solver-target-2026-10-12/README.md):
- **Solver companion:** the detailed solver target.
- **Obligations ledgers:**
  - [interactive](../evidence/efficiency-and-solver-target-2026-10-12/w-a-interactive-obligations.md) (W-A);
  - [multi-case and long-running](../evidence/efficiency-and-solver-target-2026-10-12/w-b-multicase-obligations.md) (W-B).
- **[Machinery provenance and qualification archaeology](../evidence/efficiency-and-solver-target-2026-10-12/w-c-provenance-archaeology.md)** (W-C).
- **Library-responsibility surveys**, each with a Part B mapping:
  - R1: [durable state](../evidence/efficiency-and-solver-target-2026-10-12/r1-durable-state-and-coordination.md);
  - R2: [preparation and evaluation](../evidence/efficiency-and-solver-target-2026-10-12/r2-preparation-reuse-evaluation.md);
  - R3: [observation and analytics](../evidence/efficiency-and-solver-target-2026-10-12/r3-observation-results-analytics.md);
  - R4: [numerical methods](../evidence/efficiency-and-solver-target-2026-10-12/r4-numerical-methods-native.md);
  - R5: [capacity and the Python boundary](../evidence/efficiency-and-solver-target-2026-10-12/r5-capacity-parallelism-boundary.md).
- **[Clean-sheet allocations](../evidence/efficiency-and-solver-target-2026-10-12/c1-clean-sheet-allocations.md)** (C1), designed without reading the code.
- **The two challenges:**
  - [omission and preservation](../evidence/efficiency-and-solver-target-2026-10-12/x1-omission-preservation-challenge.md) (X1);
  - [numerical methods](../evidence/efficiency-and-solver-target-2026-10-12/x2-numerical-methods-challenge.md) (X2).
- **Lead status and inquiry briefs.**

No probes, builds, tests or measurements were run. Claims about current code are *Interface-checked*. The principal re-read the decisive paths for F01, F07 and the protection path; X1 and X2 checked further claims against source. Remedies and benefits are *Proposed*.

## 2. Scope, workloads and coverage (slot 1)

**Functional target.** The profile's best-in-class simulator (blueprint §0 and §0.2). It serves:
- edit → re-solve, without structural rebuild for value changes;
- studies;
- recycles and tears;
- dynamics from the same definitions;
- extension;
- compiling and testing one responsibility.

**Workload premises** (qualitative):
- value and structural edits;
- a thousand-point sweep;
- sixteen independent cases on sixteen physical cores, on a host with 32 logical CPUs and about 188 GiB shared with other workloads;
- many small BTF blocks and some large SCCs;
- long trajectories with sensitivities;
- fitting with many observations;
- growing history and interruption.

**How the review was run.** Earlier reviews went deep into mechanisms and missed larger opportunities. This one:
- started from obligations (what each workload stage must guarantee) and from library capability (what the pinned libraries can own);
- had an architect allocate responsibilities without reading the code;
- compared complete alternatives before choosing;
- had both drafts challenged independently.

Prior findings are leads, mapped after synthesis (§10.3).

**Product concurrency, the premise everything depends on.** W-B (a) and X1 §3 separate two things:
- **Coordination across processes.** It is consumed by durable studies served by worker processes, and by several runtimes on one store (ADR-0181), if that is a real product need.
- **Durability and lineage.** These are consumed by the cross-session history and artifact APIs: `latest_results`, `results(run…)`, seed and artifact promotion, `result_analysis` and reclaim.

The interactive path consumes no cross-process coordination. The ephemeral sweep consumes neither.

**Not examined:**
- `pse-engine` session internals beyond their use sites;
- supervisor scripts beyond their product role;
- unit and property model internals;
- store behaviour under contention;
- any measurement.

Several independent structural violations establish G9. Quantitative benefits are unmeasured.

## 3. How the system works today (slots 2, 3, 5)

### 3.1 The interactive loop

Abridged from W-A, on the default durable runtime.

| Step | Scientific work | Durable, coordination and transport work carried |
|---|---|---|
| `Runtime(...)` | none | Connects to the store over WebSocket and opens it, even with `ephemeral=True` (`pse-py/src/workflow.rs::NativeRuntime::new`) |
| Package admission | parse, physical admission (DataFusion inventory), dependency checks | Validation twice; one guarded edit transaction persists every declaration object |
| Declared execution, every solve | resolve route, procedure, limits | Protect → read root declaration → release (`modeling/canonical.rs::declaration`) |
| Selected admission, every solve | Salsa compiler lane | A protection is taken; the cached admission is rechecked against the store |
| Bodies, every solve | Symbolica programs | New descriptions published, although abi3 Python refuses local replay of them (`local_python_configuration` returns `Err`) |
| Case and solve preparation | value rebind into the cached view; policy; routing | in-process |
| Run, every solve | native solve and original-space assessment | Run registration, claim, 30 s lease, 10 s heartbeat, progress stream. A new native session thread (`run.rs::launch` → `Staged::open_queued`). Ephemeral runtimes skip registration and export |
| Publication, every solve | completion capture | About 20 relations encoded as IPC blocks with row indexes; seeds; terminal manifest. A later `.table()` re-encodes |
| Structural edit | reference invalidation | The complete parent package is hydrated from the store before a new revision is published |

### 3.2 Multi-case and long-running work

Abridged from W-B and X2.

- **Ephemeral sweep:**
  - lanes keep structural preparation and rebind per point;
  - each independent point opens a new native session; sessions are kept only along single-successor continuation;
  - source leases are still renewed in the store.
- **Durable sweep:**
  - about ten store transactions per point;
  - per-process recompilation, with the engine cache disabled;
  - no batching;
  - seeds re-read per point.
- **Fitting:**
  - experiments run sequentially;
  - each oracle call builds a new evaluator worker and integrator for every transient experiment.
- **Dynamics:** results are appended only at `finish`.
- **Recycles and initialization:** one session per staged sequence; no parallel blocks.
- **Retained KINSOL inner sessions:** these are thread-local and reuse the previous point's numeric factor (`NoInitSetup`). Iteration paths therefore depend on what each thread solved before (X2 fact C).

### 3.3 Capacity and shared native resources

From W-B, R4, R5 and X1.

- **Per process:**
  - one memory pool;
  - one CPU semaphore plus `jobs` and `active_jobs`, default 4.
- **Per library, beside that accounting:**
  - **FERAL** builds a per-solver pool sized from the environment. It cannot be injected.
  - **faer `Par::rayon`** in implicit blocks runs on the global pool.
  - **Ipopt with MUMPS (the default)** is *call-serialized* by Ipopt's `mumps_call_mutex`. Only MUMPS phases queue; evaluations run concurrently.
  - **PETSc** is process-exclusive at this build.
  - **HiGHS:** direct HiGHS shares a lifecycle read lock, while Uno's HiGHS subproblems take it exclusively.

### 3.4 Machinery and its history

From W-C. These are history facts, not proofs of defects.

**Code mass.** Hand-written non-test Rust is about:

| Category | Lines |
|---|---|
| Scientific | 140k |
| Lifecycle and coordination | 85k |
| Engine admission and caching | 23k |
| Transport | 11k |
| Mixed | 52k |

The supervisor and test-resource scripts add about 38k lines, with 43.5k lines of tests.

**Admission covers three different responsibilities**, implemented in several layers:
- semantic plan admission;
- capacity admission;
- deployment and host placement admission.

**Since 2026-09-28:**
- The publication authority moved from Delta to PostgreSQL to SurrealDB.
- 37 of the 57 `canonical_*` relations arrived in two commits.
- Of 34 failed or interrupted acceptance attempts, 16 were lifecycle, 9 fixture or harness, 6 build or environment, and 2 scientific. Several of the lifecycle ones are capacity or placement failures (X1 O3).
- No comprehensive assessment of Plans 28, 30 or 33–37 records a pass.

### 3.5 Physical semantics, well-posedness and stage columns

- **Physical semantics.** No physical semantics change; the 2026-10-11 table stands.
- **Well-posedness.** As in the solver review §3.2 (PS-G2 passes). What is missing is numerical failure localization ([F12](#f12)).
- **Stage columns.** These are in the [solver companion](../evidence/efficiency-and-solver-target-2026-10-12/solver-target.md#stage-columns).

## 4. Findings by cause (slot 7)

Each finding's evidence label applies to its diagnosis. All remedies are *Proposed*.

### 4.1 Cause A: the durable store is the working authority, written by peer processes

#### <a id="f01"></a>F01: Interactive and ephemeral work carries durable coordination it does not consume

**Principles.** AP-07 and AP-06 (H1, H5, H12, H25, H26); DP-10, DP-18; PSE-S01 and S05; audit S01.

**Evidence.** *Interface-checked*: W-A §1–§4 and (a); W-B 1a; principal re-read of `NativeRuntime::new`, `canonical.rs::declaration` and `run.rs::launch`; X1 §3.

- **Every Python `Runtime` opens the store.** Every package method enters a canonical activity scope.
- **On the default durable runtime, every solve** (including a value-only `solve_bound`):
  - makes at least six store round trips before native work;
  - registers, claims and heartbeats a run;
  - exports every advertised relation.
- **An ephemeral runtime** skips registration and export. It still requires canonical sources and renews source leases.
- **A structural edit** hydrates the whole parent package.
- **The requiring contracts are rules:**
  - §14.3 and §14.4 ("persisted complete dependencies govern reuse … Salsa is an optional local accelerator");
  - §19 and §21 (durable by default; ephemeral still admits canonical sources);
  - the scope table, "Canonical store availability";
  - ADR-0164 (proposed).
- **What actually consumes it.** The interactive path consumes no cross-process coordination obligation. Its durability is consumed by the history and artifact APIs (X1 §3).

**Consequence.**
- A value edit's critical path grows with store round trips, protection churn and result size.
- Interactive behaviour cannot be exercised without a running store (AP-06).
- In-process library authorities (Salsa, cached views, native sessions) are demoted to accelerators that are re-qualified against the store on every call.

**Correction.**
1. **The live model of the owning process is the working authority.**
2. **Separate input durability from execution-record durability:**
   - Authored revisions are small and content-addressed. They are journaled eagerly to the local journal, off the solve path, so API edits survive a native crash.
   - Execution records go to the journal as attempt records and replay bundles.
3. **Publication is an explicit sink** that seals journal files with one catalog commit. A published result also publishes the content of the start it used. An unpublished predecessor is recorded as such, never as a dangling reference.
4. **"Current revision" follows a working-tree-versus-head model:**
   - named heads in the record are the cross-process authority and move only by CAS on publish;
   - the session's live model is authoritative for its own working copy.
5. **A runtime with no publication demand does not open the store.**

**Preserved.** Exact lineage and reproducibility of every *published* result, and independent assessment.

**Intentional change.** The cross-session APIs (`latest_results`, `results(run…)`, artifact and seed promotion, `result_analysis`, `reclaim_run_results`) see only published or journaled runs ([RC01](#rc01)).

**Verification.**
- A value edit performs no store operation.
- A crash after an API edit loses no authored input.
- A published result carries exact revision and start identity.

#### <a id="f02"></a>F02: Mutable lifecycle state is shared where immutable facts suffice

**Principles.** AP-01, AP-05 and AP-07 (H1, H4, H24, H25, H27); DP-16, DP-19; PSE-S05 and S07.

**Evidence.** *Interface-checked*: R1 Part A §3 and Part B; W-B (a); W-C §2–§7; generated DDL.

The durable tier keeps much *mutable* lifecycle state:
- protections, which are guarded writes even for reads;
- leases and heartbeats;
- read protections that make result reads mutations;
- readiness records and their repair;
- creator, reclaim and seal fences in both commit orders;
- uncertain-effect settlement;
- history release with permanent owner fences.

The DDL holds 57 SCHEMAFULL tables, 94 `fn::` functions, 246 `THROW` and 84 `FOR UPDATE`.

Each lifecycle concept goes through the scientific-relation pipeline: a registry declaration, generated Rust and Arrow, DDL, a `fn::` function, Python types and tests.

Store-native capabilities are unused: typed references with `ON DELETE REJECT`, sequences, unique-claim idioms. The I1–I7 capability decisions were "retain" choices made feature by feature.

**Consequence.**
- Each new workflow or failure scenario adds protocol in several owners.
- Coordination work grows with retained history and transitions.
- This is the largest single share of recent qualification failures (W-C §7, attributed three ways: here, [F03](#f03) and [F08](#f08)).

These effects are material even though each protocol answers a real race.

**Correction.**
1. **Classify durable state explicitly** (AP-05) as either an immutable fact or shared mutable coordination.
2. **Immutable facts:**
   - They are append-only.
   - **Occurrence identities** (publication, attempt, study) are minted, never reused, and tombstoned on retirement, as in ADR-0182.
   - **Content keys** only deduplicate payloads.
   - A retry of the *same occurrence* settles itself by its unique key. Re-execution is a new occurrence.
3. **Shared coordination** is kept only where several writers exist, which [F03](#f03) narrows. Where it exists, it uses store-native constraints: unique claims, sequences as fences, typed `REFERENCE REJECT`, one server function per transition. Every claim-dependent fact checks its claim fence inside its own commit.
4. **Staging and the orphan sweep:**
   - Writers stage files under a directory owned by their claim or process identity.
   - A sweep removes only directories whose owners are provably dead.
   - The commit and the sweep share one fence.
5. **Protected multi-page reads of files outside the store are weakened, and the target says so:**
   - catalog-first deletion;
   - a grace period longer than the maximum read;
   - a typed `ResultSetRetired` mid-read.
6. **In-process lifetimes** use tokio-util `TaskTracker`/`CancellationToken`.
7. **Lifecycle records get their own small declaration route.**

**Preserved:**
- no resurrection of retired identities;
- honest unknowns for shared coordination;
- drain before close;
- the artifact sufficiency model.

**Verification.**
- An interrupted publication leaves only owner-scoped staging, swept after the owner dies.
- Re-running a reclaimed computation creates a new occurrence.
- A stale worker's completion fact is refused.

#### <a id="f03"></a>F03: Separate writer processes and a supervised server carry their own machinery

**Principles.** AP-07 and AP-01 (H25, H27, H23); DP-20; PSE-S06 and S07.

**Evidence.** *Interface-checked*: X1 O1; W-C §2–§5 and §7; W-B (a); R1 F10.

Much of the machinery exists because the deployment is a set of **peer writer processes** that talk to a separately supervised SurrealDB server over WebSocket. It does not exist because state is mutable:
- the vendored SDK request-lifecycle patch (about 5k lines): bounded admissions, replay and drain;
- the ADR-0181 caller, activity and physical lifetimes;
- `surreal_server.py` (about 7.5k lines);
- `host_admission.py`;
- `native_operation.py`;
- receiver generations;
- observer pids, systemd units and cgroup placement proofs.

Plans 29, 30, 35 and 37 were largely spent there. Nine of W-C's failed attempts concern placement, drain, receivers or SDK races; none concerns shared scientific state.

Separate *processes* are needed for native-crash isolation and process-exclusive libraries. Separate *writers* are a design choice (ADR-0164, Plan 30).

**Consequence.** Classifying state ([F02](#f02)) would leave all of this in place. Every interactive call also carries network unknown-outcome settlement and transport drain.

**Correction.**
- **Durable studies.** A durable study, or a project, has **one writer per durable log**: a runner that owns the study and dispatches points to child worker processes over pipes, which carry Arrow IPC. Completion facts go to the study's log, and resume reads that log.
- **The catalog** can then be embedded SurrealDB (one process per directory) or a manifest log over Parquet. This choice stays open, with criteria (§8.2).
- **Ephemeral studies** get **non-durable process lanes**: child processes for process-exclusive or crash-prone families, with commits mediated by the owner.
- **A multi-writer server** is kept only if premise 1 holds (§9): a product feature in which several independent processes mutate one project.

**Preserved.**
- Crash isolation.
- Process-exclusive library placement.
- Durable-study resume.
- Quiesce, stop and backup, by a contract for the split layout: snapshot the catalog, then copy the referenced immutable files with garbage collection paused.

**Rule impact:** [RC05](#rc05).

**Verification.**
- A durable study survives worker crashes and owner restart with no server process.
- No transport drain or network settlement remains on the interactive path.

#### <a id="f04"></a>F04: Results and observations live as opaque bytes in the store, with no analytics route and no local tier

**Principles.** AP-07 (H6, H7, H11, H15, H16, H17); DP-14; PSE-S04; audit S06.

**Evidence.** *Interface-checked*: R3 Part A §2–§4 and Part B; W-A 1.17 and 4.1–4.4; W-B 4a; X1 O2.

**How results are stored and read today:**
- Results are IPC blocks of at most 512 KiB in store tables.
- A per-row index and per-block coordinate min/max form a hand-built zone map, which is verified against the payload on read.
- `.table()` re-encodes on every read.

**What is missing:**
- There is no relational route for cross-run analytics: study summaries, convergence-effort regression and history.
- Long trajectories are written only at `finish`.
- There is no file-backed tier for large unpublished results, failure replay bundles or interactive attempt records.

**Consequence.**
- Read and write work grows with block and index counts.
- Measurement and history need bespoke summarizers.
- Unpublished large results stay wholly in memory (H11, H17).
- Interruption loses whole trajectories.
- History never learns from interactive use.

**Correction.**
- **The store, or catalog, keeps authority facts and manifests.**
- **Bulk relations and the typed attempt record** go to the local journal as Arrow or Parquet under bounded live retention. They are sealed by a catalog commit on publication.
- **DataFusion** serves inspection and analytics over committed manifests and over the local journal.
- **Python reads the retained Arrow batches** with zero copy.
- **Long trajectories stream** as row groups.
- **Integrity:** today's digests are transport integrity, not scientific identity. Integrity hashes move to the sealed file; scientific identity is unaffected.

**If [RC03](#rc03) is kept:** a DataFusion provider over the existing blocks gives analytics but removes no amplification.

**Verification.**
- A study summary and a convergence-effort table are single queries.
- A `.table()` read performs no encoding.
- An interrupted horizon keeps its sealed prefix.
- A large unpublished result stays within bounded live memory.

### 4.2 Cause B: reuse has several authorities, and structure is not a shared product

#### <a id="f05"></a>F05: Salsa is subordinated to persisted receipts, beside several caches

**Principles.** DP-09 and AP-07 (H12, H13, H14); AP-04; PSE-S01, S02 and S07.

**Evidence.** *Interface-checked*: R2 Part A §2 and Part B; W-A (a) and 1.9–1.12; W-B (c).

The pure-product reuse authorities in one process:

| Authority | Notes |
|---|---|
| Salsa compiler lanes | |
| `modeling_cache` | eleven families with hand-written keys |
| Compiled-program cache | a DataFusion `DefaultCache` keyed by content hash |
| `selected_flights` | |
| Persisted descriptions and receipts | consulted per solve under a store recheck |

Further evidence:
- §14.4 subordinates Salsa to persisted dependencies.
- The basis key includes path demand.
- `held_preparation` is `None` on the Python path.
- Workers recompile per process.

**Consequence.**
- Proving reuse costs store round trips per call.
- Several authorities must agree on invalidation.
- Edits cannot cut off early through programs and structure.

**Correction.** Salsa is the single invalidation authority for pure products, from parse to compiled programs, with definition → pattern → point keys.

- **Coexistence.** The Python API is value-semantic (`pkg2 = pkg.with_delta(...)` keeps `pkg`). Several live revisions therefore coexist: one database per live lineage, sharing through the content-keyed memo, or revision-keyed inputs.
- **Edits** are input setters on a lineage's own database.
- **Studies.** A study freezes an immutable prepared bundle and releases its handles. A structural or discrete study (a DOE over discrete choices, fixed/free flips) prepares each distinct structure at admission, or runs on its own lineage database.
- **Memory bound.** A byte-bounded retention keyed by Salsa identities enforces memory limits.
- **Disk memo.** A content-keyed disk memo serves cold starts and workers. Its key covers the evaluator ABI, library versions and backend profile. Loading from another writer is a validated trust transition.

**Preserved.**
- Tracked queries stay pure.
- Keys stay complete.
- Every new numerical state receives independent assessment.

**Verification.**
- A value edit touches only binding inputs.
- An expression edit rebuilds one body's program.
- Alternating solves of two live revisions rebuild nothing.
- There is one retention owner per product kind.

#### <a id="f06"></a>F06: Structure facts are recomputed or ignored instead of being one shared product

**Principles.** AP-03, AP-07 and DP-06 (H9, H12, H15); PS-05; PSE-S02 and S07.

**Evidence.** *Interface-checked*: R2 §3 and A3; solver-review F02/F11 (unchanged); X2 §3.

- Matching, DM/BTF and SCC structure exist, but they do not reach method choice, per-block execution or scheduling.
- KLU's BTF duplicates pounce's.
- Colourings, levels and critical paths are not produced.
- FERAL repeats Markowitz ordering per block setup.

**Correction.** A **structure plane** of Salsa products:

| Kind of fact | Contents | Keyed by |
|---|---|---|
| Pattern facts | matching, DM, BTF blocks and levels, coupling class, colourings, orderings, sparsity | pattern identity, per weakly connected component, with component identities stable across edits |
| Form facts | affine, polynomial, closed-form solvable | block and body programs, **not pattern**, because an expression edit keeps incidence but can change form |

- **Symbolic factorizations** go only to the project's own factorizations: faer, direct KLU, FERAL in `kinsol/block.rs`. Libraries that analyse internally receive at most an **ordering** (KLU `klu_analyze_given`; MUMPS ICNTL(7); SPRAL and Pardiso permutations).
- **Pattern facts are advisory to per-block method choice** whenever value features contradict them, for example a coefficient that is zero at the current values. This produces a typed cause.

**Verification.** Plan, scheduler and diagnosis read one block structure, and one producer exists per fact.

### 4.3 Cause C: native work runs at the grain of a call

#### <a id="f07"></a>F07: Allocations and sessions are discarded at each call and point, while retained numeric state is thread-history-dependent

**Principles.** AP-07 and DP-10 (H12, H16, H17); DP-11, DP-19; PS-11; PSE-S07.

**Evidence.** *Interface-checked*: W-A 1.15 and 2.7; W-B; R4 Part B; X2 facts A–C and I; principal re-read of `run.rs::launch` and `integrated.rs::integrate`.

- Each Python solve, each independent study point and each durable attempt opens a new native session thread.
- Each transient-fit oracle call builds a new evaluator worker and integrator.
- The thread-local KINSOL inner-session cache reuses the previous point's **numeric** factor (`NoInitSetup`). Iteration paths therefore depend on thread history.
- At the pins:
  - Ipopt repeats its linear solver's symbolic analysis on every `OptimizeTNLP` and `ReOptimizeTNLP`;
  - SUNDIALS repeats KLU symbolic analysis on every `KINSol` call and after every `IDAReInit`.

  Library-retained symbolic reuse exists only in POUNCE/FERAL, Clarabel, HiGHS and the project's own factorizations.
- §17.6 already keeps allocation reuse independent of the numerical start policy, and seeds flow only along declared study edges (§19.3).

**Consequence.**
- Re-solves, sweeps and fits repay setup and allocation at every call.
- Thread-local native teams restart.
- The one place that does retain numeric state makes paths scheduling-dependent without a declared determinism class (DP-11).

**Correction.**
1. **Lanes** (long-lived threads) own **result-neutral allocation reuse**:
   - memory, contexts and compiled programs;
   - TNLP and KINSOL setup;
   - symbolic analysis where the library or the project keeps it. A project-owned `SUNLinearSolver` for KLU retention would follow the Diffsol precedent in `dynamics/linear.rs`.

   Numeric retained state (stale factors, old pivot orders, quasi-Newton memory, HiGHS basis) is reset unless the start policy allows it.
2. **Numerical warm state flows only along declared edges:**
   - study continuation chains that the study definition declares, for example along an ordered parameter path. Chains are the unit of lane affinity and of grouped durable claims, and are never derived from lane count;
   - an explicit interactive session `StartPolicy`, recorded per result.
3. **Each profile declares the terminal states after which a session may be reused.** Any other state resets the session.
4. **Sessions are bounded** by bytes or entries per lane.
5. **Transient fits keep one IDAS memory per experiment** and reinitialize it per call: fresh `IDACalcIC`, `IDASensReInit` with fresh s₀, `IDAAdjReInit`. Event-crossing fits run on Diffsol and keep the problem and the project linear solver.
6. **Retained Ipopt sessions keep today's freeze of bounds at creation,** because `warm_start_same_structure` does not recompute the fixed-variable map (X2 fact I).

**Preserved:**
- §17.6's separation;
- DP-19 exclusive workspace ownership;
- drain on cancellation;
- PS-10 assessment of every new numerical state.

**Verification.**
- Two runs of the same study with different lane counts produce identical point results within the declared determinism class.
- A second solve on a lane allocates nothing new.
- A transient fit allocates one integrator memory per experiment.

#### <a id="f08"></a>F08: Parallelism is case-only, and shared native resources are implicit

**Principles.** AP-07 and DP-20 (H21, H22, H17); AP-05; DP-11; PSE-S07.

**Evidence.** *Interface-checked*: W-B (b); R5 Part A §2 and Part B; R4 §1.10 and Part B; X1 §2; X2 item 3.

- **No structure-level parallelism.**
  - The default is `jobs = 4`.
  - BTF levels, shooting windows, fit experiments and finite-difference colour groups run sequentially.
  - The durable path never batches.
- **Capacity is charged in nested ways.**
  - There are three semaphores, with refusal at native entry.
  - faer `Par` uses the global pool.
  - FERAL builds pools that admission never lets run in parallel.
  - The OpenMP wait policy is unset.
- **Shared native resources are enforced inside adapters, invisible to admission and preference:**
  - MUMPS is call-serialized;
  - PETSc is process-exclusive;
  - Uno's HiGHS takes the HiGHS scheduler exclusively.

**Consequence.**
- A single large case cannot use the machine.
- Sixteen default-route NLP cases queue on MUMPS phases. The cost scales with the factorization share, which is unmeasured.
- Process-exclusive methods force an ephemeral study either to run serially or to become durable.

**Correction.**
- **Shared native resources with access modes are declared per profile:**
  - `mumps`: call-serialized;
  - `petsc-process` and `ibex`: exclusive;
  - `highs-scheduler`: shared or exclusive;
  - OpenMP thread settings.

  Admission and **placement** reason over resource conflicts. **Method choice comes from problem features** with one context-independent default. A context-dependent choice is allowed only as an explicit, declared study profile, recorded per point.
- **Placement.** Lanes, a serial lane, or non-durable process lanes ([F03](#f03)). Process lanes parallelize *independent* PETSc problems. They do not parallelize the subdomains of one NASM or ASPIN solve, which would need MPI.
- **One CPU budget per process,** split into lanes by study policy. Lanes can re-form between points, so a skewed study does not leave a wide case at width 1.
- **Inner library widths are fixed per profile and recorded.** They are not derived from lane context, because faer `Par` and threaded factorizations are not width-invariant. FERAL runs serially unless its pool width matches the process size.
- **Structure-level parallelism on a lane's scoped team** (wavefront blocks, shooting windows, fit experiments, colour groups) is admitted only where it matches the sequential reference. That requires:
  - level-synchronous semantics;
  - block-owned numeric state;
  - deterministic per-block budgets;
  - fixed-order reductions;
  - static assignment where native state is thread-affine.
- **Memory estimates from structure facts are heuristics** for library allocations. tokio handles I/O only.

**Preserved:**
- capacity released after native join;
- PS-10.

**Verification.**
- One large case uses its whole budget.
- An ephemeral 16-case study with a process-exclusive method runs in process lanes.
- Parallel and sequential runs agree within the declared determinism class.

### 4.4 Cause D: method choice and observation

This cause restates the solver review. The detail is in the [solver companion](../evidence/efficiency-and-solver-target-2026-10-12/solver-target.md), including X2's numerical corrections. Lead status confirms the code is unchanged.

#### <a id="f09"></a>F09: Selection models admissibility but not suitability, and structure never reaches it

This subsumes solver-review F01, F02, F03 and F11.

**Today:**
- Selection is a class plus a constant rank, with fixed statement-order composition.
- Transitions are gated on the termination category and include a SCIP-after-local-failure edge.
- Per-block choice is impossible.
- AP-01, AP-03, AP-04 and AP-05 are violated.

**Target.** A pure solve plan over the existing admission, with:
- features at three lifetimes, from [F06](#f06);
- corrected per-block rules:
  - the branch comes from the authored phase rule;
  - Brent only on a verified sign change;
  - KINSOL Dense with scaling rather than raw `block_solve`;
  - near-singularity treated as a value fact, with TR/LM rather than NLEQ-ERR;
- per-block acceptance on the global row-wise criterion, with a simultaneous polish only when global verification fails;
- cause-typed transitions;
- advisory history permuting within a tier, with its snapshot recorded.

#### <a id="f10"></a>F10: Recycle convergence records no iteration history and defaults to the weakest method

This is solver-review F04. **PS-08 (MUST) is not met.**

**Target:**
- History is recorded from the map callback (‖G(x) − x‖ per tear, Err/Tol, oscillation), because KINSOL's norm under Anderson is the accelerated step.
- A contraction diagnosis applies only over damped windows.
- The default is damped fixed point plus Anderson after a delay, with a project-owned restart rule.
- The successor is typed by cause and by unit representation: equation-oriented → simultaneous SCC; opaque → Broyden or JFNK.

It is independent of the architecture and lands first.

#### <a id="f11"></a>F11: Observation, measurement and records are thin or untrue, and bridges stop by throwing

This subsumes solver-review F05, F06, F07, F10, F12 and F13.

**Today:**
- **G2:** typed iteration counts are missing.
- Statistics are unread.
- There is no attempt time.
- Route records hold no reasons.
- **G7:** untrue claims:
  - HiGHS "native updates";
  - Ipopt "timing";
  - the §18.6/§18.7 escalation text;
  - FERAL "global pool" in §18.8 and R-41;
  - the Delta text in `row_checks.rs`.
- The Ipopt and Uno bridges stop by throwing.

**Target:**
- Typed observation families, including `inertia_mode` and FERAL `n_tiny`, as the per-adapter part of **one attempt record**.
- Cooperative stops with declared fidelity, as corrected in companion §3.4.
- A history layer that:
  - treats only budget exhaustion as censored;
  - conditions on plan position and start kind;
  - is replayed from the recorded permutation.

#### <a id="f12"></a>F12: Diagnosis runs only on request, and failed solves cannot be replayed

This is solver-review F08.

**Target:**
- A bounded automatic diagnosis tier, mapped to model elements, that never changes an outcome.
- Replay from **journaled** inputs and the recorded plan ([F04](#f04)), so interactive failures are replayable.
- An Ipopt stop during restoration loses the restoration slacks, so a diagnostic stop waits for restoration to exit.

#### <a id="f13"></a>F13: Library-owned methods are unreachable

This is solver-review F09, restated for no HSL and for shared native resources.

**Methods to bind:**
- PETSc QN, NGMRES, Anderson, NLEQ-ERR (with SSLS only), VI, NEWTONAL;
- TAO BRGN, BNTR and TRON, placed on a serial or process lane;
- the KINSOL difference-quotient Jv;
- argmin Brent;
- Ipopt statistics, timing, `nlp_scaling_method=user-scaling` from nominals, and the curvature test;
- POUNCE batches, as an alternative to lane sessions, not a complement;
- IDAS getters;
- SCIP statistics.

**Lost without HSL:**
- MC19 equilibration NLP scaling, compensated only by user scaling, which makes the nominal contract load-bearing;
- MA57 in POUNCE;
- IDAES iteration-path parity, so parity becomes solution-level ([RC10](#rc10)).

### 4.5 Cause E: assurance re-established where nothing changed

#### <a id="f14"></a>F14: Validation and admission repeat at crossings that change no premise, using a query engine where compiled expressions suffice

**Principles.** AP-07 (H18, H19, H20); DP-14; G8 warning.

**Evidence.** *Interface-checked*: W-A (b), (c); R3 §4.2 and Part B; X1 §2.

- In one journey, field validation runs at eight sites, built through a DataFusion `SessionState`:
  - re-validation at load;
  - batches built only to validate, then dropped, at persistence and delta;
  - re-encoding with validation on every Python read.
- `pse-engine` is about 35k lines. Its only production query execution is one physical-inventory admission path.
- Deployment-local reuse eligibility rests on executable-mapping observation (ADR-0164), which abi3 refuses outright.

**Consequence.**
- Assurance work grows with crossings.
- A planner-sized subsystem serves a small admission workload.

**Correction.**
- Carry admission through immutable products with typed validated wrappers (H19).
- Compile each registry validation expression once to a `PhysicalExpr`, without a session.
- Keep DataFusion's planner for the analytics plane ([F04](#f04)).
- Re-validation after decode **remains** whenever bytes come from another writer, which is a real trust transition. It is skipped only when the writer's validation-contract version equals the reader's and the digest matches. The saving there is mainly same-process, which the target serves from memory anyway.
- Under [F05](#f05), cross-process reuse becomes a content-keyed memo verified by content and loaded as a trust transition. Executable observation is narrowed to that need.

**Verification.** One validation per trust transition, and one compilation per expression.

## 5. How the findings relate

**Prerequisite contracts, in order:**
1. **The state and writer boundary:** [F01](#f01), [F02](#f02) and [F03](#f03), with the local journal ([F04](#f04)) and the split between occurrence and content identity. Without them, [F05](#f05) cannot replace store rechecks and [F07](#f07) cannot keep sessions across calls.
2. **The determinism-class owner (§6.5).** It must exist before [F07](#f07)'s retention or [F08](#f08)'s parallelism changes any result path.
3. **[F06](#f06)'s structure plane.** [F08](#f08)'s structure parallelism and [F09](#f09)'s per-block plans both consume it.
4. **The shared-resource model and non-durable process lanes,** before [F08](#f08)'s placement rules.
5. **[F04](#f04)'s observation plane and [F11](#f11)'s attempt record.** These are one design. Measurement, history and replay ([F12](#f12)) depend on them.

**Priority is not prerequisite order.** These behavioural fixes need none of the architecture and should land first:
- [F10](#f10) (PS-08);
- [F11](#f11)'s G7 and G2 corrections;
- the HiGHS basis;
- removal of the SCIP-after-local-failure edge;
- removal of the stale-factor reuse in the thread-local KINSOL cache across independent points. That reuse is a DP-11 concern today.

**The remedies fit together under five conditions:**
- the solve plan stays a pure consumer of admission;
- warm state is declared, never scheduling-derived (PS-11);
- lanes own native state exclusively (DP-19);
- every new numerical state gets independent assessment (PS-10);
- method choice is context-independent unless a declared study profile says otherwise.

## 6. Target design (recommendations)

Everything in this section is *Proposed*. It describes responsibilities, not work packages.

### 6.1 State tiers and writers

| State | Tier | Owner and form |
|---|---|---|
| Authored definitions and revisions | Journal (eager) → record on publish; named heads by CAS | Salsa inputs per declaration and field in each live lineage |
| Prepared structure, structure plane, programs | Process-local; optional content-keyed disk memo | Salsa products; frozen `PreparedProblem` bundles |
| Native sessions and factorizations | Process-local, lane-owned | Allocation reuse keyed by Salsa identities; numeric state reset unless declared |
| Starts | Recorded with results | Along declared edges only |
| Results | Memory → journal (bounded live) → sealed Parquet plus a catalog manifest on publish | Arrow |
| Attempt records and replay bundles | Journal always → sealed on publish | Typed in memory during the solve |
| Method history | Derived | DataFusion aggregate over journal and record logs; snapshot identity recorded |
| Study definitions and point-completion facts | Durable log, one writer | Runner owns the study; child workers return Arrow over pipes |
| Shared multi-writer coordination | Only if premise 1 holds | Store-native constraints |

**Catalog choice, open with criteria.**
- **Embedded SurrealDB:** transactions, unique identity and typed references. It costs a core in the build and allows one process per directory.
- **A manifest log over Parquet:** no database at all; DataFusion reads it.

Either serves one writer per log.

### 6.2 Reuse and the structure plane

This is [F05](#f05) and [F06](#f06).

| Edit | Effect |
|---|---|
| Value edit | One binding input changes. The lane re-solves with a recorded `StartPolicy`. Programs are untouched, because parameters are evaluator inputs (premise 4) |
| Expression edit | The body's program and its form facts change. Pattern facts backdate when incidence is unchanged |
| Structural edit | Pattern facts are recomputed per changed component |

### 6.3 Lanes, placement and parallelism

This is [F07](#f07) and [F08](#f08). Shared native resources at the pins:

| Resource | Access mode | Used by |
|---|---|---|
| `mumps` | call-serialized | Ipopt with MUMPS; `dependency_detector=mumps`; SCIP's nested Ipopt when so configured |
| `petsc-process` | exclusive | PETSc SNES/TS/TAO |
| `ibex` | exclusive | root isolation |
| `highs-scheduler` | shared (direct HiGHS) / exclusive (Uno's HiGHS) | HiGHS, Uno |
| OpenMP thread settings | per thread | SPRAL, MKL Pardiso |

**Free per thread or per context:** POUNCE/FERAL, SUNDIALS (one context per thread), Clarabel and SCIP. **Batch-native:** POUNCE.

### 6.4 Results and observation plane

This is [F04](#f04) and [F11](#f11).

**The attempt record carries:**
- identities: attempt, point, chain, revision, pattern key, program keys, profile and version;
- a feature reference;
- lane and placement context;
- advice used;
- start provenance and reuse kind;
- the T0 summary;
- a bounded T1 trajectory;
- `inertia_mode`;
- outcome and typed cause;
- independent verification;
- phase timings;
- environment identity.

**Analytics are DataFusion queries.** Study summaries, convergence-effort regression, history and efficiency telemetry all run over the journal and the record.

### 6.5 Determinism class: one owner

A study or solve declares its determinism class: bitwise, tolerance or statistical. The solve-plan owner enforces which context choices that class permits:
- retained numeric state;
- warm-start edges;
- inner library width;
- parallel structure execution;
- KKT solver choice;
- multi-core Symbolica optimization.

A point is re-run with its recorded context.

### 6.6 Method choice

This is [F09](#f09)–[F13](#f13); see the [solver companion](../evidence/efficiency-and-solver-target-2026-10-12/solver-target.md).

### 6.7 Answers to the seven ownership questions

1. **What owns a solve attempt's lifetime and its record?**
   - The lane owns native state.
   - Typed evidence lives in memory.
   - The attempt record is journaled always and sealed on publish.
   - A durable study appends a completion fact to its single-writer log.
2. **What owns the structure-lifetime facts that method choice consumes, and how are they invalidated?**
   - Salsa structure-plane products: pattern facts by pattern, form facts by body.
   - Invalidation is by Salsa dependency with component backdating.
3. **What owns block execution, its sessions and per-block method choice, and how does it parallelize?**
   - The block-plan sequencer, as bounded composition (PS-09), inside the lane.
   - Level-synchronous wavefronts run on the lane team under the determinism class.
4. **What owns reuse across study points?**
   - Lanes own allocation reuse.
   - Declared chains and session start policies own warm state.
   - History is a derived aggregate whose snapshot is recorded.
   - Durable seeds are journaled or published immutable artifacts.
5. **What owns stopping a trajectory?**
   - One `StopRequest` (cancel, deadline, policy) with declared fidelity per adapter.
   - `CancellationToken` trees feed it.
   - Drain is the lane join.
6. **What owns evidence retained for diagnosis and replay, and its release?**
   - The journal, with inputs, plan, advice snapshot and start.
   - Publication seals it.
   - Retention pins govern its release.
7. **What owns capacity across case, structure and library levels?**
   - One per-process budget split into lanes by study policy.
   - Shared native resources decide placement.
   - Inner widths are fixed per profile.
   - The analytics pool is separate.

## 7. Foundations and gates (slot 6)

| Foundation | Verdict | Scenario and evidence |
|---|---|---|
| AP-01 Separation of concerns | **Violated** | Retention, fences, leases, settlement, drain and receipts are each implemented in several owners: runtime, operations, engine, backend, SDK patch and scripts. Admission merges three responsibilities (semantic, capacity, placement) under one name across those layers (W-C §2–§3, X1 O3). Method-choice policy is spread across routing, effect binding, adapters and workflows |
| AP-02 Stable contracts | **Satisfied** at the adapter seam | Shared native resources are missing from the capability contract (AP-05) |
| AP-03 Composition | **Violated** | Method composition is fixed statement order. The in-process and durable executors differ in reuse, batching and warm state for the same study semantics |
| AP-04 Domain model and authority | **Violated** (adequacy) | Method choice has no adequate model ([F09](#f09)). The reuse authority is placed in persisted receipts beside several caches ([F05](#f05)) |
| AP-05 Explicit structure | **Violated** | No fact-versus-coordination classification ([F02](#f02)). Shared native resources hidden in adapter locks ([F08](#f08)). No declared determinism class ([F07](#f07)). Route records without reasons ([F11](#f11)) |
| AP-06 Local reasoning and testability | **Violated** | No runtime exists without the store ([F01](#f01)). Composition and escalation read the global `LINKED` |
| AP-07 Execution fit | **Violated** | [F01](#f01)–[F04](#f04), [F07](#f07) and [F08](#f08), all qualitative and none measured |
| **G9** | **Fail** | Without averaging |

| Gate | Judgment | Evidence |
|---|---|---|
| G1 Authority | Pass, with a warning | Several reuse authorities for derived products ([F05](#f05)) |
| G2 Semantic fidelity | **Fail (narrow)** | Typed iteration counts ([F11](#f11)) |
| G3 Validity | Pass | |
| G4 Hidden behaviour | Pass | |
| G5 Consistency and recovery | Pass | Strong |
| G6 Transformation and reuse | Pass, with forward conditions | Warm state declared; context-independent method choice; KINSOL stale-factor reuse resolved under a determinism class |
| G7 Truthful capability claims | **Fail (narrow)** | See [F11](#f11) |
| G8 Library leverage | Pass, with a warning | The hand-built zone map and the reference-protection bookkeeping exist because placement rules keep native capabilities from applying ([F02](#f02)/[F04](#f04)) |
| PS-G1 | Not applicable | No physical semantics change |
| PS-G2 | Pass | |
| PS-G3 | Pass | |
| PS-08 (MUST) | **Not met** | [F10](#f10) |
| PS-09 | Partially met | [F09](#f09) |
| PS-10 | Partially met | [F11](#f11), [F12](#f12) |
| PS-11 | Met today. The target must keep warm state on declared edges ([F07](#f07)) | |

## 8. Library fit, alternatives and tradeoffs (slots 8, 9)

### 8.1 Library ownership

From R1–R5 and Part B, with X2's corrections applied.

| Responsibility | Library owner in the target | Project keeps |
|---|---|---|
| Pure reuse and invalidation | Salsa 0.28, with byte-bounded retention keyed by Salsa ids | Durable content identities; the bundle freeze; lineage coexistence |
| Structure | pounce-presolve, KLU BTF, petgraph/rustworkx `layers`, faer symbolic factorizations for project factorizations | One producer per fact; orderings handed to libraries |
| Evaluation | Symbolica/Numerica evaluators with parameters as inputs; SIMD lanes where guards agree; JIT only after qualification (FMA, `symjit.toml`) | Guard stages; determinism class per backend |
| Journal and record | Arrow IPC/Parquet, object_store with explicit fsync, and an embedded catalog or a manifest log | Occurrence identities, staging ownership and fences, retention, backup contract |
| In-process lifetimes | tokio-util `TaskTracker`/`CancellationToken`; rayon `build_scoped` | Native checkpoint polling; owner-thread teardown |
| Child-process lanes | std process, pipes and Arrow IPC streams; pidfd for death | Dispatch and owner-mediated commits |
| Analytics | DataFusion over manifests and journal; statrs | Result and attempt schemas |
| Phase timing | tracing with a typed layer | The attempt schema |
| Native execution | As in the [solver companion §5](../evidence/efficiency-and-solver-target-2026-10-12/solver-target.md#methods) | Bounded composition only (PS-09) |

**Upgrades with material effect:**
- **salsa 0.28.5:** guards unvalidated access; `memory_usage` needs `&mut`.
- **feos 0.10.2:** fixes a convergence check and a transposed `PcSaftBinary` association Jacobian, which changes the `xtask` oracle numbers.
- **FERAL 0.19:** unreviewed.
- **SUNDIALS ≥ 7.3:** fixed-point damping and depth callbacks, if wanted.
- **Arrow 60:** waits for a DataFusion release on Arrow 60, and is a family major that needs an ADR.

### 8.2 Alternatives

| Alternative | Scenarios and locality | Machinery | Selection and revisit condition |
|---|---|---|---|
| Current baseline | All supported, at amplified cost | Largest | Rejected (G9) |
| Plan 36 target (protocol refinement) | Narrows conflict domains; one engagement; demand-shaped publication | Keeps the store as working authority and peer writers | Numerical, evaluation, build and test packages retained (§10.2); store-protocol packages superseded where the obligation disappears |
| **Live model + journal + immutable record, one writer per durable log, process lanes (recommended)** | All interactive and in-process scenarios are local. Durable studies resume from their own log. Crash isolation is kept | Removes per-call store traffic, protection churn, settlement and receipt protocols for immutable facts, the server and its supervision, the WebSocket transport patch, connection lifetimes, network unknown outcomes, per-point claim transactions and the second reuse authority | Revisit if premise 1 holds; then keep shared coordination for that state only |
| Same, with the remote SurrealDB server kept for durable studies | As above for interactive work. Durable studies keep peer workers | Keeps the transport, supervision and claim protocols, which store-native constraints simplify (R1 A) | Choose if premise 1 holds, or if independent worker processes must outlive the runner |
| SurrealDB-native lifecycle (R1 A) | Durable invariants enforced by the store | Removes bespoke reference and transition protocols; the store stays on the critical path | Adopted inside whichever shared-coordination tier remains |
| Simplest viable | The interactive path stops rechecking and stops opening the store; Python reads retain batches; recycle history | Least | A first step if RC01 or RC05 is rejected; leaves [F02](#f02) and [F03](#f03) |

**Reference practice, read for behaviour only.**
- Equation-oriented tools keep a resident model and push values to persistent solver interfaces.
- Sequential-modular tools re-solve affected blocks and reuse tear guesses.
- IDAES runs block-triangularization initialization, then a simultaneous solve.

None of this is evidence about this codebase's cost.

## 9. Verification and evidence limits (slot 10)

| Claim | Label | Basis | Gap |
|---|---|---|---|
| Store work on every solve; the store is opened even when ephemeral | Interface-checked | W-A; principal and X1 re-reads | Round trips counted from code structure |
| Coordination obligations are not consumed by interactive or ephemeral work; durability is consumed by history APIs | Interface-checked | W-B (a); X1 §3 | Premise 1 |
| Per-call sessions; per-call transient-fit rebuild; thread-history-dependent KINSOL factor | Interface-checked | W-A, W-B, X2 C | — |
| Ipopt and SUNDIALS repeat symbolic analysis per solve at the pins | Interface-checked | X2 facts A and B (pinned sources) | Corrects prior solver evidence B1–B3 and R4 Part B |
| Shared native resources (MUMPS, PETSc, HiGHS scheduler) | Interface-checked | R4, R5, X1 | Contention unmeasured |
| Solver mechanisms unchanged since 2026-10-12 | Interface-checked | Diff `ff5a4aa2f..6c368c88a` | — |
| Machinery history and failure classification | Interface-checked (history) | W-C | Untyped commits not classified |
| Every benefit | Proposed | Library contracts and reasoning | No measurement |

**Premises that could change a recommendation:**
1. **Does any product feature need several independent processes to mutate one project?** This covers several runtimes per store, and studies whose workers must outlive their runner. It decides whether a server and shared coordination remain ([F02](#f02), [F03](#f03), [RC05](#rc05)).
2. **Must unpublished interactive solves be durable,** beyond the journal ([RC01](#rc01))?
3. **What determinism class do studies require?** This decides how much warm state and parallelism a default study may use.
4. **Can authored parameters be evaluator inputs for every body class?**
5. **Is Symbolica optimization deterministic with more than one core?**
6. **Can a SUNDIALS context migrate between threads sequentially?** Until it is settled, structure tasks are assigned statically.
7. **Does the Ipopt compatibility stamp include the lb = ub set** once sessions are retained across bound edits?
8. **Do units inside causal recycle maps warm-start internally?** Anderson requires G to be a function of x.
9. **Block and SCC size distributions and wavefront widths** in real flowsheets.
10. **The SPRAL and Pardiso analysis cost per solve at small KKT sizes,** which is repeated on every solve (X2 fact A).

## 10. Rule impacts and disposition (slot 11)

### 10.1 Rule impacts

None of these takes effect until the operator confirms it in plan creation. Routes follow the binding.

| ID | Rule and where recorded | Proposed change | Findings | If kept |
|---|---|---|---|---|
| <a id="rc01"></a>RC01 | ADR-0164 (proposed) "Normal runs retain scientific outcomes"; §19/§21 durable by default; scope table "Canonical store availability"; the `pse-py` "sources live only in the canonical store" | The live model is the working authority. Authored input is journaled eagerly. Execution records go to the journal; publication seals them. A runtime without publication demand does not open the store. **Consumers whose meaning changes:** `latest_results`, `results(run…)`, `promote_seed_artifact`, `promote_scientific_artifact`, `result_analysis`, `reclaim_run_results`, which see published or journaled runs only | F01, F04 | Every interactive call keeps its store traffic; only rechecks can be trimmed |
| <a id="rc02"></a>RC02 | §14.3 protection through preparation; §14.4 "persisted complete dependencies govern reuse … Salsa is an optional local accelerator" and the guarded store check; ADR-0178 (proposed) | Salsa is the in-process reuse authority, with coexisting lineages. Persisted products are a content-keyed memo loaded as a trust transition | F05, F01 | Reuse costs store round trips per call |
| <a id="rc03"></a>RC03 | §20.1/§20.2 results as bounded IPC blocks in the store; ADR-0180; Plan 36 RC08/RC09 | The catalog holds authority facts and manifests. Bulk data and observation go to journal and sealed Parquet. Backup contract for the split layout | F04 | DataFusion provider over the blocks; the amplification stays |
| <a id="rc04"></a>RC04 | §20.6 protections, read protections, leases, heartbeats, settlement everywhere; ADR-0177, ADR-0182 (proposed); Plan 36 RC06/RC12; SurrealDB I1–I7 | Classify fact versus coordination. Immutable facts with minted occurrence ids and content-keyed payloads. Coordination only for multi-writer state, via store-native constraints. Owner-scoped staging with a fenced sweep | F02 | Protocol surface stays; consolidation only |
| <a id="rc05"></a>RC05 | ADR-0164 Options/Outcome (supervised remote server, WebSocket client); Plan 30's WebSocket and persistent-service decisions; ADR-0181 (proposed); §20.6 deployment; the substrate guide's server lifecycle | One writer per durable log. Runner-owned durable studies with child workers over pipes. Embedded catalog or manifest log. Non-durable process lanes. The server is kept only if premise 1 holds | F03 | The transport, supervision and connection-lifetime machinery stays |
| <a id="rc06"></a>RC06 | Run launcher and study executor session-per-run; §13.5 "only attempts own native state"; thread-local KINSOL inner-session numeric reuse | Lanes own result-neutral allocation reuse across attempts. Warm state only along declared chains or session start policies. §17.6's separation **kept**. Reuse-after-terminal rules per profile | F07 | Allocation repaid per call; the thread-history dependence remains |
| <a id="rc07"></a>RC07 | §18.8 threading and capacity (three semaphores; ADR-0166 refusal at native entry; FERAL "global pool" wording); `MathPolicy.jobs = 4` | One budget split into lanes. Shared native resources with access modes decide placement. Inner widths fixed per profile. Lanes re-form between points | F08 | Case-only parallelism |
| <a id="rc08"></a>RC08 | §14.2 and the `pse-engine` role: a DataFusion session for validation and admission | Compiled validation expressions; the planner kept for analytics | F14 | The planner stays in the validation path |
| <a id="rc09"></a>RC09 | ADR-0164 executable-mapping observation for deployment-local reuse | Narrowed to what a content-keyed memo needs | F14, F05 | The loader-lock observation stays |
| <a id="rc10"></a>RC10 | Solver-review RC01–RC13 (unconfirmed) | Carried, amended in [companion §7](../evidence/efficiency-and-solver-target-2026-10-12/solver-target.md#rule-impacts). RC05 (HSL) **withdrawn**; parity becomes solution-level | F09–F13 | As in that review |
| <a id="rc11"></a>RC11 | Plan 36 accepted RC03, RC04 and RC05 (engagement, receiving qualification, native-generation retention) | Superseded where the in-process authority removes the obligation; retained for any durable tier that needs them | F01, F05, F14 | Plan 36 store-protocol packets continue |

### 10.2 Plan 36 packets against the target

This classifies packets for plan creation; it does not edit Plan 36.

| Class | Packets |
|---|---|
| Implemented or in flight, **retained** | P-A1 typed bindings; P-A3 one declared preparation; P-A7 retained Salsa lane (extended by [F05](#f05)); P-C1–P-C3, P-C5–P-C7 and P-C9 numerical layouts and evaluation; P-D2 carry admission; P-D5 (complete); P-D6 build provenance; P-E2, P-E3 and P-E4 test and build boundaries |
| In flight, **redirected** | P-A2: edits become lineage input changes; the CAS stays for heads on publish. P-A5: the engagement becomes the lane-held bundle. P-B3: becomes lanes and placement. P-B6: telemetry becomes the attempt record. P-B8: seeds become journaled or published artifacts along declared edges. P-C4: the block sequence is extended to per-block plans. P-C8: IBEX admission becomes a shared-resource placement. P-D1: the validation owner becomes compiled expressions. P-D3: transport is narrowed to any remaining shared tier |
| **Superseded** where RC01–RC05 are confirmed | P-A4 receipts and acknowledgments (replaced by occurrence and content identities); P-A6 receiving qualification sharing; P-B1 exact conflict domains; P-B2 readiness; P-B4 row locators; P-B5 acknowledgment accumulator; P-B7 residue collection; P-D4 lazy file-cache capacity |
| Outside product scope; unchanged | P-D7, P-E1, P-E5, P-E6 |

### 10.3 Lead mapping

| Leads | Disposition under this review |
|---|---|
| Efficiency F01, F03, F06 | Subsumed by [F01](#f01)/[F05](#f05) |
| Efficiency F02, F04, F07 | Subsumed by [F05](#f05) |
| Efficiency F05, F23 | Subsumed by [F14](#f14) |
| Efficiency F08–F10, F12, F15 | Subsumed by [F02](#f02), [F03](#f03), [F08](#f08) |
| Efficiency F11, F13, F14 | Subsumed by [F04](#f04)/[F11](#f11) |
| Efficiency F16–F19, F21, F22 | **Stand** under retained Plan 36 packets |
| Efficiency F20 | Stands (R36-01) |
| Efficiency F24 | Subsumed by [F14](#f14)/[F04](#f04) |
| Efficiency F25–F28 | Outside product scope, except F26/F27, which [F01](#f01) eases |
| Efficiency F29 | Resolved |
| Audit A01–A02 | Seeds along declared edges ([F07](#f07)) |
| Audit A03–A05 | Stand (P-C8, P-C9) |
| Plan 28 PE01–PE06, Parallel F01–F05, PA01–PA04 | Implemented. Their open E3/E4/E5 qualification should be re-scoped to this target, not resumed |
| Plan 37 F01–F07 | Resolved. The artifact model and occurrence identities are kept |
| Solver-review F01–F13 | Subsumed by [F09](#f09)–[F13](#f13) and the companion |
| Plan 32 | Untouched |

### 10.4 Disposition

**Not yet scheduled.** The proposed owner is a revised plan series created through `plan-creation`. It:
- presents RC01–RC11 and premises 1–3 to the operator;
- takes over the retained and redirected Plan 36 packets;
- records each finding as `finding | scenario | disposition | owner | evidence or trigger`;
- orders work by §5;
- lands the behavioural fixes first.

## 11. Reconciliation with the independent challenges

Both challenges were run in fresh context against the drafts.

**X1, omission and preservation.** Adopted in substance:
- **O1, topology.** It is a separate cause, now [F03](#f03). The no-server alternative is compared, and with premise 1 as its revisit condition it is now the recommended direction. Non-durable process lanes are added.
- **O2, a missing tier.** The local journal tier is added: authored input, attempt records, replay bundles, large results.
- **O3, mislabelled causes.** Admission is split into semantic, capacity and placement. The failure attribution is three-way.
- **O4, determinism.** A determinism-class owner is added (§6.5).
- **Preservation conditions.** All are adopted:
  - occurrence versus content identity;
  - owner-scoped staging and a fenced sweep;
  - claim fences in commits;
  - the weakened protected read stated explicitly;
  - the backup contract;
  - coexisting lineages and structural studies;
  - the disk memo as a trust transition;
  - pattern facts kept advisory;
  - reuse-after-terminal rules and bounded sessions;
  - placement not refusal for call-serialized MUMPS;
  - the shared-resource model;
  - recorded study profiles;
  - lane re-formation;
  - the FERAL width;
  - decode re-validation as a real trust transition.
- **Corrections to the synthesis.** Its overstatements are corrected:
  - "default durable runtime";
  - "largest identifiable share";
  - "no cross-process coordination obligation".

**X2, numerical methods.** Adopted in substance. The solver companion carries the detail:
- **Warm reuse.** It is now allocation reuse plus declared warm edges. Facts A–C (per-solve symbolic analysis in Ipopt and SUNDIALS; KINSOL stale numeric factors) remove the draft's claimed symbolic-reuse benefit for those libraries.
- **Per-block rules.** These are corrected: branch, Brent, `block_solve`, Krylov within an SCC, NLEQ-ERR, and acceptance with a conditional polish.
- **Parallel equivalence.** Its conditions are adopted.
- **KKT choice.** It is reframed as placement, not method. The facts on Pardiso and the curvature test, FERAL `n_tiny`, and the loss of NLP scaling are added.
- **Recycles.** Corrected: map-callback history, damped-window diagnosis, the Anderson restart rule, and the successor by representation.
- **Transient fits.** Corrected: what reinitialization saves, Diffsol scope, and the joint algorithm-and-derivative rule.
- **History.** Corrected: censoring, conditioning, and replay from the recorded permutation.
- **Stop-fidelity table.** Corrected.
- **Form facts.** Keyed by body, not pattern.
- **Symbolic factorizations.** Only for project factorizations.
- **PETSc.** Process lanes do not parallelize NASM subdomains.

**Corrections to prior evidence.** X2 facts A and B contradict claims in the 2026-10-12 solver evidence (B1 on `ReOptimizeTNLP` symbolic retention; B2/B3 on KLU retention across KINSOL calls and `IDAReInit`) and in R4 Part B. Those documents are not edited; this review records the correction.

**Remaining disagreement.** None on direction. X1 recommends comparing the no-server alternative; this review goes further and recommends it, conditional on premise 1. That is a judgment the operator confirms through [RC05](#rc05).

## 12. Decision and priorities (slot 12)

- **Behavioural adequacy: Revise (narrow).**
- **Architectural fitness: Fails G9.**
- **Overall: Revise.**
- **Strongest evidence:**
  - the interactive ledger traced from the Python call;
  - the product-concurrency inventory;
  - the provenance of the transport and supervision machinery;
  - library contracts showing that allocation reuse, structure parallelism and analytics are available at the pins;
  - the clean-sheet allocation reaching the same boundaries without reading the code;
  - two independent challenges.
- **Main uncertainty:** premise 1. It changes how much shared coordination and server machinery remains, not the direction.

| Priority | Change | Findings | Acceptance evidence | Owner |
|---|---|---|---|---|
| 1 | Recycle iteration history from the map callback; damped and Anderson defaults with restart | F10 | A non-converging recycle's history is answered from the record (PS-08) | New plan |
| 1 | G7/G2 corrections; HiGHS basis; SCIP edge removed; typed iterations; no stale-factor reuse across independent points | F11, F09 (part), F07 (part) | Contracts and records agree with execution | New plan |
| 2 | State and writer boundary: live model, journal, immutable record with occurrence identities; one writer per durable log; process lanes | F01–F03 | A value edit performs no store operation; a durable study resumes without a server | New plan, after RC01, RC04 and RC05 |
| 2 | Determinism-class owner | F07, F08 | Lane count does not change point results within the class | New plan |
| 2 | Salsa single reuse authority with coexisting lineages; structure plane | F05, F06 | One retention owner per product kind | New plan |
| 2 | Lanes with allocation reuse and declared warm edges; shared-resource placement; fixed inner widths | F07, F08 | Allocation-free second solve; ephemeral process lanes | New plan, after RC06 and RC07 |
| 3 | Journal and observation plane; attempt record; analytics | F04, F11 | Study summary and convergence-effort as queries | New plan, after RC03 |
| 3 | Solve plan, transitions, cooperative stop, per-block plans, history layer | F09, F11 | See the companion | New plan |
| 3 | Diagnosis tier and replay; methods bound | F12, F13 | Exhausted solves yield located findings and replay bundles | New plan |
| 4 | Compiled validation; engine narrowed | F14 | One validation per trust transition | New plan |

**Next consequential decision:** operator confirmation of [RC01](#rc01)–[RC05](#rc05), the state and writer boundary, together with premises 1–3.
