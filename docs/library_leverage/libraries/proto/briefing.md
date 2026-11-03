# Proto ecosystem: discovery briefing for Plan64

Inspected 2026-11-03 at `01b62e928`, including the concurrent working tree.
Current-use interpretations are **Interface-checked** representative reads; research
directions are **Proposed**. No product execution, conformance campaign or benefit
measurement was performed. This is a discovery account, not an applicability assessment
or adoption decision.

The most promising territory is a richer generated engineering interface: machine-readable
contracts that support inspection, editing, validation, clients and independent reference
exchange together. Merely changing serialization would capture much less of that potential.
The inquiry ranks potential benefit across delivered and remaining work, including ideas
that could warrant reconsidering a settled choice after deeper assessment.

## Capability territory and observed use

The twelve documents under `docs/libraries/proto/` were surveyed by capability family,
with selected mechanisms checked against current documentation and installed sources.
Their original October 28–31 research dates remain distinct from this inspection.

| Capability family and overview | Current observation and additional territory |
| --- | --- |
| [Buffa](../../../libraries/proto/Buffa.md) | The host's [Buf configuration](../../../../crates/pse-host/proto/buf.gen.yaml) enables vtable reflection, a shared descriptor pool, JSON/text and generated validation. Borrowed/lazy views, custom storage and external-type mappings remain useful territory beyond ordinary owned messages. |
| [ConnectRPC](../../../libraries/proto/ConnectRPC_for_Rust.md), [local RPC and diagnostics](../../../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) | Native semantic services use typed contracts and explicit domain mappings. Generated services, protocol interoperability, service reflection and independent clients could also support engineering operations. Application custody and native settlement retain separate owners. |
| [Validation](../../../libraries/proto/protobuf_validation_rust.md) | The host already uses `protovalidate-buffa`, including explicit outgoing validation in [semantic mappings](../../../../crates/pse-host/src/semantic/wire.rs). The [maintained generator](../../../../vendor/protoc-gen-protovalidate-buffa/README.md) corrects rule admission and arithmetic/error behavior; it is a bounded CEL lowerer. Boundary constraints could extend to scientific requests without becoming scientific admission. |
| [Prost](../../../libraries/proto/prost.md), [Tonic](../../../libraries/proto/tonic.md) | Existing non-intelligence uses include canonical SurrealDB protobuf traffic and [Perfetto packet output](../../../../crates/pse-runtime/src/tooling/capture/timeline.rs). These are existing leverage, not proposed first adoption. External protocol requirements can justify retaining this stack. |
| [Reflection and PBJSON](../../../libraries/proto/prost_reflection_and_pbjson.md), [ecosystem overview](../../../libraries/proto/rust_protobuf_ecosystem_overview.md) | The host's [schema/payload reader](../../../../crates/pse-host/src/schema/payload.rs) already interprets supplied descriptor worlds. Dynamic messages, custom options and presence-aware inspection suggest scientific tooling that need not hard-code each message type. ProtoJSON and ordinary Serde have different contracts. |
| [Compilation/orchestration](../../../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md), [generators](../../../libraries/proto/protobuf_generator_ecosystem_rust.md) | The host already orchestrates Buffa, packaging, ConnectRPC and validator plugins. Descriptor-driven Rust/client, documentation and schema outputs could serve additional consumers. Import/options preservation and generator compatibility need explicit verification. |
| [Rust-first contracts](../../../libraries/proto/Rust_first_proto_contracts_and_proto_rs.md) | `proto_rs` offers another ownership direction: native types/services producing wire contracts. It is a research alternative to generated adapters, not a selected replacement for the registry's storage, relational and documentation responsibilities. The overview separates published 0.11.26 from development 0.12 features. |
| [Installed tools](../../../libraries/proto/protobuf-grpc-installed-tools.md) | The recorded inventory supplies leads for dynamic clients, schema inspection, generation and diagnostics. Its installation/qualification claims were not rerun here; availability must be checked before a selected tool is used. |

Current declarations include Buffa/descriptor/types 0.9.2, ConnectRPC 0.9.1,
Protovalidate-Buffa 0.7.2, Prost 0.14.4 and Tonic 0.14.6. Versions/features remain owned
by the [workspace manifest](../../../../Cargo.toml) and lockfile. Newer versions named
in overviews are research routes, not installed capabilities. The existing evidence-export
configuration selects Arrow/DataFusion, Tokio and Symbolica; this pass has no Proto
semantic reference census or acquired API bundle. Searches and source samples cannot
establish repository-wide absence.

## Plan64 consumers and changing baseline

The [coordinator](../../../plans/64-process-simulator-target.md) and companions own status.
64a's library-informed decisions retain registry-first declarations and generated adapters,
Buffa/ConnectRPC product services and externally required Prost/Tonic. These choices supply
the comparison baseline; discovery can identify reasons to investigate alternatives.
Blueprint §4/§6, §13/§19/§21 and §20 own schema, workflows and publication contracts.

| Consumer footprint | Representative responsibility that shapes discovery |
| --- | --- |
| 64a shared contracts; delivered 64b/64c intake and authoring | Typed declarations, exact-parent `SourceDelta`, contextual admission, source provenance and reuse/invalidation. [Editing documents](../../../../crates/pse-runtime/src/workflow/modeling/documents.rs) and [preparation bindings](../../../../crates/pse-runtime/src/workflow/modeling/project_preparation_wire.rs) expose concrete representation seams. |
| 64d scientific assets and equipment | Property/reaction/state conventions and prepared provider contracts. The concurrent CoolProp adapter explicitly selects phase, datum, source and image identity; a discoverable interface must retain those distinctions. Its presence is work underway, not qualification. |
| 64e inventory/distributed/cyclic physics; 64f specialized families | State/inventory meaning, temporal/spatial layout, event/cycle interpretation and family-specific configurations. Generic message shapes cannot establish supported dynamic or physical combinations. |
| 64g analyses, studies and applications | Scoped products, observation/noise meaning, shared estimands, surrogate artifacts and capture. [Portable lineage checks](../../../../crates/pse-runtime/src/workflow/portable_payload.rs) and current study/capture owners show that readable data and scientific reuse permission differ. |
| 64h references and qualification | A finite configuration population, independently constructed references and complete comparison outputs. The [ordinary correspondence](../../../../packages/reference/ordinary-delivery/sources.md) already distinguishes formulations, intended failures, physical conventions and historical fixtures. |

The recent footprint includes ordinary scientific delivery, physical-premise propagation,
precision context, preparation reuse and retained diagnostics. Concurrent changes add
state/reaction/inventory/steam models and an atomic capture-witness implementation.
These changes make contract interpretation and provenance more consequential; they do not
demonstrate that protobuf is the appropriate representation. Generated churn is not counted
as opportunity size. Recheck affected owners before deepening any hypothesis.

## Research priorities

### 1. Generate an inspectable engineering contract from existing meaning

Investigate registry-derived descriptors with custom options, Buffa `DescriptorPool`,
`ReflectMessage::for_each_set` and `DynamicMessage::from_options`. Together they could
expose declared quantities, alternatives, source identities and product scope to clients,
editors and inspectors without a hand-maintained interpretation per consumer. This spans
64a–64g and could support provider/configuration discovery as well as results.

The decisive question is whether one selected declaration-to-result slice can generate
useful metadata and adapters while reusing existing Rust/domain owners. Start with
preparation bindings or a scoped analysis product. Compare descriptor generation against
the current Serde/Schemars and registry projections, and against a narrow Rust-first route:
`proto_rs` 0.11.26 `#[proto_message]`, explicit tags and fallible shadow-to-domain conversion
could be compared at one adapter without requiring a global registry replacement.
If each projection needs bespoke parallel semantics, the proposed simplification fails.
Descriptor identity also does not admit a foreign scientific registry or define semantic
hashing. [Buffa reflection contract](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/reflect/index.html).

### 2. Enable additional scientific implementations through a complete provider contract

Generated services and language bindings could let an isolated or non-Rust property,
reaction or transport implementation serve 64d/64f consumers through the existing
[provider contract](../../../../crates/pse-kernels/src/lib.rs). `ProviderSpec` and
`Provider::fill` already distinguish physical ports, implementation/data identity,
derivative capability and coherent attempt-local evaluation. A service implementation
could expand accessible science or isolate a foreign runtime while retaining that meaning.

Can one bounded request deliver the complete requested state and derivative products,
with exact source/data identity, branch assumptions, refusal classes and cancellation?
Compare with the explicitly registered in-process CoolProp route. Scalar RPCs inside
frequent solver callbacks could overwhelm any benefit; batching must follow the actual
coherent operation. Mutable state, derivative actions/linearization lifetimes and physical
settlement are especially consequential. No remote provider feasibility or performance is
established here; select a concrete otherwise valuable scientific implementation before
designing a worker protocol.

### 3. Compile reusable boundary constraints and actionable explanations

`buf.validate` options, generated `Validate` implementations for owned/view messages and
structured field/rule violations could improve scientific-input and study-request feedback.
Candidate constraints include required alternatives, finite collection shapes and local
cross-field consistency. The potential gain is one declared rule serving several ingress
and diagnostic consumers rather than repeated checks and translated string errors.

Select an existing `SourceDelta` or research/study request and separate locally decidable
constraints from revision lookup, units/datum resolution, scientific permission and
transactional predicates. Which checks could disappear completely, with equally useful
error attribution? Unsupported CEL must refuse or remain explicitly outside generated
coverage. The maintained generator's precision and error-masking corrections are essential
comparison inputs; upstream conformance claims alone cannot qualify the local consumer.
[Local generator contract](../../../../vendor/protoc-gen-protovalidate-buffa/README.md).

### 4. Make scientific editing and inspection share a schema-aware surface

Dynamic field access, explicit presence, `oneof` descriptors, unknown-field handling and
standard field-path representations could enable reusable forms, selected-detail reads,
change previews and meaningful request diffs. Existing host schema/payload tools provide
working machinery to investigate for engineering use, including offline inspection.

Test the idea against an exact-parent edit that replaces a declaration, deletes another and
retains inherited data. Can generic field machinery represent the user's intent and produce
the existing typed delta without losing order, source spans or explicit deletion? A
`FieldMask` is a field-selection representation, not a ready-made scientific patch engine.
The inspected generated vtable reflection is read-only; checked mutation belongs to
`DynamicMessage::try_set`/`try_clear`, and bridging generated messages incurs a wire
round trip. Direct mutation of generated domain structs is not established by reflection.
Presence/default conversion and unknown enums may invalidate a seemingly convenient route.
Retain the current delta/admission owner if reflection merely adds another conversion layer.

### 5. Share portable reference contracts while keeping calculations independent

Descriptors, generated language bindings, ProtoJSON/textproto and Buf compatibility checks
could make 64h's case/configuration/output exchange easier to inspect and validate across
Rust and isolated reference drivers. This could also support portable study and surrogate
metadata in 64g. Shared structural contracts need not imply shared numerical calculation.

The next question is whether one ordinary recycle/exchanger case can express exact units,
datum, formulation, intended failure, output coverage and unavailable evidence without
sharing production-derived expectations. Compare with the existing exchange owners before
adding a format. Passing [Buf compatibility checks](https://buf.build/docs/breaking/)
does not establish unchanged physics or validation semantics. ProtoJSON conversions and
third-party well-known-type support need selected round-trip controls; retain Arrow for
tabular products when it already supplies the appropriate exchange.

### 6. Expose engineering operations through reusable service contracts

Generated ConnectRPC methods, reflection and independent clients could provide a common
entry point for model inspection, preparation, study submission, result discovery and
attributable failure inspection. Connect/gRPC/gRPC-Web support could broaden consumers
beyond the current native tooling. The opportunity is a coherent callable engineering
surface over existing owners, rather than a second workflow implementation.

Compare one complete finite study/readback operation with current native calls: what client,
dispatch or diagnostic machinery would be removed, and which new consumers would benefit?
Transport success, health and schema discovery cannot establish scientific readiness,
capture atomicity or native drain. Keep an explicit operation/result/cancellation contract,
including terminal failure and reconnect behavior. The current local service custody is
useful evidence, not remote deployment qualification.
[ConnectRPC 0.9.1 service and protocol capabilities](https://docs.rs/connectrpc/0.9.1/connectrpc/).

### 7. Reduce materialization at actual metadata and diagnostic boundaries

Buffa `MessageView`, `OwnedView`, opt-in lazy views and `ViewEncode` offer concrete ways to
inspect or relay payloads without constructing every owned field. These deserve attention
where selected scientific metadata or diagnostic pages cross a boundary before only a
small subset is consumed. 64c's recent selection/projection improvements make this a
specific continuation question; 64e/64g may introduce larger event/study metadata.

Find an actual encoding/decoding/copying boundary first. Does view-based handling eliminate
work before selection, or simply add a codec around already efficient native/Arrow data?
Buffer retention, repeated-field allocations, decoder limits and ownership across awaits
can defeat the gain. The canonical client's pre-allocation response checks are a distinct
obligation. Numerical callbacks should not acquire RPC or descriptor interpretation merely
to use these features. [View ownership contract](https://docs.rs/buffa/0.9.2/buffa/view/index.html).

## Boundaries, less relevant territory and handoff

These directions reinforce one another: generated metadata can support validation,
inspection, clients and exchange. Investigate a complete operation before multiplying
independent adapters. A global registry replacement remains eligible for deliberate
comparison, but `proto_rs` must account for storage, SQL/Arrow, documentation and migration
responsibilities as well as Rust/wire types. Its development-only transport features do
not strengthen the published route's evidence.

Kafka/Confluent registries, public gateways, browser clients and packet/load tools remain
eligible when a real consumer supplies value; this pass found no reason to make them
prerequisites for native simulator delivery. Kernel zero-copy and transport replacement
are lower priorities than contract reuse and useful engineering access. Serialization
does not replace atomic capture: membership/protection checks, original request identity,
retry semantics and rollback remain with the study transaction owner.

Research used the overview collection, Context7 Buffa/Protovalidate/Buf material and
selected exact installed source. Context7's ConnectRPC Rust resolution returned unrelated
results, so its released docs and installed source supplied the follow-up. In particular,
Buffa-descriptor 0.9.2 `src/reflect/{message,dynamic}.rs` confirms the reflection operations;
the current host generator configuration and maintained validator README bound local use.
No overview's latest-release or installed-tool claim was promoted to a fresh qualification.

Deepen generated engineering contracts with a representative authoring/product slice,
and provider interoperability with a concrete valuable scientific implementation. Keep
validation, editing, exchange and service consumers visible while testing the proposed
reduction in machinery and any newly enabled capability.
Concrete responsibility evidence enters `leverage-codebase-audit`; focused contracts enter
`leverage-library-assessment`; complete alternatives enter `leverage-opportunity-review`.
This shortlist creates no CBO, applicability verdict, implementation obligation or change
to Plan64's package status. Other capabilities remain eligible as the footprint changes.
