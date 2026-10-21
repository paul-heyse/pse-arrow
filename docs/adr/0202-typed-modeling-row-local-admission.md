---
id: ADR-0202
title: Admit generated modeling rows without transient Arrow materialization
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-07, DP-10]
blueprint: [§14.3.2, §14.4]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A modeling declaration adds an unsupported local facet or a native implementation binding cannot establish equivalent typed checks.
verification: Plan48b B05 compares typed admission with independently specified malformed finite/range/collection/selected-arm cases and actual changed-contract/function/configuration/invocation controls, while preserving raw native admission.
---

# ADR-0202: Admit generated modeling rows without transient Arrow materialization

## Context

The authoring loader already owns generated typed modeling rows. It materializes a complete Arrow batch only to establish local field admission and immediately discards that batch. Accepted Plan48 RC11 permits a bounded generated typed-loader route after the remaining validation obligations have been established.

## Scope

Amend blueprint §14.3.2 and §14.4 for the generated modeling-declaration family only. Reuse `LocalFieldAdmission` and its actual native predicate owner; no public trusted-row trait or independent admission hierarchy is introduced. Plan48 I06 owns the final eligibility decision and qualification.

## Drivers

Keep the schema declaration as the only field authority. Remove transient column materialization when typed values and generated checks establish precisely the required local facts. Changed engines, function meanings, configurations, invocation scopes or raw inputs must retain their actual receiving obligations.

## Options

The improved native validator remains the general route. A second handwritten document decoder would duplicate existing generated records and miss native facets. Generate only the remaining checks from current declarations; retain native residual checks or refuse typed eligibility when equivalent semantics cannot be established. Preparation of a current planner alone is not a proof that different implementations of a predicate have the same meaning.

## Outcome

The target typed helper requires exact generated declaration compatibility and actual current native-validator preparation. It checks all remaining local finite, integer-range, collection and tagged-arm obligations, deriving traversal from authoritative declarations. Typed representations establish only the shape and vocabulary they actually enforce. Unsupported future facets/row SQL cannot be silently omitted. If native function/configuration meaning affects a check, use its actual semantic owner or retain that native check; otherwise the specialized route is ineligible.

A successful qualified helper may mint the existing local-field admission associated with its actual rows and predicate owner. Private mutable row clones revoke that admission. Keys, references, complete relational admission, raw Arrow structure/metadata, source resource ownership and current invocation-sensitive obligations remain with existing owners. The explicit Arrow export/materialization route remains a real consumer capability.

### Consequences

Generated local checks become responsible for new modeling field facets. The generator must refuse an unsupported specialization rather than produce an incomplete validator. A field-family expansion is local to its declaration/check projection, not a new loader framework.

### Compensating controls

Negative cases include a malformed selected arm, nonfinite number, out-of-range integer, unsupported facet, altered native function/configuration and changed current scope. Unknown eligibility uses required native admission; it never imports historical permission.

### Confirmation

**Tested:** the default-profile, explicitly force-validated native-local selections in the
[B05 package Outcome](../plans/48b-registry-and-receiving-validation.md#outcome-recorded-after-implementation)
passed the generator guards, `typed_modeling_eligibility_uses_actual_rewrites_and_prebound_functions`
and all four typed-modeling loader controls. These cover the actual bound session factory,
malformed nested/selected arms, finite/range/cardinality checks, signed-zero preservation,
private-clone revocation and explicit Arrow export. An actual function rewrite changes the
native predicate result and denies typed eligibility; an unqualified planner retains native
evaluation. The qualified current declaration avoids the discarded transient batch.

This is focused evidence for the bounded modeling family and its actual predicate owner.
It does not establish equivalence for unsupported future facets, general Arrow inputs or
arbitrary planner implementations, nor a total-performance or assembled Python qualification.
The record remains proposed; implementation evidence does not change formal acceptance.

## Pros and cons

This removes an internal representation crossing without changing authored rows. Its eligibility is narrower than generic native validation and must remain explicit.

## More information

- [Plan48b B05](../plans/48b-registry-and-receiving-validation.md#b05--conditionally-specialize-authored-document-receiving)
- [Plan48b Outcome](../plans/48b-registry-and-receiving-validation.md#outcome-recorded-after-implementation)
- [Architecture local field admission](../authoritative_design/sections/mathematics-and-compilation.md#section-14-3-2)

## Status history

- 2026-10-20 — Proposed before the accepted RC11 conditional typed-admission implementation; formal acceptance remains separate.
