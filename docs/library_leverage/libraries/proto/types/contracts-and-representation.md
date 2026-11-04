# Descriptor composition and faithful representation

Research date: 2026-11-03. **Interface-checked** library contracts; no executable probe,
integration qualification or performance measurement. These reusable types are independent of
any particular repository, domain owner or adoption decision.

## `proto/descriptor-corpus-generation`

**Need.** Several generated consumers need the same message identities, custom options and
representation settings without compiling or interpreting independently authored mirrors.

**Mechanism.** Compile a complete import world once; pass its `FileDescriptorSet` to compatible
generators. Buffa 0.9.2 `generate_with_diagnostics` returns artifacts plus structured warnings.
`GeneratedFileKind::Companion` with `apply_companions` can mount extra generated source in a
matching package module. `SharedCorpusContext` supports reuse of schema-global analysis;
`FdsEmbedding` and a shared pool root avoid repeating descriptor embedding. Prost-build 0.14.4
`Config::compile_fds` generates from prepared descriptors; `skip_protoc_run()` consumes an
already produced descriptor artifact.

**Discharged obligations.** Compiler-owned symbol/import resolution; generator-owned Rust
representation and module scaffolding; schema-derived projection consistency for the selected
complete descriptor world.

**Residual obligations.** Semantic declaration ownership, complete compiler/backend feature
compatibility, external Rust type mapping, warning policy, artifact publication and consumer
validation. Companion inclusion is non-idempotent and requires a matching `PackageMod`; absent
matches remain files with no automatically emitted include. Filenames and package membership
must be checked before composition.

**Important limit.** Buffa's `encode_descriptor_set` strips `source_code_info` and applies
feature overrides; `shared_descriptor_root_module` embeds caller-supplied bytes. Custom
embedding can retain source info or omit effective overrides. Inspect the actual producer;
source comments and genuine source provenance remain different responsibilities.

Sources: [Buffa 0.9.2 generation source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs),
[descriptor embedding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/reflect.rs),
and [Prost-build 0.14.4 configuration](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html).

## `proto/versioned-reflection-world`

**Need.** Generic inspection, schema distribution and controlled transformations must interpret
messages without a hard-coded type visitor for every published schema.

**Mechanism.** Buffa 0.9.2 `DescriptorPool::decode_with_options` links a selected corpus.
`file_by_name`, `file_containing_symbol`, `service_by_name` and extension lookups expose schema
identity and membership. `DynamicMessage::from_options` reads linked custom options.
`add_file_descriptor_set` stages expansion transactionally but skips existing filenames;
replacement schemas therefore need deliberately distinct pool generations. `prost-reflect`
0.16.5 supplies analogous descriptor-backed dynamic message processing for Prost consumers.

**Discharged obligations.** Linked protobuf metadata, supported descriptor structural checks,
generic field/schema access and schema-aware dynamic decoding.

**Residual obligations.** Trust, type admission, descriptor provenance, interpretation versions,
resource limits and identity of the pool associated with retained values. A valid type URL or a
linked pool does not authorize effects or establish domain meaning.

**Mutation and conversion limit.** Buffa generated vtable reflection is read-only in 0.9.2.
Dynamic `ReflectMessageMut::try_set`/`try_clear` check selected field operations; direct mutation
through an `&mut Value` bypasses shape checks. Generated/dynamic bridges encode and decode;
they incur real work and do not make domain edits transactional or semantically valid.

Sources: [Buffa pool](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/pool.rs),
[reflection traits](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/message.rs),
[dynamic implementation](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs),
and [prost-reflect 0.16.5](https://docs.rs/prost-reflect/0.16.5/prost_reflect/).

## `proto/bounded-segmented-encoding`

**Need.** A producer must reject oversized messages without corrupting its existing output
buffer, or inspect and forward large fields without unnecessary owned copies.

**Mechanism.** Buffa 0.9.2 `Message` and `ViewEncode` provide bounded fallible encoding and cache
variants. Size computation and budget validation precede the write pass. `SizeCachePool`
reuses spill buffers; its retained-capacity unit is `u32` cache slots. `Rope::with_backing(Bytes)`
can capture eligible borrowed slices from that exact backing allocation; `into_segments` and
`RopeBuf` expose a segmented `Bytes`/`Buf` route.

**Discharged obligations.** Size-check failure atomicity for the sink; supported cache reuse;
reference-counted ownership of eligible original-buffer slices.

**Residual obligations.** Transport framing, total allocation admission, lifetime/retention
accounting, segmentation preservation and logical interpretation. Small fields and headers
still write. Compression or flattening may copy. Retaining a small slice may retain a much
larger backing allocation. No end-to-end zero-copy or speed benefit follows without examining
the complete consumer route.

Sources: [message encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs),
[cache storage](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/size_cache.rs), and
[segmented sinks](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs).

## `proto/representation-adoption-separation`

**Need.** A receiver must distinguish successful wire decoding from complete information
preservation, local validation and adoption into an invariant-bearing domain type.

**Mechanism.** Use a known message/descriptor interpretation, explicit presence and unknown-field
policy, local generated/runtime validation, then fallible native construction. Decode a fresh
Prost 0.14.4 value before publication when failure atomicity matters: `Message::merge` updates
fields incrementally and is not transactional. Generated Prost skips unknown fields, while enum
numbers can survive as integer representations; those are different fidelity contracts.

**Discharged obligations.** Codec-level wire structure and the constraints actually implemented
and invoked by the selected validator.

**Residual obligations.** Exact numeric meaning, original-byte preservation, semantic identity,
contextual validity, authorization and coherent publication. ProtoJSON and ordinary Serde JSON
are distinct formats. `FieldMask` requires a defined mutation policy. Wire compatibility does
not establish validation-policy or domain-interpretation compatibility.

Sources: [Prost 0.14.4 message source](https://docs.rs/prost/0.14.4/src/prost/message.rs.html),
[generated decoding source](https://docs.rs/prost-derive/0.14.4/src/prost_derive/lib.rs.html), and
[ProtoJSON specification](https://protobuf.dev/programming-guides/json/).
