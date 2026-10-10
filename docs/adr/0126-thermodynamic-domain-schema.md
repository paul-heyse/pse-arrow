---
id: ADR-0126
title: Declare the thermodynamic domain once for every library
status: superseded
date: 2026-09-29
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-01, AP-03, DP-16, PS-02, PS-03]
blueprint: [§9.1, §9.3, §9.5, §9.7, §6.15.3, §6.15.7]
review: not-required: package knowledge on the ADR-0123 mechanisms; examined by the Plan 23 knowledge-boundary audit
evidence: Proposed
supersedes: []
superseded-by: ADR-0127
revisit: A ported library needs a domain concept the schema lacks and cannot express as a refinement or relation of it, or two libraries require incompatible meanings for one declared kind.
verification: Plan 23 SM0 control package_declared_axis_indexes_a_sum_without_a_registered_shaped_type, the D0 refusal corpus (domain_schema_refusals_name_the_violated_constraint, including a missing pair selection), the SM1–SM6 seed migration with unchanged fixture expectations, and scenarios CT-S01, DM1, DM2 (extended to pairs), DM4 and DM5; the AUD review confirms that libraries add rows, forms and bindings only.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01, docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s03]
---

# ADR-0126: Declare the thermodynamic domain once for every library

## Context

**The seed hides meaning in names and identity branches.**
- Method selection is encoded in fit names and binding columns.
- The same phases are declared twice.
- Code decides behaviour by identity: `if p==equilibrium.liquid`, `if j==chem.water`.
- Coefficients carry units by convention.

Porting data banks and method families on this basis would give each library its own
implicit conventions (blueprint §9.1, §9.3).

## Scope

This record decides where the thermodynamic domain schema lives and how libraries relate
to it.

It relies on the ADR-0123 mechanisms. It adds no Rust or registry concept.

## Drivers

- AP-04 and DP-16: one declaration per concept, and extension by declaration.
- PS-02: typed parameters with provenance.
- Scenarios:
  - CT-S01 and DM5: a correlation or method family added as data;
  - DM1 and DM2: data banks and cross-bank identity;
  - DM4: roles enforced in production.

## Options

| Option | Effect |
|---|---|
| Each library declares its own kinds and conventions | Parallel meanings; conformance cannot be checked. Rejected |
| Registry families for the domain | Rejected with ADR-0123 |
| One schema in packages, which every library refines or populates | Selected |

## Outcome

**Chemical core in `pse.domain`.** The physical document names no chemistry kind as a
subject. The species, element, phase and phase-species kinds appear only as shape axes,
on 354 quantity types that no package names. The modeling path indexes package-declared
kinds without registered shaped types. Plan 23 SM0 confirms this with a control and
deletes all of these shaped types.
The chemistry kinds then move out of `pse.physical`, identities unchanged, into
`pse.domain`, where they gain their attribute schemas:
- opaque identifier schemes;
- charge;
- the formula relation, from which molar mass is derived;
- the phase type.

If a registry-shaped type over a chemistry axis proves necessary, that is a kernel-gap row,
not a reason to fix chemistry in the physical package.

**`pse.domain`** sits above `pse.physical` and is the single phase authority. It
declares:
- provenance kinds;
- property kinds;
- the keyed abstract parameter set;
- pair and group relations with declared symmetry;
- `selection` for pure-component parameter sets, and `pair_selection` for the source of pair
  and group data;
- `component_role`;
- constants.

A parameter set's datum is its form's result type, so it carries no separate reference
attribute.

**What libraries do.**

| Library | Allowed changes |
|---|---|
| Method library | Refine parameter sets into forms with dimensioned coefficients and bound functions |
| Data bank | Add rows only |
| Property package | Add a `property_package` entity with its `selection`, `pair_selection` and component-role rows, and bind its phase formulation (equation of state or gᴱ family) as a definition binding |

A generic property function dispatches through `selection` on the concrete form. Phase
and component behaviour is read from attributes, slots and roles, never from identity
comparisons.

### Consequences

- The seed is migrated (Plan 23 SM0–SM6).
- The Plan 21 knowledge-placement guide is superseded when Plan 23 closes.

### Compensating controls

- The D0 refusal corpus.
- The whole-seed conformance run after each migration packet.
- The AUD review.

### Confirmation

The Plan 23 D0, SM1–SM6 and scenario acceptances establish the behavior. Plan 23 owns
implementation status.

## Pros and cons

| For | Against |
|---|---|
| Libraries become rows, forms and bindings against one schema | The seed rewrite and the removal of unreferenced physical shaped types come first |

## More information

[Plan 23](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md); ADR-0098 (knowledge ownership);
ADR-0123. The [design review](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md)
findings F06–F08 and F15 are incorporated.

## Status history

- 2026-09-29: proposed; amended the same day with the design-review resolutions.
- 2026-09-29: accepted on the maintainer's Plan 23 authorization; design-review verdict Accept-scoped, with its condition (N2: every chemistry-axis shaped type is deleted) incorporated.
- 2026-09-29 — superseded by ADR-0127.
