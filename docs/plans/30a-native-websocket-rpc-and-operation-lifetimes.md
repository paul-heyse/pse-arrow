---
title: Native WebSocket RPC and operation lifetimes
status: draft
date: 2026-10-08
adrs: [ADR-0164]
review_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#4-revealing-scenarios]
---

# 30a: Native WebSocket RPC and operation lifetimes

## Responsibility and selected direction

This companion delivers WP01/WP02's application RPC correction. [Plan 30](30-websocket-and-persistent-agent-environment.md#finding-dispositions)
owns dispositions and confirmed rules. Canonical semantic operations stay in
`pse-operations`; one thin integration owns native WebSocket configuration and request
lifecycle. Database schema, physical meaning, canonical acknowledgments and scientific
retry policy remain at their existing owners.

**Interface-checked:** the reviewed 3.3.0 SDK uses native binary values and typed
`IndexedResults`, but finite connection capacity bounds only the route channel.
Pending requests, deferred routes and session replay history are separate retained
state. The remote WS query stream buffers; server `query_cancel` is not a cancellation
handle for this consumed path. Dropping the caller's future cannot guarantee that an
already queued request will not be transmitted.

**Proposed selection:** use the existing typed buffered WS engine with the smallest
reviewed, reproducibly pinned source patch to the affected 3.3.0 SDK family. Retain its
codec, session routing and result types. Do not build another RPC stack or wait for an
unscheduled upstream release. A0 determines the exact affected crates and source
delivery, preserving exact dependency declarations/lockfiles; a local vendored patch
of the external family is the default when no suitable released fix exists. Keep
upstream licenses and an explicit patch description. No workspace crate is introduced
solely to duplicate the driver.

## A0 — Establish the consumed lifecycle and patch contract

Inspect the exact `Query`, engine context/request data and route owners, WS dispatch,
deferred replay, sink acquisition and session teardown identified in the
[review evidence](../design_review/evidence/websocket-and-persistent-agent-environment-2026-10-08.md#transport-capability-and-contrary-evidence).
Confirm the source family's actual API before fixing the patch; Context7/current
official docs supplement, not replace, exact-release source. No dependency family
upgrade or wholesale re-resolution is a prerequisite.

Carry request-local metadata through all used application methods: original monotonic
deadline, cancellation, bounded admission and dispatch state. The patch must check
expiry/cancellation after local awaits, immediately before sending, and on every
deferred-route flush. Expired queued work completes as **not dispatched**; response
receiver closure is an additional check, not the deadline authority.

The operation's original clock includes application admission, SDK queueing, reconnect/
replay, request transmission and complete application response processing. Conflict
retry consumes its remaining time; it does not mint another full timeout. Preserve
the existing longer activation budget instead of imposing the ordinary read budget
on every operation. Native task deadlines remain separately owned.

Local clocks and server expiry have distinct meanings. Use client monotonic time for
waiting and dispatch. Capture the same operation's UTC expiry for the authenticated
same-host server fence, and derive remaining SQL timeout from the remaining original
budget at submission. UTC conversion is not proof of a monotonic server deadline;
clock discontinuity and server timeout/commit behavior remain explicit support limits.
Never claim hard abort or that durable commit finishes before a caller deadline.

A0 must establish, with exact source and bounded controls, how expiry/lease/generation
checks and remaining-time SQL `TIMEOUT` compose for every used mutation shape,
including activation and initialization. A fence is checked inside the guarded
transaction, including immediately before commit where applicable. Committed effects
are reconciled by immutable identity. If a proposed timeout expression or commit fence
cannot implement the claimed guarantee, retain the truthful unknown-outcome contract
and block that stronger claim; do not substitute an outer timeout as proof.

## Submission, cancellation and settlement

| State | Required behavior |
|---|---|
| Waiting/queued, transmission not begun | Expiry or cancellation removes the route without sending it, completes a definite local refusal and releases local admission. |
| Transmission begun | Interruption may have delivered bytes. Mark it uncertain; invalidate a partially written connection rather than call it not dispatched. |
| Submitted, awaiting complete result | Caller expiry stops waiting. A bounded drain owner retains request correlation and admission while consuming the final response. |
| Complete typed response | Inspect every statement. A definitive transaction conflict permits recomputing the guarded decision within the original clock; other failures follow their typed meaning. |
| Response lost or connection invalidated | Preserve an unknown write outcome and its original operation ID. Reconcile authoritative acknowledgments/state; no mutation replay, new scientific attempt or protocol fallback. |

Regular buffered queries use cancel-before-dispatch and drain-after-dispatch. Do not
invoke `query_cancel` for a query that was never registered through its streaming
protocol. SDK clones/attached sessions are not independently abortable physical
connections. Independent cancellation scopes receive separately owned connections.
Connection teardown affects all its siblings, whose unfinished operations must each
be classified and settled.

A drain timeout bounds local transport recovery, not proof that server execution or
commit stopped. Socket closure may release dead transport buffers; canonical effect,
server recovery and any native/cgroup obligations stay owned until their respective
settlement/drain conditions hold. The global server timeout is a backstop. Do not
free a scientific capacity owner merely because its observing caller returned.

## Bounded realization and completed reads

Initial **Proposed** application defaults are 32 outstanding application requests per physical
connection plus two reserved control requests, route capacity 34, at most 34 deferred routes, one immutably selected
application session, and at most 16 bounded setup/replay entries. Setup traffic also
uses its bounded reserved admission, so full application occupancy cannot prevent
replay/settlement control. Reject additional setup before transmission when its replay allowance
is full; replace the connection only at a controlled idle boundary. Do not discard
replay history without proving equivalent session semantics. Application operations
are never added to a reconnect replay log.

Retain a 4 MiB complete native wire-message target with finite frame/message limits,
128 KiB read/write buffers and an 8 MiB maximum write buffer. A1 checks actual SDK
configuration relationships and native codec overhead. Encoded request admission and
decoded result reservations remain separate; 32 requests are not 32 uncharged giant
replies. A0/A1 verify that chosen limits cannot deadlock replay/control traffic.

Queries select bounded canonical pages/blocks whose worst-case complete reply fits
the native message budget, including metadata and statement framing. Use authoritative
admitted payload bounds and selected cardinalities, not an average row size or only
the old protobuf accounting. Large legitimate results use additional complete pages;
they are not truncated or put in an oversized frame. Broad namespace inventory is an
explicit administrative operation, not an ordinary unbounded application query.

The server encodes ordinary replies before client receive admission, so a receive cap
alone is insufficient. A2 audits every canonical query shape and closes any unbounded
response path with bounded selection or an explicit supported refusal. Result-memory
reservations cover decoding and escaped Arrow ownership; retain original source/result
protection until the page/export's completion obligations end.

Consume all indexed statement errors before admitting a page. Arrow streaming composes
completed pages and existing coverage/order checks; it does not promise incremental WS
rows. Empty successful selection, late statement errors and an incomplete final page
retain distinct outcomes. Provisioning/import/backup support remains the explicit
control-plane contract in 30b, not an implicit application transport fallback.

## Packages and consumer migration

| Package | Inputs and delivered behavior | Migration/deletion and focused acceptance | Status |
|---|---|---|---|
| A0 — Exact lifecycle contract | R0 rule route and source above. Select/pin the minimal SDK family correction, original-clock/SQL support contract and response budgets. | Expired replay/sink wait, interrupted send, definitive conflict, clock/timeout boundary and lost-commit controls distinguish supported claims. Document remaining limits and exact affected dependency pins. | Planned |
| A1 — Working bounded native connection | A0; 30b B0 context selection; existing codec. Implement immutable WS binding and corrected queue/pending/replay ownership. | Move ordinary/activation connections and all used RPC method adapters. Test native Bytes/record IDs/unsigned/IEEE payloads, typed errors, expired queued work, session/replay limits and siblings. Delete replaced gRPC connection/configuration after migrated controls pass. | Planned |
| A2 — Complete operations and pages | A1 and existing guarded operations/result layout. Implement original-clock retry, submitted-work drain and acknowledgment settlement; close response/materialization gaps. | Migrate source/revision/protection/publication, run/study/activation, result/analysis/retention, Python/worker and fixture consumers. Test uncertain writes, all statement errors, exact multipage Arrow, late failure, byte limits and slow/abandoned readers. Delete displaced timeout/streaming helpers and gRPC-only accounting/tests. | Planned |
| A3 — Integration handoff | A2 and working B/C/D consumers. Reconcile deployment and all remaining application callers. | Actual server disconnect/reopen and mixed read/write/study journeys under new controls; D3 owns assembled acceptance. Remove obsolete application gRPC dependency features and probes only when their repair/evidence owner no longer consumes them. | Planned |

Compile touched packages with `just check-package pse-operations` and affected callers;
target owner-local codec, lifecycle and protected-result tests with force-validation.
Use owned disposable test contexts for real-server controls. A simulated server may
expose queue/replay timing, but it cannot qualify actual transaction/durability semantics.
Do not delete current diagnostic probes or preserved failing-test materials as incidental
transport cleanup.

## Verification and checkpoint

**Proposed:** package controls must distinguish no transmission from unknown transmission,
timeout from drain, correlation from effect identity, and channel capacity from total
retained state. An earlier gRPC pass is not a WS pass. The expected scientific values
and native edge cases come from independent prescribed facts, not only round-tripping
the same questionable codec.

This plan selects a library-first implementation route. A0 remains a bounded working
prerequisite; no SDK patch or new transport test has been executed in this authoring turn.
Package status stays planned; Plan 30 owns current finding disposition and D3 owns
series-level assembled acceptance.
