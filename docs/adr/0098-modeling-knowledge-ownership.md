---
id: ADR-0098
title: Own generic modeling semantics in one pure kernel
status: proposed
date: 2026-09-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, DP-13, PS-01]
blueprint: [§3.2, §9.6, §14.3]
review: git:0e725de269f18dd08331158a07b38a7d92ea0b5e:docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A scientific port needs model-specific Rust or a synthetic kernel contract fails.
verification: Synthetic specialization tests without native or storage startup; per-declaration incremental controls in the compiler.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01]
---

# ADR-0098: Own generic modeling semantics in one pure kernel

## Context

Science-specific runtime construction prevents data-only extension and isolated semantic testing. This records Plan 21 decisions A3–A4; R1–R2.

## Scope

The Plan 21 target and its authorized K0–K8 implementation boundary, including retirement
of the scientific registry and `pse-material`. This amends the target contract; qualification
and finding dispositions belong to the execution packets and Plan 21.

## Drivers

Data-only extension, complete physical meaning, explicit dependencies and locally testable
semantic operations. Plan 21 owns execution and finding dispositions.

## Options

Separate property and thermodynamic crates reproduce science-specific contracts. A second Salsa database duplicates ownership. Pure functions inside one compiler workspace preserve local tests and existing lifecycle.

## Outcome

Add only pse-modeling. It owns pure checking and specialization over generated pse-model values. pse-compiler owns the single Salsa database; pse-math owns Symbolica; runtime owns admission and effects. Scientific identities and correlations are package data.

### Consequences

Existing scientific providers remain until their knowledge and consumers are actually replaced.
Through K8, retiring the scientific registry also retires `pse-material`: element masses,
composition, phase restrictions and reaction stoichiometry belong to the authored chemistry
packages. The complete shipped engineering-element subset and compatibility names move
before their source families and generated fixtures are removed. Generic quantity and unit
admission stays in `pse-quantity` and the runtime physical inventory. The generator no longer
emits a second Rust declaration of elemental knowledge. This removes an existing crate; it
does not add another semantic owner.
The authorized through-K8 correction makes checked products immutable outside their owner,
binds them to their physical context, and admits immutable package revisions once. The single
compiler workspace tracks their dependencies; preparation does not recheck unchanged inputs.
Package visibility follows declared imports rather than the incidental loaded inventory.
See the [through-K8 review](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/design_review/reviews/design_review_modeling-kernel-through-k8_2026-09-26.md).

### Compensating controls

Typed refusal, stable semantic identities, bounded expansion and targeted positive/negative
controls accompany each mechanism. Remove an old path only after its replacement and callers move.

### Confirmation

Synthetic specialization tests without native or storage startup; per-declaration incremental controls in the compiler. Target review supports architectural reasoning only; executed evidence belongs
to the owning packet, not a retrospective change to this decision's evidence.

## Pros and cons

The generic contract localizes future science changes to packages. Its initial language,
registry and physical-type migration costs are larger than adding another special case.

## More information

[Plan 21](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/plans/21-modeling-kernel.md) and its companion documents own implementation
sequencing. [Target review](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md).

## Status history

- 2026-09-26 — proposed; implementation authorized by the maintainer. Acceptance remains on the decision-PR route.
