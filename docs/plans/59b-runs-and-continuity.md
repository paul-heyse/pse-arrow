---
title: "Plan59b: Runs and continuity"
status: active
date: 2026-10-30
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md]
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
- session identity for selected results; automatic compaction packets are no longer required.

**Sources.** [O03](../functional_reviews/rust-agent-cli-2026-10-29.md#O03),
[O20](../functional_reviews/rust-agent-cli-2026-10-29.md#O20) (Claude only), and the run part of
[O24](../functional_reviews/rust-agent-cli-2026-10-29.md#O24).
The [observability review](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md)
O01/O03/O05–O07 and [improvement review](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md)
I01–I08/I10/I11 refine identity, result, control, output and handoff. The coordinator owns
dispositions; this document schedules its part. The selected simplification/testing wave is
authorized and in progress.

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

**Rule basis.** [R2](59-agent-cli.md#rule-changes) allows owned-run supervision. The original
R3/A07 control exception remains: existing detached operations use authenticated compatible
tools without rebuilding current sources. [A09](59a-cli-foundation.md#a09) publishes the
2026-10-30 lifetime simplification before implementation relies on it. The former universal
per-run supervisor is no longer the selected target.

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

The operator-confirmed [CLI/test/runtime review](../functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md)
O02/O03/O09–O13 replaces the universal service-supervisor topology and event-reconstruction
requirements. These are **Proposed** implementation targets; existing focused evidence keeps
its original conditions. [A09](59a-cli-foundation.md#a09) owns governing publication. Production
execution remains paused.

### Lifetimes and physical ownership

| Mode | Physical owner and behavior | Intentional limits |
| --- | --- | --- |
| Foreground, default | Existing native `foreground::run_owned` ownership, extended for required capture/result composition. Wait through child reap, required stream drain and resource settlement; caller death retains native manager-enforced containment. | No generic foreground reconnectable history/result/cancel promise or crash-written finalizer. Required test/scientific evidence stays with the operation owner. |
| Explicit `--detach` | One transient user service containing a fixed Rust wrapper, payload and ordinary descendants. Client exit leaves the job running; compatible recorded control inspects/waits/cancels it. | Lifetime independence requires admitted placement; detachment never escapes enclosing finite constraints. Independently cancellable generic nested subtrees are not provided. |

These are two lifetime modes over shared capture/result capabilities, not two generic
supervisor implementations. Ordinary nested commands inherit authenticated enclosing ownership
and aggregate allocation. A product operation may retain its own cancellation contract without
creating another generic CLI process supervisor. Separately owned resident services still need
coherent finite-envelope treatment, including wrapper overhead.

For detached work, `ManagerProxy::start_transient_unit` starts the immutable fixed wrapper
before payload effects. Existing private `UnixCommand` transports raw-byte executable, argv,
cwd and environment to the wrapper; D-Bus control strings need not encode child arguments.
Descendants inherit service placement. Compare actual service admission with current scope-
only admission; source plausibility does not qualify the new topology. Do not retain external
payload relocation just to preserve the old supervisor design.

The service uses deliberate `TimeoutStartUSec`/`TimeoutStopUSec`, `Restart=no` and explicit
readiness after initialization. Consider `KillMode=mixed` where the wrapper must initiate
cooperative cleanup before final descendant termination; settle this against actual signal,
reap and cgroup-drain tests before selecting it. `RuntimeMaxUSec` represents only an explicit
workload deadline. Systemd applies stage-specific timeouts, not one universal elapsed budget.
Observation timeout ends waiting, not the job. Native job/unit acknowledgement, not sd-notify
transport success alone, establishes startup.

Systemd owns containment/forced termination; Rust owns cooperative cancellation, child reap,
required drain and truthful outcomes. Use a retained Tokio `Child` and joined futures for a
small fixed flow. Where independently ordered subsystems genuinely remain, use
`SubsystemHandle::start`/`NestedSubsystem::join` as their actual owner, replacing parallel task
registries rather than stacking them. Preserve signal-registration error behavior.
`CancellationToken` signals intent; `TaskTracker::close` does not prohibit admission or prove
physical completion. Running Rayon/blocked native work remains cooperatively canceled and
joined, not stopped by dropping a future. Replace short `try_wait` polling with async child
wait/pidfd readiness where exec/raw-byte/reap semantics permit it.

### Results, capture and control

A `CliInvocationId` identifies a host operation, never scientific RunId/AttemptId or systemd
invocation identity. Product owners retain scientific acceptance and fixture disposal. Required
result publication is constructed from original operation, capture and settlement facts before
A03 presentation. Launch acknowledgement and nonterminal wait can succeed while subject state
remains running; they never assert a completed pass. Child failure and every independent cleanup
failure remain observable. Required evidence loss is incomplete, not success.

Detached records retain immutable tool/control identity, command/context, applicable native unit
identity, raw stdout/stderr and final result. Foreground operations can publish useful captured
artifacts/results while their owner lives, but do not require a universal durable ledger and
crash recovery service merely to support generic reconnection. Reuse A04's direct exact-ID
lookup and partial prefix/history semantics for retained records; a corrupt unrelated record
cannot deny healthy exact access. Partial history cannot certify global latest.

Independent pipe readers write byte-exact `stdout.raw` and `stderr.raw`, preserving stream-local
offsets and a readable prefix on failure. Presentation cannot block required drainage. Optional
observation-order indexes exist only for a real consumer; they do not establish producer order.
Keep bounded framing/parser state and explicit oversized-record, disk-full, flush and broken
sink outcomes. A required sink failure prevents successful correctness reconciliation while
independent cleanup still runs. Machine views use A03 typed producer pages and source-bound
continuations; opaque original bytes are not domain/schema proof.

Detached `wait`/`cancel` authenticates checkout, recorded description and systemd invocation
identity against a compatible prepared control generation. A path in metadata is not executable
authority. Use existing native property/cgroup notifications plus uncached reconciliation;
manager replacement, lag and incarnation change require explicit resynchronization. Expose
unit state plus applicable service/process/accounting properties; do not synthesize service
`MainPID`/`Result` semantics for foreground scopes. No new second state database is required.
A minimal detached `ExecStopPost` record may publish crash/incomplete; it cannot manufacture
missing child status, original output or fixture settlement. No automatic effectful restart.

### Native completed test results

C02 owns private report production and exact selected population. B02 presents the resulting
domain fold: selected compiler context/binary/test identities, native runner exit, setup policy,
required capture and actual fixture/drain facts. Consume typed native JUnit with a qualified
reader, rather than reconstructing experimental libtest JSON lifecycle. Retain strict Cargo
message extraction. The selected assurance deliberately drops independent per-start/terminal
ordering, retry-name reconstruction and event byte-address proof; raw files remain original
output evidence. Missing/malformed/mismatched reports remain incomplete even on zero exit.

`result --failures` selects bounded original failures, setup/runner errors and retry/flaky facts,
with output addresses and explicit next actions. A successful read does not obscure original
subject failure. Exact rerun consumes captured package/binary/test/platform/feature/ignored
policy, checks changed source/context/fixture assumptions and performs no implicit rerun merely
to explain evidence. Native recordings/replay are conditional diagnostic attachments; replay
is not current-source qualification or frozen external fixture state.

### Composition and sessions

Composite commands keep going after independent failures and summarize each original outcome.
Nested children use enclosing physical ownership; a parent result does not invent independent
subtree control. Session grouping uses explicit `--session`/`PSE_SESSION` and available existing
harness identity, with checkout grouping when unavailable. It is correlation, not ownership.
Generic `pse handoff` packet assembly and new PreCompact/SessionStart packet hooks are removed
from required delivery. Preserve uniquely authored existing notes/evidence; this scope grants
no configuration or data deletion. Focused result/source links remain independently useful.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [B01](#b01) | in-progress | 59b | Native synchronous capture, original typed workflow errors and actual owner settlement are implemented; every synchronous selected consumer moved, and foreground service/origin watching was deleted. Current scope-end qualification remains |
| [B02](#b02) | in-progress | 59b | Original Cargo diagnostics, native Nextest completed reports, frozen failure-text pages and exact fresh-source failed reruns pass focused/public controls; native exit, raw custody and physical settlement remain distinct. Final integrated checks remain |
| [B03](#b03) | in-progress | 59b | One service now contains wrapper/payload/descendants; finite capacity is reserved before wrapper birth and reconciled only after original native drain. Six ordinary/finite exit/cancel/main-crash cases passed again on the fresh post-retirement image; public raw-byte transport passed and obsolete external/subtree recovery is deleted. Integrated/static qualification remains |
| [B04](#b04) | in-progress | 59b | Native synchronous `run-all` retains original per-child results and parent associations. Public failed-then-successful recipe control passed with one physical native unit; recipe migration and scope-end checks remain |
| [B05](#b05) | in-progress | 59b | Shared explicit/environment/harness precedence, exact session-local latest, checkout-wide history override and independent exact-ID selection are implemented. Three focused controls and actual passive/harness associations pass; current integrated execution qualification remains with the wave |

<a id="b01"></a>
### B01 — Foreground operations and retained results

**Consumes:** A09's lifetime/result publication, A03's output/finality slice, A04 paths and A05
immutable authenticated tools. Does not require B03 detachment or complete catalog federation.

**Delivers:** integrate `foreground::run_owned` with operation-owned capture, typed performing
outcome and real settlement; preserve raw adapter grammar and child exits. Use retained result
lookup/pages where the operation produces records without imposing reconnectable crash history
on all foreground commands. Share byte capture/presentation helpers with detached execution.

**Deletes:** foreground generic service launch, origin-lease watchers and crash-finalizer
recovery once their synchronous consumers move; merged LogChunk retention/shared sink locks,
history-wide exact lookup and generic-envelope terminal admission with migrated callers/tests.
Do not remove actual native caller-bound scopes or settle by process disappearance alone.

**Acceptance:** both streams/invalid UTF-8 and exit 3 preserve bytes and original status;
SIGINT and killed foreground owner stop descendants with truthful incomplete evidence; disk-
full/slow-reader/final-flush failure retains readable prefixes and all cleanup failures.
Healthy exact result beside corrupt history remains accessible. Wrong terminal combinations
fail owner construction; selected explain premises survive unchanged execution or refuse before
effects. No passing foreground test requires service reconnection or a synthetic crash result.

**Execution checkpoint (2026-10-30).** Every synchronous selected workflow uses the original
native foreground owner, joined raw-stream capture and immutable performing result. Fixed
helpers authenticate their actual process, opened original root and recorded executable image.
The generic foreground service mode, origin pidfd watcher and service observer were removed.
Original typed workflow-report failures reach the performing response. Generic operation
metadata is decoded as raw verb-owned data until an actual fixed-workflow action selects its
schema; an eager workflow decode discovered during detached testing was corrected rather than
reshaping the generic fixture. Native scope/admission/reap failures remain owner results.

<a id="b02"></a>
### B02 — Native failure results and explicit next actions

**Consumes:** A03 diagnostic/page contracts; B01 capture where needed; C02's private completed
reports and C07/C05 exact observed population for test results. Cargo diagnostics can proceed
without the entire fixture/report implementation.

**Delivers:** strict bounded native Cargo `Message` decoding with original spans, diagnostic
children, suggestions/applicability, rendered text and source/build identity. Fold C02 reports
without another event state machine. Present failure pages and exact rerun actions; retain
opaque bytes without treating arbitrary JSON/schema validity as domain finality.

**Deletes:** replaced NextestExtractor event ordering/retry spelling/ignored-tail compatibility
and its mechanism-only tests. Preserve useful Cargo extraction and raw-capture coverage.

**Acceptance:** real type error yields deduplicated typed original diagnostics; failed case and
setup error preserve exact binary/context; retries/ignored/zero-match/absence policies follow
native semantics; missing/corrupt/foreign reports are incomplete. Concurrent runs never borrow
one another's report. Failure reading retains original outcome; source/context edits are visible
before explicit rerun. No recompile is triggered merely to explain an old result.

<a id="b03"></a>
### B03 — Explicit detached service, wait and cancel

**Consumes:** A09's changed lifetime publication, A03/A04 result/control contracts, A05 immutable
control authentication and shared capture from B01. Generic detachment needs no complete B02
Nextest interpreter; managed test success additionally needs C02's working native result fold.

**Delivers:** one service containing wrapper/payload/ordinary descendants, private raw command
handoff, real service-root admission and explicit startup/stop policy. Reconnection authenticates
recorded native identity and works with broken current sources. Native observation and actual
cgroup/process settlement remain separate from task completion and result availability.
Investigate finite sharing/resident allocation and chosen KillMode against actual containment;
unsupported placement refuses rather than creating a sibling allocation or bypassing limits.

**Deletes:** external payload relocation gates, ancestry/subgroup reconstruction and external-
payload finalizer recovery whose purpose disappears; no default independent nested CLI subtree.
Keep actual resource authentication/ownership. Collection-exclusive generation claims retire
only through A05's classified migration; existing unknown claims/artifacts remain untouched.

**Acceptance:** detach `sleep 2; exit 4`, exit launcher, then observe original failed child;
bounded wait is nonterminal and leaves work running; independent cancel authenticates incarnation
and drains contained descendants. Kill wrapper during output: systemd contains/terminates its
descendants and missing original evidence stays incomplete. Exercise finite envelope including
wrapper, nested launch constraints, non-UTF-8 argv/cwd, private environment, and current-source
compile failure during compatible control. Unit replacement/lag/resync cannot combine identities.
Prepare newer tools while old work runs without collection. No topology qualification is claimed
by this planned scenario list.

**Execution checkpoint (2026-10-30).** The detached wrapper and gated child share the actual
same native service root. Explicit startup/stop timeouts and `KillMode=mixed` accompany native
whole-population cooperative TERM, actual child reap and independent stream joins. Finite
reservation/ancestor configuration precede service dispatch, so the wrapper is constrained from
birth. The authenticated main binds that reservation before payload release; neither main nor
stop-post retires a still-live service charge. Post-reservation preparation, immutable input
publication and dispatch share independent unit/allocation/request cleanup. An actual finite
publication-collision control corrected a leaked charge: an explicitly released service launcher
fence cannot stand in for a still-live gated scope child, and `release(false)` remains an error.
The original real gated-child reservation control also passed after this distinction changed.
Recorded wait/cancel and later admission reconcile
only the original inactive, empty native lifetime, including stop-post and its allocation.

An immutable operation report records child/capture facts while `native_lifetime_pending=true`
and settlement remains unknown. Completed wait composes actual native drain/capacity with
complete original observation; it does not rewrite that earlier report or infer missing reap.
Wrapper raw OS exit reflects actual operation/capture status, independently of its pending
future lifetime. Authenticated initialization errors are retained before wrapper exit; abrupt
main death leaves missing original evidence unknown.

Six independent ordinary/finite controls exercised launcher exit, bounded nonterminal wait,
cooperative cancellation of escaped-session pipe holders, and authenticated original-main
SIGKILL with native descendant drain. A public detached journey preserved exact non-UTF-8
argv/cwd and private environment bytes. The six native cases also passed against a fresh prepared image after legacy deletion; the
public raw-byte journey retains its original conditions. Scope-end integrated/static qualification
stays with the coordinator. External payload
relocation, recursive CLI subgroup/ancestry reconstruction, external-resource finalizer recovery,
launch-closure protocol, ceremonial async hierarchy and their unconsumed APIs/tests are deleted.
Detachment from an already native-owned workload explicitly refuses; ordinary nested calls
inherit enclosing foreground ownership and cannot escape its finite envelope.

<a id="b04"></a>
### B04 — Composite bundle runs

**Consumes:** B01's synchronous operation/result slice; B03 only for explicitly detached bundles.

**Delivers:** `pse run-all` composes actual operations, attempts each independent child after
failures and records their original outcomes. Children inherit enclosing physical ownership;
there is no default nested service/subgroup for every child. Migrate turn-end/ready/hygiene
callers and retire the replaced private run-all recipe together.

**Acceptance:** one failing and one incomplete child remain distinct while successful siblings
survive; parent failure view links actual child results. Canceling the actual enclosing owner
settles its population; a parent cannot imply scientific acceptance or drain for uncertain work.

<a id="b05"></a>
### B05 — Session identity

**Consumes:** B01/A04 correlation fields, not generic status/brief or a new hook.

**Delivers:** one correlation selection owner with precedence `--session`, nonempty
`PSE_SESSION`, existing `CODEX_THREAD_ID`, existing `CLAUDE_SESSION_ID`, then checkout grouping
when no identity is available. Empty environment values are unavailable; malformed selected
identity is an error rather than permission to choose another identity. Selection/history
never grants cancellation authority. Do not add
`pse handoff`, PreCompact packet recording or SessionStart packet injection as prerequisites.

Capture the selected identity at launch. Passive history and `latest` selection use the same
precedence; `history --all-sessions` explicitly selects checkout-wide metadata. A selected
session without matching invocations has no `latest` result: it never silently selects another
session's work. Exact IDs and unique prefixes remain independently selectable, and the native
ownership checks for wait/cancel are unchanged by correlation.

**Acceptance:** distinct session filters, absent identity and explicit override preserve exact
result associations; grouping never selects an ambiguous latest result or changes native control.
Automatic handoff is deferred until a concrete consumer needs more than native context and
existing result/source references. Existing authored records are preserved.

## Verification and handoff

Reusable analyzer G02 consumes an authenticated working ownership slice appropriate to its
actual persistent lifetime, with request cancellation separate from session teardown. G owns
readiness/synchronization/reuse; no whole B02/test fixture or generic packet prerequisite.
B02 preserves original compiler facts for G/H/I without rerunning compilation. C consumes
actual execution/result slices; E consumes session correlation; F retains scientific and
observation authority. The host stays independent of product/Arrow/database dependencies.

Packages use focused compile checks and targeted tests in existing host foreground/correctness/
CLI owners. Actual systemd, finite allocation, crash and pipe behavior needs real journeys;
snapshots or schema acceptance alone do not qualify them. Scope-end integrated/static
qualification stays with the coordinator under AGENTS.md. Replace callers/tests/fixtures in
the same slice that proves their replacement. No mandatory performance campaign, generic
brief, automatic generation collection or new ownership ledger is a completion prerequisite.
