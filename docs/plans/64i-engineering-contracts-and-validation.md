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
| I01 | in-progress | ADR-0239 route; actual registry/Rust owners. Produces owner annotation metadata, resolved descriptors and generated engineering projections | Source machinery, registry declarations and narrow generated consumers implemented with targeted tests. Expanded engineering publication fails maintained CEL preflight; complete generation, all selected consumer migrations and replaced-mapping deletion remain. |
| I02 | in-progress | Working I01 owner/projection slice. Produces shared local constraints, structured validation and typed exact-parent client editing | Native/local rules and narrow exact-parent editing tested. Expanded maintained-validator execution, full operation ingress/outgoing/context controls and remaining duplicate-check/string-translation deletion remain. |
| I06 | in-progress | Actual selected producers from C/E/G, consumed I01/I02 projections and service read contracts | Narrow headers select before copying; reference codec/source-custody controls pass. Full Study/cohort/model/provider producer-to-client selection, malformed/overflow/interruption controls and safe retirement of replaced passes remain; no measured benefit established. |

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

## Current implementation checkpoint — 2026-11-04

Complete Proto execution is authorized. The committed foundation reaches `1de457d87`;
additional runtime/settings/reference integration remains in the working tree. The graph
stays in `pse-model`, independent of Arrow, Buffa and runtime dependencies.

**Implemented and targeted Tested:** source-owner macros and the resolved graph declare
stable tags, recursive types, directions, versions and local rules. Registry generation now
covers the authored declaration/document/fit and numerical/accuracy/caloric/engineering
rows from their actual owners. Checked foreign Serde declarations, generic/shared ownership,
diagnostic vocabulary, nominal identity/hash roles and exact document versions are supported.
Native settings for all nine serializable backend families and integration controls carry
source-owned metadata. Signed-zero and finite-interval endpoints, nominal scalar subdomains,
optional UTF-8 byte limits, standard/positive duration and physical extension storage have
native and generator controls. Original JSON, Arrow storage and contextual admission remain
owned by their existing declarations.

Passive lower scientific Scheme/Type/DomainKey/Carrier metadata and qualified resolved-output
receipts now expose their source declarations. The passive projection does not grant native
restoration authority; original qualification/restore checks remain. Targeted model, quantity
and modeling controls distinguish readable metadata from admissible native restoration.
Generated core Rust contracts and macro-body owner relocation have been exercised.

**Implemented and Tested at the earlier narrow service snapshot:** preparation/value
owners replace the former hand-maintained projection; exact-parent file edits use original
parsing, ownership and CAS; declaration headers select before body copies; physical inspection
retains quantity/unit/dimension/basis/datum. 64j owns the actual external client evidence.

**Implemented in source, not accepted as a complete generated delivery:** full numerical,
solve/simulation/fit/horizon and Study declarations, issued-Study identity and model/provider
inspection are connected to the expanding graph. A 36-test runtime selector passed the
then-linked settings, operation, covariance/sensitivity, original-source and reference-codec
controls before the latest Study/session/inspection additions. It does not qualify those
later additions. Reference encoding/source custody now has bounded preflight and lifetime
controls; [64h](64h-reference-coverage-and-qualification.md#proto-reference-implementation-checkpoint--2026-11-04)
owns the remaining real native/reference correspondence.

**Current failing acceptance:** the full-feature engineering generation command compiled
the runtime/xtask graph, then failed maintained CEL preflight for
`pse.engineering.v1.HighsSettings.sparse_start`, rule `float64.collection_domain`, with
`unknown field: value` (one refused rule, zero-failure target). The generated collection
accessor must match the actual optional map layout without weakening finite-value admission.
Earlier successful generation covers the earlier descriptor population only. The expanded
engineering descriptors, adapters, service bindings and documentation are not published and
accepted merely because their Rust owners compile.

Remaining I work is to repair that lowering and add a maintained-validator regression;
regenerate the complete selected worlds; integrate and test all consumers, versions and
outgoing/ingress validation; complete selection/lifetime negative controls through J's real
responses; and delete every replaced mapping/check after equivalent protection is proven.
I01/I02/I06 remain in progress. No whole-lifetime performance benefit or assembled scientific
qualification is established.

## Verification and handoff

**Tested foundation snapshots, local Linux, zero-failure baseline:** the native correctness
route `scripts/pse test pse-codegen --lib -E 'test(/^codegen::boundary_projection::tests::/)|test(/^codegen::rust::boundary_nested::tests::/)'`
passed 21 selected tests. Later passive-owner coverage with
`scripts/pse test pse-model --lib -E 'test(passive_scheme_projection)|test(passive_resolved_output)|test(intrinsic_pair)|test(explicit_public_alias)|test(mixed_source_alternatives)'`
passed five selected tests. These are targeted source/descriptor and admission controls;
unselected tests were not exercised, and the subsequently expanded maintained CEL world
still has the explicit failure above. They are not a full codegen or service acceptance pass.

**Tested runtime snapshot:** the correctness nextest command below passed 36 selected
library tests, zero failures against zero, with explicit native preparation and Arrow
force-validation. It predates the latest full Study/session/inspection additions; the H
reference checkpoint consumes only its source/custody/codec and numerical-context coverage.

```bash
scripts/pse-env --native=compiler,solver,klu,isolation -- \
  scripts/cargo-bin/cargo-pse-correctness nextest run -p pse-runtime --lib \
  --features engineering-service,property-coolprop,solver-kinsol,solver-ipopt,solver-root-isolation,product-reader,native-study-tests,force-validate \
  --locked --no-fail-fast -E \
  'test(workflow::modeling::ordinary_reference::tests)|test(captured_manifest_uses_the_same_strict_owner_without_rereading)|test(study_creation_coordinates_)|test(original_study_creation_request_)|test(provider_inspection_retains_actual_factory_)|test(covariance_projection_preserves_)|test(sensitivity_projection_preserves_)|test(reconstruction_refinement_projection_)|test(named_horizon_intents_preserve_)|test(reference_exchange::tests)|test(actual_native_reports_preserve_physical_meaning_and_refuse_incomplete_selection)|test(actual_solve_settings_projection_)|test(convexity_numeric_inputs_)|test(operation_request_projects_all_four_)|test(preparation_document_preserves_all_existing_controls)|test(study_v8_physical_intent_owns_defaults_)'
```

**Failed expanded generation:**
`scripts/pse-env --native=compiler,solver,klu,isolation -- scripts/cargo-bin/cargo-pse-correctness run -p xtask --bin xtask --features full-codegen,force-validate --locked -- codegen --only engineering`
completed Rust compilation, then refused one CEL rule against a zero-failure target, as
described in the current checkpoint. The generation failure has not been repaired or rerun.

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
