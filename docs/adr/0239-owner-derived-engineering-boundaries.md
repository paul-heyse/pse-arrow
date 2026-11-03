---
id: ADR-0239
title: Derive engineering contracts and local service from their semantic owners
status: proposed
date: 2026-11-03
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-04, AP-07, DP-01, DP-19, DP-24, PS-12]
blueprint: [§4.7, §21.8]
review: docs/plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A consumed projection requires independent scientific meaning or cannot preserve exact operation effects, interpretation or custody.
verification: Plan64i independent projection/edit controls and Plan64j independent-client success, intended failure, reconnect and physical settlement journeys.
---

# ADR-0239: Owner-derived engineering boundaries

## Context and scope

The Proto review I01–I03/I05/I06 identifies useful shared contracts across authored science,
Rust-owned actions/documents, independent clients and references. The maintainer accepted
RC01–RC03 as planning directions, including versioned breaking changes, on 2026-11-03.
This proposed decision changes the target in blueprint §4.7/§21.8. Formal review required
for any affected foundational/foreign-boundary change remains outstanding; plan authoring
and operator target selection are not formal ADR acceptance or implementation acceptance.

## Decision and alternatives

Project registry-owned meaning from the registry, and Rust-owned operation/document meaning
from its actual typed/macro owner. Add deliberate stable wire choices at that projection
owner, generate descriptors/Buffa types/adapters/local rules, and retain versioned interpretation
and semantic admission independently of descriptor identity. Prefer the existing Buffa and
ConnectRPC family to adding a second runtime family. A narrow Rust-first shadow adapter can
reopen that choice with complete consumer and maintenance evidence; global protobuf-first
domain replacement is not selected. Serde/Schemars remain useful representations where they
serve actual consumers, deriving shared facets rather than acquiring editable duplicate facts.

Deliver an authenticated local effectful engineering service and an independent generated
client over shared native operation handlers. Passive readers remain passive. Commands address
original owners and their repeatability contracts; progress is observation, not native custody.
Disconnect/deadline/cancel request cannot establish physical termination or scientific permission.
Optional foreign consumers do not establish a mandatory Python product or remote deployment.

## Consequences and confirmation

64i settles annotation/emitted-metadata mechanisms, complete consumer scope and version/cutover
before generation. 64j co-designs the usable service slice. 64h independently constructs reference
answers and retains the scientific denominator. Migrate callers, generated outputs, tests and
stored/public interpretations with each replacement; remove replaced production mechanisms.
No generated file is edited directly and no descriptor hash replaces scientific identity.
Source-parent checks, units/datum, contextual validity, product permission and atomic capture
remain with their native semantic owners. No end-to-end behavior or benefit is claimed here.

## Sources

The [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md),
[64i](../plans/64i-engineering-contracts-and-validation.md) and
[64j](../plans/64j-engineering-services-and-clients.md) retain concrete library contracts,
alternatives, migration scope and closure evidence.
