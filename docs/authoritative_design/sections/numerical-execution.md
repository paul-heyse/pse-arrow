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

> Decision: [ADR-0107](../../adr/0107-sensitivity-covariance-uncertainty.md) — parametric
> sensitivity, reduced Hessian, covariance and uncertainty propagation with PS-12
> validity; [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md) — whole-model
> infeasibility explanation through the explicit POUNCE ℓ1 route. Plan 22 S1–S4, N3; not
> yet implemented.

Each analysis is opt-in or bounded, and none replaces the original candidate:

| Analysis | Owner and mechanism | Limit |
|---|---|---|
| Infeasibility rays, IIS, basis ranging, feasibility relaxation | HiGHS native diagnostics (`highs::diagnostics`), restored to original coordinates | Diagnostic data only; a relaxed point is never a primal solution. MIP IIS concerns the LP relaxation. Penalties are explicit physical declarations, never inferred from units |
| Presolve rank diagnostics | pounce-presolve equality-rank pass | No objective-changing remedy |
| Numerical PSD qualification | Resource-bounded faer eigenanalysis with residual qualification | Explicit opt-in; never repairs the matrix (§18.10) |
| Fit response rank and conditioning | faer pivoted LU for implicit response, bounded SVD for scaled observation rank | Rank does not imply covariance, estimator sensitivity or global identifiability |

Authored diagnostic profiles also drive bounded faer Jacobian SVD, selected linear and
Jacobian optimization analyses, and nonlinear elastic/deletion explanations through
existing native adapters. Reports distinguish observations, candidates, inconclusive
limits and unattempted work; a deletion heuristic is not a proof of a minimum infeasible
subsystem. Finite studies retain per-case outcomes and explicit predecessor dependence.
The generic profile/knowledge contract is proposed
[ADR-0101](../../adr/0101-modeling-analysis-knowledge.md).

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

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> resolved
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
recorded. Authored nominal, scale and initialization annotations resolve into this same
policy. Block and causal/recycle projections carry original units, physical bounds,
integrality and provenance; they cannot manufacture a second default interpretation.

### 16.5 Identity, provenance and persistence

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> completion records the actual request, effective settings and submitted start. Plan 22
> A4 (implemented) derives settings identity from serde and frames the start a step used,
> and any reuse of retained native state, into its lineage identity.

`ResolvedNumericalPolicy::key` frames the complete interpretation, including
provenance, and enters prepared-request identity. Changing a nominal, tolerance or source
therefore changes reuse identity even when a native library fingerprint could not tell
the difference. The native stopping budgets are derived from that interpretation
([§16.6](#section-16-6)), so the policy key already covers them; `ResolvedAccuracy::key`
frames the budgets themselves where a stamp needs them without the policy, as for the
session stamp of a declared root ([§17.4](#section-17-4)).

Execution settings are identified through serde, never through a hand-written field list:
`Controls::identity` (option values keep their native type and exact float bits),
`BackendSettings::identity` and `ResolvedAccuracy::key`. A new field therefore enters
identity without an edit to a hashing function. The solver profile identity
(`pse.solver.profile.v2`) frames the native session profile ([§17.6](#section-17-6)), every
control and the selection by its registry spelling.

The lineage request identity of a completed algebraic step (`pse.completed.request.v2`,
published in `runtime.run_lineage`) frames the revision, instance, preparation, selected
request, profile and resolved policy together with the start the step actually used:
whether it reused retained native state, its predecessor attempt, whether the start was
submitted, the submitted seed by content and any partial start. Seed content
(`WarmStart::content_key`) excludes the run that produced the seed. The same request
seeded differently is therefore a different lineage, and the same seed produced by another
run is the same lineage ([§20.3](identity-and-publication.md#section-20-3)).

The resolved interpretation, candidate assessments and physical checks are published
as `runtime.resolved_numerics`, `runtime.candidate_assessments` and
`runtime.physical_checks`; public serialization consumes the completed assessment and
performs no second evaluation. Authored scaling schemes and diagnostics consume the
same resolved policy; comparing policies is a sequence of explicitly prepared analyses.

### 16.6 Derived native controls and original-space acceptance

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> `CandidateUse` gains `seed_only` and `diagnostic_only` and becomes the only acceptance
> rule every workflow consumes (Plan 22 A1, implemented). The incumbent of a relaxed global
> export, which the decision also makes `diagnostic_only`, arrived with the SCIP route (Plan
> 22 G3, implemented).
> Stopping budgets are resolved from the policy and are not a user control (Plan 22 A4,
> implemented).

From the resolved policy, `pse-backend-native::solve::ResolvedAccuracy::resolve` derives
normalized native stopping budgets: feasibility is the minimum of the comparable normalized
variable/row budgets; stationarity, complementarity, integrality and gaps come from the
policy. Neither the raw maximum nor the raw minimum of differently dimensioned
tolerances is ever used. `ResolvedAccuracy` is not a user control. The caller's `Controls`
choose time and iteration limits, threads, history, the Hessian mode, reuse, the start
policy and native options, and carry no accuracy, so nothing a caller supplies can stand in
for the resolved value. Preparation resolves it once (`PreparedSolve::accuracy`; per block
for initialization; from the map's own policy for a causal map), and every adapter, runner
and qualification receives that value. A nested library solve resolves its own from its
budgets (`ResolvedAccuracy::from_policy`). Derived library options:

| Adapter | Derived from the policy |
|---|---|
| Ipopt and POUNCE | `tol`, `constr_viol_tol`, `dual_inf_tol`, `compl_inf_tol`; `bound_relax_factor = 0` and `honor_original_bounds`; acceptable-level options only when an acceptable budget is declared |
| KINSOL | `kinsol::Settings::from_policy`: each variable and residual scale is `feasibility * s / t` and the scaled-step tolerance is `feasibility`, so KINSOL's residual and step tests reduce to the original per-row and per-coordinate budgets; sign constraints from one-sided bounds on shifted coordinates and from guard signs ([§18.10](#section-18-10)). Callers choose only the method (`kinsol::Method`) |
| HiGHS | Feasibility, integrality and MIP gap controls |
| Clarabel | Primal/dual residual and gap controls, with cone-block adjustments retained in provenance |

Options that encode these semantic controls are reserved; a caller-supplied native
option that conflicts with them is refused before the library sees it.

`kinsol::Settings::from_policy` is the one KINSOL configuration. The root runner,
conditional initialization blocks ([§17.1](#section-17-1)) and the nested implicit solve
inside an evaluation all derive from it; the nested solve takes the budgets and nominals of
its unknowns and rows from the resolved policy's targets, and its normalized feasibility
budget is the smallest tolerance-to-nominal ratio of its unknowns; nested solves on one
worker reuse a cached KINSOL session per problem layout ([§18.10](#section-18-10)). The
derived controls reach every adapter through the shared representation runners
([§18.7](#section-18-7)). One NLP runner, `execution::nlp`, serves staged-sequence steps
(single solves, authored sequences, studies and initialization blocks), fitting and the
factorable runner's fixed-assignment re-solve ([§18.10.1](#section-18-10-1)): presolve
pipeline (or its terminal report), native solve, library recovery with an independent
original observation, KKT evidence and qualification. No workflow derives
native controls or acceptance of its own.

Acceptance is judged in original physical coordinates, not in the solver's normalized
space. `quality::Tolerances::from_policy` projects the frozen per-ID budgets onto one
oracle's order. Integration accuracy, integrated observables, instantaneous balances
and cumulative closure are distinct requirements.

The workflow completion owner (`pse-runtime/src/workflow/numerics.rs`) is the single
owner of candidate use. It combines native outcome, original numerical feasibility,
required model checks and physical closure into one immutable `CandidateUse`, with a
stable reason published as text and never parsed:

| `CandidateUse` | Meaning | Permits |
|---|---|---|
| `usable` | Every required original-coordinate check passed | Result and seed |
| `qualified_unclosed` | Numerically qualified; closure failed under explicit `AllowUnclosed` (no closure claim) | Result and seed |
| `seed_only` | Feasible in original coordinates, but the native stop forbids use as a result | Seed only; never published as a solution |
| `diagnostic_only` | The least-infeasible point a native infeasibility stop returns, or the incumbent of a relaxed global export | Observation only; never a seed or a result |
| `unusable` | Anything else | Neither |

The decision is made in two ordered steps.

**Native step.** For one native attempt the first applicable rule decides:

1. independent original-model validation failed, or the API supplied no candidate:
   `unusable`;
2. the native stop is `infeasible`: `diagnostic_only`;
3. the candidate is the incumbent of a relaxed global export
   (`PrimalSource::RelaxedIncumbent`, [§18.10.1](#section-18-10-1)): `diagnostic_only`;
4. original-coordinate feasibility failed or is unavailable: `unusable`;
5. qualification is `Unqualified`: `unusable`;
6. otherwise the native stop category decides: success, acceptable or feasible-only is
   `usable`; an iteration, time, solution, objective or general limit, resource
   exhaustion, an inconclusive or a numerical stop is `seed_only`; unbounded,
   infeasible-or-unbounded, cancelled, evaluation, panic or invalid is `unusable`. The
   match over `NativeTermination` is exhaustive, so a new stop category must be assigned.

An all-fixed constant evaluation has no native stop, so original quality alone decides
between `usable` and `unusable`. A dynamic trajectory is `usable` only after a completed
integration without a typed failure. A refusal before any native report is `unusable`.

**Completion step.** Required model checks and physical closure can only refuse or
qualify a `usable` native decision. Any other native decision passes through unchanged;
checks never upgrade a `seed_only` or `diagnostic_only` point.

| Condition on a `usable` native decision | `CandidateUse` |
|---|---|
| Required original-model checks failed or incomplete | `unusable` |
| Required physical closure unavailable | `unusable` |
| Closure failed the frozen budget, default `RequireClosed` | `unusable` (candidate retained) |
| Closure failed, explicit `AllowUnclosed` | `qualified_unclosed` |
| Closure closed or not required | `usable` |

Every workflow consumes this decision; none re-derives acceptance:

- a result (a commit, an advance, a publication) needs `usable` or `qualified_unclosed`.
  This covers staged-sequence steps, block-initialization commits (which add the block's
  own coordinate and finiteness checks), authored initialization stages and homotopy
  advances, fitting estimates, the feasibility witness of a nonlinear explanation and
  `RunResult::usable`;
- a study's dependent point may be seeded from a predecessor that is a result or
  `seed_only`; any other predecessor refuses the dependent point with a `conflict`
  diagnostic;
- `StartPolicy::PreviousAccepted` ([§17.6](#section-17-6)) seeds only from results; a
  `seed_only` candidate never becomes a sequence warm start or a committed block value.

Fitting keeps the native decision when it forbids use and otherwise decides from the
fresh original-model quality of the final evaluation. `runtime.candidate_assessments`
publishes each step's native termination, numerical feasibility, closure, policy, use
and reason as distinct columns.

## 17. Initialization, starts and recycles

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md) — native
> heuristic/exact tear selection and sequential initialization;
> [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages and
> explicit starts.

Initialization produces a start; it never claims an optimum and never rewrites the
case. Strategies are derived from distinct structural projections and authored
policy; libraries own every iteration.

### 17.1 Structural initialization model

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) —
> restates D13: initialization stages, including discrete fixing, are immutable overlays;
> [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages. Plan 22
> A6 (implemented) runs every block as a solve step of one staged sequence on a retained
> native session.

`pse-structural::initialization::Plan` converts a complete, structurally sound square
analysis into predecessor-first conditional blocks, each with its original rows, solved
columns and explicit predecessor inputs. Deficient, partial or non-square structure is
refused before any factorization. `pse-runtime::math::initialization::PreparedInitialization`
compiles each block as an ordinary library artifact (unselected variables bound as
fixed) and resolves every block's route before worker acquisition
([§18.7](#section-18-7)); a failed attempt does not trigger a fallback route. Typed backend
settings must belong to every block's route. Each block runs through the shared runner of
its route's representation, the same runners a solve uses: `execution::roots` for KINSOL,
with scales derived per block by `kinsol::Settings::from_policy`
([§16.6](#section-16-6)), or the one NLP runner `execution::nlp` for Ipopt and POUNCE, with
library presolve off.

Execution is transactional over immutable case bindings:

- a block commits only coordinates that passed independent original-quality checks;
  a failed block commits nothing;
- `InitializationReport` keeps the immutable original specification, the committed
  solved unknowns only (never fixed inputs or overlays), every stage overlay and stage
  candidate (including failed or cancelled stages), every block attempt with its route,
  and whether the final stage used the original bindings;
- a failed or cancelled stage retains its overlay as evidence and cannot return it as
  the authoritative specification.

Initialization accuracy comes from the numerical policy, and the schedule is finite and
bounded. `MathService::initialize` admits the whole schedule once, then runs it as one
staged sequence on one native session ([§18.8](#section-18-8)): each stage composes its
overlay over the immutable original values, and each block is an ordinary solve step
(`MathService::prepare_conditional`, run by the shared step executor
`MathService::execute`) bound to those values. Each block's view is bound once
(`PreparedBlock::bind`) and rebound to later stages' values
([§14.4](mathematics-and-compilation.md#section-14-4)). Blocks run in predecessor order on
the session's thread, and a block's solved coordinates become its successors' inputs only
after its candidate is a result. Retained native state serves a later step with the same
backend, coordinates and profile (the same block in a later stage) when that step's
`ReusePolicy` allows it; the default, `Fresh`, rebuilds. A step whose candidate is not a
result drops the retained state.

A case resolved for the initialize intent refuses a free discrete variable
(`modeling.domain`, [§6.8](schema-and-relations.md#section-6-8)); the ADR-0103 stage that
fixes discrete variables as a scoped overlay, restored on every exit (Plan 22 M2), is not
yet implemented. The public entry is the package-bound block initialization strategy
(`workflow/strategies/conditional.rs`). Authored stage and homotopy initialization
(`ModelingPackage::initialize_model`, `workflow/modeling/engines.rs`) runs on the same
staged-sequence primitive and retains original-specification acceptance
([§17.5](#section-17-5)).

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
([§13](workflows-and-results.md#section-13)). The starts they consume carry typed
provenance. Case resolution records a `StartSource` for every resolved input
(`workflow/modeling/cases.rs`): `ModelDefault`, `Case` (with its authored path),
`Annotation` (with the start annotation's declaration), `Predecessor` (the solved values
of an earlier staged step, [§17.6](#section-17-6)), `Continuation` (a continuation
override) or `Stored`, reserved for
starts read from the operational store (Plan 22 O6; not yet implemented). Workflows branch
on this type, never on a label. An integrated state without an isolated initial equation
takes its lower-endpoint start from its start annotation, evaluated through the model at the
endpoint, when its source is `Annotation`; every other source supplies the resolved
constant. An annotation source whose model output is missing is refused, not frozen.

### 17.5 Continuation

> Decision: [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages
> and explicit starts; [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> candidate use decides every advance. Plan 22 A6 (implemented) runs stages and homotopy as
> steps of one staged sequence.

Finite supplied continuation replaces declared fixed/parameter inputs stage by stage and
executes the structural block sequence transactionally ([§17.1](#section-17-1)).

Authored initialization (`ModelingPackage::initialize_model`) runs the named stages, then
bounded adaptive homotopy over the declared continuation endpoints, then the unchanged
original specification, as the steps of one staged sequence (`workflow::staged::Staged`).
Every attempt composes an overlay over the immutable original specification
([§19.1](workflows-and-results.md#section-19-1)): a stage attempt selects `stage.<name>`,
and a homotopy attempt replaces the continuation parameters by their values at the current
fraction of the path. Homotopy steps therefore change values only. They share one prepared
view ([§14.4](mathematics-and-compilation.md#section-14-4)) and, when the step's reuse
policy allows, the retained native session (`homotopy_steps_reuse_session`). The policy
bounds the initial and minimum step, the growth after an accepted step, the attempts and the
wall time, which covers binding, native execution and assessment of every step.

An attempt after an accepted one starts from the solved values of the last accepted attempt
(`Start::Accepted`, [§17.6](#section-17-6)); before any acceptance an attempt starts from the
specification. A failed attempt keeps its overlay as evidence and seeds nothing. A failed
stage, a failed first homotopy point or a non-retryable failure stops initialization with its
cause; a retryable homotopy failure halves the step, down to the minimum step. Stage and
homotopy attempts answer to the model's checks, not to final-fixture expectations; only a
separate solve of the unchanged original specification, answering to every obligation, may
commit. This is local initialization evidence, not a convergence guarantee.

### 17.6 Explicit starts and allocation reuse

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> completion records the submitted start. Plan 22 A4 (implemented) separates the
> coordinate and profile stamps and types the seed transformations; Plan 22 A6
> (implemented) types where a staged step starts (`Start`) and runs every step through one
> executor.

Numerical start policy (`StartPolicy`: `NoPriorStart` by default, `PreviousAccepted`,
`Explicit`) is independent of native allocation reuse (`ReusePolicy`: `Fresh`,
`AllowRebuild`, `RequireReuse`). Reusing a retained native model never implies a warm
start, and a disallowed start clears retained native start state, including HiGHS basis
state.

Each prepared native step carries a `Compatibility` stamp in four parts:

| Part | Frames | Governs |
|---|---|---|
| `layout` | Seed coordinates: backend, objective sense, free variables with their domains and physical maps, rows, body keys and sparsity patterns (`pse.solver.coordinates.v1`) | Whether a seed fits |
| `profile` | The native session: presolve policy, numerical policy, convexity policy, intent, the session-relevant controls and typed backend settings (`pse.solver.session.v1`) | Whether retained native state fits |
| `data` | Parameters, fixed values, coefficients, bounds and provider configurations of this attempt | Nothing; it may differ |
| `backend` | The native backend | Both |

A seed needs only `layout` and `backend` to match (`WarmStart::validate`), so a step that
changes a native option or setting keeps its predecessor's seed. Retained native state
needs `layout`, `profile` and `backend` to match (`Compatibility::same_session`); a changed
profile rebuilds the session, or `RequireReuse` refuses. The profile leaves out the
per-attempt budgets (time and iteration limits, history) and the sequencing policies
(reuse and start), because every attempt re-applies them. An explicit cone request
normalizes its coordinates at preparation, so its layout also frames the resolved policy.

A `WarmStart` is an owned, typed payload (root primal; NLP primal with optional bound
and row multipliers; HiGHS primal/dual/basis; POUNCE SQP iterate and working set) with its
compatibility stamp and an optional origin run/attempt. `StartReceipt` records what was
actually submitted, distinct from the output seed a report offers for later use: the
predecessor attempt, the seed, an explicit partial MIP seed, whether the native API
received it and the typed `SeedTransformation` path the seed took. That path is
`normalization` (by its identity), followed by `presolve` (its transformation identity and
the passes the library applied) when a pass was applied. It is recorded from what ran,
only when a seed or partial seed was supplied, never as a constant label. Its published
form is owned by [§20.3](identity-and-publication.md#section-20-3). Mutable native state
never crosses a worker boundary. Fitting starts are refused; declared parameter guesses
are its input.

**Starts in staged sequences.** Where a staged step starts is typed
(`workflow::staged::Start`), never inferred from a label: `Specification`, the
specification's own values and start annotations; `Accepted(k)`, the solved values of an
earlier step whose candidate is a result (stage chains and homotopy advance); or `Seed(k)`,
an explicit dependency on an earlier step whose candidate is a result or `seed_only` (study
points). The candidate-use decision ([§16.6](#section-16-6)) decides; a start it does not
permit refuses the step, which is recorded and seeds nothing. Seeded values enter case
resolution as `StartSource::Predecessor` for every variable the case leaves free
([§17.4](#section-17-4)).

Such a value start is independent of the native warm start. Every step, a single solve
included, runs through `MathService::execute`, which applies the step's policies: `Fresh`
reuse drops retained native state first; `NoPriorStart` submits nothing; `Explicit` submits
the step's own seed; and `PreviousAccepted` submits the output seed of the sequence's
previous step, offered only when that step's candidate is a result (`Staged::predecessor`),
with that attempt recorded in the receipt. A seed that does not fit the step's coordinates
rejects the step and drops the retained state. Authored sequences offer each step its
predecessor's seed; stage, homotopy and study steps seed values only.

Initialization distinguishes a start from a seed. A block starts from its staged values,
and that initial point is not a warm start. Only under `PreviousAccepted`, from the second
stage on, do the block's committed values from an earlier stage make its start a seed, with
that attempt as its origin and its transformation path recorded. Otherwise the receipt
records no seed and nothing submitted.

## 18. Native class-specific execution

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md) — native
> class-specific execution; [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> shared execution vocabulary; [ADR-0093](../../adr/0093-qualified-native-strategies.md)
> — contextual eligibility and qualification.

Each mathematical class has its own representation and native adapter under one
runtime lifecycle. There is no universal problem object, no solver-neutral file format
and no fallback engine. [ADR-0083](../../adr/0083-class-specific-native-execution.md)
owns the rationale; [ADR-0082](../../adr/0082-library-owned-process-mathematics.md) owns
library ownership of the mathematics.

### 18.1 Problem representations

> Decision: [ADR-0103](../../adr/0103-variable-domain-facet.md) — authored variable
> domains reach `CoefficientProblem` and routing (Plan 22 M1, implemented);
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — lowered
> constraint forms and native-form metadata (Plan 22 M3 and M4, implemented);
> [ADR-0105](../../adr/0105-scip-factorable-backend.md) — the factorable representation
> (Plan 22 G2 and G3, implemented).

The compiler's immutable case products are projected into class-specific views
(`pse-backend-native` crate root, `assembled`):

| Representation | Meaning | Consumed by |
|---|---|---|
| `OracleContract` | Content identity, source variables with bounds in native column order, row IDs, available derivative order and admitted smoothness (derivatives and smoothness are separate claims) | All callback adapters |
| `NlpOracle` | Objective, constraints, gradient, Jacobian and weighted Lagrangian Hessian as separate demands, with sparse patterns, normalization, presolve facts and structural analysis | Ipopt, POUNCE |
| `NleOracle` | Square residuals with an assembled Jacobian or Jacobian-vector product | KINSOL |
| KINSOL `Function` | Residual equations, explicit Picard splitting `F(x) = Lx - N(x)` with a declared constant `L`, or a declared causal map with an original residual validator | KINSOL |
| `CoefficientProblem` | Sparse LP/MILP/QP coefficients with objective sense and constant, the declared registry variable domains (`ModelingVariableDomain`, including `semicontinuous` and `semiinteger`) and consumed parameter assumptions | HiGHS |
| `ConicProblem` | Explicit data in Clarabel's own matrix and cone types | Clarabel |
| `FactorableProgram` | Library-neutral factorable DAG over original case columns with per-row fidelity, projected under the step's values only for a factorable route ([§7.5](mathematics-and-compilation.md#section-7-5)) | SCIP |

Coefficient views are derived only through admitted affine/degree-two proofs, and cones
are declared by an explicit request; neither is inferred from NLP rows. All-fixed
models are evaluated directly as a constant route, except that the `certify` intent and a
native constraint form are refused there ([§18.7](#section-18-7)). Dynamic representations are owned
by [§13](workflows-and-results.md#section-13).

The compiler carries each free variable's declared domain
([§6.8](schema-and-relations.md#section-6-8)) into `CaseStructure`, `ProblemFacts`, the
coefficient view and the factorable program; nothing between authoring and a backend
converts it. A free discrete column removes `smooth_nlp` and `square_root`; it makes a
coefficient-eligible model `mixed_linear`, or `mixed_integer_quadratic` with a quadratic
objective, and any other model `mixed_integer_nonlinear` ([§18.7](#section-18-7)). An
authored MILP therefore routes to HiGHS under an optimization intent, while MIQP and MINLP
route to SCIP ([§18.10.1](#section-18-10-1)), whose export refuses semicontinuous and
semiinteger columns until they have a declared lowering. Integrality is never relaxed to
reach a route.

Constraint forms and disjunctions ([§19.7](workflows-and-results.md#section-19-7)) reach
every representation as the ordinary rows and variables of their lowering. A form realized
for a native handler also travels as `CaseStructure` metadata
([§7.5](mathematics-and-compilation.md#section-7-5)) that only an adapter consuming the
handler may execute ([§18.7](#section-18-7)).

### 18.2 Evaluation programs and callback boundary

Value and derivative programs are Symbolica/Numerica artifacts prepared by the
compiler ([§7](mathematics-and-compilation.md#section-7)); scientific functions and generic external capabilities are supplied through the
authored modeling contract ([§9](physical-semantics.md#section-9)). Each attempt owns cloned
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

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — Ipopt
> linear solvers MUMPS+METIS, SPRAL SSIDS and oneMKL Pardiso, typed and explicitly
> selected, with HSL excluded (Plan 22 N1; not yet implemented: the implemented profile
> below is MUMPS without METIS);
> [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md) — POUNCE's ℓ1 options are
> reserved so that only a typed method reaches them (the reservation is implemented by
> Plan 22 A3; the typed `L1ExactPenalty` method, N3, is not yet implemented).

`pse-ipopt-sys` holds generated, committed bindgen output for the Ipopt 3.14.20 C
interface (`just codegen --only bindgen`; see [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md)
for the digest-pinned solver image). ABI tests assert 32-bit indices, `f64` numbers,
the version string and real symbol relocations. `pse-backend-native::ipopt` is the safe
driver: RAII problem ownership on the owning worker, direct callbacks over `NlpOracle`,
exact or limited-memory Hessians, primal/dual starts, reserved options and an
intermediate callback that checks cancellation and deadline. The pinned MUMPS
profile is serial.

POUNCE (`pounce`, `tnlp`) shares the same `NlpOracle`, callback policy and presolve
pipeline, and adds interior-point and active-set SQP methods, FERAL linear algebra with
admission-bounded threads ([§18.8](#section-18-8)), restoration statistics and working-set
starts. Continuous NLP for both adapters passes through pounce-presolve's qualified
wrappers (`presolve::pipeline`): policy `Off`, `Auto` (qualified source-backed passes only)
or `Explicit` with required passes that must qualify. The library owns reductions,
derivative transport and recovery; the project records effects and inverse source
attribution, not a second transformation IR. Auxiliary reduction is never automatic.

**Option hygiene.** Both adapters reserve the options that encode the derived controls
([§16.6](#section-16-6)), derivative constancy, iteration and time limits, bound
infinities, native scaling and the linear solver; a caller-supplied value for one of them
is refused. Neither library forgets an option once set, so a retained session never
carries an earlier step's option into a later one. A retained Ipopt C problem is reused
only when its coordinate and profile stamps ([§17.6](#section-17-6)), sparsity, bounds and
set of option keys all match, and every solve re-applies every value; a reused POUNCE
application starts from an empty option table. POUNCE's hidden second solves,
`mu_strategy_fallback` and `dual_divergence_retry`, are pinned off, and its ℓ1 fallbacks
(`l1_fallback_on_restoration_failure`, `l1_exact_penalty_barrier`) are reserved until a
typed method selects them (Plan 22 N3). A result therefore never comes from an undeclared
attempt beyond the admitted iteration budget. After each POUNCE solve the adapter reads the
complete effective option table back from the application: every registered option at its
current value, plus explicitly set prefixed options, with the registered defaults beside
it. That snapshot, not the adapter's request, is the report's effective options.

**Scaling and metric names.** Model coordinates are normalized before either adapter runs
([§16.1](#section-16-1)); no user scaling reaches the libraries, and the former native
`Scaling` path is removed. `nlp_scaling_method` is `gradient-based` when the policy permits
native algorithmic scaling and `none` otherwise. Ipopt progress metrics name their
coordinates: `objective.normalized`, `stationarity.normalized` and
`iterate.normalized.infinity_norm` are values of the normalized model (Ipopt's unscaled
readbacks undo only its own scaling), while `primal.native` and `dual.native` are the
infeasibilities exactly as Ipopt reports them. None of them is a physical value.

Two tested limits are qualification distinctions, not retries: with automatic presolve
a recovered bound multiplier can fail original complementarity (the candidate stays
feasible, not stationary), and HiGHS' default QP regularization can miss a stricter
requested objective gap.

### 18.6 Truthful outcomes

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — typed
> adapter evidence (Plan 22 A1, implemented), which qualification reads by evidence type,
> never by backend (Plan 22 A2, implemented); new assurances, each with stated conditions:
> `global_bound` and `proven_infeasible` (Plan 22 G3, implemented for SCIP), and
> `exact_certificate` and `sos_bound_nonrigorous` (Plan 22 G7, N5; not yet implemented).

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
`Feasible`. Beyond that, the kind of typed evidence the adapter recorded selects the rule,
and every rule but the global one needs a success or acceptable native stop. No backend is
named, so a new adapter qualifies through the evidence it produces:

- `GlobalEvidence`, recorded by a certifying adapter over a factorable export, is read first
  and follows its own rule ([§18.10.1](#section-18-10-1)): an original-feasible candidate is
  `Feasible`; only a read-back-equivalent export transfers anything global; an infeasible
  stop the backend concluded grants the assurance `proven_infeasible`, and a successful stop
  with a finite dual bound grants `global_bound`; `GapQualified` additionally needs an
  original-feasible candidate from a result source whose fresh original objective lies
  within the requested absolute gap of the bound, or within the relative gap when both share
  a sign;
- otherwise `CoefficientEvidence` needs a verified upload-equivalence readback plus either a MIP gap
  within budget (`GapQualified`, or `OptimalWithinTolerance` at a zero gap) or LP/QP dual
  feasibility and primal-dual error within budget (`OptimalWithinTolerance`);
- otherwise `ConicEvidence` needs residuals and a gap within budget
  (`OptimalWithinTolerance`);
- otherwise `KktEvidence` with original stationarity and complementarity within budget
  grants `Stationary`; an acceptable stop additionally needs a declared acceptable budget.

A root-system adapter supplies no multipliers, so the root runner tops out at `Feasible`.
A postsolve or validator failure preserves the native outcome and clears assurance;
optional diagnostic failure never replaces the original solve.

**Evidence and metrics.** Adapters record typed `solve::Evidence` on the report:
callback trial history (`CallbackEvidence`: recoverable trial rejections and whether a
terminal failure latched), whether a start was submitted through the native API, whether
the attempt reused retained native state, original-coordinate KKT acceptance
(`KktEvidence`, recorded by `quality::record_kkt`), HiGHS coefficient-model evidence
(`CoefficientEvidence`: upload equivalence, discreteness, objective, MIP gap and dual
bound, primal and dual solution status, dual infeasibility and primal-dual objective error),
Clarabel conic residuals (`ConicEvidence`) and a certifying adapter's `GlobalEvidence`
(export fidelity, box identity, objective sense, feasibility tolerance, requested gaps,
native primal and dual bounds, gap and nodes, readback, the sources of the dual bound and of
the candidate, and the infeasibility conclusion). Qualification (`quality::qualify`),
evaluation retry (`callback::retryable_evaluation`: at least one recoverable trial
rejection and no latched terminal failure), start receipts (the submitted flag) and lineage
identity (both start flags, [§16.5](#section-16-5)) read only this evidence. The
string-keyed `metrics` are observations for reporting and publication and are never an
input to a decision.

The report also retains effective options and queried native defaults, provenance,
bounded events, complete native statistics where the library exposes them, the start
receipt and an output seed. Fit response derivatives remain candidate data distinct from
estimator qualification. The shared tags are registry-owned and projected into Rust,
Arrow and Python ([ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md)); candidate
use (§16.6) combines these facts with physical closure.

### 18.7 Capability, eligibility and selection

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md),
> [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — the
> backend-execution adapter table, the explicit `certify` intent and SCIP routing for MIQP
> and MINLP (Plan 22 A2 and G3, implemented);
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — native
> realizations only on adapters with the handler (Plan 22 M3 and M4, implemented; no linked
> adapter consumes a handler before Plan 22 G7);
> [ADR-0111](../../adr/0111-multi-objective-optimization.md) — lexicographic and weighted
> multi-objective routes (Plan 22 C3; not yet implemented).

Capability is five distinct facts:

| Fact | Owner |
|---|---|
| Requested | `SolverProfile`: the registry intent `NativeSolveIntent` (`optimize`, `root`, `feasible_point`, `initialize`, `certify`), `SolverSelection::{Auto, Explicit}`, controls and typed backend settings |
| Available | `BackendExecution::linked`: the adapter is linked by feature; `runtime.solver_capabilities` publishes one row per linked adapter (§18.9) |
| Admitted | Compiler facts (`pse-math::facts::ProblemFacts`) and structural admission (§15.2): domains, bound shapes, prepared derivative order, coefficient eligibility, convexity evidence, native constraint forms |
| Eligible | `routing::admit`: every applicable typed `Ineligible` reason per adapter for this model, profile and thread count |
| Selected | `routing::Requirements::select` returns `Constant` or `Native(backend)` |

**The adapter seam.** `pse-backend-native::execution` owns one `BackendExecution` adapter
per registry `Backend`. The static table (`execution::adapter`) maps every `Backend` value
to its adapter through an exhaustive match, so a new registry value cannot compile without
one; there is no string lookup. Each adapter declares one representation (`Nlp`, `Roots`,
`Coefficients`, `Cone`, `Factorable` or `Trajectory`); the algebraic router assesses every
adapter except the trajectory ones. Each adapter owns its pse-owned settings type, one
`Capability` record (classes, derivative representation, warm-start support, general bounds,
one-sided bounds as shifted sign constraints (`sign_bounds`), parallelism, whether it
certifies global bounds (`certifies`), the native constraint handlers it consumes
(`native_forms`), reuse, cancellation and diagnostics), admission of its settings
and model contract, its native session on the owning worker, its warm-start payload and
its typed evidence. The capability record is the only source of both eligibility and the
published inventory row: `routing::admit` is a function of that record, the adapter's
linkage and the request, and adapters do not override it. Settings identity
(`BackendSettings::identity`) is derived from serde, never from a hand-written field list,
and enters the request identity and the native profile stamp ([§17.6](#section-17-6)).
Native state retained between the finite steps of a sequence is an opaque, worker-owned
`execution::Retained`: an adapter reuses only its own session, when the coordinate and
profile stamps match (`Compatibility::same_session`) and `ReusePolicy` allows, and
otherwise tears it down before building a replacement; `RequireReuse` refuses instead. A
reused session never inherits an earlier step's native options ([§18.3](#section-18-3)).
The retained state lives on the native session of a staged sequence
([§18.8](#section-18-8)), and a step whose candidate is not a result drops it.

**Shared runners.** Workflows choose a runner by representation and the adapter by table
lookup; neither step names a backend. The runners `execution::nlp`, `execution::roots`,
`execution::coefficients`, `execution::cone` and `execution::factorable` build the adapter's
native representation from original-coordinate inputs, execute it, recover original
coordinates and qualify the report (§18.6). The one NLP runner serves staged-sequence steps,
initialization blocks, fitting and the factorable runner's fixed-assignment re-solve
([§16.6](#section-16-6)); the factorable runner is described with its adapter
([§18.10.1](#section-18-10-1)). The coefficient runner re-evaluates the original compiled
model at the candidate (`execution::OriginalModel`): a projection that disagrees with it
becomes a validation failure, and the fresh original values replace the projected
observation before qualification.

**Selection.** The classes an adapter is assessed against follow from facts and intent
(`routing::problem_classes`): a root intent makes a square continuous problem
`square_root`; a continuous problem is `smooth_nlp`. For other intents a coefficient-eligible
problem adds `linear` or `mixed_linear` when it is affine, and when it is quadratic
`convex_quadratic` with convexity evidence, `nonconvex_quadratic` without it, or
`mixed_integer_quadratic` with a discrete column; a discrete problem that is not
coefficient-eligible is `mixed_integer_nonlinear`. Explicit cones and trajectories are never
inferred from algebraic facts. Automatic selection takes the eligible adapter with the
lowest automatic rank: KINSOL, HiGHS, Ipopt, POUNCE, Clarabel, then SCIP. A rank orders a
choice and never grants eligibility; Diffsol and IDAS are trajectory adapters that the
algebraic router never assesses. Roots and initialization therefore reach KINSOL, then
Ipopt, then POUNCE: one-sided bounds stay with KINSOL ([§18.10](#section-18-10)), and roots
with two-sided boxes route to a constrained NLP adapter rather than dropping bounds.
Optimization reaches HiGHS for admitted coefficient classes, then Ipopt, then POUNCE, and
SCIP only for mixed-integer quadratic and nonlinear programs, which no other adapter
represents; a continuous nonconvex QP therefore stays with the local NLP adapters unless
certification is requested. Clarabel is reached only through an explicit cone request.

An explicit selection is never substituted. An unlinked choice fails with `Unavailable`,
whose alternatives are the other linked adapters that the capability rule finds eligible
for this request; choosing one still passes structural, settings and contract admission.
Refusals outside routing, such as tear selection without HiGHS, list none. An ineligible
choice fails with every reason. The `certify` intent is only ever requested explicitly and
is served only by an adapter whose record `certifies` (today SCIP); every other adapter
carries the `Certification` reason, and without an eligible certifying adapter selection
refuses with a typed `Unsupported` before any route. An all-fixed model is refused under
`certify` rather than evaluated, because constant evaluation proves no bound.

A structure that leaves constraint forms to native handlers
([§7.5](mathematics-and-compilation.md#section-7-5)) is eligible only on an adapter whose
record lists every required `NativeConstraintForm`; every other adapter carries
`Ineligible::NativeForms` naming the missing handlers. No form is converted silently, and an
all-fixed model with native forms is refused rather than evaluated, since constant
evaluation enforces no handler. No linked record lists a handler yet, so a native
realization is refused on every route until Plan 22 G7. A refusal lists every adapter's
assessment by registry spelling.

Unsupported bound, derivative, nonsmooth and thread combinations
fail during preparation, before worker acquisition. Typed backend settings must belong to
the selected route (`BackendExecution::admit_settings`), and the route's model contract is
admitted before any worker exists (`admit_contract`, for example KINSOL's one-sided bounds
and guard signs).

Strategies are derived from distinct projections rather than topology alone: square
root blocks, declared causal fixed-point and Picard maps (§17.4), simultaneous
constrained NLP blocks and supplied continuation (§17.5). An SCC does not by itself
select KINSOL. A finite sequence of prepared steps runs on one native session as one staged
sequence ([§19.2](workflows-and-results.md#section-19-2)) and continues after an unaccepted
step only when the steps are declared independent.

### 18.8 Threading, cancellation and resource ownership

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — SPRAL
> (OpenMP) and oneMKL threads become admitted resources under this owner, with one
> BLAS/LAPACK provider, one OpenMP runtime and a pinned `MKL_CBWR` (Plan 22 N1; not yet
> implemented).

`MathService` (`pse-runtime/src/math.rs`, `math/jobs.rs`) is the single execution
owner. Its `MathPolicy` draws finite allowances from the deployment memory pool:
artifact retention, per-job foreign allowance, worker storage, workspace generations,
native stack, live jobs and flights.

- A job acquires a job slot, CPU permits for its admitted cores and a pool reservation
  covering stacks (one per worker plus a coordinator for parallel teams), numeric bytes
  and the foreign allowance; parallelism cannot exceed the effective process CPU count.
  A library that enforces its own memory limit receives the job's foreign allowance as
  `Execution::memory`, set by `MathService::execute`; SCIP turns it into `limits/memory`
  and refuses to run without it ([§18.10.1](#section-18-10-1)).
- Mutable native state is constructed, used and destroyed on the owning `pse-math`
  thread. Only owned `Send` inputs and results cross; native objects may be `!Send`.
  Salsa never holds mutable solver state.
- Dropping a `SolveHandle` requests cancellation; awaiting `finish` observes completion
  only after native teardown, thread-local destruction and join. Permits and
  reservations survive caller cancellation until that join. Retained results transfer a
  split of the job reservation to an `AllocationLease` that lives with the result.
- A staged sequence runs on one `NativeSession` (`math/staged.rs`, Plan 22 A6). Opening it
  takes one job slot and a pool reservation for its thread's stack, the foreign allowance
  and one worker share, held until the session closes; each step acquires CPU permits for
  its own admitted cores only, so the driver prepares and rebinds between steps without
  holding permits for an idle session. Every step, a single solve included
  (`MathService::solve` opens a one-step session), runs through `MathService::execute` on
  the session thread with the sequence's retained native state. The thread enters the
  scopes of the adapters its steps use (`execution::scoped`); a step that needs another
  adapter scope or thread count ends the current scope, dropping retained state, and
  re-enters. Cancelling a step sets that step's stop flag and leaves the session usable;
  closing the session joins its thread, which witnesses native and thread-local
  destruction.
- Cancellation and deadlines are checked only at library-supported checkpoints
  (Ipopt intermediate callback, POUNCE TNLP callbacks, KINSOL evaluations, HiGHS
  interrupt callbacks for simplex, IPM and MIP, Clarabel's termination callback, SCIP's
  event handler, integrator step boundaries). HiGHS' QP solver and PDLP never poll the interrupt
  callback, so they stop only at their native time limit. A long native factorization
  completes before teardown.
- Ipopt, KINSOL, Clarabel and SCIP profiles are serial (SCIP refuses a thread count other
  than one until its concurrent mode is admitted, Plan 22 G7); POUNCE and HiGHS may use
  admitted threads. FERAL factorizes on its own Rayon pool, which cannot be injected into the
  admitted local pool. It therefore runs parallel only when the admitted thread count is
  greater than one and covers that pool's whole size (`RAYON_NUM_THREADS`, otherwise the
  available parallelism), and serially otherwise; the report records the effective count
  as `linear.threads`. Foreign BLAS/OpenMP threading is environment configuration, not
  admitted by this owner (in the target, ADR-0108 admits SPRAL and MKL threads here; not
  yet implemented).
- Each attempt evaluator is built by one path, `MathService::worker`. Its provider workers
  are scoped to the attempt's cooperative cancel flag, so a nested native provider, such as
  an implicit inner solve, polls that same flag. Its numeric storage is charged to a
  per-job `WorkerBudget`, the worker share of the job's reservation, for as long as the
  evaluator lives. Staged-sequence steps (the step, the original re-evaluation and the
  step's original-model assessment, all on the session's worker), block initialization,
  causal-map units and owned-worker jobs build their
  evaluators this way, so the reservation covers all of a job's live evaluators together;
  one that does not fit is refused as a worker-storage limit.

Reservations are conservative admission policy, not allocator interception or a
process RSS ceiling. The workstation sizing rationale and measured behavior are
qualification matters ([§24.2](operations-and-validation.md#section-24-2)).

### 18.9 Capability matrix

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md),
> [ADR-0105](../../adr/0105-scip-factorable-backend.md),
> [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md),
> [ADR-0110](../../adr/0110-dynamics-profile-extensions.md) — the target adds SCIP,
> POUNCE-convex and extended dynamics; rows change only as Plan 22 packets land. Since
> Plan 22 A2 (implemented) each row is published from its adapter's capability record.
> Implemented: SCIP (G1, G3), the IDAS and Diffsol extensions (Y1, Y2) and the KINSOL
> extensions (Y6); not yet implemented: POUNCE-convex (N5) and Y3–Y5.

The linked inventory is the static adapter table `execution::LINKED`
(`pse-backend-native/src/execution.rs`). `runtime.solver_capabilities` publishes one row
per linked adapter from the capability record routing reads ([§18.7](#section-18-7)); the
test `published_capabilities_equal_routing_rules` rebuilds each adapter from its published
row alone and checks that routing assesses it identically. Feature
`pse-runtime/native-solvers` links the full profile, and Clarabel's non-SDP route is always
present. Diffsol and IDAS publish their records, but their trajectory representation
belongs to the integrator workflows ([§13.6](workflows-and-results.md#section-13-6)) and
the algebraic router never assesses them.

| Backend | Classes | Bounds | Derivatives | Starts | Threads |
|---|---|---|---|---|---|
| Ipopt | Smooth NLP | General | Exact Hessian or limited memory | Primal/dual | Serial |
| POUNCE | Smooth NLP | General | Exact Hessian or limited memory | Primal/dual, working set | Admitted pool |
| KINSOL | Square root, declared fixed point | One-sided, as shifted signs | Jacobian or JVP | Primal | Serial |
| HiGHS | LP, MILP, convex QP | General, semi domains | Coefficients | Primal/dual, basis; QP hot start | Admitted |
| Clarabel | Explicit cones (SDP with `solver-sdp`) | General | Coefficients | None | Serial |
| SCIP | LP, MILP, convex and nonconvex QP, MIQP, MINLP, smooth NLP; certifies | General; finite boxes inside nonlinear terms | Factorable | Primal incumbent | Serial |
| Diffsol | ODE, semi-explicit index-1 | None | First, smooth sensitivities | None | Serial |
| IDAS | ODE, semi-explicit index-1 | None | First, smooth sensitivities | None | Serial |

HiGHS is linked at 1.15. Its record states that simplex, IPM and MIP honour the interrupt
callback while its QP solver and PDLP stop only at the native time limit
([§18.8](#section-18-8)); a QP start is a hot start that needs the exported basis
([§18.10](#section-18-10)). SCIP is linked at 10.0.2 (`scip-sys` 0.1.28, feature `scip`,
part of `native-solvers`); its record is the only one that `certifies`, and it lists no
native constraint handler yet ([§18.10.1](#section-18-10-1)). The published row carries the
`certifies` and `native_forms` columns. `sign_bounds` means one-sided bounds of any value,
represented as sign constraints on shifted coordinates (KINSOL, [§18.10](#section-18-10)).

Outside the matrix today: native indicator, SOS and logic handlers, exact rational MILP, IIS
of nonlinear programs, global solving of models whose implicit blocks or provider outputs
enter nonlinear terms, automatic cone recognition, adjoint and second-order sensitivities,
general or higher-index DAE, finite-difference derivatives, GPU and distributed execution.
Of these, the SCIP extensions (implicit definitions and provider envelopes in the export,
Plan 22 G4; IIS, G5; native handlers, solution pool, reoptimization, concurrent and exact
modes, G7), cone recognition from exact certificates (Plan 22 C5), adjoint and second-order
sensitivities (Plan 22 Y3, Y4) and durable multi-process execution
([ADR-0114](../../adr/0114-typed-operational-store.md)) are in the design
target and not yet implemented. General or higher-index DAE, finite-difference
derivatives, GPU execution and distributing one solve remain outside the target.

### 18.10 Root, coefficient and cone adapters

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — Clarabel's
> SDP profile moves to the image's single oneMKL BLAS/LAPACK provider (Plan 22 N1; not yet
> implemented: the profile below is serial LP64 netlib);
> [ADR-0083](../../adr/0083-class-specific-native-execution.md) — class-specific adapters.
> Plan 22 Y6 (implemented) types the KINSOL method controls, shifts one-sided bounds to sign
> constraints and caches nested sessions per worker.

**KINSOL** (`kinsol`) owns Newton, line search, Picard and fixed-point iteration with
Anderson acceleration. Callers choose only the typed `kinsol::Method`, the adapter's
pse-owned settings, identified through serde: the strategy; the linear solver, which is
vendored KLU over the analytic CSC Jacobian, bounded dense, or matrix-free SPGMR, SPFGMR,
SPBCGS or SPTFQMR over the analytic Jacobian-vector product with a Krylov dimension; the
Anderson history, damping and setup interval; an optional Newton-step cap
(`KINSetMaxNewtonStep`, at least one scaled unit); for the Krylov routes, the inexact-Newton
forcing term (`KINSetEtaForm`: Eisenstat–Walker choice 1 by default, choice 2 with its
safeguard and power, or a constant) and an optional right Jacobi preconditioner built from
the analytic Jacobian diagonal; and, with Anderson acceleration, its orthogonalization and
delay. A control that does not apply to the chosen route is refused, not ignored. Scales and
the step tolerance come from `Settings::from_policy` ([§16.6](#section-16-6)).

KINSOL represents one-sided bounds exactly as sign constraints on shifted coordinates: it
iterates `u = x − l ≥ 0` for `x ≥ l` and `u = x − h ≤ 0` for `x ≤ h`, which leaves the
Jacobian and the step tests unchanged, and a guard sign at least as strong as a bound is
enforced unshifted instead. This is what a capability record's `sign_bounds` means
([§18.7](#section-18-7)): one-sided bounds of any value route to KINSOL, which keeps every
iterate inside them. Two-sided boxes, a bound facing its guard sign and constrained
fixed-point or Picard iteration are refused. Compatible layouts reuse SUNDIALS/KLU
allocations on the owning worker; a reused session keeps the controls fixed at allocation,
refreshes the others, and never carries an earlier step's Newton-step, forcing-term or
Anderson controls.

A nested implicit solve (`implicit.rs`) reuses KINSOL sessions through a per-worker-thread
cache of up to eight sessions keyed by problem layout. The solve takes a compatible session
out of the cache, so a solve nested inside it cannot alias it; refreshes it with the call's
function and parameters, keeping the SUNDIALS context, vectors and KLU analysis; and returns
it afterwards, least recently used first out. The trial problem is lent to the session for
one solve only. An inner unknown's one-sided bound is represented exactly, while a
two-sided interval keeps only its sign information and is rechecked afterwards
(`Problem::verify`). The cached sessions are not yet charged to the job's worker budget (a
recorded Plan 22 follow-up).

**HiGHS** (`highs`, HiGHS 1.15) receives a checked native upload whose full readback must
match the coefficient view before any bound or optimality claim transfers. It supports LP
(choose, simplex, IPM, PDLP), MILP with integer, binary and semi domains, and convex QP.
A mixed-integer model or a quadratic objective requires the `choose` method. HiGHS solves
a continuous QP with its active-set QP solver whatever method is named, and its QP interior
point (HiPO) is not built, so an explicit simplex, IPM or PDLP request with a nonzero
Hessian is refused as `Unsupported` rather than silently ignored. PDLP stops only at its
native time limit ([§18.8](#section-18-8)). Convex QP requires evidence: an exact rational
`GramCertificate` (`sign*Q = Rᵀ diag(w) R` with nonnegative weights, checked through
Symbolica/Numerica against the unchanged matrix and objective orientation) by default, or
an explicitly requested numerical PSD qualification with tolerances. An indefinite or
inconclusive matrix is never repaired. A submitted QP start is consumed: HiGHS 1.15 makes
the QP hot start opt-in, so the adapter sets `qp_allow_hot_start` unless the caller
supplies it, and the effective options record the value that ran. The active-set solver
returns a valid basis with every QP solution, and the output seed exports that basis with
the primal, because the hot start needs both. Scheduler teardown is exclusive and blocking.

**Clarabel** (`conic`) receives explicit cones with CSC-format, dimension and parameter
checks; SDP uses packed PSD-triangle cones in the serial LP64 netlib profile.
`SingleSolve` permits native presolve/chordal preprocessing; `ReusableData` disables
them to use native data updates. Results are post-processed into source space.

Every normalized-coordinate adapter recovers candidates, duals and certificates to original
coordinates through `transport` before quality is assessed; the factorable adapter works in
original case columns ([§18.10.1](#section-18-10-1)).

#### 18.10.1 Factorable adapter: SCIP

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md) — SCIP 10.0.2 through the
> factorable projection and a pse-owned `scip-sys` binding (Plan 22 G1 and G3, implemented;
> implicit definitions and provider envelopes in the export, G4; IIS, G5; native handlers,
> solution pool, reoptimization, concurrent and exact modes, G7; durable incumbents, G8; not
> yet implemented); [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> `global_bound`, `proven_infeasible` and `GapQualified` with their conditions.

**Binding and lifecycle.** `pse-backend-native::scip` (feature `scip`) binds the raw SCIP
10.0.2 C API through `scip-sys` =0.1.28 against the solver image's `SCIPOPTDIR` headers;
neither `russcip` nor the `bundled` or `from-source` profiles is used. SCIP has no run-time
API-version query, so the build asserts `SCIP_APIVERSION` 156, the `SCIP_Real = f64`
signatures and a 32-bit `int`; at run time `scip::abi` checks `SCIPmajorVersion`,
`SCIPminorVersion` and `SCIPtechVersion` against 10.0.2 and refuses a mismatch as
`Unsupported`. One SCIP instance per attempt is created, used and freed on the owning
worker, on every path including unwinding; nothing native is retained between attempts,
and no SCIP type leaves the module.

**Representation and export.** The adapter's representation is `Factorable` and its
derivative capability `factorable`, so routing requires no derivative order of it.
Preparation projects the case under the step's values (`CasePlan::factorable_program`,
[§7.5](mathematics-and-compilation.md#section-7-5)) only for a factorable route, in a
bounded preparation job, and admits the export there (`execution::admit_program`), before
the step reaches a native session, refusing with every typed reason: an objective without
a projection under an optimization or `certify` intent; a variable or auxiliary inside a
nonlinear term without a finite box, which spatial branching needs; a semicontinuous or
semiinteger column, which has no declared lowering yet; or a nonfinite constant or
exponent. The export plan collects the selected rows that have a
projection, the constraints of unconditional obligations, implicit residuals (`= 0`) and
their declared bounds. A strict obligation bound is closed with a relative margin:
`x > b` exports as `x ≥ b + 10⁻⁹·max(1, |b|)`, and `x < b` symmetrically. A row without a
projection is dropped, which keeps the export a relaxation and makes it `Relaxed`. A binary
column's box is `[0, 1]` intersected with its declaration. Affine functions become SCIP
linear constraints and every other function a nonlinear constraint; an affine objective
becomes variable objective coefficients and a nonlinear one an epigraph variable.

**Readback.** Before solving, the adapter reads the native model back and evaluates every
exported constraint and the objective against `FactorableProgram::evaluate` at the start
clamped into the box, with auxiliaries at their box midpoints. The largest relative
deviation must not exceed 10⁻⁹ for any global claim to transfer (`GlobalEvidence::readback`).

**Reserved options.** `misc/catchctrlc`, `limits/time`, `limits/memory`, `limits/gap`,
`limits/absgap`, `limits/totalnodes`, `numerics/feastol`, `randomization/randomseedshift`,
`lp/threads`, `nlpi/ipopt/linear_solver`, `nlpi/ipopt/hsllib`, `nlpi/ipopt/pardisolib` and
every `parallel/` and `concurrent/` option are set only from typed settings and controls; a
caller-supplied value for one of them is refused. `catchctrlc` is false, because signal
handling belongs to the Python boundary; the time limit is the attempt's remaining deadline;
the memory limit is the job's foreign allowance in MiB (`Execution::memory`,
[§18.8](#section-18-8)); the relative and absolute gaps come from `ResolvedAccuracy`, the
absolute one converted to original objective units; the feasibility tolerance is the
resolved normalized budget clamped to [10⁻⁹, 10⁻⁶]; the seed shift and the total node
budget come from `scip::Settings` (`seed`, `nodes`); `lp/threads` is one; and the nested
Ipopt's linear solver is the typed `IpoptLinearSolver` (`mumps` by default, `spral`, refused
without `OMP_CANCELLATION=TRUE`, or `pardisomkl`), while the HSL and Pardiso-project loaders
are never set. After configuration every reserved key and every admitted free-form option is
read back from the instance in its native type, and that snapshot is the report's effective
options. SCIP runs serial until its concurrent mode is admitted (Plan 22 G7).

**Cancellation and progress.** An event handler on the instance catches presolve rounds and
node, LP, best-solution and dual-bound events. On each it polls the attempt's cancel flag
and calls `SCIPinterruptSolve` once, on the owning thread; on every new best solution or
improved dual bound it pushes a `scip.bound` progress event with both bounds. A panic in the
handler is contained as a SCIP error.

**Status map.** `scip::termination` maps every raw `SCIPgetStatus` value of the 10.0.2 ABI
without a wildcard: `OPTIMAL` and `GAPLIMIT` are success with the native assurance
`global_bound`, because the requested gap limit is the successful stop; `INFEASIBLE` is
infeasible with `proven_infeasible`; `UNBOUNDED` and `INFORUNBD` keep their categories;
`USERINTERRUPT` and `TERMINATE` are cancelled; the node, total-node, stall-node and restart
limits are a limit; `TIMELIMIT` is the time limit; `MEMLIMIT` is resource exhaustion;
`PRIMALLIMIT` and `DUALLIMIT` are an objective limit; `SOLLIMIT` and `BESTSOLLIMIT` are a
solution limit; and `UNKNOWN` is inconclusive. Every stop but the first two leaves the bound
unestablished, and a raw value outside the ABI is an internal error. Qualification then
constrains the native assurance (below).

**Incumbents.** A compatible primal seed ([§17.6](#section-17-6)) is submitted through
`SCIPaddSolFree`, as a partial solution when auxiliaries or an epigraph exist, and the report
records whether SCIP stored it. SCIP's best solution in program columns is the reported
candidate and the output seed.

**The factorable runner and the candidate rule.** `execution::factorable` re-observes the
candidate against the original compiled model with fresh values, the declared boxes and
integrality ([§15.4](#section-15-4)), then decides where the candidate comes from (ADR-0105
§2):

- an original-feasible incumbent of an exact export is the candidate
  (`PrimalSource::Backend`), unless the program combines integer columns with a non-affine
  function (MIQP and MINLP);
- otherwise the incumbent is an assignment proposal. Every integer column is fixed at its
  rounded incumbent value (`AlgebraicOracle::with_fixed_assignment`), and the continuous
  problem is re-solved through the one NLP runner by the automatic NLP route, seeded at the
  incumbent, with its adapter's default settings and a coordinate and profile stamp of its
  own (`pse.factorable.fixed-assignment.v1`). An original-feasible re-solve candidate is
  adopted (`PrimalSource::FixedAssignment`) with its KKT evidence, its multipliers are
  conditional on the assignment, and it replaces the incumbent as the output seed; the
  incumbent's objective stays a metric;
- without an adopted re-solve, the incumbent of a relaxed export remains an observation
  (`PrimalSource::RelaxedIncumbent`), which candidate use makes `diagnostic_only`
  ([§16.6](#section-16-6)).

The dual bound's source is recorded as `ExactExport` or `RelaxedExport`, and the box the
backend branched over as `pse.factorable.domain.v1`.

**Qualification** ([§18.6](#section-18-6)). With `GlobalEvidence`, an original-feasible
candidate is `Feasible`. Nothing global transfers unless the export read back equivalent.
An infeasible stop whose backend concluded infeasibility then grants `proven_infeasible`,
and a successful stop with a finite dual bound grants `global_bound`: a dual bound for the
exported program over the declared box, within the recorded tolerances and export
fidelity, never interval-rigorous. `GapQualified` additionally needs an original-feasible
candidate from a result source, never a relaxed incumbent, whose fresh original objective
lies within the requested absolute gap of the bound, or within the relative gap when both
share a sign. A `Relaxed` export's own incumbent therefore supports only the bound and the
infeasibility conclusion; a gap may combine its dual bound with an adopted fixed-assignment
candidate, and both sources are recorded.

**Routing** ([§18.7](#section-18-7)). The record's classes are `linear`, `mixed_linear`,
`convex_quadratic`, `nonconvex_quadratic`, `mixed_integer_quadratic`,
`mixed_integer_nonlinear` and `smooth_nlp`, with general bounds, primal starts and
`certifies`. Its automatic rank is last, so it is selected automatically only for
mixed-integer quadratic and nonlinear programs; it is the one adapter that serves the
`certify` intent; and every other class reaches it only by explicit selection. Today the
projection carries neither implicit definitions nor provider envelopes
([§7.5](mathematics-and-compilation.md#section-7-5)), so certification and global solving
apply to factorable problems whose nonlinear terms involve no implicit block or provider
output (Plan 22 G4).

## Additional and retired section identities

#### 15.1 Thresholds (defaults preserved from IDAES) — retired

No IDAES diagnostic threshold set exists; acceptance tolerances come from the resolved
numerical policy in [§16.1](#section-16-1) and [§16.6](#section-16-6).

#### 16.3 Nominal value algebra

Authored nominal and scale annotations, including composed scheme defaults, resolve through
[§16.2](#section-16-2). They do not infer correctness from the current numerical trial.

#### 16.4 Scaling schemes

Scaling is package knowledge interpreted by the generic annotation/compiler mechanism.
The resulting targets enter the single numerical resolver; see [§16.1](#section-16-1).

#### 17.2 Authored initialization stages

Inherited stages and `when` variants specialize the same definition with explicit stage
bindings. Original model checks remain active, while final fixture expectations apply only
to the final original specification. See [§17.1](#section-17-1).

#### 17.3 Initialization order

Demanded children, effective stages and structural blocks determine execution from admitted
contracts. Failure and cancellation retain attempted/unattempted states; no caller mutation
or hidden plugin order supplies authority. See [§17.5](#section-17-5).

#### 18.4 NL backend — retired

Production NL/Pyomo routes were removed in favor of native class-specific adapters
([§18.1](#section-18-1), [ADR-0083](../../adr/0083-class-specific-native-execution.md)).

#### 18.5 Kernel adapters generated from `KernelSpec` — retired

Library-owned evaluation programs and provider workers ([§18.2](#section-18-2),
[§7](mathematics-and-compilation.md#section-7)) replaced generated kernel adapters.
