# `opentelemetry::trace::span::SpanKind`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.span.SpanKind.json).

<a id="op-dcebed2dc327b4042d3aa279"></a>
## SpanKind

`enum` · `opentelemetry::trace::span::SpanKind` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum SpanKind
```

Source: `src/trace/span.rs:226`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

`SpanKind` describes the relationship between the [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8), its parents, and
its children in a trace.

`SpanKind` describes two independent properties that benefit tracing systems
during analysis:

The first property described by `SpanKind` reflects whether the span is a
"logical" remote child or parent. By "logical", we mean that the span is
logically a remote child or parent, from the point of view of the library
that is being instrumented. Spans with a remote parent are interesting
because they are sources of external load. Spans with a remote child are
interesting because they reflect a non-local system dependency.

The second property described by `SpanKind` reflects whether a child span
represents a synchronous call.  When a child span is synchronous, the parent
is expected to wait for it to complete under ordinary circumstances. It can
be useful for tracing systems to know this property, since synchronous spans
may contribute to the overall trace latency. Asynchronous scenarios can be
remote or local.

In order for `SpanKind` to be meaningful, callers should arrange that a
single span does not serve more than one purpose. For example, a server-side
span should not be used directly as the parent of another remote span. As a
simple guideline, instrumentation should create a new span prior to
extracting and serializing the SpanContext for a remote call.

Note: there are complex scenarios where a `SpanKind::Client` span may have a
child that is also logically a `SpanKind::Client` span, or a
`SpanKind::Producer` span might have a local child that is a
`SpanKind::Client` span, depending on how the various libraries that are
providing the functionality are built and instrumented. These scenarios,
when they occur, should be detailed in the semantic conventions appropriate
to the relevant libraries.

To summarize the interpretation of these kinds:

| `SpanKind` | Synchronous | Asynchronous | Remote Incoming | Remote Outgoing |
|---|---|---|---|---|
| `Client` | yes | | | yes |
| `Server` | yes | | yes | |
| `Producer` | | yes | | maybe |
| `Consumer` | | yes | maybe | |
| `Internal` | | | | |

<a id="op-3172f50b10b4fd57d40a5ea6"></a>
## Client

`variant` · `opentelemetry::trace::span::SpanKind::Client` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Client
```

Source: `src/trace/span.rs:230`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Indicates that the span describes a request to some remote service. This
span is usually the parent of a remote `SpanKind::Server` span and does
not end until the response is received.

<a id="op-de3b5ec1a7799b919561d326"></a>
## Consumer

`variant` · `opentelemetry::trace::span::SpanKind::Consumer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Consumer
```

Source: `src/trace/span.rs:247`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Indicates that the span describes a child of an asynchronous
`SpanKind::Producer` request.

<a id="op-b6f788d6281d492ff9a3aa56"></a>
## Internal

`variant` · `opentelemetry::trace::span::SpanKind::Internal` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Internal
```

Source: `src/trace/span.rs:254`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Default value.

Indicates that the span represents an internal operation within an
application, as opposed to an operations with remote parents or
children.

<a id="op-421ca993349eb732ca27a864"></a>
## Producer

`variant` · `opentelemetry::trace::span::SpanKind::Producer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Producer
```

Source: `src/trace/span.rs:243`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Indicates that the span describes the initiators of an asynchronous
request. This parent span will often end before the corresponding child
`SpanKind::Consumer` span, possibly even before the child span starts.

In messaging scenarios with batching, tracing individual messages
requires a new `SpanKind::Producer` span per message to be created.

<a id="op-7003c96f3fed76cdce4c20d3"></a>
## Server

`variant` · `opentelemetry::trace::span::SpanKind::Server` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Server
```

Source: `src/trace/span.rs:235`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Indicates that the span covers server-side handling of a synchronous RPC
or other remote request. This span is often the child of a remote
`SpanKind::Client` span that was expected to wait for a response.

<a id="op-a50e7658d881e00f7563d390"></a>
## clone

`function` · `opentelemetry::trace::span::SpanKind::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanKind
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::SpanKind", "path": "SpanKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [225, 10], "end": [225, 15], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/span.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-042f69aed1883734791b1338"></a>
## eq

`function` · `opentelemetry::trace::span::SpanKind::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SpanKind) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::SpanKind", "path": "SpanKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [225, 24], "end": [225, 33], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/span.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7f89e77c667e337de459e2de"></a>
## fmt

`function` · `opentelemetry::trace::span::SpanKind::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::SpanKind", "path": "SpanKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [225, 17], "end": [225, 22], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
