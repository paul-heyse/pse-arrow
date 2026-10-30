---
title: "Plan59a: CLI foundation"
status: active
date: 2026-10-30
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md]
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

The [library-integration review](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md)
I01–I07/I12–I14 now amend A03–A06's substantive targets. Its RC01–RC04 were accepted by the
operator on 2026-10-30; [A08](#a08) publishes the affected rules before changed contracts are
implemented. A07's completed publication remains historical completion of its original scope.
The new work below is planned; this documentation revision does not resume production work.

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

The 2026-10-30 assessment at `09e878957` identifies unfinished foundations to correct, rather
than assuming the initial reusable helpers already enforce the target. `InvocationStore::resolve`
currently scans all history before exact lookup; `InvocationDirectory::publish_result` accepts
a generic envelope whose validation checks identity/version/presentation but not settlement or
reconciliation. `OutputOptions::emit` and `presentation::write` generate schemas twice, and
generic array budgeting misses nested producer detail. `GenerationUse::acquire` durably records
every prepared entry. Existing immutable publication, original-byte artifacts, diagnostic
structure and raw protocol bypasses remain useful; their prior focused tests establish only
their named baseline behavior, not the revised targets.

### Staged resolution and producer-owned answers

Keep parsed intent, resolved plan, admitted operation and observed execution distinct (I01).
Each operation owner resolves policy from captured argv/environment/configuration once for that
stage; explain and execution consume the same stage value. Explain reports absent preparation
as a prerequisite without preparing a fixture/provider, discovering an actual unlisted test
population or eagerly resolving every provider. Bind the relevant source/configuration/tool
premises; changed premises require explicit re-resolution or refusal before effects. C07 owns
test selection and product owners retain scientific admission/results. This is composition of
operation values, not a universal operation registry.

Producers own meaningful detail modes and pageable units (I07/I13). Source answers select
declarations with enclosing context, failure answers select original diagnostics/spans, and
product answers select their own observations. They expose typed pages, omitted/unexamined
sections, known or unknown totals, and continuations bound to input identity and backend
context. [59j J01](59j-source-queries-and-change-proposals.md#j01) owns addressed source/continuation contracts;
J02 owns shared edit proposals. A03 carries their wire views without inventing another source
or edit model. A stale continuation refuses or reads deliberately retained immutable inputs;
fresh-query and exact-original-source continuations have distinct promises. Dirty source bytes
and sole diagnostic evidence need their actual retention owner, not a generic rebuildable-cache
assumption. Reconstructible derived pages do not require retaining a huge full response on
every limited request. Generic fields/budget project these producer results mechanically and
preserve indivisible records and mandatory state, identity and coverage; they do not recursively
truncate tuples/tagged records or implement a JSON query language.

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
| [A03](#a03) | in-progress | 59a | Native output/diagnostic/schema, artifact read/continuation and typed source views integrated; producer-owned pages, typed direct serialization and schema corrections newly planned; run/test/product owner enforcement remains |
| [A04](#a04) | in-progress | 59a | Resolver/consumer migration integrated with targeted tests; exact lookup isolation, partial-history contract and responsibility-specific retention newly planned; concurrent invocation acceptance remains with B01 |
| [A05](#a05) | in-progress | 59a | Immutable publication, existing durable entry pins, stale rebuild/reexec and authenticated cold-copy integrated; cold copy passed without destination compilation; effect-shaped promotion investigation and actual descendant/finalizer settlement remain; pruning disabled |
| [A06](#a06) | in-progress | 59a | Parser-derived command/help/catalog/schema and passive native status integrated; staged shared explain, selective acquisition, actual provenance and owner federation remain planned |
| [A07](#a07) | complete | 59a | Proposed ADR-0233/blueprint315 published and scoped formal review accepts that target at Proposed strength; proposed ADR-0234/blueprint316 refines nested invocation ownership; runtime obligations remain with consumers |
| [A08](#a08) | complete | 59a | Proposed ADR-0233/0234 and blueprint317 publish accepted RC01–RC04; independent focused target advice found no required correction, not formal acceptance or runtime qualification |

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

This package's historical completion applies to that published scope and its recorded review
strength. The 2026-10-30 library-integration amendments are owned by A08, not credited to A07.

<a id="a08"></a>
### A08 — Publish the accepted library-integration rule amendments

**Consumes:** the library-integration review RC01–RC04, operator-accepted 2026-10-30, and the
existing decision/review route. The coordinator owns the single series disposition table.

**Delivers:** amend proposed ADR-0233/0234 where their contracts change, and route changed
blueprint §21.7/§24.1 owners through the required `design:` amendment and revision row.
Accepted records are not edited. A governance/SHOULD/commit-contract change uses its ADR and
design-review route; ordinary implementation choices within published contracts need no new
decision ceremony. Publish only the affected prerequisite slice before its dependent effects:

| Accepted impact | Required publication and consumers |
| --- | --- |
| RC01 | Effect-shaped generation protection and conservative promotion investigation in A05/B03/E01/G02 replace the every-entry durable-record target; existing unresolved claims remain protected and pruning stays disabled until actual users are qualified. |
| RC02 | Independent exact raw streams and optional attributed offset index replace merged retained chunks in A03/A04/B01/B02/B03/F03, preserving adapter protocols and original evidence. |
| RC03 | Producer-owned pageable wire views and the shared source-bound proposal may change immature shapes. A03/A06 and J01/J02 migrate their D/F/G/H/I consumers and versioned schema associations together, deleting replaced shape handling without a compatibility shim. |
| RC04 | J03's syntax query/preview route consumes J01/J02 without semantic-session readiness; amend the G07 dependency and affected native lifecycle bindings. G03 remains necessary only for semantic editing. |

**Acceptance:** governing owners and native dependencies agree with the accepted impacts;
the coordinator records publication readiness and any decision still outstanding. Plan wording
and operator adoption do not establish ADR acceptance, implemented contracts or qualification.
Do not gate B01/B03's basic managed custody on all of J: only a consumer's changed source or
proposal contract waits for the corresponding J01/J02 slice. No production or ADR changes are
made by this plan-authoring revision.

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

**Consumes:** A02, A07's published identity/result/diagnostic contract and A08's affected
page/raw-evidence wire amendments before changing those contracts.

**Delivers.** The [O02](../functional_reviews/rust-agent-cli-2026-10-29.md#O02) contract:
- **Format and output.**
  - Global `--format auto|human|json|ndjson` (`PSE_FORMAT`), with the format decided before
    parsing completes, from the environment variable, TTY detection or an argv pre-scan.
  - A versioned envelope with summary first, typed `invocation_id`, explicit scientific and
    observation references, command outcome, subject state, `child_exit`, settlement,
    observation completeness, `failures`, `artifacts` and `next`. Unknown/unavailable facts
    stay explicit. A CLI invocation never certifies a scientific result by itself.
  - `--fields` projects only the payload. Version, identity, state and coverage remain present;
    omitted/projected detail is explicit and has a matching typed view schema. `--budget`
    limits whole producer-defined records/pages, never serialized bytes or indivisible nested
    domain values. Producers select detail during acquisition where possible. Larger answers
    expose context-bound continuations or selected original artifacts; a limited query alone
    does not require serialization/retention of its entire rebuildable result.
  - Compact NDJSON emits one normalized record per line and a final observation summary with
    explicit terminal/nonterminal subject state. Opaque `RawValue` retains original duplicate
    keys/numeric lexemes and may contain newlines: retain exact original bytes separately and
    encode them safely for single-line transport, never splice multiline bytes into NDJSON.
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
  Normalized failures/outcomes and producer pages are typed; B02 independently qualifies the
  actual extractor streams. Derive owned serialize/deserialize views from their Serde/Schemars
  types, distinguishing defaults/skips/custom conversions by direction. Decode domain inputs
  through checked constructors (`serde(try_from)` where appropriate); schema validity, decode
  success and valid operation construction remain separate claims. Use native `Schema::pointer`
  for URI-fragment references, preserving percent decoding. Prefer derived view types and
  narrow native transforms over a general interpreter for `properties/type/$ref`.
  Generate a schema once for an emission, and serialize borrowed typed pages directly rather
  than typed→RawValue→outer-frame conversion. Keep private frame/spool preparation where
  necessary to avoid emitting a partial record on serialization failure. Retain RawValue for
  genuinely opaque upstream evidence. The owning numerical producer establishes finite/absence
  meaning before serialization; nonfinite-to-null is not a validity policy.
- **Addressed artifact reads (I12).** Keep `object_store` for immutable addressed evidence:
  native `GetOptions`/`GetRange`, streamed payloads and validators support selected tails,
  windows and exports with preserved byte offsets/encoding. ETag, version and content hash
  remain distinct. Local mutable reads retain `atomic_snapshot=false`; unsupported path keys
  refuse. Preserve private inode/UID/no-follow publication and Git transactions with their
  existing owners: the pinned local backend rejects `PutMode::Update` and follows symlinks,
  so generic conditional-write documentation is no local CAS guarantee. No remote transfer or
  object_store family move is selected by this plan revision.
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
- Nested outline/batch pages preserve whole declarations, enclosing context, omitted detail and
  unknown totals under a small budget. Changed-input and failed continuations refuse honestly;
  no arbitrary tuple/tag truncation is accepted by merely weakening its schema.
- Directional/custom-serializer schema agreement, percent-encoded schema references, unknown
  versions and malformed domain input exercise the proper constructor. An invalid terminal
  outcome combination cannot acquire authority merely by passing generic envelope validation;
  B01/B03 enforce that boundary before A03 renders the owner result.
- Artifact tails/windows preserve offsets and incomplete reads; changed validators and
  mid-stream failure refuse a false full read. Sole original evidence/authored inputs retain
  their owner-specific protection rather than generic cache disposal.
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
- There is no separate writable history index. A full ID validates and opens only its bound
  directory/record. Prefix lookup enumerates identities to establish ambiguity before opening
  the chosen metadata; a corrupt competing identity cannot be silently hidden. History/latest
  are collection queries over `invocations/*/meta.json`, with declared ordering, per-record
  diagnostics and explicit partial coverage. Incomplete scans never claim global latest;
  session fallback requires trustworthy coverage. The `run` verb remains an invocation launcher. B01 owns exclusive
  metadata/state publication and supervisor/finalizer races, using temp file, fsync and
  `renameat_with(…, NOREPLACE)` for final files.
- Retention classifies regenerable preparation, disposable presentation copies, sole original
  diagnostic evidence, uniquely authored probes, mutable Git intent, scientific data and
  uncertain fixture evidence separately. Original logs are not disposable merely because the
  invocation is terminal. Age/count applies only to
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
- Healthy exact access succeeds despite unrelated corrupt history; ambiguous prefixes refuse
  before selecting a readable candidate. History retains individual failures and partial
  coverage, and an uncertain latest selection is not labeled complete.

<a id="a05"></a>
### A05 — Host-tool lifecycle: fingerprint, pruning, stale rebuild

**Consumes:** A02, A04, A07's R3/control amendment and A08's RC01 publication. Generation-use protection must be
available to B01/B03 and C's immutable selections before pruning is enabled.

**Delivers.**
- **Fingerprint.**
  - At build time, `pse-host` computes its path-dependency closure from `cargo_metadata`: today
    `pse-host`, `pse-diagnostics`, `pse-buildinfo` with `build.rs`, and `pse-ids`.
  - It stores that path list, plus `Cargo.lock`, the root `Cargo.toml`, `rust-toolchain.toml` and
    `.cargo/config.toml`, with a hash in the generation.
  - A staleness check rehashes only the stored paths. The closure is not declared again in shell.
- **Generation protection and promotion investigation (I04).** Separate temporary passive-read
  protection from durable obligations for retained effects: foreground/nested children,
  unrecorded exec adapters, Cargo's exported `CARGO`/`PSE_CORRECTNESS_EXECUTABLE`, analyzer
  children, supervisor/finalizer and exported executable paths. Investigate a process-held
  shared generation guard under the same exclusion as publication/pruning, promoted durably
  before effects/path handoff. Qualify descriptor inheritance, atomic promotion, crash recovery
  and concurrent pruning before selecting that mechanism; an ephemeral lock alone does not
  prove descendant settlement. Exit with a selected protocol and its limits or keep conservative
  protection and pruning disabled. Existing unresolved claims remain protective; vanished PIDs
  and ledger absence do not authorize removal or retroactively establish their cause.
  Release durable obligations only after actual users/children and required finalization settle.
  Under the selected exclusion, pruning must preserve actual users plus current/previous
  convenience retention. Pinned C/D/J helper outputs share the actual lifetime premise.
  Distinguish source-equivalence checks, prepared-manifest integrity and live-use protection:
  reuse immutable integrity only within an explicit trust/mutation boundary, never substitute
  mtime for content validity or hash the entire repository on every brief query. Replace
  every-entry durable bookkeeping and its mechanism tests once the chosen consumers migrate.
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
- **Promotion investigation.** Killed passive reads, child/path export, source-copy handoff,
  promotion interruption and concurrent pruning challenge the chosen guard. Existing unknown
  claims remain protected in every case; pruning remains disabled while any necessary premise
  is unresolved. Retain individual original errors rather than infer settlement from process absence.
- `just family-check` passes with the new row.

<a id="a06"></a>
### A06 — Derived catalog, explain, provenance and status

**Consumes:** A03, A04.

**Git mechanism refinement (2026-10-30).** Use pinned `gix` for typed repository inspection,
sharing the native observation owner with D01/C03/E01. Read HEAD, staging and tracked/untracked
working-tree changes as separate facts; preserve byte paths and sort parallel status events.
Include individual untracked files rather than using `is_dirty()`, which excludes them.
Do not publish index refreshes during observation. Linked-worktree common-directory discovery
already uses isolated Gitoxide opening at the explicit checkout and preserves existing identity
spelling. A live status view is not an atomic filesystem snapshot. The library's complete
[capability overview](../libraries/gix.md) and exact-release APIs guide subsequent consumers.

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
  - Explain and execution consume the same operation-owned value at the selected semantic stage:
    parsed intent → resolved plan → admitted operation → observed execution. Capture relevant
    premises and use C07's resolved test selection where available; actual listing/admission
    remain later stages. Missing provider/fixture preparation appears as a prerequisite.
    Changed premises require explicit re-resolution/refusal before effects, not eager setup.
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
  - Select acquisition before reads: brief/line request only useful Git/freshness/invocation
    facts and never decode every historical result to discard them later. Explicit full
    inspection selects wider sources and retains partial-history diagnostics. Omitted and
    unexamined sources are visible; small output does not claim a cheap global scan.

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
- Explain→execute agrees at the same stage, explicit-default and implicit-default values retain
  distinct origins, and a changed source/configuration premise refuses before effects. Brief
  status does not read every terminal result; malformed unrelated history cannot block exact
  existing-invocation access. Delete duplicate explain/execution policy interpretation and
  duplicate schema generation with the migrated callers/tests.

## Verification and handoff

### Code-intelligence foundation extension

The earlier code-intelligence review's confirmed RC01–RC05 extend these existing packages for
G–I. The 2026-10-30 library-integration review has separate RC01–RC04 identities and A08 scope:

- **A07/A08:** retain A07's delivered scope; A08 routes newly changed source/result/proposal
  and lifetime contracts through their governing owners before dependent code. New crates/
  services use the scoped decision route.
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
59e consumes A05/A06, including live-user protections before worktree removal. A07 is delivered;
publish A08's affected amendments and establish A03/A04/A05's required owner-result/control/
protection slice before the small B01/B03 invocation journey. Pruning and full catalogue
federation are not prerequisites to that journey. J01 supplies source pages, J02 shared
proposals, J03 server-free query/preview, J04 generated navigation and J05 their integrated
consumer; A03/A06 participate by the required typed-view/discovery slice rather than treating
all J as a foundation gate. Product observation depends on F01's own
prepared owner, not host dependency-ceiling expansion. All acceptance above is planned; this
documentation amendment runs no product checks and grants no production execution restart.
