---
id: ADR-0092
title: Ordinary execution evidence and qualification
status: proposed
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [DP-16, DP-22, DP-23]
blueprint: [§24.1, §24.3]
review: docs/design_review/reviews/design_review_plan16-foundations_2026-09-25.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new execution or storage path cannot preserve these distinctions.
verification: P17 tooling tests and P18 complete qualification
---

# ADR-0092: Ordinary execution evidence and qualification

## Context

Plan 16 consolidates the comprehensive and data-model follow-up reviews. Its target
requires explicit contracts before replacing the current mechanisms. This decision
amends the active blueprint scope without rewriting accepted historical ADR bodies.

## Scope

The contract below and its existing owners; implementation sequencing is in
[the execution packet](../plans/16-p14-p18-execution.md). Acceptance through the
repository decision PR route remains pending; implementation is user-authorized.

## Drivers

Preserve authored meaning, make invalid selected requests observable, and use pinned
library mechanisms without a second semantic authority.

## Options

Keep the existing implicit conventions: rejected because callers can disagree.
Introduce a new universal registry/compiler: rejected because existing owners suffice.
Extend existing typed owners and derive consumer projections: selected.

## Outcome

P17 retains ordinary tests, benchmarks and factual execution records while removing compulsory digest-bound design verdicts and plan-specific phase ceremony. A source digest identifies inputs; it does not establish applicability, equivalence or review quality. Records distinguish executed, unchanged-input reuse, reviewed transfer and not-run evidence.

Cargo manifests own Rust pins; family checks validate resolved families against those declarations. Blueprint tables describe architectural policy, without a second exact-pin parser or tooling exemption inventory. Type-aware method/type bans belong to Clippy; scoped syntax/literal bans belong to ast-grep. Product behaviors replace source-spelling architecture witnesses.

P00–P04 use targeted compilation and functional tests. Integrated testing and nonfunctional checks are deferred until the requested functional scope is complete; Plan 16 whole-system qualification remains P18. Regeneration is part of implementation whenever registry declarations change. Existing Plan 14 M22 and ADR-0087 observations remain historical and their exclusions are not inherited.

### Consequences

Existing callers and fixtures move with their replacement. No compatibility execution
path survives merely to preserve old assumptions.

### Compensating controls

Targeted negative and equivalence tests enforce the touched contracts. Unsupported
selected behavior is refused explicitly; downstream packets retain their own gates.

### Confirmation

P17 tooling tests and P18 complete qualification. Whole-system acceptance is not inferred from these targeted checks.

## Pros and cons

One authority reduces semantic drift; explicit contracts require coordinated changes
across consumers and versioned identities.

## More information

[Plan 16](../plans/16-data-model-architecture.md),
[target review](../design_review/reviews/design_review_plan16-foundations_2026-09-25.md),
and blueprint sections named in the front matter.

## Status history

- 2026-09-25 — proposed before affected implementation; decision PR acceptance pending.
