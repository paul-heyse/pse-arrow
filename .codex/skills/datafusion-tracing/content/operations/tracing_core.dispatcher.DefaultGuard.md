# `tracing_core::dispatcher::DefaultGuard`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.DefaultGuard.json).

<a id="op-9d4a43cbb0aa3ea97d85ed6d"></a>
## DefaultGuard

`struct` · `tracing_core::dispatcher::DefaultGuard` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct DefaultGuard
```

Source: `src/dispatcher.rs:236`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A guard that resets the current default dispatcher to the prior
default dispatcher when dropped.

<a id="op-6f02c308269be587dea816a0"></a>
## drop

`function` · `tracing_core::dispatcher::DefaultGuard::drop` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::DefaultGuard", "path": "DefaultGuard"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [889, 1], "end": [901, 2], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/dispatcher.rs:891`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6f51e52440d98559bba252dc"></a>
## fmt

`function` · `tracing_core::dispatcher::DefaultGuard::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::DefaultGuard", "path": "DefaultGuard"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [235, 10], "end": [235, 15], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/dispatcher.rs:235`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
