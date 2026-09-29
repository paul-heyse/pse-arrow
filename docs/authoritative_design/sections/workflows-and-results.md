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
A discrete variable ([§6.8](schema-and-relations.md#section-6-8)) enters integration only
when the case fixes it, as a parameter that profile changes can hold piecewise constant;
a free one is refused (`modeling.domain`, analysis `integrated_dynamics`).

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

> Decision: [ADR-0119](../../adr/0119-fixture-analysis-selections.md) — authored schedules,
> whose interval values stay live parameters of the sensitivities, replace profile
> `changes`; authored events and modes replace the runtime- and Python-only mode and event
> inputs (Plan 22 Y0c and Y0d; not yet implemented).

Operations that IDAES performs by mutating a model are either new immutable revisions or
declared parts of the prepared profile:

| Need | Current operation |
|---|---|
| Change parameter values or horizon | `PreparedSimulation::rebind` prepares new immutable case bindings; unaffected bodies and artifacts are shared by semantic identity |
| Piecewise-constant inputs | Profile `changes`: fixed-time replacement of the complete selected parameter vector on either integrator; carried-state sensitivities continue, and IDAS restarts in place ([§13.6](#section-13-6)) |
| Mode switches and state jumps | Declared events: guard row, complete reset rows, `terminal`, `next_mode` and a guard tolerance, all within one state/parameter layout; on IDAS only without forward sensitivities |
| Different initial state | Initial rows evaluated from time and parameters; change the parameters or declaration, not a stored trajectory |

Located roots rewind to the native root time, apply the reset and mode change together
with coincident input changes, then reinitialize before coincident sampling.
Simultaneous actionable roots are refused. IDAES time utilities such as copying values
between time points or deactivating a model at selected points have no counterpart.

### 13.6 Native integrators and trajectories

> Decision: [ADR-0110](../../adr/0110-dynamics-profile-extensions.md) — IDAS scheduled
> inputs with recoverable trials, events without sensitivities, constraints and Krylov;
> Diffsol SDIRK, `tsit45` and KLU (Plan 22 Y1 and Y2, implemented); adjoint and
> second-order sensitivities and shooting routes (Plan 22 Y3–Y5; not yet implemented).
>
> Decision: [ADR-0119](../../adr/0119-fixture-analysis-selections.md) — IDAS sign
> constraints derive only from constant-zero bound annotations, and authored schedules and
> directional events reach both integrators (Plan 22 Y0c and Y0d; not yet implemented). The
> recorded follow-ups below close when they land.

| Route | Admitted profile | Owner |
|---|---|---|
| Diffsol (default) | Fixed `diag(I,0)` ODE/index-1 by BDF (default), SDIRK `tr_bdf2` or `esdirk34`, or explicit `tsit45` for mass-free ODEs only; faer sparse LU or SuiteSparse KLU for the implicit schemes' Newton matrices; events on every guard sign change, resets, input changes, smooth forward sensitivities and library-owned reset sensitivities | `dynamics/integrator.rs` |
| IDAS residual BDF | Fixed-mass ODE/index-1 with recoverable trial failures; consistent initialization of the algebraic states and rates, or a steady start (`IDA_Y_INIT`: every rate zero, every state computed, the requested values only a guess); scheduled input changes, across which forward sensitivities continue; events and resets without sensitivities, each with a crossing direction (either, rising or falling); per-state sign constraints (`IDASetConstraints`); KLU, or matrix-free SPGMR or SPFGMR with an optional Jacobi preconditioner from the diagonal of the compiled Newton matrix; simultaneous or staggered sensitivity correction | `dynamics/idas.rs` |

The methods are typed profile fields: `Profile::diffsol` (`DiffsolSettings`: the scheme and
the Newton linear solver) and `Profile::idas` (`IdasSettings`: the linear solver, the
sensitivity corrector, the initialization and one sign per state). `Method::Auto` selects
IDAS only when the profile declares recoverable trial failures, because Diffsol cannot
recover a typed residual trial failure. Before allocation the profile refuses recoverable
trials on Diffsol; Diffsol-specific controls on IDAS and IDAS-specific controls on Diffsol;
a directional event on Diffsol; `tsit45` with an algebraic state or a Newton linear solver;
IDAS events together with forward sensitivities, because Diffsol owns reset sensitivities; a
sign-constraint vector that is neither empty nor one entry per state, or that constrains
nothing; a zero or oversized Krylov dimension; and an unlinked backend. At each scheduled
change or reset IDAS restarts the same native memory (`IDAReInit`, `IDASensReInit`,
`IDAQuadReInit` and `IDACalcIC`, with consistent sensitivity starts from
`IDAGetSensConsistentIC`); each segment keeps its own native statistics, and output
quadratures continue across the restart. Both routes consume the same compiled functions
(`Rhs`, `Initial`, `Output`, `BalanceFlux`, `Roots`, `Reset`); IDAS is a residual adapter,
not a second compiler.

Profile identity comes from serde, never from a hand-written field list. `profile_json` is
the complete encoding of the profile plus the resolved method; Diffsol's native option types
enter through remote serde definitions, so a new field in a pinned option type fails to
compile rather than escaping identity. `pse.dynamic.profile.v3` frames that encoding with the
numerical policy key. Fitting's `pse.fit.profile.v2` frames the solver profile key, the rank
tolerance, the cell cap, each simulation's dynamic profile identity and each experiment's
declared modes by their serde encoding.

Authored workflows do not yet reach every native control: authored events detect crossings
in either direction, IDAS sign constraints come from the profile rather than from authored
bounds, and authored scheduled inputs have no kernel fixture field (recorded Plan 22
follow-ups). A singular KLU factorization inside Diffsol ends the trajectory with a contained
`panic` termination rather than a typed numerical failure (also a recorded follow-up).

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

**Limits.** Higher-index or general implicit DAEs, variable-layout modes, IDAS
sensitivities across events (hybrid IDAS sensitivities), adjoint and second-order
sensitivities and shooting routes are not supported by native integration (adjoint and
second-order sensitivities and shooting are in the target, ADR-0110, Plan 22 Y3–Y5; not yet
implemented); declarations outside the admitted profile are refused before native work. The
qualification basis for the admitted profiles is
[§24.2](operations-and-validation.md#section-24-2).

## 19. Cases, results and analytics

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md),
> [ADR-0083](../../adr/0083-class-specific-native-execution.md),
> [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md)

Public execution begins with an immutable `ModelingPackage` admitted from an explicit
package closure and physical context. Selected definitions/cases produce immutable prepared
solves, simulations, fits or strategies. Starting work returns a supervised `RunHandle`;
waiters share the joined result and cancellation joins native destruction. Preparation does
not publish or own a mutable solver. Publication remains explicit
([§20](identity-and-publication.md#section-20)). In a durable runtime every run is also an
attempt registered in the operational store, and jobs and studies run across worker
processes ([§20.6](identity-and-publication.md#section-20-6)). Owners are `workflow/modeling`,
`workflow/staged`, `workflow/run`, `workflow/completion`, `workflow/modeling_results` and
fitting preparation.

### 19.1 Cases and overlays

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) —
> restates D13 (superseding ADR-0016): cases and results never mutate the model; a
> cancelled or failed attempt is distinguished by its typed lifecycle state. Plan 22 A6
> (implemented) scopes every overlay to one staged step.

Authored case and fixture specifications resolve source paths against the selected concrete
instance. They bind values, fixed/free state and physical bounds without changing symbol
roles. Every selected scalar requires an admitted finite value; ambiguous, missing or
incompatible targets refuse. `with_declarations` returns a newly checked package revision;
failed admission leaves earlier packages usable.

Initialization, continuation and every other staged step apply immutable overlays. A step's
`Overlay` (`workflow/staged.rs`) holds specialization facts (such as a selected
initialization stage), replacements of declared parameters (such as continuation values),
temporary fixes and relaxations by case path, whose set fields replace the original's, and
temporary case values. It is composed over the original specification for that step only, so
nothing it changes survives the step, whether the step succeeds, fails, is refused or is
cancelled, and the original specification stays in force for the next step and for the
package (PS-08; `initialization_restores_overlays_on_failure`). Only independently accepted
solved values become a committed start; final acceptance evaluates the original
specification and original model checks. An algebraic start carries its source identity and
compatibility independently of native allocation reuse. A stage, start or result never
mutates package declarations.

### 19.2 Results, qualification and diagnostics

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> completion owns candidate use and records the actual request and start. Plan 22 A6
> (implemented) runs every algebraic run as one staged sequence.
>
> Decision: [ADR-0119](../../adr/0119-fixture-analysis-selections.md) — an objective-bound
> `annotation check` reads the step's certified dual bound when the step carries one, and
> `runtime.modeling_checks.basis` records `point` or `global_bound`; a check never starts a
> solve (Plan 22 G6r; not yet implemented).

`RunResult` retains the authored outcome and the typed report, the original request (including unattempted steps)
and a `Completion` computed once at join: candidate assessments, source-attributed
diagnostics, algebraic step records, the dynamic or fitting outcome and full lineage.
Lineage records model revision, case, request, preparation, profile, numerical policy,
physical context and actual environment identities, separately from the unique run ID.
Arrow tables, Python objects and publication copy this product; reading it never
evaluates or reclassifies the model.

**Staged sequences.** Every algebraic run is one staged sequence
(`workflow::staged::Staged`). `ModelingSolvePreparation::start` runs a one-step authored
sequence, and `Runtime::start_modeling` runs a finite authored sequence of at most 4,096
prepared steps from one physical context. The steps share one native session
([§18.8](numerical-execution.md#section-18-8)) and one progress stream. Each is assessed
against the original model on the session's worker, recorded with its typed outcome and
candidate-use decision, and offered the previous step's output seed, which it consumes under
`PreviousAccepted` only when that step's candidate is a result
([§17.6](numerical-execution.md#section-17-6)). An unaccepted step ends the sequence unless
the steps are declared independent; cancelling the run stops the current step and joins
native teardown. Initialization, homotopy and studies are sequences of the same primitive
([§17.5](numerical-execution.md#section-17-5), [§19.3](#section-19-3)). Below the modeling
workflow, `MathService::solve` runs one prepared step, without the modeling assessment, as a
one-step session.

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
outcome. Progress is a bounded event stream with an actual dropped-event count; a
durable attempt's progress events and incumbents are also stored without a cap and read
back in order by `Runtime::progress` ([§20.6](identity-and-publication.md#section-20-6)).
Authored report annotations project selected scalar/indexed observations into
`runtime.modeling_reports`; structured findings, original checks and conformance fixture
dispositions have their own generated relations. Reading a result never reruns a model.
Clones and exported Arrow buffers share allocation ownership through the last reader.

### 19.3 Sweeps and reuse

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — a
> dependent point starts only from a predecessor whose candidate permits it. Plan 22 A6
> (implemented) runs a study as one staged sequence over package views.
> [ADR-0114](../../adr/0114-typed-operational-store.md) — durable studies across workers,
> published once (Plan 22 O7, implemented).

`ModelingPackage::study` executes a finite inventory of points, at most 4,096 and at most the
caller's cap, as one staged sequence with explicit predecessor relationships; a predecessor
must be an earlier point. Each point is one step: it starts from its specification, or,
when it names a predecessor, from that point's solved values (`Start::Seed`,
[§17.6](numerical-execution.md#section-17-6)), which must come from a result or a
`seed_only` candidate ([§16.6](numerical-execution.md#section-16-6)); otherwise the point is
refused with `modeling.study.predecessor`. A failed point is recorded and isolated: it drops
the retained native session and seeds nothing, independent points continue, and only the
points that name it are refused (`failed_point_isolated_in_study`). Points not attempted
after cancellation remain visible as a count. Points of one structure share the package's
prepared view and rebind values ([§14.4](mathematics-and-compilation.md#section-14-4)), so a
study whose points differ only in values prepares one view; the compiler reuses equal checked
structure and library programs while values and requested analyses remain explicit inputs.
Dynamic rebinding retains the same ownership contract. Runtime cache clearing removes
retained programs without invalidating active workers; historical campaign measurements do
not qualify the new seed.

**Durable studies.** `Runtime::start_study` (Python `ModelingPackage.study(..., runtime=,
workspace=)`) runs a study's points as jobs across any number of worker processes and
publishes it once (scenario S15). It stores the package's sources and a typed, versioned
`StudyDefinition`, then creates in one transaction the study's own coordinating attempt
(kind `study`, [§20.6](identity-and-publication.md#section-20-6)), the study's one
publication intent for that attempt, one job per point carrying its `StudyPointBinding`
(study, index, binding hash, typed overlay, predecessor), and a waiting finalization job.
A point with a predecessor waits (its job `waiting`, its attempt `planned`) until the
predecessor completes, then starts from the predecessor's newest compatible stored
solution, or fresh with the reason recorded in its `job.start` event; a predecessor that
fails or is cancelled cancels its dependents transitively as `unattempted`. A completed
try writes its result tables under the intent's prefix and records them in
`study_point_members` in the transaction that completes the point. When the last point is
terminal, the study's attempt ends (completed, partial, failed or cancelled) and the
finalization is released: it writes `runtime.study_outcomes`, one row per point, and
commits one publication of the study's attempt with that summary and every completed
point's members, so a failed point contributes nothing and contaminates nothing.
Cancelling a study stops the points not yet started, asks running tries to stop and still
publishes what completed. A durable study needs the package's authored documents, so a
package changed in memory (`with_declarations`, `with_fit_data`, `with_limits`) is refused.
`Runtime.studies()` lists studies and `StudyHandle` reports, cancels and waits on one.

### 19.4 Parameter estimation

> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — covariance and
> confidence intervals by one rule: exact from the fit's KKT analysis when the fit used the
> exact Hessian, otherwise Gauss–Newton from the response SVD; profile-likelihood intervals
> as adaptive pin chains; one `LocalValidity` relation (Plan 22 S3; not yet implemented).
> ADR-0118 supersedes ADR-0107 and its transient-only Gauss–Newton rule;
> [ADR-0110](../../adr/0110-dynamics-profile-extensions.md) — Gauss–Newton and exact
> transient Hessians (Plan 22 Y4; not yet implemented).

No covariance or confidence interval is computed today; a fit reports the response rank and
conditioning below, which are not statistical claims.

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
values are the start. A discrete variable that an experiment's case leaves free is refused
(`modeling.domain`, analysis `fitting`; [§6.8](schema-and-relations.md#section-6-8)). The
IDAS route refuses hybrid fitting sensitivities. Owners: `workflow/fitting.rs`,
`fitting/{modeling,preparation,oracle,sparse,results}.rs`. Source paths bind shared
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

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md),
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — discrete
> domains, disjunctions and indicator, SOS, cardinality, piecewise and logic declarations
> enter the design target (Plan 22 M1–M5);
> [ADR-0103](../../adr/0103-variable-domain-facet.md) — the declared domain facet (Plan 22
> M1, implemented). Plan 22 M3 and M4 (implemented) lower the constraint forms and
> disjunctions; the M2 fixed-assignment initialization stage and M5 (complementarity and
> discrete phase modes) are not yet implemented.

Authored discrete domains are implemented: a variable declares `integer`, `binary`,
`semicontinuous` or `semiinteger` ([§6.8](schema-and-relations.md#section-6-8)), and a
linear model over such decisions is an authored MILP that routes to HiGHS
([§18.1](numerical-execution.md#section-18-1)); mixed-integer quadratic and nonlinear
programs route to SCIP ([§18.10.1](numerical-execution.md#section-18-10-1)).

**Constraint forms and disjunctions.** The declarations of
[§6.8](schema-and-relations.md#section-6-8) are lowered during generic specialization
(`pse-modeling::specialize::forms`) by named transformations, never during a solve. The
authored revision is unchanged (D13), and every derived row and variable keeps the authored
lineage. The specialized model records each lowering (`Lowering`: source, instance,
realization, declared `Equivalence`, derived rows and variables) in lowering order.
Structural and degree-of-freedom analysis see the lowered problem. No result relation
publishes the lowering record yet.

| Realization | Lowering | Declared equivalence |
|---|---|---|
| `bigm(M)` | With the residual `r = lhs − rhs` and a factor `f` that is zero exactly when the row must hold (`1 − y` for an active indicator or a selected alternative, `y` for `when not y`): `r ≤ M·f` and `r ≥ −M·f`, only the sides the row's sense needs. `M` becomes a member with the row's physical type | Exact only if the authored M is valid, which is the author's assertion (`AuthoredBigM`) |
| `bigm(derived[, margin])` | The same rows, with derived upper and lower bounds of the residual in place of `±M` | Exact over the admitted case box |
| `hull`, `hull(ε)` | For every variable of the disjuncts: derived bounds `L`, `U` from its case bounds, one disaggregated copy `v_k` per alternative with `L·y_k ≤ v_k ≤ U·y_k`, and the link `x = Σ v_k`. An affine disjunct row with `g = lhs − rhs` becomes `g(v_k) − g(0) + g(0)·y_k` related to zero; a nonlinear one becomes the ε-perspective `(y_k + ε)·g(v_k/(y_k + ε))` and needs a declared ε | Exact for affine rows; O(ε) for nonlinear rows (`Perspective`) |
| `indicator` | The rows unchanged, with native indicator metadata ([§7.5](mathematics-and-compilation.md#section-7-5)) | Left to a native handler (`Native`) |
| `linear` (sets, cardinality, logic) | SOS1 and SOS2: `L·z ≤ x ≤ U·z` over finite case bounds with binary selectors that admit one member, or two neighbours. Cardinality over binary members: one count row; over other members only `atmost`, by switched bounds and a count of their selectors. Logic: auxiliary binary resultants with exact linear rows | Exact |
| `sos2`, `incremental` (piecewise) | `sos2`: convex weights with `x = Σ λ_k X_k`, `y = Σ λ_k Y_k`, `Σ λ_k = 1` and segment binaries that restrict the weights to one segment. `incremental`: segment fill variables `δ_j` ordered by binaries, with `x = X_0 + Σ δ_j (X_{j+1} − X_j)` and likewise for `y` | Exact |
| `native` (sets, cardinality, piecewise, logic) | SOS, cardinality or `and`/`or`/`xor` metadata over the same variables; for a piecewise function, the convex-combination rows with SOS2 metadata on the weights. A cardinality over binary members stays one count row | Left to a native handler (`Native`) |

**Derived realization parameters.** Hull and linear lowerings take the bounds of their
variables from the case, and a derived big-M takes each side from the library's interval
enclosure of the residual. Both are parameters that the bound structure determines
(`workspace/modeling/executable/derived.rs`): their rules are prepared with the view and
enter its identity ([§14.4](mathematics-and-compilation.md#section-14-4)), and their values
are computed per binding.

- A bound of a free variable is its case bound, fixed by the structure; an infinite one
  refuses the lowering (`InfiniteBound`). A fixed variable or parameter contributes its value
  in that binding.
- A derived big-M side is the outward-rounded FBBT enclosure of the disjunct residual over
  the case box, from `pounce-presolve`'s forward pass over an enclosure program that is built
  once per view; a semi domain's box includes its zero branch. The needed side is extended
  to include zero, because the residual of an inactive row may be zero; it is widened outward
  by the relative margin and then by one more ULP. A side within the enclosure's own
  rounding resolution of zero (its largest finite end times machine epsilon) is exactly zero,
  never a subnormal. A residual whose tape is not FBBT-complete (`IncompleteInterval`) or
  whose needed side is not finite (`UnboundedInterval`) refuses the lowering, naming the row;
  no default M is ever used.

Derived values complete the values every consumer of the view evaluates with. A value
rebind recomputes them only when a fixed or parameter value they consumed changed, and a
study point that changes such a value never prepares the structure again
(`derived_big_m_follows_value_only_study_points`). Under plain `hull` a nonlinear disjunct
row is refused (`Nonlinear`). Every realization refusal is `ModelingError::Realization`
([§23.2](operations-and-validation.md#section-23-2)).

**Nesting.** Disjunctions nest through alternatives and are lowered inner-first. An inner
disjunction selects exactly one alternative while its owner is selected
(`Σ y_k = y_owner`; at top level `Σ y_k = 1`). Under `hull`, the link rows of a nested
disjunction let its copies vanish and its variables keep their own box when the owner is not
selected: `L·(1 − y_owner) ≤ x − Σ v_k ≤ U·(1 − y_owner)`. Lowering records follow the same
order.

**Routing.** Linear realizations produce ordinary rows over binary variables, which HiGHS
solves when they are linear and SCIP when a quadratic objective or nonlinear row remains. The
linear lowering of an indicator on HiGHS 1.15 matches an enumerated oracle, including a charge
at which idling is optimal (`indicator_linear_lowering_matches_native`, the regression guard
for the HiGHS 1.14.3 presolve defect). A native realization is eligible only on an adapter
whose record consumes the handler. SCIP's record consumes all seven, so a `native` or
`indicator` realization routes to SCIP and is refused before any solve on every other route
([§18.7](numerical-execution.md#section-18-7)). Automatic routing takes the GDP fixture's
indicator realization to SCIP, which reaches the hull realization's optimum and the
enumerated one at every tested demand (`gdp_indicator_matches_hull`). The seed GDP fixture
([§6.10](schema-and-relations.md#section-6-10)) reaches its enumerated optimum through the
hull realization.

Not yet implemented: complementarity declarations and discrete phase-appearance modes (Plan
22 M5), a lowering of semi domains for backends without native semi variables, and the M2
fixed-assignment initialization stage.

### 19.8 Uncertainty

> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — uncertainty propagation
> Σ_y = J·Σ_θ·Jᵀ and covariance with PS-12 validity, valid only where every upstream
> validity row holds (Plan 22 S3–S4; not yet implemented). ADR-0118 supersedes ADR-0107.

Not yet implemented: no uncertainty propagation or covariance estimate; both are in
the design target ([ADR-0118](../../adr/0118-one-kkt-point-analysis.md)). Robust
optimization is not implemented. Local response sensitivities, rank and condition from fitting
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

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> shared vocabulary (restating ADR-0090);
> [ADR-0116](../../adr/0116-typed-boundary-documents.md) (superseding ADR-0113) — typed backend settings, registry names and typed
> eligibility across the boundary (Plan 22 A5, implemented), with published JSON Schemas and
> generated Python document types (Plan 22 B5, implemented);
> [ADR-0114](../../adr/0114-typed-operational-store.md) — durable runtimes, the publication
> catalog, studies and the operational query surface (Plan 22 O3–O9, implemented).

`pse.Runtime(EngineSettings, store=None)` binds the shared runtime and memory budget;
conflicting settings refuse. With `store=pse.OperationalStore()` the runtime is durable
([§20.6](identity-and-publication.md#section-20-6)): every run is an attempt in the store
and may be published, and `runs()`, `jobs()`, `studies()`, `study()`, `work()`,
`query(sql, result=, publication=)` and `progress(attempt_id, follow=True)` read and serve
the store. `physical_from_documents` admits physical data;
`modeling_from_documents` admits the explicit package closure. `ModelingPackage` exposes
immutable declarations/limits/fit-data views, selected solve/simulation/fitting preparation,
initialization, flow/recycle and block strategies, studies, diagnostics and conformance.
`capabilities` reports linked libraries, not model eligibility. A prepared operation
exposes its selected route (`NativeRoute`) and one typed eligibility row per assessed
adapter (`NativeEligibility`, whose reasons carry registry `NativeIneligibility` codes and
typed detail), and a strategy result its ordered attempts (`NativeStrategyAttempt`), all in
registry spellings ([§18.7](numerical-execution.md#section-18-7)). Owners are
`crates/pse-py/src/workflow/`, `python/pse/_modeling.py`, `_runs.py` and `_strategies.py`.
The removed model builders have no compatibility facade.

`RunHandle.wait()` releases the interpreter while waiting and checks signals; an
interrupt requests cancellation and joins the native supervisor before the signal
propagates. `wait_async()` uses `pyo3-async-runtimes` on the process Tokio executor;
cancelling an async waiter requests native stop while the handle keeps the eventual
terminal result. Repeated waits never rerun a solver. A run handle supervises one staged
sequence ([§19.2](#section-19-2)); a single solve is a one-step sequence on its own native
session.

`Runtime.register_workspace`, `workspace` and `head`, `RunResult.prepare_publication`,
`PublicationAttempt.commit` and `settle_publication` publish through the catalog
([§20.2](identity-and-publication.md#section-20-2)). `Runtime.open(publication_id)` and
`open_head(workspace_id)` select one exact publication under a reader lease that is
renewed while the publication or a stream of it is open; `export_publication` writes an
export manifest, and `pse.open_export(location, settings=...)` opens it without the store
([§20.4](identity-and-publication.md#section-20-4)). Later writes cannot change a selection.
The former `pse.open(location, version)` over a Delta control table is removed, with no
compatibility facade.
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

> Decision: [ADR-0116](../../adr/0116-typed-boundary-documents.md) — every Rust-owned boundary document (settings, job payload,
> termination detail, source manifest) is typed and versioned, with a schemars JSON Schema and
> generated msgspec types; validated scalar settings (Plan 22 B5, implemented). As built, the
> job payload carries the typed `SolveSettings` document and a `JobStart` policy; the
> `JobProfile` that ADR-0116 Outcome 6 names was deleted with payload version 1, and the
> payload is at version 3 (`JobPayload`, with a `ModelingJob` or a study finalization).
> [ADR-0115](../../adr/0115-registry-typed-identities-and-vocabularies.md) — every enumeration crossing the boundary is a registry enum with one Rust type
> (Plan 22 B4, implemented).

Python contracts are generated from the registry into `python/pse/contracts/`
([§4.2](schema-and-relations.md#section-4-2),
[ADR-0051](../../adr/0051-generated-trees-and-regeneration-check.md)): frozen attrs
classes per relation row, enums, value types and Arrow extension types. Generic modeling declarations are generated from the same owner; no hand-written
class mirrors a relation. Entity identities are generated `NewType`s
(`pse.contracts.identities`). The native API stubs (`_native.pyi`) are generated from the
compiled extension's metadata.

**Boundary documents.** The Rust serde type owns each Rust-owned document: the backend,
solve, Diffsol and IDAS settings, the job payload, the termination detail, the source
manifest and the study definition. schemars derives its JSON Schema (draft 2020-12) into
`docs/generated/schema/`, and a closed emitter (`pse-codegen::codegen::documents`) turns
the schemas into frozen msgspec `Struct` types in `python/pse/contracts/documents/`: every
enumeration is the registry's generated enum, every object refuses unknown fields, and a
schema construct outside the mapping is a generation error. ADR-0116 Outcome 7 allowed
datamodel-code-generator or this emitter; the emitter was chosen because the external
generator emits `Any`, duplicate enums and unfrozen structs. Python's `SolveSettings`,
`BackendSettings`, `DiffsolSettings` and `IdasSettings` are these types, and the native
entry points decode the encoded document, so no `**fields: object` signature remains.
Single-value setting domains are validated types (`Tolerance`, `Fraction`,
`PositiveCount`, `FiniteBound`, built with nutype) refused at decode with a typed cause;
`admit_settings` keeps the cross-field and environment rules. `SimulationSettings` and
`ModelingFixturePolicy` stay native classes that take an encoded document.

- **Strict structuring.** cattrs converters forbid extra keys and keep detailed
  validation; msgspec structs forbid unknown fields for wire envelopes, settings and
  documents (`python/pse/codec`). A mismatched payload fails at the boundary.
- **No `Any`.** Contract classes are checked for `Any`, bare `dict` and bare `list`
  (`pse.governance`). The check resolves annotations first (`attrs.resolve_types`), so
  postponed annotations (`from __future__ import annotations`) are allowed. It also covers
  the generated msgspec document types (ADR-0116).
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
