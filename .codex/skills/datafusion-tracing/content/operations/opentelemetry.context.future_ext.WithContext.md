# `opentelemetry::context::future_ext::WithContext`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.context.future_ext.WithContext.json).

<a id="op-c0ac280a4efa69c265e9ade8"></a>
## WithContext

`struct` · `opentelemetry::context::future_ext::WithContext` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct WithContext<T>
```

Source: `src/context/future_ext.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A future, stream, or sink that has an associated context.

<a id="op-1f1455cf7389a69563943aee"></a>
## Error

`assoc_type` · `opentelemetry::context::future_ext::WithContext::Error` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Error
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [41, 1], "end": [79, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}, "trait_path": "futures_sink::Sink"}`

Source: `src/context/future_ext.rs:45`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f6fa671bffaa47dc90024408"></a>
## Item

`assoc_type` · `opentelemetry::context::future_ext::WithContext::Item` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [21, 1], "end": [29, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/context/future_ext.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e2a5c336ecda169f2886121a"></a>
## Output

`assoc_type` · `opentelemetry::context::future_ext::WithContext::Output` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "std::future::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [10, 1], "end": [19, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/context/future_ext.rs:11`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5c47cd44d236c96da64467c6"></a>
## clone

`function` · `opentelemetry::context::future_ext::WithContext::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WithContext<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [33, 14], "end": [33, 19], "filename": "src/context/future_ext.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/context/future_ext.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fa6eebc0fe06284e693213e8"></a>
## fmt

`function` · `opentelemetry::context::future_ext::WithContext::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [33, 21], "end": [33, 26], "filename": "src/context/future_ext.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/context/future_ext.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6cd5cd7295b4841e7377f6cf"></a>
## poll

`function` · `opentelemetry::context::future_ext::WithContext::poll` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn poll(Pin<&mut self>, task_cx: &mut TaskContext<'_>) -> Poll<Self::Output>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "std::future::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [10, 1], "end": [19, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/context/future_ext.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-28e5f1c0dacd1fb3cdf4d9f6"></a>
## poll_close

`function` · `opentelemetry::context::future_ext::WithContext::poll_close` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn poll_close(Pin<&mut self>, task_cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [41, 1], "end": [79, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}, "trait_path": "futures_sink::Sink"}`

Source: `src/context/future_ext.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9b85ea2ab87118593fbd4b0f"></a>
## poll_flush

`function` · `opentelemetry::context::future_ext::WithContext::poll_flush` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn poll_flush(Pin<&mut self>, task_cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [41, 1], "end": [79, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}, "trait_path": "futures_sink::Sink"}`

Source: `src/context/future_ext.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5591fe4cf8c43df5908107e"></a>
## poll_next

`function` · `opentelemetry::context::future_ext::WithContext::poll_next` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn poll_next(Pin<&mut self>, task_cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [21, 1], "end": [29, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/context/future_ext.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a223bd93204ee7a79be154b5"></a>
## poll_ready

`function` · `opentelemetry::context::future_ext::WithContext::poll_ready` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn poll_ready(Pin<&mut self>, task_cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [41, 1], "end": [79, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}, "trait_path": "futures_sink::Sink"}`

Source: `src/context/future_ext.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-53d43ecd7f9b4a0aeb3290ad"></a>
## start_send

`function` · `opentelemetry::context::future_ext::WithContext::start_send` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn start_send(Pin<&mut self>, item: I) -> Result<(), Self::Error>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::context::future_ext::WithContext", "path": "WithContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [41, 1], "end": [79, 2], "filename": "src/context/future_ext.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "futures_sink::Sink", "path": "Sink"}, "trait_path": "futures_sink::Sink"}`

Source: `src/context/future_ext.rs:56`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
