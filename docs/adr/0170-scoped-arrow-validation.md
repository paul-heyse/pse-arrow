---
id: ADR-0170
title: Validate actual Arrow closures without adding unrelated test roots
status: proposed
date: 2026-10-09
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-06, AP-07, DP-09, DP-22]
blueprint: [§24.1]
review: docs/design_review/reviews/design_review_scoped-arrow-validation-target_2026-10-09.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A new Arrow-consuming test closure cannot activate validation without selecting an unrelated package target.
verification: Pinned Cargo rejects relation feature activation for pse-quantity and pse-modeling; EFF02 targeted build-root, actual Arrow feature and feature-alternation controls.
standard: {core: '3.4', process-simulator: '1.5'}
scenarios: [docs/design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#s01]
---

# ADR-0170: Validate actual Arrow closures without adding unrelated test roots

## Context

The universal relation feature mandate currently adds pse-relations test/dev targets even
for semantic or native-adapter units with no Arrow consumer. Pinned Cargo
`check --locked -p pse-quantity --lib --features pse-relations/force-validate` and the same
command for pse-modeling refuse because those packages do not contain that dependency feature.
The maintainer confirmed the RC01 fallback to actual Arrow-consuming closures in Plan 33.

## Scope

Amend blueprint §24.1's invocation rule, agent instructions, aliases and relevant recipes.
Validation remains opt-in outside correctness checks. Preserve the pinned feature-unification
configuration, single Arrow universe and native/terminal/resource ownership contracts.

## Drivers

Independent semantic and native-adapter tests must compile their actual selected dependency
closures. Actual Arrow-consuming checks/tests, including selected dev-dependencies, must retain
full validation. Test names, unit labels and hand-maintained per-test classifiers cannot
establish that closure. A new Arrow consumer must make its validation activation explicit.

## Options

Keeping the extra relation package builds unrelated test targets. Cargo's feature flag alone
cannot serve packages lacking that dependency. Adding Arrow/relations to pure crates merely
to activate validation worsens the dependency boundary. Select opt-in validation activation
on actual boundary owners, propagated through their existing dependencies, and compose flags
for selected responsibilities without introducing another package root. Pure closures need no
validation flag. Existing resolved dependencies/manifest features own this distinction; do not
maintain a second list of test identities. Unknown or uncovered Arrow closures must refuse.

## Outcome

Correctness invocations explicitly enable force-validation for every selected actual
Arrow-consuming closure, including its test/dev-dependencies, without selecting unrelated
relation targets. Boundary owners expose opt-in `force-validate` propagation through existing
dependencies. Scope-aware command composition reads the declared/resolved dependency closure
and selects those existing flags. It must not silently omit validation when a new consumer
appears. Generated workspace feature-unifier edges do not establish semantic consumption;
they must never make validation a default or switch it on in production.

Pure closures compile/test without adding Arrow or relation roots. Additional explicit package
selections widen the required closure. Workspace commands may retain the workspace relation
validation flag because those targets are already selected. Bare-tool routes document and
enforce the same obligation; aliases cannot claim validation they do not actually activate.
Preserve user features, exact Nextest selection and normal sharing across alternating routes.

### Consequences

Migrate selectors, check-package, affected selection and bare aliases/guidance together.
Add propagation only to real boundary owners; no native or Arrow dependency enters pure
modeling/math solely for testing. Regenerate affected feature-unifier outputs through codegen,
ensuring opt-in validation is excluded from its traversal. Remove the unconditional extra
relation roots and tests whose expected arguments encoded that discarded mechanism.

### Compensating controls

Exercise pure, direct Arrow, transitive Arrow and dev-only Arrow closures, additional packages,
user-feature preservation, unknown-consumer refusal and exact selected inventory. Inspect actual
pinned Cargo targets/features and run the native-local status test without canonical setup.
Alternate semantic, composite and native selections without disabling feature unification.

### Confirmation

The two pinned Cargo rejections establish the narrow-mechanism limitation, not performance
or product acceptance. The scoped target review accepts this design at Proposed strength.
EFF02 implementation controls remain required.
Plan 33 owns all packet status and qualification; old internal reports are not current evidence.

## Pros and cons

Selected work determines build roots while real Arrow tests remain validated. Closure-aware
composition and opt-in boundary features add wiring that must be checked when a new consumer
is introduced. They remove repeated unrelated relation/dev compilation from pure routes.

## More information

- [Plan 33 EFF02](../plans/33-efficiency-principles-remediation.md#eff02).
- [Review F04](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f04).

## Status history

- 2026-10-09: Proposed under the confirmed conditional RC01 direction; scoped target review Accept at Proposed strength; affected implementation remains in progress.
