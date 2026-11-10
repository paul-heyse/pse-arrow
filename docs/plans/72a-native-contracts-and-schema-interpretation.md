---
title: "Plan72a: Native contracts and schema interpretation"
status: complete
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
| A02 | Implemented | A01 meaning and relevant decision route | Native vocabulary/shapes and producers integrated in d9e82627f; parser/generation/input/live merge and original-issuer catalog controls pass. Native stored-expression normalization preserves meaningful drift checks. |
| A03 | Implemented | A01 value requirements; 72f transport owner participates | Native U64 and exact digit projections integrated in d9e82627f; source review, codec/metadata/sequence and full indexed/native boundary controls pass. |
| A04 | Implemented | A01; A02 definitions as each family becomes available | Embedded tool prepared with bound source fixes; strict compact feedback and passive comparison implemented; six host controls, ready and owned full-schema compiler/live/drift/disposal receiving pass. |

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

G03 can enrich unsupported-definition display names from the original supplier diagnostic's
JSON `source.name`. Compact feedback retains category, reason and source hash; it deliberately
does not republish raw definition bodies. Caller-owned supplier files retain the original
context. This presentation work grants no new schema or activation authority.

The generated API names are `Runtime<Table>Create<'a>`, `Runtime<Table>Merge` and
`Runtime<Table>Replace`, with `create_<table>`, `merge_<table>` and `replace_<table>` native
conversions. Create borrows row values; optional/defaulted omission is explicit. `NativeUpdate`
exposes Keep/Set; optional mutable `NativeMerge` also exposes Remove. Named merge constraints
preserve object membership and reject required-field removal. Read-row encoders remain useful
for independent readback/equality observations. Research write procedures consume named create
shapes; staging casts at its existing operation boundary. Existing research-role membership is
an operation-owned text policy and remains with 72c's operation contract, rather than inventing
a registry vocabulary from one guard.

72b must receive a concrete lifecycle limit: native type definition/inspection support does not
widen frozen rollout removal authority. `RemoveEntities` currently admits table/field/event/index
only. Any selected type/function/module removal needs explicit typed effect handling and
dependent-definition/data policy under B01–B03. 72a fresh fixtures establish no retained-store
migration or implicit removal obligation.

Unsigned text used in keys and receipts retains plain decimal digits. V4 U64 SQL text rendering
includes a type suffix, so selected SQL key/string projections deliberately use an exact Decimal
formatting cast. This is a presentation projection of native U64, not dual storage. Native
boundary controls cover those spellings and indexed lookup across Proto widening.

## Outcome

**Implemented, 2026-11-09.** A01–A04's selected foundation is delivered. `ce3a6bbe7`
records the contract and handoffs; `cb081fa74` derives input policy; `d9e82627f` integrates
native vocabulary, operation inputs, exact unsigned values and canonical/private producers;
`924b0c3b2` supplies embedded compilation and strict compact live feedback. The final catalog
receiving correction normalizes Type ASSERT expressions through native stored-text rendering.
ADR-0261 remains proposed; implementation is not formal ADR acceptance or series qualification.

**Tested.** All final selected functional controls passed against the zero-failure baseline,
on Linux with the pinned compiler, selected source-built v4 server and explicit Arrow validation
through the native correctness routes. Selections below retain their actual boundaries:

| Command / selected scope | Final evidence |
| --- | --- |
| `pse test pse-schema --lib -E 'test(canonical_input::tests)'` | 2 passed; initial/default/readonly and optional removal semantics. |
| `pse test pse-codegen --lib -E 'test(native_vocabulary_tests)'` | 3 passed; selected-family and full-DDL parsing plus type-name collision refusal. |
| `pse test pse-codegen:native_field_declarations pse-codegen:codegen_determinism` | 18 passed; independent declarations and deterministic generation. |
| `pse test pse-operations --lib --features canonical-maintenance` with the named-type/input controls | 2 passed; native Type constraints/order and generated create/merge/replace semantics. |
| Same maintenance route with `test(named_type_assertions_compare_stored_grouping_without_losing_meaning)` and `test(catalog_named_types_preserve_constraints_and_order)` | 2 passed; harmless grouping normalization and counterexamples for casts, operands, comparison, assertion order, mixed precedence and permissions. |
| Same maintenance route selecting catalog assertions/permissions/numeric meaning, data-effect refusal/function semantics, event semantics, required installed events and rollout digest | 5 passed; existing catalog protections remain. |
| `pse test xtask:xtask --target-kind bin --no-default-features -E 'test(knowledge_schema_is_tooling_only_with_declared_fulltext_and_native_graph)'` | 1 passed; private family isolation and native vocabulary survive policy lowering. |
| `pse test pse-operations --lib --features native-service` selecting unsigned codec/Proto, metadata bound, ordered sequence and storage/query/wire boundaries | All 5 controls passed; the failed boundary control's corrected focused rerun passed. Full domain, indexed lookup, formatting, merge membership and overflow rollback are exercised. |
| `pse test pse-host --lib -E 'test(native_schema_feedback_) \| test(preparation_refusals_preserve_selection_and_recover_only_with_matching_bytes)'` | 6 passed; version/digest/error/coverage/provenance, source redaction, file admission, failed child and timeout/drain controls. |
| `pse test pse-operations:native_service --features native-service,canonical-maintenance --ignored only -E 'test(=surrealkit::surrealkit_generated_contract_uses_owned_service_and_disposes)'` | 1 passed; untouched full generated schema compiles, represented live semantics match, kind/permission drift is detected, and the original fixture is disposed. |
| Explicit fresh `pse observe prepare --installed-schema`, then the same native-service route selecting `installed_schema_cli_observes_only_the_original_v4_issuer` with its exact reader ID | 1 passed; no false merge-type drift, foreign issuer refused, state unchanged and no implicit post-stop startup. |
| `pse check pse-operations pse-codegen pse-runtime pse-host xtask --all-targets --features pse-operations/canonical-maintenance,pse-operations/native-service,xtask/evidence-tools` | Passed, zero compiler failures; changed canonical/private consumers compile together. |
| `just codegen` | Passed all 16 schema targets through the staged owning generators. |
| `just ready` | Skills sync and doctor passed for the final embedded tool inputs. |

Native service receiving completed with no pending native lifetime. Host bootstrap/lock and
discovery failures were repaired before rerunning affected work; they are not passing evidence.
One interrupted discovery frontend retained unknown original metadata without a launch record;
no test result was inferred, and a fresh selection followed confirmation that its owner was gone.

### Mistakes made and corrected

The initial merge renderer used an unsupported operator spelling; full native parser coverage
was added and the spelling corrected. The first unsigned boundary test used reserved
`SELECT VALUE` syntax; selecting complete records corrected that test. Actual full-schema
receiving exposed pinned-tool gaps for U64/named references/inline caps, insufficient diagnostic
capture, and an evidence-private reader applied to ordinary supplier output modes. Narrow
source-bound preparation and data-file admission fixes preserve strict refusal and coverage.
Independent review found a settled timeout was retaining the fixture's pending-child fence;
work and settlement failures are now distinct. Final issuer-bound observation exposed native
Boolean grouping materialization; supplier normalization now removes that false drift while
counterexamples protect meaningful differences.

### Deliberate choices and remaining receiving

**Implemented.** Named constrained strings preserve current ordering; native ordered enums
require separately declared rank. Signed microsecond time and authoritative IEEE bits remain.
Exact Decimal conversion remains only where plain-digit SQL formatting requires it. Read-row
encoders retain readback/independent comparison responsibilities. Native AST/module/issuer
witnesses remain necessary for v2's omitted meanings. No external emitter or second production
generator was selected.

The standalone tool is prepared with input identity
`a6b81b7f48b066b83fac5b7bc9ac0a75e822a0eced26a2467c0510b6f8936e80`, exact upstream sources,
feature selection and the pinned server's existing core/DiskANN compiler fixes. **Implemented
preparation limitation:** the supplier build succeeded with 13 core warnings (2 unreachable-code and
11 deprecated `fetch_update`) plus a `nix 0.31.3` future-incompatibility advisory. The supplier
warning target remains zero and is not met; no warning-free supplier qualification is claimed.

**Proposed receiving.** 72b owns retained-store transition, type-removal effects and activation;
72c–h consume the concrete handoffs above; 72i owns assembled journeys and scope-end
hygiene/governance/docs/native qualification. No configured store was reset or migrated.
Expected reductions in repeated interpretation/conversion are qualitative; no measured speedup
is claimed or required. Retain this completed companion while the remaining series consumes
its detailed decisions; 72i owns eventual retirement after enduring meaning has its owner.
