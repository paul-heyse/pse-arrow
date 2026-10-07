# Agent environment

How commands, environments, scopes, worktrees, test selection, hooks and runtime
capabilities work for the coding agents (Claude Code and Codex) and for anyone at the
terminal. `AGENTS.md` keeps the always-loaded rules; this page holds the mechanics behind
them. Plan 29 introduced most of it.

## One environment boundary: `scripts/pse-env`

Every recipe line, script recipe and backtick runs through `scripts/pse-env` (the justfile's
`set shell` and `set script-interpreter`), and any command can use it directly:

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

Each command gets its own systemd user scope: `pse-cmd-<hex>.scope` for ordinary commands,
`pse-native-<hex>.scope` for native ones, inside `$PSE_SLICE` (default `pse.slice`; `none`
keeps the caller's slice). `PSE_MEMORY_MAX` (default `120G`; a size, a percentage, `infinity`
or `off`) is the scope's `MemoryMax`, with swap disabled, so a runaway process is OOM-killed
alone. A command already inside a native operation or managed-worker scope stays there.
Without a working user manager, commands run in place. Supervised canonical servers and
workers launch their own units in the same slice.

`just slice-install` installs `.config/systemd/pse.slice`: a CPU weight below interactive
applications and memory-pressure monitoring. The slice has no aggregate `MemoryMax` by
default — a stricter ancestor would silently defeat a raised command cap — and siblings are
not guaranteed to survive aggregate exhaustion. Opt in for one boot with
`systemctl --user set-property --runtime pse.slice MemoryMax=96G`; commands then report when
their own cap is bounded by it.

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
- `PSE_NEXTEST_ACTION=list just <recipe> ...` shows a recipe's selection without running it.
- Bare tools: `cargo t -p <pkg> -E 'package(<pkg>) & test(x)'` and `cargo c -p <pkg>` carry
  force-validation; plain `cargo nextest run`/`cargo check` do not.
- Python: `just py-unit` observes the installed extension (`doctor.py --extension-kind`) and
  uses the native environment for the linked build; `just py-test` and the native suites
  require a serving canonical server (`--store`) at `$PSE_SURREAL_STATE`.
- `just codegen-check` checks every generated output, including the native stub against the
  installed extension (rebuild it with `just py-sync` after a Rust API change).

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

**Semantic navigation, when text search is not enough.** rust-analyzer (Claude's LSP plugin,
or the Codex MCP server when enabled) answers callers, implementations and type resolution
across the workspace; `rust-analyzer.toml` gives it the force-validate feature and its own
target directory, so it never waits on an agent's build lock. The library catalog
(`library-catalog` MCP, `just library-catalog` to refresh) answers where a library item is
used. Context7 serves current library documentation (`.agents/roles/worker.md`).
