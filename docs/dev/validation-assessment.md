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
Existing strict producer/artifact commands retain their own semantics. Plan39’s additional environment/provenance proof targets are withdrawn: no all-role campaign, source-completeness investigation or compiler/launcher/installer phase proof is scheduled or deferred. Do not freeze or
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
| `just codegen-check` | on demand | every generated tree equals a fresh regeneration, with no extra or untracked generated files (ADR-0051) | nothing about runtime behavior of the generated interfaces, or whether the selected correctness command activated Arrow validation (scope-aware routes inspect actual units) |
| `just family-check` | when a pinned-family dependency moves | one resolved version per dependency family, equal to the pins | nothing about whether that version behaves as documented |
| `just governance` | on demand | the workspace-level invariants hold (pins, crates registered, the dated nightly at or above the `rust-version` floor, unsafe allowlist, error taxonomy) | nothing about runtime behaviour, or whether the source still compiles on the stable floor |
| `just quality` | on demand | Python format/lint/types/import boundaries and repo config are clean | that the code works |
| `just audit-shear` | at functional scope end through `hygiene` | pinned shear finds no unused declarations, unlinked files or empty test-target warnings, using `--locked --check-test-targets --deny-warnings` | compiler name resolution, macro expansion or all feature/platform correctness |
| `just deps-report` | on demand | what is in the dependency graph and under what licences; **advisory: findings never fail it**, a tool that cannot run does | nothing — it refuses nothing and blocks nothing |
| `just policy` | on demand | the same checks, strictly: no known advisory, no disallowed licence. Opt-in, not in `ci-pr` | nothing about code you wrote, and nothing you are obliged to act on yet (register R-31) |
| `just parity` | on demand, when parity is in scope | the exercised parity checks pass against `idaes-pse==2.13.0` | nothing about cases not exercised, or other IDAES versions |
| `just docs` | on demand | documentation HTML and scoped search build; manual CI can also check internal links | nothing about whether the prose is true |
| `just adr-lint` | on demand | ADR front matter, numbering, supersession and register rows are well-formed | nothing about whether the decisions are good |

**Never report that tests pass without naming the command, the mode, and the baseline.**
"34 failed" is not information until the baseline is known — and here the baseline is zero.

## Common conditions

### Inspecting a product invocation

`just result INVOCATION --kind product-test --json` reads the authentic invocation,
frozen selection, report rows and terminal without rerunning tests. Add `--subject`,
`--failures`, `--offset`/`--limit` or bounded `--tail` to focus output. Metadata-only reads
do not hydrate payloads; explicit payload reads require a current registered evidence
owner and temporary protection. Missing protection leaves that artifact unavailable.
A completely reported failing run can be complete. A missing terminal, selected but
unreported test or interrupted producer cannot become a pass through a JUnit file.
Captured producer/composer identities are association facts; current liveness and owned
drain remain unobserved unless their owner supplies them.

Managed correctness commands return zero only after complete reconciled Pass. Raw child
status, cancellation, immutable assurance and independent cleanup remain separate facts.
A zero-exit child with missing, skipped or inconsistent selected evidence fails the managed
command. A projection/reconciliation failure also returns nonzero without rewriting an
already frozen scientific assurance result. Result-reader exit zero still means only that
the selected record was read; environment forwarding preserves its command/signal contract.
Blueprint §24.1 owns the contract; the [Plan53 coordinator](../plans/53-test-architecture-target.md)
records the implementation and its qualification boundary.

Managed nextest routes interpret supported Cargo/nextest options once, including effective
profile aliases/environment, repository and lower-priority tool configurations, and opaque
arguments after `--`. The actual pinned-tool catalogue owns selection. Report copies edit
only the selected profile's JUnit path. A known empty catalogue is retained as `empty-selection`
and `not-run`, with no child workload exit; it is not uncollected scope or passing correctness.
Python adapters likewise share supported selection/report interpretation and retain pytest's
actual collection authority. Unsupported scope options refuse before preparation.

The requirement owner attaches effects, fixture inputs, capacity and campaign roles to actual
catalogue members. Rust uses declared owner defaults and bounded fixture/peer exceptions;
pytest uses actual fixture closure plus explicit `test_requirements` control declarations;
setup tests use actual unittest selection and class-local declarations. Required role coverage
is checked even for nonempty selections. Canonical registration, native fixture preparation
and scientific context admission check demand before effects. These are managed fixture
contracts, not a hook around every direct in-process solver call. Low-level/unmanaged commands
retain their narrower automatic association and cleanup support. Existing timing/isolation
scheduling policy remains separate; Plan53f consumes the delivered requirement facts.

Source fixture owners record authenticated typed infrastructure failures before panic/error
projection. Python fixture setup aborts report assertion reach `not-reached`; a Rust helper
returning an error reports `unknown`, since its caller may handle that error or have already
asserted something. Passed negative controls remain passed. Failing setup observations are
kept distinct from test assertions and make regression isolation inconclusive rather than a
bad-revision predicate. Missing observations never establish assertion reach. Observer or
cleanup failure does not mutate frozen assurance. Restart peers join authentic frozen child
invocation identities in the same registry and report driver cleanup separately.

The canonical FastMCP surface consumes these operation/result owners through Plan52i–52k.
Exposure and reveal do not authorize effects, and transport success does not qualify tests.
Retained native drivers are implementation controls; they do not establish host attachment
or progressive-reveal qualification.

Boundary coverage executes through the installed correctness extension. The disabled
`pse-py` cdylib Rust harness is not evidence. Active Python tests exercise generated settings,
named configuration diagnostics and an actual native route refusal through its public
`DiagnosticReport` and generated envelope. Schema unknown-reference diagnostics and the
synthetic NativeReuse/source/physical/missing/WorkflowPanic combination have no corresponding
public producer in this scope; lower schema, runtime taxonomy and document codec tests remain
separate coverage, not an equivalent Python journey. No production test-only constructor is
provided to manufacture those combinations.

Canonical API tests run independently of the opt-in operator graph/query cases. The internal
`scripts.tests.canonical_boundary_run operator DIRECTORY --executor EXECUTABLE` route selects
both operator roles and captures the executor before another Cargo feature build can replace
its alias. A selected missing prerequisite fails. Its `restart DIRECTORY [--executor EXECUTABLE]`
route runs under `scripts/pse-env --resource-class build --native=solver`: the parent needs runtime
libraries and admits compilation before service creation. By default, the existing correctness-aware
Cargo owner prepares the canonical-tools initializer and returns its actual executable artifact;
missing cleaned build output therefore triggers preparation. An explicit executor must already
exist and pass initializer validation, with no fallback to another artifact. The initializer is
frozen before subsequent feature builds can replace a target alias. Rust preparation selects its
native compiler environment locally. The external server must already be installed: setup reuses its package receipt independently
of the parent's compiler or loader context and never implicitly compiles it. Restart owns a fresh nonresident service, observes a real
outage and new server invocation, joins the distinct authentic Rust result, and drains/stops
only its own service. The context-overlap driver likewise selects its actual pytest component
peer and requires the cross-language role on each side. Prepare and execute that driver inside
one admitted observer lifetime, preserving its captured environment; its Python peer uses fresh
driver-owned pytest scratch. These controls share the existing
invocation/resource/result registry; their mutable service or database isolation is not a
second evidence authority.

`just activity` exposes current build/wait/admission observations without workload setup.
Actual Cargo records identify compile/link/cache freshness; quiet CPU does not identify
the owner of an artifact lock. `just context QUERY --json` projects declared commands and
source owners; `just affected --json` previews selected Rust correctness commands and
explicit Python/native/generator/dynamic omissions. Neither executes the suggested check.

`just reference-run --state STATE --worker WORKER [--initializer INITIALIZER] -- COMMAND
ARGUMENTS` composes setup, frozen initializer selection, admitted reference startup,
initialization, literal workload, exact drain/stop and normal-service resume observations.
Execution and cleanup are separate outcomes; a successful workload cannot hide failed
teardown. The prebuilt initializer is copied before another Cargo feature build can
replace its target alias. The existing scientific/reference allocation remains authoritative.

Explicit `python -m scripts.test_resources abandon INVOCATION` inspects eligibility;
`--apply` freezes membership before release. Authenticated drain, references and unknown
effects retain their existing guards. Abandonment is a separate fence and does not fabricate
a terminal record, ACK or scientific outcome. It cannot make protected evidence disposable.
The proposed ADR-0174 records the selected distinction; acceptance of that ADR remains a
separate decision.

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

Explicit report custody conversion can move a selected drained dependency set into the
current checkout before its former worktree is removed. The resource owner copies and
verifies every producer byte, preserves original receipt/outcome/provenance meaning,
rebinds physical identity witnesses and dependent locations, and keeps old and current
locators readable. Its durable reservation fences readers and disposal until conversion
settles. A converted missing manifest remains missing; relocation grants no qualification
or disposal authority. Worktree removal follows independent current-location verification.

Collected invocation declarations and requirements are immutable digest-bound objects;
coordination records contain mutable membership and their references. Uncollected Python
staging is distinct from a collected empty selection. Current effect admission requires the
collected requirements policy; conversion preserves old terminal identities and original
metadata without inventing current qualification. Registry-format conversion is explicit,
quiescent and resumable behind a refusal fence; it does not reset canonical payloads.
Service discovery uses covered service cohorts bound to the actual registry and service.
An empty unrelated registry, unresolved original receiver association or an access marker
alone cannot establish service quiet. Disposable scratch candidates have their own rotating
access path; every candidate still requires authoritative eligibility, reader and drain checks.


Display resolves a latest pointer once and borrows that exact report plus referenced origins.
It creates no persistent consumer reference or checkpoint. New retained-reference reuse requires a sealed origin
and unchanged receipt identity; unregistered or manifest-less historical reports remain
displayable but cannot supply new reusable claims before actual owner qualification. Existing owner reservations, no-follow traversal, active-use
exclusion, actual drain and final identity rechecks still govern exact cleanup. Document
retirement metadata does not release a resource. Explicit owner adoption of a historical
attempt requires its actual outcome, provenance, references and durable authored inputs;
unknowns stay protected.

The comprehensive scope runs these gates in order:

1. `py-sync-native-correctness` with the selected `--python-profile` (`dev` by default), then
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
- `checks.json` (version 6), protected immutable `details/` objects, `failures.json`,
  `test-findings.json` and `summary.md`.

The version-6 root publishes compact gate outcomes and references immutable source and gate
detail. Detail is durable before atomic root publication. Ordinary inspection reads the root;
a selected gate or failure view opens only its requested gate detail. Source detail and a
faithful full export are explicit reader options (`--source-detail`, `--full-export`). Missing,
changed or unavailable detail refuses the requested view rather than supplying an empty or
passing population. Known sealed artifacts authenticate the exact bytes read against their
retained identity and digest; root custody alone does not authenticate every detail object.
Whole-archive inspection explicitly verifies the complete manifest. Historical version-5
receipts remain available through explicit original inspection and cannot supply automatic
unchanged-input reuse under the current policy.

The typed `assessment_read` capability uses this same protected reader. Its answer completeness
describes the requested read, independently of a gate's workload outcome: a complete selected
answer can faithfully report a failed gate. Summary-only reads retain exact detail/source
references and explicitly omit bodies. Selecting one gate acquires its detail before typed
presentation without opening unrelated gate bodies or the source map. An oversized explicitly
selected answer refuses rather than truncating native facts; the native full export remains
available. Reused gates retain their original evidence attribution, with origin protection
established before consumer publication and held while reading.

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
Python capture observes Maturin's actual Cargo child, including its extension
linking selection and `PYO3_BUILD_EXTENSION_MODULE=1`; runtime and worker builds
omit that switch. The single preparation operation refreshes selected executor
families before any role capture. Runtime and worker captures precede the actual
Maturin child, whose qualified sealed realization is required before installation
continues. Historical evidence remains separate from current deployment validity.

The runner completes all three actual captures and Python installation, then runs
the eligible Python solve/reopen control with
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
The `audit-*` gates are advisory inside `deps-report`: findings do not fail that bundle,
but a tool failure does. `policy` runs the same audits as required gates. `hygiene` also
runs `audit-shear` as required: warning-only findings fail, and malformed or failed tool
execution cannot count as a clean result. The dependency detector never edits manifests.
Security and licence reporting retain their separate cargo-audit/cargo-deny owners.

Report completion seals the current assessment's artifacts and outcome. It does not sweep
unrelated historical scratch or fixture resources. Selected test terminal owners retain their
own teardown, and historical resource maintenance remains explicit through the existing exact
`scripts.test_resources` inspection and reclamation operations.

Aggregate `codegen-check` uses `codegen-all-check` for the Rust-owned outputs: one compiled
full-codegen invocation checks registry languages, physical fixtures, document schemas and
header-derived bindings. Independent Python contract, installed compiled-stub and FastMCP
reference checks remain separate. Independent generation, candidate validation, byte comparison
and tracking failures are reported together; comparison drift does not suppress tracking or
later targets. Check mode writes only scratch candidates. The focused `codegen-*-check` commands
remain useful when a smaller owner is changing or the full generator cannot compile. A failed
generator build establishes no drift result for the targets it could not execute.

Retrieve the printed immutable path with `just result build/assessment/<run> --failures`.
`latest-<group>` is resolved once when reading. Reused evidence retains its original
artifact/log paths and evidence kind; reading it does not establish a newly tested run.
Missing records/files and unattempted declared gates remain explicit. A readable failed
or incomplete assessment gives reader exit zero; this means only that it was read.
Missing/malformed checkpoints give exit 1; usage or an unknown gate gives exit 2.

## Representative functional selections

Choose package/filter selections that reveal the changed contracts. The whole-workspace
`native-test` command also includes fresh full-schema fixtures, real-window lifecycle
controls and large store qualification cases. It is not a default fast functional bundle.
Pure controls, native-local solver/ownership checks and selected live-store transitions
should be qualified at representative scale. Page-boundary and race controls retain their
actual boundary and relevant orderings; full deployment/capacity journeys remain explicitly
selected scope. Report exclusions and interrupted cases rather than treating a smaller
selection as full qualification. Plan49's historical [qualification Outcome](https://github.com/paul-heyse/pse-arrow/blob/197616768b1b7abcbe67af2b5b8d42ab445b291b/docs/plans/49-library-native-remediation.md#outcome--remaining-49f49h-2026-10-23)
records its selected fixture corrections and evidence limits.

## Individual commands

| Command | Scope |
|---|---|
| `just test [nextest args]` | The default workspace feature graph, run with `cargo nextest run --no-fail-fast` and force-validation. |
| `just native-test [nextest args]` | The full workspace with `pse-runtime/native-solvers`, `pse-runtime/canonical-tests`, `pse-tests-conformance/native-acceptance` and force-validation, under the native environment. Nextest owns selection. The wrapper writes a raw selection artifact and native identity before execution. |
| `just doctest` | Workspace doctests with force-validation. `pse-py` is excluded because Cargo cannot run cdylib doctests. `doctest-release` is the release-profile variant. |
| `just py-sync` / `just py-sync-native` | Rebuilds the interactive editable extension without `force-validate`, ordinary or linked respectively, and regenerates its actual API stubs. Local admission and structural transport safety remain. |
| `just py-sync-correctness [mode] [profile]` / `just py-sync-native-correctness [profile]` | Builds the explicitly force-validated correctness extension and its actual stubs. `auto` preserves the installed ordinary/linked selection; explicit `dev` or `native` selects it. Compiled Rust flags, imported checkout path and binary digest are independently checked before tests. |
| `just native-python <output> [pytest args]` | Linked Python `unit or component or integration` tests, reported to `<output>/native-python.xml`. Canonical fixture consumers require an initialized supervised `PSE_SURREAL_STATE`. Build with `py-sync-native-correctness` before capturing deployment association or running tests; the observer refuses wrong-mode/stale imports and never rebuilds behind a captured receipt. |
| `just governance-tests [args]` | `pse-tests-governance` with force-validation. |
| `just setup-test` | Stdlib `unittest` discovery over `scripts/tests`: setup, guards, runner, docs and build tooling. |
| `just setup-test-report <output>` | The same discovery with an XML reporter, writing `setup-test.xml` and `setup-test-selected.json`. Fails on skips or an empty run. |
| `just unit-consolidation-tools` | Focused runner, ownership, selected prerequisite, measurement parser and isolated build snapshot controls. |

These tooling tests establish runner behaviour, not product or scientific acceptance.

### Selected behavioral investigations

The protocol-sensitive Cargo tools are pinned once in `.config/diagnostic-tools.toml`;
bootstrap, doctor and their consumers use that declaration. These are opt-in investigations:

```bash
scripts/pse-env --resource-class build -- .venv/bin/python -m scripts.coverage_correspondence capture --branch -- -p pse-ids --lib -E 'test(scientific_replay)'
scripts/pse-env --resource-class light -- .venv/bin/python -m scripts.coverage_correspondence report --capture build/assessment/CAPTURE --base HEAD --json
just affected --coverage build/assessment/CAPTURE --json
just mutants-file crates/pse-ids/src/scientific_replay.rs '' --lib
scripts/pse-env --resource-class build -- .venv/bin/python -m scripts.boundary_fuzz --seconds 60
```

Coverage captures bind source/build/instrumentation inputs and retain the actual nextest JUnit
population. Changed-source views refuse stale correspondence and leave dependency selection
conservative. Branch coverage uses the pinned nightly's unstable instrumentation; merged
coverage identifies a selected population, not an individual covering test or assertion strength.
Native C/C++ remains opaque. `coverage` and `mutants-file` choose fresh output directories by
default; explicitly named existing directories are refused.

Mutation retains an unchanged baseline in the actual tool copy before interpreting caught,
missed, unviable, timeout and setup outcomes. Native use selects the direct
`scripts.assertion_sensitivity --native` route under the appropriate `pse-env --native` context;
pure-boundary acceptance does not qualify every native mutation-copy configuration. Fuzzing
targets only the frozen inspection decoder/member validator, starts from authentic and malformed
seeds, and owns a finite corpus/artifact directory. `--replay PATH` and `--minimize PATH` inspect
concrete findings; any genuine reduced regression belongs with the ordinary boundary tests.
Neither successful tool completion nor a finite clean campaign establishes exhaustive correctness.
For this delivery, automatic review denied the instrumented fuzz execution task before the
campaign ran. The ordinary `structured_decoder_controls_keep_actual_operation_meaning_and_refuse_bad_boundaries`
test exercises actual operation meaning, exact large counters, truncations, malformed input,
unsupported versions and the 64 MiB size guard. Coverage-guided execution/minimization remain
unqualified; the owning Plan52c records the execution boundary.

`just features-combinations` and `just features-no-default` now use the assessment owner's
combination campaign. The pinned cargo-hack enumerates literal argv through the Cargo adapter;
its displayed command text is never shell-evaluated. Set `PSE_FEATURE_RESUME_FROM` to a prior
assessment path to reuse only completed successful combinations with unchanged full applicable
source/compiler/config/environment inputs. Interrupted and failed attempts remain recorded;
a real Rust repair invalidates the applicable campaign. Reused origins are explicit, and a
missing/failed combination prevents `required_checks_covered`. One driver owns overlapping
graphs in a checkout. A small matrix qualifies this mechanism; the full required feature
campaign remains an independently selected scope-end assessment.

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

`just case-measure <new-output> [--functional-from <assessment-dir-or-checks.json>]`
measures selected complete-process, preparation or admission cases. Repeat `--case <id>`
to select declared cases; no selector runs the complete campaign. Workload declarations in
`.config/process-cases.json` and `.config/preparation-cases.json` identify the functional
areas relevant to interpreting a measurement, without requiring a particular assessment
invocation or an environment/source/artifact identity proof.

Functional test evidence may be referenced with `--functional-from`, or cited separately
when reporting the observations. The optional report must contain completed, successful test
outcomes; failed commands, failed/skipped tests, missing selected results and contradictory
terminal evidence are refused. The reference retains the reported checks and scope, plus
available source and environment differences. It does not assert that the test and benchmark
conditions are equivalent. Different Cargo profiles, provider bytes or unrelated source changes
do not invalidate successful observations. Scientific assertions in each benchmark and valid
output samples remain required. Managed execution retains its configured reference allocation,
receiver ownership and cooperative drain requirements.

`just bench-case-smoke <new-output> [--case <id>]` runs untimed controls and cannot supply
performance observations.

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

`case-measure.json` (`process-cost-v4`) records the optional functional-test reference,
selected workloads, benchmark paths, profile, thread settings and observed host conditions.
Available source/environment differences are recorded, including changes during collection;
they do not automatically invalidate observations. Benchmark artifacts remain under
`process-cost/<id>/`. A failed build/case command or missing/invalid output samples prevents a
successful measurement report.
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

## Scientific test input and support ownership

The [scientific family catalog](../../tests/fixtures/scientific-inputs.toml) owns authored
roots and named projections. The ordinary bounded document reader, typed package resolver
and modeling parser derive dependencies, exact package/data bytes, imported documents and
membership/absence facts. A synthetic declaration projection is distinct from authentic
package transport. Its substitutions carry their own source paths; they never qualify
external data or an independent oracle.

The Rust selected-invocation preparer captures these bytes once when a selected consumer
package needs them; the Python controller prepares before worker launch. Children consume
`PSE_SCIENTIFIC_INPUTS`, with no mutable reference-source fallback. For a standalone command,
prepare explicitly and bind the returned path:

```bash
scripts/pse-env -- python scripts/scientific_inputs.py capture build/my-scientific-inputs.json
PSE_SCIENTIFIC_INPUTS="$PWD/build/my-scientific-inputs.json" scripts/pse-env -- cargo t -p pse-compiler -E 'test(authored_process_and_caloric_declarations_admit)'
```

Use a fresh capture destination. `just seed-conformance` prepares its invocation automatically;
ad hoc reference conformance commands use the same explicit role. Changing shared science
changes runtime input identity and preparation; those bytes are no longer embedded into each
migrated Rust test binary. `affected` consumes the same derived closure and widens on unknown,
new or deleted membership. Its optional coverage overlay never prunes dependency reach.
Selected pure historical archives stage the exact capsule as a logical role and remap it on
replay; native archival retains its existing refusal boundary.

Compatible compiler tests may share an immutable admitted `ModelingRevision` within their
process and publish it into a fresh workspace. This is separate from invocation-wide byte
capture: nextest children do not share an `Arc`. Mutable workspaces, runtimes, numerical
attempts, databases and borrower lifetimes remain independently owned. Operations
`canonical-tests` selects test participation; `test-support` selects fixture lifetime and
deterministic hook instrumentation. Production-shaped Python checks exercise the complementary
support-absent closure. These changes establish removed setup/embedding work structurally;
execution-time benefits require separate measurement.

Fresh per-test canonical databases and borrowers register with the existing invocation/resource
owners and retain the production admission and drain contracts. Isolated execution does not
create another outcome authority: selected reports and terminal reconciliation remain with the
established result composer and readers.
