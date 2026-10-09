---
title: Plan 30 runtime contracts
date: 2026-10-09
status: review
tier: design
purpose: target
standard: Core 3.4; Heuristics 1.0; Process Simulator 1.5
---

# Plan 30 runtime contracts

## 1. Decision, target and coverage

**Architectural fitness: satisfied for the proposed contracts. Behavioral/semantic
adequacy: adequate at Proposed design level. Overall decision: Accept the bounded
Plan 30 target design, including the settled execution contracts below.**

This independent R0 assessment examines the material missing choices behind the
[earlier candidate's Not Accept decision](design_review_websocket-and-persistent-agent-environment_2026-10-08.md#12-final-assessment-and-evidence-limits).
It does not change that review's historical observations or its Revise verdict on
the then-current implementation. Acceptance here does not resolve WP01–WP08,
qualify an implemented transport, or establish scientific or performance results.
[Plan 30](../../plans/30-websocket-and-persistent-agent-environment.md#finding-dispositions)
remains the sole current disposition owner.

The subject is the five Plan 30 documents and the coordinator's settled execution
approach of 2026-10-09, against source baseline
`d92fa8ee1110508e50f049bc681765e2903cbd8b`. The assessment uses Core 3.4,
Heuristics 1.0, Process Simulator 1.5 and the pse-arrow binding. Reviewer:
independent delegated design reviewer `plan30_contract_review`.

The target is dependable native application RPC and a persistent local environment
for one operator and concurrent Codex sessions: isolated mutable tests, compatible
receiver coexistence, preserved failure evidence and coordinated finite capacity.
Relevant scientific consumers are edit/re-solve, multi-case studies, retained reads
and interruption/publication. Numerical formulation, solver algorithms, all model
families, remote deployment and power-loss durability are outside this bounded
assessment. No product build, test, query, process signal or timing experiment was
run. Source observations are **Interface-checked**; replacement behavior remains
**Proposed**.

## 2. Responsibilities, dependencies and variation

The design assigns the decisions that previously crossed lifetimes to coherent
owners. The canonical adapter classifies submission and settlement; the service
owner admits disk interpretation and availability; a receiver owns immutable
artifacts and native work; a test context associates selected test identity with
resources; existing runners decide terminal outcome; existing canonical and
assessment owners protect retained references. Host admission connects actual
service/receiver/process ownership to capacity. The agent frontdoor composes those
owners without becoming another scientific retry, schema or pass/fail authority.

Early contracts remove the apparent package cycles. A1 needs B0 context meaning,
not completed residency. B2 consumes working D1 role allocation; D2 subsequently
checks placement. C3's reference guard derives from C1 registration and existing
receipt references before C2 can delete; its later release interface consumes C2
disposition. Root-owned shared declarations prevent independent profile meanings.

Changing worker bytes is a receiver-generation change, while changing server
release or interpretation is a service-generation decision. A new scientific
model continues through the existing authored semantics and execution owners;
transport and agent tooling acquire no model-specific switches. Replacing an SDK
patch changes its integration owner and tests, rather than test-disposition or
scientific retry policy. These are credible, localized variation axes.

## 3. Contracts and preserved scientific meaning

The settled execution approach makes the following consequential choices explicit:

- Patch only `surrealdb` and `surrealdb-engine-api` 3.3.0, preserving native codecs
  and typed statement results. Borrow an `Arc<Surreal<Client>>`; do not clone the
  SDK client or turn a borrowed query into an owned session implicitly.
- Capture one monotonic clock before admission: 30 seconds for ordinary complete
  operations, 90 seconds for activation/schema initialization. Retry, queueing,
  replay, sink readiness, transmission and complete processing consume it.
- Before transmission, cancellation/expiry is definite refusal. Once transmission
  begins, interruption is uncertain. Caller expiry transfers response correlation
  and admission to bounded local drain, initially 90 seconds. Its expiry invalidates
  the physical connection and classifies siblings; it does not establish server,
  canonical-effect or native drain.
- SQL remaining timeout and UTC fences provide their actual supported protections.
  Evaluate the result, check expiry and applicable lease/generation authority at the
  final executable fence, then return it. No fence follows a top-level `RETURN`.
  No hard-abort or commit-before-caller-deadline guarantee is claimed.
- Unknown acknowledgment retains the original operation identity and exact request
  bytes. A missing acknowledgment is not evidence that a previous write cannot
  commit. Only definite conflicts permit a fresh guarded decision under the same
  clock; generic uncertain-write replay is removed.
- Ordinary and activation physical connections are separately owned. Each has 32
  application requests, two reserved controls, route/deferred capacity 34, one
  immutable application session and 16 setup/replay entries. Application operations
  never enter setup replay. Complete 4 MiB native messages, finite buffers and
  admitted result pages bound transport; decode/escaped Arrow reservations are
  separately charged.
- Profile revision 2 separates service identity and receiver selection. Unknown
  profiles refuse; selected older profiles have explicit validated readmission.
  Immutable worker/supervisor closures are materialized before launch.
- Python runtime construction accepts explicit database selection through one
  validated native path. Test registration precedes creation. Drop releases/drains
  borrowers; only reconciled final outcome plus references and drain decides
  disposal. Missing outcomes and unknown reference eligibility retain pins.
- Every cooperating launch, including user-manager restart and recovery, acquires
  host admission. Original startup clocks include admission/readiness. Exclusive
  parking suppresses relaunch; actual descendant drain releases capacity.

**Physical-semantics preservation:** source rows retain units, basis, convention,
validity and identity under their authored/registry owners; result IPC retains
schema, coordinates and exact values under result admission; completion retains
typed scientific assessment and original attempt lineage. The transport is not a
new physical authority. No units or property models are changed by these contracts.
Well-posedness, derivatives, scaling, initialization, solver selection and independent
post-solve closure remain the neighboring scientific contract. No new numerical
stage is introduced, so this review does not re-establish those mechanisms.

## 4. Revealing scenarios

The original [S01–S09 definitions](design_review_websocket-and-persistent-agent-environment_2026-10-08.md#4-revealing-scenarios)
are retained rather than re-authored. Their distinguishing contract paths are:

| Scenario | Design consequence and acceptance boundary |
|---|---|
| S01/S03: replacement and interruption | Request-local lifecycle checks survive queue/replay/sink waits; exact typed results and original effect settlement survive disconnect. Actual SDK and server controls must demonstrate those paths. |
| S02/S04: overlap and rebuild | Explicit per-test database and immutable receiver association isolate histories; compatible storage survives a worker rebuild. Actual Rust/Python coexistence and changed-byte controls are required. |
| S05/S09: assertion after science and reuse | Final runner outcome controls eligibility; reference acquisition/publication coordinates with reclamation. Assertion-after-Drop and new-reference/reclaim races must retain evidence. |
| S06/S07: concurrency and owner loss | Fixed shared lanes charge independently supervised roles through actual drain; nonce/process generation and cgroup liveness govern recovery. Real placement and surviving-descendant controls are required. |
| S08: scientific extension | Authored semantic owners absorb new meaning; storage/runtime tooling retains generic consumed contracts. No persistence-specific scientific branch is introduced. |

## 5. Execution fit and library realization

The inspected SDK has request/session routing and native values, but `RequestData`
and `EngineContext` lack the consumed lifecycle metadata. Its WS dispatch can park
routes during replay and await sink access without the proposed deadline checks.
`Surreal::clone` mints a new session. The two-crate patch therefore addresses actual
integration gaps without recreating the protocol, value codec or statement model.
The private adapter must retain these distinctions rather than hide them in an
outer timeout. SDK source inspection establishes the patch surface, not its behavior.

Completed bounded pages fit the buffered WS capability. A client receive limit
alone cannot bound server reply construction. The planned query-shape audit must
use worst-case admitted payloads and framing, retain coverage/order and preserve
result/source protection across escaped Arrow ownership. The target has a credible
paged route for legitimate large results; it does not promise incremental WS rows
or silently truncate. An unsupported oversized shape must refuse explicitly while
the ordinary supported shape obtains a bounded realization.

Persistent shared storage removes ordinary service startup from each fixture;
isolated databases retain independent mutation ownership. Separate artifact
generations avoid unnecessary storage interruption on compatible worker rebuilds.
Schema installation remains explicit per new database, rather than hidden in open
or treated as a clone/import capability. Bounded restart follows crash/reopen
controls, not the assumption that listener availability proves recovery.

The fixed lane policy has a concrete operational reason: independent agents and
independently supervised native roles previously each assumed their own capacity.
Normal functional 96 GiB and timing 64 GiB compose under 160 GiB. Reference remains
the exact 16/140/4 GiB partition; exclusive functional uses its separately named
160 GiB profile. Wider requests account for all receiver/observer/control roles and
effective ancestors. Numerical pool/worker defaults are profile-specific; authored
workloads are not shrunk to fit. Pure/light tests do not acquire heavyweight native
or database setup merely because runner association is added.

Short kernel-held metadata exclusion plus existing systemd supervision is
proportionate to a local multi-agent machine. It avoids a general job broker and
age-based lock stealing. Ceilings and CPU masks do not reserve physical resources
or govern unrelated projects; the 36 GiB availability guard and initial allocations
remain proposed policies requiring bounded placement/pressure controls. Timing
explicitly records resident state and unmanaged contention. No speedup, safety at
simultaneous peak ceilings, or uncontended host guarantee is inferred.

## 6. Foundations and gates

Judgments below apply to the proposed contract design, not the baseline implementation.

| Foundation | Verdict and reason |
|---|---|
| AP-01 | Satisfied: service, receiver, effect, terminal outcome and retention lifetimes have distinct owners. |
| AP-02 | Satisfied: dispatch uncertainty, clock scope, complete results and explicit context are consumed contracts. |
| AP-03 | Satisfied: context/service/receiver/admission compose through early contracts without whole-package cycles. |
| AP-04 | Satisfied: submission, effect settlement, final test outcome, retention eligibility and capacity ownership are distinct authoritative operations. |
| AP-05 | Satisfied: bounded retained transport state, profile revisions and authenticated owner/drain checks have explicit enforcement points. |
| AP-06 | Satisfied: local SDK/lifecycle and admission policy controls need their actual dependencies; pure tests avoid unrelated infrastructure. |
| AP-07 | Satisfied: bounded pages, receiver reuse, shared admission and effect-matched drain provide credible routes without universal service sharding or whole-workflow replay. |

| Gate | Design judgment and reason |
|---|---|
| G1 | Pass: existing scientific/terminal/reference authorities remain unique; registration associates them. |
| G2 | Pass: native exact-value and complete-page contracts preserve consumed meaning; profile evolution refuses unknown interpretation. |
| G3 | Pass: explicit admission, receiver compatibility, lifecycle fences and response checks define invalid-state rejection. |
| G4 | Pass: inspection/open do not install schema or restart science; startup, recovery and disposal are explicit effects. |
| G5 | Pass: uncertain effects retain original identity; disposal/capacity release require their own settlement and drain. |
| G6 | Pass: immutable receiver premises, prepared-product validity and evidence-origin references survive composition. |
| G7 | Pass: buffered WS, local drain, UTC fences, host limits and Proposed evidence are stated without stronger claims. |
| G8 | Pass: typed SDK, RocksDB, kernel exclusion, systemd and existing runners supply the generic capabilities. |
| G9 | Pass at Proposed design level, following the individually satisfied foundations. |
| PS-G1 | Pass for the unchanged physical transport contract; native-value and IPC controls must qualify its realization. |
| PS-G2 | Not applicable: these mechanisms formulate no equation or solve order; scientific pre-solve admission remains required. |
| PS-G3 | Pass for preserved outcome/publication and native-drain contracts; no solver or model-family qualification is claimed. |

## 7–9. Findings, alternatives and obligations

No new material design blocker was found in this bounded assessment. The original
findings remain implementation obligations at Plan 30, not resolved findings here.
Direct URL substitution would still leave WP01/WP02 open. A bespoke WS client would
add routing, sessions, replay and codecs that the selected SDK already owns. Waiting
for an unspecified release does not improve the current reproducible route. Revisit
the local patch when a released upstream correction supplies the same qualified
contract.

One shared functional service with isolated contexts is the simplest initial
topology. A bounded two-service comparison is triggered by diagnosed current
physical contention; the historical initializer failure does not establish that
premise. Destructive tests and incompatible releases already justify separate
generations. The exact reference lane remains useful scientific qualification;
forcing every functional invocation through it would obstruct the concurrent target.

Critical implementation obligations are the actual final executable SQL fence for
each mutation shape; all pending/deferred/replay and decoded-memory bounds; immutable
artifact admission; launch ownership before effects; final-outcome reconciliation;
reference acquisition/reclaim coordination; and capacity charges through descendants.
A stronger deadline/abort claim would require a different assessed capability, not
renaming these obligations or citing a successful small read.

## 10. Verification boundary

**Not run:** product builds/tests, SQL controls, recovery tests, kernel-placement
experiments, performance comparisons and model conformance. This is source-grounded
design acceptance. The new implementation requires A/B/C/D targeted controls and D3
assembled journeys before its behavior is reported as Tested. The old managed
thousand-point identity remains at 28e and must run under its unchanged reference
profile; read-only stress is not substitute evidence. Historical comparison leniency
does not weaken solver stopping, physical/domain or post-solve checks.

Inspected source includes canonical connection/request/protected retry, exact codec,
staging acquisition and transaction tails, fixture Drop, supervisor profile validation,
environment placement, terminal-owner selection and receipt origin reuse. Exact SDK
inspection covers 3.3.0 `Surreal::clone`, query builders, engine context/request/route
and native WS routing. The previous supporting evidence supplies additional host and
server-response leads with their original conditions. No absence claim extends beyond
these paths. The coordinator must compare later code against this settled contract;
this assessment is not an implementation review.

## 11–12. Rule impacts, disposition and final decision

**New rule impacts: none.** This design consumes Plan 30's already confirmed RC01–RC06.
The required proposed ADR-0164/0166 and architecture amendments remain the coordinator's
route; acceptance here does not change ADR status or authorize disposal of pre-existing
stores, archives, evidence or worktrees. There is no additional operator approval flow.

**Accept the proposed Plan 30 runtime contract design.** All material choices that
caused the earlier candidate's Not Accept decision have a coherent, bounded and
truthful realization direction. No SHOULD exception or waived MUST is needed for
this accepted scope. Current implementation fitness and assembled behavior remain
unqualified until their owning packages provide evidence. Finding disposition remains
exclusively at Plan 30; the original review and failure repair retain their meaning.
