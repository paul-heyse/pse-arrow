---
title: Workflows, dynamics and results
status: current
---

# Workflows, dynamics and results

This page owns the public operations that turn an immutable model revision into native
work and the meaning of what comes back: dynamic simulation, parameter fitting, cases,
result qualification and the Python boundary. The Rust owner is
`crates/pse-runtime/src/workflow/` (revisions, preparation, jobs, dynamics, fitting,
completion and result encoding) over the native adapters in `crates/pse-backend-native`.
Python projects the same objects through `crates/pse-py` and `python/pse`. The callable
sequence, settings and examples are in the [native workflow guide](../../dev/native-workflow.md);
this page states the contracts and limits that guide relies on.

## 13. Flowsheets, time and dynamics

> Decision: [ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md),
> [ADR-0093](../../adr/0093-qualified-native-strategies.md)

Dynamics is a declared analysis over the compiled rows of an admitted computation model,
integrated by a native library. There is no time discretization, no index reduction and
no second dynamic compiler: the shared compiler supplies value and derivative programs
for selected rows, and a library integrator owns stepping, consistency, root location,
interpolation and sensitivities. Owners: preparation in
`crates/pse-runtime/src/workflow/dynamics.rs`, clock conversion in `workflow/time.rs`,
result projection in `workflow/simulation_results.rs`, and the native contract and
adapters in `crates/pse-backend-native/src/dynamics.rs`, `dynamics/integrator.rs`
(Diffsol) and `dynamics/idas.rs` (IDAS).

### 13.1 Time domain and origins

A dynamic declaration (`authored.dynamic_cases`, see the
[generated reference](../../generated/relations/authored.md)) names a selected case and
a time parameter of that case. Integration runs on one canonical clock in seconds. Model
time is `(integration time − time_origin) / time-unit scale`; an absent origin is zero,
and a nonfinite origin is refused at preparation. Horizon start/end and sample times in
the simulation profile are integration seconds, strictly increasing within the horizon.

Fit observations carry an optional time, unit and basis: `elapsed` (default) counts from
the integration start, `model_clock` from the declared origin. Both are converted once,
into prepared sample bindings, by one function; a nonfinite result is refused. Measurement
acquisition timestamps are not integration time. The separation of absolute origin from
elapsed duration is part of the shared execution vocabulary
([ADR-0090](../../adr/0090-shared-execution-vocabulary.md)).

`authored.flowsheets` and continuous-domain declarations remain authored document data
with their declared relational checks. They have no execution interpretation: the IDAES
flowsheet `dynamic`/`has_holdup` inheritance and default time sets are not reproduced,
and a selected declaration without an execution interpretation is refused
([§19.1](#section-19-1)).

### 13.2 Declared dynamic roles

Roles are explicit per state, never inferred from flowsheet flags. Each declared state
is a free continuous case variable with a `differential` flag, an initial row, a
canonical offset and scale, and an algebraic residual scale. The mass matrix is the
fixed diagonal `diag(I, 0)`: ones for differential states, zeros for algebraic states,
identical in every mode. The native contract refuses an empty or duplicated layout, a
system with no differential state, parameters that are also states, and events whose
target mode is out of range.

Each mode lists one RHS row per state (differential rate or algebraic residual) plus
optional events. Initial rows depend only on time and parameters, never on dynamic
states. Algebraic rows must admit a complete structural matching of the algebraic
partition (the index-1 structural condition); numerical consistency is left to the
integrator. State normalization, algebraic residual scaling and integration tolerances
are distinct contracts; tolerances apply in normalized state coordinates.

Holdup is expressed physically: an `authored.physical_balances` declaration binds a
conserved differential state to signed source contributions in every mode
([§10](models-and-composition.md#section-10)). `ModelBuilder::vessel` (Rust,
`workflow/vessel.rs`) is a declaration recipe for a fixed-composition vessel with
conserved amount and internal energy, algebraic temperature/density/pressure, the same
FeOS provider and optional valve outflow.

### 13.3 Time derivatives and accumulation

A differential state's RHS row must have the physically registered time-derivative
quantity of the state's quantity, inferred by `pse-quantity` from the time parameter's
unit; reset rows must match the state quantities. Derivative-role, reference or
domain-indexed template symbols are refused in the selected composition route: time
derivatives exist only as the RHS rows of a dynamic declaration, not as authored
template symbols.

Accumulation for a conserved balance is observed, not assumed. Diffsol integrates each
balance's signed flux as an output equation with explicit tolerances (`out_rtol`, one
`out_atol` per balance, required whenever balances are declared). Dynamic closure is the
canonical accumulation change minus the natively integrated flux minus declared event
impulses, carried across event segments; no host quadrature is introduced. A steady
case has no accumulation term; its balances close over contributions alone. Closure
results are reported per sample in `runtime.physical_checks` ([§19.2](#section-19-2)).

### 13.5 Dynamic operations over immutable revisions

Operations that IDAES performs by mutating a model are either new immutable revisions or
declared parts of the prepared profile:

| Need | Current operation |
|---|---|
| Change parameter values or horizon | `PreparedSimulation::rebind` edits the selected case, freezes a new revision and prepares again; unaffected bodies and artifacts are shared by semantic identity |
| Piecewise-constant inputs | Profile `changes`: fixed-time replacement of the complete selected parameter vector; carried-state sensitivities continue |
| Mode switches and state jumps | Declared events: guard row, complete reset rows, `terminal`, `next_mode` and a guard tolerance, all within one state/parameter layout |
| Different initial state | Initial rows evaluated from time and parameters; change the parameters or declaration, not a stored trajectory |

Located roots rewind to the native root time, apply the reset and mode change together
with coincident input changes, then reinitialize before coincident sampling.
Simultaneous actionable roots are refused. IDAES time utilities such as copying values
between time points or deactivating a model at selected points have no counterpart.

### 13.6 Native integrators and trajectories

| Route | Admitted profile | Owner |
|---|---|---|
| Diffsol BDF (default) | Fixed `diag(I,0)` ODE/index-1, events, resets, input changes, smooth forward sensitivities and library-owned reset sensitivities | `dynamics/integrator.rs` |
| IDAS residual BDF | Smooth fixed-mass ODE/index-1 with recoverable trial failures, consistent initialization and forward sensitivities; no events or input changes | `dynamics/idas.rs` |

`Method::Auto` selects IDAS only when the profile declares recoverable trial failures,
because Diffsol cannot recover a typed residual trial failure. Explicitly requesting
Diffsol with recoverable trials, applying Diffsol-specific controls to IDAS, or
requesting an unlinked backend is refused before allocation. Both routes consume the
same compiled functions (`Rhs`, `Initial`, `Output`, `BalanceFlux`, `Roots`, `Reset`);
IDAS is a residual adapter, not a second compiler.

The integrator computes the consistent initial state from requested values and guesses;
the report keeps both. Sensitivities cover initial and direct output parameter terms.
Diffsol's reset sensitivities record that event-time partials are numerical. Sensitivities
through terminal events, and sensitivities combining conserved balances with events, are
refused. Diffsol's infallible operators signal typed provider failure or cancellation
through a private contained unwind; the native solver is discarded before any C or
Python boundary, and no fabricated output is produced
([ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md)).

Profiles bound native steps, retained events, retained scalar cells and a cooperative wall
time. A trajectory ends in one `TrajectoryTermination` (completed, terminal event,
cancelled, time/step/event limit, failed, panic). Reports hold completed samples and
actual event records only; output completed before a later failure survives, with the
last completed time and sample count in `runtime.computation_runs`. Integration inside
fitting runs inline under the outer job's admission, without nested executor permits.

**Limits.** Higher-index or general implicit DAEs, variable-layout modes, hybrid IDAS
sensitivities, adjoint sensitivities and spatial discretization are not supported;
declarations outside the admitted profile are refused before native work. The
qualification basis for the admitted profiles is
[§24.2](operations-and-validation.md#section-24-2).

## 19. Cases, results and analytics

> Decision: [ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md),
> [ADR-0090](../../adr/0090-shared-execution-vocabulary.md)

Every public operation follows one chain: `ModelBuilder::freeze` admits an immutable
`ModelRevision`; `prepare`, `prepare_simulation`, `prepare_fit`, `prepare_initialization`
or `prepare_flow` produces an immutable prepared product with its route and identity;
`start` launches one supervised native job returning a `RunHandle`; its waiters share one
immutable `RunResult`. Preparation performs no writes and holds no native solver state;
mutable evaluators, factors and integrators belong to the attempt. Publication of a
result is a separate explicit step ([§20](identity-and-publication.md#section-20)). Owners:
`workflow/model.rs`, `workflow/run.rs`, `workflow/completion.rs`, `workflow/results.rs`.

### 19.1 Cases and overlays

A case is part of the computation-model declaration: its variables with fixed/free
decision, domain and physical bounds, parameters, instance bindings, row intervals,
objective and values. The case identity covers selected values and physical structure,
excluding display names and prose ([§5.2](identity-and-publication.md#section-5-2)).
Editing a revision yields an independent draft; failed admission leaves the previous
revision untouched. Cases and results never mutate the model
([ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md)).

When a model is composed from templates, `authored.case_specs` resolve against the
selected composition's active scalars: fixed/free and parameter treatment, value or
initial guess, bounds and scaling, converted from the specification's unit. Overlapping
specifications apply in priority order; equal priority on one scalar is refused, as is a
treatment that changes the declared symbol role (an expression cannot be fixed or
bounded) and a specification with no active target. Every selected scalar needs a
finite initial or parameter value.

Selection is exhaustive ([ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md),
proposed): each declaration family is consumed, retained as nonexecuting data, or
refused with source identities. `ModelRevision::admission` reports the accounting.
Authored families with no execution interpretation, including case sets, parent-case
chains, case activations and objectives, scenarios and flowsheets, are refused when
selected ([§6.10](schema-and-relations.md#section-6-10)).

Initialization stages are overlays over the immutable case bindings: a stage replaces
fixed/parameter values, failed stages retain their overlay as evidence, and only
independently accepted solved unknowns are committed
([§17](numerical-execution.md#section-17)). An algebraic numerical start is selected
explicitly on the prepared case (`with_start`, `with_primal_start`; Python
`RunResult.available_start` supplies a compatible seed with its origin), independently
of native allocation reuse. Neither a stage nor a start updates the case.

### 19.2 Results, qualification and diagnostics

`RunResult` holds the typed report, the original request (including unattempted steps)
and a `Completion` computed once at join: candidate assessments, source-attributed
diagnostics, algebraic step records, the dynamic or fitting outcome and full lineage.
Lineage records model revision, case, request, preparation, profile, numerical policy,
physical context and actual environment identities, separately from the unique run ID.
Arrow tables, Python objects and publication copy this product; reading it never
evaluates or reclassifies the model.

These facts stay distinct in every result:

| Fact | Meaning |
|---|---|
| Native termination | What the library reported (`NativeTermination`, `TrajectoryTermination`) |
| Candidate | Whether a point exists and its kind (final, best, feasible, constant evaluation) |
| Numerical qualification | Independent original-model check: feasible, stationary, optimal within tolerance, gap-qualified or unqualified |
| Physical closure | Contribution or accumulation-minus-flux closure against explicit tolerances; missing evaluation is `unavailable`, not a pass |
| Usability | Final decision under the closure policy: usable, qualified but unclosed, unusable |

`RunResult::usable` is true only when every requested candidate is usable. A native
success code alone never grants stationarity or optimality: presolve-recovered multipliers
or QP regularization can yield a feasible but not stationary or not gap-qualified
candidate, and the result says so ([ADR-0093](../../adr/0093-qualified-native-strategies.md),
[§18](numerical-execution.md#section-18)). Missing observations use typed unavailable
reasons, never invented NaN values.

Results are registry-generated `runtime.*` relations (solve runs/variables/constraints/
metrics, computation runs, simulation samples/events, fit parameters/variables/
constraints/observations, response sensitivities, physical checks, candidate
assessments, resolved numerics, run lineage); see the
[generated runtime reference](../../generated/relations/runtime.md). Tables are encoded
once, reserve their buffers before construction and remain valid after the result, run
handle and revision are dropped. Identity framing and reuse equality use canonical float
bits that preserve signed zero ([§5.3](identity-and-publication.md#section-5-3),
[ADR-0089](../../adr/0089-semantic-identity-projections.md)).

Diagnostics are `BoundaryDiagnostic` values classified by the shared taxonomy (invalid
model, unsupported, resource limit, trial rejected, nonfinite, infrastructure, cancelled,
conflict, incompatible, internal) with source identities, stage and observations
([§23.2](operations-and-validation.md#section-23-2)). Diagnostic capture is bounded and
optional; it reads the executed plan and cannot change a scientific or publication
outcome. Progress is a bounded event stream with an actual dropped-event count.
Stream tables, unit report layouts, KPI tables and display tags are not implemented;
consumers query the result relations directly.

### 19.3 Sweeps and reuse

A value sweep is a caller loop over revisions: edit values, freeze, prepare, start.
There is no sweep runner, sample generator or sweep result relation; `authored.case_sets`
declarations are not executed. Reuse comes from the compiler: an unchanged shape reuses
structural analysis and optimized evaluators through Salsa while each point assembles its
own program, and a changed shape recompiles only affected bodies
([§14.4](mathematics-and-compilation.md#section-14-4)). Dynamic parameter and horizon
sweeps use `PreparedSimulation::rebind`. The qualification campaign measured this reuse
([§24.2](operations-and-validation.md#section-24-2)). `Runtime::clear_program_cache`
releases retained programs without invalidating active workers.

### 19.4 Parameter estimation

A fit (`authored.fit_cases`) declares shared parameters (fixed or free, value, optional
bounds, positive scale), experiments (a case with an optional dynamic declaration) and
observation bindings (experiment, output, optional time/basis/unit, inclusion,
importance). Measurement values, units and standard deviations stay in
`authored.observations` with dataset provenance. The loss is fixed:
`0.5 · Σ importance · ((prediction − observation) / σ)²`; point conversion applies
unit offsets, standard-deviation conversion does not, and excluded observations keep
their identity without entering the loss.

| Experiment kind | Treatment |
|---|---|
| Steady | Experiment-specific unknowns join the NLP after the shared parameters; original constraints remain physical constraints; exact Hessians include the residual second-derivative term |
| Transient | Integrated inline with forward sensitivities under the outer job; limited-memory Hessians are required |
| Mixed | Both kinds in one simultaneous NLP |
| All fixed | Direct evaluation with truthful physical quality; no native passes |

Fitting uses the ordinary native NLP route (Ipopt or POUNCE) and library presolve
([§18](numerical-execution.md#section-18)). Contributions are assembled with their
declared sparse support; duplicates accumulate before faer's sparse Gram product, and
dense support stays dense only where declared. Prepared fit metadata and sparse layouts
share one admitted immutable product. Seeds for fitting are refused; declared parameter
values are the start. The IDAS route refuses hybrid fitting sensitivities. Owners:
`workflow/fitting.rs`, `fitting/oracle.rs`, `fitting/sparse.rs`, `fitting/results.rs`.

Response derivatives at the candidate are local physical partials
(`runtime.response_sensitivities`). A steady response needs a feasible, regular square
equality closure solved with faer pivoted LU and a backward-error check; otherwise the
diagnostic is unavailable and the fit result is retained. Optional dense rank
diagnostics (faer SVD of the importance/uncertainty-weighted, parameter-scaled response)
have their own cell cap and reservation; refusal preserves the sparse candidate. An
estimate is qualified only with a stationary or optimal qualification, original
feasibility and full response rank; a rank-deficient response yields an unqualified
estimate with its rank and condition reported. No covariance, confidence interval or
global identifiability follows from convergence or local rank. Data reconciliation and
multi-scenario stacking have no separate templates; the shared-parameter experiment set
is the multi-experiment form.

### 19.5 Costing

Not implemented: no costing templates, flowsheet cost aggregation, cost indices or SSLW
methods ship. The surviving pieces are the currency base dimension in `pse-quantity`, the
`costing_method` template kind and the SSLW enumerations preserved by name
([§6.14](schema-and-relations.md#section-6-14)).

### 19.6 Utility minimization

Not implemented: no heat-integration law, pinch calculation or composite-curve query
exists.

### 19.7 Optionality

Not implemented as a modeling construct: alternative sets and disjunctions have no
declaration or lowering. Integer and semi-variable domains that reach HiGHS through
admitted coefficient routes are numerical classes, not optionality
([§18](numerical-execution.md#section-18)); general MINLP is outside scope
([§25](scope-and-open-design.md#section-25)).

### 19.8 Uncertainty

Not implemented: no uncertainty propagation, covariance estimate or robust
optimization. Local response sensitivities, rank and condition from fitting
([§19.4](#section-19-4)) are the only related results and are not statistical claims.

## 21. The Python boundary

> Decision: [ADR-0024](../../adr/0024-pyo3-arrow-over-arrow-pyarrow.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md)

Python is authoring and result convenience over the Rust workflow. Everything that
computes lives in the extension `pse._native` (`crates/pse-py`); `python/pse` is the typed
boundary around it, and only `pse._build` imports the raw extension (ast-grep rule
`no-direct-native-import`). Mathematical policy, eligibility, numeric validation and all
solver work stay native; Python supplies no numerical callbacks, and provider selection
is a durable native factory declaration.

IDAES and Pyomo are isolated reference tools. They appear only in the parity harness
(`python/pse/parity`, a separate dependency group and environment) and in no production
path. Parity fails rather than skips and exercises the environment and preserved
enumeration names, not numerical equivalence
([relationship to IDAES](../../relationship-to-idaes.md)).

### 21.1 Extension module, jobs and Arrow streams

`pse.Runtime(EngineSettings)` binds the process's single shared runtime and budget,
also used by `pse.open`; conflicting settings are refused. Its surface mirrors Rust:
`physical_from_documents`, `model`/`from_declaration`/`models_from_documents`,
`capabilities` (linked libraries, not model eligibility), `start` for finite sequences,
`prepare_conic` and publication settlement. `ModelRevision` exposes `prepare`,
`prepare_simulation`, `prepare_fit`, `prepare_initialization`, `prepare_recycle` and
`prepare_flow`; cone, causal-map, initialization and tear strategies return owned
`StrategyResult` reports. Owners: `crates/pse-py/src/workflow.rs` and
`python/pse/_workflow.py`.

`RunHandle.wait()` releases the interpreter while waiting and checks signals; an
interrupt requests cancellation and joins the native supervisor before the signal
propagates. `wait_async()` uses `pyo3-async-runtimes` on the process Tokio executor;
cancelling an async waiter requests native stop while the handle keeps the eventual
terminal result. Repeated waits never rerun a solver.

`pse.open(location, version=..., settings=...)` selects one exact Delta publication;
later writes cannot change the selection ([§20](identity-and-publication.md#section-20)).
Result and publication tables cross as one-consumption `TableStream` objects exposing
`__arrow_c_stream__`, never as row objects and never materialized on both sides. The
capsule protocol, not `pyarrow`, is the contract
([ADR-0024](../../adr/0024-pyo3-arrow-over-arrow-pyarrow.md)). Streams own their final
buffer reservations independently of parent handles; closing a parent leaves streams
valid; `close` releases unread work while live arrays stay valid; `cancel` makes further
consumption fail with a cancellation diagnostic; requested-schema casts are refused.
`TableStream.extension_report` compares a consumer's schema field by field as retained,
storage-only, metadata-lost or mismatched (`python/pse/_transfer.py`); capsule export
does not prove the consumer registered the extension types.

### 21.5 Python contracts

Python contracts are generated from the registry into `python/pse/contracts/`
([§4.2](schema-and-relations.md#section-4-2),
[ADR-0051](../../adr/0051-generated-trees-and-regeneration-check.md)): frozen attrs
classes per relation row, enums, value types and Arrow extension types. `pse.modeling`
aliases the generated computation-model declarations; no hand-written class mirrors a
relation. The native API stubs (`_native.pyi`) are generated from the compiled
extension's metadata.

- **Strict structuring.** cattrs converters forbid extra keys and keep detailed
  validation; msgspec structs forbid unknown fields for wire envelopes, settings and
  documents (`python/pse/codec`). A mismatched payload fails at the boundary.
- **No `Any`.** Contract classes are checked for `Any`, bare `dict` and bare `list`
  (`pse.governance`).
- **Import-time checks.** Importing `pse` registers the extension types idempotently and
  checks package/native version and generated registry fingerprint agreement.

### 21.6 The array boundary (numpy)

A nullable Arrow column converted to a bare `ndarray` turns null into NaN.
`pse._array.to_ndarray` is the only production numpy boundary: it refuses any column
with nulls, converts zero-copy by default (refusing chunked or non-mappable layouts), and
copies only when the caller names a `copy_reason`. Signed zero survives. numpy is imported
lazily so `import pse` does not load it; import rules confine numpy/scipy to this module,
the parity harness and tests.

## Retired section identities

#### 13.4 Discretization lowering pass — retired

Dynamics integrates natively without mesh discretization ([§13.6](#section-13-6)); the
Pyomo DAE scheme names survive only as preserved IDAES enumerations
([§6.14](schema-and-relations.md#section-6-14)).

#### 21.2 Pyomo adapter algorithm — retired

Native class-specific execution replaced production Pyomo model construction
([§18](numerical-execution.md#section-18),
[ADR-0083](../../adr/0083-class-specific-native-execution.md)).

#### 21.3 Uses of the Pyomo adapter — retired

Parity is the isolated harness of [§21](#section-21), tear selection and structural
checks are native ([§15](numerical-execution.md#section-15),
[§17](numerical-execution.md#section-17)), fitting is native ([§19.4](#section-19-4)),
and disjunctive or robust optimization has no replacement ([§19.7](#section-19-7),
[§19.8](#section-19-8)).

#### 21.4 Opaque kernels and ASL — retired

Native provider factory declarations replaced external-function kernels; no Python or
ASL callback reaches a solver ([§9](physical-semantics.md#section-9),
[ADR-0084](../../adr/0084-physical-provider-and-dynamic-contracts.md)).
