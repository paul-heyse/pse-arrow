> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Challenge X1 (design-reviewer): omission and preservation challenge of the first draft. Finding numbers it cites follow the first draft; see the README mapping. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# X1: omission and preservation challenge

This is a supporting contribution to the consolidated efficiency and solver target review. It is an independent challenge of `principal-draft.md` and `solver-companion-draft.md`, not a parallel review. It has no verdict, finding IDs or decision. The coordinator owns integration.

- **Baseline.** `main` at `6c368c88a`, clean tree.
- **Standard.** Core 3.4 (§1, AP-01–AP-07, DP-09/10/11/18–21), heuristics H1–H28 and the safeguards, Process Simulator 1.5, and the pse-arrow binding (PSE-S01–S07).
- **What I read.** The common brief and the X1 brief. Both drafts. W-A, W-B and W-C. C1. R1 Part B. Targeted parts of R2, R4 and R5.
- **Source checked for this challenge** (*Interface-checked* unless stated otherwise):
  - `pse-py/src/workflow.rs::NativeRuntime::new`;
  - `pse-runtime/src/workflow/modeling/canonical.rs::declaration`;
  - `declared.rs::declared_execution_with_demand`;
  - `run.rs::launch`;
  - `integrated.rs::integrate`;
  - `study_execution.rs` (the retention rule);
  - `results.rs::table_ids`;
  - `pse-backend-native/src/{petsc.rs, highs_lifecycle.rs, pounce.rs, ipopt/settings.rs, settings/ipopt.rs}`;
  - Ipopt 3.14.20 `IpMumpsSolverInterface.cpp` and its ChangeLog;
  - FERAL 0.18.0 `numeric/solver.rs`;
  - `python/pse/_workflow.py` and `_modeling.py`;
  - ADR-0182 Outcome;
  - numerical-execution §16.5/§17.6;
  - identity-and-publication §20.3.
- **What I did not do.** No builds, tests, probes or measurements. Every remedy below is *Proposed*.

**Position in one paragraph.** The draft's direction holds: a live working model, an immutable durable record, lanes and one observation record. Its biggest gaps are not in that direction but around it:

- It keeps the costliest topology "for now" without stating what that keeps.
- Its state boundary has only two tiers, "in memory" and "published", and so has no place for three things that matter: crash-safe authored edits, large unpublished results, and failure evidence for diagnosis.
- Several remedies erase distinctions the current design deliberately makes:
  - occurrence identity versus content identity;
  - retained setup versus retained iterate;
  - call-serialized versus process-exclusive libraries.
- Three synthesis-level claims are overstated.

---

## 1. Omissions

### O1. Topology is a separate cause, and the draft drops the simplest complete alternative

**Facts.**

- The draft attributes the lifecycle mass to mutability (Cause A, F02) and calls the deployment axis "secondary" (§6.1). It then keeps the remote SurrealDB server for the durable tier, because "durable-study workers are separate writer processes" (§6.1, §8.2).
- W-C, however, shows a different spread of machinery. Much of it exists because the deployment is a set of peer processes, a separately supervised server, on-demand managed workers and Python clients, all talking over WebSocket. It does not exist because state is mutable. W-C counts:
  - the vendored SDK request-lifecycle patch: bounded admissions, replay and drain;
  - the ADR-0181 caller, activity and physical lifetimes;
  - `surreal_server.py` (7.5k lines);
  - `host_admission.py` (1.8k);
  - `native_operation.py` (2k);
  - receiver generations;
  - observer pids, systemd units and cgroup placement proofs.
- Plans 29, 30, 35 and 37 were largely spent there (W-C §5). In W-C §7, attempts 10, 14, 18, 22, 24, 26, 29, 31 and 33 concern placement, drain, receivers, fixtures borrowing host allocations, or SDK races. None of them concerns shared mutable scientific state.

**Interpretation.**

- Classifying durable state as fact or coordination (F02) does not remove this machinery while the Python process remains a remote WebSocket writer for publication, and while workers remain peer writers. The draft's verification ("one owner per lifecycle concept") would pass with all of it still in place.
- The premise "workers are separate writer processes" is a design choice, not a constraint. Native-crash isolation and process-exclusive libraries require separate *processes*. They do not require separate *writers*.
- C1 (allocation L, variant P, open premise 7) named the alternative that the draft's §8.2 omits: no database server at all.

**The omitted alternative: a single writer per log, with no server.** *Proposed* for comparison.

- **Authored revisions.** Content-addressed files, plus heads updated by `flock` and atomic rename.
- **Results and attempt logs.** Immutable Parquet with per-writer manifest logs, fsync followed by rename. DataFusion `ListingTable` serves readers over committed manifests.
- **A durable study.** It has exactly one owner, a detached runner. The runner dispatches points to child worker processes over pipes, which carry Arrow IPC. Completion facts go to the study's own log, and resume means reading that log.
- **Retention.** An exclusive project lock, or catalog-first deletion followed by a grace period and a typed `ResultSetRetired`.
- **Libraries.** All are already pinned: Arrow IPC, Parquet, DataFusion, object_store, tokio-util and std `fs`.

**What it would remove.** The server and its supervision, the WebSocket transport patch, connection lifetimes, network unknown outcomes, the host ledger for the store service, and per-point claim transactions.

**What it loses:**
- concurrent mutation of one study by several independent processes;
- multi-host operation;
- SurrealQL graph traversal over history (DataFusion SQL replaces most of it);
- the investment in Plans 28, 30 and 37.

It is a rule impact on ADR-0164 and §20.

**What the draft must add.**
- Compare this alternative in §8.2, with its revisit condition. That condition is the draft's own premise 1: a product feature in which several independent processes mutate one project.
- State which machinery each topology keeps.

Even if the remote server stays, F07 needs a process lane that is **not** durable. The draft admits a process-exclusive family under multi-lane only "on a serial lane or a process lane" (F07, companion §3.1). Process lanes exist only in the durable tier (§6.1, §6.3). An ephemeral 16-case study whose plan prefers PETSc QN or NGMRES for recycles therefore has two routes:
- run serially on one lane, which is an H11 problem for ordinary supported work;
- become durable, which contradicts F01.

So a child-process lane that returns Arrow over a pipe, with commits mediated by the owner and no store, is a prerequisite of F07. It is not a later variant.

### O2. The state boundary has no tier for local, file-backed, unpublished state

The draft has two tiers: "Arrow in memory first" and "published as a sink" (F01, F03, §6.1). Three workloads or requirements need a third tier, local and file-backed but not published.

1. **Large unpublished results: H11 and H17.** Consider three cases:
   - a long trajectory with sensitivities, run interactively;
   - a thousand-point ephemeral sweep at full detail;
   - a fit with many observations.

   Unpublished, each stays wholly in memory. Streaming row groups (F03) applies only to publication. Today's in-memory `RunResult` already shows the problem (W-B 4a). The draft moves it from durable runs to every unpublished run.
2. **Diagnosis, replay and history: maintainer requirement and F10/F11.** Interactive work is where failures are diagnosed most often, yet:
   - replay uses "published inputs plus the recorded plan" (F11, companion §3.6);
   - the history layer is "a DataFusion aggregate over the *published* attempt log" (companion §3.5).

   Under F01, an unpublished interactive failure has no replay bundle. The history layer also never learns from interactive use, which is its richest source, so its sample is biased towards durable studies.
3. **Authored input under a native crash.** See the state-boundary challenge in §2. Authored edits lose crash durability.

**What the draft must add.** *Proposed.*
- A local journal tier per project, append-only and file-backed. It keeps:
  - **authored revisions**, eagerly, because they are small;
  - **attempt T0 records and failure replay bundles**, always;
  - **large results**, spilled as Arrow IPC or Parquet scratch under bounded live retention.
- Publication then *seals* files that already exist, with one catalog commit, instead of encoding them.
- This one tier serves H11, diagnosis-as-product, history and crash recovery of input. It also makes "publish later" cheap.

### O3. Several causes are mislabelled, which over-credits F02

- **"Admission in at least seven places"** (F02 evidence, AP-01) counts at least three different responsibilities:
  - semantic plan admission: `pse-engine/session/admission.rs`, `pse-admission`;
  - capacity admission: `TaskAdmission`, `ComputeAdmission`, the math semaphores;
  - deployment and host placement admission: the SDK request admission, `host_admission.py`, `canonical_lifetime.rs`.

  These have different owners and different remedies: F13, F07 and O1 respectively. Presenting them as one lifecycle concept duplicated across layers invites a single consolidation that would merge unrelated reasons for change (AP-01).
- **Capacity and placement failures belong to Cause C (F07) and to O1, not to Cause A.** W-C §7 rows 18, 21, 24, 31 and 33 are charging, clocks, placement widening and stack exhaustion.
- **Consequence.** F02's verification and the §5 prerequisite order should not claim that the W-C failure profile follows from classifying state. The draft should split the attribution three ways: shared mutable state, topology and transport, and capacity ownership.

### O4. Nothing owns the determinism and equivalence contract that the new parallel remedies change

The target adds several sources of order or context dependence:
- lane warm starts across study points (F06);
- a KKT linear solver chosen by lane context (F07, companion §3.2);
- wavefront parallel blocks and parallel shooting and fit reductions (F07, companion §3.7);
- KLU refactor pivot reuse;
- FERAL `parallel`;
- multi-core Symbolica optimization (premise 5).

DP-11 requires the accuracy and equivalence class to be stated, along with "when hardware, backend or thread count can change results". The draft's guards are local:
- "deterministic reduction order";
- "one history snapshot per study, so point results do not depend on completion order".

F06 then reintroduces the very completion-order dependence that the history rule removes; see §2. The draft needs one owner, at the study and solve-plan level, for each of the following:
- the declared determinism class: bitwise, tolerance or statistical;
- which retained-state and context choices that class permits;
- how a single point is re-run with its recorded context.

---

## 2. Preservation challenges

Each row gives a revealing legitimate case, the guarantee at risk, and what the remedy must add. All remedies are *Proposed*.

| Remedy | Revealing legitimate case | Guarantee at risk | The remedy must add |
|---|---|---|---|
| **State boundary** (live model as working authority) | An engineer makes an hour of API edits (`with_delta`, `with_declarations`; `python/pse/_modeling.py` 726–742), then a native solver segfaults in the in-process lane. W-B says a native crash takes the process, and in-process lanes run in the Python process. Today every edit is persisted before solving (W-A 1.7, 3.5) | Crash recovery of user-authored input; AS4 failure domains | Separate **input durability** from **execution-record durability**. Authored revisions are small and content-addressed, so they are journaled eagerly and off the solve path (O2). Only execution records become optional |
| State boundary: heads | A durable worker, a second notebook or a CLI reads "project X current" while the session holds unpublished edits | AP-04: one owner per meaning ("current revision") | Define it like a working tree versus HEAD. Named heads in the record are the authority across processes. The session's live model is the authority only for its own working copy. Heads move only by CAS on publish |
| **F01** (publication as an explicit sink) | A published result whose start was warm from an *unpublished* interactive predecessor. Lineage identity already frames "predecessor attempt" and seed content (numerical-execution §16.5, `pse.completed.request.v2`) | PS-11 and DP-21 reproducibility of every *published* result, which the draft claims to preserve | Publishing a result also publishes the content of its used start (`WarmStart::content_key` already excludes the producing run) and its retained-state kind. A predecessor that is neither published nor journaled is recorded as such, never as a dangling reference |
| F01 | `latest_results(problem, …)`, `results(run, …)`, `promote_seed_artifact`, `promote_scientific_artifact(run, attempt, …)`, `result_analysis(run, attempt, …)` and `reclaim_run_results` (`_workflow.py` 145–235) applied to interactive solves | The meaning of existing product features | State explicitly that these see only published or journaled runs. Today they see every durable-runtime solve. Include this in RC01's "if kept" and in its consumer list |
| **F02** (content-addressed facts settle themselves) | A user reclaims a run's results, then re-runs the same deterministic solve. Identical content gives the same address | ADR-0182: "Artifact occurrence identity is separate from scientific content"; "a retired study identity refuses even with a different live run"; the draft's own "no resurrection of retired identities" | Keep **minted occurrence identities** (publication, attempt, study), never reused, with permanent tombstones. Use **content keys only to deduplicate payloads**. "Self-settling retry" applies to a retry of the *same occurrence*. Re-execution is a new occurrence, and retirement fences occurrences, not content |
| F02 | Durable worker A has written and fsynced its temporary Parquet files for point 17. The Python owner restarts and runs "the orphan sweep at start" (F02 verification) and deletes them. A then commits a manifest that references the deleted files | Crash recovery for real concurrent writers; DP-19 ("advertised results cannot depend on uncommitted external payloads") | Writers stage under a directory owned by their claim or process identity. A sweep removes only directories whose owner is provably dead: pidfd or cgroup empty, or claim lease expired. The commit checks that same claim generation, and the sweep and the commit are fenced by one token |
| F02 | A worker whose claim was reclaimed, or whose study was cancelled or regenerated, finishes and appends its point-completion fact | DP-19: stale asynchronous results are rejected | Every durable-tier fact that depends on a claim checks the claim fence inside its own commit. "Settlement for claims only" (§8.1) is not enough. Attempt records of losing attempts are kept as attempts |
| F02 + F03 | A multi-page reader in another process while GC retires a result set. Files sit outside the store, so `REFERENCE … REJECT` cannot protect them | Today's protected-read guarantee; truthful partiality | State the weakened guarantee: catalog-first deletion, a grace period longer than the maximum read, and a typed `ResultSetRetired` mid-read. Do not imply that the current guarantee survives |
| **F03** (Parquet bulk plus catalog) | "Quiesce, stop and backup", a product obligation in W-B (a) | Consistent backup | A backup contract for the split layout: snapshot the catalog, then copy the immutable referenced files with GC paused. Orphans are harmless |
| F03 / H11 | A large unpublished result; see O2.1 | H11, H17 | A file-backed scratch tier with bounded live retention, independent of publication |
| **F04** (Salsa as the single authority; edits as input setters) | The Python API is value-semantic: `pkg2 = pkg.with_delta(...)` "retains the original revision". An engineer alternates solves of `pkg` and `pkg2`, or runs a study on A while editing B. One Salsa database holds one current input revision. The code already keeps compiler lanes per lineage and uses a *detached* workspace on an out-of-order stamp | Valid work (scenario comparison, design alternatives) is not penalized or serialized | Say how several live revisions coexist. Either use one database per live lineage, with sharing across lineages through the content-keyed memo, or key inputs by interned revision so that both coexist and backdate. "Edits are input setters" covers only linear editing |
| F04 | A structural or discrete study: a tray-count DOE, or fixed/free flips per point (W-A 2.5: a flip changes `bound_structure`) | Edit during a study (premise 3); feasibility | A study that must prepare during execution cannot freeze one bundle. Either prepare every distinct structure at admission or run the study on its own lineage database. Otherwise interactive edits block on the study's handles |
| F04 / F13 | A cold worker loads a compiled program, or later JIT IR, from the disk memo | DP-18: data never gains executable authority implicitly; H18 trust transition | The memo key includes the evaluator ABI, the library and symjit versions and the backend profile. Loading from another writer is a validated trust transition, not a silent hit |
| **F05** (pattern plane) | A block that is structurally affine but has a coefficient that is zero at the current values, or a conditional branch selected by value | PS-04/PS-05 distinctions; method choice by pattern | Pattern-lifetime facts must stay advisory for per-block method choice when value features contradict them, with a typed cause. Per-component backdating needs component identities that are stable across edits |
| **F06** (lanes keep sessions warm across same-lane points) | A thousand-point sweep of a nonconvex flash or reactor (multiple steady states) on 16 lanes pulling from one queue. The start of each point depends on which point that lane ran before, and that depends on scheduling. Two runs, or two lane counts, can reach different local solutions. The code deliberately gives fan-out successors "independent sessions"; only a single continuation successor inherits native state (`study_execution.rs` 365–366) | DP-11; PS-11 study reproducibility; the draft's own history rule ("results do not depend on completion order") | **Distinguish two kinds of retained state.** *Result-neutral retention* is structure-lifetime: symbolic analysis, allocations, TNLP or KINSOL setup. *Iterate warm state* is primal and dual values, μ, the basis, quasi-Newton memory, and possibly scaling or pivot order. Across independent points, the default is retention with a declared start. Iterate warm state is used only along declared continuation edges, or under an explicit study start policy that is recorded and deterministic, with a fixed partition and order. Each profile declares which retained state affects the result, and so its determinism class |
| F06 | The next point reuses a retained session after a cooperative stop, a restoration failure, a native error or a cancel | DP-19 owned workspaces; PS-10 | Each profile declares the terminal states after which a session may be reused. Every other state discards or resets the session. The current rule, "a disallowed start clears retained native start state, including HiGHS basis", generalizes into this |
| F06 | Interactive work across many patterns keeps sessions per pattern | H17 | A byte or entry bound on lane-retained native sessions, with eviction. Native heaps are opaque, so estimates come from pattern facts |
| **F07** (concurrency classes as admission facts) | A user explicitly selects `Linear::Mumps` (the default: `settings/ipopt.rs` `impl Default for Linear`) for a 16-case study, for parity with earlier results | Valid work is not rejected; H11 | Ipopt+MUMPS is **call-serialized**. Ipopt's `mumps_call_mutex` is a scoped `lock_guard` per init, factor and solve call (`IpMumpsSolverInterface.cpp` 49–52, 106, 240, 414, 479, 574, 671). Evaluations and line searches still run concurrently. It is not process-exclusive like PETSc, whose adapter holds the `PROCESS` guard for the whole admitted execution (`petsc.rs` 64–82). So add a "contended or call-serialized" class that is a *preference* input, never an admission refusal. Keep "process-exclusive" for PETSc and IBEX |
| F07 | Sixteen lanes run direct HiGHS LPs while one lane runs Uno with HiGHS QP subproblems | The concurrency class is complete and truthful (AP-05) | `highs_lifecycle.rs` shows direct HiGHS taking a shared `LIFECYCLE` read lock while Uno's HiGHS takes the write lock to reset the global scheduler. This is a cross-library exclusion that a per-profile enum cannot express. Model classes as **declared shared native resources with access modes**, such as `highs-scheduler` (shared or exclusive), `mumps` (call-serialized), `petsc-process` (exclusive) and OpenMP ICVs. Admission and preference reason over resource conflicts |
| F07 | One case is solved interactively alone, and the same case is solved in a 16-lane study. The KKT solver preference flips from MUMPS to SPRAL or Pardiso with lane context (companion §3.2) | DP-11; PS-10 consistency, since a case may fail in the study and pass interactively | Choose the profile at study admission from the study's lane plan. Record it. Re-run a point with its recorded profile, not with the live context, and allow pinning |
| F07 | A skewed study: one large SCC case among many small ones. Lane widths are fixed for the lane's life because HiGHS thread count is fixed per thread. The large case ends up as a width-1 straggler while 15 cores sit idle | H22, H11 | Allow lane re-formation between points, with new lane threads for wide work. Alternatively, admit the widest point first or schedule by pattern facts. A static n × w split at study start is not enough |
| F07 | A per-lane FERAL width | Library counts derived from the lane | FERAL 0.18 builds a `Solver`-owned pool sized from `RAYON_NUM_THREADS` or `available_parallelism`, which cannot be injected (`numeric/solver.rs` 658–705; `pounce.rs::feral_pool_threads`). State that FERAL runs serial unless the lane width equals the process-wide size. Otherwise it needs an upstream change |
| **F13** (carry validation; validate once) | The draft counts "re-validation of decoded blocks after the digest check" as rechecking an unchanged premise. It also preserves validation of "decoded bytes from another writer". In the durable tier most decoded bytes do come from another process, a worker | H18/H20: a digest is not semantic correctness | Record the writer's registry and validation-contract version in the manifest. Skip re-validation only when that version equals the reader's and the digest matches. The claimed saving then applies mainly to same-process reads, which the target serves from memory anyway, so it should not be listed as a material removal |

---

## 3. Diagnosis accuracy (spot checks)

| Claim | Verdict | Basis |
|---|---|---|
| Every `Runtime` opens the store, even when ephemeral | **Confirmed** | `NativeRuntime::new`: `CanonicalStore::connect(..)` then `.open()` run before the `ephemeral` branch, which only sets `DurabilitySelection::Ephemeral` (`pse-py/src/workflow.rs` ~701–768) |
| Per-solve store work: protect, read root, release; protected selection recheck | **Confirmed** for the mechanism | `canonical.rs::declaration` (1799–1826): `admit_activity`, `protect(rev, 3600 s)`, `object`, `finish`. It is called from `declared_execution_with_demand` (`declared.rs` 341–366) before `prepare`. "About twenty relations" is right: `results.rs::table_ids` lists 23 for modeling |
| Synthesis bullet "A value edit … registers a run, claims it and heartbeats it; exports about twenty relations" | **Overstated as worded** | True only for the default *durable* runtime. Ephemeral skips registration and export (`run.rs::attempt_for`; W-A §(a) item 6). F01's evidence states this correctly; the synthesis does not |
| "No product recovery obligation consumed by the interactive or ephemeral path" | **Overstated** | W-B (a) concluded this only for the *ephemeral sweep*. For interactive solves on a durable runtime, no *coordination* obligation (lease, heartbeat, claim) is consumed by a product feature. *Durability and lineage* are consumed by `latest_results`, `results(run, …)`, the artifact and seed promotion APIs, `result_analysis` and reclaim (`_workflow.py`). Recovery's job there is to give those readers a truthful terminal state. The correct claim: "no cross-process coordination obligation; durability is consumed by cross-session history features, which an explicit publish can serve" |
| Fresh native session per call and per point | **Confirmed, with scope** | `run.rs::launch` opens `Staged::open` or `open_queued` per launch. Sessions *are* retained within staged sequences, horizons, initialization stages and single-successor continuations (W-B (c); `study_execution.rs` 364–366). That is a deliberate distinction which F06 must keep; see §2 |
| Transient fits build a worker and an integrator per oracle call | **Confirmed** | `integrated.rs::integrate` (199–222) calls `self.program.worker(..)` and then `integrate_with_progress_observed` each time. Note that the worker is a workspace, not a recompilation |
| MUMPS "serializes every MUMPS call in the process" (§3.3) | **Correct** | Ipopt 3.14.20 has a scoped per-call `lock_guard`. ChangeLog: "Calls into MUMPS are now protected by a mutex". MUMPS is the default (`impl Default for Linear` → `Mumps{Metis}`) |
| Ipopt+MUMPS classed as "process-exclusive" (§6.3; F07 "a process mutex on the default NLP route") | **Wrong class** | It is call-serialized, not process-exclusive. "Sixteen NLP cases queue on one mutex" holds only for linear-algebra calls. PETSc is genuinely process-exclusive (`petsc.rs` `PROCESS` guard held by `Admission`) |
| HiGHS is "per-thread" | **Incomplete** | Direct HiGHS is concurrent under a shared read lock. Uno's HiGHS subproblems take an exclusive lock to reset the global scheduler (`highs_lifecycle.rs`) |
| FERAL builds a per-solver pool, so the "global pool" wording in §18.8 and R-41 is untrue (G7) | **Correct**; W-B's "own global pool" is wrong | FERAL 0.18 `Solver`-owned `ThreadPoolBuilder` (`numeric/solver.rs` ~658–705) |
| "That one choice explains most of the code mass and most of the failures" (§1 cause 2) | **Overstated** | By the draft's own §3.4 table, lifecycle is about 85k of about 320k hand-written non-test Rust lines: roughly a quarter, not most. Failed attempts: 16 of 34 are lifecycle, a plurality and not a majority (W-C §7). Fixes: 14 lifecycle + 1 transport out of 65, against 29 build or toolchain (W-C §6), so not "most of the fix churn". W-C also notes that scripts and the vendor tree are shared with agent tooling. The H27 signal stands, but the wording should be "the largest identifiable share" |
| "Admission implemented in at least seven places" as one duplicated concept | **Mislabelled** | See O3: three distinct responsibilities |

---

## 4. How these change the draft's argument

- **The direction survives.** None of the challenges argues for keeping the store as the working authority. Most of them add conditions that the remedies need in order to keep guarantees that the draft already says it preserves.
- **Prerequisites to add to §5:**
  - the local journal tier (O2), before or with F01;
  - occurrence identity versus content identity (F02), before F03's catalog;
  - the retention split between result-neutral state and iterate state, plus reuse-after-terminal rules, before F06;
  - shared-resource concurrency modelling and non-durable process lanes, before F07's admission rule;
  - one determinism-class owner (O4) across F06, F07 and the solver plan.
- **Additions to §9 premises:**
  - Does any product feature need several independent processes to mutate one project (O1)? This decides whether there is a server, not only "how much coordination".
  - Must API-made edits survive a native crash?
  - What determinism class is required of studies?
- **Additions to §10.1:**
  - the ADR-0164 topology impact (O1);
  - an RC01 consumer list that includes the history and artifact APIs (§3).

## 5. Coverage and limits

- **Static reading only.** Store round-trip counts are taken from code structure, as in W-A.
- **Not checked:**
  - MUMPS 5.9.1 `libseq` thread-safety beyond Ipopt's mutex; its ChangeLog is inconclusive, so no `IPOPT_MUMPS_NOMUTEX` lever is proposed;
  - whether Ipopt `ReOptimizeTNLP` retains scaling or quasi-Newton state, which X2 should settle for the F06 result-affecting declaration;
  - the topology alternative's handling of `analysis(key)` graph persistence.
- **Counts reused from the draft without recomputation.** The line counts are the draft's, which cites W-C plus a "preparation estimate" I could not trace. W-C reports 608k non-generated Rust lines including tests.
- **File written:** this file only.
