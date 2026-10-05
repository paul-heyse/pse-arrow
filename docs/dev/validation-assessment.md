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

## What each command establishes

During implementation the inner loop is `just check-package`/`just check`, targeted
`just unit-package` and `just codegen` (see AGENTS.md, *Execution rhythm*); the static subset
(`just hygiene`) runs once all functional scope is implemented. The table describes checks available for
manual qualification.

| Command | Run | Proves | Does not prove |
|---|---|---|---|
| `just ci-fast` | on demand | the workspace formats, compiles, lints clean and its tests and doctests pass | nothing about Python, features, policy or docs |
| `just test` | on demand | Rust tests pass with Arrow `force_validate` on | nothing about doctests, other profiles, or release-only paths |
| `just codegen-check` | on demand | every generated tree equals a fresh regeneration, with no extra or untracked generated files (ADR-0051); the workspace-hack equals `cargo hakari generate` and every managed member depends on it (ADR-0122) | nothing about runtime behavior of the generated interfaces, or whether an opt-in feature reached the workspace-hack (governance `every_crate_registered` checks `force_validate`) |
| `just family-check` | when a pinned-family dependency moves | one resolved version per dependency family, equal to the pins | nothing about whether that version behaves as documented |
| `just governance` | on demand | the workspace-level invariants hold (pins, crates registered, the dated nightly at or above the `rust-version` floor, unsafe allowlist, error taxonomy) | nothing about runtime behaviour, or whether the source still compiles on the stable floor |
| `just quality` | on demand | Python format/lint/types/import boundaries and repo config are clean | that the code works |
| `just deps-report` | on demand | what is in the dependency graph and under what licences; **advisory, always exits 0** | nothing — it refuses nothing and blocks nothing |
| `just policy` | on demand | the same checks, strictly: no known advisory, no disallowed licence. Opt-in, not in `ci-pr` | nothing about code you wrote, and nothing you are obliged to act on yet (register R-31) |
| `just parity` | on demand, when parity is in scope | the exercised parity checks pass against `idaes-pse==2.13.0` | nothing about cases not exercised, or other IDAES versions |
| `just docs` | on demand | documentation HTML and scoped search build; manual CI can also check internal links | nothing about whether the prose is true |
| `just adr-lint` | on demand | ADR front matter, numbering, supersession and register rows are well-formed | nothing about whether the decisions are good |

**Never report that tests pass without naming the command, the mode, and the baseline.**
"34 failed" is not information until the baseline is known — and here the baseline is zero.

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
| `just assessment-list [--group <name>]` | Prints the declared gates as JSON (name, role, recipe, arguments, dependencies, mode, profile, input scope) and the explicit exclusions. Executes nothing. |
| `just assessment [<output>] [flags]` | Runs every gate in `scripts/validation_scope.py::comprehensive` and continues after failures. The default output is `build/assessment/<UTC timestamp>/`; pass `""` for it when adding flags. |

The comprehensive scope runs these gates in order:

1. `py-sync-native`, then the formatting, TOML and both Clippy gates.
2. Quality: Python contracts, format, lint, types and imports; repository lint; agent
   configuration; setup controls through `setup-test-report`; solver pins.
3. Generation freshness and family checks, plus ADR, index and register lint.
   Governance tests are covered by the linked workspace invocation.
4. `check`, `docs-rust`, `docs` and `python-stubs --check`.
5. Focused `feature-absence --profile ci` in the default graph, `doctest`, and
   `native-test --profile ci` as the single common Rust execution graph.
6. `inspection-fixture` and `native-python`.

A gate whose dependency did not qualify is recorded as `blocked`; the run continues.

Each run directory holds:

- source provenance over one declared product-input path set (file hashes, diff, status,
  revision and an archive of untracked files);
- `host.json`, copies of the manifests, lockfiles and nextest configuration, and
  `cargo-metadata.json`;
- `scope.json`, one `<gate>.log` per gate and a JUnit copy for each gate that produces one;
- `checks.json` (version 5), `failures.json`, `test-findings.json` and `summary.md`.

The assessment lists the focused default-feature selection once. The linked Rust wrapper
owns its one explicit nextest listing and persists the raw JSON beside binary/link
provenance; assessment consumes that artifact. Pytest records selected node IDs during
its execution. Assessment owns JUnit parsing and selected/terminal reconciliation; a
standalone native Python command calls the same composition operation. A selected identity
without a terminal result becomes `not_run`; skipped, failed, duplicate, unexpected and
truncated results fail the gate. Nonzero process exit cannot be replaced by a passing report.
Native identities retain executed binary and `ldd` library hashes, `rustc -Vv`, actual
relevant environment, feature/Cargo profile and thread settings. Default refusal controls
retain their own default graph identity, including the real serial QDLDL control.
Native Python resolves the imported package and extension in pytest's interpreter/import
environment, refuses origins outside this checkout, and checks the same loaded extension
inside pytest before collection. On-disk candidates are not execution identity.

One before/after contextual inventory captures source hashes, additions, deletions, modes,
symlink targets and Git provenance. Each gate projects that map into a versioned declared
input family: Rust product, Python product, tooling, documentation or generation. Unknown
scopes retain the whole map. Product claims exclude unconsumed plans/reviews/prose;
executable policy and data remain relevant to operations that consume them. Environment
identity excludes unrelated controls; credentials and local native override files enter as
hashes. Native provider identity comes from actual binaries and linked libraries.

The command exits 0 only when `required_checks_covered` holds: every required gate was
attempted and qualified, relevant inputs/environment remained stable, and provenance was
captured without error. Unrelated concurrent prose changes remain contextual observations
and do not fail a product claim. A gate qualifies when it passed, when it is advisory with
findings, or when it is deferred and unsupported. Ctrl-C retains terminal evidence and records
remaining gates as `not_run`, leaving the assessment incomplete. Publication setup failure
blocks only fixture consumers; independent Python cases continue. Tests own temporary
outputs, runtime facades, attempt/cancellation state, physical packages and isolated stores.
Immutable engine settings and the deployment resource service remain process-owned; tests
share one configured budget and session spill directory. Checkout changes are not attributed
to individual tests by Git polling.

**Continuation.** Every run writes a new directory. `--reuse-from <prior-output>` names
a current version-5 report, and each retained gate takes one of two repeatable flags:

- `--reuse <gate>` keeps a qualified observation only if its canonical invocation, scope definition, relevant inputs/environment,
  retained artifact hashes and applicable native bytes are unchanged.
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
| `just native-test [nextest args]` | The full workspace with `pse-runtime/native-solvers`, `pse-tests-conformance/native-acceptance` and force-validation, under the native environment. Nextest owns selection. The wrapper writes a raw selection artifact and native identity before execution. |
| `just doctest` | Workspace doctests with force-validation. `pse-py` is excluded because Cargo cannot run cdylib doctests. `doctest-release` is the release-profile variant. |
| `just py-sync-native` | Rebuilds the editable extension (`dev` profile, `force-validate,native-solvers`) and regenerates the compiled API stubs. `just py-sync` installs the default profile, which lacks native solvers. |
| `just inspection-fixture <new-dir>` | Publishes and reopens a fresh native store for component tests. |
| `just native-python <output> [pytest args]` | Linked Python `unit or component or integration` tests, reported to `<output>/native-python.xml`. Explicit fixture consumers read `PSE_INSPECTION_PUBLICATION` (default `<output>/inspection`); create it first with `inspection-fixture`. Run `py-sync-native` after Rust edits. |
| `just governance-tests [args]` | `pse-tests-governance` with force-validation. |
| `just setup-test` | Stdlib `unittest` discovery over `scripts/tests`: setup, guards, runner, docs and build tooling. |
| `just setup-test-report <output>` | The same discovery with an XML reporter, writing `setup-test.xml` and `setup-test-selected.json`. Fails on skips or an empty run. |
| `just unit-consolidation-tools` | Focused runner, ownership, selected prerequisite, measurement parser and isolated build snapshot controls. |

These tooling tests establish runner behaviour, not product or scientific acceptance.

## Case measurements

`just case-measure <new-output> --functional-from <assessment-dir-or-checks.json>`
measures selected complete-process, preparation or admission cases. Repeat `--case <id>`
to select declared cases; no selector runs the complete campaign. Workload declarations in
`.config/process-cases.json` and `.config/preparation-cases.json` reference named functional
scopes, and document admission explicitly requires the admission scope:

| Scope | Existing behavioral owners |
|---|---|
| `process` | Native runtime and conformance acceptance boundaries |
| `preparation` | Runtime/compiler preparation and provider composition |
| `admission` | Authored loading/admission and its runtime boundary |
| `lifecycle` | Runtime/store/lifecycle enforcement and publication resource controls |
| `native` | The explicit full linked workspace covering invocation |

Obtain selected prerequisite evidence with, for example,
`just assessment "" --functional-scope preparation`. For several workload groups, use
`just assessment "" --functional-scope native` to cover them in one Rust invocation.
Conservative named scopes can overlap; the explicit full native scope avoids repeating them.
Selections specify recipes, graph/profile and framework filters, never individual-test
manifests. Measurement accepts the exact declared invocation or the explicitly declared
complete full-workspace native invocation. It infers no arbitrary filter-subset equivalence.
Only consumed claims must qualify; unrelated static/documentation checks are not prerequisites.
Missing, failed, skipped, unexecuted, wrong-mode/profile or changed-input/provider observations
are refused. Reuse/transfer retains its origin and applicability labels.

The current version-5 report must carry complete terminal evidence for each consumed claim,
matching relevant product/model/policy/lock inputs and actual native environment/provider
identity. The benchmark retains its own executable identity: test and benchmark binaries need
compatible source, features, Cargo profile and providers, not identical executable bytes.
Benchmark source, workload declarations and parameters also enter measurement identity;
unrelated prose does not. `just bench-case-smoke <new-output> [--case <id>]` runs untimed
controls and cannot supply functional or performance qualification.

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
digest over relevant measurement inputs (a relevant change during measurement is refused), the binary and linked-library hashes,
the toolchain and the thread settings. Per-case artifacts are under `process-cost/<id>/`.
These are local observations of the design-stage profile, not release-profile
performance. For production-equivalent measurements, use `just bench-production`
([§24.3](../authoritative_design/sections/operations-and-validation.md#section-24-3)).

`just py-unit` and exact `just py-test -m unit` run without inspection publication.
`py-test --collect-only` also avoids publication. Mixed component selections retain explicit
setup before pytest workers; the all-extension IPC round trip is requested by one dedicated
boundary test. Runtime facades, physical packages and operational databases are owned per
test; immutable engine settings are shared for the session and configure the process-owned
deployment resource service. Exported buffers continue to charge that same service after a
facade closes.

## Shared process fixtures and independent references

`tests/fixtures/plan14` is a live test input; the directory name is historical. It
holds the model, provider, binding, balance, dynamic, fit and dataset declarations, a
physical package, and frozen references. Its consumers are the conformance
`native-acceptance` process tests (through `tests/support/plan14.rs`),
`benches/native_process`, the FeOS kernel and guarded-algebra unit tests, the runtime
vessel recipe and `python/pse/tests/test_plan14_acceptance.py`.

| Command | Generates | Consumes |
|---|---|---|
| `just plan14-reference` | `packages/reference/data/oracles/teqp-0.23.1/data/*.parquet`, `real-algebra-reference.json` and `pr-stability-reference.json` | Published parameter Parquet banks in `packages/reference/data/{gross-sadowski-2001,poling2000}`. Runs in an isolated locked CPython 3.12 environment (`build/plan14-reference`, dependency group `thermo-reference` with teqp) with no product imports. |

FeOS values are compared against offline teqp PC-SAFT states and Decimal-precision
DIPPR100 caloric integrals from 298.15 K, within the tolerances recorded in the
reference file. The ternary flash is compared against an independently solved set of
pressure, chemical-potential and material-balance equations. That reference is not a
global stability certificate, so stability has its own FeOS test. Analytic, exhaustive
and reference checks establish the stated cases only.

## Outside this scope

`just assessment-list` prints these exclusions: other platforms, wheels and remote CI;
release, coverage, feature powerset and alternate toolchains; IDAES parity (`just
parity`); performance (`case-measure`); the time-dependent `register-check`;
and architecture or scientific review, which is a judgement recorded by its owner, not
command-exit evidence.
