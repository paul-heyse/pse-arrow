---
title: Public native model and result workflow
status: implemented
date: 2026-10-06
---

# Native workflow

**Implemented:** the public Rust and Python workflow connects generated declarations,
the Salsa compiler, library-owned mathematics, native algebraic/dynamic solvers,
parameter fitting, the native supervisor and canonical retained results. Targeted
contract tests establish the boundaries below. The architecture owners are
[§13, §19 and §21](../authoritative_design/sections/workflows-and-results.md),
[§18](../authoritative_design/sections/numerical-execution.md#section-18) and
[§20](../authoritative_design/sections/identity-and-publication.md#section-20); the
current qualification basis is
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2).

## Source modeling operations

`Runtime.modeling_from_documents(bundles, physical)` admits an explicit closure of package
source maps, including `.pse` definitions, cases and tests. Each manifest declares its
dependencies and package-scoped quantity aliases against the admitted physical context. The resulting `ModelingPackage` uses the same
compiled definition for steady solving, integrated time simulation and simultaneous
discretization. Native capabilities are supplied explicitly by the Rust composition root;
source strings do not instantiate external implementations.

| Operation | Result meaning |
|---|---|
| `solve_case` | Original native outcome plus independent source checks, closure and reports |
| `initialize` | Ordered immutable stage/homotopy attempts; only an accepted final original specification commits values |
| `study` | Isolated case outcomes and explicit accepted-predecessor dependencies; with `runtime=` a durable study run by workers and retained in canonical storage ([studies across workers](operational-store.md#studies-across-workers)) |
| `simulate` | Native trajectory, actual termination and checks at requested sample times |
| `diagnose` / `diagnose_samples` | Bounded structural and numerical findings at declared points; missing free values remain missing |
| `diagnose_linear` | HiGHS IIS, rays, ranging and explicitly penalized relaxation of an affine model |
| `diagnose_jacobian` | LP/MILP evidence about local scaled-Jacobian dependence |
| `explain_nonlinear` | Bounded elastic deletion evidence under unchanged bounds; local obstruction and inconclusive outcomes remain distinct |
| `conform` | Data-authored fixtures receive the shared preparation, DoF, derivative, envelope, start-to-solve, closure and expectation checks |

Result handles and their `table()` exports own their data independently of the source
package. Generated `runtime.modeling_*` relations retain semantic coordinates and attempt
identities. Diagnostic Jacobians retain row and variable nominals alongside their scaled
singular vectors. Nonfinite candidate values retain explicit infinity/indeterminate tags;
they are never replaced by zero. Diagnostic points cannot change fixed or parameter values consumed by
compilation; prepare another case for those changes. A sample may supply a previously
missing free coordinate. Numerical solve admission still requires complete finite starts.

A case's fixture declares its modes and events (`mode <name> facts(...)`,
`event <guard> direction(...) tolerance(...) reset(...) next(...)`) and its scheduled
inputs (`schedule <input> at(...) values(...)`). A `run shooting` fixture holds schedules
free as its controls (`... free lower(...) upper(...)`) and declares `shoot single;` or
`shoot multiple nodes(...);`; its shooting problem minimizes the model's one objective
level, and conformance solves it. A mode selects Boolean facts, such as an
authored stage, while keeping the physical state, parameter and output layouts fixed; an
event selects source guard and reset expressions. Event handling belongs to the native
integrator.
The mode recorded at a coincident sample is the mode after the reset. Sampled source
checks do not certify the intervals between samples; a native completed trajectory can
still fail a source check. Definite integrals become available only at the declared
terminal point; future-dependent trial equations require simultaneous realization.

Native diagnostic evidence has narrower scope than a model solve. A discrete model's
IIS concerns its continuous relaxation. Jacobian dependence is local to the recorded
point and scaling. Positive slack at a local stationary nonlinear solution does not
prove global infeasibility. HiGHS feasibility relaxation returns an operation code and
a separate candidate/weighted penalty; the restored original model status is labelled
separately. Ranging represents finite, infinite and indeterminate limits explicitly.

Pure local Rust checks use `CompilerWorkspace::check_modeling_expectations` after publishing
the source declarations into a workspace with the actual physical inputs. This path applies
authored fixture values and explicit candidate overrides, returns physical comparisons, and
constructs no runtime or native solver. External and nested capabilities require the workflow
harness. Python `ModelingPackage.conform` and `python -m pse.conformance` run that full harness;
their oracle references identify the authored assertion, not a live upstream execution.

## Declare and prepare

`pse_runtime::workflow::Runtime::from_shared` borrows a deployment's existing
`SharedRuntime`, schema registry and engine factory. It creates no executor or
independent resource budget. Python `pse.Runtime(EngineSettings(...))` shares
those owners with `pse.open_export`; settings must agree with the process's first runtime.

`authored.modeling_declarations` is the generated generic declaration authority.
`models/*.pse` is the primary authoring surface. Document loading, direct generated
rows and immutable edits all reach the same checked package. The old computation-model,
scientific YAML and template builders have been removed.

Physical contexts come from `PhysicalInventory` or `Runtime.physical_from_documents`.
Production has no implicit test catalogue: quantities, operations, prerequisites and units
come from admitted source rows retained in canonical revisions. The fixture currency catalogue is
not a default. Package manifests bind quantity aliases; imports require the declared exact
package closure. Failed admission leaves earlier package revisions usable.

The Python sequence, with application-supplied source maps and a semantic case ID, is:

```python
runtime = pse.Runtime(engine_settings)
physical = runtime.physical_from_documents(physical_documents)
package = runtime.modeling_from_documents(package_bundles, physical)
prepared = package.prepare_solve(
    case_id,
    pse.SolveSettings(intent="optimize", backend="auto"),
)
job = prepared.start()
result = job.wait()  # or: await job.wait_async()
```

Use `intent="root"` for square equalities without an objective, `"optimize"` for an
authored objective, or an explicitly selected supported feasibility intent. Physical
numerical requirements follow the selected semantic coordinate order. Native inventory
from `capabilities()` does not establish contextual eligibility. Preparation shares
immutable compiler products, never a mutable solver.

`package.with_declarations(...)`, `.with_limits(...)` and `.with_fit_data(...)` create
independent immutable views. Limits bound expansion and retained work; raising one does
not change physical admission or truncate a model. External capabilities are registered
by the Rust composition root, not constructed from source strings or Python callbacks.

## Dynamics and fitting

The same source definition selects integrated or simultaneous analysis. Integrated
preparation derives states, rates, algebraic rows, initial conditions and outputs from
continuous declarations. Diffsol/IDAS own stepping and consistency for the supported
fixed `diag(I,0)` ODE/index-1 profile. Authored finite-difference and Radau schemes lower
to ordinary algebraic cases for simultaneous time or spatial analysis.

```python
simulation = package.prepare_simulation(
    case_id,
    pse.SimulationSettings(
        start=0.0, end=10.0, samples=[0.0, 5.0, 10.0],
        atol=state_tolerances, parameter_scales=parameter_scales,
        sensitivities=True,
    ),
)
result = simulation.start().wait()
```

State tolerances apply to normalized coordinates. Parameters, time origin, horizon,
quadrature tolerances, events and modes are explicit. Native limits bound steps, output
cells and time. Unsupported index structure, hybrid IDAS sensitivities or noncausal
integrated expressions refuse. Completed samples survive a later failure. State/output
sensitivities may coexist with terminal quadratures; integral sensitivities are unavailable.
Authored checks apply at the recorded samples and do not establish validity between them.

A generated `authored.fit_cases` row binds shared parameters and experiment/output
source paths to admitted observations and datasets. Add them with `with_fit_data` or
load their document sections. `package.prepare_fit(fit_id, settings, simulations=...)`
uses the existing sparse fitting owner. Steady experiments retain original equations;
transient experiments integrate inline under the outer job, using limited-memory Hessians.
Mixed experiments share the same parameter vector. Included observations need a finite
physical value, positive uncertainty and importance. The loss is
`0.5 * sum(importance * ((prediction - observation) / std_dev)**2)`.

Original physical checks, bounds and fixed case values remain active through fitting.
Response derivatives and bounded faer rank diagnostics are local observations; rank or
convergence alone does not imply covariance, global identifiability or a qualified estimate.
The PC-SAFT vessel and its steady/transient/mixed fitting fixtures are authored package data.

## Conformance policies

The shared harness discovers all authored tests and reports coverage, preparation, DoF,
derivatives, envelopes, native execution, original checks and expectations. Pure fixtures
can run without solver services through `ModelingConformance.pure`. Oracle text identifies
the reference supporting the authored expectation; it does not execute an upstream tool.

A fixture's execution policy is authored with it: `policy { backend …; presolve …;
derivatives …; limits …; }` in the fixture header (blueprint §6.10). The run supplies the
defaults, which the policy overrides for that fixture only. Invalid policies refuse before
execution. No policy changes scientific declarations or expected outcomes.
`just seed-conformance` runs every reference fixture once, from
`packages/reference/conformance.toml`, and writes Arrow reports to `build/seed-conformance`.
`python -m pse.conformance --manifest <toml> --report-dir <dir>` runs any manifest.

The report retains every discovered fixture independently of its detailed row cap.
`complete=False`, inconclusive, cancelled and unattempted results cannot pass. Results,
initialization histories, trajectories and structured findings survive parent-handle drops.
A filtered focused run does not prove complete package coverage.

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

## Completion and retained results

Blocking waits release Python and check signals; interruption requests cancellation
and waits for the existing native supervisor to join before propagating the signal.
Asyncio uses pyo3-async-runtimes on the same process Tokio executor. Cancelling an
async waiter requests native stop, while the handle retains access to the eventual
terminal result. Native factorization may finish before cooperative cancellation is
observed. Repeated waits never rerun a solver.

Ordinary runs retain source, attempt and scientific result identities in the canonical
store. Save `result.canonical_run_key` and `result.canonical_attempt_key`; reopen a relation
with `runtime.results(run, attempt, relation)`. Narrow run/attempt/manifest selectors,
output selection, Arrow export and persisted analysis graphs use the same substrate.
Readers and returned buffers retain exact input protection. Failed and partial runs keep
their actual scientific meaning. See [the canonical operational store](operational-store.md).
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
on its own. The local native editable profile supplies this path through its recipes. Portable
distribution repair remains release work outside the local qualification scope.

A clean native build also needs libclang's matching development resource headers.
When using a local LLVM installation, select its `LIBCLANG_PATH` and `CLANG_PATH`
together; `BINDGEN_EXTRA_CLANG_ARGS=-resource-dir=<clang-resource-directory>` can
select the directory reported by that Clang's `-print-resource-dir`. A runtime-only
libclang installation can appear to work while generated bindings are cached, then
fail fresh binding generation with a missing `stddef.h`. Use the same selection for generation, extension builds
and native test builds; do not mix resource headers from another LLVM version.

Targeted native units use `pse-relations/force-validate`. These are not convergence,
storage fault, throughput, RSS-bound, empirical-validity or general solver-coverage
claims; the process, convergence and persistence evidence has its own scope.

Shared authored process fixtures in `tests/fixtures/plan14`, independent offline
references, public Rust/Python process tests and guarded Criterion workloads exercise
the workflow. `just native-test`, `just native-python <output>` and `just case-measure`
execute them. See the [qualification guide](validation-assessment.md) for profiles,
fixture generation, measurement conditions and the functional-before-performance order.

## Physical contracts

Quantity compatibility includes basis, datum, point/difference scale, subject and shape.
Scientific packages declare phases, components, equations and validity as generic data.
Original guards survive simplification and differentiation. Nested realizations retain
branch selection and refuse unproved derivative crossings; local tests do not certify
global stability or empirical property accuracy.

Generic accumulators collect signed original contributions. Conservation closure is
independent of native feasibility, and arbitrary failed authored checks cannot be waived
by a policy allowing unavailable closure. Read `runtime.modeling_checks`, structured
findings and candidate assessments alongside solver termination. Publication retains the
same source documents, generated declarations and result contracts.

Build caching, persistent native prefixes and the parallel frontend experiment are
documented in [Rust build reuse](build-performance.md).
