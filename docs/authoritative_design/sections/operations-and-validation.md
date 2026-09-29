---
title: Operations and validation
status: current
---

# Operations and validation

This area covers how the running system reports what it did and why it failed, and how
the repository establishes that the implementation behaves as its contracts say.
`pse-diagnostics` owns the failure vocabulary; `pse-columnar::engine` classifies native
engine errors; `pse-engine::session` owns engine observation policy; runtime result
relations carry execution facts. Test crates live under `tests/`, Python tests under
`python/pse/tests`, benchmarks under `benches/`; `just --list` is the command surface.

## 23. Observability and failure semantics

Observability explains execution; provenance explains derivation; results state
scientific meaning. They share identifiers but are never merged, and observing a
computation can never change its scientific or publication outcome
([ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md)).

### 23.1 Observability

**Spans.** `tracing` spans named `pse.*` wrap the expensive boundaries:

| Owner | Spans |
|---|---|
| `pse-compiler::workspace` | `pse.case.definition_admission`, `pse.case.structural_analysis`, `pse.case.program_optimization`, `pse.case.sparse_plan` |
| `pse-runtime::math` | `pse.case.program_assembly`, `pse.case.function_assembly` |
| `pse-catalog::delta` | `pse.delta.open`, `pse.delta.write_attempt`, `pse.delta.commit`, `pse.delta.maintain` |
| `pse-engine::session` | `pse.operation` around native query, round and command execution |

Fields known only at the end of a span are declared `tracing::field::Empty` at creation
and recorded later; `#[instrument]` uses `skip_all` or explicit `skip` with explicit
`fields`. Library crates install no subscriber; tests (`pse-testkit`) and benchmarks
install their own.

**Engine observation.** `pse-engine::session::assurance::ObservationPolicy` is `Off` by
default. `Contract` installs `datafusion-tracing` instrumentation once per session state
(physical operators and optimizer phases, target `pse_engine::execution`) and records
explicit operation completion. `Diagnostic` adds bounded plan text and rule names, never
value previews. Captured plan observations are attempt-local, bounded and charged to a
diagnostic reservation; they are diagnostics, not output authority.

**Execution facts as data.** Completed runs project typed relations rather than log
text: `runtime.computation_runs` (state, native termination, qualification, candidate
facts), `runtime.solve_metrics` (effective options and native metrics, with typed
unavailable reasons instead of invented values), `runtime.execution_statistics`
(planning, load and acceleration counts) and `runtime.diagnostics_findings`. Native
solver progress is exposed as events on the joined job (`RunHandle::progress`); a durable
attempt's progress events and incumbents are also stored in the operational store and read
back by `Runtime::progress` or queried under `pse_ops`
([§20.6](identity-and-publication.md#section-20-6)). Field
detail is in the [generated runtime reference](../../generated/relations/runtime.md);
result meaning is owned by [§19](workflows-and-results.md#section-19).

**Limits.** Spans cover preparation, assembly and storage boundaries, not individual
residual or derivative evaluations. There is no metrics exporter or distributed trace
propagation beyond `tracing` task propagation inside the engine.

### 23.2 Failure taxonomy

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> `numerical` and `inconclusive` boundary classes, diagnostic severity and typed
> `ProblemError` variants (Plan 22 A1, implemented);
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — the typed
> realization refusal `modeling.realization` (Plan 22 M3 and M4, implemented);
> [ADR-0103](../../adr/0103-variable-domain-facet.md) — the recorded bound tightening
> `modeling.domain.tightened` and the conflicting-bound refusal (Plan 22 M2a, implemented);
> [ADR-0114](../../adr/0114-typed-operational-store.md) — operational-store failures are classified by SQLSTATE constants, never strings;
> CHECK and foreign-key violations are a typed invariant violation, not `Internal` (Plan 22
> B2, implemented).

**Ownership.** `pse-diagnostics` declares one vocabulary: `FailureClass` (coarse class)
and `DiagnosticCode` (detailed code, each mapped to one class). The registry projects
both into generated enums, Arrow and Python contracts; nothing else declares a code.
Codes are dotted in data (`authoring.parse.missing_id`) and rendered in Rust path form
by miette (`authoring::parse::missing_id`).

**Invariants.**

- Every public `*Error` enum in `crates/*/src` derives `thiserror::Error` and implements
  `miette::Diagnostic`, either derived or through the shared
  `pse_diagnostics::impl_diagnostic!` projection that assigns a typed `DiagnosticCode`.
  `tests/governance/tests/error_taxonomy.rs` checks the implementations; the compiler
  checks that each code is a declared variant. No `anyhow` in library crates.
- Wrappers retain original causes and related failures. Aggregates keep every leaf;
  the first error is never the only one reported. Formatting an error into a string
  does not establish its class or erase an uncertain commit.
- Findings are relations; a rendered diagnostic is a projection of a finding, never its
  storage. Boundary diagnostics carry source identities, stage and observations
  (`pse-model::diagnostic`); Python exceptions carry the same structured report.
- A non-converged or infeasible solve is a typed result (native termination, candidate
  kind, qualification), not an error ([§18](numerical-execution.md#section-18)).
  Errors cover refusals, evaluation failures, failed native operations, resource limits,
  cancellation, infrastructure and internal failures.

**Classes.** The IDAES column names the counterpart used for behavior comparison
([§6.14](schema-and-relations.md#section-6-14)).

| Class | Detailed code families and examples | IDAES counterpart |
|---|---|---|
| `authoring.parse` | `authoring.parse.{syntax, unknown_key, missing_id, nonfinite_number, budget, document_io, ambiguous_unary_power, unresolved_target}` | `ConfigurationError` |
| `authoring.reference` | `authoring.reference.{contract, derived_write, rename_named, unknown_row_key}`, `authoring.pkg.{unresolved, version_conflict}` | `ConfigurationError` |
| `validation.invariant` | declared invariant or boundary refusal; `schema.*` registry and contract admission codes | `ConfigurationError`, `PropertyPackageError` |
| `compile.property` | unsupported or ambiguous property, incomplete provider | `PropertyNotSupportedError`, `PropertyPackageError` |
| `compile.math` | `compile.math.{unit_inconsistent, quantity_operation_unsupported, domain_violation_static}` | Pyomo `UnitsError` |
| `kernel.unbound_parameter` | a selected provider lacks its executable or parameter binding | `PropertyPackageError` |
| `capability.backend` | unsupported capability, an unlinked or ineligible backend, or numerical preparation | `PropertyPackageError` |
| `solve.evaluation_error` | nonfinite or rejected trial evaluation | Pyomo evaluation errors |
| `solve.solver_error` | a native method or numerical kernel failed; a failed injected inner solve; `numerical` and `inconclusive` boundary diagnostics | — |
| `runtime.cancelled` | a cancellation token fired | — |
| `runtime.timeout` | a wall-clock deadline was exhausted | — |
| `runtime.resource_limit` | reservation, size, work or memory limit exceeded | — |
| `runtime.infrastructure` | storage I/O, integrity failure, publication conflict or incompatibility | — |
| `config.invalid` | invalid engine or platform configuration | — |
| `internal.invariant` | a postcondition failed; a platform bug | `BurntToast` |
| `user.model` | an authored query or assertion failed | `UserModelError` |

The vocabulary also declares `compile.feature`, `compile.law`, `compile.discretization`,
`plan.initialization`, `solve.infeasible`, `solve.locally_infeasible`, `solve.unbounded`
and `solve.limit`, and the detailed codes `compile.math.cyclic_expression`,
`template.guard_undecidable`, `rule.float_key`, `rule.head_schema_mismatch`,
`schema.rule_float_key` and `schema.rule_stratification`. No current Rust error
produces them; solver terminations are result tags instead. They remain declared
vocabulary, not evidence of a supported failure path.

**Boundary classes.** `pse-model::diagnostic::BoundaryClass` maps the shared boundary
vocabulary: invalid model → `validation.invariant`; unsupported → `capability.backend`;
resource limit → `runtime.resource_limit`; trial rejected or nonfinite →
`solve.evaluation_error`; infrastructure, conflict or incompatible →
`runtime.infrastructure`; cancelled → `runtime.cancelled`; internal →
`internal.invariant`; numerical or inconclusive → `solve.solver_error`. `numerical` is an
algorithmic failure without a model cause; `inconclusive` is an analysis that could not
reach its conclusion.

**Severity.** Every boundary diagnostic also carries a `DiagnosticSeverity`: `error`
(the default), `warning` or `info`. Severity is independent of class, and a warning never
makes a model invalid. `runtime.modeling_findings` publishes class and severity as
separate columns. Each numerical-diagnostics rule
(`pse-runtime/src/workflow/modeling/diagnostics.rs`) has an explicit disposition:

| Findings | Class | Severity |
|---|---|---|
| Structural under- or overdetermination; missing variable value | `invalid_model` | `error` |
| Nonfinite variable value | `nonfinite` | `error` |
| Unused variable; variable only in inequalities; potential domain evaluation error | `invalid_model` | `info` |
| Variable outside a bound; large equation residual; parallel Jacobian rows or columns; numerical rank deficiency | `numerical` | `warning` |
| Variable near a bound, fixed at zero, or of large or small magnitude; extreme Jacobian entries, rows or columns; mismatched or cancelling equation terms | `numerical` | `info` |
| Failed term evaluation | `trial_rejected` | `warning` |
| Jacobian analysis that could not complete | `inconclusive` | `warning` |

Scaling, conditioning and near-bound findings are therefore warnings or information,
never an invalid model. An undeclared rule is an `internal` error.

**Modeling domain refusals.** `pse_modeling::ModelingError::Domain` is the typed refusal
of a declared variable domain ([§6.8](schema-and-relations.md#section-6-8)). Its boundary
rule is `modeling.domain`, with the variable's instance path, its domain, the analysis and
the reason as observations and its declaration and variable identities as sources. A
quantity kind that is not a count or indicator, a binary bound outside `[0, 1]`
(`ConflictingBound`) and bounds that admit no value of the domain are `invalid_model`
(`validation.invariant`); missing finite bounds and a free discrete variable that the
analysis cannot decide are `unsupported` (`capability.backend`). An integer bound that
admission tightened inward is not a refusal: `DomainTightening` is the finding
`modeling.domain.tightened`, class `invalid_model` with severity `info`, observing the
variable, its domain and the specified and tightened bounds.

**Modeling realization refusals.** `pse_modeling::ModelingError::Realization` is the typed
refusal of a constraint-form or disjunction lowering that the bound case cannot admit
([§19.7](workflows-and-results.md#section-19-7)). It names the authored form or disjunction
(declaration and path), the subject row or variable, the declared realization and one of four
reasons (`RealizationRefusal`): `InfiniteBound`, a hull or linear lowering that needs a finite
case bound on the named variable; `IncompleteInterval`, a derived big-M row whose interval is
not FBBT-complete; `UnboundedInterval`, a derived big-M row whose interval is not finite over
the case box; and `Nonlinear`, a nonlinear disjunct row under `hull` without an ε. Its code is
`capability.backend`, and its boundary rule is `modeling.realization`, class `unsupported`,
with the form, subject, realization and reason as observations and the declaration as
source. A malformed declaration (a misplaced form, a wrong type or argument, competing
realizations) is an ordinary checking refusal (`validation.invariant`).

**Native failures.** `pse-backend-native::ProblemError` classifies a refused request or an
attributable native failure by cause; each variant keeps its cause and structural
identities, and none is flattened into a string of unknown class. Workflow diagnostics
(`pse-runtime/src/workflow/diagnostics.rs`) derive the boundary class and a stable rule
from the variant:

| Variant | Meaning | Code | Boundary class (rule) |
|---|---|---|---|
| `Unavailable` | The explicitly selected backend is not linked; other linked backends are listed; no fallback | `capability.backend` | `unsupported` (`native.unavailable`), backend observed |
| `Unsupported` | No eligible route, an ineligible explicit selection, or an adapter that cannot represent the request | `capability.backend` | `unsupported` (`native.unsupported`) |
| `Reuse` | `RequireReuse` found retained state it cannot reuse: another backend's (`Foreign`), a different structure (`Structure`) or options the step does not set (`DroppedOptions`) | `capability.backend` | `incompatible` (`native.reuse`), backend, reason and the held backend or dropped keys observed |
| `Contract` | The model or request violates a declared contract | `compile.math` | `invalid_model` (`native.contract`) |
| `Structural` | Structural deficiency with overdetermined rows and underdetermined columns | `compile.math` | `invalid_model` (`native.structural`), rows and columns as sources |
| `Math` | Mathematical evaluation failure with its domain or provider cause | the cause's code | the cause's class |
| `Provider` | A registered provider failed outside an attributed expression | the cause's code | the cause's class |
| `Numerical` | A native method or numerical kernel failed; the native status is kept when one exists | `solve.solver_error` | `numerical` (`native.numerical`), native backend, code and status observed |
| `Limit` | A declared finite allowance (`Time`, `Work` or `Memory`) was exhausted | `runtime.timeout` for `Time`, otherwise `runtime.resource_limit` | `resource_limit` (`native.limit`), limit kind observed |
| `Cancelled` | Cooperative cancellation fired | `runtime.cancelled` | `cancelled` (`native.cancelled`) |
| `Internal` | An adapter postcondition or platform invariant failed | `internal.invariant` | `internal` (`native.internal`) |

A failed native call is classified from its mapped stop category
(`ProblemError::native`): cancelled → `Cancelled`; time limit → `Limit` (`Time`);
iteration, solution, objective and general limits → `Limit` (`Work`); resource
exhaustion → `Limit` (`Memory`); invalid or panic → `Internal`; every other category →
`Numerical` with the native status. A failed injected inner solve
(`pse-math::MathError::Native`) carries `solve.solver_error` and keeps its typed native
cause, which the workflow classifies together with the block identity.

**Operational store failures.** `pse_operations::OperationsError` classifies a driver
failure once, by `SqlState` constant and connection state, never by a code's spelling
(`sqlstate_classified_by_constant`), and keeps the driver's error as an opaque cause, so no
driver type appears in the public API. Retry decisions read the variant
(`is_retryable`), never the message:

| SQLSTATE or condition | Variant | Code | Retryable |
|---|---|---|---|
| 40001, 40P01 | `Retryable` | `runtime.infrastructure` | yes |
| 55P03 | `LockUnavailable` | `runtime.infrastructure` | yes |
| 57014 | `Cancelled` | `runtime.cancelled` | no |
| 23505 | `Duplicate` | `runtime.infrastructure` | no |
| 23514, 23503 | `InvariantViolation`: the table and the violated named row check or reference | `validation.invariant` | no |
| class 08, 57P01–57P03, 53300, or a closed connection | `Unavailable`, naming the connection target without credentials | `runtime.infrastructure` | yes |
| any other SQLSTATE | `Internal` | `internal.invariant` | no |

The store's own refusals are typed as well: `SchemaMismatch` (another schema fingerprint,
remedied by `just db-reset`) and `Configuration` are `config.invalid`; an illegal lifecycle
transition, a missing row, an invalid request, a reused publication identity, an abandoned
intent and a retired input are `validation.invariant`; a lost lease, a publication
conflict, a retiring or protected publication, active readers and a lapsed reader lease
are `runtime.infrastructure`. At the workflow boundary a retryable store failure is class
`infrastructure` and any other store refusal `conflict` (rule `workflow.operations`); an
ephemeral run asked to publish (`workflow.ephemeral_publication`), an unknown job payload
version (`workflow.job_payload_version`), a former Delta control root
(`workflow.legacy_workspace`) and an expired export (`workflow.export_expired`) are
`incompatible`; an unconfirmed catalog commit is `infrastructure`
(`workflow.publication_unresolved`). A fixture whose declared solve intent differs from its
runtime fixture policy's is `conflict` (`workflow.fixture_intent_conflict`, code
`config.invalid`, [§6.10](schema-and-relations.md#section-6-10)).

**Native engine errors.** `pse-columnar::engine` classifies `DataFusionError` in one
place, by the plan's origin (`PlanOrigin`), never per call site:

| Native error | Class |
|---|---|
| `External` carrying a typed platform diagnostic | that diagnostic's declared code |
| `ResourcesExhausted` | `runtime.resource_limit` |
| `Configuration` | `config.invalid` |
| `Plan`, `SchemaError` from an authored analytics query | `user.model` |
| `Plan`, `SchemaError`, `NotImplemented` from numerical preparation | `capability.backend` |
| `Execution` inside a kernel adapter | `solve.evaluation_error` |
| other `Execution`, `ArrowError`, `IoError`, `ParquetError`, `ObjectStore` | `runtime.infrastructure` |
| `Context`, `Diagnostic`, `Shared`, `Collection` | traversed at every depth; one observation per leaf, retaining contexts and native diagnostics |
| anything else, including planning errors from platform-generated plans | `internal.invariant` |

`SessionConfig::set_str` and `Field::extension_type()` panic instead of returning errors
and are banned in `clippy.toml`; the typed configuration path and
`try_extension_type` are used instead.

## 24. Testing and qualification

Tests establish behavior; qualification is a separate, maintainer-requested activity
that runs the relevant checks once and records scope and conditions
([ADR-0092](../../adr/0092-ordinary-execution-evidence.md)). The failure baseline is
zero. A report names the command, mode and baseline. Architecture review and
design-change tracking are owned by
[§24.4](design-change-workflow.md#section-24-4).

### 24.1 Test layers

| Layer | Location | What it establishes |
|---|---|---|
| Crate tests | `#[cfg(test)]` modules and `crates/*/tests` | the owning contract: identity, admission, compilation, physical providers, native adapters, workflow, publication and settlement |
| Governance | `tests/governance` | workspace invariants: sole hasher, generated-tree equivalence, dependency pins and floors, error taxonomy, crate registration, FFI unwind containment, no shadow structs, registry governance, MSRV, unsafe allowlist |
| Engine | `tests/engine` | provider hierarchy, plan codec, pushdown truthfulness against unpruned sources, relational expansion, unified sources |
| Conformance | `tests/conformance` | `pse.canon.v2` properties (layout, null payload, signed zero, encoding round trips) and generated invariant fixtures |
| Lifecycle | `tests/lifecycle` | interrupted Delta publication leaves the old or the committed state; canonicalization, query and result memory budgets |
| Python | `python/pse/tests` | the native Python boundary: generated contracts, extension round trips, extra-key refusal, nullable-array refusal, workflows, publication streams |
| Parity | `python/pse/parity` | the IDAES 2.13.0 environment, authored compatibility enumerations and selected scientific reference comparisons (proposed [ADR-0097](../../adr/0097-modeling-scope-and-parity.md)) |

`tests/structural` currently contains only a placeholder; structural algorithms are
tested in their owning crates. Scientific checks compare against analytic, exhaustive or
independently generated references; they establish the stated cases, not untested
formulations. Parity does not establish numerical IDAES equivalence
([relationship to IDAES](../../relationship-to-idaes.md)).

**Invariants.**

- Arrow `force_validate` is a feature, not a profile: every Rust test recipe passes
  `--features pse-relations/force-validate` explicitly.
- Every Python test carries exactly one of `unit`, `component`, `integration` or
  `performance`; the repository `conftest.py` refuses collection otherwise.
  Performance tests are collected only with `--performance`. Parity tests run only with
  `--parity` and fail, never skip, when the environment is wrong. A test that leaves an
  untracked file or edits a tracked one fails the session.
- Generated trees equal a fresh regeneration
  ([ADR-0051](../../adr/0051-generated-trees-and-regeneration-check.md)).
- Test selection belongs to nextest and pytest; no exact-name acceptance manifest exists.
  Reports distinguish executed, unchanged-input reuse, reviewed transfer and not-run
  evidence.

**Commands.** During implementation: `just check-package <pkg>`, `just unit-package
<pkg> <filter>` and `just codegen` when declarations change. On request: `just test`
(default feature graph), `just native-test` (full linked native feature graph),
`just doctest`, `just py-test` or `just native-python <output>` after
`just py-sync-native`, `just governance-tests`, `just setup-test`, `just codegen-check`,
`just family-check`, `just quality`, `just parity`. `just assessment` runs every
current-environment check, continues after failures and writes logs and reports to a
new ignored directory under `build/assessment/`; `just assessment-list` prints its scope
and exclusions without executing anything. See the
[development guides](../../dev/README.md), including [manual CI](../../dev/ci.md) and
[build reuse](../../dev/build-performance.md).

### 24.2 Current qualification basis

The most recent completed qualification is Plan 22's Q1 of 2026-09-29, on the final state of
the solver, discrete-decision and operational-store work. The retired record states each
command and result:
[Plan 22 Outcome](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md#outcome-recorded-after-implementation).

**Scope and conditions.** Local Linux, the pinned dated nightly (ADR-0122) and Python 3.14.7
(parity on 3.13), against a zero failure baseline:
- the default workspace (`just test`) and the full native feature graph with explicit
  force-validation (`just native-test`: native units, conformance acceptance and the
  invariant harness);
- the store, worker and publication journeys against the local PostgreSQL 18 server;
- the linked Python suite (unit, component and integration) on a freshly built extension;
- doctests, governance, generated equivalence and family pins;
- the strict Clippy modes, including the native Clippy of the linked solver contracts;
- formatting, Python quality and documentation checks;
- the IDAES parity comparisons on this machine (`just parity`: environment, enumerations,
  sIPOPT sensitivities, parmest covariance, DegeneracyHunter and a PETSc PID loop).

Native runs used single-threaded BLAS/OpenMP unless a test admits threads.

**Measurements.** The large-KKT linear-solver and Clarabel thread measurements are single
runs on a synthetic grid (`measurement_tests`, run on demand). They are local observations
that separate orders of magnitude, not performance guarantees. The earlier 33 measured
process cases (2026-09-25/26) were not rerun, and the Plan 22 process cases were not added.

**Exclusions.**
- No IDAES numerical parity beyond the exercised comparisons.
- No exhaustive feature powerset, release or distribution, remote CI, container or
  other-platform result.
- No performance campaign.

Analytic, exhaustive and reference checks establish the stated cases and do not establish
untested formulations. The upstream `proc-macro-error2` future-incompatibility notice
remains; this is not a warning-free toolchain claim.

Raw reports are local ignored outputs under `build/assessment/` and are not part of the
repository. The previous basis, the
[final qualification](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-p14-p18-execution.md#verification)
of 2026-09-25/26, is historical.

A retired qualification is not transferred to changed paths. After a change, targeted
tests show the new behavior; comprehensive qualification runs again only when the
maintainer requests it.

### 24.3 Benchmarks

Criterion benchmarks live in `benches/` (`pse-benches`): `canonicalization`,
`native_cache`, `native_consolidation` and `native_process` (complete-process cases).

| Command | Purpose |
|---|---|
| `just bench-smoke` | runs every benchmark once as a test; no timing |
| `just case-measure <output> --functional-from <qualification>` | fresh-process case measurements; requires completed functional qualification of the same source |
| `just bench-production` | production-equivalent native measurements without force-validation, in an isolated target directory |

Measurement rules:

- There is no timing gate. A performance claim is labelled Measured and names the
  benchmark, profile, features, thread settings and machine conditions.
- Compilation is untimed setup; cold and warm costs refer to case preparation, rebuild
  and complete process operations, preserving Cargo and compiler caches.
- Compare like-for-like inputs. Report raw samples and uncertainty (mean and standard
  error, not a confidence interval unless computed), work counts and teardown.
- Pool reservations and process RSS are separate observations; neither attributes
  individual library allocations, and zero reservations after teardown does not mean
  the allocator returned pages to the operating system.
- A changed physical, numerical or lifecycle contract prevents a causal before/after
  speedup claim from older measurements.
