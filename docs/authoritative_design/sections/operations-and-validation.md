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
([ADR-0090](../../adr/0090-shared-execution-vocabulary.md)).

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
solver progress is exposed as events on the joined job (`RunHandle::progress`). Field
detail is in the [generated runtime reference](../../generated/relations/runtime.md);
result meaning is owned by [§19](workflows-and-results.md#section-19).

**Limits.** Spans cover preparation, assembly and storage boundaries, not individual
residual or derivative evaluations. There is no metrics exporter or distributed trace
propagation beyond `tracing` task propagation inside the engine.

### 23.2 Failure taxonomy

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
  Errors cover refusals, evaluation failures, resource, cancellation and infrastructure.

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
| `capability.backend` | unsupported capability or numerical preparation | `PropertyPackageError` |
| `solve.evaluation_error` | nonfinite or rejected trial evaluation | Pyomo evaluation errors |
| `runtime.cancelled` | a cancellation token fired | — |
| `runtime.resource_limit` | reservation or size limit exceeded | — |
| `runtime.infrastructure` | storage I/O, integrity failure, publication conflict or incompatibility | — |
| `config.invalid` | invalid engine or platform configuration | — |
| `internal.invariant` | a postcondition failed; a platform bug | `BurntToast` |
| `user.model` | an authored query or assertion failed | `UserModelError` |

The vocabulary also declares `compile.feature`, `compile.law`, `compile.discretization`,
`plan.initialization`, `solve.infeasible`, `solve.locally_infeasible`,
`solve.unbounded`, `solve.limit`, `solve.solver_error` and `runtime.timeout`, and the
detailed codes `compile.math.cyclic_expression`, `template.guard_undecidable`,
`rule.float_key`, `rule.head_schema_mismatch`, `schema.rule_float_key` and
`schema.rule_stratification`. No current Rust error produces them; solver terminations
are result tags instead. They remain declared vocabulary, not evidence of a supported
failure path.

**Boundary classes.** `pse-model::diagnostic::BoundaryClass` maps the shared boundary
vocabulary: invalid model → `validation.invariant`; unsupported → `capability.backend`;
resource limit → `runtime.resource_limit`; trial rejected or nonfinite →
`solve.evaluation_error`; infrastructure, conflict or incompatible →
`runtime.infrastructure`; cancelled → `runtime.cancelled`; internal →
`internal.invariant`.

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
| Parity | `python/pse/parity` | the IDAES 2.12.0 environment and the exercised compatibility names ([ADR-0003](../../adr/0003-clean-room-relationship-and-parity-pin.md)) |

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

The current implementation's most recent completed qualification is the local Linux
qualification and case measurements of 2026-09-25/26, followed by the maintainer's
independent review, which confirmed a satisfactory outcome. The retired execution
record is available at an immutable commit:
[final qualification](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-p14-p18-execution.md#verification).

**Scope and conditions.** Local Linux, Rust 1.98.1 and Python 3.14.7, against a zero
failure baseline: the default workspace and the full native feature graph with explicit
force-validation; the Python native suite (unit, component and integration) on a
freshly built extension; doctests; governance; setup controls; the strict configured
Clippy modes; formatting, generated equivalence, family pins and documentation checks.
Native runs used single-threaded BLAS/OpenMP. The 33 measured cases ran as separate
processes with Criterion CSV samples, compilation untimed, in a shared checkout
concurrently used for development: they are local observations, not isolated or
production-profile performance guarantees.

**Exclusions.** No IDAES numerical parity, exhaustive feature powerset,
release/distribution, remote CI or other-platform result is claimed. Analytic,
exhaustive and reference checks establish the stated cases and do not establish
untested formulations. The upstream `proc-macro-error2` future-incompatibility notice
remains; this is not a warning-free toolchain claim.

Raw reports are local ignored outputs under `build/assessment/` and are not part of the
repository. The earlier
[local Linux closure](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/14-m22-execution.md)
of the library-owned foundation is historical and superseded as the current basis.

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
