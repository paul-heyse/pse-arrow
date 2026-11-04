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

# Addendum — Prost Reflection and Protobuf JSON

Companion reference: Prost Reflection and Protobuf JSON — Rust Library Reference (October 31, 2026)

Research date: November 3, 2026\
Version scope: `prost-reflect` 0.16.5, `prost-reflect-build` 0.16.1, `prost-reflect-derive` 0.16.1, `pbjson` / `pbjson-build` / `pbjson-types` 0.9.0, and their `prost` 0.14 integration.

This addendum supplies advanced functionality, implementation semantics, and interoperability qualifications not fully developed in the original reference. Section references indicate where the material extends or corrects the corresponding discussion. All described library APIs target published releases unless explicitly identified as development work.

## A1. Reflection-enabled code generation: descriptor binding and compilation lifecycle

Extends original §4 and §9.3.

### A1.1 Descriptor-source configuration is mandatory

`prost-reflect-build::Builder` has two ways to make descriptor metadata available to generated `ReflectMessage` implementations. At least one must be configured explicitly.

| API                               | Generated implementation                                                                            | Architectural consequence                                                                                                              |
| --------------------------------- | --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `descriptor_pool(expr)`           | Uses an expression resolving to a `DescriptorPool`                                                  | An application-owned pool supplies schema metadata; its construction, lifetime, and schema composition can be controlled independently |
| `file_descriptor_set_bytes(expr)` | References encoded descriptor bytes and uses automatic registration with the global descriptor pool | Avoids a separately declared application-owned pool, but uses process-global descriptor registration                                   |

The configured expression is emitted into generated Rust attributes. It must therefore be valid in the context of the generated message types, not merely in `build.rs`.

The published implementation returns an `io::Error` if neither option is configured. This is significant because the 0.16.1 `Builder` method documentation includes an abbreviated `Builder::new().compile_protos(...)` example that omits both settings; that example is insufficient as written.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



A minimal descriptor-bytes arrangement is:

```
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_reflect_build::Builder::new()
        .file_descriptor_set_bytes("crate::FILE_DESCRIPTOR_SET_BYTES")
        .compile_protos(&["proto/example.proto"], &["proto"])?;
    Ok(())
}

// Crate root
const FILE_DESCRIPTOR_SET_BYTES: &[u8] =
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/file_descriptor_set.bin"
    ));
```

The `descriptor_pool` alternative can instead refer to a lazily initialized static pool. Its expression should resolve to a reusable pool rather than decoding a descriptor set repeatedly.

An important interaction follows from the global-pool alternative: the global registry does not provide arbitrary schema replacement. Descriptor files already registered under the same file name are skipped, so descriptor provenance and schema-version conflicts remain relevant even when registration is automatic.

### A1.2 `configure()` performs compilation-related work

The builder exposes three entry points, but they do not simply represent three syntactic variants of one generation step.

`compile_protos()` delegates to `compile_protos_with_config()` using a new `prost_build::Config`.

`compile_protos_with_config()` first calls `configure()`, then calls `Config::skip_protoc_run().compile_protos(...)` to generate Rust code using the already compiled descriptor artifact.

`configure()` itself calls `Config::file_descriptor_set_path(...).compile_protos(...)`, reads the resulting descriptor bytes, creates a `DescriptorPool`, and enumerates message descriptors to attach `#[derive(::prost_reflect::ReflectMessage)]` plus the required reflection attributes.

Therefore, `configure()` is not a purely in-memory configuration operation. It performs a compilation step to obtain schema metadata before configuring the final generation path. Its documented ability to leave the caller in control of subsequent generation does not mean it avoids initial compilation work.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



This matters when composing reflection generation with:

- Additional `prost-build` type and field attributes.
- Externally generated descriptors or customized code-generation stages.
- Nonstandard output locations or multiple generated packages.
- Cargo rebuild dependencies and build-script side effects.
- Other generators consuming the same descriptor artifact, including `pbjson-build`.

The builder also overrides `prost_build::Config::file_descriptor_set_path()` with its own configured path. Consequently, the effective descriptor artifact location must be understood at the `prost-reflect-build` level.

Source:

[https://docs.rs/prost-reflect-build/0.16.1/src/prost_reflect_build/lib.rs.html](https://docs.rs/prost-reflect-build/0.16.1/src/prost_reflect_build/lib.rs.html?utm_source=chatgpt.com)

### A1.3 Reflection can be derived independently of `prost-reflect-build`

The `prost_reflect::ReflectMessage` derive macro is available through the `derive` feature. `prost-reflect-build` is a convenience integration for automatically applying that derive to generated messages; it is not the only supported mechanism.

For an existing type implementing `prost::Message`, reflection can be attached directly:

```
#[derive(prost::Message, prost_reflect::ReflectMessage)]
#[prost_reflect(
    descriptor_pool = "crate::DESCRIPTOR_POOL",
    message_name = "example.Event"
)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub name: String,
}
```

The derive requires the fully qualified protobuf message name and a descriptor source. It does not infer the protobuf descriptor solely from the Rust struct's fields.

This creates two independent extension mechanisms:

1. Generation-time attachment: `prost-reflect-build` injects reflection derives and attributes across generated message types.
2. Type-level attachment: The derive macro supplies reflection for individual Rust message implementations, including those not produced by that specific builder.

Both supply the `ReflectMessage` interface, not direct dynamic field indexing over the generated Rust struct. Field-by-field reflective mutation still requires the dynamic representation or other type-specific access machinery.

There is also a documentation inconsistency: portions of the published derive documentation call the bytes attribute `file_descriptor_pool_bytes`, while the current builder implementation emits `file_descriptor_set_bytes`. The latter is the attribute implemented and used by the released generator.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Sources:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/trait.ReflectMessage.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/trait.ReflectMessage.html?utm_source=chatgpt.com)

[https://docs.rs/crate/prost-reflect-derive/0.16.1/source/](https://docs.rs/crate/prost-reflect-derive/0.16.1/source/?utm_source=chatgpt.com)

## A2. Dynamic message transformation and wire-level information preservation

Extends original §§3.1–3.5 and §9.1.

### A2.1 Mutable traversal of fields and extensions

`DynamicMessage` provides bulk traversal and in-place mutation interfaces beyond individual field getters and setters.

| API family                   | Relevant methods                                                                  | Additional capability                                                               |
| ---------------------------- | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Populated-field traversal    | `fields()`, `fields_mut()`                                                        | Inspect or mutate all semantically present ordinary fields                          |
| Extension traversal          | `extensions()`, `extensions_mut()`                                                | Apply equivalent operations to populated extension fields                           |
| Nested value access          | `Value::as_message_mut()`, `as_list_mut()`, `as_map_mut()`                        | Traverse and modify nested messages, repeated values, and maps                      |
| Scalar mutation              | `as_i32_mut()`, `as_string_mut()`, `as_bytes_mut()`, `as_enum_number_mut()`, etc. | Mutate values through typed references without reconstructing the enclosing message |
| Ownership transfer           | `take_field()`, `take_fields()`, `take_extension()`, `take_extensions()`          | Remove values while obtaining ownership of their representations                    |
| Schema-sensitive validation  | `is_valid_for_field()`, `is_valid_for_extension()`                                | Test whether a candidate value matches the protobuf type of the destination         |
| Default-sensitive comparison | `is_default_for_field()`, `is_default_for_extension()`                            | Evaluate defaults using the relevant field or extension definition                  |

These capabilities enable generic schema-aware transformations with substantially less per-message boilerplate. A transformation can enumerate fields, inspect each descriptor, traverse its dynamic value, and apply type-dependent operations without generating application-specific visitors for every message type.

The important distinction is between type compatibility and message presence. `Value::is_default_for_field()` evaluates whether a value equals the descriptor-defined default; `DynamicMessage::has_field()` determines semantic presence under protobuf's field rules. A present explicit-presence field can contain its default value. Neither predicate is a substitute for the other.

In-place mutation also does not eliminate the requirement to preserve field invariants. For example, modifying a nested `Value::List` or `Value::Map` directly requires maintaining element and key/value compatibility with its descriptor. The type-sensitive accessors provide structured access, not arbitrary mutation safety for every subsequently constructed value.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html?utm_source=chatgpt.com)

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/enum.Value.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/enum.Value.html?utm_source=chatgpt.com)

### A2.2 Draining iterators have partial-consumption semantics

`take_fields()`, `take_extensions()`, and `take_unknown_fields()` return iterators that progressively remove values from the message.

For each method, the documentation states that dropping the iterator before full consumption leaves the number of removed fields unspecified.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This is an important ownership and mutation property:

- Merely obtaining the iterator should not be interpreted as an atomic operation that has already removed every matching value.
- Stopping iteration early may leave a partially modified message.
- An error or early return during traversal can therefore produce a different message state from full consumption.
- Fully consuming the iterator is necessary when the operation requires the complete set of values to be removed.

The methods provide efficient ownership-based access to stored values; they do not provide transactional transformation, rollback, or all-or-nothing guarantees.

For fallible bulk transformations, the distinction between iterating mutable references and consuming values is consequently important. A transformation that must preserve the original message on failure requires its own state-management semantics.

### A2.3 `UnknownField` exposes protobuf wire structure

`UnknownField` is more than an opaque payload retained for later re-encoding. It is a public representation of an unrecognized protobuf wire field with independent inspection and encoding operations.

| Method           | Information or operation                                                   |
| ---------------- | -------------------------------------------------------------------------- |
| `number()`       | Original protobuf field number                                             |
| `wire_type()`    | Wire encoding category encountered during decoding                         |
| `encoded_len()`  | Length of its encoded wire representation                                  |
| `encode()`       | Serialize the unknown field independently                                  |
| `decode()`       | Decode an unknown field including its tag                                  |
| `decode_value()` | Decode a field value after its number and wire type have already been read |
| `Display`        | Render an inferred protobuf text-format-style diagnostic representation    |

`decode()` and `decode_value()` use the `prost` decoding context and buffer interfaces. `encode()` writes through `BufMut`, allowing the field to be incorporated into other wire-level output machinery.

This permits several operations without implementing a separate complete protobuf decoder: counting unknown field numbers, observing wire types, extracting unknown wire records, preserving opaque data during schema transitions, and displaying unfamiliar payloads for diagnostics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Wire type is not semantic type. A `LengthDelimited` field may represent a string, bytes, a nested protobuf message, or packed repeated scalar values. The wire encoding alone does not establish the intended interpretation.

This limitation also applies to `UnknownField`'s text rendering. Its formatter may infer plausible structures from bytes, but inferred structure is not schema-validated meaning.

Nor does unknown-field preservation establish byte-for-byte identity of a whole message after decoding and re-encoding. Protobuf permits multiple binary encodings of equivalent logical messages, and encoding order is not a universal canonicalization guarantee.

Source:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.UnknownField.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.UnknownField.html?utm_source=chatgpt.com)

### A2.4 Transcoding into generated `prost` messages can discard information

`DynamicMessage` preserves unknown binary fields, but ordinary generated `prost` message types do not generally retain unrecognized field data.

This creates an important asymmetry:

Conceptual information-flow boundary, not an assertion about exact byte preservation.

`DynamicMessage::transcode_to<T>()` serializes the dynamic representation and decodes it into the requested generated type. Unrecognized fields are not normally retained in that generated instance; transcoding it back cannot recreate the discarded data.

This distinction affects schema evolution, opaque-message forwarding, and tooling that must preserve fields introduced by newer producers.

Three separate cases should not be conflated:

| Case                       | Relevant behavior                                                                               |
| -------------------------- | ----------------------------------------------------------------------------------------------- |
| Unknown field number       | `DynamicMessage` can preserve its wire representation                                           |
| Unknown numeric enum value | Can remain represented numerically where the enum semantics permit it                           |
| Unknown JSON property      | Cannot be retained as an arbitrary protobuf unknown wire field by these ProtoJSON deserializers |

The upstream `prost` project has an open proposal to add unknown-field support to generated messages, but that proposal is not part of the reviewed stable release. The observation here applies to ordinary released `prost` types, not to hypothetical future support.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source:

[https://github.com/tokio-rs/prost/pull/1340](https://github.com/tokio-rs/prost/pull/1340?utm_source=chatgpt.com)

## A3. Advanced descriptor inspection, source provenance, and options interpretation

Extends original §§2.1–2.3 and §9.3.

### A3.1 Descriptor paths provide a bridge to original `.proto` declarations

Most descriptor types expose `path()`, an integer path locating the element within the original `FileDescriptorProto` tree.

The path can be compared against `SourceCodeInfo.Location.path` entries in compiled descriptor metadata. When source information is included, the corresponding location can supply declaration spans and comments.

The descriptor APIs additionally expose underlying protobuf descriptor structures through methods such as `message_descriptor_proto()`, `enum_descriptor_proto()`, `field_descriptor_proto()`, and `parent_file_descriptor_proto()`.

Together, these offer three views of a schema element:

1. A resolved semantic handle, including field type, parent relationships, and options.
2. Its underlying descriptor-proto structure, preserving the schema's serialized metadata model.
3. A path into optional source-location metadata.

This combination supports schema-aware documentation, source-to-descriptor navigation, structured metadata indexing, provenance tracking, and more informative diagnostics without building an independent `.proto` parser solely for declaration discovery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



The source mapping has important constraints. Descriptor `path()` identifies an element structurally even when no source locations are available. `SourceCodeInfo` is optional, and its absence means that source spans or comments cannot be recovered from the descriptor set alone. The bundled well-known-type descriptors also omit source information.

A generic schema inspector can therefore remain functional without source metadata, while source-aware capabilities degrade independently.

Sources:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.MessageDescriptor.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.MessageDescriptor.html?utm_source=chatgpt.com)

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.FileDescriptor.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.FileDescriptor.html?utm_source=chatgpt.com)

### A3.2 Custom options require both information preservation and interpretation

The descriptor-pool loading distinction already covered in the main reference is important: decoding descriptor bytes directly can preserve custom-option extension data that an intermediate `prost_types` representation might discard.

A second distinction concerns how options are represented before they have been interpreted.

`prost-reflect` supports interpretation of `uninterpreted_options` in descriptor options metadata. However, the published release history documents that values supplied through `UninterpretedOption.aggregate_value` are ignored if the `text-format` feature is disabled.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Consequently, obtaining complete machine-readable annotations depends on more than one condition:

- The relevant custom-option extension definitions must be available.
- The option values must survive descriptor ingestion.
- The representation of any uninterpreted option must be supported by the enabled features.
- The resulting `DynamicMessage` options must be interpreted with the appropriate extension descriptors.

The `text-format` dependency is particularly consequential for tooling that consumes descriptors produced through heterogeneous compilation paths. Standard interpreted options may work even while specific aggregate-valued uninterpreted options remain unavailable.

The library interprets option structures; it does not supply a general evaluation or policy-enforcement engine for their domain-specific meaning.

### A3.3 Extension metadata and extension values have distinct scopes

There are two useful directions of extension discovery:

- `MessageDescriptor::child_extensions()` finds extensions declared inside a particular message.
- `MessageDescriptor::extensions()` finds extensions targeting that message within the parent descriptor pool, including extensions declared elsewhere.

This distinction avoids confusing an extension's declaration location with the message that the extension modifies.

On the value side, `DynamicMessage` supports dynamically decoded extension values and mutable traversal. Its ProtoJSON implementation also supports extension round-tripping, as documented in the upstream changelog.

These mechanisms make descriptor-defined extensions available to generic inspection and conversion without requiring separately generated Rust accessors for every extension. The applicable extension declarations must still be present in the schema environment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Source:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.MessageDescriptor.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.MessageDescriptor.html?utm_source=chatgpt.com)

### A3.4 Additional descriptor and value APIs

Several smaller APIs complement the existing reference's descriptor inventory.

| API                                                  | Capability not fully developed in the original                                           |
| ---------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `DescriptorPool::decode_file_descriptor_proto()`     | Ingest one encoded descriptor file while preserving encoded extension-option information |
| `DescriptorPool::get_file_by_name()`                 | Resolve a specific source file from the pool                                             |
| `DescriptorPool::add_global_file_descriptor_proto()` | Register a descriptor file with the process-wide global pool                             |
| `FieldDescriptor::is_group()`                        | Distinguish legacy proto2 group encoding from ordinary length-delimited message encoding |
| `FieldDescriptor::is_required()`                     | Inspect requiredness independently of generalized presence support                       |
| `Kind::wire_type()`                                  | Obtain the expected underlying protobuf wire-encoding category                           |
| `Value::is_valid_for_extension()`                    | Validate type compatibility for dynamic extension assignment                             |
| `Value::default_value_for_field()`                   | Construct a value reflecting field-specific defaults and cardinality                     |

These capabilities are useful where generic processing must distinguish physical encoding, field cardinality, schema defaults, and extension behavior. In particular, `Kind::wire_type()` is not sufficient for describing the exact encoded representation of every field: group encoding and packed repeated fields require field-level metadata.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorPool.html?utm_source=chatgpt.com)

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.FieldDescriptor.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.FieldDescriptor.html?utm_source=chatgpt.com)

## A4. ProtoJSON deserialization and enum identity

Extends original §§5.2, 6.3, and 8.

### A4.1 Permissive JSON parsing can discard enum information

`prost-reflect::DeserializeOptions::deny_unknown_fields(false)` affects more than unknown JSON field names.

The upstream 0.15.1 release notes confirm that this option also allows unrecognized enum names to be ignored during JSON deserialization. This behavior remains part of the reviewed 0.16.5 release.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



This creates distinct information-loss cases:

| Input situation            | Semantic consequence                                                                                                               |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Unknown JSON property      | Permissive parsing skips data not recognized by the current message schema                                                         |
| Unknown symbolic enum name | Permissive parsing can ignore an unrecognized enum value                                                                           |
| Unknown numeric enum value | Protobuf's numeric enum representation provides a different compatibility path, subject to the enum's openness and parser behavior |
| Unknown binary field       | The reflective binary decoder can retain the original wire field                                                                   |

The `pbjson-build` family exposes related but separately configured behavior. `ignore_unknown_fields()` skips unknown JSON properties, while `ignore_unknown_enum_variants()` permits unknown enum variants to be replaced by the enum's default.

Replacing an unknown enum value with the default is not equivalent to preserving its identity or representing it as an unknown numeric value. It can collapse distinct input states into the same application-visible value.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This distinction matters when generated and dynamic JSON paths operate on the same message schema. Matching their acceptance policies does not automatically guarantee equivalent information preservation.

Sources:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DeserializeOptions.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DeserializeOptions.html?utm_source=chatgpt.com)

[https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html](https://docs.rs/pbjson-build/0.9.0/pbjson_build/struct.Builder.html?utm_source=chatgpt.com)

[https://github.com/andrewhickman/prost-reflect/blob/main/CHANGELOG.md](https://github.com/andrewhickman/prost-reflect/blob/main/CHANGELOG.md?utm_source=chatgpt.com)

### A4.2 Duplicate JSON field names and oneof conflicts

ProtoJSON permits two names for an ordinary protobuf field: its original `.proto` field name and its JSON name.

Consequently, semantic duplication can occur even without identical JSON keys.

Consider:

```
{
  "eventId": "123",
  "event_id": "456"
}
```

If `event_id` is a protobuf field whose JSON name is `eventId`, these are two representations of the same field, not independent properties.

Another conflict occurs when a JSON object includes more than one member of a protobuf `oneof`.

The current ProtoJSON specification requires serializers not to emit duplicate logical fields. Parsers should implement last-value-wins semantics when the input representation retains sufficient ordering information; where this cannot be implemented correctly, they must reject the ambiguous input rather than arbitrarily selecting a value.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



This is different from assuming that all JSON objects are maps whose duplicate keys have already been resolved before protobuf deserialization begins.

`prost-reflect` has an additional implementation-specific refinement: since 0.15.1, its JSON deserializer accepts multiple oneof members when only one is non-null. The changelog explicitly excludes the special handling of `google.protobuf.Value` and `google.protobuf.NullValue` from that relaxation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



This reflects another ProtoJSON principle: for ordinary message fields, JSON `null` ordinarily means unset, rather than an explicitly present application value.

A parser that first materializes JSON into a schema-free map may lose the ordering and duplicate-key information necessary to reproduce all ProtoJSON conflict semantics. Direct deserialization from a JSON token stream and deserialization from an intermediate JSON value tree are therefore not necessarily equivalent for malformed or ambiguous inputs.

Source:

[https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)

### A4.3 Enum aliases: numeric identity differs from symbolic identity

Protobuf permits enum aliases through `option allow_alias = true`. Multiple named symbols can then share a numeric value.

For example:

```
enum Status {
  option allow_alias = true;

  STATUS_UNSPECIFIED = 0;
  STATUS_OLD = 1;
  STATUS_NEW = 1;
}
```

The official ProtoJSON specification requires serializers to emit the first declared symbolic name for a numeric value, while parsers accept any declared alias.

Thus, a value of `1` serializes canonically as `"STATUS_OLD"` for the definition above. Reordering the alias declarations changes the canonical serialized symbol without changing its protobuf numeric identity. The specification explicitly uses this behavior to support staged enum renaming.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



There is a consequential distinction in the reflection API:

`EnumDescriptor::get_value(number)` returns a matching `EnumValueDescriptor`, but its documentation states that the returned alias is unspecified when multiple names share the same number.

`EnumDescriptor::get_value_by_name(name)` resolves individual symbols, while `values()` permits enumeration of the declarations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



A schema tool that needs canonical ProtoJSON output should therefore not assume that a call to `get_value(number)` alone supplies the canonical first-declared symbolic name.

This API ambiguity does not independently prove that `prost-reflect`'s own JSON serializer violates the specification. That would require examining the complete version-specific serialization path or executing an alias-focused conformance case.

Source:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.EnumDescriptor.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.EnumDescriptor.html?utm_source=chatgpt.com)

### A4.4 ProtoJSON mapping is not byte-canonical JSON serialization

The phrase canonical protobuf JSON mapping describes how protobuf values are mapped to JSON, including field naming, integer strings, enum representations, and well-known-type encodings.

It does not provide a universal guarantee that logically equivalent messages will produce byte-identical JSON text.

`prost-reflect::Value::Map` is represented as a Rust `HashMap<MapKey, Value>`. Generated messages can instead use map representations such as `HashMap` or `BTreeMap`, depending on generation settings.

Map iteration order, serializer implementation details, and formatting choices can affect output bytes independently of the protobuf message's logical value.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



A `BTreeMap` representation can reduce one source of ordering variability, but it is not a complete canonicalization scheme.

The distinction has implications for content addressing, digital signatures, reproducible snapshots, and serialized-payload equality. Systems requiring a stable byte representation need an explicitly defined serialization/canonicalization procedure beyond ProtoJSON type conformance.

Source:

[https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)

## A5. Well-known-type conformance and update semantics

Extends original §§7.1–7.3 and §8.

### A5.1 `pbjson-types::Timestamp` does not emit the specified canonical UTC suffix

The original guide identifies gaps in the `Any` and `FieldMask` implementations. The released `Timestamp` implementation has an additional source-verifiable discrepancy.

The protobuf definition specifies that a JSON serializer should output timestamps in UTC with the `Z` suffix.

`pbjson-types` 0.9.0 instead converts its `Timestamp` to `chrono::DateTime<Utc>` and serializes the value using `DateTime::to_rfc3339()`.

Its released source test explicitly expects:

```
"2016-11-08T16:07:09+00:00"
```

The canonical protobuf form of the same timestamp is:

```
"2016-11-08T16:07:09Z"
```

Both express the same instant under RFC 3339, but the first does not follow the prescribed canonical ProtoJSON representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



This is specifically a serialization canonicalization discrepancy. It does not establish that the timestamp is interpreted incorrectly.

The code also uses Chrono timestamp conversion to determine representability. The published implementation does not visibly enforce the complete protobuf `Timestamp` domain of years 0001 through 9999 independently of Chrono's supported range. This is an additional potential conformance difference inferred from source, rather than an independently demonstrated failing test.

More generally, these are separate properties of a well-known-type implementation:

- Whether a protobuf value is within its defined semantic range.
- Whether JSON parsing accepts all and only the required input forms.
- Whether JSON serialization emits the canonical required representation.
- Whether serialization followed by deserialization preserves the represented value.

Successful RFC 3339 parsing or formatting alone establishes none of the remaining properties.

Sources:

[https://docs.rs/crate/pbjson-types/0.9.0/source/src/timestamp.rs](https://docs.rs/crate/pbjson-types/0.9.0/source/src/timestamp.rs?utm_source=chatgpt.com)

[https://protobuf.dev/reference/protobuf/google.protobuf/](https://protobuf.dev/reference/protobuf/google.protobuf/?utm_source=chatgpt.com)

### A5.2 `FieldMask` connects schema-defined paths with partial updates

`google.protobuf.FieldMask` represents a set of field paths. Its JSON representation is distinct from its application semantics.

This distinction becomes particularly important for fields with implicit presence.

Under implicit-presence merge semantics, a default-valued source field is ordinarily omitted from the merge. A sender therefore cannot express a request to overwrite an existing value with zero merely by supplying an implicit-presence scalar whose value is zero.

A separate field mask can indicate that a particular destination field is intended to participate in the update, including when its requested value is the default.

The protobuf field-presence documentation identifies `FieldMask` as an external mechanism for this situation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



However, the capabilities remain separate:

- `pbjson-types::FieldMask` represents the protobuf field-mask message.
- ProtoJSON serialization converts field-mask paths to the specified string format.
- `prost-reflect` can inspect message field descriptors and manipulate corresponding dynamic values.
- None of these operations alone supplies a complete, policy-aware field-mask patch implementation.

Applying a mask requires interpretation relative to a particular target message schema, including path resolution and update semantics. The existence of matching names in a mask and message descriptor is not equivalent to applying a valid update.

### A5.3 Some valid protobuf field names cannot round-trip through FieldMask JSON

The protobuf specification recognizes a non-round-trippable class of field-mask paths.

The binary `FieldMask` representation uses protobuf field-name paths, conventionally in `snake_case`. Its JSON mapping converts these to `lowerCamelCase`.

This conversion is not invertible for every legal name. For example, `x_0` becomes `x0`, which cannot be unambiguously converted back to the original name. Consecutive, leading, or trailing underscores can cause similar ambiguities.

Unlike ordinary message-field JSON parsing, the `FieldMask` JSON parser does not necessarily have a message descriptor defining which fields the paths refer to. It therefore cannot resolve these ambiguities by consulting the target schema.

The protobuf specification explicitly documents this limitation. Some newer edition-specific compiler rules reject problematic naming patterns to reduce the ambiguity.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



This is a protocol-level round-trip limitation, separate from the `pbjson-types` implementation gap already documented in the original guide.

Source:

[https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)

### A5.4 Not all non-round-tripping well-known types indicate a library defect

The official ProtoJSON specification also acknowledges that certain valid protobuf value states cannot be faithfully expressed in JSON.

For example, `google.protobuf.Value` uses a floating-point representation for JSON numbers. Protobuf can represent floating-point `NaN`, positive infinity, and negative infinity, but arbitrary JSON numbers do not have equivalent representations for those values.

A serializer may need to reject such a value or produce an output that does not round-trip as the original protobuf value.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation



This distinguishes three sources of interoperability limits:

1. Protocol limitations: The JSON mapping fundamentally cannot represent certain protobuf states without loss.
2. Library implementation gaps: The specified mapping exists, but the selected library does not fully implement it.
3. Configuration-induced differences: The library implements a mapping but has been configured to emit or accept a non-default representation.

These should not be grouped into one undifferentiated claim of serialization compatibility.

## A6. Editions, diagnostics, and feature-independent inspection

Extends original §§3.6, 10.2, 10.3, and 10.4.

### A6.1 Edition 2023 support is not established in the released stack

`prost-reflect` 0.16.5 exposes `Syntax::Proto2` and `Syntax::Proto3`; its public `Syntax` enum has no Editions variant.

The underlying `prost` project's Edition 2023 implementation remains in an open upstream pull request at the research date. The proposed implementation introduces feature-based behavior resolution and changes involving explicit/implicit presence and open/closed enums. The proposal itself characterizes support as partial.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Therefore, the existence of Editions-related descriptor types in `prost-types` or `pbjson-types` does not establish complete Editions support in any of the following:

- `.proto` compilation through the released `prost-build` toolchain.
- Reflection of Edition-dependent semantic behavior.
- Generated Rust representations and binary encoding.
- Generated or dynamic ProtoJSON serialization.

The principal architectural issue is that newer protobuf Editions express semantics through resolved feature settings, rather than the earlier syntax distinction alone.

These settings affect field presence, enum openness, encoding rules, and related behavior. Correctly representing the descriptor structure without interpreting its resolved features may produce incorrect message semantics.

Edition 2023 implementation work should therefore be distinguished from released functionality until the relevant capability is available and verified across the actual compilation, reflection, and serialization layers.

Sources:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/enum.Syntax.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/enum.Syntax.html?utm_source=chatgpt.com)

[https://github.com/tokio-rs/prost/pull/1347](https://github.com/tokio-rs/prost/pull/1347?utm_source=chatgpt.com)

### A6.2 Structured descriptor errors and source-annotated diagnostics

`DescriptorError` provides several layers of information.

Its `file()`, `line()`, `column()`, and `path()` methods expose the file and structural location associated with a descriptor error, when available. Line and column positions are zero-based, and the associated source metadata can be absent.

With the optional `miette` feature, `DescriptorError` implements `miette::Diagnostic`. The `with_source_code()` method allows the original source text to be associated with the error.

However, annotated source diagnostics require the relevant information to be available:

| Input or capability                   | Purpose                                                |
| ------------------------------------- | ------------------------------------------------------ |
| `DescriptorError::path()`             | Identify the affected element in descriptor structure  |
| `SourceCodeInfo`                      | Associate descriptor elements with source spans        |
| `DescriptorError::with_source_code()` | Provide the corresponding original source text         |
| `miette::Diagnostic`                  | Expose information for structured diagnostic rendering |

The original source text supplied through `with_source_code()` must correspond to the file reported by the error. Without matching source text and descriptor source locations, the error can remain informative but cannot necessarily supply a fully annotated source excerpt.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This capability complements, rather than replaces, compiler diagnostics from `protoc` or `protox`. Descriptor validation occurs against the metadata supplied to the pool, while schema compilation can additionally diagnose problems in the original language.

Sources:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorError.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DescriptorError.html?utm_source=chatgpt.com)

[https://github.com/andrewhickman/prost-reflect/blob/main/CHANGELOG.md](https://github.com/andrewhickman/prost-reflect/blob/main/CHANGELOG.md?utm_source=chatgpt.com)

### A6.3 Text-format display is available without the full `text-format` feature

The original guide describes protobuf text-format parsing and configurable formatting behind the `text-format` feature.

A less obvious distinction is that `DynamicMessage` and `Value` also implement `std::fmt::Display` using protobuf text-format-style output. These implementations are available even when the full `text-format` feature is disabled.

`UnknownField` likewise implements `Display` for diagnostic inspection. Its rendering may infer structure because unknown wire fields lack complete semantic type information.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



This separates two capabilities:

- Lightweight diagnostic formatting: `Display` implementations usable without enabling the complete text-format subsystem.
- Full text-format processing: Feature-gated parsing, merging, and configurable output through `DynamicMessage` and `FormatOptions`.

The distinction is useful for libraries that require human-readable diagnostics but do not otherwise need text-format parsing or formatting configuration.

Source:

[https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html](https://docs.rs/prost-reflect/0.16.5/prost_reflect/struct.DynamicMessage.html?utm_source=chatgpt.com)

## A7. Cross-cutting information-preservation boundaries

The capabilities introduced above expose an important general distinction between schema compatibility, semantic preservation, and representation preservation.

| Transformation or operation                               | Important boundary                                                                                             |
| --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Descriptor bytes → `DescriptorPool`                       | Custom options and source metadata depend on descriptor contents and loading path                              |
| Runtime descriptors → generated reflection implementation | Generated types need a valid descriptor-source expression and matching protobuf name                           |
| Binary protobuf → `DynamicMessage` → binary protobuf      | Unknown wire fields can be retained, but byte-identical whole-message output is not guaranteed                 |
| `DynamicMessage` → generated `prost` type                 | Unknown fields may be discarded by the generated decoder                                                       |
| Dynamic value → JSON → dynamic value                      | ProtoJSON presence rules, unknown values, and special well-known-type representations can affect reversibility |
| Enum number → symbolic name                               | Aliases can destroy information about which symbolic name was originally supplied                              |
| `FieldMask` binary → JSON → binary                        | Some legal path names cannot be converted losslessly                                                           |
| Protobuf value → `pbjson-types` Serde output              | Conformance depends on the exact well-known-type implementation, including canonicalization and range handling |
| Descriptor metadata → source diagnostic                   | Source spans, comments, and source text must be present and correctly associated                               |
| Edition-based descriptor → released Rust representation   | Complete resolved-feature semantics are not established in the reviewed stable versions                        |

The practical capability distinction is that `prost-reflect` supplies a rich schema-driven value and metadata model, while `pbjson-build` supplies generated serialization behavior. Neither model automatically makes every intermediate representation lossless.

A shared descriptor set enables both mechanisms to interpret the same schema definitions, but does not eliminate the semantic boundaries introduced by generated representations, JSON mapping, field-presence disciplines, or incomplete implementations of specific protocol features.

The most consequential advanced capability is therefore not merely dynamic inspection or generated JSON serialization in isolation. It is the ability to retain schema intelligence at the points where representation changes could otherwise obscure meaning, while recognizing where that intelligence cannot recover information already discarded.

Verification scope: Public crate APIs, published 0.16.x and 0.9.0 implementation sources, upstream changelogs, protobuf specifications, and open upstream development proposals. The timestamp discrepancy is supported by released source and its published test. No independent compilation, runtime benchmark, or protobuf conformance-suite execution was performed.