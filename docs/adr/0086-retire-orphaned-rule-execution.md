---
id: ADR-0086
title: Retire orphaned rule execution and check structural governance contracts
status: accepted
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
qualification is subsequently recorded in the M22 packet under ADR-0087.

## Pros and cons

Removes unsupported machinery and misleading tests; downstream custom-rule users
would require a separately reviewed new capability.

## More information

See Plan 14's foundation contract and the linked design review. ADR-0087 authorizes
local M22 acceptance and blueprint reconciliation. Ordinary remote merge requirements
remain unchanged; no PR or merge result is claimed.

## M22 local qualification

**Tested and Measured:** the [M22 packet](../plans/14-m22-execution.md#verification)
records local Linux functional Q01–Q17, the 23 cached-development case-cost workloads,
zero required failures and retained-origin conditions. It distinguishes admitted memory
allowances from measured pool/RSS observations and excludes Rust build time.

The [independent final review](../design_review/reviews/design_review_m22-local-qualification_2026-09-24.md)
accepts the relevant scoped contracts with no open MUST finding. Companion runtime,
scientific and claims reviews cover G1–G8 and PS-G1–PS-G3. Blueprint revision 51 and
ADR-0087 govern local acceptance. Strict Clippy cleanup and release/remote/platform
qualification remain separate; no broader clean or empirical claim follows.

## Status history

- 2026-09-24 — proposed following executable failure review and explicit retirement decision.

- 2026-09-24 — accepted for local Linux M22 scope under ADR-0087 after independent final review; blueprint revision 51 reconciles the contracts. No remote or release qualification is claimed.
