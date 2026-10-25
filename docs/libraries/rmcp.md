# RMCP — Comprehensive Rust Library Overview

Library: `rmcp` · Stable release: 3.7.0 (October 22, 2026) · Research date: October 24, 2026 · Rust MSRV: 1.88 · License: Apache-2.0

## 1. Executive orientation

RMCP is the official Rust SDK for the Model Context Protocol (MCP), providing the protocol implementation and runtime infrastructure for building MCP servers, MCP clients, and applications that operate in both roles.

It is a Tokio-based asynchronous library with typed protocol models, JSON-RPC communication, automatic protocol negotiation, declarative tool registration, configurable transports, bidirectional messaging, and advanced functionality covering subscriptions, long-running tasks, authentication, sampling, elicitation, and caching.

Version 3.7.0 targets the MCP `2026-07-28` specification while retaining compatibility with older protocol versions, including `2025-11-25`. The 3.x generation is particularly significant because it introduces support for a newer protocol lifecycle, richer state management, transport-independent subscriptions, and multi-round-trip interactions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Official sources:

- Repository: [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk?utm_source=chatgpt.com)
- Rust API documentation: [https://docs.rs/rmcp/latest/rmcp/](https://docs.rs/rmcp/latest/rmcp/?utm_source=chatgpt.com)
- Crate registry: [https://crates.io/crates/rmcp](https://crates.io/crates/rmcp?utm_source=chatgpt.com)
- Version 3.7.0 release: [https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.7.0](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.7.0?utm_source=chatgpt.com)
- Version-specific SDK guide: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)

### 1.1 Capability map

| Capability family          | Main RMCP mechanisms                                                              |
| -------------------------- | --------------------------------------------------------------------------------- |
| Server construction        | `ServerHandler`, `ServiceExt`, `#[tool]`, tool/prompt routers                     |
| Client construction        | `ClientHandler`, `Peer<RoleClient>`, typed client methods                         |
| Protocol lifecycle         | Initialization, discovery, version and capability negotiation                     |
| Tool execution             | Typed arguments, JSON Schema, structured results, annotations, tool-level errors  |
| Context provision          | Resources, resource templates, prompts, argument completion                       |
| Bidirectional intelligence | Sampling, elicitation, client roots, logging, server-initiated interactions       |
| Asynchronous execution     | Cancellation, progress, subscriptions, multi-round-trip requests, MCP tasks       |
| Transport                  | stdio, child processes, Streamable HTTP, generic async streams, custom transports |
| Remote integration         | HTTP sessions, stateless operation, OAuth, enterprise-managed authorization       |
| Optimization               | Cache metadata, conditional requests, concurrency limits, pagination              |
| Extensibility              | Typed protocol models, handler traits, transport abstractions, Tower integration  |

### 1.2 What RMCP does—and does not do

RMCP is fundamentally a protocol SDK and communication runtime, not an AI agent framework or a tool-execution engine.

It handles MCP wire messages, lifecycle management, capability exchange, routing, transport coordination, and the representation of MCP functionality. Applications provide their own domain operations, authorization decisions, persistence, external service integrations, and execution policies.

This separation is important: RMCP can expose sophisticated native Rust functionality to compatible AI clients without requiring an application-specific JSON-RPC framework. Conversely, it does not intrinsically provide language-model inference, general-purpose workflow orchestration, sandboxed code execution, semantic retrieval, or distributed scheduling.

Its largest architectural benefit is often eliminating bespoke protocol infrastructure while preserving control over the implementation of the actual capabilities.

## 2. Core architecture and execution model

RMCP's architecture separates four concerns: protocol data, application behavior, connection lifecycle, and message transport.

Application implementation

Domain services · Tool handlers · Resource providers · Client callbacks

Handler dispatch

ServerHandler

Expose MCP capabilities

ClientHandler

Handle server requests

Service / Peer

RMCP protocol runtime

Typed JSON-RPC · Correlation · Lifecycle · Capabilities · Cancellation

Transport trait

Connection boundary

stdio · Streamable HTTP · Async I/O · Custom adapters

### 2.1 Handler traits versus the underlying Service trait

The most important abstractions are:

| Abstraction                | Responsibility                                                                          |
| -------------------------- | --------------------------------------------------------------------------------------- |
| `ServerHandler`            | Implements server requests, capability descriptions, notifications, and lifecycle hooks |
| `ClientHandler`            | Implements client capabilities and handles server-initiated requests or notifications   |
| `Service<R>`               | Lower-level protocol service interface parameterized by role                            |
| `RoleServer`, `RoleClient` | Encode protocol direction and associated message types                                  |
| `Peer<R>`                  | Typed interface for sending requests and notifications to the remote peer               |
| `ServiceExt`               | Starts services through compatible transports                                           |
| `RunningService<R, S>`     | Owns an active service, exposes peer operations, and manages shutdown                   |
| `RequestContext<R>`        | Holds request identity, metadata, cancellation, extensions, and peer access             |

Most applications implement `ServerHandler` or `ClientHandler`, rather than implementing the general `Service<R>` interface directly.

The underlying service model is role-aware. A server receives client request types and sends server response types; a client does the inverse. The marker types express these relationships through associated types rather than repeated ad hoc serialization and dispatch.

The runtime is bidirectional: a server can make requests to its client, and a client can receive callbacks while another operation is in progress. That makes it more expressive than a conventional one-way RPC wrapper.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



### 2.2 Request-scoped execution context

`RequestContext<RoleServer>` carries several consequential mechanisms:

- `id`: the incoming request identifier.
- `ct`: a `tokio_util::sync::CancellationToken` associated with that request.
- `meta`: MCP request metadata.
- `extensions`: non-serialized transport or execution metadata.
- `peer`: access to the connected remote party.

It also exposes `protocol_version()`, `client_info()`, and `client_capabilities()` on the server side.

This enables handlers to make decisions based on the actual requesting client and negotiated protocol version, rather than assuming all connections support identical functionality.

The cancellation token is particularly useful for long-running operations because a handler can propagate request cancellation into Tokio tasks or domain-specific execution code.

Importantly, the SDK provides the cancellation signal; the domain operation must cooperate with it for cancellation to actually stop its work.

Source: [https://docs.rs/rmcp/latest/rmcp/service/struct.RequestContext.html](https://docs.rs/rmcp/latest/rmcp/service/struct.RequestContext.html?utm_source=chatgpt.com)

### 2.3 Connection lifecycle and peer operations

Calling `.serve(transport).await` establishes a running MCP service. The returned `RunningService` exposes methods for interacting with the peer and controlling the connection.

`peer()` obtains the peer interface; `waiting()` waits for termination; `cancel()` requests shutdown. The service also provides explicit close operations, including timeout-aware closing.

Requests are correlated with responses by the protocol runtime, permitting multiple outstanding operations. `Peer` exposes both typed operations and lower-level methods such as `send_request`, `send_notification`, and `send_request_with_option`.

`RequestHandle` and `PeerRequestOptions` offer additional control over outbound request execution. `RequestIdProvider` and `ProgressTokenProvider` provide extension points for identifier generation.

Source: [https://docs.rs/rmcp/latest/rmcp/service/](https://docs.rs/rmcp/latest/rmcp/service/?utm_source=chatgpt.com)

### 2.4 Protocol lifecycle: initialize versus discover

One of the most consequential changes in the 3.x SDK is that there are now two important connection-lifecycle patterns.

|                              | Legacy initialization       | 2026-07-28 discovery            |
| ---------------------------- | --------------------------- | ------------------------------- |
| Startup                      | `initialize` exchange       | `server/discover`               |
| Ready signal                 | `notifications/initialized` | Discovery completes startup     |
| Client identity/capabilities | Negotiated connection state | Included in per-request `_meta` |
| Version selection            | Initialization negotiation  | Discovery of supported versions |
| Typical use                  | Older servers and clients   | Modern stateless protocol       |
| RMCP entry                   | `.serve()`                  | `serve_with_lifecycle(...)`     |

`ClientLifecycleMode` exposes `Initialize`, `Discover`, and `Auto`.

`Auto` probes discovery and can fall back to legacy initialization when discovery is unsupported or the probe times out.

On the server side, `ServerHandler::discover`, `negotiate_initialize`, and `supported_protocol_versions` provide control over discovery and version agreement. The server can advertise a narrower version set than the SDK itself understands.

Important distinction: protocol capability negotiation is not authentication or authorization. A client advertising a feature does not establish that it should be permitted to invoke a particular operation.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)

## 3. MCP tools: typed operations exposed through the protocol

Tools are RMCP's central mechanism for making application functionality callable by an MCP client.

Their protocol representation consists of identity and descriptive metadata, an input schema, optional output schema, annotations, and execution results.

### 3.1 Declarative tool registration

The SDK's companion `rmcp-macros` crate supplies procedural macros that eliminate most of the repetitive registration, dispatch, argument conversion, and schema-generation code.

Relevant macros include `#[tool]`, `#[tool_router]`, and `#[tool_handler]`.

```

use rmcp::{
    handler::server::wrapper::Parameters,
    schemars, tool, tool_router,
};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct SearchArgs {
    query: String,
    limit: Option<usize>,
}

#[derive(Clone)]
struct SearchServer;

#[tool_router(server_handler)]
impl SearchServer {
    #[tool(description = "Search indexed records")]
    async fn search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> String {
        format!("Searching for {}", args.query)
    }
}

```

Illustrative tool declaration, based on the 3.7.0 macro API. It demonstrates registration and typed decoding, not a complete search implementation.

`#[tool_router(server_handler)]` is convenient for a tools-only server. For a server combining tools with resources, prompts, and custom metadata, an explicit `ServerHandler` implementation and `#[tool_handler]` usually provide a clearer extension surface.

### 3.2 JSON Schema as the external type contract

Tools integrate with `serde` and `schemars`.

`serde::Deserialize` governs conversion from received JSON values into Rust values. `schemars::JsonSchema` describes the argument contract to clients.

That relationship is significant because the same Rust structures can participate in deserialization and protocol-facing schema generation. Explicit schema duplication is therefore often unnecessary.

The SDK uses JSON Schema draft 2020-12. Schemas can communicate properties, required fields, nested structures, enums, constraints, and field descriptions.

Important qualifications:

- Generated JSON Schema describes the public data contract; it does not replace business-rule validation.
- Rust types and JSON Schema are not semantically identical. Custom deserializers, defaults, flattening, and complex schema constructs deserve careful inspection.
- `#[tool]` accepts custom schema metadata where generated schemas are insufficient.
- Tool argument validation and remote client behavior must not be assumed to be identical merely because the same schema is advertised.

A particularly useful recent change is that `outputSchema` can represent JSON types other than objects, and structured tool content can be any JSON value under protocol `2026-07-28`. This permits scalar and array results without artificially wrapping every output in an object. Older peers may need different representation handling.

Source: [https://docs.rs/rmcp/latest/rmcp/attr.tool.html](https://docs.rs/rmcp/latest/rmcp/attr.tool.html?utm_source=chatgpt.com)

### 3.3 Typed output and structured results

RMCP supports several result representations.

`Json<T>` can represent a typed structured output, while `CallToolResult` gives more explicit control over the content blocks and error state.

`CallToolResult` supports text, images, audio, and embedded resources. Binary media commonly travels as Base64 with a MIME type.

This allows a single tool result to include machine-readable data and human-readable explanatory content, where supported by the client.

The distinction between structured content and ordinary text is important. Producing JSON-looking text is not the same as returning `structuredContent` with a declared `outputSchema`.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)

### 3.4 Tool errors versus protocol errors

RMCP distinguishes two classes of failure:

| Failure                     | Representation                | Meaning                                                    |
| --------------------------- | ----------------------------- | ---------------------------------------------------------- |
| Operation failed            | `CallToolResult::error(...)`  | The tool was invoked successfully but the operation failed |
| Protocol or dispatch failed | `Err(McpError)` / `ErrorData` | The server could not process the MCP request correctly     |

The first should generally carry domain-level failures that are meaningful to the calling model or user: missing records, rejected business operations, upstream service failures, or invalid domain conditions.

The second represents protocol-level problems, such as malformed request parameters or an unsupported operation.

This is not merely a cosmetic distinction. Clients may surface tool-result errors in model-visible content while treating JSON-RPC errors as infrastructure failures.

### 3.5 Tool annotations and discoverability

Tools support descriptive annotations, including titles and semantic hints such as read-only behavior.

These can help clients organize available functionality, present meaningful descriptions, or apply risk-sensitive handling.

However, annotations are declarations by the server, not enforcement of permissions or guarantees. A `readOnlyHint` does not make an implementation incapable of mutation.

Tool-list change notifications allow clients to refresh their available tools without restarting a connection. For dynamically produced tool collections, the server must ensure that enumeration, schema lookup, and dispatch remain mutually consistent.

Source: [https://docs.rs/rmcp/latest/rmcp/handler/server/trait.ServerHandler.html](https://docs.rs/rmcp/latest/rmcp/handler/server/trait.ServerHandler.html?utm_source=chatgpt.com)

## 4. Resources, prompts, and completion services

These capabilities are complementary to tools and materially affect how an MCP server can represent a domain.

### 4.1 Resources: addressable context rather than operations

Resources expose readable data under URI identifiers.

The server-side interface provides `list_resources`, `read_resource`, and `list_resource_templates`. The client provides corresponding discovery and read methods.

Resources can represent files, documents, database records, reports, diagnostics, or other externally addressable information. The protocol returns textual or Base64-encoded binary content.

Resource templates define parameterized URI patterns, such as `records://{record_id}/summary`, that clients expand into concrete resource URIs.

This offers a useful architectural distinction:

| Mechanism         | Best conceptual use                                             |
| ----------------- | --------------------------------------------------------------- |
| Tool              | Perform an operation with explicit arguments                    |
| Resource          | Read identifiable information                                   |
| Resource template | Describe a parameterized family of readable information         |
| Prompt            | Provide a reusable conversation or workflow template            |
| Completion        | Help users or clients construct valid prompt/resource arguments |

Resources can eliminate the need to expose every read operation as a callable tool, particularly where data has stable identity and can be discovered systematically.

They do not, by themselves, provide an underlying filesystem, database, index, or access-control model. The application defines URI resolution and enforces access permissions.

### 4.2 Resource change mechanisms

For compatible protocol versions, servers can announce changes to their resource lists and to individual resources.

The 2026 protocol introduces a different transport-neutral subscription model, discussed later. Older resource-specific subscription methods remain available for legacy peers but are no longer the preferred modern mechanism.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)

### 4.3 Prompts: reusable message structures

Prompts expose reusable conversation structures.

The `#[prompt]`, `#[prompt_router]`, and `#[prompt_handler]` macros provide declarative registration, typed argument handling, and dispatch.

The main protocol structures include `PromptMessage`, `GetPromptResult`, and the relevant request parameters.

A prompt can return a sequence of messages with user or assistant roles, an optional description, and content including text or supported multimedia representations.

This is useful when a server has domain-specific analytical procedures, review instructions, or task formulations that should be accessible through MCP without making them executable tools.

Prompt definitions are supplied by the server, but their use and interpretation remain under the client's control.

### 4.4 Argument completion

The `complete()` handler exposes completion suggestions for prompt arguments and resource-template parameters.

`CompleteRequestParams` identifies the referenced prompt or resource template, the partially entered argument, and contextual values already supplied.

This enables context-sensitive suggestions—for example, restricting candidate table names based on a previously selected database or suggesting identifiers based on a selected resource family.

RMCP supplies the protocol and response structures; the application supplies the lookup logic.

Architectural implication: tools, resources, prompts, and completions can jointly describe an application interface rather than relying on a flat set of tool calls. This can improve semantic discoverability and reduce duplicated operational interfaces.

Sources:

[https://docs.rs/rmcp/latest/rmcp/attr.prompt_router.html](https://docs.rs/rmcp/latest/rmcp/attr.prompt_router.html?utm_source=chatgpt.com)

[https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)

## 5. Bidirectional interactions and client-provided capabilities

RMCP supports more than client-initiated tool calls. A server can request information or action from its client during an operation, subject to the client's advertised capabilities.

### 5.1 Elicitation: obtaining additional user input

Elicitation allows a server to request information that was not available when an operation began.

RMCP supports two modes:

| Mode             | Mechanism                           | Typical purpose                                            |
| ---------------- | ----------------------------------- | ---------------------------------------------------------- |
| Form elicitation | JSON Schema describing input fields | Missing parameters, user choices, clarification            |
| URL elicitation  | Client directs user to a webpage    | External consent, account setup, interactive authorization |

For form elicitation, `elicit_safe!(T)` marks a type suitable for the elicitation schema, while `ctx.peer.elicit::<T>()` obtains a typed response.

For URL elicitation, `ctx.peer.elicit_url(...)` coordinates an external interaction. The client responds with an `ElicitationAction`, distinguishing acceptance, decline, and cancellation.

Client applications implement `ClientHandler::create_elicitation()` to collect the input. RMCP does not create the user interface or automatically approve the request.

A particularly valuable aspect is that elicitation composes with multi-round-trip requests and tasks: a long-running operation can request missing input without inventing a separate application-specific conversation protocol.

Availability: `elicitation` feature; the relevant functionality must also be supported by the negotiated client and protocol.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 5.2 Sampling: server-initiated LLM inference

Sampling reverses the usual direction of interaction. Instead of a client invoking a server tool, the server requests an LLM completion from the client.

The server uses `context.peer.create_message(CreateMessageRequestParams ...)`; the client implements `ClientHandler::create_message()`.

Requests may include messages, maximum tokens, system instructions, model preferences, temperature, and preferences relating to cost, speed, and capability.

RMCP transmits these requests and models their responses. The client must supply the actual model integration, approval behavior, and usage policies.

Deprecated

Sampling is deprecated under SEP-2577 in the targeted protocol generation. It remains implemented in 3.7.0 but is scheduled for eventual removal. This matters when considering a new architecture that relies on server-initiated inference as a fundamental execution primitive.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 5.3 Roots: client workspace descriptions

Roots allow a client to report workspace or project locations, commonly represented by `file://` URIs.

Servers can request roots using `peer.list_roots()`, while clients implement `ClientHandler::list_roots()` and may notify servers when the set changes.

Deprecated

Roots are also deprecated under SEP-2577, though still available in 3.7.0. Moreover, a root URI is contextual information, not an authorization boundary or an independently verified filesystem permission.

### 5.4 MCP logging versus runtime observability

RMCP supports protocol logging, including severity filtering through `set_level()` and server-to-client `notify_logging_message(...)` notifications. Clients receive these through `on_logging_message()`.

This MCP logging mechanism is likewise deprecated.

It should be distinguished from Rust's `tracing` ecosystem. `tracing` and OpenTelemetry-style instrumentation can describe internal spans, distributed request relationships, timing, and execution behavior independently of whether clients implement MCP logging.

RMCP's protocol lifecycle and request identifiers provide useful correlation points, but they do not constitute a complete tracing or telemetry backend.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 6. Asynchronous execution, subscriptions, and tasks

This is one of RMCP's richest capability areas. Several mechanisms address superficially similar problems but have very different semantics.

| Mechanism                | Interaction model                                      | What remains active                             |
| ------------------------ | ------------------------------------------------------ | ----------------------------------------------- |
| Normal request           | Request → response                                     | One RPC operation                               |
| Progress notification    | Asynchronous status during request                     | Original request                                |
| Cancellation             | Request to stop ongoing work                           | Original operation until it cooperates          |
| Subscription             | Long-lived notification stream                         | Subscription request                            |
| Multi-round-trip request | Request → input needed → revised request → result      | Logical operation represented across RPC rounds |
| MCP task                 | Submit → task identifier → poll/update/cancel → result | Server-managed task lifecycle                   |

### 6.1 Progress, cancellation, and liveness

Progress messages use `ProgressNotificationParam`, carrying a progress token, completed work measure, optional total, and descriptive message.

Cancellation uses `notifications/cancelled`. In the receiving handler, `RequestContext.ct` exposes a Tokio `CancellationToken` that can be propagated into nested asynchronous operations.

`on_cancelled()` is available for custom handling. RMCP also implements symmetric `ping` handling for connection liveness.

The important distinction is between cancelling an RPC's interest in a result and ensuring the underlying operation stops. RMCP supports cancellation propagation, but external services, blocking native code, and non-cooperative tasks may continue unless the application provides a cancellation mechanism.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 6.2 Transport-neutral subscriptions

The `2026-07-28` protocol introduces `subscriptions/listen`, replacing legacy resource-specific subscriptions and standalone HTTP GET event streams.

Unlike a resource-only subscription, this mechanism can deliver different notification categories through a long-lived request with an explicitly negotiated filter.

On the server, relevant APIs include:

- `ServerHandler::accepted_subscription_filter()` — determines which requested categories the server accepts.
- `ServerHandler::listen()` — implements the subscription.
- `SubscriptionContext` — holds the accepted filter, cancellation, and a filter-enforcing notification sink.

On the client, `client.listen(...)` creates a `Subscription` handle. Its `acknowledged()` method exposes the accepted filter, `next()` receives notifications, and `cancel()` terminates the subscription.

The SDK checks notifications against the accepted filter and the server's advertised capabilities. Thus, subscription delivery is not merely an unconstrained stream of arbitrary messages.

Backpressure and recovery: `listen()` buffers 64 notifications per subscription by default; `listen_with_capacity()` adjusts the capacity. A slow consumer can cause a `SubscriptionEnd::Lagged` termination. Abrupt disconnections require a new subscription rather than transparent restoration of the old stream.

This mechanism is valuable for change feeds, invalidation events, execution notifications, and other asynchronous state transitions. It does not independently provide durable event replay.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 6.3 Multi-round-trip requests (MRTR)

MRTR, introduced through SEP-2322, allows operations to pause logically for additional input without maintaining one continuously open server execution context.

A server handling `tools/call`, `prompts/get`, or `resources/read` may return an `InputRequiredResult` instead of a final response. The response contains embedded requests for information and an opaque `requestState`.

The client fulfills the embedded requests, then resubmits the original operation with `inputResponses` and the echoed `requestState`.

Client

Logical exchange

Server

Initial `tools/call`

`InputRequiredResult` + `requestState`

Client obtains requested input

Retry with `inputResponses`

`CallToolResponse::Complete`

The SDK's high-level `call_tool()`, `get_prompt()`, and `read_resource()` helpers automatically perform the rounds using the local `ClientHandler`, with a default maximum of 10 rounds.

Lower-level methods such as `call_tool_once()` expose the intermediate results when the caller needs to control the interaction manually.

Security and state integrity: `requestState` is supplied back by the client and is therefore untrusted. The optional `request-state` feature provides `RequestStateCodec`, which uses authenticated HMAC-based state sealing. `new_with_keyring()` supports signing-key rotation.

HMAC integrity does not imply encryption, replay prevention, authorization, or transactional idempotency. Sensitive contents, expiration, duplicate execution, and caller binding remain separate application concerns.

MRTR is particularly valuable for stateless services and operations that require intermittent user decisions rather than continuous server-side computation.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 6.4 MCP tasks: independently managed long-running work

RMCP implements the MCP Tasks extension, SEP-2663, identified as `io.modelcontextprotocol/tasks`.

This is an extension capability, not an unconditional property of every MCP connection.

Clients advertise task support using `ClientCapabilities::builder().enable_tasks()`. A server can materialize an eligible `tools/call` as a task and return a `CreateTaskResult` rather than awaiting its full completion.

The task-specific protocol operations are:

| Operation      | Meaning                                                 |
| -------------- | ------------------------------------------------------- |
| `tasks/get`    | Read status and available result/error information      |
| `tasks/update` | Provide responses to input required by the running task |
| `tasks/cancel` | Request cooperative task cancellation                   |

The SDK supplies `rmcp::task_manager::TaskManager`, `TaskContext`, and `TaskOptions`.

`TaskManager` launches asynchronous operations, tracks task state, handles input updates, and exposes cancellation. A task may progress through `working`, `input_required`, and terminal states, with result or error information retained for retrieval.

A subtle but critical guarantee: after `TaskManager::spawn()` returns, the task is already observable through its task identifier. This avoids returning a task ID before the corresponding record exists.

The provided task manager is in-memory, however. The documentation's use of “durable” concerns protocol-visible observability, not persistence across process restarts or replica failover. Its default task TTL is five minutes; expiration and cleanup behavior need attention for extended workloads.

It is useful as an SDK-level executor and lifecycle coordinator, but not a substitute for a durable distributed job store.

Source: [https://docs.rs/rmcp/latest/rmcp/task_manager/](https://docs.rs/rmcp/latest/rmcp/task_manager/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.5 Choosing between the asynchronous models

The architectural distinctions can be summarized by where operational state resides.

| Requirement                                       | Relevant mechanism                            | Important constraint                               |
| ------------------------------------------------- | --------------------------------------------- | -------------------------------------------------- |
| Report progress on an existing call               | Progress notification                         | Call still needs an execution lifetime             |
| React to changing data                            | Subscription                                  | Stream is not durable event storage                |
| Obtain intermittent input in a stateless workflow | MRTR                                          | State verification and replay policy required      |
| Run work longer than an HTTP request              | MCP task                                      | Server-side task management and retention required |
| Receive input while background work executes      | Task plus `tasks/update`                      | Requires task extension support                    |
| Expose a remote change event                      | Subscription plus resource/tool notifications | Consumer must handle lag and reconnect             |

MRTR and tasks therefore solve different problems. MRTR is principally a protocol-level continuation model, while tasks describe identifiable work with an independent lifecycle.

These mechanisms can be combined with elicitation, capability negotiation, and transport choices, but their individual limitations do not disappear through composition.

## 7. Client-side caching and discovery efficiency

RMCP 3.7.0 implements SEP-2549 response caching for selected operations:

- `server/discover`
- `tools/list`
- `prompts/list`
- `resources/list`
- `resources/templates/list`
- `resources/read`

The server can attach cache hints using `ttlMs` and `cacheScope`. RMCP clients cache eligible responses transparently, and matching change notifications invalidate affected entries.

Caching is enabled by default but requires a positive TTL from the server unless a client-side default is configured.

`ClientCacheConfig` provides controls for default and maximum TTL, maximum entries, principal-specific partitioning, stale-result behavior, and disabling the cache.

A consequential default is `serve_stale_on_error`: when a refresh fails, RMCP can return an expired cached response as a successful `Ok(...)` rather than propagating the error.

This is useful for availability-oriented clients but potentially unsuitable for applications requiring strict freshness or explicit detection of upstream failures.

The cache applies to discovery and resource reads, not arbitrary tool execution results. It should not be understood as automatic memoization of MCP tools.

Architecturally, this can remove repetitive discovery calls and reduce server traffic without implementing a separate local metadata cache. For authenticated clients, cache partitioning by principal is essential where responses differ by authorization context.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 8. Transport architecture and HTTP execution

RMCP's transport layer is intentionally decoupled from handlers and protocol semantics.

The core `Transport<R>` trait requires `send`, `receive`, and `close`. It uses role-specific message types, allowing one protocol runtime to operate over different communication channels.

### 8.1 Supported transports

| Transport               | Rust interface / feature                       | Principal characteristics                                          |
| ----------------------- | ---------------------------------------------- | ------------------------------------------------------------------ |
| Standard I/O            | `stdio()`, `transport-io`                      | Local MCP server over stdin/stdout                                 |
| Spawned process         | `TokioChildProcess`, `transport-child-process` | Client launches and manages a child MCP server                     |
| Generic async I/O       | `transport-async-rw`                           | MCP messages over compatible Tokio read/write channels             |
| In-process worker       | `transport-worker`                             | Client/server communication without an external network or process |
| Streamable HTTP server  | `StreamableHttpService`                        | Tower service for HTTP frameworks                                  |
| Streamable HTTP client  | `StreamableHttpClientTransport`                | Pluggable HTTP client implementation                               |
| Reqwest HTTP backend    | `transport-streamable-http-client-reqwest`     | Standard remote-client implementation                              |
| Unix-socket HTTP client | `transport-streamable-http-client-unix-socket` | Streamable HTTP over a Unix-domain socket                          |
| Custom transport        | `Transport<R>`, `IntoTransport`                | Application-defined messaging infrastructure                       |

`IntoTransport` can adapt compatible `Sink`/`Stream` and `AsyncRead`/`AsyncWrite` pairs as well as worker implementations.

This permits the same handler implementation to operate through different transports without rewriting its domain operations.

The `Transport` trait also has a meaningful concurrency contract: `send()` returns a `Send + 'static` future because sends may execute concurrently, whereas receives are sequential. Custom implementations need to preserve their framing, ownership, and ordering semantics.

Source: [https://docs.rs/rmcp/latest/rmcp/transport/trait.Transport.html](https://docs.rs/rmcp/latest/rmcp/transport/trait.Transport.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Transport boundaries worth understanding:

- RMCP does not ship a native WebSocket transport feature in 3.7.0. External adapters can use its generic transport abstractions, but that is different from first-party WebSocket support.
- SSE remains supported within Streamable HTTP responses, but the separate two-endpoint HTTP+SSE transport from MCP `2024-11-05` is deliberately not provided.
- For stdio servers, stdout is the protocol channel. Diagnostics and ordinary logging should use stderr to avoid corrupting JSON-RPC framing.
- An in-process transport can be valuable for testing and embedding because it avoids managing a child process or HTTP server. It does not automatically eliminate the protocol's data-model and serialization boundaries.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 8.2 Streamable HTTP server integration

The HTTP implementation exposes `StreamableHttpService`, which implements Tower's service model.

It can therefore be mounted inside an Axum router or integrated with compatible Hyper/Tower HTTP infrastructure.

This means that HTTP concerns such as routing, middleware composition, connection handling, TLS termination, and deployment infrastructure can remain with the surrounding HTTP stack rather than being reinvented in an MCP-specific server.

The RMCP service owns MCP parsing, protocol dispatch, and transport-specific response handling.

`StreamableHttpServerConfig` exposes important options:

| Configuration                          | Role                                                              |
| -------------------------------------- | ----------------------------------------------------------------- |
| `legacy_session_mode`                  | Controls sessions for protocol versions earlier than `2026-07-28` |
| `json_response`                        | Prefers JSON over SSE for simple eligible responses               |
| `allowed_hosts`                        | Validates incoming HTTP Host values                               |
| `allowed_origins`                      | Controls browser Origin validation                                |
| `max_request_body_bytes`               | Bounds incoming POST body size                                    |
| `stateless_protocol_metadata_required` | Enforces modern per-request protocol metadata                     |
| `cancellation_token`                   | Coordinates server shutdown                                       |
| `session_store`                        | Optional external restoration of legacy session metadata          |
| `sse_keep_alive`, `sse_retry`          | Streaming connection behavior                                     |

Stateless server behavior is a significant architectural distinction.

For protocol `2026-07-28`, requests are stateless regardless of the legacy session configuration. The server uses request-scoped protocol metadata instead of relying on a persistent MCP session.

The server's handler factory can therefore run for each request. Shared databases, pools, caches, and other durable application dependencies should be captured through appropriately shared handles rather than assuming handler-local mutable state persists between requests.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 8.3 Session restoration and event persistence

Legacy Streamable HTTP supports `SessionManager`, `LocalSessionManager`, and `NeverSessionManager`.

There is also an optional `SessionStore` interface for external storage of session initialization metadata. This supports restoring an identified legacy session on another server instance, although it does not by itself preserve arbitrary handler state or ongoing work.

RMCP also defines an `EventStore` abstraction for storing and replaying HTTP streaming events.

There is a version-sensitive qualification here: the 3.0 migration guide describes distributed event replay, including earlier stateless replay support, while the current 3.7.0 README explicitly states that modern `2026-07-28` subscriptions do not use `Last-Event-ID` restoration. These statements should not be collapsed into a guarantee of reconnectable modern subscriptions.

For the targeted version, the supported conclusion is that the session/event-store APIs exist, but modern `subscriptions/listen` streams are not automatically resumed after reconnection.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Sources:

[https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/session/](https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/session/?utm_source=chatgpt.com)

[https://github.com/modelcontextprotocol/rust-sdk/discussions/969](https://github.com/modelcontextprotocol/rust-sdk/discussions/969?utm_source=chatgpt.com)

### 8.4 HTTP client concurrency, recovery, and flow control

`StreamableHttpClientTransportConfig` controls several aspects that have direct consequences for throughput and correctness.

The default limit is 16 concurrent ordinary POST requests. Cancellation and reply traffic use a separate control path, so they are not necessarily blocked behind the ordinary request limit.

An open SSE response stream does not consume one of the ordinary POST slots.

Other controls include channel capacity, SSE retry policy, SSE event-size limits, request timeouts, authentication headers, and expired-session recovery.

When a legacy session expires, optional reinitialization can recover the connection. RMCP deliberately avoids automatically retrying ambiguous in-flight POSTs whose effects may already have occurred.

That last behavior is important: transparent retries of non-idempotent tool calls could duplicate side effects. Protocol recovery and application-level operation retries must remain distinguishable.

Custom HTTP integrations implement `StreamableHttpClient`. The built-in adapters enforce SSE event-size limits, but a custom adapter must implement the size-aware methods correctly rather than assuming the generic trait automatically applies those limits.

Source: [https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_client/struct.StreamableHttpClientTransportConfig.html](https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_client/struct.StreamableHttpClientTransportConfig.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



### 8.5 Standardized HTTP routing headers

SEP-2243 adds HTTP headers that expose selected MCP request metadata to intermediaries:

- `Mcp-Method`: the MCP method being invoked.
- `Mcp-Name`: a tool, prompt, resource, or task identifier.
- `Mcp-Param-*`: selected tool arguments promoted from their input schema.

This is particularly relevant to gateways, middleware, and request-routing infrastructure because intermediaries can make some decisions without parsing the entire JSON-RPC body.

Promotion is opt-in through the schema extension `x-mcp-header`.

Only supported primitive properties are eligible. Nested fields require a direct `properties` traversal; arbitrary `$ref` or composed-schema paths are not interchangeable with this requirement.

The SDK generates and validates these headers for negotiated modern protocol versions.

Headers may improve routing efficiency, but they also affect data exposure. Sensitive arguments should not be promoted merely because it is technically possible; HTTP headers can appear in proxy logs and infrastructure telemetry.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 9. Authentication, authorization, and security

RMCP's authentication functionality is substantial, particularly on the client side.

It is useful to separate three concerns: acquiring credentials, transporting credentials, and authorizing domain operations.

### 9.1 OAuth client functionality

With the `auth` feature enabled, RMCP provides OAuth infrastructure for protected remote MCP servers.

Supported functionality includes:

| Functionality                   | Standard / mechanism                                               |
| ------------------------------- | ------------------------------------------------------------------ |
| Authorization Code flow         | OAuth with PKCE S256                                               |
| Protected resource discovery    | RFC 9728                                                           |
| Authorization server metadata   | RFC 8414 and OpenID Connect discovery                              |
| Resource-specific token binding | RFC 8707 resource indicators                                       |
| Dynamic client registration     | RFC 7591                                                           |
| Client ID Metadata Documents    | SEP-991                                                            |
| Token refreshing                | OAuth refresh tokens                                               |
| Scope selection                 | Authentication challenge and metadata                              |
| Scope upgrades                  | Handling `403 insufficient_scope`                                  |
| Client credentials              | Supported client-credentials flows and optional JWT authentication |

The central abstractions are `OAuthState`, `AuthorizationManager`, `AuthorizationRequest`, `AuthClient`, and `OAuthHttpClient`.

`OAuthState` manages the authorization lifecycle. `AuthorizationRequest` describes the available registration information and requested authorization parameters.

Registration selection follows a defined priority: pre-registered client credentials first, Client ID Metadata Documents when supported, then Dynamic Client Registration when available.

The SDK also derives scopes from available authorization metadata and HTTP challenges where the caller has not specified them.

These capabilities remove a substantial amount of bespoke protocol and OAuth negotiation code.

The application still owns user interaction, callback handling, secure credential storage choices, and policy around when authorization should be initiated or repeated.

For example, `is_authorization_required()` can classify an initialization failure, but the SDK does not automatically open a browser or complete user consent.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md](https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



### 9.2 Credential storage and HTTP customization

RMCP provides interfaces for credential/state storage and alternative OAuth HTTP clients.

`OAuthHttpClient` allows custom TLS, proxy, network-access, and transport policies. That is particularly relevant when the default Reqwest network behavior does not match an application's execution constraints.

Authorization-server discovery also involves a security-sensitive boundary: information from a remote server can point a client toward further URLs. The SDK provides validation and network-policy mechanisms, but the caller remains responsible for determining which remote resources and authorization servers are trustworthy.

There is also an important distinction between published metadata and compatibility-generated metadata.

`AuthorizationManager::resolve_metadata()` reports the source of the discovered configuration. A `LegacyEndpointFallback` means endpoints were synthesized for an older MCP authorization convention rather than established through authoritative published metadata.

Applications that require verified metadata provenance should distinguish these cases.

### 9.3 Enterprise-managed authorization

The `auth-enterprise-managed` feature adds support for enterprise-managed authorization flows involving identity-provider credentials and ID-JAG exchanges.

It supports explicitly configured authentication mechanisms such as:

- `None`
- `ClientSecretBasic`
- `ClientSecretPost`
- `JwtAssertion`

The SDK handles specific exchange and resource-binding operations, including client-side checks of ID-JAG structure and expected claims.

However, the documented implementation does not provide every possible OAuth extension: Rich Authorization Requests (`authorization_details`) and DPoP are not supported in this exchange profile.

It also does not perform the authorization server's signature-verification responsibilities, discover organizational approvals automatically, or establish enterprise SSO policy.

This makes it an integration mechanism for appropriately provisioned enterprise authorization systems, not a replacement for them.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md](https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 9.4 HTTP server security controls

The Streamable HTTP server includes some unusually important security defaults:

Host validation is enabled for loopback hosts by default, protecting locally running servers against certain DNS rebinding attacks.

Origin validation is not enabled by default for backward compatibility. Applications expecting browser-origin enforcement should configure `allowed_origins` or explicitly enforce Origin validation.

The server also supports a streaming request-body size limit, independent of whether the client supplies `Content-Length`.

These features are valuable safeguards, but they are not equivalent to authenticating the user, authorizing an MCP tool, constraining tool side effects, or applying execution quotas.

For publicly reachable servers, the surrounding HTTP stack and application must still provide the appropriate authentication enforcement, access checks, transport security, and abuse controls.

Source: [https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/tower/struct.StreamableHttpServerConfig.html](https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/tower/struct.StreamableHttpServerConfig.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



## 10. Extensibility, composition, and observability

### 10.1 Router composition and dynamic capability exposure

RMCP offers both procedural-macro routing and explicit router objects.

`ToolRouter` and `PromptRouter` organize method registration and dispatch. `Router<S>` can wrap a `ServerHandler` and combine registered tool and prompt routes with the underlying service.

`Router<S>` supports `with_tool`, `with_tools`, `with_prompt`, and `with_prompts`.

This can support modular construction of an MCP interface while retaining a central service implementation.

For dynamically selected tools, `ServerHandler::get_tool()` provides an additional discovery hook alongside `list_tools()`.

However, a router is not automatically a multi-server MCP gateway. Combining independent remote servers, reconciling identities, forwarding authentication, rewriting schemas, and handling remote lifecycle failures are separate responsibilities.

Source: [https://docs.rs/rmcp/latest/rmcp/handler/server/router/struct.Router.html](https://docs.rs/rmcp/latest/rmcp/handler/server/router/struct.Router.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 10.2 Protocol extension mechanisms

RMCP exposes custom request and notification hooks through both handler traits.

`on_custom_request` and `on_custom_notification` permit handling protocol extensions or application-specific messages without modifying the core SDK.

The underlying typed models preserve additional extension metadata. `RequestContext.extensions` provides a separate non-serialized mechanism for locally propagated information.

These are distinct:

| Extension mechanism         | Where it exists              |
| --------------------------- | ---------------------------- |
| Custom protocol methods     | MCP wire protocol            |
| `_meta` fields              | Serialized protocol metadata |
| `RequestContext.extensions` | In-process execution context |
| Custom `Service<R>`         | Service dispatch             |
| Custom `Transport<R>`       | Communication boundary       |
| Tower wrappers/middleware   | Service integration boundary |

The distinction matters when an application acts as a proxy. In-process extensions are not automatically serializable across a process boundary.

### 10.3 Metadata models and distributed tracing

RMCP's metadata system uses several specialized structures:

- `MetaObject` for general protocol metadata.
- `RequestMetaObject` for request-specific metadata.
- `NotificationMetaObject` for notification-specific metadata.

These preserve extensible JSON metadata while providing typed accessors for well-known protocol fields.

RMCP also exposes W3C Trace Context helpers for `traceparent`, `tracestate`, and `baggage`, following SEP-414.

`RequestParamsMeta` and related model helpers allow callers to attach or inspect this information without manually manipulating arbitrary JSON.

This is a useful integration point for Rust `tracing`, OpenTelemetry, and multi-service observability.

Nevertheless, trace-context support is not the same thing as end-to-end automatic distributed instrumentation. Applications must still define appropriate spans, extract and propagate context across their execution boundaries, and connect exporters or collectors.

A particularly important transport detail is SEP-2260 request association. Server-initiated sampling, elicitation, and roots requests created while processing another request are associated with their origin, allowing appropriate routing over an HTTP response stream. A custom cross-process session manager must preserve that association because the in-memory marker is not serialized automatically.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+3



Sources:

[https://docs.rs/rmcp/latest/rmcp/model/struct.MetaObject.html](https://docs.rs/rmcp/latest/rmcp/model/struct.MetaObject.html?utm_source=chatgpt.com)

[https://docs.rs/rmcp/latest/rmcp/model/trait.RequestParamsMeta.html](https://docs.rs/rmcp/latest/rmcp/model/trait.RequestParamsMeta.html?utm_source=chatgpt.com)

[https://github.com/modelcontextprotocol/rust-sdk/discussions/969](https://github.com/modelcontextprotocol/rust-sdk/discussions/969?utm_source=chatgpt.com)

### 10.4 Pagination and scalable discovery

The client API includes convenience methods such as:

`list_all_tools()`, `list_all_prompts()`, `list_all_resources()`, and `list_all_resource_templates()`.

These traverse paginated protocol responses automatically.

The lower-level list methods expose cursors to callers that need explicit pagination or want to avoid accumulating the entire result collection.

This becomes important when an MCP server has a large dynamically generated tool catalog or resource namespace.

Caching and pagination can work together; pagination cursors participate in cache identity. Neither mechanism means the server automatically has an indexed catalog—the application still provides efficient enumeration.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 11. Build configuration, compatibility, and important limitations

### 11.1 Cargo feature selection

RMCP is modular at compile time. Some functionality is always present in the protocol model, while runtime features are gated.

The default crate features are `base64`, `macros`, and `server`. Client functionality and most transports require explicit features.

| Feature                       | Enables                                                 |
| ----------------------------- | ------------------------------------------------------- |
| `server`                      | Server handlers, tools, schema support                  |
| `client`                      | Client handlers and peer operations                     |
| `macros`                      | Declarative tool and prompt registration                |
| `elicitation`                 | Elicitation support                                     |
| `request-state`               | HMAC-based MRTR state codec                             |
| `auth`                        | OAuth support                                           |
| `auth-client-credentials-jwt` | JWT-oriented client authentication                      |
| `auth-enterprise-managed`     | Enterprise-managed authorization                        |
| `tower`                       | Tower service integration                               |
| `local`                       | Selected local, single-threaded client-handler behavior |
| `transport-*`                 | Specific communication transports                       |

HTTP TLS support is independently configured, with features for Rustls, native TLS, or Rustls without a default crypto provider.

Enabling the Reqwest-backed transport does not necessarily select the desired TLS backend by itself.

For example, a server-focused dependency might use:

```

[dependencies]
rmcp = { version = "3.7.0", default-features = false, features = [
    "server",
    "macros",
    "transport-streamable-http-server",
    "elicitation",
    "request-state",
] }

```

This expresses a capability selection, not a complete HTTP application's dependency list.

The underlying Rust ecosystem includes `tokio`, `serde`, `serde_json`, `schemars`, `tracing`, Tower, Reqwest, and OAuth-related crates. `rmcp-macros` is the companion procedural-macro crate, rather than a separate MCP runtime.

Source: [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/Cargo.toml](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/Cargo.toml?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 11.2 Consequential 2.x → 3.x differences

The 3.x migration is more than a crate-version change.

| Area                 | Consequential change                                                                               |
| -------------------- | -------------------------------------------------------------------------------------------------- |
| Handler return types | `call_tool`, `get_prompt`, `read_resource` now use response enums supporting intermediate outcomes |
| Result discriminator | `resultType` distinguishes complete, input-required, and extension-defined outcomes                |
| Task protocol        | Earlier experimental task API replaced by the Tasks extension                                      |
| Discovery            | New `server/discover` lifecycle                                                                    |
| HTTP sessions        | Modern protocol stateless; legacy session flag renamed                                             |
| Subscriptions        | `subscriptions/listen` supersedes older resource subscriptions                                     |
| Metadata             | More precise request, notification, and general metadata types                                     |
| Structured output    | Non-object JSON values now supported                                                               |
| OAuth initiation     | Unified `AuthorizationRequest` interface                                                           |
| Protocol unions      | Non-exhaustive enums require forward-compatible matching                                           |

One particularly important detail: `RunningService::call_tool()` automatically handles MRTR but does not automatically poll an MCP task until completion. Clients expecting a task handle should use the one-round invocation API, inspect the returned variant, then use task-specific methods.

The 3.0 migration guide also records removals of previously deprecated aliases and APIs. Version 3.7.0 has `ProtocolVersion::LATEST` set to `2026-07-28`, while `LATEST_WITH_INITIALIZE` identifies `2025-11-25`, the newest version supporting the old initialization handshake.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Sources:

[https://github.com/modelcontextprotocol/rust-sdk/discussions/969](https://github.com/modelcontextprotocol/rust-sdk/discussions/969?utm_source=chatgpt.com)

[https://docs.rs/rmcp/latest/rmcp/model/struct.ProtocolVersion.html](https://docs.rs/rmcp/latest/rmcp/model/struct.ProtocolVersion.html?utm_source=chatgpt.com)

### 11.3 Architectural limits

RMCP removes a great deal of protocol code, but several responsibilities remain deliberately outside its scope.

| Responsibility             | What RMCP provides                                           | What remains external                                   |
| -------------------------- | ------------------------------------------------------------ | ------------------------------------------------------- |
| Tool execution             | Typed dispatch and request context                           | Domain implementation, side-effect control              |
| Authorization              | OAuth/client infrastructure and protocol capabilities        | Resource permissions and domain policies                |
| Long-running jobs          | Task lifecycle and in-memory manager                         | Durable distributed execution where required            |
| State continuation         | MRTR representation and integrity helper                     | Application state semantics, replay prevention          |
| Observability              | Request identity, trace metadata, Rust instrumentation hooks | Span design, metrics, collector/export pipeline         |
| Retry/recovery             | Transport recovery and selected session handling             | Domain idempotency and retry policy                     |
| Tool discovery             | Schemas, pagination, routing, caching                        | Efficient domain catalog and authoritative metadata     |
| Sandbox execution          | No general-purpose sandbox runtime                           | Process/container isolation and execution policy        |
| Multi-server composition   | Service abstractions and protocol types                      | Aggregation, delegation, identity translation           |
| Cross-language integration | Standard MCP wire compatibility                              | Language-specific service implementation and deployment |

The most substantial engineering opportunities come from recognizing which responsibilities are already addressed by RMCP's native combinations of typed handlers, negotiation, transports, continuation, tasks, subscriptions, and caching, rather than independently implementing them.

The strongest remaining design constraints generally concern state ownership, authorization, concurrency, durability, and execution-side effects.

## 12. Testing, conformance, and reference resources

### 12.1 Testing and conformance

The upstream SDK includes example servers and clients, transport tests, protocol tests, and integration tests involving the Python and JavaScript MCP ecosystems.

The published stable 3.0.0 conformance baseline reported:

| Suite                    | Reported result               |
| ------------------------ | ----------------------------- |
| Core server              | 40/40 scenarios               |
| Core client              | 32/32 scenarios               |
| Tasks extension          | 9/9 runnable server scenarios |
| Client credentials basic | 8/8 scenarios                 |
| Client credentials JWT   | 9/9 scenarios                 |

These are release-specific upstream conformance results, not measurements of arbitrary applications or a guarantee covering every optional extension. The upstream report also identifies skipped or expected-failure extension cases.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



RMCP's in-process worker transport can support protocol-level integration tests without a separately launched server. Standard stdio and Streamable HTTP tests remain important for observing transport-specific behavior.

The official MCP Inspector is a separate testing utility, useful for interactively examining tool catalogs, invoking methods, and inspecting protocol interactions. It is not part of the RMCP crate.

MCP Inspector: [https://github.com/modelcontextprotocol/inspector](https://github.com/modelcontextprotocol/inspector?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 12.2 Primary-source reference map

The following are the most useful direct sources for further investigation, with URLs written in full.

| Topic                              | Source                                                                                                                                                                                                                                                        |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Version-specific complete guide    | [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md?utm_source=chatgpt.com)                                                                             |
| Rust API index                     | [https://docs.rs/rmcp/latest/rmcp/](https://docs.rs/rmcp/latest/rmcp/?utm_source=chatgpt.com)                                                                                                                                                                 |
| ServerHandler API                  | [https://docs.rs/rmcp/latest/rmcp/handler/server/trait.ServerHandler.html](https://docs.rs/rmcp/latest/rmcp/handler/server/trait.ServerHandler.html?utm_source=chatgpt.com)                                                                                   |
| ClientHandler API                  | [https://docs.rs/rmcp/latest/rmcp/handler/client/trait.ClientHandler.html](https://docs.rs/rmcp/latest/rmcp/handler/client/trait.ClientHandler.html?utm_source=chatgpt.com)                                                                                   |
| Service and execution context      | [https://docs.rs/rmcp/latest/rmcp/service/](https://docs.rs/rmcp/latest/rmcp/service/?utm_source=chatgpt.com)                                                                                                                                                 |
| Tool macros and routing            | [https://docs.rs/rmcp/latest/rmcp/attr.tool_router.html](https://docs.rs/rmcp/latest/rmcp/attr.tool_router.html?utm_source=chatgpt.com)                                                                                                                       |
| Composable server router           | [https://docs.rs/rmcp/latest/rmcp/handler/server/router/struct.Router.html](https://docs.rs/rmcp/latest/rmcp/handler/server/router/struct.Router.html?utm_source=chatgpt.com)                                                                                 |
| Task manager                       | [https://docs.rs/rmcp/latest/rmcp/task_manager/](https://docs.rs/rmcp/latest/rmcp/task_manager/?utm_source=chatgpt.com)                                                                                                                                       |
| Transport interfaces               | [https://docs.rs/rmcp/latest/rmcp/transport/](https://docs.rs/rmcp/latest/rmcp/transport/?utm_source=chatgpt.com)                                                                                                                                             |
| HTTP server configuration          | [https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/tower/struct.StreamableHttpServerConfig.html](https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_server/tower/struct.StreamableHttpServerConfig.html?utm_source=chatgpt.com)       |
| HTTP client configuration          | [https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_client/struct.StreamableHttpClientTransportConfig.html](https://docs.rs/rmcp/latest/rmcp/transport/streamable_http_client/struct.StreamableHttpClientTransportConfig.html?utm_source=chatgpt.com) |
| OAuth and enterprise authorization | [https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md](https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md?utm_source=chatgpt.com)                                                                   |
| Cargo features and dependencies    | [https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/Cargo.toml](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/Cargo.toml?utm_source=chatgpt.com)                                                   |
| 3.x migration and compatibility    | [https://github.com/modelcontextprotocol/rust-sdk/discussions/969](https://github.com/modelcontextprotocol/rust-sdk/discussions/969?utm_source=chatgpt.com)                                                                                                   |
| Release 3.7.0                      | [https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.7.0](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.7.0?utm_source=chatgpt.com)                                                                                 |
| Upstream examples                  | [https://github.com/modelcontextprotocol/rust-sdk/tree/rmcp-v3.7.0/examples](https://github.com/modelcontextprotocol/rust-sdk/tree/rmcp-v3.7.0/examples?utm_source=chatgpt.com)                                                                               |
| MCP protocol specification         | [https://modelcontextprotocol.io/specification/2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28?utm_source=chatgpt.com)                                                                                                                   |
| MCP Inspector                      | [https://github.com/modelcontextprotocol/inspector](https://github.com/modelcontextprotocol/inspector?utm_source=chatgpt.com)                                                                                                                                 |

### Overall architectural perspective

RMCP's distinctive strength is not simply that it can publish Rust functions as MCP tools. It is that it provides a common, typed runtime for a fairly sophisticated bidirectional application protocol, including capability discovery, stateless request continuation, asynchronous task lifecycles, structured content, filtered subscriptions, caching, HTTP routing metadata, and transport customization.

For straightforward integrations, the procedural macros and stdio transport can eliminate most of the infrastructure code.

For more advanced systems, the `Service`, `Peer`, handler, transport, and metadata abstractions permit considerably deeper control without abandoning protocol compatibility.

The most important distinctions to preserve when using those advanced facilities are:

Protocol state versus application state; request cancellation versus operation termination; MRTR versus tasks; notification delivery versus durable event storage; advertised capabilities versus authorization; and protocol trace metadata versus actual telemetry instrumentation.

Those boundaries determine how far RMCP's native functionality can simplify a design and where additional application infrastructure is still necessary.