---
title: "Plan72a: Native contracts and schema interpretation"
status: draft
date: 2026-11-09
adrs: [ADR-0261]
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
| A01 | Implemented | Current declaration, codec and installed-interpretation owners | Shared contract/projection boundary and consumer footprint recorded below; proposed ADR-0261 and blueprint §4.1/§20.5 amendments precede dependent changes. |
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

## Execution checkpoint — November 9

Execution starts from clean `273326931a18204b9a795578dff750ca1a7af894`. Plan71 is
complete; its original evidence, structured reports and fixture ownership are receiving
foundations. Plan66 remains independently owned. A01–A04 statuses above remain the native
progress account; implementing a prerequisite does not close its enclosing package.

### Shared contract decisions

| Concern | Selected contract and consumer footprint |
| --- | --- |
| Vocabulary | `EnumSpec` owns stored spellings and membership. Emit named constrained strings (`AS string ASSERT ...`); ordinary string ordering remains. Its member ordinal is presentation-only. Native ordered unions require an explicit rank declaration. |
| Presence | Existing optional scalar missing/NONE means absent; NULL is refused unless explicitly declared. Creation omission/defaulting, merge unchanged/set/remove and complete replacement are distinct inputs. |
| Mutation | Existing field policy owns immutable/mutable/default/initial semantics. Immutable fields may be creation inputs. Server-derived fields come from operation context. Input types feed authorized operations; they grant no authority. |
| Identity | Semantic IDs retain exact binary width and role; opaque lookup keys remain distinct from scientific identity. Native record links must match table and projected key. Graph occurrence and endpoint meaning survive every projection. |
| Unsigned | Native U64 storage and exact domain coercion are selected. Proto `Number::wide` preserves value as Int/Decimal while dropping width. Generated decoding checks exact unsigned values; string roundtrips and redundant Decimal storage constraints retire with consumers. |
| Temporal/numerical | Current signed microsecond timestamps and authoritative IEEE bits retain their meaning, including signed zero. Query floats remain projections. No new scientific frame or hash meaning is implied. |
| Interpretation | Canonical, knowledge and evidence retain separate generated DDL/codecs and installation identities. Changed storage/input interpretation advances its owner explicitly. A v2 semantic digest does not replace schema/codec compatibility or native installation admission. |
| Enforcement | Generated domain checks replace duplicated membership/shape interpretation. Transition legality, cross-field predicates, issuer/generation/lease authority and foreign-value admission remain at their owning boundaries. |

The implementation footprint includes canonical registry lowering, private knowledge/evidence
lowering, generated rows/codecs, SQL signatures and arithmetic, direct object producers,
semantic readback comparisons, payload-size estimates, installation/maintenance witnesses,
passive catalog readers, host output and native fixtures. Unsigned SQL is classified by domain,
not changed by blanket Decimal substitution. Missing native conversion support refuses rather
than silently degrading precision or using a second transport.

### Schema feedback contract

Use three separately identified comparisons: independent declaration expectations against
compiler interpretation; represented compiler semantics against the selected live server;
and complete intended definitions against the native installed witness. Type assertions and
ordered definitions, extension digest/capabilities, authentication details, codec/history
meaning and original issuer remain independently checked where v2 omits them.

SurrealKit preparation explicitly selects embedded memory support and binds features into its
build identity. Compiler/live staging has an isolated working directory and explicit strict v2
engine configuration. Fresh output files and independently parsed diagnostics are required;
JSON/stdout/lenient/acknowledgement modes do not establish acceptance. Unsupported coverage
remains visible even with matching digests. Full function bodies remain intact; any unsupported
embedded extension dependency is an explicit whole-definition coverage exclusion.

### Receiving contracts for the remaining series

| Receiver | A's delivered foundation | Receiver-owned remaining decisions |
| --- | --- | --- |
| 72b | Source/target field mapping, presence/default/removal policy, interpretation and conformance expectations | Reconstruction/native/backfill effects, retained data, index readiness, activation and original-operation recovery |
| 72c | Typed operation projections and policy carriers, exact-value conversions and independently required checks | Transaction/isolation, authority, readiness acquisition, retries and ingress placement |
| 72d | Record/reference/occurrence identity, declared ordering and graph/schema hooks | Topology, indexes/composite locality, complete selection and ranked retrieval |
| 72e | Source/interpretation identity and explicit projection loss | Freshness/dependencies, subscriptions, resynchronization and durable replay |
| 72f | Numeric/temporal/presence/identity mapping and actual widened wire size implications | Protocol assembly, bounded delivery, admission, artifact/connection custody and history |
| 72g | Generated fixture inputs/readback and structured schema differences | Database case composition and developer/operator workflows using Plan71 facilities |
| 72h | Versioned coverage/difference observations with provenance | Execution diagnostics, shared query observation and scratch lifecycle |
| 72i | Author/change/inspect/install/write/read cases, actual migrated producers/consumers | Assembled acceptance, scope-end qualification and retirement |

Every handoff distinguishes agreed semantics, implemented APIs, executed targeted evidence and
remaining receiving. Disposable native fixtures use fresh owned state. Ordinary open remains
validation-only; configured retained-state transition belongs to 72b, with historical reader
obligations received jointly with 72f before activation where applicable. No compatibility shim
or reset is implied by a new interpretation.
