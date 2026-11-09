---
title: "Plan72a: Native contracts and schema interpretation"
status: draft
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, libraries, design, validation]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md]
---

# Native contracts and schema interpretation

## Target and foundation assessment

**Proposed.** A contract author changes one semantic declaration and receives consistent
native database vocabulary, typed operation inputs, codecs and inspectable installed meaning.
Keep scientific definitions and physical admission authoritative; generate the representations
that consumers need rather than independently interpreting the same shape at every boundary.
Breaking representation and public-input changes are acceptable. The
[coordinator](72-surrealdb-functional-improvements.md) owns shared decisions and O01/O02/O14
dispositions; this companion owns A01–A04 progress.

**Interface-checked baseline:** `21f703887cc307f986975f9a2fba998f8bd2afa7` plus the
concurrent working tree inspected on 2026-11-09. The
[Surreal lowering](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-codegen/src/codegen/surreal.rs)
already generates tables, field constraints, native row codecs and selected graph policies.
It repeats literal enum unions, lowers UInt64 to Decimal, and separates authoritative finite
float bits from a query projection. The
[canonical codec](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_codec.rs)
checks native kinds, exact identity and unsigned range. Those distinctions carry meaning;
their incidental rendering/parsing is replaceable. Existing source-owned generation is the
suitable foundation, while repeated unions, mutation-shape interpretation and normalization
are concrete consolidation targets. No second editable schema or universal descriptor mapper
is selected.

Use core 3.4, efficient architecture 1.0, process-simulator 1.5 and the binding selected by
[standard.toml](../design_review/design_principles/standard.toml). Blueprint §4/§5/§20
remain the current contracts. RC01 was explicitly accepted for planning on 2026-11-09.
Name exact contract changes and complete any required ADR/design route before implementing
them; ordinary code generation inside the existing contract needs no invented approval gate.

## Selected design

Generate `DEFINE TYPE` for named closed vocabularies and constrained shapes, then reference
them in fields and callable arguments. Their name, coercion and ordering belong to declared
meaning. Keep transition legality, scientific admission and context-specific cross-field
conditions with their operation. Generate distinct create/merge/replace inputs where omission,
defaulting and removal differ. Retain exact NONE/NULL and native record-ID versus text meaning.
The [v4 named-type reference](https://surrealdb.com/docs/reference/query-language/statements/define/type)
and [review O01–O02](../functional_reviews/surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md)
describe the mechanism and its limits.

Use engine-normalized interpretation as developer feedback and an installed-schema cross-check.
SurrealKit rc.1 `compile` and live `typegen`/v2 supply structured kinds, presence and diagnostics;
their `semanticDigest` covers a selected schema projection, not the whole codec/history contract.
Prepare an embedded backend explicitly for offline compilation. The current CLI-only selection
does not provide it. Compare the same selected engine semantics with the patched live server;
report unsupported definitions visibly rather than accept lenient coverage as agreement.
[Compiler](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs),
[v2 specification](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md).

Native `u64` is the preferred simplified representation for the full unsigned domain only
with a faithful selected query/wire path. Tagged v4 native Proto conversion widens values;
decoder support alone does not establish encoder type identity. A03 settles this concrete
correctness risk with source inspection and, only if necessary, a focused boundary control.
It does not postpone vocabulary and compiler improvements or require proof of a speedup.
Keep authoritative IEEE bits and declared temporal precision even when another native scalar
is convenient. [Tagged conversion](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs).

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| A01 | Planned | Current declaration, codec and installed-interpretation owners | One declared contract/projection boundary and exact decision route for the selected changes; consumers and representation requirements identified. |
| A02 | Planned | A01 meaning and relevant decision route | Named vocabulary and operation-shaped generated conversions used by selected producers and all affected consumers; duplicate rendering/mapping removed. |
| A03 | Planned | A01 value requirements; 72f transport owner participates | Full native unsigned/temporal representation choice settled and selected end-to-end path integrated, or exact current representation deliberately retained with the unsupported native condition recorded. |
| A04 | Planned | A01; A02 definitions as each family becomes available | Strict engine-normalized schema feedback and live correspondence for represented contracts; unsupported coverage explicit; superseded normalization/publication removed. |

<a id="A01"></a>
### A01 — Declare the native contract and projection boundary

Trace vocabulary, native kind/presence, identity and operation inputs from existing owners to
generated DDL, codecs, function arguments and installed-definition checking. Keep one source
for each meaning. Record the selected storage/query/wire representation and the information
each projection intentionally omits. Set declaration order where named enums affect sorting.
Identify which repeated validations protect a changed trust/interpretation boundary and which
can disappear once the generated contract carries the property.

Produce the consumer/migration footprint and exact architectural amendment where an enduring
contract changes. Do not require a new registry/type/service merely to name a responsibility.
A revealing acceptance case is adding a state vocabulary or optional nested field without
independent interpretation edits in database generation, operation input and native readback.

<a id="A02"></a>
### A02 — Generate shared vocabulary and operation-shaped values

Extend the existing generator to emit reusable native type definitions before dependent fields
and functions. Derive typed read/create/merge/replace conversions from declared field policies;
normalize at the owning write boundary rather than repeatedly in every controlled reader.
Keep independent admission for foreign/corrupt or differently interpreted values. Consolidate
scalar-key/record-link duplication only where one native identity plus generated projection
preserves rename, role and occurrence meaning. Composite keys replace indexes only for a
concrete locality/access benefit agreed with 72d.

Migrate callers, installed-definition comparison and generated fixtures in the same change.
Use `just codegen` for generator changes; never edit generated trees. Remove superseded literal
rendering, repetitive object construction and obsolete operation-input adapters. Targeted tests
exercise a wrong kind, omitted versus cleared input, enum ordering, invalid transition and
typed endpoint identity; generation self-agreement is not an independent semantic oracle.

<a id="A03"></a>
### A03 — Settle exact native representation and complete the boundary cut

Inspect selected outgoing/incoming protocol conversions with 72f, including sized numbers,
temporal values and archive/inspection consumers. Prefer direct generated tagged conversion
when it preserves the required domain with less machinery than Decimal normalization/parsing.
If the selected supplier path cannot carry the required meaning, retain the exact representation
for that family and record the concrete limitation; do not introduce fallback transports or
dual normal storage. This decision settles the representation branch, not the whole series.

For a selected change, migrate storage declaration, write binding, query comparison, readback,
interpretation identity and historical reader through 72b/72f together. The serious risk is
silent value/type degradation, so targeted cases include zero, `i64::MAX`, its successor,
`u64::MAX`, fractional/wrong-kind input, overflow, NONE/NULL and signed zero. Source-established
correctness can settle the design; no throughput comparison or broad codec campaign follows.

<a id="A04"></a>
### A04 — Make schema disagreement inspectable

Prepare the required SurrealKit embedded capability under its existing host owner, without
making help/read commands build it. Feed generated definitions through strict compilation and
compare its represented semantic projection with introspection of an owned selected-server
fixture. Preserve provenance, engine/version and unsupported-definition diagnostics. Named
types, extensions and operation-specific presence must not disappear behind a successful digest.

Expose useful normalized differences through existing structured tooling, consuming Plan71's
current evidence/report facilities where applicable. Keep the production generator authoritative.
Use an external emitter only for a disjoint useful projection whose publication it actually
simplifies; its manifest/unchanged-byte/stale-file/`--check` behavior can replace that output's
old publisher, not wrap a second owner around the same files. No bundled Rust emitter is assumed.
Acceptance includes an intentionally wrong kind/presence/permission, unsupported definition,
and a compiler/live engine mismatch. Remove replaced scrapers, normalizers and file publishers.

## Coverage and verification

| Review direction | Owner |
| --- | --- |
| O01 named vocabulary, exact kinds, field-scoped flexibility, stored constraints and identity/locality | A01/A02, with A03 for representation risk and 72d for graph/access changes |
| O02 compiler/live interpretation, v2 coverage, mutation inputs and selective emitter publication | A02/A04 |
| O14 shared policy-aware mapping, numeric/temporal fidelity | A03 for meaning and generated conversion; 72f owns transport/client lifetime and incremental delivery |

During implementation use scoped `pse check` and targeted `pse test` for changed generation,
codec and installed-schema controls, retaining force-validation where applicable. No benchmark
is required for removing repeated rendering or conversion. Integrated qualification belongs to
72i. **Proposed:** this draft records design and planned verification only; no implementation,
tests or measured benefit are claimed.
