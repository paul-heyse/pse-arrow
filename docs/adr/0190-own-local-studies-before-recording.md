---
id: ADR-0190
title: Own local studies before explicit recording
status: proposed
date: 2026-10-15
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, DP-09, PS-10]
blueprint: [§0.5, §19.3, §21, §21.1]
review: docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f09
evidence: Tested
supersedes: []
superseded-by: null
revisit: A new portable scientific input form cannot retain exact admitted source identity through explicit recording.
verification: Public no-store explicit and lazy study controls, exact recorded-source readmission, occurrence ordering and cancellation/owner-lifetime controls in Plan 41c C1/C5.
---

# ADR-0190: Own local studies before explicit recording

## Context

Public study admission requires canonical source publication although local model and physical admission already retain immutable scientific authority. Review F09 identifies the resulting store dependency; the operator accepted RC01 and authorized Plan 41c execution.

## Scope

Amend blueprint §0.5, §19.3, §21 and §21.1 for runtime-local study ownership and explicit durable recording. Existing scientific admission, occurrence policy, source identity, durable fences and result assessment remain authoritative. RC02 uses existing artifact promises and grants no stronger transient certification.

## Drivers

Interactive studies must execute using their actual scientific dependencies. Recording and transport need genuine portable receipts; local execution needs owned source lifetimes. Keeping those responsibilities separate avoids storage effects during local admission and avoids repeating binding, lazy expansion or preparation when recording.

## Options

Compulsory publication preserves one portable descriptor but couples local science and tests to a store. Fabricated receipts weaken authority. Select an owned local handle and a recorded descriptor sharing one admitted scientific payload; explicit recording publishes retained inputs through the existing canonical owners. This introduces one public handle and a deliberate API migration, without a second admission policy, solver path or storage manager.

## Outcome

Local study admission retains actual immutable model and physical owners and performs no publication. Explicit recording publishes those exact retained inputs and yields genuine canonical handles. Durable startup revalidates the recorded descriptor at its existing receiver boundary. Lazy recording rebases parent attribution without expanding unselected occurrences.

### Consequences

Rust and Python migrate to an owned local handle and explicit recording/start operations. Document-based physical ingress retains exact bytes for recording; missing recording prerequisites refuse honestly. Recorded document versions change only if their wire meaning changes; no compatibility execution path is retained.

### Compensating controls

No-store explicit/lazy controls exercise the public boundary. Recording controls check exact source identity, missing sinks, duplicate occurrence order, selected-only admission, cancellation and actual owner drain. Durable fences and source protection remain unchanged.

### Confirmation

Tested on 2026-10-15 through the named native local/recording controls and public Python no-store/lazy-source controls in Plan 41c's Outcome. The retained owner can carry genuine canonical-origin attribution without requiring publication; explicit recording supplies the portable descriptor. Plan 41c owns detailed results and exclusions; combined qualification remains with Plan 41f. Proposed status is unchanged pending the decision-PR route.

## Pros and cons

Scientific admission and local tests lose unnecessary storage coupling. Consumers must migrate to an explicit ownership distinction, and exact document bytes remain retained when later recording is supported.

## More information

[Plan 41c](../plans/41c-studies-response-artifacts-and-result-resources.md) owns delivery and verification. [Plan 41](../plans/41-simulation-pipeline-efficiency-and-effectiveness.md#rule-decisions) owns RC01/RC02 disposition. The existing review supplies design evidence; proposed ADR status remains unchanged pending the decision-PR route.

## Status history

- 2026-10-15 — proposed; maintainer accepted RC01 and authorized implementation.
