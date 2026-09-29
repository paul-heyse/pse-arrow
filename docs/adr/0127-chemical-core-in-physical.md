---
id: ADR-0127
title: Declare the thermodynamic domain once, with the chemical core in pse.physical
status: accepted
date: 2026-09-29
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-01, AP-03, DP-16, PS-01, PS-02, PS-03]
blueprint: [§8.1, §9.1, §9.3, §9.5, §9.7, §6.15.3, §6.15.7]
review: docs/design_review/reviews/design_review_typed-domain-model_2026-09-29.md#f07
evidence: Interface-checked
supersedes: [ADR-0126]
superseded-by: null
revisit: A package above pse.physical needs quantity types whose subject is a kind it declares (register R-51), a ported library needs a domain concept the schema cannot express as a refinement or relation of it, or two libraries require incompatible meanings for one declared kind.
verification: Plan 23 SM0 (package_declared_axis_indexes_a_sum_without_a_registered_shaped_type; the 354 shaped chemistry-axis types deleted; the chemistry module admitted inside pse.physical with its subject-bearing quantity types intact), the D0 refusal corpus (domain_schema_refusals_name_the_violated_constraint, including a missing pair selection), the SM1–SM6 seed migration with unchanged fixture expectations, and scenarios CT-S01, DM1, DM2 (extended to pairs), DM4 and DM5; the AUD review confirms that libraries add rows, forms and bindings only.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01, docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s03]
---

# ADR-0127: Declare the thermodynamic domain once, with the chemical core in pse.physical

## Context

ADR-0126 placed the chemical core (the species, element, reaction and phase kinds) in
`pse.domain`. Its premise was that the physical document names no chemistry kind as a
subject. That premise is false.

Plan 23 SM0 checked it against `packages/reference/physical/materials/physical.yaml`:

- **Quantity subjects.** 177 quantity types name species (146), element (29) or reaction (2)
  as their subject. After SM0 deletes the shaped chemistry-axis types, 34 remain, and they
  back live quantity names: mole fraction, component flow, amount, molar density, mass flow,
  element flow, volumetric reaction rate and reaction heat.
- **Operations.** Six operations name species or element as their result subject.
- **Finite reductions.** Both finite reductions take species as their domain.

The physical inventory is admitted from `pse.physical` alone (blueprint §8.1). Kinds
referenced there cannot be declared in a package above it.

Everything else in ADR-0126 stands, and this record restates it with the placement
corrected.

## Scope

This record governs where the thermodynamic domain schema lives and how libraries relate to
it. It builds on the ADR-0123 mechanisms and adds no Rust or registry concept.

## Drivers

- **Principles:**
  - AP-04 and DP-16: one declaration per concept, and extension by declaration.
  - PS-01: basis, reference and subject in the type.
  - PS-02: typed parameters with provenance.
- **Scenarios:**
  - CT-S01 and DM5: a correlation or a method family added as data.
  - DM1 and DM2: data banks and cross-bank identity.
  - DM4: roles enforced in production.

## Options

| Option | Effect |
|---|---|
| Chemical core in `pse.domain` (ADR-0126) | Not admissible: `pse.physical`'s quantity types and operations would reference kinds declared above it |
| Compose the physical inventory across the package closure, so that `pse.domain` owns chemistry-subject quantity types | A general kernel extension to quantity-registry identity and admission that this readiness scope does not need. Deferred to register R-51 |
| Chemical core in a `chemistry` module of `pse.physical`; the rest of the schema in `pse.domain` | Admissible now. Every chemistry kind has one owner, co-located with the physical types that name it. Selected |

## Outcome

### Where the schema lives

**`pse.physical`** holds the chemical core in its `chemistry` module:

- the species, element, reaction and phase kinds;
- their attribute schemas: opaque identifier schemes, charge, the phase type, and the
  formula relation from which molar mass is derived;
- the canonical phases.

This module is the single phase authority.

**`pse.domain`**, above `pse.physical`, declares:

- provenance kinds;
- property kinds;
- the keyed abstract parameter set;
- pair and group relations with declared symmetry;
- `selection` for pure-component parameter sets;
- `pair_selection` for the source of pair and group data;
- `component_role`;
- constants.

A parameter set's datum is its form's result type.

### How libraries relate to the schema

| Library | What it adds |
|---|---|
| Method library | Refines parameter sets into forms with dimensioned coefficients and bound functions |
| Data bank | Rows only |
| Property package | A `property_package` entity with its `selection`, `pair_selection` and component-role rows, and binds its phase formulation as a definition binding |

A generic property function dispatches through `selection` on the concrete form. Phase and
component behaviour is read from attributes, slots and roles, never from identity
comparisons.

### Consequences

- The seed is migrated (Plan 23 SM0–SM6).
- The chemistry module is admitted inside `pse.physical`, and the dependents' manifests do not change for it.
- The Plan 21 knowledge-placement guide is superseded when Plan 23 closes.

### Compensating controls

- The D0 refusal corpus.
- The whole-seed conformance run after each migration packet.
- The AUD review.
- Register R-51 for composing the physical inventory across packages.

### Confirmation

The SM0 inventory check establishes the placement (Interface-checked). Plan 23 D0, SM1–SM6
and the scenario acceptances establish the behavior. Plan 23 owns implementation status.

## Pros and cons

| For | Against |
|---|---|
| Admissible now, with one owner per chemistry kind next to the physical types that name it | The schema spans two packages, and a new domain with its own subject kinds waits for R-51 |

## More information

- [Plan 23](../plans/23-thermodynamic-domain-and-campaign.md).
- ADR-0098 (knowledge ownership) and ADR-0123.
- The [design review](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md) raised F06–F08 and F15; their resolutions are carried over. Its F07 premise is corrected in review §13.

## Status history

- 2026-09-29: accepted on the maintainer's Plan 23 authorization. It supersedes ADR-0126,
  whose placement premise the SM0 inventory check disproved; the rest of ADR-0126's
  decision is restated unchanged.
