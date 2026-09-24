---
title: Native dynamics and parameter fitting
status: complete
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
evidence: Implemented — targeted native and Python contract evidence below; M22 retains scientific and integrated qualification
---

# M17–M18 execution packet

Implements the approved native dynamics and fitting scope of Plan 14 on M00–M16.
Steady, smooth transient and mixed fitting use the existing compiler, physical
providers, native NLP adapters, admitted runtime and publication protocol.
Completion here means implementation and targeted contracts, not M22 qualification.

## Completed packages

| Package | Implemented deliverable | Targeted control |
|---|---|---|
| M17.0 | Diffsol behind `solver-diffsol`, included in `native-solvers`; proposed ADR-0083/0084 contract updates | Default and linked profile compilation |
| M17.1 | Explicit derivative coordinates and selected outputs through `CasePlan`, bounded Salsa function/partition queries and existing artifact service | Parameter derivatives independent of optimizer variables; composed physical runtime fixture |
| M17.2 | Generated dynamic declarations; complete physical types, canonical normalization, time conversion, fixed diag(I,0), initial functions and algebraic matching | Non-SI time and nontrivial state offset/scale; state-dependent initial-condition refusal |
| M17.3 | Native Diffsol equation operators, BDF, sparse faer solve and consistent initialization; contained fallible callbacks | Analytic ODE and index-1 consistency/forward sensitivity controls |
| M17.4 | Finite sampling, rewind to native root time, reset/mode/input ordering, cancellation and partial results | Endpoint reset/input, native failure prefix, cancellation and step limit |
| M17.5 | Native forward sensitivities; generated FeOS factory and conserved-state vessel construction recipe | Initial and direct output derivative terms; ordinary compiler/provider contracts |
| M18.1 | Existing observations/datasets plus generated fitting bindings; physical units, uncertainty, importance and fixed least-squares loss | Weighted objective, gradient, exact Hessian; invalid uncertainty and ownership refusal |
| M18.2 | Simultaneous steady experiments with shared parameters and original physical constraints in the native NLP | Fixed/free identity, source retention, all-fixed joined evaluation |
| M18.3 | Smooth transient and mixed `NlpOracle`; inline integrations, native forward sensitivities and limited-memory Hessians | Mixed steady/transient objective, gradient and response |
| M18.4 | Physical responses, regular implicit closure solve and local faer rank diagnostics | Analytic response, feasibility/regularity refusal, scaled singular values |
| Shared | Public prepared operations, one joined lifecycle, retained Arrow tables, full native metrics and existing publication | Repeated waits, final-buffer lifetime, source/edit roundtrip and Python settings |
| Close | Removed unused DAE placeholder; current plans, proposed decisions and workflow documentation updated | Concrete consumers and generated boundaries compile |

## Compiler, physical and ownership contracts

`CasePlan::functions` selects actual output contributions before evaluation and
accepts explicit variable/parameter derivative coordinates. Unrelated outputs do
not execute. Bounded Salsa queries own the immutable function plan, derivative
requests and algebraic structural matching. Trial values, integrators, evaluators,
optimizer caches and factors remain worker-local. Existing Symbolica/Numerica
artifacts own expression derivatives; no alternative symbolic language is added.

`authored.dynamic_cases`, `authored.fit_cases` and `authored.native_providers` are
schema-generated source contracts alongside existing datasets and observations.
They participate in immutable model identity, edit/document roundtrip and retained
publication sources. Dynamic units use registered physical operations: a rate must
have the declared state's time-derivative quantity, not merely a compatible-looking
dimension. Native elapsed time is seconds. State offset/scale is canonical physical
normalization; algebraic residual scaling and numerical tolerances stay explicit.

Dynamic states are free continuous variables. Other inputs retain fixed/parameter
semantics. Initial functions may depend on time and parameters but not dynamic
states. Each mode has the same state and mass layout. Complete structural matching
of the mass-zero block is necessary admission evidence, not proof of numerical
regularity, solvability or index over the whole trajectory.

`MathService` supplies existing CPU/pool admission and completion-owned threads.
Fitting integrations execute inline inside the admitted outer job. They acquire no
nested job permits. Immutable compiled programs may be shared; mutable native state
cannot. Results and exported Arrow buffers retain their allocation owners. Logical
cell/history limits and conservative foreign-memory allowances are implemented;
these are not measured RSS caps.

## Library-owned dynamics

Diffsol 0.16.2 owns BDF, error control, nonlinear consistency, root location,
interpolation and forward sensitivity equations. The admitted mass is diag(I,0),
fixed across modes and independent of state/time/parameters. Operators supply known
sparsity and analytic derivatives; no NaN-based discovery is used. Native faer
context and explicit faer products/factors use sequential execution under the outer
admission. Complete pinned initial-condition and ODE options remain available in
Rust and roundtrip through Python `SimulationSettings` JSON; they enter profile identity.

Diffsol operators are infallible. A private Rust-only abort signal carries a recorded
typed callback failure to a catch boundary around the owned native operation.
The affected solver is discarded; no unwind crosses C/Python, no global panic hook
changes, and no NaN or stale output substitutes for failure. Native counters and
completed samples remain available when a callback fails after useful progress.

Sampling and events are finite. Completed samples survive failure; unfilled output
does not become fabricated data. Root state is rewound to the native root time
before reset. Simultaneous actionable roots and immediate ambiguous retriggers are
refused. Root handling precedes a coincident scheduled input change; regular
coincident samples observe the post-event consistent state, including at the final
time. Terminal roots retain a coincident final sample. Smooth forward sensitivities
exclude roots, resets and discontinuous input changes. Output derivatives include
both propagated state response and direct parameter terms.

The named FeOS factory reconstructs the existing methane/ethane/propane provider
from generated physical ports and component/reference identity. `ModelBuilder::vessel`
constructs ordinary generated definitions for conserved amount/internal energy and
algebraic temperature/density/pressure, with fixed composition and optional valve
outflow. It introduces no numerical engine. Its actual balance, valve, property
validity and independent physical-reference journeys remain M22.

## Native fitting and interpretation

Fitting reuses authored datasets/observations. Included measurements have finite
values, positive standard deviations and positive dimensionless importance. The
objective is `0.5 * sum(importance * ((prediction - value) / sigma)^2)`.
Point values use affine conversion; standard deviations use difference conversion.
Replicates retain their IDs; elapsed simulation time is distinct from timestamps.
Excluded measurements do not demand a prediction or contribute to the loss.

Free shared parameters occupy the first fit columns in declaration order; each
steady experiment's free continuous states follow in canonical source order.
Experiment-specific state aliases remain attributable to original source IDs.
Required tolerance/scaling vectors follow this assembled layout. Parameter scales
used for rank diagnostics are separate from optional native numerical scaling.

Steady experiments retain the simultaneous physical NLP. Transient responses use
single shooting and Diffsol forward sensitivities; any transient block requires
native limited-memory Hessians. The same `NlpOracle`, Ipopt/POUNCE adapters and
qualified library presolve pipeline are reused. Required unsupported presolve or
derivative profiles fail. An all-fixed fit evaluates directly, reports physical
quality and refuses required native passes it cannot execute. It invents no native
solver success. Failed trial points cannot reuse earlier predictions.

The intrinsic squared loss uses faer norms/products and library-generated local
partials; it does not require another authored expression graph. Exact steady
Hessians include both the Gram term and weighted local second derivatives. Final
candidate values and physical constraints are independently reevaluated.

For a regular square steady equality closure, physical response solves
`F_x * dx/dp = -F_p` with faer pivoted LU, then composes `H_x * dx/dp + H_p`.
Feasibility, scaled numerical rank and solve residual are checked. The observation
Jacobian is weighted by uncertainty/importance and declared parameter scales before
faer SVD. Inequality/rectangular/singular closures decline this diagnostic while
preserving the fitting result. These are local response/rank observations, not
active-set sensitivity, covariance, confidence intervals or global identifiability.

## Public results and publication

Prepared simulations/fits use the same `RunHandle`, cancellation, joined terminal
result, Python blocking/async waits and retained Arrow tables as ordinary solves.
The generated result families are `computation_runs`, `simulation_samples`,
`simulation_events`, `response_sensitivities`, `fit_parameters`, `fit_variables`,
`fit_constraints` and `fit_observations`, alongside existing `solve_metrics`.
Fit states/constraints preserve original experiment/source identities, units, bounds
and tolerances. Native options, defaults, metrics, progress and presolve receipts use
the ordinary solver report encoder. Absence, native termination, physical feasibility
and diagnostic availability remain distinct.

The existing one-use publication attempt writes exact retained declarations/results
through member writes and control-last settlement. It does not rerun a model or
introduce a second storage protocol. Full write/reopen/fault journeys remain M22.

## Verification

**Tested:** `just unit-dynamics-fitting` runs the selected native dynamics, fitting,
compiler-coordinate and workflow units with `pse-runtime/native-solvers` and explicit
`pse-relations/force-validate`: **20 passed, 0 failed**, with 148 tests outside
this selection. The failure baseline/target is zero.
The existing local Symbolica license environment is used for parallel native units;
license values are never part of source or reports.

**Tested:** `just unit-native-contracts` covers the broader existing native adapter,
compiler, math, ABI and lifecycle units under the same feature/validation conditions:
**114 passed, 0 failed**, 67 outside this selection; zero failure baseline/target.
`just py-native-contracts` exercises Python unit contracts against the actual
`just py-sync-native` editable extension and native runtime library environment:
**27 passed, 0 failed**, `unit` marker, zero failure baseline/target. This includes
complete native settings roundtrip and the all-fixed public fit/source-edit lifecycle.
These selections overlap; counts are not additive scientific coverage.

**Interface-checked:** `just check-package pse-runtime` (default profile, all targets,
force-validation) and `just check-native-python` (linked native/Python profile)
complete with zero compilation errors. `just codegen-contracts` regeneration and
compiled-API stub generation through `just py-sync-native` pass. Generators, never
generated files, define the contracts. Cargo still reports the pre-existing
`proc-macro-error2 2.0.1` future-incompatibility notice; it is not a clean whole-plan
quality claim. Focused front-matter/relative-file-link checks pass for the seven
updated current-state documents.
Whole-repository format/lint/governance/docs/installed acceptance remains M22.

M22 retains actual vessel conservation/error/event journeys, known-parameter fit
recovery, independent sensitivity comparisons, complete Python execution/publication,
foreign memory/performance campaigns and scientific qualification. General implicit
DAE, hybrid gradients, adjoints, covariance and dynamic transcription remain deferred.

## Outcome

**Implemented:** the complete M17–M18 target uses existing compiler, math, native
solver and result/publication owners. M19 cleanup, M20 acceptance inventory, M21
implementation closure and M22 qualification remain open. ADR-0082–0084 remain
proposed pending formal review/decision and blueprint reconciliation.

**Mistakes corrected:** root callbacks initially risked using the solver's state past
the located event; explicit native rewind and endpoint ordering now have regression
controls. Output pruning now happens before evaluator construction. Initial and
direct parameter derivative terms are included. A first multi-workspace regression
run used the restricted Symbolica environment; rerunning with the already provisioned
local license resolved that environment failure. Pure contract generation now runs
on the writable host rather than the read-only native test-container mount.

**Deliberate refinements:** use fixed intrinsic least squares with faer products;
keep structural partition admission in Salsa; expose full pinned native options;
share the ordinary native metrics encoder; record original fit state/constraint
semantics rather than publishing only fitted parameter values. None expands the
claims to general DAE, statistical uncertainty or full scientific acceptance.
