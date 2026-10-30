---
title: "Plan59b: Runs and continuity"
status: active
date: 2026-10-30
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md]
---

# Runs and continuity

Part of [Plan59](59-agent-cli.md).

This plan gives agents owned CLI invocations instead of `/tmp` logs polled with `tail` and `ps`.
The `run` verb remains the launcher; its invocation identity is distinct from scientific runs,
attempts and observations. Specifically:
- a run ledger;
- foreground and detached execution;
- `wait`, `result`, `log`, `cancel` and `history`;
- composite bundle runs;
- extraction of cargo and nextest failures;
- Claude's compaction handoff.

**Sources.** [O03](../functional_reviews/rust-agent-cli-2026-10-29.md#O03),
[O20](../functional_reviews/rust-agent-cli-2026-10-29.md#O20) (Claude only), and the run part of
[O24](../functional_reviews/rust-agent-cli-2026-10-29.md#O24).
The [observability review](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md)
O01/O03/O05–O07 and [improvement review](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md)
I01–I08/I10/I11 refine identity, result, control, output and handoff. The coordinator owns
dispositions; this document schedules its part. Production execution remains stopped.

The [library-integration review](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md)
I01–I07/I12–I14 amend the substantive target below: staged selection, owner-enforced finality,
effect-shaped generation protection, independent raw streams, selective invocation lookup,
native observation and producer-owned pages. Its RC01–RC04 were operator-accepted 2026-10-30;
[A08](59a-cli-foundation.md#a08) owns their prerequisite publication. These amendments are
planned work, preserving completed primitives and the paused production boundary.

**Why it matters.** Over one day in the review's measured window:
- 491 runs were backgrounded, writing 386 ad hoc log paths.
- 29% of the runs whose result was observed failed, and most were found only by reading logs.
- Observing runs consumed 21% of output tokens.

**Rule basis.** [R2](59-agent-cli.md#rule-changes) allows owned-run supervision.
[R3](59-agent-cli.md#rule-changes) allows the per-run supervisor and makes the ledger the stop
authority for detached runs.
The operator confirmed the R3 existing-invocation control exception on 2026-10-29; its governing
amendment is [A07](59a-cli-foundation.md#a07), rather than evidence already supplied by A01.

## Baseline

Facts from a code map at `68e075493`.

**Foreground execution** (`foreground::run_owned`):
- Creates an owner scope, an operation slice and a workload scope.
- Binds the workload to the caller with `BindsTo` and pidfds.
- Releases a gate pipe only after binding.
- Settles all three units concurrently under a drain deadline.
- Re-execs `/proc/self/exe __foreground-exec` as a trampoline.
- Child stdio is inherited.

It is caller-bound by design, so it cannot outlive the caller.

**Transient services** (`manager::start_service`, `start_service_inner`):
- Set `AddRef`, `Type=exec`, `KillMode=control-group`, `RemainAfterExit=true`,
  `CollectMode=inactive-or-failed`, `ExecStartEx` and `Environment`.
- Have no `ExecStopPost`, `WorkingDirectory`, `Slice` (unbound) or standard-output file.
- Use `LaunchIntent::fresh()` to name units `pse-host-{pid}-{nanos}-{seq}.service` with a nonce
  description.

**Ownership and identity.**
- Reconciliation compares the exact description and the 16-byte invocation ID.
- Destructive actions require the caller's bus name in `Unit.Refs`, which a later process cannot
  have.

**Observation.**
- `HostManager::observe` and `BorrowedUnit::observe` read state, PID, exit code and control group.
- `OwnedUnit::{kill_and_drain, stop_and_drain}` wait for cgroup emptiness.
- `Group` reads cgroup files, but not `memory.current` or `cpu.stat`.

**Admission.**
- `Coordinator::{open, ordinary}`, `admit`, `admit_ordinary` and `Permit::{prepare, bind, release}`
  cover finite and unbounded classes.
- Permits bind to a foreground `WorkloadScope`.

**Correctness adapter.**
- It spawns `cargo-nextest` with inherited stdio.
- `--cargo-message-format json` and `--message-format libtest-json-plus` pass `reject_reuse`
  unchanged. `-R` and `--rerun` are refused.
- nextest's libtest JSON needs `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1`.

**Bundles.**
- `just turn-end`, `ready` and `hygiene` call `just run-all <recipes…>`, which keeps going after a
  failure and prints `Failed recipes:`.

The library-integration assessment at `09e878957` and the package table below distinguish that
historical map from the paused implementation. Immutable selection/result publication,
independent pipe readers, strict Cargo decoding and service-main/readiness primitives exist.
The shared retained-chunk sink, history-wide exact lookup and generic envelope terminal
publication still need the amended mechanisms below. The assembled service supervisor/
finalizer, native resource settlement and nextest reconciliation are not established by those
component tests. Their original evidence remains at the coordinator checkpoint.

## Target

A **CLI invocation** is one supervised command with a typed `CliInvocationId`, a directory in the
checkout store ([59a A04](59a-cli-foundation.md#a04)) and an invocation result. Its file ledger
owns launch/selection, command/process outcomes and settlement evidence. Canonical scientific
run/attempt owners retain scientific acceptance and fixture owners retain disposal decisions;
their exact references are linked, never inferred from a successful child or ledger record.
Observations retain their own operation/process/source identities and coverage. One invocation
can contain many attempts, and several observer invocations can inspect the same science.
Systemd supplies detachment, cgroups and kill policy, not scientific authority (I01/I03).

```text
<store>/invocations/<invocation-id>/
  meta.json       version, invocation_id, command, cwd, environment digest, immutable tools/generation,
                  placement, kind, unit name + systemd invocation ID, session, parent invocation,
                  selection stage/premises, explicit scientific/observation associations
  stdout.raw      exact original stdout bytes, stream-local offsets and retained prefix on failure
  stderr.raw      exact original stderr bytes, independently written/drained and synchronized
  stream-index    optional compact stream/offset observations when interleaving is requested
  events.ndjson   normalized invocation events, ending in an observation summary
  result.json     command/process outcome, subject state, settlement, observation completeness,
                  child_exit, typed failures, counts, durations, artifacts — published once (NOREPLACE)
```

**CLI invocation IDs** are generated by the launcher before anything starts. The implemented
representation is a validated UUIDv7 (time and random identity), with inspected checkout and
repository IDs bound and verified separately in immutable metadata. This preserves checkout
provenance and uniqueness without making an invocation ID a storage-path identity. The distinct systemd invocation ID is recorded once the unit
exists. It cannot be the salt, because the CLI ID is already in `ExecStartEx`. A full ID validates
and opens only its own bound directory/record. A prefix enumerates candidate identities and
establishes ambiguity before opening chosen metadata; unreadable matching records do not justify
silently choosing another. History and `latest` (per session, then per checkout) are collection
queries with declared ordering, per-record diagnostics and explicit partial coverage. Uncertain
coverage cannot certify global latest or an unjustified session fallback. Avoid a second writable
history index. Command arguments historically called `<run>` select
this CLI identity; they do not select canonical scientific `RunId`.

**Finality and settlement.** A successful managed response means the command fulfilled its
contract. Launch acknowledgement and a nonterminal bounded wait may exit 0 with explicit state;
neither is a reconciled pass. Child reap, supervisor termination, operation/workload/owner scope
drain, permit release, scientific acceptance and fixture disposal are separate facts. Preserve
`foreground::run_owned`'s full settlement obligation and attempt every independent cleanup after
an error, retaining the original operation failure and every cleanup failure. A finalizer that
only knows systemd's supervisor result publishes failed/unknown settlement and an incomplete
subject, never ordinary completed work. Result publication is exclusive between supervisor and
finalizer; once published it is an immutable statement of the evidence then available. Any later
authenticated recovery evidence is separately attributed by the actual settlement owner.

B01/B03 must enforce this at the owner boundary (I13). The baseline
`InvocationDirectory::publish_result` accepts a generic envelope and validates binding/version,
non-running state and absence of projection; that does not establish terminal child/reconciliation
evidence. Construct a valid terminal owner result from original process, capture, reconciliation
and settlement facts before exclusive publication, and project that result into A03's envelope.
The generic renderer/schema is not a terminal admission authority. Do not merge all product
states into one universal enum: scientific/fixture owners keep their own validity operations.
Launch acknowledgement and observation timeout remain typed nonterminal responses. Targeted
constructor tests reject invalid combinations even when their JSON/schema shape is valid.

Selection follows A06's stage boundary (I01): preserve parsed intent, resolved plan, admitted
operation and captured execution population with their premises. Explain and execution consume
the same stage value, but explain cannot prepare unavailable tools/fixtures or invent actual
nextest listing. Changed premises re-resolve/refuse before effects. C07/C02 supply selected test
population and required terminal rules when that consumer joins the generic custody path.

**Execution modes:**

| Mode | Mechanism | Lifetime | Use |
| --- | --- | --- | --- |
| Foreground (default) | The same B03 service supervisor/finalizer as detached execution; `pse` observes its attributed output/result. Establish the authenticated origin pidfd/start-identity lease before payload release. | Bound to origin life: its disappearance cancels the owned invocation, including escaped descendants. | Managed executing verbs by default. Passive/help/source reads remain direct. An explicit workload deadline bounds execution; `pse wait --timeout` bounds observation only. |
| Detached (`--detach`) | A transient **user service** whose main process, `__supervise`, owns the child placement/admission/settlement path with piped output. B03 first makes and verifies that path from an authenticated service context. Returns `launched` plus the invocation ID. | Outlives the caller. Supervisor lifetime is bounded by completion, `RuntimeMaxUSec` or `pse cancel`; actual workload settlement is recorded separately. | Long builds and tests, delegated workers, bundles. |

**Detached property set.** It does not reuse `start_service_inner`, because `RemainAfterExit=true`
would leave the unit active(exited) and the finalizer would never run.
- `Type=notify`, `NotifyAccess=main`, `RemainAfterExit=false`, `KillMode=control-group`.
  The service sends native readiness only after original invocation, origin lease (foreground),
  payload custody and control readiness are established. Observe the matching manager job/unit
  as acknowledgement; successful datagram transport alone is not startup completion.
- `ExecStartEx`: the pinned generation's `pse __supervise <invocation-id>`.
- `ExecStopPostEx`: that generation's `pse __finalize <invocation-id>`, which receives `$SERVICE_RESULT`, `$EXIT_CODE` and
  `$EXIT_STATUS`.
- `StandardOutputFileToAppend` and `StandardErrorFileToAppend` pointing at a separate
  `supervisor.log` in the run directory. The child's streams go to independent `stdout.raw`
  and `stderr.raw` writers, separate from supervisor diagnostics.
- `WorkingDirectory`, `Environment` and `RuntimeMaxUSec`.
- The usual `CollectMode`.

**Library-owned lifecycle target.** Use pinned zbus/
zbus_systemd for transient-service/dependency/job state, `sd-notify` for native readiness/status,
and `tokio-graceful-shutdown` with a concrete host error type for named cooperative supervisor
subsystems. Tokio/tokio-util own cancellation tokens and the appropriate task joins/tracking;
independent errors remain collected, rather than fail-fast cleanup. Pipe/extractor custody must
survive the control task's shutdown request until physical cancellation and required output
observation have been attempted. `TaskTracker::close` does not close new-work admission;
the actual parent-launch owner still supplies that cutoff. Async abort/runtime shutdown does
not stop running blocking/Rayon work or establish external descendant drain.

The installed systemd255 path governs capability selection, independently of generated bindings
for262. Its `ExecStopPost` does not normally inherit fd-store descriptors (`EXEC_PASS_FDS` is
on the main start path, not ordinary stop-post). Keep fd-store adoption conditional on an actual
supported retention/recipient protocol; it cannot silently replace the finalizer's original
cgroup identity authentication. No automatic restart/retry is selected for possibly committed
operations. The actual notifier must reject an absent or unsupported notification endpoint
rather than treat `sd-notify`'s optional no-op as managed readiness.

**The supervisor** (`__supervise`):
- Runs the child through the established ownership path. Current `foreground_placement` accepts
  only scopes and `native_workload` authenticates scope relationships; service entry is not an
  existing passing path. B03 must implement the required authenticated service placement and
  prove full admission/settlement before detached delivery is usable.
- Follow [ADR-0234](../adr/0234-managed-invocation-subtree-ownership.md)'s proposed shared
  supervisor realization for both managed modes. Keep the service controller in its service
  cgroup, immutable original root distinct from actual membership, and nested payload groups
  beneath their enclosing invocation. Core-only delegation retains original aggregate limits;
  per-invocation memory readings may consequently be unavailable. A borrowed reservation is
  never released by the nested owner. Close parent launch authority before recursive drain,
  and qualify the origin-death lease and explicit finalizer cleanup of external payload groups.
- Holds its own handles with `kill_on_drop(true)`.
- Reads both pipes through independent owned readers/writers; narrower task populations use
  `JoinSet` or `TaskTracker` where appropriate, not both as duplicate registries. Required
  capture survives cancellation of a view/main control task through its explicit drain policy.
- Runs the kind-specific extractor.
- Honours a `CancellationToken` wired to `SIGTERM`.
- Constructs the valid terminal owner result, then writes `result.json` by temp file, fsync and
  `renameat_with(…, NOREPLACE)` in competition with the authenticated finalizer.

**Output and failure policy.** Separate pipe readers, bounded extractor state and presentation
queues keep slow terminal/NDJSON/optional telemetry consumers off the child-settlement path.
Replace the current shared `managed/streams.rs::State::accept` sink lock and retained base64
`LogChunk` NDJSON with exact per-stream files (I05). Each raw writer owns its offset and
durability/failure report; a stalled stdout sink cannot serialize stderr capture behind its
write. A required sink has a bounded failure/drain policy that can mark incomplete capture and
continue pipe discard, preserving every independent writer/reader/final-flush failure. Optional
attributed index entries carry stream/offset extents; their cross-pipe order is observation order,
never exact producer execution order. Omit the index when offsets suffice. JSON transport may
encode non-UTF8 bytes; it does not redefine raw retention. Tokio reads and cancellation-safe
framing, including tokio-util codecs where suitable, must preserve oversized/truncated original
records rather than silently discarding them under a default decoder policy.
Coalesce/drop optional observations with explicit sequence/loss coverage. Required test-verdict
records have an independent completeness path; overload, oversized records, disk-full or reader
failure marks reconciliation incomplete, never a pass. On log write failure retain the readable
prefix and failure, continue draining pipes to discard as necessary, and settle the original
child/resources. A broken presentation pipe stops that view, not scientific ownership; an
explicit caller cancellation still follows the owning foreground contract. Budgeting selects
whole normalized records/pages, never cuts JSON bytes or hides required state. Tokio tasks and
bounded Futures stream composition are mechanisms, not proof that dropped work has settled.

`pse log`, B02 extractors and F03 capture/follow migrate to exact stream/offset references in the
same replacement slice. Remove merged-chunk encoding/reconstruction and its mechanism tests
once these consumers move; no parallel production log format or compatibility shim remains.
Original diagnostic evidence is retained by its owner; terminality/age does not make the sole
original copy disposable. A04 classifies authored probes, Git intent, preparation and scientific
data separately. I12's `object_store` range/tail/stream reads serve immutable addressed artifacts,
not the protected mutable invocation ledger: local `PutMode::Update` is unsupported and generic
local storage follows symlinks, so no generic CAS or multi-file publication guarantee is inferred.
Bounded tails/windows/export preserve offsets, encoding and validator distinctions (ETag/version/
hash); local mutable reads report their non-atomic scope. Provider transfer remains separately
selected work, not an implicit storage migration.

**The finalizer** writes a result only if none exists. It records the systemd result for a
supervisor crash or `RuntimeMaxUSec` timeout. It must recover authenticated original scope and
permit identities to establish their settlement; otherwise it reports unknown/failed settlement.
Supervisor SIGKILL is not evidence that separately placed descendants drained. Preserve the
generation-use pin until finalization and actual protected users settle, including failure paths.

**Waiting:**
- `pse wait <invocation>` blocks until a result exists or its observation timeout elapses. The default
  timeout is below both harnesses' tool timeouts.
- It uses unit property changes (`Subscribe`, `JobRemoved`, `PropertiesChanged`) or a pidfd
  readiness wait on `ExecMainPID` as wakeups and rechecks the ledger/settlement evidence.
  A reaped supervisor is not a complete invocation or scientific result.
- With `--format ndjson` it prints one line per transition, which Claude's background tasks and
  Monitor can consume.
- A unit that vanished without a result is reported as `lost observation`, with a recovered
  systemd result only if available and explicit unknown settlement. Wait expiry is nonterminal,
  distinct from workload timeout, cancellation, incomplete result and actual completed success.

Native reconciliation remains with the existing manager owner (I03). Use typed zbus_systemd
list operations for summaries and interface-scoped `GetAll` for selected detail instead of
serial uncached scalar reads. Preserve original unit-incarnation fences: several interface
reads are not an atomic snapshot. One request/session-scoped owner can publish latest state via
Tokio `watch`, with bounded `broadcast` only for actual transition consumers. Lag/disconnect/
manager replacement requires explicit resynchronization and coverage; latest state is not
lossless history. Extend existing property/cgroup subscriptions instead of another polling
supervisor. Retain checkpoint polling for product sources with no push facility, and keep
native sd-notify readiness/status/stopping scoped to the authenticated service. Watchdog/socket
activation/fdstore remain conditional capabilities, not finalizer shortcuts or a resident daemon.

**Stop authority** ([R3](59-agent-cli.md#rule-changes)):
- `pse cancel <invocation>` validates the record/control version and prepared generation before
  reading the unit name and distinct systemd invocation ID from `meta.json`. Read/wait/cancel
  resolve existing work through A05's compatible recorded control generation without rebuilding
  current sources; unsupported versions expose the compatible prepared-tool recovery route.
- It reconciles the live unit through a new ledger-authority reconcile that checks only the exact
  description and invocation ID. The existing `HostManager::reconcile` calls `require_reference`,
  which needs the original bus client in `Unit.Refs` and so always fails for a later process.
- It stops the unit and drains the cgroup.
- A mismatched invocation ID refuses, because the unit is not the recorded run.
- It also follows the original separately placed resource identities to their owning settlement
  path. Stop acknowledgement, leader exit or an empty service cgroup alone cannot release an
  independently retained fixture. Report incomplete settlement if authenticated recovery fails.

**Extraction:**
- **cargo** (`check`, `build`, `clippy`). Run with plain `--message-format=json` and parse with
  the native `cargo_metadata::Message` Serde model. Decode each bounded complete record strictly:
  `Message::parse_stream` in pinned0.23.1 converts malformed JSON to `TextLine` and disables
  the recursion limit, which would hide missing required protocol evidence here.
  - From each `CompilerMessage`, build the one-line form from the primary span.
  - Deduplicate on (level, code, primary span, message).
  - Keep the full `rendered` text in the run directory.
- **nextest.** Set `NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1` and pass `--message-format
  libtest-json-plus --message-format-version 0.1` and `--cargo-message-format json`.
  - Split stdout records on `type` (libtest) versus `reason` (cargo).
  - Classify the exit with `NextestExitCode`.
  - Capture each failed test's inline output.
  - Reconcile the actual selected binary/package/target/test population, setup outcomes and
    terminal runner evidence from [C07](59c-tests-and-fixtures.md#c07)'s resolved selection and
    C02's result contract. Preserve features, ignored policy, zero matches and setup failure;
    a test name alone is not an identity across binaries.
  - Feature-detect the stream. If it is missing or incomplete, the exit class plus log tail
    is diagnostic fallback only; retain an incomplete managed outcome, never a reconciled pass.
  - Match the pinned0.9.146 runner's actual event naming against C05's inventory. Its
    `package::binary$test` event spelling omits target kind and is not `RustBinaryId`;
    same-named targets of different kinds can be ambiguous. Refuse an invented reconciliation
    where event context cannot disambiguate. Setup exit105, retries, ignored/filter policy
    and suite terminals retain their independent meanings. The stream is experimental and
    nextest-metadata0.15.0 supplies listing models, not a typed execution-event protocol.
- **Generic.** Exit, duration, log tail and any declared output files.
- The JSON passes through the adapter's inherited-stdio path unchanged. Confirming that in
  practice is part of B02's acceptance.
- Keep original upstream records opaque in retained artifacts; typed normalized failures and
  completeness drive the result. A03's Schemars/RawValue envelope schema does not validate
  cargo/nextest semantics. Verify stream compatibility against the selected pinned runner.

**Composite runs.**
- `pse run-all <recipe…>` replaces `just run-all`. Each child (`just <recipe>`) becomes a child
  invocation with its own ID and exact parent association. The parent's result names each
  failed or incomplete child and its first error, preserving command and settlement states.
  Keep going after a child failure and attempt every child's independent teardown; parent
  observation terminality cannot hide an unsettled child or decide its fixture disposal.
- `turn-end`, `ready` and `hygiene` keep their justfile definitions, so the justfile stays the
  authority for bundle membership. Their bodies call `pse run-all`.

**Viewing:**
- `pse status` (from [59a A06](59a-cli-foundation.md#a06)) lists live and recent runs across
  checkouts with cgroup `memory.current` and `cpu.stat`.
- `pse history [--session]` lists one session's runs and outcomes.
- `pse result <run> [--failures] [--fields …]` returns the envelope.
- `pse log <run> [--grep RE] [--tail N]` gives bounded raw output.
- Producers expose useful detail modes and whole-record pages: failure pages prioritize original
  diagnostics/related spans before optional raw output, history pages retain partial coverage,
  and log windows reference exact streams/offsets. A03 only mechanically projects these typed
  results. No view recursively clips a tagged result or hides mandatory settlement/completeness.
  Push selection into acquisition where possible; brief status does not read every terminal
  result. Continuations bind original evidence/selection context and refuse changed premises;
  [59j J01](59j-source-queries-and-change-proposals.md#j01) supplies source-bound continuations
  only where source/diagnostic navigation consumes them. Basic custody does not wait for all J.
- Product phases, scientific references and inspection statements come through
  [F01/F02](59f-simulation-observability-and-diagnostics.md#f01), using exact declared links and
  source coverage. [F03](59f-simulation-observability-and-diagnostics.md#f03) owns bounded
  product follow; ledger NDJSON describes invocation transitions, not numerical event replay.
  Neither cgroup samples nor silence imply deadlock/non-convergence. Capture/escalation stays
  an explicit product operation in F05, independent of viewing or CLI invocation success.

**Session identity:**
- An explicit `--session` or `PSE_SESSION` applies when given.
- B05 confirms whether the Claude harness exposes its session identifier to Bash commands, and
  uses it if so. Otherwise runs group by checkout, with handoff keyed by the hook payload's
  `session_id`.
- This plan does not change environment activation. The operator's concurrent environment work
  owns the per-command bridge, and the repository `SessionStart` environment hook has been
  removed in favour of a global adapter.
- No Codex change.

**Claude handoff** ([O20](../functional_reviews/rust-agent-cli-2026-10-29.md#O20)):
- `pse handoff [--next "…"] [--record|--show]` writes or prints `<store>/handoff/<session>.json`
  with:
  - HEAD and dirty paths;
  - live and recent runs and their outcomes;
  - open failures;
  - the plan pointer;
  - the agent's own "next" line.
- `.claude/settings.json` gains two context-only hooks:
  - a `PreCompact` hook running `scripts/pse-host agent-hook pre-compact`, which records the
    handoff;
  - a `SessionStart` hook running `scripts/pse-host agent-hook context`, which returns
    `pse handoff --show` as `additionalContext` when the start source is `compact`, and the
    [59e E02](59e-environment-and-adoption.md#e02) status brief otherwise.
- Keep the record compact. These hooks only add context. They never touch the environment.
- Handoff is a compact derived view plus authored next intent: link exact immutable invocation,
  selected artifact and authoritative plan/scientific references. Preserve open/incomplete
  failures, coverage and unknown settlement; do not create a second mutable status ledger or
  rerun finished work merely to reconstruct context. This carries O07/I10 into the consumer.

### Decisions with reasons

- **Foreground by default.** Agents get the envelope and live summary without having to choose a
  mode. Harness cancellation keeps working, which matters given the two editor-wide OOM incidents.
- **A user service rather than double-fork.** A double-forked process stays in the harness's
  cgroup and dies with it, and it gets neither `RuntimeMaxUSec` nor accounting. The service path
  reuses the existing zbus manager.
- **The invocation ledger rather than systemd as its record.** Successful units are
  garbage-collected and current properties flush failed results. The ledger survives both;
  the finalizer can record supervisor failure without inventing original workload settlement.
  Canonical scientific results remain with their own owner.
- **nextest libtest JSON over the record store.** The record store has no JSON `list`/`info` in
  0.9.146, its format changed in 0.9.150, and `-R latest` races between agents. It stays a human
  tool for `cargo nextest replay`.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [B01](#b01) | in-progress | 59b | Checkout-bound immutable selection/result store and independent pipe-reader primitives integrated with targeted tests; independent raw sinks replace the shared merged format, owner-finality enforcement, exact/partial lookup and joined CLI custody remain planned |
| [B02](#b02) | in-progress | 59b | Strict native Cargo diagnostic extraction integrated with five targeted tests including actual scratch compilation; nextest population/setup/terminal reconciliation and joined CLI extraction remain |
| [B03](#b03) | in-progress | 59b | Root/member and delegated payload primitives, original service-main authentication and native readiness/cooperative-task helpers integrated; serialized recovery stays test-only; joined custody, original settlement, observation fan-out and effect-shaped generation integration remain planned |
| [B04](#b04) | planned | 59b | `pse run-all` composite runs; `turn-end`/`ready`/`hygiene` bodies migrated; `just run-all` retired |
| [B05](#b05) | planned | 59b | Session identity; `pse handoff`; Claude context-only PreCompact and SessionStart hooks |

<a id="b01"></a>
### B01 — Ledger and foreground runs

**Consumes:** [59a A07](59a-cli-foundation.md#a07) (governing amendments),
[A03](59a-cli-foundation.md#a03) (identity, output, exit and finality),
[A04](59a-cli-foundation.md#a04) (store) and A05's generation-use protocol before launching
children that retain tool paths; A08 publishes the changed raw-capture/page contracts first.
Full catalogue federation and all J packages are not prerequisites.

**Delivers:**
- The independent raw-stream directory layout and selective typed ID resolution: direct full
  value, unambiguous identity prefix and explicitly covered `latest`/history, with per-record
  diagnostics and exact optional scientific/observation associations. Cooperating product
  association producers integrate in F01, rather than host inference from log strings.
- Ledger recording added to `pse run`, which [59a A02](59a-cli-foundation.md#a02) ports. It
  gains `[--kind K]`, uses B03's shared native service owner with piped and teed output, gives a
  bounded live summary and writes a terminal `result.json`. The in-process tee is reusable
  library code inside that supervisor, not a second independently qualified lifetime path.
- **Nested runs.** Inside an existing workload scope, `run_owned`'s nested path `exec`s the child
  and could not record it. `pse run` and the managed verbs therefore spawn and record the child
  there too. Only the `boundary` (pse-env) exec adapter keeps the `exec` pass-through, unrecorded,
  as today.
- `pse result`, `pse log` and `pse history`.
- An owner-finalization constructor consumes original process/capture/settlement facts and
  kind-owned reconciliation before granting terminal publication; A03 serializes that result.
  Preserve every cleanup failure alongside operation failures and explicit unknown settlement.
- Independent stdout/stderr drainage, bounded rendering/extractor state and explicit
  disk/oversized-record/slow-consumer policy above; terminal outcomes retain loss/completeness.
- Retention eligibility from A04. Enable pruning only after A05's selected effect-shaped protection and B03's
  uncertain-settlement/finalizer protections are available; never prune active owner evidence.

**Deletes:** merged raw `LogChunk` encoding/reconstruction, shared sink-lock capture and their
mechanism tests with migrated log/extractor/F03 consumers; history-wide exact resolution and
generic-envelope terminal admission are replaced with their callers/tests. Preserve raw exec
adapter grammar/exit behavior and opaque protocol evidence.

**Acceptance:**
- A foreground run of a command that writes to both streams and exits 3 yields `result.json`
  with `child_exit: 3`, exit 3 for the exec-adapter `run`, and byte-exact `stdout.raw`/`stderr.raw`.
- An interrupted foreground run (SIGINT) yields status `interrupted` and exit 130.
- `latest` resolves per session before per checkout.
- Slow/broken human or NDJSON readers and oversized lines cannot deadlock child/resource
  settlement or break framing. Full log storage retains the readable prefix and explicit
  incomplete evidence; required verdict loss cannot produce success.
- Operation failure plus independent cleanup failures all survive in the result. Child reap,
  scientific status, unknown drain and retained fixture disposal remain separate.
- A blocked stdout sink leaves stderr drainage independent; invalid UTF8, oversize messages,
  optional subscriber breakage and final-flush failure preserve original offsets and individual
  incompleteness. Invalid terminal combinations reach and fail the owning constructor.
- A healthy exact ID remains accessible beside corrupt unrelated metadata; ambiguity remains
  visible even with an unreadable candidate. Partial history cannot certify global latest.
- Explain→execute preserves the selected stage; a changed premise refuses, and unavailable
  fixture/provider explanation performs no preparation. Nested/limited typed pages retain
  indivisible records, coverage and context-bound continuation without full-result retention.

<a id="b02"></a>
### B02 — Failure extraction

**Consumes:** B01 and A03's normalized records/diagnostics. Generic/cargo extraction can land
first; nextest reconciled success additionally requires C07's actual resolved selection and
C02's setup/terminal contract. This is a contract slice, not a whole-59c barrier.

**Delivers:**
- The cargo extractor (strict bounded native `Message` decoding, primary-span one-liner, dedup, full rendered
  text kept) and the nextest extractor (libtest-json-plus with format version 0.1, `type` versus
  `reason` split, `NextestExitCode` classification, per-test failure output).
- Run kinds `cargo-check`, `cargo-build`, `clippy`, `nextest` and `generic`.
- `pse result --failures` gives the first N unique failures within `--budget`.
- Typed normalizations preserve source diagnostics and selection identity; opaque retained
  upstream JSON is not treated as a schema-validated semantic model.
- Exact source stream/byte ranges lead back to independently retained raw bytes. Typed failure
  pages preserve diagnostic children, source/build identity and required reconciliation while
  opaque RawValue retains upstream duplicate keys and numeric lexemes. A03's typed Serde/
  Schemars view describes presentation, not protocol completeness or scientific validity.

**Acceptance:**
- **cargo fixture.** A fixture crate with one type error yields exactly one deduplicated failure
  with file, line, column and code.
- **nextest fixture.** A fixture with one failing test yields that test's ID and panic location.
- **Real pass-through.** A run through the real correctness adapter (`cargo-pse-correctness
  nextest run … --message-format libtest-json-plus`) shows the stream intact.
- **Fallback.** Missing/truncated/oversize stream or terminal evidence yields an exit-class/log-tail
  explanation and incomplete reconciliation, even when child exit is 0.
- **Selection.** Two binaries sharing a test name, setup failure, zero matches and feature-absence
  controls preserve exact selected outcomes. A full output destination cannot silently erase a
  mandatory record or falsely qualify the population.

<a id="b03"></a>
### B03 — Detached runs, wait and cancel

**Consumes:** B01's generic invocation/result slice, A03 finality, A05 compatible recorded
control and generation-use pins, and [A07](59a-cli-foundation.md#a07)'s R3 amendment. Full B02
test extraction is required for managed test success, not for the first generic detached
result/wait/cancel journey.
The affected protection/capture/page amendments additionally consume A08's published slice;
J01 source navigation and J02 proposals are not prerequisites to this generic journey.

**Delivers:**
- `manager.rs` gains a detached service start with the property set above.
- First settle authenticated service entry: inspect `foreground_placement`/`native_workload`,
  extend the actual placement owner for a service root, and exercise ordinary plus explicitly
  finite admission/release without an extra hidden admission path. Exit this investigation
  with the implemented original-resource route and passing native journey, or an explicit
  blocked detached delivery; the present scope-only function cannot be assumed to work.
- A ledger-authority reconcile in `manager.rs` that matches on description and invocation ID
  without `Unit.Refs`.
- The hidden subcommands `__supervise` and `__finalize`.
- `--detach` on `run`, `test`, `check` and bundle verbs.
- `pse wait` with property-change or pidfd waiting and NDJSON transitions.
- Extend the native reconciliation owner with typed summary/list and selected `GetAll` detail,
  original-incarnation fencing and scoped `watch`/optional `broadcast` fan-out. Lag and manager
  replacement trigger explicit resynchronization rather than fabricating complete history.
- `pse cancel` reconciles by recorded description and invocation ID and stops under ledger
  authority.
- Select/version-check a compatible prepared control generation from authenticated tool-root
  metadata; read/wait/cancel bypass current-source rebuild. Document incompatible-record
  recovery and retain original supervisor/finalizer pins until their actual users settle.
- Supervisor/finalizer results distinguish process failure from child reaping, original scope
  drain and permit release. Unknown/failed recovery stays incomplete; independent fixture
  retention/disposal remains with its actual owner.
- `Group` readers for `memory.current` and `cpu.stat`.
- Integrate A05's selected protection protocol before retained child/path/finalizer effects,
  including durable promotion if that candidate is selected. Conservative durable protection
  with pruning disabled remains usable while its replacement investigation is unresolved.
  Unresolved original generation claims stay protected; no PID-absence inference or speculative
  pruning enables this package. Replace duplicate foreground/detached lifecycle assembly with
  this shared owner as consumers move, keeping raw adapters/passive reads direct.

**Acceptance:**
- **Detached completion.** A detached `sleep 2; exit 4` returns `launched`, and `pse wait` then
  reports `child_exit: 4` and managed exit 1 after the launching process has exited. A shorter
  observation wait exits 0 with nonterminal state and does not stop the child.
- **Cancel.** `pse cancel` from a different process stops a running detached run and drains its
  cgroup. A run whose recorded invocation ID no longer matches the live unit is refused.
- **Crash and timeout.** Killing the supervisor with SIGKILL produces a finalizer record with the
  systemd result and unknown/failed settlement unless original-resource evidence proves drain.
  Include separately placed descendants and a retained fixture. A `RuntimeMaxUSec` timeout is
  distinct from a nonterminal `wait` observation timeout.
- **Finite placement.** A finite-class detached run is admitted and released.
- **Broken current sources.** After detached launch, make current host sources uncompilable;
  another process can read/wait/cancel with no build and prove original physical settlement.
  Mismatched unit identity and incompatible/forged generation records refuse safely.
- **Lifetime.** Ordinary pruning retains an unrecorded long adapter and delayed finalizer;
  cleanup failure preserves all other attempted cleanup results and protected evidence.
- **Native observation.** Race a unit incarnation replacement, omit an unavailable interface,
  and lag/disconnect a subscriber: explicit resynchronization/coverage survives and no mixed
  snapshot is called atomic. Killing a passive observer does not authorize generation disposal.

<a id="b04"></a>
### B04 — Composite bundle runs

**Consumes:** B01–B03.

**Delivers:**
- `pse run-all <recipe…>` runs each child as `just <recipe>` in its own child run, keeps going
  after failures, and records children in the parent.
- The `turn-end`, `ready` and `hygiene` recipe bodies call it.

**Deletes:** the `run-all` private recipe.

**Acceptance:**
- A bundle with one failing child reports that child's ID and first error.
- `pse result <parent> --failures` lists only failed or incomplete children.
- Include incomplete children and cleanup failures without hiding successful siblings. A
  parent's final response cannot imply physical/scientific completion for an uncertain child.

<a id="b05"></a>
### B05 — Session identity and Claude handoff

**Consumes:** B01 and [59a A06](59a-cli-foundation.md#a06) (`status`).

**Delivers:**
- The session-identity resolution above.
- `pse handoff`.
- New `agent-hook pre-compact` and `agent-hook context` actions.
- `.claude/settings.json` adds the context-only `PreCompact` and `SessionStart` hooks. The
  `compact` source returns `pse handoff --show`; ordinary starts return the
  [59e E02](59e-environment-and-adoption.md#e02) status brief.
- `xtask agent-config-check` accepts the added hook commands, which still contain
  `/scripts/pse-host` and `agent-hook`.
- Before editing `.claude/settings.json`, coordinate with the operator's concurrent environment
  changes to that file.

**Acceptance:**
- A `pre-compact` payload produces a handoff record keyed by its `session_id`.
- A `SessionStart` payload with source `compact` returns that record as additional context
  within its size bound.
- An ordinary start returns the status brief.
- Compaction after a finished invocation preserves exact selected artifact/outcome references;
  an uncertain settlement or scientific attempt remains uncertain and is not relaunched.

## Verification and handoff

### Reusable analysis and original diagnostic integration

**B03** supplies authenticated owned placement and actual child settlement to G02; G owns
analyzer readiness, synchronization and reuse beyond one query. Canceling one request does
not tear down the shared session; a revision update may cancel all old snapshots. Teardown
settles analyzer/proc-macro children through the original owner. Include these actual users
in prepared-generation protection; no new queue/lease/recovery platform or scientific ledger.

**B02** preserves original compiler primary/secondary spans, children, suggestions/applicability,
macro context and available source/build identity for G/H/I. Explanations reuse the original
result without rerunning checks or interpreting rendered prose as typed facts. IDE diagnostics
remain a separate provider. B05 handoff retains evidence pointers, not durable analyzer IDs.

Acceptance adds two queries with one canceled, old-snapshot invalidation on edits, independent
session teardown/child drain and source-generation mismatch on original diagnostics. Generic
B03 does not wait on all G capability packages; G integrates the working ownership slice.

During packages, run compile checks and targeted tests on `pse-host`: the in-crate
`*_tests.rs` files and `tests/{foreground,correctness}.rs` plus new `tests/runs.rs`. The detached
cases need the user manager, as the existing foreground tests already do.

[59c](59c-tests-and-fixtures.md) consumes the run modes and extractors for `pse test`/`check`.
[59e](59e-environment-and-adoption.md) consumes the handoff and session identity for its harness
surfaces.
[59f](59f-simulation-observability-and-diagnostics.md) consumes invocation correlation and
output contracts, and owns product interpretation, follow, study bridge investigation and
capture/export. Closing a product observer never supplies cancellation authority.

Publish A08's affected rule amendments alongside the already completed A07 foundation; establish
A03/A04/A05's required owner-result/control/protection slice, then one foreground/detached generic invocation
with result/wait/cancel and original settlement. Integrate C07/C02's real test/fixture journey
and F01/F02's passive product journey before expanding the verb catalogue. Native journey
acceptance above is planned, not executed by this document revision; no product qualification
or performance claim follows from authoring these plans.

Each producer→consumer slice uses an actual CLI journey and its distinguishing failure case;
schema acceptance alone cannot establish finality, exact byte capture or selected population.
Native observation refactoring can progress on the existing manager owner independently of the
whole B03 delivery; product checkpoint readers remain independently useful. Bring J03's
syntax preview and base product observation forward on their narrower slices, adding semantic/
compiler enrichment as those producers become ready. Pruning/full A06 federation/all J are not
gates for the basic custody journey. Remove replaced callers/tests/fixtures in the slice that
migrates them; do not accumulate another qualification ledger or mandatory benchmark.
