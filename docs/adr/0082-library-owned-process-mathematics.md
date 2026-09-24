---
id: ADR-0082
title: Compile typed process definitions through library-owned mathematics
status: proposed
date: 2026-09-24
deciders: [paul-heyse]
level: decision
principles: [DM-07, DM-24, DM-38, DM-56]
blueprint: [§D6, §D10, §D11, §7, §14]
review: docs/design_review/reviews/design_review_incremental-math-structure_2026-09-24.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A provider or guarded composition cannot preserve the selected value or derivative contract.
verification: Plan 14 M00–M21 targeted units and M22 qualification
---

# ADR-0082: Compile typed process definitions through library-owned mathematics

## Context

The maintainer approved Plan 14 and its hard M00–M05 cutover. The two math reviews
identify competing mathematical ownership and incomplete library composition contracts.

## Scope

The approved M21 extension removes caller-authored numerical policy hashes and
obsolete identity/grammar contracts. The compiler owns guarded-real semantics;
physical balance declarations lower through the same typed Symbolica/Salsa path.
Existing data, graph, incremental and retention mechanisms require current consumers.


The concrete replacement text is prepared in
[Plan 14 foundation contract](../plans/14-math-foundation-contract.md#concrete-decision-amendment-prepared-for-the-design-pr).
Apply it to the cited blueprint contracts through the designated design PR.
This proposed record describes the implemented mathematical foundation and M19 cleanup.
Native solver/workflow implementation is described by ADR-0083–0084; whole-plan
scientific, runtime and formal acceptance remain M22 work.
Accepted decisions are reconciled through formal supersession before decision PR acceptance.

## Drivers

Library-owned algorithms, physical meaning, explicit failure and bounded repeated compilation.

## Options

Retaining the old engine would preserve conflicting authority and is rejected.
The selected native library composition is qualified by narrow executable controls.

## Outcome

Symbolica owns algebra and evaluation after physical typing and pre-normalization obligations. Introduce pse-math; delete pse-mathir and pse-numerics with their callers. Real-algebra equivalence replaces ordered floating-point identity. Source and quantity meanings remain authoritative.

The M06–M08 implementation uses Numerica vectorized jets inside arithmetic regions
and Symbolica-generated provider lifts. Semantic preparation, effect-owned compilation
and mutable workers are separate. Output-specific obligations survive cancellation;
compilation options belong to artifact identity, not semantic body identity. Identical
local demands share immutable programs within a case preparation.

### Consequences

The approved M19 cleanup removes the unused pse-plans and pse-kernels-ext crate
identities and compiled mesh/stencil/quadrature contracts. Finite authored domains,
the native compiler/initialization owners and pse-kernels' actual FeOS provider remain.
The unused expression-payload sharing pass is deleted. Generated semantic values and
native codecs retain their existing owners, with typed AST transformations and compact
physical fixtures; no second codec or allocator framework is introduced. See
[M19–M20 execution](../plans/14-m19-m20-execution.md). The removals were recorded before
implementation and are now implemented; formal decision acceptance and blueprint
reconciliation remain pending.

The approved M09–M10 extension separates immutable case planning from evaluator
construction. Salsa owns semantic dependency tracking over actual admitted physical
inventories and provider descriptors. Compiler-issued artifact requests include exact
source/build/feature identity and evaluator options. The runtime owns bounded native
construction, retention and mutable workers; neither native state nor cache side effects
belong in tracked queries. The old preparation-local BodyStore is deleted.
Structural matching and DM/BTF use pounce-presolve over complete all-branch support,
with bounded stack admission and explicit inequality/objective coupling.
See [M09–M10 execution](../plans/14-m09-m10-execution.md).

Native solving replacements are implemented through the later Plan 14 packages.
No old code is retained as a fallback or as historical evidence. Compilation and
targeted contracts do not establish full native process qualification.

### Compensating controls

Pinned profiles, pre-normalization physical/domain checks, explicit capability admission,
source-attributed failures and targeted positive/negative unit controls.

### Confirmation

The package-specific units and deleted caller inventory establish implementation;
Plan 14 M22 alone establishes full qualification.

## Pros and cons

The change removes duplicate engines and exposes library capabilities. It requires
an atomic caller/schema cut and explicit native/profile qualification.

## More information

The [M21 implementation review](../design_review/reviews/design_review_m21-design-closure_2026-09-24.md)
assesses the final contract/deletion changes at mechanism scope. It does not supply
independent M22 acceptance.

[Execution packet](../plans/14-m00-m05-execution.md) and
[main plan](../plans/14-library-owned-process-simulator.md).

## Status history

- 2026-09-24: Proposed before the approved implementation; scoped decision review and
  blueprint amendment are part of this work, not an implied whole-plan acceptance.

- 2026-09-24: M00–M05 implementation and targeted controls complete; scoped review
  accepts the foundation. Formal ADR acceptance and blueprint supersession remain pending.

- 2026-09-24: M06–M08 and foundation enhancements implemented; 45 combined targeted
  units pass with explicit force-validation and zero failures. The enhancement review
  accepts this scope. Formal decision acceptance and full M22 qualification remain open.

- 2026-09-24: M09–M10 implemented and scoped-reviewed; 63 targeted force-validation
  units pass with zero failures. Shared workspace compilation passes. Formal decision
  acceptance, blueprint reconciliation and full M22 qualification remain open.
