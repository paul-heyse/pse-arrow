---
title: Registry and receiving validation
status: done
date: 2026-10-20
adrs: [ADR-0202, ADR-0210]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [current-work, preparation, validation, performance, design]
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
---

# Registry and receiving validation

## Purpose and authority

This companion to [Plan 48](48-execution-performance-and-numerical-efficiency.md) removes
repeated immutable registry and validation preparation while preserving what a receiving
operation must establish now. It develops AF03/AF04 and the corrected R2 P03/P04 directions
from the [completed audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md)
and [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md).
The coordinator owns finding dispositions, inquiry state and operator rule decisions.
The targets began as **Proposed**; the execution checkpoint below distinguishes implemented
package scope from remaining inquiry and qualification. The operator accepted RC01–RC12 as written on
2026-10-20; the coordinator owns that record and decision/design routing. B05 adoption remains
conditional on I05/I06 and an effective RC11 contract; acceptance does not select the loader.

The source contracts remain [blueprint §4.1, §4.2 and §4.6](../authoritative_design/sections/schema-and-relations.md#section-4-6):
registry declarations are authoritative; generated descriptions are derived; physical layout,
value/domain validity and relational obligations are distinct properties. A declaration's
fingerprint describes content. It does not admit a newly supplied array or import an old
engine's function/configuration bindings.

## Current foundations and target

The audit examined commit `1ee81798a9b884f3054b685ee9629e2c8d124498`; this plan additionally
inspected current `pse-schema::{builder::native,fingerprint,resolved_contract}` and
`pse-relations::validate::{predicates,occurrences,prepared}`. The focused assessment reuses
the audit under Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5. It does not
re-review the entire schema system or establish measured startup dominance.

The registry already owns resolved contracts, generated expectations, derived implementation
memoization and semantic closure descriptions. Those are the natural owners for B01's
immutable sharing. `builder::native::BOOTSTRAP` is already a `LazyLock`; `materialize` forces
it before deciding whether a self-describing relation needs fallback. Deferring that force
is useful without redesigning custom/partial registry behavior. Fingerprints currently frame
canonical native descriptions in `fingerprint.rs`; caching or streaming that representation
preserves the encoding, while hashing member hashes would change the preimage. The latter
is outside this target.

`DomainEnvironment::current` rebuilds an enum-member map. Nested `occurrences::Node::prepare`
recurses through children and calls `field_local`, which constructs that environment again.
`field_local_in` and other environment-aware predicate operations already exist. B02 passes
one resolved environment through the actual preparation tree. Verified recorded domains
remain distinct from the current registry's domain population.

`ValidationContext` already shares a `ValidationOwner` with actual registry identity, planner,
immutable template cells and implementation caches. `ScopedContracts` belongs to the invocation
view. `AdmissionScope::same_invocation` compares owned identity independently of timestamps;
`same_bindings` compares the actual shared owner. B03 extends this separation and removes
demonstrated repetition. It does not introduce a generic trusted-pointer handle or treat
the scope as absent. Existing schema identity, `RelationContractHandle`, checked batches and
retained local contracts establish different facts and remain the vocabulary for callers.

The target has a narrow interaction: registry owners share immutable semantic descriptions;
validation preparation consumes one actual domain environment and actual receiving bindings;
the invocation binds and evaluates its scope-sensitive obligations. Callers can carry proven
facts through preserving operations, but fresh or unsafe/native construction and changed
interpretation retain their obligations. A new enum or nested field changes its declaration
and regenerated products. A new engine function or configuration changes receiving preparation,
without changing registry semantic identity or silently inheriting another owner's proof.

## Delivery sequence

B01 and B02 can proceed independently. B03 consumes the shared domain environment where
nested templates require it. B04 first compares improved runtime reuse, a generated static
product and an archive; usefulness and representability determine whether any additional
generation is justified. B05 waits for the smaller reuse corrections and
an actual consumer gap; it is not an assumed replacement for generic validation. The checked
artifact work in [48a](48a-source-checking-and-incremental-preparation.md) must reconstruct
current receiving shells through these owners. Mathematical/native execution in
[48c](48c-mathematical-products-and-compiled-execution.md) still performs current provider,
domain and original numerical assessment.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B01 — shared semantic descriptions and deferred bootstrap force | Existing immutable registry owner and canonical fingerprint contracts | Reuse descriptions, normalized SQL, expected closures and invariant indexes; force fallback only for actual missing intrinsic relations; preserve custom registries and exact fingerprint bytes, then remove repeated derivation. | done: implemented; targeted verification passed; selected assembled acceptance recorded in48h |
| B02 — shared DomainEnvironment | Existing current/recorded domain interpretation and environment-aware predicates | Resolve one domain environment per actual preparation owner/tree and pass it through nested predicates; verify nested/null/container behavior and retire repeated per-node map construction. | done: implemented; targeted verification passed; selected assembled acceptance recorded in48h |
| B03 — existing templates and current receiving | B02 for nested templates; I05 for any merged operation scope | Extend actual ValidationOwner/template and retained-contract paths; preserve engine/function/configuration and invocation-sensitive checks; prove same-binding reuse and foreign/scope refusal without a second trust hierarchy. | done: implemented; targeted verification passed; selected assembled acceptance recorded in48h |
| B04 — frozen static catalog and archive inquiry | B01 semantic description identity and improved runtime reuse; existing declaration/codegen ownership | Resolve I04 across runtime memoization, static generation and archives, then adopt only justified products; no additional generation is a valid result. Retain runtime-owned caches and general RegistryBuilder behavior. | done: ADR-0210 complete standard hydration implemented; full normal generation and focused tests passed; selected assembled acceptance recorded in48h |
| B05 — conditional typed loader | B02/B03 useful reuse, I05 actual scope, I06 loader decision and accepted RC11 routed into its owner | Only if justified, generate an authored-document path with the same nested/null/key/reference/SQL/invocation obligations, migrate its consumers and retire replaced validation work; otherwise retain the existing path. | done: one-family implementation regenerated; targeted verification passed; selected assembled acceptance recorded in48h |

### B01 — derive immutable meaning once under its registry

Given an immutable registry and requested semantic closure, return its existing complete
description/contract and index its declared invariants without repeatedly rebuilding canonical
semantic populations. Expected-contract capture and self-description consumers should use the
same registry-owned reusable product. SQL normalization remains deterministic for its consumed
interpretation; normalized source is not a substitute for binding to actual engine functions.
Share immutable completed products through existing derived implementation owners, preserving
typed construction failure, cycle detection and independent callers' completion behavior.

Move bootstrap forcing inside actual fallback selection. A complete production registry then
materializes under its own declarations without building another full bootstrap. A custom or
partial registry still obtains missing intrinsic descriptors from the same canonical fallback;
unknown intrinsic declarations or malformed self-description continue to fail explicitly.
Do not convert an existing error into an empty description or default contract.

Targeted schema tests compare registry and relation fingerprint bytes, expected closures and
materialized rows before/after immutable reuse, including custom registries, missing intrinsic
relations and changed reachable enum/reference meaning. Use independent expected byte fixtures
where framing is promised; a new hash of a new representation is not an independent oracle.
Exercise repeated calls and construction failure/cycle behavior where the shared memo changes.
After consumers move and tests pass, remove redundant capture/description paths and eager
bootstrap force, retaining the canonical fallback builder. No hash-of-hashes identity migration
is included.

### B02 — carry the domain interpretation through nested preparation

The operation receives the actual current registry or verified recorded contract, constructs
its resolved `DomainEnvironment` once at the appropriate immutable preparation owner, and
passes it to every nested field/occurrence predicate builder. Container traversal and local
SQL expression construction remain with their existing owners. An environment from recorded
history cannot expand a closed domain with today's extra member; a current registry change
cannot reuse an environment whose interpretation differs.

Migrate recursive `Node::prepare`, field-local preparation and any root predicate preparation
that otherwise rebuilds the same map. Reuse existing `_in` operations rather than duplicating
enum dispatch. Check dictionaries, structs, lists/maps, unions/tagged alternatives, slices
and null parents, including invalid child metadata and hidden invalid enum values. Existing
admission tests `masked_struct_payload_does_not_leak_invalid_enum_strings` and
`nested_metadata_is_checked_even_when_every_parent_value_is_null` supply revealing cases.
Add a current-versus-recorded domain difference and preparation-construction count at a small
nested depth. Delete per-node environment construction only after its full consumer migration;
independent preparation owners still construct their own current interpretation.

### B03 — reuse templates while establishing current eligibility

Preparing a relation or column receives actual registry/planner bindings, schema and operation
scope. Immutable context-owned predicates and templates survive a new invocation when their
bindings remain the same; Stable/ambient predicates bind and recheck in the current scope.
New arrays undergo their required structural/value checks even when the schema pointer is
unchanged. Retained checked input owners and property-preserving operations can reuse the
facts they actually establish, avoiding rescans caused solely by another wrapper.

Migrate consumers that construct redundant contexts/templates only after their real owner and
scope are known. Keep SQL planning on an actual engine context; a local planner must continue
to refuse predicates requiring unavailable runtime semantics. Construction, native planning,
poisoned shared state, changed configuration/functions and scope-sensitive checks retain typed
failure behavior. Successful local checks do not silently satisfy foreign-key/visible-reference
or publication obligations owned elsewhere.

**I05 asks whether putatively related validations are one real operation.** Trace representative
RunResult and authored-input admission consumers, identifying actual engine/function bindings,
scope creation and child lifetimes. Compare passing one enclosing `AdmissionScope` to legitimate
children with retaining separate scopes and only sharing immutable templates. Equal timestamps
are explicitly insufficient. Evidence includes an actual same-operation child and distinct
operations with equal timestamps, plus a changed receiving function/configuration case.
Stop when these callers have an explicit scope owner and the proposed sharing either preserves
current invocation semantics or is rejected. Adopt only demonstrated same-operation sharing;
no-adoption keeps separate scopes. Reopen when a consumer's operation boundary or ambient
predicate contract changes. Its live state and decision are recorded only at the coordinator.

Verify unchanged-owner template reuse, foreign owner/schema refusal, newly invalid values
under an equal schema, same/different operation identity and Stable/ambient reevaluation.
Move affected generated builders/views and runtime consumers as part of the same change;
delete redundant preparation wrappers/caches when replaced, preserving distinct trust facts.
Correctness closures continue explicit force-validation.

### B01–B03 execution checkpoint — 2026-10-20

The registry now memoizes canonical local semantic descriptions (including normalized SQL),
indexes invariant positions and retains compact declaration-bounded closures plus their original
framed identities. Memo slots cover declared singletons/profiles, the empty set and the complete
catalog; arbitrary caller combinations compose singleton closures without retained slots. Expected-contract capture reuses those products. Mutable interchange
witnesses remain independent values; a full cloned witness is not retained for every root.
Native self-description requests the existing bootstrap only for a missing intrinsic declaration.
No fingerprint framing, catalog declaration or generated schema product changed in this scope.

`ValidationOwner` now resolves the current domain environment once and passes it through
recursive occurrence preparation. The replaced registry-taking `field_local` wrapper was
removed. The existing template index has one retained schema-pointer hint before its exact
structural lookup; it retains no batch validity. A single `ObligationTemplates::bind_required`
call lazily snapshots its supplied planner once for all selected local relation checks, giving
those actual children one explicit scope. Separate calls remain separate invocations.

The I05 consumer trace found existing scope ownership in `RunResult::result_validation`
and its projection owner, runtime activity admission, and nested native execution. Resetting
result projections or starting a distinct activity retains its current fresh-scope behavior.
No cross-result/activity scope fusion was adopted. The coordinator owns the inquiry's live
state and disposition. Targeted compile and behavior checks passed as recorded in the partial
Outcome below. B04's bounded inquiry subsequently selected runtime memoization without an additional product;
B05's one-family source checkpoint below records its later conditional implementation.

### B04 — decide whether runtime reuse needs a static or archived product

Start from B01's improved runtime memoization rather than assuming a frozen product is needed.
If I04 justifies static generation, derive the selected frozen descriptors/contracts and
normalized declarations from the existing production catalog declaration. Such a product is
an immutable projection, not another catalog. General `RegistryBuilder`, extension/custom
registries and current validation remain supported. Runtime locks, implementation caches and
actual native bindings are constructed by their runtime owners, not serialized into an
imagined static image. Any selected generator change regenerates affected outputs through
`just codegen`; generated paths are never hand-edited.

**I04 asks whether a static catalog or archive adds value after B01.** Compare improved runtime
memoization with no additional generation, generated Rust/static descriptors, and a serialized
image or archive approach such as rkyv/bincode where actually representable. Establish actual
`SchemaRef`, map, metadata, pointer and synchronization reconstruction requirements, source/image
validation and retained memory. The audit does not establish that today's Registry is an
automatically zero-copy archive. Use a bounded catalog slice and then the actual production
shape to settle usefulness, representability and total startup/retention work before adopting
an additional product; do not select a format from its label. Stop when those tradeoffs support
runtime reuse alone, a static generated product or an archive. No additional generation is a
complete no-adoption outcome. Reopen for a concrete deployment/startup workload or catalog/library
change invalidating the deciding usefulness/representability premise. Coordinator state records
the choice.

For a selected archive, corrupted/incompatible images fail safely or rebuild from declarations
under the existing owner; no stale registry identity is admitted. Verify exact descriptions/
fingerprints and custom-registry behavior for every selected option, changed catalog regeneration
if generation is adopted, and archive failures if an image is adopted. Remove replaced runtime
derivation or image paths when the selected product's consumers migrate. Runtime reuse alone
retains B01's owner without adding another catalog representation. Keep declaration-derived
rebuild as authority rather than a compatibility implementation.

### B04 bounded comparison — 2026-10-20

**Measured, bounded decision probe:**
`just unit-package pse-schema registry_preparation_and_frozen_witness_reconstruction_probe --execution-effects native-local --success-output immediate`
passed 1/1 library tests (78 unselected), zero failures against the zero target. Conditions
were this checkout's dev test profile with explicit `pse-schema/force-validate`, five independent
`catalog::assemble` samples and five reconstructions for each image shape. Complete catalog
construction had a 288,159 us median. The catalog has 168 relations, 232 enums and 225 invariants.

| Declaration-derived projection | Canonical image bytes | Serialization us | Parse median us | Parse plus semantic/native equivalence checks median us |
| --- | ---: | ---: | ---: | ---: |
| One dimensions relation | 1,564 | 52 | 4 | 14 |
| Full semantic contract and exact native schemas | 4,654,427 | 15,853 | 7,776 | 30,799 |

The serde image preserved semantic identities and every exact native schema serialization,
including nested metadata and dictionary ordering. Image bytes are serialized size, not measured
resident memory. These parse timings do not construct a complete `Registry`: the image omits
its declaration/static-string ownership, relation/enum/obligation indexes, self-description
batches, resolved contract graph and live lock/proof/cache lifetimes. `Registry`, `RelationSpec`
and `FieldContract` are not presently archive/serde-derived live images; enabling serde on Arrow
schemas does not change that. An archive requires an explicit image and owned reconstruction.
An rkyv remote derive/wrapper or bincode decoder is a possible implementation tool, not a proof
that this runtime owner is already directly representable or zero copy. The comparison introduced
no new serialization dependency or production archive machinery.

The coordinator selected **runtime memoization with no additional static/archive generation**
for this bounded inquiry. Existing generated `ExpectedContract` entries freeze independent
identities; they do not supply the complete current meaning required for changed/custom
registries. Another generated Rust product or archive could avoid some cold derivation, but
it would need a complete single-authority restoration route and its additional index/cache/
self-description lifetimes. The probe establishes no qualified complete startup win for that
route. This choice claims neither a universal lack of archive benefit nor a memory improvement.
Reopen if full startup attribution remains registry-dominant after preparation corrections,
or a complete single-authority image/reconstruction demonstrates materially better complete
behavior. The coordinator owns I04's live decision state.

### B04 reopened target — complete standard hydration

The operator's later deployment-first direction reopens I04: absence of a qualified complete
startup comparison is not a reason to reject a sound removal of deterministic cold work.
The coordinator selected ADR-0210 before dependent source changes. The existing generator
will independently source-assemble the standard catalog, then emit complete resolved products
and exact self-description IPC. Only the standard process-registry initializer will hydrate
these products into fresh runtime maps, owner identity, locks and empty receiving-proof caches.
Custom/changed registries retain their actual source builder; source generation must never
consume the generated hydration path. The first generated payload is published before the
initializer changes, without a hand-written generated seed.

The earlier parse measurements remain projection-only observations. The new complete
source/hydrated meaning, receiving and failure controls now pass, with separate complete-operation
measurements below. The bounded target adds no archive/proof platform
and does not change fingerprint framing or the obligation to validate new arrays.

The complete generator now emits canonical native JSON slices and self-description IPC;
the normal Rust-contract publication completed. Its first activated compile exposed a
development-cost defect: one 4.66 MB hydration expression exhausted its 40 GiB scope before
tests started. This was an actual scope OOM, not a scientific failure or an external source
cancellation. A temporary source initializer and test-only generated module allowed the
generator to bootstrap finite declaration and family helpers. That chunked payload was
normally published and activated, and the schema/codegen all-target compile check passed
without warnings. Its optimized unit build was then deliberately cancelled and drained
before Nextest started after compiler memory peaked at 11.7 GiB. One non-inlined slice
constructor removes length-specific semantic-set iterators, but a subsequent optimized build
still grew and was deliberately cancelled before tests. Source inspection found 512 remaining
owned-array collection sites (489 empty and five of length 168), including large fallible
obligation/schema/description arrays. The next correction outlines each relation's obligations
and builds fallible families by sequential pushes into a reserved Vec, followed by the existing
owned bulk collection. Contract and semantic subfamilies have separate typed helpers. This
avoids large partial-array cleanup graphs without changing data, ordering or cloning products.
Normal publication of that correction is complete, the temporary bootstrap is removed, and
both standard initializers hydrate the generated products. All five complete-product controls
pass. Raising the memory cap was not the repair.

IPC metadata determinism also follows the actual pinned producer boundary: Arrow IPC
59.3.0's schema and nested-field encoders both call `metadata_to_fb`, which sorts map keys
before emitting them. Our schema producer supplies five-key unordered schema metadata and
nested field metadata, and self-description rows are sorted by declared primary keys.
The independent source-generation equality control exercises the complete source, JSON pool
and IPC byte products; it supplements this source contract rather than establishing it alone.

### B05 — conditionally specialize authored-document receiving

**I06 asks whether a typed loader removes material work left after B02/B03.** Compare the
improved existing validator with a generated typed authored-document path. Identify which
source shapes and consumers benefit, which checks the construction can establish, and which
actual engine-bound SQL, relational and invocation obligations remain. Include nested nulls,
keys, references, extensions and unknown fields, plus custom registries and unsafe/native inputs.
Use I05's actual operation scope rather than inventing one scope per result. Stop when one
bounded document family has a complete equivalent obligation map and a credible removed-work
case, or when existing template reuse makes another path unjustified. Adopt only the justified
scope after the accepted RC11 contract is effective; a no-adoption outcome retains the existing
path and closes the inquiry without manufacturing a second validator. Reopen for a demonstrated remaining loader
cost or a newly supported document family with different receiving needs.

Any adopted loader is generated from the same registry/document contracts and uses shared
predicate/relational owners. It cannot hand-copy rules, bypass SQL checks by claiming typed
construction or transfer a producer's eligibility. Change consumers and generated adapters
together, with targeted valid/invalid case equivalence and independent expected refusals for
the revealing cases. Delete replaced work for those consumers as soon as proven; support for
other legitimate external/generic inputs is not legacy to remove. If the inquiry finds a
contract difference, return it to the coordinator before choosing a new rule or migration.

### B05 source checkpoint — 2026-10-20

The coordinator selected a bounded modeling-row target after publishing proposed ADR-0202 and
blueprint §14.4. Generated strict documents already exist; no second YAML decoder was added.
The actual removed work is `project_documents` materializing a full modeling-declaration batch
only to discard it after local checks. New `typed_local.rs` derives remaining finite/range/
collection/tagged-arm traversal from that one declaration and reuses `LocalFieldAdmission`.
Exact generated contract admission and actual receiving-template preparation remain required.
Private mutable `ModelingRows` clones still revoke evidence; explicit Arrow export and raw
native input admission retain their actual array/metadata checks.

Receiving preparation alone proved insufficient: pinned DataFusion 55.1.0
`SessionState::create_physical_expr` applies analyzer function rewrites even to prebound
expressions. The existing planner now defaults to denying typed representation eligibility.
`NativeValidation` grants it only when its actual immutable state has no function rewrites.
The current one-family expressions otherwise use primitive operators and prebound
`get_field`/`array_length`/`encode`/`octet_length` kernels, whose implementation consumes no
configuration semantics for this shape. SQL expression/type/query-planner hooks are absent
from this already-bound physical-expression route, which selects the default physical
planning context. An unknown/custom planner retains native evaluation. Unsupported future
facets, extensions, collection uniqueness and row SQL fail generation instead of being omitted.
This eligibility is associated with the existing actual planner owner; it is no transferable
proof token. Root regeneration completed, and the focused checks below passed. Generation also
refuses a fixed-array declaration whose cardinality excludes its actual width, and terminal
extensions cannot conceal unsupported nested facets. Typed/native differential controls include
malformed outer and named nested selected arms, nonfinite nested values, bounded integers,
payload-free alternatives, empty collections and unchanged signed-zero bits. Explicit Arrow
export still evaluates the new arrays; an unqualified planner takes ordinary native evaluation.
The additional actual bound-session-factory eligibility/adoption control passed in the
subsequent seven-test runtime/admission focused run recorded below.

## Scoped completeness handoff — H01

This handoff maps every recommendation-coverage row linked to B01–B05 to its actual
producer and receiving consumer. It records focused package evidence, without closing
Plan 48 or duplicating the coordinator's inquiry/disposition authority.

| Linked coverage row | Actual producer → consumer | Qualified scope or bounded inquiry outcome |
| --- | --- | --- |
| P03 description, canonical-SQL and invariant memoization | `fingerprint.rs` registry-owned descriptions/indexes → semantic contract capture, `resolved_contract.rs` and `product.rs` support closure | **Tested:** the 66-test schema/relations selection below covers unchanged meaning, failures/cycles and independent callers. |
| P03 member-hash closure digest plus unchanged fingerprints | Declaration-bounded `SemanticProducts` → existing complete-description fingerprint framing | **Tested:** frozen bytes, complete canonical preimages and 64 arbitrary root combinations; no hash-of-hashes migration or unbounded subset retention. |
| P03 lazy BOOTSTRAP | `builder/native.rs::materialize` fallback → self-description of complete/custom/partial registries | **Tested:** actual missing-intrinsic fallback remains; eager forcing was removed, not the fallback authority. |
| P03 generated frozen production catalog; P03 rkyv/bincode/zero-copy image | Independent `catalog::assemble` → generated complete resolved products/native IPC → standard initializer with fresh runtime owners | **Tested and Measured:** ADR-0210 complete hydration passes five controls including independent complete meaning, current receiving, empty custom registries and bad IPC. Five fresh-owner construction/drop samples are below. No extra archive/proof platform; custom source assembly remains actual authority. |
| P04 shared DomainEnvironment | Current `ValidationOwner` → recursive `occurrences::Node::prepare` and environment-aware predicates | **Tested:** one current interpretation per actual owner and current/recorded nested-domain differences; per-node construction wrapper removed. |
| P04 prepared templates; P04 new trusted pointer handle | Existing owner/template cells and exact schema hint → current relation/column preparation | **Tested:** existing owners reused; foreign owner/schema and changed bindings refuse. No new trust handle was introduced. |
| P04 full validation only at external ingress | Current checked input/retained local contract → newly supplied arrays and current receiving bindings | **Tested:** invalid values under an unchanged schema still fail. No ingress-only validation rule was adopted. |
| P04 one scope per RunResult | `ObligationTemplates::bind_required` actual call → its selected child validations; existing RunResult/activity scope owners remain | **Tested:** one call shares one lazy snapshot; separate invocations remain distinct despite equal timestamps. I05 retained separate result/activity operations rather than fusing them. |
| P04 typed authored-document loader | `rust/typed_local.rs` generated exact declaration traversal plus actual planner eligibility → `document/load.rs::project_documents` and existing `LocalFieldAdmission` | **Tested:** generator guards, actual rewrite denial, four loader controls and typed/native negatives. Only qualified modeling rows avoid the discarded transient batch; raw/export/custom planner paths retain native checks. |
| P18 interned IDs and streaming hashing (B01 share) | Registry descriptions and original framing → semantic identity/expected contracts | **Tested:** B01 reuses derived meaning without changing ID/framing authority. A01/A04 own the separate source-symbol and typed masking portions of this shared row. |

B01–B03/B05 have focused functional evidence and I05 retains existing operation scopes.
B04's subsequent ADR-0210 target now has normal Rust-contract publication, actual initializer
migration and complete equivalence/receiving/failure tests. The subsequent full normal
generation completed all five schema targets and the native/Python binding stages.
Assembled engine/Python receiving journeys, final codegen drift/static checks and cross-plan
qualification remain with H02/H03. The B05 default-deny planner fallback and generic/native
array routes are required support, not legacy paths awaiting deletion.

## Verification and handoff

**Proposed verification:** use touched-package compile checks and targeted
`just unit-package pse-schema <filter>` / `just unit-package pse-relations <filter>` tests
for descriptions, bootstrap, domain interpretation and receiving behavior. Choose the existing
integration-test target where admission cases live; the executor should not label a unit filter
that selects no tests as coverage. Regenerate declaration-derived outputs when their generator
changes. Actual Arrow-consuming correctness routes explicitly enable force-validation.
At series scope end the coordinator selects relevant engine/component/Python receiving journeys,
codegen drift and static/manual checks once, with exact command/mode/scope and zero failures
as the target. Startup/retention benefits need matched measurements of the chosen scope;
registry source inspection alone supplies no quantitative claim or new SLA.

The original plan-writing step ran no tests, builds, services, formatters or environment proofs.
Receiving correctness and reusable immutable preparation are separate acceptance claims;
completion of B01/B02 does not select a loader or authorize B05 implementation. Enduring changes move to existing
schema/receiving owners through the coordinator's decision/design and closeout route.

## Outcome (recorded after implementation)

### What was built

**Implemented and Tested, completed adopted scope (2026-10-20):** the package table records
all delivery/inquiry outcomes. [Plan48h](48h-qualification-and-design-publication.md#final-qualification-accounting)
records completed selected assembled acceptance and scope-end checks. The individual
observations below retain their original conditions and partial-run boundaries; they do
not assert a universal speedup or extend unsupported capabilities. There is no outstanding
authorized package work in this companion.

**Implemented and Tested (2026-10-20):** B01–B03 runtime changes are described in the
execution checkpoint. `just check-package pse-schema -p pse-relations` passed in dev
all-targets mode with zero errors/warnings against the zero target. The scoped
`just unit-package pse-schema '(package(pse-schema) & (test(immutable_reuse_unit) | test(semantic_tests) | test(resolved_contract) | test(implementation_cache))) | (package(pse-relations) & test(validate))' -p pse-relations --execution-effects native-local`
passed 66/66 library tests (84 unselected), Nextest default profile, zero failures against
the zero target. Both commands used the repository environment and verified explicit
`pse-schema/force-validate` / `pse-relations/force-validate` closure activation.

Coverage includes frozen catalog/nonempty contract fingerprint bytes, unchanged complete
semantic preimages, 64 root subsets with bounded memo retention, independent completion,
construction failures/cycles, actual fallback selection, current/recorded nested enum
interpretations, one environment per actual owner, new invalid values under an unchanged
schema, actual native function owners and same/distinct invocation identities. No whole
integration suite, performance comparison, Python journey or full-plan qualification was run.
**Implemented and Tested — B04:** ADR-0210 complete standard hydration now replaces source
assembly only in the standard process initializer. Independent source assembly and custom
builders retain declaration authority; runtime owners and receiving-proof caches are fresh.
`just unit-package pse-codegen 'test(registry::tests)' --features pse-codegen/force-validate
--execution-effects native-local --success-output immediate` passed 2/2 tests (25 unselected)
in 1.231 seconds, with zero failures and six then-unreferenced pre-activation helpers.
Normal `just codegen-rust-contracts` publication passed. After activation,
`just check-package pse-schema -p pse-codegen` passed dev all-target compilation in 3.64 seconds
with zero errors/warnings against the zero target. `just unit-package pse-schema
 'test(builder::products::tests)' --execution-effects native-local --success-output immediate`
passed 5/5 tests (79 unselected) in 1.712 seconds, with zero failures/errors/warnings against the
zero target and explicit Arrow force-validation. This includes complete independent-source
meaning/encoding and current receiving, fresh owners, empty custom registries preserving
intrinsic scalars, and corrupt/missing/extra native batch refusal.

**Measured — B04:** five fresh source/generated Registry owners in the current development
unit binary reported `(source construction, hydration, source drop, hydrated drop)` microseconds:
`[(316225,30863,14553,9705),(301866,22583,12352,8358),(293720,21972,10552,8174),
(287596,21700,10589,8039),(287011,21679,10311,7969)]`. Median construction was 293720 versus
21972 microseconds; construction plus drop medians were 304272 versus 30146 microseconds
(a source/hydration elapsed-time ratio of about 10.09 in this fixture). Every owner weak reference expired
on drop. This is a same-process fresh-owner comparison, not whole-process startup, RSS,
allocator-only cost or a scientific workload speed claim. Published products contain
4041187 bytes of Rust, 15730425 bytes of canonical JSON and 15 IPC files totaling 3327992
bytes; file bytes are not resident memory. The subsequent full normal generation completed
all five targets and binding stages. Assembled receiving, final drift/static checks and
whole-plan qualification remain with the coordinator.

B05 one-family specialization is **Implemented and Tested** after root `just codegen`.
`just check-package pse-codegen -p pse-relations -p pse-engine -p pse-admission -p pse-backend-native -p pse-runtime`
passed dev all-target checks with zero errors/warnings against the zero target.
`just unit-package pse-codegen '(package(pse-codegen) & test(typed_local::tests)) | (package(pse-engine) & test(typed_modeling_eligibility)) | (package(pse-admission) & test(typed_modeling)) | (package(pse-backend-native) & test(transport::tests::quality_recovery)) | (package(pse-columnar) & test(flight::tests))' -p pse-engine -p pse-admission -p pse-backend-native -p pse-columnar --execution-effects native-local`
passed 15/15 tests (600 unselected) in the Nextest default profile, zero failures against the
zero target, with explicit force-validation closure activation. This selected B05's two generator
guards, actual changed-function/configuration eligibility control and three loader tests, plus
the separately assigned F09 recovery and shared-flight controls. Two earlier preparations failed
on concurrently changed default-mode root-progress snapshot and flight result-move source;
their owners repaired those errors before the clean reruns.
`just unit-package pse-runtime '(package(pse-runtime) & (test(frontier_tests) | test(occurrence_execution_tests) | test(preparation_key_excludes_only_output) | test(presentation_only_successive_admission))) | (package(pse-admission) & test(typed_modeling))' -p pse-admission --execution-effects native-local`
subsequently passed 7/7 default-profile tests (880 unselected), zero failures against the
zero target, including all four typed-modeling loader controls. This run emitted one
adjacent mathematical-owner dead-code warning, reported to its owner for correction.
No B05 total-performance benefit
or integrated qualification is claimed. Current findings, inquiries and rule decisions stay with the coordinator; this partial
outcome closes no series exit.

### A mistake made and corrected

Initial memoization retained arbitrary caller-selected root sets, risking combinatorial
process-lifetime growth. Review caught that error before acceptance. The corrected immutable
slot inventory is declaration-bounded; arbitrary combinations reuse singleton closures without
being retained. The passing varying-subset test exercises this limit along with unchanged identities.

The first generated hydration constructor was too large for the development compiler and
exhausted its 40 GiB scope. Splitting functions alone did not remove the large fallible
array cleanup graphs; two later optimized builds were cancelled and drained before tests.
Sequential Vec initialization and bounded typed helpers removed those arrays. The final
all-target check and all five complete hydration controls pass without increasing the cap.

### Deviations from the plan, deliberate

The memo retains shared descriptions and compact closure/identity products instead of
retaining a full cloned mutable interchange witness for every root. This preserves the
existing witness API and framing while avoiding duplicated closure value trees. Existing
result/activity scope ownership was retained; only one actual obligation-binding call's
local children were grouped.
