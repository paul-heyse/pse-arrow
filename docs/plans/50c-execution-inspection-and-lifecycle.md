---
title: "Plan 50c: Execution inspection and lifecycle workflows"
status: complete
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/50-codex-effectiveness-and-diagnostics.md
doc_topics: [diagnostics, environment, storage, validation]
---

# Plan 50c: Execution inspection and lifecycle workflows

The foundation paragraphs describe the named authoring baseline. Selected implementation,
inquiry outcomes, corrected mistakes and qualification conditions are recorded in the
[coordinator Outcome](50-codex-effectiveness-and-diagnostics.md#outcome).

## Purpose and foundation assessment

Deliver O06–O09 through the existing command, admission, server and resource owners. Codex
should explain a live wait, read the correct product-test evidence, run a reference fixture
under one lifetime and explicitly settle obsolete generated work. The
[coordinator](50-codex-effectiveness-and-diagnostics.md#package-state) owns package state.
**Interface-checked:** source `bb61ae3f5`; **Acceptance criteria:** target and acceptance.

`test_resources.invocation/publish_selection` owns exact selected identities before execution.
`test_run.finish` already joins result rows, report errors, workload exit, artifacts, process
and drain evidence and freezes an authentic terminal. A fully reported failure can be complete;
selected-minus-reported is not known per-test interruption. The registry terminal/membership
is authority, while an adjacent `.terminal.json` is a useful projection. `just result` currently
reads assessment `checks.json` v5, a different scope.

`activity` reports systemd races/partial results correctly. `host_admission.readonly_snapshot`
observes owners and parked services; `status` reconciles and may resume them. `acquire` keeps
its retry reason locally, so a useful live explanation requires a producer observation seam.
The documented reference workflow and exact-resource release/reclaim commands already exist.
Compose them; do not invent another ledger, scheduler, registry or scientific terminal.

## Target interfaces and semantics

Use the existing invocation identity/declaration as the run handle. Resolve an explicit path
or exact identity within the explicitly selected resource registry. Show which registry/run was
read; never pick an unrelated `latest` report or guess an invocation from an arbitrary JUnit.
Assessment and product-test readers share presentation/helpers where useful but retain
kind-specific parsing and exit/coverage meanings. An explicit `--kind` selector is sufficient;
no generic execution descriptor language is required.

Live explanations compose owner-local observations with actual units/processes and parked
intent. The detached `readonly_snapshot` mode writes nothing and never reconciles, resumes or
releases resources; it does not protect report artifacts against concurrent cleanup. Protected
payload reading is a separate mode: `borrow_report` writes temporary protection metadata and
releases that borrow afterward. Preserve its drain/release semantics and unavailable states;
do not describe it as a no-write snapshot or read unprotected payloads through snapshot references.
Fixed observers must remain usable when workload admission or compiler preparation is unavailable.

Commands that execute or dispose work are separate: a reference workflow uses an execution
owner; invocation abandonment is an explicit owner operation. Their results separate action,
test/scientific outcome, physical drain and retained evidence. Neither an observation nor a
tool's exit zero grants scientific success or disposal permission.

## Packages

<a id="C01"></a>

### C01 — Project authentic run and subject identities

Add shared reader helpers beside existing invocation/resource/report owners. Resolve selection,
frozen terminal and artifact references using their current versions, identities and protections.
Read incomplete images as incomplete. Expose exact run kind/identity, selected catalog, report
rows/errors, workload outcome, process/drain state and references, without deriving a new terminal.

Accept existing assessment directory, invocation declaration or explicit invocation identity;
reject ambiguous kind/path/registry combinations. Source provenance and artifact digests remain
references; do not hydrate every manifest by default. Include missing/unavailable/read-error
states and bounded page/detail selection. This contract is the early slice used by C02/C03 and
50e's associated artifact summaries, not a prerequisite to migrate every result command at once.
Standalone diagnostics without an invocation use E01's action-document reader directly. C03
follows their artifacts only when a real invocation association exists; never invent a test ID.

<a id="C02"></a>

### C02 — Publish and explain live waits

Have the command/admission owner publish a compact atomic diagnostic checkpoint for its actual
admission attempt: process/start identity, invocation association when available, current reason,
reason observation time and relevant conflicting owner references. Use its existing observation
area and original deadline; publish changes/useful current progress rather than a log row for
every retry. The checkpoint is a derived observation, never an input to admission order or a
replacement owner record. Before invocation creation, process/attempt identity is sufficient;
do not invent a test run ID.

Extend the fixed activity view with selected run/unit/owner explanations using that checkpoint
and `readonly_snapshot`. Preserve snapshot age, source availability and unknown lock/conflict
owners. Unit disappearance stays an explicit partial observation; `/proc` populations remain
best-effort. Show parked service, retained resume intent and observed blocking allocation without
promising that exit immediately restarts a service.

First consumers are a command waiting for reference admission and an empty scope awaiting manager
teardown. Present relevant inotify/user-manager evidence when reached, distinguishing configured
limits from remaining capacity. A manager/read failure cannot become empty success. Test no calls
to reconcile/start/ensure/reclaim and no compiler preparation in the observer.

<a id="C03"></a>

### C03 — Read product-test outcomes and their resources

Extend the `just result` discovery surface with an explicit product-test kind while preserving
existing assessment grammar. Use C01 to display selected, reported pass/failure, selected but
unreported, invocation interruption, report errors, terminal completeness, owned drain and
retention reasons. Do not label all unreported tests interrupted or treat a complete failing suite
as incomplete. Reader exit status reports readability/usage; the displayed workload result is
independent, matching the existing assessment-reader distinction.

Provide selected failure/log tails, binary/extension association and the exact JUnit/worker/raw
artifact references. Listing recent handles is convenience only. Validate artifact association
and protections through current readers, not source-name heuristics. Migrate the ordinary Rust,
managed native and Python command summaries to print the handle this reader accepts.

Targeted controls cover complete pass, complete fail, SIGINT with unreported selection, malformed
or missing report, report/selection mismatch, missing frozen terminal, partial drain and protected
resource. Independent fixture documents specify expectations; running a solver is unnecessary
to test report interpretation. One actual Rust and Python run validate producer/reader composition.

<a id="C04"></a>

### C04 — Compose the reference execution lifetime

Add a fixed reference-run recipe/command accepting explicit state/profile/worker and literal
trailing workload argv. Reuse current setup, native-operation and host-admission owners. Worker
build and persistent setup can precede the reference allocation; start, initialization, selected
workload, quiesce and drained stop share the lifetime-dependent owner. Nested recipes reuse its
authentic handoff rather than creating another reference allocation.

Return a C01 run handle plus distinct setup/workload/signal/drain/cleanup/normal-resume outcomes.
Use owner-aware finalization, not a trap that unconditionally kills an unrelated service.
Honor retained resume intent through existing reconciliation and report its result/refusal;
do not guarantee immediate restart. Preserve failure resources and logs under their existing
roles. No hard-coded ports, private directory strings or new fixture registry belongs here.

Accept a short real managed control, cancellation, setup failure and failed drained stop. Verify
literal argv, same-owner phases and unrelated-store preservation. Do not tie repetition count
to sixteen production lanes; a separate chosen concurrency control can exercise sixteen owners
when that behavior is what the test establishes.

<a id="C05"></a>

### C05 — Explicit invocation abandonment and exact release

Extend the current test-resource owner with an explicit invocation-disposition action and
inspection-first output. Existing terminal finalization freezes authentic outcomes and membership;
it cannot supply an abandonment fence without that terminal. For an invocation lacking a terminal,
add an owner-owned abandonment/membership fence within the same authority, separately from outcome
finalization. Reject later registration, preserve the selected membership and support interrupted
resumption without manufacturing a terminal. Observe actual owners/descendants and return a bounded
resource plan with pins, references, creation uncertainty and cleanup reasons. Apply only on an
explicit action request to that exact invocation. Reuse current release/reclaim/recovery operations.

Record an abandonment/release disposition separately from the authentic test terminal. Existing
frozen outcomes are immutable; a missing terminal stays missing, not a synthesized pass. Release
execution capacity only through the current host owner after authenticated physical drain and
allocation-guard reconciliation. Exact known-owned disposable fixtures can then be retired through
their owner; scientific effects may remain incomplete. Where creation effects are uncertain,
use the existing owner to establish/fence their actual scope before disposal. Unknown external
effects, unknown owners, active borrows and protected evidence remain blocked with a concrete
next observation/recovery route. A dead PID alone is insufficient.

Compose storage observation with the disposition explanation: bytes held, role, owner and
supported release route. Do not scan disk on ordinary command startup, delete evidence by
extension/name or build a generic prune service. The already retired historical invocation and
poisoned store supply a scenario pattern, not cleanup to execute again.

Targeted controls use a new private fixture: missing terminal with authenticated dead scopes,
known regenerable state, unknown creation borrower, active reference, changed generation and
interrupted resumable disposition. Verify no forged result/ACK and no deletion outside the exact
owner scope, including registration rejection after the new missing-terminal fence. Confirm its
fit with §24.1's single-composer/selection contract before implementation. If the fence or a
required release route changes that contract, obtain the scoped decision before dependent work;
do not silently weaken §24.1 or describe current finalization as already supplying this behavior.

## Dependencies, evidence and publication

C01 is an early reader contract. C02/C03 can proceed independently; C04/C05 consume the needed
selection/resource/observer slices. C05 coordinates Plan49g TOOL04/RUN01 recovery disposition and
Plan32 producer lifecycle. It does not finish either plan. Product boundary projections use the
existing catalogue only where they actually cross Rust/Python, not for every host-only JSON view.

**Acceptance criteria:** package-local functional controls and one relevant actual producer/reader/lifecycle
composition. 50g owns combined acceptance and scope-end checks. The zero target applies to checks
run, but a documented protected resource is not an artificial test failure or cleanup permission.
Publish durable mechanics to the environment, substrate and qualification guides through their
existing decision routes. Sources: `scripts/test_run.py`, `test_resources.py`, `validation_result.py`,
`host_admission.py`, `native_operation.py`, `activity.py`, `surreal_server.py`, `build_storage.py`,
blueprint §§20.6, 24.1 and 24.4.1, and [current operating guidance](../dev/agent-environment.md).

## Outcome

**Implemented:** the selected packages in this companion, with the optional mechanism decisions
recorded at the coordinator. **Tested:** package-local controls and selected composed consumers
under the commands, modes, zero-failure baseline and exclusions in the
[series Outcome](50-codex-effectiveness-and-diagnostics.md#outcome). That owner also records
corrected mistakes and deliberate deviations; this companion does not duplicate live status.
