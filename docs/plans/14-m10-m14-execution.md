---
title: M10–M14 unified native solver execution
status: complete
date: 2026-09-24
plan: 14-library-owned-process-simulator
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
evidence: Tested — targeted unit acceptance only; native convergence and full qualification remain M22
---

# M10–M14 execution

The approved target is one solver lifecycle over distinct mathematical representations:
admitted model, pure Salsa preparation, deterministic class routing, native compilation,
bounded worker execution, original-model validation and a structured report. Five native
backends are in scope: Ipopt, POUNCE, KINSOL, HiGHS and Clarabel. No Pyomo, legacy
fallback, second expression IR or custom iteration algorithm is retained.

## Dependency order

1. **Shared contracts and M10 extension.** Establish solve intent, compiler facts,
   auto/explicit selection, typed profiles, availability and capability descriptions,
   prepared representations, warm compatibility, finite sequences and reports. Separate
   structure from numeric assumptions. Coefficient preparation consumes `CasePlan`
   without building evaluators. Worker-local oracles need not be `Send`. Salsa owns
   immutable products, never native mutable models, bases or convergence state.
2. **Model and result semantics.** Admit semi-continuous and semi-integer domains,
   including their zero branch, physical bindings and fixed-value checks. Introduce
   explicit PSD cone declarations and upper-column svec normalization. Preserve source
   identity, objective sense/constants, exact sparse maps, original residuals, multiplier
   meaning and unavailable data. Native termination, candidate availability, assurance
   and independently recomputed quality are separate report dimensions.
3. **Runtime lifecycle.** Reuse `MathService` admission and completion-owned native
   threads. Construct/destroy native state on its worker; retain CPU/memory through join
   and native scheduler shutdown. Acquire nested work's full budget once. Provide bounded
   progress and cancellation with an awaitable terminal report. Finite sequences reuse
   compatible allocations and starts; failures stop unless independent continuation is
   explicit. Only immutable compilation is single-flight.
4. **M11 Ipopt.** Direct pinned C interface, RAII, checked indices, null/zero-length
   callbacks, exact weighted Hessian or explicit limited memory, transactional callback
   outputs, panic containment, typed failures, scaling, options, current-iterate diagnostics,
   cancellation, all statuses and primal/dual warm starts. Disable option-file influence.
   Finite authored bounds must not disappear at native infinity thresholds.
5. **M12 POUNCE.** Same NLP oracle through native TNLP/application 0.12.0, FERAL,
   interior point and explicit active-set SQP. Use library statistics, timing, crossover,
   working-set/iterate starts and derivative proofs. Disable general presolve until M15.
   Explicit admitted Rayon execution owns parallel FERAL resources and joins them.
6. **M13 KINSOL and initialization.** Bundled SUNDIALS 7.1.1, vendored SuiteSparse
   KLU, analytic sparse Jacobian; explicit SPGMR/JVP and bounded dense profiles. Native
   Newton/line search and explicit fixed-point/Picard/Anderson profiles. Arbitrary box
   bounds are not sign constraints; fixed-point/Picard reject constraints. Use M09 DM/BTF
   predecessor order and explicit boundaries, commit successful blocks only, retain
   source-attributed failures, distinguish structural deficiency/numeric singularity,
   and support finite supplied continuation schedules without custom globalization.
7. **M14 HiGHS.** Workspace highs 2.4.0 / highs-sys 1.14.3 (native 1.14.0), LP/MILP,
   semi domains and certified convex continuous QP. Native starts/bases/updates, typed
   method controls, faithful solution/basis availability, full applicable metrics and
   supported opt-in rays/IIS/ranging/relaxation diagnostics. Reject MIQP/nonconvex QP.
   Preserve native concurrency with a shared execution/exclusive scheduler-shutdown
   gate; teardown blocks on `Highs_resetGlobalScheduler(1)` before releasing admission.
   The pinned native header has no QP interrupt callback; report that coverage honestly.
8. **M14 Clarabel.** All declared cone families plus PSDTriangle with `sdp-netlib`,
   serial LP64 BLAS/LAPACK; keep incompatible `faer-sparse` off. Preserve native settings,
   information and solution data; implement original-space quality. Reusable-data mode
   disables presolve, chordal decomposition and sparse-zero dropping; use native update
   eligibility and setters. Do not claim an external iterate warm-start interface.
9. **Tears and recycle.** Complete typed flow nodes and connection occurrence IDs,
   explicit decision groups and physical bindings. HiGHS solves weighted feedback-edge
   MILP with order variables and one binary per decision group, then independently check
   the residual DAG. Preserve incumbent/bound/gap and heuristic versus optimal assurance.
   Explicit unweighted petgraph greedy FAS remains a heuristic. KINSOL owns coupled SCC
   roots or declared sequential causal fixed-point maps; no differentiation through
   iteration histories and no private recycle solver.
10. **Acceptance and documentation.** Targeted unit contracts and compilation per packet;
    delete replaced mechanisms and update inventory/current-state documents. M22 alone
    runs native convergence journeys, integration, Python, performance and whole-plan
    qualification. M15 retains general presolve/postsolve/scaling qualification; basic
    warm compatibility and bounded native reuse are pulled forward here.

## Routing and failure contracts

Auto selection is all-fixed evaluation, HiGHS for linear/mixed-linear/certified convex
QP, Clarabel for explicit continuous cones, KINSOL for eligible square roots, and Ipopt
for smooth continuous NLP or an explicitly constant feasible-point objective. POUNCE is
an explicit alternative. Missing default backends return availability errors and named
alternatives, never silently change methods or mathematical classes.

Domain and explicitly recoverable provider trial/singularity errors reject a trial.
Contract/resource/terminal-provider errors and panics terminate. Cancellation and
deadlines are distinct. A recovered trial error cannot overwrite later native success.
Native option extensions are validated and cannot bypass typed semantic controls.
Limits do not imply optimality; local NLP infeasibility is not a global certificate.
Quality retains each physical row/quantity and uses dimensionless aggregate tolerances.

## Implemented interfaces and library ownership

| Package | Entry point / owner | Built-in capabilities used and explicit limits |
|---|---|---|
| M10 extension | `CompilerWorkspace::{prepare,prepare_flow,prepare_initialization_blocks}`; `MathService::{prepare_solve,prepare_conic,solve}` | Salsa tracks immutable semantic facts, physical projections and conditional programs. DataFusion's shared pool/cache remains the retention owner. Native mutation never occurs in tracked queries. Coefficient-only preparation avoids evaluator construction. |
| M11 | `pse_backend_native::ipopt::Session` | Direct C problem lifecycle, exact weighted Hessian/limited-memory mode, intermediate callback, current iterate/violations, typed scaling, available primal/dual seeds and conservative constant-derivative proofs. Option files and semantic overrides are refused; serial MUMPS profile only. |
| M12 | `pse_backend_native::pounce::Session` | Native TNLP/application, interior point and active-set SQP, FERAL, native restoration, full SolveStatistics/timing/linear summaries and starts. Scoped Rayon owns admitted parallel FERAL. Application reuse is reported separately from factor reuse; general presolve disabled. |
| M13 | `pse_backend_native::kinsol::Session`; `MathService::{prepare_initialization,initialize,solve_declared_root}` | KINSOL Newton/line search, explicit fixed-point/Picard/Anderson, KLU, bounded dense or analytic SPGMR/JVP. Complete BTF schedules, explicit conditional inputs and finite supplied continuation. Successful original-quality-validated blocks commit atomically; constrained maps/arbitrary boxes refuse. |
| M14 | `pse_backend_native::highs::Session` | Native checked LP/MILP/convex-QP uploads, semi domains, data updates, bases/primal-dual/sparse MIP starts, option/default readback, status/solution availability and available native information. Opt-in rays/IIS/ranging/independent relaxation preserve diagnostic scope. QP has time-limit interruption only in this native pin. |
| M14 | `pse_backend_native::conic::Session` | Native Clarabel CSC/cones including PSDTriangle, full settings/info/results, explicit bound-row maps, original-space quality and native certificates. Reuse follows native update eligibility; no external iterate warm-start API is claimed. |
| M13/M14 | `MathService::{prepare_flow,select_tears}` and `pse_backend_native::recycle::CausalMap` | Complete directed multigraph/isolates, occurrence IDs, decision groups, cost/policy and physical conversion. HiGHS order-variable MILP plus independent petgraph DAG witness; explicitly unweighted greedy FAS alternative. Declared causal sweeps are maps consumed by KINSOL, never custom iteration algorithms. |

A `SolveSequence` is finite and executes on one admitted owner thread. Native state is
retained only while compatible with the declared reuse policy. Completion awaits join,
TLS cleanup and native teardown; HiGHS teardown acquires an exclusive scheduler gate
and blocks on `Highs_resetGlobalScheduler(1)`. The POUNCE local pool is joined inside
admission. Initializations and tears use the same cancellation/progress/result ownership
mechanism. Native report envelopes and constant results retain their result lease even
when extracted/cloned; explicit caller-made copies are outside runtime ownership.

Source-space quality retains physical row/variable identity and explicit tolerances.
Only dimensionless ratios are aggregated. Native termination, feasible candidate,
assurance and certificates are distinct. Recoverable trial errors do not poison later
success; panic/terminal failure clears assurance. Failed postsolve validation preserves
the actual native termination. Optional diagnostics cannot erase the original attempt.
Native option extensions are finite, library-validated and cannot override typed
semantics, including through restoration prefixes. Progress bounds cover count and
per-event retained extent; omitted events are counted.

### Native build profiles

`pse-runtime/native-solvers` selects Ipopt, POUNCE, KINSOL, HiGHS and Clarabel SDP.
The individually selectable runtime features are `solver-ipopt`, `solver-pounce`,
`solver-kinsol`, `solver-highs` and `solver-sdp`; non-SDP Clarabel remains available
by default. This packet compiled the default and full native profiles; independent
feature combinations and other platforms remain final-qualification obligations.

The Ipopt C ABI and serial MUMPS/netlib runtime come from the existing digest-pinned
solver image. `scripts/native-solver-env.sh` provides its build prefix and container
runner. `scripts/native-math-env.sh` builds only KLU/AMD/BTF/COLAMD/config from the
SuiteSparse 7.7.0 source vendored by pinned suitesparse_sys 0.1.4, static/PIC with
CHOLMOD/CUDA/OpenMP disabled. The build is serialized and keyed by `klu-profile-v1`;
profile changes must change that key. No upstream source is edited. Clarabel SDP uses
serial LP64 netlib; its incompatible optional faer-sparse profile remains disabled.
SUNDIALS uses its bundled 7.1.1 f64/64-bit-index profile. The license key is supplied
locally by `SYMBOLICA_LICENSE`, passed by environment name into native test containers
and activated before Symbolica calls; it is never recorded in source or result identity.

## Verification

**Tested, baseline zero:** `just unit-native-contracts` — **95 passed, 0 failed,
58 outside the selected filter**. Conditions: pinned Rust 1.98.1, native solver
container, local Symbolica license, `pse-runtime/native-solvers`, and explicit
`pse-relations/force-validate`. The recipe selects backend/math/compiler units,
Ipopt ABI controls, structural flow/initialization units and runtime math lifecycle
units. It executes native constructors/uploads and adapter callbacks, not native
convergence journeys.

The controls cover callback null/zero slices, transactional failure and unwind
containment; analytic objective/Jacobian/weighted Hessian mapping; native status
scope, quality and semi-domain zero branches; KINSOL allocation/pattern/constraint
refusals; HiGHS checked upload and authored sense/constant; every cone and svec;
Clarabel data updates; complete physical flows, independent tiny exhaustive tear
witnesses and fresh causal sweeps; Salsa membership/absence and conditional demands;
framed sparse identity; native option protection/event extent; result leases,
shared single-flight admission and cancellation through TLS destruction.

**Interface-checked:** `just check-solver-contracts` compiles all targets for backend,
runtime, compiler and relations with the full native/force-validation profile.
`just check-package pse-runtime` compiles the default runtime/all-target profile.
No task-owned compiler warnings remain. Cargo still emits the existing transitive
`proc-macro-error2 2.0.1` future-compatibility notice; M22 must resolve or characterize
that dependency issue. The Python editable environment/extension was stale at startup;
this packet does not claim a Python rebuild or acceptance.

**Not claimed:** native convergence or numerical parity; scientific validity of the
FeOS envelope; universal warm-state speedups; exact foreign allocation accounting;
RSS enforcement; public model/Python workflows; durable result publication;
format/lint/governance/codegen/document aggregate qualification. Those are M16/M20/M22
or, for transformed original-space recovery, M15. No full campaign was started.

## Outcome

**Implemented:** all approved M10–M14 interfaces above, hard-cut deletions preserved,
and current plan/inventory/foundation/README/status documents reconciled. The
[scoped review](../design_review/reviews/design_review_unified-native-solvers_2026-09-24.md)
records contract-level acceptance and remaining qualification boundaries.

**Mistakes made and corrected:** relying on suitesparse_sys's broad vendor profile
selected unwanted CHOLMOD/CUDA and exported an unusable dependency path; the explicit
pinned KLU-only profile replaces it, with an actual native-allocation unit proving the
link. The test runner initially dropped the supplied Symbolica license at the container
boundary; forwarding the variable and public initialization now pass concurrent units.
Review also corrected ambiguous sparse-column hashing, postsolve diagnostic errors
that could discard a native report, and result leases that could be lost when a step
was extracted. Convexity checks now execute on admitted workers, and tear witnesses
retain their lease with one separately owned native attempt. Dedicated
negative/ownership controls cover these contracts.

**Deliberate deviations:** basic warm starts and finite native reuse are pulled forward
from M15 to establish one lifecycle now. General nonlinear presolve/postsolve remains
M15; no private solver algorithm fills that gap. Bounded explicit continuation and
causal-map declarations are implemented, while automatic process workflow construction
remains M16. Backend native settings/statistics are retained rather than flattened into
a false universal option or metric vocabulary. Native memory uses finite admission
allowances and library lifetime control; measured allocator/RSS guarantees remain M22.

## Current checkpoint and remaining work

1. **M15:** qualified presolve/scaling/postsolve, original-variable and dual recovery,
   transformation-sensitive starts/reuse and supported presolve subset admission.
2. **M16:** public source/model/Python orchestration, scientific workflow construction,
   durable result/publication schema and interrupted-publication semantics.
3. **M17/M18:** native dynamics and parameter fitting through the shared model/runtime.
4. **M19–M21:** cleanup, proposed ADR/blueprint reconciliation, current acceptance and
   performance inventories, complete implementation/deletion closure.
5. **M22:** one full native convergence/integration/Python/performance/feature/governance
   qualification campaign. Historical Plan 13 evidence does not close these gates.
