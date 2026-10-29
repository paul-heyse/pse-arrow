---
title: Public native model and result workflow
status: implemented
date: 2026-10-15
---

# Native workflow

**Implemented:** the callable Rust workflow connects generated declarations,
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
| `admit_study` / `study` | Owned local study and isolated outcomes with explicit accepted-predecessor dependencies; admission/execution need no store. `LocalStudy.execute()` retains its model and physical inputs. |
| `Runtime.record_study` / `start_study` / `execute_study` | Explicit exact-source recording yields a portable descriptor; start submits it to durable workers and execute joins its retained outcomes ([studies across workers](operational-store.md#studies-across-workers)). |
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

The native application uses `Runtime::from_shared_local(shared, bound_model)` for local
science. The application supplies its existing `SharedRuntime` and registry-bound engine
model; construction creates no executor, canonical service or Python dependency. Package
document ingress retains its manifest dependency closure and explicit physical definitions.
For already admitted `package`, explicit `request` and application-owned `runtime`, the
generic local composition is:

```rust
let stop = pse_runtime::CancelSource::new();
let prepared = package.prepare_operation(
    &runtime.preparation_services(),
    &pse_runtime::workflow::OperationSource::of(&package),
    &pse_runtime::workflow::PreparationSettings::default(),
    request,
    None,
    &stop,
).await?;
let run = prepared.start(&runtime.execution_services(), &stop).await?;
let result = run.wait().await?;
```

The [public local caller](../../crates/pse-runtime/tests/local_operation.rs) supplies complete
source admission and independent equation checks. The
[immutable workspace caller](../../crates/pse-runtime/tests/native_workspace.rs) edits values
and structure with `SourceDelta`/`with_delta`, then re-solves while retaining the earlier result.
Typed failures and partial results retain their original scientific scope. Detaching an
observer does not cancel its run; actual cancellation and native completion remain separate.
Specialized dynamics, strategies and host-only shooting use their existing native owners;
they need no persisted study variant merely to share this entry point.

`pse_runtime::workflow::Runtime::from_shared` borrows a deployment's existing
`SharedRuntime`, schema registry and engine factory, and attaches its explicit
`CanonicalDeployment`. It creates no executor or independent resource budget. Python
`pse.Runtime(engine_settings, substrate=state_directory)` shares the process owners;
settings must agree with the process's first runtime. Initialize the selected substrate
through the [operator route](operational-store.md#setup-and-runtime-ownership) before opening it.

`authored.modeling_declarations` is the generated generic declaration authority.
`models/*.pse` is the primary authoring surface. Document loading, direct generated
rows and immutable edits all reach the same checked package. The old computation-model,
scientific YAML and template builders have been removed.

Physical contexts come from `PhysicalInventory` or `Runtime.physical_from_documents`.
Production has no implicit test catalogue: quantities, operations, prerequisites and units
come from admitted source rows retained in canonical revisions. The fixture currency catalogue is
not a default. Package manifests bind quantity aliases; imports require the declared exact
package closure. Failed admission leaves earlier package revisions usable.

The retiring Python binding's sequence, with application-supplied source maps and a semantic case ID, is:

```python
runtime = pse.Runtime(engine_settings, substrate=state_directory)
physical = runtime.physical_from_documents(physical_documents)
package = runtime.modeling_from_documents(package_bundles, physical)
prepared = package.prepare_solve(
    case_id,
    pse.SolveSettings(intent="optimize", backend="auto"),
)
job = prepared.start()
result = job.wait()
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

Native design generation uses `ModelingPackage::generate_design` with typed physical
coordinates, bounded work and explicit cancellation. It starts no solver and needs no
canonical store. `GeneratedDesign::points` borrows the actual owned overlays; `export`
retains their realized content and attribution independently of the producer. Record those
realized points for study replay. A seed documents generation and does not replace the
recorded inputs. The [store-free native design caller](../../crates/pse-runtime/tests/native_design.rs)
exercises all six supported kinds, physical geometry, duplicate occurrences and refusals.
The Python design-generation projection is retired; native generation remains unchanged.

The same source definition selects integrated or simultaneous analysis. Integrated
preparation derives states, rates, algebraic rows, initial conditions and outputs from
continuous declarations. Diffsol/IDAS own stepping and consistency for the supported
fixed `diag(I,0)` ODE/index-1 profile. Authored finite-difference and Radau schemes lower
to ordinary algebraic cases for simultaneous time or spatial analysis.

Physical coordinate inputs are Arrow producers with the registered relation schema.
Import its complete metadata with `pyarrow.schema(runtime.physical_input_schema(
PhysicalInputKind.STATE_TOLERANCES))`; provide each coordinate's actual identity,
quantity-type identity, representation-unit identity and positive Delta value. The
existing runtime owns an immutable scalar snapshot before preparation starts.

```python
from pse.contracts.enums import DynamicSensitivity

simulation = package.prepare_simulation(
    case_id,
    pse.SimulationSettings(
        runtime=runtime,
        start=0.0, end=10.0, samples=[0.0, 5.0, 10.0],
        atol=state_tolerances, parameter_scales=parameter_scales,
        sensitivity=DynamicSensitivity.FORWARD,
    ),
)
result = simulation.start().wait()
```

Physical Delta state tolerances are converted into the prepared state units and
resolved numerical scales once. Characteristic parameter scales and integral tolerances
use their own identity-keyed Delta inventories. Point offsets never apply to these
Delta values. Physical time remains signed binary64 seconds, preserving subsecond
precision. Parameters, time origin, horizon,
quadrature tolerances, events and modes are explicit. Native limits bound steps, output
cells and time. Unsupported index structure, hybrid IDAS sensitivities or noncausal
integrated expressions refuse. Completed samples survive a later failure. State/output
sensitivities may coexist with terminal quadratures; integral sensitivities are unavailable.
Authored checks apply at the recorded samples and do not establish validity between them.

A generated `authored.fit_cases` row binds shared parameters and experiment/output
source paths to admitted observations and datasets. Add them with `with_fit_data` or
load their document sections. `package.prepare_fit(fit_id, request)` consumes a required Version 1
`PhysicalFitRequest`; its shared fitting settings contain pending physical simulation
requests by experiment identity. Study Version 8 persists the same pending intent and
admits it against each actual prepared model. Native replay retains the actual admitted
integration profiles. The existing sparse fitting owner performs the fit. Steady experiments retain original equations;
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

Retained native reads can construct `RetainedDataServices::new(store, bound_model)` directly
from an opened canonical store and configured engine model. This creates an independent
logical caller without constructing `SharedRuntime`, source preparation or mathematical
services. Closing that caller seals new admission; protected returned data retains its actual
custody. Extracting `runtime.retained_data_services()` instead shares that runtime's logical
caller, including its close behavior.

`FrozenStudyCohort::relation` supplies a protected `TableProvider`, its configured
`SessionState` and traversal/classification information. Use that state with DataFusion's
native DataFrame joins, filters and aggregates. Native states deliberately omit a mutable
default catalog; `read_table`, `alias` and `join_on` compose captured providers directly.
An unrelated deployment pool refuses. Preserve quantity/unit identities and distinguish
unavailable captured members from empty selections and numerical zero.

The [native repeated-analysis caller](../../crates/pse-runtime/tests/native_repeated_analysis.rs)
compares original and corrected engineering contexts after scientific shutdown, retains the
earlier frozen population, and replays recorded realized points under their original source
and context. Domain effective time and revision ancestry remain distinct from storage MVCC.
Native `FrozenStudyCohort::aggregate` and `RetainedDataServices::export_study_analysis`
retain aggregation/export; the Python `aggregate`/`StudyKpiAnalysis` projection is retired.

Native waits are asynchronously repeatable and never rerun a solver. Cancellation
requests stop; the original native supervisor owns actual completion and resource drain.
Native factorization may finish before cooperative cancellation is observed. The retiring
Python binding's blocking wait releases the interpreter and checks signals; interruption
requests cancellation and joins the supervisor before propagating the signal. Its asyncio
run-wait bridge is removed; unrelated asynchronous cursor adapters retain their own owners.

Ordinary runs retain source, attempt and scientific result identities in the canonical
store. Save `result.canonical_run_key` and `result.canonical_attempt_key`; reopen a relation
with `runtime.results(run, attempt, relation)`. Narrow run/attempt/manifest selectors,
output selection, Arrow export and persisted analysis graphs use the same substrate.
Readers protect exact storage inputs while paging. Fully decoded Arrow arrays retain their
accounted allocation independently of the reader and database; they need no further storage
access or indefinite database pin. Failed and partial runs keep their actual scientific
meaning. See [the canonical operational store](operational-store.md).
Stored declarations rebuild mathematical products; CAS display text, Salsa handles
and native solver factors are never durable authorities.

## Development profile and verification

Rust correctness commands run through the prepared native environment and explicitly
validated Cargo route described in [the environment guide](agent-environment.md).
Product execution does not require installing a Python extension. For remaining Python
consumers, `just py-sync-native` installs the explicit native-solver editable profile and
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
findings and candidate assessments alongside solver termination. Canonical retention preserves the
same source documents, generated declarations and result contracts.

Build caching, persistent native prefixes and the parallel frontend experiment are
documented in [Rust build reuse](build-performance.md).

## Durable control without scientific loading

Native `CanonicalStore` owns compact browsing, study cancellation and original receipt
reconciliation independently of scientific `Runtime`, `SourceServices`, mathematical services
and Python. Open the explicitly selected initialized deployment through its native connection
owner; ordinary connection startup neither provisions actors nor installs schema. Use
`control_study_workbench_page` with a bounded `StudyWorkbenchRequest`, `cancel_study_command`
with a retained `ControlCancelRequest`, and `reconcile_study_cancel` with the original
`ControlReconcileRequest`. The types live in `pse_operations::canonical_control`.

Retain the command's issuing store, operation identity and immutable study target until its
outcome settles. Identical repetition observes the original decision; changed intent under
that identity refuses. A missing receipt is unobserved, not proof of rollback or permission to
issue a new effect. Restore or foreign selection cannot retarget an old request. A receipt
records durable stop intent, while the original worker/native/process owners establish joining
and physical release. Cohort capture, scientific observation and protected payload readers
retain their existing capabilities.

The constrained record-actor API remains available for an independently selected external
consumer. It requires `Content-Type: application/json`, preserves opaque strings and checks
grants before parsing/state/receipt access. Explicit `issue_control_actor` is privileged and
refuses existing identities rather than overwriting grants. The existing optional
`pse-canonical-init --issue-control-actor` command uses that same owner; it never installs
schema implicitly. The Python `pse.control` client and its retry/lifetime harnesses are deleted;
there is no replacement Python proxy. Plan58 D04/G03 own remaining native consumer integration,
and H owns assembled qualification.

## Independent native reference exchanges

The `pse-runtime` targets `reference_degeneracy_exchange`, `reference_covariance_exchange`,
`reference_sensitivity_exchange` and `reference_pid_exchange` construct the candidate through
public Rust APIs. Their isolated Python processes run only the independent IDAES/Pyomo
reference side; they need no product extension or Python candidate adapter. Missing reference
packages/executables fail. Keep the declared parity environment and explicitly prepared native
capabilities; ordinary reference execution does not install them.

For the fitting, sensitivity and PID selection, prepare an exact source capture once and run:

```bash
reference_inputs_dir=$(mktemp -d)
scripts/pse-env -- cargo run --locked -p xtask --features physical-admission,force-validate -- \
  scientific-inputs tests/fixtures/scientific-inputs.toml "$reference_inputs_dir/inputs.json"
export PSE_SCIENTIFIC_INPUTS="$reference_inputs_dir/inputs.json"
export PSE_COVARIANCE_REFERENCE_PYTHON="$PWD/.venv-parity/bin/python"
export PSE_SENSITIVITY_REFERENCE_PYTHON="$PWD/.venv-parity/bin/python"
export PSE_PID_REFERENCE_PYTHON="$PWD/.venv-parity/bin/python"
scripts/pse-env --native=compiler,solver,klu -- /bin/sh -c '
  export PATH="$IPOPT_DIR/bin:$PATH"
  exec scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast \
    -p pse-runtime --locked --features solver-ipopt,solver-diffsol \
    --test reference_covariance_exchange --test reference_sensitivity_exchange \
    --test reference_pid_exchange
'
```

The degeneracy target selects `solver-highs`, `compiler,solver` and
`PSE_DEGENERACY_REFERENCE_PYTHON` instead. These selected exchanges retain scientific tolerances,
physical interpretation and adverse-result comparisons. They do not constitute all-family parity
or assembled deployment qualification.
