# Agent environment

How commands, environments, scopes, worktrees, test selection, hooks and runtime
capabilities work for the coding agents (Claude Code and Codex) and for anyone at the
terminal. `AGENTS.md` keeps the always-loaded rules; this page holds the mechanics behind
them. Plan 29 introduced most of it.

## Preparing and activating a checkout

Run `bash scripts/setup-env` for locked development tools and the existing agent/skill
configuration. It preserves installed native packages and does not build the extension.
Choose `python`, `native` or `docs` explicitly to add the existing `just py-sync`,
`just py-sync-native` or `just bootstrap-docs` preparation. Installed Rust toolchains are
selected by `rust-toolchain.toml`; setup does not install them.

Review `.envrc`, then run `direnv allow` in each checkout. Activation removes inherited
Python environment paths before composing the existing export owner, and does not run
doctor, synchronize packages or build native code. `.envrc.local` remains the ignored
personal override. A checkout without its own `.venv` needs explicit preparation for
Python work; it must not borrow another checkout's packages.

On this workstation, Codex's `local-env` profile initializes direnv before every Bash
command through `~/.config/codex/bash-env.bash`. New or changed `.envrc` files need review
and approval before commands run; activation failures stop the command. Keep using the
workload boundary below for execution and native/store admission.

After successful activation, the workstation bridge unsets `BASH_ENV`. Codex supplies it
afresh for each top-level shell; nested Just/compiler/native children preserve the environment
already selected by their workload owner. A shell opened before that bridge update may still
carry its old startup mapping: use a fresh activated shell for propagation checks.

## One native environment and foreground boundary

The selected Plan58 ordinary path is `scripts/pse-env`, a shell exec adapter over
`pse-host boundary`. It starts no Python controller, schema client, database or scientific
runtime. Construct or update the native tools explicitly with `scripts/bootstrap-host`.
The seed builds only locked `pse-host` binaries, validates both completed executables and
atomically selects one immutable pair in `target/host-tools/current`. A failed or interrupted
update preserves the previous pair. Running commands and nextest adapters keep their actual
executable generation. Help, reads and ordinary launches never rebuild missing tooling.

```bash
scripts/pse-env -- <command>          # native environment and owned foreground execution
scripts/pse-env --docs -- <command>   # select already provisioned .venv-docs
scripts/pse-env --cache off -- <cmd>  # explicitly disable compiler wrapping
scripts/pse-env --cache on -- <cmd>   # require supported installed native sccache
scripts/pse-env --explain --json      # source-free, redacted provisional selections
scripts/pse-env --print               # fixed Bash activation program, no private values
scripts/pse-host activity [UNIT]      # direct current/named unit observation, no admission
```

Recipes, direnv and the Claude SessionStart hook select this same native owner. Composition
preserves command selections over caller values, caller values over personal `.envrc.local`,
and personal values over repository defaults. Explicit empty values remain selections.
Personal shell evaluation happens once per explicit activation in an isolated shell. `--print`
itself never evaluates it: the returned fixed Bash program obtains a complete private NUL
transfer, waits for a successful producer and validates the whole mapping before changing the
parent shell. The transfer is data, never evaluated shell source. Failed activation leaves
parent environment values unchanged. Private values do not appear in the printed program,
explanation or debug formatting. Explanation leaves unread personal selections unresolved.

The native environment selects checkout-local optional Python paths, the declared Cargo width,
existing native compiler inputs and installed native sccache directly. It does not provision
`.venv`/`.venv-docs`, synchronize packages or start a canonical server. Missing optional cache
in auto mode means direct compilation; explicit invalid cache selection refuses. Unsupported
sccache logging/distribution selections compile directly in auto mode and refuse in required
mode. The old Python per-compiler wrapper is not selected. Cache service disappearance alone
does not invalidate a successful compiler result; actual compiler failure still propagates.
`--cache off` disables wrapping even when an inherited explicit cache binary is missing.

## Native foreground ownership and supported placement

Configured `PSE_BUILD_MODE=unbounded` ordinary work uses all host CPUs and the declared Cargo
jobs without a new memory cap or slot ledger. Recipe class hints do not recreate finite
allocation policy. Explicit finite/scientific classes, `PSE_MEMORY_MAX`, custom slice requests,
reference/timing selections, `--native[=...]`, `--store` and the retired allocation-binding
protocol currently refuse before command execution. Their Plan58 B/C/D/F owners must supply
working capabilities before those workflows return; there is no Python fallback or silent
resource downgrade. This selected wave is database-free. Existing stores and services are
preserved. `--docs` selects an existing environment only.

For a top-level supported app-scope caller, the host starts a same-image child behind a private
exec gate, creates a caller-only scope and a separate workload scope in the actual prior slice,
and binds workload lifetime to caller lifetime. Only after containment and population are
established does it release the gate. The prior application unit remains a lifetime dependency.
The gate closes without execution if the controller dies during startup; after release the user
manager stops the workload if the controller dies, including SIGKILL. The child locally execs
raw Unix argv/environment/cwd with inherited stdio and controlling terminal; no payload is
serialized through D-Bus and no TTY relay or resident supervisor is introduced.

Nested calls authenticate actual manager scope identity, native role/dependencies and current
kernel membership, then exec within that containment. An environment marker grants no authority.
`--no-scope` requires this actual enclosing native workload; it cannot bypass containment.
Uncertain views, unsupported/dedicated prior placement and displaced finite CPU/memory/cpuset or
custom lower task constraints refuse. Common slice ancestry and the user manager's default task
limit remain; splitting ordinary scopes is not finite aggregate allocation accounting.

Startup has one absolute deadline. Cleanup has one absolute drain deadline and attempts every
independent settlement, preserving the original operation failure alongside cleanup failures.
A completed manager stop job or direct-child exit is insufficient: the bound original cgroup
must become physically empty. Each private workload has an explicit manager stop grace within
the drain budget. Existing borrowed units are observation-only. Private manager mutation supports
one controller per fresh intent; external same-UID restart/reconfiguration is unsupported because
systemd255 offers no atomic StopUnit-if-InvocationID operation.

Exit status is the original command status or `128+signal`; missing/unexecutable commands retain
127/126. Boundary or unresolved cleanup failure returns125 and names the failure without claiming
physical release. A cleanup error also retains the original command status in its diagnostic.
Read-only `just activity` runs directly through the native host and accepts a current/named unit,
not arbitrary commands or retired build-stage/watch flags. It stays independent of environment
activation, compiler preparation, database readiness and scientific initialization.

## Native Cargo correctness

`cargo c`, `cargo t` and the generic check/unit recipes select the native correctness executable.
It asks pinned Cargo for the actual compiler unit graph, including host/build and selected-root
dev units, adds necessary validation opt-ins to only consuming selected roots, then verifies the
final root identity and Arrow leaves before compilation. Pure closures stay pure. Unknown graph
coverage refuses; graph stdout remains private and compiler JSON stays unchanged for nextest.

For nextest, the native composer invokes the installed `cargo-nextest` runner directly because
Cargo overwrites `CARGO` for external subcommands. An invocation-local native Cargo adapter sees
nextest's actual translated compiler arguments. One static setup restores actual Cargo for test
attempts. Source-build run/list in this checkout are supported; reused archives/binaries, foreign
workspaces and workspace/tool configuration overrides refuse before runner launch. Compiler
requests remain graph-validated even if a caller supplied a nextest phase marker. Nextest retains
its native argument grammar and test scheduling; no Python selector, fixture permission registry,
per-test supervisor or generated runner configuration is inserted into the generic unit path.

## Builds in one checkout

`PSE_TEST_RESOURCE_STATE=/absolute/private/directory` selects an isolated test-resource
metadata registry without changing shared host admission. A fresh empty private directory
initializes its own exact registry. Existing legacy metadata requires explicit fenced
migration/recovery; ordinary tests refuse it rather than silently importing or resetting
live ownership. This selector does not select or initialize a canonical database.

Cargo uses individual build-unit locks (`[unstable] fine-grain-locking` in `.cargo/config.toml`).
That setting does not establish safe concurrency for overlapping dirty graphs. Upstream
[Cargo #17508](https://github.com/rust-lang/cargo/issues/17508) reports a permanent deadlock
between concurrent checks sharing dirty units; no fix is verified in the pinned nightly.
Keep an overlapping same-target feature matrix or Clippy graph under one Cargo driver.
Compiler jobs remain at the configured 32, and independent non-Cargo work can run concurrently.
Work that genuinely needs isolated concurrent builds uses its own checkout and target; do not
create cold build copies merely to parallelize a matrix when storage/cold cost defeats the gain.
Cargo-hack's native partitions are disjoint command groups, but their dependencies still share
units in one target, so partitioning alone does not remove this limitation.

Ordinary editable Python preparation separates dependency setup, artifact construction,
generated stubs and consumer attachment. Explicit extension preparation invokes the trusted
Cargo/Maturin path; Cargo owns incremental rebuild decisions. The preparation owner copies a
completed image to a private immutable generation, observes its compiled contract once and
atomically publishes the descriptor in the existing receipt. Build, installation, copy or
publication failure before the descriptor commits preserves the preceding selection. An uncertain
replacement outcome retains both possible generations; no selected image is deleted. Partial
installation cannot become an issued generation. Stub generation retains its independent declaration/output responsibilities.

Consumers use that issued contract. They do not reconstruct source/compiler/environment inputs,
parse Cargo dep-info or logs, hash the image again or fence its metadata on every read. Linting,
unrelated builds and source edits do not invalidate the issued generation. Supported local images
load from their published immutable path. A retained worker keeps that generation until explicit
drain/replacement; new publication does not silently replace an existing import or cursor owner.
Native provider resources retain their original lifetimes. Serving-code edits require draining
and reloading affected endpoints before edited code serves operations, including lazy imports.
There is no source watcher or secondary freshness ledger establishing that editing contract.

Capability requirements distinguish the installed correctness extension from explicit native
correctness. The installed receipt supplies omitted mode/profile selections; it never defaults an
unknown installation to dev or discards a native installation's provider requirements. Prepare the
chosen mode explicitly, then restart/reattach after replacement. Eligible endpoint reads retain
actual native objects only within their original immutable source/context/image premises. EOF and
endpoint teardown settle admitted work and close those owners; observer cancellation alone does
not release them. See the [capability guide](capability-interface.md#native-schemas-actions-and-protected-reads)
for cursor and lifetime limits.

## Worktrees

```bash
just worktree <name> [--ref REF] [--python | --native]
```

creates `~/pse-arrow-wt/<name>` on a new branch from REF (default `HEAD`) and prints the
commit and any uncommitted work it does *not* carry. It copies `.envrc.local`, allows direnv,
links the selected skills and reports `just doctor`. Child setup recomposes the managed
compiler router for the new checkout, preserving explicit custom/empty wrappers and the
selected real cache binary. `--python` builds that checkout's own
venv and dev extension, `--native` the linked one. Claude-created worktrees receive
`.envrc.local` through `.worktreeinclude`. Each checkout builds into its own `target/`; never
set `CARGO_TARGET_DIR` (sccache keys compilations on `CARGO_*` variables, and Cargo keys
workspace units by relative path with checkout-local fingerprint state, so sharing a target
can reuse another checkout's artifacts despite the content-freshness policy. Keep each
checkout isolated — ADR-0122). Remove a worktree with `git worktree remove <path>` once merged.

## Choosing tests

Managed routes interpret actual selected fixture, effect and exclusion requirements through
one requirements owner. Nextest and pytest still select tests; undeclared acquisition refuses
before effects. The native runner derives its per-run scheduling policy from that selected
catalogue rather than a second regular-expression policy. Placement width is separate from
semantic exclusion. Pure controls and local inspection settings need no unrelated canonical
or solver context; native/thread/timing and actual canonical demands retain their guards.
Per-run exclusivity never establishes cross-run service quiet. Actual service maintenance
uses the service's covered registry cohort and lifecycle admission; database-local assertions
retain database isolation. See blueprint §24.1 and the [Plan53 coordinator](../plans/53-test-architecture-target.md).

Selected demand also controls preparation. Effect-free ordinary Rust selections reuse their
verified nextest artifacts directly, without preparing a native worker or acquiring a store
observer. Canonical-only selections retain the real observer and store admission; only actual
native demand requires the receiver artifact. Managed native routes retain their worker
requirement. Test selection, exact artifact/source checks, before-effect guards and terminal
reconciliation still apply. Authored scientific capsules prepare for their reviewed consumer
owners, rather than every test in a package that also contains those consumers.

- `just unit-package <pkg> <filter> [args]` — a bare test-path word means `test(word)`; any
  other filterset passes through. The selection is intersected with `package(...)` over the
  final package set, so `pse-relations` (needed in the selection for force-validation)
  contributes no tests, and an extra `-p other` in `args` widens it. The command is printed;
  an empty selection fails (nextest exit 4).
- `just affected [--base REF] [--run] [--features F]` — previews `rdeps()` of the packages
  changed against REF (staged, unstaged, deleted, untracked), what that cannot cover
  (native feature-gated tests, Python consumers, generator inputs) and the exact command;
  `--run` runs it. Root build configuration widens it to the workspace.
  This is a raw nextest route: it does not automatically associate a retained
  canonical fixture with the managed runner's terminal report. Use `unit-package`
  or the existing managed native recipes when that association is required;
  inspect retained fixture diagnostics before intentional cleanup.
- `PSE_NEXTEST_ACTION=list just <recipe> ...` shows a recipe's selection without running it.
- A prepared Rust invocation resolves Arrow validation and captures its selected unit
  graph, Cargo/binary metadata and test inventory once. Execution reuses those exact
  executable associations within that invocation. Source, Cargo configuration,
  explicit build environment, selection and executable changes refuse reuse; this
  is not a cross-run cache. Authenticated observer admission, recomputed automatic
  widths and JUnit report composition are execution context, not new build inputs.
- Bare tools: `cargo t -p <pkg> -E 'package(<pkg>) & test(x)'` and `cargo c -p <pkg>` carry
  force-validation; plain `cargo nextest run`/`cargo check` do not.
- Python: `just py-unit` observes the installed extension (`doctor.py --extension-kind`) and
  uses the native environment for the linked build; `just py-test` and the native suites
  require a serving canonical server (`--store`) at `$PSE_SURREAL_STATE`.
  `just py-test` defaults to unit and component tests with the admitted runner width;
  fixture-owner groups keep mutable native contexts serial within each owner.
  An explicit selection such as `just py-test -m integration` keeps the ordinary
  process allocation and excludes tests marked `managed_primary`.
  `just py-test --managed-primary-route -m integration` runs that managed subset
  in a fresh observer process with pytest distribution disabled; its native work
  belongs to the configured primary's sixteen lanes. File, keyword and marker
  selections still apply. The runner binds fixtures to its existing selection and consumes a private JUnit terminal report;
  `just native-python <output> --managed-primary-route` owns the native receipt route.
- `just codegen-check` checks every generated output, including the native stub against the
  installed extension (rebuild it with `just py-sync` after a Rust API change).

An extension-kind/path check and a matching Python lock describe what is installed;
they do not prove that the extension incorporates current Rust source. After relevant
Rust edits, use `just py-sync` for the default extension or `just py-sync-native` for
the linked extension before inspecting Python behavior. Read-only queries do not trigger
an automatic rebuild.

The exact pinned SuiteSparse/SUNDIALS/Diffsol suppliers have local source selections
to correct consumed-input observation and preserve identical generated binding
bytes and mtimes. Their upstream trees and optional features remain available;
[supplier provenance](../../vendor/native-supplier-notes.md) owns the source hashes
and local corrections. Native-provider selection still belongs to the authenticated
operation owner, including explicit external-prefix admission.
Complete startup source observation includes those imported trees. Its finite
allowance covers both live inventories during reconfirmation; the startup lease
retains the admitted pool charge for its lifetime. A smaller pool may refuse
honestly. Do not omit a source family to fit an observation allowance.

These interactive builds omit `force-validate`. `just py-unit` and `just py-test` first
build the explicitly validated correctness extension, preserving the installed ordinary or
linked selection. `just py-sync-correctness dev` and `just py-sync-native-correctness`
select it directly. Native qualification constructs/publishes its associated immutable image
before deployment capture. Tests attach that issued generation; construction establishes compiled
feature facts, while test startup retains the separate explicit correctness-feature requirement.
Readers do not hash the image or repeat construction checks. Environment text never certifies
compiled features.

Discover the native grammar with `just --usage check-package` and inspect composition
with `just --show codegen`. For a focused machine-readable view, filter a transient
`just --dump --dump-format json` result, for example
`just --dump --dump-format json | jq '.recipes["check-package"]'`.
Structured compiler feedback is available with
`just check-package pse-structural --message-format json`:
Cargo's `compiler-message` records carry the diagnostic and spans, while non-diagnostic
records describe build progress. This does not measure compiler or solver performance.

For managed durable studies, select the reference execution profile and an already
built linked worker when creating state:

```bash
export PSE_SURREAL_STATE=/absolute/path/to/private/reference-state
just deployment-build
scripts/pse-env --native -- .venv/bin/python -m scripts.arrow_validation build -p pse-runtime --bin pse-worker --locked --features product-operators,native-solvers
scripts/pse-env -- .venv/bin/python -m scripts.arrow_validation build -p pse-operations --bin pse-canonical-recovery --locked
scripts/pse-env --native -- just surreal setup --execution-profile reference \
  --worker-executable "$PWD/target/debug/pse-worker" \
  --payload-verifier "$PWD/target/debug/pse-canonical-recovery"
scripts/pse-env --resource-class reference -- bash -euo pipefail -s <<'SH'
cleanup() {
  just surreal quiesce --state "$PSE_SURREAL_STATE"
  just surreal stop --drained --state "$PSE_SURREAL_STATE"
}
trap cleanup EXIT
just canonical-init "$PSE_SURREAL_STATE"
just surreal start --state "$PSE_SURREAL_STATE"
# Run the selected managed study here, before leaving its allocation.
SH
```

The setup records the selected worker bytes and the exact reference allocation.
Receiver publication captures the prepared native environment. Keep the allocation
materializer on its separate `deployment-build` route: it must run before native
library paths are prepared. A fresh state initializes through its closed lifecycle
owner before ordinary startup; normal start cannot bypass required validation.
Start, managed work and drained stop share one reference allocation: the reference
store exceeds the ordinary resident functional lane. A plain light start does not
establish that allocation. The trap stops the owned service without deleting its data.
A compatible rebuilt worker is frozen into a new context receiver generation;
existing receivers retain their admitted bytes and storage remains running.
Existing legacy state requires quiescing and draining its workers, stopping the
server, then `just surreal reconfigure --execution-profile reference
--worker-executable "$PWD/target/debug/pse-worker"` before restarting. The
[local substrate guide](surreal-substrate.md) owns state selection and lifecycle;
ordinary canonical consumers do not require the managed study profile.

## Selected reproducible experiments

`just experiment` composes explicit investigations through the existing test-result,
resource and measurement owners. Ordinary regeneration remains the default; no test
archive or sandbox is created automatically.

```bash
just experiment archive --output build/my-archive --package pse-admission \
  --test authoring_load_package \
  --filter 'test(explicit_and_named_fixtures_decode_generated_rows_and_original_source_spans)' \
  --fixture tests/fixtures/packages/minimal_explicit \
  --fixture tests/fixtures/packages/minimal_named
just experiment inspect build/my-archive
just experiment replay build/my-archive --output build/my-replay
```

Archive creation prepares and executes the selected producer, then freezes its
nextest executable archive, exact selection, validated unit graph, configuration,
declared fixtures and runtime-library context. Replay remaps those fixtures into a
private workspace and produces a new invocation with historical producer attribution.
It never qualifies edited source. `inspect` reads sealed ownership/provenance without
launching binaries. Missing/changed inputs refuse; nextest is pinned with the other
protocol-sensitive Cargo tools. Native deployment/ABI/license/config/data closure
is not packaged: a selected native archive refuses before compilation/execution and
the investigation instead regenerates the exact revision through its native owner.

Investigation reports remain pinned under `scripts.test_resources`. Replay acquires
an exact receipt/digest reference before using its origin; releasing a pin alone does
not remove an active reference or bypass drain. Use that owner's `status`,
`release-reference --receipt PATH --digest SHA256`, `release` and `reclaim` operations
for an explicitly finished investigation. Archive/extracted binary detail is declared
disposable; retained summaries and provenance remain available after reclamation.

```bash
just experiment bisect --repo /path/to/repository --good FULL_SHA --bad FULL_SHA \
  --predicate authoring-load-package --output build/my-search --cleanup
just experiment compare --pairs /path/to/paired-samples.json --output build/comparison.json
```

The initial regression adapter uses the exact authored-package semantic test. It
freezes its controller outside the historical checkout, verifies both endpoint
predicates, and traverses an independently owned checkout with its own target directory.
Changed/missing predicate source is untestable; setup/infrastructure errors abort.
Bounded repeated mixed outcomes and skipped ambiguous intervals remain inconclusive.
Cleanup requires authenticated descendant drain and matching owned-directory identity.
Main's dirty source is never checked out or reset by the search.

Comparisons consume named samples from existing build/case reports, exact observed
input/start/cache/build/native conditions and explicit baseline/candidate source
inventory treatment. Declare at least three independent pairs and their execution
order. Missing context/pressure, uncontrolled cache, reused membership and noisy
samples remain inconclusive; output is descriptive and selects no automatic winner.
Criterion populations do not acquire fabricated invocation membership. See the
`scripts.regression_experiments.measurement` adapter for each supported producer's
condition projection; this finite route is separate from Plan52f's broader analysis.

Private directories plus `pse-env` remain the default tooling environment. For a
selected permission-isolation question, `just experiment tool-probe surrealql-validate
--input FILE --output build/my-probe` uses optional Bubblewrap 0.13.0 and the admitted
server CLI. It freezes selected inputs/runtime, disables networking, exposes only
owned writable output/private temporary state, and records parser, drain and cleanup
outcomes separately. It does not qualify database execution or a solver in a container.

The 3.3.4 server CLI validator's core parser disables experimental grammar; its
`validate` command does not apply `SURREAL_CAPS_ALLOW_EXPERIMENTAL`. For Surrealism
syntax, use the existing pinned `surrealdb-syn` parser through a selected tooling build:

```bash
scripts/pse-env --resource-class build -- .venv/bin/python -m scripts.arrow_validation \
  build -p xtask --bin xtask --no-default-features --features surrealql-tools --locked
just experiment tool-probe surrealql-validate --parser-binary target/debug/xtask \
  --input crates/pse-operations/src/generated/surreal.surql --output build/my-module-parse
```

That command reports the parser version compiled from the lockfile and the
`all-features` syntax profile. Parsing module/file syntax loads no module or file and
grants no runtime capability. A server CLI grammar rejection is retained as that
profile's diagnostic, rather than silently falling back to another parser. Podman
images remain unselected until a concrete different-userspace/dependency requirement.

## Bundles, logs and preflight

`just turn-end`, `just ready` and `just hygiene` are groups of the assessment runner
(`scripts/validation_scope.py::GROUPS`, `python3 -m scripts.validation --help`). Every step
runs, each logs to `build/assessment/<run>/<step>.log`, and the bundle ends with the steps
that failed and a non-zero status. The console stays short: each step's command and log path
(follow it with `tail -f`), one result line, and the last 20 log lines of a failed step;
pass `--live` (`just hygiene --live`) to stream everything. `latest-<group>` beside the runs
is a convenience; use the printed path.

`seed-conformance` and `modeling-conformance` first run
`scripts/preflight.py` for what they need (solver prefix, linked extension, manifest,
canonical store), so a missing prerequisite exits 125 with its fix before native setup;
`PSE_PREFLIGHT=off` proceeds without it (for example to collect partial evidence). A
pass does not predict code, fixture or scientific failures. Python test collection exits
125 with a `pse-env:` line when the installed native extension cannot load its libraries
outside the native environment.

`assessment` forwards its native flags unchanged and selects prerequisites from the
chosen gates. For example, `just assessment --group ready` needs no native solver
preparation. Read the printed run with `just result <run-path> --failures`; reader exit
zero means the checkpoint was readable, not that its assessment passed. The
[qualification guide](validation-assessment.md) owns result, reuse and completeness semantics.

### Detached watches and pressure observations

`just activity --watch --run ID --registry ROOT` watches one exact invocation;
`--attempt ID` selects an admission attempt and `--unit NAME` selects one unit.
The bounded response returns a cursor for the same selection (`--cursor TOKEN`,
`--max-wait 0..30`). Reconnecting reconciles current owner records. Cancellation
detaches the observer and leaves its producer running. Missing units, unavailable
owners and terminal records without protected detail remain explicit; disappearance
alone establishes neither success nor drain. The watch starts no daemon and promises
no replay of transitions its sources did not retain.

`just activity --pressure --unit NAME --json` adds host PSI and selected-cgroup
CPU/I/O/memory counters plus effective placement/ancestor restrictions. `--cgroup PATH`
selects an exact kernel group. Deltas require the same kernel/group identity and
interval; reset, missing permission and unavailable fields are reported, not zeros.
`storage-health --pressure` uses the same sampler. Idle CPU alone does not explain a
wait, and these observations apply no topology or concurrency tuning.

## Hooks and permissions

`scripts/agent-hooks.py` is the only hook script. `guard` (PreToolUse, both runtimes) refuses
edits to generated outputs, `build/`, `target/`, `external/`, `.git/`, the authoritative
design and accepted ADRs. The design exception is visible: an untracked `.design-edit` file
at the checkout root (it shows in `git status`; delete it to end the exception) or
`PSE_DESIGN_EDIT=1` in the runtime's launch environment. A committed `.design-edit` does not
count. `session-env` (Claude SessionStart) appends `pse-env --print` to `$CLAUDE_ENV_FILE`, so
later Bash commands have the checkout environment; it follows a worktree session's cwd and
never blocks a session. Hooks guard edit tools only; they are not a shell sandbox.

`.claude/settings.json` allows the tools and denies the protected paths with anchored
`Edit(/…)` rules (Claude Code consults only `Edit` and `Read` path rules). Claude Code ignores
`bypassPermissions` in project and local settings; launch with `--dangerously-skip-permissions`
or set it in user settings. Recipes that reach outside the working copy (`solver-image`,
`labels-sync`, `gh-setup`, `solver-pin-update`) keep their `just` confirmation; run one
deliberately with `just --yes <recipe>`.

## Runtime capabilities

### Canonical capability entry point

`just capabilities` launches this checkout's fixed, source-only FastMCP STDIO entry point without
workload admission or startup preparation. Each attachment owns a process. Follow the
[capability interface guide](capability-interface.md) for connection configuration, exact
browse/describe/select/reset, progressive instructions and same-endpoint read/action invocation
for hosts retaining a static callable inventory. Trusted-project Codex startup loads the checked-in
`.codex/config.toml` attachment through `just --quiet capabilities`, including from checkout
subdirectories; restart the client/MCP connection after configuration changes and inspect `/mcp`.
`just codex-session` supplies explicit selected-service/language overrides to the same attachment
and demand-driven internal resources. Native tool routes remain available;
effectful operation admission, source protection and terminal/drain truth retain their native owners.
Scientific reads require an explicitly prepared compatible extension/import environment, as the
interface guide describes; browsing performs no such preparation. Restart for relevant
serving code/instruction changes. This exception accepts no arbitrary executable or command.

Use `capability_readiness` for one binding's passive declarations/recorded preparation, and
`capability_readiness_check` only for explicitly selected live prerequisites. Import checks
observe the fixed checkout interpreter in the endpoint's inherited provider environment; they
do not prepare native providers. A prepared artifact can therefore coexist with an incompatible
import. The explicit `python_extension_dev_stage`/`python_extension_native_stage` tools consume
the existing environment, prepare, stubs, stubs-check and admit stages separately. Native stage
preparation happens in its child; it does not change the already-running endpoint's environment.
Reattach from the prepared native environment before native reads that require it. Foreground
progress is optional observation; authentic invocation readers retain result and drain authority.

### Focused command and source context

Use `just context QUERY --json` for selected recipe grammar; `--owner`/`--scope` read native
plan rows, and `--changed` reports the actual dirty delta. These projections execute no suggested
work. Source-only operations do not enumerate unrequested recipe definitions or start a database.
Readable output includes every section also returned by JSON.

- `just context --concept contract-section --identity 20.5 --json` selects the stable
  contract identity through the blueprint's section owners. `--concept decision --identity ADR-0222`
  reads authored decision identity/state; publication scope remains separate. `--concept plan-package`
  consumes native scope/status ownership. `--concept generated-output --identity python/pse/_native.pyi`
  follows exact declaration–generator–specific-output records. Unknown undeclared links stay unknown.
  `--topic`, `--role`, `--owner` and `--document` narrow owner-native populations;
  these source-only views start no database, language server or model.
- `just context --task docs/plans/PLAN.md#PACKAGE --path scripts/owned.py --json` composes the
  native status owner, exact selected content digests, dirty/deleted paths, bounded Git rationale
  and declared producer links. `--evidence HANDLE` adds explicit references without reading payloads.
- `--symbol TYPE` adds bounded text leads and producer-declared contributors. Declarations,
  generators, outputs, direction and registry/catalogue/API identities come from each actual
  producer. Source/generator byte digests describe current files separately from producer identities.
  Unknown declaration locations, stale generation premises and dynamic/string consumers remain
  unresolved; generated output is edited through its declared owner. `just codegen` and
  `just python-stubs` explicitly refresh their independently owned sidecars.
  Task paths match exact declared source/generator/output fields; `--symbol` remains separately
  labeled text discovery, rather than an inferred cross-language relationship.
- `just context 'source freshness' --search --source-root docs/authoritative_design --source-root crates/pse-operations/src --json`
  reads bounded current source bytes and lexical excerpts with native roles/section IDs.
  `--literal`, `--source-role`, `--offset` and `--limit` narrow selection. Current, reference and
  historical scope comes from the documentation owner; a better word match cannot promote old authority.
- `just context --library surrealdb-types --reference crates/pse-operations/src/canonical_codec.rs --json`
  reports exact declarations, locks, patches/local source and selected signature leads. `--feature`
  is intent. `--target`, `--cargo-metadata` and `--rustdoc` can bind supplied build observations
  to manifest/lock/source digests; wrong or unknown formats/contexts remain unsuitable/unknown.
  Reading a packet never resolves dependencies, installs tools or compiles a library.
- `--compare alternatives.json` consumes exactly two explicit named proposals, selected paths,
  removals and unknowns. It reports current owners/generators and conservative all-target package
  reach separately from proposed consequences; `--library` can attach selected research.
  There is no automatic design approval or score.

For historical Git leads use `--history-query TEXT --history-mode literal` (or `regex`) with
selected task/source paths, or `--line-history START,END:PATH`. Historical commits are rationale,
not current instructions. `just affected --json` retains its existing force-validation selection
and Python/native/generator/dynamic exclusions without executing tests.

`just context --retrospective --hours 24 --json` explicitly scans relevant local Codex activity
across creation dates, selected archives, explicit `--related-checkout` roots and parent/subagent
associations. Discovery and record reads are bounded and independently report incompleteness;
metadata alone cannot prove inactivity. `--save-history-checkpoint PATH` writes a disposable
metadata/hash/locator checkpoint; `--history-checkpoint PATH` resumes or refreshes it.
`--recall TEXT` returns exact historical locators. Write one selected locator to JSON and use
`--history-excerpt LOCATOR.json` for an explicit bounded raw read; changed sources are refused.
Prompts, arguments and raw outputs are absent from default/checkpoint projections. Quoted failures,
retries and polls are observations, not incident or productivity totals. The current conversation
retains authorization. The selected file adapter neither starts an App Server nor repairs state,
and never writes global memories.

## Explicit host controls

### Optional native telemetry

The process host resolves telemetry once; constructing a scientific Runtime does not install
providers. An explicit `OTEL_EXPORTER_OTLP_ENDPOINT` or the selected local service's public
configuration enables export. `OTEL_SDK_DISABLED=true` disables it. Signal-specific OTLP
endpoint, headers, protocol and timeout settings override their base settings; `none` disables
a selected signal. The supported transport is HTTP/protobuf. `just telemetry configuration`
shows effective selection, bounds and installed versions with credential values and endpoint
paths redacted. Unavailable export does not change native admission, cancellation or results.

Both SDK transports apply the resolved header selection at their public HTTP-client seam;
an SDK's ambient-header merge cannot add authentication afterward. Python's owned requests
session also disables ambient proxy/netrc authentication and uses uncompressed protobuf.
Exporter/transport diagnostics are classified and redacted before their stderr handlers;
application log payloads remain the application's responsibility. Exported `process.pid` is
an integer; embedded Python and Rust share the actual PID and service incarnation while
retaining their distinct SDK language/scope attributes.

The service state is `PSE_TELEMETRY_STATE`, otherwise `$XDG_STATE_HOME/pse/telemetry`, falling
back to `~/.local/state/pse/telemetry`. It belongs to the telemetry service, independently of
checkouts, `.store`, scientific data and build caches. Collector queues and OpenObserve data
have separate directories there. Private credentials stay in protected service material;
public configuration is not an authentication file. Setup creates an immutable configured
generation and refuses incompatible reuse rather than silently replacing it.

Owned storage profiles record the resolved telemetry state path at setup. Their child
environment and launch identity use that recorded selection even when a later shell selects
a different pipeline. Recording a path does not configure or start telemetry. A profile
created before this selection was recorded remains inspectable; explicitly readmit its
stopped supervisor with closed admission before starting it with current code. Quiesced and
validation-required targets retain their existing admission state through readmission. Readmission settles the
original launch before adding the selection and preserves an already recorded path.
Finite diagnostic capture can override tracing while metrics retain the recorded route.

Explicit operations use the same owner from any attachment:

```bash
just telemetry install
just telemetry setup
just telemetry start
just telemetry inspect
just telemetry health
just telemetry restart
just telemetry stop
just telemetry reconfigure
just telemetry recover-configuration
```

`reconfigure` validates a new immutable generation before draining the authenticated old
services. It retains backend data, queues, credentials and listener selection, then restores
the prior running/login selection on the replacement. Activation failure attempts rollback;
`recover-configuration` explicitly restores the authenticated prior generation recorded by
that operation, including interrupted selection publication. Neither command accepts an
arbitrary generation or rewrites sealed files. Existing producer processes retain their resolved
configuration until restarted; service replacement does not mutate their SDK settings.

Installation verifies the selected native archive and executable digests. Setup validates
Collector configuration before activation. Start, stop and restart authenticate the configured
unit and current process incarnation; a name or reused PID alone grants no control. Setup/start
do not enable login startup. `just telemetry enable` explicitly enables the selected user units;
`disable` removes that selection. The operator selected login startup at final cutover and
seven-day ordinary backend retention. The active [Plan52 checkpoint](../plans/52-agent-effectiveness-and-engineering-workflows.md#52p52q-execution-checkpoint--2026-10-24)
owns actual activation and qualification state. Disposable acceptance generations never enable
login startup and drain their owned services before returning to Plan54 measurements.

All commands accept `--state PATH`; setup alone accepts explicit listener `--ports` JSON for
isolated fixtures. Passive configuration/health reads install no SDK, start no service and run
no scientific probe. Health keeps current-process recording, verified service availability and
Collector cumulative receipt/export/queue counters separate from backend acceptance and known
record visibility. Rust's owned OpenTelemetry SDK 0.33.1 trace `BatchSpanProcessor` additionally
exposes a passive loss observation: its first full-queue drop diagnostic can be observed during
process life, while a cumulative dropped-span count is available only if the SDK emits its
shutdown diagnostic. That diagnostic precedes shutdown control and waiting; the independent
shutdown state records unselected, not attempted, attempted, succeeded or failed. Neither the
count nor successful shutdown proves queue drain, export or backend ingestion. Missing counts
are unknown, abrupt-exit buffered loss stays unknown, and queue capacity is not occupancy.
Python SDK queue/drop counters and other unsupported SDK observations remain unavailable.
A restart during a read invalidates attribution to the previous process. The local diagnostic
readers described in the [diagnostic guide](simulation-diagnostics.md) remain available when
telemetry fails.

`just telemetry lookup-trace --trace-id ID --start-us START --end-us END` reads one exact
nonzero trace ID in an explicit microsecond interval with a finite row limit. It requires a
current verified backend before acquiring credentials or making a request, and discards
responses if backend process, ownership or configuration changes during the read. An empty
selection does not establish nonexecution, loss, sampling or expiry. The read is live and
bounded, without an atomic population or continuation guarantee.

The ordinary environment boundary and fixed scientific worker launcher share passive native
host selection. The private settings carrier contains secrets and is excluded from shell
exports and environment explanations. Its shape comes from the Rust settings declaration through
the existing schema generator into `scripts/generated/telemetry_settings.py`. This private
source-only binding imports neither the scientific extension nor the SDK. The Python host resolves
precedence once and retains immutable encoded settings; decoded views cannot mutate that selection.
Rust validates the bounded resolved document before composition. Public inspection is a separate
redacted projection, never the private settings document. Embedded Python/Rust SDKs share the actual process
incarnation; real native children replace it with their own. Long-lived hosts drain providers
after native work settles. The private one-shot native lane performs no synchronous network
drain on scientific exit; its final batches, or all spans from a short solve, may be lost.
Scientific join/reap/deadline semantics remain unchanged. Detailed continuity coverage and
backend retention are separate from a healthy service or successful SDK shutdown.

Ordinary Rust trace export selects spans declaring the exact `pse.owner.stage` metadata field,
plus the exact `pse.canonical.request` span from `pse_operations::transport::client::owned`.
Name prefixes do not select arbitrary inner spans or tracing events. Explicit finite diagnostic
capture retains its own broader inner-event contract; this ordinary-export filter does not
change that capture's scope.

The repository Codex attachment forwards names of supported `OTEL_*`, `PSE_TELEMETRY_STATE`
and `XDG_STATE_HOME` settings through its checked-in `env_vars` allowlist. It stores no secret
values. Restart an existing attachment to change its process-owned telemetry selection.
The canonical FastMCP endpoint exposes configuration, health, typed queries, copied offline
investigations, explicit native capture escalation and native alert controls through the
[capability guide](capability-interface.md#typed-telemetry-investigations). Configuration and local
producer health belong to the calling MCP process; target process observations use the original
native diagnostic readers. Passive reads start no services.

Owner-issued operational metrics remain independent of span sampling. Native checkpoint6
retains supervisor admission/dispatch/join populations, overlapping frontier/basis acquisition,
retained reuse, shared producers and waiters, and the original source epoch. Cache populations
retain their own cache owner. Host instruments project actual Cargo records/artifact freshness,
child lifetime, assessment execution and engineering stages; a fresh artifact is not a compiler
cache hit, and selected/reused/unattempted cases are not newly executed cases. Host pressure
observations retain their original sampling interval and can expire rather than become zero.
The published metric declarations describe units, bounded attributes, buckets, reset and scope.
SDK cumulative start times do not replace native source epochs. Exact invocation identities and
scientific outcomes remain with their native records rather than series dimensions. Database
client observations and the selected server/MX metric routing do not establish causal
Rust→database→Surrealism parentage; [F02/I11](../plans/52f-evidence-analysis-and-trace-correlation.md#F02)
remains open.

Native OpenObserve OSS watches are explicit operations:

```bash
just telemetry watch-configure
just telemetry watch-read --start-us START --end-us END
just telemetry watch-disable
```

The fixed conditions cover Collector queue pressure/export failures and native MX poison/persistence
pressure. OpenObserve evaluates them and delivers locally to its own `pse_alerts` stream through
the narrowly configured loopback destination. Missing source schemas require a later explicit
configure call. `watch-read` retains the selected interval, evaluation history, source freshness,
configuration/incarnation and local delivery separately; disabled, missing-data, backlogged and
unsupported observations are not successful evaluations. Backend-down diagnosis remains with
`health`. No MCP polling scheduler, external notification or automatic remediation is installed.

Focused actual SDK→Collector→OpenObserve→canonical STDIO acceptance covers selected populations,
estimators, Python related examples and investigation readers; it does not establish all
cross-language propagation, assembled agent workflows or whole-series qualification. Those
remaining obligations retain their [52g](../plans/52g-agent-and-operator-workflows.md) and
[52h](../plans/52h-qualification-and-publication.md) owners.

### Project and evidence commands

`just deployment-build` builds the product allocation materializer before managed service
setup/readmission; ordinary compiler admission remains usable before that artifact exists.
`just project-build` builds the ordinary runtime project operator, while `just evidence-build`
builds the private development operator. `just canonical-init` uses the product
`pse-canonical-init` binary and its versioned description/initialization acknowledgment. Ordinary code generation can rebuild `xtask` without replacing these
artifacts. `just project --help` and `just evidence --help` forward literal arguments;
neither command installs or compiles a missing executable. The
[substrate guide](surreal-substrate.md#project-and-derived-evidence-operations)
describes initialization, roles, source cuts and retained draft intents.

Evidence service operations hold a kernel-backed service activity descriptor through
the native child and actual connection drain. They do not register a scientific
Runtime or native worker. The supervisor observes those activities before stopping,
rebuilding or disposing the service. Detached `evidence snapshot` remains available
without a service, native extension or built operator executable.

### Selected performance tooling on this workstation

The explicitly configured perf executable is `/usr/local/libexec/pse-arrow-perf/perf`:
the current kernel's real perf ELF, owned by root in a `root:paul` directory with mode
`0750` and **CAP_PERFMON only**. `/usr/bin/perf` is a dispatcher script and cannot carry
that file capability. Global `perf_event_paranoid` and `kptr_restrict` remain unchanged.
Kernel upgrades require deliberately refreshing this executable and checking its version
and digest; `--perf-binary` retains the selected tool identity in each action.

This host's tracefs does not support POSIX ACLs. Its top directory has read/traverse access
for group `paul`; selected scheduling event IDs/formats, trace headers and saved command names
are group-readable. Tracing controls are not writable. Exact rules are in
`/etc/tmpfiles.d/pse-arrow-perf.conf`; application with `systemd-tmpfiles --create` was tested,
but reboot persistence is not established. These are explicit host changes, never automatic
doctor/observer repairs. The [diagnostic guide](simulation-diagnostics.md#explicit-active-diagnostics)
describes action effects and coverage; access does not itself establish useful capture.

Optional tools are explicit installs: `just diagnostic-tools mcpls`, `just diagnostic-tools rr`
and `just diagnostic-python-tools`. Passive context, run and activity readers install nothing.
The separate cooperative taskdump Python environment is built with `just py-sync-taskdump`;
select its interpreter explicitly and keep it separate from ordinary/native correctness
extensions. The cargo-hack feature adapter selects that same `diagnostic-taskdump` profile
and existing taskdump flag configuration when a check explicitly selects the feature. Its
force-validation unit-graph probes use the same environment; ordinary combinations keep their
ordinary profile and flags. An explicitly conflicting profile is refused before Cargo runs.
Broad workstation sudo permission does not change these command effects or make
root the owner of ordinary builds.

The opt-in kernel and subprocess controls live in `scripts/tests/plan30_*_check.py`,
separate from `just setup-test` discovery. Their environment switches and assertions are
unchanged; setup discovery runs ordinary unit controls and requires zero skips. Use a new
receipt directory for each explicitly selected host control:

```bash
PSE_PLAN30_PLACEMENT=1 PSE_PLAN30_RECEIPTS=build/placement-control-new \
  scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.plan30_placement_check
PSE_PLAN30_PLACEMENT=1 PSE_PLAN30_RECEIPTS=build/capacity-control-new \
  scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.plan30_capacity_modes_check
PSE_PLAN30_C_ACTUAL=build/resource-control-new \
  scripts/pse-env --resource-class light -- .venv/bin/python -m unittest \
  scripts.tests.plan30_resources_actual_check.ActualResourceControls.test_actual_runner_ownership_and_retention
PSE_PLAN30_C_COLLECTION_ACTUAL=build/collection-control-new \
  scripts/pse-env --resource-class light -- .venv/bin/python -m unittest \
  scripts.tests.plan30_resources_actual_check.ActualResourceControls.test_nested_collection_preserves_parent_catalog_and_later_fixture_association
```

These commands exercise their named placement or filesystem ownership scope; they do not
qualify scientific execution. Historical receipts retain their original module names.
