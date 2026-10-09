---
id: ADR-0167
title: Frame flow projection collections and parent boundaries explicitly
status: proposed
date: 2026-10-09
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, DP-07, DP-10]
blueprint: [§5.3, §17.4]
review: docs/design_review/reviews/design_review_flow-projection-v2_2026-10-09.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A consumer proposes using a flow fingerprint as the sole equivalence key across physical registries.
verification: Independent scoped design review; flow framing golden and reordered-declaration controls, parent segmentation controls, physical policy and forbidden-cycle tests; runtime recycle and native tear consumers.
---

# ADR-0167: Frame flow projection collections and parent boundaries explicitly

## Context

FlowProjectionV1 emits variable-length port and binding sequences without explicit collection or parent counts. The graph/hash review's U01 inquiry did not demonstrate an admitted collision, but left byte-role segmentation unresolved. The maintainer selected Graph/hash RC04 on 2026-10-09 to remove that premise rather than make future consumers depend on an unproved property.

## Scope

Amend blueprint §5.3 and §17.4 for newly admitted flow projections. Preserve historical V1 identity meaning, full graph equality and physical admission. This decision does not select durable fast hashing, persistent compiler state or a sole-fingerprint flow cache.

## Drivers

Flow admission owns unit/port occurrences, connection occurrences, tear policy and checked physical bindings. Identity framing belongs to pse-ids; structural admission writes the canonical projection. Consumers must receive the new identity through the existing FlowGraph contract without rediscovering framing or changing numerical policy. Parent boundaries must remain evident when port and binding cardinalities vary.

## Options

Retaining V1 avoids identity churn but leaves the unresolved segmentation premise. Adding counts under V1 would reinterpret historical identities. A new tagged/count-framed V2 keeps the existing BLAKE3 owner and admitted projection, with a small local encoding change and explicit downstream identity change. Replacing the graph with a generic serialized operation language or introducing another hash family adds unrelated machinery. Select V2.

## Outcome

New FlowGraph admission uses `Frame::FlowProjectionV2` (`pse.flow.projection.v2`). Write, in order, the UTF-8 tags `nodes`, `node`, `ports`, `port`, `connections`, `connection`, `bindings`, `binding`, `decisions`, `decision` at their respective collection/item boundaries. Each collection tag is followed by its u64 item count, including ports separately for each node and bindings separately for each connection. Each item tag is followed by the existing canonical fields in their existing order. Tags use FramedHasher::str, counts use u64, and identifiers use the existing typed digest framing. Preserve sorted canonical declarations, canonical f64 conversion/cost bits and policy values. No current admission path emits or falls back to V1.

### Consequences

New flow documents, tear oracle assumptions/compatibility and recycle causal identities change because their flow key changes. Their public record shapes and fresh-session/start policies remain. Retain the V1 catalog entry and historical stored identities without recomputing or relabeling them. No stored FlowGraph family or migration was identified by the bounded inquiry.

### Compensating controls

The graph key covers quantity/unit identifiers and conversion values, not every physical-registry field. V2 establishes explicit sequence boundaries; it does not establish complete semantic equivalence across physical registries. Full graph equality and current checked context remain mandatory wherever required. Tests distinguish raw encoding fixtures from admitted physical examples, and preserve exact occurrence witnesses, policies and canonical floats. No cross-context sole-key cache is introduced.

### Confirmation

The independent review accepted the bounded target design at Proposed/Interface-checked evidence level before dependent V2 implementation. That historical review remains unchanged. **Implemented:** current admission emits the specified V2 grammar through the existing identity owner; the V1 catalog meaning remains historical with no production fallback.

**Tested — 2026-10-09:** the root ran the following direct checks through the pinned checkout environment with explicit force-validation supplied by the recipes and Nextest's `local` profile. All selected tests passed against the zero failure target; the tested source was unchanged at the final targeted acceptance boundary.

| Command | Result and scope |
| --- | --- |
| `just unit-package pse-structural flowsheet --profile local --status-level pass --final-status-level pass` | 6/6: population limits, V2 empty/count/tag preimages, raw-ID parent boundaries, reordered canonical inventories/full equality, forbidden-cycle witnesses and affine physical bindings |
| `just test-package pse-ids -p pse-relations -E 'package(pse-ids)' --profile local --status-level pass --final-status-level pass` | 61/61: identity owner and frame catalog tests |
| `just unit-package pse-backend-native tears --profile local --status-level pass --final-status-level pass` | 3/3: tear consumer controls |

The final native runtime selection additionally passed 24/24, including the flow tear deadline and authored causal recycle controls, after temporary inquiry hooks were removed. Its exact selection, native/force-validation conditions and other tested boundaries are recorded in the [hashing evidence](../design_review/evidence/graph-hash-followups-2026-10-09/hashing-results.md#permanent-implementation-acceptance). These tests establish the selected local behavior, not sole-key physical equivalence, a performance benefit or full Plan 28 qualification. Production PC-SAFT smoke, Python journeys and final gates remain pending.

Implementation acceptance remains owned by Plan 28k and the affected Plan 28b/28f/28e journeys. Proposed status remains until the decision-PR route; implementation authorization is already explicit.

## Pros and cons

Explicit framing makes collection and parent boundaries locally testable and keeps identity ownership unchanged. It intentionally changes current identity values and requires consumer coverage; it supplies no measured latency benefit or stronger physical equality claim.

## More information

- [Graph/hash investigation and execution owner](../plans/28k-graph-kernels-and-hashing-investigations.md)
- [Original U01 inquiry](../plans/28k-graph-kernels-and-hashing-investigations.md#gh4--flow-framing-and-persistent-pure-reuse)
- [Original review](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md)
- [Identity contract](../authoritative_design/sections/identity-and-publication.md#section-5-3)
- [Structural contract](../authoritative_design/sections/numerical-execution.md#section-17-4)

## Status history

- 2026-10-09 — proposed after maintainer selection of Graph/hash RC04; independent review and implementation acceptance tracked separately.
- 2026-10-09 — Implemented/Tested evidence added after bounded review acceptance, direct framing/identity/tear checks and final runtime flow/recycle controls; status remains proposed and broader qualification remains separate.
