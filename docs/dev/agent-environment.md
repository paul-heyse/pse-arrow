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

## One environment boundary: `scripts/pse-env`

Workload recipe lines and scripts run through `scripts/pse-env` (the justfile's
`set shell` and `set script-interpreter`), and any workload command can use it directly.
The fixed read-only `just activity` observer runs directly, without workload admission or
compiler preparation, so unavailable admission remains diagnosable. Invoke it without an
outer `scripts/pse-env` wrapper; it accepts only its observer options. Failed observations
are reported as unavailable with partial independent observations retained.

Recipe arguments remain positional data. A composed recipe supports a native mode across
all stages or rejects it before effects: `just codegen` publishes the full target and accepts
no arguments; `just codegen-check` compares it without first refreshing outputs.

Any workload command can use the environment boundary directly:

```bash
scripts/pse-env -- <command>               # checkout environment + its own capped scope
scripts/pse-env --native -- <command>      # plus the linked solver environment
scripts/pse-env --native=solver -- <cmd>   # only the capabilities named
scripts/pse-env --store -- <command>       # first require a serving canonical server
scripts/pse-env --docs -- <command>        # provisioned docs-only Python environment
scripts/pse-env --explain [--native]       # what a command would get, and why
scripts/pse-env --explain --json --native -- cargo build --profile producer
                                         # structured selection before effects
scripts/pse-env --print                    # shell exports (direnv, the Claude hook)
```

The launcher itself uses the checkout `.venv/bin/python` when available, before
performing host admission or inspecting native module archives. System Python is only
a bootstrap fallback before that environment exists.

**Composition, in precedence order:** command options, then values the caller already has,
then `.envrc.local` (personal, gitignored) for anything unset, then repository defaults:
`PSE_PYTHON` from `.python-version`, `UV_PROJECT_ENVIRONMENT=.venv` with its `bin` first on
`PATH`, a local `/opt/pse-solvers` stack's tools, `PSE_SURREAL_STATE` (the supervisor's
default state), and the compiler/cache configuration of `scripts/build_environment.py`
(sccache wrapper, LLVM, jobs). Empty values pass through; composing twice changes nothing.

**Refused overrides.** Some values are corrected to keep compiler supervision and checkout
isolation: a `CARGO_TARGET_DIR` that names another checkout or this one's default `target/`
(choose one with `PSE_CARGO_TARGET_DIR`), `CLANG_PATH`/`LLVM_CONFIG_PATH`, the sccache
client-side and direct-mode pins. Native setup owns `MKL_CBWR`, `MKL_DYNAMIC` and
`OMP_CANCELLATION` (ADR-0108) and the prefixes it prepares. A replaced value is reported
(`pse-env: refused NAME=value: reason; using ...`): agent-chosen ones on every command, the rest
by `--explain`.

**Native defaults you can override.** Native commands default `OMP_NUM_THREADS`,
`OPENBLAS_NUM_THREADS` and `MKL_NUM_THREADS` to `1` and take `OMP_PROC_BIND`, `OMP_PLACES`
and `HWLOC_COMPONENTS` from the solver image. A caller value wins; `off` removes the variable,
and the choice survives nested native setup. Assessment gates pin their required single-thread
budget; case measurements record the thread settings used.

**Secrets.** `--print` never contains a `.envrc.local` value: local keys are rendered as
deferred reads that source the file at use. `--explain` never evaluates local shell
configuration, imports the extension, prepares a provider or opens admission metadata.
When `.envrc.local` exists, absent caller selections are marked unresolved; reported
defaults and tool paths are provisional where that file could affect them. An execution
owner can supply its already evaluated local mapping to `operation_description`, using
the same compiler and native-runtime selectors as execution. Only selected nonsecret
configuration is exposed. Provider reuse remains unknown until actual admission.
Use `doctor` or `activity` for live readiness/ownership observations. `--json` requires
`--explain`; build arguments are intent, separate from actual installer-child evidence.

For strict producer preparation, `just producer-deployment <new-output-directory>`
captures runtime/worker roles and lets Maturin select the Python Cargo child. The private
single-use observer records actual argv/cwd/SDK/compiler context and seals raw evidence
before permitting installation. It does not run a sibling Python compilation. Ordinary
`py-sync` remains ordinary editable installation. Historical inspection uses
`target/debug/xtask producer-history-verify <role-build-evidence.json.closure>` and reads
protected dep-info, inputs, graph and artifact association without live Cargo paths.
Historical success never grants current receiving freshness; final actual artifact/input
checks still refuse changed earlier roles. A target-local strict-operation guard is acquired before dependency setup and follows actual writer descendants. Executor families refresh once before role captures; exact same-operation compilation evidence can support later Fresh use. Unknown families, changed original outputs/premises and failed/incomplete operations refuse; retry uses a new output operation. This guard does not exclude arbitrary bare Cargo or replace host admission. Within this existing strict command, source/SDK/native qualification and final current-path association retain their conservative semantics. The maintainer withdrew Plan39’s additional all-role campaign, environment/source-completeness and compiler/launcher/installer phase-proof targets; no deferred work or revisit obligation is created. Ordinary development uses its functional build/install/test routes.

**Exit status** is the command's own (pse-env replaces itself with it). `125` means the
boundary failed — the line starting `pse-env:` says why and what to run; `126`/`127` mean the
command cannot be executed or was not found. Signal deaths report `128+N`.

## Documentation environment

`--docs` explicitly selects `.venv-docs`, provisioned by `just bootstrap-docs` with
`uv sync --locked --only-group docs --no-install-project`. It visibly replaces an inherited
product environment selector and rejects paths that alias the product environment before
effects. The normal `.venv`, its installed extension and ordinary dependency groups remain
independent. Documentation recipes and direct documentation consumers select this mode;
normal document reads never synchronize packages or initialize native capabilities.
The [documentation guide](documentation.md) owns metadata, scope-query and retirement usage.

## Placement and memory

Commands acquire a registered host allocation before their systemd user scope starts.
`.config/agent-capacity.toml` owns bounded memory partitions, shared build slots and physical-core
lanes; `scripts/host_admission.py` coordinates independent sessions under private XDG
state. The aggregate parent initially caps cooperating work at 160 GiB. Functional
work uses cores 8–15 (and their SMT siblings), timing uses cores 0–7. Kernel quotas
bound CPU time; inherited scheduler affinity enforces placement when the user manager
has no delegated cpuset controller.

A machine-local `export PSE_BUILD_MODE=unbounded` in `.envrc.local` selects ordinary
build/test defaults without an OS memory budget or startup memory-pressure guard.
Four build owners may overlap, each sharing all sixteen physical cores with a 1600%
CPU quota. Cargo's declared default is sixteen jobs; an explicit `CARGO_BUILD_JOBS`
remains a caller choice. Build allocations live under `psebuild.slice`, outside the
bounded `pse.slice` hierarchy. Their scopes omit `MemoryMax` and `MemorySwapMax` and
verify actual `memory.max=max` through every ancestor, including the cgroup root.
A finite ancestor refuses this mode instead of substituting a RAM-sized cap.

Resident functional/timing store counts are declared by `store_slots` beside each
lane's existing `store_gib` in `.config/agent-capacity.toml`. Each admitted store retains
its complete finite charge. An unknown prior owner still occupies its slot; admitting
another deployment never settles or frees the first deployment's effects.

Compile-heavy aggregate recipes (`clippy`, `hygiene`, `quality`, `governance` and
`features-powerset`) select the compile default at their entry. Their nested commands
retain that admitted owner, so an ordinary functional parent does not restrict a later
compiler leaf. Explicit caller resource selections retain precedence.

Run separate Clippy feature graphs sequentially: concurrent default/native Clippy runs
have formed a read/write artifact-lock upgrade cycle on the pinned Cargo implementation.
Independent ordinary builds and functional work may still overlap. Native provider
preparation can also wait for an active Cargo artifact writer; this does not lower the
declared compiler-job count.

Explicit `--resource-class`, `PSE_RESOURCE_CLASS` and finite `PSE_MEMORY_MAX` choices
keep their existing precedence; local mode changes recipe defaults only. Managed-primary,
reference and timing routes keep their selected bounded allocations and isolation.
Build owners conflict with live timing/reference owners. Registration, authentic
cgroup/handoff binding, cancellation and descendant drain apply equally to both modes;
an unobserved child still retains its owner. The ordinary canonical test observer uses
the uncapped build owner while retaining its finite scientific pool declaration. Managed
receiver/worker roles retain their finite role limits; an actual narrower scientific
child cap is never widened by the ordinary build policy.

Light commands use a 2 GiB slot, compilation a 24 GiB slot, ordinary functional work
one 40 GiB slot, and wide work both functional slots. Native scientific selections
with a 64 GiB observer pool use the named exclusive-observer partition. Pure Python
unit selections bypass that observer. Reference preserves its original 160 GiB
partition, 128 GiB pool and sixteen lanes. The runner builds and enumerates native
artifacts before entering its selected observer; the observer executes admitted binaries.
Rust test runners default `RUST_MIN_STACK` to the `[test-runner]` stack declaration
in `.config/agent-capacity.toml` (16 MiB), preserve an explicit positive finite byte
count, and record it in native provenance. This libtest thread setting is separate
from production native-job stack policy.

Cargo, Nextest and ordinary pytest default their widths to the allocation's
physical-core count, bounded by actual usable logical affinity and any finite
ancestor CPU quota. SMT siblings remain visible in affinity but
do not increase the physical-core CPU quota or these defaults. An explicit caller
environment or CLI width remains an override inside the kernel allocation. Nested
allocations recompute repository-supplied defaults for their own narrower placement.
Nextest's timing-sensitive exclusive controls reserve `num-test-threads`, the actual
configured scheduler width, so changing the allocation does not weaken exclusion.

The managed `scripts/sccache` wrapper gives the shared cache daemon its own finite
light-service allocation. Its immutable binary/configuration identity selects a private
socket; readiness verifies the actual process, unit invocation and socket peer. Compiler
children remain in the build allocation. Completed builds can therefore release capacity
while the cache stays resident. `PSE_SCCACHE_BINARY` identifies the installed executable;
the wrapper never administers the global cache server or deletes cache materials.

The pinned cache client can lazily start a server after a connection race. The wrapper
contains that fallback in its client process group and refuses the result if the admitted
service dies. This bounds surviving work; it does not establish that no transient fork
or cache effect occurred during the race. Explicit measurement servers keep their own
foreground lifecycle and endpoint.

`PSE_MEMORY_MAX` requests a positive finite cap. Requests above 40 GiB widen to both
functional slots; above 80 GiB requires exclusive heavy admission. A larger request
also raises its aggregate ancestor after checking physical capacity and nominal
non-agent headroom. Nested commands reuse the actual owner, and cannot silently
request more than an enclosing role or ancestor provides. Start that work in a
widened allocation instead. Nested light and compile jobs keep the enclosing CPU
lane rather than selecting a new functional lane. `off` and `infinity` are not finite
`PSE_MEMORY_MAX` values; use the explicit ordinary build mode for uncapped work.
Missing supervision and unverifiable ownership are refused in either mode. Memory ceilings and pressure observations do not reserve RAM
or exclude unrelated projects.

Capacity remains charged until every registered kernel group drains, including a
child whose launcher died. Resource/evidence pins survive capacity release. Only
explicitly owned, drained resident services may be parked for exclusive admission;
unknown services remain preserved and count as external pressure. The user-manager
unit installed by `just slice-install` supplies CPU weight and pressure monitoring;
the host owner materializes the selected bounded aggregate before finite-class execution.

Persistent storage is separately charged: a functional or timing caller acquires its
store allocation independently of the caller's execution slot. Exclusive modes charge
borrowed storage inside their aggregate envelope. Storage readiness verifies its own
unit/cgroup, memory cap, finite CPU quota and affinity; it does not require the caller's
receiver placement. The host ledger retains a resident store's charge while a qualified
automatic restart is pending. A restart rebinds that charged owner only after the prior
storage lifetime drains; unknown ownership remains a refusal.

Timing work selects both the timing host class and a dedicated timing store/context.
`setup --execution-profile timing` records that service class; using
`--resource-class timing` alone does not convert the functional store. Cross-lane
context registration is refused. The [local substrate guide](surreal-substrate.md)
shows dedicated-state setup and owns the stopped reconfiguration, readmission and
recovery routes. Recovery qualification uses its own administrative probe database,
so it also works when isolated tests have never created the base canonical database.

An ordinary native observer temporarily borrows its caller's control-memory limit. After
verified observer drain, it restores the exact prior finite limit only while the caller's
unit, cgroup, invocation and inode still match. A surviving or unobservable observer keeps
its charge and the control limit. Compilation and scientific execution still require their
respective admitted profiles; restoring a limit does not enlarge an allocation.

Foreground observer handoffs belong to their launching native operation. Cancellation
fences pending handoffs before stopping each bound observer whose unit, invocation,
cgroup and inode still match, then joins the direct payload and requires actual drain.
The operation's creator also settles foreground handoffs before retiring its ownership,
including when a child reports interruption without signalling that creator. Parent-owned
observer supervisors use separate sessions so a gate process-group kill cannot preempt
their drain handlers; nested borrowers do not retire their creator's handoffs.
A late binding records its scope and refuses payload execution after that fence.
An unbound launch remains charged until its authentic binding and drain can be observed;
unknown or replaced bound lifetimes retain their charges and pins and surface a cancellation
failure. Persistent primary services and independently launched durable workers keep
their separate lifetime owners; cancelling a foreground assessment does not stop them.

`just activity` (read-only; `--json`) lists the `pse-*` scopes and services with memory, age
and command, canonical servers and workers, Cargo processes in this checkout and the slice
limits. It is not a lock: a bare Cargo lock owner is reported as unknown.
Use repeated exact `--unit <name.service|name.scope>` options to inspect only selected
units, with `--json` for selection/completeness fields. Selected mode bypasses the global
unit listing; a disappeared selected unit is unavailable while healthy independent
observations remain. Errors include bounded manager stderr and endpoint context.
Observation never grants drain, signal or cleanup authority.

If an empty owned scope stays active and its user-manager journal reports inotify
`ENOSPC`, check `fs.inotify.max_user_watches` and `fs.inotify.max_user_instances`.
This is watch exhaustion, distinct from free disk space. On this workstation the
maintainer raised the runtime limits to 524288 watches and 1024 instances during
Plan37 qualification. Preserve exact drain requirements; increasing a cleanup timeout
cannot restore missing manager notifications. These runtime settings reset on reboot.

## Builds in one checkout

`PSE_TEST_RESOURCE_STATE=/absolute/private/directory` selects an isolated test-resource
metadata registry without changing shared host admission. A fresh empty private directory
initializes its own exact registry. Existing legacy metadata requires explicit fenced
migration/recovery; ordinary tests refuse it rather than silently importing or resetting
live ownership. This selector does not select or initialize a canonical database.

Cargo locks individual build units (`[unstable] fine-grain-locking` in `.cargo/config.toml`),
so concurrent check-type commands on independent crates do not wait for each other. Builds
and tests that produce artifacts still contend for the package cache and the artifact
directory; give a concurrent agent its own worktree for those.

## Worktrees

```bash
just worktree <name> [--ref REF] [--python | --native]
```

creates `~/pse-arrow-wt/<name>` on a new branch from REF (default `HEAD`) and prints the
commit and any uncommitted work it does *not* carry. It copies `.envrc.local`, allows direnv,
links the selected skills and reports `just doctor`; `--python` builds that checkout's own
venv and dev extension, `--native` the linked one. Claude-created worktrees receive
`.envrc.local` through `.worktreeinclude`. Each checkout builds into its own `target/`; never
set `CARGO_TARGET_DIR` (sccache keys compilations on `CARGO_*` variables, and Cargo keys
workspace units by relative path and mtime, so two checkouts sharing a target reuse each
other's artifacts — ADR-0122). Remove a worktree with `git worktree remove <path>` once merged.

## Choosing tests

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
select it directly. Native qualification builds it before deployment capture; observers
then refuse a mismatched imported path, binary digest or actual Rust feature report rather
than rebuilding beneath a receipt. Environment text never certifies compiled features.

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
scripts/pse-env --native -- .venv/bin/python -m scripts.arrow_validation build -p xtask --bin pse-worker --locked --features native-solvers
just surreal setup --execution-profile plan28-reference \
  --worker-executable "$PWD/target/debug/pse-worker"
scripts/pse-env --resource-class reference -- bash -euo pipefail -s <<'SH'
cleanup() {
  just surreal quiesce --state "$PSE_SURREAL_STATE"
  just surreal stop --drained --state "$PSE_SURREAL_STATE"
}
trap cleanup EXIT
just surreal start --state "$PSE_SURREAL_STATE"
just canonical-init "$PSE_SURREAL_STATE"
# Run the selected managed study here, before leaving its allocation.
SH
```

The setup records the selected worker bytes and the exact reference allocation.
Start, managed work and drained stop share one reference allocation: the reference
store exceeds the ordinary resident functional lane. A plain light start does not
establish that allocation. The trap stops the owned service without deleting its data.
A compatible rebuilt worker is frozen into a new context receiver generation;
existing receivers retain their admitted bytes and storage remains running.
Existing legacy state requires quiescing and draining its workers, stopping the
server, then `just surreal reconfigure --execution-profile plan28-reference
--worker-executable "$PWD/target/debug/pse-worker"` before restarting. The
[local substrate guide](surreal-substrate.md) owns state selection and lifecycle;
ordinary canonical consumers do not require the managed study profile.

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

### Focused command and source context

Use `just context QUERY --json` for a bounded projection of actual Just declarations and
source owners; `--owner`, `--scope`, `--changed`, `--offset` and `--limit` narrow it.
`just context --symbol TYPE --json` adds catalogue/generator, Rust/PyO3, generated stub,
facade and test leads. These are explicit source matches, not invented semantic edges
across language boundaries. Refine them using the appropriate language server.
`just affected --json` previews the exact selected force-validation command and its
Python/native/generator/dynamic exclusions without executing a test.

`just context --retrospective --hours 24 --json` explicitly inspects local Codex records
for this checkout and the selected time window. It reads named date partitions with a
ten-second/128 MiB bound and a one-MiB record ceiling, deduplicates inherited records and
returns tool/error-class counts and source paths. It does not copy prompts, arguments,
environments or raw outputs into a new index. An error-pattern count is an investigation
lead, not a number of failed tests or a causal diagnosis. Partial/oversized input remains
explicit. This work never runs during startup or ordinary activity polling.

What a session loads costs context and processes, so optional capabilities are opt-in:

- **Claude Code** — the project disables the synced plugins and claude.ai connectors unrelated
  to this repository (superpowers' workflow injection competes with the process skills) and
  the pyright LSP (the repository's checker is pyrefly). Re-enable one for a session with
  `--settings` or `/plugin`.
- **Codex** — `.codex/config.toml` raises `project_doc_max_bytes` so AGENTS.md is never
  truncated, sets static non-secret command defaults, and disables inherited machine
  database/navigation registrations until `just codex-session` selects this checkout.
  Codex runs `bash -lc` without direnv; prefix commands with `scripts/pse-env --`.

**Native SurrealDB inspection.** Start a selected session with `just codex-session
[--state STATE] -- CODEX_ARGS`. It attaches to the existing owned server's verified
HTTP MCP endpoint; application SDK traffic continues to use gRPC. The supervisor's fixed
`mcp-headers` action verifies the explicit owned state, endpoint and frozen selection
fence, then supplies the private selection VIEWER credential through Codex's header helper.
It uses ordinary light admission, no compiler setup and one eight-second deadline.
Failure does not start/reconcile the service or initialize a database. Never print or save
the helper's Authorization output yourself.

The exposed native tools are `info`, `list` and `query`; the database VIEWER authority
enforces read access. Select namespace and database explicitly in every inspection call
and establish existence with `list` before reading schema/indexes. Missing `pse/canonical`
is an error to report, not permission to initialize it. Retained/quiesced contexts can be
read independently of scientific write admission. Codex caches headers; reconnect or
restart after state, port, service identity or credential changes. The frozen selection
refuses replacement even at the same endpoint. Do not launch `surreal mcp` on the persistent
files: that opens another datastore. The disabled repository registration deliberately has
no usable service selection; merely enabling it is not a supported attachment route.

For an explicitly selected session, `just codex-session [--state STATE] -- CODEX_ARGS`
observes the existing service and supplies the validated endpoint and pinned checkout header
helper to Codex. This fixed launcher bypasses workload admission and starts no database.
Rust/Python navigation and database inspection are requested by default; `--without-rust`,
`--without-python` and `--without-database` opt out. An unavailable optional capability is
reported and disabled; `--require-rust`, `--require-python` or `--require-database` makes it a prerequisite. `--explain --json` reports
nonsecret selection/readiness, never attachment or credentials. Explicit trailing Codex overrides
remain last and can change the preset; actual attachment is established by client interaction.

For scientific meaning use the typed APIs and exact identifiers described in
[the substrate guide](surreal-substrate.md#typed-scientific-inspection), rather than
interpreting raw rows as a qualified scientific result.

**Semantic navigation, when text search is not enough.** The selected Codex session uses
pinned mcpls 0.8.0, a maintained LSP bridge, with the pinned rustup rust-analyzer and checkout
`.venv/bin/pyrefly lsp`. Install the bridge explicitly with `just diagnostic-tools mcpls`;
attachment never installs it. `python -m scripts.semantic_navigation describe --language python`
reports availability without starting an LSP. The explicit temporary configuration selects the
checkout and `.pyi` coverage. Rust configuration derives from `rust-analyzer.toml`; automatic
checks are disabled for navigation, while explicit correctness recipes remain unchanged.
The bridge clears ambient environment by design. Its Rust declaration explicitly forwards
only the already selected compiler/native/admission keys through the environment owner's
allowlist; credentials and licence values are excluded. This keeps compiler-cache startup
under its managed owner instead of falling back to a machine-global wrapper. A navigation
client's shutdown must drain its actual children, independently of a successful semantic reply.
The bridge adds implementation and call-hierarchy queries absent from rust-analyzer-mcp 0.4.0.
Readiness and missing capabilities belong to the real request, not an executable-presence check.
mcpls positions are **one-based**; rust-analyzer-mcp's direct tool positions are zero-based.
Neither server crosses the PyO3 boundary semantically. `just context --symbol TYPE --json`
returns bounded catalogue, generator, Rust, stub, facade and test text leads with explicit
coverage and truncation; inspect each language's native references to refine those leads.

rust-analyzer (Claude's LSP plugin,
or the Codex MCP server when enabled) answers callers, implementations and type resolution
across the workspace; `rust-analyzer.toml` gives it the force-validate feature and its own
target directory, so it never waits on an agent's build lock. Context7 serves current library
documentation; optional docs/libraries/ overviews orient capability research. The former library
catalog MCP, refresh/read helpers and resolver are removed. Native-solver-libraries remains the
only selected shared skill; existing setup/readiness/worktree routes synchronize that link.
Other central skills are inactive historical references. Retained reports and capability maps
keep their original evidence scope; they are not live catalog lookup services.

## Explicit host controls

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
extensions. Broad workstation sudo permission does not change these command effects or make
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
