# Diagnostics, results and observability

Inspected 2026-11-03 at `f55951cfe995a689f9b75185b8e89cb362021616` with supplied
uncommitted Proto reference additions. Local mechanisms and selected APIs are
**Interface-checked**; recommendations and benefits are **Proposed**. No diagnostic journey,
scientific calculation or performance campaign was run. Criteria: core 3.4, efficient
architecture heuristics 1.0 and process-simulator profile 1.5. SurrealDB v4 is excluded.

## Target: one diagnostic meaning, several faithful projections

Generate diagnostic and result metadata contracts from the native owners that already know
the facts. Use typed Proto for independent inspection, machine-readable failures, selected
progress and retained-result descriptions. Keep scientific result arrays in their useful
Arrow representation and existing external telemetry protocols in their supplied schemas.
Make completeness, unavailable information and original evidence explicit at every projection.

This creates a shared machine-readable account across native CLI, local clients, scientific
workers and retained readers. It does not merge operational observations, solver outcomes and
durable scientific results into one undifferentiated status message. Each has different
authority, lifetime and completeness.

## Observed owners and consumers

[`pse-diagnostics::observation`](../../../../../crates/pse-diagnostics/src/observation.rs)
stores bounded copied facts. `Policy` controls diagnostic bytes, operation/metric/detail
populations and retention independently of scientific execution limits. Its observations do
not own cancellation or native resources. The runtime's
[`diagnostic_documents`](../../../../../crates/pse-runtime/src/workflow/diagnostic_documents.rs)
retains source chains, native annotations, spans, notes and help. Modeling
[`diagnostic_source_map`](../../../../../crates/pse-runtime/src/workflow/modeling/diagnostic_source_map.rs)
distinguishes authored fields, body-local source tokens, display paths and unavailable source
joins; a shared mathematical body is not an authored source occurrence.

Results have a different flow:
[`result_projection`](../../../../../crates/pse-runtime/src/workflow/result_projection.rs)
selects declared fields and produces admitted batches with original scientific coordinates;
[`result_inspection`](../../../../../crates/pse-runtime/src/workflow/result_inspection.rs)
exposes protected, budgeted field/path inspection and explicit metadata omission. Canonical
result blocks retain Arrow payload and descriptors. A new Proto envelope must reference that
ownership instead of constructing a second scientific result store.

External diagnostics already use Proto.
[`tooling/capture/timeline.rs`](../../../../../crates/pse-runtime/src/tooling/capture/timeline.rs)
uses Prost and `tracing-perfetto-sdk-schema` to emit standard Perfetto packets. Its `PacketWriter`
tracks complete/flushed extent and writes concatenated one-packet `Trace` encodings, not a
length-delimited message stream. OTLP configuration in `Cargo.toml` supplies the separately
versioned OpenTelemetry path. These are positive retain cases for standard external formats.

## Typed error projection

The precise released ConnectRPC 0.9.1 mechanisms are `ConnectError`, `ErrorCode`,
`ErrorDetail::from_message`, `ConnectError::with_detail`, response metadata, and
`ConnectError::with_source`/`with_shared_source`. The last pair retain local-only causes;
source errors are not serialized. `ErrorDetail::from_message` accepts the bare fully qualified
message name and handles protocol base64/type-URL representation. Use it rather than
hand-constructing a subtly incompatible detail. The installed source was read directly; see
the [tagged error implementation](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs).

**Proposed composition:** the native error/outcome owner constructs one typed diagnostic
projection. The RPC adapter selects a stable coarse transport code, adds a bounded typed
detail and preserves the original local source. Independent clients decode recognized details
by descriptor and report unknown or malformed details separately. A public detail should carry
an attributable native code/stage and relevant source/operation/result identities; it must not
derive them by parsing display strings. Large related diagnostics belong in a protected
paginated/readable record, with the error detail identifying that record and honestly stating
whether the inline detail is complete.

For gRPC consumers, standard `google.rpc` details provide useful interoperability:
`BadRequest` for field failures, `ErrorInfo` for reason/domain metadata, `PreconditionFailure`,
`ResourceInfo`, `RequestInfo`, and `Help`. `RetryInfo` conveys suggested delay, not permission
to repeat a scientific effect. Tonic consumers can use `tonic-types` 0.14.6 `StatusExt` and
`ErrorDetails`. Prefer fallible `check_error_details` extraction where malformed details must
remain distinguishable from absence; convenience `get_error_details` may erase that distinction.
This companion is independently versioned and is an eligible addition, not an installed
declaration asserted by this review. See its
[versioned API](https://docs.rs/tonic-types/0.14.6/tonic_types/trait.StatusExt.html).

The Buffa product-service route should generate compatible detail messages in its own
descriptor world rather than introducing a Tonic/Prost dependency just to emit a standard
detail. Custom scientific detail is appropriate when the standard vocabulary cannot express
the actual distinction. Neither protocol's generic `Internal` nor a single convergence enum
is an adequate scientific outcome record.

## Validation, field attribution and completeness

Generated Protovalidate violations can reuse structured field/rule paths instead of another
hand-maintained dotted-string convention. The useful composition is owner constraint →
`buf.validate` options → generated validation → typed violations → source-aware rendering.
Validation paths identify message fields; the declaration/source mapper still owns physical
source attribution, including repeated element identity and missing spans.

The [validation addendum](../../../../libraries/proto/protobuf_validation_rust.md) distinguishes
trusted full diagnostics from bounded public details, malformed from unrelated typed details,
descriptor-enriched paths from reflection-free validation, and truncation without an explicit
completeness flag. The target must make omission observable rather than inferring a complete
account from successful detail decode. Use the maintained generator's actual supported rules;
upstream conformance claims do not qualify local field-path/source composition.

For result metadata and large diagnostic pages, Buffa borrowed views and `ViewEncode` can avoid
constructing unrequested owned fields; `OwnedView` can retain a page buffer across async access.
Generation must push selection to the producer. Encoding the whole detailed native document
and then reading a small view has already paid most of the unwanted work. `Rope`/segmented
encoding is useful only if the transport retains segments and does not compress/flatten away
the advantage. [Representation](representation-reflection-and-validation.md) owns the exact
conditions. A low-copy view can extend buffer lifetime and worsen retained memory.

## Progress and result streams

Publish progress as attributable observations of the original operation; publish terminal
scientific outcomes from their performing owner; publish durable result references only after
the existing commit/acknowledgement contract. A connection reaching EOF is not automatically
the terminal operation result. A stream may yield valid partial data and later fail. Client
code must inspect the terminal protocol result and preserve a valid prefix with its incomplete
extent instead of either discarding it silently or treating it as complete.

Keep request correlation, operation identity, invocation identity, trace/span identity,
result-set identity and producer sequence distinct. `tower-http` request-ID layers are an
eligible way to attach and propagate HTTP correlation, but they do not mint scientific
operation identity. Reuse an existing issued request identity rather than making two
competing correlation schemes. Middleware order must insert the identifier before recording
it. Sensitive-header marking helps only when every formatter/exporter honors it.

`console-subscriber`/`tokio-console` offers an optional execution view of task poll/scheduling
and synchronization resources. It requires the documented Tokio tracing/`tokio_unstable`
configuration; installing a console client is insufficient. Its gRPC endpoint and task IDs are
separate diagnostic capabilities. Keep it explicitly prepared and local; connect task/trace
observations to existing operation facts without making the console a source of scientific
completion. [Local RPC §§9.1–9.3](../../../../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md)
and [console-subscriber 0.5.0](https://docs.rs/console-subscriber/0.5.0/console_subscriber/)
bound this candidate. No console deployment or quantitative overhead claim is established here.

## Alternatives and selected proposed realization

| Option | Whole-system consequence | Disposition |
| --- | --- | --- |
| Keep independent Serde/CLI/RPC diagnostic shapes | Easy local additions but repeated interpretation, detail-loss risks and client-specific failure parsing. | Retain useful native documents as semantic owners; generate their selected independent projections. |
| Put every failure and every result into one universal Proto envelope | Superficial uniformity obscures observation versus operation versus science and duplicates large Arrow payloads. | Reject as the default target. Use explicit related records and stable references instead. |
| Source-derived typed metadata/details with native evidence and Arrow payloads | Shared client interpretation, attributable omissions and reuse of standard external telemetry. Requires truthful mapping and protected evidence readers. | Recommended target. Remove superseded manual mapping and string parsing once complete consumers migrate. |

Retain Perfetto's supplied schema and Prost output. Retain OTLP as an external standard rather
than converting telemetry into PSE scientific records. A future common diagnostic projection
may feed both, but each export retains its original timestamp, extent and protocol rules.

## Revealing controls and handoff

Inspect or probe a failure with multiple source causes, missing source coordinates, malformed
recognized detail, unknown detail type and deliberately exceeded inline detail size. Require
independent consumers to distinguish each case. Exercise partial rows followed by terminal
failure, retained results read after a service disconnect, and observation expiry while the
scientific operation remains active. A deliberate source rename must not change domain
identity; a shared expression used twice must preserve two source occurrences.

Add a new solver termination and a new diagnostic product through their owner and generated
metadata, then verify that existing generic inspection does not reinterpret an unknown enum
as success. Keep numerical qualification and physical usability independent of transport
success. A dynamic inspector must show unsupported interpretation as unknown, not invent a
source span or scientific conclusion.

Sources comprise the linked native owners, the supplied
[Tonic](../../../../libraries/proto/tonic.md),
[Local RPC](../../../../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md),
[reflection/PBJSON](../../../../libraries/proto/prost_reflection_and_pbjson.md) and validation
additions, Context7 Tonic/Buffa lookups, and exact installed ConnectRPC 0.9.1 source.
Context7's Tonic lookup supplied general `Status` APIs but not the precise rich-detail companion;
the versioned companion reference therefore remains the explicit evidence route for that option.
Rendering, RPC interoperability, selected projection fidelity and overhead need their own
controls during implementation. No broad diagnostic or result qualification follows from this
source-based assessment.
