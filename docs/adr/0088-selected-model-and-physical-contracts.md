---
id: ADR-0088
title: Selected model admission and physical provider contracts
status: accepted
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [DP-01, DP-03, DP-06, DP-08, DP-15, PS-01, PS-02, PS-03]
blueprint: [§0.5, §6.4, §6.6, §9.8, §14.3]
review: git:8950dd3d6ddb:docs/design_review/reviews/design_review_plan16-foundations_2026-09-25.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new execution or storage path cannot preserve these distinctions.
verification: Selected admission, connection, provider permutation, reference and reaction unit tests
---

# ADR-0088: Selected model admission and physical provider contracts

## Context

Plan 16 consolidates the comprehensive and data-model follow-up reviews. Its target
requires explicit contracts before replacing the current mechanisms. This decision
amends the active blueprint scope without rewriting accepted historical ADR bodies.

## Scope

The contract below and its existing owners; implementation sequencing is in
[the execution packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-p00-p04-execution.md). Implementation was user-authorized;
the maintainer accepted the decision after Plan 16 closed.

## Drivers

Preserve authored meaning, make invalid selected requests observable, and use pinned
library mechanisms without a second semantic authority.

## Options

Keep the existing implicit conventions: rejected because callers can disagree.
Introduce a new universal registry/compiler: rejected because existing owners suffice.
Extend existing typed owners and derive consumer projections: selected.

## Outcome

The workflow owns one selected-model admission boundary shared by typed builders and document loading. Every selected declaration is consumed, explicitly retained as nonexecuting data, or refused with a source identity. Schema support closure is distinct from selected row dependency closure. Compiler Inputs are derived from the admitted immutable revision; flow and initialization preparation publish and prepare under one workspace lock.

Reusable templates retain parameters, finite domains, guards, equations, ports and contributions until selected specialization. Unresolved guards, continuous execution, unsupported scaling and tear policies are refused. Extensive fanout requires declared splitting; a transfer has one identity and opposite signs.

Provider registration binds actual species to PC-SAFT and DIPPR records, ordered coordinates, complete quantity kinds, basis, separate enthalpy/entropy references, data provenance and validity. FeOS owns thermodynamics; num-dual owns derivatives. Homogeneous explicit-density execution is supported; phase-equilibrium execution is explicitly refused. Mechanical and global phase-stability observations remain separate. Selected reactions require declared phases/species, element closure and either formation enthalpies or explicit heat without double counting. Numerical policy remains a separately resolved input (P05), never an adapter default.

The existing pse-math crate receives a narrowly scoped FFI allowance for GMP's
version string and MPFR's get_version API only. It reads process-lifetime immutable
C strings during explicit initialization; no arithmetic FFI or mutable pointer escapes.
The workspace allowlist records this scope; no new crate is required. The Miri deferred
trigger remains governed by its register entry and final qualification.

### Consequences

P05 resolves one numerical policy before execution: analysis, case priority, model,
bound property defaults, quantity nominals and recorded canonical-unit fallback.
Normalization is distinct from native algorithmic scaling; hard guards, integer
lattices and cone membership survive transport. Frozen absolute/relative budgets
govern original-space acceptance. Presolve certificates must survive the corresponding
per-bound tolerance expansion. Conservation and integrated-output requirements remain
distinct; failed closure retains an unusable candidate unless explicit qualified use
is requested. P06 derives mathematical capability from admitted operations, shares
bound-dependent affine/obligation facts and separates exact Gram evidence from opt-in
numerical PSD qualification. These additions are Implemented; the
[P05–P06 packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-p05-p06-execution.md) records scoped test evidence.

Existing callers and fixtures move with their replacement. No compatibility execution
path survives merely to preserve old assumptions.

### Compensating controls

Targeted negative and equivalence tests enforce the touched contracts. Unsupported
selected behavior is refused explicitly; downstream packets retain their own gates.

### Confirmation

Selected admission, connection, provider permutation, reference and reaction unit tests
are recorded in the execution packet. P03/P04 implement the homogeneous physical
boundary; whole-system acceptance is not inferred from these targeted checks.

## Pros and cons

One authority reduces semantic drift; explicit contracts require coordinated changes
across consumers and versioned identities.

The [P05–P06 execution packet](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-p05-p06-execution.md) and
[implementation review](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/design_review/reviews/design_review_plan16-numerics-facts_2026-09-25.md)
record the numerical/fact extensions and their scoped qualification.

## More information

[Plan 16](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/16-data-model-architecture.md),
[target review](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/design_review/reviews/design_review_plan16-foundations_2026-09-25.md),
and blueprint sections named in the front matter.

## Status history

- 2026-09-25 — proposed before affected implementation; decision PR acceptance pending.
- 2026-09-26 — accepted by the maintainer as implemented through Plan 16 (review verdict Accept;
  local Linux qualification and independent review recorded in blueprint §24.2).
