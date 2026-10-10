---
id: ADR-0115
title: Declare entity identities in the registry, keep one Rust type per vocabulary, and catalog hash frames
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, DP-01, DP-02, DP-04, DP-24, PS-02]
blueprint: [§4.1, §4.2, §5.1, §5.3, §6.8, §21.5]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md#decision
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An entity needs two identity types at once (for example a run that is also a study point) and the declaration cannot express that; or a generated typed id forces a conversion inside a hot numerical kernel; or a frame spelling must change for an existing identity, which would need a new frame version rather than an edit.
verification: Architecture scenario S22, settled by Plan 22 B3, B4 and B7. B3 tests: compile_fail doctests for an attempt/run swap and a workspace/publication swap, frame_spellings_unique, and the pse-ids golden vectors unchanged. B4 tests: extrapolation_policy_typed, source_owned_vocabularies_have_one_rust_type, test_solve_settings_enum_types. B7 tests: a compile_fail doctest for a root/instance swap. Also just codegen-check leaving no drift.
standard: core-3.1/process-simulator-1.1
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s22]
---

# ADR-0115: Declare entity identities in the registry, keep one Rust type per vocabulary, and catalog hash frames

## Context

The [typed data contracts review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md)
found four gaps.
- **TD04.** Workflow, operational and modeling identities share one untyped `SemanticId`:
  - 277 hand-written `*_id` fields, and generated rows typing every `*_id` the same way;
  - 53 functions take two or more identities, so swapping them compiles.
- **TD06.** Decision vocabularies live outside the registry, against ADR-0113 Outcome 2:
  - the Ipopt linear solver, orderings, Hessian mode and reuse policy;
  - the `annotation valid` extrapolation policy, a `String` compared in six places;
  - twelve source-owned vocabularies get a second, unused generated Rust enum.
- **TD08.** Hash-frame contexts are literals at 90 call sites (119 distinct spellings), neither discoverable nor guaranteed unique.
- **TD10.** One generated file is not protected against hand edits.

A typed-id mechanism already exists, but it serves only the physical-registry ids in
`pse-quantity` (13 types).

## Scope

This record binds:
- a registry declaration kind for **entity identities**, and the generation of one typed id per entity;
- the rule that every decision vocabulary crossing a boundary is a registry enum, with one Rust type;
- one catalog of hash-frame contexts in `pse-ids`.

It **amends** §4.1 (declaration kinds), §4.2 (generated trees), §5.1 (forms of identity) and
§5.3 (framing). It does not change any identity's bytes, any frame spelling or any hash. The
store's representation of these types is ADR-0114's. Typed dense index spaces at coordinate
boundaries (Plan 22 B6) are a refactor within existing contracts and need no record.

## Drivers

- **DP-02 and DP-04.** "Which thing" is a type, not a naming convention; a swap between entities fails to compile (S22).
- **DP-01 and AP-04.** Each vocabulary and identity is declared once and derived everywhere, in Rust, Python and PostgreSQL.
- **DP-24.** Frame versions are discoverable in one place.
- **PS-02.** The extrapolation policy is a typed, recorded choice.
- **Maintainer direction (2026-09-28).** Programmatic derivation over static text.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep bare `SemanticId`, with naming conventions | Swaps compile; FK meaning lives in field names | Rejected |
| Hand-written newtypes per entity in each crate | Duplicates the registry's knowledge of keys and FKs; drifts from generated rows | Rejected |
| **Registry-declared entity identity on key columns, inherited by FK columns; typed ids generated** | One declaration. Generated rows, PostgreSQL domains and Python types follow. Reuses the existing typed-id macro, moved to `pse-ids` | **Selected** |
| slotmap keys | Salsa owns authoring identities, and ids are durable UUIDs, not arena handles | Rejected |
| Frame literals, with a uniqueness lint | A new alignment lint; spellings stay scattered | Rejected |
| **A `Frame` enum in `pse-ids`** | One declaration; the compiler enforces use; uniqueness is a unit test | **Selected** |
| A second generated enum for source-owned vocabularies (current) | Two Rust types for one vocabulary | Replaced by a re-export |

## Outcome

1. **Entity identities are declared.**
   - A registry relation's key column may declare an entity identity: run, attempt, job, publication, workspace, study, solution, source bundle, settlement, reader lease; then root, instance, definition, member and block.
   - Foreign-key columns referencing it inherit the identity, and assembly rejects a conflicting declaration.
   - `reference.schema_*` relations describe the declarations, so the registry still describes itself.
2. **Typed ids are generated.**
   - The generator emits one typed id per declared entity into `pse-model`, through a single macro in `pse-ids`. `pse-quantity`'s `semantic_id_newtype!` moves there, and its 13 physical-registry ids use it.
   - A typed id is a transparent wrapper over `SemanticId` with `From` in both directions and `Display`.
   - Generated rows type their identity columns with it. Minting stays on the UUIDv7 path, or on keyed derivation, as §5.1 says.
3. **One vocabulary, one registry enum, one Rust type.**
   - Every enumeration that decides behaviour and crosses a crate, process, store or language boundary is a registry enum. Newly declared by this record:
     - the Ipopt linear solver and orderings, Hessian mode and reuse policy (ADR-0113 Outcome 2);
     - `ExtrapolationPolicy` for `annotation valid`;
     - `TerminationCode`, `RetentionPhase` and `SettlementOutcome` (with ADR-0114);
     - the numerics provenance field kinds that are read by name today.
   - Where a vocabulary's source is hand-written Rust (`DiagnosticCode`, `FailureClass`, the `pse-schema` platform enums, the `pse-quantity` enums), the generator emits a `pub use` of that type instead of a second enum.
   - No decision compares a vocabulary's spelling as a string.
4. **Frame catalog.**
   - `pse-ids` declares every hash-frame context once, as a `Frame` enum with `as_str`, and `FramedHasher::new` takes a `Frame`.
   - Spellings are unchanged and the golden vectors still hold. A new frame or version is a new variant.
   - The generated documentation lists the catalog.
5. **Protected generated files.** Every generated file, `python/pse/_native.pyi` included, is listed in AGENTS.md prime directive 2, the edit hook and the deny rules.

### Consequences

- **Consumer refactor.** Consumers change signatures from `SemanticId` to typed ids: operational and workflow first (B3), modeling and compiler next (B7). Arithmetic on identities is refused by construction.
- **Python.** The generated Python contracts gain distinct id aliases. `SolveSettings` takes enum types rather than strings, and `_native.pyi` shows them.
- **Registry and generator.** The registry gains one declaration kind and the generator one emission path.

### Compensating controls

- `compile_fail` doctests for representative swaps.
- The `pse-ids` golden vectors.
- The regeneration check.
- Registry assembly rejects conflicting identity declarations on an FK.

### Confirmation

**Architectural reasoning.** The review's slots 3, 4 and 7 (TD04, TD06, TD08, TD10).

**Interface-checked.**
- The existing typed-id macro and its 13 users.
- The generator's existing re-export of the 16 quantity enums.
- `FramedHasher::new(&'static str)`.

**Tested.** The tests in `verification:` settle the implementation in Plan 22 B3, B4 and B7.

## Pros and cons

Typed ids and one vocabulary type remove whole classes of silent swap and drift for a
mechanical signature refactor. The cost is churn across consumer crates, spread over three
packets.

## More information

- [Typed data contracts review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_typed-data-contracts_2026-09-28.md): TD04, TD06, TD08, TD10.
- Architecture companion [§12.2 and §12.4](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#12-typed-data-contracts).
- [ADR-0114](0114-typed-operational-store.md) (store domains and ENUM types).
- [ADR-0116](0116-typed-boundary-documents.md) (Python boundary).
- Plan 22 packets B1, B3, B4 and B7.

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's approval of the typed data contracts review (Accept-scoped, author review, Proposed evidence level).
