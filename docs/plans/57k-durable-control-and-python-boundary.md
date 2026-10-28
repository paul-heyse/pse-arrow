---
title: "Plan57k: Durable control and Python boundary"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, design, storage, libraries, validation]
---

# Plan57k: Durable control and Python boundary

## Purpose, baseline and foundation assessment

**Proposed:** make durable commands and compact reads usable without constructing a
scientific runtime or loading its Python extension. This develops review R01/DA-046;
the [coordinator](57-library-leverage-and-correctness.md#database-boundary-review-integration)
owns disposition and confirmed rule directions. Plan creation is authorized; production
execution remains paused pending reconciliation of this target.

Consume the [principal review](../design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md),
its production/test accounts and [research basis](../design_review/evidence/study-execution-and-database-boundary.md).
Review publication is `277eee026`, audit publication `5e8aea0cc`, product baseline
`64a217de2` plus the retained three-file operations inheritance corrections and 57j
checkpoint. Core 3.4, efficiency heuristics 1.0, test assessment 1.2 and process-simulator
1.5 apply. This focused foundation assessment does not reopen the broad audit.

The current PyO3 constructor acquires `inspection::runtime::acquire`, checks the loaded
artifact and builds a local runtime before canonical access. Yet the workbench query
needs issuing-store/cursor identity, activity and bounded response storage. Moreover,
`python/pse/__init__.py` eagerly imports native scientific exports. Merely moving a
method to a different Python file would retain the dependency. Reuse compact operations
queries, original receipts and responsibility-owned generated declarations; separate
their connection/data capabilities from scientific composition. Do not reimplement
scientific admission in Python or in a second application scheduler.

## Target and boundary decisions

A typed durable client receives connection/authentication configuration and finite
request/response policy. Browsing, receipt reconciliation and cancellation acquire no
EngineSettings, BoundModel or native scientific producer. Source submission transfers
exact authored input and request intent; scientific admission and assessed publication
remain with shared Rust owners and admitted workers. Local preparation, generated designs
and explicit ephemeral execution remain available through the scientific package path.

Public operations cover submit/inspect/cancel, bounded headers, cohort capture, retained
manifest selection and receipt reconciliation. These are behavior groups; K01 maps the
actual declared operations and consumers rather than inventing a parallel catalogue.
Commands bind stable operation identity, immutable intent and relevant revision premises.
Results distinguish accepted, refused and unresolved. A lost acknowledgement reconciles
the original receipt before another effect; reconnect supplies no mutation replay right.
Live browsing, frozen membership, readability and scientific usability stay distinct.

Reuse generated Rust-owned documents and typed errors through a leaf Python import.
Select the minimum package/import rearrangement that works in a fresh process with
`pse._native` unavailable, including contract imports and importlib metadata access.
If independent installation requires a distribution/build split, K02 must settle it
explicitly before migration; lazy loading alone proves only import independence.
Scientific entry still verifies its actual extension when first consumed. I02/I03's
conditional native public classes apply to scientific families, not every durable value.

### K02 decision: exact ingress and Python implementation

Compare official SDK sync/async HTTP and WebSocket clients with a constrained HTTP
`DEFINE API` facade, using exact released sources and the effective patched server.
The [official Python docs](https://surrealdb.com/docs/languages/python) are discovery
material; no SDK version is selected by this plan. Context7 development examples do
not establish released timeout, streaming or reconnection behavior.

Prefer one bounded command per transaction. Evaluate actor isolation, typed values,
original deadlines, request/response allocation before expansion, close/cancellation,
unknown dispatch and generated contract integration. A custom facade must justify its
total machinery compared with the SDK. Do not expose arbitrary privileged query execution.
`DEFINE API` bodies bypass ordinary table permissions after endpoint authorization;
explicit principal, resource and operation checks must therefore be part of the design.
Ordinary stored functions retain caller permissions. Neither mechanism is a permission
substitute for the other. The current authenticated loopback deployment remains the
support boundary; this adds no internet service or multi-host rollout requirement.

K02 delivers the exact dependency/release, public transport, authentication lifecycle,
package arrangement, bounds and compatibility decision. If none satisfies the contract,
keep the question open and seek focused design advice; do not quietly substitute a native
client that fails R01. Rust worker gRPC can remain where useful; public ingress and worker
transport do not need identical implementations, but must share one durable domain protocol.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| K01 | pending | Plan57k | Operation/capability and consumer map, actor/receipt contract and rule-amendment scope settled |
| K02 | open | Plan57k | Exact ingress, SDK/package and bounded-delivery decision supports native-independent access |
| K03 | pending | Plan57k | Selected typed client/import implementation and server ingress work together |
| K04 | pending | Plan57k | Durable consumers migrate; replaced forwarding and unnecessary producer prerequisites are deleted |
| K05 | pending | Plan57k | Data-access separation and actual acceptance handoff reach 57j |

### K01 — Shared command and capability contract

Inventory all declared durable operations and actual Python, Rust, CLI/MCP, evidence and
knowledge consumers. Classify connection-only, retained-data and scientific requirements;
explain exceptions individually. Preserve exact source identity, receipt semantics, cursor
issuer/predicates, finite response ownership and negative authorization behavior. L01 owns
server transition design and actor enforcement; K01 owns its public representation. Agree
that small common contract first, avoiding a whole-document K/L dependency cycle.
Prepare the ADR/formal design route for confirmed review RC01–RC03 before K03. Amend proposed
ADRs where appropriate; accepted ADR-0164-related decisions require supersession if changed.
Targeted future controls exercise wrong actor/revision, duplicate identity with changed intent,
cursor mismatch and unresolved acknowledgement without constructing a scientific runtime.

### K03–K04 — Implement and migrate by complete operation

K03 requires K02's settled selection, the applicable decision/design amendments and L02's
working endpoint slice. Implement client construction, finite conversion, authentication and
receipt recovery together. Import-only controls run with native loading refused; endpoint
controls use the effective server, not only a permissive fake. Preserve original deadlines
through conversion, dispatch and delivery. Cancellation of a waiter does not imply rollback.

K04 moves each operation's producer, callers, docs, generated adapters and tests together.
Delete migrated Runtime/PyO3 forwarding and unrelated artifact checks as soon as its replacement
passes targeted tests and callers move. Keep scientific methods and actual producer qualification.
Port capability descriptions and SDK/xtask consumers without adding another authoritative schema.
Coordinate I's generator edits and regenerate through the owner; never hand-edit generated types.

### K05 — Metadata versus data capability

Develop retained manifest/result access without constructing mathematical services. Compact
headers must not hydrate payloads. A data reader acquires exact selection/protection, finite
decode allowance and real delivery custody. It may use a qualified native data implementation;
R01 does not require remote SDK Arrow zero-copy. Preserve independent chunk ownership, physical
metadata, terminal completeness and refusal semantics from A/57j. L04 settles payload placement
before replacing any file protocol. Actual Python/native data consumer qualification belongs to J.

## Verification and handoff

**Interface-checked:** source/research contracts above. **Proposed:** K01–K05; no new client,
SDK pin or behavioral qualification exists through this document. Review scenarios S01/S03/S04/
S05/S06 apply. Targeted tests distinguish absence of native loading from renamed construction,
and permission refusal from setup failure. Joined acceptance must browse/cancel in a fresh
Python process, submit through the same durable protocol, execute on an independently supervised
worker, read bounded results and reconcile interrupted acknowledgement. Rebuild the correctness
extension for scientific consumer changes, preserving its independent evidence boundary.

Startup/memory benefits remain unmeasured. Adding a physical model should change scientific
owners, not the client; replacing ingress should change transport adapters, not receipt meaning.
57j owns actual managed/native/Python/trace acceptance and publication. Package completion does
not close DA-046 until the public path and migrated consumers establish the intended separation.
