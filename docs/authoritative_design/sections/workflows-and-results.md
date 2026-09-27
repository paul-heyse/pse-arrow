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

Dynamics is an analysis of the same authored definitions used for steady and simultaneous
execution. Continuous axes and physical derivatives are checked by `pse-modeling` and lowered
by compiler modeling queries. An integrated route supplies native library callbacks; a
simultaneous route expands an authored discretization scheme into ordinary algebraic rows.
Neither route owns a second scientific model. Proposed
[ADR-0099](../../adr/0099-modeling-language-and-identities.md) and
[ADR-0101](../../adr/0101-modeling-analysis-knowledge.md) record the implemented refinement.

### 13.1 Time domain and origins

An authored continuous axis has explicit bounds, physical type, realization and boundary
conditions. Integrated profiles use a canonical clock in seconds. Model time is
`(integration time − time_origin) / time-unit scale`; a nonfinite origin refuses. Horizons
and sample times must be ordered and within the admitted interval.

Fit observations carry optional time, unit and basis: elapsed time counts from the
integration start and model-clock time from the declared origin. One conversion binds
observations to prepared samples. Acquisition timestamps are not integration time.

### 13.2 Declared dynamic roles

Compiler lowering identifies differential states, rates, algebraic equations, initial
conditions and outputs from the selected authored analysis. Native integration admits the
fixed diagonal mass matrix `diag(I,0)` ODE/index-1 profile. The algebraic partition requires
complete structural matching; the integrator establishes numerical consistency. Rates must
be affine in derivative coordinates for this lowering; unsupported residual structure refuses.

Initial conditions and guesses, normalized-state tolerances and algebraic residual scaling
remain separate. Integrated contexts retain selected bindings, original quantities and
source lineage. Dynamic holdup and vessel equations are authored package definitions,
including PC-SAFT/DIPPR functions and optional directional flow; no Rust vessel builder
or production FeOS provider remains.

### 13.3 Time derivatives and accumulation

Quantity inference admits each derivative against the actual axis type. Authored integrals
become native quadratures for causal terminal integrated outputs or scheme-weighted sums
for simultaneous realization. Integral sensitivities are not exposed; ordinary state/output
sensitivities may coexist with terminal quadratures. Missing quadrature tolerances refuse.

Conservation checks consume original contributions or accumulation-minus-integrated-flux
with declared impulses and tolerances. They remain distinct from numerical feasibility.
Original authored checks, validity envelopes and sampled expectations survive into retained
trajectory results. An unavailable or incomplete check cannot become a pass.

### 13.4 Discretization lowering

Difference stencils and Jacobi collocation parameters are authored scheme data. Symbolica
owns root/polynomial mathematics; the compiler lowers checked scheme coefficients, boundary
conditions, continuous children and quadratures to ordinary case rows. There is no closed
scientific scheme enum or independent discretized model. The seed exercises backward finite
differences and order-three Radau PFR/dynamic cases. Expansion and body limits refuse large
meshes explicitly; a small successful fixture does not qualify all mesh sizes.

### 13.5 Dynamic operations over immutable revisions

Operations that IDAES performs by mutating a model are either new immutable revisions or
declared parts of the prepared profile:

| Need | Current operation |
|---|---|
| Change parameter values or horizon | `PreparedSimulation::rebind` prepares new immutable case bindings; unaffected bodies and artifacts are shared by semantic identity |
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
sensitivities and adjoint sensitivities are not supported by native integration;
declarations outside the admitted profile are refused before native work. The
qualification basis for the admitted profiles is
[§24.2](operations-and-validation.md#section-24-2).

## 19. Cases, results and analytics

> Decision: [ADR-0016](../../adr/0016-cases-and-results-never-mutate-the-model.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md),
> [ADR-0090](../../adr/0090-shared-execution-vocabulary.md)

Public execution begins with an immutable `ModelingPackage` admitted from an explicit
package closure and physical context. Selected definitions/cases produce immutable prepared
solves, simulations, fits or strategies. Starting work returns a supervised `RunHandle`;
waiters share the joined result and cancellation joins native destruction. Preparation does
not publish or own a mutable solver. Publication remains explicit
([§20](identity-and-publication.md#section-20)). Owners are `workflow/modeling`,
`workflow/run`, `workflow/completion`, `workflow/modeling_results` and fitting preparation.

### 19.1 Cases and overlays

Authored case and fixture specifications resolve source paths against the selected concrete
instance. They bind values, fixed/free state and physical bounds without changing symbol
roles. Every selected scalar requires an admitted finite value; ambiguous, missing or
incompatible targets refuse. `with_declarations` returns a newly checked package revision;
failed admission leaves earlier packages usable.

Initialization and continuation apply immutable overlays. Only independently accepted
solved values become a committed warm start; final acceptance evaluates the original
specification and original model checks. An algebraic start carries its source identity and
compatibility independently of native allocation reuse. A stage, start or result never
mutates package declarations.

### 19.2 Results, qualification and diagnostics

`RunResult` retains the authored outcome and the typed report, the original request (including unattempted steps)
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
Authored report annotations project selected scalar/indexed observations into
`runtime.modeling_reports`; structured findings, original checks and conformance fixture
dispositions have their own generated relations. Reading a result never reruns a model.
Clones and exported Arrow buffers share allocation ownership through the last reader.

### 19.3 Sweeps and reuse

Authored studies execute finite case inventories with explicit predecessor relationships,
point caps and interruption policy. Failures retain structured causes and do not suppress
independent points; unattempted points remain visible. The compiler reuses equal checked
structure and library programs while values and requested analyses remain explicit inputs
([§14.4](mathematics-and-compilation.md#section-14-4)). Dynamic rebinding retains the same
ownership contract. Runtime cache clearing removes retained programs without invalidating
active workers; historical campaign measurements do not qualify the new seed.

### 19.4 Parameter estimation

A fit (`authored.fit_cases`) declares shared parameters (fixed or free, value, optional
bounds, positive scale), experiments (an authored case with an optional integrated analysis) and
observation bindings (experiment, source output path, optional time/basis/unit, inclusion,
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
`workflow/fitting.rs`, `fitting/{modeling,preparation,oracle,sparse,results}.rs`. Source paths bind shared
parameters and outputs through the same checked package; original checks, fixed values
and bounds are retained. Integration controls are scoped to their experiment instance.

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

The authored SSLW heat-exchanger seed composes design, material and tube-length tables,
explicit pressure validity, CEPCI currency units and accounting accumulators. The
low-pressure upstream comparison explicitly selects extrapolation; the default correlation
refuses it. This is a selected costing method and flowsheet-accounting demonstration, not a
claim that the full SSLW catalogue has been ported. See `packages/reference/process`.

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
path. Parity fails rather than skips and exercises the environment, preserved
enumeration names and explicitly selected reference comparisons; it does not establish
full numerical equivalence
([relationship to IDAES](../../relationship-to-idaes.md)).

### 21.1 Extension module, jobs and Arrow streams

`pse.Runtime(EngineSettings)` binds the shared runtime and memory budget, also used by
`pse.open`; conflicting settings refuse. `physical_from_documents` admits physical data;
`modeling_from_documents` admits the explicit package closure. `ModelingPackage` exposes
immutable declarations/limits/fit-data views, selected solve/simulation/fitting preparation,
initialization, flow/recycle and block strategies, studies, diagnostics and conformance.
`capabilities` reports linked libraries, not model eligibility. Owners are
`crates/pse-py/src/workflow/`, `python/pse/_modeling.py`, `_runs.py` and `_strategies.py`.
The removed model builders have no compatibility facade.

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
classes per relation row, enums, value types and Arrow extension types. Generic modeling declarations are generated from the same owner; no hand-written
class mirrors a relation. The native API stubs (`_native.pyi`) are generated from the compiled
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
