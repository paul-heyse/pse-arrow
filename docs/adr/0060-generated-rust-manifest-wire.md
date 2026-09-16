---
id: ADR-0060
title: Generate Rust manifest wire types from ManifestSpec
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-15, DM-42, DM-53]
blueprint: [§4.2, §20.2, §20.5]
review: docs/design_review/reviews/design_review_wave2-contracts_2026-09-14.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A manifest wire field, version, nullability, native binding or supported native codec must change.
verification: crates/pse-schema/tests/codegen_determinism.rs; crates/pse-catalog/src/store/manifest.rs recursive parity and malformed-envelope tests; just codegen-check; just test-package pse-catalog -p pse-schema -p pse-relations
---

# ADR-0060: Generate Rust manifest wire types from ManifestSpec

## Context

Register R-27 is triggered by Wave 2. Python and documentation already project the
registry's `ManifestSpec`, while the catalog repeats its Rust wire fields manually.
The existing recursive parity test detects drift but does not remove that duplicate
declaration (blueprint §4.2 and §20.2).

## Scope

Generate the Rust envelope and all nested wire objects under
`crates/pse-catalog/src/generated/`. Preserve the current JSON field names, declaration
order, required/optional behavior, textual identity codecs, numeric widths and format
versions. This completes the existing generation contract; it introduces no new
membership or hashing rule. Root-owned blueprint and Wave 2 review amendments accompany
this proposed record before acceptance.

## Drivers

One declaration must determine the complete wire shape. Existing callers should keep
semantic ID/hash role types rather than lose type distinctions when generation starts.
Unknown fields and inconsistent native projections must fail explicitly.

## Options

Keep hand-written Rust structs and strengthen parity tests: retains duplicate authority.
Hard-code field-path-to-Rust-type mappings in the generator: moves that duplication.
Generate generic JSON-shaped types and rebuild a second typed Manifest: repeats the
fields and adds conversion allocation. Generate from declared fields with checked
native bindings: selected.

## Outcome

`ManifestSpec` controls every generated field recursively. An optional closed native
Rust binding annotation on a `ManifestField` preserves the existing `SnapshotKind`,
`EncodingFormat`, `SchemaVersion`, hash-role and `SnapshotParent` representations.
Each annotation is checked against its underlying `ManifestType` before generation.
It selects a representation/codec, never overrides the declared wire shape, and is
not an additional semantic identity input.

The generator emits strict serde structs and version constants. The catalog re-exports
existing public names as aliases, and keeps codecs, envelope checks, actual-content
admission and storage operations hand-written against those generated types. The parent
adapter uses a generated nested wire object; no hand-written parent wire declaration
remains. `None` continues to encode as JSON null, and optional fields continue to admit
absence or null. No successful decode or matching digest certifies snapshot validity.

### Consequences

The Rust generation target gains the catalog generated root. ADR-0051's existing
complete inventory and byte comparison cover it. A generator-compatible declaration
change updates Rust/Python/docs together; a wire change still requires the explicit
version/compatibility decision required by blueprint §20.5.

### Compensating controls

Registry admission refuses incompatible native annotations. Generated structs reject
unknown or duplicate object fields at every depth; required members and numeric widths
remain enforced by serde. Existing recursive `ManifestSpec` parity, malformed manifests,
parent/member uniqueness and source/content admission tests remain independent controls.

### Confirmation

**Interface-checked:** inspected `model/manifest.rs`, `catalog/manifest.rs`, the Rust and
Python emitters, `Language::roots`, the xtask writer, catalog codecs and recursive parity
tests in this checkout. Executable confirmation requires the commands in `verification:`
after actual regeneration and catalog integration. This record does not claim a passed
runtime or regeneration gate at drafting time.

## Pros and cons

Generated fields eliminate the shadow envelope while aliases preserve callers. Native
bindings add a small checked declaration surface; that surface is explicit beside the
wire field rather than hidden in generator conditionals.

## More information

Blueprint §4.2, §20.2 and §20.5; ADR-0031, ADR-0051; register R-27;
[plan 04 W2-03](../plans/04-wave-2-semantic-compilation.md#w2-03--one-manifest-declaration-in-rust-and-python).

## Status history

- 2026-09-14 — proposed; R-27 activated by Wave 2. Wire compatibility is retained.
