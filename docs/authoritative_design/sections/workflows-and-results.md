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

> Decision: ADR-0142 (proposed; maintainer-authorized implementation).

An authored conservation descriptor declares a conserved subject, inventory expression,
original signed flux/source, physical tolerance and permitted event transfers. The same meaning
formulates rates and independently assesses inventory change minus accumulated original flux
and transfers. Checks do not integrate the solved accumulation derivative. Inventories and
state-dependent transfers are evaluated in the actual mode and input segment. Subject identity
persists across mode definitions. Terminal events apply no reset/impulse and retain terminating
mode and active input at their endpoint, including coincident scheduled changes. Continuous
checks apply to integrated and simultaneous realizations; event-bearing simultaneous requests
refuse under the admitted scope. Closure observations and coverage are handed to the result
qualification owner (§19.2); missing evidence never becomes NotRequired.

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
conditions. A temporal analysis owns one shared time axis and expands the selected
stateful definitions over it; scalar parameters remain shared, and spatial axes remain
independent. Missing or competing initial conditions refuse before native execution.
Numerical conditionals retain the runtime clock, including in validation checks. Integrated profiles use a canonical clock in seconds. Model time is
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

> Decision: ADR-0163 (proposed; authorized implementation).

Output goals may protect already-declared integrated observables. Their error evidence is
separate from native local integration weights and original cumulative conservation checks.

Quantity inference admits each derivative against the actual axis type. Authored integrals
become native quadratures for causal terminal integrated outputs or scheme-weighted sums
for simultaneous realization. Integral sensitivities are not exposed; ordinary state/output
sensitivities may coexist with terminal quadratures. Conservation-generated flux quadratures
inherit the tightest consuming physical closure tolerance as their absolute budget
and inherit the integration relative budget unless an explicit quadrature budget is supplied.
There is no additional automatic inner tightening factor; independently assessed inventory
closure remains required.
Unrelated authored integrals still require explicit quadrature tolerances.

A conserved descriptor retains the original inventory and flux expression, physical tolerance
and permitted event transfers. Direct stocks supply their rate equation. Composite stocks
without existing differential coordinates introduce a derived stock equation and project
explicit coordinate initial conditions into its initial inventory. The original coordinate
initial values retain their source identities and required initial-consistency checks; start
guesses cannot replace those obligations. A composite observation over
an already authored differential system adds an independent audit without another rate
system. Transfer paths bind to actual guard member identities in their owning instance, so
nesting or renaming a child does not change the permitted event. Transfers use the inventory
difference convention and evaluate in the pre-event state.
Algebraic-coordinate resets refuse before integration because consistency recovery cannot
provide a declared stock reset mapping. Shooting stitches original inventory, flux and transfer
facts against one global baseline and retains continuity defects.

Conservation checks consume original contributions or inventory-minus-integrated-flux
with declared transfers and tolerances. They remain distinct from numerical feasibility.
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

> Decision: [ADR-0119](../../adr/0119-fixture-analysis-selections.md) — schedules, whose
> interval values stay live parameters of the sensitivities, replace profile `changes`
> (Plan 22 Y0c, implemented: the backend and the authored fixture clause); authored events
> and modes replace the runtime- and Python-only mode and event inputs (Plan 22 Y0d,
> implemented).

Operations that IDAES performs by mutating a model are either new immutable revisions or
declared parts of the prepared profile:

| Need | Current operation |
|---|---|
| Change parameter values or horizon | `PreparedSimulation::rebind` prepares new immutable case bindings; unaffected bodies and artifacts are shared by semantic identity |
| Piecewise-constant inputs | Scheduled inputs (`Profile.schedule`: per input, its contract parameter and strictly increasing change times after the start, a change at the end observed by the final sample only). Each interval's value is its own integration parameter, so forward and adjoint sensitivities are taken with respect to every interval, across every change, on both integrators; IDAS restarts in place at each change ([§13.6](#section-13-6)). An integration fixture declares one as `schedule u at(t₁, …) values(v₀, …);`, and a schedule held `free` (with optional `lower`/`upper`) is a shooting control |
| Mode switches and state jumps | Authored fixture `mode` and `event guard direction(…) tolerance(…) reset(…) next(…)` clauses: guard row, complete reset rows, crossing direction (`EventDirection`), next mode and guard tolerance, all within one state/parameter layout; a directional event routes to IDAS, and IDAS events run without forward sensitivities |
| Different initial state | Initial rows evaluated from time and parameters; change the parameters or declaration, not a stored trajectory |

The default `endpoint fixed_horizon;` requires completion at the horizon. `endpoint declared_terminal_event(guard);` admits exactly one declared terminal guard without reset or successor. Both integrators retain an actual endpoint separately from the requested sample grid: state, mode, quadratures and live inputs before a coincident scheduled change. Runtime checks evaluate original endpoint obligations and conserved prefix closure there. Future observations remain missing, and whole-domain integrals keep their authored extent; an admitted prefix cannot supply them. `runtime.trajectory_endpoints` publishes canonical physical coordinates with stable IDs, endpoint satisfaction and prefix coverage ([ADR-0145](../../adr/0145-declared-execution-and-composed-qualification.md), proposed; authorized implementation).

Located roots rewind to the native root time, apply the reset and mode change together
with coincident input changes, then reinitialize before coincident sampling.
Simultaneous actionable roots are refused. IDAES time utilities such as copying values
between time points or deactivating a model at selected points have no counterpart.

### 13.6 Native integrators and trajectories

> Decision: ADR-0163 (proposed; authorized implementation).

Compatible sampled/endpoint/integrated goals share one bounded comparator integration,
retaining consumed scalar products and matched schedule/event/branch meaning. Paired output
stability is empirical Estimated evidence, never a global trajectory certificate. No-goal
integration acquires no comparator solely for this feature; native controllers remain owned
by their libraries.

> Decision: [ADR-0110](../../adr/0110-dynamics-profile-extensions.md) — IDAS scheduled
> inputs with recoverable trials, events without sensitivities, constraints and Krylov;
> Diffsol SDIRK, `tsit45` and KLU (Plan 22 Y1 and Y2, implemented); pse-owned Diffsol
> linear solvers (Y0a), adjoint sensitivities on both integrators (Y3a, Y3b), exact
> transient Hessians on IDAS (Y4b), single and multiple shooting (Y5b), simultaneous
> dynamic optimization (Y5a) and rolling horizons, NMPC and MHE with the advanced step
> (Y5c), implemented.
>
> Decision: [ADR-0119](../../adr/0119-fixture-analysis-selections.md) — IDAS sign
> constraints derive only from constant-zero bound annotations, and authored schedules and
> directional events reach both integrators (Plan 22 Y0c and Y0d, implemented).

| Route | Admitted profile | Owner |
|---|---|---|
| Diffsol (default) | Fixed `diag(I,0)` ODE/index-1 by BDF (default), SDIRK `tr_bdf2` or `esdirk34`, or explicit `tsit45` for mass-free ODEs only; pse-owned faer sparse LU or SuiteSparse KLU linear solvers for the implicit schemes' Newton matrices (`dynamics/linear.rs`); events on every guard sign change, resets, scheduled inputs, smooth forward sensitivities and library-owned reset sensitivities; adjoint gradients with checkpointing | `dynamics/integrator.rs`, `dynamics/integrator/adjoint.rs` |
| IDAS residual BDF | Fixed-mass ODE/index-1 with recoverable trial failures; consistent initialization of the algebraic states and rates, or a steady start (`IDA_Y_INIT`: every rate zero, every state computed, the requested values only a guess); scheduled inputs, across which sensitivities continue; events and resets without sensitivities, each with a crossing direction (either, rising or falling); per-state sign constraints (`IDASetConstraints`); KLU, or matrix-free SPGMR or SPFGMR with an optional Jacobi preconditioner from the diagonal of the compiled Newton matrix; simultaneous or staggered sensitivity correction; adjoint gradients with checkpointing and forward-over-adjoint second-order sensitivities | `dynamics/idas.rs` |

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
compile rather than escaping identity. `pse.dynamic.profile.v6` frames that encoding, which
includes the scheduled inputs and the typed sensitivity, with the numerical policy key; the
IDAS settings (document version 2) no longer carry signs, which come from authored bounds. A
modeling dynamic simulation's preparation (`pse.modeling.dynamic.v2`) also frames the
derivative order its functions were prepared at. Fitting's `pse.fit.profile.v3` frames the
solver profile key, the rank tolerance, the cell cap, the derivative source, each
simulation's dynamic profile identity, and the uncertainty request when one is present
([§19.4](#section-19-4)); modes are the fixture's, not the fit's.

Authored workflows reach the native controls (Plan 22 Y0c and Y0d): events carry their
crossing direction, IDAS sign constraints derive from constant-zero bound annotations, and
schedules are fixture clauses. The runtime- and Python-only mode and event inputs are
deleted. *Tested* by `kernel_fixture_schedules_inputs`,
`authored_directional_event_routes_to_idas`, `idas_sign_constraints_from_authored_bounds`
and `simultaneous_route_refuses_authored_events` (runtime and kernel units).

**Failing linear solves** (Plan 22 Y0a). Diffsol's Newton matrices are factored by
pse-owned linear solvers over faer LU and KLU (`dynamics/linear.rs`, selected by
`DiffsolSettings.linear`). A failed or singular factorization, or a nonfinite solution, is
returned to Diffsol as an error, so Diffsol reduces its step; a final failure ends the
trajectory `Failed` with a typed numerical error (`ProblemError::Numerical`), never a
contained panic. *Tested* by `singular_newton_matrix_fails_the_solve` and
`diffsol_singular_factorization_is_typed_numerical` (native backend units).

**Sensitivities** (`DynamicSensitivity`: `none`, `forward` or `adjoint`). Forward
sensitivities integrate the state sensitivities with the trajectory. The adjoint route (Plan
22 Y3a, Y3b, `dynamics::gradient`) computes the gradient of a scalar functional of the
sampled outputs with respect to the parameters, including every interval of a scheduled
input, by one forward pass and one backward pass. Diffsol's adjoint takes its
operator adjoints from the one compiled CSC Jacobian, keeps its own checkpoints, at least two
per segment, and runs the backward pass segment by segment with a consistent restart at every
sample, starting from the forward pass's last step. IDAS runs `IDAAdjInit` with `IDASolveF`
per scheduled segment, then the backward problem (`IDACreateB` through `IDASolveB`, with
quadratures for the parameter gradient). `AdjointSettings` bounds the checkpoints (250 steps
between checkpoints and at most 400 by default). Before any native work the adjoint route
refuses events and resets, declared quadratures and a steady start (`Unsupported`), a
scheduled change at the end or fewer checkpoints than two per segment (`Contract`), and a
checkpoint estimate above the job's foreign allowance (`Limit`, memory); a forward pass that
would exceed the checkpoint bound stops as a memory limit. *Tested* by
`scheduled_input_sensitivities_cross_changes`,
`final_scheduled_change_is_reinitialized_and_observed`,
`idas_scheduled_inputs_with_recoverable_trials`,
`diffsol_adjoint_gradient_equals_forward_and_differences`,
`idas_adjoint_gradient_equals_forward_and_differences`,
`diffsol_adjoint_starts_at_a_nearly_steady_stop`, `checkpoint_memory_bounded`,
`idas_checkpoint_memory_bounded` and `adjoint_profile_limits_are_typed_refusals` (native
backend units).

**Second order** (Plan 22 Y4b). On IDAS, the exact Hessian of an adjoint objective comes from
forward-over-adjoint second-order sensitivities (`IDAInitBS`, `IDAQuadInitBS`), one backward
problem per direction over the exact block lower-triangular Newton matrix factored by KLU; a
dynamic program is prepared at second order on request. Diffsol has no second-order adjoint
and is refused with a typed `Unsupported`; a program without second derivatives is refused
the same way, and an estimate above the allowance is a memory limit. The oracle surface is
`Oracle::weighted_hessian`. *Tested* by `second_order_adjoint_matches_finite_difference` and
`second_order_route_limits_are_typed_refusals` (native backend units).

**The integrated experiment and shooting.** `IntegratedExperiment`
(`pse-runtime::workflow::integrated`) is the one integration that fitting and shooting share:
it binds a consumer's parameter values into the integration vector, integrates once, and
maps output sensitivities, an adjoint gradient or an exact Hessian back to those parameters.
Shooting (Plan 22 Y5b, `workflow/shooting.rs`, `ModelingSimulation::shooting`) optimizes a
dynamic simulation through the one NLP runner ([§18.7](numerical-execution.md#section-18-7)):
a `ShootingProfile` names the method (`ShootingMethod`: `single` or `multiple`), the nodes,
the controls (scheduled inputs held free, each interval value an NLP variable within the
control's bounds), path bounds on outputs, which become rows at every sample, and a terminal
or integral objective over declared quadratures, which each window observes as outputs.
Single shooting integrates one window over the horizon; multiple
shooting integrates one window per node interval, each anchored at its start state
(`dynamics::Anchored`), with the differential states at the inner nodes as variables closed
by continuity rows. A terminal objective takes its gradient from the forward sensitivities
the rows need anyway, and an integral one from the adjoint. The shooting NLP supplies first
derivatives only, so it requires the limited-memory Hessian and refuses any other mode. A
`ShootingReport` returns the solve, controls, node states, objective, continuity residual,
trajectory and checks. A fixture selects it with `route integrated; procedure shooting;`, a `shoot single;` or
`shoot multiple nodes(…);` clause and at least one free schedule
(`ModelingSimulation::authored_shooting`); an integrated simulation admits the model's
objective only when its fixture runs shooting. *Tested*
by `shooting_matches_simultaneous_optimum` (against an analytic optimum),
`multiple_shooting_continuity_closes`, `shooting_path_bounds_hold_at_samples` and
`shooting_fixture_needs_authored_controls` (runtime units) and
`anchored_window_continues_the_horizon` (native backend units).

Shooting's public `start`/`start_with_initial` enter the existing asynchronous run supervisor. The raw solve is internal. Cancellation, capacity, native destruction, completion and publication share the other run owners; optimizer facts and trajectory transport retain the composed permission. No second direct execution path remains.

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

**Simultaneous dynamic optimization** (Plan 22 Y5a). The simultaneous route optimizes an
authored dynamic model through its collocation, like any steady optimization: a bounded
continuous input is a convex QP on HiGHS, and an on/off input a MIQP on SCIP whose
candidate is the continuous re-solve under the node assignment, stated as its commitment
([§6.8](schema-and-relations.md#section-6-8)). The same definition integrates with the input
scheduled. *Tested* by `simultaneous_dynamic_optimization_matches_analytic` and
`simultaneous_dynamic_optimization_with_discrete_decision` (native acceptance, the
`control_fixtures` `SaturatedProcess` and `SwitchedProcess`).

**Rolling horizons** (Plan 22 Y5c, `workflow/horizon.rs`, `Runtime::start_horizon`). A
horizon is one run and, under a durable runtime, one attempt registered before any effect
(`pse.durable.horizon_request.v1`). Its steps are strictly sequential and share one
worker-owned native session, so they are not a study's independent points. At every sample
the loop measures the plant, runs the estimator once its window is measured (MHE: a
least-squares window whose free initial state carries an arrival cost; the next prior is
the solved state one period into the window), runs the controller (NMPC over its horizon
from the measured or estimated state) and integrates the plant for one period on the same
native session from the previous period's end state with the moves held, so the solves'
retained native state survives. Every estimator and controller step composes a value-only
overlay over its immutable specification, so each prepares its structure once and rebinds
values (A6); a step starts from the previous accepted step of its role and offers its native
seed (N2), which a value-only rebind keeps across presolve (`pse.presolve.transformation.v3`
frames value-dependent facts only when a pass consumed them). The steps are the run's
modeling steps, each with its stored seed under a durable runtime (O6), and every sample
streams one `horizon.step` event (O5). `HorizonDecision` records how a sample's moves were
decided (`open_loop`, `solved`, `held`, `predicted`, `fallback`).

*Advanced step* (Plan 22 Y5c2). An advanced-step controller (`HorizonController.advanced`)
binds its measured or estimated state as the parameters of a sensitivity request. After
applying its moves it solves in the background at the values it predicts for the next
sample (its own solution one period ahead for each mapped state, the next setpoint, the
moves just applied), keeping that solve's parametric factor
([§15.5.1](numerical-execution.md#section-15-5-1)). At the next sample one backsolve
corrects the prediction to the actual state; a refused prediction (no factor kept, or an
active-set change) falls back to a full solve at the sample, and the step records the
`Fallback` reason. *Tested* by `nmpc_closed_loop_on_antiwindup`,
`horizon_reuses_prepared_view`, `mhe_recovers_initial_state`,
`horizon_records_one_durable_attempt`, `advanced_step_matches_full_resolve` (every sample
equal to the fully re-solving loop within 1e-8 at KKT budgets of 1e-12) and
`advanced_step_falls_back_on_active_set_change` (runtime units).

**Limits.** Higher-index or general implicit DAEs, variable-layout modes, IDAS
sensitivities across events (hybrid IDAS sensitivities), adjoints across events or resets or
over declared quadratures, and second-order sensitivities on Diffsol are not supported by
native integration; declarations outside the admitted profile are refused before native
work. The
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

> Decision: ADR-0163 (proposed; authorized implementation).

Completion retains immutable physical goal assessments, method, validity, strength and
unavailable reasons. Publication and Python/durable consumers report that assessment without
refining or reinterpreting usability. A selected output and the optimum objective are distinct
subjects: objective-gap evidence cannot certify arbitrary variables. For an asymmetric optimum
interval, value resolution is assessed about the reported representative, not silently about
the midpoint. Native terminal/refusal obligations retain precedence.

> Decision: [ADR-0160](../../adr/0160-centralize-testing-responsibility.md)

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> completion owns candidate use and records the actual request and start. Plan 22 A6
> (implemented) runs every algebraic run as one staged sequence.
>
> Decision: [ADR-0119](../../adr/0119-fixture-analysis-selections.md) — an objective-bound
> `annotation check` reads the step's certified dual bound when the step carries one, and
> `runtime.modeling_checks.basis` records `point` or `global_bound`; a check never starts a
> solve (Plan 22 G6r, implemented).

`RunResult` retains the authored outcome and the typed report, the original request (including unattempted steps)
and a `Completion` computed once at join: candidate assessments, source-attributed
diagnostics, algebraic step records, the dynamic or fitting outcome and full lineage.
Lineage records model revision, case, request, preparation, profile, numerical policy,
physical context and actual environment identities, separately from the unique run ID.
Arrow tables, Python objects and publication copy this product; reading it never
evaluates or reclassifies the model.

`ModelingTrajectory` is a clone-shared immutable completion snapshot. Read-only accessors
expose the native outcome and retained checks; `accepted()` derives the composed permission.
The completion owner creates the successful Simulation header once. Shooting transport
consumes its coherent joined report and enclosing optimizer header. Exporters never select
an independent kind or recalculate qualification.

Direct trajectory transport lazily publishes one successful complete map of checked batches,
serialized across clones. Failed encoding releases partial allocations and leaves no cache;
a later direct access can retry without another solve or assessment. The outer `RunResult`
retains its existing sticky encoding-error contract. Container metadata is admitted before
growth and retained with escaped batches; shared buffers keep their existing leases. First
access materializes the whole map, so a budget fitting one relation may still refuse. No
speed or memory claim follows without representative measurements.

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

**Objective-bound checks** (Plan 22 G6r). The compiler classifies an authored
`annotation check` that bounds the objective. Such a check reads the step's certified dual
bound instead of its point value when the step carries one: an assurance of `global_bound`
or `exact_certificate` with a finite dual bound, and only when that bound bounds the checked
side (a lower-bound check under minimization, an upper-bound check under maximization).
`runtime.modeling_checks` (version 5) records each check's `basis`, `point` or
`global_bound`; a point result states no global property, and a check never starts a solve.
*Tested* by `objective_bound_check_classified` (compiler units),
`objective_bound_check_uses_certified_bound` (runtime units) and `tpd_certifies_stable_feed`
(native acceptance conformance).

> Implemented amendment: [ADR-0141](../../adr/0141-applicability-evidence-and-permissions.md), Plan 25b.

Scientific applicability observations in `runtime.modeling_checks` preserve the actual
consuming instance, claim owner and checked nominal ancestry, selected records, physical
inputs, evidence basis, unknown reason and required or alternative status. A matched
permission retains its identity, scope, authored targets and independent unknown/extrapolation
flags. Refusals retain the same typed assessment through boundary diagnostics. These are
low-level data-use observations; result qualification is owned by
[§19.2](#section-19-2).

Results are registry-generated `runtime.*` relations (solve runs/variables/constraints/
metrics, computation runs, simulation samples/events, fit parameters/variables/
constraints/observations, response sensitivities, physical checks, candidate
assessments, resolved numerics, run lineage; local validity, parametric sensitivities,
reduced Hessians, infeasibility certificates, the solution pool and, for durable runs,
incumbents; parameter covariances and intervals, profile points, response directions and
propagated covariances); see the
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

### 19.3 Studies and reuse

> Proposed amendment: [ADR-0154](../../adr/0154-declared-numerical-strategy.md), Plan 25n N7–N10.

Modeling, studies, fitting, shooting, horizons and applicable dynamic initialization supply their
mathematical target and original assessor to one numerical binder. Sampling, statistical stopping,
control application, scientific time/events and durable occurrence policy retain their owners.
Preparation inspection may be conditional; execution traces contain actual decisions and products.
Automatic trace publication uses the already admitted request identity, including empty and
preparation-only refusal prefixes. Such a prefix is an observation, not a new execution declaration;
explicit execution declarations retain their admission checks.
`runtime.solve_strategy_products` retains every actual point/action receipt with consumed source,
normalization, original-space point, derivative/action order, branch and error class; capability
declarations create no rows. Effective caller output obligations are checked before original
permission; rejected evidence remains traceable without becoming a consumable product. A later
modeling or horizon failure retains completed targets and the actual failed target trace. Plant/task
failures name no solve target, and unattempted targets receive no invented failure trace.
A fresh study batch is eligible only when each member needs the same lawful direct operation;
otherwise members use the common automatic binder individually under their own original scope.
Screened proposals retain their actual producer origin and separately granted recovery permission;
they do not rewrite an original start policy to Explicit.

> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

Completion keeps candidate presence, evaluated original-space quality, unavailable assessment and native termination distinct. Only evaluated violations justify an infeasibility refusal. One shared exhaustive projection retains detailed causes and native limit categories without weakening result/seed/incumbent policy; generated and recorded consumers preserve those meanings.

> Supplement: [ADR-0148](../../adr/0148-studies-diagnostics-and-admitted-bindings.md)
> (proposed; maintainer-authorized Plan 25f implementation). The shared occurrence contract
> replaces the former in-process predecessor policy and durable automatic fresh fallback.
> Existing preparation, session and batching mechanisms remain owned by ADR-0106/0109/0114.

**Admission and bindings.** `ModelingPackage::admit_study_points` admits a typed request
against one immutable modeling revision and physical context. A `StudyDefinition` records
ordered occurrences with unique keys, reconstructable operation descriptors, canonical bindings,
explicit dependencies, start selection and attempt limits. Seed consumption comes from the
operation owner rather than an author-supplied claim. Both executors validate decoded definitions,
including source/context, binding identity, route/procedure and producer/consumer seed roles.

Submitted assignments name paths or member identities and complete quantity/unit meaning.
Admission resolves writable targets, compares complete physical contracts through A's checked
conversion, rejects conflicting aliases and composes replacements over authored defaults once.
Overlay workers consume canonical member coordinates without resolving submitted paths or
converting again. Paths and supplied units remain attribution. Signed zero retains its identity bits. The binding
content identity is [§5.3](identity-and-publication.md#section-5-3); equal content can share
preparation but never merges occurrence, run or attempt identity. Horizon inputs, estimator
priors and controller trajectories follow the same physical admission before their existing
owner receives numerical coordinates. Their operation-owned destination paths are reconstructed
by that owner and checked against the persisted canonical member identities; replay does not
convert the admitted values again.

**One pure policy.** `pse-model::study` owns the semantic facts; `pse-operations::study_policy`
consumes them without database, artifact or native solver effects. Ordering edges wait for a
terminal predecessor, including failure. Usable-result edges require E's aggregate scientific
permission. Continuation defaults to a usable compatible predecessor seed; seed-only permission
and fresh fallback on declared absence/incompatibility require explicit recorded choices.
Internal acquisition errors never authorize fallback. An explicit incompatible seed refuses.
A seed-free operation records `NotNeeded` while retaining its declared dependency condition.

The transition returns an action per occurrence with its expected state revision: Wait, Start,
Refuse, Cancel or Reconcile. Its conclusion keeps scientific availability and operational
lifecycle separate. Every requested occurrence remains observable, including unstarted
cancellation, preparation failure and dependency refusal. Attempt outcomes retain source-owned
diagnostics, scientific decisions, chosen start and publication-effect knowledge independently
of available result members. A partial multi-result run remains governed by E's final decision;
counting tables or selecting the first result cannot promote it.

**Execution and preparation.** `ModelingPackage::study` consumes the admitted definition within
the caller's bound and the shared maximum. `StudyReport` retains outcomes and original joined
`RunResult` objects; its table and findings use the same row projectors as durable finalization.
A solve uses the existing staged mathematical session; simulations, fits and horizons invoke
their existing operation owners. Unsupported operation/dependency combinations refuse before
scheduling. Independent fresh case preparations can share the existing `Staged::batch` operation.
Each remains independently assessed and recorded. The native adapter owns batching eligibility,
thread admission and fallback to individual execution.

`StudyReport.preparations` projects the mathematical service's task-local `PreparationCounts`:
views, observation programs, value-dependent rebuilds and shares. Counting includes preparation
performed within study execution and stays local to concurrent studies. Definition admission
prepares the bound operation to freeze its actual seed needs before execution; any view cached
there is outside these execution-local counters. An execution that consumes that admitted view
records a value rebind for its first point as well as subsequent points, without another view.
Equal structures reuse compiler/library
products while admitted values remain explicit inputs. Historic measurements retain their
original scope; the owning [Plan 25f](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25f-studies-diagnostics-and-continuation.md)
records replacement-control evidence and [25k](../../plans/25k-integrated-qualification-and-closure.md)
owns assembled qualification.

**Durable adapter.** `Runtime::start_study` admits a raw request; `start_defined_study` consumes
the same immutable definition as the in-process executor and verifies its exact stored source
bundles. Creation owns one coordinating study attempt, publication intent, job per occurrence
and finalization job. A claim permits input/seed acquisition. Under the existing locks, live
lease and revision fence, native dispatch rereads facts and applies the shared policy's chosen
start. Retry keeps the occurrence key and adds an attempt. Only declared transient failure with
known absent or proven idempotent effect permits replay; unknown effects reconcile first through
the existing native member-receipt owner ([§20.6](identity-and-publication.md#section-20-6)).

Finalization publishes one structured `runtime.study_outcomes` row per requested occurrence
and every actually recorded member, including members available after partial failure or
cancellation. Member availability is not inferred from operational completion. Cancellation
prevents new native dispatch while completed work remains inspectable. Source-backed durable
packages retain exact authored documents; an in-memory package without that closure cannot be
submitted for durable reconstruction. The preserving schema transition retains historical
identities and payloads, marking absent historical policy/scientific facts explicitly unavailable.

### 19.4 Parameter estimation

> Supplement: [ADR-0144](../../adr/0144-selected-mathematical-realizations-and-square-response.md) (proposed; authorized implementation).

Steady fitting response and public Root parameter sensitivity consume one qualified regular-square operation. It solves `F_x X_p = -F_p`, consumes complete original equality support and physical feasibility, applies explicit scaling, checks numerical rank and normwise backward error, and returns physical state/parameter coordinates. Guard/selector derivative checks and inactive state bounds qualify the local neighborhood; failure withholds response while retaining the base solution. This response is separate from optimizing KKT analysis and statistical covariance assumptions.

> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — covariance and
> confidence intervals by one rule: exact from the fit's KKT analysis when the fit used the
> exact Hessian, otherwise Gauss–Newton from the response SVD; profile-likelihood intervals
> as adaptive pin chains; one `LocalValidity` relation (Plan 22 S3, implemented). ADR-0118
> supersedes ADR-0107 and its transient-only Gauss–Newton rule;
> [ADR-0110](../../adr/0110-dynamics-profile-extensions.md) — adjoint gradients (Plan 22
> Y3), Gauss–Newton Hessians (Y4a) and exact transient Hessians on IDAS (Y4b), implemented.

A fit (`authored.fit_cases`) declares shared parameters (fixed or free, value, optional
bounds, positive scale), experiments (an authored case with an optional integrated analysis) and
observation bindings (experiment, source output path, optional time/basis/unit, inclusion,
importance). Each observation selects an admitted measured attribute by declaration
identity and attribute name. Values retain complete physical types, source provenance and
canonical difference-unit standard deviations; unsupported uncertainty is refused.
The loss is fixed:
`0.5 · Σ importance · ((prediction − observation) / σ)²`; point conversion applies
unit offsets, standard-deviation conversion does not, and excluded observations keep
their identity without entering the loss.

An explicit qualified-result export emits generated `runtime.fitted_parameter_cells`
with quantity, unit, value, run, fit and source-revision lineage. Export requires joined
feasibility, stationarity, response-rank and current-check evidence. A receiving package
explicitly admits the fitted set with fit lineage; export never installs a bank implicitly.

| Experiment kind | Treatment |
|---|---|
| Steady | Experiment-specific unknowns join the NLP after the shared parameters; original constraints remain physical constraints; exact Hessians include the residual second-derivative term |
| Transient | Integrated inline under the outer job through the shared `IntegratedExperiment` ([§13.6](#section-13-6)): forward sensitivities give the response Jacobian, or in the gradient-only mode adjoint sensitivities give the objective gradient alone; the Hessian is limited memory, Gauss–Newton, or exact from forward-over-adjoint second-order sensitivities on IDAS |
| Mixed | Both kinds in one simultaneous NLP |
| All fixed | Direct evaluation with truthful physical quality; no native passes |

Fitting uses the ordinary native NLP route (Ipopt or POUNCE) and library presolve
([§18](numerical-execution.md#section-18)). Contributions are assembled with their
declared sparse support; duplicates accumulate before faer's sparse Gram product, and
dense support stays dense only where declared. Prepared fit metadata and sparse layouts
share one admitted immutable product. Seeds for fitting are refused; declared parameter
values are the start. A discrete variable that an experiment's case leaves free is refused
(`modeling.domain`, analysis `fitting`; [§6.8](schema-and-relations.md#section-6-8)). The
IDAS route refuses hybrid fitting sensitivities, and a fit refuses a solve-level sensitivity
request ([§15.5.1](numerical-execution.md#section-15-5-1)). Owners: `workflow/fitting.rs`,
`fitting/{modeling,preparation,oracle,sparse,results}.rs`. Source paths bind shared
parameters and outputs through the same checked package; original checks, fixed values
and bounds are retained. Integration controls are scoped to their experiment instance.

**Hessians and derivative sources.** The Hessian mode is a solver control
([§16.6](numerical-execution.md#section-16-6)). `exact` includes the residual
second-derivative term; `gauss_newton` (Plan 22 Y4a) is `JᵀWJ` plus the constraint-multiplier
Hessians, without residual curvature, defined only for the least-squares fit objective, so
fitting is the one request that declares it and every other request is refused before
native work (`least_squares`, [§18.7](numerical-execution.md#section-18-7)); a transient
experiment contributes only its Gram term. A transient experiment's exact Hessian (Plan 22
Y4b) comes from IDAS forward-over-adjoint second-order sensitivities, prepared at second
order on request; Diffsol refuses it. `FitProfile.derivatives` (`FitDerivatives`) selects
the derivative source: `responses`, the response Jacobian from forward sensitivities, or
`gradient`, the objective gradient alone from adjoint sensitivities of the transient
experiments, which requires the limited-memory Hessian and reruns the forward sensitivities
once at the final assessment for the response rank. *Tested* by
`gauss_newton_hessian_matches_jtwj`, `gauss_newton_fit_admits_transient`,
`adjoint_gradient_equals_forward_on_transient_fit` and
`exact_transient_fit_hessian_matches_finite_difference` (runtime units).

Response derivatives at the candidate are local physical partials
(`runtime.response_sensitivities`). A steady response needs a feasible, regular square
equality closure solved with faer pivoted LU and a backward-error check; otherwise the
diagnostic is unavailable and the fit result is retained. Optional dense rank
diagnostics (faer SVD of the importance/uncertainty-weighted, parameter-scaled response)
have their own cell cap and reservation; refusal preserves the sparse candidate. An
estimate is qualified only with a stationary or optimal qualification, original
feasibility and full response rank; a rank-deficient response yields an unqualified
estimate with its rank and condition reported. Convergence and local rank establish no
covariance or interval by themselves; the rule below states both with their validity, and
global identifiability is not claimed. Data reconciliation and multi-scenario stacking have
no separate templates; the shared-parameter experiment set is the multi-experiment form.

**Covariance** (Plan 22 S3, `workflow/fitting/covariance.rs`). Every fit with a free
parameter derives one covariance of its free parameters, published in
`runtime.parameter_covariances` with its approximation (`CovarianceApproximation`). A fit
whose requested Hessian mode is `exact` reads the inverse reduced Hessian over its parameter
columns, `B·K⁻¹·Bᵀ`, from the KKT factor of its own solve
(`kkt::Analysis::inverse_reduced_hessian`, [§15.5.1](numerical-execution.md#section-15-5-1)),
certified by the same LICQ, strict-complementarity and second-order verdicts; it is labelled
`exact`. Under a limited-memory or Gauss–Newton Hessian the covariance is
`Σ = S·V·diag(s⁻²)·Vᵀ·S` from the singular values `s` and right singular vectors `V` of the
weighted, parameter-scaled response `R·S·√importance/σ`, with `S` the declared parameter
scales, never forming `JᵀWJ`; it is labelled `gauss_newton`. The label follows the requested
Hessian mode. Here `exact` identifies the library-derived curvature used in the local
statistical approximation; it does not assert an exact sampling distribution or global
uncertainty law. The declared standard deviations are taken as absolute, so no residual
variance rescales the covariance. Admission keeps the declared-deviation rule: every
included observation needs a finite value, a positive standard deviation and positive
importance.

The covariance is withheld, with its reason in `runtime.local_validity`, for no candidate or
an estimate that is not stationary; an included observation whose importance is not one
(`nonunit_importance`), since the weighted loss is then not a likelihood; unavailable
responses (`responses_unavailable`); a response rank below the number of free parameters
(`rank_deficient`), the rank counting singular values above `rank_tolerance` times the
largest; a free parameter within its acceptance budget of a declared bound
(`parameter_at_bound`); or, for the exact covariance, a failed verdict or backsolve.
`runtime.response_directions` publishes the right singular vectors whenever the responses
exist, whatever the covariance's outcome: the first `rank` span the locally identifiable
subspace, and the others are the parameter combinations the observations do not determine.

**Intervals.** `FitProfile.uncertainty` (`FitUncertainty`: a confidence `level` below one,
optional `ProfileControls`, and whether to propagate to the predictions,
[§19.8](#section-19-8)) requests intervals, published in `runtime.parameter_intervals`. A
Wald end is `θ̂ ± z·√Σₖₖ`, with `z` the standard-normal quantile of `(1 + level)/2`. A
profile-likelihood end is the pinned value at which `√(2(f − f*))` reaches `√χ²₁(level)`:
two chains per free parameter, one per end, each point pinning the parameter on the fit
prepared once (`transform::Pinned`) and re-solved through the one NLP runner without a local
analysis, seeded from its chain's latest accepted point. A chain steps by a secant predictor
limited to doubling, halves its step on failure, and ends with a secant root once the end is
bracketed (at most 40 points and a relative tolerance of 10⁻³ by default). Every point is
published in `runtime.profile_points` with its seed, qualification, objective, statistic and
acceptance, and an end reached at a bound or stopped by its point budget says so
(`IntervalOutcome`). A profile needs a valid estimate, not full rank. With absolute
deviations both methods use the same quantile, so they agree on a linear model.

*Tested* by `linear_regression_covariance_analytic` (exact, limited-memory and Gauss–Newton
against `(XᵀWX)⁻¹`), `gauss_newton_covariance_labelled`,
`covariance_withheld_with_nonunit_importance`, `covariance_withheld_without_declared_sigma`,
`fit_uncertainty_admission`, `profile_likelihood_matches_wald_on_linear_model` and
`profile_chain_seeds_from_predecessor` (runtime units),
`inverse_reduced_hessian_over_solve_columns` (native backend units) and
`unidentifiable_fit_withholds_covariance` (conformance acceptance). The exact covariance
of a transient fit, read from the KKT analysis of a fit on IDAS second-order adjoints, is
*Tested* by `exact_transient_covariance_matches_gauss_newton`, and the `parameter_at_bound`
and `responses_unavailable` withholdings by `covariance_withheld_at_bound` and
`covariance_withheld_without_responses` (runtime units). The fit's covariance agrees with
parmest's within 1e-6 on a weighted linear regression
(`test_fit_covariance_agrees_with_parmest`, `just parity`).

### 19.5 Costing

The authored SSLW heat-exchanger seed composes design, material and tube-length tables,
source-reported pressure/length regions, CEPCI currency units and accounting accumulators.
Area/base and material evidence with no supplied region remains unknown. The selected seed
names those unknown-evidence families explicitly; its low-pressure upstream comparison adds
separate pressure extrapolation permission. Positivity remains a mandatory mathematical domain. This is a selected costing method and flowsheet-accounting demonstration, not a
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
> M1, implemented) and its inward bound tightening (M2a, implemented). Plan 22 M3 and M4
> (implemented) lower the constraint forms and disjunctions; M5a and M5b (implemented)
> lower complementarity and route its penalty realization, so phase appearance runs in all
> three forms; the initialization fixes (M2b) and results conditional on one commitment
> (M2c) are implemented ([§6.8](schema-and-relations.md#section-6-8)).

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

**Complementarity** (Plan 22 M5a). A declaration
`complements name[i in s]: (a >= 0, b >= 0);` ([§6.8](schema-and-relations.md#section-6-8))
states `0 ≤ a ⊥ b ≥ 0` and has no default realization. Every lowering keeps both members
nonnegative:

| Realization | Lowering | Declared equivalence |
|---|---|---|
| `smooth(f, width)` | One row `f(a, b, width) == 0` with the package's smoothing function, called as authored; with `math.smooth_min`, the CHKS function `(a + b − √((a − b)² + width²))/2`, the row holds exactly where `a·b = width²/4` with both members positive, so no inequality row is added and a square system stays square | `Smoothed` |
| `disjunctive` | Nonnegative slack columns equal to each member and one native SOS1 set over them (weights 1 and 2) | `Native` |
| `penalty(l1)` | The rows `a ≥ 0`, `b ≥ 0` and `a·b ≤ 0`, and the structural requirement `l1_exact_penalty` in the case structure (`pse.math.case-structure.v5`) | `ExactPenalty` |

The smoothing choice belongs to the author: the realization names the function and its
width, which may reference a parameter, so continuing the width rebinds values without a
new structure ([§14.4](mathematics-and-compilation.md#section-14-4)). Two other smoothings
were rejected: Scholtes' relaxation `a·b ≤ ε` is degenerate for interior-point methods at
its limit, and a Fischer–Burmeister row would be a second function for the meaning
`smooth_min` already carries. The disjunctive
realization's SOS1 set is a native form, so it routes to SCIP and is refused on HiGHS. The
`l1_exact_penalty` requirement enters the structure's identity and routing reads it as a
fact (Plan 22 M5b): only POUNCE's ℓ1 exact penalty honours it, as the author's selection,
and the result states that the penalty relaxes every row
([§18.7](numerical-execution.md#section-18-7)). The reference
thermodynamics package declares `ComplementarityVLE` (`equilibrium.pse`), phase
disappearance by temperature slacks complementary to the liquid fraction and to the vapor
fraction, with smooth, disjunctive and penalty definitions and nine flash fixtures, three
feeds in each form. *Tested* by `complements_parses_and_renders` (authoring units),
`complementarity_lowerings_keep_members_nonnegative` and
`smooth_complementarity_row_vanishes_on_eps_sq_over_4` (compiler units),
`smooth_complementarity_product_equals_eps_sq_over_4` and
`disjunctive_complementarity_refused_on_highs` (runtime units), and
`flash_phase_disappearance_agrees_across_realizations` (native acceptance conformance: on
every feed the smooth form on Ipopt leaves the equilibrium temperature within O(width²) of
the disjunctive form on SCIP, and the penalty form on POUNCE's ℓ1 route agrees with both).

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

Semi domains reach SCIP, which has no native semi variable, through the `semi(indicator)`
lowering of its export ([§18.10.1](numerical-execution.md#section-18-10-1)). An
initialization may hold the free discrete variables in its stage and homotopy steps (M2b),
and duals and sensitivities of a mixed-integer candidate are stated conditional on one
commitment (M2c), both in [§6.8](schema-and-relations.md#section-6-8).

### 19.8 Uncertainty

> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — uncertainty propagation
> Σ_y = J·Σ_θ·Jᵀ and covariance with PS-12 validity, valid only where every upstream
> validity row holds (Plan 22 S3–S4, implemented). ADR-0118 supersedes ADR-0107.

A parameter covariance ([§19.4](#section-19-4)) propagates to outputs as `Σ_y = J·Σ_θ·Jᵀ`
(Plan 22 S4, `workflow/uncertainty.rs`), published in `runtime.propagated_covariances`. `J`
is one of two first-order maps:

- a modeling step's parametric sensitivities
  ([§15.5.1](numerical-execution.md#section-15-5-1)), requested through
  `SensitivityRequest.propagation`: the covariance as a fit run published it
  (`ParameterCovariance`, which `RunResult::parameter_covariance` hands on) and the solved
  variables to propagate to. Parameters are matched by identity, and a sensitivity
  parameter the covariance does not name is held fixed;
- a fit's own responses over its included observations, requested by
  `FitUncertainty.predictions` (admitted when the square output covariance fits the fit's
  cell cap).

The result is valid while both its inputs are: its `LocalValidity` is their conjunction, and
a withheld input withholds it as `upstream_withheld`, naming that input and its reason.
*Tested* by `uncertainty_propagation_linear_exact` and
`propagation_withheld_when_upstream_withheld` (runtime units). Propagation is first-order
only, and robust optimization is not implemented.

## 21. The Python boundary

> Decision: [ADR-0160](../../adr/0160-centralize-testing-responsibility.md)

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
path. Parity fails rather than skips and exercises the environment and explicitly selected
scientific reference comparisons; it does not establish
full numerical equivalence
([relationship to IDAES](../../relationship-to-idaes.md)).

### 21.1 Extension module, jobs and Arrow streams

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed target). Native typed problem/revision/run/output selectors and bounded Arrow streams replace Runtime, modeling-knowledge and TableReader SQL convenience. Streams protect their exact immutable selection and treat rows as provisional until successful statement completion. Ordinary runtime construction is durable; ephemeral execution is explicitly requested.

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> shared vocabulary (restating ADR-0090);
> [ADR-0116](../../adr/0116-typed-boundary-documents.md) (superseding ADR-0113) — typed backend settings, registry names and typed
> eligibility across the boundary (Plan 22 A5, implemented), with published JSON Schemas and
> generated Python document types (Plan 22 B5, implemented);
> [ADR-0114](../../adr/0114-typed-operational-store.md) — durable runtimes, the publication
> catalog, studies and the operational query surface (Plan 22 O3–O9, implemented).

`pse.Runtime(EngineSettings, store=None)` binds the shared runtime and memory budget.
The process-owned resource service retains one configured budget, spill directory and executor;
conflicting settings refuse. Escaped Arrow buffers retain their accounted owners. Test sessions
share this immutable deployment configuration; runtime facades, admitted packages, databases and
mutable journey effects remain owned by their individual test. With `store=pse.OperationalStore()` the runtime is durable
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
registry spellings ([§18.7](numerical-execution.md#section-18-7)). A sensitivity request
travels in the typed `SolveSettings` document (`SensitivityRequest`, with an optional
`Propagation`; [§15.5.1](numerical-execution.md#section-15-5-1)), and `prepare_fit` takes the
derivative source (`FitDerivatives`) and the uncertainty request (`FitUncertainty`)
([§19.4](#section-19-4)); their results are ordinary result tables. `SimulationSettings`
takes the dynamic sensitivity (`DynamicSensitivity`) and the adjoint checkpoint settings
(`AdjointSettings`); scheduled inputs and shooting are Rust-only today
([§13.6](#section-13-6)). Owners are `crates/pse-py/src/workflow/`,
`python/pse/_modeling.py`, `_runs.py` and `_strategies.py`.
The removed model builders have no compatibility facade.

`prepare_solve`, `solve_case`, native conformance and pure conformance accept the
existing generated `PreparationSettings` document: one complete compiler profile and
expansion policy. A provided policy reaches the existing Rust preparation owner unchanged,
including its finite conserved class-proof allowance; it replaces standalone expansion
limits for that operation. An omitted policy retains the Rust compiler default and the
selected package or explicit pure-expansion limits. Python declares no compiler defaults.
Conformance manifests forward the same document in both execution modes. Missing required
fields refuse rather than silently defaulting, and byte admission remains independent of
conservative symbolic work (§14.3.2).

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
> current payload is at version 8 (`JobPayload`, including admitted study operations and finalization).
> Complete nested preparation contracts use study operation/request version 4 and stored
> study definition version 6, with SolveSettings v3 (ADR-0155's authorized automatic cutover). Durable readers check envelope versions before nested decoding;
> unsupported historical readmission preserves the recorded bytes and refuses explicitly.
> [ADR-0115](../../adr/0115-registry-typed-identities-and-vocabularies.md) — every enumeration crossing the boundary is a registry enum with one Rust type
> (Plan 22 B4, implemented).

Python contracts are generated from the registry into `python/pse/contracts/`
([§4.2](schema-and-relations.md#section-4-2),
[ADR-0051](../../adr/0051-generated-trees-and-regeneration-check.md)): frozen attrs
classes per relation row, enums, value types and Arrow extension types. Generic modeling declarations are generated from the same owner; no hand-written
class mirrors a relation. Entity identities are generated `NewType`s
(`pse.contracts.identities`). The native API stubs (`_native.pyi`) are generated from the
compiled extension's metadata.

Package document mappings accept exact `str | bytes` values through one Rust converter;
Parquet documents remain binary. Typed fit selections replace the old measurement-relation
API. Read-only knowledge inspection and explicit fitted-parameter export retain their
source revision; the existing resource report projects pool and process high-water marks
separately.

**Boundary documents.** The Rust serde type owns each Rust-owned document: the backend,
solve, Diffsol and IDAS settings, the job payload, the termination detail, the source
manifest, workspace/publication/export/settlement, completion, fitting declarations and
preparation, knowledge/inspection, flow/tear/recycle, initialization, simulation profiles,
study request/definition/outcomes/status/cancellation and complete boundary diagnostics. schemars derives its JSON
Schema (draft 2020-12) into `docs/generated/schema/`, and a closed emitter
(`pse-codegen::codegen::documents`) turns the schemas into frozen msgspec `Struct` types in
`python/pse/contracts/documents/`: every
enumeration is the registry's generated enum, every object refuses unknown fields, and a
schema construct outside the mapping is a generation error. ADR-0116 Outcome 7 allowed
datamodel-code-generator or this emitter; the emitter was chosen because the external
generator emits `Any`, duplicate enums and unfrozen structs. Python's `SolveSettings`,
`BackendSettings`, `DiffsolSettings` and `IdasSettings` are these types, and the native
entry points decode the encoded document, so no `**fields: object` signature remains.
Single-value setting domains are validated types (`Tolerance`, `Fraction`,
`PositiveCount`, `FiniteBound`, built with nutype) refused at decode with a typed cause;
`admit_settings` keeps the cross-field and environment rules. `SimulationSettings` stays a
native class that takes an encoded document; fixture execution policies are authored data
([§6.10](schema-and-relations.md#section-6-10)).

Operation-owned request/result types are registered directly for generation; there is no
parallel hand-written schema or Python mirror. Python sends typed documents through the
common codec, then Rust admission resolves contextual identities and defaults. Closed
getters and their compiled stubs expose generated enums, while explanations stay separate.
Source-authoring grammar schemas remain distinct from hydrated serde document schemas.

Row collection cardinality uses attrs validators; uniqueness uses linear equality-consistent
keys derived by admitted element type. Nested records, sequences and reordered maps retain
their equality contract, including equal signed zeros; content hashes are not equality keys.
The emitter uses deterministic petgraph dependency ordering with attributable cycle/missing
reference errors. Common strum mechanics own vocabulary iteration, parsing and spelling;
operation-specific semantic projections remain with their owner.

> Supplement: [ADR-0151](../../adr/0151-generated-operation-boundaries.md)
> (proposed; authorized implementation).

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
