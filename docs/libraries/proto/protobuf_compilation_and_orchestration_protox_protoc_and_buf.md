# Protocol Buffers compilation and orchestration for Rust

## `protox` · `protoc` · Buf CLI, plugins, and images

Technical capability reference | Researched October 31, 2026

Verified version scope

| Component         | Current stable version | Role                                                                                 |
| ----------------- | ---------------------- | ------------------------------------------------------------------------------------ |
| `protox`          | 0.10.0                 | Native Rust `.proto` compiler                                                        |
| `protoc`          | 36.2                   | Reference Protocol Buffers compiler and plugin host                                  |
| Buf CLI           | 1.75.0                 | Schema compilation, dependency management, validation, code-generation orchestration |
| Buf configuration | v2                     | Module, workspace, and generation configuration format                               |

The upstream release pages confirm these versions. Protocol Buffers 37.0-rc1 is available as a prerelease but is not the stable target. Buf 1.75.0 adds support for Protobuf edition 2026; this does not imply that every Rust compiler or code generator supports all edition 2026 features.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



## 1. Architectural orientation

The most important distinction is that schema compilation, code generation, and generation orchestration are separate capabilities.

A Protobuf compiler transforms `.proto` source into a graph of fully resolved protocol descriptors. A code generator transforms those descriptors into Rust source or other artifacts. An orchestrator manages input selection, dependencies, plugin execution, compatibility checks, and output organization.

Protocol source

`.proto` files · imports · options · editions

protox

In-process Rust

protoc

Reference compiler

Buf

CLI compiler

Descriptor graph

`FileDescriptorProto` · `FileDescriptorSet` · Buf image

Code generation

`prost-build` · `tonic-build` · `protoc-gen-*`

Descriptor consumers

Reflection · inspection · dynamic messages · tooling

The three compiler/orchestration options have different strengths:

| Dimension                         | `protox`                        | `protoc`                           | Buf                             |
| --------------------------------- | ------------------------------- | ---------------------------------- | ------------------------------- |
| Implementation                    | Rust library                    | Native compiler executable         | Go-based CLI                    |
| Programmatic Rust integration     | Direct library APIs             | Process execution                  | Process execution               |
| Compiles `.proto`                 | Yes                             | Yes                                | Yes                             |
| Descriptor output                 | Rust descriptor structures      | Serialized descriptor sets         | Buf images or descriptor sets   |
| Direct Rust code generation       | Via companion generators        | Via external plugins               | Via local or remote plugins     |
| Schema dependency management      | Customizable file resolution    | Import paths and descriptor inputs | Workspaces, modules, lockfile   |
| Lint and breaking-change analysis | Not its primary role            | Not its primary role               | Native capabilities             |
| Best distinguishing property      | In-process compiler integration | Reference compiler behavior        | End-to-end schema orchestration |

Key implication: These tools are not mutually exclusive. Buf can manage a schema workspace while `protox` performs compilation inside a Rust application or build script. `protoc` can remain a compatibility reference or provide built-in generators, even where Buf performs day-to-day orchestration.

Primary references:

- `protox` documentation: [https://docs.rs/protox/latest/protox/](https://docs.rs/protox/latest/protox/?utm_source=chatgpt.com)
- Protocol Buffers compiler repository: [https://github.com/protocolbuffers/protobuf](https://github.com/protocolbuffers/protobuf?utm_source=chatgpt.com)
- Buf CLI repository: [https://github.com/bufbuild/buf](https://github.com/bufbuild/buf?utm_source=chatgpt.com)
- Buf compiler architecture: [https://buf.build/docs/reference/cli/buf/build/](https://buf.build/docs/reference/cli/buf/build/?utm_source=chatgpt.com)

## 2. `protox`: native Rust compilation

`protox` is a Rust implementation of the Protobuf compiler intended primarily for integration with `prost-build` and other descriptor consumers. Its output is a compiled schema, not generated Rust source. It avoids invoking an external `protoc` process, while leaving Rust type generation to a separate library.

### 2.1 Compiler API and compilation model

The principal API is `protox::Compiler`, with a convenience function for one-shot compilation.

| API                                | Capability                                   | Important distinction                                          |
| ---------------------------------- | -------------------------------------------- | -------------------------------------------------------------- |
| `protox::compile(files, includes)` | Compile specified `.proto` files             | Returns `prost_types::FileDescriptorSet`                       |
| `Compiler::new(include_paths)`     | Establish compiler and import roots          | Includes built-in resolution for Google well-known descriptors |
| `open_file`, `open_files`          | Parse and resolve explicit input files       | Imports are resolved transitively                              |
| `include_imports(bool)`            | Include transitive imports in output         | Defaults to `false`                                            |
| `include_source_info(bool)`        | Retain source locations and comments         | Defaults to `false`                                            |
| `file_descriptor_set()`            | Retrieve compiled descriptors as Rust values | Files are in dependency-first topological order                |
| `encode_file_descriptor_set()`     | Serialize the descriptor set                 | Preserves encoded custom extension options                     |
| `descriptor_pool()`                | Retrieve resolved reflective descriptors     | Provides the complete referenced descriptor graph              |
| `files()`                          | Inspect files and their metadata             | Useful for dependency awareness and build invalidation         |

The distinction between the two descriptor-output methods is significant. `prost_types` does not generally represent arbitrary custom option extensions as native typed fields. Consequently, converting a reflective descriptor graph into an ordinary `FileDescriptorSet` and re-encoding it can discard information. `encode_file_descriptor_set()` explicitly preserves extension options in the serialized output.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Direct reference: [https://docs.rs/protox/0.10.0/protox/struct.Compiler.html](https://docs.rs/protox/0.10.0/protox/struct.Compiler.html?utm_source=chatgpt.com)

### 2.2 Extensible import resolution

One of `protox`'s more valuable features is its `FileResolver` abstraction. Import resolution does not have to be restricted to a directory on disk.

The `protox::file` module provides:

| Resolver                    | Function                                                  |
| --------------------------- | --------------------------------------------------------- |
| `IncludeFileResolver`       | Resolve sources under filesystem include roots            |
| `GoogleFileResolver`        | Resolve bundled Google Protobuf definitions               |
| `DescriptorSetFileResolver` | Resolve definitions from already-compiled descriptor sets |
| `ChainFileResolver`         | Combine multiple resolution sources                       |
| `FileResolver` trait        | Implement custom source lookup                            |

This creates an important architectural possibility: a compiler can resolve one schema from local `.proto` files, another from precompiled dependencies, and others from a caller-defined source. It can therefore participate in a schema registry, embedded compiler, or schema-analysis service without requiring every dependency to exist as a physical `.proto` file.

`DescriptorSetFileResolver::decode()` preserves custom option extensions from serialized descriptor bytes, while construction from an ordinary typed `FileDescriptorSet` does not offer the same preservation guarantee. Custom resolvers must also provide stable logical file names and consistent dependency resolution; chaining resolvers alone does not resolve conflicting definitions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Direct references:

- [https://docs.rs/protox/0.10.0/protox/file/](https://docs.rs/protox/0.10.0/protox/file/?utm_source=chatgpt.com)
- [https://docs.rs/protox/0.10.0/protox/file/trait.FileResolver.html](https://docs.rs/protox/0.10.0/protox/file/trait.FileResolver.html?utm_source=chatgpt.com)
- [https://docs.rs/protox/0.10.0/protox/file/struct.DescriptorSetFileResolver.html](https://docs.rs/protox/0.10.0/protox/file/struct.DescriptorSetFileResolver.html?utm_source=chatgpt.com)

### 2.3 Rust code generation through `prost-build`

A compact Cargo `build.rs` integration can compile `.proto` source using `protox` and then hand the descriptors directly to Prost:

```
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptors = protox::Compiler::new(["proto"])?
        .include_source_info(true)
        .open_file("proto/example/v1/example.proto")?
        .file_descriptor_set();

    prost_build::Config::new()
        .compile_fds(descriptors)?;

    Ok(())
}
```

This is an illustrative build script, not an executed example. The schema path is a placeholder. `include_source_info(true)` retains comments and locations that can matter for generated documentation; transitive imports remain excluded from the returned descriptor set unless explicitly requested.

For gRPC services, the corresponding modern generator is `tonic-prost-build`, whose `configure().compile_fds(...)` method consumes a `prost_types::FileDescriptorSet`. This is distinct from `tonic-build`, which is responsible for service-code generation infrastructure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



The separation enables programmatic inspection of the schema before generation, including selecting code-generation attributes based on message types or service method descriptors. `prost-build::Config` also supports `extern_path`, `bytes`, `btree_map`, `type_attribute`, `field_attribute`, custom service generators, and other controls. None requires modifying the original `.proto` definitions.

Direct references:

- [https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html?utm_source=chatgpt.com)
- [https://docs.rs/tonic-prost-build/0.14.6/tonic_prost_build/struct.Builder.html](https://docs.rs/tonic-prost-build/0.14.6/tonic_prost_build/struct.Builder.html?utm_source=chatgpt.com)

### 2.4 Diagnostics, runtime properties, and limits

`protox` uses `miette` diagnostics for errors such as unresolved symbols, syntax failures, and import-resolution problems. The parser component, `protox-parse`, can parse an individual file into a descriptor-shaped representation, but parsing alone is not equivalent to compiling and linking the complete schema.

The compiler is not `Send` or `Sync`; sharing a single mutable compiler instance across concurrent threads is therefore not a supported usage pattern. Independent compiler instances or a separately designed coordination layer are different choices. Version 0.10.0 requires Rust 1.85 or newer.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Boundary: `protox` does not provide Buf's workspace-level dependency locking, policy checks, breaking-change engine, or standard code-generation plugin orchestration. It is strongest when compilation and descriptor inspection need to be controlled from Rust itself.

## 3. `protoc`: the reference compiler and plugin execution contract

`protoc` is both Google's reference schema compiler and an executable host for code generators. Its role in Rust is broader than simply generating code: it establishes the standard descriptor and plugin interchange formats shared by the ecosystem.

### 3.1 Descriptor-set compilation

A conventional descriptor build is:

```
protoc \
  -I proto \
  --include_imports \
  --include_source_info \
  --descriptor_set_out=schema.binpb \
  proto/example/v1/example.proto
```

`--descriptor_set_out` produces a serialized `google.protobuf.FileDescriptorSet`. Without the two inclusion flags, an output set ordinarily contains the explicitly selected files but omits imported file descriptors and source-code information.

Including imports produces a more self-contained descriptor graph. Including source information preserves declaration positions and comments, valuable for documentation generation, precise diagnostics, and source-aware analysis, at the cost of a larger artifact.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



Other important compiler interfaces include `--proto_path`/`-I` for resolving logical import paths and `--descriptor_set_in` for compiling or generating against previously built descriptors. The latter allows Buf-compiled descriptors to be consumed by `protoc` without repeating its normal source compilation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



### 3.2 The standard code-generation plugin protocol

For a command such as:

```
protoc \
  -I proto \
  --prost_out=generated/rust \
  proto/example/v1/example.proto
```

`protoc` discovers `protoc-gen-prost`, invokes it as a subprocess, and exchanges two binary Protobuf messages:

| Protocol object         | Major contents                                                                              |
| ----------------------- | ------------------------------------------------------------------------------------------- |
| `CodeGeneratorRequest`  | `file_to_generate`, `parameter`, `proto_file`, compiler/version information                 |
| `CodeGeneratorResponse` | Generated file names and contents, errors, optional insertion points and supported features |

The request arrives on the plugin's standard input. Its response is serialized to standard output. The compiler, not normally the plugin, takes responsibility for materializing the generated files. Plugin options are encoded into the request's `parameter` string.

This contract is the reason a Rust generator can be invoked by either `protoc` or Buf without being rewritten for each host. It also permits custom generators for code, schemas, metadata, documentation, validation artifacts, or other outputs, provided they implement the contract.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



The full protocol is defined here:

[https://github.com/protocolbuffers/protobuf/blob/main/src/google/protobuf/compiler/plugin.proto](https://github.com/protocolbuffers/protobuf/blob/main/src/google/protobuf/compiler/plugin.proto?utm_source=chatgpt.com)

### 3.3 Important plugin semantics

Plugins must distinguish files selected for generation from descriptors supplied for type resolution. `file_to_generate` identifies the former; the request's descriptor collection supplies schema context, including imports. Treating every supplied descriptor as a requested output can generate unwanted code for dependencies.

`CodeGeneratorResponse` also supports insertion points, generated-code annotations, and capability declarations such as support for proto3 optional fields and editions. These are protocol mechanisms rather than automatic guarantees: a plugin must correctly implement the corresponding language semantics before advertising them.

An important Rust-specific distinction is that Google's official Rust Protobuf implementation and Prost are different code-generation/runtime ecosystems. Google's Rust implementation uses kernel-backed message APIs and has its own generation and version compatibility expectations. Generated output is not interchangeable with Prost merely because both originate from the same `.proto` files.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation+1



Direct references:

- [https://protobuf.dev/reference/rust/rust-generated/](https://protobuf.dev/reference/rust/rust-generated/?utm_source=chatgpt.com)
- [https://protobuf.dev/reference/rust/building-rust-protos/](https://protobuf.dev/reference/rust/building-rust-protos/?utm_source=chatgpt.com)
- [https://github.com/protocolbuffers/protobuf/blob/main/src/google/protobuf/compiler/plugin.proto](https://github.com/protocolbuffers/protobuf/blob/main/src/google/protobuf/compiler/plugin.proto?utm_source=chatgpt.com)

## 4. Buf: module-aware compilation and orchestration

Buf combines its own Protobuf compiler with a workspace model, dependency resolution, schema validation, and plugin execution. Its central contribution is making the same declared schema inputs available consistently to compilation, code generation, linting, and compatibility analysis.

Buf's normal local compilation path does not invoke `protoc`; it uses its internal compiler. The project tests descriptor equivalence against `protoc`, while documenting some differences in `SourceCodeInfo` and unknown/custom option encodings. Accordingly, semantic interchangeability is a reasonable target, but unconditional byte-for-byte identity is not.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



### 4.1 Workspace, modules, and dependencies

Buf's v2 configuration separates the workspace from its constituent modules. A workspace can contain multiple modules, each with its own logical import root and optionally its own lint or breaking-change configuration.

For example:

```
# buf.yaml
version: v2

modules:
  - path: proto

deps:
  - buf.build/googleapis/googleapis

lint:
  use:
    - STANDARD

breaking:
  use:
    - FILE
```

`path: proto` establishes a module root; a file at `proto/acme/example/v1/service.proto` is addressed by its logical import path relative to that root, not by including the leading `proto/` directory.

The dependency model has two layers. Modules within one workspace can resolve each other without explicit external dependency declarations. Dependencies published to the Buf Schema Registry (BSR) are declared under `deps`, and their resolved commits and content digests are recorded in `buf.lock`, including transitive dependencies. `buf dep update`, `buf dep prune`, and `buf dep graph` manage that state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



This is fundamentally different from `protoc` include paths: the lockfile makes externally resolved schema versions explicit, while the workspace provides a shared namespace for all Buf operations. It does not, however, pin locally installed code-generation binaries; those have their own version-management requirements.

Direct references:

- [https://buf.build/docs/configuration/v2/buf-yaml/](https://buf.build/docs/configuration/v2/buf-yaml/?utm_source=chatgpt.com)
- [https://buf.build/docs/configuration/v2/buf-lock/](https://buf.build/docs/configuration/v2/buf-lock/?utm_source=chatgpt.com)
- [https://buf.build/docs/cli/modules-workspaces/](https://buf.build/docs/cli/modules-workspaces/?utm_source=chatgpt.com)

### 4.2 Important local commands

| Command                         | Capability                                              | Engineering significance                                         |
| ------------------------------- | ------------------------------------------------------- | ---------------------------------------------------------------- |
| `buf build`                     | Compile source or transform image inputs                | Produces reusable descriptor artifacts                           |
| `buf generate`                  | Execute configured generators                           | Separates generation policy from command-line scripts            |
| `buf lint`                      | Enforce schema rules                                    | Provides structured diagnostics independently of code generation |
| `buf breaking --against ...`    | Compare schema versions                                 | Captures compatibility rules at multiple semantic levels         |
| `buf format -w`                 | Canonicalize `.proto` source                            | Consistent, deterministic formatting                             |
| `buf format --diff --exit-code` | Detect formatting changes                               | Useful for automated checks                                      |
| `buf dep update`                | Resolve external dependencies                           | Updates `buf.lock`                                               |
| `buf dep graph`                 | Inspect dependency graph                                | Reveals external module relationships                            |
| `buf export`                    | Materialize `.proto` source files                       | Bridges registry or Git inputs into filesystem-based tools       |
| `buf config ls-modules`         | Inspect workspace modules                               | Makes the resolved module model observable                       |
| `buf config ls-lint-rules`      | Inspect available lint rules                            | Discovers rule sets and plugin-provided checks                   |
| `buf stats`                     | Summarize schema statistics                             | Lightweight inspection of schema scope                           |
| `buf lsp serve`                 | Run a Protobuf language server                          | Enables editor navigation, completion, rename, and diagnostics   |
| `buf convert`                   | Convert encoded messages between binary, text, and JSON | Schema-aware payload inspection                                  |
| `buf curl`                      | Invoke RPC endpoints using schema information           | Supports API investigation without generated application clients |

`buf lsp serve` is particularly useful outside conventional code generation: it exposes definitions, references, workspace symbols, import organization, formatting, and lint diagnostics to an LSP-aware editor. Its focus is `.proto` language intelligence, rather than analysis of the Rust code subsequently generated from those definitions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+2



For Rust tools that need to invoke Buf, these operations are naturally consumed through subprocess execution. A caller can use the CLI's exit status, standard error, JSON diagnostic formats where supported, and binary descriptor outputs. Unlike `protox`, Buf does not expose its principal compiler/orchestrator as a native Rust library API.

CLI reference: [https://buf.build/docs/reference/cli/](https://buf.build/docs/reference/cli/?utm_source=chatgpt.com)

### 4.3 Breaking-change detection is richer than wire compatibility

Buf provides four principal breaking-change categories:

| Category    | Protects against                                         |
| ----------- | -------------------------------------------------------- |
| `FILE`      | Generated-source breakage, including file-level identity |
| `PACKAGE`   | Generated-source breakage at package granularity         |
| `WIRE_JSON` | Binary wire and Protobuf JSON compatibility              |
| `WIRE`      | Binary wire compatibility                                |

These are ordered from strictest to most permissive. `FILE` is the default. Moving a declaration between files may affect languages with file-specific generated artifacts even where package and wire semantics remain unchanged. Conversely, some field-name changes can preserve binary compatibility while disrupting Protobuf JSON or generated application interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



For Rust, the consequential distinction is between wire compatibility and generated-code source compatibility. A wire-safe change can still require modifying consumers of generated Rust types. Buf's rules provide a practical safeguard, but cannot prove that all custom generators, reflection consumers, or application-level invariants remain compatible.

Direct reference: [https://buf.build/docs/breaking/rules/](https://buf.build/docs/breaking/rules/?utm_source=chatgpt.com)

## 5. Buf images and `FileDescriptorSet` interoperability

The descriptor artifact is the strongest interoperability point among all three compiler approaches.

### 5.1 What an image contains

A standard `FileDescriptorSet` contains `FileDescriptorProto` entries describing files, packages, imports, messages, fields, enums, services, options, and potentially source-code locations and comments.

A Buf image extends that representation with metadata that Buf uses for module-aware operations. Buf documents an `ImageFileExtension` attached using field number `8042`, with information including:

- Whether a file is an imported dependency.
- The Buf module to which the file belongs.
- Whether the source omitted an explicit syntax declaration.
- Which imported dependencies are unused.

These are additions to the descriptor representation, not new Protobuf source-language constructs. They must also be distinguished from custom Protobuf options, which are a separate extension mechanism.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



### 5.2 Image construction and output variants

```
# Buf image with Buf-specific metadata
buf build -o schema.image.binpb

# Standard FileDescriptorSet
buf build \
  --as-file-descriptor-set \
  -o schema.binpb

# Smaller runtime-oriented descriptor artifact
buf build \
  --as-file-descriptor-set \
  --exclude-source-info \
  -o schema.runtime.binpb

# Generate Rust code from an existing descriptor artifact
buf generate schema.binpb --template buf.gen.yaml
```

These commands are illustrative; a real workspace must have appropriate configuration and the generation template must declare usable plugins.

Important `buf build` controls include `--exclude-imports`, `--exclude-source-info`, `--exclude-source-retention-options`, `--path`, `--exclude-path`, and `--type`. Output formats include binary Protobuf (`binpb`), JSON, TextProto, and YAML. Type selection can create a descriptor view limited to relevant symbols and their required dependencies, useful for more focused generation or inspection.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+2



Source-retention options deserve particular attention. A descriptor used as a runtime contract may intentionally omit source-only metadata, whereas a descriptor used for documentation or source-aware code generation may need it. An output optimized for one purpose is not necessarily lossless for another.

### 5.3 Which information survives conversion?

| Transformation                                                            | Preserved                                | Potentially lost                                   |
| ------------------------------------------------------------------------- | ---------------------------------------- | -------------------------------------------------- |
| Buf image → standard `FileDescriptorSet` using `--as-file-descriptor-set` | Standard schema descriptors              | Buf-specific module and import metadata            |
| `protoc` descriptors → Buf image input                                    | Standard compiled schema                 | Buf metadata never present in the original         |
| Binary descriptors → ordinary `prost_types::FileDescriptorSet` → encode   | Known descriptor fields                  | Unknown fields and custom option extension bytes   |
| Binary descriptors → `prost_reflect::DescriptorPool::decode()`            | Descriptors and extension options        | Buf-specific metadata is not guaranteed to survive |
| Descriptors with `SourceCodeInfo` → stripped output                       | Schema needed for many runtime consumers | Source spans and comments                          |

The Protobuf wire format permits an implementation to decode an extended message even when it does not understand its additional fields. That is not a guarantee that decoding and re-encoding through a particular typed Rust struct will preserve those unknown fields. Buf therefore provides `--as-file-descriptor-set` specifically for consumers that only need the native Protobuf representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



This matters when treating descriptor artifacts as canonical data: if byte-level preservation of custom option data or Buf-specific metadata matters, retain the original serialized artifact or use an API with a documented preservation guarantee.

### 5.4 Descriptor consumers in Rust

The compiled descriptor can directly support Rust functionality beyond static code generation.

`prost-reflect::DescriptorPool` can decode descriptor sets, resolve messages and services, and provide runtime reflective access. `DynamicMessage` extends this with schema-driven message instances. This can support dynamic message inspection, schema-aware serialization, configuration systems, RPC tooling, and descriptor-driven extension behavior without generating static Rust types for every schema.

For example:

```
use prost_reflect::DescriptorPool;

let bytes = std::fs::read("schema.binpb")?;
let pool = DescriptorPool::decode(bytes.as_slice())?;

let message = pool.get_message_by_name("acme.example.v1.Request");
```

`DescriptorPool::decode()` preserves custom option extensions from the serialized descriptor input, making it particularly relevant where custom annotations are part of the schema's meaning. The example assumes the descriptor contains the named message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Direct references:

- [https://buf.build/docs/reference/images/](https://buf.build/docs/reference/images/?utm_source=chatgpt.com)
- [https://buf.build/docs/reference/cli/buf/build/](https://buf.build/docs/reference/cli/buf/build/?utm_source=chatgpt.com)
- [https://docs.rs/prost-reflect/latest/prost_reflect/struct.DescriptorPool.html](https://docs.rs/prost-reflect/latest/prost_reflect/struct.DescriptorPool.html?utm_source=chatgpt.com)

An image is not source code. Descriptor sets contain a compiled representation and optional source locations/comments; they do not generally reconstruct the original `.proto` text faithfully. `buf export` instead operates on source/module inputs and materializes actual `.proto` files, including dependencies unless excluded.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



## 6. Buf code-generation orchestration for Rust

### 6.1 `buf.gen.yaml`: generators as declared build steps

Buf v2 code-generation configuration supports three principal plugin modes.

| Mode             | Mechanism                                           | Principal condition                         |
| ---------------- | --------------------------------------------------- | ------------------------------------------- |
| `local`          | Execute a locally installed `protoc-gen-*` plugin   | Binary or command must be available locally |
| `remote`         | Execute a published generator through the BSR       | Registry access and remote execution        |
| `protoc_builtin` | Invoke a generator built into a `protoc` executable | Compatible `protoc` executable required     |

For Rust/Prost generation, a minimal local template is:

```
# buf.gen.yaml
version: v2

plugins:
  - local: protoc-gen-prost
    out: gen/rust

  - local: protoc-gen-tonic
    out: gen/rust
```

Here `protoc-gen-prost` generates message types, while `protoc-gen-tonic` generates Tonic service clients and servers. The latter is designed to follow the former in the generation sequence, and certain shared options, notably `extern_path` or well-known-type handling, must remain consistent across both.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



The Rust plugins in this example are separate from `protox`; they can be driven using either `protoc` or Buf.

### 6.2 Rust generator ecosystem

The `protoc-gen-prost` family offers additional functionality beyond generating message definitions:

| Plugin                   | Function                                                                                              |
| ------------------------ | ----------------------------------------------------------------------------------------------------- |
| `protoc-gen-prost`       | Generate Prost Rust messages and enums                                                                |
| `protoc-gen-tonic`       | Generate Tonic clients and servers                                                                    |
| `protoc-gen-prost-serde` | Extend generated definitions with Serde-related implementations                                       |
| `protoc-gen-prost-crate` | Generate module include files, Cargo-crate scaffolding, and optional feature dependency relationships |

`protoc-gen-prost` supports numerous Prost generation controls, including `bytes`, `btree_map`, `boxed`, `extern_path`, `type_attribute`, `field_attribute`, well-known-type handling, and optional embedded descriptor sets. Its `file_descriptor_set` option generates scoped descriptor data for individual modules and preserves raw extension information rather than blindly passing descriptors through a lossy typed conversion.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`protoc-gen-prost-crate` is especially interesting for modular Rust output: it can generate `mod.rs` and Cargo feature relationships from the schema package graph. It requires `strategy: all`, because building those relationships requires visibility across the full set of generation targets. Some template-based crate generation options are not compatible with remote execution because the remote generator cannot see the local template filesystem.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Direct references:

- [https://docs.rs/protoc-gen-prost/0.5.0/protoc_gen_prost/](https://docs.rs/protoc-gen-prost/0.5.0/protoc_gen_prost/?utm_source=chatgpt.com)
- [https://docs.rs/protoc-gen-tonic/0.5.0/protoc_gen_tonic/](https://docs.rs/protoc-gen-tonic/0.5.0/protoc_gen_tonic/?utm_source=chatgpt.com)
- [https://docs.rs/protoc-gen-prost-crate/0.5.0/protoc_gen_prost_crate/](https://docs.rs/protoc-gen-prost-crate/0.5.0/protoc_gen_prost_crate/?utm_source=chatgpt.com)

### 6.3 Invocation strategy: `directory` versus `all`

This is a consequential and easily overlooked Buf setting.

`strategy: directory`, the default for local plugins, groups input files by directory and invokes plugin processes in parallel. For ordinary file-oriented generators, it reduces unnecessary work and allows parallelism.

`strategy: all` passes all generation targets for one input to a single invocation. It is appropriate for generators that need a global view, such as a generator producing a single schema index, aggregate metadata, Cargo feature graph, or consolidated output.

Remote plugins use the `all` strategy. Multiple generation inputs may still have distinct invocation boundaries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



This setting affects correctness, not just performance. A custom Rust plugin that writes the same output filename for every invocation can conflict under `directory` mode. Conversely, forcing a very large schema through one process can increase peak memory consumption if the plugin does not need global knowledge.

### 6.4 Target selection and generated imports

`buf generate` distinguishes the schema needed for compilation from the schema selected for generated outputs.

By default, imported dependencies are available to the generator for type resolution but are not all generated as independent output targets. `--include-imports` broadens those targets, while `--include-wkt` adds the well-known types and requires imported-file generation to be enabled. `--path`, `--type`, `--exclude-path`, and `--exclude-types` can further narrow the generated surface.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



This matters when Rust crates already provide shared Protobuf types. Generating additional copies of those message types can create incompatible Rust type identities even though their schemas describe equivalent wire messages. Prost's `extern_path` provides a way to map a Protobuf type or package to an existing Rust implementation, avoiding duplicate generated definitions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.5 Managed mode and output lifecycle

Buf managed mode can override selected language-specific Protobuf options during generation without editing `.proto` files. Examples include Go package paths and Java or C# naming options. These overrides affect the descriptors presented to the generators; they do not inherently rewrite Rust module layouts or provide a general Rust-specific package-mapping system.

The `clean: true` setting removes configured output directories or archives before generation. That is useful for preventing obsolete generated files from surviving schema removals, but the output locations must be dedicated to generated artifacts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



Direct reference: [https://buf.build/docs/configuration/v2/buf-gen-yaml/](https://buf.build/docs/configuration/v2/buf-gen-yaml/?utm_source=chatgpt.com)

### 6.6 Local versus remote execution

Local plugins run where Buf is invoked, using locally available executables. Remote plugins execute through the BSR. That removes the need to install individual generator binaries locally but introduces network availability, published plugin versioning, execution sandbox limits, and potential sensitivity around sending schema data to an external service.

A remote plugin reference can specify an upstream version and a Buf packaging revision. Omitting a version means resolving the latest published version, which is not equivalent to a fixed generator toolchain.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



A noteworthy distinction: `protoc_builtin` in Buf's documented supported list covers generators such as C++, Java, Python, and others, but does not list Rust. Although Google's official toolchain documents `protoc --rust_out`, Buf users should not assume that `protoc_builtin: rust` is a supported configuration without verifying that particular Buf/protoc combination.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



## 7. Extending the compiler workflow with Rust plugins

### 7.1 Implementing a standard `protoc` plugin in Rust

A custom generator need not parse `.proto` itself. It can implement the standard request/response protocol using `prost` and `prost-types`.

The following is a minimal conceptual generator that emits a count of explicitly selected input files:

```
use prost::Message;
use prost_types::compiler::{
    code_generator_response,
    CodeGeneratorRequest,
    CodeGeneratorResponse,
};
use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;

    let request = CodeGeneratorRequest::decode(input.as_slice())?;

    let mut response = CodeGeneratorResponse::default();
    response.file.push(code_generator_response::File {
        name: Some("schema_index.rs".into()),
        content: Some(format!(
            "pub const FILE_COUNT: usize = {};\n",
            request.file_to_generate.len()
        )),
        ..Default::default()
    });

    io::stdout().write_all(&response.encode_to_vec())?;
    Ok(())
}
```

This is a protocol illustration, not a feature-complete generator. It should be run with `strategy: all` if a single consolidated output is desired. It deliberately does not advertise proto3-optional or editions support; a production generator must correctly implement the relevant semantics and protocol feature declarations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



For real generators, `request.proto_file` provides the descriptor graph, while `request.file_to_generate` identifies the output targets. `CodeGeneratorResponse` supports generated files, insertion-point contributions, error reporting, and other metadata. A plugin can return a schema-generation error in the response instead of terminating abnormally; malformed input or a plugin infrastructure failure belongs on standard error and a failing process exit.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



### 7.2 Insertion points versus independent outputs

Insertion points allow one generator to extend another generator's output without rewriting the entire source file. The primary generator inserts markers such as `@@protoc_insertion_point(...)`; later plugins return additional content associated with those markers.

`protoc-gen-prost` exposes insertion points used by companion generators such as `protoc-gen-prost-serde`. Buf processes plugin results and insertion points in configured order, even though plugin invocations can be parallelized internally. This offers a structured alternative to post-generation text replacement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



However, insertion points are cooperation contracts between particular generators. They are not general syntax-aware Rust codemods. A generator that does not expose a suitable insertion point may be better extended through its supported attributes, service-generator hooks, or a separate Rust syntax transformation.

### 7.3 Three distinct meanings of “plugin” in Buf

This terminology is worth separating explicitly:

| Extension mechanism                                 | Input/output protocol                            | Purpose                                                   |
| --------------------------------------------------- | ------------------------------------------------ | --------------------------------------------------------- |
| Standard Protobuf code generator                    | `CodeGeneratorRequest` → `CodeGeneratorResponse` | Generate Rust or other artifacts                          |
| `protoc-gen-buf-lint` and `protoc-gen-buf-breaking` | Standard Protobuf plugin interface               | Run Buf checks inside an existing `protoc`-based workflow |
| Buf check plugin                                    | Bufplugin API over PluginRPC                     | Add custom rules to `buf lint` or `buf breaking`          |

The third mechanism is not interchangeable with a code-generation plugin. Buf check plugins are configured under `plugins` in `buf.yaml`, and their rules are selected under `lint.use` or `breaking.use`. They may be local executables, local Wasm modules, or BSR-published plugins.

Buf provides a documented Go SDK for check-plugin authors, while authors in other languages can implement the underlying Bufplugin and PluginRPC specifications. Thus custom Rust check plugins are an integration possibility, but should not be confused with implementing the considerably simpler `CodeGeneratorRequest` protocol. Buf's check-plugin authoring framework is still evolving.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+1



Direct references:

- [https://buf.build/docs/bsr/checks/plugins/](https://buf.build/docs/bsr/checks/plugins/?utm_source=chatgpt.com)
- [https://buf.build/docs/cli/protoc-plugins/](https://buf.build/docs/cli/protoc-plugins/?utm_source=chatgpt.com)
- [https://github.com/bufbuild/bufplugin](https://github.com/bufbuild/bufplugin?utm_source=chatgpt.com)

## 8. Cross-tool compositions and their tradeoffs

The descriptor boundary enables several useful architectures without requiring custom parsers or competing schema models.

| Composition                                   | Main benefit                                                  | Principal limitation                                                |
| --------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------- |
| `protox` → `prost-build::compile_fds`         | All-Rust compilation and generation within Cargo              | Does not inherit Buf's workspace governance and dependency locking  |
| `protox` → `tonic-prost-build`                | All-Rust Protobuf compilation plus gRPC stubs                 | Must verify compiler and generator language-feature compatibility   |
| `protoc` → `protoc-gen-prost`                 | Reference compiler semantics with Rust code generation        | Requires external binaries and explicit import/plugin orchestration |
| Buf → local Prost/Tonic plugins               | Workspace-wide compilation and generation configuration       | Requires separately managed local plugin executables                |
| Buf → native descriptor set → `prost-reflect` | Reusable compiled schema for dynamic Rust consumers           | Needs imported descriptors and appropriate preservation of options  |
| Buf image → `protoc --descriptor_set_in`      | Buf compilation with reference-compiler generators            | Built-in generator/runtime requirements still apply                 |
| Buf source workspace + in-process `protox`    | Shared schema governance and Rust-native embedded compilation | Two compilation implementations must be kept semantically aligned   |

### 8.1 A common compiled-schema boundary

One particularly powerful composition is to produce descriptors once and allow several independent consumers to use them.

For example, the same compiled schema can supply a static Rust generator, a dynamic reflection engine, a service descriptor index, and another schema-aware tool. These consumers do not each need to parse `.proto`, locate imports, and resolve type names independently.

That is a general architectural opportunity rather than an automatic property of every generator: some build APIs only accept source files, some accept `FileDescriptorSet`, and others require a `CodeGeneratorRequest`. Converting among these forms may require constructing target-file lists, preserving options, or implementing the standard plugin host.

### 8.2 Choosing where compilation happens

`protox` is attractive when an application needs to invoke a compiler as part of normal Rust execution: dynamically accepting schema files, providing immediate diagnostics, using custom import resolvers, or inspecting descriptor objects without crossing a process boundary.

Buf is attractive when the problem is larger than compilation: many modules, shared external dependencies, multiple language generators, compatibility guarantees, and consistent local or CI behavior.

`protoc` remains relevant when exact reference behavior, a particular compiler release, or a built-in language generator matters. It also offers a useful compatibility baseline when adopting an alternative compiler.

These approaches can coexist, but doing so introduces responsibility for verifying language-version support and descriptor equivalence where it matters.

## 9. Consequential compatibility and correctness constraints

### 9.1 Protobuf Editions support is not inherited transitively

As of October 31, 2026, `protoc` 36.x supports edition 2026, and Buf 1.75.0 added edition 2026 support. Yet the supported edition of a `.proto` file and the capabilities of a Rust code generator are separate facts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protobuf.dev\&sz=32)

Protocol Buffers Documentation+1



In particular, Prost's upstream issue to add editions support remains open, and its published documentation describes proto2/proto3 generation. A successful schema compilation by Buf or `protoc` therefore does not establish that a Prost generator can consume every resulting descriptor.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



An additional implementation detail: the published `prost-types` 0.14.4 representation of `CodeGeneratorResponse` exposes proto3-optional support but does not expose the newer editions-related response fields. A custom Rust plugin targeting modern editions may therefore require updated protocol types or deliberate handling of those fields rather than assuming `prost_types::compiler` exposes the complete latest protocol.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



The current `protox` documentation does not establish full Protobuf Editions 2023/2024/2026 compatibility. Until verified against its parser and compiler implementation, that support should be considered unconfirmed rather than inferred from the age of its release.

### 9.2 Compiler, generator, and runtime versions are different concerns

There are several independent version axes:

| Layer             | Examples                                                   | Compatibility question                                              |
| ----------------- | ---------------------------------------------------------- | ------------------------------------------------------------------- |
| Compiler          | `protox`, `protoc`, Buf internal compiler                  | Can it parse and resolve the schema?                                |
| Descriptor schema | `google.protobuf.FileDescriptorProto`, `FileDescriptorSet` | Can downstream tooling represent the required options and features? |
| Code generator    | `prost-build`, `protoc-gen-prost`, Google's Rust generator | Can it generate semantically correct Rust code?                     |
| Runtime           | `prost`, `google-protobuf`, Tonic                          | Can it compile and execute the generated code?                      |
| Plugin protocol   | `plugin.proto`                                             | Does the plugin advertise and implement required features?          |

The official Google Rust Protobuf implementation specifically documents the need for its `protoc` generator version to match the corresponding runtime release version. Prost uses a different toolchain with its own compatibility expectations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 9.3 File identity and import closure matter

Protobuf file names are logical import identities, not merely filesystem locations. A file found at two different physical paths with the same logical name may represent a duplicate or conflicting definition. A schema that compiles with an accidental include-path ordering can therefore behave differently when imported into a properly declared Buf workspace.

Similarly, a descriptor set containing only root files may be sufficient for some generators but insufficient for runtime descriptor-pool construction. A self-contained reflective descriptor graph ordinarily needs the imported definitions.

`buf build`, `protoc`, and `protox` all support descriptor selection, but their defaults and mechanisms differ. These settings must be chosen for the eventual consumer rather than treated as a purely storage-size optimization.

### 9.4 Performance and reproducibility

Buf's internal compiler uses parallel parsing. Buf has published an example comparing approximately 0.9 seconds for its compiler with 1.6 seconds for `protoc` 21.7 on a 2022 M1 Max machine compiling 3,944 Google APIs files. This is an upstream historical workload, not a benchmark of the current 1.75.0 release against `protoc` 36.2, and not a prediction for Rust build times.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs



The performance-relevant mechanisms are more general. `protox` avoids process startup and serialization between an external compiler and an in-process Rust caller. Buf supports parallel compilation and per-directory parallel plugin execution. Precompiled descriptor sets can avoid repeated source parsing and dependency resolution.

However, large descriptor sets have serialization and memory costs, custom generators can dominate end-to-end runtime, and `strategy: all` may trade fewer invocations for larger per-process working sets. None of these mechanisms establishes universal performance superiority.

Reproducibility also extends beyond the schema lockfile. The Buf CLI version, local or remote generator versions, generator options, and relevant Rust runtime dependencies collectively determine whether regenerated output stays stable.

## 10. Consolidated architectural assessment

The most useful general model is to treat these tools as occupying three complementary layers:

`protox` is the embedded Rust compiler layer. It provides precise in-process compilation control, extensible import resolution, structured diagnostics, and direct access to descriptor graphs. Its advanced value is reducing custom source-loading, parsing, linking, and descriptor construction code inside Rust applications.

`protoc` is the reference compiler and interchange contract. Its descriptor format and plugin protocol give the ecosystem a stable way to connect independently implemented schema compilers and generators. Its advanced value is compatibility and broad tooling interoperability, not workspace-level lifecycle management.

Buf is the schema-management and orchestration layer. It combines module-aware compilation, locked dependencies, source formatting, linting, breaking-change analysis, descriptor images, and local or remote code generation. Its advanced value is eliminating bespoke coordination around compiler inputs, schema versioning, generators, and compatibility policies.

The most consequential shared primitive is the compiled descriptor graph. It lets schema analysis, reflection, code generation, and interoperability tooling operate on resolved Protobuf semantics rather than independently interpreting source files.

For Rust implementations, the key questions are therefore not simply which compiler to use, but where schema compilation should occur, whether descriptor artifacts must preserve full semantic metadata, whether runtime reflection is needed, and whether generation is an in-process Rust operation or a workspace-managed build operation.

## Primary reference map

| Subject                          | Direct source                                                                                                                                                                                                             |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `protox` APIs                    | [https://docs.rs/protox/0.10.0/protox/](https://docs.rs/protox/0.10.0/protox/?utm_source=chatgpt.com)                                                                                                                     |
| `protox` file resolution         | [https://docs.rs/protox/0.10.0/protox/file/](https://docs.rs/protox/0.10.0/protox/file/?utm_source=chatgpt.com)                                                                                                           |
| Prost generation                 | [https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html](https://docs.rs/prost-build/0.14.4/prost_build/struct.Config.html?utm_source=chatgpt.com)                                                             |
| Tonic Prost generation           | [https://docs.rs/tonic-prost-build/0.14.6/tonic_prost_build/](https://docs.rs/tonic-prost-build/0.14.6/tonic_prost_build/?utm_source=chatgpt.com)                                                                         |
| Prost reflection                 | [https://docs.rs/prost-reflect/latest/prost_reflect/](https://docs.rs/prost-reflect/latest/prost_reflect/?utm_source=chatgpt.com)                                                                                         |
| Protobuf plugin protocol         | [https://github.com/protocolbuffers/protobuf/blob/main/src/google/protobuf/compiler/plugin.proto](https://github.com/protocolbuffers/protobuf/blob/main/src/google/protobuf/compiler/plugin.proto?utm_source=chatgpt.com) |
| Protobuf compiler releases       | [https://github.com/protocolbuffers/protobuf/releases](https://github.com/protocolbuffers/protobuf/releases?utm_source=chatgpt.com)                                                                                       |
| Protobuf edition/version support | [https://protobuf.dev/support/version-support/](https://protobuf.dev/support/version-support/?utm_source=chatgpt.com)                                                                                                     |
| Buf CLI reference                | [https://buf.build/docs/reference/cli/](https://buf.build/docs/reference/cli/?utm_source=chatgpt.com)                                                                                                                     |
| Buf workspace configuration      | [https://buf.build/docs/configuration/v2/buf-yaml/](https://buf.build/docs/configuration/v2/buf-yaml/?utm_source=chatgpt.com)                                                                                             |
| Buf generation configuration     | [https://buf.build/docs/configuration/v2/buf-gen-yaml/](https://buf.build/docs/configuration/v2/buf-gen-yaml/?utm_source=chatgpt.com)                                                                                     |
| Buf image representation         | [https://buf.build/docs/reference/images/](https://buf.build/docs/reference/images/?utm_source=chatgpt.com)                                                                                                               |
| Buf internal compiler            | [https://buf.build/docs/reference/internal-compiler/](https://buf.build/docs/reference/internal-compiler/?utm_source=chatgpt.com)                                                                                         |
| Buf breaking-change rules        | [https://buf.build/docs/breaking/rules/](https://buf.build/docs/breaking/rules/?utm_source=chatgpt.com)                                                                                                                   |
| Buf check-plugin architecture    | [https://buf.build/docs/bsr/checks/plugins/](https://buf.build/docs/bsr/checks/plugins/?utm_source=chatgpt.com)                                                                                                           |
| Buf release history              | [https://github.com/bufbuild/buf/blob/main/CHANGELOG.md](https://github.com/bufbuild/buf/blob/main/CHANGELOG.md?utm_source=chatgpt.com)                                                                                   |