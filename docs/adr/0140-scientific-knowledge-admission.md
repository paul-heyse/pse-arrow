---
id: ADR-0140
title: Admit complete chemistry and parameterization selections
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-06, DP-01, DP-03, PS-01, PS-02, PS-03]
blueprint: [§6.15.2, §9.1, §9.3, §9.7, §9.9, §14.3]
review: docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu01
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A scientific extension requires a second coefficient authority, a sentinel key or implicit permission for missing evidence.
verification: Plan 25b focused composition, reaction projection, parameter selection and applicability controls with explicit force-validation; integrated qualification in Plan 25k.
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs01, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs02, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs03]
---

# ADR-0140: Admit complete chemistry and parameterization selections

## Context

Unknown composition currently supplies conserved zeros, kinetic bindings can replace checked reaction coefficients, and pair selectors identify whole sources rather than scientific records. F11 and FU01–FU03 require one admitted scientific authority.

## Scope

Amend blueprint §9.1/§9.3/§9.7/§9.9 and preparation identity within existing authored-domain ownership. ADR-0127 remains accepted and immutable; its chemical-core placement and generic declaration ownership remain. This amendment replaces its package-wide pair-source and mandatory-phase parameter contracts without changing that placement.

## Drivers

Conservation requires complete participating evidence; kinetic and heat laws must share the reaction's extent convention. Independent scientific fits must compose without coefficient copies or provenance used as fit identity.

## Options

Dense completeness or molar-mass proxies lose sparse/unknown distinctions. Checking callback equality retains a second coefficient authority. Per-pair source selectors still conflate variants. Selected: authored claims, concrete checked reaction/material projections and individually identified parameter records, using existing generic predicates, keys and specialization.

## Outcome

Complete/unknown composition is declared explicitly; complete-empty is a complete sparse vector with no nonzero cells. Charge evidence is independent and atomic weights govern mass derivation only. Conservation obtains complete participant vectors before checking elements/charge. Elemental control-volume projections use the same admitted composition authority.

Concrete reaction/material admission derives coefficients from the authoritative reaction, verifies every nonzero participant, permits inert extras and retains the explicit extent normalization. Selected kinetic and heat records carry that convention; independent bare callbacks cannot assert it or redefine the source matrix. Explicit conserved transformations may change species bases.

Parameterization, scientific family, subjects, variant and provenance are distinct. A keyless common parameter carrier has separate complete keyed pure/phase-specific and ordered/symmetric pair shapes; no optional or tuple key extension is needed. Family means the declared parameter contract, not automatically the numerical form or publication. Existing symmetric relation canonicalization remains authoritative. Ordered key pairs preserve both orientations and independently declare whether self-pairs are excluded; diagonal exclusion does not imply transposition.

Selection references records or an explicitly selected predictive rule per required pair. Stored zero, missing pair and predictive zero are different products. Dependencies close over the selected model/subjects; conflicting variants refuse. Atomic-fit membership is separate from derivation lineage and allows only declared subsystem projections. No unrelated publication-wide closure is required. Reaction parameter records join this identity convention.

A bounded generic `selection_closure(roots, context, dependencies)` follows authored dependency
callables once per reached identity. Admission retains roots, consuming context, reached record
identities and edges as immutable products; specialization retains consumed products. Typed
structural tuple/set construction and slot projection permit generic conflict checking without
scientific-name dispatch. Dataset bindings may supply declared nonderived attributes as well as
keys, preserving attached evidence without turning it into an identity key. Bindings cannot replace
derived or kind-bound attributes. Reference-set cells carry plain and keyed references in one typed member list, and named set references resolve against their declared element type. Atomic-fit backlinks derive from the sole authored group membership. A finite structural `require_present` operation obtains a typed optional reference or refuses absence; no missing scientific record becomes a numeric default. Changed finite-function and dispatch-body frames retain selection
products in preparation identity.


### Consequences

Banks, import bindings, selectors, methods and seed consumers migrate together. Changed key/preparation contracts receive new frames/relation revisions; no conversion of persisted history or compatibility path is introduced. B4 owns applicability references; Plan 25g owns later persisted evolution.

### Compensating controls

Plan 25b controls sparse/empty/unknown evidence, charge versus elements, missing products/inert support, both reactor source vectors, normalization mismatches, ternary mixed-source selections, variants/orientation, joint-fit closure and predictive provenance.

### Confirmation

**Implemented, 2026-10-01. Tested:** `just unit-package pse-modeling 'test(scientific_composition_tests)' --test-threads 1`
and `just unit-package pse-modeling 'test(scientific_selection_tests)' --test-threads 1`
passed 5 and 12 controls respectively against zero, with explicit force-validation in the
optimized test profile. Reaction, potential and consumed-identity compiler controls have
composite successful receipts in the [Plan 25b Outcome](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25b-scientific-knowledge-and-applicability.md#verification).
Full product qualification belongs to Plan 25k. This record remains proposed pending its decision PR.

## Pros and cons

Meaning stays authored and locally testable, while the one-time migration changes several related scientific consumers and keys together.

## More information

[Plan 25b](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25b-scientific-knowledge-and-applicability.md), [Plan 25 dispositions](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25-design-remediation.md#finding-dispositions), ADR-0127 and blueprint §9.

## Status history

- 2026-09-30 — proposed before maintainer-authorized Plan 25b implementation.
