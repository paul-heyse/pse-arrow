# Representation, reflection and validation

Inspected 2026-11-03 at `f55951cfe995a689f9b75185b8e89cb362021616`, including
the supplied uncommitted Proto reference additions. Library and current-source mechanisms are
**Interface-checked**; the local target and benefits are **Proposed**. No product, conformance,
performance or conversion probe was run. SurrealDB v4 is excluded.

## Target: preserve meaning at the actual representation boundary

Use Buffa owned messages and generated views for first-party Proto contracts, a shared descriptor
world for inspection, and generated validation for locally decidable boundary rules. Keep native
domain objects authoritative during scientific execution. Use the existing Prost/Tonic stack
where an independently owned protocol requires it; cross stacks at an explicit contract boundary
rather than creating an in-memory second type universe for every message.

Reflection should make an interface understandable without hard-coded per-message inspectors.
It should not turn decoded field names into a second scientific model, authorize edits or replace
invariant-bearing construction. The target distinguishes four acts: decoding a representation,
checking its local constraints, adopting it into a known native interpretation, and admitting it
in the actual scientific/revision context.

## Existing mechanism and local applicability

`pse-model::boundary_projection` carries actual scalar, optional, repeated, set, map and alternative
types, their stable tags and `LocalRule`s. `ProjectionRecord`/`ProjectionAlternative` expose native
field tuples; generated adapters call the owner's constructor and preserve typed failures such
as `UnknownField`, `UnknownAlternative`, `DuplicateKey`, `InvalidValue` and `LayoutMismatch`.
This is substantive existing capability, not a proposal for a first generic conversion layer.

The host already generates vtable reflection, a shared descriptor pool, JSON/text and validation.
Its schema/payload readers interpret selected descriptor worlds. The engineering/reference
generation route already emits bounded local rules and invokes maintained validation preflight.
The remaining opportunity is broader reuse and complete consumer coverage, including diagnostics,
worker requests, retained metadata and independent readers, rather than adding another generic
representation layer beside the current one.

Representative source: [projection types and local rules](../../../../../crates/pse-model/src/boundary_projection.rs),
[native adoption failures](../../../../../crates/pse-model/src/boundary_projection/access.rs),
[rule/descriptor generation](../../../../../crates/pse-codegen/src/codegen/boundary_projection.rs),
[engineering generation](../../../../../xtask/src/codegen/engineering.rs), and
[schema readers](../../../../../crates/pse-host/src/schema/payload.rs).
CBO-195 and CBO-042 motivate sharing mechanical interpretation; CBO-040's registry products retain
their independent publication and semantic identity responsibilities.

## Descriptor and reflection mechanisms

### Own a complete, versioned schema world

Buffa 0.9.2 `DescriptorPool::decode_with_options` applies caller-selected decode limits before
linking schema metadata. The pool checks duplicate symbols, invalid field numbers, unresolved
references and other structural inconsistencies. It also imposes a 512-byte fully qualified symbol
limit and a 65,535-field-per-message bound. Encoded input size alone therefore does not describe
all schema materialization constraints.

`add_file_descriptor_set` stages additions transactionally against a clone and preserves existing
indices on success. Existing filenames are skipped: this is pool expansion, not a live replacement
protocol. Batch related additions, and issue a separate pool generation for a changed schema
world. Bind retained values/readers to that generation rather than assuming a mutable global pool
can reinterpret old messages safely.

`file_by_name`, `file_containing_symbol`, `service_by_name`, `extension_for`, `extensions_of` and
schema iterators can serve discovery without a hand-maintained parallel symbol index.
`DynamicMessage::from_options` exposes custom metadata from linked options, including extension
fields. This could carry source-derived quantity facets, interpretation versions, effect metadata
and inspectability hints. Those options describe the owner's meanings; they do not enforce units,
permissions or operation custody themselves.

For a Prost-owned protocol, `prost_reflect::DescriptorPool` and `DynamicMessage` supply the
corresponding generic interpretation route. Prefer an explicitly owned pool to process-global
registration when multiple retained interpretations may coexist. The 0.16.1 reflection builder
requires `descriptor_pool(...)` or `file_descriptor_set_bytes(...)`; `configure()` itself performs
compilation-related work, so it is not a passive setting operation. A descriptor-derived
`ReflectMessage` implementation can instead be attached to an already-generated type where that
avoids repeating compilation.

### Separate inspection from generic mutation

Generated Buffa vtable reflection exposes read-only `ReflectMessage` operations such as
`for_each_set`; a general generated `reflect_mut()` is not released in 0.9.2. Dynamic messages
support checked `ReflectMessageMut::try_set` and clearing, while `field_mut`/`field_by_number_mut`
return a mutable dynamic value for an existing field. Direct replacement through that reference
bypasses shape checks. The target uses checked mutation for generic transformations.

`DynamicMessage::try_from_message` and `to_message` bridge generated and dynamic values by wire
encoding and decoding. They are convenient for occasional inspection/redaction/transformation;
they are not a zero-cost way to mutate native/generated messages inside repeated numerical work.
Inspect generated data directly when vtable reads suffice, and do not materialize a dynamic
copy solely to enumerate fields.

Use field selection to communicate demanded detail before production. Treat `FieldMask` as a
selection notation whose update semantics the domain must define. Exact-parent editing still
requires typed intent for deletion, clearing, replacing collections and retaining inherited data.
Protobuf merge rules cannot supply that scientific editing policy.

## Information fidelity across formats and implementations

| Distinction | Target handling and reason |
| --- | --- |
| Absent/default/null | Retain explicit presence when it changes meaning. Proto3 implicit scalar defaults cannot express all native optional states; synthetic descriptor oneofs are presence machinery, not additional user choices. `Struct`/`Value` is not a replacement for a precise quantity or edit type. |
| Unknown field versus unknown enum | Forwarding can preserve an unknown representation; adoption must refuse information the native interpretation cannot retain. Generated Prost skips unknown fields but keeps enum fields as integer representations. Do not infer one behavior from the other. |
| Original bytes versus logical value | Encoding again can reorder fields, normalize aliases or defaults and change bytes while preserving a logical value. Archive original bytes separately when byte identity is promised. No protobuf encoder here establishes the semantic hashing contract. |
| Exact numeric meaning | Keep semantic IDs/hashes as fixed validated bytes and preserve integer/decimal/precision conventions explicitly. JSON numbers and `double` must not silently replace exact large integers or arbitrary precision values. |
| ProtoJSON versus native Serde JSON | Choose a declared format. ProtoJSON's enum names, integer strings, WKT handling and presence differ from ordinary derived JSON; a successful JSON parse does not prove interchangeable contracts. |
| `Any` versus extensibility admission | Resolve type URLs against an admitted descriptor world and bind interpretation. A discoverable type is not authorized executable behavior or a trusted scientific plugin. |

Prost 0.14.4 `Message::merge` mutates fields as it consumes input and can fail after earlier changes.
Decode into a fresh value, validate and adopt before publishing a replacement. Duplicate scalar,
message, repeated and map occurrences have their protobuf merge semantics, not a universal
duplicate-rejection policy. Wire decoding can also accept missing Proto2 required values without
establishing the presence contract a caller expected. Schema-aware presence controls remain
necessary where the distinction matters.

The supplied Prost addendum identifies a 0.14.4 implicit-packed encoding defect when unrelated
field options are present. Treat those schema shapes as a release-specific discriminator before
using Prost for new owned contracts; do not generalize it into a rejection of existing third-party
protocol traffic. Exact release controls, explicit packing or a verified fixed release can settle
that branch without redesigning domain semantics.

For JSON, use Buffa's generated JSON support on first-party types. Use `prost-reflect` for dynamic
descriptor-aware ProtoJSON and `pbjson-build::Builder::register_descriptors`/`build` when a
statically generated Prost protocol actually needs compiled JSON implementations. Align
`prost-build::extern_path` with `pbjson-types` only for the WKT mappings selected. The supplied
0.9.0 PBJSON reference documents incomplete descriptor-aware `Any` handling and a source-based
`FieldMask` concern: a blanket mapping of every WKT is not a justified interoperable route.

## Validation as a complete execution contract

### Choose the native generated route for stable first-party contracts

The recommended first-party route is source-owned local constraints → `buf.validate` options →
maintained `protoc-gen-protovalidate-buffa` preflight and generation → generated `Validate` on owned
messages/views. Invoke validation explicitly on every selected ingress and relevant outgoing
boundary. Conversion into a native type then uses the owner's local invariant checks; contextual
scientific/revision/transaction predicates remain with the domain owner.

The local generator is based on published 0.7.2 with explicit admission and arithmetic/error
corrections. Its README records refusal of unsupported required rules, exact mixed numeric
comparisons, overflow/conversion error preservation, and decisive boolean error masking. It is a
bounded typed CEL lowerer, not universal CEL support. The new overview's upstream-main claims do
not broaden that maintained local contract.

Separate ordinary `violations` from `compile_error` and `runtime_error` in
`protovalidate_buffa::ValidationError`. Compilation/evaluation failure cannot be relabelled as an
invalid user argument merely because ordinary violations are also present. Borrowed-view
validation is valuable where it avoids making an owned copy solely to reject a request; admission
of the retained bytes and later native work remains separate.

Keep module and naming controls consistent with message generation: `idiomatic_field_names`,
`strategy: all`, `packaging`, `file_per_package` and `proto_module` affect concrete generated field
paths and packaging. They are not cosmetic. Programmatic `scan::gather` plus
`emit::render_with_options` is available for a selected in-process generator composition.

### Retain a deliberate dynamic-validation alternative

For a genuinely runtime-selected Prost descriptor world, `prost-protovalidate` 0.6.0 offers
`ValidatorOption::MessageDescriptors` with `DisableLazy` to preload a closed validation population.
`AdditionalDescriptorSetBytes` supplies extension definitions. Per-call `ValidationOption::NowFn`
can make clock-relative rules explicit; `Filter` and `FailFast` change validation scope/evidence.
These are useful capabilities, not reasons to add runtime CEL to every stable first-party request.

`prost-protovalidate-build` supports descriptor bytes/path input, external Rust mappings, backend
selection, `fail_on_runtime_only(true)` and a Buffa runtime bridge. Default unsupported native
generation emits warnings and skips implementations; it is not an implicit runtime fallback.
Native-only selection should require complete coverage. A runtime bridge adds reflection/CEL,
descriptor embedding and layout constraints; it conflicts with native-only failure mode and
cannot freely combine with custom output placement. Compare the complete preparation and error
route before choosing it over the maintained dedicated Buffa generator.

Buf's `PROTOVALIDATE` lint can identify schema-level rule issues, but does not prove local backend
enforcement. Wire compatibility, validation-policy compatibility, generator coverage and actual
invocation coverage require distinct evidence. Allowing unknown constraint fields can improve
forward parsing while silently weakening what successful validation establishes; it cannot be
the target's completeness policy.

## Encoding and allocation mechanisms beyond views

Use these mechanisms at real boundaries where they eliminate work:

- `Message`/`ViewEncode::try_encode_bounded` and cache variants check encoded size before writing
  to the sink. Size-check failure leaves it unchanged. This can remove an extra temporary buffer
  used solely to establish a frame budget; it does not reserve every downstream allocation.
- `SizeCachePool::new` or `sequential` reuses spill storage beyond `SizeCache`'s inline slots.
  Pool capacity is in `u32` cache slots, not payload bytes. It amortizes allocation, not traversal;
  an undersized retention limit can cause repeated shrink/regrowth.
- `Rope::with_backing(Bytes)` can retain eligible borrowed wire slices with `Bytes::slice_ref`;
  `with_min_segment`, `into_segments` and `RopeBuf` expose a segmented output route. Headers and
  small fields still write; unrelated slices copy. Flattening, compression or a contiguous-only
  transport can remove the benefit, while retained slices extend the backing allocation's lifetime.
- Reader APIs check framed lengths before acquiring the full body, but still buffer serialized
  input before decoding. They are convenient bounded framing, not constant-memory field streaming.

Choose the scope that owns the buffers and its lifetime before introducing pooling. Numerical
callbacks must not acquire descriptor interpretation, RPC or encoding merely because those APIs
exist. Demand selection before building an unused diagnostic/result is generally more valuable
than making serialization of that unused detail faster. Quantitative benefit remains unmeasured.

## Countercases and proposed acceptance controls

Use native-only types when there is no separately interpreted or persistent contract. Retain Arrow
for selected bulk scientific products where its existing representation and readers fit. Retain
Prost/Tonic for externally specified protocols without translating all their values to Buffa.
Do not add PBJSON beside Buffa JSON for the same contract merely to standardize package names.

Before implementation acceptance, discriminate absent/default/null; unknown nested fields and
enum values; duplicate keys; large exact integers; invalid dynamic mutation; malformed tails after
successful merge prefixes; validation compilation/evaluation errors; new constraints unknown to
an old backend; and buffers retained across async waits. Compare original bytes, logical values
and native adopted meaning separately. Unsupported scientific interpretation must refuse
attributably rather than defaulting or becoming a superficially valid message.

## Sources and evidence limits

Context7 lookup preceded exact installed-source inspection of Buffa 0.9.2 reflection/codegen and
Prost 0.14.4 merge. The supplied addenda provide broader precise release leads. Claims about
unadopted validator/PBJSON alternatives are interface-level research, not local integration tests.
Context7 did not resolve the specific Rust validator or reflection companions; direct released
documentation supplied the follow-up. The 0.6.0 validator options and native-builder pages were
retrieved directly, confirming closed descriptor selection and runtime-bridge feature, output
layout and conflicting-option constraints. The web renderer failed for those two URLs; direct
HTTPS retrieval succeeded. PBJSON limitations retain the supplied reference's source-based
evidence scope rather than receiving fresh runtime confirmation.

- [Buffa 0.9.2 dynamic-message source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs) and [reflection traits](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/message.rs).
- [Buffa 0.9.2 pool](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/pool.rs), [encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs), [size cache](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/size_cache.rs), and [segmented sinks](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs).
- [Prost 0.14.4 message source](https://docs.rs/prost/0.14.4/src/prost/message.rs.html) and [prost-reflect 0.16.5 dynamic message](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html).
- [Reflection builder 0.16.1 source](https://docs.rs/prost-reflect-build/0.16.1/src/prost_reflect_build/lib.rs.html), [PBJSON builder 0.9.0](https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html), and [ProtoJSON specification](https://protobuf.dev/programming-guides/json/).
- [Maintained local validation contract](../../../../../vendor/protoc-gen-protovalidate-buffa/README.md), [runtime validator 0.6.0 options](https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidatorOption.html), and [native generation builder](https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/struct.Builder.html).
- Supplied [Buffa](../../../../libraries/proto/Buffa.md), [Prost](../../../../libraries/proto/prost.md), [reflection/PBJSON](../../../../libraries/proto/prost_reflection_and_pbjson.md), and [validation](../../../../libraries/proto/protobuf_validation_rust.md) references retain their original research and addendum dates.
