---
id: ADR-0267
title: Own native discovery, observation and incremental result delivery
status: proposed
date: 2026-11-10
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, DP-15]
blueprint: [§20.5, §21.7]
review: docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A named consumer requires retained notification replay, native bucket publication closes its durability gap, or a selected embedding model supplies source-owned semantic retrieval.
verification: Plan72d-h late-match/rank-epoch, path completeness, subscription race/revocation, tentative stream terminal, Collector delivery, scratch readiness and external consumer settlement controls.

---

# ADR-0267: Own native discovery, observation and incremental result delivery

## Context

Plan72d-h execution is authorized after the Plan73 mathematical/result cutover.
The selected v4 source supports gRPC subscriptions and batched query responses.
Current capped discovery, full-response assembly and absent exporter configuration
leave opportunities to improve complete operations under blueprint §20.5/§21.7.

## Scope

Amend current discovery, observation, result-delivery and diagnostic contracts.
Scientific meaning and original effect/protection authority retain their owners.
This proposed decision records the authorized execution target, not acceptance.

## Drivers

Complete selected populations must precede page caps. Ranked continuation must survive
only an unchanged ranking population. Observation and tentative delivery must retain
their original physical owner without becoming scientific or effect authority.

## Options

Reuse native SurrealQL, direct gRPC and the existing service/fixture/report owners.
An SDK or WebSocket sibling would not remove current admission/custody obligations.
Native local buckets lack fsynced atomic publication and recoverable reservation failure;
retain sealed files. Query-first summaries and rebuilds avoid incomplete dependency maintenance.
The maintainer selected lexical/graph retrieval without an embedding-model lifecycle.

## Outcome

Search filters the full selected population before paging; ranked cursors bind a corpus
epoch covering staging/reset and analyzer/index interpretation. Evidence-bearing paths
retain occurrence identity, finite work bounds and honest completeness.

Follow owns a dedicated fixed-authentication gRPC generation. It registers before reading,
coalesces invalidations, resnapshots on loss and explicitly kills/drains registrations.
An identified snapshot is an as-of observation; quiet sockets do not establish currentness.
Compact revalidation occurs within 30 seconds or the earlier original/session deadline;
next_revalidation_at is not a lease or an immediate-revocation guarantee.

Incremental query delivery retains request/generation/transaction/allocation/driver custody
through terminal accounting. Batches remain tentative until QueryEnd, EOF and trailers.
Cumulative query admission never resets when a batch is released. Subscriptions instead
have bounded per-frame/in-flight admission and finite lifetimes.

The owned server receives explicitly selected Collector gRPC settings and private headers;
environment clearing remains. Delivery must be observed before claiming signal availability.
Explicit scratch initialization must establish the actual spill store before readiness,
including a narrow maintained source correction where warning-and-disable would hide failure.
Ordinary supplier scheduling and the maintainer's host placement remain unchanged.

### Consequences

Move consumers and delete capped selection, duplicate hydration/aggregation/plan parsing
and full assembly on selected stream routes. Preserve Plan73 original typed result readers.
SurrealKit owns inner logical setup; the existing native fixture owns physical settlement.

### Compensating controls

Keep exact issuer/cut/interpretation and permission checks, native cumulative wire preflight,
explicit deadlines, source-bound server preparation and independent malformed/readback cases.
Observations never acknowledge writes or renew/release protections.

### Confirmation

The functional review and selected source contracts support this design. Actual tests and
remaining obligations belong to Plan72d-h; Plan72i retains assembled series acceptance.

## Pros and cons

Native composition removes avoidable materialization and additional protocol participants.
It requires explicit streaming/follow lifetime handling and honest observation gaps.

## More information

[Plan72](../plans/72-surrealdb-functional-improvements.md) and its d-h companions;
[library briefings](../library_leverage/libraries/surrealdb/README.md).

## Status history

- 2026-11-10 — proposed.
