# Protocol Buffers generator ecosystem for Rust: Prost, Tonic, Serde, JSON Schema, and documentation

Technical capability reference | Research date: October 31, 2026 | Focus: code generation, runtime integration, schema publication, compatibility, and reproducibility

## 1. Architecture and capability map

The requested tools form a multi-output Protocol Buffers compilation ecosystem, rather than a single Rust library. They share protobuf schemas as input but produce different artifacts, have different runtime dependencies, and impose distinct requirements on generated-code packaging and publication.

The relevant Rust generator family is maintained in `neoeinstein/protoc-gen-prost`. The JSON Schema generator is indeed the separate PUBG project, `pubg/protoc-gen-jsonschema`, implemented in Go. The documentation generator is `pseudomuto/protoc-gen-doc`, also implemented in Go.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

neoeinstein/protoc-gen-prost · GitHub+2



### 1.1 Verified generator versions

These are the latest published versions identified from upstream registries and releases, not claims about what is installed on a particular machine.

| Generator                    | Verified release | Responsibility                                                  |
| ---------------------------- | ---------------- | --------------------------------------------------------------- |
| `protoc-gen-prost`           | 0.5.0            | Rust protobuf message and enum definitions                      |
| `protoc-gen-tonic`           | 0.5.0            | Tonic gRPC clients, servers, and service integration            |
| `protoc-gen-prost-serde`     | 0.4.0            | Protobuf JSON-compatible Serde implementations                  |
| `protoc-gen-prost-crate`     | 0.5.0            | Rust module inclusion, Cargo crate generation, package features |
| `pubg/protoc-gen-jsonschema` | v0.8.0           | JSON Schema documents from protobuf descriptors                 |
| `pseudomuto/protoc-gen-doc`  | v1.5.1           | HTML, Markdown, DocBook, and documentation-oriented JSON        |

The Rust generator releases dated November 30, 2025 use the Prost 0.14 generation ecosystem, with Tonic generation built on `tonic-build` 0.14.1 and Serde generation built on `pbjson-build` 0.8.0. These are generator dependencies, not automatically the dependencies declared by a generated crate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+5



Primary repositories:

- Rust generator family: [https://github.com/neoeinstein/protoc-gen-prost](https://github.com/neoeinstein/protoc-gen-prost?utm_source=chatgpt.com)
- PUBG JSON Schema: [https://github.com/pubg/protoc-gen-jsonschema](https://github.com/pubg/protoc-gen-jsonschema?utm_source=chatgpt.com)
- Documentation generator: [https://github.com/pseudomuto/protoc-gen-doc](https://github.com/pseudomuto/protoc-gen-doc?utm_source=chatgpt.com)
- Buf generation configuration: [https://buf.build/docs/configuration/v2/buf-gen-yaml/](https://buf.build/docs/configuration/v2/buf-gen-yaml/?utm_source=chatgpt.com)

### 1.2 The compilation architecture

Canonical `.proto` schemas and imported definitions

Messages, enums, services, comments, options, package identities

`protoc` / Buf compiler and plugin protocol

File descriptors, generator parameters, insertion points, selected inputs

protoc-gen-prost

Rust messages and enums

protoc-gen-tonic

gRPC clients and servers

protoc-gen-prost-serde

ProtoJSON Serde implementations

protoc-gen-prost-crate

Modules, features, Cargo manifest

PUBG JSON Schema

JSON validation/schema artifacts

protoc-gen-doc

Human-readable API reference

Rust runtime consumers

`prost`, `tonic`, `tonic-prost`, `serde`, `pbjson`, `pbjson-types` as applicable

Published contract consumers

JSON Schema validators, schema catalogs, documentation sites, other language clients

The diagram captures the central architectural distinction: code generators, generated artifacts, runtime libraries, and schema-publication artifacts are separate layers.

The protobuf descriptor model links these layers, but does not guarantee that every generator interprets the same descriptor semantics identically.

### 1.3 Where the major risks arise

Four boundaries deserve particular attention:

1. Generated Rust code versus runtime versions. Generator releases, Prost versions, Tonic versions, and PBJSON versions must be treated as related but independently versioned components.

2. Generator output versus Rust module layout. File paths, protobuf package names, insertion points, and Cargo module declarations must agree. Successfully generating files does not establish that the resulting Rust crate compiles.

3. ProtoJSON versus ordinary JSON Schema. A schema that accurately describes protobuf types is not necessarily accurate for the JSON representation emitted by the Rust runtime. The PUBG generator explicitly defaults to representing 64-bit integers differently from ProtoJSON.&#x20;

   [image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

   GitHub

4. Source generation versus distribution. Locally installed plugins, Buf remote plugins, generated Cargo manifests, published schema artifacts, and published Rust crates have different reproducibility and dependency-management requirements.

These distinctions determine whether the stack is a coherent compilation system or merely several generators that happen to accept the same `.proto` input.

## 2. `protoc-gen-prost`: Rust message generation

Role: Convert protobuf messages, enums, fields, and type references into Rust definitions compatible with the Prost runtime.

This is the foundation of the Rust output pipeline. The other Rust plugins either augment its output or organize its generated definitions.

### 2.1 Exact supported options

The following options are documented for `protoc-gen-prost` 0.5.0. They are parameter names passed through `--prost_opt` or a Buf plugin's `opt:` configuration, not Rust method calls.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



| Option                                     | Effect and integration significance                                                                       |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------- |
| `boxed=<proto_path>`                       | Box selected message fields; affects ownership and recursive representations.                             |
| `btree_map=<proto_path>`                   | Generate `BTreeMap` rather than `HashMap`; changes map iteration behavior and representation.             |
| `bytes=<proto_path>`                       | Use `bytes::Bytes` for selected protobuf byte fields instead of `Vec<u8>`.                                |
| `default_package_filename=<value>`         | Specify the generated filename for protobuf definitions without a package.                                |
| `skip_debug=<value>`                       | Control generated `Debug` implementation behavior.                                                        |
| `disable_comments=<proto_path>`            | Exclude protobuf comments from the generated Rust code.                                                   |
| `extern_path=<proto_path>=<rust_path>`     | Resolve a protobuf package or type to an external Rust implementation instead of generating another copy. |
| `compile_well_known_types`                 | Enable generation of protobuf well-known types rather than using the default external Prost definitions.  |
| `retain_enum_prefix`                       | Preserve protobuf enum-value prefixes that Prost would otherwise remove.                                  |
| `field_attribute=<proto_path>=<attribute>` | Insert arbitrary Rust attributes on selected fields.                                                      |
| `type_attribute=<proto_path>=<attribute>`  | Insert arbitrary attributes on selected generated types.                                                  |
| `enable_type_names`                        | Enable generation of Prost type-name information for protobuf messages.                                   |
| `file_descriptor_set`                      | Embed a binary `FileDescriptorSet` in generated module output.                                            |
| `flat_output_dir`                          | Flatten generated output into the configured directory rather than mirroring protobuf source paths.       |

Three groups of options are especially consequential.

Representation control. `bytes`, `boxed`, and `btree_map` change the generated Rust data model. The corresponding Serde generator must be configured consistently where it also generates code that depends on those choices. For example, enabling `btree_map=.` for all maps in Prost but not in the JSON generator can produce incompatible generated implementations.

Namespace and identity control. `extern_path`, `default_package_filename`, and `retain_enum_prefix` affect generated references or names. They must be treated as part of the generated-code compatibility contract, rather than inconsequential formatting preferences.

Extension control. `type_attribute` and `field_attribute` offer direct access to Rust attributes without creating a custom generator. This can be useful for adding derives, serialization attributes, deprecation annotations, or integrations with other Rust tooling. Arbitrary injected attributes are not guaranteed to be compatible with all generated type forms.

For example:

```
opt:
  - bytes=.
  - btree_map=.example.v1
  - file_descriptor_set
  - type_attribute=.example.v1.Record=#[derive(Eq\, Hash)]
```

The explicit comma escaping in injected attributes matters because `protoc` represents plugin parameters as a comma-separated string. Protobuf path selectors beginning with `.` match from the fully qualified protobuf root; unqualified paths match suffixes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 2.2 Output layout and generated module boundaries

Two independent naming systems exist:

- Protobuf package identity: A declaration such as `package example.api.v1;` determines the logical protobuf namespace and contributes to generated Rust type paths.
- Filesystem layout: The plugin normally mirrors protobuf input paths within the output directory, while generated module files use protobuf package-derived names. For example, the documented package filename convention for `helloworld.abstract.v1` is `helloworld.abstract.v1.rs`.

`flat_output_dir` changes the directory organization, not the underlying protobuf identity.

This distinction is critical when placing generated code in a published crate. A physical Rust file named `example.api.v1.rs` is not automatically an importable Rust module hierarchy. The hierarchy must be constructed by an include file, explicit module declarations, or an equivalent mechanism.

The plugin also provides insertion points:

| Insertion point   | Scope                                                                                    |
| ----------------- | ---------------------------------------------------------------------------------------- |
| `module`          | Append generated additions to an individual protobuf-package module file                 |
| `<proto_package>` | Append content inside the corresponding package module when an include file is generated |

These allow companion generators to attach implementations without modifying already-emitted message definitions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.3 Descriptor emission is a separate capability

`file_descriptor_set` is more important than its name might suggest.

Unlike the `prost-build::Config::file_descriptor_set_path` mechanism, this plugin can embed module-scoped descriptor sets into generated output. Upstream specifically documents two properties:

1. Each module's descriptor output is scoped to that module rather than indiscriminately embedding the complete generation input.
2. The descriptors are preserved as raw encoded data instead of being decoded and re-encoded through Prost first, preserving otherwise unrecognized tags and extensions.&#x20;

   [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

   Docs.rs



That can support reflection, dynamic message interpretation, descriptor registries, and schema discovery without requiring a separate compilation pipeline.

It does not mean Prost supplies full runtime reflection. The consumer still needs a reflection implementation, such as `prost-reflect`, or a gRPC reflection service implementation.

Descriptor publication also presents an important distinction: a module-scoped descriptor artifact must include or be accompanied by sufficient imported descriptor definitions for its intended downstream consumer.

Primary source: [https://docs.rs/crate/protoc-gen-prost/0.5.0](https://docs.rs/crate/protoc-gen-prost/0.5.0?utm_source=chatgpt.com)

## 3. `protoc-gen-prost-serde`: ProtoJSON-compatible serialization

Role: Generate `serde::Serialize` and `serde::Deserialize` implementations for Prost-generated Rust types using the protobuf JSON mapping rather than ordinary Rust-derived JSON conventions.

This plugin builds on `pbjson-build`. It is not equivalent to adding `#[derive(Serialize, Deserialize)]` through `type_attribute`.

### 3.1 Exact supported options

| Option                                 | Effect                                                                                               |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `btree_map=<proto_path>`               | Match map representation selected during Prost generation.                                           |
| `default_package_filename=<value>`     | Match the no-package output naming policy.                                                           |
| `extern_path=<proto_path>=<rust_path>` | Resolve externally implemented protobuf definitions.                                                 |
| `retain_enum_prefix`                   | Align enum naming with Prost generation.                                                             |
| `preserve_proto_field_names`           | Serialize using original protobuf field names rather than the default JSON names.                    |
| `ignore_unknown_fields`                | Change JSON deserialization behavior for unrecognized object properties.                             |
| `emit_fields`                          | Control emission of default-valued fields in JSON output.                                            |
| `use_integers_for_enums`               | Represent enums by numeric values rather than symbolic names.                                        |
| `no_include`                           | Produce a standalone generated implementation file without inserting an include into Prost's output. |

These are the documented options for version 0.4.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 3.2 Why PBJSON matters

ProtoJSON introduces serialization semantics that differ from ordinary Serde derives.

| Concern            | ProtoJSON-sensitive behavior                                                                 |
| ------------------ | -------------------------------------------------------------------------------------------- |
| `int64` / `uint64` | Normally serialized as decimal strings to preserve precision across JSON consumers           |
| Enums              | Normally use symbolic protobuf names; numeric representation requires an explicit convention |
| Field names        | JSON names commonly use lowerCamelCase or explicit protobuf `json_name` options              |
| Bytes              | Base64-encoded JSON strings                                                                  |
| Presence           | Field omission, defaults, explicit presence, and `null` require protobuf-specific handling   |
| Well-known types   | Types such as `Timestamp`, `Duration`, `Any`, and wrapper messages have specialized mappings |
| Unknown fields     | Deserialization acceptance depends on JSON implementation and options                        |

These conventions are important for interoperating with standard protobuf clients, schema validators, and external services.

An ordinary Serde derive may produce a valid JSON representation of a Rust struct while failing to produce the JSON representation defined by the protobuf specification.

`protoc-gen-prost-serde` supplies the protobuf-aware implementation and uses `pbjson` as supporting runtime functionality.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 3.3 The well-known-type bridge

The documented integration uses:

```
- compile_well_known_types
- extern_path=.google.protobuf=::pbjson_types
```

on the Prost generator, and corresponding compatible type mappings on the service generator.

The result is that references to `google.protobuf` types can resolve to `pbjson_types`, providing well-known types with JSON serialization implementations.

The two settings have distinct responsibilities:

`compile_well_known_types` controls how the generator handles well-known-type definitions. `extern_path` controls which Rust implementation generated references use. The combination allows the external `pbjson_types` crate to supply those implementations while other messages refer to them consistently.

Without a coherent external-type mapping, one part of the output may refer to `prost_types::Timestamp` while another expects `pbjson_types::Timestamp`. These are distinct Rust types even if they represent the same protobuf message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 3.4 Generated implementation placement

In the normal combined generation mode, the Serde plugin inserts generated code into the Prost modules using insertion points.

In separate invocations, the compiler cannot necessarily resolve the original insertion target. The documented solution is:

```
protoc -I proto example.proto \
  --prost_out=gen \
  --prost_opt=compile_well_known_types \
  --prost_opt=extern_path=.google.protobuf=::pbjson_types

protoc -I proto example.proto \
  --prost-serde_out=gen \
  --prost-serde_opt=no_include
```

The resulting `.serde.rs` file must then be included separately in the correct Rust module context. A generated implementation file that exists on disk but is never included has no effect on compiled types.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Primary sources:

[https://docs.rs/crate/protoc-gen-prost-serde/0.4.0](https://docs.rs/crate/protoc-gen-prost-serde/0.4.0?utm_source=chatgpt.com)

[https://docs.rs/crate/pbjson-build/0.8.0](https://docs.rs/crate/pbjson-build/0.8.0?utm_source=chatgpt.com)

## 4. `protoc-gen-tonic`: gRPC service generation

Role: Generate Rust implementations of protobuf service interfaces for the Tonic gRPC framework.

It complements `protoc-gen-prost`: the message generator defines request and response types, while the Tonic generator creates client and server bindings that reference those types.

### 4.1 Exact supported options

| Option                                          | Effect                                                                                         |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `default_package_filename=<value>`              | Align package-less module naming.                                                              |
| `extern_path=<proto_path>=<rust_path>`          | Apply external protobuf-to-Rust type resolution to service signatures.                         |
| `compile_well_known_types`                      | Align well-known-type resolution.                                                              |
| `disable_package_emission`                      | Change the package information emitted into generated service definitions.                     |
| `server_attribute=<proto_path>=<attribute>`     | Inject attributes on selected server elements.                                                 |
| `server_mod_attribute=<proto_path>=<attribute>` | Inject attributes on generated server modules.                                                 |
| `client_attribute=<proto_path>=<attribute>`     | Inject attributes on selected client elements.                                                 |
| `client_mod_attribute=<proto_path>=<attribute>` | Inject attributes on generated client modules.                                                 |
| `no_server`                                     | Disable server generation.                                                                     |
| `no_client`                                     | Disable client generation.                                                                     |
| `no_transport`                                  | Disable the generated convenience connection method using `tonic::transport::Channel`.         |
| `no_include`                                    | Skip inclusion into the Prost-generated module and emit independently includable service code. |

These are the documented `protoc-gen-tonic` 0.5.0 options, including the subset inherited from `tonic-build`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



`no_transport` is not equivalent to `no_client`. It can retain a generated client that accepts an existing compatible transport or service while omitting Tonic's convenience connection constructor.

Similarly, `no_server` and `no_client` allow different generated artifacts to expose only the relevant half of a service interface. This is a code-generation boundary, not an RPC protocol change.

### 4.2 Generated service capabilities

Tonic's service model supports unary, client-streaming, server-streaming, and bidirectional-streaming RPCs. The generator converts the streaming declarations and message references into Tonic client and server machinery.

The generated bindings can participate in Tonic's existing ecosystem for transport, Tower service composition, interceptors, deadlines, metadata, compression, and streaming. Those mechanisms belong to Tonic and its runtime dependencies, not to the generator executable itself.

### 4.3 An easily missed runtime dependency: `tonic-prost`

For the verified 0.5.0 generator, the source implements its service-method codec path as:

```
fn codec_path(&self) -> &str {
    "tonic_prost::ProstCodec"
}
```

This is significant: generated gRPC code refers to `tonic_prost::ProstCodec`, not only to `tonic` and `prost`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



A minimal dependency family for such generated code therefore includes:

```
[dependencies]
prost = "0.14"
tonic = "0.14"
tonic-prost = "0.14"
```

Additional dependencies depend on which features are generated and the schema's types.

`tonic-prost` supplies the protobuf codec. `tonic` supplies the gRPC runtime, transport and service infrastructure. `prost` supplies protobuf message encoding and decoding. The plugin produces Rust source that joins them.

This separation is especially important in Tonic 0.14, which also has `tonic-prost-build` for integrating Prost compilation with Tonic in a Cargo build script. The existence of `tonic-build` alone should not be interpreted as covering every part of the Prost runtime or build integration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### 4.4 Inclusion and sequencing

`protoc-gen-tonic` normally expects a preceding Prost generation step and inserts service bindings into the corresponding generated module.

The plugin explicitly supports a standalone mode:

```
protoc -I proto example.proto --prost_out=gen
protoc -I proto example.proto \
  --tonic_out=gen \
  --tonic_opt=no_include
```

The independently produced `.tonic.rs` file must be included by the consuming Rust crate. Otherwise, the service bindings are not compiled.

When using Buf, the documented order is Prost before Tonic, with matching well-known-type and external-path options where those have been customized.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Primary sources:

[https://docs.rs/crate/protoc-gen-tonic/0.5.0](https://docs.rs/crate/protoc-gen-tonic/0.5.0?utm_source=chatgpt.com)

[https://docs.rs/tonic-build/latest/tonic_build/](https://docs.rs/tonic-build/latest/tonic_build/?utm_source=chatgpt.com)

[https://docs.rs/tonic-prost/latest/tonic_prost/](https://docs.rs/tonic-prost/latest/tonic_prost/?utm_source=chatgpt.com)

## 5. `protoc-gen-prost-crate`: generated modules and Cargo packaging

This generator has an unusually important responsibility: it transforms a collection of generated protobuf module files into a Rust crate that can be compiled, feature-gated, and potentially published.

It addresses a distinct problem from protobuf message generation.

### 5.1 Exact options and defaults

| Option                             | Semantics                                                                                              |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `include_file=<path>`              | Specify the generated Rust include/module file                                                         |
| `gen_crate[=<template_path>]`      | Generate or update a Cargo crate using a manifest template                                             |
| `default_package_filename=<value>` | Match Prost's naming of package-less generated files                                                   |
| `only_include=<proto_path>`        | Restrict generated module inclusion and features to packages with the specified fully qualified prefix |
| `no_features`                      | Suppress package-based Cargo feature generation                                                        |
| `package_separator=[-_.+]`         | Choose the separator used when constructing package-derived feature names                              |

The important defaults are:

| Mode                | Default include file |
| ------------------- | -------------------- |
| Without `gen_crate` | `mod.rs`             |
| With `gen_crate`    | `src/lib.rs`         |

The `include_file` option overrides these defaults.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 Generating a feature dependency graph

The crate generator can construct Cargo features reflecting protobuf package dependencies.

For example, the upstream documentation shows the general form:

```
[features]
default = ["proto_full"]

proto_full = ["example-v1", "shared-v1"]
"example-v1" = ["shared-v1"]
"shared-v1" = []
```

This is more sophisticated than enabling or disabling modules independently. If a generated package depends on another protobuf package, its generated feature can activate the dependent package.

The value is primarily build-time: consumers of a generated Rust SDK can compile a smaller surface when only certain package families are required.

This graph concerns protobuf package relationships, not arbitrary Cargo dependencies. The generator documentation explicitly states that manifest template generation does not infer or maintain the external crate dependency list.

For example, generating services and Serde implementations does not automatically establish the correct dependency declarations for `tonic`, `tonic-prost`, `pbjson`, or `pbjson-types`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 5.3 Manifest template insertion points

The generator uses a special comment marker to identify the section of `Cargo.toml` where the feature graph belongs:

```
[package]
name = "example-protos"
version = "0.1.0"
edition = "2021"

[features]
default = ["proto_full"]
## @@protoc_insertion_point(features)

[dependencies]
prost = "0.14"
```

The plugin manages the generated feature section, preserving surrounding manually maintained content.

If the insertion marker is absent, the documentation states that the features insertion is silently skipped. This makes the marker a functional requirement rather than an optional comment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



The upstream README also contains older sample manifests with `prost = "0.10.0"`. That value is an illustrative historical dependency, not the correct version to infer for the 0.5.0 generator family.

### 5.4 `only_include` and private dependencies

`only_include=.example` can limit the public module set and generated features to selected protobuf packages. This can be useful when the generation input includes vendored schemas or imported protobuf modules that should not become part of the public SDK.

However, this filtering does not automatically resolve references to omitted types.

For packages that remain external, `extern_path` or an equivalent explicit Rust dependency integration may still be necessary. Filtering an include tree is not the same as substituting an external implementation for referenced protobuf types.

### 5.5 Publication constraints

Two documented constraints can directly invalidate a generated crate intended for crates.io:

Feature names: `package_separator=.` produces feature names that the generator's documentation identifies as unsuitable for crates.io publication. The default is `-`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Incomplete manifests: The generator does not infer dependencies. A generated crate may look structurally complete while its manifest lacks the runtime dependencies required by the generated Rust source.

There is also a maintenance boundary: generated include files and manifest feature sections are regeneration targets. Arbitrary manual modifications to them are not reliable places for lasting custom code.

The upstream alternative is to maintain a stable handwritten `lib.rs` that includes generated modules and adds application-specific extensions outside the regeneration surface.

Primary source:

[https://docs.rs/crate/protoc-gen-prost-crate/0.5.0](https://docs.rs/crate/protoc-gen-prost-crate/0.5.0?utm_source=chatgpt.com)

## 6. PUBG `protoc-gen-jsonschema`: publishable JSON contracts

Role: Generate JSON Schema documents from protobuf definitions, primarily for validating and documenting JSON representations of protobuf messages.

This is a separate Go-based compiler plugin, not a Prost or Tonic extension. Its output can be consumed by Rust JSON Schema validators and schema catalogs, but it does not generate Rust validator implementations.

### 6.1 Scope and supported formats

The v0.8.0 generator supports protobuf `proto2` and `proto3`, with JSON Schema drafts 04, 06, 07, 2019-09, and 2020-12 documented as supported. The plugin option reference also lists `Draft05`, although the introductory support statement does not. Treat that discrepancy as a documentation inconsistency rather than a verified additional compatibility guarantee.

It can emit JSON or YAML schema documents. Its stated goal is to cover protobuf JSON representations and common validation patterns, not to implement the entire JSON Schema vocabulary or translate arbitrary JSON Schema semantics into protobuf.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Most importantly, the v0.8.0 release notes explicitly remove claimed protobuf Edition 2023 support, clarifying that this edition was never actually supported. This is a concrete schema-language compatibility boundary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 6.2 Exact plugin options

| Option                       | Default        | Significance                                                                      |
| ---------------------------- | -------------- | --------------------------------------------------------------------------------- |
| `entrypoint_message`         | Empty          | Select the message that serves as the schema's entry-point object.                |
| `output_file_suffix`         | `.schema.json` | Set the output suffix; `.json`, `.yaml`, and `.yml` are supported endings.        |
| `pretty_json_output`         | `true`         | Control whitespace formatting for JSON output.                                    |
| `draft`                      | `Draft202012`  | Select the JSON Schema dialect.                                                   |
| `mandatory_nullable`         | `false`        | Change how optional protobuf fields are treated with respect to JSON nullability. |
| `int64_as_string`            | `false`        | Deprecated alias for `respect_protojson_int64`.                                   |
| `respect_protojson_int64`    | `false`        | With `true`, represent 64-bit protobuf integers as JSON strings.                  |
| `preserve_proto_field_names` | `false`        | Retain protobuf source field names instead of JSON names / lowerCamelCase.        |
| `additional_properties`      | `DoNothing`    | Control whether generated object schemas permit additional properties.            |
| `respect_protojson_presence` | `false`        | Adjust JSON Schema required-field generation based on protobuf presence.          |

The `additional_properties` option accepts `AlwaysTrue`, `AlwaysFalse`, `DefaultTrue`, `DefaultFalse`, and `DoNothing`. The `Default*` forms affect messages lacking explicit overrides, whereas `Always*` imposes the setting globally.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



A typical invocation intended to more closely describe protobuf JSON output is:

```
protoc -I proto \
  --jsonschema_out=gen/schema \
  --jsonschema_opt=draft=Draft202012 \
  --jsonschema_opt=respect_protojson_int64=true \
  --jsonschema_opt=preserve_proto_field_names=false \
  proto/example/v1/example.proto
```

The upstream README additionally recommends `respect_protojson_presence=true` for ProtoJSON alignment. However, it describes that option as generating JSON Schema `required` entries for certain fields with protobuf presence. Presence and requiredness are not equivalent concepts: a field can support tracking whether it was set without being required in every JSON document. Consequently, enabling that option deserves validation against the intended message contract rather than automatic adoption.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 6.3 Schema annotations through `jsonschema.proto`

PUBG provides its own protobuf option extensions. Importing its `jsonschema.proto` makes these annotations part of the source schema, where the generator can read them from protobuf descriptors.

| Annotation scope | Available capabilities                                                                                     |
| ---------------- | ---------------------------------------------------------------------------------------------------------- |
| File             | Entry-point message, title, description                                                                    |
| Message          | Title, description, object constraints such as `additional_properties`, minimum/maximum property count     |
| Field            | Title, description, nullability, default value, array constraints, numeric constraints, string constraints |
| Enum             | Title, description, enum mapping mode                                                                      |
| Enum value       | Custom JSON value, represented using `google.protobuf.Any`                                                 |

The supported constraint groups include array length and uniqueness; numeric minimum, maximum, exclusivity, and multiples; and string length, pattern, and format. Enum mappings distinguish `MapToString`, `MapToNumber`, and `MapToCustom`.

These are useful extension points because they allow a protobuf contract to describe more than protobuf's intrinsic field types. For example, an application might attach a string pattern or numeric interval that should appear in the generated JSON Schema.

They also introduce a crucial distinction: additional JSON Schema constraints are not automatically enforced by Prost or Tonic. A service can successfully decode a protobuf message that violates a PUBG-specific numeric or string constraint unless another validation mechanism enforces that constraint.

The annotation reference also documents `visibility_level`, but marks it work in progress and not functioning. It should not be treated as a released filtering mechanism.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Source: [https://github.com/pubg/protoc-gen-jsonschema/blob/main/jsonschema.proto](https://github.com/pubg/protoc-gen-jsonschema/blob/main/jsonschema.proto?utm_source=chatgpt.com)

Reference: [https://github.com/pubg/protoc-gen-jsonschema/blob/main/options.md](https://github.com/pubg/protoc-gen-jsonschema/blob/main/options.md?utm_source=chatgpt.com)

### 6.4 JSON Schema and ProtoJSON are different contracts

The 64-bit integer default demonstrates the danger particularly clearly.

```
message Measurement {
  int64 observation_id = 1;
}
```

A conformant ProtoJSON serializer normally emits:

```
{"observationId":"9007199254740993"}
```

With PUBG's default `respect_protojson_int64=false`, the generated JSON Schema instead describes an integer representation. Enabling that option changes the schema to expect a string, matching canonical ProtoJSON output more closely.

The protobuf specification also requires attention to several other differences:

| Semantic concern  | Compatibility issue                                                                                                                                     |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Omitted fields    | A protobuf scalar without explicit presence can be omitted at its default value; a JSON Schema `required` constraint has different semantics.           |
| Explicit `null`   | ProtoJSON parsers accept `null` for ordinary fields as an unset value, even though serializers should generally not emit it.                            |
| Enum values       | Serializers normally emit names; protobuf parsers can accept numeric values.                                                                            |
| Field names       | ProtoJSON parsers may accept protobuf field names and JSON field names; schemas commonly describe only a selected spelling.                             |
| Extra fields      | Standard ProtoJSON parsing rejects unknown fields unless an ignore-unknown option is enabled; JSON Schema controls this through `additionalProperties`. |
| Well-known types  | `Timestamp`, `Duration`, `Any`, `Value`, `Struct`, and `FieldMask` have type-specific representations.                                                  |
| Numeric precision | JSON number representations and 64-bit protobuf integers have different interoperability constraints.                                                   |

These distinctions follow from the official ProtoJSON specification, and they are not all resolved merely by enabling PUBG's two `respect_protojson_*` switches.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation+2



The v0.8.0 release also addresses nondeterministic output ordering, which matters for reproducible schema publication and generated-artifact diffs. Stable ordering does not, by itself, establish semantic equivalence or compatibility with every JSON Schema validator.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Architectural implication: PUBG-generated schema is best understood as a separately published JSON contract whose accepted language should be checked against the particular PBJSON serializer/deserializer and consumer validator being used. It is not a formal guarantee that every valid schema instance round-trips through ProtoJSON unchanged.

Primary sources:

[https://github.com/pubg/protoc-gen-jsonschema](https://github.com/pubg/protoc-gen-jsonschema?utm_source=chatgpt.com)

[https://github.com/pubg/protoc-gen-jsonschema/releases/tag/v0.8.0](https://github.com/pubg/protoc-gen-jsonschema/releases/tag/v0.8.0?utm_source=chatgpt.com)

[https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)

## 7. `protoc-gen-doc`: protobuf documentation generation

Role: Convert protobuf descriptors and source comments into publishable API documentation.

Its outputs are documentation artifacts, not executable Rust source or JSON Schema validation contracts.

### 7.1 Formats and invocation

Version 1.5.1 supports these built-in formats:

| Format             | Typical use                                           |
| ------------------ | ----------------------------------------------------- |
| `html`             | Browsable API documentation                           |
| `markdown`         | Repository or static-site documentation               |
| `docbook`          | Structured document publishing                        |
| `json`             | Structured representation of documentation content    |
| Custom Go template | Application-specific report and documentation formats |

The general `protoc` invocation is:

```
protoc -I proto \
  --doc_out=gen/docs \
  --doc_opt=markdown,protobuf-api.md \
  proto/example/v1/example.proto
```

The parameter grammar is:

```
--doc_opt=FORMAT_OR_TEMPLATE,OUTPUT_FILENAME[,default|source_relative]
```

The third parameter selects aggregate/default output versus source-relative output behavior. `source_relative` places documents according to the relative location of their input protobuf files. The default mode can collect multiple protobuf inputs into one documentation artifact.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 7.2 Less obvious controls

File exclusion. A colon-delimited section can exclude files using path patterns:

```
protoc -I proto \
  --doc_out=gen/docs \
  --doc_opt='html,index.html:google/*,third_party/*' \
  proto/example/v1/example.proto
```

The patterns refer to protobuf input paths. This is documentation filtering; it does not change the generated Rust code.

Field naming. Another colon-delimited section supports `camel_case_fields=true|false` (default `false`):

```
protoc -I proto \
  --doc_out=gen/docs \
  --doc_opt='json,api.json::camel_case_fields=true' \
  proto/example/v1/example.proto
```

This can align displayed field names more closely with JSON consumers, but it is independent of the actual PBJSON and PUBG naming options.

Custom templates. Template paths replace the built-in format name. This allows domain-specific documentation output without writing a compiler plugin.

Styling. The HTML generator can pick up a `stylesheet.css` placed alongside the generated output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 7.3 Source comments are part of the documentation input

The generator can document files, messages, fields, enums and enum values, services and methods, and extensions.

Comment placement has semantic consequences:

- Leading comments are broadly supported.
- Trailing comments are supported for fields, service methods, enum values, and extensions.
- File-level documentation attaches to a leading comment on the `syntax` directive.
- `@exclude` in a source comment is recognized as an instruction to omit that comment from generated documentation.

The output therefore depends on source-location information retained during compilation, not merely message and service descriptors. A descriptor set stripped of source comments cannot recreate the same documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 7.4 An important documentation-version trap

The project's current README explicitly documents Go templates. Its GitHub wiki still contains Mustache-template examples. The project's v1.0.0 release notes establish why these conflict: Mustache templates were replaced with Go templates in that release. For v1.5.1, the current README and implementation-era template system take precedence over the historical wiki examples.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



This is a concrete example of why an apparently helpful upstream documentation page can be incompatible with the targeted release.

### 7.5 Documentation JSON is not PUBG JSON Schema

Both plugins can emit `.json`, but the meaning is different.

- `protoc-gen-doc` JSON: Metadata describing protobuf files, declarations, comments, and API surfaces; suitable for rendering or indexing documentation.
- PUBG JSON Schema: A validation-oriented representation of message instances under a selected JSON Schema dialect.

Neither should be substituted for the other's output based on file extension alone. The former describes an API; the latter describes the expected structure of JSON messages.

Primary sources:

[https://github.com/pseudomuto/protoc-gen-doc](https://github.com/pseudomuto/protoc-gen-doc?utm_source=chatgpt.com)

[https://github.com/pseudomuto/protoc-gen-doc/blob/master/README.md](https://github.com/pseudomuto/protoc-gen-doc/blob/master/README.md?utm_source=chatgpt.com)

[https://github.com/pseudomuto/protoc-gen-doc/releases](https://github.com/pseudomuto/protoc-gen-doc/releases?utm_source=chatgpt.com)

## 8. Generation orchestration: `protoc`, Buf, output paths, and Cargo

The same generators can be invoked directly through `protoc` or orchestrated through Buf. The generated Rust code is broadly the same; the significant differences concern plugin invocation grouping, output application, dependency resolution, and reproducibility.

### 8.1 Exact executable and invocation names

| Executable               | `protoc` output flag | `protoc` options flag | Buf local name           |
| ------------------------ | -------------------- | --------------------- | ------------------------ |
| `protoc-gen-prost`       | `--prost_out`        | `--prost_opt`         | `protoc-gen-prost`       |
| `protoc-gen-prost-serde` | `--prost-serde_out`  | `--prost-serde_opt`   | `protoc-gen-prost-serde` |
| `protoc-gen-tonic`       | `--tonic_out`        | `--tonic_opt`         | `protoc-gen-tonic`       |
| `protoc-gen-prost-crate` | `--prost-crate_out`  | `--prost-crate_opt`   | `protoc-gen-prost-crate` |
| `protoc-gen-jsonschema`  | `--jsonschema_out`   | `--jsonschema_opt`    | `protoc-gen-jsonschema`  |
| `protoc-gen-doc`         | `--doc_out`          | `--doc_opt`           | `protoc-gen-doc`         |

The executables must be discoverable by the compiler or supplied through an explicit plugin path. When using Buf v2, `local:` accepts executable names and paths, while `remote:` references plugins hosted on the Buf Schema Registry.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+3



### 8.2 Combined Buf generation

The following is an illustrative integration configuration, constructed from the plugins' documented interfaces. It has not been executed.

```
version: v2

plugins:
  # Protobuf message definitions
  - local: protoc-gen-prost
    out: gen/rust/src
    opt:
      - compile_well_known_types
      - extern_path=.google.protobuf=::pbjson_types
      - file_descriptor_set

  # ProtoJSON Serde implementations
  - local: protoc-gen-prost-serde
    out: gen/rust/src

  # Tonic gRPC services
  - local: protoc-gen-tonic
    out: gen/rust/src
    opt:
      - compile_well_known_types
      - extern_path=.google.protobuf=::pbjson_types

  # Module tree and Cargo crate
  - local: protoc-gen-prost-crate
    out: gen/rust
    strategy: all
    opt:
      - gen_crate=Cargo.toml.tpl

  # JSON Schema publication
  - local: protoc-gen-jsonschema
    out: gen/schema
    opt:
      - draft=Draft202012
      - respect_protojson_int64=true

  # Aggregated API documentation
  - local: protoc-gen-doc
    out: gen/docs
    strategy: all
    opt:
      - markdown,protobuf-api.md
```

This configuration assumes the six local executables are installed and available, and that a valid `Cargo.toml.tpl` can be resolved by the crate generator. The template must supply package metadata, runtime dependencies, and the feature insertion marker.

The Rust output is logically organized as:

```
gen/
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── ... protobuf package files and inserted code
├── schema/
│   └── ... generated JSON Schema files
└── docs/
    └── protobuf-api.md
```

The exact Rust package-file paths depend on protobuf input paths and `flat_output_dir`; the package-derived filename convention is discussed in Section 2. The Serde and Tonic generators attach implementations and services to those packages rather than defining separate public Rust namespaces by default.

### 8.3 Invocation order is an output-application contract

Buf documentation states that plugins can execute in parallel while their outputs are written in the order listed in the configuration. This distinction is particularly important for this generator family: the Prost output must appear before the Serde and Tonic insertion-point responses are applied.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



For direct `protoc` use, the supported combined approach is to put the related output flags in the same compiler invocation. When generators are instead run in separate `protoc` invocations, the base file's insertion points are not necessarily available to the later invocation, and `no_include` becomes necessary.

The fact that two processes write to the same filesystem directory does not make them equivalent to one combined compiler response.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 8.4 Why `strategy: all` is consequential

Buf's default strategy for local plugins is `directory`, in which it divides inputs by directory and makes separate plugin calls. `strategy: all` supplies the complete selected input in one invocation.

The crate generator explicitly requires `strategy: all` to construct a complete protobuf package dependency graph. Without it, the generated Cargo features and module set may describe only a subset of the schema input.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



For `protoc-gen-doc`, `strategy: all` is also the natural choice when generating one aggregate documentation file, rather than having independent directory-scoped invocations target the same output filename.

For the remaining generators, `strategy: all` is a generation-granularity choice rather than a universal requirement.

### 8.5 An additional crate option verified in source

The `protoc-gen-prost-crate` 0.5.0 README omits an option accepted by the actual parameter parser:

`flat_output_dir[=true|false]`

The tagged source passes this setting into the same module-request machinery used to calculate generated paths. Therefore it belongs to the crate generator's supported option surface even though the README's option inventory omits it. It should be aligned with the Prost generator's setting to preserve generated include paths.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Source: [https://docs.rs/protoc-gen-prost-crate/0.5.0/src/protoc_gen_prost_crate/lib.rs.html](https://docs.rs/protoc-gen-prost-crate/0.5.0/src/protoc_gen_prost_crate/lib.rs.html?utm_source=chatgpt.com)

### 8.6 Local plugins versus Buf remote plugins

| Property                 | Local plugin               | BSR remote plugin                                                |
| ------------------------ | -------------------------- | ---------------------------------------------------------------- |
| Execution                | Local process              | Hosted plugin execution                                          |
| Version selection        | Installed binary / package | Registry plugin version and revision                             |
| Local template access    | Possible                   | Generally unavailable                                            |
| Installation requirement | Binary required            | Hosted plugin required                                           |
| Custom plugin support    | Direct local executable    | Must be published to a supported registry                        |
| `gen_crate`              | Supported                  | Explicitly unsupported by upstream crate-generator documentation |

In particular, `gen_crate` requires access to a local Cargo manifest template. The upstream crate generator documents that its remote BSR form is incompatible with this operation. A remotely generated include file is possible; remotely generating an actual crate from a local manifest template is not interchangeable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



There is an additional version trap: the Rust crates are published as 0.5.0/0.4.0, while the examples in their README files still refer to older BSR plugin versions, including `neoeinstein-prost:v0.2.3`, `neoeinstein-tonic:v0.4.1`, and `neoeinstein-prost-serde:v0.3.1`. Those example references do not establish which newer BSR releases are available or whether a given remote combination emits the same runtime references as the current local plugins.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



Important distinction: a registry-hosted plugin version, a crates.io generator version, and a generated crate's own published version are three independent identities.

Primary sources:

[https://buf.build/docs/configuration/v2/buf-gen-yaml/](https://buf.build/docs/configuration/v2/buf-gen-yaml/?utm_source=chatgpt.com)

[https://buf.build/docs/bsr/remote-plugins/usage/](https://buf.build/docs/bsr/remote-plugins/usage/?utm_source=chatgpt.com)

[https://buf.build/docs/bsr/remote-plugins/custom-plugins/](https://buf.build/docs/bsr/remote-plugins/custom-plugins/?utm_source=chatgpt.com)

## 9. Rust runtime compatibility and alternative build mechanisms

The generator family is useful precisely because it lets a polyglot protobuf compilation system emit Rust artifacts. But the resulting Rust code still depends on ordinary Rust runtime libraries.

### 9.1 Runtime dependency bridge

| Generated capability                  | Relevant Rust runtime                                  |
| ------------------------------------- | ------------------------------------------------------ |
| Binary protobuf messages              | `prost`                                                |
| Protobuf well-known types             | `prost-types` or a coherent external replacement       |
| `bytes::Bytes` field representation   | `bytes`                                                |
| gRPC services                         | `tonic`                                                |
| Prost encoding/decoding within Tonic  | `tonic-prost`                                          |
| ProtoJSON Serde implementations       | `serde`, `pbjson`                                      |
| ProtoJSON-compatible well-known types | `pbjson-types`                                         |
| Descriptor-based dynamic reflection   | Additional reflection library, such as `prost-reflect` |
| gRPC server reflection                | Additional Tonic reflection integration                |

The current verified stable runtime lines include Prost 0.14.4 and Tonic 0.14.9. Tonic 0.14.9 was published October 29, 2026, and the Tonic repository explicitly warns that its development branch is preparing breaking changes; the released 0.14 line should not be conflated with unreleased branch code.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



There is a less obvious PBJSON distinction:

- `protoc-gen-prost-serde` 0.4.0 depends on `pbjson-build` 0.8.0.
- `pbjson` and `pbjson-types` have separately published 0.9.0 releases.
- A newer runtime version should not be assumed compatible with emitted code merely because both belong to the PBJSON project.

The generated implementations' referenced APIs and the selected PBJSON runtime line form an actual source-compatibility requirement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



Also, the `pbjson-types` documentation explicitly acknowledges incomplete well-known-type coverage and possible nonconformant implementations. The package is an important bridge, not a blanket guarantee of complete ProtoJSON fidelity for every well-known type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 9.2 Two legitimate compilation architectures

External compiler / Buf architecture

`.proto` definitions are compiled into Rust files, Cargo modules, optional crate manifests, JSON Schema artifacts, and documentation. The Rust crate then compiles the emitted source.

This architecture is particularly valuable where multiple languages and publication formats must derive from the same schema corpus.

Cargo `build.rs` architecture

A Rust-centric crate can instead use `prost-build`, `pbjson-build`, and `tonic-prost-build` directly. This avoids managing separate `protoc-gen-*` Rust executables for the same compilation responsibilities.

Current Tonic documentation identifies `tonic-prost-build` as the Prost build integration:

```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .compile_protos(&["proto/service.proto"], &["proto"])?;
    Ok(())
}
```

This build mechanism is not inherently a replacement for PUBG JSON Schema generation or `protoc-gen-doc`; those are separate outputs. Nor does it remove the need to manage protobuf compiler availability and code-generation versions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



The distinction can be understood as where compilation is orchestrated, not necessarily which Rust runtime is used.

### 9.3 Minimum supported Rust version and feature choices

The verified documentation reports a Prost 0.14.4 MSRV of Rust 1.85 and a Tonic 0.14.9 MSRV of Rust 1.88. A combined generated crate cannot assume Prost's lower MSRV applies to the complete dependency graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Tonic's `codegen`, `transport`, `channel`, and related runtime features also matter. Disabling generated `connect` convenience methods using `no_transport` can reduce the code's direct reliance on Tonic's transport convenience surface, but it does not automatically disable all transport dependencies or determine which Cargo features the eventual application requires.

## 10. Compatibility and publication failure modes

The central question is not merely whether the generators succeed. It is whether they emit a consistent Rust API, compatible runtime behavior, and durable published contracts.

| Compatibility boundary       | Failure that can remain hidden after successful generation                                               |
| ---------------------------- | -------------------------------------------------------------------------------------------------------- |
| Prost / Serde representation | `BTreeMap`/`HashMap`, external types, enum prefixes, or generated paths disagree.                        |
| Prost / Tonic service types  | gRPC signatures refer to different Rust types or unresolved external paths.                              |
| Tonic runtime                | Generated references to `tonic_prost::ProstCodec` are unresolved.                                        |
| PBJSON runtime               | Generated Serde code uses helpers not available in the selected PBJSON version.                          |
| Well-known types             | `prost_types` and `pbjson_types` are substituted inconsistently.                                         |
| Module layout                | `flat_output_dir`, `default_package_filename`, and include generation disagree.                          |
| Cargo features               | Required protobuf package features are missing or transitive dependencies are not activated.             |
| Rust package publication     | Generated files are absent from the `.crate` archive, or manifest dependencies are incomplete.           |
| JSON Schema                  | Integer, presence, naming, enum, null, or unknown-field semantics diverge from actual JSON output.       |
| Documentation                | Old Mustache templates, lost source comments, or document output collisions produce incorrect artifacts. |
| Schema syntax                | A generator such as PUBG receives unsupported protobuf Editions input.                                   |
| Remote generation            | An older registry plugin emits materially different code, or local template access is unavailable.       |

### 10.1 Rust compilation compatibility is not protobuf wire compatibility

Protobuf can preserve wire compatibility while generated Rust types or names change. Conversely, a generated crate can compile successfully even though its JSON representation is incompatible with existing consumers.

Buf exposes this distinction explicitly through its breaking-change categories:

- `FILE` and `PACKAGE` focus on generated-source compatibility.
- `WIRE_JSON` covers binary wire and protobuf JSON compatibility.
- `WIRE` covers binary wire compatibility.

Buf also documents that custom protobuf options are outside its standard breaking-change rules, because their meanings are application-defined. Therefore changing PUBG field constraints or documentation-specific options can change published validation or reference behavior without necessarily triggering Buf's built-in compatibility checks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



Source: [https://buf.build/docs/breaking/rules/](https://buf.build/docs/breaking/rules/?utm_source=chatgpt.com)

### 10.2 Cargo crate publication is a distinct build environment

`protoc-gen-prost-crate` can generate module and feature declarations, but it does not prove that the published `.crate` contains every required file or has all necessary dependencies.

Cargo's publication tooling provides two useful checks:

```
cargo package --list
cargo publish --dry-run
```

These check which files are included and exercise packaging before actual publication. Cargo can exclude files through manifest include/exclude rules and version-control ignore behavior; a generated directory that exists during local development is not necessarily part of the distributable archive.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



Source: [https://doc.rust-lang.org/cargo/reference/publishing.html](https://doc.rust-lang.org/cargo/reference/publishing.html?utm_source=chatgpt.com)

The distinction is especially consequential for generated code included through `include!`: Rust may have compiled successfully in a developer workspace while the files required by the same include statements are absent from the published package.

### 10.3 Generated artifacts have different publication stability requirements

| Artifact                         | Primary compatibility surface                                  |
| -------------------------------- | -------------------------------------------------------------- |
| `.proto` source / descriptor set | Protobuf identities, tags, services, imports, source metadata  |
| Rust crate                       | Public modules, types, feature names, Cargo dependencies, MSRV |
| gRPC bindings                    | Service/method signatures, transport and codec compatibility   |
| ProtoJSON output                 | JSON field names, enum names, scalar mappings, presence        |
| JSON Schema                      | Draft, `$ref` structure, constraints, validation strictness    |
| Generated documentation          | Public contract descriptions, comments, template compatibility |

Publishing all of them from one `.proto` corpus does not make them interchangeable or automatically version-compatible.

The distinction also means that an otherwise wire-compatible protobuf change might warrant a generated Rust crate release, a JSON contract revision, or a documentation update for reasons that protobuf wire compatibility alone cannot capture.

### 10.4 Generated-source reproducibility

A reproducible pipeline needs to preserve more than the `.proto` files:

- Compiler or Buf version and selected schema inputs.
- Exact generator binaries or pinned remote plugins.
- Per-plugin options and invocation grouping.
- External protobuf package-to-Rust type mappings.
- Cargo manifest template and runtime dependency resolution.
- JSON Schema draft and application-specific annotations.
- Documentation templates and preserved source comments.

These are not merely build configuration details. In this ecosystem, they can change the generated public API and wire-adjacent semantics.

## 11. Overall architectural interpretation

This generator family is most useful when protobuf is the central, language-neutral source of interface definitions and Rust is one of several consumers. It offers a coherent path from that source to a reusable Rust SDK, a Tonic service API, protobuf-aware JSON serialization, validation-oriented JSON Schema, and documentation.

The significant architectural opportunities are:

One descriptor pipeline, multiple purpose-specific outputs. Protobuf compilation and descriptors can be reused across Rust code generation, JSON publication, reflection, and documentation without implementing separate ad hoc schema parsers.

Generated modules as the stable Rust boundary. `protoc-gen-prost-crate` provides namespace assembly and package-aware Cargo feature dependencies, leaving higher-level runtime behavior and custom extensions outside the regeneration surface.

Explicit separation between transport and serialization. Prost supplies protobuf message types, PBJSON supplies protobuf JSON behavior, Tonic supplies RPC infrastructure, and PUBG supplies JSON Schema documents. These can be combined or omitted according to the actual interface requirements.

Compatibility treated as a multi-layer contract. The critical decisions are generator/runtime alignment, consistent external types and output paths, protobuf binary versus ProtoJSON semantics, and what is actually included and versioned in published artifacts.

The principal limitation is that the system does not create a universal source of semantic enforcement. JSON Schema annotations are not automatically enforced by Rust protobuf decoding, documentation does not prove runtime behavior, and generated Cargo manifests do not independently manage every dependency.

For a Rust-only build, direct `prost-build` / `pbjson-build` / `tonic-prost-build` orchestration can be simpler. For a polyglot system that publishes code and non-code contracts together, the external plugin family provides meaningful additional capabilities—provided those boundaries remain explicit.

## Primary source map

All URLs are direct, independently accessible references.

Rust generator family

- [https://docs.rs/crate/protoc-gen-prost/0.5.0](https://docs.rs/crate/protoc-gen-prost/0.5.0?utm_source=chatgpt.com)
- [https://docs.rs/crate/protoc-gen-tonic/0.5.0](https://docs.rs/crate/protoc-gen-tonic/0.5.0?utm_source=chatgpt.com)
- [https://docs.rs/crate/protoc-gen-prost-serde/0.4.0](https://docs.rs/crate/protoc-gen-prost-serde/0.4.0?utm_source=chatgpt.com)
- [https://docs.rs/crate/protoc-gen-prost-crate/0.5.0](https://docs.rs/crate/protoc-gen-prost-crate/0.5.0?utm_source=chatgpt.com)
- [https://github.com/neoeinstein/protoc-gen-prost](https://github.com/neoeinstein/protoc-gen-prost?utm_source=chatgpt.com)

Rust runtime and build integration

- [https://docs.rs/prost/0.14.4/prost/](https://docs.rs/prost/0.14.4/prost/?utm_source=chatgpt.com)
- [https://docs.rs/prost-build/latest/prost_build/](https://docs.rs/prost-build/latest/prost_build/?utm_source=chatgpt.com)
- [https://docs.rs/tonic/latest/tonic/](https://docs.rs/tonic/latest/tonic/?utm_source=chatgpt.com)
- [https://docs.rs/tonic-prost/latest/tonic_prost/](https://docs.rs/tonic-prost/latest/tonic_prost/?utm_source=chatgpt.com)
- [https://docs.rs/tonic-prost-build/latest/tonic_prost_build/](https://docs.rs/tonic-prost-build/latest/tonic_prost_build/?utm_source=chatgpt.com)
- [https://docs.rs/pbjson-build/0.8.0/pbjson_build/](https://docs.rs/pbjson-build/0.8.0/pbjson_build/?utm_source=chatgpt.com)
- [https://docs.rs/pbjson-types/latest/pbjson_types/](https://docs.rs/pbjson-types/latest/pbjson_types/?utm_source=chatgpt.com)

JSON Schema and documentation

- [https://github.com/pubg/protoc-gen-jsonschema](https://github.com/pubg/protoc-gen-jsonschema?utm_source=chatgpt.com)
- [https://github.com/pubg/protoc-gen-jsonschema/blob/main/options.md](https://github.com/pubg/protoc-gen-jsonschema/blob/main/options.md?utm_source=chatgpt.com)
- [https://github.com/pubg/protoc-gen-jsonschema/releases/tag/v0.8.0](https://github.com/pubg/protoc-gen-jsonschema/releases/tag/v0.8.0?utm_source=chatgpt.com)
- [https://github.com/pseudomuto/protoc-gen-doc/blob/master/README.md](https://github.com/pseudomuto/protoc-gen-doc/blob/master/README.md?utm_source=chatgpt.com)
- [https://protobuf.dev/programming-guides/json/](https://protobuf.dev/programming-guides/json/?utm_source=chatgpt.com)

Generation and publication

- [https://buf.build/docs/configuration/v2/buf-gen-yaml/](https://buf.build/docs/configuration/v2/buf-gen-yaml/?utm_source=chatgpt.com)
- [https://buf.build/docs/bsr/remote-plugins/usage/](https://buf.build/docs/bsr/remote-plugins/usage/?utm_source=chatgpt.com)
- [https://buf.build/docs/breaking/rules/](https://buf.build/docs/breaking/rules/?utm_source=chatgpt.com)
- [https://doc.rust-lang.org/cargo/reference/publishing.html](https://doc.rust-lang.org/cargo/reference/publishing.html?utm_source=chatgpt.com)

The reference above distinguishes upstream-documented functionality from architectural implications and illustrates configuration rather than claiming that a particular combined stack was built or tested.