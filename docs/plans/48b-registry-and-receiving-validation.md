---
title: Registry and receiving validation
status: draft
date: 2026-10-20
adrs: []
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
All changes described here are **Proposed**. The operator accepted RC01–RC12 as written on
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
| B01 — shared semantic descriptions and deferred bootstrap force | Existing immutable registry owner and canonical fingerprint contracts | Reuse descriptions, normalized SQL, expected closures and invariant indexes; force fallback only for actual missing intrinsic relations; preserve custom registries and exact fingerprint bytes, then remove repeated derivation. | open |
| B02 — shared DomainEnvironment | Existing current/recorded domain interpretation and environment-aware predicates | Resolve one domain environment per actual preparation owner/tree and pass it through nested predicates; verify nested/null/container behavior and retire repeated per-node map construction. | open |
| B03 — existing templates and current receiving | B02 for nested templates; I05 for any merged operation scope | Extend actual ValidationOwner/template and retained-contract paths; preserve engine/function/configuration and invocation-sensitive checks; prove same-binding reuse and foreign/scope refusal without a second trust hierarchy. | open |
| B04 — frozen static catalog and archive inquiry | B01 semantic description identity and improved runtime reuse; existing declaration/codegen ownership | Resolve I04 across runtime memoization, static generation and archives, then adopt only justified products; no additional generation is a valid result. Retain runtime-owned caches and general RegistryBuilder behavior. | open |
| B05 — conditional typed loader | B02/B03 useful reuse, I05 actual scope, I06 loader decision and accepted RC11 routed into its owner | Only if justified, generate an authored-document path with the same nested/null/key/reference/SQL/invocation obligations, migrate its consumers and retire replaced validation work; otherwise retain the existing path. | open |

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

This plan-writing task ran no tests, builds, services, formatters or environment proofs.
Receiving correctness and reusable immutable preparation are separate acceptance claims;
completion of B01/B02 does not select a loader or authorize B05 implementation. Enduring changes move to existing
schema/receiving owners through the coordinator's decision/design and closeout route.

## Outcome (recorded after implementation)

### What was built

**Proposed; not implemented.** This companion specifies work and bounded alternatives.
No registry generation, validation, scope or loader behavior changed. What was built, a
mistake made and corrected, deliberate deviations and actual verification remain to be
recorded after implementation; current findings, inquiries and rule decisions stay with
the coordinator.

### A mistake made and corrected

**Proposed; not implemented.** No implementation mistake or correction is recorded yet.
Record the actual lesson when the work is performed.

### Deviations from the plan, deliberate

**Proposed; not implemented.** No implementation deviation is recorded yet. Record deliberate
changes and their consequences when the work is performed.
