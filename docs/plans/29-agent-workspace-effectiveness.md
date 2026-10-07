---
title: Agent workspace effectiveness
status: in-progress
date: 2026-10-07
adrs: []
review_sources: [docs/design_review/reviews/design_review_agent-workspace-effectiveness_2026-10-07.md]
scenario_sources: []
---

# 29: Agent workspace effectiveness

## Context

The maintainer asked for an assessment of how the environment, command surface, tooling and
agent configuration could make highly capable agents (Claude Code and Codex, weighted equally)
more *effective*:

- less up-front thinking about how to phrase a command;
- fewer environment-shaped complications;
- related actions reachable through one parameterised command;
- legible feedback;
- no tooling that boxes in an agent that already knows what it wants.

Machine-specific configuration is welcome; this is a single-operator project on one workstation.

This plan records the assessment's findings and the changes it proposes. Plan 28 keeps
production-code speed (the 2026-10-07 production efficiency review, 28e and 28h). Plan 29 owns the
harness side:
- the environment;
- recipes;
- runner configuration;
- test scheduling;
- agent-runtime configuration;
- process isolation.

Where a finding belongs to Plan 28, it is routed there.

An independent [design review](../design_review/reviews/design_review_agent-workspace-effectiveness_2026-10-07.md)
(Revise) kept the main direction but corrected how the remedies compose: precedence and lifetime, run
ownership, truthful scope, and failure and override semantics. Its findings F01–F07 and additional
capabilities A1–A6 are integrated below. On 2026-10-07 the maintainer confirmed the Decisions and the
rule changes RC01–RC06, to the extent they serve the functional outcomes above.
Implementation has not started.

**Method.** All evidence is local. The raw artifacts are in gitignored
`build/agent-effectiveness/`; no transcript content entered the repository.

| Workstream | What it examined |
|---|---|
| W1 transcript mining | 129,664 shell commands from 561 Claude and Codex sessions and threads (2026-09-14 to 10-07). Commands are scoped by working directory, stratified by runtime × main/subagent, and tagged by configuration period. P4–P5 (10-01 onward) is the current configuration: 46,055 commands, of which only 677 are Claude's, so runtime-specific rates and benefits are uneven. The classifier was hand-checked on 126 commands; it had a 4% error rate, and those errors were fixed. |
| W2 command surface | `just --dump --dump-format json` (212 recipes), cross-referenced with W1 usage |
| W3 instruction and configuration load | Always-loaded instruction volume per runtime, duplication, stale references |
| W4 capability research | Version-specific documentation and source for Claude Code 2.1.293, Codex 0.160/0.161, nextest 0.9.146, just 1.58, nightly Cargo and systemd 255 |
| W5 warm probes | Sequential, memory-capped timings while another project was compiling (load 10–23), plus the existing JUnit from the interrupted 2026-10-07 native gate |
| W6 cold-start drills | Four tasks, each given to `claude -p --permission-mode plan` and `codex exec -s read-only` |
| W7 operational isolation | Kernel and oomd journal, cgroup placement, memory pressure |

**Standing preferences carried into every proposal:**
- Tooling changes stay light; each new capability is exercised once.
- No new alignment lints.
- Judgment over evidence ceremony.
- No `df` guards.
- Caps without fd-inheriting locks.
- Low parallel test load.

**Rubric.** Every proposal was judged against these properties:
1. The bare tool path is first-class; recipes are shortcuts, not gates.
2. Wrappers are thin and transparent, and pass arguments through.
3. Selection is composable and reports what it covers.
4. Default output is concise, full logs go to unique per-run paths, and structured results are available on request.
5. Infrastructure failures are distinguishable from code failures.
6. Behaviour is the same under any inherited environment.
7. Commands are safe to run concurrently, and contention is reported honestly.
8. Every guard or cap can be overridden from inside a session, and effective limits are visible.
9. Tools are self-describing from source.
10. Tools offer capabilities, not mandated procedures.
11. Always-loaded text pays its way.
12. Both runtimes get the same capability, or the gap is stated.

## Decisions

Confirmed by the maintainer on 2026-10-07. None alters D1–D14 or the decision of an accepted ADR.

1. **One command boundary composes existing owners.** `scripts/pse-env` is a thin adapter over
   `build_environment.configure` (compiler and cache), `.envrc.local`, the selected venv, and, for
   native commands only, `native_operation.Operation` (admission, generation protection,
   descendant ownership).
   - **Precedence**, defined once and in this order:
     1. command options;
     2. explicitly supported caller values;
     3. local defaults;
     4. repository defaults.

     Empty and `off` values count as real values. A refused override, such as a checkout-isolation
     input that `build_environment` must correct, is reported rather than silently replaced.
   - **`--print`** emits only ordinary exports: licence, venv and compiler/cache settings. It never
     prepares native assets, creates an `Operation` or starts a scope, so a session-start export
     cannot become persistent native authority.
   - **Command execution** (`pse-env [--native[=caps]] -- <cmd>`) runs the command in the
     resource scope chosen in D4 and, for native capabilities, under the existing `Operation`.
     Status and signals propagate.
   - **`--explain`** shows the selected interpreter, compiler/cache defaults, the installed extension
     kind (from `doctor.check_extension`), the requested capabilities and the effective scope limits.
     It prepares nothing, prints no secret values, and labels observations as observations rather
     than readiness.
   - **Claude's `SessionStart` hook** (RC02) appends `--print` output to `$CLAUDE_ENV_FILE`. That
     reaches Claude's Bash commands only, not hooks, MCP or LSP processes; those processes use their
     own launch configuration or `pse-env` itself.
   - **Codex** uses non-secret defaults in `.codex/config.toml` `shell_environment_policy.set`, and
     agents prefix bare commands with `scripts/pse-env --`.

   This decision and D4 are designed together before any recipe is rewritten.
2. **Bare tools are first-class, and the docs say exactly what each layer supplies** (RC01).
   - rust-analyzer reads the force-validate feature from `rust-analyzer.toml`.
   - Cargo aliases (`cargo c`, `cargo t`) carry `--locked --features pse-relations/force-validate`.
   - Plain `cargo check` and `cargo nextest run` read no feature configuration, so bare commands still
     pass the flag explicitly. Correctness tests keep explicit force-validation.

   AGENTS.md, `.claude/rules/rust.md` and the justfile header stop prescribing recipes as the only
   path.
3. **AGENTS.md headroom first, compact core second.**
   - `project_doc_max_bytes = 65536` in `.codex/config.toml` removes the truncation risk.
   - A ≤ 20 KB always-loaded core is a compactness target, not evidence of effectiveness. It keeps the
     task routes, the consequential constraints, the instructions to load relevant owners, and short
     deliberate overlap where an isolated worker needs it. Detailed mechanics move behind links into
     `docs/dev/agent-environment.md`.
4. **Placement and protection are separate.** A dedicated `pse-agents` systemd user slice provides
   CPU weighting below the editor and takes agent work out of the editor's scope.
   - **Failure boundary:** the per-command scope (`memory-cap.sh`, `PSE_MEMORY_MAX`).
   - **Aggregate cap:** the slice has no aggregate `MemoryMax` by default, so a child's raised cap is
     never silently defeated by an ancestor. One can be opted into, and effective limits are
     reported (RC06).
   - **No overclaim:** CPU weighting and low parallel defaults reduce contention. The plan does not
     claim siblings survive aggregate exhaustion.
   - **Scope naming:** names stay compatible with `native_operation.scope_owner`
     (`pse-native-<hex32>.scope`).
5. **Canonical test scheduling follows each test's effect** (RC04).
   - **Database-only consumers** (`canonical_fixture_store` creates a UUID database and owns its
     cleanup) share one bounded server instead of serialising.
   - **Server-lifecycle and `ManagedWorkerCase` journeys** keep exclusive ownership of their server
     resources, provided per run: a run-owned namespace plus slot, with unique state, ports and units.
     Slot numbers alone are not enough, because separate nextest runs reuse them.
   - **Reuse:** the existing supervisor and immutable server binary are reused, and
     override-selected external state is never auto-destroyed.
   - **Coordination:** with 28e, which owns fixture qualification.
6. **Optional capabilities are opt-in through mechanisms the installed versions support** (RC05).
   - **Claude:** workflow injections that compete with the repository's process skills (superpowers)
     and unrelated connectors and synced plugins are disabled for this project. The pyright LSP is
     disabled. A pyrefly LSP binding is optional.
   - **Codex:** library-catalog and rust-analyzer MCP default to `enabled = false` at project level,
     with `-c mcp_servers.<name>.enabled=true` per session. Codex 0.160 role files cannot change MCP
     servers (`AgentRoleOverrides` has no such field), so a subagent-specific default is a stated
     gap, not a claim.

### Confirmed rule changes

Recorded here because the maintainer confirmed them for these functional outcomes. Under AGENTS.md
these are tooling changes, so the owning packets update the named rule text and no ADR is required.
ADR-0161's decision to remove end-of-turn automation is unchanged.

| ID | Current rule / location | Change | Packet |
|---|---|---|---|
| RC01 | AGENTS.md *Start here* ("If no recipe fits, say so…"), justfile header, `.claude/rules/rust.md` bare-test guidance | Bare tools are first-class. The docs say exactly which configuration bare tools share and which flags only aliases and recipes add. | P2, P8 |
| RC02 | AGENTS.md *Agent runtimes*: PreToolUse is "the only hook" | Allow one narrow, environment-only Claude `SessionStart` hook. End-of-turn automation stays removed (ADR-0161). | P2 |
| RC03 | AGENTS.md design-edit escape; `agent-hooks.py` reads only `PSE_DESIGN_EDIT` | Add a visible, removable checkout override: an untracked `.design-edit` file that `git status` shows. The env var and session authorisation are kept. | P4 |
| RC04 | `.config/nextest.toml` broad `store` serialisation | Schedule by effect, with per-run, per-server ownership for exclusive journeys (D5) | P5 |
| RC05 | Current MCP/plugin/LSP activation in project and user configuration | Selected optional capabilities become opt-in at project or session level, using supported mechanisms, with deliberate enabling paths kept (D6) | P8 |
| RC06 | Native thread defaults that override callers; resource placement and caps | Supported defaults become overridable, and command placement is separated from aggregate protection (D1, D4) | P2, P4, P9 |

## Architectural drivers and scenarios

The scenarios are agent tasks, taken from W1 frequencies and the W6 drills:

| ID | Scenario |
|---|---|
| S1 | Edit → compile → targeted unit test in one crate. This is the dominant loop: `unit-native-package` 486, `unit-package` 476 and `check-package` 333 invocations in P4–P5. |
| S2 | Select the tests a change affects, across crates and native feature gates |
| S3 | Regenerate generated outputs and confirm there is no drift |
| S4 | Onboard a parallel worktree for a second agent |
| S5 | A full native/Python qualification run |
| S6 | Diagnose a failure as environment-shaped vs code-shaped |
| S7 | Several agents building or testing in one checkout |

The consequential variations are:
- selecting another package or feature set;
- changing which kind of Python extension is installed;
- starting a second test run;
- creating a worktree;
- enabling an optional runtime capability;
- interrupting a command.

## Findings

Evidence labels follow design principles §D. "Agent-h" means summed agent wall time; subagents overlap,
so these sums exceed elapsed time. W1's counts are historical workload evidence, not measurements of
the proposed remedies.

### Execution reliability and environment (S1, S6)

- **AE-01 Agent shells lack the repository environment** (*Measured*).
  - A Codex login shell (`shell_environment_policy.inherit = "core"`) has no `SYMBOLICA_LICENSE`, no `.venv/bin` and no native-library paths. Line recipes get compiler settings from `build-shell.sh` but neither the venv nor `.envrc.local`. `.envrc.local` is loaded only by direnv, by `native_exec.sh` and by one recipe.
  - W1 P4–P5 counts:
    - workarounds: 458 `direnv exec .`, 164 `source .envrc.local`, 223 ad hoc `native_exec.sh` and 149 ad hoc `memory-cap.sh` wrappings;
    - failures: 85 environment failures (2.6 agent-h), including 32 Symbolica-unlicensed banners, 17 of them SIGABRT in `unit-package`.
- **AE-02 `py-unit` is not routed by the installed extension kind** (*Measured*).
  - With the native (MKL-linked) extension installed, `just py-unit` fails at conftest import with `ImportError: libmkl_gnu_thread.so.3`, while `just py-unit-native` works.
  - `doctor.py` (`EXTENSION_PROBE`, `check_extension`) already observes the extension's linking; `py-unit` just does not use that observation before conftest imports the extension.
- **AE-03 Hand-set knobs the recipes should default** (*Measured*, W1 P4–P5).
  - `NEXTEST_TEST_THREADS` was set by hand 355 times, and `PSE_SURREAL_STATE` 281 times (P5).
  - A 49-occurrence compiler-environment cluster rebuilds the producer-identity environment by hand.
  - `XDG_RUNTIME_DIR`/`DBUS_SESSION_BUS_ADDRESS` were set by hand for the memory cap. `memory-cap.sh` now supplies both, so that one is historical.
- **AE-04 The design-edit escape is unreachable from inside a session** (*Interface-checked*,
  *Measured*). `scripts/agent-hooks.py` reads `PSE_DESIGN_EDIT` from the hook's own process
  environment. Neither a Bash `export` nor Codex `apply_patch` can set it. W1 counts 90
  `PSE_DESIGN_EDIT=1 python3 - <<PY` rewrites, which no hook sees, so the guard is bypassed
  rather than used.
- **AE-05 The first native setup in a session costs about 34 s** (*Measured*: `native_exec.sh true`
  took 33.96 s cold, then 2.1 s for all capabilities and 0.2–1.3 s for one). Routed to 28h, which
  owns `native_cache.prepare` receipt verification.
- **AE-24 Native setup replaces explicit caller thread values** (*Interface-checked*, *Tested* by the
  review's coordinator probe). `native_operation.environment` sets `OMP_NUM_THREADS`,
  `OPENBLAS_NUM_THREADS` and `MKL_NUM_THREADS` to `1` unconditionally, even when no capability is
  requested. Explicit values 3, 4 and 5 were replaced.

### Latency and concurrency (S1, S5, S7)

- **AE-06 Build-lock contention in the shared checkout is the most frequent current friction**
  (*Measured*, W1 P4–P5).
  - 145 build/test commands showed `Blocking waiting for file lock`: 9.2% of visible build/test outputs, up from 2.8% in P0–P3. Agents also ran 986 `ps`/`pgrep` checks for other builds.
  - 4.7 agent-h is the total wall time of the commands that showed lock messages, including their compilation and test execution. It is not separately measured waiting time.
  - Nightly `-Z fine-grain-locking` narrows a `check`'s lock from the whole build directory to the unit being built. Probe (*Measured*, one small run): two concurrent `cargo check`s sharing `pse-math`; the second waited on `pse-math` only.
  - Cargo's tracking issue limits this parallelism to commands that produce no artifacts. Builds and tests still contend, and shared dirty units still wait.
- **AE-07 The full native run is serialised by the `store` test group** (*Measured*, JUnit of the
  2026-10-07 native gate, 2,426/2,821 tests before SIGTERM).
  - 2,135 tests finished within about the first 60 s; mean concurrency was about 15 in the first 30 s.
  - The 291 canonical-store tests (1,508 s of 2,143 s total test time) then ran at concurrency 1.0 for about 24 of the run's 25.5 minutes.
  - The cause is `test-groups.store.max-threads = 4` combined with `threads-required = 4`.
  - The effects differ. `canonical_fixture_store` creates an isolated UUID database. `ManagedWorkerCase` locks a deployment state and owns its finite worker slots. Supervisor operations affect a whole deployment. No inspected source requires one exclusion across every database-only test (*Interface-checked*, design review F03).
- **AE-25 Python component tests share the persistent canonical database** (*Interface-checked*,
  P5 survey). `python/pse/tests/conftest.py::canonical_substrate` opens `pse/canonical`, which
  `py-test` runs under xdist; those tests are neither isolated nor cleaned up, unlike the Rust
  fixture's UUID databases.
- **AE-26 One shared server limits database-only concurrency** (*Measured* by the P5 helper,
  32 GiB-profile private server, 2026-10-07). With effect-based scheduling, store/1 tests overlap
  (concurrency 2.5–3.6), but per-test durations grew 3–3.6× over serial, so net throughput was
  about 1× across the former store subset and about 2× in a 4-way probe; server CPU was mostly
  idle and ~20% of completions clustered within 0.15 s, suggesting a shared server-side wait
  point (commit path, database creation/removal or the namespace catalog; not identified).
  34 tests that pass at baseline failed with RocksDB `TransactionConflict` ("MemTable only
  contains changes newer than …") while fixtures created their full schema concurrently in one
  long DDL transaction that is never retried. Counter-example: eight `canonical_studies` tests
  ran together at serial speed. Per the design-phase rule these are codebase and server-profile
  findings, not reasons to re-serialize.
- **AE-08 Long recipes fail late** (*Measured*, W1).
  - `seed-conformance` ran 2.0 h and failed; `assessment` ran up to 1.8 h and failed; `modeling-conformance` succeeded 18% of the time and ran up to 1.6 h; `features-powerset` took 41–53 min per run.
  - `just` accounts for 73.6 of ~85 shell agent-h in P4–P5. Agents waited through 561 `sleep` calls (5.7 h requested) and 1,089 `wait_agent` calls.
  - A preflight can catch missing infrastructure and known prerequisites, but not late code or scientific failures.
- **AE-09 Build costs** (*Measured*, load 10–23 while another project compiled).
  - Cold `check-package pse-math`: 49 s. Warm no-op: 0.3 s. Leaf touch: 1.0 s.
  - First test-profile build for `unit-package pse-math`: 3 min 6 s (the native gate's was 4 min 11 s). Warm: 0.7 s; after touch: 1.9 s.
  - sccache served 37% of Rust compiles during the cold rebuild; workspace crates build incrementally, so sccache does not cache them.
  - A bare `cargo check -p pse-math` without force-validate built a second feature graph (11 s), and returning cost nothing. That is a one-time cost, not thrash.
  - Toggling `IPOPT_DIR`/`PKG_CONFIG_PATH` re-ran only `pse-backend-native`'s build script (about 1 s) in check mode. Linked builds were not measured.

### Command formation and selection (S1–S3)

- **AE-10 Selection errors and retries without edits** (*Measured*, W1 P4–P5).
  - 59 usage errors, about 43 of them nextest or pytest selection. A bare test name passed to `unit-package` fails with exit 94; an empty match exits 4.
  - 246 retries without an edit in between (7.1 agent-h), mostly with changed flags, filters or thread counts.
  - Both drill runtimes wrapped their filters in `package(<pkg>)`, because the recipes silently add `-p pse-relations`.
- **AE-11 There is no "tests affected by this change" selection** (*Measured*, drill D2).
  - Finding the tests that exercise `pse_math::numerics::term_scale` took 97–129 s (18 turns for Claude).
  - The answers spanned three packages, plus a `solver-kinsol` feature gate that plain `unit-package` silently skips.
  - nextest's `rdeps()` gives a crate-dependency selector (*Interface-checked*). It does not prove every affected test has been found: feature-gated tests, Python consumers, shared fixtures, generators and root configuration need visible treatment.
- **AE-12 No single codegen drift check** (*Measured*, drill D3).
  - The two runtimes proposed different procedures.
  - `codegen-check` and `hygiene` check different subsets. `codegen-check` lacks `codegen-rust-contracts-check`, and `hygiene` lacks the python, bindgen and schema checks. Neither covers the `_native.pyi` stub.
  - `codegen-relations-check` omits `--locked`.
  - `python-stubs --check` reads metadata from the installed compiled extension, so it cannot establish freshness when that extension is stale.
- **AE-13 Pass-through gaps** (*Interface-checked*, `just --dump`).
  - 11 nextest recipes take no extra arguments, and 12 hard-code `nextest run`, ignoring `PSE_NEXTEST_ACTION`.
  - `check`/`check-package`/`clippy-*` accept no arguments, so `check-package` has no escape to a second package or `--message-format`.
  - `unit-package` names one package but forwards `*args`, so an extra `-p` works.
  - The recipes request at least ten distinct `--features` sets.
- **AE-14 Recipe sprawl and churn** (*Measured*).
  - 212 recipes; `just --list` is 25 KB.
  - 28 recipes have never been used, and 95 went unused in P4–P5.
  - Agents invoked 56 recipe names that no longer exist.
  - Many fixed-filter recipes are artifacts of finished packets (`unit-m21*`, `dev-native-*`, `unit-*-contracts`, `unit-dynamics-fitting`).
- **AE-15 AGENTS.md discourages the bare path:** "If no recipe fits, say so rather than improvising
  a long command line". In P4–P5, raw cargo is already rare: 99 commands, mostly `cargo metadata`/`cargo run -p`.

### Feedback (S1, S5, S6)

- **AE-16 Output handling costs turns** (*Measured*).
  - The default nextest profile prints a line for every passing test (`status-level` defaults to pass). `local` and `ci` are already quiet.
  - In P4–P5, agents redirected 1,894 build/test runs to a log, then read logs 2,069 times within the next five calls.
  - Bundles stream every step's output inline and save no logs. `scripts/validation.py` already provides selected scopes, logs, structured results and interruption handling.
  - Pipe-masked failures (an exit code of 0 because of a pipe) dropped from 540 in P0 to 3 in P4–P5.
- **AE-17 Wrong-path reads** (*Measured*). 1,850 reads of nonexistent paths in P4–P5 (4% of commands).
  313 of them confused `mod.rs` with `foo.rs`; HEAD has 19 `mod.rs` modules and 88 `foo.rs` + `foo/` pairs.

### Instructions and runtime configuration (all scenarios)

- **AE-18 Codex will silently truncate AGENTS.md soon** (*Interface-checked*, Codex 0.160 source).
  - `project_doc_max_bytes` defaults to 32,768 bytes, shared by every project doc. AGENTS.md is 31,021 bytes, leaving 1,747.
  - When the budget is exceeded the last document is cut, with only a log line.
  - The largest sections are Agent runtimes (4.0 KB), Decisions and documentation (3.6 KB), Execution rhythm (3.1 KB) and Prime directives (3.0 KB).
- **AE-19 Duplicated and stale guidance** (*Interface-checked*).
  - The principles-and-heuristics paragraph appears in 16 files.
  - Context7 guidance reaches a Claude session four ways.
  - Three files cite AGENTS.md "Context7/capability routes" that AGENTS.md doesn't contain.
  - `worker.md` routes to the `deltalake` skill, which isn't selected.
  - AGENTS.md recommends `defaultMode` in `.claude/settings.local.json`. Since Claude Code 2.1.257, `bypassPermissions` is ignored from project and local settings, and the maintainer launches Claude with `--dangerously-skip-permissions`.
  - Path rules miss the root `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `scripts/**` and the `justfile`.
- **AE-20 Stale artifacts** (*Interface-checked*).
  - The tracked `.claude/hooks/format-after-edit.sh` calls an action that no longer exists. It and `guard-protected-paths.sh` are unwired, but `lint-shell` still shellchecks them.
  - `.claude/path-allowlist.txt` lists paths that don't exist.
  - Seven `rustc-ice-*.txt` files are committed in the root. They record ENOSPC during a build on 09-30, not compiler bugs.
  - `~/.codex/config.toml` keeps trust entries for removed hooks.
  - The docs give the local nextest bound as 2 h; the profile sets 4 h.
  - `setup-test` describes itself as "stdlib only" but imports `xmlrunner`.
- **AE-21 Claude sessions carry unrelated context** (*Interface-checked*).
  - Eight synced plugins (superpowers, engineering, design, Notion, Dropbox and others), claude.ai connectors and Claude-in-Chrome add skill listings, about 250 deferred tool names and MCP instruction blocks.
  - superpowers' `SessionStart` injection mandates its own brainstorming and planning workflow, which competes with the repository's process skills and plan location.
  - Project settings can turn these off (`enabledPlugins` false, `disableClaudeAiConnectors`, `deniedMcpServers`).
- **AE-22 Code-intelligence tools are unused while their processes accumulate** (*Measured*).
  - Neither runtime made any LSP, `mcp__ide__*` or rust-analyzer-MCP call, and there were 11 library-catalog calls in total.
  - The catalog has 0 capability records, and its index was built 14 commits before HEAD.
  - Each Codex thread starts its own servers: 17 `rust-analyzer-mcp`, 10 library-catalog and 21 `node_repl` processes, about 1.4 GB in total.
  - Codex 0.160 role files cannot change MCP servers (*Interface-checked*, `AgentRoleOverrides`).
  - context7 is the code-intelligence tool agents actually use: 518 calls in P4–P5.

### Isolation (S5, S7)

- **AE-23 Agent processes share the editor's cgroup** (*Measured*).
  - Claude and Codex run inside `app.slice/app-cursor-8021.scope`. Every uncapped recipe runs there too: `test`, `test-package`, `unit-package`, `unit-libraries`, non-native `check*`, `clippy*`, `doctest`, `py-test`, `py-unit`, `dev-native-*` and `coverage`.
  - Native recipes such as `py-unit-native` and `py-sync-native` are already capped through `memory-cap.sh` or `native_exec.sh`.
  - Since 10-01, OOM kills have stayed inside capped units. oomd stopped the editor scope on 09-27 and 09-29.
  - Swap is full (7/7 GiB). The CPU controller is delegated, but the IO controller is not, so `IOWeight` would have no effect.
  - A slice's `MemoryMax` bounds its descendants together, and oomd kills every process in the cgroup it selects (systemd 255 manuals).
  - A systemd scope costs about 0.01 s to start.

## Plan

**Sequencing follows dependencies, not a strict order.**
- P1 and the supported parts of P8 are independent and can start at any time.
- P2 and P9 are designed together as the common command boundary: what it configures, what it prepares lazily, which operation owns descendants, and how overrides change effective limits. That design comes before recipes are rewritten onto it.
- P3's claims are settled before anyone relies on shared-checkout builds.
- P5 establishes run-owned resource identity before any parallel server control.

**Rules for every packet:**
- **Acceptance** means exercising each changed capability once in its revealing case, and deleting whatever it replaces in the same change. The review's acceptance examples are bounded demonstrations, not a matrix; broaden a check only when a failure or material doubt warrants it.
- **No lints:** no packet adds one.
- **Override, don't gate:** every new default can be overridden from the caller's environment or arguments, and the effective value is visible.

| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced code / deletion | Status |
|---|---|---|---|---|
| P1 Stale cleanup | Untrack the `rustc-ice-*.txt` files and ignore `rustc-ice-*`. Delete the unwired `.claude/hooks/*.sh` and their `lint-shell` consumer. Keep `path-allowlist.txt`: `lint-agents` uses it for generated-but-absent paths. Fix the three dead "Context7 route" references, the `deltalake` route, AGENTS.md's `settings.local.json` bypass advice, and the setup-test and 2 h/4 h text. Add `--locked` to the `cargo xtask` alias, which covers `codegen-relations-check` and `doc-lint`. In user config, remove only the identified stale Codex `hooks.state` entries. Align the stated agent efforts with the adapters. | `just lint-agents` and `just setup-test` pass once | Unwired hooks, ICE files | done (2026-10-07) |
| P2 Common command boundary | `scripts/pse-env` (D1), with `--print`, `--explain` (A1) and `-- <cmd>` [`--native[=caps]`]. `build-shell.sh` and every shebang recipe consume it, replacing their individual env sourcing. Fix `native_operation.environment` so caller thread values and `off` win (AE-24). RC02: the Claude `SessionStart` hook appends `--print` to `$CLAUDE_ENV_FILE`. Codex gets non-secret `shell_environment_policy.set` defaults and a `pse-env --` prefix. Bare-path parity per D2/RC01: `rust-analyzer.toml` (force-validate feature, own target subdirectory) and cargo aliases, with the docs naming what plain commands still need. Designed with P9. | S1/S6, one revealing case each: from `env -i … bash -lc`, `just unit-package pse-math 'test(numerics)'` runs licensed. An explicit `OMP_NUM_THREADS=4` survives `pse-env --native`. Native nesting keeps the existing `Operation` owner. `--print` performs no native preparation and quotes values correctly. `--explain` prints no secrets. | Ad hoc `.envrc.local` sourcing (`unit-m21`, `native-execution-env.sh`); duplicated setup in shebang recipes | done (2026-10-07) |
| P3 Shared-checkout concurrency | Enable `fine-grain-locking` in `.cargo/config.toml [unstable]` (config, not env, so sccache keys are unchanged). It is claimed for concurrent `check`-type commands only. Worktrees remain the route for concurrent artifact-producing work. `just activity` (A6) is a read-only view of active `pse-*` scopes, their operation identities, elapsed time and known Cargo contention. It reports "unknown" for a bare Cargo lock owner, and it is never a lock or a job ledger. `just worktree <name> [--ref R] [--python] [--native]`: the base step is `git worktree add` from a printed ref, copy `.envrc.local`, `direnv allow` and `just ready`. Uncommitted work is never carried implicitly. Optional Python/native preparation builds the new checkout's own venv and extension. Add `.worktreeinclude` for Claude worktrees. | S4/S7: two disjoint `check`s run without blocking. A shared-unit check and a test build report the remaining contention. `just activity` lists two active commands and handles a stale entry. `just worktree x --python` yields a checkout whose licence, environment and `pse` import origin are its own. | — | done (2026-10-07) |
| P4 Recipe defaults and state | Overridable defaults: `PSE_SURREAL_STATE` points at a recipe-managed state for the canonical recipes, and the nextest default profile gets a `test-threads` value suited to concurrent agents. `py-unit` routes by the installed extension kind observed through `doctor.check_extension` and the selected interpreter. A missing native library yields a rebuild instruction, never a silent replacement or weakened tests. No separately maintained marker; checkout-origin and artifact association stay with `producer_deployment` and 28h. RC03: the design-edit guard also honours a visible, removable, untracked `.design-edit` file. The proposed doctor cache is dropped; readiness stays an explicit `ready`/doctor call. | S1/S6: `just py-unit` with the dev extension, with the native extension, and with one missing library. An authorised design edit works through Edit or `apply_patch`, and stops when the file is removed. | Hand-set `PSE_SURREAL_STATE`/`NEXTEST_TEST_THREADS` guidance; `py-unit-native` once `py-unit` routes (or kept as an alias) | done (2026-10-07) |
| P5 Effect-based canonical scheduling | RC04 / D5. A test that fails under parallel scheduling is a code or test-structure finding (shared state, timing assumptions, fixture ownership) for follow-up, never a reason to re-serialize it; only the tests the design marks exclusive stay exclusive. Reclassify the `store` overrides: database-only consumers get `threads-required = 1` with a bounded group cap on the shared server. Server-lifecycle and `ManagedWorkerCase` journeys get exclusive per-run resources: a recipe-generated run namespace plus group slot, with unique state, ports and units. Nextest setup scripts are not used, because they are experimental in 0.9.146. Reuse the supervisor and immutable server binary. Cancellation removes only the run's owned processes, and override-selected external state is never auto-destroyed. Coordinate with 28e, which owns the fixture contract and qualification. | S5/S7, at low concurrency: a database-only pair shares a server. A worker/server-control pair keeps its exclusion. Two overlapping nextest invocations get distinct resources. Fixture cleanup is awaited. Then re-time the store subset with the JUnit concurrency analysis; the benefit stays *Proposed* until measured. | The blanket `threads-required = 4` serialisation | done (2026-10-07); value finding AE-26 open |
| P6 Selection and pass-through | `check-package` takes several packages and `*args`. In `unit-package`, the package restriction is built from the *final* selected package set (named plus any `-p` in `*args`), so the implicit `-p pse-relations` contributes no tests and an explicit `-p` still widens the run. Only a simple word is rewritten as `test(word)`; expressions, regexes and tool arguments pass through unchanged. An empty selection still fails, and the effective selection is printed. `just affected [base]` (A5) previews by default and runs on request. It compares against HEAD including staged, unstaged, deleted and relevant untracked files; a supplied base changes the comparison point. It maps changes to workspace crates and `rdeps(...)` with force-validate, suggests native features explicitly, and lists what it cannot cover: feature-gated tests, Python consumers, generators and root configuration. Every nextest recipe takes `*args` and honours `PSE_NEXTEST_ACTION`; `check*`/`clippy*` take `*args`. A complete `codegen-check` delegates every generated output to its owning check, and reports a missing or stale installed extension as a prerequisite result (28h owns artifact admission). `codegen` decides on bootstrap ordering itself. Discovery (A2): domain-named `[group]`s, and the docs point to `just --usage`, `--list --group` and the JSON dump. | S1–S3: one selected-test listing shows a simple word, an existing expression, two packages, an empty match, a feature-gated consumer, and a root or shared-input change. Every generated-output owner is reached, and an absent extension gives an honest prerequisite result. | Only fixed-filter recipes that `affected` or a documented filterset demonstrably replaces. Useful shortcuts stay regardless of usage frequency. | done (2026-10-07) |
| P7 Feedback and logs | The `default` nextest profile becomes quiet like `local`/`ci`: `status-level = "fail"`, `success-output = "never"`, `failure-output = "immediate-final"`. Live or full output is available on request (`--status-level pass`). Bundles reuse `scripts/validation.py` (`run_gates`/`execute`/`checkpoint`) rather than a second runner (A4). Run directories carry a unique run component, the immutable path is printed, `latest` is only a convenience, and the original exit status is preserved. The `pse-env:` classification comes only from the operation that knows the cause; arbitrary conftest failures are not relabelled. Long recipes get a selective preflight for missing infrastructure and known fixture prerequisites, without claiming to prevent late code or scientific failures. Optional: nextest's experimental `[record]` in user config for `cargo nextest run -R latest`. | S5/S6: a two-step bundle with one failure prints immutable log paths and keeps its exit status. Its log can be followed while active. An interrupted child leaves retained status and drained descendants. An unlicensed run reports `pse-env:`. | Inline streaming in `_bundle` | done (2026-10-07) |
| P8 Instructions and runtime configuration | D3: set `project_doc_max_bytes = 65536` first, then compact AGENTS.md toward ≤ 20 KB, moving mechanics to `docs/dev/agent-environment.md` with usable links and keeping task routes and deliberate overlap. RC01 wording in AGENTS.md, the justfile header and `.claude/rules/rust.md`. One owner for the principles paragraph, with a short overlap retained where an isolated worker needs it. Extend the rule `paths:` to root manifests, `scripts/**` and the `justfile`. D6/RC05: project `.claude/settings.json` disables superpowers and the unrelated connectors and synced plugins, disables the pyright LSP, and sets `skillListingMaxDescChars`. A pyrefly LSP plugin is optional. Codex `.codex/config.toml` sets library-catalog and rust-analyzer MCP to `enabled = false`, enabled per session with `-c`; the subagent gap is documented. A3: a short "semantic navigation" note in the agent-environment doc says when rust-analyzer, pyrefly or the catalog is worth enabling. | All: inspect the instructions actually loaded in a fresh Claude and a fresh Codex session, and follow one representative task route. A fresh session has no unrelated connector tools and no MCP children by default. Enabling a capability deliberately works. | Duplicated paragraphs; superseded instruction text | done (2026-10-07) |
| P9 Isolation | D4/RC06: `~/.config/systemd/user/pse-agents.slice` with `CPUWeight` below the editor and per-slice `ManagedOOMMemoryPressure`. No aggregate `MemoryMax` by default; it can be opted into, and effective ancestor limits are reported through `pse-env --explain` and `just activity`. `memory-cap.sh` and `pse-env` place per-command scopes in the slice, keeping `scope_owner`-compatible names (check `unit_observation` under the nested slice). Supervised workers that launch their own units are placed separately. `PSE_MEMORY_MAX` and `PSE_SLICE` override; `PSE_SLICE=none` keeps today's placement. Optional: `ManagedOOMPreference=avoid` for the editor scope (unverified on GNOME transient scopes). Designed with P2. | S5/S7: observe the actual command and worker placement. A small synthetic cap with a harmless sibling shows the over-cap failure stays isolated and descendants drain. Inspect aggregate behaviour and the override route. No machine-wide stress run. | — | done (2026-10-07) |

**Not proposed (rejected or held):**

| Idea | Reason |
|---|---|
| Cranelift dev backend | Conflicts with the no-FMA/float-equivalence invariant, and Plan 28 owns speed |
| pytest-testmon | Cannot see Rust changes; Python 3.14 is untested upstream |
| Per-agent `CARGO_TARGET_DIR` | ADR-0122 and sccache keys; worktrees and fine-grain locking cover the need |
| Splitting the justfile into modules | Renames would add to the 56 stale-name invocations already seen; cleanup, groups and `--usage` suffice |
| Retiring recipes by recent usage alone | Six days of mostly Codex usage is weak evidence; only demonstrably redundant recipes go (P6) |
| A lock-holder file or job ledger | Several unit locks have no single owner, and a bare Cargo owner can't be identified; `just activity` observes instead |
| A doctor readiness cache | Input hashes miss removed tools, mutated venvs, replaced extensions and unreachable servers |
| An independently maintained "native installed" marker | It can disagree with the artifact; `check_extension` observes it directly |
| An aggregate slice `MemoryMax` by default | It would silently override raised per-command caps; available as an opt-in |
| Per-slot servers for every database-only test | Isolated UUID databases can share a bounded server |
| Nextest setup scripts for run resources | Experimental in 0.9.146; the recipe owns the run namespace |
| A bacon daemon, `-Zshare-generics`, `--jobs-frontend` | No measured need yet |
| yq, difftastic, a newer `gh` | No measured need yet |
| `IOWeight` | The IO controller is not delegated |
| Any machine-wide build lock | Deadlock history |
| rpath for native libraries in dev artifacts | Interacts with the artifact identity 28h owns |
| Normalising `mod.rs` layout | Optional; the 313 mixed-layout misses are real but cheap per miss, and keeping it consistent would need a lint |

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| AE-01, AE-24 | S1, S6 | scheduled | P2 | W1 P4–P5 counts; `env -i` login-shell probe; `native_operation.environment` source |
| AE-02, AE-03, AE-04 | S1, S6 | scheduled | P4 | `py-unit` probe; W1 knob counts; hook source |
| AE-05 | S1, S5 | deferred | 28h (native setup lifetime) | Revisit if 28h's receipt verification lands without reducing first-call setup |
| AE-06 | S7 | scheduled | P3 | W1 lock messages; fine-grain-locking probe (checks only) |
| AE-07 | S5, S7 | scheduled | P5, with 28e | 2026-10-07 native gate JUnit; fixture and worker source |
| AE-08 | S5 | scheduled | P7 (preflight, logs); fixture failures stay with 28e | W1 long-run outcomes |
| AE-09 | S1 | disproved as thrash; open as cost | Monitor only | Revisit if test-build time dominates after P3/P5 |
| AE-10, AE-11, AE-12, AE-13, AE-14 | S1–S3 | scheduled | P6 | W1 usage/retries; drills D2/D3; `just --dump`; `python_stubs.rs` |
| AE-15 | all | scheduled | P2 (bare-path parity), P8 (wording) | AGENTS.md text; RC01 |
| AE-16 | S5, S6 | scheduled | P7 | W1 redirect-then-read counts |
| AE-17 | S1 | deferred | Maintainer | Revisit if wrong-path reads stay above 4% after P8 |
| AE-18, AE-19, AE-21, AE-22 | all | scheduled | P8 | Codex 0.160 source limits; instruction audit |
| AE-20 | all | scheduled | P1 | Instruction and config audit |
| AE-23 | S5, S7 | scheduled | P9 | Journal; cgroup placement; systemd 255 manuals |
| AE-26 | S5 | open | Maintainer decision: 28e (fixture schema creation retry or serialization; server write-buffer profile) and/or a Plan 29 follow-up sharding database-only consumers across run-owned per-slot servers | P5 helper runs 2026-10-07; revisit when the shared wait point is identified |
| AE-25 | S5, S7 | deferred | 28e (canonical fixture contract) | Revisit when 28e qualifies Python canonical consumers or a shared-database interference is observed |
| Review F01 | S1, S4, S6 | scheduled | P2 (with P9) | Precedence, export and native-lifetime contract in D1 |
| Review F02 | S7 | scheduled | P3 | Locking claims scoped to checks; observational `just activity` |
| Review F03 | S5, S7 | scheduled | P5, with 28e | Effect-based scheduling with run-owned resources (D5) |
| Review F04 | S1–S3 | scheduled | P6 (D2 wording in P2/P8) | Final-package-set restriction; truthful `affected` coverage; extension prerequisite |
| Review F05 | S1, S6 | scheduled | P4 | Observation-based routing; doctor cache dropped |
| Review F06 | S5, S7 | scheduled | P9 | Placement separate from protection (D4) |
| Review F07 | S4, S6, S7 | scheduled | P3, P8 | Version-supported opt-in; worktree baseline |

## Verification

Each packet exercises its changed capabilities once, in the revealing case named in its row. In this
design phase a check establishes only two things: that the premised change is correctly implemented,
and that no first-principles basis undermines its value. Existing tests failing under a change is a
separate question — the production code or the tests may be what needs changing — so such failures
are recorded as codebase findings for later work, not as acceptance failures. At scope end,
`just lint-agents`, `just setup-test` and `just hygiene` run once. None of these is a merge gate.
Evidence labels go into the Outcome when the plan closes.

Optionally, the maintainer may ask for a longitudinal comparison:
- re-run the four drills in both runtimes;
- rebuild the W1 parser (it lived in the assessment session's scratchpad) and compare lock messages,
  environment failures, selection errors and retries without edits per 1,000 commands.

## Execution checkpoint (2026-10-07)

Executed in the main checkout with the Plan 28 session paused, one commit per step:
P1 `4eb27638b`, P8 runtime configuration `96460bdaa`, the command boundary B1–B5
(`26da21318`, `95bad201f`, `348f6b92b`, `898e57040`, `bca4ec81f`), its implementation review's
fixes `3070b39f9`, C2/P3 `0e712cd29`, C1/P4 `733aaf444`, C3/P6 `9f76a6dcd`, P8 instructions
`188039243`. P7 `1f3da9dd4`, P5 with the nextest default-profile changes `c7dcbd2b8`, the supervisor
script-mode fix `1d2f34333` and the parity-collection fix `c3d46dad6` followed.

Decisions made during execution:
- The slice is `pse.slice` (systemd nests dashed names, so a CPU weight must sit beside
  `app.slice`), and ordinary commands get `pse-cmd-*` scopes that native ownership never
  matches; native and worker scopes keep their names.
- `memory-cap.sh`, `native_exec.sh`, `native-recipe-env.sh`, `build-shell.sh` and `build-env.sh`
  are deleted, with no shims; `.envrc`, the manual native helpers, the builders and the
  supervisor all use the boundary.
- An implementation review of the boundary found two High issues (reused builder scope names;
  `off` lost under nesting) and Medium status/limit/explain gaps; all were fixed.
- Deviation: `.claude/path-allowlist.txt` stays (lint-agents uses it).
- Deviation: recipe groups stay cost tiers (`mutating` is safety-relevant). Discovery comes
  from the justfile header, `--usage` and the JSON dump.
- Deviation: AGENTS.md is 21,895 bytes, not ≤ 20 KB. The remaining text is task routes,
  directives, invariants and gotchas.
- Deviation: `just worktree` reports `doctor` instead of running `just ready`, because a fresh
  checkout without `--python` legitimately lacks a venv.
- New finding AE-25: Python component tests share the persistent `pse/canonical` database
  (no isolation, no cleanup). Routed to 28e.
- New observation: an sccache daemon started by a build stays in that build's scope, so it
  keeps the scope populated (and generations pinned) until sccache idles out, as before.
- New observation: user-level Codex plugins add about 60 skill entries, many duplicated three
  or four times. That is a user-level choice.

Next: scope-end hygiene and the Outcome.

## Open items

These are non-blocking; each packet proceeds on the stated default.

1. **library-catalog future.** P8 makes it opt-in per session. Whether to populate its capability
   records and refresh it in `just ready`, or retire it, is open. Default: opt-in, unchanged otherwise.
2. **Exact plugin and connector list.** Default per D6: disable superpowers; the Gmail, Calendar,
   Drive, Dropbox and Notion connectors; and the notion-memory, design, cowork-plugin-management and
   dropbox synced plugins. Engineering, desktop-commander, GitHub and Chrome stay enabled unless the
   maintainer removes them.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
