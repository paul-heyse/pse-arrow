---
title: "Plan59a: CLI foundation"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md]
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

The [observability review](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md)
O01/O05/O08 and [improvement review](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md)
I01–I05/I08/I09/I11 refine these foundations. The coordinator owns their dispositions.
This revision schedules proposed work only: production execution remains stopped. A01 is
retained completed rule work; it does not establish implementation of the revised contracts.

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

### Foundation assessment and authority boundaries

The original baseline above describes the pre-CLI host, not delivered Plan59 behavior.
The current diagnostic owner already supplies `TypedDiagnostic::diagnostic_facts` and
`diagnostic_children`; `HostError::Settlement` retains operation and cleanup failures.
Those facts must survive presentation. `manager::foreground_placement` currently requires a
scope, so a service-hosted supervisor needs an actual placement/admission change and journey
in [B03](59b-runs-and-continuity.md#b03), rather than an assertion that reuse already works.

The host owns CLI invocations, process control and presentation. Canonical scientific state
keeps its existing run/attempt owners; observations keep their operation/process/source
identities and coverage. A typed `CliInvocationId` links them without aliasing scientific
`RunId` or `AttemptId`. Product inspection is delegated through the prepared owner interface
in [59f F01](59f-simulation-observability-and-diagnostics.md#f01). Response bytes and schema
identity may cross that boundary; runtime/model/Arrow interpretation stays with the product.
The file ledger is usable without a database or monitoring service. These boundaries support
another observing invocation or another product verb without moving scientific policy into
host handlers (AP-01–AP-06, DP-18, PS-10).

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
- **Description walker shared by `#[path]`.** The walker over `clap::Command`, used by `pse` and by
  xtask's schema export, is one source file included by both crates with `#[path]`, the way
  `scripts/workspace.rs` is already shared. That avoids a new crate. Adding a `pse-host`
  dependency to xtask would pull zbus, tokio and rustix into xtask; putting clap into
  `pse-buildinfo` would widen a crate many product closures use. This shares description
  code; it cannot serialize arbitrary typed parsers or establish delegated semantic equivalence.
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
| [A02](#a02) | complete | 59a | Shared clap tree/three binaries merged; targeted parser/protocol and bootstrap tests pass; broader qualification remains at series end |
| [A03](#a03) | in-progress | 59a | Output/diagnostic/schema implementation underway after A07 scoped target acceptance |
| [A04](#a04) | in-progress | 59a | Resolver and consumer migration integrated and targeted tests pass; concurrent invocation/history acceptance remains with B01 |
| [A05](#a05) | in-progress | 59a | Fingerprint/manifest/use-claim foundations integrated; bootstrap, all-entry pinning, stale rebuild and safe pruning remain |
| [A06](#a06) | planned | 59a | Derived catalog with `valid_values`, `explain`, `env --explain` provenance and `status` |
| [A07](#a07) | complete | 59a | Proposed ADR-0233/blueprint315 published; scoped formal review accepts corrected target at Proposed strength; runtime obligations remain with consumers |

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

This records the earlier completed R1–R5 edits. The 2026-10-29 R3 control exception and
newly selected contract amendments belong to A07; A01 is not reopened or credited for them.

<a id="a07"></a>
### A07 — Amend the control and observability contracts

**Consumes:** the coordinator's operator-confirmed R3 refinement (2026-10-29), the selected
O01–O08/I01–I11 scope and existing ADR/architecture routes.

**Delivers:** reconcile proposed ADR-0233 and the applicable blueprint §21/§23/§24 owners
with distinct invocation/scientific/observation identities, command success versus subject
state and physical settlement, and typed product delegation. Record that existing-invocation
read/wait/cancel uses a compatible prepared recorded generation without rebuilding current
sources; new work retains failed-rebuild refusal. Use the required decision/review route and
architecture revision row; this plan amendment alone changes no accepted ADR or architecture.
Any new crate or service proposed by later investigation returns to its scoped decision route.

**Acceptance:** the governing owners agree on these distinctions and the coordinator records
their actual readiness. A03's revised contracts, A05/B03 control and F01 product delegation
cannot be implemented on plan wording alone while their governing amendment is outstanding.

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
- Early format/help detection stops at each adapter/child boundary. Neither a child's
  `--format` nor `-h` is a host option; raw Cargo adapter argv bypasses the host parser.
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

**Consumes:** A02 and A07's governing identity/result/diagnostic contract amendments.

**Delivers.** The [O02](../functional_reviews/rust-agent-cli-2026-10-29.md#O02) contract:
- **Format and output.**
  - Global `--format auto|human|json|ndjson` (`PSE_FORMAT`), with the format decided before
    parsing completes, from the environment variable, TTY detection or an argv pre-scan.
  - A versioned envelope with summary first, typed `invocation_id`, explicit scientific and
    observation references, command outcome, subject state, `child_exit`, settlement,
    observation completeness, `failures`, `artifacts` and `next`. Unknown/unavailable facts
    stay explicit. A CLI invocation never certifies a scientific result by itself.
  - `--fields` projects only the payload. Version, identity, state and coverage remain present;
    omitted/projected detail is explicit and has a matching view schema. `--budget` bounds
    rendered records/detail, never serialized bytes or mandatory terminal information. Page
    larger results or return a retained artifact reference.
  - Compact NDJSON emits one normalized record per line and a final observation summary with
    explicit terminal/nonterminal subject state. Opaque `RawValue` may contain newlines;
    retain original bytes separately or normalize before framing, never embed them verbatim.
- **Exit classes.**
  - Foreground *exec-adapter* verbs (`boundary`, `run`) forward the child's status, or
    128+signal. Detached `run` returns the launch acknowledgement below, since no child result
    is available at that point.
  - *Managed* verbs use 0 for a successful command response, 1 for a terminal subject failure
    or incomplete required reconciliation, 2 (usage), 125 (boundary or tool) and 130
    (interrupted). A terminal success assertion requires its owner's reconciled evidence.
  - A `launched` detached run, and a `pse wait` whose timeout elapsed while the run was still
    `running`, exit 0 with explicit command success and nonterminal subject state in the
    envelope. This is an observation timeout, distinct from an actual workload timeout.
  - `pse wait` on a finished run exits with that run's managed code.
- **Protocol outputs keep their raw formats** regardless of `--format`:
  - `pse-env --print`, which `.envrc` captures without a TTY;
  - the `--version` "protocol 1" probes;
  - the `agent-hook` stdin and exit protocol;
  - the line-oriented `cargo-tools` and `diagnostic-tools` listings read by
    `scripts/bootstrap.sh`.
- **Diagnostics.**
  - Adapt the native `TypedDiagnostic` interpretation into a versioned owned `ErrorRecord`:
    code/class, structured facts (sources, locations, rules, observations), ordered diagnostic
    children, ordinary `std::error::Error::source` causes, miette `diagnostic_source`
    delegation, severity, help,
    URL, labels and related diagnostics. Preserve each occurrence and both operation/cleanup
    failures in `HostError::Settlement`; assigning a new code does not preserve its children.
    Resolve labels through `SourceCode::read_span`, retaining source identity, non-ASCII
    locations and inherited source context for related diagnostics. Redact secret values
    without discarding their category/provenance. thiserror keeps typed ownership; miette
    supplies the diagnostic protocol and existing human renderers, not a second interpretation.
  - `NarratableReportHandler` renders the same native diagnostic for non-TTY stderr and
    `GraphicalReportHandler` (miette `fancy-no-syscall`) on a TTY. Never `JSONReportHandler`, whose 7.6.0 output has unescaped
    `url` and offset-only labels.
  - clap parse errors map `ErrorKind`/`context()` into the same record. `DisplayHelp` and
    `DisplayVersion` exit 0.
- **Schemas.** `pse schema <verb>` prints the envelope schema from
  `SchemaSettings::draft2020_12().for_serialize()`. Workspace `schemars` gains `raw_value` so
  `serde_json::value::RawValue` can carry cargo and nextest records unmodelled. Its schema is
  arbitrary JSON: envelope schema validation establishes no upstream record semantics.
  Normalized failures/outcomes are typed; B02 independently qualifies the actual extractor
  streams. Full and projected schemas describe their respective presentations.
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
- Projected/oversized responses retain required state and valid JSON; pretty raw upstream
  records cannot break NDJSON framing. Missing stream completeness cannot become a pass.
- Nested causes, related diagnostics with inherited source, non-ASCII labels, quoted URLs,
  multiple settlement failures and secret-bearing configuration retain actionable structure.
- A clap usage error yields exit 2 with a structured record.
- An exec-adapter child exiting 7 yields exit 7. A managed verb whose child exits 7 yields exit 1
  with `child_exit: 7`.
- Launch acknowledgement, nonterminal wait expiry, workload timeout, supervisor failure and
  unknown drain have distinct states. `pse run -- child --format json -h` preserves child argv.

<a id="a04"></a>
### A04 — One state resolver and the checkout store

**Consumes:** A02.

**Delivers.**
- `environment.rs` gains the single resolver:
  - `checkout_id` comes from the checkout's absolute path and its git common directory, so
    worktrees of one repository relate.
  - Typed paths cover the store (`invocations/`, `fixtures/`, `api/`, `probes/`, `scratch/<session>/`,
    `handoff/`), telemetry state, Surreal state, sccache, tool roots, native cache and the
    admission runtime directory.
  - Empty variables are treated as unset uniformly.
- All ten current resolution sites migrate to it and their local `base()` copies are deleted.
- There is no separate index. History and `latest` are derived by scanning
  `invocations/*/meta.json`; the `run` verb remains an invocation launcher. B01 owns exclusive
  metadata/state publication and supervisor/finalizer races, using temp file, fsync and
  `renameat_with(…, NOREPLACE)` for final files.
- Retention classifies disposable terminal presentation logs, protected artifacts, probes,
  scientific data and uncertain fixture evidence separately. Age/count applies only to
  eligible disposable state; active invocations, unresolved settlement, generation users and
  independently retained fixtures are protected. Scratch age alone cannot prove disposal.
  A04 defines eligibility; pruning waits for A05/B03's real lifetime protections. This
  schedules a policy, not deletion of existing caches/data.

**Acceptance:**
- Resolver unit tests cover each path with and without the relevant variables, including empty
  values.
- Two worktrees of one repository produce distinct checkout IDs and a shared relation.
- Two concurrent runs each write their own directory, and both appear in `history`.
- Retention refuses live/uncertain state and fixture evidence whose actual owner has retained it.

<a id="a05"></a>
### A05 — Host-tool lifecycle: fingerprint, pruning, stale rebuild

**Consumes:** A02, A04 and A07's R3/control amendment. Generation-use protection must be
available to B01/B03 and C's immutable selections before pruning is enabled.

**Delivers.**
- **Fingerprint.**
  - At build time, `pse-host` computes its path-dependency closure from `cargo_metadata`: today
    `pse-host`, `pse-diagnostics`, `pse-buildinfo` with `build.rs`, and `pse-ids`.
  - It stores that path list, plus `Cargo.lock`, the root `Cargo.toml`, `rust-toolchain.toml` and
    `.cargo/config.toml`, with a hash in the generation.
  - A staleness check rehashes only the stored paths. The closure is not declared again in shell.
- **Generation use and pruning.** A small lease/pin protocol covers every supported entry
  retaining an executable: foreground and nested launches, unrecorded boundary/compatibility
  adapters, Cargo's exported `CARGO`/`PSE_CORRECTNESS_EXECUTABLE`, supervisor, finalizer and
  existing-invocation control. Acquire before handing off paths; release only after actual
  users/children and required finalization settle. Define crash recovery conservatively from
  authenticated ownership evidence; ledger absence is insufficient and no global process
  scanner is introduced. Under `flock`, pruning preserves these users plus current/previous
  convenience retention. Pinned immutable helper outputs in C/D use the same lifetime premise.
- **Copying between worktrees.** `bootstrap-host --from <checkout>` copies a matching generation
  instead of building; [59e E01](59e-environment-and-adoption.md#e01) uses it for worktrees.
- **Automatic rebuild** (R3; operator decision 2026-10-29).
  - When a top-level verb launching new work finds the host stale, it rebuilds it through
    `bootstrap-host`, then re-execs the new generation before proceeding.
  - **If the rebuild fails, launching new work fails with the build error.** No older
    generation launches that work. Separate worktrees do not protect control of work already
    launched from the now-broken checkout.
  - **Existing-invocation exception** (operator confirmed 2026-10-29): read, wait and cancel
    use a compatible prepared control protocol and the invocation's recorded generation,
    without rebuilding current sources. Resolve/version-check the record first; authenticate
    the generation against the prepared tool root/manifest and unit identity before mutation.
    Never execute an arbitrary recorded path. Unsupported protocols refuse explicitly and
    identify the documented compatible-tool recovery route without weakening stop authority.
  - **Where it never rebuilds:**
    - read-only verbs (`-h`, `status`, `explain`, `env --explain`/`--print`, `doctor`, `schema`,
      `commands`), which report `host.stale_tools` instead;
    - `cargo-pse-correctness` adapter mode, which runs inside Cargo while Cargo holds the build
      lock;
    - `agent-hook`, with its seconds-long timeout;
    - the internal `__foreground-exec`, `__supervise` and `__finalize`;
    - any call nested inside a workload scope.
  - **Build coherence and re-exec.** Re-exec happens at most once, guarded by an environment
    marker. Compare the complete fingerprint before/after construction and before atomic
    publication (or build from a fixed source snapshot). If inputs changed, do not publish or
    launch under a misattributed fingerprint; return a retryable stale-build diagnostic.
    The marker prevents loops; it is not evidence that the binary matches its declared source.
- **Host ceiling.** `[workspace.metadata.pse.dependency-ceilings.host]` declares
  `roots = ["pse-host"]` and forbids `pse-engine`, `pse-runtime`, `pse-operations`, `arrow*`,
  `datafusion*`, `parquet`, `pse-schema` and `surrealdb*`. `just family-check` enforces it.

**Acceptance:**
- `tests/bootstrap.rs` gains pruning with recorded and unrecorded long users, delayed finalizer,
  fingerprint-match copy, fingerprint-mismatch rebuild and source-change-during-build cases.
- **Stale host.** A stale generation is reported by `pse status`, rebuilt by `pse run -- true`,
  and not rebuilt in adapter mode or from a hook.
- **Failed rebuild.** With a host source that does not compile, `pse run -- true` fails with the
  compiler error, and `current` still points to the previous generation, unused.
- **Independent recovery.** Start a detached invocation, break current host compilation, then
  read/wait/cancel from another process using its prepared generation with no rebuild. Original
  resources settle; mismatched invocation or incompatible record refuses. A forged executable
  path is never invoked. Pruning cannot remove that control/supervisor/finalizer generation.
- `just family-check` passes with the new row.

<a id="a06"></a>
### A06 — Derived catalog, explain, provenance and status

**Consumes:** A03, A04.

**Delivers.**
- **Catalog.**
  - `pse commands --json` and `pse help --json <path…>` come from the shared `#[path]` walker
    over `Cli::command()`. Full export calls `build()` and may expand deferred subtrees; scoped
    command-path help does not hydrate unrelated candidate domains. For each leaf it records arguments, possible
    values with help, env mirrors, defaults, groups and conflicts, rendered `-h`/`--help` text,
    examples and output-schema IDs. Derive available `requires`/group/conflict metadata from
    the authoritative parser; opaque/private rules remain explicitly unavailable. Do not
    redeclare validation constraints in extension metadata. Descriptive examples and schema
    associations may use `unstable-ext` `Command::add`/`Arg::add` where no existing declaration
    supplies them.
  - Scoped command-path and argument-candidate queries accept prefix and page limits as well
    as full export. Candidate records name source, freshness, continuation and exhaustiveness;
    unknown metadata is not an empty valid domain, and a custom parser may accept infinitely
    many values. `valid_values` come from the CLI's own value-domain providers: packages and targets from
    cached metadata, run IDs from the store, fixtures and capability sets from their
    declarations. Providers are fast and side-effect-free; there is no shell completion.
- **Federated xtask subtree.**
  - `just codegen` gains an `xtask commands --json` export into a generated schema file, using
    the same walker and recording each verb's features and invocation route.
  - The export describes discovery only and carries executable/source/schema version identity.
    It cannot reconstruct arbitrary clap validators, typed parsers or dynamic semantics.
  - Use narrow external-subcommand forwarding to the prepared owner's authoritative parser
    by default. Before adding shared parser code or a richer federation, compare its concrete
    consumer benefit and dependency burden with that forwarding path. Exit this investigation
    with a selected dispatch route and direct/delegated rejection cases, not a second parser.
    Unknown/stale association refuses dispatch or refreshes explicitly before effects.
  - Dispatching an xtask verb is new effectful work: select/build through the correctness
    adapter and execute its immutable pinned output. Schema publication remains explicit
    development work; help/status works without prepared richer tools.
- **Explain and provenance.**
  - `pse explain <verb…>` and a global `--explain` print the resolved plan without effects:
    underlying commands, placement, capabilities, features, fixtures and environment changes.
  - Explain and execution consume one resolved operation value, including C07's resolved test
    selection when applicable; changed external premises require explicit re-resolution.
  - `pse env --explain [KEY]` reports repository default, pending local configuration, captured
    caller environment or command line. `ArgMatches::value_source` distinguishes explicit CLI
    input/default/env, not manual export from direnv inheritance. PATH includes contributing
    layer context; values are redacted where secret-bearing.
  - `explain` never sources shell. Unevaluated `.envrc.local` values are reported as pending,
    using the existing `local_pending`.
  - Assess Figment `Provider`/`Metadata`, merge/profile and typed extraction only for a typed
    settings subset where they replace actual manual merging. Exit with a chosen subset (or
    explicit no-adoption rationale) and precedence tests for false/empty/absent values. Build
    providers from captured inputs; built-in Env's global/lossy reads cannot replace byte-exact
    `Environment`/`OsString` activation. No change to the separate direnv environment effort.
- **`pse status [--brief|--line|--all]`** reports:
  - HEAD and dirty count;
  - the active plan pointer from `docs/plans/README.md` links;
  - live and recent runs (from 59b once present);
  - stale host;
  - checkout ID, and other worktrees with `--all`.
  - Product observations/canonical links join only through F01/F02's explicit associations,
    retaining source, freshness and unavailable coverage rather than inferring scientific state.

**Acceptance:**
- The catalog lists every non-hidden verb with rendered help.
- `valid_values` for `pse test`'s target argument lists real `pkg:target` pairs.
- Scoped queries work with absent metadata and report unknown/non-exhaustive domains honestly.
- Direct/delegated typed parser cases reject the same invalid input; stale discovery cannot
  silently dispatch a different executable/schema meaning.
- `explain` output matches the command `pse test` then runs, compared on the same inputs.
- `env --explain` attributes a command-line override to `CommandLine` and a default to
  `DefaultValue`.
- Captured environment, pending local settings, derived PATH, explicit false/empty and secret
  redaction retain honest provenance; explain does not build, source shell or prepare fixtures.

## Verification and handoff

### Code-intelligence foundation extension

The coordinator's confirmed RC01–RC05 extend these existing packages for G–I:

- **A07:** route changed source/result/proposal and reusable-session contracts through their
  governing owners before dependent code; new crates/services use the scoped decision route.
- **A03:** provider/context, analysis generation, availability, source coordinates and typed
  next actions survive projection. Distinguish CLI positions, byte offsets and negotiated LSP
  encoding with Unicode controls. Hover/debug text stays rendered when no typed field exists.
  Unknown/no-match/unprepared/stale/canceled/truncated are distinct. Roughly 800–1,200 tokens
  is a combined-answer presentation aim, not a tokenizer dependency or serialized-byte limit.
- **A04:** use the existing resolver for analysis/artifacts; derived caches are distinct from
  authored probes, edit intent and original evidence. G owns analysis reuse; D02 API acquisition.
- **A05:** actual users include analyzer/proc-macro children and separately prepared compiler
  tools. Protect their generations; help/status/source-only reads and existing-work control
  never prepare analysis. Preserve the host dependency ceiling and independent cold startup.
- **A06:** derive new query discovery/schemas from owning parsers and actual provider support.

Acceptance adds ambiguous crate contexts, Unicode positions, unavailable preparation, budgeted
responses retaining mandatory facts and active-session generation retention. These extend
existing package acceptance, not a second output/store framework. G01 design can precede working
foundations; production consumers require their actual slices.

Per AGENTS.md:
- **During packages:** compile checks (`just check-package pse-host`) and targeted tests (`just
  unit-package pse-host <filter>`, plus the named `tests/*.rs`).
- **At series scope end, once:** `just hygiene` and the integration journeys.

Snapshots use `insta`; executable `trycmd` cases are deferred until help examples stabilize.
59b, 59c, 59d and 59f consume A03/A04; C07 supplies resolved test selection to A06 and B02.
59e consumes A05/A06, including live-user protections before worktree removal. Establish A07,
A03, A04 and A05's control/pin slice before the small B01/B03 invocation journey; complete
catalogue federation independently of that journey. Product observation depends on F01's own
prepared owner, not host dependency-ceiling expansion. All acceptance above is planned; this
documentation amendment runs no product checks and grants no production execution restart.
