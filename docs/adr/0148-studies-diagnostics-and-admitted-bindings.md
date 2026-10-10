---
id: ADR-0148
title: Preserve typed failures and execute one occurrence-based study contract
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, DP-01, DP-04, DP-19, DP-21, PS-01, PS-10]
blueprint: [§5.3, §19.3, §20.6, §21.5, §23.2]
review: docs/design_review/reviews/design_review_studies-diagnostics-and-admitted-bindings_2026-10-01.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new operation needs a seed or result role the admitted study contract cannot represent.
verification: Plan 25f targeted diagnostic, physical binding, occurrence policy and terminal-record controls; paired adapter decisions from identical facts; integrated durable restart journeys remain Plan 25k.
---

# ADR-0148: Preserve typed failures and execute one occurrence-based study contract

## Context

Blueprint §19.3 records divergent in-process and durable study policies. Detailed failures and physical overlay meaning are lost across those boundaries (F05–F08, F17 and FU09).

## Scope

Amend blueprint §19.3 and §23.2 for typed diagnostic identity and one occurrence-based study policy; version binding identity and generated boundary documents under §5.3 and §21.5. Durable changes use ADR-0146's preserving evolution route under §20.6. This does not complete the wider 25g–25j scope.

## Drivers

An executor substitution must preserve scientific dependency and start decisions. Equal physical bindings may share preparation while repeated experiments remain distinct. A failure must remain interpretable before a scientific result exists. The pure policy is testable without a database or solver.

## Options

Retaining two policies duplicates scientific authority and changes behavior across executors. A new generic workflow engine adds unrelated execution ownership. Select one runtime-owned admitted definition, shared plain semantic facts in pse-model, and a pure transition in pse-operations, applied by the existing executors under their effect and resource contracts.

## Outcome

Typed source codes own failure identity; coarse class, boundary disposition, severity and retry are separate projections. Closed rules and structured observations survive all boundaries. Admit physical assignments against the immutable selected revision once, then frame normalized member/context/canonical-value content under a new binding role. Occurrence identity and submission idempotency remain separate.

Ordering, usable-result and seed dependencies have distinct meanings. Continuation requires a usable compatible predecessor seed by default; seed-only permission and fresh fallback are explicit. Unknown publication effects reconcile before retry. E owns aggregate scientific usability, including multi-result operations; executors never derive success from table counts.

### Consequences

Both executors consume the same policy and operation-owned descriptors. Generated request documents, durable definitions and terminal envelopes receive new versions. The operational schema removes content-based point uniqueness and stores immutable typed policy, outcome history and pre-effect member receipts. Historical identities remain unchanged and missing historical evidence stays unavailable.

### Compensating controls

Focused matrices distinguish scientific usability, seed permission, operational terminal state, cancellation and effect state. Binding controls distinguish affine conversion, physical incompatibility and duplicate targets. Durable decisions are checked under existing locks/fencing; migrations preserve histories through exact source/checksum admission.

### Confirmation

**Implemented/Tested, 2026-10-01:** [Plan 25f Verification](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25f-studies-diagnostics-and-continuation.md#verification) names the focused diagnostic, physical binding, shared policy, terminal-history, native occurrence and generated Python controls. The linked independent review assesses the implemented hashing/Python boundary at that bounded scope. This record remains proposed pending its decision-PR acceptance; integrated durable journeys and assembled qualification execute in 25k.

## Pros and cons

The shared contract removes executor-dependent scientific policy and makes occurrence decisions locally testable. It requires a coordinated public and durable cutover, explicit historical interpretation and broader source-error migration.

## More information

[Plan 25f](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25f-studies-diagnostics-and-continuation.md), [series coordinator](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25-design-remediation.md), ADR-0146 and the domain-alignment reviews.

## Status history

- 2026-10-01 — proposed before the diagnostic, binding and study contract changes.
