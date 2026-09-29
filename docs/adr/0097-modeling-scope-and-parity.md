---
id: ADR-0097
title: Target full modeling scope against IDAES 2.13
status: proposed
date: 2026-09-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, DP-13, PS-01]
blueprint: [§3.1, §24.1]
review: git:0e725de269f18dd08331158a07b38a7d92ea0b5e:docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A scientific port needs model-specific Rust or a synthetic kernel contract fails.
verification: Parity version preflight and authored compatibility-name checks; K8 shared conformance compares the selected scientific seed with explicitly attributed oracle inputs and tolerances. K9 campaign parity remains separate.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01]
---

# ADR-0097: Target full modeling scope against IDAES 2.13

## Context

The reference must cover the current IDAES modeling library while production remains a clean-room Rust implementation. This records Plan 21 decisions A1–A2.

## Scope

The Plan 21 target and its authorized K0–K8 implementation boundary. This amends the
target contract; executed seed evidence belongs to the K8 packet. The wider IDAES
knowledge ports and K9 campaign are not claimed as implemented or qualified.

## Drivers

Data-only extension, complete physical meaning, explicit dependencies and locally testable
semantic operations. Plan 21 owns execution and finding dispositions.

## Options

Retaining 2.12 would leave reference coverage stale; embedding IDAES would violate the production boundary. A pinned test-only oracle preserves reproducibility.

## Outcome

Use idaes-pse 2.13.0 as the parity oracle. Full modeling capability is the target; the kernel is implemented before data-only scientific ports.

### Consequences

A reference version is not a claim that every model has been replicated.

### Compensating controls

Typed refusal, stable semantic identities, bounded expansion and targeted positive/negative
controls accompany each mechanism. Remove an old path only after its replacement and callers move.

### Confirmation

Parity version preflight and authored compatibility-name checks exercise the reference
boundary. K8 conformance compares selected scientific fixtures with explicit oracle
parameters, references and tolerances; K9 campaign parity remains separate. Target review
supports architectural reasoning only; executed evidence belongs to the owning packet,
not a retrospective change to this decision's evidence.

## Pros and cons

The generic contract localizes future science changes to packages. Its initial language,
registry and physical-type migration costs are larger than adding another special case.

## More information

[Plan 21](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/plans/21-modeling-kernel.md) and its companion documents own implementation
sequencing. [Target review](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md).

## Status history

- 2026-09-26 — proposed; implementation authorized by the maintainer. Acceptance remains on the decision-PR route.
