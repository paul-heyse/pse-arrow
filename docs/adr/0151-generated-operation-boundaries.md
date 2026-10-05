---
id: ADR-0151
title: Generate public boundaries from Rust-owned operations
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-06, DP-01, DP-04, DP-19, PS-01, PS-10]
blueprint: [§21.1, §21.5, §23.2]
review: docs/design_review/reviews/design_review_admission-reuse-and-generated-boundaries_2026-10-01.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: An exported operation has contextual admission or a result distinction the generated boundary cannot preserve.
verification: Plan 25j isolated document/enum/default/refusal codecs, flow selection admission and generator equality/order controls; complete cross-language journeys remain 25k.
---

# ADR-0151: Generate public boundaries from Rust-owned operations

## Context

Plan 25j closes handwritten Python mirrors, adapter-local request policy and string enum getters left around the settled 25a-to-25g operations. Blueprint §21 owns a thin typed Python boundary; a generated shape alone does not establish scientific admission.

## Scope

Rust-owned exported operation documents, their native codec, generated Python consumers and closed vocabularies. Covers publication/workspace/export, settlement, attempts/diagnostics, flow selection and remaining analysis/strategy/inspection boundaries. Source-authoring schemas retain their distinct omission/hydration contract.

## Drivers

Success, partial execution, cancellation and pre-result refusal must preserve operation identity and typed causes. Defaults and contextual flow/physical admission must have one Rust owner. Adding an enum member or field should regenerate consumers instead of requiring another handwritten mirror.

## Options

Maintaining adapter-local JSON and Python records duplicates meaning. A separate schema merely generating those mirrors retains the duplicate authority. Select actual owned Rust serde/schemars documents and registry declarations through the existing emitters, with the existing msgspec document and attrs/cattrs row conventions.

## Outcome

An operation decodes transport once and, when context matters, admits it against the selected immutable revision. Python performs shape/codec conversion; it does not independently choose defaults, quantities or routes. Register actual request/result/error types, generate models/enums/native stubs, and migrate builders, readers, exceptions and getters as one operation slice. Human explanations remain separate from closed reason values. No old/new public API pair survives the slice.

Use strum for shared closed-vocabulary mechanics while semantic projections remain with their owners. Use petgraph for deterministic declaration dependency ordering with attributable cycles, attrs for cardinality, and type-directed equality-consistent uniqueness keys for the admitted element domain. Collection signed zeros compare equal and reordered/nested maps preserve equality; identity hashes from ADR-0150 are inappropriate for uniqueness.

Owned document schemas retain declared array bounds and uniqueness in generated msgspec models. Strategy ID-set decoding refuses duplicate occurrences before collecting into sets. The document generator also emits isolated transport fixtures from the actual operation-owned Rust products under `python/pse/tests/fixtures/generated-native-boundaries/`; these exercise generated Python decoding without claiming solver or storage qualification.

### Consequences

Handwritten wire mirrors and ad hoc JSON construction disappear; genuine Python handle wrappers remain. Defaults derive mechanically from Rust or are selected by native admission. Persisted changes take ADR-0146's explicit evolution route and do not reinterpret older artifacts.

### Compensating controls

Codec controls cover malformed/unknown fields, refusal before results, cancellation and partial products. Test omitted versus explicit Rust defaults, duplicate/context-invalid flow selections, enum getters, deterministic dependency ordering and scalar/structured collection equality.

### Confirmation

**Implemented/Tested, 2026-10-02:** [25j Verification](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25j-generated-boundaries-and-library-consolidation.md#verification)
records the focused Rust owner/generator/flow controls and 49 selected linked Python unit
controls, including typed diagnostics/getters/stubs, defaults, eight actual Rust-serialized
outcome branches, duplicate refusal and scalar/structured collection equality. Final generation,
workspace/all-target compilation and the explicit force-validate/native-solvers editable rebuild
pass after the recorded integration repairs. These establish the scoped boundary behavior;
scientific/storage workflow qualification and measurements remain 25k. The bounded linked
review retains its original Proposed target evidence and its separately scoped implemented
cache-ownership follow-up. This record remains proposed pending its decision PR.

## Pros and cons

One operation owner removes cross-language semantic drift. The native/Python cutover must migrate every consumer and distinguish source schemas from hydrated documents.

## More information

[25j](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25j-generated-boundaries-and-library-consolidation.md), [series coordinator](../plans/25-design-remediation.md), ADR-0150 and ADR-0148.

## Status history

- 2026-10-01 — proposed before generated operation boundary consolidation.
