---
id: ADR-0169
title: Encode immutable results by relation and bounded cursor
status: proposed
date: 2026-10-09
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-06, AP-07, DP-09, DP-22, PS-13]
blueprint: [§19.2, §21.1]
review: docs/design_review/reviews/design_review_relation-local-result-transport-target_2026-10-09.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An advertised relation cannot be projected in bounded chunks without changing its scientific meaning.
verification: Plan 33 EFF07 relation isolation, whole-table refusal followed by bounded export, exact global manifests, interruption and escaped-buffer controls; scoped target review.
standard: {core: '3.4', process-simulator: '1.5'}
scenarios: [docs/design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#s05]
---

# ADR-0169: Encode immutable results by relation and bounded cursor

## Context

F09 identifies complete-map encoding behind selective access, Python name discovery and
durable publication. Direct trajectory and enclosing run access also disagree about encoding
retries. The maintainer selected relation-local failures and bounded export in Plan 33.
Scientific completion remains the authority under blueprint §19.2.

## Scope

Amend blueprint §19.2 and §21.1 for ordinary, modeling, trajectory, fit/shooting, Python and
durable result transport. This changes representation/error boundaries, not scientific
completion, schemas, original numerical assessment, external parity or source retention.

## Drivers

Discover names without allocating payloads, request one relation despite an unrelated defect,
and publish large relations under bounded transport memory. These variations belong to the
encoding owner. Clones share immutable completion and relation-local encoding state; mutable
cursors and delivery belong to each request. Escaped buffers retain their allocation owners.

## Options

The complete-map cache materializes unrelated payloads and makes failure depend on wrapper
choice. Caching every stream chunk repeats that memory cost. Separate convenience and durable
encoders duplicate scientific projection. Select one relation-aware projection and bounded
cursor reused by convenience aggregation, Python streams and durable writers. This removes
whole-map work and scopes recovery to the demanded representation, at the cost of migrating
callers and preserving explicit global storage coordinates. Existing checked builders, leases
and manifest sealing supply the mechanisms; no new framework or scientific registry is needed.

## Outcome

Immutable completion/request supplies the relation inventory and scientific headers. Name
discovery never encodes payloads. Each relation has one projection serving request-owned bounded
cursors with checked chunks and explicit terminal completion/error. Empty relations retain
schema and inventory membership; a delivered prefix never establishes relation completion.

Clones share intrinsic encoding failures per relation: content, schema or encoding-contract
defects are sticky for that relation without poisoning another or changing scientific completion.
Use typed classification. Cancellation, destination/transport errors and allocation refusal
caused by requested materialization are request-local and cannot poison bounded export.
A convenience cache may retain a successful complete relation; streaming neither requires that
cache nor retains every emitted chunk.

Single-table convenience aggregates only its requested relation. Complete-table convenience
requires every advertised relation and may independently refuse container/assembly allocation.
Python names come from inventory; selected access consumes its relation as a TableStream.
Standalone modeling results and diagnostics use this same selective boundary.

Durable export owns a cursor per required relation with stable global rows, block ordinals
and exact ranges. Public simulation order remains sample-major; canonical traversal may be
output-major under its existing indexed-storage contract. One projection supplies both orders
and their exact coordinate correspondence. Never sort or renumber independently per chunk.
Seal every required relation, including empty schema/manifest membership, before activation.
Backpressure bounds outstanding chunks; retained escaped buffers remain separately charged.

### Consequences

Migrate RunResult, trajectory, fitting/shooting, standalone results/diagnostics, Python and
durable callers. Delete complete-map-first implementations and tests solely for retired retry
semantics. Complete convenience APIs aggregate the same projection. Regenerate signatures if
changed. Disposable internal state needs no legacy encoder or historical reader.

### Compensating controls

Exercise unrelated intrinsic failure, sticky clone-shared failure, complete-access refusal,
inventory under pressure, whole-batch refusal followed by bounded export, exact global chunk
coverage, empty relations, interruption without activation and escaped-array ownership.
Fresh round trips establish fidelity; analytical and external references supply scientific
expectations. Do not make invalid historical internal values an oracle.

### Confirmation

The scoped target review accepts the Proposed design; EFF07 controls remain required. This record claims no
executed remedy or measured improvement; Plan 33 owns implementation and qualification.

## Pros and cons

Demand follows transport work and scientific ownership stays local. Integration must preserve
exact storage ranges and distinguish intrinsic defects from request resource/delivery failures.

## More information

- [Plan 33 EFF07](../plans/33-efficiency-principles-remediation.md#eff07), sole status owner.
- [Review F09](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f09).
- [Workflows/results](../authoritative_design/sections/workflows-and-results.md).

## Status history

- 2026-10-09: Proposed for maintainer-authorized Plan 33 execution; target review and product acceptance remain separate.
