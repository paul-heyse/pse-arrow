---
id: ADR-0008
title: Adopt D5: every quantity carries a quantity_type_id, and pse.* extension types are registered
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-06, DM-07, DM-08, DM-44]
blueprint: [§D5, §4.4, §8]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0039
revisit: A quantity-bearing column appears without a `quantity_type_id`, or the engine's extension-type registry stops validating storage types at planning time
verification: `python / test` `test_extension_round_trip`; the wrong-storage-type planning rejection test in `tests/conformance`

---

# ADR-0008: Adopt D5: every quantity carries a quantity_type_id, and pse.* extension types are registered

## Context

Blueprint D5 requires every quantity-bearing column and every symbol to carry a `quantity_type_id` resolving dimension vector, quantity kind, basis, reference state, affine semantics and index shape. The second review's R2-3 found §4.3's claim that "nothing in DataFusion acts on these keys" false: the engine's `ExtensionTypeRegistry` resolves `ARROW:extension:name` and rejects a wrong storage type during planning.

## Scope

Binds the physical-typing contract and the registration of the ten `pse.*` extension types on both sides of the Python boundary. Unit inference itself is §8.3.

## Drivers

A unit string cannot express basis, reference state or affine semantics; heterogeneous value columns need a per-row type; an unvalidated extension type is a typed hole.

## Options

Unit strings on columns — rejected: `enth_mol_phase` in two bases is the same string. Compile-time unit typing with `uom` — rejected by ADR-0026.

## Outcome

Every quantity-bearing column and symbol carries a `quantity_type_id`; heterogeneous value columns carry a per-row one. The ten `pse.*` extension types are registered in the engine's extension-type registry, which becomes a declared validity mechanism rather than inert metadata.

### Consequences

Null never means "unknown to be solved" (§7.6); an unregistered extension name is an engine error on the Rust side while Python degrades to the storage type, so the bundle carries a per-column loss profile (§21.1).

### Compensating controls

The registration is one generated function, not new machinery; a batch with the wrong storage type fails at planning, not at read time.

### Confirmation

`test_extension_round_trip` on the Python side and the planning-rejection test on the Rust side run in `python / test` and `rust / test`.

## Pros and cons

Carrying a type id on every quantity column is verbose; the alternative is the class of bug that the basis/reference-state distinction exists to prevent.

## More information

Blueprint §D5, §4.3 (metadata conventions), §4.4 (extension types), §8 (physical typing); review finding R2-3.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0039.
