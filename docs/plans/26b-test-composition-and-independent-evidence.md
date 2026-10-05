---
title: "26b: Test composition and independent evidence"
status: draft
date: 2026-10-05
adrs: [ADR-0051, ADR-0092, ADR-0119]
review_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md#revealing-changes"]
---

# 26b: Test composition and independent evidence

## Purpose and assessed foundations

Replace test-owned package reconstruction and repeated per-instance verification with
production selection, independent mechanism expectations and useful boundary tests. The
[coordinator](26-testing-architecture.md) owns F01/F02/F04/F07/F09 dispositions and shared
decisions. This companion owns B1–B5. All corrections and acceptance are **Proposed**.

The intact loader, `OwnedDocumentSet`, `ModelingFixtureSelection::Selected` and
`ModelingPackage::declared_execution` already supply the relevant production operations.
The existing selected-fixture control distinguishes unknown/non-test identities, selected
execution, unselected failure and package coverage. Reuse it rather than reconstructing policy.

`pse-schema::builder::integrity` projects declaration-owned `RelationObligations` into native
queries. This is one mechanism per family, not independently authored SQL per relation.
`InvariantKind` does not identify origin: ordinal-derived queries use a domain kind and an
authored query can use a generic kind. Name prefixes are equally unsuitable. Add a typed
derivation descriptor where the producer knows the meaning.

The inspected production emitter supplies generic integrity rules only. Historical fixture
branches for entity/field agreement, package resolution and parent cycles are not current
catalog rules. Blueprint §14.2 places shared predicates at source/physical admission. Preserve
those boundary controls; do not manufacture a custom registry to retain old fixture code.

`xtask::codegen::run` is the strongest emitted-tree freshness owner. Owner-local generation,
native ABI/link, raw/checked field admission, scientific references and resource controls still
establish distinct claims. Their existence does not justify the weaker overlapping tree test,
empty bootstrap tests or upstream-only probes in product qualification.

## Selected designs

### Intact authored workloads

Load actual package documents through `load_package_documents_owned` and share their immutable
owned input at the scope justified by the journey. Select actual fixture/declaration identities
through production operations. A selected execution does not claim package-wide conformance.

Delete the `reference_documents` parse/filter/descendant/render machinery and its hard-coded
test-ID whitelist, plus dynamic optimization and complementarity restoration helpers. Shared
support locates inputs and supplies explicit budgets; it does not author a reduced package.
Intentional malformed documents or authored mutations remain valid tests where they attack
admission or a particular declaration. Do not confuse those inputs with workload selection.

Keep bounded engine support in `pse-testkit` and workflow runtime construction at their existing
dependency levels. Do not make every query/codec test initialize a runtime, package or store.

### Producer-owned invariant origin

Add a native typed descriptor with `AuthoredQuery` versus `GeneratedIntegrity(binding)`.
Bindings identify primary-key, named unique-key, table-reference, nested reference-occurrence
and ordinal-occurrence derivation. Resolve them against existing relation declarations and
obligation paths instead of copying their column/type definitions.

The generic producer alone can construct generated provenance. Ordinary public invariant
declaration defaults to authored origin and has no setter that blesses arbitrary SQL as a
generated rule. Expose immutable registry access for consumers; preserve repeated derivation
idempotency and conflicting manual declaration rejection. A generic-looking name or kind on
a genuinely different authored query remains authored.

Keep this descriptor outside durable metadata and fingerprint projections. Current invariant
specification/declaration types have no Serde boundary; fingerprint, recorded descriptors,
native `schema_invariants` rows and generated Python fields explicitly project existing
semantic fields. Adding native derivation identity does not require changing those fields,
schema versions or recorded interpretation. Document it in coordinator P0's decision without
inventing a new metadata/hash migration. Supply a native read-only descriptor/accessor for the
truthful distinction between generated derivation and authored semantics. No durable/Python
projection, extra reporting surface or separate rule-inspection feature is required.

### Three complementary verification responsibilities

1. **Mechanism value semantics:** independently specify adversarial cases over small synthetic
   declarations and execute the actual generated query. Exercise each consequential branch;
   avoid a Cartesian product and a case per ordinary production relation.
2. **Actual catalog binding/projection:** reuse production binding/compilation to check every
   real query's exact inputs, output key types and correspondence to the bound declaration.
   A new existing-family instance inherits this mechanical check.
3. **Actual enforcement and authored semantics:** exercise typed findings, selected required
   obligations and real candidate admission. Genuinely authored predicates require their own
   independent expectations rather than borrowed generic-family coverage.

| Mechanism or variation | Independent discriminating expectations |
|---|---|
| Primary keys | Singleton zero/one/multiple rows; scalar/composite duplicates versus distinct tuples; exact offending keys |
| Unique keys | Distinct primary keys sharing a unique tuple; composite correlation/order; partial/full absence; repeated offending-row deduplication |
| Table references | Scalar/composite pair correlation, self-reference, missing/empty targets, nullable component visibility |
| Nested references | Struct/list/map correlation, nested collections, empty/null parents, hidden child storage, active tagged/union arms |
| Representation lowering | Supported list variants, maps, dictionaries, run-end encoding and unions; collision-safe projection preserving root keys and occurrence values |
| Ordinals | Negative, zero, last-valid, equal-to-count and empty-target cases; scalar/nested occurrence visibility |

Reuse existing obligation/nested-value boundary tests where they establish the same risk.
Native lowering coverage alone does not prove generated SQL semantics: retain a discriminating
SQL case for each consequential branch not already challenged. Expected keys/counts are
specified independently of the query; do not call production evaluation to obtain an oracle.

Use a narrow test-local raw `MemTable`/query executor for deliberately hostile witnesses that
production value admission would reject before the query runs. Retain the useful essence of
the current fixture executor, without YAML discovery or a general fixture language. Do not
add unchecked production constructors or fabricate a `FieldCheckedBatch`. Separately run the
typed production invariant path and actual candidate admission for field-admissible PK/FK
violations, exact required selection and diagnostic identity.
Nested reference presence enforcement remains distinct from resolution SQL: hostile partially
present values can challenge raw query semantics, while `AllOrNone` expectations belong to the
actual value-admission boundary.

Strengthen the existing catalog binding control using production `compile_individual` key-type
projection and exact required-ID checking. Preserve singular/batched binding tests where the
consumed APIs differ; do not keep overlapping wrappers proving only the same key names.
Add one synthetic authored-query case with a generic kind/spelling to establish origin
discrimination and a separately specified semantic predicate. There are no current custom
production SQL cases established by the inspected emitter; future authored rules supply their
own oracle at introduction rather than reviving stale generator branches.

After this coverage is working, delete standard valid/violating invariant YAML, dynamic
per-declaration query cases, directory coverage, standard fixture generation and corresponding
freshness recipes. Moving every pair into runtime discovery would preserve repeated work and
is rejected. Do not retain all pairs as historical evidence.

### Freshness and retirement

Keep the nonmutating emitted-tree comparison in `xtask::codegen::run`: required outputs,
missing/extra/untracked detection, current target partitioning, Python candidate validation,
query preparation and applicable native binding generation. Remove the weaker governance
`codegen_regeneration` test and selections. Retain focused generator semantics/determinism,
compiled codecs and transformations where their failures differ. Generation remains part of
declaration implementation; generated files are never hand edited.

Delete the three `phase0_placeholder` cases. Remove the empty structural test crate and its
workspace/feature/dependency edges through coordinator P0. Delete proven-unused
`tests/support/physical_source.rs`. Remove upstream-only `math_composition.rs`, its exclusive
feature/dependency edges and any exclusively consumed PC-SAFT fixture after scoped caller
confirmation. Keep actual ABI/link/native-profile controls and useful production derivative/
assembly regressions; add new adapter tests only for a demonstrated gap.

Consolidate canonicalization and transport coverage by distinct framing, null-mask, slice,
dictionary, signed-zero, visible-change and reservation risks. Do not port tests of a deleted
mechanism or flatten independent boundary risks because a schema is shared. For expensive
journeys, a small set proving binding/reuse/refusal is the ordinary control; scale remains an
explicit performance/scientific workload where scale itself is the claim.

Retire IDAES enumeration/name compatibility-only cases and their authority obligation by
default under the hard pivot. Keep independently useful scientific comparisons under matching
conditions/tolerances. Do not preserve a name because blueprint §6.14 historically listed it;
an explicitly selected current public naming contract would be a material new requirement,
recorded at its owner before retaining its test.

## Packages

| Package | Prerequisites and delivered capability | Replacement/deletion and acceptance | Status |
|---|---|---|---|
| <a id="b1"></a>B1 Intact selection | Existing owned loader/production selectors; migrate all affected journeys | Delete package surgery/restore. An unselected declaration and descendants remain loaded but do not execute; unknown/non-test selection is refused | planned |
| B2 Native derivation identity | Production declaration/obligation producer; private generated descriptor and immutable registry access | Remove consumer prefix/kind inference. Repeated derivation is idempotent and manual masquerade/collision fails | planned |
| <a id="b3"></a>B3 Independent mechanisms and enforcement | B2 actual origin; existing native binding and narrow raw-query seam | Establish variations/catalog/admission/custom-query controls, then delete per-instance YAML/discovery/generation and stale semantic branches | planned |
| <a id="b4"></a>B4 Freshness consolidation | P0 superseding ADR-0051; B3 before retiring invariant artifact checks | Retain complete emitted-tree equivalence; delete weaker test/recipe selections; keep distinct semantics | planned |
| B5 Purposeful retirement | P0 crate/policy decisions; scoped consumer confirmation and retained distinct risks | Remove placeholders, structural shell, unused support, upstream-only probes and compatibility-only cases/edges | planned |

B1 and B2 can start independently. B3 depends on real B2 APIs, not an agreed name alone.
Ordinary unused-support retirement can proceed independently; no shared test files or manifests
have multiple writers. `justfile` and final execution configuration migrate with 26c C3.
Removal of declaration fixture generators includes their tooling, documentation and recipe
consumers; leaving an unused generator is not completion.

## Verification

Acceptance is **Proposed**. Use `just check-package` for touched packages, targeted
`just unit-package pse-schema <filter>` and `just unit-package pse-rules <filter>` for new
origin/mechanism/binding controls, and relevant engine/relations owner controls. Existing useful
controls include `a_conflicting_manual_integrity_projection_cannot_replace_the_declaration`,
`every_declared_query_binds_against_its_exact_native_inputs`,
`native_duplicate_keys_produce_one_typed_finding`, `kernel_conformance_runs_only_selected_fixtures`
and `source_and_physical_boundaries_refuse_the_same_package_closure_and_cycle`.

Mechanism cases run actual generated predicates with independent expected keys. Their retained
expectations must reject wrong null policy, pair mapping, visibility/traversal, ordinal boundary
or diagnostic projection. Use revealing negative controls; no permanent mutation-testing
framework or coverage registry is required. Catalog binding alone is insufficient evidence.

Demonstrate one ordinary synthetic relation addition using an established family inherits
catalog binding without new witness files or bespoke semantic execution. A new authored
predicate still gets its own oracle. Keep raw hostile, local certificate and actual admission
controls separate where each can expose a different defect.

For generation changes run `just codegen`; final freshness/governance and integrated reference
journeys belong to coordinator Q1. No scientific tests or native campaigns were run during
authoring. No numerical benefit is labelled **Measured** here.

## Outcome (recorded after implementation)

### What was built

Not implemented; populate from actual landed behavior and scoped evidence.

### A mistake made and corrected

Not yet applicable; record an actual correction.

### Deviations from the plan, deliberate

None recorded.
