---
id: ADR-0086
title: Retire orphaned rule execution and check structural governance contracts
status: proposed
date: 2026-09-24
deciders: [paul-heyse]
level: decision
principles: [DP-01, DP-13, DP-16, DP-24]
blueprint: [§3.1, §14.2, §23.2]
review: docs/design_review/reviews/design_review_test-contract-repair_2026-09-24.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A production consumer requires declarative recursive inference.
verification: Rust workspace nextest with force-validation; governance error taxonomy and unsafe allowlist tests.
---

# ADR-0086: Retire orphaned rule execution and check structural governance contracts

## Context

Plan 14 replaces the compiler that consumed custom inference. Production registry
assembly declares no rules; remaining execution consumers are tests and benchmarks.
The live invariant checker still serves inspection and physical fixture generation.

## Scope

Retire rule execution, its exclusive declarations, projections, fixtures and tests.
Keep the invariant checker. Amend blueprint §14.2 to describe this boundary.
Governance checks in blueprint §23.2 must recognize generic diagnostic implementations
and reasoned module-local unsafe allowances. Blueprint §3.1 must agree with the
Plan 14 FeOS composition's num-dual 0.14.2 pin.

## Drivers

Keep one supported execution architecture and test real contracts. Preserve typed
errors and the narrowest unsafe scope. Do not change dependency identity to satisfy
an obsolete documentation literal.

## Options

Restore the executor: rejected because it has no production consumer.
Retire the entire rules crate: rejected because invariants have live consumers.
Require blanket unsafe allowances: rejected because narrower allowances are stronger.

## Outcome

Remove custom rule execution as a supported capability. Parse governance-relevant
Rust declarations structurally, retaining compiler enforcement of trait and lint contracts.

### Consequences

Custom rule APIs and their tests disappear. Invariant execution remains supported.
The current num-dual pin stays unchanged.

### Compensating controls

Retain current invariant, authored foreign-key, publication and numerical contracts.
Reject missing diagnostic implementations and unreasoned unsafe allowances.

### Confirmation

`cargo nextest run --no-fail-fast --workspace --locked --features
pse-relations/force-validate` passed all 1695 tests with zero failures and zero skips.
The linked native dynamics/fitting selection passed all nine selected tests with
`pse-runtime/native-solvers,pse-relations/force-validate`. The implementation record
lists exact commands, conditions and remaining static qualification. Full Plan 14
qualification remains M22; this evidence does not change the ADR's proposed status.

## Pros and cons

Removes unsupported machinery and misleading tests; downstream custom-rule users
would require a separately reviewed new capability.

## More information

See Plan 14's foundation contract and the linked design review. This proposed ADR
requires an `adr` / `needs-review` PR and the associated blueprint revision before merge.

## Status history

- 2026-09-24 — proposed following executable failure review and explicit retirement decision.
