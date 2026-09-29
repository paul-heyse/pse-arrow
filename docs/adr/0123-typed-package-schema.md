---
id: ADR-0123
title: Declare domain knowledge through a typed package schema
status: accepted
date: 2026-09-29
deciders: [paul-heyse]
level: decision
principles: [DP-02, AP-04, DP-01, AP-05, DP-07, DP-24, PS-02]
blueprint: [§5.3, §6.1, §6.5, §6.13, §6.15.1, §9.1, §9.3, §9.10, §22.3]
review: docs/design_review/reviews/design_review_typed-domain-model_2026-09-29.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A package port needs a data shape the schema cannot state (a kernel gap under the knowledge-boundary test), or admission of a declared data bank exceeds its workspace limits after KR5 and KR9 land.
verification: Plan 23 KR3–KR8 targeted tests (type_arena_render_parse_roundtrip, identifier_values_are_never_interpreted, keyed_identity_is_the_key_declaring_kind_and_typed_keys, same_key_in_two_forms_is_refused, ref_calls_dispatch_one_body_per_concrete_kind, non_static_ref_is_refused, self_referential_lineage_admits_and_cycles_are_refused, completeness_over_declared_sets_checked_at_admission, symmetric_pair_answers_both_orientations, production_root_reading_an_oracle_row_is_refused, increment_guards_its_integration_interval, import_requires_dependency_by_identity), the Outcome 8 revision-identity tests (source_revision_changes_with_one_data_byte, source_revision_changes_with_a_physical_name_binding, unchanged_inputs_reproduce_the_source_revision), the D0 refusal corpus, and scenarios DM2–DM4 and CT-S10.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s01, docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#s10]
---

# ADR-0123: Declare domain knowledge through a typed package schema

## Context

Scientific knowledge is package data (blueprint §6.5, §9.3), but its durable form is not
typed. `authored.modeling_declarations` stores type names, data cells, missing-value
policies, citations, entity attributes and imports as UTF-8 text, which admission
re-parses (§6.15.1). In the shipped packages, kinds carry no attributes, and method
selection, phase, source and units are carried by entity and column names, while
provenance is free text. Before data banks and method families are ported, the schema that
every library conforms to must be typed and relational.

## Scope

This record amends the modeling IR and admission contracts. It adds no scientific concept
to Rust or the registry. It binds:

- the structured IR;
- entity records and keyed identity;
- relation admission;
- provenance facets;
- envelopes;
- naming and imports;
- the source identity frame.

It does not cover units and coefficient types (ADR-0124), bulk data documents (ADR-0125)
or the thermodynamic schema itself (ADR-0126).

## Drivers

Scenarios:

- CT-S01: add a correlation as data.
- CT-S10: an iterate leaves a correlation's range.
- Plan 23 DM2–DM4: cross-bank identity, wrong coefficient units, and oracle data in production.

Principles:

- DP-02: decisions never parse free text.
- AP-04: one representation per concept.
- AP-05: declarations enforce meaning.
- DP-07: relationships keep their kind, direction, multiplicity and provenance.
- PS-02: parameters are versioned data with provenance, and their envelopes are enforced.

## Options

| Option | Effect |
|---|---|
| Keep the text IR and add conventions | No contract change, but every consumer re-parses and meaning stays in names. Rejected |
| Registry-declared scientific families (species, parameter sets) | Strong Rust typing, but it reverses §6's removal of science families and puts every new domain concept behind Rust and codegen. Rejected by the maintainer, 2026-09-29 |
| Generic typed-relational mechanisms in the kernel, with the domain declared in packages | One authority per concept in package data. The kernel enforces types, keys, references, completeness, facets and envelopes generically. Selected |

## Outcome

### 1. Structured IR

The modeling IR becomes structured syntax, in the next relation version after Plan 23 H1's
fixture-policy version:

- types are a post-order arena;
- data cells are typed tagged values: quantity, reference, identifier, text, integer, boolean or missing, with an optional uncertainty;
- names occur only as path segments, which the checker resolves once to identities.

The vocabulary the kernel acts on consists of registry enums:

- missing-value policy;
- annotation kind;
- analysis-fact namespace;
- version operator;
- key symmetry;
- the data facets of Outcome 5.

### 2. Entity records and keyed identity

Entity kinds are typed records:

- attribute schemas with defaults;
- single refinement;
- opaque identifier schemes, whose values are never interpreted and are unique within the package closure.

A keyed kind's row identity is a catalog frame over two things: the **key-declaring kind**,
and its ordered, typed key values, including defaults.

- The concrete refinement is content, not identity.
- Key uniqueness holds across all refinements of the key-declaring kind.
- Enumeration members used as keys have identities, so renaming a member does not re-key rows.
- A key supplied by a dataset (for example its source) is an explicit, declared binding of that dataset.

A kind may bind inherited attributes, including functions. A `Ref` must be static at
specialization; a `Ref` that depends on a runtime value is refused. A call through a static
`Ref` resolves the most-derived kind's bound function and specializes per resolved
function, on the existing function-specialization path. Constants are typed declarations.

### 3. Relation admission

Relations are admitted in three phases:

1. Row identities are formed from keys.
2. Order-free checks run: foreign keys, completeness over declared sets, pair symmetry with a diagonal policy, uniqueness, and integer-range keys.
3. Derived columns and row requirements are evaluated in the directed acyclic graph of their value dependencies.

Self-referential relations and lineage admit when they are acyclic at row level. A cycle
is refused with its rows named.

### 4. Validity layers

Validity is the intersection of three layers:

| Layer | Source |
|---|---|
| Form | The function's domain |
| Data | Envelopes declared on a relation or kind |
| Closure | `annotation valid` |

- Data declares envelopes only. The property package, or the analysis, selects the extrapolation policy per layer, and the selection is recorded.
- A form declares which of its arguments, or which integration interval, each envelope guards. For example, `dh(T0, T)` guards the whole interval.
- Every check records its layer.

### 5. Provenance

Every dataset, constant and test names a source entity and a role. The source entity is a
kind carrying the provenance facet. A role is a package enumeration member that declares
its kernel facets:

- `test_only`: the data may be read only by test fixtures;
- `requires_lineage`: the data must name its acyclic lineage.

Each row's origin role is a property the kernel provides, not a declared attribute.

Test-only taint propagates along resolved row and entity references. It is resolved
statically where keys are literal, and otherwise at specialization. It is the only
authority for "production reads no test-only data": a root outside a fixture that reads a
test-only row is refused. A package's own selection preferences filter candidates; they
restate no policy.

### 6. Physical references

Quantity types and reference states are named once, in the physical document. Packages
reach them through generic physical-reference types.

A datum-carrying quantity's datum is read from its type. No parameter-set attribute
restates it.

### 7. Imports

Imports resolve by package identity with a typed version requirement.

### 8. Identity frames

Source identity is framed by `ModelingSourceRevisionV2`. Its preimage has three parts:

1. The structured declaration rows.
2. For each data document: its identity and its byte-level content hash (ADR-0125).
3. The physical-inventory identity, which replaces V1's alias map.

Every frame whose preimage changes receives a new catalog variant. This includes the
dispatch-body and function-specialization frames whose preimages render unit literals.
Plan 23 KR3 lists them, and the golden-vector list gains each one (DP-24).

### Consequences

- Every type-bearing site moves in one change (Plan 23 KR3).
- Stores and fixtures are regenerated from source, with no migration (§20.5). Run artifacts carrying an earlier source frame read as requiring migration.
- The seed packages are rewritten onto the schema (Plan 23 SM0–SM6).

### Compensating controls

- A property round-trip test covers the parser and renderer.
- Revision-identity tests: one changed data byte or name binding changes the source identity, and unchanged inputs reproduce it.
- The whole-seed conformance run (Plan 23 H1) is the regression net for every migration packet.
- The knowledge-boundary audit (Plan 23 AUD) classifies each Rust change.

### Confirmation

- The targeted tests named in `verification:` and the D0 refusal corpus establish the executed behavior.
- The Plan 23 AUD review establishes the architectural claim that no scientific concept entered Rust or the registry.
- Plan 23 owns implementation status.

## Pros and cons

| For | Against |
|---|---|
| Data banks and method families become pure data, checked at admission, and libraries share one schema | A wide IR change and a full seed rewrite come before new science lands |

## More information

- [Plan 23](../plans/23-thermodynamic-domain-and-campaign.md) owns execution and finding dispositions.
- The [design review](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md) raised F01, F03–F05, F08 and F11–F13; this text incorporates their resolutions.
- Related records:
  - ADR-0099: modeling language and identities; this record refines its IR shape.
  - ADR-0100: functions read immutable package data.
  - ADR-0101: typed annotations.
  - ADR-0115: registry enums for behaviour-deciding vocabulary.
  - ADR-0119: fixture analysis selections.

## Status history

- 2026-09-29: proposed; amended the same day with the design-review resolutions.
- 2026-09-29: accepted on the maintainer's Plan 23 authorization; design-review verdict Accept after the resolution check (§12–§13).
