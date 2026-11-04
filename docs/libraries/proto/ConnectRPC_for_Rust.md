# ConnectRPC for Rust — Comprehensive Library and Ecosystem Overview

Research date: October 31, 2026 | Version scope: ConnectRPC 0.9.x

ConnectRPC for Rust is a Tower-based RPC implementation that exposes Protocol Buffers services through three interoperable protocols: Connect, gRPC, and gRPC-Web. It provides typed asynchronous Rust services and clients, all four RPC streaming patterns, configurable transport and message handling, and integration with existing HTTP infrastructure.

Its defining architectural feature is that a single generated service implementation can serve clients using all three protocols, with the runtime handling protocol detection, encoding, framing, status translation, compression, deadlines, and dispatch.

The ecosystem also provides first-party libraries for protobuf code generation, standard gRPC health checking, and runtime service reflection. These allow protocol infrastructure, service discovery, and operational interfaces to be assembled largely from existing library capabilities rather than custom RPC machinery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



## 1. Scope, maturity, and ecosystem

### 1.1 Verified release versions

| Package                 | Latest verified release | Responsibility                             |
| ----------------------- | ----------------------- | ------------------------------------------ |
| `connectrpc`            | 0.9.1                   | Runtime, protocols, dispatch, transport    |
| `connectrpc-build`      | 0.9.0                   | Cargo build-time protobuf generation       |
| `connectrpc-codegen`    | 0.9.0                   | Service-stub generator and `protoc` plugin |
| `connectrpc-health`     | 0.9.0                   | gRPC health checking                       |
| `connectrpc-reflection` | 0.9.0                   | gRPC schema reflection                     |

The project targets Rust 1.88 or newer, uses the Rust 2024 edition, and remains pre-1.0. Minor releases can introduce breaking API changes.

The 0.9.1 runtime release, dated September 21, 2026, includes a security fix for prematurely terminated client-streaming and bidirectional-streaming calls that could otherwise retain stalled request-body readers and HTTP/2 resources. Version 0.9.1 is therefore the appropriate baseline within the 0.9 release family.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+3



Primary repository:\
[https://github.com/connectrpc/connect-rust](https://github.com/connectrpc/connect-rust?utm_source=chatgpt.com)

Version-specific user guide:\
[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

Release history:\
[https://github.com/connectrpc/connect-rust/blob/main/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/main/CHANGELOG.md?utm_source=chatgpt.com)

### 1.2 Ecosystem architecture

Protocol Buffers schemas

Service definitions, messages, RPC cardinality, descriptors

connectrpc-build

Cargo build.rs integration

connectrpc-codegen

Standalone plugin and generation API

Generated Rust bindings + Buffa message types

Service traits, clients, registration, metadata, owned messages and borrowed views

connectrpc runtime

Tower service · router · dispatcher · interceptors · codecs · HTTP transports

Connect

gRPC

gRPC-Web

connectrpc-health

Check / Watch; service readiness

connectrpc-reflection

Schema discovery and descriptors

Conceptually, the ecosystem separates into three layers:

- Schema and generation: `.proto` definitions become Rust message types, service interfaces, client bindings, and metadata.
- RPC execution: the runtime implements protocol interoperability, asynchronous request handling, streaming, and HTTP transport integration.
- Operational services: health and reflection are ordinary RPC services registered alongside application services, rather than separate server implementations.

The message representation is supplied primarily by the companion Buffa ecosystem. ConnectRPC does not require its own separate protobuf message model or a parallel reflection system.

## 2. Core runtime: `connectrpc`

API reference:\
[https://docs.rs/connectrpc/0.9.1/connectrpc/](https://docs.rs/connectrpc/0.9.1/connectrpc/?utm_source=chatgpt.com)

### 2.1 Protocol interoperability

The runtime performs protocol selection at the HTTP boundary. Application handlers operate on generated Rust types and RPC metadata rather than manually processing protocol-specific wire formats.

| Capability              | Connect                 | gRPC                           | gRPC-Web             |
| ----------------------- | ----------------------- | ------------------------------ | -------------------- |
| Unary RPC               | Yes                     | Yes                            | Yes                  |
| Server streaming        | Yes                     | Yes                            | Yes                  |
| Client streaming        | Yes                     | Yes                            | Yes\*                |
| Bidirectional streaming | Yes\*                   | Yes                            | Yes\*                |
| Binary protobuf         | Yes                     | Yes                            | Yes                  |
| Protobuf JSON           | Yes                     | Supported by runtime           | Supported by runtime |
| Unary HTTP GET          | Yes, idempotent methods | No                             | No                   |
| HTTP/1.1 compatibility  | Yes                     | No, standard gRPC needs HTTP/2 | Yes                  |

\*Protocol support is distinct from browser transport capabilities. Browsers cannot generally stream request bodies through the provided Fetch integration, and true full-duplex bidirectional streaming requires HTTP/2. Some streaming patterns are therefore usable with native clients but not directly with browser clients.

The Connect protocol has several important properties:

- Unary calls use conventional HTTP POST requests with bare protobuf or JSON bodies, avoiding mandatory gRPC-style message framing.
- Methods declared idempotent can also use HTTP GET, allowing URL-addressable read operations and potential HTTP caching.
- Streaming messages use length-delimited envelopes and an explicit end-of-stream response, rather than depending on HTTP trailers.
- Error codes map to meaningful HTTP statuses for unary Connect calls.
- Streaming and unary calls remain part of the same protobuf service contract.

The ConnectRPC repository reports passing 3,600 server and 6,872 client conformance cases across the three protocols. These are upstream conformance results, not a guarantee of interoperability with every proxy, browser, middleware stack, or HTTP deployment.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Protocol specifications:\
[https://connectrpc.com/docs/protocol/](https://connectrpc.com/docs/protocol/?utm_source=chatgpt.com)

### 2.2 Tower-native execution architecture

The runtime is organized around `tower::Service`, making RPC handling a composable HTTP service rather than requiring a proprietary server lifecycle.

The principal abstractions are:

| Abstraction             | Role                                                                      |
| ----------------------- | ------------------------------------------------------------------------- |
| `Router`                | Maps procedure paths to registered service handlers                       |
| `ConnectRpcService`     | Tower-compatible HTTP RPC dispatcher                                      |
| `Server`                | Optional standalone Hyper-based server                                    |
| `RequestContext`        | Per-request protocol, metadata, deadline, peer information and extensions |
| `ServiceRequest<'_, M>` | Borrowed, view-backed unary request                                       |
| `Response<B>`           | Response body, headers, trailers and compression policy                   |
| `ServiceResult<B>`      | `Result<Response<B>, ConnectError>`                                       |
| `Spec`                  | Static service-method metadata                                            |
| `Limits`                | Resource ceilings for body reception and message decoding                 |
| `DeadlinePolicy`        | Server-controlled request and stream timeouts                             |

There are three primary hosting patterns.

Axum integration. `Router::into_axum_service()` produces a service suitable for `axum::Router::fallback_service`, while `into_axum_router()` constructs an Axum router directly. This allows RPC methods and ordinary HTTP routes to coexist behind one listener and middleware stack.

Standalone Hyper server. The optional `server` feature enables `Server::new(router)`, with support for HTTP/1.1, HTTP/2, TLS where enabled, and graceful shutdown.

Custom Tower/Hyper integration. `ConnectRpcService` can be supplied to a custom Hyper accept loop through `hyper_util::service::TowerToHyperService`. This gives access to lower-level HTTP/2 controls including flow-control windows, concurrency limits, frame sizes and keepalive behavior.

The distinction is useful: ConnectRPC handles RPC semantics, while Tower, Axum, Hyper and rustls remain responsible for their respective middleware, HTTP server and TLS machinery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 2.3 Typed service contracts and borrowed message views

Generated service traits expose asynchronous methods with handler signatures determined by the protobuf service definition.

For unary requests, a handler typically receives:

```
async fn greet(
    &self,
    ctx: RequestContext,
    req: ServiceRequest<'_, GreetRequest>,
) -> ServiceResult<GreetResponse>
```

`ServiceRequest` dereferences to a Buffa message view. String fields can therefore be read as `&str` borrowed from the retained message buffer rather than allocated as separate owned `String` values.

The distinction between message representations is consequential:

| Representation              | Properties                               | Suitable operations                                 |
| --------------------------- | ---------------------------------------- | --------------------------------------------------- |
| `ServiceRequest<'_, M>`     | Borrows dispatcher-owned request storage | Synchronous/asynchronous inspection during the call |
| `MView<'a>`                 | Typed borrowed Buffa view                | Zero-copy field access                              |
| `OwnedView<MView<'static>>` | Owns backing bytes while exposing a view | Retaining messages, pass-through responses          |
| `M`                         | Conventional owned Rust message          | Mutation, independent ownership, long-lived state   |
| `StreamMessage<M>`          | Owns a streaming item's backing buffer   | Streaming processing, buffering, task transfer      |

`ServiceRequest` exposes `view()`, `bytes()`, `to_owned_view()` and `to_owned_message()`. The owned-view path is particularly valuable for message forwarding: the buffer can remain reference-counted instead of copying every string or byte field into an owned protobuf structure.

A borrowed request cannot directly escape its lifetime into a spawned task or returned stream. The owning variant is required when an operation must outlive the original handler call.

#### Response views and encoding polymorphism

Generated handler interfaces use the runtime's `Encodable<M>` abstraction, allowing responses to be represented by an owned message or a compatible view-backed object.

Relevant capabilities include:

- `OwnedFooView` aliases generated for relevant RPC types.
- `Encodable<M>` implementations that can encode views without constructing the corresponding owned message.
- `PreEncoded` and `MaybeBorrowed` mechanisms for conditional or pre-existing message representations.
- `Response::with_header`, `with_trailer` and `compress` for returning metadata and controlling encoding behavior.

This can simplify proxies, validators, protocol adapters and read-mostly handlers that often return a message substantially unchanged.

Important limitation: these view-body response paths are principally protobuf-binary paths. They do not automatically provide JSON serialization for arbitrary borrowed views. The handler must select a representation compatible with its desired response codec.

Also, “zero-copy” does not mean decoding is free. The runtime still has to parse and validate protobuf wire structure; ownership of buffers and avoidance of per-field allocations are the important optimizations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 2.4 Streaming: four RPC execution shapes

All four protobuf RPC cardinalities are supported, with distinct Rust execution interfaces.

| RPC shape        | Server input                 | Server output                 |
| ---------------- | ---------------------------- | ----------------------------- |
| Unary            | Borrowed `ServiceRequest`    | Single response               |
| Server streaming | Borrowed `ServiceRequest`    | `ServiceStream<ResponseType>` |
| Client streaming | `InboundStream<RequestType>` | Single response               |
| Bidirectional    | `InboundStream<RequestType>` | `ServiceStream<ResponseType>` |

The streaming APIs use `futures::Stream` and boxed `Send` streams, allowing interoperability with channels, combinators and other asynchronous stream-producing libraries.

Inbound streams yield `StreamMessage<M>` objects. These own their buffers, can expose fields through view accessors and can be converted to owned messages.

Outbound streams yield response messages or `ConnectError` values. A handler can construct streams through `futures::stream`, channel receivers or other compatible producers.

Generated clients provide stream-specific interfaces:

- Server streaming: repeated `.message().await?`.
- Client streaming: an asynchronously produced request stream.
- Bidirectional streaming: `.send()`, `.message()`, `.close_send()`, and `.into_split()`.

The `BidiStream::into_split()` capability, introduced in 0.9.0, is particularly important. It separates the sending and receiving halves into independently owned handles, allowing separate async tasks to drive both directions without adding an application-level mutex around the entire stream.

A clean `Ok(None)` from `message()` means the response stream completed normally. A terminal RPC error is returned as `Err`, including certain malformed or incomplete gRPC stream terminations. This distinction prevents stream truncation from being silently interpreted as a successful completion.

For client streaming, a partially failed upload similarly yields an error rather than a normal end-of-stream marker. Dropping a client-streaming future cancels the outstanding upload.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 2.5 Typed middleware versus HTTP middleware

ConnectRPC supports two complementary middleware layers.

| Property                             | Tower middleware                                      | ConnectRPC interceptor                                         |
| ------------------------------------ | ----------------------------------------------------- | -------------------------------------------------------------- |
| Execution level                      | HTTP service                                          | RPC invocation                                                 |
| Main inputs                          | HTTP request/response                                 | `RequestContext`, `Spec`, `Payload`                            |
| Understands protobuf method identity | Only by parsing HTTP request information              | Yes                                                            |
| Message access                       | Raw HTTP body                                         | Codec-aware lazy `Payload`                                     |
| Main capabilities                    | Authentication, HTTP tracing, routing policy, headers | RPC authorization, validation, instrumentation, transformation |
| Can terminate an RPC                 | Yes                                                   | Yes                                                            |
| Can replace a response               | Yes                                                   | Yes                                                            |
| Streaming participation              | HTTP body transformation                              | RPC stream transformation                                      |

The released 0.9.x `Interceptor` abstraction provides two main hooks:

- `intercept_unary`: receives a unary request and `Next` continuation.
- `intercept_streaming`: receives streaming request metadata, an inbound `PayloadStream` and `NextStream`.

Interceptors run in registration order. Their continuation objects allow forwarding, short-circuiting, substituting a result, or—in the unary case—re-running an invocation when permitted by the method's idempotency contract.

An interceptor can call `Payload::message::<M>()` to obtain a typed message. Decoding is lazy and cached, reducing redundant decoding when the handler subsequently accesses the same payload. `Payload::set_message()` allows an interceptor to replace the message before handler invocation.

For streaming calls, the interceptor runs once at stream establishment. Per-message behavior is achieved by wrapping the inbound or outbound `Stream` with adapters; there is not a separate interceptor invocation for every streaming message.

Security and resource distinction: in the released 0.9.1 execution path, RPC interceptors execute after relevant request-body reading and decompression. An HTTP-level Tower authentication layer can reject unauthorized clients before that work occurs. This is especially important for untrusted callers and large request bodies.

The upstream development branch has evolved this area further; in particular, the `intercept_head` mechanism visible on `main` is not treated here as a released 0.9.1 API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+4



### 2.6 Static RPC metadata and generated constants

Generated code includes a static `Spec` for every method, with the method's fully qualified procedure path, streaming shape, idempotency level and origin.

`Spec` provides a compact bridge between the protobuf contract and runtime policy:

- `Spec::procedure` identifies the method without reconstructing the URI.
- `StreamType` distinguishes unary, client-streaming, server-streaming and bidirectional methods.
- `IdempotencyLevel` carries protocol-declared information relevant to GET calls and safe retries.
- `SpecOrigin` distinguishes the client and server representations.

The method metadata is `Copy` and uses static data; it is suitable for lookup tables, policy associations, instrumentation and validation.

`RequestContext::spec()` exposes the registered method metadata during dispatch, while `RequestContext::protocol()` provides the negotiated wire protocol and `RequestContext::path()` the actual request path.

The separation is important: `Spec` describes a method; `RequestContext` describes an invocation. This avoids repeatedly extracting semantic information from HTTP requests that the code generator already knows.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 2.7 Client transports and call control

The generated clients are generic over the `ClientTransport` abstraction. A compatible Tower HTTP service can be adapted to serve as the transport.

The runtime supplies:

| Component                | Capability                                                                  |
| ------------------------ | --------------------------------------------------------------------------- |
| `HttpClient`             | Hyper-backed HTTP client with connection pooling                            |
| `Http2Connection`        | Explicit HTTP/2 connection management, appropriate for gRPC                 |
| `Http2ConnectionBuilder` | Connection, keepalive, HTTP/2 flow-control and source-address configuration |
| `ClientConfig`           | Base endpoint, protocol and default request settings                        |
| `CallOptions`            | Per-invocation overrides                                                    |
| `ServiceTransport`       | Adapter for Tower-based client transport composition                        |

`HttpClient` provides separate cleartext and TLS-backed constructors, with scheme checks that prevent accidentally using an HTTPS address with a plaintext-only client.

`Http2Connection` provides more explicit transport control, including connection sharing, HTTP/2 keepalive, adaptive flow-control windows and local source-address binding. Its builder also exposes underlying Hyper configuration for less common requirements.

The released runtime defaults to a 20-second overall connection-establishment budget, plus a 5-second per-address TCP connection timeout. Both can be adjusted or disabled through the relevant builders. These bound connection setup, not the duration of an entire RPC.

Client configuration operates at two scopes:

- Client-wide defaults: base URI, protocol, default timeout and headers.
- Per-call overrides: timeout, headers, maximum message size and relevant encoding/compression options.

A generated client exposes ordinary methods and `_with_options` variants to preserve both compact call sites and granular control.

Client response handles also support multiple access patterns: `view()` for zero-copy inspection while retaining response metadata, `into_view()` for an owned view, `into_owned()` for an allocated Rust message, and `into_owned_parts()` when both the converted message and metadata are needed.

For cross-platform integrations, `ClientTransport` is an important extension point. The upstream WebAssembly example uses generated Rust clients with a `web-sys::fetch` transport rather than the native Hyper transport.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+3



### 2.8 Errors, compression and runtime resource controls

Errors. `ConnectError` unifies server-returned and client-observed errors. It carries an `ErrorCode`, optional human-readable message, typed `ErrorDetail` values, response headers/trailers, and an optional local source error.

The dispatcher translates these errors into the appropriate protocol-specific representation, including Connect HTTP status codes, gRPC status information, and streaming terminal errors.

Local `.with_source(error)` causes are deliberately excluded from serialized responses, allowing richer internal diagnostics without automatically exposing underlying error objects to clients. Structured protobuf error details, by contrast, are designed to cross the RPC boundary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Compression. The runtime includes identity, gzip and Zstandard support, subject to Cargo feature settings. `CompressionRegistry` and `CompressionProvider` allow additional algorithms; `Response::compress` provides per-response policy overrides. Custom decompression integrates with runtime size limits rather than allowing unrestricted expansion.

Resource and deadline controls. The two primary facilities are `Limits` and `DeadlinePolicy`.

`Limits` distinguishes three different quantities:

| Limit                        | Default | Scope                                      |
| ---------------------------- | ------- | ------------------------------------------ |
| Maximum request body         | 4 MiB   | Incoming body bytes                        |
| Maximum decoded message      | 4 MiB   | Individual post-decompression message      |
| Decode element-memory budget | 32 MiB  | Memory amplification from decoded elements |

These are not interchangeable. A small compressed payload may expand beyond its message limit, and a relatively small protobuf message can expand into substantial in-memory collections.

`Router::with_route_limits()` permits individual RPC methods to replace service-wide limits with a complete method-specific profile. This is more precise than applying one maximum payload policy to every endpoint.

`DeadlinePolicy` can impose a minimum timeout, maximum timeout, default timeout, streaming-response deadline enforcement and an inter-message timeout. The last option is useful for streams that remain connected but cease producing messages.

An especially consequential detail is that, without explicit stream enforcement, the absolute deadline primarily bounds response establishment, not necessarily the entire subsequent streaming body. Application work that must survive cancellation must manage its own task lifetime and commit boundaries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 3. Code generation: `connectrpc-build` and `connectrpc-codegen`

References:\
[https://docs.rs/connectrpc-build/0.9.0/connectrpc_build/](https://docs.rs/connectrpc-build/0.9.0/connectrpc_build/?utm_source=chatgpt.com)\
[https://docs.rs/connectrpc-codegen/0.9.0/connectrpc_codegen/](https://docs.rs/connectrpc-codegen/0.9.0/connectrpc_codegen/?utm_source=chatgpt.com)

### 3.1 The two principal generation architectures

The ecosystem supports different generation workflows with equivalent application-facing service APIs.

| Dimension                      | `connectrpc-build`              | `protoc-gen-connect-rust`             |
| ------------------------------ | ------------------------------- | ------------------------------------- |
| Invocation                     | Cargo `build.rs`                | `protoc` or Buf generation            |
| Input                          | Proto files or descriptor set   | Compiler-generated descriptor request |
| Message generation             | Unified with service generation | Normally separate Buffa plugin        |
| Output                         | `$OUT_DIR` by default           | Explicit generated source directories |
| Module integration             | `include_generated!()`          | Generated module tree                 |
| External message crates        | Less direct; unified generation | Explicit `extern_path` mapping        |
| Multi-language pipelines       | Possible but not its focus      | Works naturally with Buf pipelines    |
| Programmatic library interface | Through build configuration     | `connectrpc_codegen`                  |

This separates two concerns that are sometimes conflated:

Message generation translates protobuf message definitions into Rust data types, their view representations and serialization implementations.

Service generation translates protobuf RPC definitions into Rust handler traits, clients, registration extensions and method metadata.

`connectrpc-build` coordinates both. The standalone codegen plugin normally concentrates on service bindings and references types generated independently by Buffa.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 3.2 `connectrpc-build`: build-system integration

The configuration entry point is `connectrpc_build::Config`, which handles compiler invocation, descriptor preparation, code generation and output assembly.

Its major capability groups are:

| Configuration             | Function                                                          |
| ------------------------- | ----------------------------------------------------------------- |
| `files`, `includes`       | Select source schemas and import roots                            |
| `use_buf`                 | Use Buf's schema compilation and module resolution                |
| `descriptor_set`          | Generate from a precompiled `FileDescriptorSet`                   |
| `out_dir`, `include_file` | Control output location and generated module inclusion            |
| `emit_descriptor_set`     | Retain a descriptor set for runtime consumption                   |
| `file_per_package`        | Change output layout to one Rust file per protobuf package        |
| `generate_json`           | Enable or disable protobuf JSON bindings                          |
| `encodable_impls`         | Control which messages get view-based `Encodable` implementations |
| `gate_client_feature`     | Conditionally compile generated clients                           |
| `client_feature_name`     | Customize the Cargo feature controlling clients                   |
| `strict_utf8_mapping`     | Honor protobuf string fields that do not require UTF-8 validation |
| `buffa_config`            | Pass through broader Buffa generation settings                    |
| `emit_rerun_directives`   | Control Cargo rebuild directives                                  |
| `emit_register_fn`        | Control type-registration helper generation                       |

Three descriptor-source modes are significant:

1. `protoc`: compile source `.proto` files through the normal protobuf compiler.
2. Buf: compile using Buf's schema configuration, including its dependency resolution.
3. Precompiled descriptor set: operate from compiled schema descriptors without requiring a compiler at build time.

The descriptor-set option is useful for reproducible builds or systems in which schema compilation is a separate pipeline stage. The precompiled descriptor set must include imported definitions; otherwise, referenced types may be missing and code generation fails. Source-code information must also be retained if generated documentation should preserve protobuf comments.

`emit_descriptor_set()` can save the already-computed descriptor set for reflection, avoiding a separate compiler invocation. The output includes the transitive imports needed for resolving the reflected schema.

The original compiler-generated source information is stripped from emitted runtime descriptor sets for the `protoc` and Buf paths, reducing embedded size and limiting exposure of source comments. Precompiled descriptor sets are handled differently and are emitted without that stripping transformation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.3 `connectrpc-codegen`: reusable generator and plugin

`connectrpc-codegen` exposes generation logic directly, in addition to the `protoc-gen-connect-rust` executable.

Two important APIs distinguish its modes:

- `codegen::generate_files`: unified generation of Buffa message types and ConnectRPC service stubs.
- `codegen::generate_services`: service-only generation, referencing message types through external paths.

The plugin implements the protobuf compiler plugin mechanism and integrates with Buf, including the Buf Schema Registry remote plugin `buf.build/connectrpc/rust`.

For independently generated messages, `buffa_module` or `extern_path` mappings establish where the Rust service bindings locate their corresponding protobuf message types. The `buffa_module` form is a convenient shorthand for a catch-all protobuf package mapping.

These paths are resolved at generation time. Changing generated module placement without updating the mappings can therefore invalidate the generated references.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 3.4 Output layout, modularity and Rust ownership rules

The generator offers two main output layouts:

- Per-proto layout: multiple generated files and package-level module stitchers.
- Per-package layout: one `<dotted.package>.rs` output per protobuf package.

The second layout can simplify generated SDK packaging, but requires generation inputs to cover the complete package. Generating multiple partial fragments of the same protobuf package into a single target filename can silently overwrite earlier output.

Another consequential configuration is `EncodableImpls::AllMessages`.

Normally, view-based `Encodable` implementations are generated for RPC output message types. When protobuf messages live in a shared Rust crate and services live in other crates, Rust's orphan rules prevent a downstream service crate from implementing a foreign trait for a foreign view type.

Generating `Encodable` implementations in the message-owning crate for all message types resolves this ownership problem and allows downstream service handlers to return supported view-backed responses without reconstructing owned message values. This is an important enabling capability for independently published schema crates and modular service libraries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 3.5 Additional Buffa-driven generation capabilities

Through `buffa_config`, the build integration can expose broader message-level options, including:

- Field-specific owned representations for strings, bytes, maps, repeated fields and pointer-backed message fields.
- Attribute injection for generated message and field types.
- Unknown-field preservation.
- Lazy message views, alongside conventional eager views.
- Optional reflection and text-format support.
- Protobuf Editions feature overrides.
- Optional `arbitrary` generation for fuzz-oriented workloads.
- Explicit UTF-8-validation behavior for relevant protobuf editions and proto2 fields.

These are principally Buffa code-generation capabilities, not independent ConnectRPC protocol features. ConnectRPC service generation depends on the resulting message types and views being compatible with the runtime.

The distinction allows protobuf data representation and RPC service binding to evolve along different organizational boundaries, while retaining a shared contract.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.6 Proto-only compilation and feature gating

A protobuf-binary-only deployment can disable runtime JSON message handling and avoid generating `serde` derives for its message types.

That requires coordinated configuration:

- Generate messages with `generate_json(false)` or the plugin's `no_json` option.
- Compile `connectrpc` with its `json` feature disabled.
- Ensure the entire Cargo dependency graph does not re-enable that feature.

Cargo feature unification is important: if another dependency enables runtime JSON, serde requirements reappear and protobuf-only generated types may no longer satisfy the expected bounds.

The runtime still uses JSON internally for Connect error representations, so removing message-level JSON support does not remove every serde-related dependency.

Separately, the generator's `gate_client_feature` option makes generated client types conditional on a Cargo feature. This allows server-only builds to omit client transport machinery instead of carrying the client-facing stack merely because the generated source contains client implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



## 4. Standard gRPC health checking: `connectrpc-health`

API reference:\
[https://docs.rs/connectrpc-health/0.9.0/connectrpc_health/](https://docs.rs/connectrpc-health/0.9.0/connectrpc_health/?utm_source=chatgpt.com)

`connectrpc-health` implements the standard `grpc.health.v1.Health` protobuf service, making health information available through the same ConnectRPC server and protocols as the application itself.

It provides two RPC methods with different operational roles:

| RPC     | Semantics                                                  | Primary consumers                                                           |
| ------- | ---------------------------------------------------------- | --------------------------------------------------------------------------- |
| `Check` | Returns one health status for a named service              | Kubernetes gRPC probes, `grpc_health_probe`, diagnostic tools               |
| `Watch` | Returns an initial status, then streams subsequent changes | gRPC client-side load balancing, service meshes, long-lived health monitors |

This is gRPC health checking, not an ordinary HTTP `/health` endpoint. An HTTP health endpoint remains a separate Axum or HTTP-router concern.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.1 Health model and service granularity

Health checking uses fully qualified protobuf service names, such as `acme.user.v1.UserService`.

The empty service name, `""`, represents whole-server health. The default static checker initializes it to `Serving`, allowing probes that do not specify a service to work without additional registration.

Each named service can report a status independently. This enables a server hosting multiple RPC services to communicate different availability states rather than exposing only one process-level Boolean.

The `Status` abstraction represents gRPC health status values, including `Serving` and `NotServing`. The checker's reported status is application-controlled. The library does not automatically inspect database connectivity, dependency health, queue saturation or application readiness.

That separation makes health checking broadly reusable but leaves the meaning of readiness under the application's control.

### 4.2 `StaticChecker`: dynamic state without custom RPC handlers

The most direct implementation uses `StaticChecker`, a thread-safe in-memory registry of service names and status values.

The `install_static` helper constructs the checker, registers the health RPCs on a `Router`, and returns a shared handle for subsequent status updates.

```
use connectrpc::Router;
use connectrpc_health::{install_static, Status};

let (router, health) = install_static(
    Router::new(),
    ["acme.user.v1.UserService"],
);

// Change the reported availability.
health.set_status(
    "acme.user.v1.UserService",
    Status::NotServing,
)?;

// During graceful shutdown:
health.shutdown();
```

Illustrative fragment based on the documented API, not a standalone executable. Generated `*_SERVICE_NAME` constants are preferable to literal service names.

Beyond `set_status`, the checker exposes useful lifecycle operations:

| API                     | Behavior                                                                      |
| ----------------------- | ----------------------------------------------------------------------------- |
| `new` / `with_services` | Construct a registry with the whole-process entry and optional named services |
| `register`              | Add a service initially reporting `Serving`                                   |
| `register_with_status`  | Register a service with a specified initial status                            |
| `set_status`            | Update a previously registered service                                        |
| `remove_service`        | Remove a service and terminate its existing health watches                    |
| `services`              | Retrieve the currently registered service names                               |
| `shutdown`              | Change all registered service statuses to `NotServing`                        |

The registry is intentionally strict: `set_status` returns `UnknownServiceError` for unregistered names rather than silently creating new entries.

This catches service-name mistakes and distinguishes registration from state transition.

`register_with_status` is useful when a service should exist in the health registry before becoming available. It avoids initially reporting `Serving` during initialization.

`shutdown` changes the current registry entries; it does not permanently prohibit later registrations from reporting `Serving`. The application still controls the surrounding shutdown lifecycle.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.3 Watch-based coordination

For `StaticChecker`, `Watch` is backed by an asynchronous status-notification mechanism.

A new watcher receives the current health status, followed by updates when the value changes. Redundant transitions are suppressed, and updates can be coalesced if a consumer is slow.

This means `Watch` is a latest-state subscription, not an event log. A client should not interpret every received item as one unique transition with complete historical coverage.

This distinction matters for long-lived control-plane consumers:

- Health-aware load balancers can maintain a current picture of server availability without repeated polling.
- Service meshes can react to status changes with lower coordination overhead.
- The application does not need a separate subscription registry, websocket channel or notification broker solely to implement health watches.

Removing a registered service terminates its existing watch streams; subsequent requests for that service return a not-found error.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.4 Custom health policies through `Checker`

`Checker` separates the gRPC wire service from the logic that determines health.

It provides `check` and `watch` behavior. `HealthService::new` and `HealthService::from_arc` adapt a custom checker into the registered RPC service.

A custom checker can derive status from operational state, such as dependency availability or an application-controlled draining flag, without rewriting protobuf handling or protocol responses.

However, the default implementation of `Checker::watch` returns `Unimplemented`. A custom checker that implements only `check` works with ordinary unary health probes but not with consumers that require streaming health updates.

That distinction is important: a component may pass Kubernetes `grpc:` probes while still being unsuitable for a gRPC client configured for Watch-based health checking.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.5 Unknown services and specification differences

For a nonempty unregistered service name, the crate returns `ConnectError::not_found(...)` from both `Check` and `Watch`.

The gRPC health specification also describes a `SERVICE_UNKNOWN` keep-stream-open behavior for an unknown service in `Watch`. The Rust implementation does not implement that particular behavior; it follows the analogous Connect-Go health implementation.

Ordinary probes that interpret an error as unhealthy continue to work. Consumers explicitly depending on the `SERVICE_UNKNOWN` subscription semantics need to account for the difference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 4.6 Resource limits and feature boundaries

`install_static` applies a specialized request-limit profile to the health `Check` and `Watch` routes:

- Maximum request message: 16 KiB.
- Framed request bodies: allowance includes the envelope overhead.
- Decode element-memory limits: inherited from normal runtime defaults.

A manually registered `HealthService` does not automatically receive that profile. The crate exposes `request_limits()` and `apply_request_limits(router, limits)` to apply or customize it.

The optional `HealthClient` is useful for integration tests, internal probes and monitoring tools. The crate's `client` feature is enabled by default and pulls in the runtime's HTTP client transport. Server-only users can disable it with `default-features = false`, subject to Cargo-wide feature unification.

The `wire` module re-exports standard health request/response types and method metadata, so an independent caller need not regenerate `grpc.health.v1` solely to speak the health protocol.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 5. Runtime schema discovery: `connectrpc-reflection`

API reference:\
[https://docs.rs/connectrpc-reflection/0.9.0/connectrpc_reflection/](https://docs.rs/connectrpc-reflection/0.9.0/connectrpc_reflection/?utm_source=chatgpt.com)

`connectrpc-reflection` implements the gRPC server reflection protocol. Its purpose is to make protobuf schemas discoverable to authorized clients that do not already possess the local `.proto` files.

It supports both:

- `grpc.reflection.v1.ServerReflection`
- `grpc.reflection.v1alpha.ServerReflection`

The protocol's principal operation, `ServerReflectionInfo`, is bidirectional streaming. Reflection-aware tools can query schema information over an open stream and use the returned descriptors to construct requests dynamically.

This supports `grpcurl`, `buf curl`, `grpcui`, Postman and other reflection-capable tools without writing custom schema-discovery endpoints.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.1 Descriptor-backed discovery

The central abstraction is `Reflector`, an index of protobuf file descriptors. `ReflectionService` exposes that index through the standard reflection RPC.

The reflection protocol supports discovery operations such as:

| Query category            | Information returned                                 |
| ------------------------- | ---------------------------------------------------- |
| List services             | Fully qualified service names                        |
| File by name              | The descriptor of a specified `.proto` file          |
| File containing symbol    | The file defining a service, message or other symbol |
| File containing extension | The file declaring an extension                      |
| Extension numbers         | The extension numbers defined for a message          |
| Dependency closure        | Descriptors needed to resolve a requested file       |

The descriptor index handles the schema-level lookup; the ConnectRPC runtime handles HTTP transport, streaming envelopes, codec behavior and RPC errors.

Consequently, schema discovery can be combined with an ordinary application server without introducing a separate reflection server process.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 Two descriptor sources

`Reflector` supports two materially different ways to acquire its schema.

A. Embedded `FileDescriptorSet` bytes

`connectrpc-build::Config::emit_descriptor_set("app.fds.bin")` writes a descriptor set during code generation, including its transitive imports.

The application can then embed those bytes:

```
use connectrpc_reflection::{install, Reflector};

let descriptor_bytes =
    include_bytes!(concat!(env!("OUT_DIR"), "/app.fds.bin"));

let reflector =
    Reflector::from_descriptor_set_bytes(descriptor_bytes)?;

let router = install(router, reflector);
```

This path does not require Buffa reflection generation. It preserves the original compiler-generated per-file descriptor bytes used to answer reflection queries.

B. Existing Buffa `DescriptorPool`

Where Buffa reflection code generation is enabled, a generated package can expose a reusable descriptor pool.

```
let reflector =
    Reflector::from_descriptor_pool(
        proto::descriptor_pool().clone()
    )?;
```

This avoids embedding and decoding a second independently emitted descriptor set.

Responses generated from a pool are re-encoded from its parsed descriptors. They remain semantically faithful, including unknown-field preservation, but are not guaranteed to be byte-identical to the compiler's original descriptor encoding.

The distinction matters when callers require exact original descriptor bytes rather than an equivalent decoded schema representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 5.3 Combining multiple descriptor sets

A less obvious capability is `Reflector::add_descriptor_set_bytes()`.

It allows a reflector constructed from descriptor bytes to incorporate additional descriptor sets. Shared imported files may appear in multiple sets; files already registered under the same name are skipped, so common protobuf imports can be merged without treating every duplicate as a new conflict.

This supports schema assembly from independently generated protobuf packages without forcing them into a single initial compiler invocation.

There are important ownership and consistency constraints:

- First registration wins for duplicate file names.
- A reflector built from a shared `DescriptorPool` cannot necessarily be extended by adding descriptor bytes; the API returns `ReflectionError::SharedPool` where the backing pool is already shared.
- An error while merging can leave part of the new set incorporated. The documentation advises discarding the reflector after a failed merge rather than assuming an atomic rollback.

These semantics make descriptor-set merging a useful composition mechanism, but not a transactional schema registry. Conflicting definitions under identical filenames still require explicit management outside the reflector.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.4 Service discovery versus schema disclosure

`Reflector::with_services()` permits the advertised `ListServices` response to be customized.

This is useful when a descriptor set contains schemas for more services than the server actually registers. `Reflector::service_names()` exposes the list that will be advertised.

The crucial distinction is that `with_services()` is not a schema-access filter.

It changes the service list returned by `ListServices`, but does not make the underlying file descriptors or symbols inaccessible through other reflection queries.

If a descriptor set contains internal messages, imported schemas or services that are not mounted on the runtime router, those definitions may still be discoverable.

Furthermore, reflection does not independently examine live router registration to guarantee that the exposed service list matches the actual application endpoints.

This produces three separate concepts:

| Concept                       | Source of truth                            |
| ----------------------------- | ------------------------------------------ |
| Callable RPC procedures       | Registered `connectrpc::Router` services   |
| Advertised service names      | `Reflector` service-list configuration     |
| Resolvable schema definitions | Descriptor files loaded into the reflector |

They can be made consistent, but they are not inherently the same registry.

Reflection should therefore be regarded as intentional schema publication. If some definitions must remain private, either restrict the supplied descriptor set or restrict access to the reflection routes. Merely removing a name from `ListServices` is insufficient.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 5.5 Self-description and version compatibility

The reflection service includes its own schema definitions and recognizes queries about `grpc.reflection.*`.

Even when no application descriptor set explicitly includes the reflection protocol itself, reflection-aware tools can discover the reflection methods needed to communicate with the server.

The `install` function registers both v1 and v1alpha services. This accommodates tools that differ in their supported version of the reflection protocol.

An application can register one version manually through the generated extension traits if only one is desired. That differs from the default installation helper in resource-limit configuration, discussed below.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 5.6 Request limits and descriptor-budget errors

The `install` helper applies a 16 KiB message-size profile to both reflection routes.

Because `ServerReflectionInfo` is bidirectional streaming, that limit applies per incoming request message, not to the accumulated size of the entire stream.

Manual service registration must separately apply limits using `apply_request_limits()`.

The size of the schema being published is a different concern. A server may legitimately return file descriptors far larger than the client's query.

When building a `Reflector`, descriptor decoding also enforces an element-memory budget through Buffa. A descriptor set exceeding that budget reports `ReflectionError::ElementBudget`.

Other construction failures include malformed encoding, unnamed files, descriptor-linking errors, and incompatible sharing during descriptor-set merging.

For unusually large schemas, the relevant distinctions are:

- The generation-time descriptor budget, which can be configured separately.
- The reflection initialization budget, which guards decoding of descriptor sets.
- The reflection request limit, which protects the server against oversized client requests.

Increasing a generation-time budget does not automatically increase the runtime reflection budget. Removing `SourceCodeInfo` and constraining the included schema are documented ways to reduce descriptor-set size.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 5.7 Client-side reflection tooling

The crate also provides an optional `ServerReflectionClient`, gated by a default-enabled `client` feature, and a `wire` module exposing the generated reflection messages and method metadata.

This permits Rust programs to implement reflection-aware tooling using existing typed RPC clients and descriptor types rather than manually encoding the reflection protocol.

Examples of general-purpose capabilities enabled by these components include dynamic schema inspection, contract comparison, integration-test discovery and tooling that constructs requests from reflected schemas.

The library provides the protocol and descriptors; it is not itself a complete interactive API explorer, schema registry, compatibility checker or OpenAPI generator. Those higher-level functions require companion tooling.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 6. How the components compose

The most substantial architectural advantage comes from the interaction of generation, runtime metadata and operational services.

### 6.1 One protobuf schema, several operational representations

A single protobuf service definition can produce:

| Result                            | Mechanism                           | Function                                     |
| --------------------------------- | ----------------------------------- | -------------------------------------------- |
| Owned messages and borrowed views | Buffa                               | Typed data representation                    |
| Rust service traits               | ConnectRPC codegen                  | Handler contracts                            |
| Typed client methods              | ConnectRPC codegen                  | Client invocation                            |
| Static `Spec` metadata            | ConnectRPC codegen                  | Method identity, streaming type, idempotency |
| Runtime dispatch routes           | `Router` and generated registration | Callable RPC endpoints                       |
| Binary descriptor set             | `emit_descriptor_set`               | Runtime schema discovery                     |
| Health service names              | Generated constants                 | Precise health registration                  |

This integration offers opportunities to avoid maintaining separate procedure-name strings, handwritten RPC route tables, bespoke health handlers and separately maintained API-description files.

However, common schema origin does not imply live synchronization. Health registration, reflection publication and runtime route registration are separate steps, and each can become inconsistent if assembled from different schema sets.

### 6.2 A coherent service composition

For a typical server, the arrangement is conceptually:

```
Compiled protobuf descriptors
    ├── Buffa messages and views
    ├── ConnectRPC service traits and clients
    ├── Static RPC metadata
    └── FileDescriptorSet or DescriptorPool
                  │
                  ▼
         ConnectRPC application Router
                  │
                  ├── Application RPC services
                  │     ├── Unary
                  │     └── Streaming
                  │
                  ├── grpc.health.v1.Health
                  │     ├── Check
                  │     └── Watch
                  │
                  └── grpc.reflection.*
                        └── ServerReflectionInfo
                  │
                  ▼
       Tower / Axum / Hyper transport
                  │
                  ▼
           Connect / gRPC / gRPC-Web
```

All registered RPCs pass through the common protocol dispatcher. Health and reflection do not require independent gRPC network listeners.

The underlying HTTP integration still determines TLS configuration, CORS, network exposure, authentication and connection-level resource controls. In particular, browser-facing and native gRPC deployments may require different HTTP transport settings even when they share the same generated service implementation.

### 6.3 Shared data structures and reduced conversion cost

A useful composition is:

Buffa borrowed message views → ConnectRPC typed handlers → view-backed responses → generated clients.

For workloads that inspect data without changing it, this can avoid creating independently owned Rust strings, byte vectors and maps at every intermediate boundary.

For RPC gateways or validation-oriented services, retaining buffers and returning view-backed messages may reduce allocation overhead.

But the benefit depends on the message's lifecycle:

- A borrowed request view remains tied to the handler's input lifetime.
- Independent asynchronous tasks require owned values or owning views.
- Message mutation may make an owned representation more appropriate.
- JSON output requires supported JSON serialization paths.
- Cross-service forwarding still incurs network framing and serialization costs.

The gain is therefore not an unconditional elimination of copies. It is the ability to defer or avoid unnecessary owned-materialization steps while preserving typed protobuf access.

### 6.4 Operational controls are orthogonal

The runtime's cross-cutting mechanisms compose with health and reflection, but operate at different layers.

| Concern                                       | Appropriate library mechanism                   |
| --------------------------------------------- | ----------------------------------------------- |
| Early authentication before body reading      | Tower HTTP middleware                           |
| RPC-aware authorization or payload validation | ConnectRPC interceptor                          |
| Method identity and metadata                  | Generated `Spec`                                |
| Request body/message ceilings                 | `Limits`, including per-route overrides         |
| Execution duration and stalled streams        | `DeadlinePolicy`                                |
| Application availability                      | `Checker`, `StaticChecker`, Health `Watch`      |
| Runtime schema disclosure                     | `Reflector` and reflection route access control |
| HTTP transport capacity and TLS               | Hyper/Axum/rustls configuration                 |

An important practical implication is that operational RPCs deserve their own access policies. Reflection and health often have different intended audiences and different confidentiality requirements from application RPCs.

## 7. Performance, compatibility, and testing

### 7.1 Performance mechanisms

ConnectRPC's upstream benchmarks compare the runtime with Tonic 0.14, using Buffa for the ConnectRPC implementation and Prost for Tonic.

Selected published results:

| Benchmark                                    | ConnectRPC | Tonic 0.14 |
| -------------------------------------------- | ---------- | ---------- |
| Small unary, latency                         | 87.6 µs    | 170.8 µs   |
| Unary, 50 log records, latency               | 195.0 µs   | 338.5 µs   |
| Client streaming, 10 messages, latency       | 166.1 µs   | 223.8 µs   |
| Server streaming, 10 messages, latency       | 109.8 µs   | 110.1 µs   |
| Small echo, 256 concurrency, requests/sec    | 252,000    | 247,167    |
| Log ingestion, 256 concurrency, requests/sec | 112,027    | 84,171     |

Upstream benchmark results on Intel Xeon Platinum 8488C hardware. Latency tests use concurrency 1; the quoted throughput tests use the upstream configuration, including multiple HTTP/2 connections. These are workload-specific measurements, not universal performance guarantees.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

Three implementation mechanisms are particularly relevant:

Borrowed protobuf views. The server avoids allocating separate owned string and map data merely to read fields. This has the greatest potential benefit for decode-heavy workloads.

Generated dispatch. Service generation produces method-specific dispatch code, reducing reliance on repeated dynamic handler dispatch for the common path.

Protocol-specific framing. Connect unary requests can use simpler HTTP bodies than gRPC unary requests, and stream-message batching can reduce transport write overhead.

These benefits can become less important when business logic dominates, compression costs outweigh serialization, downstream services are slow, or network latency determines end-to-end response time.

The benchmark's relatively similar server-streaming latency illustrates why performance conclusions should remain tied to the relevant operation rather than extended across all RPC shapes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 7.2 Interoperability testing

The upstream project reports passing all 3,600 server and 6,872 client conformance cases, covering Connect, gRPC and gRPC-Web.

This establishes substantial protocol compatibility within the tested conformance suite.

It does not establish compatibility with every application gateway, HTTP proxy configuration, browser transport or third-party streaming implementation.

The source repository includes dedicated test and demonstration paths for:

- Protocol conformance across server and client variants.
- TLS and mutual TLS.
- Tower middleware and request identity propagation.
- Streaming RPC behavior.
- WebAssembly client transport.
- Reflection using descriptor-set and descriptor-pool sources.
- Multi-service routing and integration with external tools.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Docs.rs+1



### 7.3 Version-sensitive integration points

For the researched 0.9 release family, the main compatibility boundaries are:

| Boundary                              | Relevant constraint                                                        |
| ------------------------------------- | -------------------------------------------------------------------------- |
| Runtime ↔ generated service bindings  | Use compatible `connectrpc` and codegen releases                           |
| Service bindings ↔ messages           | Generated code expects compatible Buffa message and view traits            |
| Runtime ↔ feature configuration       | Cargo additive feature unification can re-enable JSON or client transports |
| Protobuf package ↔ Rust module path   | Generated external-path mappings must match the actual module layout       |
| Reflection ↔ schema build             | Descriptor closure and registered services must be coordinated             |
| Rust toolchain                        | MSRV Rust 1.88                                                             |
| Development branch ↔ published crates | APIs present on `main` may not exist in published 0.9.x releases           |

The repository's development branch already contains API evolution beyond the published 0.9.1 runtime. Consequently, tagged documentation and version-specific Rustdoc should be preferred over unversioned source when making implementation decisions.

The 0.9.1 client-streaming resource fix is also a reminder that pre-1.0 compatibility and operational correctness need to be assessed separately.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



## 8. Key architectural distinctions

The following distinctions summarize where the ecosystem offers particular leverage and where implementation responsibilities remain external.

| Distinction                                                           | Consequence                                                                                     |
| --------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| ConnectRPC is a Tower-based RPC runtime, not a separate web framework | It composes with Axum, Hyper and existing HTTP infrastructure                                   |
| Code generation is separate from protobuf serialization               | Service APIs can be generated independently, but compatible Buffa types remain necessary        |
| Generated metadata differs from live invocation metadata              | `Spec` is static; `RequestContext` carries per-call state                                       |
| Message views differ from owned messages                              | Borrowed access can reduce allocations, but independent lifetimes require ownership             |
| Streaming transport differs from browser capability                   | Native clients support full streaming patterns; browser transports impose restrictions          |
| Interceptors differ from Tower middleware                             | Protocol-aware handling can occur after body reception; earlier rejection belongs at HTTP level |
| Health is declared state, not automatic dependency monitoring         | Applications decide when named services are ready or draining                                   |
| Health `Watch` is state subscription, not an audit stream             | Intermediate transitions may be coalesced                                                       |
| Reflection disclosure differs from service advertisement              | `with_services` affects `ListServices`, not descriptor accessibility                            |
| Schema generation differs from descriptor publication                 | Generated code alone does not imply reflection is enabled or exposed                            |
| Per-route limits differ from global limits                            | Operational methods can be constrained without imposing the same ceiling on application uploads |

The broader capability is a relatively unified protobuf RPC system: generated Rust contracts, one tri-protocol dispatcher, standard health and reflection services, and composable asynchronous transport and policy mechanisms.

Its primary limitations are not a lack of RPC functionality, but the boundaries between generated types, ownership, HTTP transport, runtime configuration and operational metadata. Those boundaries determine how much integration remains necessary.

## Primary source map

All URLs below are direct, independently accessible references.

| Resource                               | URL                                                                                                                                                                                             |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Version-specific ConnectRPC user guide | [https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)                             |
| ConnectRPC runtime API                 | [https://docs.rs/connectrpc/0.9.1/connectrpc/](https://docs.rs/connectrpc/0.9.1/connectrpc/?utm_source=chatgpt.com)                                                                             |
| ConnectRPC build integration API       | [https://docs.rs/connectrpc-build/0.9.0/connectrpc_build/](https://docs.rs/connectrpc-build/0.9.0/connectrpc_build/?utm_source=chatgpt.com)                                                     |
| ConnectRPC codegen API                 | [https://docs.rs/connectrpc-codegen/0.9.0/connectrpc_codegen/](https://docs.rs/connectrpc-codegen/0.9.0/connectrpc_codegen/?utm_source=chatgpt.com)                                             |
| Health service API                     | [https://docs.rs/connectrpc-health/0.9.0/connectrpc_health/](https://docs.rs/connectrpc-health/0.9.0/connectrpc_health/?utm_source=chatgpt.com)                                                 |
| Reflection service API                 | [https://docs.rs/connectrpc-reflection/0.9.0/connectrpc_reflection/](https://docs.rs/connectrpc-reflection/0.9.0/connectrpc_reflection/?utm_source=chatgpt.com)                                 |
| Connect protocol specification         | [https://connectrpc.com/docs/protocol/](https://connectrpc.com/docs/protocol/?utm_source=chatgpt.com)                                                                                           |
| Upstream changelog                     | [https://github.com/connectrpc/connect-rust/blob/main/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/main/CHANGELOG.md?utm_source=chatgpt.com)                                   |
| Upstream examples                      | [https://github.com/connectrpc/connect-rust/tree/v0.9.1/examples](https://github.com/connectrpc/connect-rust/tree/v0.9.1/examples?utm_source=chatgpt.com)                                       |
| Standard gRPC reflection protobuf      | [https://github.com/grpc/grpc-proto/blob/master/grpc/reflection/v1/reflection.proto](https://github.com/grpc/grpc-proto/blob/master/grpc/reflection/v1/reflection.proto?utm_source=chatgpt.com) |
| Standard gRPC health protobuf          | [https://github.com/grpc/grpc-proto/blob/master/grpc/health/v1/health.proto](https://github.com/grpc/grpc-proto/blob/master/grpc/health/v1/health.proto?utm_source=chatgpt.com)                 |
| Buffa protobuf implementation          | [https://github.com/anthropics/buffa](https://github.com/anthropics/buffa?utm_source=chatgpt.com)                                                                                               |

Overall: The distinguishing capabilities of ConnectRPC Rust 0.9.x are its combined Connect/gRPC/gRPC-Web protocol implementation, close integration with Buffa's owned and borrowed message representations, generated statically typed service contracts, Tower-native composability, and first-party operational services. The health and reflection crates are particularly useful because they integrate established interoperability standards directly into that same runtime, avoiding custom implementations while retaining explicit control over service availability, schema publication and resource usage.

# ConnectRPC for Rust — Advanced Capabilities and Corrections Addendum

Research date: November 3, 2026\
Version scope: `connectrpc` 0.9.1; `connectrpc-build`, `connectrpc-codegen`, `connectrpc-health`, and `connectrpc-reflection` 0.9.0\
Companion document: ConnectRPC for Rust — Comprehensive Library and Ecosystem Overview, October 31, 2026

This addendum supplements the existing reference with additional capabilities, implementation mechanisms, and consequential semantic distinctions. It addresses runtime transport behavior, service composition, connection lifecycle, resource management, interceptor capabilities, encoding efficiency, generation constraints, and testing interfaces.

All APIs described as available target the published 0.9.x releases. Development-branch functionality is distinguished explicitly. The addendum avoids restating the existing treatment of basic RPC operations, Buffa message types, code-generation workflows, health checking, and reflection.

## 1. Protocol semantics and resource-limit corrections

### 1.1 Connect GET: side-effect freedom versus idempotency

Connect HTTP GET eligibility requires a unary RPC whose protobuf method option declares `NO_SIDE_EFFECTS`:

```
rpc GetRecord(GetRecordRequest) returns (GetRecordResponse) {
  option idempotency_level = NO_SIDE_EFFECTS;
}
```

This is stronger than declaring an operation `IDEMPOTENT`.

| Protobuf idempotency level | Meaning                                                                    | Eligible for Connect GET            |
| -------------------------- | -------------------------------------------------------------------------- | ----------------------------------- |
| `IDEMPOTENCY_UNKNOWN`      | No declared idempotency guarantee                                          | No                                  |
| `IDEMPOTENT`               | Repeating the operation has the same intended effect as performing it once | No, not solely from this annotation |
| `NO_SIDE_EFFECTS`          | Operation is semantically safe, equivalent to an HTTP safe method          | Yes, for unary methods              |

An idempotent state-changing operation may be safe to retry under an appropriate policy without being suitable for HTTP GET.

The Rust server automatically recognizes eligible GET requests. Generated Rust client methods do not currently select GET automatically; callers requiring GET must use a client implementation or lower-level request path that explicitly supports the Connect GET encoding. gRPC and gRPC-Web calls remain POST-based.

GET also introduces requirements beyond RPC method metadata:

- Cacheability depends on HTTP response directives such as `Cache-Control`; GET alone does not guarantee caching.
- Shared caches require particular attention to authenticated or user-specific responses.
- Stable request serialization improves cache-key consistency. Semantically equivalent messages may not necessarily produce byte-identical URLs across all codecs or implementations.
- Long encoded messages can encounter intermediary URL-length restrictions.
- GET-specific behavior must be distinguished at the HTTP layer because `RequestContext` does not directly expose the HTTP method. Tower middleware can retain `http::Method` in request extensions for handlers.

This makes GET useful for query-style RPCs exposed through caches and general HTTP infrastructure, but it does not turn arbitrary protobuf operations into safely cacheable resources.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://connectrpc.com\&sz=32)

Connect+1



Sources:

[https://connectrpc.com/docs/protocol/](https://connectrpc.com/docs/protocol/?utm_source=chatgpt.com)

[https://connectrpc.com/docs/rust/get-requests-and-caching/](https://connectrpc.com/docs/rust/get-requests-and-caching/?utm_source=chatgpt.com)

### 1.2 Distinct scopes of request-body, message, and element-memory limits

The three `Limits` controls apply to different stages of processing:

| Control                      | Default | What it bounds                                                     |
| ---------------------------- | ------- | ------------------------------------------------------------------ |
| `with_max_request_body_size` | 4 MiB   | Whole received body for unary and server-streaming requests        |
| `with_max_message_size`      | 4 MiB   | Individual decoded message size, including decompression expansion |
| `with_element_memory_limit`  | 32 MiB  | Decode-time element allocation amplification                       |

The critical distinction is that the whole-request-body limit does not cap the total accumulated size of a client-streaming or bidirectional-streaming RPC.

Unary and server-streaming methods receive one request message, so the dispatcher can apply a total body limit during reception. Client-streaming and bidirectional methods receive an incrementally consumed sequence of messages; applying the same total body ceiling would eliminate their ability to transfer arbitrarily long streams of individually valid messages.

Consequently:

- A client stream may transfer many individually valid messages whose combined size exceeds the unary request-body limit.
- Each message remains subject to the message-size and relevant decode-memory limits.
- Total upload quotas, maximum message counts, and cumulative application-resource budgets remain separate concerns.
- A stream can continue consuming resources indefinitely unless cancellation, execution deadlines, or other lifecycle controls terminate it.

Streaming RPCs also have a different authentication-cost profile. In 0.9.1, an interceptor normally sees a streaming call after the dispatcher has begun processing its input, but not after the entire client stream has been collected. The runtime may already have read or decompressed a small number of messages. By contrast, an HTTP-level Tower layer can reject the request before consuming the body.

A `Router::with_route_limits` override replaces the entire route's limit profile; it does not merge individual settings with the service defaults. Generated monomorphic dispatchers do not provide that per-route override facility.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

### 1.3 JSON decoding does not provide identical pre-allocation protection

Buffa's element-memory budget primarily protects against element amplification, where small wire encodings cause disproportionately large in-memory collections.

For binary protobuf, the 0.9.x budget is enforced during view or message decoding, including interceptor decoding. Repeated elements, maps, strings, and bytes are charged according to the relevant element footprint.

JSON follows a different processing path. The implementation uses `serde_json` to materialize an owned representation before the relevant view-decoding budget is consulted. The budget therefore provides less protection against allocations occurring during JSON parsing.

The distinction applies to server-side request decoding and client-side response decoding. A configured 32 MiB element budget must not be interpreted as a universal peak-memory ceiling, and an individual very large byte payload may not exhaust the element-count budget even though it consumes considerable memory.

The effective resource boundary is the combination of message-size ceilings, element-memory limits, HTTP transport concurrency, and application-level resource management. JSON and binary protobuf do not provide identical allocation guarantees under the same settings.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 1.4 gRPC-Web runtime support versus browser-client support

ConnectRPC Rust implements the four RPC cardinalities at the server/runtime level. This should not be interpreted as identical support across all clients speaking related protocols.

The standard `grpc-web` JavaScript browser implementation supports unary and server-streaming calls, but not client-streaming or bidirectional-streaming calls. Its server-streaming support also depends on the wire-format mode supported by that implementation.

Browser restrictions on request-body streaming independently constrain what a Fetch-based client can do, even when the underlying RPC protocol or native Rust transport supports a streaming operation.

For cross-platform compatibility, distinguish three layers:

1. RPC schema cardinality: the operations declared by the protobuf service.
2. Server/protocol runtime: the operations ConnectRPC Rust can dispatch.
3. Client and HTTP transport: which operations a particular browser, proxy, or native client can actually perform.

A service can expose a bidirectional method through the Rust runtime without making that method usable by a conventional browser gRPC-Web client.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

[https://github.com/grpc/grpc-web/blob/master/README.md](https://github.com/grpc/grpc-web/blob/master/README.md?utm_source=chatgpt.com)

[https://connectrpc.com/docs/protocol/](https://connectrpc.com/docs/protocol/?utm_source=chatgpt.com)

## 2. Dispatch architecture and composable service registration

### 2.1 Generated monomorphic dispatch versus dynamic routing

ConnectRPC exposes a lower-level `Dispatcher` abstraction beneath its HTTP service. Two dispatcher strategies are available in 0.9.1.

Generated service-specific dispatch. A generated `FooServiceServer<T>` implements `Dispatcher` and can be passed directly to `ConnectRpcService::new`. It uses statically generated procedure matching and concrete handler dispatch, avoiding the dynamic router's string-key lookup and type-erased handler dispatch.

Dynamic service routing. `Router` also implements `Dispatcher`, but maintains registrations by procedure path. It supports independently registered services, runtime composition, per-route limits, and route introspection.

These are alternative implementations of the same RPC dispatch boundary, not alternative wire protocols.

| Dimension                   | Generated `FooServiceServer<T>`     | `Router`                                    |
| --------------------------- | ----------------------------------- | ------------------------------------------- |
| Procedure dispatch          | Generated service-specific matching | Dynamic lookup by procedure path            |
| Registered service set      | Fixed by generated service type     | Composed from multiple registrations        |
| Static method metadata      | Generated directly                  | Attached during generated registration      |
| Per-route limits            | No; service-wide limits apply       | Yes, through `with_route_limits`            |
| Runtime router merging      | Not intrinsic                       | Supported                                   |
| Primary extension mechanism | Generated trait implementation      | Service registration and low-level handlers |

Both can use the surrounding `ConnectRpcService` machinery for protocol negotiation, codecs, interception, deadline enforcement, and HTTP transport integration.

The dynamic approach is not intrinsically preferable. It supplies modularity and registration flexibility at the cost of an additional dispatch layer. The generated approach is useful when a small, statically known service benefits from minimizing that machinery.

The `dispatcher` module also exposes `Chain`, which composes dispatchers by trying one and falling through to another when a procedure is not found. This enables dispatcher composition without requiring every service to be converted into registrations in one dynamic router. A custom implementation of `Dispatcher` is another extension point for specialized routing.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

[https://docs.rs/connectrpc/latest/connectrpc/dispatcher/index.html](https://docs.rs/connectrpc/latest/connectrpc/dispatcher/index.html?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

### 2.2 Router merging, conflict semantics, and modular registration

`Router` provides several forms of composition:

| API                  | Behavior                                                    |
| -------------------- | ----------------------------------------------------------- |
| `add_service`        | Registers a generated service's procedures                  |
| `merge`              | Combines routers, consuming the inputs                      |
| `merge_in_place`     | Adds another router to an existing router                   |
| `merge_routers`      | Combines a collection of routers                            |
| `try_merge`          | Fallible merge with structured collision reporting          |
| `try_merge_in_place` | Fallible in-place merge with transactional failure behavior |
| `allow_overrides`    | Enables last-wins replacement of duplicate procedure paths  |

By default, registering a procedure path already present in the destination causes a panic. The fallible variants instead return `RouterMergeError`, exposing the conflicting procedure paths.

`try_merge_in_place` is particularly consequential: its collision check is transactional, so the destination remains unchanged if the merge fails. This avoids partially registering a service collection after discovering a conflict.

`allow_overrides` is an explicit departure from collision protection. It is a property of the destination router, not a transferable policy from whichever source router happens to be merged. Last-wins semantics can be useful for intentionally replacing registrations, but can also hide accidental duplicate service definitions.

The normal `add_service` path relies on a generated `ServiceRegister` implementation. Where a single concrete Rust type implements multiple generated service traits, the marker type may be ambiguous. The corresponding generated `FooServiceExt::register` method provides explicit service selection.

These behaviors make the router a useful native service-composition registry, including for modular libraries and dynamically assembled server components, without requiring a second handwritten procedure registry.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/connectrpc/latest/connectrpc/router/struct.Router.html](https://docs.rs/connectrpc/latest/connectrpc/router/struct.Router.html?utm_source=chatgpt.com)

### 2.3 Router introspection, attached specifications, and replacement hazards

The `Router` API includes `methods()`, `has_method()`, and `with_spec()`.

`methods()` and `has_method()` inspect actually registered procedures. This is different from gRPC reflection, which exposes descriptor information, and different from the health registry, which tracks application-declared serving status.

`with_spec()` allows a low-level manually registered route to carry the same `Spec` metadata as generated registrations. It connects manually assembled handlers to interceptors and instrumentation that rely on procedure identity, streaming cardinality, or idempotency information.

Generated service registration already attaches the corresponding specifications, so manual attachment is principally relevant to lower-level registration.

Two constraints are important:

- `with_route_limits()` requires the procedure to have already been registered. It installs a complete `Limits` profile rather than overriding individual fields of the global profile.
- Overriding an existing registration can discard the previously attached per-route limits. Route configuration therefore belongs to the effective registration, not merely to the path string.

The combination of `methods()`, `has_method()`, `with_spec()`, and fallible merging permits consistency checks against expected procedure contracts without requiring reflection to be exposed on the network.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/connectrpc/latest/connectrpc/router/struct.Router.html](https://docs.rs/connectrpc/latest/connectrpc/router/struct.Router.html?utm_source=chatgpt.com)

## 3. Client transport, load management, and connection lifecycle

### 3.1 `SharedHttp2Connection`: readiness-aware, shareable HTTP/2 transport

The released client module distinguishes the general-purpose pooled `HttpClient` from the dedicated HTTP/2 transport family.

`Http2Connection` establishes a dedicated HTTP/2 connection. Calling `.shared(max_pending)` produces a cloneable `SharedHttp2Connection`, allowing generated clients to share one transport handle without an application-managed mutex around all RPC invocations.

The shared transport includes a connection/reconnection state machine and meaningful Tower readiness reporting.

By contrast, `HttpClient` wraps Hyper's general HTTP client pool and reports `poll_ready` as always ready. This is adequate for many clients but less informative to Tower load balancers that use readiness and pending-request estimates.

A minimal illustration:

```
let conn = Http2Connection::connect_plaintext(uri.clone())
    .await?
    .shared(1024);

let config = ClientConfig::new(uri)
    .with_protocol(Protocol::Grpc);

let client_a = FirstServiceClient::new(conn.clone(), config.clone());
let client_b = SecondServiceClient::new(conn, config);
```

The `shared(1024)` argument configures the shared transport's pending-request capacity; it is not an RPC payload-size limit.

Scaling across connections. A single HTTP/2 connection can encounter contention in the `h2` transport implementation even when individual RPC handlers are inexpensive. ConnectRPC's documentation describes using multiple `SharedHttp2Connection` instances, with either explicit request distribution or `tower::balance::p2c::Balance` and `tower::load::PendingRequests`, to spread load.

The key distinction is between connection pooling and load-aware dispatch:

- A pool can maintain reusable HTTP connections.
- A readiness-aware balancer can choose among connections using information about their current capacity or outstanding work.
- An always-ready transport does not, by itself, expose the same load signal.

`SharedHttp2Connection` is therefore an important native foundation for multi-connection gRPC clients. Its benefit is conditional: workloads dominated by downstream service time, network latency, or application computation may not be constrained by a single connection's internal contention.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/connectrpc/latest/connectrpc/client/index.html](https://docs.rs/connectrpc/latest/connectrpc/client/index.html?utm_source=chatgpt.com)

### 3.2 Dedicated HTTP/2 connection configuration

`Http2ConnectionBuilder` provides transport-level controls distinct from RPC method options:

- Connection-establishment deadlines and per-address TCP connection timeouts.
- HTTP/2 keepalive interval, timeout, and idle behavior.
- Initial stream and connection flow-control window sizes.
- Adaptive flow-control window management.
- Additional Hyper HTTP/2 settings through `h2_settings`.
- Source-address binding through `local_address(IpAddr)`.

The connection-establishment budget encompasses DNS resolution, TCP connection, TLS establishment where applicable, and HTTP/2 setup. The documented defaults are 20 seconds overall and 5 seconds per TCP address attempt. These are not whole-RPC execution deadlines.

`local_address` binds the built-in TCP connector to a particular source IP address. This is valuable on multi-homed hosts and for network paths where source addresses determine routing or access controls. Its behavior is limited to the built-in TCP connector, rather than applying automatically to arbitrary custom connectors or Unix-domain sockets.

Flow-control configuration interacts with memory use and throughput. Larger HTTP/2 windows permit more data in flight but can increase buffering requirements; adaptive windows attempt to respond to observed transport conditions. Neither setting replaces per-message RPC resource limits.

The API is therefore an important bridge between application-level RPC semantics and lower-level Hyper/Tokio networking behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

### 3.3 Server connection retirement and transport resource controls

The standalone `Server` exposes connection-level resource governance that operates independently of `Limits` and `DeadlinePolicy`.

| Control                            | Behavior                                                     |
| ---------------------------------- | ------------------------------------------------------------ |
| `with_max_connection_age`          | Retires connections after an age threshold                   |
| `with_max_connection_age_grace`    | Bounds the grace interval after connection retirement begins |
| `with_max_requests_per_connection` | Retires a connection after a configured number of requests   |
| `with_max_connection_idle`         | Reclaims idle connections                                    |
| `with_max_concurrent_streams`      | Controls concurrent HTTP/2 streams per connection            |
| `with_http2_keepalive_interval`    | Configures HTTP/2 keepalive probes                           |
| `with_http2_keepalive_timeout`     | Controls keepalive failure detection                         |
| `with_header_read_timeout`         | Bounds initial incomplete request/handshake behavior         |
| `with_http2_adaptive_window`       | Controls adaptive HTTP/2 flow-control windows                |

Connection retirement differs from server shutdown. An HTTP/2 connection that reaches its configured maximum age can receive GOAWAY, allowing the client to establish another connection while the server continues accepting new traffic. For HTTP/1.1, the comparable behavior involves disabling persistence and closing the connection appropriately.

The age limit incorporates jitter, reducing the risk that many connections retire simultaneously. The grace interval permits existing work to drain before forced connection closure. Request-count retirement provides another independent trigger for connection recycling.

Idle retirement also differs from keepalive. Idle retirement reclaims connections that are not carrying active work; keepalive detects peers that have become unreachable or unresponsive.

Security-sensitive handshake behavior. The 0.9.1 security fix made the default 30-second header-read timeout effective for connections that send no bytes or stall partway through the HTTP/2 preface. Without effective early connection timeouts, a peer can hold transport resources without submitting an RPC that reaches the dispatcher.

These mechanisms operate at different scopes:

- `Limits`: request and message consumption.
- `DeadlinePolicy`: RPC execution and streaming timing.
- Server connection controls: sockets, HTTP/2 sessions, idle resources, and connection retirement.

No single mechanism subsumes the others.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

[https://docs.rs/connectrpc/latest/connectrpc/server/index.html](https://docs.rs/connectrpc/latest/connectrpc/server/index.html?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 3.4 TLS-aware hosting and trusted peer identity

With the applicable `axum` and `server-tls` features, `connectrpc::axum::serve_tls` provides rustls-backed Axum hosting and propagates transport-derived information to RPC request contexts.

The relevant types and accessors are:

| API                            | Information                                          |
| ------------------------------ | ---------------------------------------------------- |
| `PeerAddr`                     | Connection peer socket address                       |
| `PeerCerts`                    | Presented TLS client certificate chain               |
| `RequestContext::peer_addr()`  | Optional typed peer address                          |
| `RequestContext::peer_certs()` | Optional typed certificate chain                     |
| `RequestContext::extensions()` | Other request-scoped data inserted by the HTTP stack |

This permits transport-level identity to participate in RPC authorization and auditing without modifying the protobuf contract or trusting arbitrary caller-supplied metadata.

The trust boundary requires precision:

- A presented certificate is not, by itself, proof of an authorized application identity. TLS verification and application authorization remain separate.
- Peer socket addresses may identify a reverse proxy rather than the ultimate caller.
- A custom Hyper accept loop must propagate trusted peer metadata into request extensions explicitly if it requires the same behavior.
- Application middleware can attach its own verified principal or identity information to `http::Extensions`, which the RPC handler can inspect through `RequestContext`.

This makes TLS hosting, Tower identity middleware, and typed `RequestContext` accessors complementary rather than interchangeable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Sources:

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/tree/v0.9.1/examples/mtls-identity](https://github.com/connectrpc/connect-rust/tree/v0.9.1/examples/mtls-identity?utm_source=chatgpt.com)

## 4. Protocol-aware middleware and encoding mechanisms

### 4.1 Protocol-correct errors before RPC dispatch

`ConnectError::into_http_response(request_headers)` is an important integration facility for Tower middleware that needs to reject an RPC before the dispatcher reads its body.

It converts a `ConnectError` into an `http::Response<ConnectRpcBody>` using protocol information from the inbound request headers.

| Detected protocol | Error representation                                    |
| ----------------- | ------------------------------------------------------- |
| Connect unary     | Appropriate HTTP error status and JSON error body       |
| Connect streaming | HTTP 200 with a terminal `EndStreamResponse` envelope   |
| gRPC              | HTTP 200 with gRPC status metadata                      |
| gRPC-Web          | HTTP 200 with the corresponding gRPC-Web status framing |

An HTTP middleware layer can therefore reject unauthenticated clients, apply request admission policy, or enforce coarse authorization without constructing three separate protocol encoders.

This is consequential because a generic HTTP 401 or 403 response does not have the same meaning to every RPC client. A gRPC client expects its protocol's status representation; returning an arbitrary HTTP response can produce a transport error instead of the intended `Unauthenticated` or `PermissionDenied` RPC status.

The method derives protocol selection from the request headers. Missing or unrecognized content types fall back to a Connect-unary JSON error response. This fallback is appropriate for ordinary HTTP rejection but should not be mistaken for fully negotiated RPC behavior.

The architectural separation is:

- Tower middleware determines whether the request should be admitted before costly body processing.
- `ConnectError::into_http_response` preserves the appropriate wire-level error contract.
- RPC interceptors handle policies that require generated method metadata or decoded message contents.

This combination avoids both early request-body processing and bespoke multiprotocol rejection logic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 4.2 Interceptor composition, typed response substitution, and invocation replay

The interceptor layer provides several advanced facilities beyond simple observation and validation.

Closure-based construction. `unary_interceptor` and `streaming_interceptor` allow interceptors to be created from closures without defining a separate struct and trait implementation. For stateful interceptors shared among multiple services, `with_interceptor_arc(Arc<dyn Interceptor>)` reuses an existing instance.

Registration order is outermost-first: the first registered interceptor observes the inbound request first and the outbound result last. With no interceptors registered, the dispatch path avoids constructing the interception machinery.

Typed response substitution. `Payload::from_message` constructs a response payload from a typed message with deferred encoding in the selected codec. An interceptor can return it without invoking the handler or inner interceptors:

```
if let Some(reply) = cache.lookup(&key) {
    return Ok(Response::new(
        Payload::from_message(reply, req.payload.format())
    ));
}

next.run(req).await
```

Illustrative fragment; the surrounding interceptor supplies `req`, `next`, and `cache`.

This supports cached responses, synthetic replies, and policy-controlled substitutions while retaining the runtime's normal protocol encoding and metadata handling. Unlike replacing an HTTP body with raw bytes, the interceptor does not need a parallel protobuf framing implementation.

Controlled re-execution. `Next` implements `Clone`, while `UnaryRequest::try_clone()` provides a way to duplicate an invocation's request context and payload for another run through the remaining interceptor chain and handler.

This can implement retry-like behavior within a server process. It is not equivalent to replaying a transport operation: it invokes application code again. Such repetition is appropriate only when the method's idempotency and side-effect behavior permit it. The declared `Spec::idempotency_level` can help evaluate eligibility but does not replace application correctness.

Streaming interception. `intercept_streaming` runs once when the stream is established. Per-message inspection or transformation requires adapting the inbound `PayloadStream` or outbound stream with `futures::Stream` combinators. This permits lazy message transformation and backpressure-preserving composition without introducing a second application streaming protocol.

The distinction between a once-per-stream hook and per-message processing matters for auditing, validation, quotas, and policies that depend on the total number of messages. A policy checked only at stream establishment cannot enforce a cumulative per-message constraint unless it also wraps the stream.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

### 4.3 Segmented protobuf encoding and transport-level copy avoidance

The 0.9.0 runtime introduced a substantial optimization through `Encodable::encode_segments` and the `EncodedBody` representation.

For eligible protobuf-binary responses, the encoder can retain large field buffers by reference count and pass them through the HTTP body layer without copying their contents into one contiguous encoded-message allocation.

The mechanism is more specific than merely returning an `OwnedView`.

1. The encoder produces protobuf framing fragments and one or more reference-counted field segments.
2. Segments at or above the 16 KiB framing threshold can be emitted as separate HTTP body frames.
3. Smaller framing fragments, including protobuf tags and length prefixes, are accumulated into a batch buffer.
4. The resulting protobuf wire bytes remain equivalent to contiguous serialization, even though their HTTP body segmentation differs.

The optimization's applicability depends on the response representation, RPC shape, codec, and compression state.

| Response path                                         | Segmented-field behavior                                                         |
| ----------------------------------------------------- | -------------------------------------------------------------------------------- |
| Server-streaming and bidirectional protobuf responses | Eligible for `encode_segments`                                                   |
| Client-streaming unary response                       | Eligible for segmented encoding                                                  |
| gRPC or gRPC-Web unary response                       | Eligible for segmented encoding                                                  |
| Connect-protocol unary response                       | Uses contiguous encoding                                                         |
| Owned message body                                    | Encodes contiguously, although whole-buffer framing may still avoid another copy |
| `OwnedView` exposing large fields                     | Can benefit from reference-counted segments                                      |
| JSON response                                         | Uses the JSON encoding path, not segmented protobuf encoding                     |
| Calls passing through an RPC interceptor              | Unary segmented encoding is not retained on that path                            |
| Compressed response                                   | Requires a contiguous representation for compression                             |

The compression interaction is particularly important.

The default `CompressionPolicy` compresses responses from 1 KiB when an accepted encoding is available. Therefore, a sufficiently large view-backed response may be compressed instead of retaining its segmented encoding. The runtime must then consolidate the message before compression, which can introduce more work than ordinary contiguous serialization.

`Response::compress(false)` permits an individual response to avoid this. `CompressionPolicy::with_min_size` provides a broader threshold adjustment.

These controls can matter when large protobuf fields already contain compressed data or when bandwidth is inexpensive relative to memory-copy and compression costs. They should not be interpreted as a universal preference for disabling compression: smaller network transfers may outweigh encoding overhead in network-constrained workloads.

Custom dispatcher compatibility. The 0.9.0 change also affected lower-level code implementing `Dispatcher` directly. Streaming results use `EncodedStream`, containing `EncodedBody` items, rather than simply a stream of `Bytes`. `EncodedBody::into_contiguous()` provides an explicit conversion back to a single buffer when required.

Generated handlers do not need to manage these representations manually, but custom dispatchers and RPC test doubles may need to accommodate the richer body type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

## 5. Client-side resource controls and streaming correctness

### 5.1 Response decode limits at client and invocation scopes

The released client API provides configurable decode-memory limits that mirror an important part of the server's request protection.

| Scope                | Message-size control                          | Element-memory control                            |
| -------------------- | --------------------------------------------- | ------------------------------------------------- |
| Client-wide defaults | `ClientConfig::with_default_max_message_size` | `ClientConfig::with_default_element_memory_limit` |
| Individual RPC       | `CallOptions::with_max_message_size`          | `CallOptions::with_element_memory_limit`          |

Per-call settings override the corresponding client defaults while leaving other default settings intact.

The element-memory budget applies to incoming responses across unary, client-streaming, server-streaming, and bidirectional calls. It constrains the element footprint of protobuf decoding, not simply the serialized byte count.

An over-budget response produces `ResourceExhausted`, making the failure distinguishable from malformed message data or a transport problem. The diagnostic identifies the relevant limit that can be adjusted.

As established in §1.3, the protection is codec-dependent. Binary protobuf enforces the element budget during decoding; JSON may already have materialized an owned message through `serde_json` before the element budget is checked.

For streaming clients, these limits protect individual decoded messages. They are not cumulative limits on every message received over the lifetime of a stream. Aggregate memory consumption also depends on how many decoded messages the application retains or processes concurrently.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

[https://docs.rs/connectrpc/latest/connectrpc/client/struct.ClientConfig.html](https://docs.rs/connectrpc/latest/connectrpc/client/struct.ClientConfig.html?utm_source=chatgpt.com)

[https://docs.rs/connectrpc/latest/connectrpc/client/struct.CallOptions.html](https://docs.rs/connectrpc/latest/connectrpc/client/struct.CallOptions.html?utm_source=chatgpt.com)

### 5.2 Asynchronous client-stream production and transport backpressure

In 0.9.0, generated client-streaming methods adopted an asynchronous request-stream interface.

`ClientRequestStream<Req>` accepts a `Stream<Item = Req> + Send + 'static`. This enables request messages to be produced as needed instead of requiring an eagerly available collection or a synchronous iterator.

The stream directly backs the HTTP request body, so the transport polls it as capacity becomes available. This allows HTTP/2 flow control to propagate backpressure to an upstream producer.

The resulting execution model is:

```
Asynchronous message producer
        ↓
ClientRequestStream<Req>
        ↓
Protobuf encoding and framing
        ↓
HTTP request body / HTTP/2 flow control
        ↓
Remote streaming handler
```

The library handles the transport-facing consumption of the stream. Application code still determines whether its own producer and any intermediate channels have bounded capacity.

A producer using an unbounded queue can accumulate substantial memory even if the HTTP transport itself applies backpressure. The benefits are strongest when the entire chain uses bounded channels or lazy streams.

`connectrpc::stream_iter` provides the corresponding adapter for an existing collection, avoiding a separate asynchronous producer when the messages are already available.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 5.3 Bidirectional half-close and cancellation semantics

`BidiStream::into_split()` returns independently owned `BidiSendHalf` and `BidiRecvHalf` handles.

The distinction between closing one direction and canceling the RPC is fundamental:

| Action                                 | Semantics                               |
| -------------------------------------- | --------------------------------------- |
| `BidiSendHalf::close_send()`           | Ends the request-message stream cleanly |
| Dropping `BidiSendHalf`                | Also closes the sending direction       |
| Continuing to read from `BidiRecvHalf` | Allows subsequent server responses      |
| Dropping `BidiRecvHalf`                | Cancels the RPC                         |

This enables independent sender and receiver tasks without adding an application-level mutex around the entire duplex stream.

However, true response-dependent full-duplex operation requires an appropriate HTTP/2 transport. On HTTP/1.1, a response may not become available until request upload completion, even when the RPC API exposes the same bidirectional abstraction.

Half-close should also not be confused with acknowledgement or durable acceptance. Successfully completing the local sending direction does not establish that the remote application has processed and committed every submitted message.

The 0.9.1 resource fix further clarifies the cancellation boundary. When a handler stops reading a client-streaming or bidirectional request, the runtime no longer retains an incomplete body reader indefinitely. It drains or discards a bounded amount before releasing the HTTP/2 stream or closing the HTTP/1.x connection as appropriate.

This prevents premature handler completion from leaving abandoned uploads indefinitely attached to transport resources. It does not provide transactional delivery semantics or automatic recovery of partially processed streams.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 5.4 Terminal stream state, sticky errors, and trailers

`ServerStream::message()` distinguishes a successful stream termination from any terminal failure:

- `Ok(Some(message))`: another message was received.
- `Ok(None)`: the protocol's successful termination conditions were satisfied.
- `Err(error)`: decoding, transport, deadline, or remote RPC termination failed.

An error is terminal and sticky. Repeated calls to `message()` return the same terminal error; they do not reinterpret a failed stream as a cleanly completed one.

`ServerStream::error()` exposes the recorded terminal failure after the fact, while `ServerStream::trailers()` exposes termination metadata once it has been received.

This matters for streams where an application has already consumed useful messages before the server reports a failure. Those previously received messages are real, but the overall RPC did not finish successfully.

Missing or malformed protocol termination metadata is not silently converted into a successful end of stream. A truncated Connect stream missing its end envelope, or a gRPC response without valid terminal status, therefore remains distinguishable from success.

The separation permits consumers to inspect partial results, terminal diagnostics, and trailers while retaining an unambiguous success criterion.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/connectrpc/latest/connectrpc/client/struct.ServerStream.html](https://docs.rs/connectrpc/latest/connectrpc/client/struct.ServerStream.html?utm_source=chatgpt.com)

### 5.5 Deadline budgeting for downstream RPCs

`RequestContext::time_remaining()` provides a saturating `Option<Duration>` representing the remaining request deadline.

Unlike manually subtracting timestamps, it accounts for elapsed time and avoids underflow. A missing deadline returns `None`, rather than implying an arbitrary default budget.

The value can be used to construct downstream client options:

```
if let Some(remaining) = ctx.time_remaining() {
    let budget = remaining.saturating_sub(
        Duration::from_millis(50)
    );

    options = options.with_timeout(budget);
}
```

Illustrative fragment; the margin is application-specific.

This connects server-side deadline enforcement to client-side timeout propagation.

The remaining budget is not a guarantee that the downstream call can finish, nor does it automatically reserve enough time for response encoding, network transfer, or local cleanup. It is a useful basis for deriving a bounded downstream call budget while preserving explicit responsibility for application-level commit and cancellation semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

### 5.6 Lower-level unary GET client API

Although generated clients do not select Connect GET automatically, the released client module includes `call_unary_get`.

This supplies a lower-level path for invoking an eligible unary method using the Connect GET protocol. It complements the server's GET support and provides a distinction between protocol capability and generated-client convenience.

The operation remains subject to the protobuf method's `NO_SIDE_EFFECTS` requirement and the HTTP caching and intermediary considerations discussed in §1.1.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/connectrpc/latest/connectrpc/client/index.html](https://docs.rs/connectrpc/latest/connectrpc/client/index.html?utm_source=chatgpt.com)

## 6. Code generation, testing, and release compatibility

### 6.1 Descriptor decoding for very large schemas

The 0.9.0 code-generation tools use a larger Buffa element-memory budget when decoding compiler-produced descriptor sets.

The default tooling budget is 1 GiB, rather than the ordinary runtime default of 32 MiB. This reflects an important trust and workload distinction: large descriptor sets are expected during code generation, whereas runtime protobuf messages may originate from untrusted peers.

Two configuration mechanisms are available.

| Mechanism                               | Scope                                                         | Example                                             |
| --------------------------------------- | ------------------------------------------------------------- | --------------------------------------------------- |
| `BUFFA_ELEMENT_MEMORY_LIMIT`            | All participating Buffa-based generation tools in the process | `BUFFA_ELEMENT_MEMORY_LIMIT=4294967296 cargo build` |
| `element_memory_limit` generator option | The individual plugin receiving that option                   | `element_memory_limit=4294967296`                   |

Both accept a byte count or `unlimited`.

The environment variable is particularly useful with `buf generate`, where multiple plugins process the same schema and need compatible budgets. It is also the available override for `connectrpc-build`, which does not receive a `protoc` plugin-option string in the same way.

The per-plugin option is more selective, but every generator that needs a higher limit must receive its own configuration.

A finite large budget preserves an important diagnostic: malformed or unexpectedly amplified descriptor data produces an explicit failure rather than exhausting process memory.

Runtime reflection remains separate. Increasing the generation budget does not change the memory budget used when `connectrpc-reflection` initializes a `Reflector`. Excessively large runtime descriptor sets can produce `ReflectionError::ElementBudget`.

This is especially consequential where generation and reflection use the same schema but have different exposure and resource expectations. The schema can be valid for generation while still exceeding the reflection initialization budget.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 6.2 Protobuf Edition 2024

`protoc-gen-connect-rust` 0.9.0 advertises Protobuf Edition 2024 as its maximum supported edition.

This enables the generator to accept service definitions using `edition = "2024"` rather than rejecting them during compiler-plugin negotiation.

The edition introduced capabilities such as `enforce_naming_style` and `default_symbol_visibility`, but those are principally interpreted and enforced by `protoc`. The ConnectRPC service generator receives the resulting descriptors and does not require a different RPC execution model for those features.

This is a compatibility distinction rather than a new runtime service capability. It matters when schema compilation evolves independently of Rust service implementation and publishing workflows.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

### 6.3 Direct handler testing without an HTTP server

Generated service handlers are ordinary Rust trait methods and can be exercised directly without constructing a network listener, HTTP/2 connection, or protobuf-framed HTTP request.

The relevant construction APIs are:

| API                           | Testing role                                                  |
| ----------------------------- | ------------------------------------------------------------- |
| `RequestContext::new`         | Constructs a request context from headers                     |
| `ServiceRequest::from_parts`  | Combines a borrowed Buffa message view with its backing bytes |
| `StreamMessage::from_message` | Creates an owned streaming message from a typed message       |
| `futures::stream::iter`       | Produces a deterministic inbound test stream                  |
| `Encodable::encode`           | Converts an opaque handler response into bytes for assertions |

An important Rust type-system detail is that generated handler traits can return `ServiceResult<impl Encodable<M>>`. Even if the concrete implementation returns a familiar owned message, code invoking the method through the generated trait may see the opaque `Encodable` return type.

Consequently, direct tests may need to encode the response through `Encodable::encode` and decode it into the expected message type before asserting on fields.

This is different from a test whose only purpose is validating a domain function that happens to be called by a handler.

The layering provides several distinct test surfaces:

- Direct handler calls exercise application behavior and typed request/response handling.
- `ConnectRpcService` tests exercise dispatch, interceptors, limits, and protocol negotiation.
- Generated-client tests using an injected `ClientTransport` exercise client behavior without necessarily opening a real socket.
- Network tests exercise HTTP transport, TLS, proxy behavior, and real interoperability.

These layers should not be treated as equivalent evidence. A direct handler test does not verify protocol framing, while a transport test may obscure which failure originated in application logic.

The generated-client abstraction and `ServiceTransport` adapter provide a way to introduce Tower-compatible test transports without replacing generated RPC method implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

[https://docs.rs/connectrpc/latest/connectrpc/client/index.html](https://docs.rs/connectrpc/latest/connectrpc/client/index.html?utm_source=chatgpt.com)

### 6.4 Generated return types and `refining_impl_trait` diagnostics

The generated service traits use return-position `impl Trait` in traits to permit owned messages and alternative encodable representations.

An implementation returning `ServiceResult<ConcreteResponse>` can refine a trait return declared as `ServiceResult<impl Encodable<ResponseType>>`. Rust may emit `refining_impl_trait_internal` or `refining_impl_trait_reachable` warnings because the implementation exposes a more concrete return type.

This is intentional in the ConnectRPC generation model, not necessarily evidence that the handler signature is incorrect.

The tagged guide documents narrowly allowing the relevant refining-impl-trait lint for these generated implementations. This is worth recording because it prevents attempts to rewrite otherwise valid handler signatures to match the much more verbose generated trait declaration.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source: [https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

### 6.5 Cargo feature boundaries and independently generated bindings

The 0.9.1 runtime has a feature layout that materially affects dependency composition:

| Feature        | Default  | Capability boundary                              |
| -------------- | -------- | ------------------------------------------------ |
| `json`         | Enabled  | Protobuf JSON message encoding and decoding      |
| `gzip`, `zstd` | Enabled  | Bundled compression providers                    |
| `client`       | Disabled | Native client HTTP transports                    |
| `client-tls`   | Disabled | TLS-enabled client transport                     |
| `server`       | Disabled | Built-in Hyper server                            |
| `server-tls`   | Disabled | TLS-enabled server support                       |
| `tls`          | Disabled | Convenience combination of client and server TLS |
| `axum`         | Disabled | Axum routing integration                         |

A small protocol-only or custom-transport build does not need to include every networking feature. However, Cargo feature unification remains additive across the dependency graph.

For protobuf-binary-only builds, both message generation and runtime configuration must disable JSON support consistently. Disabling default features also disables the default compression features, so selected compression algorithms must be explicitly re-enabled if needed.

The 0.9.0 release also establishes a hard generated-code compatibility boundary with Buffa 0.9. Buffa's size-arithmetic changes mean bindings produced by older code generators may no longer compile against the new runtime. Independently generated message crates referenced through `extern_path` must likewise use compatible Buffa-generated representations.

This is different from a protobuf schema compatibility issue: the same `.proto` contract can remain valid while its generated Rust representations need regeneration for the newer library APIs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Sources:

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md?utm_source=chatgpt.com)

## 7. Version-specific boundaries and capability interactions

### 7.1 Do not substitute development-branch APIs for the 0.9.1 release

The upstream repository's `main` branch contains work beyond the tagged 0.9.1 runtime. In particular, development-branch interfaces such as `intercept_head` and newer connection-management abstractions must not be described as released 0.9.1 functionality without an independently verified release.

For the version covered here, early HTTP-layer authorization is provided by Tower middleware, with protocol-correct rejection available through `ConnectError::into_http_response`. The released interceptor model remains `intercept_unary` and `intercept_streaming`.

This distinction is particularly important for a pre-1.0 Rust library whose development documentation can describe APIs not yet present in the corresponding published crate.

Sources:

[https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md?utm_source=chatgpt.com)

[https://github.com/connectrpc/connect-rust/blob/main/docs/guide.md](https://github.com/connectrpc/connect-rust/blob/main/docs/guide.md?utm_source=chatgpt.com)

### 7.2 Consequential capability combinations

Several of the added capabilities become more significant when considered together:

| Capability combination                                               | Resulting architectural option                                           | Important boundary                                                          |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| `SharedHttp2Connection` + Tower load balancing + transport readiness | Multiple managed gRPC connections with load-aware dispatch               | Connection-level signals do not measure application capacity directly       |
| `Router::try_merge` + `methods()` + generated `Spec`                 | Collision-checked service assembly and registered-procedure inspection   | Runtime registration remains separate from reflection descriptors           |
| Tower admission middleware + `ConnectError::into_http_response`      | Early rejection with protocol-correct errors                             | Middleware must choose errors from trusted request information              |
| `OwnedView` + `Encodable::encode_segments` + compression controls    | Reduced copying for eligible protobuf responses                          | JSON, compression, and certain unary/interceptor paths remain contiguous    |
| `Limits` + client decode budgets + streaming backpressure            | Resource controls on both sides of an RPC                                | Per-message ceilings are not aggregate stream quotas                        |
| `RequestContext::time_remaining()` + `CallOptions::with_timeout`     | Explicit propagation of residual deadline budgets                        | Application cancellation and commit semantics remain independent            |
| Large-schema generation budget + runtime reflection budget           | Independent limits for trusted generation and runtime schema publication | Raising one budget does not change the other                                |
| TLS peer metadata + request extensions + interceptors                | Typed, transport-derived identity available to RPC policies              | Authentication, certificate verification, and authorization remain separate |

The overarching distinction is that ConnectRPC's advanced functionality is distributed across generated service contracts, the dispatcher, RPC interceptors, Tower HTTP services, protobuf representation, client transport, and the hosting layer.

Those layers are deliberately composable, but their guarantees are not interchangeable. In particular, a message-size limit is not a stream quota; a TLS peer certificate is not an authorization decision; a borrowed protobuf view is not automatically an allocation-free HTTP response; and a generated service declaration is not automatically a live runtime or reflection registration.

Understanding these boundaries reveals where existing library mechanisms can replace custom RPC infrastructure while preserving precise control over performance, protocol correctness, resource consumption, and operational behavior.

End of addendum — scoped to the published ConnectRPC Rust 0.9.x API family.