# Rust `cdylib` — Comprehensive Technical Reference

Target: Rust 1.99.1 (stable, October 15, 2026) | Research date: October 28, 2026

## 1. Conceptual model and capability map

`cdylib` is a native Rust crate-output type, not a standalone library or external crate. It instructs `rustc` to compile Rust code into a native dynamic library intended for consumption outside Rust's normal crate-linking system.

Its principal purpose is to expose Rust implementations through foreign-function interfaces (FFI), allowing other languages, runtimes, applications, and native plugin hosts to invoke Rust code without depending on the Rust compiler's internal ABI or crate metadata.

Official specification: `https://doc.rust-lang.org/reference/linkage.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference



### What `cdylib` provides

| Capability                       | Mechanism and significance                                                                                                    |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Native shared-library production | Generates `.so` on Linux, `.dylib` on macOS, and `.dll` on Windows                                                            |
| Foreign-language integration     | Enables C, C++, Python, and other language runtimes to load and call exported Rust functions                                  |
| Conventional native loading      | Supports platform dynamic loaders and FFI consumers such as `dlopen`, `dlsym`, and Windows `LoadLibrary`                      |
| Encapsulated Rust implementation | Keeps Rust-specific implementation types, dependencies, and internal symbols behind a deliberately defined external interface |
| Runtime extensibility            | Enables independently distributed native modules and plugin implementations                                                   |
| Mixed-language systems           | Lets a native host reuse Rust components without converting the entire host application to Rust                               |
| Multi-language distribution      | Provides a common native artifact that language-specific bindings can wrap                                                    |
| Compiler optimization            | Supports Rust's conventional release optimization, code generation, and link-time optimization facilities                     |

These capabilities arise from the combination of `cdylib`, Rust's FFI mechanisms, the platform's native linking system, and optional companion libraries. The crate type itself produces an artifact; it does not automatically generate a stable API, C headers, ownership protocol, or language bindings.

### Distinction from other Rust crate types

| Crate type   | Output                    | Primary role                                         |
| ------------ | ------------------------- | ---------------------------------------------------- |
| `rlib`       | `.rlib`                   | Rust compiler-consumable library with Rust metadata  |
| `dylib`      | Native dynamic library    | Dynamically linked Rust-to-Rust dependency           |
| `cdylib`     | Native dynamic library    | Shared library for foreign consumers                 |
| `staticlib`  | `.a` / `.lib`             | Statically linked native library for foreign linkers |
| `proc-macro` | Compiler-loadable library | Rust procedural macros                               |

Two distinctions are particularly important:

- `cdylib` vs. `dylib`: Both generate dynamic binaries, but `dylib` participates in Rust's compiler-managed crate dependency system. `cdylib` is designed as a native-library boundary and is not a conventional `extern crate` dependency.
- `cdylib` vs. `staticlib`: Both are suitable for exposing Rust functionality to foreign consumers. `cdylib` is loaded dynamically; `staticlib` contributes compiled code to a subsequent native link, with different dependency, distribution, and symbol-management consequences.

The Rust Reference explicitly defines these output categories and supports generating multiple crate types from one library target.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference+1



Source: `https://doc.rust-lang.org/cargo/reference/cargo-targets.html`

## 2. Compilation, symbol exports, and linkage architecture

### 2.1 Library target configuration

The principal Cargo configuration is `[lib] crate-type = ["cdylib"]`.

Important capabilities and behaviors:

- Multiple outputs: `crate-type = ["rlib", "cdylib"]` can produce both a Rust-consumable library and a foreign-consumable shared library from the same library target.
- Compiler-level specification: `rustc --crate-type=cdylib` and `#![crate_type = "cdylib"]` provide equivalent crate-type selection mechanisms. Explicit command-line crate-type selections override source-level crate-type attributes.
- One Cargo library target: A package ordinarily has one `[lib]` target, but it can request multiple output formats. Several independently named native libraries generally require separate package or target arrangements.
- Build configuration: Cargo features and conditional compilation allow different external functionality to be included in target-specific or feature-specific builds.
- Cross-compilation: The output is built for a Rust target triple and its ABI, rather than for a portable universal binary representation.

A useful architectural distinction is that the `rlib` exposes a conventional Rust programming interface while the `cdylib` exposes a native binary interface. Both can be backed by the same implementation, without attempting to make Rust-native types universally ABI-stable.

Sources:

`https://doc.rust-lang.org/reference/linkage.html`

`https://doc.rust-lang.org/cargo/reference/cargo-targets.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference+1

### 2.2 Exported symbol control

Generating a `cdylib` is separate from defining which functions are externally callable.

| Mechanism                         | Function                                                                                     |
| --------------------------------- | -------------------------------------------------------------------------------------------- |
| `extern "C" fn`                   | Uses the platform's C calling convention                                                     |
| `#[unsafe(no_mangle)]`            | Exports a symbol using the Rust item's identifier                                            |
| `#[unsafe(export_name = "...")]`  | Assigns an explicit external symbol name                                                     |
| `#[repr(C)]`                      | Establishes C-compatible representation rules for relevant data types                        |
| `#[repr(transparent)]`            | Preserves the layout and ABI of a qualifying wrapped field                                   |
| `#[unsafe(link_section = "...")]` | Places a function or static in a specified object-file section                               |
| `#[used]`                         | Retains a static in intermediate object output; does not guarantee survival of final linking |
| `#[link(...)]`                    | Declares linking requirements for external native libraries                                  |

Important distinctions:

Public Rust visibility does not establish a native FFI contract. `pub` controls Rust-level visibility; `extern "C"` controls calling convention; and symbol-export attributes control externally visible names.

The 2024 edition requires unsafe attributes to use the `#[unsafe(...)]` syntax. This explicitly acknowledges that symbol collisions and section placement can violate soundness.

Exported FFI entry points also need monomorphic, ABI-compatible signatures. Generic implementation functions can be exposed through specialized non-generic wrappers.

`cdylib` ordinarily retains the native entry points required by external consumers while allowing Rust-internal implementation details to be optimized without exposing a broad Rust symbol surface. This is an important distinction from the `dylib` format.

Sources:

`https://doc.rust-lang.org/reference/abi.html`

`https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html`

`https://github.com/rust-lang/rfcs/blob/master/text/1510-cdylib.md`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference+2

### 2.3 Dependency incorporation and binary boundaries

The `cdylib` is a final shared-library artifact, not a reusable intermediate Rust library.

Its dependency treatment has important consequences:

- Rust dependencies: Compiled Rust implementation code is generally incorporated into the resulting shared object rather than exposed as separately resolved Rust crate dependencies. Compiler configuration and available dynamic Rust artifacts can affect linkage.
- Native static dependencies: Native static libraries can be linked into the shared object, subject to platform and linker restrictions.
- Native dynamic dependencies: These can remain external runtime dependencies, requiring compatible shared libraries to be available when the artifact is loaded.
- Standard library: Rust standard-library implementation code can be linked into the artifact. This avoids imposing a normal Rust crate-linking requirement on a foreign host, but does not eliminate every system-library dependency.
- Dead-code elimination: The final native-library boundary can permit removal of implementation code that is neither reachable from exports nor otherwise retained.
- Independent artifacts: Separately compiled `cdylib` files can contain distinct copies of Rust dependencies and their global state. Sharing a Rust crate in source does not imply shared runtime identity between independently built libraries.

That last distinction matters for dynamically composed systems: duplicate allocators, singleton registries, logging implementations, runtime state, or Rust types must not be assumed to share identity merely because their originating source crates are identical.

Source: `https://doc.rust-lang.org/reference/linkage.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference



### 2.4 Target-specific linker integration

Cargo provides an unusually useful `cdylib`-specific build-script mechanism:

`cargo::rustc-link-arg-cdylib=FLAG`

This applies a native linker argument specifically when producing a `cdylib`.

Other relevant mechanisms include `cargo::rustc-link-lib`, `cargo::rustc-link-search`, the more general `cargo::rustc-link-arg`, and target-specific Cargo linker configuration.

These enable:

- Runtime search-path and loader-metadata customization.
- Native symbol-export policies and, where supported, linker versioning.
- Integration with platform-specific libraries and frameworks.
- Custom linker selection and target-specific native build arrangements.
- Coordination with generated C/C++ components, link scripts, and native object files.

These controls are valuable because the Rust crate model and the native platform linker model are distinct. Final shared-library packaging often requires both.

Source: `https://doc.rust-lang.org/cargo/reference/build-scripts.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



## 3. FFI data representation and memory semantics

This is the most consequential technical boundary in `cdylib` integration.

`cdylib` makes Rust code loadable by a foreign runtime, but the compatibility of every exchanged value remains a separate contract.

### 3.1 ABI-compatible data representations

| Rust mechanism                             | FFI role                                            | Important qualification                                                |
| ------------------------------------------ | --------------------------------------------------- | ---------------------------------------------------------------------- |
| `i8`–`i64`, `u8`–`u64`, `f32`, `f64`       | Fixed-width scalar values                           | C declarations must match width and signedness                         |
| `core::ffi` / `std::ffi` C types           | Native C scalar compatibility                       | Includes types such as `c_int`, `c_char`, `c_void`                     |
| `#[repr(C)] struct`                        | C-compatible structure layout                       | Field types must themselves have compatible representations            |
| `#[repr(C)] union`                         | C-compatible union storage                          | Active-member and valid-value rules remain the caller's responsibility |
| `#[repr(transparent)]`                     | ABI-transparent type wrappers                       | Useful for strongly typed wrappers around compatible representations   |
| `#[repr(u32)]` and related representations | Explicit enum discriminant representation           | Valid discriminant values still matter                                 |
| Raw pointers                               | Borrowed buffers, output parameters, opaque objects | No automatic lifetime or ownership checking                            |
| `extern "C" fn` pointers                   | Callbacks and function tables                       | Exact signatures, calling conventions, and unwind behavior must agree  |
| `Option<extern "C" fn(...)>`               | Nullable callback pointer                           | Uses a guaranteed nullable-pointer representation                      |

The important distinction is layout compatibility versus value validity.

For example, a C enumeration may legally hold arbitrary integers, but a Rust enumeration has stronger restrictions on valid discriminants. Consequently, even a `repr(C)` enum is not automatically a safe representation of an unrestricted C enum.

Similarly, `repr(C)` controls the layout of a containing struct; it does not make every Rust field type valid for foreign callers. A struct containing Rust-native strings, vectors, trait objects, or references may still have unsuitable ABI semantics.

The Rust Reference also distinguishes memory layout from function-call ABI. Identical field offsets do not, by themselves, prove two types are passed identically by value.

Sources:

`https://doc.rust-lang.org/reference/type-layout.html`

`https://doc.rust-lang.org/nomicon/ffi.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference+2

### 3.2 Opaque handles and encapsulated state

An important architectural capability is exposing arbitrarily complex Rust functionality without exposing the memory layout of its implementation types.

A typical opaque-object interface provides:

1. A constructor returning a pointer or opaque handle.
2. Operations accepting that handle.
3. Explicit state-inspection or mutation functions.
4. A destructor responsible for reclaiming the underlying Rust resources.

This permits the implementation to contain collections, trait objects, asynchronous runtime state, caches, synchronization primitives, and other native Rust constructs without exposing those representations directly.

The external handle can remain stable while its internal Rust representation changes.

Two important consequences:

- Opaque types are especially valuable for ABI evolution, because internal data layout need not become part of the public contract.
- Raw opaque handles still require explicit validity rules: nullability, ownership, borrowing, thread affinity, destruction, and whether multiple handles may refer to the same resource.

The Rustonomicon describes representations that deliberately prevent foreign opaque types from accidentally acquiring Rust traits such as `Send`, `Sync`, or `Unpin`.

Source: `https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rustonomicon



### 3.3 Buffer and string exchange

`cdylib` supports both copy-based and zero-copy data transfers, but the latter is enabled by the data representation and ownership contract, not by the crate type itself.

Borrowed buffers: A pointer and length can expose contiguous native memory without copying. Validity depends on the allocation remaining alive, stable, appropriately aligned, and accessible for the entire borrowing period. Mutable buffers additionally require coordinated exclusive access.

Owned buffers: A producer can transfer or share an allocation through a pointer, length, and corresponding release mechanism. The consumer must use the defined release operation, not an arbitrary local deallocator.

Strings: `CString` and `CStr` support NUL-terminated C strings. Binary data and UTF-8 text can instead use pointer-and-length representations, avoiding the ambiguity of embedded NUL bytes. Encoding validation and ownership must still be explicit.

Rust `Vec<T>`: Its native representation is not a portable C ABI. Although its allocation can be transferred using raw components, reconstruction requires the correct allocation metadata and ownership invariants. Merely knowing a pointer and length is not always enough to reclaim a Rust allocation correctly.

Rust `String` and `&str`: These should not be treated as universally ABI-compatible C string values. They require an appropriate bridge or external representation.

A major failure mode is freeing memory using a different allocation mechanism from the one that created it. The Rust documentation explicitly cautions against calling C `free()` on a pointer transferred from `CString::into_raw()`.

Source: `https://doc.rust-lang.org/std/ffi/struct.CString.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust



### 3.4 Callback interfaces and bidirectional control

Native dynamic libraries can support more than simple request/response calls.

A `cdylib` may export registration functions, accept foreign callback pointers, call host functions, and maintain callback state across operations.

Useful mechanisms include:

- Callbacks with user context: Pair a function pointer with a `void*` context pointer, allowing host-specific state to be associated with callbacks without using process-global variables.
- Function tables: Return a structure containing pointers to callable operations, permitting a compact and versioned native interface.
- Host-provided services: Accept allocator, logging, error-reporting, event-delivery, or scheduling callbacks instead of introducing independent implementations.
- Event-driven integration: Use callbacks to report completion, progress, diagnostic information, or asynchronous results.
- Opaque callback registrations: Return a registration handle so the host can explicitly cancel subscriptions and release resources.

The crucial complication is asynchronous callback lifetime. A callback may arrive from another thread or after the original function has returned. The library and caller must agree on synchronization, thread ownership, cancellation, and when callback state may be destroyed.

The Rustonomicon explicitly discusses both object-targeted callbacks and callbacks invoked from foreign threads.

Source: `https://doc.rust-lang.org/nomicon/ffi.html#asynchronous-callbacks`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rustonomicon



## 4. Execution safety, panics, and concurrency

### 4.1 Panic and exception boundaries

Rust's panic behavior is not automatically compatible with a foreign runtime's exception mechanism.

| Mechanism                    | Behavior                                                                   |
| ---------------------------- | -------------------------------------------------------------------------- |
| `extern "C"`                 | Non-unwinding ABI; a Rust panic reaching the boundary causes process abort |
| `extern "C-unwind"`          | Permits appropriate native unwinding across the boundary                   |
| `panic = "unwind"`           | Enables Rust panic unwinding, subject to ABI constraints                   |
| `panic = "abort"`            | Panics terminate the process rather than unwind                            |
| `std::panic::catch_unwind`   | Can intercept unwinding Rust panics within an appropriate Rust boundary    |
| Explicit status/error return | Provides a language-neutral error contract                                 |

A foreign exception unwinding through a non-unwinding Rust ABI boundary is undefined behavior. The `"C-unwind"` ABI does not make arbitrary exception interaction safe: the involved language runtimes, compiler exception model, and native frames still matter.

`catch_unwind` is not a general-purpose protection mechanism for all failures. It does not catch aborting panics, memory corruption, or every form of foreign exception.

For libraries expected to be embedded in long-running hosts, explicit error-status values, output error structures, and boundary-level conversion of recoverable Rust failures provide a more predictable contract.

Sources:

`https://doc.rust-lang.org/reference/items/functions.html#unwinding`

`https://doc.rust-lang.org/reference/panic.html`

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Reference+1

### 4.2 Thread safety and ownership

Rust's compile-time guarantees remain effective within the Rust implementation. They do not automatically validate pointers, mutation, or concurrency originating in foreign code.

A native interface therefore needs to define:

- Whether handles are thread-safe or thread-affine.
- Whether calls against a single handle may execute concurrently.
- Whether callbacks may be invoked on foreign-created threads.
- Whether pointers remain valid after a function returns.
- Whether the foreign caller may mutate memory while Rust is reading it.
- Whether callback deregistration guarantees that no callback is still running.
- Whether destruction requires operations to have completed.

`Send` and `Sync` constrain Rust behavior but cannot independently enforce these rules against arbitrary C or Python callers.

Consequence: A Rust component can use sophisticated internal concurrency while maintaining a small external ABI, provided that external operations preserve the ownership and synchronization invariants expected by the implementation.

### 4.3 Asynchronous Rust behind a synchronous ABI

A `cdylib` can encapsulate Tokio, async tasks, channels, background workers, or other concurrency machinery without requiring the host to understand Rust's `Future` representation.

The native interface can instead expose operations such as:

- Submit work and receive an operation handle.
- Poll or await completion through host-specific mechanisms.
- Register a completion callback.
- Cancel pending work.
- Retrieve results and release operation state.

These are interface patterns built on top of `cdylib`, not built-in asynchronous APIs.

Their value is the separation of the internal execution model from the external binary interface. The principal complexity is lifecycle coordination: a loaded library cannot safely be destroyed while its callbacks, tasks, or function pointers remain usable.

## 5. Dynamic loading, native plugins, and ABI evolution

### 5.1 Runtime loading and symbol resolution

A `cdylib` may be linked into a native application at build time or loaded dynamically while the host runs.

Platform loading mechanisms include:

| Platform       | Loading mechanism                              | Relevant distinction                                               |
| -------------- | ---------------------------------------------- | ------------------------------------------------------------------ |
| Linux / ELF    | `dlopen`, `dlsym`, `dlclose`                   | Symbol visibility, loader namespaces, dependency search paths      |
| macOS / Mach-O | `dlopen`, `dlsym`                              | Install names, `@rpath`, framework and code-signing considerations |
| Windows / PE   | `LoadLibrary`, `GetProcAddress`, `FreeLibrary` | DLL search behavior, import libraries, export naming               |

Rust's `libloading` crate provides a cross-platform abstraction around native loading primitives, including `Library`, `Library::get`, and the lifetime-bound `Symbol<'lib, T>`.

A particularly useful guarantee is that its normal `Symbol` abstraction ties the symbol's lifetime to the loaded `Library`, reducing the risk of calling a function after the library is unloaded.

However, `libloading` does not verify that an arbitrary requested function signature matches the loaded symbol's actual ABI. That remains an unsafe programmer-supplied assertion.

Source: `https://docs.rs/libloading/latest/libloading/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.2 Versioned plugin interfaces

Native dynamic libraries can serve as independently compiled plugin modules, but the crate type does not implement a plugin protocol.

A durable plugin interface can be formed around a small set of exports:

- An ABI version or compatibility query.
- A function that returns a versioned API table.
- A capability-discovery mechanism.
- Constructor and destructor operations.
- Host-service callback registration.
- An error-reporting convention.

One advanced pattern is a versioned function table. Rather than exporting every operation as an independently named symbol, the library exports one known entry point that returns a structure of operations.

This allows the interface to represent optional features explicitly. A size field, ABI version, and supported-capability flags can enable compatible extensions, provided both sides agree on how to interpret old and new table layouts.

The mechanism resembles conventional C plugin APIs, virtual-function tables, and systems interfaces. It is particularly useful when a host may load independently distributed implementations or implementations compiled using different Rust releases.

### 5.3 Rust-to-Rust plugins using `abi_stable`

The companion crate `abi_stable` deserves special attention because it addresses a problem `cdylib` deliberately does not solve.

`abi_stable` provides an FFI-safe Rust-to-Rust interface system with:

- `StableAbi` type-layout descriptions and compatibility checks.
- `RootModule` and runtime-loaded modules.
- `#[sabi_trait]` for FFI-safe trait-object-style interfaces.
- `DynTrait` for type-erased behavior.
- ABI-safe substitutes for selected standard-library types.
- Prefix types supporting compatible interface extension.
- Non-exhaustive enum facilities.
- Load-time checking of type-layout compatibility.
- Support for loading implementations built with different Rust compiler versions.

Together, `cdylib` and `abi_stable` can support more expressive Rust-native plugin architectures than manually exposing only primitive C functions.

Important limitation: `abi_stable` explicitly focuses on runtime loading without supporting library unloading. It also introduces its own interface conventions and type restrictions; it does not make arbitrary Rust types or trait objects ABI-stable.

Source: `https://docs.rs/abi_stable/latest/abi_stable/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 6. Language bindings and companion libraries

The principal ecosystem distinction is between libraries that create the native artifact, libraries that define its foreign interface, and libraries that package or load it.

### 6.1 C and C++ interoperability

| Library      | Principal capabilities                                                    | Relationship to `cdylib`                             |
| ------------ | ------------------------------------------------------------------------- | ---------------------------------------------------- |
| `cbindgen`   | Generates C, C++, and Cython declarations from Rust definitions           | Documents and exposes the native ABI                 |
| `safer-ffi`  | FFI-safe representations, export attributes, header generation            | Provides higher-level facilities for building C APIs |
| `bindgen`    | Generates Rust declarations from C/C++ headers                            | Primarily supports Rust consuming foreign APIs       |
| `cxx`        | Type-checked Rust/C++ interoperability with generated bridges             | Facilitates bidirectional Rust/C++ integration       |
| `autocxx`    | Higher-level C++ interoperability built around CXX and generated bindings | Reduces manual C++ bridge implementation             |
| `libloading` | Dynamic library loading and symbol resolution                             | Consumes native shared libraries at runtime          |

`cbindgen` is especially relevant to long-lived external C interfaces. It can generate headers from exported functions, statics, constants, and types with suitable representations. Configuration supports symbol naming, namespaces, preprocessing conditions, language selection, and header organization.

An important limitation is that generating a header does not independently establish the soundness of a native interface. Ownership rules, allocation, error semantics, and runtime compatibility still need to be defined.

`safer-ffi` goes further into interface construction, with attributes such as `#[ffi_export]` and representation-aware facilities for FFI-compatible Rust types.

`cxx` is valuable when C++-specific semantics are consequential. Its bridge supports appropriate interoperation with `std::string`, vectors, unique/shared pointers, Rust strings and slices, shared structures, and opaque classes. The generated bridge handles cases where equal memory layout does not guarantee identical by-value calling ABI.

However, the CXX bridge is not equivalent to establishing an independently versioned, general-purpose C ABI. Its generated components must be understood as part of the integrated Rust/C++ build.

Sources:

`https://github.com/mozilla/cbindgen/blob/main/docs.md`

`https://docs.rs/safer-ffi/latest/safer_ffi/`

`https://cxx.rs/`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

### 6.2 Python integration: PyO3, maturin, CFFI

Python supports several significantly different pathways for consuming a Rust `cdylib`.

| Pathway         | Interface model                                                      | Material distinction                                                 |
| --------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------- |
| PyO3            | Native Python extension using CPython API                            | Rich Python objects, classes, functions, exceptions, conversions     |
| maturin + PyO3  | PyO3 extension packaged as a Python wheel                            | Build, packaging, compatibility tags, distribution                   |
| maturin + CFFI  | Generated interface around C-compatible native functions             | Interpreter-independent C-interface design                           |
| Python `ctypes` | Direct loading of exported C functions                               | Minimal binding infrastructure but manual declarations               |
| `cffi`          | C declarations and FFI invocation                                    | More expressive C-level interfacing than manual `ctypes` definitions |
| UniFFI          | Generated Python bindings from a shared Rust interface specification | Useful when Python is one of several target languages                |

The key distinction is that a native Python extension is not merely a generic C library renamed to `.so`.

A PyO3 extension exports the initialization interface expected by Python and supplies Python-compatible objects and method implementations. `#[pymodule]`, `#[pyfunction]`, and `#[pyclass]` participate in this specialized integration.

PyO3 0.30.1 also supports Python limited-API packaging through `abi3` and the newer `abi3t` mechanism associated with Python 3.15's free-threaded-compatible stable ABI. These mechanisms address Python runtime compatibility; they are separate from Rust's `cdylib` output type.

`maturin` handles much of the native-extension build and distribution process, including wheel generation, target compatibility, module naming, and interpreter configuration. It also supports alternative binding backends, including CFFI and UniFFI.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Sources:

`https://pyo3.rs/v0.30.1/building-and-distribution`

`https://www.maturin.rs/bindings`

`https://www.maturin.rs/config`

A meaningful architectural choice follows:

- PyO3 is suited to exposing a Python-native object model.
- CFFI or `ctypes` is suited to consuming an existing C ABI, potentially shared with non-Python hosts.
- UniFFI is suited to generating multiple language bindings from a common interface model.

### 6.3 UniFFI: multi-language bindings

Mozilla's UniFFI is a higher-level companion for exposing Rust functionality through generated foreign-language bindings.

Core capabilities include:

- Rust interface definitions using procedural macros or an interface definition language.
- Generated Rust FFI scaffolding.
- Generated foreign-language data conversions and bindings.
- Support for complex interface concepts rather than only raw C functions.
- Primary support for Python, Swift, and Kotlin, with additional Ruby and community binding implementations.

UniFFI is particularly relevant when one Rust implementation needs several consistent language-facing APIs without separately maintaining each binding layer.

The boundary is important: UniFFI provides interface generation, while Cargo, native compilers, platform packaging mechanisms, and distribution tooling still produce and deploy the binaries.

Source: `https://mozilla.github.io/uniffi-rs/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://mozilla.github.io\&sz=32)

mozilla.github.io



### 6.4 WebAssembly and `wasm-bindgen`

A less obvious capability is that `cdylib` also appears in Rust's WebAssembly compilation workflows.

For `wasm32-unknown-unknown`, `wasm-bindgen` commonly uses `crate-type = ["cdylib"]` to produce a Wasm module that can subsequently be processed into JavaScript-compatible bindings.

This enables:

- Exporting Rust functions and types to JavaScript.
- Importing JavaScript functionality into Rust.
- Bridging strings, objects, closures, and richer values.
- Generating JavaScript and TypeScript interoperability artifacts.

This is not the same loading model as an ELF or PE shared library. WebAssembly has its own module, import/export, memory, and runtime semantics. The crate type participates in artifact generation, while `wasm-bindgen` supplies language-level adaptation.

Source: `https://rustwasm.github.io/docs/wasm-bindgen/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustwasm.github.io\&sz=32)

rustwasm.github.io+1



### 6.5 Native application and database extensions

Native shared-library hosts provide another significant application family.

Examples include native database extensions, dynamically discovered application plugins, computation engines, and process-hosted components.

The Rust `pgrx` ecosystem illustrates the distinction: PostgreSQL loads compiled native extensions, while `pgrx` supplies PostgreSQL-specific interfaces, macros, type conversions, and extension tooling.

The upstream `pgrx` 0.18 release notes explicitly describe simplifying extension builds into `cdylib` crates, with SQL metadata embedded in a linker section for build-tool extraction.

This demonstrates that the native artifact can participate in sophisticated host-specific extension systems even when the host does not expose an ordinary C function API to users.

Source: `https://github.com/pgcentralfoundation/pgrx/releases`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 7. Advanced data interchange through standardized native interfaces

One of the more valuable architectural possibilities is using `cdylib` as an execution boundary while adopting an existing data ABI rather than inventing a bespoke serialization or buffer protocol.

### 7.1 Apache Arrow C Data Interface

The Apache Arrow C Data Interface provides ABI-stable structures for exchanging columnar data between independently implemented runtimes.

Its principal structures are:

- `ArrowSchema`: schema, types, metadata, and nested field descriptions.
- `ArrowArray`: array length, offsets, buffers, children, dictionaries, and ownership.
- Producer-provided release callbacks: transfer responsibility for releasing exported resources.

Rust Arrow libraries provide relevant interoperability functionality through facilities such as `arrow::ffi`, `to_ffi`, `from_ffi`, `FFI_ArrowArray`, and `FFI_ArrowSchema`, where the appropriate crate features are enabled.

Crucially, the interface is designed to support zero-copy data sharing within one process. A `cdylib` can therefore exchange Arrow-compatible data with a host without requiring both implementations to share the same Arrow language library or internal object representation.

This can avoid maintaining bespoke serialization, field-layout, null-handling, and memory-transfer code.

The compatibility guarantee applies to the standardized C interface; it does not imply that arbitrary Rust Arrow types can cross the boundary directly.

Sources:

`https://arrow.apache.org/docs/format/CDataInterface.html`

`https://arrow.apache.org/rust/arrow/array/ffi/index.html`

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow v26.0.0+1

### 7.2 Apache Arrow C Stream Interface

The Arrow C Stream Interface extends this approach to sequences of record batches.

`ArrowArrayStream` supplies a small callback-based API:

| Operation        | Purpose                           |
| ---------------- | --------------------------------- |
| `get_schema`     | Retrieve the shared stream schema |
| `get_next`       | Obtain the next array or batch    |
| `get_last_error` | Retrieve an error description     |
| `release`        | Release stream resources          |

This can support streaming producers and consumers across shared-library boundaries without introducing a full custom stream protocol.

The interface specifies a blocking pull-style iteration model. It does not automatically supply asynchronous scheduling, backpressure across independent execution systems, or distributed transport.

Source: `https://arrow.apache.org/docs/format/CStreamInterface.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://arrow.apache.org\&sz=32)

Apache Arrow v26.0.0



### 7.3 DLPack and tensor exchange

DLPack provides an analogous standardized native representation for multidimensional arrays and tensors.

Its managed tensor structures represent:

- Data pointers, shapes, strides, and types.
- Device identity, including CPU and accelerator devices.
- Byte offsets.
- Ownership and resource-release callbacks.
- Versioned exchange metadata.

`DLManagedTensorVersioned` supplies ABI version information and managed-resource semantics, permitting compatible consumers to understand the tensor representation without sharing the producer's native container implementation.

In Rust, relevant companion packages include `dlpack-ffi` and higher-level DLPack integrations.

This provides a possible bridge between Rust native libraries and runtimes such as PyTorch or other DLPack-capable systems. GPU memory lifetime, device synchronization, stream semantics, and unsupported layouts still require appropriate handling.

Sources:

`https://github.com/dmlc/dlpack/blob/main/include/dlpack/dlpack.h`

`https://docs.rs/dlpack-ffi/latest/dlpack_ffi/`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1

Architectural significance: A `cdylib` combined with an established data-interface specification can preserve an independently compiled execution boundary while avoiding the cost and maintenance burden of a custom cross-language data representation.

## 8. Binary optimization, distribution, and observability

### 8.1 Optimization opportunities

A `cdylib` is eligible for normal Rust compiler and linker optimization mechanisms.

| Mechanism                       | Potential benefit                                              | Consequential qualification                                            |
| ------------------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Release optimization            | Faster native execution                                        | Increases compilation cost according to optimization settings          |
| Link-time optimization          | Cross-crate optimization and dead-code elimination             | May increase link time and memory usage                                |
| Code-generation units           | Controls optimization/compile-time tradeoff                    | Lower parallelism can improve whole-program optimization opportunities |
| Native CPU targeting            | Enables target-specific machine instructions                   | Reduces portability to older processors                                |
| Stripping                       | Reduces nonessential symbol/debug information                  | Can hinder debugging and diagnostics                                   |
| `panic = "abort"`               | Simplifies panic runtime behavior and may reduce artifact size | A panic terminates the process                                         |
| Static dependency incorporation | Reduces separately distributed Rust dependency artifacts       | Can duplicate implementation code across separately built modules      |

The final-artifact nature of `cdylib` is significant. Unlike a Rust `dylib` intended for future Rust crate linkage, it does not need to preserve the full set of compiler-consumable Rust implementation symbols.

This can give the compiler more latitude to optimize code behind the exported interface.

However, a shared-library call is not generally equivalent to an inlined Rust function call. Calls crossing a separately compiled ABI boundary may have additional dispatch, indirect-call, or language conversion costs. Optimization of the internal implementation does not eliminate those boundaries.

Source: `https://doc.rust-lang.org/rustc/codegen-options/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustc book



### 8.2 Platform compatibility and packaging

A successful native build does not imply that the artifact will load everywhere.

Important dimensions include:

- Architecture: x86-64, ARM64, and other target-specific machine-code requirements.
- OS binary format: ELF, Mach-O, and PE are distinct.
- System ABI: glibc versus musl, Windows MSVC versus GNU toolchains, platform calling conventions.
- Runtime dependencies: Native shared libraries required by the artifact must be discoverable and compatible.
- Loader metadata: Linux SONAME/RPATH/RUNPATH, macOS install names and `@rpath`, Windows DLL search behavior.
- Code signing and platform security: Particularly relevant to restricted macOS and other managed deployment environments.
- Host integration: Python extension naming and wheel tags, database extension versions, or native plugin API requirements.

`cdylib` produces a target-specific native artifact; it does not automatically bundle its native dependencies, install itself, or manage the system dynamic loader.

Source: `https://doc.rust-lang.org/rustc/platform-support.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The rustc book



### 8.3 Binary verification and interface inspection

Although not facilities supplied by `cdylib` itself, conventional native tooling provides valuable visibility into the resulting artifact.

| Platform/tool                       | What it can reveal                                                     |
| ----------------------------------- | ---------------------------------------------------------------------- |
| Linux `nm -D`, `readelf`, `objdump` | Dynamic symbol exports, dependencies, relocations, symbol versions     |
| Linux `ldd`                         | Loader dependency resolution, subject to platform and security caveats |
| macOS `nm`, `otool`                 | Exported symbols, linked libraries, Mach-O metadata                    |
| Windows `dumpbin`, LLVM tools       | Export tables, import tables, DLL dependencies                         |
| `llvm-readobj`                      | Structured inspection of supported native binary formats               |
| `cargo rustc` / linker diagnostics  | Compiler and linker configuration, selected target behavior            |

These tools can help establish whether the binary actually exports the intended interface and which dynamic dependencies remain.

For durable native-library interfaces, the distinction between the symbols intended for export and the symbols actually exported by the final linker is worth preserving.

## 9. Rust 1.99-specific capabilities and experimental boundaries

### 9.1 C-variadic function definitions — stabilized in Rust 1.99.0

Rust 1.99.0, released October 1, 2026, stabilized defining C-ABI variadic functions using `"C"` and `"C-unwind"`.

Previously, stable Rust supported calling foreign C-variadic functions, but defining such functions in Rust itself was restricted.

The released capability includes:

- Rust-defined C-compatible functions with `...` arguments.
- `core::ffi::VaList` for processing variadic arguments.
- `VaArgSafe` for governing supported argument access.
- Applicable `"C"` and `"C-unwind"` calling conventions.

This expands the native interfaces that Rust can implement directly, including certain callback and legacy C-interface patterns that expect variadic entry points.

C variadics retain their normal type, promotion, calling-convention, and safety constraints. This is not a mechanism for exporting arbitrary Rust heterogeneous argument lists.

Source: `https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://blog.rust-lang.org\&sz=32)

Rust Blog



### 9.2 Cargo artifact dependencies — still unstable

A particularly relevant experimental capability is Cargo's artifact dependencies (`-Z bindeps`).

This allows a dependency declaration to request compiled artifacts such as `bin`, `cdylib`, or `staticlib`, rather than treating every dependency solely as a Rust library dependency.

Its intended capabilities include:

- Building a dependency specifically as a `cdylib`.
- Making the compiled artifact path available to build scripts.
- Supporting different target triples for requested artifacts.
- Coordinating native artifact production with dependent build operations.

For example, Cargo's documentation describes building a WebAssembly `cdylib` as a dependency and obtaining its artifact path through the build-script environment.

As of Rust/Cargo 1.99, this remains a nightly-only Cargo feature, not ordinary stable Cargo functionality.

Source: `https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Cargo Book



## 10. Consequential distinctions and architectural implications

The following distinctions capture the most important choices exposed by `cdylib` and its surrounding ecosystem.

| Design question                                              | Mechanism or distinction                           | Why it matters                                                             |
| ------------------------------------------------------------ | -------------------------------------------------- | -------------------------------------------------------------------------- |
| Can foreign code call Rust?                                  | `cdylib` plus exported compatible ABI              | Native loading and valid invocation are separate requirements              |
| Can another Rust crate import its Rust API?                  | Usually use `rlib` or `dylib` instead              | A `cdylib` does not expose the ordinary Rust crate-linking interface       |
| Can different Rust versions interoperate?                    | Stable external ABI, potentially `abi_stable`      | `cdylib` alone does not stabilize Rust-native layouts and types            |
| Can data exchange avoid copying?                             | Borrowed buffers, Arrow C interfaces, DLPack       | Shared memory requires explicit representation and lifetime contracts      |
| Can implementations be replaced independently?               | Versioned symbols, function tables, opaque handles | Binary compatibility is possible without preserving internal Rust types    |
| Can the library use internal async execution?                | Native task handles, callbacks, polling            | Rust futures need not be exposed across the ABI                            |
| Can foreign callers own Rust resources?                      | Explicit ownership-transfer and release functions  | Prevents allocator mismatch and invalid destruction                        |
| Can the library be unloaded safely?                          | Native loader plus lifecycle protocol              | Live callbacks, tasks, or pointers can make unloading unsafe               |
| Can a single Rust implementation support many languages?     | C ABI, UniFFI, PyO3, generated bindings            | Separates implementation reuse from language-specific interface adaptation |
| Can build tooling manage `cdylib` artifacts as dependencies? | Cargo artifact dependencies                        | Currently an experimental rather than stable Cargo mechanism               |

### Most important architectural opportunities

1\. Separate Rust implementation evolution from native interface evolution. Opaque handles and narrowly defined external functions permit extensive internal changes without exposing unstable Rust types or compiler-specific representations.

2\. Use established binary and data contracts instead of inventing them. C ABI conventions, Arrow C Data/Stream interfaces, DLPack, and companion FFI libraries can remove considerable bespoke interface code while retaining independently compiled components.

3\. Make plugin extensibility an interface-level property. Versioned function tables, capability discovery, explicit lifecycle contracts, and ABI validation enable modular native components. `cdylib` supplies the binary delivery mechanism, while those additional contracts provide interoperability.

### Principal limitations

`cdylib` does not itself provide Rust ABI stability, cross-language type safety, automatic memory ownership, exception translation, native-library unloading safety, version negotiation, shared mutable state across libraries, or portable binary distribution.

These capabilities must come from explicit interface design, additional libraries, runtime protocols, or platform tooling.

## Conclusion

`cdylib` is best understood as Rust's native dynamic-library boundary mechanism.

Its significance is not merely the production of `.so`, `.dylib`, or `.dll` files. It enables Rust implementations to participate in systems where the controlling runtime, language, dependency graph, and binary deployment process are independent of Rust.

The greatest capabilities emerge through composition:

- `cdylib` + C ABI + `cbindgen` or `safer-ffi` for conventional native interfaces.
- `cdylib` + `libloading` + versioned function tables for runtime plugins.
- `cdylib` + `abi_stable` for more expressive Rust-to-Rust plugin interfaces.
- `cdylib` + PyO3 + maturin for distributable Python extensions.
- `cdylib` + UniFFI for multi-language bindings.
- `cdylib` + Arrow C interfaces or DLPack for standardized high-performance data interchange.
- `cdylib` + `wasm-bindgen` for WebAssembly integration.

The governing distinction throughout is that the compilation artifact, calling convention, data representation, ownership model, and runtime protocol are different layers. Understanding their boundaries makes `cdylib` considerably more useful than treating it simply as a way to compile a Rust shared library.