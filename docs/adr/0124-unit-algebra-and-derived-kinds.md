---
id: ADR-0124
title: Resolve units and coefficient types by algebra over declared kinds
status: accepted
date: 2026-09-29
deciders: [paul-heyse]
level: decision
principles: [PS-01, DP-02, AP-04, DP-01, DP-04]
blueprint: [§8.1, §8.2, §8.3, §9.3]
review: docs/design_review/reviews/design_review_typed-domain-model_2026-09-29.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A ported form needs a coefficient whose monomial cannot be declared as a quantity kind, or chain resolution types an expression that the registered stepwise rules type differently.
verification: Plan 23 KR1 tests unit_product_is_order_independent (identity equality), composite_literal_needs_no_registered_whole_unit, affine_unit_only_as_sole_factor, defined_unit_dimension_and_scale_are_derived, rational_unit_exponents_canonicalize and report_in_a_composite_unit_carries_its_identity; KR2 tests seed_forms_type_without_intermediate_kinds (dippr100, Shomate, RPP4 cp, dh and ds), eligible_leaves_are_exactly_true_zero_ratio_points_and_differences, nonzero_datum_leaf_is_refused_with_its_factor, basis_must_agree_or_be_declared, undeclared_monomial_is_refused_with_factors, rule_and_chain_disagreement_is_refused and type_expression_resolves_by_monomial; scenario DM3.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01]
---

# ADR-0124: Resolve units and coefficient types by algebra over declared kinds

## Context

**Unit literals are looked up as strings.** The system resolves a unit literal by exact
lookup of its joined symbol string (blueprint §8.2). So `{J/(K*mol)}` and `{J/(mol*K)}` are
different units, and every composite unit must be registered whole.

**Coefficients carry no dimension.** Correlation coefficients are authored as `Scalar`. A
separate `scale` column holds their scaling, and conventions such as `T/(1000 K)` are written
as digits in function bodies (§9.3). The quantity system cannot check a coefficient's
dimension, and the coefficient's meaning lives in prose.

## Scope

This record amends two contracts:
- the unit contract (§8.2);
- quantity inference (§8.3).

The existing registered operation rules, basis semantics and reference-state semantics keep
their meaning. Chain resolution is an additional route: it applies only where every factor is
eligible.

## Drivers

- **PS-01:** basis, reference state and convention are part of the type.
- **DP-02:** no meaning is parsed from text.
- **DP-04:** identity is never derived from spelling.
- **Scenario CT-S01:** a correlation with typed coefficients.
- **Scenario DM3:** wrong coefficient units are refused.

## Options

| Option | Effect |
|---|---|
| Keep symbol lookup and `Scalar` coefficients | Coefficient dimensions stay unchecked; composite units multiply. Rejected |
| Synthesize quantity types for any monomial on demand | Types appear without an owner. Canonical units, nominal magnitudes and revision identity would then depend on usage. Rejected |
| Canonical unit products; coefficient types resolved against declared derived kinds with a defined result policy | One authority, the physical document. A new coefficient type costs one declared kind row. Selected |

## Outcome

### Units

- **Literal form.** A unit literal is a canonical product of atomic units with rational exponents.
- **Product identity.** A unit product's identity is a catalog-frame derivation over its canonical factors, independent of spelling.
- **Defined units.** A defined unit states its composition; its dimension and scale are derived from it.
- **Affine units.** An affine or datum-restricted unit may appear only as the sole factor, with exponent one.

### Derived kinds

- **Declaration.** A quantity kind may be declared as a monomial of other kinds, together with its canonical unit.
- **Resolution.** A multiplicative chain resolves by its canonical monomial against the declared kinds. Language type expressions such as `MolarCp/Temperature^2` resolve the same way.
- **Leaf eligibility.** A factor may take part in the chain in any of these cases:
  - it is a difference;
  - it is dimensionless;
  - it is a point of a ratio-scale kind whose canonical unit has a true zero (for example, absolute temperature).

  A point with a nonzero datum (gauge pressure, or enthalpy measured from a datum) makes the chain refused. The refusal names that factor, and the chain is never partly resolved.
- **Basis.** Basis must be equal across all basis-carrying factors, or be declared on the derived kind.
- **Result type.** The result's scale, reference, subject and shape come from the derived kind's declaration. Increments such as `c3*T^3/3` therefore resolve to the declared `DeltaH` kind.
- **Unknown monomials.** An undeclared monomial is refused, and the refusal names its factors.
- **No synthesis.** Kinds are never synthesized.
- **Agreement with registered rules.** When a registered stepwise rule and chain resolution both type an expression, the two results must agree. If they disagree, the expression is refused, and the refusal names both results. Neither route takes precedence.

### Consequences

- The physical document gains derived-kind and composite-unit definitions, each with its declared canonical unit.
- Seed coefficients move to dimensioned attributes, and the digits in function bodies are deleted (Plan 23 SM3 and SM4).

### Compensating controls

- Tests fix the exact set of eligible leaves.
- Refusals name the ineligible factor or the missing monomial.
- The registered stepwise rules are unchanged, and a test covers expressions they already type.

### Confirmation

The KR1 and KR2 targeted tests over the actual seed forms, and scenario DM3, establish the
behavior. Plan 23 owns implementation status.

## Pros and cons

| For | Against |
|---|---|
| Coefficient units are checked at admission; composite units need no registration | Each new coefficient dimension needs a declared kind row |

## More information

- [Plan 23](../plans/23-thermodynamic-domain-and-campaign.md).
- The [design review](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md) raised findings F02 and F14; their resolutions are incorporated in the text above.
- ADR-0123.

## Status history

- 2026-09-29: proposed; amended the same day with the design-review resolutions.
- 2026-09-29: accepted on the maintainer's Plan 23 authorization; design-review verdict Accept-scoped, with its condition (N1, agreement with registered rules) incorporated.
