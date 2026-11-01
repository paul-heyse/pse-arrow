# Surrealism (Rust) — Comprehensive Library Capability Reference

Research date: October 28, 2026 | Target: Surrealism 0.6.4 | SurrealDB 3.3.4

## 1. Architectural orientation

Surrealism is SurrealDB's native extensibility framework for executing custom Rust functionality inside the database engine through sandboxed WebAssembly components. It enables application-specific computations, domain functions, database interactions, and reusable Rust libraries to become callable from SurrealQL without modifying or recompiling SurrealDB itself.

Surrealism is not an ORM, a general-purpose Rust database client, or merely a foreign-function interface. Its distinguishing capability is embedding programmable behavior into SurrealDB's execution environment, with explicit function signatures, controlled host capabilities, module lifecycle management, and database-level integration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



### 1.1 Principal capabilities

| Capability family                | Native mechanism                                     | Architectural significance                                          |
| -------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------- |
| Database-native functions        | `#[surrealism]` procedural macro                     | Make Rust functions directly callable from SurrealQL                |
| Structured Rust interoperability | `surrealdb-types`, `SurrealValue`                    | Exchange typed database values and application-defined structures   |
| Database interaction             | Guest imports `sql`, `sql_with_vars`, `run`          | Invoke database functionality from within an extension              |
| Module-local state               | `kv` host interface                                  | Maintain state associated with module execution                     |
| Modular organization             | Namespaced exports, default exports, export renaming | Build discoverable, reusable function libraries                     |
| Initialization                   | `#[surrealism(init)]`                                | Initialize module resources and caches                              |
| Resource packaging               | `.surli` archives, attached filesystem               | Distribute WASM, metadata, and supporting assets together           |
| Capability restrictions          | Query, function, scripting, and network permissions  | Constrain an extension's available host operations                  |
| Runtime isolation                | WASI Preview 2, Wasmtime component model             | Execute extensions without native access to database process memory |
| Execution management             | Controller pools, timeouts, resource limits          | Manage concurrent execution and resource consumption                |
| Database integration             | `DEFINE MODULE`, module metadata                     | Register and resolve functions without rebuilding the database      |
| Distribution                     | Database buckets, Silo packages                      | Deploy independently versioned extension artifacts                  |

The distinction is between functions that execute as part of a database operation and conventional external services or clients that execute database requests. Surrealism creates a third option between implementing logic in SurrealQL and moving it entirely into application code.

### 1.2 Version and maturity

The current published stable crate version verified for this reference is `surrealism 0.6.4`, released October 13, 2026. Its associated runtime, macros, and shared-type crates use the same 0.6.4 series. The corresponding SurrealDB stable version is 3.3.4.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



The principal crates are:

| Crate                | Responsibility                                                       |
| -------------------- | -------------------------------------------------------------------- |
| `surrealism`         | Guest-side SDK, exported functions, WIT bindings, host imports       |
| `surrealism-macros`  | Procedural macros and exported-function registration                 |
| `surrealism-types`   | Shared argument-marshalling types, ABI-related errors                |
| `surrealism-runtime` | Host-side Wasmtime execution, module loading, capability enforcement |
| `surrealdb-types`    | Canonical SurrealDB value representation and Rust type conversions   |

Maturity qualification: Surrealism is released and usable, but SurrealDB still gates the functionality behind the `surrealism` experimental capability. The files/buckets mechanism used by many module deployment workflows is separately experimental. Thus, a published stable crate version should not be interpreted as a fully stabilized database extension interface or ABI.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



Primary references:

- Crate releases: [https://docs.rs/crate/surrealism/latest](https://docs.rs/crate/surrealism/latest?utm_source=chatgpt.com)
- Guest SDK: [https://docs.rs/surrealism/0.6.4/surrealism/](https://docs.rs/surrealism/0.6.4/surrealism/?utm_source=chatgpt.com)
- Host runtime: [https://docs.rs/crate/surrealism-runtime/0.6.4](https://docs.rs/crate/surrealism-runtime/0.6.4?utm_source=chatgpt.com)
- Official architecture: [https://surrealdb.com/docs/learn/extensions/guides/module-architecture](https://surrealdb.com/docs/learn/extensions/guides/module-architecture?utm_source=chatgpt.com)
- Official extension overview: [https://surrealdb.com/docs/learn/extensions/plugins/overview](https://surrealdb.com/docs/learn/extensions/plugins/overview?utm_source=chatgpt.com)

## 2. Rust export model and function semantics

### 2.1 Procedural-macro registration

The `#[surrealism]` attribute is the central interface for defining functions exported to SurrealQL. It generates the registration and dispatch machinery needed by the component runtime, rather than requiring authors to implement the WIT ABI manually.

The export system supports several distinct forms:

| Attribute                        | Behavior                                      | Important distinction                                           |
| -------------------------------- | --------------------------------------------- | --------------------------------------------------------------- |
| `#[surrealism]`                  | Exports a Rust function                       | Unannotated Rust functions remain internal                      |
| `#[surrealism(name = "...")]`    | Overrides the exported function name          | Rust symbol naming and public query naming can differ           |
| `#[surrealism(default)]`         | Defines the unnamed/default export            | Enables invocation through the module path alone                |
| `#[surrealism(writeable)]`       | Declares that the function can perform writes | Influences transaction-mode selection and query planning        |
| `#[surrealism(comment = "...")]` | Adds export metadata                          | Available to module introspection and database metadata         |
| `#[surrealism(init)]`            | Registers the initialization function         | Runs on component instantiation, not once per database lifetime |
| `#[surrealism] mod ...`          | Introduces a nested function namespace        | Supports modular, hierarchical API organization                 |

Namespaced exports can compose across multiple nested Rust modules. For example, `math::util::negate` can be an export of the module registered as `mod::scientific`, giving the SurrealQL path `mod::scientific::math::util::negate`. This allows large collections of domain functions to retain coherent namespaces without each becoming an independent WASM artifact.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



Sources:

- Attribute semantics: [https://surrealdb.com/docs/learn/extensions/guides/surrealism-attribute-reference](https://surrealdb.com/docs/learn/extensions/guides/surrealism-attribute-reference?utm_source=chatgpt.com)
- Export renaming and defaults: [https://surrealdb.com/docs/learn/extensions/plugins/further-examples](https://surrealdb.com/docs/learn/extensions/plugins/further-examples?utm_source=chatgpt.com)

### 2.2 Mutability declarations are execution contracts

The `writeable` attribute is particularly consequential. It is not merely documentation attached to a function.

SurrealDB uses the declaration to distinguish read-only functions from functions requiring write-capable execution. Read-only functions may participate in planning decisions that exploit read-only transaction modes or parallel execution.

This creates a responsibility absent from ordinary Rust function attributes: the export's mutability metadata must accurately describe its possible effects, including effects reached through other functions or host calls. A Rust implementation that invokes SurrealQL writes but is declared read-only should not be expected to work correctly in a read-only execution context.

The flag also does not grant permission to write. Permissions and capability restrictions are separate controls.

The host interface explicitly exposes `function-writeable`, alongside the function's argument types and return type, making mutability information available to the engine without invoking the function itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 2.3 Initialization and module-instance lifecycle

`#[surrealism(init)]` is intended for one-time initialization after a WASM component has been instantiated. Documented uses include loading packaged resources, initializing caches, and validating module-local state.

The relevant lifetime is the instantiated execution context. Surrealism pools and reuses module controllers; more than one controller may exist for the same logical module. Consequently:

- Initialization is not a database-wide migration mechanism.
- A process-wide exactly-once guarantee should not be inferred.
- Initialization may run again for a newly instantiated controller or after module eviction/reloading.
- An initialization error marks the module instance unhealthy and prevents normal exported-function calls through it.
- The WIT contract makes host imports available during initialization, including database queries, function calls, and module KV access.

These distinctions matter when initialization interacts with durable records or performs side effects: an initialization hook is not equivalent to a serialized database lifecycle transaction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 2.4 Synchronous and asynchronous Rust execution

Surrealism supports Rust functionality compiled into WASI components, including asynchronous functions in the newer runtime generation.

The SurrealDB 3.1 release line introduced support for `async fn` exports and asynchronous operations within the module's Tokio runtime, with the Tokio runtime initialized for a module instance and reusable across pooled invocations. This broadens the range of Rust networking and asynchronous libraries that can be composed into extensions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



However, async compatibility is not the same as native-platform compatibility. A dependency must still compile for the WASI component target and work within the runtime's allowed host interfaces. Native operating-system APIs, process spawning, arbitrary filesystem access, and unsupported system dependencies do not automatically become available.

The conceptual distinction is that Surrealism can host asynchronous Rust computations, but the WASI sandbox and its capability grants define the effective execution environment.

## 3. Type system and host–guest ABI

### 3.1 SurrealDB values rather than generic JSON

Surrealism uses `surrealdb-types` to exchange database values with Rust functions.

The primary abstraction is the `SurrealValue` trait, which converts between Rust types and SurrealDB's `Value` representation. A derive macro permits application-defined structs to participate in this conversion without manually assembling or parsing generic objects.

This is more expressive than imposing JSON as the universal intermediary. SurrealDB's data model includes types such as:

- Numeric values, strings, booleans, arrays, and objects.
- Record identifiers and other database-specific values.
- Datetimes, spatial geometries, and additional domain-specific value variants.
- Explicit type descriptors through `Kind`.

`SurrealValue` provides conversion methods such as `into_value` and `from_value`; `Kind` supports type descriptions used by the function interface. Structured returns can therefore preserve useful type information at the database boundary rather than always becoming opaque strings.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



A critical distinction remains: a Rust type being serializable does not guarantee that it preserves every SurrealQL semantic distinction. Numeric precision, missing versus null, custom record identifiers, and compound values require the corresponding SurrealDB value representations and compatible conversions. For database-native types, `Value` and the specialized types in `surrealdb-types` are the relevant integration surface.

Sources:

- Shared values and conversion traits: [https://docs.rs/crate/surrealdb-types/3.3.4](https://docs.rs/crate/surrealdb-types/3.3.4?utm_source=chatgpt.com)
- Surrealism typed function example: [https://surrealdb.com/docs/learn/extensions/plugins/quick-tutorial](https://surrealdb.com/docs/learn/extensions/plugins/quick-tutorial?utm_source=chatgpt.com)

### 3.2 WIT component contract with FlatBuffers payloads

The host–guest interface is specified by `surrealism/wit/surrealism.wit`, using the WebAssembly Component Model.

A useful architectural distinction is that the interface has two type layers:

1. WIT-level function contracts describe the available imports and exports, including strings, optional values, results, and byte buffers.
2. FlatBuffers-encoded SurrealDB values carry structured arguments, return values, variable bindings, and type descriptors inside those buffers.

The WIT interface defines aliases such as `serialized-value`, `serialized-vars`, `serialized-args`, `serialized-kind`, and `serialized-arguments`. These are binary payloads, not JSON documents. The corresponding FlatBuffers schemas live under the SurrealDB shared-types implementation and must remain compatible between host and guest versions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



The ABI exposes the following principal guest exports:

| Guest export         | Purpose                                   |
| -------------------- | ----------------------------------------- |
| `invoke`             | Execute a named or default export         |
| `list-functions`     | Enumerate available exports               |
| `function-args`      | Retrieve named argument and type metadata |
| `function-returns`   | Retrieve a declared result type           |
| `function-writeable` | Retrieve declared mutability              |
| `function-comment`   | Retrieve documentation metadata           |
| `init`               | Initialize a component instance           |

This makes introspection a first-class part of the plugin protocol. The engine can discover argument types and mutability information before execution; build tooling can additionally extract this metadata into the module archive, avoiding repeated component instantiation merely to inspect its exports.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



The `surrealism-types` crate supplies shared interface machinery, including the `Args` abstraction for converting ordered Rust arguments into database values and error types used around the boundary.

Source: [https://github.com/surrealdb/surrealdb/blob/main/surrealism/wit/surrealism.wit](https://github.com/surrealdb/surrealdb/blob/main/surrealism/wit/surrealism.wit?utm_source=chatgpt.com)

### 3.3 Errors and typed result conversion

Exported functions can return Rust `Result` types. The official examples include parsing functions that return either a typed numeric result or an error. At the host interface, invocation and host operations use error-bearing WIT `result` types; the guest SDK translates host failures into Rust errors with operation context.

It is useful to distinguish:

- Domain errors: the computation intentionally rejects an input.
- Value conversion errors: arguments or returned values cannot be converted to the expected `SurrealValue` type.
- Capability or authorization errors: the host refuses a requested query, function, or operation.
- Execution errors: component initialization, invocation, resource limits, or runtime execution fail.

These are different failure sources even if they eventually surface as query errors to the caller. Applications requiring stable, machine-readable domain error categories may prefer returning structured values for expected business outcomes rather than relying on arbitrary error strings.

Sources:

- Typed errors in extensions: [https://surrealdb.com/docs/learn/extensions/plugins/quick-tutorial](https://surrealdb.com/docs/learn/extensions/plugins/quick-tutorial?utm_source=chatgpt.com)
- Host imports and error conversion: [https://github.com/surrealdb/surrealdb/blob/main/surrealism/src/imports.rs](https://github.com/surrealdb/surrealdb/blob/main/surrealism/src/imports.rs?utm_source=chatgpt.com)

## 4. Native database and inter-module integration

### 4.1 Executing SurrealQL from a Rust extension

The guest SDK exposes two query helpers:

| API                                   | Main capability                                               |
| ------------------------------------- | ------------------------------------------------------------- |
| `sql<S, R>(query)`                    | Execute SurrealQL and convert the result to `R: SurrealValue` |
| `sql_with_vars<S, V, R>(query, vars)` | Execute SurrealQL with explicitly bound typed variables       |

The second interface is significant because variable values are passed separately as typed SurrealDB values, rather than requiring query construction through string substitution.

This permits Rust computations to compose relational, graph, document, and other SurrealDB query operations with library-specific logic. For example, a module can retrieve records through a SurrealQL expression, perform a specialized calculation in Rust, and return a typed result to the calling query.

The host's WIT contract says queries execute with the module's configured authorization scope. They do not have unrestricted access to database process memory or an implicit right to bypass authorization. Module capabilities further determine whether arbitrary queries are permitted.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Architectural tradeoff: these host calls remain database operations. Their latency, transaction behavior, permissions, and error possibilities still matter even though the call originates from Rust running in the same server. Likewise, a large typed result must cross the WASM boundary and be encoded and decoded.

Source: [https://github.com/surrealdb/surrealdb/blob/main/surrealism/src/imports.rs](https://github.com/surrealdb/surrealdb/blob/main/surrealism/src/imports.rs?utm_source=chatgpt.com)

### 4.2 Calling other functions through `run`

The `run` host import allows an extension to invoke another SurrealDB function using its fully qualified name, optional version, and typed arguments.

The WIT contract explicitly describes cross-module function calls. When a version is specified, it selects a particular version; when omitted, the interface resolves the latest version. Calls remain subject to the caller's capability grants.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



This provides a native composition mechanism for libraries of database functions. Rather than linking every implementation into one WASM artifact, a function can delegate to separately defined functionality where supported by the host's resolver and capability policy.

Useful distinctions:

- Rust-internal function calls do not require host dispatch and stay inside the component.
- Cross-module `run` calls use the Surrealism host interface, introducing resolution, serialization, capability checks, and an additional execution boundary.
- SurrealQL `sql` calls execute query text, potentially involving broader query planning and database work.

The native Rust call is generally the lowest-overhead form for tightly coupled computations; host-mediated calls provide more flexible database-level composition and independently managed modules.

### 4.3 Integration with full-text analyzers

A less obvious integration is SurrealDB's `DEFINE ANALYZER ... FUNCTION` clause.

An analyzer may reference a registered Surrealism `mod::` function as its text-preprocessing stage, executed before the analyzer's tokenizers and filters. The function must accept a string and return a string.

This permits Rust-based normalization, language-specific preprocessing, deterministic canonicalization, and other specialized text transformations to participate in SurrealDB's full-text indexing machinery.

The resulting pipeline is conceptually:

Rust preprocessing function → SurrealDB tokenizer → SurrealDB filters → full-text indexing/search.

This can avoid duplicating preprocessing logic in an external ingestion service, but it also imposes search-specific responsibilities: transformations should be compatible with indexing and query-time analysis, and expensive Rust processing can affect indexing or search costs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: [https://surrealdb.com/docs/reference/query-language/statements/define/analyzer](https://surrealdb.com/docs/reference/query-language/statements/define/analyzer?utm_source=chatgpt.com)

### 4.4 Other SurrealQL expression contexts

Once a function is registered, it can be called through its `mod::...` path from SurrealQL. This makes Rust computations available as composable expressions alongside database-native functions.

Potential applications include domain validation, canonicalization, specialized mathematical functions, identifier processing, geospatial computations, and transformations invoked by queries. These are general possibilities, not separate APIs implemented by Surrealism.

A key boundary is that being callable within SurrealQL does not make a Rust function an intrinsic query optimizer operator or index access method. The engine still has to evaluate the function, and its mutability, cost, and type behavior influence where it is appropriate. Nor does Surrealism automatically provide an aggregate state machine, custom storage backend, or native vector-index implementation.

The analyzer integration above is a specifically documented engine integration; other SQL contexts should be assessed against the rules for that particular context.

## 5. Module-local key-value storage

### 5.1 A stateful capability separate from SurrealDB's persistent store

Surrealism includes a native module-scoped, in-memory KV store, exposed through the `surrealism::kv` API. This is an important distinction from a conventional stateless UDF system: an extension can reuse state between invocations without requiring every intermediate value to become a database record.

The API provides:

| Operation family      | Functions                                       | Semantics                                         |
| --------------------- | ----------------------------------------------- | ------------------------------------------------- |
| Single-key operations | `get`, `set`, `del`, `exists`                   | Retrieve, overwrite, delete, and test values      |
| Batch operations      | `get_batch`, `set_batch`, `del_batch`           | Operate on multiple entries through one host call |
| Range operations      | `del_rng`, `keys`, `values`, `entries`, `count` | Inspect, enumerate, or remove bounded key ranges  |

Values use SurrealDB's typed serialization, while keys are UTF-8 strings limited to 1,024 bytes. The range interfaces accept inclusive/exclusive boundaries through Rust's `RangeBounds<String>` abstraction. `get` distinguishes an absent key through `Option`, rather than requiring an error or sentinel value.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



The WIT contract explicitly describes `kv-set-batch` as an atomic batch update. That guarantee is local to the module KV operation; it should not be confused with an ACID transaction spanning SurrealDB records and arbitrary module state.

Source: [https://github.com/surrealdb/surrealdb/blob/main/surrealism/wit/surrealism.wit](https://github.com/surrealdb/surrealdb/blob/main/surrealism/wit/surrealism.wit?utm_source=chatgpt.com)

### 5.2 State lifetime and isolation

The module KV store is volatile. Its contents can disappear on server restart, module cache eviction, or runtime destruction. The official contract explicitly prohibits treating it as durable storage.

The store is scoped to a module identity. Separate modules cannot read one another's keys. Furthermore, SurrealDB's module registration model isolates state by namespace and database: two databases registering the same Silo package still have separate module state.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



This enables several useful classes of functionality:

- Memoization: caching expensive, repeatable calculations.
- Precomputed reference data: loading small dictionaries or lookup structures during initialization.
- Incremental temporary computations: retaining reusable intermediate results within the lifetime of the loaded module.
- Per-module execution state: storing ephemeral counters or configuration-derived data.

It is not a substitute for a persistent table, a distributed cache, or transactional state. In particular, a multi-node SurrealDB deployment must not assume that module-local KV entries are automatically replicated.

The distinction between reusable WASM instance state and module KV state is also important: pooling can reuse warm execution contexts, but the KV interface is the explicit mechanism for host-managed state associated with the logical module.

### 5.3 Why batching and ranges matter

Batch APIs can reduce repeated guest–host transitions and repeated serialization overhead when many related keys must be handled together. Range operations provide a more structured alternative to maintaining a separate list of keys inside the guest.

These interfaces make the KV facility useful beyond one-off `get` and `set` calls. However, returned batches are still materialized into values crossing the ABI. Very large ranges can therefore impose memory and serialization costs, even without persistent database I/O.

## 6. Module packaging, artifacts, and distribution

### 6.1 The `.surli` artifact

Surrealism does not deploy an ordinary native dynamic library or a raw Rust crate. Its distribution artifact is a `.surli` package containing a compiled WebAssembly component and its associated configuration and metadata.

The runtime documentation describes the archive format as tar with Zstandard compression, containing:

| Component                         | Responsibility                                         |
| --------------------------------- | ------------------------------------------------------ |
| `mod.wasm`                        | Compiled WASM component                                |
| Module configuration              | Package identity, capabilities, and execution settings |
| Export manifest                   | Function names, signatures, mutability, and comments   |
| Optional `surrealism/fs/` content | Read-only resources attached to the module             |

The runtime accepts WASM components rather than arbitrary native binaries. The export manifest is especially valuable: external tooling and the database can inspect the function surface without recompiling or invoking all functions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



A typical module project combines a Rust `cdylib` target, a `surrealism.toml` file, and the `wasm32-wasip2` compilation target. Current scaffolding also configures `tokio_unstable`, which is needed by the Surrealism asynchronous Rust build path.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 6.2 Configuration and capability declaration

The package manifest identifies an organization, package name, and semantic version. The current examples use `target = "rust"` and `abi = 2`.

The `[capabilities]` configuration includes:

| Setting                   | Role                                               |
| ------------------------- | -------------------------------------------------- |
| `allow_arbitrary_queries` | Whether the module may execute arbitrary SurrealQL |
| `allow_functions`         | Declares functions the module may invoke           |
| `allow_scripting`         | Controls access to scripting functionality         |
| `allow_net`               | Declares permitted external network destinations   |
| `strict_timeout`          | Selects strict timeout enforcement behavior        |

The `[attach]` section can specify a filesystem directory to package with the module.

These are module requests or declarations. They do not override server-level restrictions, database permissions, or the WASI sandbox.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 6.3 Attached filesystem versus database buckets

Surrealism has two different file-related mechanisms that should not be conflated.

Attached filesystem: `[attach] fs = "fs"` packages a directory inside the `.surli` artifact. The runtime exposes it read-only within the WASM sandbox. The SurrealDB 3.1 release notes describe this content as mounted at `/`.

It is suitable for versioned dictionaries, fixed reference tables, small models, templates, or other static resources that travel with the extension. It is not writable module storage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



Database buckets: `DEFINE BUCKET` is a SurrealDB database facility for storing and addressing files, including `.surli` packages used by `DEFINE MODULE`.

The available backends include in-memory and local filesystem storage, plus Amazon S3/S3-compatible storage, Google Cloud Storage, and Azure Blob Storage in native SurrealDB 3.3 builds. The cloud backends belong to the database engine, not the Surrealism guest SDK. The filesystem backend is subject to `SURREAL_BUCKET_FOLDER_ALLOWLIST`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



The distinction creates two independent lifecycles: a static resource can be embedded in a particular extension version, whereas the executable package can be uploaded and referenced through a database bucket or package registry.

Source: [https://surrealdb.com/docs/reference/query-language/statements/define/bucket](https://surrealdb.com/docs/reference/query-language/statements/define/bucket?utm_source=chatgpt.com)

### 6.4 Current CLI and development tooling

The currently documented command group is `surreal db module`.

| Command                   | Capability                                       |
| ------------------------- | ------------------------------------------------ |
| `surreal db module init`  | Scaffold a Rust extension project                |
| `surreal db module build` | Compile and package a `.surli` artifact          |
| `surreal db module info`  | Inspect a packaged module's metadata             |
| `surreal db module sig`   | Inspect a particular exported function signature |
| `surreal db module run`   | Execute a packaged function through the CLI      |

`init` supports noninteractive scaffolding through `--headless`, `--org`, and `--name`. `build --debug` avoids the normal optimization pass, favoring compilation speed over runtime performance. `run` permits direct function testing without first registering the package in a database.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



This provides a practical separation of concerns:

- The Rust compiler checks implementation and target compatibility.
- The Surrealism builder validates and packages the executable interface.
- Archive inspection verifies the public function contract.
- CLI invocation exercises guest execution.
- Database integration testing exercises actual SurrealQL context, permissions, and host operations.

The last distinction matters: successfully running a pure function through the CLI does not prove that database-dependent behavior will work under a server's actual capability and authorization configuration.

CLI version qualification: the older `surreal module` form is deprecated. A standalone `surreal mod` launcher component is documented as the intended replacement, but its first release had not shipped as of this research date. `surreal db module` is the released interface.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Sources:

- CLI module reference: [https://surrealdb.com/docs/reference/cli/db/commands/module](https://surrealdb.com/docs/reference/cli/db/commands/module?utm_source=chatgpt.com)
- Module tooling status: [https://surrealdb.com/docs/reference/cli/modules](https://surrealdb.com/docs/reference/cli/modules?utm_source=chatgpt.com)
- Creating custom modules: [https://surrealdb.com/docs/learn/extensions/guides/creating-custom-modules](https://surrealdb.com/docs/learn/extensions/guides/creating-custom-modules?utm_source=chatgpt.com)

### 6.5 Registration and Silo distribution

In SurrealDB 3.3, a bucket-backed module is registered using syntax such as:

`DEFINE MODULE mod::scientific FROM f"modules:/scientific.surli" UNSIGNED;`

The module becomes callable through its registered `mod::` path.

SurrealDB 3.3 also supports direct registration of a versioned package published through Silo:

`DEFINE MODULE silo::organisation::package::<1.0.0> UNSIGNED;`

The Silo form does not require a local bucket path or a separate `mod::` alias. SurrealDB retrieves the package through a configured HTTPS endpoint, with an engine-side cache. The endpoint is configurable through `SURREAL_SURREALISM_SILO_ENDPOINT`; the default is `https://silo.surrealdb.com`. The 3.3 release notes document a 256 MiB package-fetch limit and restrictions on package identifiers to prevent path traversal.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+2



Silo fetching is host-side module resolution, not a guest-initiated network request. Its configured HTTPS endpoint therefore does not require an `allow_net` permission inside the extension.

The same package can be registered independently in multiple databases, each with its own state. `REMOVE MODULE` and replacement through `DEFINE MODULE ... OVERWRITE` affect the registering database's copy.

Signature qualification: SurrealDB 3.3 requires the literal `UNSIGNED` keyword. Silo signs packages on upload, but the official reference explicitly says signature verification had not yet shipped. The keyword should therefore not be interpreted as proof that the runtime has validated a package's origin or integrity.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Source: [https://surrealdb.com/docs/reference/query-language/statements/define/module](https://surrealdb.com/docs/reference/query-language/statements/define/module?utm_source=chatgpt.com)

## 7. Host runtime, pooling, and resource management

### 7.1 Wasmtime execution architecture

`surrealism-runtime` is the host-side implementation. It loads `.surli` artifacts, compiles their WASM components, manages execution controllers, implements host imports, and enforces runtime policies.

Its architecture includes:

- Compiled runtime: shared, reusable compiled WASM representation.
- Controllers: execution contexts that can be pooled and reused.
- Host interface: implementation of WIT imports for queries, function calls, and KV.
- Capability resolution: configuration and policy enforcement for available host operations.
- Epoch-based execution control: machinery used for strict timeout handling.
- Per-module state and cache management: warm module retention and KV lifetime.

These facilities are provided by Surrealism's host runtime, not by the guest `surrealism` crate alone.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.2 Warm controllers and initialization reuse

SurrealDB pools module controllers so that repeated calls can reuse execution contexts rather than paying the complete instantiation and initialization cost every time.

This is particularly relevant for modules that load dictionaries, establish asynchronous runtime infrastructure, or initialize reusable lookup data. The benefit depends on cache residency and whether the same module is invoked repeatedly.

Two separate reuse mechanisms are involved:

1. Compiled module caching: avoids recompiling the same WASM artifact.
2. Controller pooling: reuses instantiated execution resources and their initialized state.

These mechanisms can reduce warm-call overhead, but they do not eliminate argument encoding, host dispatch, or the work performed by a function. Nor should controller-local memory be assumed to be shared among all controllers.

### 7.3 Runtime configuration and actual defaults

The current SurrealDB environment-variable reference exposes the following controls:

| Variable                                | Documented default           | Behavior                                                   |
| --------------------------------------- | ---------------------------- | ---------------------------------------------------------- |
| `SURREAL_LAZY_SURREALISM`               | `false`                      | Selects lazy rather than eager startup compilation         |
| `SURREAL_SURREALISM_CACHE_SIZE`         | `100`                        | Number of modules cached across transactions               |
| `SURREAL_SURREALISM_MAX_POOL_SIZE`      | `8`                          | Documented setting is ignored; effective ceiling remains 8 |
| `SURREAL_SURREALISM_MAX_MEMORY`         | Unlimited                    | Per-module WASM linear-memory ceiling in bytes             |
| `SURREAL_SURREALISM_MAX_EXECUTION_TIME` | Unlimited                    | Per-invocation execution-time ceiling in milliseconds      |
| `SURREAL_SURREALISM_MAX_KV_ENTRIES`     | Unlimited                    | Maximum module KV entry count                              |
| `SURREAL_SURREALISM_MAX_KV_VALUE_BYTES` | Unlimited                    | Maximum size of a module KV value                          |
| `SURREAL_SURREALISM_LOG_LEVEL`          | `debug`                      | Logging level used for module stdout                       |
| `SURREAL_SURREALISM_SILO_ENDPOINT`      | Silo's public HTTPS endpoint | Package-resolution endpoint                                |

An important operational detail: the documentation expressly states that `SURREAL_SURREALISM_MAX_POOL_SIZE` currently has no effect, because the server does not read it. The effective controller pool ceiling remains eight. Merely changing that variable must not be assumed to alter concurrency.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



Likewise, memory and execution-time ceilings are not enabled by default according to the current reference. The availability of a safety setting is not equivalent to an active bound.

Source: [https://surrealdb.com/docs/reference/cli/db/environment-variables](https://surrealdb.com/docs/reference/cli/db/environment-variables?utm_source=chatgpt.com)

### 7.4 Strict versus non-strict timeouts

The runtime supports two execution modes, selected through `strict_timeout`:

- Strict mode: uses a guarded Wasmtime engine with epoch interruption for time-budget enforcement.
- Non-strict mode: uses a faster engine path intended for trusted, compute-heavy modules.

The distinction was introduced with the WASI Preview 2 runtime overhaul. The expected tradeoff is stronger deadline enforcement versus reduced interruption overhead. It is not a published quantitative speed guarantee for a particular workload.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



The interaction with `SURREAL_SURREALISM_MAX_EXECUTION_TIME` is important: a strict interruption mechanism does not itself establish a finite maximum duration when no execution-time ceiling is configured.

The non-strict execution path may be relevant for trusted, bounded numerical computations where interruption overhead is meaningful; strict execution is more important where long-running or adversarial computations could threaten overall database responsiveness.

### 7.5 Eager and lazy compilation

By default, SurrealDB eagerly compiles defined Surrealism modules at startup. Lazy compilation can be selected through `surreal db start --lazy-surrealism` or `SURREAL_LAZY_SURREALISM=true`.

This changes where compilation latency is paid:

| Mode  | Benefit                                             | Cost                                               |
| ----- | --------------------------------------------------- | -------------------------------------------------- |
| Eager | Less first-call compilation delay                   | Startup work and resource use                      |
| Lazy  | Faster initial startup when many modules are unused | First invocation may incur compilation and loading |

The relevant architectural variables are module count, size, compilation expense, expected first-use latency, and cache residency. Neither strategy is inherently superior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database



### 7.6 Async runtime and network execution

The SurrealDB 3.1 release added asynchronous exported functions and the ability to use async Rust dependencies, including examples involving `reqwest` and `sqlx`. A module-instance Tokio runtime can persist across pooled invocations, avoiding reinitializing the asynchronous runtime for every call.

At the same time, network access is constrained by Surrealism's declared `allow_net` policy and WASI socket capabilities. SurrealDB 3.2.5 specifically closed UDP socket access for Surrealism modules, while preserving allowlisted outbound TCP.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+1



The important distinction is that async execution enables concurrency and nonblocking operations; it does not automatically turn a database extension into an unrestricted networking process.

## 8. Security and correctness boundaries

### 8.1 Multiple independent trust boundaries

Surrealism's security posture depends on multiple controls operating together.

| Boundary               | Enforcement mechanism                     | What it does not guarantee                                   |
| ---------------------- | ----------------------------------------- | ------------------------------------------------------------ |
| Feature availability   | Experimental capability gate              | Does not authorize individual queries or data access         |
| WASM sandbox           | Wasmtime/WASI                             | Does not make malicious database logic harmless              |
| Host imports           | WIT API and module capabilities           | Does not guarantee safe semantic use of permitted operations |
| Database authorization | SurrealDB permissions and execution scope | Does not constrain every external side effect                |
| Runtime quotas         | Memory, execution, and KV settings        | Does not automatically bound all host allocations            |
| Network access         | `allow_net` and server policy             | Does not prevent misuse of an explicitly allowed destination |
| Package registration   | `DEFINE MODULE` and artifact resolution   | Does not yet imply signature verification                    |

The `surrealism` experimental capability must be enabled on the database server. Bucket-backed workflows commonly require the separate `files` experimental capability. The server configuration can use `SURREAL_CAPS_ALLOW_EXPERIMENTAL=surrealism,files` or the corresponding startup flag.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 8.2 Effects, transactions, and state

A Surrealism function can combine several effect classes with different guarantees:

- A SurrealQL host query performs database operations under database execution rules.
- A module KV update modifies volatile host-managed state.
- A network request produces an effect outside the database.
- A read-only attached-file lookup accesses packaged resources.

These effects must not be assumed to form a single atomic transaction merely because they occur inside one Rust function. In particular, there is no general guarantee that a database rollback reverses a module KV update or compensates for a network request.

The `writeable` declaration is therefore necessary execution metadata, not a replacement for reasoning about effects.

SurrealDB 3.3 also tightened write rules for computed fields and permission expressions. Writes reached through functions can be rejected where the containing expression must remain read-only. This is particularly relevant to extensions reused across query projections, schema expressions, and other execution contexts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



### 8.3 Sandboxing is not a complete resource-security guarantee

The WASM linear-memory ceiling addresses memory allocated within the WASM component's configured memory. Other costs can arise in the host: encoded inputs and outputs, deserialization, database query results, network buffers, and shared runtime infrastructure.

This distinction has a concrete security history. SurrealDB's 3.3 release included a fix for a vulnerability where Surrealism module output could exhaust host memory (GHSA-jj89-wvrw-c4gp), alongside Wasmtime security dependency updates. The release notes also describe fixes to network-capability enforcement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB



Consequently, the appropriate model is defense in depth: trusted artifacts, least-privilege capabilities, resource bounds, controlled result sizes, current patched database versions, and care with expensive or recursive host operations.

### 8.4 Package and dependency compatibility

Version compatibility involves more than the `surrealism` crate number. The guest SDK, procedural macros, `surrealism-types`, `surrealdb-types`, WIT definitions, FlatBuffers representations, Wasmtime host runtime, and target SurrealDB version participate in the effective interface.

For the verified 0.6.4 release, the guest crate depends on `surrealdb-types ^3.3.4`, `surrealism-macros ^0.6.4`, `surrealism-types ^0.6.4`, and `wit-bindgen ^0.51`. The matching runtime uses Wasmtime 48.0.3.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



The runtime also exposes an SDK-version constant used by build tooling to verify SDK compatibility. The practical implication is that compatibility should be checked against the actual SDK/runtime/engine combination, rather than assuming every arbitrary Surrealism package will run across all SurrealDB 3.x releases.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 9. Architectural choices and capability interactions

### 9.1 Where Surrealism sits among adjacent mechanisms

| Mechanism             | Native strengths                                                                | Important distinction                                                                      |
| --------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| SurrealQL expressions | Built-in functions, query operations, database-native semantics                 | No WASM packaging or host–guest boundary                                                   |
| `DEFINE FUNCTION`     | Reusable database-defined logic expressed in SurrealQL                          | Not arbitrary compiled Rust execution                                                      |
| Surrealism            | Typed Rust extensions, WASI isolation, reusable modules, host imports           | Requires compilation, registration, sandbox compatibility, and experimental feature gating |
| Rust `surrealdb` SDK  | External database clients, application services, embedded engines, live queries | Runs on the application side rather than as a SurrealQL plugin                             |
| External service      | Independent resource scaling and system integrations                            | Requires service orchestration and database/application communication                      |

The core architectural opportunity is moving suitable reusable computations to the point where database expressions can invoke them, without replacing database-native query functionality.

Surrealism is especially differentiated when functionality requires a substantial Rust library, benefits from being expressed as a callable database function, or needs an extension-specific execution boundary.

SurrealDB's ordinary Rust SDK remains the appropriate tool for a different responsibility: external connectivity, request orchestration, live subscriptions, and application-side control over database operations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

Database+1



### 9.2 Composing capabilities to avoid custom infrastructure

Several combinations are particularly significant.

Typed exports + introspection + database registration. An extension can expose a stable collection of typed functions, accompanied by argument metadata and comments discoverable through archive inspection or `INFO FOR DB STRUCTURE`. This reduces the need for independently maintained function-signature descriptions.

Rust preprocessing + full-text analyzers. The explicitly supported `mod::` analyzer function interface allows reusable text-processing logic to become part of database search preprocessing, rather than requiring a separate transformation pipeline outside the database.

Attached filesystem + initialization + module KV. A package can distribute fixed resources, load useful information during initialization, and retain reusable volatile state across calls. This can replace repeated parsing or repeated database lookups for immutable reference data.

Async Rust + allowlisted network access + typed SurrealQL results. A function may integrate external computations or services and return data in native SurrealDB value form. The tradeoff is that a query can now depend on external latency, availability, authorization, and side effects.

Cross-module `run` + versioned registration. Separately deployed function libraries can compose without a single monolithic extension artifact, at the expense of host dispatch and version-management complexity.

These are architectural possibilities supported by combinations of documented primitives, not automatic optimizations or guarantees.

### 9.3 Performance implications

Surrealism can reduce application/database round trips when computations are naturally executed within a query. It can also expose optimized Rust algorithms that would otherwise require lengthy SurrealQL expressions or JavaScript scripting functions.

However, this benefit is balanced against:

- FlatBuffers encoding and decoding at the ABI boundary.
- Wasmtime invocation and dispatch overhead.
- Potentially frequent function execution over query result rows.
- WASM execution versus native-code characteristics.
- Host calls that still incur ordinary database work.
- Cold compilation and initialization costs.
- Resource competition with the database server.

A pure, lightweight transformation invoked once per record over a very large dataset has a different cost profile from an expensive computation performed once on a compact aggregate. Likewise, a function that performs another database query for every row may introduce substantial repeated work.

Surrealism is an execution placement option, not a blanket performance improvement. Its value may instead be expressiveness, type preservation, maintainability, security isolation, or avoiding external service infrastructure.

### 9.4 What Surrealism does not provide

Several capabilities should be kept outside its native scope unless separately implemented by the database or another integration:

- It does not turn arbitrary native Rust crates into usable WASI components without checking their dependencies and system interfaces.
- It does not provide unrestricted access to SurrealDB internals or arbitrary Rust memory inside the database process.
- It does not expose a general API for implementing new storage engines, physical query operators, or index backends.
- It does not turn module KV into durable, replicated database state.
- It does not guarantee database-transaction atomicity for external HTTP operations or ephemeral state changes.
- It does not automatically expose GPU device access or native GPU compute through the WASI sandbox.
- It does not make every possible SurrealQL function context safe for writes or side effects.

These boundaries distinguish the released plugin framework from broader future extensibility ambitions.

## 10. Version evolution and release-status distinctions

| Release                            | Consequential change                                                                                                                                    |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SurrealDB 3.0                      | Initial released Surrealism extension framework, still experimental                                                                                     |
| SurrealDB 3.1                      | WASI Preview 2 component architecture, WIT/FlatBuffers ABI, improved export metadata, attached resources, strict timeout mode, async exported functions |
| SurrealDB 3.2                      | Further runtime and database integrations, including module functions in analyzer preprocessing                                                         |
| SurrealDB 3.2.5                    | UDP access closed for Surrealism modules; outbound TCP limited by `allow_net`                                                                           |
| SurrealDB 3.3                      | `DEFINE MODULE ... FROM ... UNSIGNED`, Silo package resolution, improved module lifecycle and security behavior                                         |
| SurrealDB 3.3.4 / Surrealism 0.6.4 | Current verified stable target pair for this reference                                                                                                  |

The 3.3 release also fixed a previously observed deadlock when two modules in one bucket were first loaded concurrently, underscoring that module loading and bucket handling have their own operational considerations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+3



The broader Surrealism project has discussed support for languages beyond Rust, including JavaScript and Python. That should be treated as an announced direction, not as a released capability of the 0.6.4 Rust plugin framework. Similarly, the standalone `surreal mod` launcher and package signature verification should not be described as shipped features.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://surrealdb.com\&sz=32)

SurrealDB+2



Primary release sources:

- SurrealDB 3.1: [https://surrealdb.com/releases/3.1](https://surrealdb.com/releases/3.1?utm_source=chatgpt.com)
- SurrealDB 3.2: [https://surrealdb.com/releases/3.2](https://surrealdb.com/releases/3.2?utm_source=chatgpt.com)
- SurrealDB 3.3: [https://surrealdb.com/releases/3.3](https://surrealdb.com/releases/3.3?utm_source=chatgpt.com)
- Surrealism release history: [https://docs.rs/crate/surrealism/latest](https://docs.rs/crate/surrealism/latest?utm_source=chatgpt.com)
- Surrealism runtime release history: [https://docs.rs/crate/surrealism-runtime/latest](https://docs.rs/crate/surrealism-runtime/latest?utm_source=chatgpt.com)

## 11. Most consequential architectural takeaways

Surrealism's distinctive value is not simply the ability to call Rust code from SQL. It is the combination of database-native function registration, explicit typed ABI contracts, component sandboxing, host-mediated database access, reusable module state, introspection, and independently distributed WASM artifacts.

Three distinctions are central to understanding the system:

1\. Computation versus database execution. Ordinary Rust computation runs inside a sandboxed component; database queries and cross-module function calls go through controlled host imports. Neither path is equivalent to unrestricted native access to SurrealDB's internals.

2\. State versus durability. Warm controllers and module KV can retain useful state between calls, while database records provide persistence. Treating the two as interchangeable would create incorrect lifecycle, concurrency, and transaction assumptions.

3\. Extensibility versus stability. The 0.6.4 crate has a substantive released API, including advanced async and resource-management capabilities, but the containing database feature remains experimental. Its ABI, deployment syntax, packaging model, and version compatibility deserve more scrutiny than those of a fully stabilized database extension interface.

For an advanced engineering reference, the highest-value areas for deeper investigation are the WIT contract, host import semantics, `SurrealValue` conversions, function mutability and transaction interaction, module KV lifetime, controller pooling, timeout behavior, and the security boundary between module-declared capabilities and database-server policy. Together, these determine what Surrealism can reliably replace with native facilities—and which responsibilities should remain in SurrealQL, the Rust SDK, or external services.

Source coverage note: This overview is grounded in the published 0.6.4 crate metadata, official SurrealDB 3.3 documentation and release notes, and upstream interface/source documentation. Some public rustdoc and repository pages lag the latest release; where a claim concerns a specific server behavior, the current database documentation takes precedence. No runtime benchmark or executable compatibility test was performed for this reference.