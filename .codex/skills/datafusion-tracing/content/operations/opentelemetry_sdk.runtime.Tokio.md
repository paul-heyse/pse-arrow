# `opentelemetry_sdk::runtime::Tokio`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.Tokio.json).

<a id="op-0455ef123c3045b19b5cd60c"></a>
## Tokio

`struct` · `opentelemetry_sdk::runtime::Tokio` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Tokio
```

Source: `src/runtime.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Runtime implementation, which works with Tokio's multi thread runtime.

<a id="op-f1efc8b2a67ca9dece0e4b08"></a>
## Receiver

`assoc_type` · `opentelemetry_sdk::runtime::Tokio::Receiver` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Receiver
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [206, 1], "end": [220, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}, "trait_path": "opentelemetry_sdk::runtime::RuntimeChannel"}`

Source: `src/runtime.rs:207`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aa63374b8cf38c78785d1dd0"></a>
## Sender

`assoc_type` · `opentelemetry_sdk::runtime::Tokio::Sender` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Sender
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [206, 1], "end": [220, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}, "trait_path": "opentelemetry_sdk::runtime::RuntimeChannel"}`

Source: `src/runtime.rs:208`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-058067bdc84a9a7ecc297f70"></a>
## batch_message_channel

`function` · `opentelemetry_sdk::runtime::Tokio::batch_message_channel` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn batch_message_channel<T: Debug + Send>(&self, capacity: usize) -> (Self::Sender<T>, Self::Receiver<T>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [206, 1], "end": [220, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}, "trait_path": "opentelemetry_sdk::runtime::RuntimeChannel"}`

Source: `src/runtime.rs:210`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fab384de414c94c9f6095510"></a>
## clone

`function` · `opentelemetry_sdk::runtime::Tokio::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Tokio
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 17], "end": [66, 22], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/runtime.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0dcfeb09f7057cb7d8e61adb"></a>
## delay

`function` · `opentelemetry_sdk::runtime::Tokio::delay` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [74, 1], "end": [87, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}, "trait_path": "opentelemetry_sdk::runtime::Runtime"}`

Source: `src/runtime.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6ba5566797d3bc0c19d2fde3"></a>
## fmt

`function` · `opentelemetry_sdk::runtime::Tokio::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 10], "end": [66, 15], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/runtime.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f86fd7acfe2971f48d8ff1a"></a>
## spawn

`function` · `opentelemetry_sdk::runtime::Tokio::spawn` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn<F>(&self, future: F) where F: Future<Output = ()> + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::Tokio", "path": "Tokio"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [74, 1], "end": [87, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}, "trait_path": "opentelemetry_sdk::runtime::Runtime"}`

Source: `src/runtime.rs:75`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
