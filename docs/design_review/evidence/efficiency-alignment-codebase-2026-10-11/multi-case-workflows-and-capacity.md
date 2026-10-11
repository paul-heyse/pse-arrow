# Multi-case workflows and shared capacity: bounded supporting assessment

Inquiry **D** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`D-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

This is a bounded supporting assessment for the 2026-10-11 efficiency-alignment review. The baseline is `main` at `b01a45d71`. The standard is Core 3.4, Process Simulator 1.5 and the pse-arrow binding. The work was read-only: nothing was built, tested, measured or probed. Labels `D-nn` are local to this inquiry.

## 1. Scope examined and coverage limits

**Source read directly:**

- **Durable worker (`pse-runtime/src/workflow/worker.rs`):** `Runtime::serve`, `group_lane`, `work_once_scoped`, `CandidateGroup` and `ensure_managed_primary`.
- **Studies (`workflow/study.rs`):** `start_defined_study`, `prepare_study_candidate`, `apply_study_prediction`, `prediction_sample`, `rebind_prediction_source`, `record_study_attempt`, `recover_study_point`, the first half of `finalize_canonical_study`, `StudyHandle::terminal_inventory` and `admit_study_points`.
- **Study execution (`workflow/study_execution.rs`):** `study_durable`, `study_inner`, `execute_lane`, `continuation_start` and `scoped_action`.
- **Study operations:** `workflow/study_operations.rs::DeclaredStudyAdmission` and `StudyOperation::admit_binding_seed_need`; `workflow/bindings.rs::prepare_bound_operation`.
- **Preparation call sites:** `modeling/preparation.rs::settle_preparation`, `modeling.rs::prepare`, the two `prepare` call sites in `modeling/declared.rs` and `modeling/cases.rs`, and `modeling/engines.rs::prepare_analysis_attempt`.
- **Operations crate (`pse-operations`):**
  - `canonical_studies.rs`: `create_study` (head), `study_candidates`, `study_scope`, `claim_study_point`, `finish_study_claim`, `begin_study_finalization`, `study_assigned_page`, `observe_study_point` and `study_page`.
  - `study_policy.rs::action` and `may_retry`.
  - `canonical.rs`: `ProblemTurns`, `protected_query`, `protect` and `release`; `canonical_execution.rs::protected_execution_query`.
  - The generated `surreal.surql`: the `fn::pse_study_v1::*` functions, `fn::pse_execution_v1::{touch, available, writer, study_cancelled, append, renew, ...}` and the study indexes.
- **Capacity:**
  - Runtime: `env.rs::SharedRuntime::build`, `budget.rs`, `math.rs::{MathPolicy, MathService, reserve}`, `math/jobs.rs::{admit_entry, reserve_entry, admitted}` and `math/staged.rs::{open_session*, session_on_reserved}` with its scope-stack reservation.
  - Native backend: `pse-backend-native/src/mkl.rs::Threads`, `pounce.rs::with_threads` and `execution.rs::compute_scoped`.
  - Implicit blocks: `pse-math/src/implicit.rs::Problem::new`, checked against faer 0.24.4 `get_global_parallelism` in the registry source.
- **Fitting:** `fitting/oracle.rs::{FitOracle::new, evaluate_demand, objective, gradient}`, `fitting/profile.rs::{dispatch, profiles}`, `fitting.rs::profile_workers`, `run.rs` (fit `start_owned`) and `uncertainty.rs::propagate_checked`.
- **Horizon:** `horizon/driver.rs::{prepare_solve, execute_solve, estimate}`.
- **Deployment:** `scripts/surreal_server.py::{reference_execution, resources}`, the managed-primary command, and `packages/reference/conformance.toml [settings]`.
- **Prior evidence:** `docs/design_review/evidence/canonical-selection-2026-10-06/README.md`. It records an `IndexScan` on `study_candidates`, but on empty tables.

**Not read, or only outlined:**

- `dynamics*.rs`, `modeling/dynamics*.rs`, `trajectory.rs`, `integrated.rs`, `shooting.rs`, `strategies/` and `pse-model/src/study.rs`.
- Fitting covariance, regression and results.
- `pse-columnar/src/resource*`.
- `DurableAttempt` internals beyond `store_tables`, and the recover, seal and record internals (sibling E).
- The pse-py and Python study bindings.
- The internal faer and POUNCE batch thresholds, beyond the one check noted under D-08.

Trajectory output detail, and whether integrator setup repeats per sample, are left to C and E. An unexamined area is not a defect.

## 2. Complete operations examined

**(a) Independent durable sweep, point N+1, in the managed primary.** The primary runs the worker with `--maximum-in-flight 16`, `math_jobs = 32` and `threads = 16`. One lane runs these stages in order:

1. **Discovery.** It reads the config file (`check_write_admission`), then makes these calls, each one RPC: `study_page` from the first study, `study_candidates` from ordinal −1, and one `study_scope` per candidate. The candidate query is served by the composite index `study_candidates(study, settled, assigned, ordinal)`, which supersedes operation-map row b.3 #4.
2. **Preparation before the claim.** `prepare_study_candidate` reads the point and the study header. It then calls `physical_source` and `store.revision`, and `modeling_revision` (`SourceRevision::open`, which protects and releases). After that it calls `prepare_bound_operation`, which consists of:
   - `declared_execution`: a `declaration` read and `prepare` #1.
   - `prepare_declared`: `prepare` #2, a rebind job and `prepare_solve`.

   Each `prepare` performs a `protect` mutation, same-revision adoption, one acknowledgment RPC per portable body, and a `release` mutation. Optionally it adds a seed read, plus a second full preparation of the predecessor when prediction cannot rebind.
3. **Claim.** `claim_study_point` makes two protected RPCs plus a point re-read. The attempt then starts, followed by `mark_study_started` and `renew_attempt`. A heartbeat renews every 10 s.
4. **Native solve.** `Staged::open_queued` takes one job permit with an immediate try-acquire. It reserves 32 MiB of stack, 64 MiB of foreign allowance and 16 MiB of inner allowance, and starts one OS thread. CPU permits are taken per step. The session closes before publication.
5. **Publication.** About 22 relations are published, with scalar cells as indexed records, plus seeds and completion. Then come close, reconcile, seal, a re-read, and `observe_study_point`. Every mutation calls `SELECT problem` first, then takes the per-problem write guard.

Live state per lane is the prepared operation, the in-memory `RunResult` until `record_study_attempt` returns, and the session only during the solve.

**(b) Continuation-chain or dependency study under `serve`.** This is the same mechanism with blocked candidates (D-01).

**(c) Ephemeral `study_inner`.** An event-driven frontier uses a `dependents` map. `prepare_bound_operation` runs per point, concurrently up to `width`. Each sequence future gets its own `Staged`. A single continuation successor inherits the predecessor's retained `Staged`, and eligible POUNCE points are batched. Every point's full `RunResult` is retained until the study ends.

**(d) Fit with profile intervals as one case.** A single `submit_with` job acquires `threads × profile_workers` CPU permits and `bytes × workers` memory for the entire fit. Estimation evaluates experiments serially. Profile chains then run on scoped threads.

**(e) MHE or NMPC horizon step.** `prepare_analysis_attempt` runs the full `resolve_case` → `prepare` chain per sample. Steps run on one retained session, and `requests` and `results` grow per step.

## 3. Candidate findings

### D-01: Durable discovery re-examines every blocked candidate, per lane, after every completion

**Observed cause** (source-traced, Interface-checked):

- `worker.rs::work_once_scoped` restarts discovery at the first study and ordinal −1 on every call. `canonical_studies.rs::study_candidates` filters only `settled = false AND assigned = false`, so points still waiting for predecessors are returned as candidates.
- Runnability is learned only through one `study_scope` RPC per candidate, followed by the pure `study_policy::action`. That function returns `Wait(Dependency)` when a predecessor is unsettled, and `work_once_scoped` then `continue`s.
- After the scan, one lane per study issues `finalize_canonical_study`. That costs `canonical_study`, `canonical_run`, `canonical_attempt` and `begin_study_finalization`. The last is a protected mutation that `touch`es the `study:` guard even when points remain pending: the `fn::pse_study_v1::finalize` server function touches first and returns `NONE` afterwards.
- `serve` latches `idle` once any lane returns `Processed::Idle`. It then stops refilling lanes for the rest of the round, so the next round starts only when every in-flight lane has finished.

**Consequence:**

- **Continuation sweep.** In a sweep of N points where each continues from the previous one (an ordinary supported study shape), every completion is followed by a round. In that round, w−1 lanes each scan all remaining waiting points, at one `study_scope` RPC per point plus page RPCs, and one finalize probe takes the study-wide guard. Discovery work grows as roughly (w−1)·N²/2 RPCs over the study instead of O(N). For 16 lanes and 1,000 points that is on the order of 10⁷ scope reads, from source arithmetic and not a timing claim.
- **Several chains with skewed solve times.** With fewer chains than lanes, the idle latch makes each round a barrier: every chain advances one point, and then all wait for the slowest chain's current point.
- **Layout dependence.** With chains laid out chain-major, even full-width discovery pays O(chain length) scope RPCs to reach the next chain's ready head.
- **Contention.** The scanning lanes do no useful work, and each finalize probe contends with the running lane's publication (see D-02).

This falls under AP-07, H5, H9, H10, H13 and H22, and under DP-20 for broad barriers.

**Coordinator lead:** confirmed. `scope.action` returns `Wait(WaitReason::Dependency)` for unsettled predecessors. The same check exists for the continuation edge in `study_policy::action`. The candidate index itself is used, per the 2026-10-06 `IndexScan` evidence on empty tables, so the cost lies in candidates that are blocked but unsettled, not in settled points.

**Simpler conforming alternative** (Proposed):

- **Event-driven readiness, as `study_inner` already does with `dependents`.** The settle and observe server functions already touch each point and own the dependency edges. They could maintain readiness on dependents (a blocked-predecessor count, or a `ready` flag in the candidate index). That keeps discovery O(ready) and total maintenance O(edges).
- **Lane affinity for single-successor continuations.** A lane that settles point k attempts its sole successor directly. This also enables native session retention and in-memory seed hand-off, as the ephemeral `retained` logic does.
- **Continuous refill.** Replace the idle latch with continuous refill that sleeps only when nothing is in flight, so dependency release triggers new lanes.
- **Cheap finalization check.** Make the not-yet-finalizable check a read, such as a pending count, before any guarded mutation.

**Preservation:**

- The claim still fences the point and its immediate predecessors' revisions (`fn::pse_study_v1::claim`).
- The pure policy remains the authority over actions. A readiness flag is a hint, and the claim re-establishes the premises.
- Retries (`may_retry`), cancellation, refusal settlement and ordinal result placement are preserved.

**Uncertainty:** none about the structure. The magnitude depends on RPC latency and N. **Verification:** P2.

### D-02: All of a study's points share one serialized write stream and one study-wide conflict record

**Observed cause** (source-traced, Interface-checked):

- **Local pacing.** `canonical.rs::ProblemTurns::run` gives every `QueryEffect::Mutation` the exclusive `tokio::sync::RwLock` write guard for its run's problem. `protected_execution_query` resolves that problem with a `SELECT problem` RPC per call. Every point run of a study is created with the study's revision (`start_defined_study` builds each `StoredRequest` with the same `revision`), so all points share one problem.
- **Mutations per point.** A point performs roughly 35–45 paced mutations:
  - Preparation: `protect` and `release` for `SourceRevision::open`, and `protect` and `release` again for each of the two `prepare` calls.
  - Lifecycle: claim, `started` and renew.
  - Publication: progress batches, around 22 relation appends plus scalar-cell windows, seeds and completion chunks.
  - Settlement: close, manifest, seal and observe.
- **Waiting reads.** Reads wait behind queued writers because the lock is fair. The retry loop in `protected_query`, with `conflict_backoff`, runs while the guard is held.
- **Study-wide touch.** Independently of local pacing, `fn::pse_execution_v1::writer` calls `study_cancelled`, which runs `touch('study:' + study)`, a write (`UPDATE canonical_guards SET generation += 1`). The writer function is used by `append`, `close`, `renew`, `fence` and `seed`, and `seal` and `recover` call `study_cancelled` directly. So every append, heartbeat and close of every point of a study writes the same guard record. The study functions `claim`, `started`, `observe`, `finalize` and `guard` touch it too.
- **What the touch is for.** Its purpose is to make a concurrent `cancel` (which touches the study guard and updates `canonical_studies`) conflict with in-flight writes. The conflict domain is therefore the whole study, although the effect being protected is "this run's write must not commit after its study was cancelled".

**Consequence:**

- **Sixteen-case premise.** Under the 16-lane premise (one study, or 16 durable cases of one flowsheet), all lanes' coordination and publication mutations proceed single file, whatever the lane count. Point throughput is bounded by the serialized mutation stream rather than by native capacity. For small, fast solves such as flash or unit sweeps, 16 lanes degrade toward one; no speed is claimed.
- **Collateral waits.**
  - Heartbeat renewals (30 s lease, 10 s heartbeat) and preparation reads queue behind other lanes' publication.
  - An interactive durable re-solve of the same flowsheet queues behind sweep publication with no priority between the two workload classes.
  - A cross-process conflict retried under backoff stalls every lane.

This falls under AP-07, H21, H22, H23 and DP-20. It is a study-specific residual of parallel-execution F01: the read/read narrowing in Plan 35 SD05 left mutations problem-wide and added nothing for the study guard.

**Simpler conforming alternative** (Proposed):

- **(i) Scope cancellation to the bounded set it affects.**
  - **Variant 1: cancellation updates in-flight runs.** `cancel` already writes the study, and it could also mark or revoke each assigned, unsettled point's run. There are at most as many such points as lanes. The writer then checks only its own run, whose `execution-run:` guard it already touches.
  - **Variant 2: read-tracked check.** Read the study row in a way that conflicts with a cancelling writer but not with other readers. This uses the library-owned capability, `SELECT … FOR UPDATE` on record ids, with semantics to be confirmed by P1.
- **(ii) Key local pacing by the guards a transaction actually writes.** For execution writes that is the run (and attempt) rather than the problem. `protect`/`release` keep problem-scoped pacing where `retention:$problem` really is shared.
- **(iii) Carry the run's problem in the `AttemptFence`.** This removes the per-mutation `SELECT problem`. The problem of a run is immutable, so this is H19.

**Preservation:**

- Cancellation still prevents later publication.
- Exact lost-acknowledgment settlement remains (`canonical_execution_operations`).
- Lease fencing, `retention:$problem` protection against reclamation, and the immutable-batch checks are kept.
- The coordination and server guards remain the distributed authority, and local pacing stays an optimization only.

**Uncertainty:**

- Whether SurrealDB 3.3 `FOR UPDATE` readers conflict with each other on the RocksDB backend (P1).
- The actual queue time (P3).

**Rule impact:** RC-D1.

### D-03: Case lanes fuse native capacity with I/O-bound preparation and publication

**Observed cause** (source-traced, Interface-checked):

- `serve` concurrency is a single width, `min(cores, jobs/2)`, capped by `maximum_in_flight`. A lane holds its slot through discovery, the RPC-bound preparation, the claim, the native solve, all of publication and the settlement.
- The native session itself is released before publication. Its scope is limited correctly: `run.rs::launch` closes the staged sequence before `finished`. The lane slot, however, stays occupied.
- Width cannot simply be raised. `MathService::open_session_queued`, `open_session` and `admit_entry` take job permits with `try_acquire_owned`, refusing with `Limit("native jobs")` instead of queueing. The width therefore has to stay at or below `jobs/2`, a static partition recorded in `conformance.toml` as "sixteen active workers plus sixteen idle session owners".

**Consequence:** under the 16-case premise, a lane's native slot sits idle during preparation and publication, which are I/O stages. With D-02 removed, native utilization is still bounded by the solve's share of the lane's time. A new admission that arrives when permits are exhausted, such as an extra preparation job or another workload class in the same process, is refused rather than paced. This falls under H21, H22 and DP-20, and under H11 because refusal by itself does not establish fitness.

**Alternative** (Proposed):

- Admit lanes for preparation and publication separately from native work, bounded by memory and by the existing pool.
- Let native entry wait on the job and CPU semaphores under the original deadline and cancellation. The memory wait in `reserve_entry` already has this shape.

This decouples the stages without a new scheduler.

**Preservation:** finite admission, honest refusal of oversize entries, drain on stop (`group_lane`), and the per-session reservation of stacks and foreign allowance.

**Relationship:** its benefit appears only once D-02 is narrowed. Fix D-02 first.

### D-04: Every point re-establishes study-invariant preparation through canonical crossings

**Observed cause** (source-traced, Interface-checked):

- **What `prepare_study_candidate` does per point.** In both executors, `prepare_study_candidate` (durable) and `study_inner` (ephemeral) call `prepare_bound_operation` for every point. Before that, the durable path re-reads the point and the study header and reopens `physical_source`, `store.revision` and `SourceRevision::open`.
- **What `prepare_bound_operation` repeats.** It runs `declared_execution`, which reads the declaration (about 5 RPCs) and calls `prepare` #1, then `prepare_declared`, which calls `prepare` #2 with demand-extended bindings, followed by a rebind job and `prepare_solve`. Each `prepare` runs `checked_selection_scoped`: a `protect` mutation, `adopt_dependencies_from_same_revision`, `settle_preparation` with one `acknowledge_description_until` RPC per portable body (sequential), and a `release` mutation.
- **Prediction case.** When prediction cannot rebind (implicit systems, providers, sensitivity or numerics goals in `rebind_prediction_source`), `prediction_sample` calls `prepare_bound_operation` again for the predecessor.
- **Admission is discarded.** The study already builds a `DeclaredStudyAdmission` (the structural basis, keyed by operation descriptor, paths and members) in `admit_study_points`, and builds it again in `start_defined_study`, and both copies are dropped.
- **The study's revision is fixed.** The study is pinned to one immutable revision (`definition.modeling_revision`).
- **What is cached.** Compiler products are cached (`ModelingCache` Selected, Basis and Solver entries), so the repetition consists of canonical premise re-establishment, protection churn and a value-compare or rebind job, not compilation.

**Consequence:**

- **Per point.** Each point pays roughly 25 + 2B sequential RPCs before its claim, where B is the number of portable bodies, and about 7 of them are paced mutations (D-02). A 1,000-point sweep of a flowsheet with tens of bodies therefore performs tens of thousands of preparation crossings to re-prove premises that cannot change during the study.
- **Horizons.** The same entry point is used per MHE/NMPC sample (`horizon/driver.rs::prepare_solve` → `prepare_analysis_attempt`).

This falls under AP-07, DP-10, H12, H14, H19 and PS-11, and matches the "structure rebuilt per case" shape of profile calibration, in its canonical rather than compiler form.

**Alternative** (Proposed): hold one study-scoped prepared operation for as long as a worker is engaged with the study. That means one protection on the revision and selection for that lifetime, renewed with the lease, together with the structural basis and solver view. Per point, only `validate_binding`, rebind or value-compare and `prepare_solve` would run. `DeclaredStudyAdmission` is the natural owner to extend, keyed by descriptor and members. For horizons, retain the stage's prepared basis across samples in the same way.

**Preservation:**

- Per-point binding admission and the seed-need check.
- The independent PS-10 assessment of each new numerical state.
- Exact revision and context identity.
- A protection lifetime that does not outlive the engagement, and retention or reclamation of inactive studies.

**Seam:** sibling A owns the internals of `prepare` (two passes, per-body acknowledgment). D owns the lifetime: per point versus per study.

### D-05: Per-case result detail is fixed at full single-solve detail, regardless of study demand

**Observed cause** (source-traced, Interface-checked):

- **Durable.** `DurableAttempt::store_tables` publishes every relation in `RunResult::table_ids()` for every point: around 22 for modeling, including `solve_variables` and `solve_constraints` as scalar-indexed cell records. That is one indexed record per scalar cell, per sibling E and the operation map.
- **No demand to consult.** `StudyPoint` and `StudyRequest` carry an operation, preparation, overlay and policy, but no output demand. `StudyResults` returns `(point, run, attempt)` handles only.
- **Ephemeral.** `study_inner` retains every point's full `Arc<RunResult>` in `results` until the study returns. Continuation needs only the predecessor and, for prediction, the one before it.

**Consequence:**

- **Sweep scale.** A 1,000-point sweep of a model with thousands of variables writes millions of indexed cell records and full diagnostic relations, even when the consumer needs a few KPIs per point plus diagnostics for failures. This is write and lifecycle amplification under H6, H16, H27 and the retained-history premise.
- **Ephemeral memory.** Live memory grows as O(N × full report), under H17. Larger models predictably meet the pool limit and are refused, which bears on H11.

**Alternative** (Proposed): a study-level output demand, consisting of selected relations or members, a compact per-point summary relation, and full detail for failed or flagged points. Applying the same demand to retained ephemeral results would release unselected detail after the point is recorded. Detail that is not retained could be reproduced from the binding, seed and revision only within the declared determinism class, and must be labeled honestly as such.

**Preservation:** outcome facts, scientific usability, diagnostics, PS-10 evidence summaries, start provenance and result identity.

**Seam:** sibling E owns publication cost; this finding concerns demand. **Rule impact:** RC-D3.

### D-06: A fit with profile intervals holds its peak CPU and memory charge for the whole fit

**Observed cause** (source-traced, Interface-checked):

- `run.rs` (fit `start_owned`) computes `workers = profile_workers(cores)`. Requested workers default to unbounded, giving `min(chains, cores/threads)`. The job then submits with `cores = threads × workers` and `bytes = problem.bytes × workers`.
- `math/jobs.rs::admitted` acquires the CPU permits and grows the reservation before dispatch and holds both until the native thread joins.
- The parallel width is used only by the profile phase (`fitting/profile.rs::profiles`, which runs after the estimate and covariance). The estimation phase evaluates experiments serially in `FitOracle::evaluate_demand`.

**Consequence:**

- A study or batch of fits with, for example, 8 free parameters and `threads = 1` gives each fit `workers = 16`. Each such fit claims every CPU permit and 16 solve extents for its whole duration.
- `tokio::sync::Semaphore` is fair, so the pending `acquire_many_owned(16)` also blocks smaller requests queued behind it. The study effectively serializes, with 15 cores idle during each fit's estimation.

This falls under H21, DP-20 (charge at real demand, coordinate nested pools) and parallel-execution F02 (a new manifestation).

**Alternative** (Proposed):

- Admit the estimation phase at `threads`, then grow to the profile width (pausing and resuming or regrowing through the existing `ComputeAdmission` path) only when profiles run. Bound `workers` by the case-lane share rather than by total cores.
- Alternatively, run profile chains as their own admitted jobs.

**Preservation:**

- Deterministic chain results and their order.
- Typed worker-failure outcomes.
- Stacks charged to the admitted team.
- The original deadline.

### D-07: A rolling horizon retains every step's preparation and full result until the run ends

**Observed cause** (source-traced): `horizon/driver.rs::Loop::execute_solve` pushes `prepared.clone()` into `requests` and every `ModelingResult` into `results` for every estimator and controller step. Per-step preparation crosses canonical premises (D-04). The horizon is one run and one attempt, so its tables are published at finish (module doc in `horizon.rs`).

**Consequence:** for long MHE/NMPC runs, live memory and the final publication batch grow linearly with the number of samples, and a crash replays the whole horizon. This falls under H17 and H24 and the long-trajectory premise. Replaying a deterministic simulated plant can be acceptable at modest lengths, so retention, not recovery, is the material part.

**Alternative** (Proposed): stream each step's result rows to durable progress or append-only result blocks as it completes. Keep in memory only what later steps consume: the last accepted step per role, the priors and the seeds.

**Preservation:** step identity and order, recorded seeds (O6), strict sequencing and the shared native session.

**Uncertainty:** the exact per-step `ModelingResult` size, and whether `RunReport` construction needs every step resident. The publication encoders were not read.

### D-08: Implicit-block factorization picks its parallelism from faer's global rayon pool, outside CPU admission

**Observed cause** (Interface-checked against the registry source):

- `pse-math/src/implicit.rs::Problem::new` stores `faer::get_global_parallelism()`. In faer 0.24.4 the default is `Par::rayon(0)`, meaning `rayon::current_num_threads()`, which is the global pool sized to the logical CPUs. It is used for both scratch sizing and numeric LU.
- Other native paths pass `Par::Seq` explicitly or an admitted `Par`: `dynamics/linear.rs`, `square_response`, `kkt/path` and fitting.
- Nothing in the workspace calls `set_global_parallelism` or `build_global` (searched under `crates/`).

**Consequence:** under 16 lanes, every factorization on faer's supernodal path can fan out across the global pool, which holds about 32 logical CPUs, beyond the 16 admitted CPU permits. This falls under DP-20 and H21 (uncoordinated nested pools), and also enlarges scratch.

**Alternative** (Proposed): derive `Par` from the admitted compute scope (`Par::Seq` inside lanes, matching `FaerLu`), or set the global parallelism once at runtime construction.

**Uncertainty:** faer uses parallelism only on the supernodal path (`factorize_symbolic_lu` chooses it when flops/nnz exceeds a threshold), so small inner blocks are unaffected. Materiality depends on block sizes (P4). The L1 inquiry flagged the same seam. Code ownership lies with B/C; the capacity judgment lies with D.

### D-09 (minor): Finalization and inventory read occurrences one at a time

**Observed cause** (source-traced): `finalize_canonical_study` pages `study_point_page`, then calls `canonical_study_point` once per point to obtain the definition for `study_tables::outcome_row` (the case id). The same item-by-item re-read appears in `prediction_sample`.

**Consequence:** N sequential RPCs once per study, under H8. This is minor next to D-01 to D-05.

**Alternative:** project the needed descriptor fields in the page query.

## 4. Opportunities and uncertainties that are not findings

- **Fitting evaluates experiments serially** (`FitOracle::evaluate_demand`). They are independent and each has its own worker. Parallel evaluation with ordered assembly would preserve the deterministic addend order (H22). This is not a finding because the fit admits only `threads` permits, so parallel experiments would compete with lanes. It is worth considering together with D-06.
- **Per-evaluation clones.** Each fit evaluation clones `layout.responses` and `layout.constraints` (`AssemblyMatrix`, including `col_ptr`, `row_idx` and the refill map), plus the steady `values` and the Jacobian blocks. That is O(nnz) structure copying beside O(nnz) refill, which is probably minor next to the evaluations (H12).
- **Transient gradient fits** still integrate forward twice when the solver requests the objective and then the gradient at the same iterate (N7 lead, residual). The best replacement, a checkpointed forward used as the objective versus eager backward on rejected trials, remains open, as the prior review said.
- **`uncertainty.rs::propagate_checked`** forms a dense m×m output covariance with scalar triple loops. A faer GEMM, or diagonal-only demand, would be library-owned, but `propagation_preserves_scalar_sum_bits` shows that the summation order is an explicit determinism contract (DP-11). This is not a finding without a large-m workload.
- **`study_inner` frontier.** It rescans its frontier on every completion (O(N²) in-memory policy evaluations), which is cheap. It is noted only as a contrast with D-01.
- **Repeated admission.** Interleaved operations in one study drop and re-prepare `DeclaredStudyAdmission` at every switch, and admission runs twice (`admit_study_points`, then `start_defined_study`). The second run is partly justified because the definition crossed Python as JSON (H18).
- **Per-mutation cost.** Each mutation re-reads `ensure_writes` (`config.json`); this belongs to E and H.
- **Defaults.** The default `MathPolicy.jobs = 4` gives two study lanes, while the reference execution configures 32. Whether default Python runtimes are meant to support 16 concurrent cases was not determined.

## 5. Strengths and preservation constraints

- **One policy authority.** `study_policy` is pure and shared by both executors. Durable claims fence only the point and its immediate premises (`fn::pse_study_v1::claim`/`scope`), with exact operation receipts. EF03 is resolved.
- **Capacity coordination that already works:**
  - One CPU semaphore covers both compiler and native jobs (`SharedRuntime::build`).
  - Memory admission waits under the original clock and calls on retention pressure (`reserve_entry`, `admitted`, `Pressure::demand`/`retry_growth`).
  - Native teardown is joined before permits are released.
  - MKL and OpenMP thread counts are scoped per worker (`mkl.rs::Threads`).
  - POUNCE uses local scoped pools, with team stacks reserved (`staged.rs` `scope_stack_bytes`).
  - faer calls use `Par::Seq` inside lanes.
  - Sessions close before publication.
- **Drain on stop.** Stop and failure drain correctly in `serve`/`group_lane` and `study_inner`. Study cancellation is durable and lets in-flight attempts drain.
- **The ephemeral scheduler** is event-driven: `dependents` frontier, single-successor session retention, POUNCE batch grouping and failed-point isolation. It is the model for D-01's remedy.
- **The candidate discovery query uses its composite index.**
- **Fitting:**
  - Prepared per-experiment observation and response mappings (F10 resolved).
  - Same-point caching.
  - Analytic sparse gradients.
  - Profile chains with typed scheduling outcomes.
- **Study creation** is bulk, bounded and cancellable (`creation_checkpoint`).

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| Parallel F01 (broad retention conflict domain) | Residual | Reads were narrowed (Plan 35 SD05), but mutations are still problem-paced (`ProblemTurns`), and a study-wide `touch('study:')` sits in every execution write (D-02) |
| Parallel F02 (maximum capacity charged as demand) | Residual | Known bounds are now used (`requirements_allocation_bound`, `rebind_allocation_bound`), with a `worker_bytes` fallback (`math/modeling.rs`). Fit profile charging is a new manifestation (D-06) |
| Parallel F03 (native coexistence and exclusion) | Not examined, apart from the D-08 seam | Sibling C |
| Parallel F04 (POUNCE team stacks) | Resolved | `math/staged.rs` reserves `math:native-scope-stacks` through `scope_stack_bytes`, retained while idle |
| Parallel F05 (serial frontier) | Resolved for dispatch; throughput limit moved | `study_inner` runs concurrent sequence futures; the primary runs `--maximum-in-flight 16`. Serialization now arises in D-02 and D-03 |
| PE04 (occurrences reacquire source and admission) | Residual | Compiler products are cached, but per-point canonical re-establishment remains (D-04) |
| EF03 (dispatch evaluates the whole study) | Resolved | `study_scope` and `claim` use immediate premises; `transition` runs only at the end or in the ephemeral path |
| Enhancement F01 (creation repeats admission) | Residual | Adjacent-equal keys are shared, but admission runs twice and interleaved operations re-admit; creation cancellation is now honored |
| Enhancement N7 (transient fitting demand) | Residual | The combined report is consumed; a double forward remains for objective-then-gradient |
| 10-09 F10 (fitting response search) | Resolved | `mapping.observations` and `mapping.responses` are direct (`oracle.rs::evaluate_demand`) |
| State F01 (rebind ancestry) | Resolved at `own_rebind` | No parent chain passed; the `SharedAllocation` anchors were not re-examined |
| State F02 (retention yields) | Resolved on job and session admission paths | `pressure.try_grow`/`retry_growth`; other paths not examined (sibling B) |
| Operation map b.3 #4 (no index) | Refuted | `DEFINE INDEX study_candidates … (study, settled, assigned, ordinal)`, plus the 2026-10-06 `IndexScan` evidence |

## 7. Candidate rule impacts

- **RC-D1: per-problem local pacing (blueprint §14.4, Plan 35 RC01, `canonical.rs` `QueryEffect` doc).**
  - Proposed change: pace each transaction by the guard domain it actually writes (run for execution writes; problem only where `retention:$problem` is shared).
  - Associated change: express study cancellation without a study-wide write-touch in execution writes (the `pse-codegen` SurrealQL generators for `writer`/`study_cancelled`).
  - If the rule is kept: D-02 reduces to removing only the study touch, and per-problem write serialization remains a declared capacity limit.
- **RC-D2: `MathPolicy.jobs` as an immediate-refusal ceiling "including admission waiters" (`math.rs`; the `conformance.toml` "idle session owners" comment).**
  - Proposed change: queued native entry under deadline and cancellation, with lane width independent of `jobs/2`.
  - If the rule is kept: D-03 holds width at `jobs/2` and accepts idle native capacity during I/O.
- **RC-D3: study result contract, under which every occurrence is a full single-solve run with all relations (workflows-and-results section; study definition v8).**
  - Proposed change: a study-level output demand.
  - If the rule is kept: D-05 reduces to bounding ephemeral retention.
- **RC-D4: "A horizon is one run and one attempt" with in-memory step history (`horizon.rs` doc; ADR-0110 Outcome 5; I16).**
  - Proposed change: stream per-step results.
  - If the rule is kept: D-07 becomes a documented length limit.

## 8. Proposed light probes

- **P1 (D-02 remedy choice).** Does SurrealDB 3.3.0 on RocksDB let two transactions that each run `SELECT * FROM ONLY study:s FOR UPDATE` and then `CREATE` distinct records both commit, while a third transaction doing `UPDATE study:s` makes them fail with a retryable conflict?
  - Command: a scratch Rust test modelled on the neo4j-surrealdb skill's `sb091_probe`, using `surrealdb = "=3.3.0"` with the `kv-rocksdb` and `kv-mem` engines, two client transactions plus one updater. Run as `scripts/pse-env --resource-class light -- cargo test --manifest-path <scratch>/Cargo.toml`.
  - Discriminating outcome: if the readers do not conflict with each other but do conflict with the updater, a read-tracked check replaces the touch (variant 2). Otherwise use per-run cancellation propagation (variant 1).
- **P2 (D-01 magnitude).** Count the `study_scope` and `begin_study_finalization` calls for one 64-point continuation chain served with `maximum_in_flight = 4`.
  - Method: wrap the store calls in a counting tracing span inside the existing canonical-tests study fixture (`workflow/study_tests.rs` route), via `scripts/pse-env --resource-class light -- cargo t -p pse-runtime --features canonical-tests <new test>`.
  - Expected under the current design: about 3 × 64²/2 ≈ 6,000 scope reads and about 64 finalize probes. An event-driven design would need about 64 reads.
- **P3 (D-02 materiality).** Record `ProblemTurns` write-guard wait time and mutation count per point for a 16-lane, 64-point independent sweep of a small model.
  - Method: a tracing span around `gate.write_owned()`.
  - Discriminating outcome: whether the cumulative wait is comparable to the solve time per point.
- **P4 (D-08 materiality).** For the largest implicit block in the reference packages, does `factorize_symbolic_lu` select the supernodal path?
  - Method: log the `SymbolicLu` debug output, which starts with `Simplicial` or `Supernodal` (faer 0.24.4 has no public `raw()` accessor), in an existing `pse-math` implicit unit test, run as `scripts/pse-env --resource-class light -- cargo t -p pse-math implicit`.
  - Discriminating outcome: the seam is material only if the supernodal path is chosen.
