---
id: ADR-0156
title: Separate library-owned iterations from declared numerical composition
status: proposed
date: 2026-10-03
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, DP-13, DP-17, PS-09]
blueprint: [§1.3, §3.2, §18.6, §24.4]
review: docs/design_review/reviews/design_review_declared-solve-pipeline-decisions_2026-10-03.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A fitting library becomes available for project-owned numerical composition.
verification: Bounded proposal review under Core 3.3 / ProcessSimulator 1.3; adopted clarification and versioned binding; P6–P10 native controls and Plan 25k K5 assembled review.
standard: core-3.3 / process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08, docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s12]
---

# ADR-0156: Separate library-owned iterations from declared numerical composition

## Context

PS-09 forbids reimplementation of established numerical iteration but its current wording does not distinguish task composition, prepared homotopies and continuation transport from native iteration. The accepted target needs that distinction to govern ownership honestly.

## Scope

Governance decision: clarify the process-simulator profile and binding through their owners, with a versioned standard change before relying on the clarification. This is no MUST waiver and does not admit custom Newton, globalization or factorization where a qualified library fits.

## Drivers

Preserve original scientific permission, explicit applicability, bounded lifetime and change locality while integrating the complete Plan 25m target. Mechanism substitution and pure policy testing must stay within their responsible owners.

## Options

An absolute prohibition on composition prevents the selected target; an unqualified exception permits solver reimplementation. Select explicit responsibility boundaries and source-backed library fit, with target changes reviewed and implementation correctness independently checked.

## Outcome

Libraries own nonlinear iteration, line-search/trust-region acceptance, factorization, Anderson acceleration, native restoration and surrogate model management where admitted. The simulator owns scientific problem/family definitions, consumed contracts, start and branch permissions, bounded orchestration and original assessment. A domain-level path/shift/block composition may connect admitted library operations when no fitting library supplies that entire domain contract; record its inputs, correspondence, derivative actions, accepted/rejected state and caps. Do not duplicate a library controller merely to expose metrics. PETSc owns admitted pseudo-time growth and stage rejection; bounded shifted-root composition remains a different scoped construction. Scientific time stepping, statistical brackets and durable retries do not become shared numerical strategy. Current PS-09 class/capability requirements remain.

### Consequences

Migrate producers and consumers together and remove replaced paths after targeted acceptance. The existing Plan 25m packet/finding table owns implementation progress; no parallel ledger is added.

### Compensating controls

Typed admission/refusal, finite work, actual native contract tests and independent original assessment constrain integration. No license restriction or hypothetical benchmark ranking excludes a selected library.

### Confirmation

**Proposed:** scoped decision review precedes changed-contract implementation. Prior source reviews establish the library/design basis, not acceptance or product qualification. Named checks are planned, not executed claims.

## Pros and cons

The target makes consequential distinctions explicit and permits isolated policy testing, at the cost of coordinated producer/consumer migration and foreign lifetime contracts.

## More information

[Implementation and disposition owner](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25m-integrated-solve-pipeline.md); [source review](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md).

## Status history

- 2026-10-03 — proposed for maintainer-authorized Plan 25m implementation; decision-PR acceptance and publication remain separate.
