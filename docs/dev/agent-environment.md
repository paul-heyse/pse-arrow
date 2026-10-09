# Agent environment

How commands, environments, scopes, worktrees, test selection, hooks and runtime
capabilities work for the coding agents (Claude Code and Codex) and for anyone at the
terminal. `AGENTS.md` keeps the always-loaded rules; this page holds the mechanics behind
them. Plan 29 introduced most of it.

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
scripts/pse-env --explain [--native]       # what a command would get, and why
scripts/pse-env --print                    # shell exports (direnv, the Claude hook)
```

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
and the choice survives nested native setup. Assessment gates and case measurements pin the
single-thread budget their receipts require.

**Secrets.** `--print` never contains a `.envrc.local` value: local keys are rendered as
deferred reads that source the file at use. `--explain` names where a value came from
without printing it.

**Exit status** is the command's own (pse-env replaces itself with it). `125` means the
boundary failed — the line starting `pse-env:` says why and what to run; `126`/`127` mean the
command cannot be executed or was not found. Signal deaths report `128+N`.

## Placement and memory

Commands acquire a finite host allocation before their systemd user scope starts.
`.config/agent-capacity.toml` owns the machine's memory partitions and physical-core
lanes; `scripts/host_admission.py` coordinates independent sessions under private XDG
state. The aggregate parent initially caps cooperating work at 160 GiB. Functional
work uses cores 8–15 (and their SMT siblings), timing uses cores 0–7. Kernel quotas
bound CPU time; inherited scheduler affinity enforces placement when the user manager
has no delegated cpuset controller.

Light commands use a 2 GiB slot, compilation a 24 GiB slot, ordinary functional work
one 40 GiB slot, and wide work both functional slots. Native scientific selections
with a 64 GiB observer pool use the named exclusive-observer partition. Pure Python
unit selections bypass that observer. Reference preserves its original 160 GiB
partition, 128 GiB pool and sixteen lanes. The runner builds and enumerates native
artifacts before entering a capped observer; the observer executes admitted binaries.
Rust test runners default `RUST_MIN_STACK` to the `[test-runner]` stack declaration
in `.config/agent-capacity.toml` (16 MiB), preserve an explicit positive finite byte
count, and record it in native provenance. This libtest thread setting is separate
from production native-job stack policy.

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
lane rather than selecting a new functional lane. `off`, `infinity`, missing supervision and unverifiable
ownership are refused. Memory ceilings and pressure observations do not reserve RAM
or exclude unrelated projects.

Capacity remains charged until every registered kernel group drains, including a
child whose launcher died. Resource/evidence pins survive capacity release. Only
explicitly owned, drained resident services may be parked for exclusive admission;
unknown services remain preserved and count as external pressure. The user-manager
unit installed by `just slice-install` supplies CPU weight and pressure monitoring;
the host owner materializes its finite aggregate cap before execution.

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

`just activity` (read-only; `--json`) lists the `pse-*` scopes and services with memory, age
and command, canonical servers and workers, Cargo processes in this checkout and the slice
limits. It is not a lock: a bare Cargo lock owner is reported as unknown.

## Builds in one checkout

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
- Bare tools: `cargo t -p <pkg> -E 'package(<pkg>) & test(x)'` and `cargo c -p <pkg>` carry
  force-validation; plain `cargo nextest run`/`cargo check` do not.
- Python: `just py-unit` observes the installed extension (`doctor.py --extension-kind`) and
  uses the native environment for the linked build; `just py-test` and the native suites
  require a serving canonical server (`--store`) at `$PSE_SURREAL_STATE`.
  `just py-test` defaults to unit and component tests with four pytest workers;
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
scripts/pse-env --native -- cargo build -p xtask --bin pse-worker --locked --features native-solvers
just surreal setup --interpretation pse.substrate.v2 --execution-profile plan28-reference \
  --worker-executable "$PWD/target/debug/pse-worker"
just surreal start
just canonical-init "$PSE_SURREAL_STATE"
```

The setup records the selected worker bytes and the exact reference allocation.
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

What a session loads costs context and processes, so optional capabilities are opt-in:

- **Claude Code** — the project disables the synced plugins and claude.ai connectors unrelated
  to this repository (superpowers' workflow injection competes with the process skills) and
  the pyright LSP (the repository's checker is pyrefly). Re-enable one for a session with
  `--settings` or `/plugin`.
- **Codex** — `.codex/config.toml` raises `project_doc_max_bytes` so AGENTS.md is never
  truncated, sets static non-secret command defaults, and makes the `library-catalog` and
  `rust-analyzer` MCP servers opt-in: `codex -c mcp_servers.library-catalog.enabled=true`.
  Codex 0.160 role files cannot change MCP servers, so there is no subagent-only setting.
  Codex runs `bash -lc` without direnv; prefix commands with `scripts/pse-env --`.

**Native SurrealDB inspection.** This machine's `pse-surreal` registration is disabled
by default. Start a fresh session with `codex -c mcp_servers.pse-surreal.enabled=true`.
It attaches to the existing server's `http://127.0.0.1:18240/mcp`; application SDK traffic
continues to use WebSocket. The supervisor's fixed `mcp-headers` action verifies the
explicit owned state and endpoint and supplies the private selection VIEWER credential
through Codex's header helper. It uses ordinary light admission, no compiler setup and
one eight-second deadline. Failure does not start/reconcile the service or initialize a
database. Never print or save the helper's Authorization output yourself.

The exposed native tools are `info`, `list` and `query`; the database VIEWER authority
enforces read access. Select namespace and database explicitly in every inspection call
and establish existence with `list` before reading schema/indexes. Missing `pse/canonical`
is an error to report, not permission to initialize it. Retained/quiesced contexts can be
read independently of scientific write admission. Codex caches headers; reconnect or
restart after state, port or credential changes and keep registration/helper endpoints
matched. Do not launch `surreal mcp` on the persistent files: that opens another datastore.

For scientific meaning use the typed APIs and exact identifiers described in
[the substrate guide](surreal-substrate.md#typed-scientific-inspection), rather than
interpreting raw rows as a qualified scientific result.

**Semantic navigation, when text search is not enough.** rust-analyzer (Claude's LSP plugin,
or the Codex MCP server when enabled) answers callers, implementations and type resolution
across the workspace; `rust-analyzer.toml` gives it the force-validate feature and its own
target directory, so it never waits on an agent's build lock. The library catalog
(`library-catalog` MCP, `just library-catalog` to refresh) answers where a library item is
used. Context7 serves current library documentation (`.agents/roles/worker.md`).

## Explicit host controls

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
