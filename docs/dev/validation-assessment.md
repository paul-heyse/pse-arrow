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

For current development, the maintainer selected a ±10% relative allowance on
2026-10-08 when assessing final results against historical IDAES/reference values.
The authored `numerical_policy.historical_reference_relative` constant owns this
allowance. Historical values are comparison evidence, not proof that the earlier
solver was more accurate. Zero-reference comparisons retain their explicit absolute
allowance. This criterion does not change ordinary engineering solver stopping
budgets, physical feasibility checks or conservation checks. Accuracy qualification
beyond this development criterion remains separate work.

During design, dependency, toolchain, environment and artifact fingerprints are provenance,
not automatic grounds to invalidate an earlier result or require a new artifact qualification.
Use the development profile by default and run targeted tests for changed behavior. Cargo may
need to compile edited code to execute it; that is separate from a qualification-driven rebuild.
Retain historical evidence with its original conditions, without claiming it exercised new code.
Strict producer/artifact qualification remains available on explicit request. Do not freeze or
copy the Cargo environment to keep historical evidence valid, or infer scientific invalidity
from a refused exact cache match. No automatic change-impact classification is required.

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
| `just deps-report` | on demand | what is in the dependency graph and under what licences; **advisory: findings never fail it**, a tool that cannot run does | nothing — it refuses nothing and blocks nothing |
| `just policy` | on demand | the same checks, strictly: no known advisory, no disallowed licence. Opt-in, not in `ci-pr` | nothing about code you wrote, and nothing you are obliged to act on yet (register R-31) |
| `just parity` | on demand, when parity is in scope | the exercised parity checks pass against `idaes-pse==2.13.0` | nothing about cases not exercised, or other IDAES versions |
| `just docs` | on demand | documentation HTML and scoped search build; manual CI can also check internal links | nothing about whether the prose is true |
| `just adr-lint` | on demand | ADR front matter, numbering, supersession and register rows are well-formed | nothing about whether the decisions are good |

**Never report that tests pass without naming the command, the mode, and the baseline.**
"34 failed" is not information until the baseline is known — and here the baseline is zero.

## Common conditions

- Rust correctness routes activate and verify force-validation for each actual selected
  Arrow-consuming normal/build/test-dev closure, including host dependencies (ADR-0170).
  Pure closures select no unrelated relation targets. Bare commands have the same obligation.
- Native recipes run through `scripts/pse-env --native` and the common operation owner,
  beginning the supervised lifetime before setup. Each recipe requests its actual native
  capability closure; nested consumers reuse verified immutable generations while that
  operation is alive. Local `.envrc.local` overrides remain supported. The selected solver
  prefix supplies its loader path, and native thread settings remain `1`. A missing required
  library or capability fails. Manual sourced environment helpers perform full verification
  on every unscoped use and retain conservative generation guards; they do not grant scoped
  reuse, cancellation or reclamation.
- Results are local outputs under the ignored `build/` directory; they are not
  committed. The runner refuses an output path that Git does not ignore or that
  already exists.
- Local scientific validation uses Nextest's `local` profile. It reports slow tests
  and retains a finite four-hour whole-run bound, while the production task owns its
  numerical deadline. It does not terminate a declared 600-second solve after 120
  or 360 seconds. Resource groups, memory caps, force-validation and zero retries
  remain in effect. The separate `ci` profile is available for an explicit CI run.

## Full local assessment

| Command | What it does |
|---|---|
| `just assessment-list [--group <name>]` | Prints the declared gates as JSON (name, role, recipe, arguments, dependencies, mode, profile, input scope) and the explicit exclusions. Executes nothing. |
| `just assessment [flags]` | Runs the selected gates (all comprehensive gates by default) and continues after failures. Use `--group ready` for that group and `--output build/assessment/example` for a fresh output directory. The default is a new `build/assessment/<UTC time>-<selection>-<pid>-<random>/`. The validation owner checks only the selected prerequisites and exits 125 before native setup when one is missing; nonnative groups need no native preparation. |
| `just result RUN_PATH [--failures] [--gate NAME] [--tail N] [--json]` | Reads one version 5 checkpoint and identifies the concrete run, incompleteness, required coverage, baseline and reused origins. Reads at most 64 KiB from each selected log, with 20 lines by default for failures or an explicit gate; `--tail 0` disables tails. Executes no checks or cleanup. Temporarily borrows the selected report and referenced origins before reading. |

For measurement prerequisites, `--functional-scope native` covers the ordinary
scopes. Combine it with `--functional-scope managed-primary` when the selected
cases require managed execution; this runs the existing authenticated managed-native
route as a distinct gate. Use the same explicitly selected worker, native environment,
store and `PSE_MEMORY_MAX` for qualification and measurement. Receipts bind both
the invocation context and the actual observer's transformed environment and finite
placement. They derive force-validation and native features from the current Cargo
selection. An assembled report whose setup injected a different producer context
requires a matching standalone assessment before measurement; transfer does not
repair a context mismatch.

The reference host route is `--resource-class reference` with its 140 GiB primary
allocation (`PSE_MEMORY_MAX=140G` if explicitly set). Its server and observer bring
the declared envelope to 160 GiB. Setting an exclusive caller itself to 160 GiB
requests a different, larger envelope and is not the reference route.

Assessment resources seal producer-declared artifact roles after final output: receipts,
provenance, consumed evidence, scratch or unknown. Only explicit scratch is disposable;
summary/receipt names, required source/environment snapshots and referenced logs/XML remain
retained. Extension or filename does not confer release. The sealed manifest binds finalized
identities and digests; re-sealing cannot downgrade evidence. Incomplete, failed, drifted or
manifest-less historical attempts remain protected, including authored diff/tar snapshots.
New lifecycle activation waits for affected old producers and reclaimers to drain.

Display resolves a latest pointer once and borrows that exact report plus referenced origins.
It creates no persistent consumer reference or checkpoint. New retained-reference reuse requires a sealed origin
and unchanged receipt identity; unregistered or manifest-less historical reports remain
displayable but cannot supply new reusable claims before actual owner qualification. Existing owner reservations, no-follow traversal, active-use
exclusion, actual drain and final identity rechecks still govern exact cleanup. Document
retirement metadata does not release a resource. Explicit owner adoption of a historical
attempt requires its actual outcome, provenance, references and durable authored inputs;
unknowns stay protected.

The comprehensive scope runs these gates in order:

1. `py-sync-native` with the selected `--python-profile` (`dev` by default), then
   the formatting, TOML and both Clippy gates.
2. Quality: Python contracts, format, lint, types and imports; repository lint; agent
   configuration; setup controls through `setup-test-report`; solver pins.
3. Generation freshness and family checks, plus ADR, index and register lint.
   Governance tests are covered by the linked workspace invocation.
4. `check`, `docs-rust`, `docs` and `python-stubs --check`.
5. Focused `feature-absence --profile local` in the default graph, `doctest`, and
   the actual `producer-fixture` capture. With `--python-profile producer`, capture
   the actual runtime, worker and Python deployment targets through
   `producer-deployment`, then run the installed Python solve/reopen association
   control. `native-test --profile local` retains the single common Rust execution
   graph and waits for that association in the producer campaign.
6. The ordinary `native-python` partition against the installed extension, plus separate
   `managed-native` and `managed-python` partitions through the exact reference observer.
   Ordinary Python uses four grouped workers; managed Python uses one observer process.
   Managed Rust explicitly selects the otherwise default-excluded identities. The selected
   Python build profile does not change the Rust test graph.

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

For explicitly requested strict assembled persisted-reuse qualification, use `--python-profile producer` and
supply the reviewed host inputs through `PSE_RUNTIME_PRODUCER_DECLARATIONS`,
`PSE_WORKER_PRODUCER_DECLARATIONS`, `PSE_PYTHON_PRODUCER_DECLARATIONS` and
`PSE_NATIVE_PRODUCER_INPUTS`. The declaration variables name current source-bound
actual-build review contexts; the native-input variable names a JSON list of files.
`PSE_PYTHON_PRODUCER_INPUTS` can select a distinct native-input list for Python.
The capture command consumes these reviews; it does not create completeness or
source-review assertions. Missing or ineligible inputs fail qualification.
Python capture replays [Maturin's extension linking selection](https://github.com/PyO3/maturin/blob/v1.15.0/src/compile.rs#L834)
with `PYO3_BUILD_EXTENSION_MODULE=1`; runtime and worker builds omit that switch.

The runner installs the producer-profile extension first, captures the three actual
deployment targets, then runs the eligible Python solve/reopen control with
`PSE_PYTHON_DEPLOYMENT_OBSERVATION_OUTPUT` naming a fresh output file. The wrapper
passes that output to the control only after recording the imported binary and its
input captures. That control writes the
actual imported extension's run-header context after verifying its mapped code against
the selected artifact capture. `PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS` supplies independent
observations of the actual deployed role artifacts. The native cross-role control checks
each eligible capture against its role observation and refuses each receipt for the other
receiving role. Role artifacts and their outer observations need not match. Provenance
hashes the captures, role observations and header bytes; a receipt's self-reported artifact
cannot supply this control's independent expected observation.
For a standalone capture, `just producer-deployment <fresh-output>` consumes the
same reviewed inputs. Subsequent native commands use
`PSE_WORKER_PRODUCER_RECEIPT`, `PSE_PYTHON_PRODUCER_RECEIPT` and
`PSE_PYTHON_DEPLOYMENT_ATTESTATION` and `PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS` for their
actual input receipts, observed header and independently observed role artifacts;
the association control must have produced the header first.

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
The production deployment capture and imported Python association always execute
fresh within an explicitly selected producer campaign; `--reuse` and `--transfer` refuse those two gates. This
is a condition for a new current-artifact claim, not a requirement to start such a campaign
after a dependency/environment change or a rejection of historical scientific results. Their host review contexts
and native-input lists can select external inputs, so ordinary source-only reuse
cannot establish a current deployment. Other eligible gates retain the continuation
behavior above.

**Aggregates and advisory checks.** `quality`, `governance`, `ci-fast`, `ci-pr`, `clippy`,
`fmt-check`, `codegen-check`, `adr-lint`, `deps-report`, `policy` and the working bundles
`turn-end`, `ready` and `hygiene` call the same runner with `--group`. Each writes its own
run directory without source provenance and updates the convenience link
`build/assessment/latest-<group>`; the printed path is the immutable one. The console shows
each gate's command and log path as it starts and one result line as it ends, with the last
20 log lines of an unqualified gate; `--live` (or `PSE_VALIDATION_LIVE=1`) also streams gate
output (`ready` always does). A signal (SIGINT, SIGTERM, SIGHUP) terminates the running
gate's process group, records it `interrupted` and the rest `not_run`, and exits 128+N.
The `audit-*` gates are advisory: findings do not fail `deps-report`, but a tool failure
does. `policy` runs the same audits as required gates.

Retrieve the printed immutable path with `just result build/assessment/<run> --failures`.
`latest-<group>` is resolved once when reading. Reused evidence retains its original
artifact/log paths and evidence kind; reading it does not establish a newly tested run.
Missing records/files and unattempted declared gates remain explicit. A readable failed
or incomplete assessment gives reader exit zero; this means only that it was read.
Missing/malformed checkpoints give exit 1; usage or an unknown gate gives exit 2.

## Individual commands

| Command | Scope |
|---|---|
| `just test [nextest args]` | The default workspace feature graph, run with `cargo nextest run --no-fail-fast` and force-validation. |
| `just native-test [nextest args]` | The full workspace with `pse-runtime/native-solvers`, `pse-runtime/canonical-tests`, `pse-tests-conformance/native-acceptance` and force-validation, under the native environment. Nextest owns selection. The wrapper writes a raw selection artifact and native identity before execution. |
| `just doctest` | Workspace doctests with force-validation. `pse-py` is excluded because Cargo cannot run cdylib doctests. `doctest-release` is the release-profile variant. |
| `just py-sync-native` | Rebuilds the editable extension (`dev` profile, `force-validate,native-solvers`) and regenerates the compiled API stubs. `just py-sync` installs the default profile, which lacks native solvers. |
| `just native-python <output> [pytest args]` | Linked Python `unit or component or integration` tests, reported to `<output>/native-python.xml`. Canonical fixture consumers require an initialized supervised `PSE_SURREAL_STATE`. Run `py-sync-native` after Rust edits. |
| `just governance-tests [args]` | `pse-tests-governance` with force-validation. |
| `just setup-test` | Stdlib `unittest` discovery over `scripts/tests`: setup, guards, runner, docs and build tooling. |
| `just setup-test-report <output>` | The same discovery with an XML reporter, writing `setup-test.xml` and `setup-test-selected.json`. Fails on skips or an empty run. |
| `just unit-consolidation-tools` | Focused runner, ownership, selected prerequisite, measurement parser and isolated build snapshot controls. |

These tooling tests establish runner behaviour, not product or scientific acceptance.

## Case measurements

The host owner records the admitted lane, actual process placement, competing
cooperating owners and external pressure alongside measurement context. Service
generation, receiver identity and process residency are distinct from RocksDB cache,
filesystem cache, imported extension and prepared-product residency. A fresh process
or database does not establish a cold filesystem. Uncontrolled cache/load conditions
are labeled; quotas and a configured CPU mask do not establish timing isolation.
Long-lived service RSS/high-water observations are not attributed to an individual
case. Keep build, preparation, numerical execution, publication and serving intervals
separate when describing benefits.

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
| `lifecycle` | Runtime/store/lifecycle enforcement and connected result resource controls |
| `native` | The explicit full linked workspace covering invocation |

Obtain selected prerequisite evidence with, for example,
`just assessment --functional-scope preparation`. For several workload groups, use
`just assessment --functional-scope native` to cover them in one Rust invocation.
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
  execution, validation and results access, connected result reopening, and teardown. Cold
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

`just py-test` starts pytest directly. Fixture consumers explicitly require the initialized
canonical substrate; collection and pure units do not start a storage fixture service.
The all-extension IPC round trip is requested by one dedicated
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
