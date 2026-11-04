---
title: "Plan64i: Engineering contracts and validation"
status: in-progress
date: 2026-11-03
adrs: [ADR-0239]
review_sources: [docs/plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
---

# Engineering contracts and validation

## Purpose, authority and assessed baseline

**Proposed:** derive inspectable engineering projections and local constraints from their
actual semantic owners, use them for typed exact-parent authoring and selected readback,
and remove superseded mappings/checks throughout the affected native consumers. This
companion owns review-local I01/I02 and I06's producer selection/materialization work.
The [coordinator](64-process-simulator-target.md) alone owns review dispositions, accepted
rule changes, integrated sequencing and completion. [64j](64j-engineering-services-and-clients.md)
and [64k](64k-prepared-scientific-workers.md) consume these contracts; [64h](64h-reference-coverage-and-qualification.md) retains I05's
independent reference construction, semantic compatibility and assembled acceptance.

The maintainer selected all six review interventions on 2026-11-03, including breaking
replacement of delivered B/C, and accepted RC01–RC04: actual registry/Rust owners derive
projections; breaks are explicitly versioned; the local service permits optional foreign
clients; the local prepared worker has its own binding route. ADR-0239 is the proposed
boundary decision route, not acceptance or execution evidence. This plan authorizes no production execution.

**Interface-checked:** source baseline `2550fa665`, preserving concurrent 64d–64g and capture
edits. The [review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md)
and [briefing](../library_leverage/libraries/proto/briefing.md) supply rationale. Core 3.4, heuristics 1.0,
ProcessSimulator 1.5 and test-assessment 1.2 apply. Authoring ran no product/generator/build/numerical/measurement work.

`pse-schema::FieldContract` wraps recursive Arrow/physical facets; registry/codegen project
model, relational, storage and document views (blueprint §4/§6).
`pse-runtime::boundary_documents::for_each_boundary_document!` registers actual Rust
document owners and direction. `host_operations/project.rs` declares
`ProjectAction` and `EffectFacts` together through `project_actions!`. `pse-codegen`'s
`codegen/documents.rs` and `xtask/src/codegen.rs::document_schemas` currently publish compiled
Schemars output; their JSON Schema values are publication artifacts, not a typed proto IR.

`workflow/modeling/documents.rs` owns `SourceDelta`, document/fit ownership and bounded
pages. `project_preparation_wire.rs` manually projects Number/Coordinate bits, quantity
IDs, Missing, Set, Tuple and ordered Rows. Completion/portable-payload owners retain
applicability, permission and lineage; another interpreter/catalogue would duplicate them.

## Resulting contracts and selected mechanisms

Registry-owned science projects from resolved registry contracts. Rust-owned requests,
responses and effects project from their typed declarations. Co-locate stable field tags,
alternative tags, semantic-role links, direction, version and supported local rules with
the relevant owner; one declaration may emit several representations. Retain Serde/Schemars
for intentional native JSON consumers and document interpretation. Never reconstruct proto
semantics by parsing emitted JSON Schema, source spelling, enum ordinal or field order.

The selected runtime is Buffa's existing descriptor-based family, rather than a second
proto runtime. Generate a complete `FileDescriptorSet`, owned messages/adapters, supported
views/validators, source links and client-facing documentation as one codegen closure.
Preserve imports and custom options. Feed resolved `FileDescriptorProto`s to
[`buffa_codegen::generate_with_diagnostics`](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/fn.generate_with_diagnostics.html)
using `CodeGenConfig`; surface warnings rather than discard them through `generate`.
`SharedCorpusContext` may share corpus analysis across package projections. I01 settles output
layout, registers generated ownership/provenance and publishes only with `just codegen`.

Use exact manifest pins: `buffa =0.9.2` with `json,text`, `buffa-descriptor =0.9.2` with
`reflect,json,text`, `buffa-types =0.9.2` with `reflect,json`, and
`protovalidate-buffa =0.7.2`. If directly consumed by the generator, declare
`buffa-codegen =0.9.2` once in workspace dependencies. Existing host generation enables
`reflection=true,reflect_mode=vtable,shared_descriptor_pool=true,text=true,json=true` and
packaging; reuse that composition for the product owner without importing product science
into the independent host. ConnectRPC `=0.9.1` is the service companion's transport owner.

[`DescriptorPool` and reflection](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/reflect/index.html)
support immutable shape/option inspection; generated vtable `ReflectMessage::for_each_set`
reads fields without a generic owned copy. `DynamicMessage::from_options` reads custom
options from the matching descriptor world. Where editable candidates help, use the
`DynamicMessage` implementation of `ReflectMessageMut::try_set/try_clear`: the trait's
default methods alone do not establish checked mutation. Check expected descriptor-world
identity before crossing pools; matching full names do not establish scientific identity.
Generated-to-dynamic conversion and foreign-pool adoption can encode/decode; prefer direct
typed construction when it avoids this. A `FieldMask` cannot encode exact-parent removal or
omitted/default intent. Exact 0.9.2 [codegen](https://github.com/anthropics/buffa/blob/1f5d80e2c8d31c147742d8ec826a3219c4fcd4a7/buffa-codegen/src/lib.rs)
and [dynamic reflection source](https://github.com/anthropics/buffa/blob/1f5d80e2c8d31c147742d8ec826a3219c4fcd4a7/buffa-descriptor/src/reflect/dynamic.rs) supplement release docs.

Static descriptors expose declared shapes, effects, owner links and local constraints.
Selected revision/physical-context inspection returns actual units, bases, datums,
component/phase identities, prepared provider availability and original refusals. Static
declarations establish no readiness, derivative support or scientific permission; fingerprints
replace no semantic hash, registry admission, contextual check or completion decision.

## Versioning, constraints and exact-parent editing

Give each externally consumed proto package an explicit major version and stable field
numbers. Reserve removed numbers/names; never recycle them for new meanings. Native
standalone document versions and input/output directions remain separate from package
versions. A changed datum, alternative interpretation, missing/default contract or stronger
validation requires an explicit interpretation/version decision even if wire-compatible.
RC02 authorizes designing such breaks; cut over all affected producers, clients, schemas,
fixtures and retained-read interpretations together. Where existing durable artifacts must
remain readable, implement a bounded explicit migration through the durable owner and test
it before removing the old interpretation; do not retain a parallel production adapter.

Publish complete descriptor artifacts and use [Buf structural compatibility rules](https://buf.build/docs/breaking/rules/)
for externally consumed contracts. Compare within the supported major; document an intended
major break rather than suppressing an unexplained failure. Binary unknown-field retention,
unknown alternatives and JSON/text presence are explicit consumer contracts. H owns the
independent semantic-change counterexamples; descriptor compatibility establishes no physics.

Declare local constraints once beside the owning field/alternative, deriving native/JSON
and `buf.validate` projections where consumed. Start with `ModelingDeclarationRequest.limit`
in `documents.rs` (1–128), required alternatives and selected request-local collection
relationships. Contextual parent membership, unit/datum resolution, source validity,
provider readiness, scientific permission and transactional predicates stay with their
existing owners. Invoke validation on uncontrolled ingress and explicitly on outgoing
mappings where a producer can violate the declared envelope. Carry field path, rule ID,
source/operation identity and typed failure category into original diagnostics.

Retain the [maintained validator generator](../../vendor/protoc-gen-protovalidate-buffa/README.md)
based on published 0.7.2; it preflights malformed/unsupported required rules, preserves
integer arithmetic/conversion errors, exact mixed numeric comparison and decisive boolean
error masking. Owned/view validators share lowering. Its `--preflight DESCRIPTORS --file
AUTHORED.proto` targets authored files with complete imports. Unsupported CEL fails
generation; it does not become omitted enforcement or an automatic runtime-CEL fallback.
The existing `buf.validate` schema selection is v1.2.2. Replace local patches only after
an upstream candidate preserves these named semantics and strict error categories.

Editable candidates submit the existing typed `SourceDelta` against an exact parent:
additions append; replacements keep original positions; omissions inherit; removals are
explicit; documents/fits retain their actual ownership. Preserve source spans/attribution
and semantic IDs through generated adapters. A client can inspect a declaration, author
one property/unit change and submit it without implementing physical interpretation.
Invalid admission leaves the original revision intact. Local rule violations, rule
compilation/evaluation errors, contextual refusals and infrastructure failures stay distinct.

## Package state

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| I01 | in-progress | ADR-0239 route; actual registry/Rust owners. Produces owner annotation metadata, resolved descriptors and generated engineering projections | Foundation decision and vertical preparation/action/completion slice below; remove replaced mappings with migrated consumers |
| I02 | in-progress | Working I01 owner/projection slice. Produces shared local constraints, structured validation and typed exact-parent client editing | Add/replace/remove/inherit plus invalid ingress/outgoing/context cases; remove duplicate envelope checks/string translations |
| I06 | in-progress | Actual selected producers from C/E/G, consumed I01/I02 projections and service read contracts | Selection before hydration/encoding, bounded lifetime and refusal tests; remove superseded projection/pass only after protections move |

### I01 — Settle metadata emission, then integrate the complete projection slice

First make a bounded source-level foundation decision covering one recursive structural
value, `ModelingDeclarationRequest`, one `ProjectAction` with effect facts and one scoped
completion product. Compare extending the existing declaration macros with adding an owner
derive that emits typed projection metadata; inspect current derives/macro expansion and
recursion, directional Serde behavior, external IDs and constraint attachment. The output
is the chosen annotation/emitted-metadata shape, dependency placement, tag ownership,
unsupported-type diagnostics and consumer migrations. Resolve this before committing the
general generator design; no unspecified future research, schema parser or parallel catalogue.

Then integrate discovery/bindings, the real action/request and completion slice through
registry/Rust metadata, descriptors, Buffa adapters, native JSON/schema publication and
generated client inspection. Extend the same mechanism across affected B/C and D–G
boundaries, plus the service/worker/reference envelope contracts selected by their owners.
Do not generate the unrelated entire host/product document catalogue merely for uniformity.
Unsupported selected types block that slice explicitly rather than degrading to JSON bags.

Migrate native preparation serialization, document/schema publication, operation metadata,
inspection/edit consumers, service clients and corresponding retained readback/fixtures in
the same coherent changes. Remove replaced hand-maintained structural-value conversion,
duplicate metadata and their obsolete tests once targeted replacement tests pass. Retain
distinct JSON/Arrow formats only for actual distinct consumers, with shared facets derived.
Challenge E/F extension from lumped inventory to distributed/cyclic configuration: owned axes,
phase allocation/events must reach inspection/trajectory metadata; integrator sensitivity
support stays contextual. Changed layout invalidates preparation; compatible value-only edits retain reuse.
The narrow `proto_rs` 0.11.26 shadow adapter's second runtime/mapping/validation closure
currently loses to Buffa. A global protobuf-first registry replacement adds storage/science
migration without established benefit. Reopen on evidence that this slice cannot retain
meaning or creates materially greater machinery.

### I02/I06 — Integrate authoring and useful selected reads

Move producer checks and client construction onto I02's shared local declarations while
retaining canonical delta/admission and completion authority. The service companion can
start with I01's working operation slice; it need not wait for every D–G projection.
Reference construction can consume stable envelopes while H independently supplies science.

For I06, follow `declaration_page/declaration_window`, inspection detail, preparation discovery
and selected study/product metadata to their actual producers. Carry fields/pages/products
there before hydration, copies and encoding. Preserve C's ordered full-inventory route;
do not implement it as repeated public pages that decode the same ingress again. Keep
numerical arrays in native/Arrow execution rather than adding proto crossings to callbacks.
At encoded boundaries use [`MessageView`, `OwnedView` and `ViewEncode`](https://docs.rs/buffa/0.9.2/buffa/view/index.html)
only where they remove a real owned reconstruction. Account for repeated/map allocations,
decoder limits, backing-buffer retention across awaits and lazy nested decode failures;
a compact owned projection may release substantially more memory than a borrowed view.

`bound_observation` currently counts serialized JSON extent before returning a response,
with a 64 KiB limit. If the selected proto encoder replaces this pass, move extent admission
to the authoritative output owner and preserve refusal before any publication/partial
output, overflow handling and native callers' protections. Do not simply delete the early
check because a later encoder also visits the response. Canonical pre-allocation wire
checks, complete observation/capture coverage and G08's atomic transaction remain intact.

## Current implementation checkpoint — 2026-11-03

Execution is authorized for the complete selected Proto scope. Source-owner macros and a
resolved projection graph now declare stable tags, recursive types, interpretation versions,
directions and local rules. The Buffa lowering/generation and native shared-rule validation
are implemented; consumer migration and integrated generation remain underway. The graph
stays in `pse-model`, independent of Arrow, Buffa and runtime dependencies.

After the maintainer's `cargo clean`, damaged edits were restored and the native host and
providers were explicitly prepared again. Foundation tests and complete generation now
run successfully. Actual preparation/value owners replace the former hand-maintained
wire projection. The exact-parent file-edit frontend delegates parsing, ownership and CAS
to the original source owner; compact declaration headers are selected before body copies.
The registry's selected quantity-type and basis rows now emit their projection metadata
from the resolved declarations, with explicit source tags and informative field facets.
Actual physical inspection projects the native quantity/unit/dimension/basis/datum closure;
checked ratio construction and fixed dimension cardinality survive native adoption.
The full runtime feature check and focused registry, header, edit and original-report
controls pass. Independent review found a worker deadline interpretation error and an
I/O reservation lifetime error; 64k owns their correction and affected verification.
Complete consumer migration, study/model inspection, assembled acceptance and performance
benefits remain open. The independent reference environment now includes pinned Protobuf tooling
for direct consumption of the generated descriptor world; structural controls do not
establish matched ordinary-process reference qualification.

The next foundation increment adds checked foreign records by extending their existing
Serde remote declarations in place, shared ownership without requiring foreign records
to implement Clone, native generic record access, nominal occurrence/hash/canonical-key
roles, exact document versions and Duration reconstruction. The original compiler/math
profile and Diffsol options now supply their own metadata. Native scalar constructors
and declared integer domains share exact bit/range predicates with the generated rules;
compiled model and descriptor controls exercise those boundaries. Declared registry
vocabularies now generate their projection traits directly, replacing the consumer
selection list. The actual canonical project-context row supplies explicit field tags.
Full solve/simulation/fit/horizon, authored nested-row and Study graph integration remains
underway. Executing the maintained validators over those newly consumed domains, complete
service journeys and independent ordinary-process comparisons remain acceptance work.

The authored declaration/document/fit rows and numerical, accuracy, caloric and engineering
rows now carry explicit source-owned presentation. Native types and field meanings remain
on their existing registry trees; source layouts check complete field and alternative
membership. Nested native tagged payloads use their existing selection and construction
methods for checked oneof access, retaining original JSON. Named structures retain source
presentation independently of value/execution identity, and diagnostic vocabulary aliases
project the actual lower owner's closed string members. Core Rust contracts were regenerated;
compiled native-row controls cover exact JSON, field domains and invalid overlapping arms.
The complete runtime settings/operation closure and full service descriptor publication
remain in progress. These foundation controls do not establish their assembled acceptance.

Exact finite numeric intervals now share their original endpoint inclusion rules between
native predicates and fixed64/CEL projections. Checked nominal scalars retain their original
subtypes: a consuming confidence field can require a strict upper endpoint without changing
the underlying Fraction domain. Source-owned hooks identify these scalars; similarly shaped
records acquire no numeric meaning. Targeted model and generator controls cover signed zero,
adjacent represented endpoints, invalid intervals and nested optional nominal values. The
complete settings graph and maintained-validator publication remain the next integration work.

## Verification and handoff

**Proposed:** I01 targeted tests independently specify exact bits (including signed zero),
IDs, quantity/basis/datum links, Missing versus default, Set/Tuple, ordered rows, directional
schemas, stable/reserved tags, recursive alternatives and retained unknown binary fields.
One owner field/alternative change must reach descriptors, clients, local rules and docs;
missing/duplicate tags, unsupported projection and stale descriptor worlds must refuse.
A round trip using the same transformation is insufficient evidence of preserved meaning.

**Proposed:** I02 exercises add/replace/delete/inherit, document/fit ownership, source
attribution, stale/nonmember parent, unknown alternative, missing/default and incompatible
unit/datum; rejection leaves the parent intact. Challenge malformed/unsupported CEL,
integer boundaries, decisive boolean errors, owned/view agreement and outgoing-invalid
producer mappings. I06 exercises selected/full/empty reads, corrupt ingress, late nested
decode failure, byte overflow, cancellation, partial-output refusal and backing-buffer
release. Count actual hydration/encoding work; a small returned page proves no avoided work.

Implementation uses affected `scripts/pse check <pkg> --all-targets` and targeted
`scripts/pse test <pkg> --lib -E '<filterset>'` for `pse-codegen`/`pse-runtime`, validator-tool
tests under its locked manifest, and `just codegen` for generator/declaration changes.
The root selects combined journeys and scope-end static/manual checks under H07 after
functional scope; report actual command, mode, scope, zero-failure baseline and result.
No such check ran during planning.

I finishes when required projections, constraints, consumers and deletions are integrated
and targeted negative controls pass. H retains assembled scientific acceptance and I05;
service/worker companions retain their execution/custody tests. Whole read/encode/consume
lifetime measurement is needed before a memory/latency benefit claim, routed through H06
when such a claim is made. No codec microbenchmark qualifies native solve performance.
At closeout move enduring meanings to blueprint §4/§6/§21/§23 through ADR-0239's route,
record actual evidence and deliberate deviations, and let the coordinator reconcile lifecycle.
