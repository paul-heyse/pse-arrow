---
title: Study execution and database boundary
date: 2026-10-28
tier: design
purpose: target
decision: Revise
doc_role: review
reviewer: Independent design-reviewer, coordinator publication
doc_owner: docs/plans/57-library-leverage-and-correctness.md
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  test-assessment: "1.2"
  process-simulator: "1.5"
---

# Study execution and database boundary

## Assessment and scope

**Architectural fitness: Revise. Behavioral adequacy: unresolved for the remaining assembled journeys. Preferred target: a coherent hybrid with database-facing durable commands and reads, optional local preparation, and independently supervised native execution.** This is a Proposed target, not implementation acceptance.

The strongest established defect is a responsibility boundary: compact metadata access requires the scientific runtime composition root. Moving that access to a narrower client removes an unnecessary dependency. Moving every scientific operation or physical lifetime into the database would introduce different problems and is not supported by the inspected capabilities.

This independent principal review uses the combined library, ownership, composition, execution-fit and general-improvement focus. It covers Python entry, local preparation, durable study control, database transactions/modules, native workers, result access and their assurance boundaries. Numerical algorithms and whole-system scientific qualification are outside this bounded review.

The source baseline is `64a217de26385129fda8d65d3d62acd4f7117fa3`, including the three uncommitted operations inheritance corrections and the concurrent Plan57j checkpoint. The refreshed [production account](../audit/study-execution-and-database-boundary.md), [test account](../test-audit/study-execution-and-database-boundary.md) and [research basis](../evidence/study-execution-and-database-boundary.md) were consumed at publication revision `5e8aea0cccbcd81e35d9a55ebcf090b25c84da75`. Their scoped refresh does not refresh their historical whole-account baselines.

The review inspected decisive current implementations, including `NativeRuntime::new`, `inspection::runtime::acquire`, canonical deployment opening, workbench selection, worker discovery/claim/execution, database readiness generation, transport custody and attribution wrappers. No tests, builds, services or measurements ran for this review.

## What belongs together

The functional target is interactive modeling and re-solving, reusable studies, truthful results and diagnosable recovery. Three responsibilities have different inputs and lifetimes:

| Responsibility | Owned decisions and state | Necessary dependencies |
|---|---|---|
| Scientific preparation and assessment | Physical interpretation, structural validity, numerical policy, reusable preparation and original-model checks | Authored sources, physical context, mathematical/provider libraries |
| Durable application control | Immutable intent, membership, claims, fences, cancellation, receipts, terminal visibility and bounded selection | Database transactions, indexes, authenticated command actor |
| Physical execution and data delivery | Native workspaces, actual process/task completion, buffers, files and transport drain | Native libraries, host supervisor and selected I/O implementation |

These semantic boundaries do not require three services or a network crossing between every operation. They do require callers to acquire only the capabilities they consume.

The current system already delegates substantial control correctly. Database functions own guarded claims, leases, generations, receipts and cancellation. Readiness acquires bounded database scopes, invokes shared Rust policy through Surrealism and applies hints within the database snapshot. It does not export readiness snapshots to the client. Native execution occurs outside database transactions.

The refreshed account also defeats older premises at inspected sites: study creation resumes bulk insertion; outcomes are compact projections; small payloads are inline; finalization coalesces pages; exact settlement is shared. Those are preserved strengths, not work to recreate.

## Complete architecture comparison

| Architecture | Benefits | Material burden and judgment |
|---|---|---|
| Current Python → PyO3 Runtime → Rust database client | One established scientific API; local preparation and shared resources; existing ownership protections | Browsing/control inherit native package, model/session, resource-budget and loaded-artifact dependencies. Retain scientific use; revise the mandatory durable-caller boundary. |
| Database-facing application, with all preparation requested remotely | Thin callers; durable commands and queries colocate with authoritative state; native workers can serve several clients | Every edit/preparation depends on service availability and remote publication. It does not eliminate workers, native completion or large-data delivery. Credible for service deployments, unnecessarily restrictive as the sole application architecture. |
| Coherent hybrid | Thin durable callers; optional local preparation and explicit ephemeral execution; native workers share the same scientific operations | Requires a deliberate public control contract and package split. Best fit across interactive local work and durable studies. Select this direction. |
| Broad Surrealism-hosted execution | Useful for bounded Rust policy and eligible computations beside data | WASI/library compatibility, host allocation, transaction duration, server contention and physical resource ownership remain consequential. Eligible capability by capability; not an established replacement for the complete native workflow. |

The hybrid is one durable control path with optional local scientific capabilities. It must not become two independently maintained study schedulers or two scientific admission implementations.

## Preferred target interfaces and execution

### Public durable client

Expose a typed durable client whose construction requires connection/authentication configuration and finite request/response policy. Browsing and cancellation require neither `EngineSettings`, a bound model nor a qualified native producer.

Its operations should express application intent: submit an exact source/request, inspect or cancel a study, obtain bounded headers, capture a cohort, select a retained manifest and reconcile an operation receipt. These are contract examples, not adopted API spellings.

Commands carry a stable operation identity, immutable intent and required revision premises. Responses distinguish accepted, refused and unresolved outcomes. A lost acknowledgement is settled against the original receipt; reconnecting must not silently issue a different command. Live browsing, frozen membership and scientific usability remain distinct.

Client-side generated validation improves usability; the server enforces durable invariants. A requester may ask for scientific work without acquiring permission to fabricate scientific admission or terminal results.

The official Python SDK's synchronous/asynchronous HTTP and WebSocket clients are eligible implementations. `DEFINE API` is a promising constrained ingress. Its body runs with ordinary table permission checks disabled after endpoint authorization, so endpoint-specific actor and operation checks are essential. Neither mechanism is qualified here as a drop-in replacement. Select the exact release and transport during boundary design, using the contract above.

### Local scientific capabilities and native workers

Retain local authored-model preparation, reusable evaluators, generated designs and explicit ephemeral execution where useful. Recording transfers exact sources and requests into the same durable protocol. The worker invokes shared scientific owners rather than reproducing their meaning.

Workers retain native admission, producer qualification, workspace ownership, cancellation, independent numerical assessment and physical joining. The host supervisor owns process establishment and teardown. Database state can express desired work and durable ownership without pretending that a committed record proves a host process ended.

Move additional database-only compositions inward when they can complete under one bounded transaction. Prefer ordinary SurrealQL/functions before adding guest calls. Surrealism's `sql`, `sql_with_vars` and `run` permit further composition under the caller transaction, but require explicit capability and effect contracts. Current purity restrictions are selections, not capability absence.

Notifications may wake workers or observers. Authoritative snapshots or changefeeds must establish catch-up: selected gRPC Subscribe rejects `resume_from`, and a lost notification cannot establish absence of work. Events are not a lease-expiry clock or an exactly-once external executor.

### Data and transport

Keep compact metadata separate from scientific payload hydration. Large Arrow consumers should obtain bounded data through a reader requiring data capabilities, without automatically constructing mathematical services.

Evaluate colocating payload/catalog transitions where one database transaction genuinely replaces cross-resource coordination. Large sealed files still need integrity, durability, attachment and external-reader protection when they remain outside that transaction.

The current custom transport has real consumed obligations: pre-Prost shape admission, connection-bound sessions, copied-buffer flush custody and descendant joining. A Tonic decoder could remove duplicate framing, but encoded buffering precedes its callback and supplier stubs hardcode `ProstCodec`. Channel executors and connectors are useful alternatives, not established substitutes for the complete contract. Do not preserve every custom mechanism by default; do not delete protection merely because a library supplies an adjacent capability.

## Findings and corrective judgment

<a id="r01"></a>

### R01 — Separate durable callers from scientific construction

Primary diagnosis: [DA-046](../audit/study-execution-and-database-boundary.md#da-046). Principles: AP-01/02/06/07; DP-10/16/17.

`NativeRuntime::new` acquires the process runtime, verifies the loaded extension and constructs the local workflow runtime before canonical opening. `inspection::runtime::acquire` builds `SharedRuntime`, scientific sessions, `BoundModel` and the executor/async bridge. Yet `study_workbench_page` consumes issuing-store/cursor identity, activity, response storage and a compact query.

A cold browser therefore inherits unrelated scientific construction failures; a warm browser inherits scientific-budget compatibility. The root is reused, not rebuilt per page, and construction does not prove solver execution. Nevertheless, the public dependency is unnecessary.

The correction includes Python packaging. `python/pse/__init__.py` eagerly imports `_build` and scientific exports; `_build` binds native symbols. Adding a nominally thin client under that eager initializer would preserve the dependency.

Delete migrated durable-only Runtime/PyO3 forwarding and producer-admission requirements, migrate consumers and tests, and make the control import path independent of scientific initialization. Preserve qualification where computation, executable reuse or scientific publication actually depends on it.

Verification must demonstrate browsing and cancellation without loading the native scientific extension, while unauthorized result publication still fails. This distinguishes boundary removal from renaming.

<a id="r02"></a>

### R02 — Resolve attribution lifetime explicitly

Primary diagnosis: [DA-047](../audit/study-execution-and-database-boundary.md#da-047). Principles: AP-02/05/06; DP-08/21.

Observation, OpenTelemetry, span and subscriber wrappers have different poll/destruction behavior. `ObservedBody` explicitly scopes both; other APIs explicitly describe polling scope. The difference alone does not establish a permission or drain defect.

For boundaries promising lifecycle attribution, include relevant cancellation/destruction diagnostics under the captured owner and test restoration. Preserve intentionally polling-only contracts where appropriate. No universal wrapper is selected, and no misattributed event was reproduced. This bounded uncertainty does not justify rebuilding the ownership system.

## Change scenarios and preservation

| Scenario | Expected target behavior and settling evidence |
|---|---|
| S01: Browse/cancel from a fresh Python process | No scientific runtime admission; typed actor/command checks and bounded responses |
| S02: Add a study composition or physical model | Extend shared scientific/policy owners; no parallel client/server scientific interpretation |
| S03: Lose acknowledgement or replace a worker | Reconcile original receipt; stale fences cannot publish; actual old work remains owned until drained |
| S04: Grow case count or result detail | Reuse preparation; compact queries and demand-shaped hydration; coordinated native capacity |
| S05: Lose notifications or database connectivity | Re-establish authoritative state; distinguish unavailable observation from scientific failure |
| S06: Replace transport/library | Preserve consumed allocation, session, outcome and completion contracts or explicitly revise them |

No new physical quantity or formulation is introduced by this recommendation:

| Boundary | Preserved scientific meaning |
|---|---|
| Source/physical inputs | Units, basis, reference state, envelopes and exact source revisions remain under their scientific owners |
| Preparation/evaluation/solve | Declared variable roles, structural rejection, guards, derivatives, scaling and solver capabilities remain unchanged |
| Outcome/publication | Independent residual/domain/closure assessment, original completion, partiality and warm-start dependencies remain mandatory |

The inspected `modeling::assessment` separates native status from original-model acceptance. No unit/property conformance suite, derivative campaign or physical reference comparison was renewed. Numerical-stage implementation and tolerances are preserved contracts, not newly qualified results.

## Architectural foundations and gates

These judgments concern the current bounded implementation; proposed corrections do not make it pass.

| Foundation | Verdict | Basis |
|---|---|---|
| AP-01 | Violated | R01 couples catalog access to scientific construction |
| AP-02 | Satisfied in inspected durable operations | Explicit revisions, receipts, refusals and typed outcomes; replacement must preserve them |
| AP-03 | Satisfied in inspected composition | Shared study policy and scientific operations; no need for a second scheduler |
| AP-04 | Satisfied in inspected scope | Scientific intent, operational facts and assessed results have distinct governing owners |
| AP-05 | Unresolved narrowly | R02's lifecycle-attribution expectation needs settlement |
| AP-06 | Violated | Public metadata callers require unrelated native/model dependencies |
| AP-07 | Violated at caller boundary | R01 imposes unnecessary construction and retained compatibility obligations; no speed claim |

| Gate | Judgment and limit |
|---|---|
| G1–G2 | No current violation established in inspected authorities/distinctions |
| G3 | Unresolved for remaining assembled acceptance |
| G4 | No undeclared scientific effect established; target client effects must remain explicit |
| G5 | Unresolved for actual managed completion/publication/drain |
| G6 | No new violation established; alternative marshaling/reuse remains unqualified |
| G7 | Passing claims remain limited to their named routes; corrected overclaims are not renewed |
| G8 | No complete library replacement established for retained transport obligations; alternatives remain eligible |
| G9 | Fail through AP-01/06/07; R01 requires revision |
| PS-G1–PS-G3 | Not requalified; no new scientific defect established by this boundary review |

The test account preserves focused inheritance controls and distinguishes them from managed acceptance. Twelve inherited-custody tests passed with zero failures; broader selected controls and a worker build have separate limits. Actual managed/native/Python/trace acceptance remains open. Historical receiver/controller and future-construction failures are not asserted to remain current bugs.

## Rule impacts, migration and decision

| Impact | Proposed change | If retained |
|---|---|---|
| <a id="rc01"></a>RC01 | Revise blueprint §21 and Python-boundary rules requiring native Runtime composition for durable access; separate control imports and contracts | R01 remains; a native-backed thin client can reduce some coupling but cannot deliver native-independent access |
| <a id="rc02"></a>RC02 | Revise the canonical client/protocol selection in Plan57 and blueprint §20.6 where HTTP/WS/API ingress replaces mandatory gRPC for public control | Use a narrow native control client; retain connection-bound gRPC semantics |
| <a id="rc03"></a>RC03 | Scope loaded-artifact/producer requirements, including ADR-0164-related composition, to operations consuming scientific provenance | Browsing/control retain unnecessary producer dependencies |
| <a id="rc04"></a>RC04 | Change module capability selections only for specifically justified transactional compositions | Keep pure modules; pursue database-only composition through existing functions |

No rule changes take effect through this review. No SHOULD exception is requested.

Migration dependencies are the command/actor contract, import/dependency separation, then consumer migration and deletion of replaced paths. Native admission and data ownership must remain explicit throughout. The existing Plan57 coordinator/57j remains the proposed disposition owner; adoption and scheduling require subsequent instruction.

**Decision: Revise the current boundary and carry the coherent hybrid into plan creation.** Prioritize R01. Settle R02 proportionately, preserve actual managed acceptance obligations, and select transport/modules against their complete consumed contracts. The research companion retains further sampling, surrogate, array and storage opportunities; they do not justify bundling scientific method changes into this boundary correction. Startup, memory, throughput and capacity benefits remain unmeasured.
