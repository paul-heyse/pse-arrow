# Prost Reflection and Protobuf JSON — Rust Library Reference

Libraries: `prost-reflect`, `prost-reflect-build`, `pbjson`, `pbjson-build`, `pbjson-types`\
Research date: October 31, 2026\
Scope: Current stable releases, architectural capabilities, advanced APIs, integration mechanisms, and important semantic limitations.

## 1. Executive orientation

The `prost-reflect` and `pbjson` families extend Rust's `prost` Protocol Buffers implementation in two complementary directions:

- `prost-reflect` adds runtime schema intelligence: protobuf descriptor inspection, dynamically typed messages, field-level operations, extension handling, binary transcoding, and schema-driven JSON serialization.
- `pbjson` adds statically generated protobuf JSON support: schema-aware `serde` implementations for ordinary `prost` message structs, without requiring runtime reflection for every serialization operation.
- Their build-time companions connect both capabilities to a shared protobuf descriptor set, allowing one schema to support strongly typed Rust code, runtime reflection, and protobuf-compatible JSON representations.

The fundamental architectural distinction is between interpreting a protobuf schema at runtime and compiling schema-aware serialization behavior into Rust code. Both approaches can coexist and serve different operational needs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 1.1 Package map and verified releases

| Crate                 | Stable version | Primary responsibility                                                                                |
| --------------------- | -------------- | ----------------------------------------------------------------------------------------------------- |
| `prost-reflect`       | 0.16.5         | Descriptor pools, message reflection, dynamic values, encoding/decoding, JSON and text-format support |
| `prost-reflect-build` | 0.16.1         | Generate `ReflectMessage` implementations and descriptor artifacts alongside `prost-build`            |
| `pbjson`              | 0.9.0          | Runtime support for generated protobuf-JSON Serde implementations                                     |
| `pbjson-build`        | 0.9.0          | Generate `Serialize` and `Deserialize` implementations from protobuf descriptors                      |
| `pbjson-types`        | 0.9.0          | Protobuf well-known types with JSON-aware Serde implementations                                       |

These are separately versioned crates, not a single unified release. In particular, `prost-reflect-build` does not have to share the exact patch version of `prost-reflect`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+4



Primary documentation:

- `prost-reflect`: [https://docs.rs/prost-reflect/latest/prost_reflect/](https://docs.rs/prost-reflect/latest/prost_reflect/?utm_source=chatgpt.com)
- `prost-reflect-build`: [https://docs.rs/prost-reflect-build/latest/prost_reflect_build/](https://docs.rs/prost-reflect-build/latest/prost_reflect_build/?utm_source=chatgpt.com)
- `pbjson`: [https://docs.rs/pbjson/latest/pbjson/](https://docs.rs/pbjson/latest/pbjson/?utm_source=chatgpt.com)
- `pbjson-build`: [https://docs.rs/pbjson-build/latest/pbjson_build/](https://docs.rs/pbjson-build/latest/pbjson_build/?utm_source=chatgpt.com)
- `pbjson-types`: [https://docs.rs/pbjson-types/latest/pbjson_types/](https://docs.rs/pbjson-types/latest/pbjson_types/?utm_source=chatgpt.com)

### 1.2 The shared architectural model

Protobuf schema definitions (`.proto`)

Messages · enums · fields · oneofs · services · extensions · options

`protoc` / `prost-build`

FileDescriptorSet

Machine-readable compiled schema metadata

prost-build

Generated Rust message types

prost-reflect

Runtime descriptors and dynamic messages

pbjson-build

Generated JSON Serde implementations

Conceptual capability flow. The build companions configure generation, while the runtime crates consume generated code or descriptor metadata.

A `FileDescriptorSet` is particularly consequential because it provides a machine-readable description of protobuf types independently of their Rust implementations.

This supports several broader capabilities:

- A single source of schema information for both generated and dynamically interpreted messages.
- Schema-driven inspection of externally supplied protobuf messages.
- Metadata-driven tooling without hard-coding every message type.
- Protobuf-to-JSON conversion using either runtime reflection or compiled Serde implementations.
- Runtime inspection of services and methods, independently of RPC transport implementation.

The descriptor set is metadata, not a complete application schema registry, RPC server, validation engine, or persistent data store. Those additional responsibilities remain with surrounding systems.

## 2. `prost-reflect`: runtime schema metadata

### 2.1 `DescriptorPool` as a compiled schema graph

`DescriptorPool` is the central runtime schema object. It resolves protobuf names and relationships across a collection of file descriptors, exposing messages, enums, services, methods, extensions, and their metadata through typed descriptor handles.

Its principal entry points are:

| Operation               | Relevant APIs                                                                             | Significance                                                                 |
| ----------------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Create a pool           | `new`, `decode`, `from_file_descriptor_set`                                               | Construct a schema environment independently of generated Rust message types |
| Incrementally extend it | `add_file_descriptor_proto(s)`, `add_file_descriptor_set`, `decode_file_descriptor_set`   | Combine descriptor sets or assemble one from constituent schemas             |
| Resolve named types     | `get_message_by_name`, `get_enum_by_name`, `get_service_by_name`, `get_extension_by_name` | Dynamic name-to-schema resolution                                            |
| Enumerate a schema      | `files`, `all_messages`, `all_enums`, `services`, `all_extensions`                        | Discover all schema elements available in the pool                           |
| Re-export metadata      | `encode`, `encode_to_vec`, `file_descriptor_protos`                                       | Transfer or retain descriptor information                                    |
| Use shared built-ins    | `global`, `decode_global_file_descriptor_set`                                             | Work with a process-wide pool containing protobuf well-known descriptors     |

Important distinction: raw descriptor bytes versus decoded descriptor structs. `DescriptorPool::decode` and `decode_file_descriptor_set` preserve custom extension options encoded in the descriptors. Constructing the pool from `prost_types::FileDescriptorSet` does not necessarily preserve those options, because the intermediate generated representation can discard unknown extension fields. For metadata-rich schemas, this difference is consequential.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html?utm_source=chatgpt.com)

The pool is inexpensive to clone because it uses internal reference counting. However, modifying a cloned pool does not update previously created clones. This makes descriptor handles useful as references into a stable schema view, but it also means a system with evolving schemas must explicitly manage which pool version each operation uses.

Another easily overlooked constraint: adding a descriptor file whose name already exists skips the duplicate, even if the new file contains a different schema. Consequently, repeated loading is not equivalent to replacing a schema, and schema version conflicts need to be handled outside the pool.

For individually added files, referenced imports must already be available; when adding multiple file descriptors as a collection, those files may be supplied in arbitrary order.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.2 Descriptor classes: structural and semantic inspection

The descriptor API is substantially richer than extracting field names and types.

| Descriptor                               | Important information exposed                                                                                                          |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `FileDescriptor`                         | File name, package, syntax, imports and public imports, top-level declarations, source locations, file options                         |
| `MessageDescriptor`                      | Fully qualified name, nested messages/enums, fields, oneofs, map-entry identity, extension ranges, reserved names/ranges, options      |
| `FieldDescriptor`                        | Tag number, protobuf name, JSON name, `Kind`, cardinality, presence, packing, map/list status, oneof ownership, default value, options |
| `EnumDescriptor` / `EnumValueDescriptor` | Symbol names, numeric values, reserved declarations, enum/value options                                                                |
| `OneofDescriptor`                        | Membership, name, options, synthetic-oneof distinction                                                                                 |
| `ServiceDescriptor` / `MethodDescriptor` | Service and method names, input/output message descriptors, client/server streaming flags, options                                     |
| `ExtensionDescriptor`                    | Extending message, field number, kind, cardinality, name, options                                                                      |

This enables schema-aware inspection of message structure and RPC contracts, including nested and cross-file relationships. Services and methods are inspectable as metadata; `prost-reflect` does not itself serve gRPC or ConnectRPC requests.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+4



Sources:

- [https://docs.rs/prost-reflect/latest/prost_reflect/struct.MessageDescriptor.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.MessageDescriptor.html?utm_source=chatgpt.com)
- [https://docs.rs/prost-reflect/latest/prost_reflect/struct.FieldDescriptor.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.FieldDescriptor.html?utm_source=chatgpt.com)
- [https://docs.rs/prost-reflect/latest/prost_reflect/struct.ServiceDescriptor.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.ServiceDescriptor.html?utm_source=chatgpt.com)
- [https://docs.rs/prost-reflect/latest/prost_reflect/struct.MethodDescriptor.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.MethodDescriptor.html?utm_source=chatgpt.com)

Two structural distinctions deserve particular attention.

Field type versus field cardinality. `Kind` describes the underlying protobuf value category, including references to message or enum descriptors. `Cardinality` and additional field metadata establish whether a field is optional, required, repeated, packed, or represented as a map. Treating `Kind` alone as the full type definition would lose important schema information.

Synthetic versus real oneofs. Proto3 `optional` fields use synthetic oneof declarations in their descriptor representation. They are not semantically equivalent to mutually exclusive application-level alternatives. `OneofDescriptor::is_synthetic` makes this distinction inspectable, avoiding the need to infer it solely from descriptor structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 2.3 Custom protobuf options as machine-readable annotations

Protobuf custom options can attach domain-specific metadata to schema elements. `prost-reflect` supports this through extension descriptors and through descriptor `options()` methods, which return options as dynamically inspectable messages.

For example, a schema can define annotations for a message, field, or RPC method. At runtime, a consumer can resolve the corresponding extension descriptor and inspect the option value rather than parsing comments or maintaining an external mapping.

This is important because options can bind declarative metadata directly to the protobuf element to which they apply. It enables general-purpose schema annotation and introspection machinery without requiring generated, hand-maintained Rust types for every custom option.

The capability has two prerequisites: the relevant extension definitions must be present in the descriptor pool, and the encoded option values must survive descriptor loading. The raw-byte loading distinction in §2.1 is therefore especially relevant here.

Boundary: `prost-reflect` exposes and interprets schema annotations; it does not automatically enforce application-specific policies expressed by those annotations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 3. `DynamicMessage`: runtime-defined protobuf values

### 3.1 Message construction, binary decoding, and field operations

`DynamicMessage` represents a protobuf instance whose structure is determined by a `MessageDescriptor`, rather than by a generated Rust struct.

It supports `new`, schema-guided binary `decode`, runtime field access, mutation, extensions, and `prost::Message` binary encoding/merging. Its underlying value model includes scalar values, bytes, enum numbers, nested messages, lists, and maps.

For a message definition obtained at runtime, a caller can:

1. Resolve its `MessageDescriptor` from a pool.
2. Decode the binary payload with `DynamicMessage::decode`.
3. Inspect, modify, or construct fields through descriptor-aware APIs.
4. Re-encode the message through the normal `prost::Message` interface.

The field APIs support descriptor, name, and tag-number addressing. `fields()` iterates fields considered set; `get_field()` can retrieve a field's default value even when it is not set. Extensions have a parallel family of `get_extension`, `set_extension`, `has_extension`, `clear_extension`, and iteration operations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html?utm_source=chatgpt.com)

### 3.2 `Value`, `MapKey`, and typed access

A `DynamicMessage` stores fields as runtime `Value` instances, rather than arbitrary JSON values.

The principal `Value` variants include `Bool`, `I32`, `I64`, `U32`, `U64`, `F32`, `F64`, `String`, `Bytes`, `EnumNumber`, `Message`, `List`, and `Map`.

An enum is represented by its numeric value, allowing the runtime model to represent numeric enum values that are not declared as named variants. Lists and maps recursively contain `Value` instances, with protobuf-compatible `MapKey` types for map keys.

A `Value` can be checked against a field's descriptor using `is_valid_for_field`. This checks whether the value can be encoded for that protobuf field; it is not comprehensive business validation and does not imply that an enum number must have a declared symbolic name.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/prost-reflect/latest/prost_reflect/enum.Value.html](https://docs.rs/prost-reflect/latest/prost_reflect/enum.Value.html?utm_source=chatgpt.com)

The non-panicking mutation APIs are significant for dynamic systems:

| API                                                 | Behavior                                                             |
| --------------------------------------------------- | -------------------------------------------------------------------- |
| `set_field`                                         | Assign a value; may panic if the value is incompatible               |
| `try_set_field`                                     | Return `SetFieldError` for an incompatible value                     |
| `set_field_by_name` / `set_field_by_number`         | Do nothing if the named or numbered field does not exist             |
| `try_set_field_by_name` / `try_set_field_by_number` | Report missing fields as well as incompatible values                 |
| `get_field_mut`                                     | Materialize a default value when necessary and return mutable access |
| `take_field` / `take_fields`                        | Remove field values while transferring ownership                     |

For tools consuming runtime-provided paths or schemas, the `try_` variants provide a more reliable error boundary than the infallible setters.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.3 Presence, default values, and oneofs

Presence is a protobuf semantic property, not merely a question of whether a Rust value exists.

For fields with explicit presence, an explicitly set scalar value can be distinguished from an unset value even when the scalar equals its default. For implicit-presence scalar fields, assigning the default value does not make the field present.

For example:

```
syntax = "proto3";

message Sample {
  int32 count = 1;
  optional int32 override_count = 2;
}
```

After setting both fields to zero, `count` is treated as absent for ordinary protobuf encoding, while `override_count` can remain explicitly present.

`FieldDescriptor::supports_presence()` and `DynamicMessage::has_field()` make this distinction available at runtime. This matters for partial updates, merges, default emission, and conversion between message representations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.4 Unknown binary fields and forward compatibility

A major advantage of `DynamicMessage` over a simple schema-to-JSON conversion is its handling of unknown protobuf wire fields.

When decoding a binary message, a field number absent from the current descriptor can be retained as an `UnknownField`. `unknown_fields()` exposes those fields, and binary re-encoding preserves them. `take_unknown_fields()` permits explicit extraction and removal.

This allows a runtime using an older schema to inspect known fields and re-encode the message without necessarily discarding newly introduced wire fields.

However, there are two distinct forms of forward compatibility:

- Unknown wire fields: retain their raw wire representation without knowing their meaning.
- Known extensions: require an extension descriptor and can be interpreted as typed values.

The former does not imply the latter. Nor does binary preservation imply JSON preservation: protobuf JSON has no equivalent representation for arbitrary unknown wire fields.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 3.5 Bridging static and dynamic messages

`DynamicMessage::transcode_from<T: prost::Message>` and `transcode_to<T: prost::Message + Default>` provide conversion between generated Rust messages and dynamic representations.

The mechanism is protobuf binary transcoding: encoding with one representation and decoding with the other. It is therefore not a zero-copy reinterpretation of a generated Rust struct.

This creates a useful boundary between strongly typed application logic and generic introspection or transformation logic. It also introduces real costs and conditions: serialization work, allocations, compatible schemas, and potentially different treatment of fields unknown to the generated Rust type.

The `ReflectMessage` trait further connects generated messages to their schema descriptors, including a `transcode_to_dynamic()` convenience method.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

- [https://docs.rs/prost-reflect/latest/prost_reflect/trait.ReflectMessage.html](https://docs.rs/prost-reflect/latest/prost_reflect/trait.ReflectMessage.html?utm_source=chatgpt.com)
- [https://docs.rs/prost-reflect/latest/prost_reflect/struct.DynamicMessage.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.DynamicMessage.html?utm_source=chatgpt.com)

### 3.6 Protobuf text format

With the optional `text-format` feature, `prost-reflect` can parse and render protobuf's text representation.

Relevant APIs include `DynamicMessage::parse_text_format`, `merge_text_format`, `to_text_format`, and `to_text_format_with_options`.

`FormatOptions` controls pretty printing, field ordering, default emission, unknown-field rendering, and expansion of `google.protobuf.Any` values. This is useful for diagnostic output, inspecting unfamiliar messages, authoring test fixtures, and manipulating protobuf data in a human-readable form.

Text format is distinct from both binary protobuf and protobuf JSON. It should not be treated as an interchangeable implementation of either.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/prost-reflect/latest/prost_reflect/text_format/struct.FormatOptions.html](https://docs.rs/prost-reflect/latest/prost_reflect/text_format/struct.FormatOptions.html?utm_source=chatgpt.com)

## 4. `prost-reflect-build`: reflection for generated Rust types

Reflection need not require converting all messages into `DynamicMessage`. `prost-reflect-build` integrates with `prost-build` so that generated message structs implement `prost_reflect::ReflectMessage`.

This makes a generated type capable of reporting its message descriptor while preserving its normal, statically typed Rust interface.

The `Builder` supports three primary generation arrangements:

| Build arrangement                      | API                          | Purpose                                                                                                 |
| -------------------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------- |
| Convenience generation                 | `compile_protos`             | Compile protobuf sources with reflection support                                                        |
| Customized generation                  | `compile_protos_with_config` | Retain a configured `prost_build::Config`                                                               |
| Separate configuration and compilation | `configure`                  | Add reflection-related generation settings to an existing `prost_build::Config` that will compile later |

It also supports two ways to locate the descriptor metadata used by generated implementations:

- `descriptor_pool(...)` refers to a Rust expression yielding a `DescriptorPool`, allowing an application-owned, lazily initialized pool.
- `file_descriptor_set_bytes(...)` refers to encoded descriptor bytes, which the generated integration can use for reflection.

`file_descriptor_set_path(...)` controls where the build writes the descriptor set; its default is `$OUT_DIR/file_descriptor_set.bin` and it overrides the equivalent path supplied through `prost_build::Config`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/prost-reflect-build/0.16.1/prost_reflect_build/struct.Builder.html](https://docs.rs/prost-reflect-build/0.16.1/prost_reflect_build/struct.Builder.html?utm_source=chatgpt.com)

The key architectural benefit is using the same generated message model for conventional Rust code and descriptor-driven tooling. There is no need to replace generated structs with dynamic messages merely to discover their schema.

The derived interface is still reflection metadata, not a universal dynamically indexed API for arbitrary generated structs. When generic field-by-field access and mutation are required, the dynamic representation supplies that capability.

## 5. Protobuf JSON through `prost-reflect`

The optional `serde` feature gives `DynamicMessage` schema-aware JSON serialization and deserialization. This is not ordinary Serde derivation over a Rust representation: the implementation interprets the protobuf descriptor to apply the Protocol Buffers JSON mapping.

### 5.1 JSON serialization and deserialization

A `DynamicMessage` implements `serde::Serialize` with the feature enabled. Deserialization requires the message schema, supplied either through `DynamicMessage::deserialize(descriptor, deserializer)` or through the descriptor-based Serde seed interface.

The latter distinction matters: `DynamicMessage` does not have a universal `serde::Deserialize` implementation that can determine its protobuf type from arbitrary JSON alone. Unlike a generated Rust type, its runtime representation cannot know the intended protobuf schema without additional context.

An illustrative conversion is:

```
use prost_reflect::{DescriptorPool, DynamicMessage};

let pool = DescriptorPool::decode(descriptor_bytes)?;
let descriptor = pool.get_message_by_name("example.Event").unwrap();

// Protobuf binary -> dynamic representation -> JSON
let message = DynamicMessage::decode(descriptor.clone(), protobuf_bytes)?;
let json = serde_json::to_string(&message)?;

// JSON -> dynamic representation
let mut deserializer = serde_json::Deserializer::from_str(&json);
let restored = DynamicMessage::deserialize(descriptor, &mut deserializer)?;
deserializer.end()?;
```

This example is illustrative; `descriptor_bytes` and `protobuf_bytes` represent externally supplied buffers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 5.2 Serialization configuration

`SerializeOptions` provides runtime-selectable protobuf JSON formatting behavior:

| Option                      | Default | Effect                                                                   |
| --------------------------- | ------- | ------------------------------------------------------------------------ |
| `stringify_64_bit_integers` | `true`  | Emit 64-bit integer values as strings to preserve JSON numeric precision |
| `use_enum_numbers`          | `false` | Emit numeric enum values instead of symbolic names                       |
| `use_proto_field_name`      | `false` | Use original `.proto` field names rather than JSON names                 |
| `skip_default_fields`       | `true`  | Omit fields treated as unset/default under the library's presence rules  |

`DeserializeOptions::deny_unknown_fields` defaults to `true`. Disabling it skips unrecognized JSON fields rather than retaining them for a later schema version.

The separation of the two options objects is useful: serialization policy and acceptance policy can be selected independently at runtime, rather than encoded into generated implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

- [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.SerializeOptions.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.SerializeOptions.html?utm_source=chatgpt.com)
- [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DeserializeOptions.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DeserializeOptions.html?utm_source=chatgpt.com)

### 5.3 Well-known types and `Any` resolution

The protobuf JSON mapping gives several well-known types special representations. `prost-reflect` implements schema-aware handling for these rather than treating every protobuf message as an ordinary JSON object.

The most important example is `google.protobuf.Any`, whose standard JSON representation embeds a type URL and the decoded contents of the referenced message:

```
{
  "@type": "type.googleapis.com/example.Event",
  "eventId": "1234567890123456789"
}
```

This requires a descriptor for the embedded message to interpret its binary payload. The pool therefore acts not only as metadata for the outer message but also as the runtime type resolver for nested, type-erased protobuf content.

`prost-reflect` supports protobuf JSON `Any` mappings, including embedded well-known-type representations. Support is documented in its release history. The supported type URL domains have also been relaxed in earlier releases.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



A crucial distinction remains: protobuf's `Any` is not an arbitrary Rust trait object. It contains a protobuf type URL and an encoded protobuf message. The descriptor pool supplies schema interpretation; it does not dynamically load executable Rust implementations.

## 6. `pbjson` and `pbjson-build`: compiled protobuf JSON implementations

Where `prost-reflect` interprets descriptors at runtime, `pbjson-build` uses descriptors during code generation to produce Serde implementations for ordinary `prost` message types.

This avoids requiring a `DescriptorPool` or `DynamicMessage` on the normal JSON serialization path.

### 6.1 Division of responsibilities

`pbjson-build` is the generator. It consumes a compiled `FileDescriptorSet` and emits Rust implementations of `serde::Serialize` and `serde::Deserialize`.

`pbjson` is the supporting runtime crate used by the generated code. It is not a reflection engine, a standalone protobuf descriptor registry, or a direct substitute for `serde_json`.

`prost` and `prost-build` continue to own ordinary protobuf message types and binary serialization.

This separation is valuable because JSON behavior is derived from protobuf schema metadata without changing the core `prost` message model. The resulting Rust structs remain usable with strongly typed methods and the ordinary `prost::Message` interface.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 6.2 Build pipeline

The documented generation sequence is:

1. Compile `.proto` definitions with `prost-build`, retaining a `FileDescriptorSet`.
2. Register those descriptor bytes with `pbjson_build::Builder::register_descriptors`.
3. Select protobuf packages or type prefixes with `Builder::build`.
4. Include the generated `*.rs` and `*.serde.rs` files in the same Rust module.

For example, an illustrative `build.rs` is:

```
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("proto_descriptor.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        .compile_protos(&["protos/event.proto"], &["protos"])?;

    let descriptors = fs::read(descriptor_path)?;

    pbjson_build::Builder::new()
        .register_descriptors(&descriptors)?
        .build(&[".example"])?;

    Ok(())
}
```

In the generated package module:

```
include!(concat!(env!("OUT_DIR"), "/example.rs"));
include!(concat!(env!("OUT_DIR"), "/example.serde.rs"));
```

The exact package name and generated filenames depend on the schema. The well-known-type remapping is the pattern shown by upstream; it is not a universal requirement for every protobuf project.

Unlike `#[derive(Serialize, Deserialize)]`, the generated Serde implementations implement protobuf-specific JSON rules, including field naming, integer formatting, enum representation, and protobuf presence semantics. Using a second Serde derive on the same types would produce conflicting implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/pbjson-build/0.9.0/pbjson_build/](https://docs.rs/pbjson-build/0.9.0/pbjson_build/?utm_source=chatgpt.com)

### 6.3 Configuration beyond the basic generator

`pbjson-build::Builder` exposes several controls that are easy to overlook:

| Capability                    | API                                 | Architectural consequence                                               |
| ----------------------------- | ----------------------------------- | ----------------------------------------------------------------------- |
| Register descriptor bytes     | `register_descriptors`              | Generate from an existing descriptor-set artifact                       |
| Register decoded descriptors  | `register_file_descriptor`          | Integrate programmatically assembled descriptor metadata                |
| Select packages/types         | `build(&[prefixes])`                | Generate only implementations within specified protobuf namespaces      |
| Exclude selected types        | `exclude`                           | Avoid conflicting implementations or defer to externally supplied types |
| Custom output sink            | `generate(prefixes, write_factory)` | Integrate with alternative generation/output systems                    |
| Output directory              | `out_dir`                           | Control artifact placement                                              |
| Match `BTreeMap` fields       | `btree_map`                         | Keep Serde code aligned with `prost-build` map representation           |
| Match external Rust types     | `extern_path`                       | Respect types generated elsewhere or provided by another crate          |
| Match enum variant names      | `retain_enum_prefix`                | Coordinate enum naming with `prost-build`                               |
| Emit enum numbers             | `use_integers_for_enums`            | Select numeric rather than symbolic enum output                         |
| Emit default-valued fields    | `emit_fields`                       | Adjust default omission behavior                                        |
| Preserve `.proto` field names | `preserve_proto_field_names`        | Override default JSON field naming                                      |
| Ignore unknown JSON fields    | `ignore_unknown_fields`             | Permit schema-forward input with lossy omission                         |
| Ignore unknown enum variants  | `ignore_unknown_enum_variants`      | Replace unrecognized enum variants with the default                     |

The last two controls change deserialization semantics, not just formatting. In particular, substituting a default for an unknown enum value can lose meaning that protobuf binary encoding might otherwise preserve numerically.

These are generation-time choices: the generated implementations embody the selected behavior. They do not expose the equivalent per-message runtime options offered by `prost-reflect`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html](https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html?utm_source=chatgpt.com)

### 6.4 Code generation as a performance and maintenance boundary

The `pbjson` approach makes the message schema available to the Rust compiler through generated implementations. It can therefore avoid runtime descriptor lookup and dynamic value dispatch on the ordinary JSON path.

This is a structural advantage for a known set of message types. It does not, by itself, establish a measured throughput advantage over `prost-reflect` in every workload. The relative cost depends on message shape, JSON serializer, allocation patterns, schema reuse, and whether dynamic inspection is also required.

The main tradeoff is that changes to serialization policy or message schema may require regeneration and recompilation. In return, normal application code can remain statically typed.

## 7. `pbjson-types`: well-known protobuf types with Serde

`pbjson-types` supplies Rust implementations of Google's `google.protobuf` types with Serde support intended for the protobuf JSON mapping.

Its role is broader than just `Timestamp` and `Duration`. It also provides wrappers, JSON-structured values, descriptor messages, and other well-known protobuf schema types.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.1 Particularly useful type families

| Family                      | Representative types                                                 | Distinctive behavior                                                                     |
| --------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Temporal                    | `Timestamp`, `Duration`                                              | Protobuf JSON string encodings rather than ordinary objects                              |
| Dynamic JSON data           | `Struct`, `Value`, `ListValue`, `NullValue`                          | Representation of JSON objects, arbitrary values, arrays, and null                       |
| Scalar wrappers             | `Int64Value`, `StringValue`, `BoolValue`, `BytesValue`, etc.         | Scalar-shaped JSON representations with wrapper/message presence                         |
| Type erasure                | `Any`                                                                | A protobuf message containing a type URL and serialized payload                          |
| Update/projection semantics | `FieldMask`                                                          | Paths identifying fields, with a special JSON string representation in the specification |
| Schema metadata             | `FileDescriptorSet`, `DescriptorProto`, `FieldDescriptorProto`, etc. | The protobuf schema-description model itself                                             |

`Timestamp` and `Duration` have specialized Serde implementations and additional Rust conversion conveniences. The documented `Timestamp` API includes conversions involving `chrono::DateTime<Utc>`; `Duration` supports conversion with `std::time::Duration`, subject to conversion constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



The `Value` family is particularly relevant for heterogeneous JSON. A protobuf `Value` distinguishes the JSON kinds through its nested `kind` representation, and the crate provides conversions from several Rust scalar and collection types. This reduces the amount of bespoke conversion machinery needed when protobuf messages legitimately contain open-ended JSON structures.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

- [https://docs.rs/pbjson-types/latest/pbjson_types/struct.Timestamp.html](https://docs.rs/pbjson-types/latest/pbjson_types/struct.Timestamp.html?utm_source=chatgpt.com)
- [https://docs.rs/pbjson-types/latest/pbjson_types/struct.Duration.html](https://docs.rs/pbjson-types/latest/pbjson_types/struct.Duration.html?utm_source=chatgpt.com)
- [https://docs.rs/pbjson-types/latest/pbjson_types/struct.Value.html](https://docs.rs/pbjson-types/latest/pbjson_types/struct.Value.html?utm_source=chatgpt.com)
- [https://docs.rs/pbjson-types/latest/pbjson_types/struct.Struct.html](https://docs.rs/pbjson-types/latest/pbjson_types/struct.Struct.html?utm_source=chatgpt.com)

### 7.2 `pbjson-types` is not interchangeable with `prost-types` at the Rust type level

Both crates represent protobuf-defined well-known message types, but they provide different Rust types and trait implementations.

A generated field using `prost_types::Timestamp` does not automatically become a `pbjson_types::Timestamp` simply because both represent `google.protobuf.Timestamp`.

This is why `prost-build::Config::extern_path(".google.protobuf", "::pbjson_types")` matters: it controls the Rust types referenced by generated messages.

Similarly, `prost-reflect` already provides `ReflectMessage` implementations for well-known types from `prost-types`. Those implementations should not be assumed to extend automatically to the distinct types supplied by `pbjson-types`.

Binary protobuf transcoding is a possible interoperation boundary where compatible message schemas are available, but it incurs conversion work and is not an implicit Rust type conversion.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.3 Critical limitation: incomplete well-known-type JSON conformance

The `pbjson-types` documentation explicitly warns that coverage is incomplete and some implementations may not conform to the protobuf JSON specification. This should be treated as a substantive limitation rather than boilerplate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



The clearest confirmed example is `google.protobuf.Any`. Upstream issue #2 remains open and states that its specialized `@type` JSON representation is not implemented. The ordinary generated representation instead exposes its underlying `typeUrl` and base64 `value` fields, which is not the protobuf JSON representation required for interoperable `Any` values.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Issue: [https://github.com/influxdata/pbjson/issues/2](https://github.com/influxdata/pbjson/issues/2?utm_source=chatgpt.com)

`FieldMask` also warrants special attention. The `pbjson-types` 0.9.0 build script excludes various well-known types from generated Serde code so handwritten implementations can supply special mappings. `FieldMask` is not among those exclusions, despite the protobuf JSON specification requiring a comma-separated string with field-path name conversion. That is strong source-level evidence of a conformance gap, although I have not independently run a 0.9.0 serialization test for this type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



Build source: [https://docs.rs/crate/pbjson-types/0.9.0/source/build.rs](https://docs.rs/crate/pbjson-types/0.9.0/source/build.rs?utm_source=chatgpt.com)

Consequently, `pbjson-types` should not be viewed as proof that every well-known protobuf type has fully compatible JSON handling. The capabilities are significant, but interoperability depends on the particular types involved.

## 8. Protobuf JSON semantics: the interoperability contract

Both library families operate against the Protobuf JSON specification rather than ordinary Rust struct-to-JSON conventions. This distinction is especially important because a protobuf message has semantics that are not captured by a generic JSON object alone.

### 8.1 Core mapping rules

| Protobuf construct             | Standard JSON mapping                     | Consequential detail                                                  |
| ------------------------------ | ----------------------------------------- | --------------------------------------------------------------------- |
| Field names                    | `lowerCamelCase`, or explicit `json_name` | Parsers must also accept original protobuf field names                |
| `int32`, `uint32`              | JSON number                               | Normal integer conversion rules                                       |
| `int64`, `uint64`              | Decimal string                            | Avoids loss of precision in JavaScript-style numeric environments     |
| `bytes`                        | Base64 string                             | Binary payload is encoded as text                                     |
| Enums                          | Symbolic name                             | Numeric enum values may also be accepted                              |
| Floating-point                 | JSON number or special string             | `NaN`, `Infinity`, `-Infinity` require special representations        |
| Repeated fields                | JSON array                                | Empty arrays are ordinarily omitted without explicit default emission |
| Maps                           | JSON object                               | Protobuf map keys are converted to JSON object keys                   |
| Message fields                 | JSON object                               | Presence and `null` handling matter                                   |
| `oneof`                        | Normal field names                        | Mutual-exclusion semantics are retained                               |
| `Timestamp`                    | RFC 3339 timestamp string                 | Precision and timezone normalization apply                            |
| `Duration`                     | Seconds string with `s` suffix            | Fractional nanosecond precision                                       |
| `Struct`, `Value`, `ListValue` | Arbitrary JSON object/value/array         | Not the ordinary nested protobuf struct representation                |
| `FieldMask`                    | Comma-separated string                    | Field-path conversion applies                                         |
| `Any`                          | `@type` plus expanded message contents    | Requires resolution of the embedded type                              |

These are requirements of the protocol's JSON representation, not intrinsic behavior of `serde_json`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



Specification: [https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)

### 8.2 Binary compatibility is stronger than JSON compatibility

Protobuf's binary encoding uses field numbers, while its JSON encoding ordinarily uses field names. Consequently, changes that are safe in binary protobuf may break JSON consumers.

In particular, renaming fields or enum values can break JSON compatibility even when binary field numbers remain unchanged. Unknown binary fields can be retained by `DynamicMessage`, but unknown JSON properties cannot be preserved through protobuf JSON parsing.

The protocol specification explicitly documents these differences under its JSON wire-safety guidance.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



This affects the significance of the two library families:

- `prost-reflect` is especially useful when a system must inspect schemas and manage binary messages whose definitions may evolve.
- `pbjson` is especially useful when Rust types are known but must interoperate with protobuf-compatible JSON consumers.
- Neither library makes protobuf JSON an inherently lossless interchange representation for arbitrary evolving binary messages.

### 8.3 Distinguishing optional absence, defaults, and JSON null

For protobuf fields with explicit presence, an absent field and a field explicitly set to its default value can be semantically different. JSON output must respect the presence model.

For ordinary fields, JSON `null` is generally accepted as an unset value; `google.protobuf.NullValue` is a special case representing an actual JSON null.

This makes generic `serde_json::Value` manipulation potentially misleading: adding a zero, an empty string, or `null` is not always equivalent to preserving the original protobuf field state.

Where correctness depends on partial updates or message merging, protobuf descriptors and presence-aware APIs provide information that a schema-free JSON representation lacks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation+1



## 9. Combining reflection and generated JSON

The greatest architectural leverage comes from treating descriptor metadata as a shared artifact while keeping static and dynamic execution paths distinct.

### 9.1 Complementary combinations

| Operation or workload                                | More direct capability                   | Why                                                         |
| ---------------------------------------------------- | ---------------------------------------- | ----------------------------------------------------------- |
| JSON for a fixed collection of generated messages    | `pbjson-build` + `pbjson`                | Uses generated Serde implementations                        |
| Decode messages identified only at runtime           | `prost-reflect`                          | Resolves schema and values dynamically                      |
| Inspect protobuf schema structure                    | `DescriptorPool`                         | Exposes names, relationships, types, options, and services  |
| Obtain descriptors from statically generated structs | `prost-reflect-build`                    | Adds `ReflectMessage` support                               |
| Dynamic protobuf JSON conversion                     | `prost-reflect` with `serde`             | Deserializes and serializes from runtime descriptors        |
| Standard JSON for supported well-known types         | `pbjson-types`                           | Supplies specialized Rust types and Serde behavior          |
| Preserve unknown protobuf wire fields                | `DynamicMessage`                         | Retains unknown fields through binary decoding and encoding |
| Inspect custom schema annotations                    | `DescriptorPool` + extension descriptors | Interprets protobuf custom options                          |
| Publish gRPC reflection metadata                     | Companion RPC reflection service         | Uses descriptor sets but is not implemented by these crates |

One plausible combination is to use generated Rust structs with `pbjson` for ordinary application messages, then retain a descriptor pool for generic inspection, diagnostics, or messages introduced independently of the compiled application.

The capability combination avoids making every operation dynamic merely because some operations require reflection.

### 9.2 Sharing descriptor sets across tooling

A descriptor set can feed several independent consumers:

- `prost-build` to generate Rust message types.
- `pbjson-build` to generate JSON Serde implementations.
- `prost-reflect` to build a runtime `DescriptorPool`.
- `prost-reflect-build` to connect generated Rust types to descriptors.
- A gRPC reflection service, such as `tonic-reflection`, to expose schemas to compatible RPC clients.

For example, `tonic_reflection::server::Builder` supports `register_encoded_file_descriptor_set`, and `tonic` provides an `include_file_descriptor_set!` convenience macro. Those are capabilities of the Tonic ecosystem, not of `prost-reflect`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/tonic-reflection/latest/tonic_reflection/server/struct.Builder.html](https://docs.rs/tonic-reflection/latest/tonic_reflection/server/struct.Builder.html?utm_source=chatgpt.com)

The shared artifact reduces the need for separate metadata definitions, but its contents must still be appropriate for every consumer. In particular, source information, imports, custom options, and well-known-type dependencies can affect what downstream tooling can inspect.

### 9.3 Build-time compatibility and configuration consistency

Both crate families currently integrate with the `prost` 0.14 generation ecosystem. The latest `prost-build` documentation consulted here describes version 0.14.4.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



The important configuration boundaries are:

Rust type mappings. `prost-build` can select `BTreeMap` or `HashMap`, `bytes::Bytes` or `Vec<u8>`, external Rust types, and enum naming conventions. A separate generator emitting implementations for those types must use matching assumptions. `pbjson-build` exposes `btree_map`, `extern_path`, and `retain_enum_prefix` for precisely this kind of coordination.

Descriptor paths. `prost-build` can write descriptor sets to a chosen path. The reflection builder has its own descriptor-path configuration and override behavior. These should not be mistaken for independent, automatically synchronized outputs.

Source metadata. `prost-build::Config::skip_source_info` can reduce descriptor size by omitting source-code information. That choice also reduces the source-location information available for schema inspection and diagnostics.

Custom generated attributes. `prost-build` supports `type_attribute`, `message_attribute`, `field_attribute`, and related customizations. Reflection derives and generated JSON implementations must coexist with the resulting code without duplicate trait implementations or inconsistent type mappings.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Source: [https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html?utm_source=chatgpt.com)

### 9.4 Alternative descriptor compilation with `protox`

The companion `protox` crate provides a Rust implementation of protobuf schema compilation. Its `compile` and `Compiler` interfaces can construct file descriptors without relying on a `protoc` executable for that step.

`prost-build` accepts precompiled descriptors through `compile_fds`, providing a useful integration route for Rust-native schema compilation. `protox` also exposes source-aware diagnostic information through `miette`.

This is an adjacent compilation capability, not a feature of `pbjson` or `prost-reflect` themselves. Generated JSON and runtime reflection still depend on obtaining a compatible descriptor set.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/protox/0.10.0/protox/](https://docs.rs/protox/0.10.0/protox/?utm_source=chatgpt.com)

## 10. Operational characteristics and important boundaries

### 10.1 Runtime work and resource ownership

The two serialization paths move work to different stages:

| Concern                        | Runtime reflection                              | Generated JSON                        |
| ------------------------------ | ----------------------------------------------- | ------------------------------------- |
| Schema interpretation          | Runtime descriptor pool                         | Build-time generator                  |
| Message type discovery         | Dynamic name/descriptor lookup                  | Fixed Rust types                      |
| Field access                   | Runtime `Value` representation                  | Native struct fields                  |
| Serialization behavior         | Per-message runtime options                     | Generated implementations             |
| Binary unknown-field retention | Supported by `DynamicMessage`                   | Not supplied by `pbjson`              |
| Schema update                  | New descriptor pool can be loaded               | Rust code may need regeneration       |
| Principal costs                | Descriptor management, dynamic values, dispatch | Build complexity, generated code size |

The direct performance implications are conditional. A reused descriptor pool avoids repeated schema decoding. Static JSON implementations avoid generic runtime field inspection. Conversely, transcoding between static and dynamic representations adds binary encode/decode work.

There is no basis here for assigning a universal latency or throughput multiplier to either approach. Notably, `prost-reflect` 0.16.5 specifically fixed a `DynamicMessage` encoding performance regression; this is a release-specific fix, not a general benchmark.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 10.2 Diagnostics and error handling

`prost-reflect` distinguishes several error domains: invalid descriptor sets, protobuf binary decoding, JSON deserialization, text-format parsing, and incompatible dynamic field assignment.

`DescriptorError` can expose file, line, column, and descriptor path information. With the optional `miette` feature, it can integrate with structured source diagnostics, provided that the necessary source metadata is available.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/prost-reflect/latest/prost_reflect/struct.DescriptorError.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.DescriptorError.html?utm_source=chatgpt.com)

These diagnostics are useful for schema tooling, but they do not turn the library into a full semantic validator. Rules such as cross-field consistency, business constraints, authorization, and allowed state transitions remain separate concerns.

### 10.3 Feature flags and platform assumptions

The relevant optional `prost-reflect` features include:

| Feature       | Enables                                                         |
| ------------- | --------------------------------------------------------------- |
| `serde`       | Protobuf JSON serialization/deserialization of dynamic messages |
| `derive`      | `ReflectMessage` derive macro                                   |
| `text-format` | Parsing and formatting protobuf text                            |
| `miette`      | Enhanced descriptor error diagnostics                           |

The published `prost-reflect` documentation states a minimum supported Rust version of 1.82 for this release family. The wider build toolchain may impose a newer effective requirement; current `prost` documentation, for example, identifies an MSRV of 1.85.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Feature and dependency requirements should therefore be evaluated across the combined dependency graph rather than inferred from one crate alone.

### 10.4 Newer protobuf editions

The currently documented `prost-reflect::Syntax` enum distinguishes `Proto2` and `Proto3`. Meanwhile, protobuf schema definitions and descriptor messages now contain concepts associated with newer Editions.

The existence of those types does not by itself establish complete support for all edition-specific features and their semantics in these crates. For Edition-based schemas, the relevant compiler, descriptor representation, generated Rust code, and JSON implementation need to be checked together.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 10.5 Input trust and schema compatibility

When accepting externally supplied schemas or messages, these libraries provide parsing and structural interpretation, not a complete security boundary. Descriptor provenance, supported message types, payload sizes, nesting, and resource budgets remain relevant to the surrounding system.

This is particularly true for `Any`: interpreting an embedded message requires a known type definition, and having its schema available is different from trusting the meaning or provenance of its payload.

## 11. Capability synthesis

The overall ecosystem supports three distinct architectural roles.

First, a reflective schema plane. `DescriptorPool`, descriptor handles, extension metadata, and `ReflectMessage` make protobuf definitions inspectable as structured runtime information. This is broader than binary serialization and can replace many hand-maintained schema lookup tables.

Second, a dynamic value plane. `DynamicMessage` provides runtime-defined messages with typed field operations, protobuf binary encoding, unknown-field preservation, JSON conversion, and optional text-format support. It is useful wherever the message definition cannot be assumed to be fixed at compile time.

Third, a compiled JSON plane. `pbjson-build` emits protobuf-aware Serde implementations for known Rust message types, while `pbjson-types` supplies many standard protobuf types with JSON behavior. This keeps ordinary message processing statically typed and moves schema interpretation into the build.

The most consequential distinction is that reflection and generated JSON are complementary, not competing implementations of the same capability. A shared descriptor artifact can support both, but correctness still depends on protobuf presence semantics, unknown-field behavior, type mappings, and the actual well-known-type conformance of the chosen JSON path.

## 12. Primary source map

| Topic                                    | Direct source                                                                                                                                                                                               |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `prost-reflect` concepts and examples    | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/](https://docs.rs/prost-reflect/0.16.5/prost_reflect/?utm_source=chatgpt.com)                                                                           |
| Descriptor pool, loading, custom options | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html?utm_source=chatgpt.com)                       |
| Dynamic message operations               | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html?utm_source=chatgpt.com)                       |
| `ReflectMessage` integration             | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/trait.ReflectMessage.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/trait.ReflectMessage.html?utm_source=chatgpt.com)                         |
| Reflection code-generation builder       | [https://docs.rs/prost-reflect-build/0.16.1/prost_reflect_build/struct.Builder.html](https://docs.rs/prost-reflect-build/0.16.1/prost_reflect_build/struct.Builder.html?utm_source=chatgpt.com)             |
| Dynamic JSON serialization options       | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.SerializeOptions.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.SerializeOptions.html?utm_source=chatgpt.com)                   |
| Dynamic JSON deserialization options     | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DeserializeOptions.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DeserializeOptions.html?utm_source=chatgpt.com)               |
| Text-format options                      | [https://docs.rs/prost-reflect/0.16.5/prost_reflect/text_format/struct.FormatOptions.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/text_format/struct.FormatOptions.html?utm_source=chatgpt.com) |
| `pbjson-build` usage and generation      | [https://docs.rs/pbjson-build/0.9.0/pbjson_build/](https://docs.rs/pbjson-build/0.9.0/pbjson_build/?utm_source=chatgpt.com)                                                                                 |
| `pbjson-build::Builder` configuration    | [https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html](https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html?utm_source=chatgpt.com)                                           |
| `pbjson-types` catalog                   | [https://docs.rs/pbjson-types/0.9.0/pbjson_types/](https://docs.rs/pbjson-types/0.9.0/pbjson_types/?utm_source=chatgpt.com)                                                                                 |
| `pbjson-types` generation source         | [https://docs.rs/crate/pbjson-types/0.9.0/source/build.rs](https://docs.rs/crate/pbjson-types/0.9.0/source/build.rs?utm_source=chatgpt.com)                                                                 |
| `pbjson` `Any` limitation                | [https://github.com/influxdata/pbjson/issues/2](https://github.com/influxdata/pbjson/issues/2?utm_source=chatgpt.com)                                                                                       |
| Protobuf JSON specification              | [https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)                                                                                       |
| `prost-build` configuration              | [https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html?utm_source=chatgpt.com)                                               |
| `protox` compiler integration            | [https://docs.rs/protox/0.10.0/protox/](https://docs.rs/protox/0.10.0/protox/?utm_source=chatgpt.com)                                                                                                       |
| Tonic gRPC reflection service            | [https://docs.rs/tonic-reflection/latest/tonic_reflection/server/struct.Builder.html](https://docs.rs/tonic-reflection/latest/tonic_reflection/server/struct.Builder.html?utm_source=chatgpt.com)           |
| `prost-reflect` release history          | [https://github.com/andrewhickman/prost-reflect/blob/main/CHANGELOG.md](https://github.com/andrewhickman/prost-reflect/blob/main/CHANGELOG.md?utm_source=chatgpt.com)                                       |

Verification note: This reference is based on published crate documentation, upstream source, release history, and the protobuf specification as available on October 31, 2026. No local builds, execution tests, conformance suite, or performance measurements were conducted. The `pbjson-types` `Any` limitation is explicitly documented upstream; the `FieldMask` concern is identified from the released generator configuration and should be treated as source-based inference rather than an independently executed test.