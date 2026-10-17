# Parallel execution, native update grain and prior-scope coverage

Supporting inquiry for [the principal audit](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target-audit_2026-10-12.md); the principal owns the integrated verdict and evidence limits.


Supporting coordinator inquiry for the independent efficiency/solver audit. Baseline:
`6c368c88acda548d421b44e8fc4f457c95bcec7e`, plus the source review's uncommitted documents.
Source/interface inspection only; no probes, builds, tests, services or measurements.
The analysis applies Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5.
It supplies bounded evidence and recommendations, not a separate verdict or status ledger.

## Parallel execution already has more than case-level machinery

The F08 headline and consequence are too broad. `pse-backend-native/src/pounce.rs::with_threads`
already installs a local Rayon pool for a whole finite sequence. `pse-backend-native/src/mkl.rs::Threads`
sets and restores the owning worker's OpenMP and MKL-local widths. Profile-likelihood
analysis already creates an admitted team of independent parameter chains:
`pse-runtime/src/workflow/fitting/profile.rs::FitProblem::profiles` computes
`cores = threads * workers`, accounts parent-resident versus additional worker/team state,
and calls `MathService::quiescent_phase` followed by scoped dispatch and adapter scopes.
The returned chains record actual parallelism and scheduling failures. This is source evidence
of a path, not a new successful execution or speed claim.

BTF wavefronts, shooting windows and fit experiments remain distinct opportunities. A
single large case does not necessarily benefit from each one: useful parallelism depends on
the dependency graph and solver phases, and a coupled SCC may need a native inner solver.
The corrective contract is coordinated operation-shaped parallelism, not a promise that every
case occupies every available core. Do not change the model or repeat science solely to
raise utilization. Independent profile chains are a useful existing composition to extend.

### Pools follow execution context

`pse-math/src/implicit.rs::Worker::problem` selects `Par::Seq` or
`Par::rayon(scope.admitted_threads())`. Faer uses the ambient Rayon registry through spindle;
the global pool is the fallback when no explicit pool is installed. A nested implicit
operation invoked inside `pounce::with_threads` can therefore execute on that local pool.
The review's universal global-pool claim is incorrect. Other entry points still need an
explicit, verified pool binding; a numeric width does not itself install one.

Context7 resolved Rayon to `/websites/rs_rayon_rayon` and returned the primary
`ThreadPoolBuilder::build_scoped`, `ThreadPool::install` and current-registry documentation.
The project pin is Rayon 1.12.0; the native auditor separately read faer 0.24.4 and spindle
0.2.6. Current documentation is discovery evidence, not proof of this checkout's runtime.
Source: https://docs.rs/rayon/1.12.0/rayon/struct.ThreadPoolBuilder.html and
https://docs.rs/rayon/1.12.0/rayon/struct.ThreadPool.html .

### Three semaphores represent different obligations

`pse-runtime/src/math.rs::MathService` has CPU, population (`jobs`) and active-job
(`active_jobs`) counters. `math/jobs.rs::PopulationLane` can retain a resident session and
lend its population to one dependent child. `ActivePermits` acquires CPU and active-job
capacity tentatively; `try_active` returns both or drops the tentative job permit.
`reserve_active` reserves construction memory before active capacity, releases tentative
memory before waiting, and observes the original cancellation/deadline. A quiescent phase
leaves parent memory charged while lending active compute to the nested team, then resumes.

This is meaningful resident/active/queued ownership, not evidence of three independent
CPU budgets or duplication merely because three counters exist. A simpler scheduler can
replace the mechanism, but must still bound parked sessions, active teams, queued work and
resident memory and avoid hold-and-wait deadlocks. Preserve actual join-before-charge-release
and owner-thread destruction. Profile compatibility must distinguish scientific capability,
requested inner width and placement capacity.

`pse-runtime/src/env.rs::SharedRuntime::build` currently shares the engine memory/spill
runtime, cache owner and CPU semaphore with mathematics. A separate analytics executor is
reasonable isolation, but its memory, CPU, spill, IPC and queued work must remain children
of one deployment envelope. The target's phrase "analytics pool is separate" does not
specify that composition. This is an additional target-design obligation, not proof that
the existing shared envelope is wrong. Process lanes likewise need parent/child capacity,
bounded IPC/backpressure, authenticated job identities and result rejection after cancellation.
Pipes remove network RPC machinery; they do not establish these contracts by themselves.

## Additional native-update opportunity

**Interface-checked current cause:** `pse-backend-native/src/highs.rs::Session::update`
validates compatible layout and then visits every column cost/bound, every stored matrix
nonzero, every row bound, objective sense/offset and Hessian. The loop uses scalar
`Highs_changeColCost`, `Highs_changeColBounds`, `Highs_changeCoeff` and
`Highs_changeRowBounds`. Even a compatible bound-only update sends unchanged matrix
coefficients through the native boundary. This is separate from the review's retained-basis
and allocation discussion. It does not make the review's "native updates are untrue"
accusation correct: those native model updates are real.

**Proposed correction:** the coefficient realization owner can carry a complete value delta
over a compatible native layout. Skip matrix/Hessian refresh when its complete value premises
are unchanged; batch changed costs and bounds through native set/range calls. Where exact
delta provenance is unavailable, bulk updates still remove scalar crossings, while a checked
full update remains valid. Do not retain a second editable coefficient authority or infer
unchanged values from pattern identity. Preserve binary-domain clipping, row/objective
coordinates, Hessian meaning and all native error checks. A partially failed mutation must
invalidate/rebuild or otherwise reconcile the session before reuse. Start/basis policies
remain independently declared; no unconditional removal of `Highs_clearSolver` follows.

**Existing library fit:** Cargo.lock selects highs 2.4.0 / highs-sys 1.15.0. The bundled
`highs/interfaces/highs_c_api.h` exposes `Highs_changeColsCostBy{Range,Set,Mask}`,
`Highs_changeColsBoundsBy{Range,Set,Mask}` and `Highs_changeRowsBoundsBy{Range,Set,Mask}`.
The project already uses `Highs_changeColsCostByRange` in `restore_objective`, confirming a
local seam. No equivalent matrix-value bulk update was established in this inspection;
use the actual exposed coefficient operation for changed entries rather than inventing one.
Context7 `/ergo-code/highs` supplied current primary C-API and modification documentation;
the pinned bundled header is the version-specific evidence.
Primary source: https://github.com/ERGO-Code/HiGHS/blob/v1.15.0/highs/interfaces/highs_c_api.h .

**Verification, not run:** on identical admitted coefficient problems, compare a full native
update and a delta/bulk update for bound-only, objective-only, matrix-value, Hessian and
structural-change cases. Check exact effective model values, fresh original assessment,
explicit basis versus fresh starts, and native-update failure invalidation. Count actual
adapter calls to establish removed work; measure complete repeated solves before claiming
latency/throughput improvement. This is a targeted AP-07/DP-10/H8/H12/H16 opportunity,
independent of any store, journal or server replacement.

## Plan36 coverage: assessment only, no status changes

The principal subject's blanket "superseded" and "outside product" rows are too coarse.
Removing a particular store protocol does not remove its completeness, ownership, identity,
memory or interruption obligation. The following is a comparative recommendation, not a
second editable backlog. Plan36 retains its current native status and all original evidence.

| Packets | Audit recommendation and preservation |
|---|---|
| P-G0/P-G1 | Retain decision routing and affected assembled acceptance. New rule changes do not retroactively close stopped evidence. Re-scope only in later plan creation. |
| P-A1 | Retain typed value/fixed/start binding and original provenance. |
| P-A2 | Retain complete reference/namespace/absence invalidation and immutable parent/case meaning; adapt local edits and published-head CAS to selected authority. |
| P-A3 | Retain one complete declared preparation and operation-shaped demand. |
| P-A4 | Conditional mechanism redirection; preserve complete content/dependency meaning, exact associations and acknowledgement under the selected memo/publication contract. |
| P-A5 | Retain the held engagement/bundle and its allocation owner. Server removal is not its prerequisite. |
| P-A6 | Narrow store-specific receiving work only where removed; process lanes and imported native/memo artifacts still need authentic current execution/build admission. |
| P-A7 | Retain granular pure preparation, complete dependencies, live-lineage coexistence and detached historical products. |
| P-B1 | Store conflict mechanics conditional; preserve reader/creator/retire/unknown-effect ordering in every remaining writable/GC tier. |
| P-B2 | Redirect readiness to the selected runner/queue/log owner; retain eligible-frontier locality, discovery/recovery and bounded finalization. |
| P-B3 | Retain resident/population/active/queued accounting, quiescent lending and actual drain. |
| P-B4 | Retain exact row/field/original-coordinate selection and completeness across any Arrow/Parquet representation. |
| P-B5 | Redirect acknowledgements to actual journal/manifest commit semantics; preserve complete publication and honest unknown/partial outcomes. |
| P-B6 | Retain bounded noninterfering ordinary telemetry, drop/partiality evidence and terminal scientific authority. |
| P-B7 | Conditional protocol replacement; preserve owner-authorized collection and late replay/acquisition fencing. |
| P-B8 | Retain complete eligible seed discovery before cutoff and one owned admitted seed; promoted artifacts do not justify losing either obligation. |
| P-C1/P-C2 | Retain compact trial/binding layouts and demand-shaped scalar/derivative preparation. |
| P-C3 | Retain complete cubic memo/branch/IFT premises and certified fallback; no blanket exact-root/refinement substitution. |
| P-C4 | Extend retained conditional allocations and transactional block sequences with shared suitability policy and independent per-block profiles. |
| P-C5 | Retain same-point factor/action reuse and honest rank/fill limits. QR diagnostics need a distinct convention from existing SVD singular-mode semantics. |
| P-C6 | Retain SCC-local constraints with globally coupled decisions/costs and an independent acyclicity witness. |
| P-C7 | Retain indexed resolution and complete policy/physical/attribution premises. |
| P-C8 | Retain request-local quiescent exclusion and charge handoff for IBEX; process placement can replace only the relevant mechanism. |
| P-C9 | Retain compact surrogate advancement, charged snapshots and requested/achieved shooting derivative order. |
| P-D1/P-D2 | Retain actual immutable validation context and carried admission. Existing session-free local planning is a strength; SQL/UDF semantics require their actual binding. |
| P-D3 | Redirect applicable transport representation; preserve bounded decode/malformed-input integrity, ownership and response cancellation. |
| P-D4 | Still relevant to file-backed analytics/journal readers: lazy file capacity and a shared budget are not obsolete because bulk results move to files. |
| P-D5 | Already complete; preserve explicit float round trips and original qualified scope. |
| P-D6 | Retain truthful interactive/correctness extension modes and actual binary provenance. |
| P-D7 | Product-shaped scope must not be excluded wholesale: delayed Python native pulls/SIGINT/drain, response ownership, weighted physical admission, vectorized gathers and IPC export. Optional untouched questions remain conditional. |
| P-E1 | General host metadata lifecycle is outside this product audit, but current obligations and unknown/pinned evidence remain with its owner. |
| P-E2 | Product-driven launch/dispatch/cancellation and repeated setup remain relevant; generic unrelated tool cleanup is outside. |
| P-E3/P-E4 | Retain responsibility-shaped tests, lower admission/generation roots, coherent surrogate ownership and exact supported dependency/feature closure. |
| P-E5/P-E6 | General host policy/tuning stays outside; product-driven native build inputs, repeated invocation preparation, library teams and shared-envelope consequences remain relevant. No unexamined host change is authorized. |

This covers all39 packets (2 coordinator + 37 companions). R36-01 remains a sparse rank
design/binding decision with semantic constraints; R36-02's public unused hashing option
is not silently removed; R36-03 optional publication is reconsidered through the new target's
explicit rule decision. None changes status here. Existing Plan28 E3/E4/E5, Plan36 P-G1
and Plan32 remain with their native owners. The Plan37 artifact and identity contracts are
preservation leads that may change by an explicit rule impact, not immutable criteria.

## Failure archaeology does not establish savings

W-C explicitly samples 34 failed/interrupted attempts and uses primary/secondary categories,
not a complete test or defect census. Several rows mix fixtures, placement, stack frames,
environment and transport. Its16 lifecycle-primary count is a count within that selected
sample, not a measured defect rate, causal estimate, architecture score or forecast that
removing the server prevents16 failures. Row33 itself combines a wrapper placement error,
an inline future stack failure and a mistaken graph fixture. At least the generic native
stack/capacity obligations survive a journal/process-lane design. The source review's line
counts similarly establish surface size only, not responsibility duplication by themselves.

Keep those observations as leads to the concrete ownership/failure path. The independent
current-source arguments, simpler complete alternatives and preserved obligations should
settle the recommendation. No new historical qualification or speedup is established by
this inquiry, and no stopped campaign was resumed.
