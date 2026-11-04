# Buffa 0.9.2 — Comprehensive Rust Library and Toolchain Overview

Research date: October 31, 2026 | Latest verified stable release: 0.9.2 (September 3, 2026) | Project: Anthropic | License: Apache-2.0

Buffa is a pure-Rust Protocol Buffers implementation combining a binary serialization runtime, zero-copy message views, protobuf editions support, ProtoJSON and text-format serialization, runtime reflection, dynamic messages, and a descriptor-driven code-generation toolchain.

Unlike protobuf libraries primarily centered on converting generated Rust structs into binary messages, Buffa has a broader architecture: the same protobuf schema can support owned application data, borrowed read-only representations, dynamic schema-aware processing, and customizable generated representations.

The latest verified release is v0.9.2, published September 3, 2026. This overview targets that release rather than unreleased `main` branch functionality. The project remains in the pre-1.0 version series, so generated API shape and version coordination merit particular attention.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Primary repository: [https://github.com/anthropics/buffa](https://github.com/anthropics/buffa?utm_source=chatgpt.com)

Release documentation: [https://github.com/anthropics/buffa/releases/tag/v0.9.2](https://github.com/anthropics/buffa/releases/tag/v0.9.2?utm_source=chatgpt.com)

Version-specific user guide: [https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md?utm_source=chatgpt.com)

## 1. Architecture and component responsibilities

### 1.1 Package ecosystem

| Component          | Responsibility                                                                                                                                                     |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `buffa`            | Core runtime: message traits, wire encoding/decoding, owned and borrowed representations, optional JSON/text support, unknown fields, and encoding infrastructure. |
| `buffa-build`      | Cargo `build.rs` integration: invokes `protoc` or Buf, accepts descriptor sets, configures code generation, and writes generated Rust files.                       |
| `buffa-codegen`    | Reusable code-generation engine: transforms resolved protobuf descriptors into Rust implementations.                                                               |
| `protoc-gen-buffa` | Executable compiler plugin: implements the standard protoc plugin protocol and connects Buf/protoc to `buffa-codegen`.                                             |
| `buffa-types`      | Generated and extended implementations of Google protobuf well-known types, including special JSON representations and time conversions.                           |
| `buffa-descriptor` | Self-hosted protobuf descriptor definitions, edition metadata, descriptor pools, runtime reflection, and dynamic-message support.                                  |

Important additional companions are `protoc-gen-buffa-packaging` for generated Rust module hierarchies, `buffa-proto-options` for schema-level custom generation settings, and `buffa-remote-derive` for integrating external Rust types into generated messages.

All principal crates belong to a coordinated, same-version workspace.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 1.2 End-to-end processing model

Protocol Buffers schema

`.proto` files · proto2 / proto3 / editions 2023 and 2024

External schema compiler — `protoc` or Buf

Parsing · import resolution · type checking · edition feature resolution

Standard protobuf descriptors

`FileDescriptorProto` / `FileDescriptorSet`

`buffa-build`

Cargo build-script entry point

`protoc-gen-buffa`

Compiler plugin entry point

`buffa-codegen`

Schema-to-Rust mapping · generation options · message implementations

Owned messages

Mutable structs

Borrowed views

Zero-copy reads

Reflection

Descriptor-based access

Runtime ecosystem

`buffa` · `buffa-types` · `buffa-descriptor`

The key architectural boundary is the compiled descriptor. Buffa deliberately does not implement its own `.proto` parser or schema-linking compiler. Instead, it consumes standard protobuf descriptors produced by external compilers. Its code-generation engine is therefore usable independently of the mechanism that obtained those descriptors.

This separation supports several system designs: Cargo-integrated generation, ahead-of-time generation, Buf Schema Registry workflows, externally supplied descriptor sets, and custom code-generation infrastructure, all sharing the same protobuf semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 2. `buffa`: Core runtime and protobuf semantics

The `buffa` crate is the runtime dependency of generated messages. Its design combines strongly typed generated structures with a relatively general wire-format engine and several storage and encoding abstractions.

Runtime API reference: [https://docs.rs/buffa/0.9.2/buffa/](https://docs.rs/buffa/0.9.2/buffa/?utm_source=chatgpt.com)

### 2.1 Generated Rust data model

A typical protobuf message maps to a Rust struct:

```
message Person {
  string name = 1;
  int32 id = 2;
  repeated string tags = 3;
  Address address = 4;
  optional string nickname = 5;
}
```

Illustrative generated representation:

```
pub struct Person {
    pub name: String,
    pub id: i32,
    pub tags: Vec<String>,
    pub address: buffa::MessageField<Address>,
    pub nickname: Option<String>,
    pub __buffa_unknown_fields: buffa::UnknownFields,
}
```

The important distinctions are:

| Protobuf concept         | Buffa representation                | Consequence                                           |
| ------------------------ | ----------------------------------- | ----------------------------------------------------- |
| Implicit-presence scalar | Native scalar (`i32`, `bool`, etc.) | Default and absent are indistinguishable              |
| Explicit-presence scalar | `Option<T>`                         | Absence can be represented independently of the value |
| Singular message         | `MessageField<T, P>`                | Presence-aware, with convenient default-value access  |
| Repeated field           | `Vec<T>` by default                 | Native collection access, customizable storage        |
| Open enum                | `EnumValue<E>`                      | Preserves values unknown to the generated enum        |
| Closed enum              | Native Rust enum                    | Unknown numeric values enter unknown-field storage    |
| Oneof                    | `Option<GeneratedOneofEnum>`        | Mutually exclusive alternatives                       |
| Unknown wire fields      | `UnknownFields`                     | Retained for schema-version round-tripping            |

Generated owned messages implement traits including `Clone`, `Debug`, `PartialEq`, `Default`, and `Message`.

Their shapes can evolve with the schema. Buffa explicitly excludes exhaustive struct construction and destructuring from its struct-evolution compatibility guarantees; construction through `Default` and field updates is the stable pattern.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 2.2 `MessageField<T, P>`: Presence without unnecessary indirection

This is one of Buffa's more consequential departures from the familiar `Option<Box<T>>` representation.

`MessageField<T, P>` distinguishes a missing sub-message from a present one, while implementing `Deref` to a default instance if the field is absent.

```
// Access defaults without unwrapping.
let city = &person.address.city;

// Presence remains explicitly observable.
if person.address.is_set() {
    // Address was provided.
}

// Initialize a missing sub-message and mutate it.
person.address
    .get_or_insert_default()
    .city = "New York".into();

// Recover the optional representation.
let address: Option<&Address> = person.address.as_option();
```

In v0.9.2, the default pointer representation is `Inline<T>`, equivalent in layout to `Option<T>`. Recursive message fields and explicitly configured alternatives use boxed storage.

This has three implications. First, ordinary nonrecursive nested messages need not incur a separate heap allocation for their presence wrapper. Second, the generated Rust representation can preserve presence without imposing `Option` handling throughout application code. Third, controlling whether fields are inline or boxed gives code generation a meaningful influence over message size, allocation behavior, and recursive type layout.

One semantic caution: dereferencing an absent field does not make it present. Code that must distinguish absence from an explicitly supplied default-valued sub-message must still inspect presence.

Source: [https://docs.rs/buffa/0.9.2/buffa/struct.MessageField.html](https://docs.rs/buffa/0.9.2/buffa/struct.MessageField.html?utm_source=chatgpt.com)

### 2.3 `EnumValue<E>`: Open-enum compatibility

Proto3 and open-edition enums must accept numeric values absent from the compiled schema.

Buffa uses:

```
enum EnumValue<E> {
    Known(E),
    Unknown(i32),
}
```

A generated field of type `EnumValue<Status>` can therefore distinguish a recognized variant from a newer enum value while still retaining the underlying integer.

This is valuable for mixed-version systems: an older consumer can receive and subsequently transmit a newer enum value without pretending that it is a known variant.

For closed enums, Buffa uses the native enum directly and places unknown wire values into `UnknownFields`. The code generator also supports selective enum-openness overrides for interoperability cases where the incoming schema cannot be changed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



Source: [https://docs.rs/buffa/0.9.2/buffa/enum.EnumValue.html](https://docs.rs/buffa/0.9.2/buffa/enum.EnumValue.html?utm_source=chatgpt.com)

### 2.4 Binary encoding, decoding, and streaming representations

`buffa::Message` supports several useful entry points:

| API                   | Capability                                                           |
| --------------------- | -------------------------------------------------------------------- |
| `encode_to_vec()`     | Encode into owned contiguous bytes                                   |
| `encode_to_bytes()`   | Produce `bytes::Bytes` output                                        |
| `encode(&mut sink)`   | Encode through the `EncodeSink` abstraction                          |
| `decode_from_slice()` | Decode from a byte slice                                             |
| `decode(&mut buf)`    | Decode using a `bytes::Buf`                                          |
| `merge_from_slice()`  | Merge another serialized message into an existing value              |
| `clear()`             | Reset a message to defaults                                          |
| `encoded_len()`       | Determine encoded message size                                       |
| `try_encode_*()`      | Fallible counterparts for operations subject to protobuf size limits |

Merging follows protobuf semantics: scalar fields generally overwrite prior values, repeated fields append, and nested messages merge recursively.

`EncodeSink` and `Rope` are especially important. A blanket implementation supports ordinary `bytes::BufMut` sinks, while `Rope` represents output as segments of `bytes::Bytes`. Large byte fields can be retained as reference-counted segments instead of copied into a new contiguous output buffer.

This separates the logical serialization operation from the output's physical storage. When an application can transmit segmented buffers, `Rope` can avoid copying substantial byte payloads. When a contiguous buffer is necessary, the advantage may disappear during flattening.

Buffa also provides length-delimited encoding and decoding entry points useful for protobuf record streams, where serialized messages are framed by their lengths rather than by a transport-specific envelope.

### 2.5 Two-pass serialization and `SizeCache`

Protobuf nested messages require a size prefix before their encoded contents. Naively recomputing descendant sizes at every nesting level can perform redundant traversal.

Buffa's solution is an external, pre-order `SizeCache`:

1. `compute_size()` walks the message tree and records nested encoded lengths.
2. `write_to()` emits wire bytes while consuming those cached lengths.

The result is linear-time serialization over the message structure, rather than repeated recursive size calculations. Because the cache is external, generated message structs do not need interior mutability or stored serialization-state fields.

This model also applies to view encoding through `ViewEncode`, which permits borrowed values to be serialized directly.

The ordinary encode methods create and manage the cache automatically. Lower-level callers can use it when integrating a specialized sink or manually implementing `Message`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



Source: [https://docs.rs/buffa/0.9.2/buffa/struct.SizeCache.html](https://docs.rs/buffa/0.9.2/buffa/struct.SizeCache.html?utm_source=chatgpt.com)

### 2.6 Decode safety and resource controls

Binary protobuf input can be small on the wire yet expensive to materialize, particularly when many empty messages or short unknown fields expand into substantial Rust structures.

`DecodeOptions` provides independently useful controls:

| Control              | v0.9.2 default | Purpose                                                                   |
| -------------------- | -------------- | ------------------------------------------------------------------------- |
| Recursion depth      | 100            | Bound nested-message traversal                                            |
| Maximum message size | 2 GiB − 1 byte | Bound input size                                                          |
| Unknown field count  | 1,000,000      | Bound the number of retained unknown elements                             |
| Element memory       | 32 MiB         | Bound materialized repeated-message/string/bytes elements and map entries |

The element-memory limit addresses amplification not covered by wire-byte limits. For example, a very short encoded empty sub-message can materialize into a relatively large Rust struct.

These controls have important qualifications:

- They apply to owned, eager-view, and dynamic binary decoders, but are not a general cap on total process memory.
- Packed scalar arrays are deliberately exempt from the element-memory budget in typed decoders; dynamic reflection charges them because each scalar occupies a larger generic `Value`.
- JSON deserialization through Serde is not protected by `DecodeOptions`. JSON requires separate input and resource controls.
- Textproto has its own element-memory and recursion controls.
- Lazy views defer some validation and apply certain resource budgets per deferred subtree rather than as one global materialization budget.

For output, the protobuf message-size limit also matters: ordinary encoding entry points panic if the message is too large, while the `try_*` variants return `EncodeError::MessageTooLarge`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



Source: [https://docs.rs/buffa/0.9.2/buffa/struct.DecodeOptions.html](https://docs.rs/buffa/0.9.2/buffa/struct.DecodeOptions.html?utm_source=chatgpt.com)

## 3. Zero-copy views, lazy decoding, and ownership

Buffa's view system is arguably its most distinctive runtime capability. It creates several representations with different ownership, validation, and performance properties.

### 3.1 Eager borrowed views: `MessageView`

For each generated owned message, Buffa normally generates a borrowed view under `pkg::__buffa::view`.

For example:

```
pub struct PersonView<'a> {
    pub name: &'a str,
    pub id: i32,
    pub tags: buffa::RepeatedView<'a, &'a str>,
    pub address: buffa::MessageFieldView<AddressView<'a>>,
    pub nickname: Option<&'a str>,
}
```

String and byte payloads borrow directly from the input, while numeric scalars are decoded into native values.

The distinction is important: zero-copy payload access does not mean the entire decode has zero allocations. Repeated fields use a `Vec`-backed view sequence, and `MapView` also stores its decoded entries. The map representation performs linear lookup, which is suitable for small protobuf maps but not a replacement for a hash index over large maps.

An eager view validates its nested content as it decodes. The resulting view can be used for read-only access, encoded again, converted to an owned message, or reflected when the corresponding reflection implementation is generated.

### 3.2 `OwnedView<V>`: Lifetime-safe buffer ownership

A borrowed view's lifetime normally prevents it from being returned independently of its input buffer.

`OwnedView<V>` pairs a view with the `bytes::Bytes` allocation it borrows from. Codegen additionally emits convenient per-message wrappers such as `PersonOwnedView`.

This creates a useful separation between ownership of the buffer and ownership of each decoded field.

`OwnedView` can move between asynchronous tasks and service boundaries while retaining borrowed access to its own buffer. The key API is `reborrow()`, which ties the returned field references to the lifetime of the handle rather than exposing the view's synthetic internal `'static` lifetime.

```
let handle = PersonOwnedView::decode(bytes)?;

let name: &str = handle.name();

// Or access the full borrowed view:
let view = handle.view();
```

The handle can be cheaply cloned through reference-counted `Bytes` ownership. Converting an eager owned-view handle to a fully owned generated message is infallible; it was already validated when decoded.

A consequence for RPC architectures is that the transport body does not necessarily need to be copied into a hierarchy of owned `String`, `Vec<u8>`, and message allocations just to satisfy `'static` requirements.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



Source: [https://docs.rs/buffa/0.9.2/buffa/view/struct.OwnedView.html](https://docs.rs/buffa/0.9.2/buffa/view/struct.OwnedView.html?utm_source=chatgpt.com)

### 3.3 Lazy view family: `lazy_views(true)`

Buffa can generate an additional `FooLazyView<'a>` representation without changing the normal eager view.

The mechanism is selective decoding: the initial scan identifies nested message boundaries and records their encoded byte ranges. The nested content is decoded only when requested.

```
use buffa::LazyMessageView;

let view = PersonLazyView::decode_lazy(&bytes)?;

// Decode the nested message on demand.
if let Some(address) = view.address.get()? {
    println!("{}", address.city);
}

// Repeated nested values decode individually.
for item in view.friends.iter() {
    let friend = item?;
    // Inspect friend.
}
```

This is particularly interesting for messages containing large repeated sub-message collections when only a small number of fields will be accessed.

However, it changes several guarantees:

| Property                 | Eager view                          | Lazy view                     |
| ------------------------ | ----------------------------------- | ----------------------------- |
| Nested-message decoding  | During initial decode               | On access                     |
| Error detection          | Eager for represented fields        | Can be deferred               |
| Repeated nested access   | Decoded value retained              | Re-decodes on repeated access |
| Whole-message conversion | Infallible for wire-decoded views   | Fallible                      |
| Re-encoding              | Encode parsed representation        | Can replay deferred bytes     |
| Runtime reflection       | Supported with generated reflection | Not supported                 |
| `OwnedView` integration  | Supported                           | Not supported                 |
| Textproto surface        | Supported when generated            | Not supported                 |

Groups, oneof message variants, map message values, and external message types remain eagerly decoded.

The architectural advantage is avoiding work on subtrees that are not examined. The cost is deferred validation, potentially repeated decoding, and a narrower integration surface. Lazy decoding is therefore not interchangeable with eagerly validated message input at a trust boundary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 3.4 `ViewEncode`: Borrowed output construction

View types can also be constructed directly and serialized through `buffa::ViewEncode`.

This is distinct from decoding: a caller can assemble a message from existing `&str`, `&[u8]`, and other borrowed values without first building an owned struct.

It creates a useful symmetry:

- Inbound binary → borrowed view → field inspection.
- Borrowed application values → view → outbound binary.
- Inbound binary → borrowed view → outbound binary, without full ownership conversion.

For message emitters already working with borrowed data, this can eliminate allocations that would otherwise be incurred simply to populate generated owned fields. It is also a possible basis for efficient gateways that inspect and forward protobuf messages without modifying their logical contents. Byte-for-byte preservation of arbitrary original wire ordering should not be assumed solely from this re-encoding capability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 3.5 `HasMessageView`: Generic framework integration

`HasMessageView` associates a generated owned message with its borrowed-view and handle types:

```
M::View<'a>
M::ViewHandle
M::decode_view_handle(bytes)
```

This allows framework code to operate generically over a message type without independently naming its generated view or wrapper.

A generic RPC transport, for example, can decode a body into `M::ViewHandle` and pass the result to a handler whose message type is known statically, while still using borrowed representation internally.

This is a significant capability interaction: typed protobuf schemas, zero-copy decode, and safe asynchronous ownership can be composed without bespoke self-referential message wrappers for every type.

Source: [https://docs.rs/buffa/0.9.2/buffa/trait.HasMessageView.html](https://docs.rs/buffa/0.9.2/buffa/trait.HasMessageView.html?utm_source=chatgpt.com)

## 4. `buffa-build`: Cargo and descriptor-driven compilation

`buffa-build` is the integration layer for generating Rust code during a Cargo build. It delegates protobuf parsing and linking to `protoc` or Buf and then uses `buffa-codegen`.

API reference: [https://docs.rs/buffa-build/0.9.2/buffa_build/struct.Config.html](https://docs.rs/buffa-build/0.9.2/buffa_build/struct.Config.html?utm_source=chatgpt.com)

### 4.1 Three compilation paths

| Input path             | Mechanism                                                  | Architectural implication                                                    |
| ---------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `protoc` (default)     | Invoke `protoc` with `.proto` files and import directories | Conventional local build integration; requires a suitable installed compiler |
| `use_buf()`            | Run `buf build` to obtain descriptors                      | Integrates with Buf module and dependency resolution                         |
| `descriptor_set(path)` | Consume an existing binary `FileDescriptorSet`             | Decouples Rust compilation from schema compilation entirely                  |

The third mechanism is easy to overlook. It allows schema compilation and code generation to occur in separate stages, potentially managed by different tools or build systems. It also makes it possible to generate Rust types from a previously validated or externally obtained descriptor artifact without invoking either `protoc` or Buf in that step.

An illustrative build script:

```
fn main() {
    buffa_build::Config::new()
        .files(&["api/v1/service.proto"])
        .includes(&["proto"])
        .include_file("proto_mod.rs")
        .generate_views(true)
        .generate_json(true)
        .compile()
        .unwrap();
}
```

With the default output directory, the consuming Rust module can use:

```
include!(concat!(env!("OUT_DIR"), "/proto_mod.rs"));
```

`include_file()` generates the module hierarchy instead of requiring manually maintained `pub mod` declarations. When `out_dir()` is explicitly configured, its generated includes can be sibling-relative, supporting checked-in generated sources.

Buf path semantics matter: when invoking `use_buf()`, `.files()` expects names relative to the owning Buf module rather than necessarily relative to the filesystem location of `buf.yaml`. The `includes()` setting is ignored because Buf resolves imports through its own configuration. For broad multi-module workspaces, direct `buf generate` or a precompiled descriptor set may offer a cleaner boundary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 4.2 Code-generation configuration families

`Config` exposes substantially more than file selection:

| Family                      | Representative controls                                                                                                | What it changes                                        |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| Runtime surfaces            | `generate_views`, `lazy_views`, `generate_json`, `generate_text`, `generate_arbitrary`                                 | Optional generated traits and representations          |
| Reflection                  | `generate_reflection`, `reflect_mode`, `shared_descriptor_pool`                                                        | Runtime descriptor access and dynamic introspection    |
| Rust names and layout       | `type_name_prefix`, `idiomatic_enum_aliases`, `idiomatic_field_names`, `include_file`, `file_per_package`              | Generated Rust API and file organization               |
| External dependencies       | `extern_path`, `exclude_package`                                                                                       | Reuse message types generated or implemented elsewhere |
| Field storage               | `bytes_type`, `string_type`, `map_type`, `repeated_type`, `box_type`, `unbox_oneof`                                    | Physical Rust representations                          |
| Schema compatibility        | `override_feature_in`, `open_enums_in`, `strict_utf8_mapping`, `allow_message_set`                                     | Treatment of selected protobuf semantics               |
| Custom API surface          | `type_attribute`, `message_attribute`, `field_attribute`, `enum_attribute`, `oneof_attribute`, `generate_with_setters` | Additional generated traits and conveniences           |
| Optional crate capabilities | `gate_impls_on_crate_features` and feature-name overrides                                                              | Conditional compilation of generated implementations   |

Most configuration rules targeting fields or types use fully qualified protobuf paths, with `"."` selecting the whole corpus and more specific paths selecting individual packages, messages, or fields.

This allows representation decisions to be fine-grained rather than global. A large payload field can use `bytes::Bytes` while other byte fields remain `Vec<u8>`; a specific map can use `BTreeMap` while the rest use `HashMap`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.3 Generated-feature gating

A library distributing generated message types may want consumers to choose whether to compile JSON, textproto, views, or reflection support.

Buffa provides `gate_impls_on_crate_features(true)` to emit conditional implementations behind consumer-defined Cargo features. The feature names can be customized for several capability families.

This is distinct from the `generate_*` controls:

- `generate_json(false)` does not emit JSON implementations at all.
- `generate_json(true)` with feature gating emits them conditionally.
- `generate_json(true)` without gating emits them unconditionally, requiring the corresponding runtime support.

The feature names must exist in the consuming crate, not merely in `buffa`. An undeclared generated `cfg(feature = "...")` can silently suppress the implementation, producing confusing missing-trait diagnostics later.

This mechanism is especially relevant to published protobuf model crates that expose several optional functionality levels to downstream consumers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 5. `buffa-codegen` and `protoc-gen-buffa`: Generation engine and compiler plugin

### 5.1 Separate library and executable responsibilities

`buffa-codegen` is a Rust library, not a `.proto` compiler. It consumes resolved descriptors and emits source artifacts.

Its public surface includes:

- `generate()` and `generate_with_diagnostics()` for code emission.
- `CodeGenConfig` for generated representation and API configuration.
- `GeneratedFile` and `GeneratedFileKind` for output artifacts.
- `generate_module_tree()` for package-based Rust module structure.
- `SharedCorpusContext` for reusing schema-global analysis across code-generation passes.
- Supporting utilities for descriptor-set embedding, package exclusion, names, and plugin-request decoding.

`protoc-gen-buffa` wraps this engine in the standard compiler-plugin protocol: read `CodeGeneratorRequest` from standard input, generate code, and write `CodeGeneratorResponse` to standard output.

Its execution model works with both `protoc` and Buf, including Buffa's published Buf Schema Registry plugin.

Library API: [https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/?utm_source=chatgpt.com)

Configuration: [https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/struct.CodeGenConfig.html](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/struct.CodeGenConfig.html?utm_source=chatgpt.com)

Buf plugin: [https://buf.build/anthropics/buffa](https://buf.build/anthropics/buffa?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.2 Descriptor compilation is separate from Rust generation

The division of responsibilities is deliberate.

`protoc` or Buf determines protobuf symbol identity, import resolution, field types, syntax validity, and edition-specific semantics. Buffa then maps those validated descriptors to Rust data structures and implementations.

Consequently, `buffa-codegen` is not an AST-oriented `.proto` transformation library. It is a schema-to-Rust compiler backend.

This enables custom generators that consume a single normalized descriptor corpus, but it does not replace protobuf source parsing, schema validation, or edition feature resolution. Any schema transformations performed before generation must still result in descriptors that faithfully express the intended wire semantics.

### 5.3 Module structure and naming

Buffa separates normal message and enum names from auxiliary generated functionality.

For a protobuf package `company.api.v1`, representative output paths include:

```
company::api::v1::Request
company::api::v1::Status

company::api::v1::__buffa::view::RequestView
company::api::v1::__buffa::view::RequestOwnedView

company::api::v1::__buffa::oneof::request::Payload
company::api::v1::__buffa::ext::MY_EXTENSION
```

Nested protobuf messages are represented through corresponding Rust modules. Keyword collisions and protobuf naming conventions are handled by the generator.

`type_name_prefix()` can prefix generated message and enum type names without changing the protobuf wire schema. This can be useful when independently generated API domains must coexist in the same Rust namespace.

Buffa also generates `UpperCamelCase` aliases for protobuf enum variants by default. These are additive associated constants; they do not rename the authoritative protobuf enum variants. Ambiguous aliases are suppressed rather than emitted inconsistently.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 5.4 External type mapping and package reuse

`extern_path` maps a protobuf package or fully qualified type to an existing Rust module or type.

For example:

```
buffa_build::Config::new()
    .extern_path(
        ".company.common",
        "::company_common_proto",
    )
```

The generator can reference shared types from another crate instead of generating a duplicate implementation.

This capability is more important than ordinary module organization: it supports separating protobuf ownership across Rust crates while preserving a common schema.

Specific mappings take precedence over broader package mappings, and the generator automatically maps Google well-known types to `buffa-types` and protobuf descriptor types to `buffa-descriptor`.

There are constraints. An externally mapped message used in borrowed-view output must expose the corresponding view functionality. Reflection likewise requires compatible reflection capabilities from those external types. Excluding a package without supplying the necessary mappings does not magically resolve its references.

A useful general distinction is between protobuf type identity (the schema's fully qualified name) and Rust type location (the compiled crate/module path). `extern_path` controls the latter without changing the former.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.5 Corpus-wide analysis and reusable generation state

`SharedCorpusContext` is an advanced API for programs generating different Rust crates or packages from the same protobuf corpus.

Some code-generation decisions require visibility into the entire schema, including which message edges are recursive, which oneof message variants may be inlined, and the collection of source comments. Repeating those analyses for every output package would duplicate work.

A shared context allows this information to be computed once and reused across multiple `generate()` calls, even when each output uses different external-path mappings.

The context is not an unconstrained cache: it must match the descriptor corpus and relevant pointer/oneof configuration. Buffa checks these invariants and rejects mismatched contexts.

This is a meaningful extension point for multi-crate protobuf generation infrastructure. It supports coordinated generation without having to reimplement cross-schema recursion analysis or textual comment collection.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.6 Build/plugin capability differences

The plugin and build-script interfaces share a code-generation engine, but not every advanced configuration is exposed identically.

The compiler plugin provides options for common generation features, reflection, field openness overrides, type prefixes, external paths, package exclusion, and several layout decisions.

The `buffa-build` API exposes richer Rust-native customization, including custom field storage types, map/repeated collection strategies, and granular generated attributes. Direct `buffa-codegen` callers have access to its lower-level configuration model.

A practical distinction follows: a feature supported by the Rust code-generation library is not necessarily configurable through the published remote Buf plugin.

The separate `protoc-gen-buffa-packaging` plugin can generate a Rust module tree from a schema corpus. It is particularly useful when generated source is intended to be checked in rather than included automatically from Cargo's `OUT_DIR`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



Release-scope clarification: `buffa-proto-options`, visible in some mutable upstream development material, is not a member of the tagged v0.9.2 workspace. It should not be treated as a released companion to the version documented here. Native protobuf custom options and extensions are a separate, supported facility discussed below.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 6. `buffa-types`: Well-known protobuf types and semantic adapters

`buffa-types` provides pre-generated implementations of Google's well-known protobuf messages, together with functionality that ordinary message generation cannot supply by itself.

API reference: [https://docs.rs/buffa-types/0.9.2/buffa_types/](https://docs.rs/buffa-types/0.9.2/buffa_types/?utm_source=chatgpt.com)

### 6.1 Type families

| Family               | Types                                                                    | Significant capabilities                                                 |
| -------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| Time                 | `Timestamp`, `Duration`                                                  | Nanosecond-precision protobuf representation, validation and conversions |
| Polymorphic messages | `Any`                                                                    | Type URL plus embedded serialized protobuf; typed packing and unpacking  |
| Dynamic values       | `Struct`, `Value`, `ListValue`, `NullValue`                              | JSON-like objects, heterogeneous arrays and tagged values                |
| Field selection      | `FieldMask`                                                              | Standard protobuf field-path representation and specialized JSON syntax  |
| Nullable wrappers    | `Int32Value`, `Int64Value`, `BoolValue`, `StringValue`, etc.             | Well-known scalar wrapper message identities                             |
| Empty payload        | `Empty`                                                                  | Standard empty message                                                   |
| API/type metadata    | `Api`, `Type`, `Enum`, `SourceContext`, `Method`, `Mixin`, `Field`, etc. | Google's protobuf-defined API and type metadata model                    |

These are actual protobuf message types rather than aliases of ordinary Rust types. The package adds ergonomic conversions while maintaining their binary identities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.2 Time conversions and representations

`Timestamp` and `Duration` implement conversions to and from common Rust time types.

The available integrations are feature-dependent:

| Feature  | Integration                                    |
| -------- | ---------------------------------------------- |
| `std`    | `std::time::SystemTime`, `std::time::Duration` |
| `chrono` | `chrono::DateTime` and `chrono::TimeDelta`     |
| `jiff`   | `jiff::Timestamp` and `jiff::SignedDuration`   |

The signedness distinction matters. Protobuf `Duration` can represent negative durations, whereas `std::time::Duration` cannot. Conversions therefore have validity and range constraints; not every protobuf duration can be represented by every target type.

The `chrono` and `jiff` integrations are available in `no_std` configurations compatible with those libraries' corresponding feature sets.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.3 `Any` and type registries

`Any` carries two pieces of information: a type URL identifying a protobuf message and its binary-encoded payload.

`Any::pack` and corresponding unpacking helpers avoid manually assembling these fields when the message type is statically known.

At the next abstraction level, Buffa's `TypeRegistry` can register generated message types for operations needing runtime type resolution, including expanding `Any` values in JSON and textproto.

The distinction is between type identity and type implementation:

- An `Any` value preserves an embedded message and its declared type URL without requiring the surrounding message to know that type.
- Interpreting its fields requires a compatible type definition, either registered as generated code or available through runtime descriptors.
- Arbitrary or unrecognized type URLs cannot be assumed to decode into concrete Rust message types.

This makes `Any` useful for extensible message envelopes, but its success does not establish that a receiver knows how to interpret the embedded schema.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 6.4 Specialized ProtoJSON semantics

Well-known types are not all serialized to JSON as conventional nested objects.

| Type                             | ProtoJSON representation                                                   |
| -------------------------------- | -------------------------------------------------------------------------- |
| `Timestamp`                      | RFC 3339 timestamp string                                                  |
| `Duration`                       | Duration string with `s` suffix                                            |
| `FieldMask`                      | Comma-separated JSON-style field paths                                     |
| `Struct` / `Value` / `ListValue` | Ordinary JSON objects, values, and arrays                                  |
| Scalar wrappers                  | JSON scalar or `null`, according to their protobuf semantics               |
| `Any`                            | Type metadata under `@type`, plus the embedded value's JSON representation |

`buffa-types/json` supplies the specialized serde implementations needed for this mapping.

Important exception: the `Api`, `Type`, `Enum`, `SourceContext` family and their component messages do not have JSON serde implementations in v0.9.2. A generated message containing these types can fail to compile when JSON generation is enabled. This is a real feature-coverage boundary, not just a missing convenience function.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 6.5 Separate package dependencies and feature forwarding

Protobuf messages importing well-known types automatically reference `buffa-types` through generated Rust paths. A consumer only needs this dependency if its generated types actually reference those message types.

Reflection across messages containing well-known types needs an additional step: enable `buffa-types/reflect` so those embedded types participate in the `buffa-descriptor` reflective interface.

The `reflect` feature is independent of JSON support and requires `std`. This is an example of an important package boundary: generating reflective implementations for the outer message does not automatically enable reflection on every externally defined nested message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 7. `buffa-descriptor`: Descriptor model, reflection, and dynamic messages

`buffa-descriptor` has two distinct roles:

1. It provides strongly typed Rust representations of protobuf's own descriptor schemas.
2. With the `reflect` feature, it becomes a runtime schema engine capable of linking descriptors, inspecting types, and decoding messages that have no corresponding compiled Rust struct.

API reference: [https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/?utm_source=chatgpt.com)

### 7.1 Self-hosted schema representation

The crate includes generated representations of:

- `google/protobuf/descriptor.proto`: `FileDescriptorSet`, `FileDescriptorProto`, `DescriptorProto`, `FieldDescriptorProto`, `FeatureSet`, `Edition`, options, services, methods, extensions, and related metadata.
- `google/protobuf/compiler/plugin.proto`: `CodeGeneratorRequest`, `CodeGeneratorResponse`, and the compiler-plugin protocol.

These representations are generated by Buffa itself and checked into its source tree. The compiler tools can therefore read protobuf descriptors without depending on prost, Google's Rust protobuf runtime, or another serializer.

The package exposes descriptor type families separately from the well-known messages in `buffa-types`. A project merely using `Timestamp` does not need the descriptor crate, while a schema compiler, inspection tool, or dynamic-protobuf processor generally does.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.2 `DescriptorPool`: Linked schema identity

A serialized `FileDescriptorSet` is data: a collection of file descriptors and their declarations.

A `DescriptorPool` adds the linked semantic model. It resolves references among types and exposes structured, navigable handles such as:

| Descriptor                               | Provides                                                             |
| ---------------------------------------- | -------------------------------------------------------------------- |
| `MessageDescriptor`                      | Message identity and declared fields                                 |
| `FieldDescriptor`                        | Tag number, name, kind, cardinality, resolved type, edition features |
| `EnumDescriptor`                         | Enum identity, values, names, and numeric mappings                   |
| `OneofDescriptor`                        | Mutually exclusive field grouping                                    |
| `ExtensionDescriptor`                    | Externally declared field and extendee identity                      |
| `ServiceDescriptor` / `MethodDescriptor` | RPC declarations, input and output message relationships             |

The pool exposes lookup operations by names and indices. Field metadata includes normalized semantic information rather than requiring every client to interpret the raw `FieldDescriptorProto` themselves.

This is the difference between storing a descriptor and using a schema graph. The linked representation can support generic schema inspection, dynamically configured message processors, generic validation, API discovery, and tools that need to understand relationships among message types.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.3 `DynamicMessage`: Decode without generated Rust types

With reflection enabled, `DynamicMessage` represents an actual protobuf message through descriptor identity and runtime values.

Its core model consists of:

- A shared `Arc<DescriptorPool>`.
- An index identifying the message schema.
- A field-number-indexed collection of dynamically typed values.
- Unknown-field state for protobuf compatibility.

It supports binary decode and encode, inspection and mutation of fields, repeated and map values, JSON conversion when enabled, and reflective traversal.

Values are distinguished by their protobuf scalar, message, list, and map kinds. A caller can therefore inspect arbitrary message structures by descriptor rather than matching against a Rust enum containing every known generated message type.

For example, a dynamic processor may load an external descriptor set, locate the descriptor of `company.events.Event`, decode an event's wire payload, inspect a field by tag number, and re-encode the modified message. The Rust binary need not have been compiled with `Event` as a generated struct.

Unlike a loosely typed JSON document, this retains protobuf-specific information including field numbers, integer and enum types, presence rules, and binary wire encoding.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### 7.4 Generated-type reflection: vtable versus bridge

Buffa can connect its generated, strongly typed messages to the same descriptor-driven interfaces through `Reflectable`, `ReflectMessage`, and related traits.

There are two strategies:

`ReflectMode::VTable`

Default when reflection enabled

Generated traits expose fields directly from the owned message or borrowed view.

Avoids a serialization round-trip and is suited to repeated field inspection. Generates more Rust implementation code.

`ReflectMode::Bridge`

Reflection materializes a `DynamicMessage` through binary encode/decode.

Produces smaller generated code but incurs conversion and allocation costs when accessing the reflective representation.

Both expose a common reflective interface, so caller code need not fundamentally change when choosing between them.

In vtable mode, `ValueRef<'a>` provides borrowed access to values: strings and bytes remain borrowed, nested messages are available through reflective message handles, and lists or maps use reflective collection interfaces. This avoids materializing a dynamic tree merely to inspect a known generated object.

That makes the reflection system broader than conventional descriptor lookup: it unifies static typed messages, borrowed views, and dynamic messages behind a schema-aware access model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



### 7.5 Shared descriptor pools and generated-artifact size

Generated reflection embeds a `FileDescriptorSet` and exposes `descriptor_pool()` and `FILE_DESCRIPTOR_SET_BYTES` from the generated package.

If many generated packages each embed the entire schema corpus, the descriptor data can be duplicated substantially.

`shared_descriptor_pool(true)` changes this by emitting a shared root that owns one descriptor set and one lazily initialized `DescriptorPool`; individual packages delegate to it.

This brings two benefits:

- Less duplicated descriptor data in generated artifacts.
- Consistent pool identity across packages, which matters when dynamically typed values must be composed or compared using shared schema metadata.

The mechanism has real build-system requirements.

In `buffa-build`, it requires reflection plus `include_file()`. It emits a binary `*.descriptor_set.binpb` sidecar referenced through `include_bytes!`, which must accompany checked-in generated source.

For the Buf plugin workflow, `shared_descriptor_pool=true` must be passed consistently to both `protoc-gen-buffa` and `protoc-gen-buffa-packaging`. Their input corpora must agree. Otherwise the generated code can reference a missing shared root or a descriptor pool lacking necessary types.

The plugin path also restricts certain combinations with feature overrides, feature gating, and `file_per_package`; the build-script path supports combinations the plugin cannot coordinate.

This capability is particularly consequential for large descriptor corpora because it addresses duplicate schema embedding rather than only per-message runtime cost.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 7.6 Reflective interoperability and its limits

`DynamicMessage::try_from_message` supports bridging a generated message into a dynamic representation, returning an error for incompatible schema or decoding conditions.

The reflective APIs are powerful but cannot infer an unknown schema from arbitrary wire bytes. Protobuf binary encoding carries field numbers and wire types, not complete message and field definitions. A descriptor pool must identify what each field means.

Likewise, descriptor identity matters: two independently constructed pools do not automatically provide the same object identity even when their schemas appear structurally equivalent. Shared-pool arrangements can avoid this class of integration friction.

Reflection requires `std` in this release, and its feature dependency is separate from the default lightweight descriptor message types. The normal code-generation toolchain can use `buffa-descriptor` without bringing the runtime reflection engine into applications.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2



## 8. Protobuf editions, presence semantics, and compatibility

### 8.1 Unified treatment of proto2, proto3, and editions

Buffa supports proto2, proto3, and protobuf editions 2023 and 2024.

A defining design choice is to treat them through the resolved protobuf feature model, rather than maintaining largely separate runtime implementations for each syntax generation.

Relevant features include:

| Edition feature           | Important alternatives                    | Effect on generated behavior                                   |
| ------------------------- | ----------------------------------------- | -------------------------------------------------------------- |
| `field_presence`          | `EXPLICIT`, `IMPLICIT`, `LEGACY_REQUIRED` | Whether absence is distinguishable from a default-valued field |
| `enum_type`               | `OPEN`, `CLOSED`                          | Whether unknown numeric enum values are directly representable |
| `repeated_field_encoding` | `PACKED`, `EXPANDED`                      | Default wire encoding of repeated numeric values               |
| `utf8_validation`         | `VERIFY`, `NONE`                          | String decoding requirements                                   |
| `message_encoding`        | `LENGTH_PREFIXED`, `DELIMITED`            | Conventional nested-message versus group-style encoding        |
| `json_format`             | `ALLOW`, `LEGACY_BEST_EFFORT`             | JSON semantics under the resolved edition                      |

This design means that the generated message representation reflects the effective semantics of the schema, not simply the source file's `syntax` declaration.

The external protobuf compiler remains responsible for resolving feature inheritance. That is why its version matters:

| Schema support    | Minimum `protoc` version |
| ----------------- | ------------------------ |
| Proto2 and proto3 | 21.12                    |
| Editions 2023     | 27.0                     |
| Editions 2024     | 33.0                     |

Buf's own protobuf compiler is an alternative, without the need to install a corresponding standalone `protoc`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 8.2 Path-scoped feature overrides

`override_feature_in()` and `open_enums_in()` allow selected semantics to be changed at code-generation time when the original `.proto` source cannot be modified.

In v0.9.2, the supported override family is principally opening closed enums.

This can be useful when a system wants explicit access to unknown enum numbers rather than having them retained only in unknown-field storage.

However, overrides are not purely cosmetic. They can change generated Rust field types, presence behavior, and reflected descriptors.

An especially subtle limitation is that a field-scoped enum-openness override is Buffa-specific; other protobuf implementations consuming the exported descriptor set may not recognize it as a valid edition feature. An enum-declaration-level override generally has better schema portability when that declaration is available.

An unmatched override produces a diagnostic rather than silently appearing to have worked.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 8.3 UTF-8 validation versus byte-preserving strings

By default, Buffa represents protobuf `string` fields as `String` and `&str`, validating UTF-8 even when the schema's effective `utf8_validation` feature is `NONE`.

That is stricter than some proto2-compatible behavior, but avoids allowing invalid UTF-8 into Rust string values.

`strict_utf8_mapping(true)` instead maps those fields to `Vec<u8>` and `&[u8]`, allowing arbitrary byte sequences.

This creates a consequential tradeoff:

- Binary interoperability can improve for legacy schemas containing non-UTF-8 bytes in string fields.
- Rust APIs lose ordinary string typing and require explicit character decoding.
- ProtoJSON interoperability can change because the mapped field may serialize using base64 byte semantics instead of the expected JSON string representation.
- In proto2, where `utf8_validation = NONE` is generally the default, this option can change a large number of generated field types.

It is therefore a semantic interoperability control, not merely a performance toggle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 8.4 Unknown fields and version-tolerant round trips

Unknown-field preservation is enabled by default.

When an older implementation reads a newer message, it can retain unrecognized wire fields and write them back during subsequent serialization. This helps preserve data through intermediate services that do not understand every field.

Unknown-field preservation does not guarantee byte-for-byte identity. Field ordering, serialization decisions, and representation can still change.

The cost is principally memory and generated representation size: approximately 24 bytes per owned message for the unknown-field vector header, plus an additional handle for borrowed views. Disabling preservation removes that storage but sacrifices an important forward-compatibility property.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 9. Extensions, custom options, and registries

### 9.1 Typed extensions on top of unknown-field storage

Proto2 extensions and protobuf custom options are supported.

Buffa generates typed extension constants under the generated package's `__buffa::ext` module.

The `ExtensionSet` trait supplies accessors such as:

```
use buffa::ExtensionSet;

let rules = field_options.extension(&MY_EXTENSION);

if field_options.has_extension(&MY_EXTENSION) {
    // The extension tag is present.
}

field_options.set_extension(&MY_EXTENSION, value);
field_options.clear_extension(&MY_EXTENSION);
```

`Extension<C>` is parameterized by the codec, not merely by the Rust value type. This distinction lets different protobuf wire encodings produce the same Rust scalar type without collapsing their serialization semantics.

Extension values are retained through unknown-field storage and decoded lazily when accessed, rather than requiring every registered extension to be materialized during initial message decoding.

This avoids a registration-order problem common to eager extension systems: a value decoded before its extension definition is registered can still be interpreted later because the raw field was retained.

The costs are repeated decoding on access and the need to preserve unknown-field state. Extension access also checks the declaring extendee type; mismatched access generally panics, while `has_extension()` returns false.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### 9.2 Extensions in JSON, textproto, and reflection

Protobuf JSON represents extension fields using bracketed, fully qualified names:

```
{
  "[company.validation.field]": {
    "required": true
  }
}
```

Buffa's `TypeRegistry` can register both message types and extensions. Generated `register_types()` functions register relevant definitions for their packages.

The same registry infrastructure supports `Any` expansion and bracketed extension names in JSON and textproto.

A notable default is that unregistered bracketed extension keys may be discarded during JSON parsing. `JsonParseOptions::strict_extension_keys(true)` changes that behavior to reject unknown extension names.

Another important asymmetry: borrowed-view JSON serialization does not emit extension fields. The owned message serialization path is needed when extension-preserving JSON output is required.

Runtime reflection offers a different route for inspecting and manipulating descriptor-known extensions without relying entirely on statically generated extension constants.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 9.3 Legacy MessageSet

Buffa also includes support for the legacy protobuf MessageSet wire format, but this is deliberately opt-in through `allow_message_set`.

The feature is relevant to compatibility with specialized older protobuf ecosystems, not ordinary contemporary schema design.

It illustrates a general property of Buffa's coverage: some historical protobuf wire capabilities are implemented without being enabled as the default code-generation behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



## 10. ProtoJSON and textproto

### 10.1 JSON is schema-aware, not ordinary Rust serde mapping

Buffa implements protobuf's JSON mapping through generated serde implementations and runtime helpers.

Important differences from a generic `#[derive(Serialize, Deserialize)]` include:

| Protobuf feature          | JSON behavior                                              |
| ------------------------- | ---------------------------------------------------------- |
| `snake_case` field        | Uses configured JSON name, conventionally `lowerCamelCase` |
| 64-bit integer            | Encoded as a decimal string                                |
| `bytes`                   | Base64 string                                              |
| Known enum                | Protobuf enum name                                         |
| Unknown open-enum value   | Numeric value                                              |
| Implicit-presence default | Commonly omitted                                           |
| Explicit-presence field   | Can retain presence-sensitive representation               |
| `Any` and other WKTs      | Specialized mapping                                        |

Generated owned messages support serialization and deserialization when JSON is enabled. Generated borrowed views support serialization, but not the same general deserialization surface as owned types.

JSON requires both `buffa/json` and code generation with JSON enabled, plus the appropriate serde dependencies and relevant well-known-type features.

A normal serde-derived representation of the same Rust fields is not a substitute for ProtoJSON. It may differ in integer handling, enum formatting, names, default omission, and well-known-type behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 10.2 JSON parsing controls and security boundaries

`JsonParseOptions` configures selected parsing behaviors, including strictness around extension names.

In `std` builds, scoped options can be applied through `with_json_parse_options`, using thread-local state for the duration of a parsing operation.

Two limitations matter:

First, these controls are distinct from the binary codec's `DecodeOptions`. Binary size, recursion, and element-memory budgets should not be assumed to constrain a Serde JSON decode.

Second, thread-local scoped configuration has different availability in `no_std` environments, which have a more limited global-configuration model.

This makes it important to distinguish the security and resource contracts of Buffa's three serialization formats rather than treating them as equivalent front ends to one decoder.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 10.3 Textproto as a separate serialization capability

Buffa provides protobuf text-format encoding and decoding behind its `text` feature and generated `TextFormat` implementations.

Representative APIs include:

```
use buffa::text::{
    decode_from_str,
    encode_to_string,
    encode_to_string_pretty,
};

let text = encode_to_string_pretty(&message);
let decoded: MyMessage = decode_from_str(&text)?;
```

The format supports nested messages, repeated fields, maps, oneofs, groups, `Any`, and registered extensions.

It is particularly relevant to readable configuration artifacts, fixture files, diagnostics, and golden-file testing.

The generated parser rejects unrecognized field names by default, helping reveal misspelled fields. More specialized decoders can deliberately skip values through lower-level `TextDecoder` facilities.

The encoder also supports options such as printing unknown fields numerically for diagnostics.

Textproto is not a canonical interchange format: whitespace, float formatting, and presentation conventions can vary across implementations. Binary protobuf and ProtoJSON are better choices where standardized cross-runtime interchange is required.

Textproto support is independently enableable from JSON and remains compatible with `no_std + alloc`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



## 11. Custom storage representations and extension traits

One of Buffa's deeper architectural strengths is that the schema's field type and the Rust type chosen to store it are not always the same decision.

### 11.1 Pluggable scalar payload storage

`ProtoString` and `ProtoBytes` are traits for adapting other Rust-owned representations to protobuf fields.

Their contracts cover more than `AsRef`: generated code relies on operations such as constructing a value from a wire payload, borrowing it through `Deref`, and converting from conventional owned values.

This enables choices such as:

- `bytes::Bytes` for reference-counted binary payload storage.
- Small-string or compact-string representations for workloads with many short values.
- Crate-local newtypes wrapping specialized storage implementations.

Owned representations change; protobuf wire types and borrowed-view `&str`/`&[u8]` semantics remain the same.

`bytes::Bytes` is especially interesting because it can share backing storage with a `Bytes`-based input buffer. That benefit depends on the underlying decode input and representation: choosing `Bytes` does not imply every possible decode path is zero-copy.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 11.2 Custom collections

The customization extends beyond strings and bytes:

| Representation        | Built-in or customizable choices                                      | Primary tradeoff                                                  |
| --------------------- | --------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Repeated field        | `Vec<T>` or custom `ProtoList<T>`                                     | Allocation layout, small-vector optimization, collection behavior |
| Map field             | `HashMap`, `BTreeMap`, custom `MapStorage`                            | Lookup cost, iteration order, memory and allocation profile       |
| Singular message      | Inline or boxed `MessageField` storage; custom pointer representation | Message size, recursion requirements, allocation                  |
| Oneof message variant | Boxed by default; selectively inline nonrecursive values              | Allocation versus Rust enum size                                  |

Selecting `BTreeMap` can provide deterministic key iteration and may help when stable map traversal order matters. It should not be confused with a guarantee that an entire protobuf message serializes into canonical bytes.

Custom repeated and map containers may need separate integrations for reflection, serde, and arbitrary-data generation. The binary codec trait alone does not establish compatibility with those other surfaces.

Recursive type layout is also a hard constraint. A recursive message edge cannot simply be converted to an inline, infinitely sized Rust structure; Buffa's codegen analyzes recursion and retains indirection where needed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 11.3 `buffa-remote-derive`

The companion `buffa-remote-derive` crate reduces boilerplate for newtypes wrapping types from foreign crates.

It provides derives for integrations such as:

- `ProtoString`
- `ProtoBytes`
- `ProtoList`
- `ProtoBox`
- `MapStorage`

This addresses Rust's orphan rule: a foreign data type cannot directly implement a trait owned by Buffa unless the implementation is permitted by Rust's coherence rules.

The derive macros generate the binary-codec adapter behavior, including relevant conversions and dereferencing.

They do not automatically supply every trait needed by JSON, fuzzing, or reflection. Additional serde, `Arbitrary`, `ReflectList`, or `ReflectMap` implementations may still be necessary.

Source: [https://docs.rs/buffa-remote-derive/0.9.2/buffa_remote_derive/](https://docs.rs/buffa-remote-derive/0.9.2/buffa_remote_derive/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

### 11.4 Custom handwritten messages and external types

For cases that need a fundamentally different Rust representation, the core `Message` trait can be implemented manually.

A corresponding `extern_path` mapping allows other generated messages to refer to that custom implementation.

This is a stronger form of substitution than choosing custom field storage: the entire protobuf message implementation is replaced.

It preserves a route for specialized representations, but the burden increases because the custom implementation must supply the appropriate encoding, decoding, presence, view, and optional reflective behavior required by its consumers.

The general engineering opportunity is to reuse Buffa's serialization framework and type wiring without having to accept its default generated structs as the only possible runtime representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 12. Platform, ecosystem, and performance considerations

### 12.1 Feature and runtime boundaries

| Capability                 | Availability                                                          |
| -------------------------- | --------------------------------------------------------------------- |
| Binary codec               | Core `buffa` runtime                                                  |
| Borrowed views             | Generated by default; no separate full reflection requirement         |
| Lazy views                 | Opt-in code generation                                                |
| ProtoJSON                  | `buffa/json`, generated JSON implementations, serde                   |
| Textproto                  | `buffa/text`, generated text implementations                          |
| Runtime reflection         | `buffa-descriptor/reflect`, requires `std`                            |
| Well-known-type reflection | Additional `buffa-types/reflect`                                      |
| Fuzzing derives            | Generated `arbitrary` support and forwarding features                 |
| Embedded / `no_std`        | Core binary, borrowed views, text, and alloc-compatible JSON features |

The core runtime's `no_std` model requires `alloc`; it is not a zero-allocation, `core`-only protobuf implementation.

Without `std`, map storage uses alternative collections such as `hashbrown`, and some standard-library integrations are unavailable.

Generated source, runtime crate versions, feature flags, and externally mapped message implementations form a coherent compatibility unit. Version alignment is especially valuable when using multiple code-generation entry points and optional reflection traits.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 12.2 Performance characteristics

Buffa publishes benchmarks covering owned decode, eager borrowed views, lazy views, encoding, ProtoJSON, and reflection.

The published comparative dataset in the v0.9.2 repository was measured against Buffa v0.8.0, not v0.9.2. It used an Intel Xeon Platinum 8488C system with turbo disabled, a performance governor, one core per implementation, and five measurement passes. These are valuable observations of mechanisms, but should not be presented as new measurements of v0.9.2.

Two examples illustrate different regimes:

Binary decoding — LogRecord benchmark

Reported MiB/s, Buffa v0.8.0 dataset; higher is better

05001K1.5K2KProst ownedBuffa ownedBuffa eager viewBuffa lazy view

Measured on a Xeon Platinum 8488C; not a v0.9.2 measurement.

In this message shape, avoiding ownership of decoded strings and fields produces the clearest gain; lazy decoding avoids additional nested-message work.

Reflection read — LogRecord benchmark

Decode plus scan all set fields, MiB/s; higher is better

03507001K1.4KBridge reflectionFully dynamicVTable + view

Direct reflection over a generated borrowed view avoids the bridge's encode/decode round-trip and the dynamic representation's per-field materialization.

The key performance distinction is architectural rather than a universal speed ranking.

Generated owned decoding pays for owned strings, collections, and sub-message storage. Borrowed decoding avoids some or much of that cost, while lazy decoding can avoid visiting entire nested subtrees. Dynamic decoding pays for runtime schema dispatch and a generic value tree. Vtable reflection permits generic field inspection without converting the typed message to that tree.

The published results also show cases where an alternative implementation is faster, and where `bytes::Bytes` substantially improves prost on byte-heavy payloads. Performance comparisons should therefore match the actual message structure and operations rather than treating a single benchmark ratio as representative.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



### 12.3 Conformance and interoperability

The tagged upstream README reports passing the protobuf binary and JSON conformance suites against protobuf v33.5, including editions through 2024, in both `std` and `no_std` configurations. It also documents a tested matrix of `protoc` versions.

This is important evidence of protobuf compatibility, but it does not establish compatibility with arbitrary Rust frameworks built specifically around `prost::Message`.

Buffa uses its own traits and generated Rust layouts. Integrations expecting prost types are not automatically source-compatible just because both libraries produce wire-compatible protobuf messages.

Related distinctions include:

- `prost` and `prost-build` offer a well-established typed protobuf generation path; Buffa adds native borrowed views, richer built-in ProtoJSON support, and integrated dynamic reflection facilities.
- `tonic` supplies gRPC transport and service infrastructure. Buffa's message runtime and schema descriptors do not themselves establish drop-in compatibility with tonic's prost-bound codec and service-generation APIs.
- Buf supplies compilation, module management, registry services, linting, breaking-change detection, and remote plugin execution. These are not features implemented inside the Buffa runtime.
- Third-party `buffa-reflect` is a separate project from Anthropic's native `buffa-descriptor/reflect` functionality. Its similarly named concepts and version numbers should not be conflated with the APIs described here.

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+2

### 12.4 Important limitations and maturity considerations

Several boundaries are worth retaining in a technical reference:

| Boundary                         | Practical consequence                                                                     |
| -------------------------------- | ----------------------------------------------------------------------------------------- |
| Pre-1.0 release series           | Generated Rust API conventions and integration surfaces may change between releases       |
| External protobuf compiler       | Buffa does not independently parse or validate raw `.proto` source                        |
| Lazy views                       | Validation can be deferred; some reflection/text/handle surfaces are unavailable          |
| Dynamic messages                 | More flexible, but incur runtime dispatch and generic-value storage costs                 |
| Descriptor reflection            | Requires `std`; generated and externally mapped types need compatible reflection features |
| Custom collections               | Binary traits do not automatically provide JSON, reflection, or fuzzing integration       |
| Unknown-field preservation       | Supports semantic round-tripping, not original wire-byte identity                         |
| JSON resource controls           | Binary `DecodeOptions` are not a JSON memory-budget guarantee                             |
| External transport               | Protobuf wire compatibility does not imply ready-made gRPC framework integration          |
| Experimental `idiomatic_imports` | Generated naming and layout are explicitly subject to change                              |

`idiomatic_imports`, in particular, is explicitly marked experimental in v0.9.2 and requires `file_per_package`. It shortens generated type references through imports, but does not change the underlying message model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 13. Architectural synthesis

Buffa's capabilities can be understood as four complementary layers.

| Layer              | Core mechanism                                       | General architectural value                                                 |
| ------------------ | ---------------------------------------------------- | --------------------------------------------------------------------------- |
| Schema compilation | Buf/protoc → descriptors → `buffa-codegen`           | Separates schema semantics from Rust output generation                      |
| Typed execution    | Generated owned messages, `Message`, `ViewEncode`    | Efficient and strongly typed message processing                             |
| Borrowed execution | `MessageView`, `OwnedView`, lazy views               | Reduces materialization and copying when data is primarily read             |
| Dynamic execution  | `DescriptorPool`, `ReflectMessage`, `DynamicMessage` | Enables schema-driven systems without compiling every possible message type |

Their strongest interactions are not isolated optimizations.

A system can use statically generated types for ordinary execution while exposing descriptor-driven field access to generic instrumentation or tooling. It can pass `OwnedView` handles between asynchronous components while retaining borrowed field access. It can reserve `DynamicMessage` for schemas known only at runtime without forcing the entire application into a dynamic representation. It can centralize code generation around one descriptor corpus, reuse schema-wide analyses, and distribute generated packages across crates through `extern_path`.

Similarly, customizable field storage and collection types allow the Rust memory layout to be selected independently from the protobuf wire contract, while preserving the ability to fall back to standard generated representations when specialized storage is unnecessary.

The central technical value of Buffa is this combination: schema-accurate Protocol Buffers support, several execution representations, and a code-generation architecture that lets those representations coexist without maintaining separate protobuf implementations.

## 14. Primary-source reference map

These URLs point to the v0.9.2 release or its corresponding API documentation rather than relying on mutable `main` branch content.

| Subject                                | Source                                                                                                                                                                                    |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository and release                 | [https://github.com/anthropics/buffa/releases/tag/v0.9.2](https://github.com/anthropics/buffa/releases/tag/v0.9.2?utm_source=chatgpt.com)                                                 |
| Complete user guide                    | [https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md?utm_source=chatgpt.com)                                     |
| Architecture and design rationale      | [https://github.com/anthropics/buffa/blob/v0.9.2/DESIGN.md](https://github.com/anthropics/buffa/blob/v0.9.2/DESIGN.md?utm_source=chatgpt.com)                                             |
| Runtime API                            | [https://docs.rs/buffa/0.9.2/buffa/](https://docs.rs/buffa/0.9.2/buffa/?utm_source=chatgpt.com)                                                                                           |
| `buffa-build::Config`                  | [https://docs.rs/buffa-build/0.9.2/buffa_build/struct.Config.html](https://docs.rs/buffa-build/0.9.2/buffa_build/struct.Config.html?utm_source=chatgpt.com)                               |
| `buffa-codegen` API                    | [https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/?utm_source=chatgpt.com)                                                           |
| `CodeGenConfig`                        | [https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/struct.CodeGenConfig.html](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/struct.CodeGenConfig.html?utm_source=chatgpt.com)         |
| Well-known types                       | [https://docs.rs/buffa-types/0.9.2/buffa_types/](https://docs.rs/buffa-types/0.9.2/buffa_types/?utm_source=chatgpt.com)                                                                   |
| Descriptor types and reflection        | [https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/?utm_source=chatgpt.com)                                               |
| Companion derive macros                | [https://docs.rs/buffa-remote-derive/0.9.2/buffa_remote_derive/](https://docs.rs/buffa-remote-derive/0.9.2/buffa_remote_derive/?utm_source=chatgpt.com)                                   |
| Published Buf plugin                   | [https://buf.build/anthropics/buffa](https://buf.build/anthropics/buffa?utm_source=chatgpt.com)                                                                                           |
| Prost migration reference              | [https://github.com/anthropics/buffa/blob/v0.9.2/docs/migration-from-prost.md](https://github.com/anthropics/buffa/blob/v0.9.2/docs/migration-from-prost.md?utm_source=chatgpt.com)       |
| Google protobuf migration reference    | [https://github.com/anthropics/buffa/blob/v0.9.2/docs/migration-from-protobuf.md](https://github.com/anthropics/buffa/blob/v0.9.2/docs/migration-from-protobuf.md?utm_source=chatgpt.com) |
| Benchmarks and measurement methodology | [https://github.com/anthropics/buffa/tree/v0.9.2/benchmarks](https://github.com/anthropics/buffa/tree/v0.9.2/benchmarks?utm_source=chatgpt.com)                                           |

The most useful deeper investigations are the generated representation rules in `CodeGenConfig`, the borrowed-view lifetime model, the reflection bridge/vtable implementations, the typed-extension registry, and the shared descriptor-pool mechanism. Together, these explain most of the consequential distinctions between Buffa and a conventional generated-protobuf serialization library.

# Buffa 0.9.2 — Capability Reference Addendum

Research date: November 3, 2026\
Target release: Buffa 0.9.2, September 3, 2026\
Companion document: Buffa 0.9.2 — Comprehensive Rust Library and Toolchain Overview

This addendum documents additional runtime, reflection, code-generation, interoperability, and diagnostic capabilities not developed in the main reference. It also clarifies consequential implementation constraints and version boundaries. All capabilities described as available refer to the tagged v0.9.2 release unless explicitly qualified.

## 1. Advanced serialization: bounded output, working-memory reuse, and segmented buffers

Extends original §§2.4–2.6 and 3.4.

### 1.1 Atomic, budget-constrained encoding

Buffa provides output-budget enforcement independently of its protobuf-wide maximum message size.

The `Message` and `ViewEncode` traits expose:

| API                                                     | Functionality                                                         |
| ------------------------------------------------------- | --------------------------------------------------------------------- |
| `try_encode_bounded(max_bytes, sink)`                   | Encodes only when the complete message fits the specified byte budget |
| `try_encode_bounded_with_cache(max_bytes, cache, sink)` | Same operation using a reusable `SizeCache`                           |
| `try_encoded_len()`                                     | Computes encoded size without panicking on the protobuf size limit    |
| `try_encode_with_cache(...)`                            | Fallible serialization with caller-managed size-cache storage         |
| `SizeCachePool::try_encode_bounded(...)`                | Budget-constrained encoding with pooled cache storage                 |
| `SizeCachePool::try_encode_view_bounded(...)`           | Equivalent operation for borrowed message views                       |

The important guarantee is failure atomicity with respect to the sink. The encoder completes its size computation and verifies the output budget before starting the write pass.

Consequently, an encoding operation that fails its size check does not leave a partially encoded message in an existing output buffer.

Two errors have distinct meanings:

- `EncodeError::MessageTooLarge`: the encoded message exceeds the protobuf limit of \\(2^{31}-1\\) bytes.
- `EncodeError::ExceedsBudget`: the message satisfies the protobuf limit but exceeds the caller's lower budget.

The protobuf-wide limit takes precedence when both would be exceeded. Successful bounded encoding returns the encoded body length as `u32`, excluding any framing prefix added by another layer.

This is useful for bounded network frames, message queues with maximum payload sizes, storage records, and interfaces that must reject oversized messages without corrupting a reusable output sink.

It also removes the need for a separate application-managed temporary serialization buffer solely to establish that the output fits.

Important distinction: `DecodeOptions::with_max_message_size` governs incoming data. Bounded encoding governs output production. Neither replaces the other.

Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs`

### 1.2 `SizeCachePool`: amortizing serialization working-memory allocation

The main guide explains Buffa's external two-pass `SizeCache`. The complementary `SizeCachePool` addresses repeated serialization workloads in which constructing a fresh cache causes unnecessary heap allocation.

A `SizeCache` stores its first 16 nested-message size slots inline. Only messages whose size-traversal requirements exceed this capacity allocate a spill buffer.

A `SizeCachePool` retains and reuses those spill buffers across encoding operations.

Its principal operations are:

| API                                                  | Behavior                                                                |
| ---------------------------------------------------- | ----------------------------------------------------------------------- |
| `SizeCachePool::new(max_buffers, max_capacity)`      | Creates a bounded pool of reusable spill buffers                        |
| `SizeCachePool::sequential(max_capacity)`            | Specialized pool retaining at most one buffer                           |
| `acquire()` / `release()`                            | Explicitly check out and return a `SizeCache`                           |
| `encode()` / `encode_view()`                         | Encode owned messages or views with automatic cache checkout and return |
| `encoded_len()` / `try_encoded_len()`                | Compute lengths using reusable working memory                           |
| `try_encode()` / `try_encode_view()`                 | Fallible pooled serialization                                           |
| `try_encode_bounded()` / `try_encode_view_bounded()` | Combine allocation reuse with output-budget enforcement                 |

Two independent bounds control retention:

- `max_buffers` limits how many spill buffers the pool holds.
- `max_capacity` limits each retained buffer's capacity, measured in `u32` size-cache slots.

An oversized spill buffer is shrunk when released rather than being retained indefinitely at its peak capacity. Conversely, setting the capacity too low can cause every encode to regrow the buffer and every release to shrink it, negating the benefit.

The pool does not introduce global synchronization or hidden serialization state. It is caller-owned, works with `alloc` without requiring `std`, and can be scoped to a worker, thread, connection, or request.

When pooling matters: repeated encoding of deeply nested messages, large repeated-submessage collections, or message structures with many length-delimited descendants.

When it offers little benefit: messages requiring no spill allocation, or workloads where a single manually reused `SizeCache` already provides sufficient reuse. Pooling does not reduce the work of walking the message itself; it amortizes allocation of the size-cache spill buffer.

Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/size_cache.rs`

### 1.3 `Rope::with_backing`: preserving borrowed wire-buffer segments

The existing guide describes `Rope` as a segmented encoding sink. An additional mechanism connects Buffa's borrowed-view representation to genuinely buffer-sharing output.

`Rope::with_backing(bytes::Bytes)` associates a reference-counted input buffer with the output rope.

When encoding a borrowed slice, the rope checks whether the slice lies within that original backing buffer. If it does, and the slice meets the segmentation threshold, the rope can retain the slice through `Bytes::slice_ref` rather than copying its contents.

The available controls include:

- `Rope::with_min_segment(size)`: sets the minimum slice size eligible for segmentation; the default is 4 KiB.
- `Rope::with_backing(bytes)`: establishes the source buffer eligible for zero-copy slice capture.
- `Rope::into_segments()`: produces ordered `Bytes` segments.
- `Rope::segment_count()` and `len()`: expose the output's physical segmentation and logical length.
- `Rope::to_contiguous_bytes()`: explicitly flattens the result, copying when necessary.

This mechanism is different from using a `Bytes`-backed owned protobuf field. A borrowed message view normally contains `&str` and `&[u8]` references. Those slices do not independently own the underlying allocation.

The backing-aware rope gives the encoder a way to recover safe reference-counted ownership of eligible portions of the original input buffer.

Borrowed-buffer forwarding path

Incoming `bytes::Bytes`

Reference-counted wire buffer

Generated `MessageView`

Borrowed field inspection

`ViewEncode` → `Rope::with_backing`

Re-encode with eligible payload slices retained

`Vec<Bytes>` or `RopeBuf`

Segmented network or I/O output

The companion `RopeBuf` implements `bytes::Buf` over a completed rope. Its `From<Rope>` implementation transfers the segments into the buffer, and its `chunks_vectored` implementation exposes multiple outstanding segments as `IoSlice` values when `std` is enabled.

This can simplify integration with consumers accepting chunked buffers, vectored I/O, or multiple transport frames.

The zero-copy benefit has several limits:

- The backing buffer must be the allocation from which the borrowed view was decoded. Unrelated or modified slices are copied normally.
- Small fields and encoded field headers still involve writes into the rope's contiguous tail.
- The wire representation is regenerated, so original field ordering and byte-for-byte identity are not guaranteed.
- The receiving transport must preserve segmentation for the avoided copies to remain avoided end to end.
- Segments can retain the original backing allocation, increasing its lifetime and potentially retaining more memory than the visible slice occupies.

Thus Buffa can avoid large payload copies during inspection-and-forwarding, but this is not automatically a zero-copy implementation of the complete network stack.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs`

### 1.4 Reader-oriented decoding and frame-boundary enforcement

With `std` enabled, `DecodeOptions` also accepts `std::io::Read` sources:

- `decode_reader<M>(&mut reader)` reads through EOF before decoding a complete message.
- `decode_length_delimited_reader<M>(&mut reader)` reads a varint size prefix and exactly one framed message.

These APIs are separate from `bytes::Buf` decoding and establish a direct integration with ordinary Rust readers.

For length-delimited streams, the reader checks the claimed frame length before reading its body. It also avoids immediately allocating the full declared size: initial capacity is capped at 64 KiB and grows as bytes actually arrive.

This prevents an input containing a very large declared length but no corresponding payload from immediately forcing an equivalently large allocation.

`without_reader_size_limit()` relaxes the read-until-EOF reader limit; it does not remove the protobuf-wide message-size limit from length-delimited record decoding.

Both reader APIs buffer the serialized input before performing ordinary message decoding. They are therefore convenient framing and I/O adapters, not streaming field-by-field parsers with constant memory requirements.

Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs`

## 2. Descriptor pools, schema discovery, and dynamic message processing

Extends original §7.

### 2.1 Transactional descriptor-pool expansion

`DescriptorPool` supports incremental addition of compiled protobuf schemas through:

`DescriptorPool::add_file_descriptor_set(&mut self, set)`

This is distinct from constructing an entirely new pool through `DescriptorPool::decode`.

The method registers additional message, enum, service, and extension declarations and links them to previously registered definitions. It supports multiple descriptor sets contributing to one coherent runtime schema.

Its key guarantees and tradeoffs are:

- Transactional mutation: new descriptors are staged against a clone of the existing pool. A linking or validation failure leaves the original pool unchanged.
- Idempotent file registration: files already registered under the same filename are skipped rather than re-added.
- Stable existing indices: successful additions append new entries without renumbering existing message and enum descriptors.
- Batching advantage: each addition containing new files deep-clones the existing pool, so loading many files together is more efficient than repeated single-file additions.

This is useful for assembling descriptor corpora from independent schema modules or incrementally discovered definitions.

It does not, by itself, implement live schema replacement. Existing filenames are skipped, and a pool already shared through `Arc` requires an appropriate ownership or replacement strategy before it can be mutated.

The distinction between pool expansion and schema evolution is significant: adding new definitions is supported, but replacing existing definitions while preserving the identity of all previously issued descriptors is not implied.

### 2.2 Descriptor-pool construction limits and validation

Beyond ordinary binary message limits, `DescriptorPool` validates a descriptor corpus's internal structure.

Relevant checks include duplicate symbols, invalid or reserved field numbers, conflicting field declarations, reserved enum values, invalid oneof references, malformed map entries, unresolved type references, and invalid extension ranges.

Two implementation limits deserve explicit mention:

| Constraint                            | v0.9.2 behavior                                   |
| ------------------------------------- | ------------------------------------------------- |
| Maximum fully qualified symbol length | 512 bytes (`MAX_SYMBOL_LEN`)                      |
| Maximum fields per message            | 65,535, reflecting the pool's `u16` field indices |

These limits bound aspects of linked-schema materialization that are not necessarily controlled by the incoming descriptor-set byte count.

`DescriptorPool::decode_with_options(bytes, &DecodeOptions)` allows caller-specified decode limits. This is especially useful for large trusted descriptor sets whose materialized descriptor objects exceed the ordinary 32 MiB element-memory budget.

The tagged implementation makes an important distinction between trusted and untrusted descriptor input: a compiler-produced descriptor corpus may legitimately need a larger budget, whereas externally supplied descriptor bytes should retain appropriately restrictive limits.

A successfully linked pool is not equivalent to a complete `.proto` compiler pass. Source parsing and the full compiler validation process remain responsibilities of `protoc` or Buf.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/pool.rs`

### 2.3 Runtime schema discovery for reflection services

`DescriptorPool` retains the original `FileDescriptorProto` objects in addition to its linked descriptors.

It provides several lookup operations that can eliminate custom schema-indexing infrastructure:

| API                                     | Capability                                                                                       |
| --------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `file_by_name(name)`                    | Retrieve the original descriptor for a named `.proto` file                                       |
| `file_containing_symbol(full_name)`     | Resolve the source file declaring a message, nested message, enum, service, method, or extension |
| `service_by_name(full_name)`            | Resolve a service declaration                                                                    |
| `extension_by_name(full_name)`          | Resolve a registered extension declaration                                                       |
| `extension_for(extendee, number)`       | Find an extension by the extendee message and field number                                       |
| `extensions_of(extendee)`               | Enumerate an extendee's registered extensions in field-number order                              |
| `messages()` / `enums()` / `services()` | Enumerate linked schema definitions                                                              |

The symbol-to-file lookup is particularly relevant to the standard gRPC server-reflection operation `FindFileContainingSymbol`.

`file_containing_symbol` includes methods and extensions, not merely top-level message declarations.

Combined with retained raw file descriptors, these operations supply much of the schema-indexing machinery needed by server reflection, generic protocol explorers, API discovery services, and tooling that distributes descriptor definitions to other processes.

Buffa does not itself become a gRPC server-reflection transport through these methods. The RPC service implementation and transport remain separate concerns.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 2.4 Direct reflective mutation

`DynamicMessage` is not limited to reflection through immutable field access.

It exposes mutable accessors:

- `field_by_number_mut(number) -> Option<&mut Value>`
- `field_mut(&FieldDescriptor) -> Option<&mut Value>`

These permit mutation of an already-present field's value without converting the entire message to a different representation.

A nested message can be visited recursively through `Value::Message`; repeated and map fields can likewise be traversed through their dynamic collection representations.

The important distinction is between mutating an existing value and assigning a new field value.

`field_mut` returns `None` for an absent field. To insert or replace fields using descriptor-aware validation, the `ReflectMessageMut` interface exposes `set` and `try_set`.

`DynamicMessage` implements checked mutation behavior that can detect foreign field descriptors and values whose runtime shape does not match the declared protobuf field kind.

By contrast, directly replacing a value through an `&mut Value` returned by `field_mut` bypasses this validation. The encoder defensively skips certain invalid stored values, so direct mutation should not be treated as guaranteeing that all subsequent serialization operations retain the intended information.

Generated reflection is asymmetric in v0.9.2: `Reflectable::reflect()` exposes a read-only reflective handle for generated messages. A general generated `reflect_mut()` entry point is not released. A caller needing generic reflective mutation of generated data can materialize a `DynamicMessage`, modify it, and convert it back, with the associated conversion costs.

Tagged sources:

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs`

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/message.rs`

### 2.5 Dynamic `Any`, custom options, and generated-type bridging

`DynamicMessage` provides specialized operations that work through descriptor identity rather than compile-time Rust types.

| API                             | Capability                                                                                               |
| ------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `pack_any()`                    | Wrap a dynamic message in a descriptor-backed `google.protobuf.Any`                                      |
| `unpack_any()`                  | Resolve an `Any` type URL against the pool and decode its payload                                        |
| `from_options(pool, options)`   | Convert a generated protobuf options message into a reflective dynamic representation                    |
| `try_from_message()`            | Convert a statically generated message into a dynamic message, resolving its descriptor by `MessageName` |
| `try_from_message_with_index()` | Equivalent conversion with an explicitly supplied message index                                          |
| `to_message<M>()`               | Convert dynamic data into a generated Rust message through protobuf encoding and decoding                |

`pack_any()` requires the pool to contain the `google.protobuf.Any` descriptor. `unpack_any()` requires the pool to contain the embedded message type.

Custom options are a particularly important capability interaction.

A descriptor's `options()` accessor returns a generated options message. Custom protobuf options may be preserved there as extension wire data that is not directly accessible as a known generated field.

`DynamicMessage::from_options` reinterprets those options against the descriptor pool, allowing registered custom options to be resolved and read by extension descriptor.

This supports descriptor-driven processors that inspect externally defined schema annotations without requiring separately generated Rust accessors for every custom option.

Pool identity and compatibility matter. Reflective field descriptors belong to their originating pool, and a structurally similar field from another pool is not automatically the same descriptor object.

When dynamic values cross pool boundaries, Buffa can adopt compatible nested messages by re-encoding and decoding them into the destination pool. This preserves the ability to compose certain cross-crate reflective values, but the conversion is not free.

A runtime schema engine using these capabilities can perform inspection, transformation, annotation interpretation, and typed conversion without maintaining a second protobuf descriptor representation.

Source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs`

## 3. ProtoJSON policy, format-specific limits, and diagnostic safety

Extends original §§9–10 and §12.

### 3.1 `JsonParseOptions`: unknown enum handling

`JsonParseOptions` provides a v0.9.2 parsing policy not developed in the main guide:

`ignore_unknown_enum_values(bool)`

The setting controls what happens when JSON contains an enum name that the generated schema does not recognize.

With the option disabled, the unknown value is a parse error. With it enabled, the `std` implementation provides the following behavior:

| Location       | Unknown enum name                               |
| -------------- | ----------------------------------------------- |
| Singular field | Replaced with the default enum value            |
| Repeated field | Unknown element skipped                         |
| Map value      | Entry containing the unknown enum value skipped |

The `no_std` implementation has a narrower capability: unknown singular enum values can be replaced by the default, but repeated and map-container filtering is unavailable, so those unknown entries remain errors.

This setting addresses a different problem from `strict_extension_keys(true)`, which governs bracketed extension names such as `"[pkg.option]"`.

It is also distinct from binary protobuf's unknown open-enum handling, which preserves unrecognized numeric values rather than interpreting unknown JSON strings.

An unknown enum string and an unknown numeric enum value are not interchangeable compatibility cases.

Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/json.rs`

### 3.2 Generated versus dynamic JSON parsing strictness

There is a consequential asymmetry between generated-message JSON parsing and the reflective dynamic decoder.

| JSON parsing path                                 | Treatment of ordinary unknown fields                              |
| ------------------------------------------------- | ----------------------------------------------------------------- |
| Generated message Serde deserialization           | Ignores unknown ordinary fields by default                        |
| `DynamicMessage::from_json`                       | Rejects unknown fields by default                                 |
| `DynamicMessage::from_json_ignoring_unknown`      | Discards unknown fields                                           |
| `DynamicMessageSeed::ignore_unknown_fields(true)` | Configures unknown-field tolerance for seed-based deserialization |

The lenient dynamic behavior propagates to nested messages, repeated elements, and map values.

It does not relax every protobuf JSON constraint. Duplicate keys, conflicting oneof members, and malformed values on known fields remain errors.

`DynamicMessageSeed` is the lower-level interface for integrating descriptor-aware deserialization with arbitrary compatible Serde deserializers. It carries the descriptor pool and message index into the parsing operation.

Unlike ordinary `Deserialize`, this requires schema context supplied externally to the Rust type being deserialized.

This distinction is useful in generic JSON-processing systems where the expected protobuf message type is selected at runtime.

### 3.3 Resource limits differ by format and direction

The main guide correctly distinguishes binary decode limits from JSON parsing limits. Several additional qualifications are important.

Generated ProtoJSON parsing is not bounded by `DecodeOptions`. In particular, the binary element-memory budget does not automatically apply when Serde materializes repeated messages or maps from JSON.

Reflective JSON parsing in v0.9.2 also does not expose an equivalent configurable element-memory budget through `DynamicMessageSeed`. Its tagged configuration consists of descriptor context and unknown-field handling. Resource-budget APIs described in newer development documentation must not be attributed to this release.

Reflective JSON serialization does enforce a nesting limit. `DynamicMessage` uses `buffa::RECURSION_LIMIT`, including nested messages discovered by expanding `google.protobuf.Any` payloads.

This distinction matters because an `Any` message can appear shallow at the wire level while containing another serialized `Any`, which in turn contains another.

Ordinary binary message-depth validation cannot detect the full depth of an opaque nested-`Any` chain.

Buffa's generated `Any` expansion has a separate limit:

`buffa::type_registry::MAX_ANY_EXPANSION_DEPTH`

In v0.9.2 this is 100. The response to exceeding it differs by format:

- ProtoJSON serialization returns an error.
- Textproto falls back to an unexpanded `type_url`/`value` representation.

The limit is independent of a caller's `DecodeOptions::with_recursion_limit`.

Textproto materialization has its own element-memory control. `decode_from_str_with_element_memory_limit` permits adjustment independently of the binary decoder.

Together, these facilities provide different contracts for binary input, JSON input, text input, and schema-aware serialization. They do not constitute a single interchangeable resource-governance system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com+1



Tagged sources:

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/json.rs`

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/type_registry.rs`

### 3.4 Standard protobuf `debug_redact` support

Buffa's generated `Debug` implementations recognize the standard protobuf field option:

```
string credential = 1 [debug_redact = true];
```

Such fields print an unquoted `[REDACTED]` marker rather than the contained value.

The behavior applies across generated owned messages, generated views, oneof representations, and reflective `DynamicMessage` debugging.

A generated type containing a redacted field uses a specialized `Debug` implementation rather than an ordinary Rust derive, so it can avoid printing the redacted field's value.

This is diagnostic redaction, not a serialization or authorization mechanism.

Binary protobuf, ProtoJSON, and textproto serialization retain the field's actual value. Application logging that serializes messages through one of those formats does not automatically inherit `Debug` redaction.

Similarly, copying a field into an unrelated object and debugging that object is outside Buffa's field-option guarantee.

The feature reduces the need for manually implemented `Debug` across protobuf-generated types, but it does not replace controls governing which data an application may log, persist, or transmit.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md`

## 4. Protobuf semantic helpers, extensions, and typed identity

Extends original §§2.1, 6, 8, and 9.

### 4.1 `MessageName`: protobuf identity independent of wire encoding

Generated message types implement `buffa::MessageName`, exposing four associated constants:

| Constant    | Meaning                                                        |
| ----------- | -------------------------------------------------------------- |
| `PACKAGE`   | Protobuf package name                                          |
| `NAME`      | Message name relative to the package, including nesting levels |
| `FULL_NAME` | Fully qualified protobuf message name                          |
| `TYPE_URL`  | Conventional `type.googleapis.com/...` type URL                |

`MessageName` is a standalone trait rather than a supertrait of `Message`.

Consequently, a handwritten external Rust type can implement protobuf identity without necessarily implementing Buffa's binary codec.

This separation is useful for type registries, schema routing, event envelopes, and metadata systems that need protobuf type names independently of serialization.

It also prevents conflating Rust module paths with protobuf type identity: code-generation naming options or `extern_path` mappings may change Rust paths without changing `FULL_NAME`.

### 4.2 `Any` typed access and URL matching

The tagged v0.9.2 `buffa-types::Any` helper implementation provides:

| API                               | Behavior                                                        |
| --------------------------------- | --------------------------------------------------------------- |
| `Any::pack(message, type_url)`    | Encodes a message with an explicitly supplied type URL          |
| `Any::try_pack(...)`              | Fallible alternative for oversized messages                     |
| `Any::unpack_if<T>(expected_url)` | Decodes only if the stored URL exactly matches the supplied URL |
| `Any::unpack_unchecked<T>()`      | Attempts decoding without validating type identity              |
| `Any::is_type(url)`               | Checks exact URL equality                                       |
| `Any::type_url()`                 | Returns the stored type URL                                     |

The distinction between URL checking and message decoding is important.

`unpack_unchecked<T>()` verifies the payload's wire decodability as `T`, but does not verify that the envelope identifies `T`.

Protobuf wire compatibility can allow different message definitions to decode the same bytes. Successful decoding is therefore not evidence of matching declared type identity.

Conversely, `unpack_if<T>()` performs an exact URL comparison. A semantically equivalent type URL using a different prefix will not match merely because its final protobuf type name is identical.

The release's helper API should not be confused with newer development-branch conveniences such as `pack_message` and `unpack_message`, which are not present in the tagged `any_ext.rs` implementation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-types/src/any_ext.rs`

### 4.3 Well-known dynamic-value construction

The `Value`, `Struct`, and `ListValue` implementations in `buffa-types` provide direct helpers for constructing and examining protobuf's JSON-like value model.

Examples of their API families include:

- `Value::null`, `is_null`, and typed `as_*` accessors.
- `Value::from` conversions for supported primitive Rust values.
- `Struct::new`, `from_fields`, `insert`, and `get`.
- `ListValue::from_values`, `iter`, `len`, and `is_empty`.

These helpers permit the assembly of protobuf `Struct` and `Value` messages without manually constructing the generated oneof variants.

The benefit is distinct from `DynamicMessage`: `Struct` and `Value` are concrete protobuf-defined types representing JSON-like data. They do not preserve arbitrary externally supplied protobuf field identities and scalar types in the way descriptor-backed `DynamicMessage` does.

`FieldMask` likewise provides `from_paths`, `contains`, `iter`, `len`, and `is_empty`.

In v0.9.2, `FieldMask::contains` performs exact case-sensitive path membership, not hierarchical path coverage. A stored path such as `user` does not make `contains("user.name")` true.

Methods such as `covers`, `overlaps`, `normalize`, `union`, and `intersection` appearing in newer source material should not be attributed to the tagged v0.9.2 implementation.

Tagged sources:

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-types/src/value_ext.rs`

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-types/src/field_mask_ext.rs`

### 4.4 Default-aware and fallible extension operations

`ExtensionSet` includes two additional behaviors beyond ordinary get/set/clear.

Default-aware retrieval: `extension_or_default(&extension)` returns the declared proto2 default if a singular extension is absent, or the Rust type's default if no protobuf default was declared.

This does not make an absent field present. `extension()` continues to distinguish explicit presence.

Fallible writes: `try_set_extension(&extension, value)` supports encoding a replacement extension value without panicking merely because its serialized message exceeds the protobuf size limit.

The implementation stages the encoded extension in temporary unknown-field storage and only updates the carrier after encoding succeeds. An encoding failure leaves earlier occurrences intact.

There are further semantic distinctions:

- `has_extension` checks field-number presence, not whether the retained bytes form a valid value for the extension's codec.
- `extension()` may return an absent-style result for malformed singular extension data or exhausted decoding budgets, rather than exposing a dedicated parsing error.
- The extension decoder shares its decode budget across that extension's retained wire records.
- Using an extension whose declared extendee does not match the message remains a programming error and can panic, even with `try_set_extension`.

These behaviors matter for schema-aware tooling because presence, a successful value decode, and a declared default are three distinct states.

Source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/extension.rs`

### 4.5 Proto2 defaults and enum iteration

Two smaller generated-model capabilities complete the original guide's discussion of protobuf semantics.

Proto2 scalar `[default = ...]` declarations influence the generated message's Rust `Default` implementation. Required scalar fields use bare Rust types and are serialized even when they hold the default value, rather than being suppressed as implicit-presence fields.

A generated Rust field's current value should not be mistaken for evidence of its original wire presence; this is particularly relevant when interpreting legacy required-field semantics.

For generated enums, the `Enumeration::values()` method returns a static slice of primary variants in protobuf declaration order.

Aliases represented by associated constants are not additional Rust enum variants and do not appear in this slice.

This allows generic enum enumeration without depending on debugging output, string parsing, or an independently maintained list of variants.

Source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md`

## 5. Code-generation extensibility and distribution

Extends original §§4–5.

### 5.1 Companion generated files through `apply_companions`

An easily overlooked code-generation extension point is:

`buffa_codegen::apply_companions`

The `GeneratedFileKind` enum includes a `Companion` variant intended for source files produced by another generator but distributed alongside Buffa-generated message code.

A custom generator can create `GeneratedFile` entries with:

- `kind = GeneratedFileKind::Companion`
- a target protobuf package
- a sibling output filename
- the additional Rust source content

`apply_companions` finds the corresponding package's `PackageMod` file and appends an `include!` directive for the companion at package scope.

This allows another code generator to add service stubs, implementation traits, dispatch adapters, or other schema-derived Rust definitions without rewriting Buffa's generated source or maintaining parallel module-tree logic.

The integration contract has several constraints:

- The companion's package must match a generated `PackageMod` for automatic inclusion.
- Companion filenames must follow the sibling-file convention and avoid collisions with Buffa's own generated filenames.
- The operation is not idempotent: applying the same companion twice emits duplicate includes.
- A companion whose package has no generated module is retained in the file collection but is not automatically included.
- Companion items appear at the generated package root, not in Buffa's reserved `__buffa` module.

This is particularly useful for generating additional service interfaces from the same descriptor corpus while leaving the message runtime and protobuf compiler backend unchanged.

It is not itself a complete gRPC or ConnectRPC service generator. It is the composition mechanism through which such a generator can participate in Buffa's output layout.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs`

### 5.2 Code-generation diagnostics as structured output

`generate_with_diagnostics()` provides a lower-level alternative to ordinary code generation, returning both generated artifacts and code-generation warnings.

This allows build infrastructure to inspect warnings programmatically rather than depending exclusively on textual compiler output.

Examples of relevant diagnostics include unmatched feature-override paths and references to types that were excluded from generation without adequate external mappings.

The distinction is useful when code generation is embedded in larger schema-processing tools: emitted source and generation diagnostics can participate in separate workflows.

`SharedCorpusContext`, described in the original guide, complements this interface by avoiding repeated schema-global analysis when multiple generation operations share the same descriptor corpus.

### 5.3 Build-tool descriptor decoding budgets

The protobuf compiler plugin and the Cargo build integration use a different element-memory budget from ordinary runtime decoding.

The default code-generation budget in v0.9.2 is 1 GiB, intended to accommodate wide descriptor structures and large schemas while still rejecting excessively expensive or corrupted inputs.

The plugin accepts:

`element_memory_limit=<bytes|unlimited>`

The shared environment variable is:

`BUFFA_ELEMENT_MEMORY_LIMIT`

The environment variable is the configuration route for `buffa-build`; plugin options take precedence when both are supplied.

The compiler plugin must obtain its parameter string from the incoming `CodeGeneratorRequest` before it can determine the desired budget. Buffa handles this by scanning the request wire data for the parameter field before performing the full descriptor materialization.

This avoids the need to perform an unbounded preliminary decode merely to discover the decode limit.

The distinction between the 1 GiB tooling budget and the 32 MiB default runtime element-memory budget should remain explicit. They protect different workloads and are not interchangeable settings.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs`

### 5.4 Buf SDK distribution and packaging constraints

Buffa supports another distribution model besides local `build.rs` execution or checking generated Rust source into a repository.

For compatible Buf Schema Registry modules, consumers can use pre-generated Rust SDK crates from Buf's Cargo registry.

This delegates protobuf generation and packaging to the schema-publishing infrastructure. The consuming Rust project uses generated crate dependencies instead of invoking Buffa's generator during its own build.

It is an external Buf service capability, not an independent feature of the Buffa runtime.

The local packaging plugin also has specific constraints. When using `protoc-gen-buffa-packaging` to assemble a module tree from Buf-generated source:

- The packaging plugin requires `strategy: all`.
- It expects the per-proto source/stitcher layout, not `file_per_package=true`.
- Package exclusions must be coordinated with the main generator.
- Shared descriptor-pool generation requires matching options and matching input corpora across both plugins.

These are compatibility requirements for composing generator outputs, rather than protobuf wire-format constraints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md`

### 5.5 External shared descriptor-pool roots

At the lower-level `CodeGenConfig` interface, `shared_descriptor_pool_root` permits an explicitly specified Rust path to the shared descriptor-pool root.

This removes the assumption that the root must be reached through the module-tree generator's conventional relative `super::` path.

The configured path must be absolute (`::...`) or crate-relative (`crate::...`), because it is reused from generated packages at potentially different module depths.

It is valid only when shared descriptor-pool generation is enabled.

This extension point matters for custom code-generation frameworks that coordinate several separately emitted Rust modules and choose their own enclosing module hierarchy.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Tagged source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs`

## 6. Custom storage: field-position-dependent integration constraints

Extends original §11.

Buffa's pluggable storage traits permit specialized Rust representations, but the required integration surface depends on where a custom type appears in the protobuf schema.

### 6.1 Custom representations inside maps

A `string_type` rule applies to protobuf string positions within its targeted field, including map keys and values.

For a `map<string, string>` field, one field-scoped representation rule affects both the key and value string slots.

The map container itself remains independently configurable through `map_type`.

Custom map keys must also satisfy the container's ordering or hashing constraints:

| Container           | Relevant key requirements                        |
| ------------------- | ------------------------------------------------ |
| `HashMap`           | `Hash + Eq`                                      |
| `BTreeMap`          | `Ord`                                            |
| Custom `MapStorage` | The storage implementation's required key traits |

A custom string used in a map may additionally require Serde and `Arbitrary` implementations when the corresponding generated capabilities are enabled.

Unlike singular string fields, several optional, repeated, oneof, and map configurations serialize custom values through their own Serde implementations.

### 6.2 Rust coherence and generated auxiliary traits

Under vtable reflection, custom repeated elements and map key/value types may need `ReflectElement` or `ReflectMapKey` implementations.

For certain custom byte representations, ProtoJSON also introduces element-specific adapter requirements.

Buffa's code generator can emit these implementations for appropriate crate-local types. It cannot override Rust's orphan rules by implementing a foreign trait for an unrelated foreign type.

A local newtype therefore does more than adapt binary storage: it provides a legal target for generated cross-cutting trait implementations.

This explains why an external storage type that works as a singular field may fail to compile when used as a repeated element or map key with JSON or reflection enabled.

The primary design distinction is not simply `String` versus a compact alternative. It is which generated functionality must operate on that representation and which crate owns the relevant Rust type.

Source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md`

## 7. Version, feature, and publication boundaries

Clarifies original §§1, 5, 7, and 12.

### 7.1 Descriptor reflection is not uniformly `std`-dependent

The tagged `buffa-descriptor` crate separates the descriptor runtime's `reflect` feature from its `std` feature.

Its feature declaration is:

```
reflect = []
```

The lower-level descriptor and dynamic reflection code uses `alloc` facilities in several places, rather than requiring `std` throughout.

By contrast, generated reflection in v0.9.2 requires `std` because the generated embedded descriptor-pool accessor uses `std::sync::OnceLock`.

`buffa-types/reflect` also enables `std` and the descriptor reflection dependency.

There is therefore a real distinction among:

- Descriptor pool and dynamic reflection support.
- Generated `Reflectable` implementations.
- Well-known-type reflection integrations.
- Convenience APIs explicitly gated by `std`, such as dynamic `from_json` and `to_json`.

The tagged source and feature declarations establish these separate boundaries. They should not be collapsed into a blanket statement that all descriptor reflection requires `std`. This review did not independently compile every `no_std + alloc` feature combination.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



Source:\
`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/Cargo.toml`

### 7.2 Descriptor message types require feature forwarding

`buffa-descriptor` independently gates generated implementations for:

`views`, `json`, `text`, and `arbitrary`.

A protobuf schema importing `descriptor.proto` only to declare custom options does not necessarily require `buffa-descriptor` as a runtime message dependency.

However, when generated messages contain descriptor message types as ordinary fields, the corresponding feature implementations must be available on `buffa-descriptor`.

For example, generating JSON for a message that contains `FileDescriptorProto` requires the descriptor type's JSON implementation, not merely JSON enabled on the surrounding message.

Descriptor enum types can generally be referenced without enabling these optional message implementations.

This is an additional cross-crate feature-forwarding boundary alongside the `buffa-types/reflect` dependency described in the main guide.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://raw.githubusercontent.com\&sz=32)

raw\.githubusercontent.com



### 7.3 Embedded descriptors omit source-code annotations

Generated reflection exposes `FILE_DESCRIPTOR_SET_BYTES`, but the embedded set has its `source_code_info` removed.

The generated descriptor set covers the generation corpus and applicable imports rather than being a minimal per-package source-descriptor artifact.

It is therefore suitable for runtime message identity and schema inspection, but it does not preserve source comments and source-location metadata needed by all documentation or source-analysis tools.

Those applications require a separately produced descriptor set retaining source information.

The shared-pool mechanism deduplicates descriptor data; it does not restore omitted source metadata.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 7.4 Unpublished and development-only functionality

The v0.9.2 repository contains useful implementations and examples that are not separately published production crates.

| Component or feature                                                                    | Release classification                          | Consequence                                                                      |
| --------------------------------------------------------------------------------------- | ----------------------------------------------- | -------------------------------------------------------------------------------- |
| `buffa-yaml`                                                                            | In-tree, `publish = false`                      | YAML adapter exists in the tagged repository but is not a published v0.9.2 crate |
| `buffa-smolstr`                                                                         | Unpublished example/reference crate             | Demonstrates `ProtoString` integration; not a normal published dependency        |
| `buffa-proto-options`                                                                   | Not part of tagged v0.9.2 workspace             | Do not treat as a released companion                                             |
| `FieldMask::normalize`, `covers`, `union`, `intersection`                               | Absent from tagged implementation               | Do not import newer FieldMask APIs into v0.9.2 documentation                     |
| `deny_unknown_json_fields`, `preserve_unknown_fields_in`, `skip_debug` codegen controls | Absent from tagged v0.9.2 `buffa-build::Config` | Development-branch documentation must not be used as the released API contract   |
| `LinkOptions` and newer descriptor-linking controls                                     | Not part of the tagged v0.9.2 pool interface    | Do not assume later import-visibility configuration exists in this release       |

The in-tree `buffa-yaml` adapter is worth noting as a development capability. It routes Buffa's protobuf-aware JSON Serde implementations through a YAML carrier, retaining ProtoJSON-oriented field and value behavior rather than implementing a wholly independent protobuf YAML semantic model.

Its limitations include missing specialized protoyaml features and the ordinary resource risks associated with YAML anchor and alias expansion.

For v0.9.2, however, it belongs in a clearly marked unpublished integration note rather than the principal package catalog.

The v0.9.2 workspace declares Rust 1.75 as its minimum supported Rust version. Optional dependencies and generated-code feature combinations introduce additional compatibility requirements that should be evaluated independently from that workspace-level declaration.

Tagged manifests:

`https://github.com/anthropics/buffa/blob/v0.9.2/buffa-yaml/Cargo.toml`

`https://github.com/anthropics/buffa/blob/v0.9.2/examples/buffa-smolstr/Cargo.toml`

`https://github.com/anthropics/buffa/blob/v0.9.2/Cargo.toml`

## 8. Capability interactions and architectural implications

The additional capabilities strengthen four areas of Buffa's architecture.

| Capability combination                                        | What the library provides                                                                                                                           | Important boundary                                                                                      |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Borrowed views + backing-aware rope + bounded encoding        | Inspects incoming protobuf data and re-encodes it into size-constrained segmented buffers while retaining eligible payload slices                   | Logical re-encoding does not guarantee byte-identical forwarding; transport may flatten segments        |
| DescriptorPool + DynamicMessage + custom options              | Supports dynamic schema discovery, field-level transformation, extension interpretation, and conversion between generated and runtime-defined types | Requires compatible descriptor corpora and careful handling of pool identity                            |
| Codegen diagnostics + shared corpus context + companion files | Enables external generation infrastructure to reuse Buffa's schema analysis, add Rust artifacts, and integrate diagnostics with generation outputs  | Buffa does not itself provide every downstream service or framework generator                           |
| Custom storage + generated reflection/JSON adapters           | Allows physical Rust representation to differ from protobuf wire identity while retaining multiple generated execution interfaces                   | Supported combinations depend on field position, trait availability, Rust coherence, and crate features |

A common theme across these mechanisms is the separation of responsibilities that might otherwise become entangled in a custom protobuf implementation:

Schema semantics remain defined by protobuf descriptors. Representation can be owned, borrowed, or dynamic. Serialization policy can govern size and buffer reuse. Introspection can use linked descriptors. Generation can combine Buffa output with other schema-derived Rust artifacts.

These mechanisms can be composed selectively. Their value is not conditional on using every advanced capability simultaneously; in many cases the useful optimization is confined to one boundary, such as message output, schema discovery, or custom field storage.

## 9. Additional primary-source references

The following sources supplement the original guide's reference map. All URLs are pinned to the v0.9.2 tag.

| Subject                                        | Primary source                                                                            |
| ---------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Bounded encoding and reader decoding           | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs`                    |
| Size-cache internals and pooling               | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/size_cache.rs`                 |
| Rope, backing-aware encoding, vectored buffers | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs`                |
| Dynamic message mutation and `Any`             | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs` |
| Dynamic JSON parsing and serialization         | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/json.rs`    |
| Linked descriptor pools                        | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/pool.rs`            |
| Reflective access and mutation traits          | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/message.rs` |
| Typed extension machinery                      | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/extension.rs`                  |
| Codegen companion output and configuration     | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs`                |
| Protobuf `Any` helpers                         | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-types/src/any_ext.rs`              |
| Field-mask helpers                             | `https://github.com/anthropics/buffa/blob/v0.9.2/buffa-types/src/field_mask_ext.rs`       |
| Version-specific integration guide             | `https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md`                           |

These additions complete the most consequential gaps identified in the original reference without changing its overarching architectural model or substituting development-branch behavior for the released v0.9.2 API.