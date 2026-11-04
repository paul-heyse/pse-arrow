# Clients, code intelligence and reference tooling

Inspected 2026-11-03 at `f55951cfe995a689f9b75185b8e89cb362021616`, including the
supplied uncommitted Proto overviews. Local mechanisms and selected library APIs are
**Interface-checked**; target integration and benefits are **Proposed**. No independent-client
journey, conformance run or scientific reference calculation was executed. Criteria: core 3.4,
efficient architecture heuristics 1.0 and process-simulator profile 1.5. SurrealDB v4 is excluded.

## Target: one useful contract surface for independent local consumers

Publish generated contracts, complete descriptors and faithful native adapters for code
intelligence, engineering operations, scientific inspection and independent reference
exchange. Share generation and descriptor tooling where their meaning agrees; keep each
consumer's authority and lifecycle explicit. A native CLI remains a useful consumer, while
an independent client can inspect or invoke selected operations without duplicating Rust
domain logic or becoming a required Python product layer.

The target includes generated language clients, dynamic inspection, offline schema-aware
tools, descriptor-driven scientific forms and meaningful request/result explanation. It does
not require browser delivery, public gateways, a universal management daemon or remote solving.
Keep extension seams for those consumers without making them prerequisites for local work.

## Existing mechanisms to extend

[`pse-host::code_intelligence_contracts`](../../../../../crates/pse-host/src/code_intelligence_contracts.rs)
already exposes generated Buffa messages, validator modules, generated ConnectRPC services,
and a complete source-aware `DESCRIPTORS` artifact. Its embedded runtime pool is a derived
projection. [`semantic/wire.rs`](../../../../../crates/pse-host/src/semantic/wire.rs) performs
explicit native conversion: configuration, feature selection, source bytes and opaque original
provider evidence have deliberate meanings. `checked_outgoing` invokes generated `Validate`.
This is observed leverage, not evidence that every native mapping can already be generated.

[`semantic/service.rs`](../../../../../crates/pse-host/src/semantic/service.rs) supplies
session ownership, original operation terminals and CPU settlement. Live LSP and native
snapshot have different capabilities/lifetimes. Reusing the transport/codegen infrastructure
for engineering services must not turn them into the same session kind or let a dropped
client stream release active native work.

[`schema/payload.rs`](../../../../../crates/pse-host/src/schema/payload.rs) already decodes
selected binary/JSON payloads through `DynamicMessage` and reflects generated messages through
their own descriptors. Its reports include field and unknown-field observations; it provides
a concrete base for scientific payload inspection. Extending it should consume additional
descriptor worlds rather than introduce a second generic decoder/field walker.

[`reference_exchange.rs`](../../../../../crates/pse-runtime/src/reference_exchange.rs) has
`DecodedReference::known` for checked native adoption and `forward` for the retained original
message. Its bounded `decode`/`encode` path uses generated adapters and validators. The
Python helper [`tests/reference/proto_exchange.py`](../../../../../tests/reference/proto_exchange.py)
uses `FileDescriptorSet`, a private `DescriptorPool`, import-order loading and
`message_factory.GetMessageClass` to construct the independent envelope type. It bounds bytes,
file population and duplicate/unresolved descriptors. The module explicitly contains no
scientific answers. These are existing product/reference boundary owners.

## Complete consumer compositions

### Descriptor delivery, reflection and offline inspection

Publish a selected complete descriptor set alongside each generated client/contract package.
Retain imports, custom options, defining package identity, source artifact identity and semantic
interpretation version. Runtime reflection pools can omit source information: Buffa's embedded
`FILE_DESCRIPTOR_SET_BYTES` strips `source_code_info`, so use the separate source-aware artifact
when exact source/documentation attribution is needed. Schema text, descriptor symbol identity,
and byte-accurate source ranges are different products.

Use Buffa 0.9.2 `DescriptorPool`, `DynamicMessage`, descriptor field/oneof/enum information and
custom-option decoding for generic inspection and selected UI/tool generation. The inspector
can expose supported operations, physical metadata, required alternatives and product scope.
It must report unknown versions/options rather than infer that structural readability grants
scientific admission. Reflection metadata is not a security or capability grant.

Dynamic mutation is useful for a schema-aware request editor, but generated reflection and
dynamic reflection have different mutation surfaces and conversion costs. A `FieldMask` names
fields; the domain edit owner must still define replacement, merge, explicit deletion, order
and exact-parent semantics. Generate an edit preview over the existing delta operation rather
than changing generated structs and assuming a correct scientific patch follows. See
[representation and validation](representation-reflection-and-validation.md).

For a consumer standardized on Prost, `prost-reflect::DescriptorPool` and `DynamicMessage`
provide an alternate reflective world and PBJSON provides ProtoJSON-compatible generated
Serde implementations. Do not pass descriptor-local IDs between Buffa and Prost pools or
treat ordinary Serde JSON as ProtoJSON. Prefer a shared wire/descriptor boundary over a
repository-wide second reflection model. The supplied
[reflection/PBJSON addendum](../../../../libraries/proto/prost_reflection_and_pbjson.md)
and [Prost](../../../../libraries/proto/prost.md) distinguish unknown fields, enum preservation,
merge behavior, JSON rules and source fidelity.

### Generated clients and independent invocation

Generate clients from the same selected service descriptors, then make one independent local
consumer complete an actual operation: discover/inspect, submit a typed request, observe the
original outcome, read protected results and close its own resources. Use the existing
explicitly prepared/authenticated local endpoint, not an implicit build/start during client
discovery. The service adapter dispatches existing typed handlers; it does not implement a
parallel scientific workflow.

For dynamic diagnostics, `grpcurl` supports gRPC reflection or offline `-protoset`/source
schemas and Unix socket invocation. `buf curl` supports explicit Connect/gRPC/gRPC-Web protocol
selection and streaming JSON documents. `granc`/`granc-core` offers a Rust dynamic-client
candidate. Select tools by the actual local socket/HTTP/protocol and streaming shape; do not
assume every tool supports every Connect transport. The
[Local RPC tooling sections](../../../../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md)
identify the routes and their primary sources, including
[grpcurl](https://github.com/fullstorydev/grpcurl) and
[Buf curl usage](https://buf.build/docs/curl/usage/).

An independent client has its own response bounds, deadline, descriptor support and formatting
limits. For example grpcurl's message bound can refuse an otherwise valid server response;
successful transport and successful full diagnostic display are distinct observations.
Schema discovery, connectivity and health do not establish prepared scientific readiness or
operation settlement. The installed-tools overview is an inventory lead; verify availability
and selected flags before an executable journey rather than inheriting old qualification.

### Scientific reference exchange

Share structural contracts while constructing the reference calculation and its outputs
independently. Generate or dynamically load the envelope type in the isolated reference
environment. Carry case/configuration identity, exact units/datum/formulation, intended failure,
output extent, unavailable evidence and implementation context as appropriate to the selected
comparison. The reference helper must not translate candidate answers into expected answers.

Keep original binary messages where forward preservation matters, and checked native
projections for the adopted scientific interpretation. Unknown fields may survive forwarding
without becoming supported science. A JSON conversion is not an archival identity-preserving
operation; generated Prost messages and dynamic representations also differ in unknown-field
behavior. The current reference route's two representations are a useful explicit contract.

Use Arrow for tabular scientific exchange when it already carries the appropriate data and
metadata. Use Proto envelopes for selected configuration, identity, outcome and descriptor
metadata; do not rebuild large columnar products as repeated generic values merely to use one
codec. Native source identity and physical comparison remain outside protobuf's wire guarantees.

## Build, evolution and assurance tooling

Treat the compiler, message generator, service generator, validator, descriptor producer and
formatter as one intentionally selected generation composition. They can have different
version lifecycles. `protoc`/`protox`/Buf, custom options, editions and imported type mappings
must be checked together for the actual schema corpus. Buf lint and breaking checks can
establish selected schema-policy properties; they do not establish scientific equivalence,
validation-policy compatibility or database mapping compatibility.

Compare Rust-first `proto_rs` as a declaration alternative where native types/services are the
right owner. Explicit field tags and fallible conversion remain necessary; development-only
features do not establish a released service route. A registry-derived projection remains
strong where the same owner must also generate Arrow/Surreal representations and documentation.
The complete choice belongs to [contract ownership](contract-ownership-and-generation.md),
not whichever generator emits the shortest example.

Reuse source-aware descriptors and generated diagnostics to improve code navigation and
generator error attribution. Do not reconstruct exact UTF-8 ranges from protobuf descriptor
line/column values: native syntax/source owners establish exact byte ranges and provenance.
An API inventory, resolved reference census and a parser spelling match supply different
evidence. Extend the existing code-intelligence tooling only where the actual semantic
capability and preparation context support the claim.

Upstream Protobuf conformance, Connect conformance, gRPC interoperability, validation
conformance and the local operation journey test separate layers. A library passing its
selected upstream suite does not qualify authenticated Unix custody, source mapping,
scientific admission or native drain. Preserve failed and unsupported configurations as
explicit limits. See the supplied
[compilation/orchestration](../../../../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md),
[generator ecosystem](../../../../libraries/proto/protobuf_generator_ecosystem_rust.md),
[Rust-first alternatives](../../../../libraries/proto/Rust_first_proto_contracts_and_proto_rs.md)
and [installed tools](../../../../libraries/proto/protobuf-grpc-installed-tools.md) references.

## Comparative conclusion and extension controls

**Recommended proposed target:** source-derived interoperable contract packages, one reusable
schema/payload inspection engine, generated independent local clients, existing native CLI
handlers, and isolated reference consumers. Retain explicit adapters for semantically different
provider documents or externally specified protocols. Remove redundant schema walkers,
client-specific domain reinterpretation and manual contract copies where complete generated
coverage replaces them.

Keeping separate hand-maintained native/wire/client schemas has lower immediate generator work
but makes every scientific capability extension a repeated interpretation exercise. A universal
dynamic client instead of typed operation handlers removes useful static/domain checks and
still needs the same admission/custody. The recommended mixed design shares generic capability
without delegating scientific decisions to arbitrary descriptors.

Challenge it with a new physical quantity/alternative, a capability unknown to an old client,
an exact-parent edit with deletion, a non-UTF-8 source path, a descriptor missing an import,
unknown/malformed error detail, and a reference envelope with future fields. A new consumer
should use generated metadata without editing the scientific implementation; a new scientific
capability must still update its actual semantic owner and admission, not merely its schema.

Primary library evidence includes [Buffa's versioned guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md)
and [ConnectRPC's versioned guide](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md),
with the exact local source owners linked above. Context7's Buffa response was used for discovery
and kept distinct from tagged 0.9.2 semantics. No fresh host-tool availability, universal protocol
interoperability, performance benefit or additional numerical reference coverage is claimed.
