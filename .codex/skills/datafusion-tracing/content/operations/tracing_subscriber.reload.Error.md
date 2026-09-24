# `tracing_subscriber::reload::Error`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.reload.Error.json).

<a id="op-a06bf3b39712f2ba607c53b0"></a>
## Error

`struct` · `tracing_subscriber::reload::Error` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Error
```

Source: `src/reload.rs:102`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Indicates that an error occurred when reloading a layer.

<a id="op-3c0723e676357b7462da9fa6"></a>
## fmt

`function` · `tracing_subscriber::reload::Error::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::reload::Error", "path": "Error"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 10], "end": [101, 15], "filename": "src/reload.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/reload.rs:101`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-af93bfa576c371db2e4892b9"></a>
## fmt

`function` · `tracing_subscriber::reload::Error::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::reload::Error", "path": "Error"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [384, 1], "end": [392, 2], "filename": "src/reload.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/reload.rs:385`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0020618754a32ca14d85f178"></a>
## is_dropped

`function` · `tracing_subscriber::reload::Error::is_dropped` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn is_dropped(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::reload::Error", "path": "Error"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [364, 1], "end": [382, 2], "filename": "src/reload.rs"}, "trait": null, "trait_path": null}`

Source: `src/reload.rs:379`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this error occurred because the `Subscriber`
containing the reloadable layer was dropped.

<a id="op-89cd951ab989e5da5fd2448d"></a>
## is_poisoned

`function` · `tracing_subscriber::reload::Error::is_poisoned` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn is_poisoned(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::reload::Error", "path": "Error"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [364, 1], "end": [382, 2], "filename": "src/reload.rs"}, "trait": null, "trait_path": null}`

Source: `src/reload.rs:373`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this error occurred because the layer was poisoned by
a panic on another thread.
