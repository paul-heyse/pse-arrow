---
id: ADR-0055
title: Keep memory reservations with Arrow buffer owners
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-29, DM-30, DM-37, DM-40, DM-44]
blueprint: [§3.1, §14.3, §20.1]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A new external buffer or execution operator cannot retain the required final-owner lease
verification: `just family-check`; `just test-package pse-ids -p pse-relations`; last-owner, query/result/decode budget and cancellation fixtures

---

# ADR-0055: Keep memory reservations with Arrow buffer owners

## Context

A guard on a snapshot or query report releases too early when a RecordBatch/ArrayRef/Buffer clone survives it. The already-resolved bytes 1.12.1 offers a safe owner-preserving route into Arrow Buffer.

## Scope

ADR-0074 and Plan 11 govern the integrated lifetime, evidence-preservation and
implementation-testing amendments. This proposed record retains its historical
evidence; its prior per-boundary or whole-stage mechanisms do not restrict that target.

**Current construction target:** ADR-0067 and blueprint revision 37 replace this
record's former local replay, row-copy and phase-limited execution mechanisms.
Plan 05 owns implementation; the domain/identity/lifetime requirements retained
below are implemented through its single native preparation/completion route.

Amends the cited blueprint sections within the approved Wave 1 boundary. It supplements existing accepted decisions; their arguments remain immutable. Implementation is authorized by the maintainer's approved execution plan; formal ADR acceptance remains the decision-PR lifecycle.

## Drivers

Explicit semantics, enforceable validity, complete dependencies and honest capability claims before reuse or performance work.

## Options

Retain the inconsistent scaffold: rejected because its consumers cannot preserve the declared meaning. Build a general replacement platform: rejected without a demonstrated need. Complete the existing typed contracts: selected.

## Outcome

Promote bytes =1.12.1 to a direct workspace pin with pse-ids as the reservation-buffer helper owner. Wrap allocated buffers with Bytes::from_owner and Arrow Buffer::from; the owner retains the original buffer and shared reservation lease. Recursively cover values, offsets, null bitmaps, dictionaries and children. Clone/slice readers retain the lease without duplicate charges. Use alignment-required IPC decoding; make a reserved aligned copy when needed. Native query results transfer owned streams and final-buffer leases. Use an explicitly reserved copy only when the boundary cannot safely transfer ownership, accounting for coexisting buffers without inferring allocation identity from pointers. Preserve ADR-0046's exclusion of infallible/native allocations from the fallible guarantee. No new unsafe surface or pointer-dedup registry is introduced.

### Consequences

Consumers must use checked contracts and explicit unsupported outcomes. Fixtures and generated output change with their authoritative declarations; this record does not claim runtime acceptance.

### Compensating controls

The scoped design review, negative fixtures and plan 03 terminal gates guard the correction. Existing canonical framing and dependency-family pins remain unchanged except the explicitly named direct promotion.

### Confirmation

`just family-check`; `just test-package pse-ids -p pse-relations`; last-owner, query/result/decode budget and cancellation fixtures. Each result records mode and failure count against baseline zero. The execution ledger distinguishes source inspection, focused tests and complete-wave acceptance.

## Pros and cons

The correction removes an ambiguity or false guarantee with bounded implementation work. Explicit ownership removes routine copies while preserving required external isolation; actual target workloads measure remaining transfer costs.

## More information

[Wave 1 execution plan](../plans/03-wave-1-foundations.md); blueprint §3.1, §14.3, §20.1; charter DM-59 and G1-G7. The authorized blueprint amendment uses PSE_DESIGN_EDIT=1 and must carry a revision row in the decision PR.

## Status history

- 2026-09-14 — proposed before implementation; maintainer approved the correction plan and its validation-before-hashing clarification.

- 2026-09-14 — reconciled with ADR-0067 and Plan 05; prior receipts describe their original code and do not certify the hard-pivot implementation.
