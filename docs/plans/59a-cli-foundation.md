---
title: "Plan59a: CLI foundation"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md]
---

# CLI foundation

Part of [Plan59](59-agent-cli.md). This plan establishes the contracts every later verb
consumes:
- the rule-change edits;
- the clap command tree;
- the output and exit contract;
- the state store;
- host-tool lifecycle;
- the derived catalog with `explain` and provenance.

Source opportunities: [O01](../functional_reviews/rust-agent-cli-2026-10-29.md#O01),
[O02](../functional_reviews/rust-agent-cli-2026-10-29.md#O02),
[O06](../functional_reviews/rust-agent-cli-2026-10-29.md#O06),
[O13](../functional_reviews/rust-agent-cli-2026-10-29.md#O13),
[O24](../functional_reviews/rust-agent-cli-2026-10-29.md#O24), and the host-ceiling item of
[O11](../functional_reviews/rust-agent-cli-2026-10-29.md#O11). The clap decisions come from the
review's [integrated clap capabilities](../functional_reviews/rust-agent-cli-2026-10-29.md#clap-capabilities)
contract, which this plan implements.

## Baseline

Facts from a code map of `crates/pse-host` at `68e075493`.

**Dispatch.**
- `main.rs` matches `argv[1]` by hand against fourteen words. Each module has its own `run_cli`
  loop: `boundary`, `environment`, `agent_hooks`, `doctor`, `native`, `server`, `study_module`
  and `telemetry_service`. `activity`, `cargo-tools`, `diagnostic-tools` and `run` are handled
  inline in `main.rs`.
- Errors print one `pse-host: …` (or `pse-env: …`) line and exit 125.
- `checkout_root()` is duplicated in seven modules; it walks up to `.config/diagnostic-tools.toml`.
- There is no command framework, no `--format`, and no versioned output except `doctor --json`.

**Diagnostics.** `HostError`, `CorrectnessError` and `EnvironmentError` derive
`miette::Diagnostic` through `pse_diagnostics::impl_diagnostic!`, but every variant carries the
single code `RuntimeInfrastructure`, and none is rendered.

**Bins.** Two are auto-discovered: `pse-host` (`src/main.rs`) and `cargo-pse-correctness`
(`src/bin/`). Tests reach them through `CARGO_BIN_EXE_pse-host` and
`CARGO_BIN_EXE_cargo-pse-correctness`.

**Contracts a port must keep exactly:**
- **`boundary` (pse-env).**
  - The first non-flag word starts the command; `--` ends options.
  - Last-wins between `--native` and `--native=CAPS`; `--native=` is meaningful.
  - The caller's `PSE_NATIVE_CAPABILITIES` applies only to a bare `--native`.
  - `--store`/`--bind-allocation` refusal; `--docs`/`--native` exclusion; `-h` only before the
    command.
  - Exit: the child's status, 128+signal, 127/126 for exec failures, 125 for boundary errors.
- **`agent-hook guard|session-env`.**
  - Reads stdin JSON to EOF, then exits 0, or exits 2 with `BLOCKED: …`.
  - `session-env` writes `CLAUDE_ENV_FILE` and always exits 0.
  - `xtask agent-config-check` requires hook commands to contain `/scripts/pse-host` and
    `agent-hook`.
- **`cargo-pse-correctness`.**
  - Called by Cargo as `cargo-pse-correctness pse-correctness …`.
  - `--nextest-test-env` first; `--version` prints `… protocol 1`.
  - An adapter mode (`PSE_CORRECTNESS_ADAPTER=1` or `CARGO_HACK_CARGO_SRC`) receives Cargo's own
    argv, which is *not* clap-parseable.
  - Errors print `arrow-validation: …` and exit 2.
- **`__foreground-exec`.** The trampoline is re-executed through `/proc/self/exe`, so it must
  exist in every binary that launches foreground scopes.

**Host tools.**
- `scripts/bootstrap-host` builds `--package pse-host --bins` under a `flock`, stages
  `generation.*`, probes each tool with `--version`, and publishes `current` atomically.
- Its tool loop is hard-coded to `pse-host cargo-pse-correctness`.
- Twelve generations exist and none is ever pruned.
- There is no source fingerprint. The `pse-host` path-dependency closure is `pse-host`,
  `pse-diagnostics`, `pse-buildinfo` (with `build.rs`) and `pse-ids`.

**State roots.** Ten resolvers across `telemetry.rs`, `telemetry_service.rs`, `environment.rs`,
`diagnostic_tools.rs`, `native.rs`, `server.rs`, `study_module.rs` and `admission.rs`:
- They mix `pse/` and `pse-arrow/` prefixes.
- They disagree on whether an empty variable counts as unset.
- All read the passed `Environment` map rather than `std::env`. That is the testable shape to
  keep.

**Dependencies.**
- `pse-host` has no clap, tracing-subscriber or schemars.
- The workspace pins `clap =4.6.7` (`derive`), `tracing-subscriber =0.3.23` (`env-filter`,
  `registry`; no `fmt` or `json`) and `schemars =1.2.2` (no features).
- `[workspace.metadata.pse.dependency-ceilings]` has `semantic`, `columnar`, `generator` and
  `admission` rows, but no `host` row.

## Target

`pse-host` gains a library module tree `cli` (the clap types and handlers) and `envelope` (output
and diagnostics). Three thin bins call `pse_host::cli::main(BinKind)`:

| Bin | Role |
| --- | --- |
| `pse` | The agent-facing front door. All verbs. |
| `pse-host` | The same tree. It stays because hooks, seeds and tests exec it; `agent-hook` and `boundary` live here and are hidden from `pse -h`. |
| `cargo-pse-correctness` | Cargo external subcommand. A clap parser handles `pse-correctness check\|build\|test\|run\|clippy\|nextest run\|list` and `--version`. Adapter mode skips clap entirely and forwards Cargo's own argv to the existing `resolve_with`/`execute` path. |

`pse-env` remains `scripts/pse-env` → `pse-host boundary`. Its grammar is ported to a clap
subcommand with `trailing_var_arg` + `allow_hyphen_values` on the command positional, which keeps
the first-word-starts-command rule. That exact behaviour is the exec adapter's contract, and the
parser tests that pin it move with it. `pse run -- <argv…>` is the agent-facing equivalent with
`last = true` pass-through.

The shared argument structs, typed value parsers, groups, `defer` subtrees, `ValueSource` use and
feature choices are exactly the review's
[clap contract](../functional_reviews/rust-agent-cli-2026-10-29.md#clap-capabilities).

### Decisions with reasons

- **Thin bins, not `multicall`.** clap forbids arguments on a multicall root, and dispatch would
  depend on `argv[0]`. Three bins over one library tree give one build, one generation and one
  error contract.
- **Catalog walker shared by `#[path]`.** The walker over `clap::Command`, used by `pse` and by
  xtask's schema export, is one source file included by both crates with `#[path]`, the way
  `scripts/workspace.rs` is already shared. That avoids a new crate. Adding a `pse-host`
  dependency to xtask would pull zbus, tokio and rustix into xtask; putting clap into
  `pse-buildinfo` would widen a crate many product closures use.
- **One state resolver in `environment.rs`.** It owns every host path:
  - the checkout store `$XDG_STATE_HOME/pse-arrow/<checkout-id>/`;
  - telemetry state;
  - Surreal state;
  - the sccache cache and socket;
  - tool roots;
  - native cache;
  - the admission runtime directory.

  The `pse-arrow/` prefix becomes the single spelling for new paths. Existing paths keep their
  current location but are resolved in one place, so later moves are one-line changes and an
  empty variable means unset everywhere. Consumers in `telemetry*.rs`, `native.rs`, `server.rs`,
  `study_module.rs`, `diagnostic_tools.rs` and `admission.rs` migrate in the same package.
- **Envelope owned by the host.** The envelope and `ErrorRecord` live in `pse-host`. Diagnostic
  codes become per-variant: `impl_diagnostic!`'s single code is replaced by the §23.2 dotted
  vocabulary in `pse-diagnostics`, adding codes there for host-specific failures such as
  `host.stale_tools` and `fixture.unprepared`.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [A01](#a01) | complete | 59a | Rule-change edits R1–R5 landed in their owners (R4's guard change is D06) |
| [A02](#a02) | complete | 59a | clap tree in the library; three thin bins; every existing verb ported and old parsers deleted (`5e7744bd9`) |
| [A03](#a03) | planned | 59a | Envelope, exit classes, diagnostics and `pse schema` |
| [A04](#a04) | planned | 59a | One state resolver; checkout store; consumers migrated |
| [A05](#a05) | planned | 59a | Host-tool fingerprint, pruning and stale-host rebuild; `host` dependency ceiling |
| [A06](#a06) | planned | 59a | Derived catalog with `valid_values`, `explain`, `env --explain` provenance and `status` |

<a id="a01"></a>
### A01 — Land the accepted rule changes

**Consumes:** the operator decisions R1–R5 in the [coordinator](59-agent-cli.md#rule-changes).

**Delivers:** text changes in each rule's owner, so later packages implement allowed behaviour.
- **R1.** Plan58 §Purpose and the Outcome sentence "CLIs are optional conveniences… no command
  framework, universal operation catalogue…" now point to Plan59 and the derived catalog.
  Update the `docs/plans/README.md` current-work text ("CLIs remain optional").
- **R2.** Plan58's target row "Bespoke build observation and derivative signals retire", 58b
  §B (bespoke build observation and its wait consumers) and 58h's acceptance wording now
  distinguish build-stage telemetry (still retired) from owned-run supervision (allowed). Also
  update the environment guide §Detached watches.
- **R3.** Proposed ADR-0233 is amended in place: one CLI grammar and a per-run supervisor; the
  ledger as stop authority for detached runs; "help/read/explain never build", with effectful
  verbs rebuilding a stale host. Add the `blueprint.md` revision row and the §3.2 `pse-host`
  description and §24.1 host wording, created with the design-edit authorization that exists
  until R4 lands.
- **R4.** Recorded as a decision here. The guard change itself is [59d D06](59d-knowledge-and-workflow-verbs.md#d06),
  where its owner lives.
- **R5.** AGENTS.md's checkout workflow and execution-rhythm wording allow any agent to commit
  its own paths through `pse commit` and to format the files it commits whole. The `pse commit`
  verb is [59d D08](59d-knowledge-and-workflow-verbs.md#d08).

**Acceptance:** each owner states the new rule, and the ADR lint and lifecycle validation accept
the edits. This is a docs-only package with no static or product checks.

<a id="a02"></a>
### A02 — One clap tree behind three thin bins

**Consumes:** A01 R3.

**Delivers.**
- Workspace `clap` features become `derive`, `env`, `string` and `unstable-ext` (one feature set
  for every consumer under ADR-0122). `pse-host` takes `clap`.
- `pse_host::cli` declares `Cli` with the shared `GlobalArgs`/`OutputArgs` structs, and ports
  every existing verb as a derived subcommand: `boundary`, `agent-hook`, `doctor`, `cargo-tools`,
  `diagnostic-tools`, `telemetry-service`, `native`, `server`, `study-module`, `env`, `activity`,
  `run` (as `pse run -- <argv…>`), and the hidden `__foreground-exec`. Each keeps its current
  semantics, outputs and exit codes. [59b B01](59b-runs-and-continuity.md#b01) later adds
  ledger recording to `run`; A02 owns the port.
- `__foreground-exec` is dispatched by checking `argv[1]` before clap, output-format detection,
  staleness checks or a tokio runtime run, because the trampoline must start before any
  descriptor setup.
- `[[bin]]` entries are declared explicitly: `pse` (new), `pse-host` and
  `cargo-pse-correctness`.
- `scripts/bootstrap-host` builds, probes and publishes all three, and gains a `scripts/pse` seed
  like the others.
- Checkout self-location replaces the seven `checkout_root()` copies, in this order: `-C`, then
  the executable's generation path, then the current directory.

**Deletes:** every hand-rolled `run_cli`/`parse` loop and the inline `main.rs` dispatch, plus the
tests that exercised only hand parsing. Behavioural tests move to `try_parse_from` cases that pin
the same contracts:
- the `boundary` data-after-first-word rule;
- `--native` last-wins and the empty value;
- the refusals;
- the agent-hook exit codes;
- correctness argv shapes and adapter-mode bypass.

**Acceptance:**
- `Cli::command().debug_assert()` passes.
- The pinned parser cases above pass.
- The existing `tests/{boundary,activation,activity,foreground,correctness,bootstrap,native,server}.rs`
  pass unchanged in meaning; they may reach the new `CARGO_BIN_EXE_pse` where they test agent
  verbs.
- `xtask agent-config-check` still accepts the unchanged hook commands.

<a id="a03"></a>
### A03 — Output, exit and diagnostic contract

**Consumes:** A02.

**Delivers.** The [O02](../functional_reviews/rust-agent-cli-2026-10-29.md#O02) contract:
- **Format and output.**
  - Global `--format auto|human|json|ndjson` (`PSE_FORMAT`), with the format decided before
    parsing completes, from the environment variable, TTY detection or an argv pre-scan.
  - A versioned envelope with summary first, `run`, `child_exit`, `failures`, `artifacts` and
    `next`.
  - `--fields` and `--budget`.
  - NDJSON event streams that end with a `summary`.
- **Exit classes.**
  - *Exec-adapter* verbs (`boundary`, `run`) forward the child's status, or 128+signal.
  - *Managed* verbs use 0 (reconciled pass), 1 (subject failed or incomplete), 2 (usage), 125
    (boundary or tool) and 130 (interrupted).
  - A `launched` detached run, and a `pse wait` whose timeout elapsed while the run was still
    `running`, exit 0 with the status in the envelope. Harnesses treat non-zero as failure.
  - `pse wait` on a finished run exits with that run's managed code.
- **Protocol outputs keep their raw formats** regardless of `--format`:
  - `pse-env --print`, which `.envrc` captures without a TTY;
  - the `--version` "protocol 1" probes;
  - the `agent-hook` stdin and exit protocol;
  - the line-oriented `cargo-tools` and `diagnostic-tools` listings read by
    `scripts/bootstrap.sh`.
- **Diagnostics.**
  - An owned `ErrorRecord` is serialized from `miette::Diagnostic`: code, severity, help, url,
    labels and related.
  - `NarratableReportHandler` renders it for non-TTY stderr and `GraphicalReportHandler` (miette
    `fancy-no-syscall`) on a TTY. Never `JSONReportHandler`, whose 7.6.0 output has unescaped
    `url` and offset-only labels.
  - clap parse errors map `ErrorKind`/`context()` into the same record. `DisplayHelp` and
    `DisplayVersion` exit 0.
- **Schemas.** `pse schema <verb>` prints the envelope schema from
  `SchemaSettings::draft2020_12().for_serialize()`. Workspace `schemars` gains `raw_value` so
  `serde_json::value::RawValue` can carry cargo and nextest records unmodelled.
- **Existing JSON outputs move into envelopes:**
  - `doctor --json`;
  - `env --explain`, where `--json` becomes the global format;
  - `activity`;
  - `telemetry-service`;
  - `study-module`.
- **Error codes.** Per-variant §23.2 codes replace the single `RuntimeInfrastructure` code for
  `HostError`, `CorrectnessError` and `EnvironmentError`.

**Acceptance:**
- An `insta` snapshot for each verb's envelope and `-h`/`--help`.
- Every envelope validates against its `pse schema` output.
- A clap usage error yields exit 2 with a structured record.
- An exec-adapter child exiting 7 yields exit 7. A managed verb whose child exits 7 yields exit 1
  with `child_exit: 7`.

<a id="a04"></a>
### A04 — One state resolver and the checkout store

**Consumes:** A02.

**Delivers.**
- `environment.rs` gains the single resolver:
  - `checkout_id` comes from the checkout's absolute path and its git common directory, so
    worktrees of one repository relate.
  - Typed paths cover the store (`runs/`, `fixtures/`, `api/`, `probes/`, `scratch/<session>/`,
    `handoff/`), telemetry state, Surreal state, sccache, tool roots, native cache and the
    admission runtime directory.
  - Empty variables are treated as unset uniformly.
- All ten current resolution sites migrate to it and their local `base()` copies are deleted.
- There is no separate index. History and `latest` are derived by scanning `runs/*/meta.json`.
  Each run directory is written only by its own run, with temp file, fsync and
  `renameat_with(…, NOREPLACE)` for terminal files, so concurrent runs never contend.
- Retention keeps the last N runs per checkout. Scratch is pruned at session end or after a fixed
  age.

**Acceptance:**
- Resolver unit tests cover each path with and without the relevant variables, including empty
  values.
- Two worktrees of one repository produce distinct checkout IDs and a shared relation.
- Two concurrent runs each write their own directory, and both appear in `history`.

<a id="a05"></a>
### A05 — Host-tool lifecycle: fingerprint, pruning, stale rebuild

**Consumes:** A02, A04.

**Delivers.**
- **Fingerprint.**
  - At build time, `pse-host` computes its path-dependency closure from `cargo_metadata`: today
    `pse-host`, `pse-diagnostics`, `pse-buildinfo` with `build.rs`, and `pse-ids`.
  - It stores that path list, plus `Cargo.lock`, the root `Cargo.toml`, `rust-toolchain.toml` and
    `.cargo/config.toml`, with a hash in the generation.
  - A staleness check rehashes only the stored paths. The closure is not declared again in shell.
- **Pruning.** Under the existing `flock`, `bootstrap-host` keeps the current generation and the
  previous one. It never deletes a generation referenced by a live run in the ledger, such as a
  long nextest run executing the adapter from its generation path, or a detached unit's
  `ExecStopPostEx`.
- **Copying between worktrees.** `bootstrap-host --from <checkout>` copies a matching generation
  instead of building; [59e E01](59e-environment-and-adoption.md#e01) uses it for worktrees.
- **Automatic rebuild** (R3; operator decision 2026-10-29).
  - When a top-level effectful verb finds the host stale, it rebuilds it through
    `bootstrap-host`, then re-execs the new generation before proceeding.
  - **If the rebuild fails, the verb fails with the build error.** It never runs an older
    generation. This is safe because concurrent agents work in separate worktrees, each with its
    own `target/host-tools`, so one agent's in-progress host edit cannot break another's verbs.
  - **Where it never rebuilds:**
    - read-only verbs (`-h`, `status`, `explain`, `env --explain`/`--print`, `doctor`, `schema`,
      `commands`), which report `host.stale_tools` instead;
    - `cargo-pse-correctness` adapter mode, which runs inside Cargo while Cargo holds the build
      lock;
    - `agent-hook`, with its seconds-long timeout;
    - the internal `__foreground-exec`, `__supervise` and `__finalize`;
    - any call nested inside a workload scope.
  - **Re-exec once.** Re-exec happens at most once, guarded by an environment marker. The
    fingerprint taken before the build is recorded, because sources can change during it.
- **Host ceiling.** `[workspace.metadata.pse.dependency-ceilings.host]` declares
  `roots = ["pse-host"]` and forbids `pse-engine`, `pse-runtime`, `pse-operations`, `arrow*`,
  `datafusion*`, `parquet`, `pse-schema` and `surrealdb*`. `just family-check` enforces it.

**Acceptance:**
- `tests/bootstrap.rs` gains pruning (including retention of a generation referenced by a live
  run), fingerprint-match copy and fingerprint-mismatch rebuild cases.
- **Stale host.** A stale generation is reported by `pse status`, rebuilt by `pse run -- true`,
  and not rebuilt in adapter mode or from a hook.
- **Failed rebuild.** With a host source that does not compile, `pse run -- true` fails with the
  compiler error, and `current` still points to the previous generation, unused.
- `just family-check` passes with the new row.

<a id="a06"></a>
### A06 — Derived catalog, explain, provenance and status

**Consumes:** A03, A04.

**Delivers.**
- **Catalog.**
  - `pse commands --json` and `pse help --json <path…>` come from the shared `#[path]` walker
    over `Cli::command()` after `build()`. For each leaf the walker records arguments, possible
    values with help, env mirrors, defaults, groups and conflicts, rendered `-h`/`--help` text,
    examples and output-schema IDs. `requires`, examples and schema IDs travel through
    `unstable-ext` `Command::add`/`Arg::add`.
  - `valid_values` come from the CLI's own value-domain providers: packages and targets from
    cached metadata, run IDs from the store, fixtures and capability sets from their
    declarations. Providers are fast and side-effect-free; there is no shell completion.
- **Federated xtask subtree.**
  - `just codegen` gains an `xtask commands --json` export into a generated schema file, using
    the same walker and recording each verb's features and invocation route.
  - `pse` reads that file at runtime with the `string` feature and reports staleness.
  - Dispatching an xtask verb is an effectful run that builds xtask through the correctness
    adapter.
- **Explain and provenance.**
  - `pse explain <verb…>` and a global `--explain` print the resolved plan without effects:
    underlying commands, placement, capabilities, features, fixtures and environment changes.
  - `pse env --explain [KEY]` reports each value's layer: repository default, `.envrc.local`,
    caller environment or command line (`ArgMatches::value_source`).
  - `explain` never sources shell. Unevaluated `.envrc.local` values are reported as pending,
    using the existing `local_pending`.
- **`pse status [--brief|--line|--all]`** reports:
  - HEAD and dirty count;
  - the active plan pointer from `docs/plans/README.md` links;
  - live and recent runs (from 59b once present);
  - stale host;
  - checkout ID, and other worktrees with `--all`.

**Acceptance:**
- The catalog lists every non-hidden verb with rendered help.
- `valid_values` for `pse test`'s target argument lists real `pkg:target` pairs.
- `explain` output matches the command `pse test` then runs, compared on the same inputs.
- `env --explain` attributes a command-line override to `CommandLine` and a default to
  `DefaultValue`.

## Verification and handoff

Per AGENTS.md:
- **During packages:** compile checks (`just check-package pse-host`) and targeted tests (`just
  unit-package pse-host <filter>`, plus the named `tests/*.rs`).
- **At series scope end, once:** `just hygiene` and the integration journeys.

Snapshots use `insta`; executable `trycmd` cases are deferred until help examples stabilize.
59b, 59c and 59d consume A03 and A04. 59e consumes A05 and A06.
