---
id: ADR-0160
title: Centralize testing responsibility and scoped qualification
status: proposed
date: 2026-10-05
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, DP-09, DP-19, DP-23, PS-10, PS-13]
blueprint: [§3.2, §6.14, §14.2, §19.2, §21, §24.1, §24.3]
review: docs/design_review/reviews/design_review_testing-responsibility_2026-10-05.md
evidence: Proposed
supersedes: [ADR-0051]
superseded-by: null
revisit: Selective trajectory export under smaller budgets becomes required, or representative measurements show consequential unused retention.
verification: The bounded testing-responsibility design review assesses responsibility and change locality; Plan 26 Q1 exercises completed-result agreement, independent integrity mechanisms, owned effects and scoped terminal evidence.
standard: Core 3.3; Process Simulator 1.4
---

# ADR-0160: Centralize testing responsibility and scoped qualification

## Context

The testing-architecture review found duplicated completion projections, integrity declarations and execution evidence beside production owners. The design phase permits replacing those mechanisms directly. Blueprint §19.2, §21 and §24.1 must describe the resulting responsibility boundaries rather than preserve historical tests.

## Scope

This decision amends those architecture contracts and blueprint §24.3. It removes the empty structural test crate, historical enum-name compatibility obligation and duplicate generated-tree governance test. It preserves production enforcement, scientific independent evidence, explicit correctness force-validation and the scope of historical qualification receipts.

## Drivers

A completed trajectory must have one immutable permission/header authority across Rust, Python and publication. A generated integrity implementation needs independent mechanism oracles and real enforcement controls, without a second catalog declaration. Tests must own their effects; qualification must describe the selected workload and inputs actually consumed.

## Options

Retaining mutable result projections and additional test-time registries leaves competing truth and repeated calculation. Per-relation lazy export would add another dispatcher and partially initialized state. The selected target seals the existing completion product and reuses whole-collection encoding, typed native producer origin, runner selection and the complete xtask regeneration comparison. The first export consequently retains the whole map; no performance improvement is claimed without measurement.

## Outcome

Completed trajectories privately retain one clone-shared snapshot, composed permission and kind-correct final header. Their synchronized transport publishes only a fully successful checked map; direct export may retry after a transport failure, while the outer RunResult keeps its existing sticky encoding-error contract. Exporters perform no scientific recompletion.

Integrity declarations carry read-only native producer origin that does not enter the durable schema or fingerprint. Independent family witnesses test different failure risks; admitted production registries execute intact. Generated-tree freshness has one complete comparison owner, xtask, and is not rerun by ordinary product tests.

Tests own isolated files, databases and mutable runtime state. The Python deployment resource service remains process owned with immutable settings and one shared budget, including escaped-buffer charges; individual test journeys own runtime facades and mutable effects. Runner categories follow actual effects. Comprehensive qualification runs the common native graph once and retains separate feature-absence controls. One terminal composer owns each receipt; current receipts identify conservative gate-specific inputs, actual selected cases and binary/native provenance. Unknown scope captures all inputs. Retired formats receive no adapters. Measurement prerequisites are named recipe identities, not a second test selector.

### Consequences

All consumers move to the replacement directly and replaced APIs, test helpers, fixtures and registrations are deleted. Shared transport requires accounted metadata and existing escaped buffer leases. Receipt scope cannot substitute for native provenance, and partial or cancelled execution cannot become successful evidence.

### Compensating controls

Independent scientific and integrity witnesses, hostile admission controls, completion/refusal/lifetime tests and receipt terminal reconciliation retain the risks that static types cannot establish. Explicit force-validation remains in correctness recipes. Scoped evidence never qualifies an unexecuted workload.

### Confirmation

The bounded review establishes an architectural judgment at Proposed design level. Plan 26 owns implementation progress and named checks; decision acceptance is separate from product qualification. No speed or memory claim follows from this decision.

## Pros and cons

One owned meaning reduces repeated work and setup. Whole-map first export can refuse under a budget that would fit one relation; reopen selective export only at the observable trigger above.

## More information

[Plan 26](../plans/26-testing-architecture.md) owns delivery and finding dispositions. The [testing architecture review](../design_review/reviews/design_review_testing-architecture_2026-10-05.md) records the diagnosis; the bounded review records the target judgment. ADR-0092 remains the production builder/validation contract. This record replaces ADR-0051's duplicate test responsibility, preserving complete regeneration equivalence.

## Status history

- 2026-10-05 — supersedes ADR-0051 to retain complete xtask equivalence while removing its duplicate product-test responsibility.
- 2026-10-05 — proposed for the authorized Plan 26 implementation; maintainer decision acceptance remains separate.
