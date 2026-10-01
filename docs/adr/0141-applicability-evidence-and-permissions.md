---
id: ADR-0141
title: Separate applicability evidence from scoped data-use permission
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-06, DP-01, DP-03, PS-01, PS-02, PS-03]
blueprint: [§9.3, §9.10, §14.3, §19.3]
review: docs/design_review/reviews/design_review_plan25b-contracts_2026-09-30.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A scientific extension requires a second coefficient authority, a sentinel key or implicit permission for missing evidence.
verification: Plan 25b focused composition, reaction projection, parameter selection and applicability controls with explicit force-validation; integrated qualification in Plan 25k.
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs01, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs02, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs03]
---

# ADR-0141: Separate applicability evidence from scoped data-use permission

## Context

Existing interval guards do not distinguish absent empirical evidence from unrestricted use, and exported boolean checks conflate observations with permission. F12 and R2 require a separate evidence contract.

## Scope

Amend applicability admission/evaluation and its generated observations within existing modeling and numerical ownership. This record changes semantic identity and generated boundary conventions and requires a scoped design review. It does not implement Plan 25e result qualification or Plan 25g persisted-history migration.

## Drivers

Evidence must remain honest through composition and numerical simplification. Explicit permission is a consuming binding's decision, never a mutation or upgrade of a source claim.

## Options

Mandatory fabricated unbounded envelopes would mislabel unknown evidence. Blanket analysis permission silently covers unrelated later selections. Selected: explicit claims, one generic evaluation/use gate and permissions naming records or declared families.

## Outcome

Claims name owner, scope, evidence and either Region(predicate, basis), Unrestricted or Unknown(reason). Region basis preserves fitted/recommended/validated distinctions. Form, selected-record/collection and consuming-model claims compose while retaining responsible layers and record/call/input attribution.

The generic evaluation emits Applicable, OutsideRegion or UnknownEvidence for each demanded claim. Mathematical and hard model-domain requirements remain mandatory attributable failures under every permission. Missing claims cannot become unrestricted. B4 applies the low-level use gate; E3 later assesses result qualification from the unchanged observations.

Required dependencies retain every observation: outside and unknown obligations require independent permissions. Declared alternatives form unions: a known applicable alternative satisfies the union; otherwise unknown alternatives make it unknown, and wholly known failures make it outside. Unrelated fits are never implicitly unioned. Interval increments require explicit whole-interval guards; endpoint coverage is sufficient only for declared interval regions.

A binding's independent allow-unknown and allow-extrapolation permissions default false, name records or declared families and retain authorizing declaration/lineage. Scope inheritance may resolve an existing named permission, but cannot widen its target. Claims, selection closures, conventions and permissions enter admission/preparation identity with changed frame versions. Observations survive active branches, derived arguments, cancellation and differentiation; inactive branches are not evaluated.

Migrate every represented parameter/form family, including empirical costing, without inventing source ranges. Seed consumers select individually justified claims or named permissions. Universal constants may declare genuinely unrestricted claims without invented regression intervals.

### Consequences

Registry declarations generate the new transport, and modeling/compiler/math/runtime consumers move together. Existing interval machinery remains an implementation of declared regions; it cannot create unknown/unrestricted semantics or decide final result qualification.

### Compensating controls

Plan 25b tests unknown/unrestricted distinctions, the two permissions independently, scope inheritance, union/dependency composition, interval coverage, branch demand, attribution, identity changes and mathematical-domain refusals under every policy.

### Confirmation

Proposed contract reviewed at the scoped decision boundary. Plan 25b Outcome owns focused executed evidence; complete product/scientific qualification remains Plan 25k.

## Pros and cons

A generic claim/observation boundary preserves meaning for every scientific family; the migration crosses authoring, preparation, evaluation and generated results once.

## More information

[Plan 25b](../plans/25b-scientific-knowledge-and-applicability.md), F12/R2, blueprint §9.10 and §14.3.

## Status history

- 2026-09-30 — proposed before maintainer-authorized Plan 25b implementation.
