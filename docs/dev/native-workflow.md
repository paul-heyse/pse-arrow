---
title: Public native model and result workflow
status: implemented
date: 2026-09-24
---

# Native workflow

**Implemented:** Plan 14's [M15–M16](../plans/14-m15-m16-execution.md) and
[M17–M18](../plans/14-m17-m18-execution.md) packets connect generated declarations,
the Salsa compiler, library-owned mathematics, native algebraic/dynamic solvers,
parameter fitting, the existing supervisor and exact publication. Targeted
contract tests establish the boundaries below. Full scientific and installed
workflow qualification remains M22.

## Declare and prepare

`pse_runtime::workflow::Runtime::from_shared` borrows a deployment's existing
`SharedRuntime`, schema registry and engine factory. It creates no executor or
independent resource budget. Python `pse.Runtime(EngineSettings(...))` shares
those owners with `pse.open`; settings must agree with the process's first runtime.

`authored.computation_models` is the generated declaration authority. Its model,
definition, lexical domain/group and selected-case rows retain authored DSL source,
physical identities, fixed/free declarations, instance bindings, row intervals,
objective sense and values. Rust aliases and Python `pse.modeling` aliases refer
to generated DTOs; neither introduces a mathematical intermediate representation.
Package documents use `computation_models/*.yaml` with a `computation_models`
section. They pass through the existing native document loader into the same
builder admission used by typed calls. Legacy template pipelines are not invoked
implicitly by this frontdoor.

Physical contexts come from `PhysicalInventory` or
`Runtime.physical_from_documents`. The latter takes a complete package-relative
source mapping including `package.toml`. Production has no implicit test catalog:
physical meanings, operations, prerequisites and unit conversions come from actual
admitted rows, retained for publication. The fixture currency catalog is never a
default. Multi-package Rust callers can assemble an `OwnedDocumentSet` through the
existing accounted package loader before physical admission.

Rust `ModelBuilder::freeze` and Python `ModelBuilder.freeze()` produce immutable
revisions. `edit()` creates an independent draft. Failed admission does not mutate
an earlier revision. Preparation atomically selects a revision's inputs and computes
its products under the existing compiler lock; revisions share derived Salsa caches,
not mutable native solver objects.

Python's mechanical sequence is:

```python
runtime = pse.Runtime(settings)
physical = runtime.physical_from_documents(physical_documents)
revision = runtime.from_declaration(generated_model_row, physical).freeze()
prepared = revision.prepare(
    case_id,
    pse.SolveSettings(
        intent="optimize",
        backend="auto",
        variable_tolerances=variable_tolerances,
        row_tolerances=row_tolerances,
    ),
    coefficients=False,
)
job = prepared.start()
result = job.wait()  # alternatively: await job.wait_async()
```

Physical tolerance and numerical scaling vectors follow canonical semantic-ID
order, excluding fixed variables. Row tolerances remain in each row's physical
quantity. Coefficient projection is explicit; requesting it for an unsupported
expression fails rather than silently choosing another representation. `route`
reports the admitted case route. `capabilities()` reports libraries linked into the
binary; a linked library is not evidence that every model is eligible for it.

The Python algebraic frontdoor exposes native optimization, square root/initialization
intent, explicit feasibility, finite sequences, common controls and native options.
Rust's `Runtime::native()` additionally exposes the existing typed cone, conditional
block initialization, flow/tear and declared-map services. Rust `ModelBuilder::provider` accepts typed native factories. The generated
`NativeProviderDeclaration` selects the durable `feos-light-hydrocarbons` factory
in Rust or Python; Python numerical callbacks are not admitted. Dynamic and fitting
models use generated declarations through the builder. Arbitrary cone and flowsheet
construction remains in the typed Rust services.

## Dynamics and fitting

`pse.modeling.DynamicDeclaration`, `FitDeclaration` and `NativeProviderDeclaration`
are aliases of schema-generated contracts. A builder's `dynamics`, `fit`,
`native_provider`, `dataset` and `observation` methods retain them in its immutable
source envelope. `edit()` preserves these rows. Package documents use the corresponding
`dynamic_cases`, `fit_cases`, `native_providers`, `datasets` and `observations`
sections; ordinary source admission still applies.

A dynamic declaration names an existing case, time parameter, free continuous state
variables, initial output rows, RHS rows per mode, selected parameter coordinates,
outputs and optional root/reset declarations. The mass is exactly diag(I,0), fixed
across all modes. Initial functions depend only on time and parameters. Differential
RHS quantities must be registered time derivatives of their state quantities.
Canonical state offsets/scales, algebraic residual scales and elapsed time in seconds
are distinct from declared port units. Complete algebraic structural matching is
necessary; numerical consistency still belongs to Diffsol.

```python
simulation = revision.prepare_simulation(
    dynamic_id,
    pse.SimulationSettings(
        start=0.0, end=10.0, samples=[0.0, 5.0, 10.0],
        atol=state_tolerances, parameter_scales=parameter_scales,
        sensitivities=True,
    ),
)
result = simulation.start().wait()
```

`atol` follows normalized state order; `parameter_scales` follows the declaration's
selected parameter order. Profiles bound steps, events, output cells and wall time.
`SimulationSettings.to_json()`/`from_json()` retain all pinned native initialization
and ODE options as well as scheduled parameter changes. Unknown fields and invalid
controls fail admission. `capabilities()` advertises Diffsol only when linked.

Diffsol owns BDF, consistent initialization, root location and interpolation. Events
rewind to the located time, apply reset/mode and coincident input changes, then
reinitialize before sampling. Simultaneous actionable roots are refused. Smooth
forward sensitivities include initial and direct output parameter terms; they reject
roots/resets/input discontinuities. General implicit DAE, hybrid gradients and adjoints
are outside this profile. Completed output survives a later failed integration.

Rust's `ModelBuilder::vessel(VesselRecipe)` is a declaration recipe for fixed-composition
conserved amount/internal energy with algebraic temperature, density and pressure.
It uses the same FeOS factory and expression compiler. Its documented role map
requires explicit physical ports, operation contracts, values and positive scales;
optional valve outflow requires the positive pressure-drop domain. The recipe is
implemented; its physical process validation remains M22.

A fit declaration binds shared fixed/free parameters, steady or dynamic experiments
and existing observation IDs. Included observations require finite physical values,
positive standard deviations and positive dimensionless importance. The loss is
`0.5 * sum(importance * ((prediction - observation) / std_dev)**2)`. Point unit
conversion includes offsets; standard-deviation conversion does not. Excluded
observations retain identity but have no requested prediction/loss. Observation
`time` is elapsed simulation seconds, separate from acquisition timestamps.

`revision.prepare_fit(fit_id, solve_settings, simulations=...)` accepts ordinary
native NLP controls and a mapping from dynamic experiment IDs to
`SimulationSettings`. Use limited-memory Hessians whenever a transient experiment
is present. Ipopt and POUNCE consume the same oracle/presolve pipeline; integrations
run inline under the outer job. Exact steady Hessians use compiled local second
derivatives plus faer products. Shared free parameters appear first in declaration
order, followed by each steady experiment's canonical free-variable order. Variable
and row tolerance/scaling vectors follow that assembled layout. Required unsupported
presolve passes fail; all-fixed fits evaluate directly and cannot apply native passes.

Reported responses are physical partials. A steady experiment needs a feasible,
regular square equality closure for an implicit response; otherwise that diagnostic
is unavailable while the fitting result is retained. faer SVD reports local rank of
the uncertainty/importance-weighted observation response with declared parameter
scales. This is not covariance, confidence intervals or global identifiability.

Dynamic/fit results use `runtime.computation_runs`, `simulation_samples`,
`simulation_events`, `response_sensitivities`, `fit_parameters`, `fit_variables`,
`fit_constraints` and `fit_observations`, plus the existing `runtime.solve_metrics`
(all result names carry the `runtime.` prefix). Fitting state/constraint rows preserve
original experiment/source IDs, units, bounds and tolerances. Native termination,
fresh physical quality and diagnostic availability stay separate. Native options,
progress and presolve receipts use the ordinary solver encoder. Exact source rows
and physical declarations are retained for the same publication workflow below.

## Transformation and result meaning

Continuous NLP uses the same POUNCE presolve wrappers for direct Ipopt and POUNCE.
`auto` enables qualified passes; `off` preserves coordinates; `explicit` accepts
`presolve_options` through POUNCE's registered option API and `required_passes`
which must qualify. Fixed library thresholds and opaque tape coverage remain
visible. Auxiliary reduction is never enabled automatically. General bounds, row
constants, source guards, actual tapes, consumed values and native maps participate
in qualification and identity.

The pinned affine library deliberately stands down when every column would be
eliminated. That case remains a native problem. Authored all-fixed cases instead
use the compiler's existing direct evaluation path. Neither is reported as an
invented native solver success.

Native finalization owns recovery. Independent original-model observation reports
raw constraints, equality residuals only for equalities, side-specific violations,
authored objective/sense, fixed values and parameters. KKT values are observations,
not certified sensitivities. The multiplier convention is
`L = sense*f + lambda*g - z_lower*x + z_upper*x`. Missing or invalid multipliers do
not become zero-valued sensitivity data. Complete native POUNCE statistics use its
serde contract; serialized null is explicitly recorded as unavailable or nonfinite.
HiGHS rays, IIS, ranges and separate relaxation results remain diagnostic data,
never replacement primal solutions.

For ordinary algebraic solving, the result relations are `runtime.solve_runs`, `runtime.solve_variables`,
`runtime.solve_constraints` and `runtime.solve_metrics`, plus the exact model and
physical declarations. Failed, limited, cancelled and unattempted steps are retained.
`diagnostics()` exposes structured native admission/execution causes; `progress()`
exposes bounded native events and their dropped count. Each table's Arrow C stream
owns its final buffers independently of the result and model handles.

## Completion and publication

Blocking waits release Python and check signals; interruption requests cancellation
and waits for the existing native supervisor to join before propagating the signal.
Asyncio uses pyo3-async-runtimes on the same process Tokio executor. Cancelling an
async waiter requests native stop, while the handle retains access to the eventual
terminal result. Native factorization may finish before cooperative cancellation is
observed. Repeated waits never rerun a solver.

`result.prepare_publication(base_directory_uri, workspace_id, parent=...)` constructs
a reviewable one-use `PublicationAttempt`. Preparation does not write. `commit()`
returns the exact control URI and version for `pse.open`. The command uses existing
member writes, conditional parent checks and control-last settlement. Attempt and
publication identities remain available after an error. Inspect native effect
settlement before retrying an unresolved write; no marker alone deduplicates retries.
Stored declarations rebuild mathematical products; CAS display text, Salsa handles
and native solver factors are never durable authorities.

## Development profile and verification

`just py-sync-native` installs the explicit native-solver editable profile and
generates stubs from the actual compiled API. `just py-native-contracts` runs the
boundary units with the pinned Ipopt/MUMPS runtime path. `just py-sync` remains the
default profile and can replace that extension with a different linked capability
set. For direct Python commands under the native profile, source
`scripts/native-solver-env.sh` and prepend `$IPOPT_DIR/lib` to `LD_LIBRARY_PATH`.
The compiled module's primary RUNPATH does not resolve transitive MUMPS libraries
on its own. Distribution repair and installed-environment qualification remain M22.

A clean native build also needs libclang's matching development resource headers.
When using a local LLVM installation, select its `LIBCLANG_PATH` and `CLANG_PATH`
together; `BINDGEN_EXTRA_CLANG_ARGS=-resource-dir=<clang-resource-directory>` can
select the directory reported by that Clang's `-print-resource-dir`. A runtime-only
libclang installation can appear to work while generated bindings are cached, then
fail fresh binding generation with a missing `stddef.h`. Use the same selection for generation, extension builds
and native test builds; do not mix resource headers from another LLVM version.

The execution packets record exact executed counts and remaining gates. Targeted native units
use `pse-relations/force-validate`. These are not convergence, storage fault, throughput,
RSS-bound, scientific-validity or general solver-coverage claims.

[M19–M20](../plans/14-m19-m20-execution.md) supplies shared authored process fixtures
in `tests/fixtures/plan14`, independent offline references, public Rust/Python
acceptance bodies and guarded Criterion workloads. `just plan14-discover` compiles
and enumerates the declared native selection without running it. Actual execution
via `plan14-native`, `plan14-python`, `plan14-measure` and `plan14-reviews` belongs to
M22 after M21 closure. See the [acceptance guide](architecture-acceptance.md) for
profiles, evidence contracts and the functional-before-performance sequence.

## Physical contracts after M21

A native FeOS provider declaration includes a finite `envelope` for temperature,
density, pressure and three ordered component fractions plus provenance. Pressure
is checked from the same state even when it is not an output request. NPT guesses
also obey the declared window. Outside-window trial failures remain recoverable;
an operating declaration is not an empirical accuracy certificate.

`ModelBuilder.balance(...)` accepts the generated physical-balance declaration.
Use typed inlet/outlet/generation/consumption/work/internal-transfer roles over source
outputs; the compiler derives the equation. Do not author a second balance row.
Internal transfers require matching opposite contributions. Dynamic declarations
bind a conserved state to its balance in every mode; optional term mode selectors
express changed physical fluxes. State reset impulses require explicit event IDs.

`SimulationSettings` requires `out_rtol` and one `out_atol` per conserved balance
when balances are present. Diffsol controls these integrated physical flux errors
separately from state tolerances. `runtime.physical_checks` exposes closure, canonical
unit, tolerance, nullable acceptance, errors and declaration provenance for solves,
simulation and fitting. Read it alongside native status and mathematical feasibility.
The same generated sources/results survive exact publication; M22 qualifies the
complete installed-process and reopen journeys.
