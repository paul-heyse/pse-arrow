---
title: B1 caller, activity and physical connection lifetime target review
date: 2026-10-12
reviewer: Independent bounded design reviewer
tier: change
purpose: target
standard: core-3.4
profiles: [process-simulator-1.5]
baseline: a6aedd31c51688e3686723306d7deac14bd4072c
status: review
---

# B1 caller, activity and physical connection lifetime target review

**Decision: Accept the proposed B1 contract for implementation, at Proposed design strength.** ADR-0181 and Plan 37b B1 distinguish the three lifetimes needed for concurrent local simulator use: logical caller admission, an admitted activity's continuing dependencies, and authoritative teardown of the actual physical clients. The refusal rule preserves existing work; the successful-close rule excludes new work before teardown; interrupted teardown remains closed with an authentic recovery owner. No new blocking target finding was identified.

**Architectural fitness: acceptable in this bounded proposed design. Behavioral/semantic adequacy: adequate as a specified lifetime contract, unqualified as implementation.** This accepts neither the current implementation of F04 nor the rest of Plan 37. The implementation constraints below explain the consumed contract; they are not optional alternatives or additional product scope. Actual wiring and failure behavior require B1's targeted checks and Q1's assembled evidence.

## 1. Scope, drivers and evidence

Subject: [proposed ADR-0181](../../adr/0181-separate-caller-activity-and-physical-connection-lifetimes.md), [Plan 37b B1](../../plans/37b-connection-and-transport-lifetimes.md#b1-logical-release-and-authoritative-shutdown), and the relevant current Rust/Python lifetime owners. This is a change-tier target review under Core 3.4, its efficiency heuristics, Process Simulator 1.5 and the pse-arrow binding. Plan 37 remains the sole disposition owner.

Baseline: commit `a6aedd31c51688e3686723306d7deac14bd4072c`, plus the proposed ADR in the current worktree. Other agents' tooling/resource/investigation changes were observed and preserved; this review does not assess those implementations. Decisive source routes and limitations are in [source lifetime evidence](../evidence/b1-connection-lifetimes-2026-10-12/source-lifetimes.md).

The functional driver is concurrent local simulation, studies, optimization/fitting and dynamics: releasing one runtime must not invalidate another caller's renewal, scientific publication or read. Work may outlive its submitting waiter. The gate must add only short ownership transitions, with no mutex or transaction held over numerical work, query waits, joins or transport teardown. Ordinary extension of a workflow must consume the same ownership operations rather than invent another close policy.

**Implemented:** inspected baseline paths. **Proposed:** ADR/B1 target and controls. Read-only source inspection was performed; compilation, tests, services, probes, installations and measurements were **not_run**. There is no fresh Tested or Measured claim. B2/B3 restoration/admission repairs, artifact/history work, persistent maintenance, remote support and whole-product qualification are outside this acceptance.

## 2–3. Responsibilities, consumed contracts and preservation

| Responsibility | Owned decision and consumed contract | Dependency and local test context |
|---|---|---|
| Logical caller | Open/closed admission for one caller identity; ordinary clones share that identity, explicit independent construction gets independent authority | Runtime and originating aliases consume this gate; local race/policy controls need no solver |
| Admitted activity | Capability to finish precisely the work already admitted, including required children, publication, drain and settlement or explicit recovery transfer | Supervisors, durable attempts, readers and recovery owners retain it; caller close cannot revoke it |
| Physical connection owner | Shared state for the actual ordinary/activation client pair; refusal or Open → Closing → drained terminal state | Canonical integration owns the transition and SDK teardown; mock peers and interrupted-close controls exercise it |
| Protected selection | Exact server grant, current permission and release responsibility | Existing selection/read owners remain; transfer to asynchronous cleanup must be tracked before the old owner releases |
| Deployment/fixture administrator | Explicit physical shutdown and authentic recovery; service-wide exclusion remains a separate operational contract | Fixture and supervisor owners consume local quiescence; one empty local gate does not prove every process or service effect drained |
| Scientific/native owner | Numerical cancellation, native join, original assessments and coherent publication | Lifetime ownership preserves its access; a transport close never becomes numerical completion or server abort |

The distinction is sufficient without a universal lifecycle controller, a new registry of tasks or a new production crate. An idle `Runtime`, `Operations`, prepared object or completed `RunResult` alias is not by itself active work. Conversely, a live supervisor, publication child, protected read or cleanup/recovery activity cannot become invisible because its public wrapper disappeared.

Physical quantities, units/bases, source identities, validity envelopes and numerical assessments retain their existing scientific owners. B1 changes no formulation, derivative source, scaling, solver class, tolerance or status mapping. Well-posedness remains the upstream modeling/structural admission responsibility; this review does not establish its enclosing scientific adequacy. There is no new numerical stage to certify. The preservation obligation is that lifetime changes cannot bypass scientific terminal admission or convert cancellation/unknown effects into success.

## 4–5. Revealing scenarios and implementing constraints

The source supports the proposed seam, but several existing holders have different meanings. These cases determine where the new ownership must live.

1. **Caller close races with new admission.** Caller eligibility and physical Open state must participate in one effective admission decision. Either the activity is admitted before close and owns its continuation, or close wins and the call receives a typed closed/unavailable refusal. Checking a boolean and registering later leaves a gap. Ordinary `Runtime` clones, preparations and extracted originating aliases must not manufacture new caller authority. Explicit independent runtime construction remains legitimate while the physical owner is Open.

2. **A run continues after its caller releases.** `RunHandle` drop requests cancellation; the spawned supervisor still owns actual work and join. Bind the long activity before the first required canonical or native admission, transfer it into that supervisor/attempt, and retain it across the entire operation. `Heartbeat` alone is insufficient: `DurableAttempt::terminate` stops heartbeat before `seal_acknowledged`, and rejection cleanup can issue more canonical effects. Progress writers can outlive the dropped `Streamer`. Their continuation must retain the admitted capability until their actual bounded append/drain ends.

3. **Admitted continuation uses RPCs after logical close.** A plain caller-bound `Operations` clone is not enough if every subsequent RPC rechecks that caller's closed state. Internal continuation uses its existing activity authority; public fresh work still passes caller admission. This applies to renewal, result export, study/fit/horizon supervision, cancellation and exact recovery. Sharing an activity across required children may use ordinary ownership; no global task scan or new activity on every heartbeat is needed.

4. **A protected reader becomes an asynchronous release.** `ProtectionOwner::drop` currently creates a tracked release task while decrementing the live-reader count. Preserve that handoff without an unowned interval: retain/transfer the activity before the count can appear empty, and keep required release RPCs usable. `ProtectedSelection::finish`, returned block owners and fixture fallback drain consume the same rule. A new physical gate must not move Closing and then reject the cleanup it promised to join. Copied arrays retain allocation ownership; copying bytes grants no new DB-page permission.

5. **Physical close races with admission or the last activity.** Under one short state transition, refuse while real activities/readers remain and leave both physical state and required admission usable. With none remaining, publish Closing before new admissions can succeed, then release the lock and drain. Coordinate any preserved reader gate rather than marking it closing before a later activity check refuses. Count actual dependencies for this driver pair; neither Arc counts nor a deployment-path singleton expresses them. Distinct connections to the same service keep distinct local ownership; the supervisor still owns service-wide maintenance exclusion.

6. **Close is interrupted between drivers.** The existing disconnect sequence awaits ordinary then activation teardown. Cancellation or an error after the first must leave Closing, forbid fresh work, and retain an authentic path to finish draining the remainder. Repeated physical-close calls must join or continue that owned teardown rather than reopen, duplicate teardown unsafely or report success early. A local close operation does not prove remote transaction settlement. Reopening constructs/revalidates a new owner; stale caller/activity tokens cannot attach to it.

7. **Unknown work and escaped handles remain truthful.** Dropping a submitting future is not settlement. Keep exact operation identity and the capability needed through local drain and settlement, or transfer them to an explicitly owned recovery activity before releasing the original. A detached task retaining only a raw store alias is insufficient. Existing result/reader operations may continue when they already own an activity; an idle completed result or study handle does not keep arbitrary future work admitted forever. Pure inspection of copied immutable facts needs no transport permission.

The inspected `Runtime::canonical_store`, `Runtime::canonical`, `Operations::store`, direct canonical operations, initialization/activation and fixture administration are relevant escape routes. Migrate their actual admission/continuation semantics, not merely the Python `close` body. SDK disconnect remains available beneath authentic physical administration and transport tests; ordinary runtime release stops calling it. Regenerate the Python contract/stubs from their declarations, and keep cache clearing separate.

These constraints follow the accepted target rather than selecting a particular synchronization library or API spelling. They preserve a slow but legitimate activity: physical shutdown refuses and leaves it usable rather than waiting indefinitely or cancelling work owned elsewhere.

## 6–7. Architectural assessment, gates and findings

| Foundation | Verdict for proposed B1 | Reason |
|---|---|---|
| AP-01 | Satisfied | Caller release, dependency completion and physical administration have distinct reasons to change |
| AP-02 | Satisfied | Admission, refusal, terminal close and interrupted recovery have explicit consumed meanings |
| AP-03 | Satisfied | Existing workflow/reader owners compose one activity capability and the existing transport owner |
| AP-04 | Satisfied | Caller, admitted activity and physical incarnation adequately model the scoped lifetimes and govern behavior |
| AP-05 | Satisfied | Atomic admission/close decisions and explicit recovery transfer replace incidental alias-count behavior |
| AP-06 | Satisfied | Gate and protocol-peer controls can exercise ownership without unrelated numerical execution; assembled checks remain |
| AP-07 | Satisfied | Short transitions follow actual dependency lifetimes without a global solve mutex, repeated registration or indefinite shutdown wait |

| Gate | Proposed-design judgment and scope |
|---|---|
| G1 | Pass: one authority per caller and physical lifetime; scientific/operational authorities remain separate |
| G2 | Pass: caller release, refusal, Closing, drain and remote uncertainty remain distinguishable |
| G3 | Pass: closed admission and stale-owner refusal have specified enforcement boundaries |
| G4 | Pass: physical teardown and cache clearing remain explicit effects |
| G5 | Pass: continuation, cleanup transfer and interrupted teardown preserve ownership; no implied server abort or scientific completion |
| G6 | Pass in affected scope: idle aliases and copied immutable content do not acquire stronger permission; no numerical reuse rule changes |
| G7 | Pass: target and current implementation evidence are distinct; B1 local drain does not claim service/native/DB settlement |
| G8 | Pass: existing SDK owns physical drivers; domain caller/activity ownership is not supplied by SDK reference counting |
| G9 | Pass for the proposed contract: applicable foundations above are satisfied; current F04 implementation remains open |
| PS-G1 / PS-G2 | Not applicable to this change judgment: no physical quantity contract, formulation or structural-admission behavior changes; enclosing science unqualified |
| PS-G3 | Not applicable as numerical certification: no numerical algorithm/status/tolerance changes; PS-10 truthful outcome and native-drain constraints remain mandatory preservation boundaries |

**New target findings: none.** The source review's F04 remains the historical implementation finding and Plan 37 owns its disposition. Acceptance here does not resolve it. The seven implementing constraints are the detailed meaning of the reviewed contract, not seven new findings or a second status ledger.

## 8–9. Library fit and alternatives

| Alternative | Judgment |
|---|---|
| Keep runtime-triggered shared disconnect | Does not serve independent caller release; historical F04 remains |
| Count Arc aliases | Counts cached/completed/idle references and misses actual admitted continuation meaning |
| Serialize whole workflows behind one mutex | Adds unnecessary coordination across native work and publication; changes resource lifetime without solving authority |
| Selected small gate plus activity capability | Simplest fitting domain composition over existing SDK drivers and tracked reader release; selected |
| Replace transport or introduce universal lifecycle controller | Neither is necessary to express the missing domain lifetime; a replacement would still need this contract |

The retained patched SDK handles transport queues, original clocks, dispatch uncertainty, payload ownership and actual driver drain. The added canonical gate supplies the domain distinction the SDK cannot infer. No library upgrade, dependency addition, new cancellation framework or performance claim follows from this review. Local synchronization mechanics remain the implementer's choice within the contract.

## 10. Verification and remaining evidence

All controls below are **Proposed**, not executed. Use the current package/mechanism routes and explicit force-validation for Arrow-consuming closures.

| Scenario | Distinguishing evidence |
|---|---|
| Caller release versus admission | Barrier-controlled both-order race; admitted work continues, losing admission refuses; independent caller remains usable |
| Long activity versus public-handle drop | Supervisor keeps ownership through native join, progress writer, heartbeat shutdown, seal and failure cleanup |
| Physical refusal | Protected read and durable activity each cause refusal; their actual next RPC still succeeds after refusal |
| Last dependency versus close | Exactly one atomic outcome; no admitted activity starts after Closing |
| Reader cleanup handoff | Pause between final reader release and release RPC; close cannot strand that RPC or report drain before it finishes |
| Partial teardown | Pause/cancel/fail between ordinary and activation closes; new work refuses and authentic retry/continuation proves both clients drained |
| Unknown effects | Interrupt after dispatch; exact operation/recovery ownership survives and no blind write replay or false completion occurs |
| Escaped capabilities | Direct store/Operations aliases and prepared/result/reader handles follow their caller or admitted continuation authority, including after close |
| Public wiring | Regenerated Python declaration and affected Rust callers implement logical release; Q1 exercises rebuilt native/Python durable publication/cancellation |

A mock-peer or gate-policy pass does not establish real native/Python wiring, scientific qualification, remote support, service-wide exclusion or power-loss recovery. B1 implementation review and Q1 must inspect the landed paths; tests of a surrogate gate cannot establish that every real caller consumes it.

## 11–12. Rule impact, disposition and decision

<a id="rc01"></a>
**RC01 — Caller close and physical administration.** Affected rule: existing runtime close/disconnect behavior and blueprint §20.5/§20.6/§21. Required change: logical caller release plus independently retained activity and authentic physical teardown, as ADR-0181 and Plan 37b specify. This is the already-confirmed principal review RC04 impact; no new operator decision or additional scope is proposed. Keep accepted records immutable and apply the repository's existing decision/design route. If this change were not implemented, F04 would remain unresolved rather than gaining safe concurrent close semantics.

No SHOULD exception is selected. No other rule impact was identified in this bounded review. Plan 37 owns current finding/package status and receives this review as B1 preimplementation evidence; ADR status and implementation acceptance remain distinct.

**Final decision: Accept ADR-0181/Plan 37b B1's proposed lifetime contract for implementation, scoped to the existing local-first connection integration.** The decisive evidence is the separation of actual caller admission, long continuation and physical teardown in the target, supported by the inspected baseline owners and revealing failure cases. Preserve the implementing constraints above. B1 correctness, historical F04 closure and assembled Plan 37 qualification remain to establish after implementation.
