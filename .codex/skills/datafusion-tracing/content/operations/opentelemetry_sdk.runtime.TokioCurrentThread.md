# `opentelemetry_sdk::runtime::TokioCurrentThread`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.TokioCurrentThread.json).

<a id="op-e9d65f6d647d360d58183da6"></a>
## TokioCurrentThread

`struct` · `opentelemetry_sdk::runtime::TokioCurrentThread` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TokioCurrentThread
```

Source: `src/runtime.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Runtime implementation, which works with Tokio's current thread runtime.

<a id="op-61aba49aa03f4d4154c83318"></a>
## Receiver

`assoc_type` · `opentelemetry_sdk::runtime::TokioCurrentThread::Receiver` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Receiver
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [233, 1], "end": [247, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}, "trait_path": "opentelemetry_sdk::runtime::RuntimeChannel"}`

Source: `src/runtime.rs:234`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0a59cf6ae017328d63eeff22"></a>
## Sender

`assoc_type` · `opentelemetry_sdk::runtime::TokioCurrentThread::Sender` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Sender
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [233, 1], "end": [247, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}, "trait_path": "opentelemetry_sdk::runtime::RuntimeChannel"}`

Source: `src/runtime.rs:235`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-850ad9386da5a1457c294a74"></a>
## batch_message_channel

`function` · `opentelemetry_sdk::runtime::TokioCurrentThread::batch_message_channel` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn batch_message_channel<T: Debug + Send>(&self, capacity: usize) -> (Self::Sender<T>, Self::Receiver<T>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [233, 1], "end": [247, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}, "trait_path": "opentelemetry_sdk::runtime::RuntimeChannel"}`

Source: `src/runtime.rs:237`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eeba8a01910fb320bc8874b8"></a>
## clone

`function` · `opentelemetry_sdk::runtime::TokioCurrentThread::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> TokioCurrentThread
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 17], "end": [101, 22], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/runtime.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-db348ca7d5b5816f4b348fec"></a>
## delay

`function` · `opentelemetry_sdk::runtime::TokioCurrentThread::delay` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [115, 1], "end": [138, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}, "trait_path": "opentelemetry_sdk::runtime::Runtime"}`

Source: `src/runtime.rs:135`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-23395f8381382df2f77094fe"></a>
## fmt

`function` · `opentelemetry_sdk::runtime::TokioCurrentThread::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 10], "end": [101, 15], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/runtime.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8ffa76f61ce0dc61d3c45838"></a>
## spawn

`function` · `opentelemetry_sdk::runtime::TokioCurrentThread::spawn` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn<F>(&self, future: F) where F: Future<Output = ()> + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TokioCurrentThread", "path": "TokioCurrentThread"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [115, 1], "end": [138, 2], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}, "trait_path": "opentelemetry_sdk::runtime::Runtime"}`

Source: `src/runtime.rs:116`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
