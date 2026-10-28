---
id: ADR-0232
title: Separate durable control from scientific runtime construction
status: proposed
date: 2026-10-28
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-06, AP-07, DP-01, DP-15, DP-19]
blueprint: [§3.3, §20.6, §21]
review: docs/design_review/reviews/design_review_native-independent-durable-control_2026-10-28.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: The selected HTTP ingress cannot preserve actor isolation, exact command settlement or original-deadline bounded delivery.
verification: Native-loading-refused public browse/cancel/reconcile, effective-server actor and receipt controls, worker cancellation and actual owner drain; Plan57k/L and 57j own execution evidence.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md]
---

# ADR-0232: Separate durable control from scientific runtime construction

## Context

Review R01/DA-046 establishes that compact durable browsing requires unrelated scientific
composition and loaded-artifact checks. The maintainer confirmed review RC01–RC03 and
conditional RC04, then authorized the bounded browse/cancel/reconcile execution scope.
Current cancellation has study-state idempotence, not a request-specific command receipt.

## Scope

Amend the Python/control boundary and the public-ingress selection of proposed ADR-0164/0214.
The first delivery retains one distribution and adds an async-first `pse.control.ControlClient`.
Scientific workers, numerical meanings, persistent MX and protected data readers retain their
existing owners. No new crate, synchronous bridge, backend replacement or module privilege
extension is selected. Existing records remain proposed; this document changes no ADR status.

## Drivers

Connection/control operations must acquire only their actual capabilities. One authoritative
database transition must serve public and native callers. Lost acknowledgement must preserve
truthful original-command identity without pretending cancellation proves physical completion.
Local construction and fault tests must work with the scientific extension unavailable.

## Options

Retaining Runtime forwarding fails native-independent access. The released SurrealDB Python
SDK 2.0.0 buffers HTTP responses before decoding without a public admission hook; replacing
its transport internals adds obligations beyond a narrow control facade. Its embedded-engine
imports also add an unnecessary native closure, though that alone is not an R01 violation.
Select constrained DEFINE API ingress with HTTPX2 2.16.0 async raw streaming and AnyIO 4.15.1
original-deadline cancellation. This reuses maintained HTTP mechanics while keeping the
application contract explicit. Ordinary SDK queries remain eligible for other consumers.

## Outcome

Durable clients use generated Rust-owned request/result types and explicit connection/auth
configuration, without EngineSettings, BoundModel or scientific producer admission. Browse
retains finite pages and issuer/predicate-bound cursors. Cancel binds operation ID and immutable
study intent; actor and issuing database scope the command. Persist result and cancellation
atomically. Replay returns the original result; identity reuse for different intent refuses.
Receipt keys encode the issuing-store, authenticated actor and operation identity as an unambiguous tuple. Compact receipts, including authenticated domain refusals, survive study retirement for the issuing database lifetime; restored or foreign issuer selections cannot reuse them silently. Reconcile reads that original receipt. An unobserved receipt does not prove an in-flight command cannot commit. Uncertain delivery never grants automatic new mutation.

DEFINE API authenticates record actors with explicit enabled browse/cancel grants; request
bodies confer no actor authority. Handler checks are essential because ordinary table checks
are disabled inside the API. Authorization precedes receipt reads and writes as well as state access. Native internal entry requires trusted system identity, never a caller-supplied actor bypass, and shares the guarded transition. Restore changes issuing-store identity, so old command identities cannot alias restored ones. Generation/count encoding preserves exact integers.

### Consequences

The package initializer, generated contract/identity helpers and codec must stop eagerly
loading science on the control path. Scientific consumers retain actual-artifact and registry
checks at the actual _build/scientific gate, including direct private-module imports. Source-only version metadata may be unavailable; it must not fall back to loading native_version(). Client request/response limits and the original deadline precede
expansion; unexpected compression/redirects refuse. Async resource cleanup settles local
HTTP owners; the operation deadline does not promise deadline-bounded native drain, and database receipts alone do not prove native drain or remote rollback.

### Compensating controls

Effective-server tests distinguish unauthorized actors, changed-intent replay, unknown replies,
original deadlines, bounded decoding and stale publication fences. All generated changes come
from their declaration/generator owners. Source-only test startup must not preload the extension;
actual Arrow correctness routes still require force-validation.

### Confirmation

Proposed pending scoped target review and implementation. Review baseline is `2138a72b5` plus
the explicitly retained operations inheritance corrections; the refreshed audit remains the
source diagnosis. No behavioral acceptance or measured performance benefit is claimed here.

## Pros and cons

The selected boundary removes scientific initialization from compact access and preserves one
state authority. It adds an explicit public actor/receipt contract and HTTP admission adapter,
whose whole-operation and cleanup semantics require actual server qualification.

## More information

[Plan57 integration](../plans/57-library-leverage-and-correctness.md#database-boundary-review-integration),
[57k](../plans/57k-durable-control-and-python-boundary.md),
[57l](../plans/57l-database-control-and-synchronization.md), and the review research companion
retain library source details and the single scheduled-disposition route.

## Status history

- 2026-10-28 — proposed; maintainer authorized scoped implementation, formal status unchanged.
