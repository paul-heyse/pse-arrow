---
title: Numerical execution
status: current
---

# Numerical execution

This area turns an admitted, prepared case into native numerical work and truthful
outcomes. It owns structural admission, the single resolved numerical policy,
initialization and recycle strategies, class-specific adapter selection, attempt-owned
native execution and the result envelope. Libraries own every iterative algorithm; the
project owns contracts, coordinate transport, eligibility and independent original-space
assessment. The main owners are `crates/pse-structural`, `crates/pse-backend-native`,
`crates/pse-runtime/src/math/` and `crates/pse-runtime/src/workflow/`, with numerical
vocabulary in `pse-model::numerics` and `pse-math::{numerics,normalization}`.

## 15. Structural analysis and diagnostics

Structural analysis answers whether the selected equations can determine the selected
unknowns before any value is computed, and gives initialization and recycle strategies
their block and cycle structure. Numerical diagnostics answer what a completed attempt
actually achieved in original coordinates. Both are consumers of the complete admitted
case from the compiler ([§14](mathematics-and-compilation.md#section-14)); neither
evaluates a partial inventory as if it were complete.

IDAES `DiagnosticsToolbox` parity is not a current capability: there is no
threshold relation, no named structural/numerical check catalogue and no
`report_*`/`assert_*` view family. The checks below are the ones the native lifecycle
actually performs.

### 15.2 Structural checks without numerical values

Structural admission is class-aware and consumes the compiler's existing analysis
rather than repeating matching (`pse-backend-native::structural`):

| Mode | Admission rule | Refusal |
|---|---|---|
| Roots (square equations, declared fixed point) | Every original equality row and free variable participates in the maximum matching | `ProblemError::Structural` naming overdetermined rows and underdetermined columns by semantic ID |
| NLP | Every equality row matches; inequalities and genuine optimization degrees of freedom remain admissible | Overdetermined equality rows by semantic ID |

A scope marked partial is refused; structure over a subset cannot certify the case.
Fixed-point and Picard admission observe the original residual support, not the
iteration map. A reused matching witness is checked linearly against the current row
inventory, equality membership and incidence edges before it is trusted; opaque
callback oracles receive the same analysis from their original residual Jacobian.
Physical unit consistency is established earlier by physical admission
([§8](physical-semantics.md#section-8)), so it is not a structural check here.

Structural matching never certifies numerical rank. A structurally square system can
still be singular at a point; that is a numerical observation (§15.4, §15.5).

### 15.3 Structural algorithms

`pse-structural::incidence` is a checked semantic boundary around pounce-presolve:

- **Incidence.** Complete selected-case equality rows (including rows with no
  incidence), free columns, conservative all-branch support of guarded expressions and
  objective/inequality coupling. Original contribution provenance (instance, output
  ordinal) survives even where matching deduplicates pairs.
- **Matching, Dulmage–Mendelsohn and BTF.** pounce-presolve's Hopcroft–Karp matching,
  DM partition (over-, under- and square-determined parts) and block triangular form.
  Library results are validated against the declared dimensions and equality coverage
  before they are converted back to semantic IDs.
- **Deterministic order.** rustworkx lexicographical topological sort breaks ties by
  semantic ID; petgraph and rustworkx supply components, condensation and cycle finding
  for flow and dependency projections (`projection`, `flowsheet`).
- **Block identity.** A block's `BlockId` hashes its scope and canonical membership, so
  identity is independent of execution position. A different valid topological order is
  not wrong; positional interpretation without the identity mapping is.
- **Coupling.** Blocks record conservative auxiliary coupling. Coupling and conditional
  partitions do not establish independent eliminability or independent subproblems.

The recursive matching search is qualified to 100,000 rows on a dedicated 32 MiB
thread stack; larger inputs are refused, not truncated. Graph admission checks node and
edge limits before allocation. Cancellation is checked between library stages.

### 15.4 Numerical checks that require values

After any native attempt, the adapter re-evaluates the original model at the returned
candidate through the original oracle (`pse-backend-native::quality`). The observation
never reuses a solver's possibly stale row buffer. It records:

- raw constraint values, original row bounds and signed equality residuals for finite
  equality rows only;
- per-side row and variable-bound violations in each source's declared physical unit,
  each paired with its resolved physical tolerance (§16.6);
- integrality violations for discrete domains;
- a dimensionless maximum of violation divided by its tolerance, the only aggregated
  quantity, because unlike units are never combined;
- original-coordinate stationarity and complementarity under the minimization
  convention `L = sense*f + lambda*g - z_lower*x + z_upper*x`, or an explicit dual
  error when multipliers or derivatives are missing or invalid.

Physical conservation checks (`runtime.physical_checks`) are computed separately from
mathematical feasibility by the workflow completion owner; balance closure is owned by
[§10](models-and-composition.md#section-10). Presolve infeasibility proofs are
confirmed on per-bound tolerance-expanded original normalized facts and retain original
row, column and contribution identities. Missing evidence carries a typed unavailable
reason; it is never zero or a fabricated NaN.

### 15.5 Advanced analyses

Each analysis is opt-in or bounded, and none replaces the original candidate:

| Analysis | Owner and mechanism | Limit |
|---|---|---|
| Infeasibility rays, IIS, basis ranging, feasibility relaxation | HiGHS native diagnostics (`highs::diagnostics`), restored to original coordinates | Diagnostic data only; a relaxed point is never a primal solution. MIP IIS concerns the LP relaxation. Penalties are explicit physical declarations, never inferred from units |
| Presolve rank diagnostics | pounce-presolve equality-rank pass | No objective-changing remedy |
| Numerical PSD qualification | Resource-bounded faer eigenanalysis with residual qualification | Explicit opt-in; never repairs the matrix (§18.10) |
| Fit response rank and conditioning | faer pivoted LU for implicit response, bounded SVD for scaled observation rank | Rank does not imply covariance, estimator sensitivity or global identifiability |

A singular-value toolbox over general Jacobians, degeneracy search, minimal
infeasible-system explanation and convergence studies over case sets are not present.

### 15.6 Reports

Structural refusals and native failures reach callers as structured values, not prose.
Every public error carries a `pse-diagnostics` code and failure class
([§23.2](operations-and-validation.md#section-23-2)); workflow boundary diagnostics
(`pse-model::diagnostic::BoundaryDiagnostic`) add the stage, violated rule, affected
semantic IDs, typed observations and authored source locations enriched from the model
revision (`pse-runtime/src/workflow/diagnostics.rs`). Diagnostic capture is bounded and
cannot change a scientific or publication outcome.

Declared relational invariants write `runtime.diagnostics_findings` through
`pse-rules::invariants` for inspection and fixture validation. Solve outcomes, metrics
and assessments are result relations ([§19](workflows-and-results.md#section-19)); see
the [generated runtime relations](../../generated/relations/runtime.md) for columns.

## 16. Numerical policy and scaling

> Decision: [ADR-0090](../../adr/0090-shared-execution-vocabulary.md) — resolved
> numerical requirements and candidate assessment extend the shared vocabulary.

Numerical meaning is resolved once per admitted analysis, before presolve or any
adapter runs. The single resolved interpretation then feeds model normalization, native
stopping controls and original-space acceptance. Adapters never reinterpret raw
tolerances or infer magnitudes from trial values.

### 16.1 Resolved numerical policy and normalization

`pse-model::numerics::NumericalPolicy` holds library-neutral controls: ID-keyed
analysis requirements, `strict_nominals`, `native_scaling`, independent KKT budgets
(stationarity, complementarity), an optional separately qualified acceptable-stop
budget, integrality, continuous and MIP gaps, and the closure policy.
`pse-math::numerics::resolve` combines it with the selected targets and sourced
declarations into an immutable `ResolvedNumericalPolicy`.

A target is a variable, row, objective, observable or closure coordinate with its full
quantity type and unit. Each resolved target records:

- a nominal, converted into the target's own unit, frozen for relative acceptance;
- a positive coordinate scale; integer coordinates keep scale one so the lattice
  survives;
- an absolute tolerance in physical units and a dimensionless relative tolerance;
- the frozen budget `absolute + relative * nominal`;
- selected and overridden provenance for every field.

`pse-math::normalization::Normalization` is positive diagonal coordinate transport:
`x = Sx z`, normalized rows `= rows / Sr`, normalized minimization objective `= f / Sf`.
It carries derivatives, sparse coefficients, bounds, starts, duals and FBBT tapes in
both directions; it is not a unit conversion and not a new evaluator. Native
algorithmic scaling (for example Ipopt's gradient-based scaling) is a separate,
explicitly permitted step after normalization. Nonpolyhedral cone blocks share one
positive row scale. Hard guards are never relaxed by scaling.

### 16.2 Sources and precedence

| Rank | `NumericalSource` | Origin |
|---|---|---|
| 5 | `Analysis` | Requirements in the request's `NumericalPolicy` |
| 4 | `Case` | Selected case declarations |
| 3 | `Model` | Explicit model target declarations (`authored.numerical_requirements`) |
| 2 | `PropertyDefault` | Explicitly bound provider-output/property defaults |
| 1 | `QuantityNominal` | Quantity registry nominal |
| 0 | `CanonicalFallback` | Recorded canonical-unit nominal of one |

Higher rank wins per field. Equal-rank conflicts, duplicate requirement identities,
requirements naming unselected coordinates, nonpositive or nonfinite magnitudes, a
unit on a normalized-coordinate requirement and a nominal that disagrees with its
scaling factor all fail resolution. A declaration cannot promote itself to an analysis
override. With `strict_nominals`, the canonical fallback is refused instead of
recorded. Scaling or initialization strategies attached to authored templates are
refused at composition admission; numerical policy is the only scaling authority.

### 16.5 Identity, provenance and persistence

`ResolvedNumericalPolicy::key` frames the complete interpretation, including
provenance, and enters prepared-request identity; native `Accuracy::key` separately
frames every stopping budget. Changing a nominal, tolerance or source therefore changes
reuse identity even when a native library fingerprint could not tell the difference.
The resolved interpretation, candidate assessments and physical checks are published
as `runtime.resolved_numerics`, `runtime.candidate_assessments` and
`runtime.physical_checks`; public serialization consumes the completed assessment and
performs no second evaluation. There is no scaling profiler; comparing policies is an
ordinary sequence of prepared analyses.

### 16.6 Derived native controls and original-space acceptance

From the resolved policy, `pse-backend-native::solve::Accuracy::resolve` derives
normalized native controls: feasibility is the minimum of the comparable normalized
variable/row budgets; stationarity, complementarity, integrality and gaps come from the
policy. Neither the raw maximum nor the raw minimum of differently dimensioned
tolerances is ever used. Derived library options:

| Adapter | Derived from the policy |
|---|---|
| Ipopt and POUNCE | `tol`, `constr_viol_tol`, `dual_inf_tol`, `compl_inf_tol`; `bound_relax_factor = 0` and `honor_original_bounds`; acceptable-level options only when an acceptable budget is declared |
| KINSOL | Variable and residual scales, strict sign constraints from exact sign bounds, scaled-step tolerance |
| HiGHS | Feasibility, integrality and MIP gap controls |
| Clarabel | Primal/dual residual and gap controls, with cone-block adjustments retained in provenance |

Options that encode these semantic controls are reserved; a caller-supplied native
option that conflicts with them is refused before the library sees it.

Acceptance is judged in original physical coordinates, not in the solver's normalized
space. `quality::Tolerances::from_policy` projects the frozen per-ID budgets onto one
oracle's order. Integration accuracy, integrated observables, instantaneous balances
and cumulative closure are distinct requirements. The workflow completion owner
(`pse-runtime/src/workflow/numerics.rs`) combines native outcome, numerical feasibility
and closure into one immutable `CandidateUse`:

| Condition | `CandidateUse` |
|---|---|
| Native outcome forbids use, or required numerical acceptance failed or is unavailable | `Unusable` |
| Required physical closure unavailable | `Unusable` |
| Closure failed the frozen budget, default `RequireClosed` | `Unusable` (candidate retained) |
| Closure failed, explicit `AllowUnclosed` | `QualifiedUnclosed` (no closure claim) |
| All required original-coordinate checks passed | `Usable` |

## 17. Initialization, starts and recycles

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md) — native
> heuristic/exact tear selection and sequential initialization;
> [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages and
> explicit starts.

Initialization produces a start; it never claims an optimum and never rewrites the
case. Strategies are derived from distinct structural projections and authored
policy; libraries own every iteration.

### 17.1 Structural initialization model

`pse-structural::initialization::Plan` converts a complete, structurally sound square
analysis into predecessor-first conditional blocks, each with its original rows, solved
columns and explicit predecessor inputs. Deficient, partial or non-square structure is
refused before any factorization. `pse-runtime::math::initialization::PreparedInitialization`
compiles each block as an ordinary library artifact (unselected variables bound as
fixed) and resolves every block's route before worker acquisition
([§18.7](#section-18-7)); a failed attempt does not trigger a fallback route.

Execution is transactional over immutable case bindings:

- a block commits only coordinates that passed independent original-quality checks;
  a failed block commits nothing;
- `InitializationReport` keeps the immutable original specification, the committed
  solved unknowns only (never fixed inputs or overlays), every stage overlay and stage
  candidate (including failed or cancelled stages), every block attempt with its route,
  and whether the final stage used the original bindings;
- a failed or cancelled stage retains its overlay as evidence and cannot return it as
  the authoritative specification.

Initialization accuracy comes from the numerical policy; it is serial, uses fresh native
allocation and admits a bounded finite schedule. The public entry is
`ModelRevision`'s revision-bound initialization strategy
(`workflow/strategies/conditional.rs`).

### 17.4 Flowsheet recycles and dynamic starts

`pse-structural::flowsheet::FlowGraph` is the admitted physical flow projection:
explicit connection occurrences (parallel occurrences are never merged), node isolates,
declared decision groups with finite nonnegative costs and `Free`/`Mandatory`/`Forbidden`
policy. Tear selection (`pse-backend-native::tears`) has two explicit routes:

- **Exact.** A HiGHS MILP with order variables, `r_u + 1 <= r_v + |V| t_group` per
  occurrence; model size is linear and cycles are not enumerated. MIP incumbent, bound,
  gap and optimality remain distinct observations.
- **Heuristic.** An explicit unweighted petgraph greedy feedback-arc route. Its computed
  authored cost is not an optimality claim.

Both results are independently checked: the remaining graph must be acyclic under
petgraph, forbidden groups are respected, and a cycle that the allowed policy cannot
break is reported concretely. A selected tear set feeds `RecycleRequest`, which names
each unit's causal direction port by port. `CausalMap` evaluates declared causal units
over the witnessed order with physical port conversions; KINSOL owns the fixed-point
iteration and Anderson acceleration. Input bounds cannot be enforced by fixed-point
iteration, so bounded causal inputs are refused with a request for a constrained
simultaneous strategy. No arbitrary residual is reinterpreted as a causal map.

Consistent initial conditions for dynamics belong to the integrators
([§13](workflows-and-results.md#section-13)).

### 17.5 Continuation

Continuation is a finite, supplied list of stages. Each stage replaces declared finite
fixed/parameter inputs (never solved unknowns) and runs the structural block sequence
transactionally. Prior values seed a stage only under the `PreviousAccepted` start
policy. There is no adaptive homotopy step control, step-size acceleration or
rollback-and-cut schedule; a caller who needs one supplies the stages.

### 17.6 Explicit starts and allocation reuse

Numerical start policy (`StartPolicy`: `NoPriorStart` by default, `PreviousAccepted`,
`Explicit`) is independent of native allocation reuse (`ReusePolicy`: `Fresh`,
`AllowRebuild`, `RequireReuse`). Reusing a retained native model never implies a warm
start, and a disallowed start clears retained native start state, including HiGHS basis
state.

A `WarmStart` is an owned, typed payload (root primal; NLP primal with optional bound
and row multipliers; HiGHS primal/dual/basis; POUNCE SQP iterate and working set) with a
compatibility record separating layout identity from numeric data, and an optional
origin run/attempt. Layout and backend must match exactly; numeric data may differ.
`StartReceipt` records what was actually submitted and the transformations applied,
distinct from the output seed a report offers for later use. Mutable native state never
crosses a worker boundary. Fitting starts are refused; declared parameter guesses are
its input.

## 18. Native class-specific execution

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md) — native
> class-specific execution; [ADR-0090](../../adr/0090-shared-execution-vocabulary.md) —
> shared execution vocabulary; [ADR-0093](../../adr/0093-qualified-native-strategies.md)
> — contextual eligibility and qualification.

Each mathematical class has its own representation and native adapter under one
runtime lifecycle. There is no universal problem object, no solver-neutral file format
and no fallback engine. [ADR-0083](../../adr/0083-class-specific-native-execution.md)
owns the rationale; [ADR-0082](../../adr/0082-library-owned-process-mathematics.md) owns
library ownership of the mathematics.

### 18.1 Problem representations

The compiler's immutable case products are projected into class-specific views
(`pse-backend-native` crate root, `assembled`):

| Representation | Meaning | Consumed by |
|---|---|---|
| `OracleContract` | Content identity, source variables with bounds in native column order, row IDs, available derivative order and admitted smoothness (derivatives and smoothness are separate claims) | All callback adapters |
| `NlpOracle` | Objective, constraints, gradient, Jacobian and weighted Lagrangian Hessian as separate demands, with sparse patterns, normalization, presolve facts and structural analysis | Ipopt, POUNCE |
| `NleOracle` | Square residuals with an assembled Jacobian or Jacobian-vector product | KINSOL |
| KINSOL `Function` | Residual equations, explicit Picard splitting `F(x) = Lx - N(x)` with a declared constant `L`, or a declared causal map with an original residual validator | KINSOL |
| `CoefficientProblem` | Sparse LP/MILP/QP coefficients with objective sense and constant, variable domains (including semi-continuous/semi-integer) and consumed parameter assumptions | HiGHS |
| `ConicProblem` | Explicit data in Clarabel's own matrix and cone types | Clarabel |

Coefficient views are derived only through admitted affine/degree-two proofs, and cones
are declared by an explicit request; neither is inferred from NLP rows. All-fixed
models are evaluated directly as a constant route. Dynamic representations are owned
by [§13](workflows-and-results.md#section-13).

### 18.2 Evaluation programs and callback boundary

Value and derivative programs are Symbolica/Numerica artifacts prepared by the
compiler ([§7](mathematics-and-compilation.md#section-7)); physical providers are
FeOS/num-dual workers ([§9](physical-semantics.md#section-9)). Each attempt owns cloned
evaluators, provider workers and scratch; shared programs stay immutable.

`pse-backend-native::callback::CallbackState` is the single trial-evaluation boundary
for all callback adapters:

- work runs under `catch_unwind`; a Rust panic becomes a `Panic` termination and never
  unwinds across a C or library callback;
- typed causes (never diagnostic strings) classify failures: domain violations and
  provider trial, envelope or singular failures are recoverable trials; cancellation is
  cancellation; everything else is fatal evaluation failure;
- outputs are published only after a successful evaluation, so a failed trial cannot
  leave partial buffers or return the previous trial's values;
- only terminal failures latch; later successful trials preserve native success;
- per-demand call counts and wall time, including rejected trials, become metrics.

### 18.3 In-process NLP: Ipopt C and POUNCE

`pse-ipopt-sys` holds generated, committed bindgen output for the Ipopt 3.14.20 C
interface (`just codegen --only bindgen`; see [ADR-0028](../../adr/0028-solver-acquisition-source-built-ipopt.md)
for the digest-pinned solver image). ABI tests assert 32-bit indices, `f64` numbers,
the version string and real symbol relocations. `pse-backend-native::ipopt` is the safe
driver: RAII problem ownership on the owning worker, direct callbacks over `NlpOracle`,
exact or limited-memory Hessians, primal/dual starts, typed scaling, reserved options
and an intermediate callback that checks cancellation and deadline. The pinned MUMPS
profile is serial.

POUNCE (`pounce`, `tnlp`) shares the same `NlpOracle`, callback policy and presolve
pipeline, and adds interior-point and active-set SQP methods, FERAL linear algebra on an
admitted Rayon pool, restoration statistics and working-set starts. Continuous NLP for
both adapters passes through pounce-presolve's qualified wrappers
(`presolve::pipeline`): policy `Off`, `Auto` (qualified source-backed passes only) or
`Explicit` with required passes that must qualify. The library owns reductions,
derivative transport and recovery; the project records effects and inverse source
attribution, not a second transformation IR. Auxiliary reduction is never automatic.

Two tested limits are qualification distinctions, not retries: with automatic presolve
a recovered bound multiplier can fail original complementarity (the candidate stays
feasible, not stationary), and HiGHS' default QP regularization can miss a stricter
requested objective gap.

### 18.6 Truthful outcomes

`pse-backend-native::solve::SolveReport` is one envelope per attempt, including attempts
without a usable candidate. Its facts are independent:

| Fact | Representation |
|---|---|
| Native termination | Raw code, native name and message, plus a shared `NativeTermination` category (success, acceptable, feasible-only, infeasible/unbounded variants, limit variants, resource exhaustion, inconclusive, cancelled, numerical, evaluation, panic, invalid) |
| Candidate kind | `FinalIterate`, `BestIterate`, `FeasiblePoint` or `ConstantEvaluation`: what the API actually supplied |
| Feasibility | Original-space `Quality` (§15.4) |
| Stationarity / optimum / gap | `Qualification`: `Unqualified` < `Feasible` < `Stationary`, `OptimalWithinTolerance`, `GapQualified` |
| Certificates | Native infeasibility/unboundedness certificates, never exposed as primal solutions |
| Completeness | Unattempted sequence steps are counted; partial trajectories and dropped events are explicit |
| Evidence gaps | `EvidenceUnavailableReason` on metrics and observations; absent means unavailable, never zero |

`quality::qualify` grants only what original observations support. A validation error,
missing candidate or infeasible quality yields `Unqualified`. Feasibility yields
`Feasible`. Beyond that, the native stop must be success (or acceptable with a declared
acceptable budget) and, per class: NLP needs original stationarity and complementarity
within budget; HiGHS needs a verified upload-equivalence readback plus either a MIP gap
within budget or LP/QP dual feasibility and primal-dual error within budget; Clarabel
needs residuals and a gap within budget. Roots top out at `Feasible`. A postsolve or
validator failure preserves the native outcome and clears assurance; optional
diagnostic failure never replaces the original solve.

The report also retains effective options and queried native defaults, provenance,
bounded events, complete native statistics where the library exposes them, the start
receipt and an output seed. Fit response derivatives remain candidate data distinct from
estimator qualification. The shared tags are registry-owned and projected into Rust,
Arrow and Python ([ADR-0090](../../adr/0090-shared-execution-vocabulary.md)); candidate
use (§16.6) combines these facts with physical closure.

### 18.7 Capability, eligibility and selection

Capability is five distinct facts:

| Fact | Owner |
|---|---|
| Requested | `SolverProfile`: intent (`Optimize`, `Root`, `FeasiblePoint`, `Initialize`), `SolverSelection::{Auto, Explicit}`, controls and typed backend settings |
| Available | `BackendCapabilities::available`: the adapter is linked by feature; `runtime.solver_capabilities` exposes the static inventory |
| Admitted | Compiler facts (`pse-math::facts::ProblemFacts`) and structural admission (§15.2): domains, bound shapes, prepared derivative order, coefficient eligibility, convexity evidence |
| Eligible | `routing::Requirements::eligibility`: every applicable reason per backend for this model, profile and thread count |
| Selected | `routing::Requirements::select` returns `Constant` or `Native(backend)` |

Automatic selection is deterministic: roots and initialization prefer KINSOL, then
Ipopt, then POUNCE among eligible adapters, so boxed roots route to a constrained NLP
adapter rather than dropping bounds; optimization prefers HiGHS for admitted
coefficient classes, then Ipopt, then POUNCE. Clarabel is reached only through an
explicit cone request. An explicit selection is never substituted: an unlinked choice
fails with `Unavailable` listing eligible alternatives, and an ineligible choice fails
with every reason. Unsupported bound, derivative, nonsmooth and thread combinations
fail during preparation, before worker acquisition. Typed backend settings must match
the selected route.

Strategies are derived from distinct projections rather than topology alone: square
root blocks, declared causal fixed-point and Picard maps (§17.4), simultaneous
constrained NLP blocks and supplied continuation (§17.5). An SCC does not by itself
select KINSOL. `SolveSequence` runs finite prepared steps on one admitted worker and
continues after a failure only when steps are declared independent.

### 18.8 Threading, cancellation and resource ownership

`MathService` (`pse-runtime/src/math.rs`, `math/jobs.rs`) is the single execution
owner. Its `MathPolicy` draws finite allowances from the deployment memory pool:
artifact retention, per-job foreign allowance, worker storage, workspace generations,
native stack, live jobs and flights.

- A job acquires a job slot, CPU permits for its admitted cores and a pool reservation
  covering stacks (one per worker plus a coordinator for parallel teams), numeric bytes
  and the foreign allowance; parallelism cannot exceed the effective process CPU count.
- Mutable native state is constructed, used and destroyed on the owning `pse-math`
  thread. Only owned `Send` inputs and results cross; native objects may be `!Send`.
  Salsa never holds mutable solver state.
- Dropping a `SolveHandle` requests cancellation; awaiting `finish` observes completion
  only after native teardown, thread-local destruction and join. Permits and
  reservations survive caller cancellation until that join. Retained results transfer a
  split of the job reservation to an `AllocationLease` that lives with the result.
- Cancellation and deadlines are checked only at library-supported checkpoints
  (Ipopt intermediate callback, POUNCE TNLP callbacks, KINSOL evaluations, HiGHS
  interrupt callbacks with a native time limit for QP, Clarabel's termination callback,
  integrator step boundaries). A long native factorization completes before teardown.
- Ipopt, KINSOL and Clarabel profiles are serial; POUNCE and HiGHS may use admitted
  threads. Foreign BLAS/OpenMP threading is environment configuration, not admitted
  by this owner.

Reservations are conservative admission policy, not allocator interception or a
process RSS ceiling. The workstation sizing rationale and measured behavior are
qualification matters ([§24.2](operations-and-validation.md#section-24-2)).

### 18.9 Capability matrix

The linked inventory is `BackendCapabilities::capabilities` in
`pse-backend-native/src/solve.rs`; feature `pse-runtime/native-solvers` links the full
profile, and Clarabel's non-SDP route is always present.

| Backend | Classes | Bounds | Derivatives | Starts | Threads |
|---|---|---|---|---|---|
| Ipopt | Smooth NLP | General | Exact Hessian or limited memory | Primal/dual | Serial |
| POUNCE | Smooth NLP | General | Exact Hessian or limited memory | Primal/dual, working set | Admitted pool |
| KINSOL | Square root, declared fixed point | Sign only | Jacobian or JVP | Primal | Serial |
| HiGHS | LP, MILP, convex QP | General, semi domains | Coefficients | Primal/dual, basis | Admitted |
| Clarabel | Explicit cones (SDP with `solver-sdp`) | General | Coefficients | None | Serial |
| Diffsol | ODE, semi-explicit index-1 | None | First, smooth sensitivities | None | Serial |
| IDAS | ODE, semi-explicit index-1 | None | First, smooth sensitivities | None | Serial |

Outside the matrix: general or higher-index DAE, global MINLP, disjunctive programs,
arbitrary cone recognition, finite-difference derivatives, GPU and distributed
execution.

### 18.10 Root, coefficient and cone adapters

**KINSOL** (`kinsol`) owns Newton, line search, Picard and fixed-point iteration with
Anderson acceleration, over vendored KLU, bounded dense or matrix-free SPGMR. Only exact
sign bounds are representable; arbitrary boxes and constrained fixed-point/Picard are
refused. Compatible layouts reuse SUNDIALS/KLU allocations on the owning worker.

**HiGHS** (`highs`) receives a checked native upload whose full readback must match the
coefficient view before any bound or optimality claim transfers. It supports LP
(choose, simplex, IPM, PDLP), MILP with binary and semi domains, and convex QP.
Convex QP requires evidence: an exact rational `GramCertificate` (`sign*Q = Rᵀ diag(w) R`
with nonnegative weights, checked through Symbolica/Numerica against the unchanged
matrix and objective orientation) by default, or an explicitly requested numerical PSD
qualification with tolerances. An indefinite or inconclusive matrix is never repaired.
Scheduler teardown is exclusive and blocking.

**Clarabel** (`conic`) receives explicit cones with CSC-format, dimension and parameter
checks; SDP uses packed PSD-triangle cones in the serial LP64 netlib profile.
`SingleSolve` permits native presolve/chordal preprocessing; `ReusableData` disables
them to use native data updates. Results are post-processed into source space.

Every adapter recovers candidates, duals and certificates to original coordinates
through `transport` before quality is assessed.

## Retired section identities

#### 15.1 Thresholds (defaults preserved from IDAES) — retired

No IDAES diagnostic threshold set exists; acceptance tolerances come from the resolved
numerical policy in [§16.1](#section-16-1) and [§16.6](#section-16-6).

#### 16.3 Nominal value algebra — retired

Nominals are declared and resolved by precedence ([§16.2](#section-16-2)); no
expression-walking nominal or constraint-scaling scheme exists.

#### 16.4 Scaler templates — retired

Template-attached scaling strategies are refused at composition admission; the
resolved numerical policy ([§16.1](#section-16-1)) is the only scaling authority.

#### 17.2 Standard plan templates — retired

Per-unit initializer templates were replaced by structure-derived block initialization
([§17.1](#section-17-1)); template initializers are refused at composition admission.

#### 17.3 Plug-ins and initialization order — retired

Initialization order is the structural predecessor order of [§17.1](#section-17-1);
there is no plug-in prepare/finalize protocol.

#### 18.4 NL backend — retired

Production NL/Pyomo routes were removed in favor of native class-specific adapters
([§18.1](#section-18-1), [ADR-0083](../../adr/0083-class-specific-native-execution.md)).

#### 18.5 Kernel adapters generated from `KernelSpec` — retired

Library-owned evaluation programs and provider workers ([§18.2](#section-18-2),
[§7](mathematics-and-compilation.md#section-7)) replaced generated kernel adapters.
