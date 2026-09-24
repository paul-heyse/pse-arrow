# `opentelemetry::context::ContextGuard`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.context.ContextGuard.json).

<a id="op-30c719e39d73839fb9d98820"></a>
## ContextGuard

`struct` · `opentelemetry::context::ContextGuard` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ContextGuard
```

Source: `src/context.rs:456`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A guard that resets the current context to the prior context when dropped.

<a id="op-c062c2f4e48bde6e9b2c899a"></a>
## drop

`function` · `opentelemetry::context::ContextGuard::drop` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::ContextGuard", "path": "ContextGuard"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [463, 1], "end": [470, 2], "filename": "src/context.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/context.rs:464`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-777440cfefd1eea8808144c8"></a>
## fmt

`function` · `opentelemetry::context::ContextGuard::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::ContextGuard", "path": "ContextGuard"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [455, 10], "end": [455, 15], "filename": "src/context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/context.rs:455`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
