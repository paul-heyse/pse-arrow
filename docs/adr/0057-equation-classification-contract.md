---
id: ADR-0057
title: Make equation classification and role contracts self-contained
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-06, DM-07, DM-15, DM-43, DM-59]
blueprint: [§6.9, §7.5, §14.1]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: P14 classification is implemented or a consumer requires a family not declared here
verification: `just test-package pse-schema -p pse-relations`; family and role dictionary admission and generated-contract tests
---

# ADR-0057: Make equation classification and role contracts self-contained

## Context

The blueprint cites an unavailable semantic_math_basis document. Blueprint §7.5 already names families and six roles, but does not declare a complete dictionary or a truthful pre-classification state. Its product detection example can misclassify nonlinear factors as bilinear. The maintainer authorized authoring the missing contract during Wave 1.

## Scope

Complete the schema contract and correct the classification conditions in blueprint §7.5. Wave 1 implements dictionary/admission support and preserves authored hints; it does not implement P14 classification or claim solver eligibility.

## Drivers

One available authority, conservative validity, explicit incomplete analysis, and no classification by digest, name or matching syntax alone.

## Options

Invent the missing external source: rejected. Accept any text as a classification: rejected. Preserve the existing named categories, define them locally and add honest fallback states: selected.

## Outcome

The EquationFamily dictionary is BOUND, AFFINE_EQUALITY, AFFINE_INEQUALITY, NETWORK_BALANCE, SIMPLEX_ALLOCATION, BILINEAR, SMOOTH_TRANSCENDENTAL, NONSMOOTH_CONVEX, BLACK_BOX, DEFINITION, REPORTING_DEFINITION, GENERAL_NONLINEAR and UNCLASSIFIED. EquationRole is HARD_FEASIBILITY, DEFINITION, LINKING, DOMAIN_GUARD, REPORTING and APPROXIMATION. The meanings and enforcement boundaries are in blueprint §7.5. These uppercase spellings preserve the existing blueprint vocabulary.

Family is a case-bound classification. UNCLASSIFIED is the explicit absence of established classification, including before P14. GENERAL_NONLINEAR means nonlinearity was established but no supported more-specific class was established; it is not an alias for unknown. A family hint is an authored expectation, never proof. Roles originate with the producing equation declaration/derivation; P14 checks and preserves them rather than inferring engineering purpose from expression shape.

### Consequences

Consumers must admit UNCLASSIFIED and refuse specialized assumptions when classification evidence is absent. P14 must bind case substitutions, active variable sets, relevant domain/quantity facts and validator contracts to its derivation. Family or role equality alone does not validate reuse.

### Compensating controls

Conservative predicates and explicit precedence in §7.5 prevent overstated classes. Future classifiers require counterexamples such as x*x versus disjoint-group bilinear factors, nonlinear factors, overlapping classes and changed fixed/free assignments.

### Confirmation

Dictionary admission and generated schema checks are Wave 1 evidence. P14 semantic classification tests are deferred to its implementation and cannot be replaced by schema tests.

## Pros and cons

The design becomes self-contained without introducing a new analysis framework. Two additional states prevent incorrect positive claims; consumers must handle them explicitly.

## More information

Blueprint §7.5; ADR-0056; plan 03. PSE_DESIGN_EDIT=1 is used for the authorized amendment and the decision PR must explain it.

## Status history

- 2026-09-14 — proposed after the maintainer authorized completing the missing design reference.
