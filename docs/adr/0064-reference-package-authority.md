---
id: ADR-0064
title: Generate physical fixtures from shipped reference package declarations
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-17, DM-41, DM-44, DM-52, DM-53]
blueprint: [§6.2, §6.15, §9.3, §22.1]
review: docs/design_review/reviews/design_review_wave2-contracts_2026-09-14.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A reference provision needs a new physical convention or a leaf fixture cannot be mechanically projected.
verification: standard_fixture_matches_yaml, reference_package_admission, method_realization_units; just codegen-check; just family-check.
---

# ADR-0064: Generate physical fixtures from shipped reference package declarations

## Context

The Wave 1 quantity fixture contains hand-written physical records. Shipping the
Wave 2 package independently would give the same units and quantity operations two
authorities. Core pure-component methods also need complete formulas, coordinates,
validity and parameter source evidence before they can advertise provisions.

## Scope

Amend blueprint §6.15.7 and §22.1. Use the existing schema/codegen mechanism and
package DocumentSpecs. No new crate, dependency family, IDAES runtime dependency or
solver capability is introduced. The reference data is clean-room authored.

## Drivers

One physical authority, Arrow-free quantity/math leaves, generic stock templates,
reproducible scientific coefficients and independent conformance checks.

## Options

1. Maintain YAML and Rust constants independently: rejected competing authority.
2. Make quantity depend on the Arrow authoring engine: rejected dependency inversion.
3. Generate the leaf fixture projection from the admitted package declarations:
   selected, using the registry-owned generation entrypoint.

## Outcome

Shipped package documents own the physical records and generic engineering
templates. The generator validates their declared shape and emits the Arrow-free
Rust fixture projection; production package loading uses ordinary admission.
Method formulas and coefficient sources are explicit and versioned. Natural
coordinates and reference-state shifts use declared quantity operations. NIST
Shomate, RPP4 polynomial and Perry liquid provisions ship only with exercised
formulas, parameter contracts, bounds and source evidence.

Registered reductions also declare their actual bound domain kind in
`reference.quantity_operation_reductions`. A Species SumOver contraction and a
Phase sum can have the same operand quantity but different physical meaning.
The registry checks the exact axis kind before selecting an operation; the
generated fixture projects this companion with the same admission rules. An
applicable failed contract never falls back to a preserving sum.

### Consequences

Generated fixture output becomes protected and checked by regeneration. Changing a
physical declaration updates both runtime package data and fixture projection.
Stock mechanisms cannot dispatch by template or property display name.

### Compensating controls

The generator fails on duplicate/conflicting identities or unsupported projection
shapes. Tests compare complete admitted records and numerical identities against
independent formula evaluations; source snippets from IDAES are not copied.

### Confirmation

The verification field names the required gates. Proposed scientific coverage is
not a claim of tested numerical solving or backend support.

## Pros and cons

The projection adds a small generation step but removes independently maintained
physical decisions. Package generation is allowed only where the fixture has an
actual leaf-test consumer.

## More information

[Wave 2 plan](../plans/04-wave-2-semantic-compilation.md), ADR-0004, ADR-0058,
[relationship to IDAES](../relationship-to-idaes.md).

## Status history

- 2026-09-14 — proposed before implementation; formal decision PR pending.

### Element balance coefficients

`reference.element_projection_contracts` declares each supported law/source-basis
projection, its complete coefficient quantity and unit, and its multiplication
operation. A molar contribution uses the actual species-element count. A mass
contribution uses that count divided by its actual positive finite species
molecular weight in kg/mol (blueprint §6.4 and §10). Separate physical coefficient
kinds distinguish dimensionless molar counts from amount-per-mass coefficients;
registered multiplication proves the actual source basis and the Element result.
An absent or unsupported basis/contract refuses; failed operation preconditions
never select a fallback. The shipped reference records remain the only authority
for these quantities and operations, and the ordinary fixture generator projects
them through source admission.
