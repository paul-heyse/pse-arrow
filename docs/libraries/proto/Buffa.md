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