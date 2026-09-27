---
id: ADR-0099
title: Author generic models through registry-defined language contracts
status: proposed
date: 2026-09-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, DP-13, PS-01]
blueprint: [§5.3, §6.15.1, §11.3]
review: docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A scientific port needs model-specific Rust or a synthetic kernel contract fails.
verification: Parse/render/parse, generated contract validation, explicit-ID rename and generic kind controls.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/21-modeling-kernel.md]
---

# ADR-0099: Author generic models through registry-defined language contracts

## Context

The existing expression language and generated semantic contracts can support generic package authoring without a second model authority. This records Plan 21 decisions A5; R3.

## Scope

The Plan 21 K0–K8 target and its implemented generic language/identity boundary.
Execution evidence belongs to the owning packet; decision acceptance is still pending.

## Drivers

Data-only extension, complete physical meaning, explicit dependencies and locally testable
semantic operations. Plan 21 owns execution and finding dispositions.

## Options

Handwritten JSON AST contracts duplicate registry authority. Science-specific YAML extensions propagate knowledge into code. A source AST may preserve syntax but is not another semantic schema.

## Outcome

Use a versioned modeling language whose durable IR is declared in pse-schema and generated into pse-model. Explicit IDs survive rename; instance IDs derive from parent, declaration and member identities. Source spans and lineage are distinct from execution keys.

### Consequences

Through K8, Rust and Python admit explicit package dependency closures. Physical aliases
are package-owned references; adapters do not choose them. Fixture execution, policies,
expected outcomes and physical/relative tolerances are typed registry declarations.
Incompatible durable contracts reject explicitly rather than entering a compatibility path.

Registry and caller migrations follow consumer replacement. Quantity axes and type subjects
refer to generic declared kinds; an optional reference-datum subject instead names an actual
authored entity, not its kind. Physical admission checks that distinction. The physical identity domain advances to
`pse.math.physical-inventory.v2`, framing kind IDs, axis IDs and optional subject IDs. Display names
and backend-local slots do not enter that identity. Finite function and dispatch identities
frame explicit typed structure and membership; runtime initial values remain separate.

Finite reductions preserve their consumed entity kind after scalar enumeration, including
the physical prototype for empty and filtered sets. A nonempty lexical fold composes an
authored type-preserving step in admitted membership order. It lowers to shared local
bindings, so the compiler contains no smoothing formula or model-specific aggregator.

Structural projections of selected definitions use the same constructor/default/effective-
member binding as instantiation. Constructed child instances may refine an inherited
interface through declared extension or implementation, retaining its members and indexed
contracts. Input parameters, physical quantities and function signatures remain invariant.
Conservation equations retain signed original terms as separate sparse-assembly contributions;
this does not add another arithmetic or differentiation implementation.

### Compensating controls

Typed refusal, stable semantic identities, bounded expansion and targeted positive/negative
controls accompany each mechanism. Remove an old path only after its replacement and callers move.

### Confirmation

Parse/render/parse, generated contract validation, explicit-ID rename and generic kind controls. Target review supports architectural reasoning only; executed evidence belongs
to the owning packet, not a retrospective change to this decision's evidence.

## Pros and cons

The generic contract localizes future science changes to packages. Its initial language,
registry and physical-type migration costs are larger than adding another special case.

## More information

[Plan 21](../plans/21-modeling-kernel.md) and its companion documents own implementation
sequencing. [Target review](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md).

## Status history

- 2026-09-26 — proposed; implementation authorized by the maintainer. Acceptance remains on the decision-PR route.
