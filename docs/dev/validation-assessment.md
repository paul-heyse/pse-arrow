---
title: Qualification commands
status: current
---

# Qualification commands

This guide describes the ordinary command surface for checking the current implementation.
[Blueprint §24](../authoritative_design/sections/operations-and-validation.md#section-24)
owns the test layers and measurement rules, and
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2)
records the current qualification basis and its exclusions.
[ADR-0092](../adr/0092-ordinary-execution-evidence.md) gives the rationale.
`just --list` is the authority for recipe names.

Comprehensive qualification is requested by the maintainer (`AGENTS.md`, *Execution
rhythm*). It is not a prerequisite for a commit, push, merge or plan close. During
implementation, use `just check-package`, targeted `just unit-package` and `just codegen`.
The failure baseline is zero. A report names the command, mode, scope and result.

## Common conditions

- Every Rust test recipe passes `--features pse-relations/force-validate` explicitly.
- Native recipes run through `scripts/native_exec.sh`, which sources
  `scripts/native-execution-env.sh`. That script loads the build, solver and math
  environments and optional local `.envrc.local` overrides. It prepends `$IPOPT_DIR/lib`
  to `LD_LIBRARY_PATH` and sets `OMP_NUM_THREADS`, `OPENBLAS_NUM_THREADS` and
  `MKL_NUM_THREADS` to `1`. A missing native library or capability fails; it is never
  skipped.
- Results are local outputs under the ignored `build/` directory; they are not
  committed. The runner refuses an output path that Git does not ignore or that
  already exists.

## Full local assessment

| Command | What it does |
|---|---|
| `just assessment-list [--group <name>]` | Prints the declared gates as JSON (name, role, recipe, arguments, dependencies, mode, profile) and the explicit exclusions. Executes nothing. |
| `just assessment [<output>] [flags]` | Runs every gate in `scripts/validation_scope.py::comprehensive` and continues after failures. The default output is `build/assessment/<UTC timestamp>/`; pass `""` for it when adding flags. |

The comprehensive scope runs these gates in order:

1. `py-sync-native`, then the formatting, TOML and both Clippy gates.
2. Quality: Python contracts, format, lint, types and imports; repository lint; agent
   configuration; setup controls through `setup-test-report`; solver pins.
3. Governance (`governance-tests`, the four `codegen-*-check` gates, `family-check`) and
   ADR, index and register lint.
4. `check`, `docs-rust`, `docs`, `conformance-fixtures-check` and `python-stubs --check`.
5. `test --profile ci`, `doctest` and `native-test --profile ci`.
6. `inspection-fixture` and `native-python`.

A gate whose dependency did not qualify is recorded as `blocked`; the run continues.

Each run directory holds:

- source provenance over one declared product-input path set (file hashes, diff, status,
  revision and an archive of untracked files);
- `host.json`, copies of the manifests, lockfiles and nextest configuration, and
  `cargo-metadata.json`;
- `scope.json`, one `<gate>.log` per gate and a JUnit copy for each gate that produces one;
- `checks.json` (version 4), `failures.json`, `test-findings.json` and `summary.md`.

Before `test` and `native-test` run, nextest lists the selected test identities; pytest
records its collected node IDs. A selected identity without a terminal result becomes
`not_run`, and skipped, failed, duplicate or unexpected results fail the gate.
`native-test` and `native-python` also record a `native-profile-v1` identity: hashes of
the executed binaries and their `ldd`-resolved libraries, `rustc -Vv`, and the thread
settings.

The command exits 0 only when `required_checks_covered` holds: every gate was attempted
and qualified, sources did not change during the run, and provenance was captured without
error. A gate qualifies when it passed, when it is advisory with findings, or when it is
deferred and unsupported. Ctrl-C stops the current gate and records the remaining gates
as `not_run`, which leaves the run incomplete.

**Continuation.** Every run writes a new directory. `--reuse-from <prior-output>` names
a version-4 report, and each retained gate takes one of two repeatable flags:

- `--reuse <gate>` keeps a qualified observation only if its declared scope, sources,
  environment, retained artifact hashes and native bytes are all unchanged.
- `--transfer <gate>` keeps a qualified observation despite changed inputs. It requires
  `--change-reason`, and the record lists the changed inputs.

Retained records keep their origin and are labelled `unchanged-input-reuse` or
`reviewed-transfer`, never fresh execution.

**Aggregates and advisory checks.** `quality`, `governance`, `ci-fast`, `ci-pr`, `clippy`,
`fmt-check`, `codegen-check`, `adr-lint`, `deps-report` and `policy` call the same runner
with `--group`. Each writes its own `build/assessment/<timestamp>/` directory without
source provenance. The `audit-*` gates are advisory: findings do not fail `deps-report`,
but a tool failure does. `policy` runs the same audits as required gates. The
`--advisory` flag passed by `deps-report` has no further effect.

## Individual commands

| Command | Scope |
|---|---|
| `just test [nextest args]` | The default workspace feature graph, run with `cargo nextest run --no-fail-fast` and force-validation. |
| `just native-test [nextest args]` | The full workspace with `pse-runtime/native-solvers`, `pse-tests-conformance/native-acceptance` and force-validation, under the native environment. Nextest owns selection. `PSE_NEXTEST_ACTION="list --message-format json"` lists the selection without executing it. |
| `just doctest` | Workspace doctests with force-validation. `pse-py` is excluded because Cargo cannot run cdylib doctests. `doctest-release` is the release-profile variant. |
| `just py-sync-native` | Rebuilds the editable extension (`dev` profile, `force-validate,native-solvers`) and regenerates the compiled API stubs. `just py-sync` installs the default profile, which lacks native solvers. |
| `just inspection-fixture <new-dir>` | Publishes and reopens a fresh native store for component tests. |
| `just native-python <output> [pytest args]` | Linked Python `unit or component or integration` tests, reported to `<output>/native-python.xml`. Component tests read `PSE_INSPECTION_PUBLICATION` (default `<output>/inspection`); create it first with `inspection-fixture`. Run `py-sync-native` after Rust edits. |
| `just governance-tests [args]` | `pse-tests-governance` with force-validation. |
| `just setup-test` | Stdlib `unittest` discovery over `scripts/tests`: setup, guards, runner, docs and build tooling. |
| `just setup-test-report <output>` | The same discovery with an XML reporter, writing `setup-test.xml` and `setup-test-selected.json`. Fails on skips or an empty run. |
| `just unit-consolidation-tools` | `scripts.tests.test_validation` only: runner selection, reports, reuse and transfer, and measurement CSV parsing. |

These tooling tests establish runner behaviour, not product or scientific acceptance.

## Case measurements

`just case-measure <new-output> --functional-from <assessment-dir-or-checks.json>`
measures the complete-process cases after functional qualification. It refuses a
functional report unless the report is version 4, `required_checks_covered` holds, its
scope equals the current comprehensive scope, its source files match the working tree,
and its native identities still verify.

- **Untimed build.** The benchmark is built with `cargo bench -p pse-benches --bench
  native_process --no-run`, using the profile named in `.config/process-cases.json`
  (currently `dev`) and `native-process,pse-relations/force-validate`. The compile is
  untimed setup.
- **Fresh process per case.** Each workload declared in `.config/process-cases.json`
  runs in its own process: 10 flat Criterion samples, 250 ms warmup and a 1 s target
  measurement time, which Criterion extends for slow operations.
- **What a sample times.** Source admission, preparation or rebuild, joined native
  execution, validation and results access, optional publication, and teardown. Cold
  cases construct their runtime inside each sample; warm cases retain revision and
  runtime.
- **Statistics.** The collector reads Criterion's `raw.csv` and reports mean, sample
  standard error (not a confidence interval), minimum and maximum.
- **Pool versus RSS.** `pool_peak_bytes` is the runtime pool's per-operation observation
  peak. `process_peak_rss_bytes` is the dedicated process's lifetime VmHWM. The scopes
  differ, and neither attributes individual library allocations. Retained and
  after-teardown pool reservations are reported separately.

`case-measure.json` (`process-cost-v3`) binds the functional report digest, the source
digest (a change during measurement is refused), the binary and linked-library hashes,
the toolchain and the thread settings. Per-case artifacts are under `process-cost/<id>/`.
These are local observations of the design-stage profile, not release-profile
performance. For production-equivalent measurements, use `just bench-production`
([§24.3](../authoritative_design/sections/operations-and-validation.md#section-24-3)).

## Shared process fixtures and independent references

`tests/fixtures/plan14` is a live test input; the directory name is historical. It
holds the model, provider, binding, balance, dynamic, fit and dataset declarations, a
physical package, and frozen references. Its consumers are the conformance
`native-acceptance` process tests (through `tests/support/plan14.rs`),
`benches/native_process`, the FeOS kernel and guarded-algebra unit tests, the runtime
vessel recipe and `python/pse/tests/test_plan14_acceptance.py`.

| Command | Generates | Consumes |
|---|---|---|
| `just plan14-reference` | `tests/fixtures/plan14/thermo-reference.json`, `real-algebra-reference.json` and `tests/fixtures/thermo-entropy-reference.json` | `crates/pse-kernels/data` parameter files. Runs in an isolated locked CPython 3.12 environment (`build/plan14-reference`, dependency group `thermo-reference` with teqp) with no product imports. |
| `just plan14-fixtures` | The remaining `tests/fixtures/plan14` declarations and `package/` | `tests/fixtures/packages/physical-primitives`, kernel data and the frozen `thermo-reference.json`. Run it after `plan14-reference`. |
| `just unit-plan14-sources` | Nothing | Decodes the shared declarations through generated Python contracts, without constructing a runtime or solving. |

FeOS values are compared against offline teqp PC-SAFT states and Decimal-precision
DIPPR100 caloric integrals from 298.15 K, within the tolerances recorded in the
reference file. The ternary flash is compared against an independently solved set of
pressure, chemical-potential and material-balance equations. That reference is not a
global stability certificate, so stability has its own FeOS test. Analytic, exhaustive
and reference checks establish the stated cases only.

## Outside this scope

`just assessment-list` prints these exclusions: other platforms, wheels and remote CI;
release, coverage, feature powerset and alternate toolchains; IDAES parity (`just
parity-container`); performance (`case-measure`); the time-dependent `register-check`;
and architecture or scientific review, which is a judgement recorded by its owner, not
command-exit evidence.
