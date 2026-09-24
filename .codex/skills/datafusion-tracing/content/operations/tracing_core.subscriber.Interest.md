# `tracing_core::subscriber::Interest`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.subscriber.Interest.json).

<a id="op-6a8f81daa049c40c19442579"></a>
## Interest

`struct` · `tracing_core::subscriber::Interest` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Interest
```

Source: `src/subscriber.rs:589`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Indicates a [`Subscriber`]'s interest in a particular callsite.

`Subscriber`s return an `Interest` from their [`register_callsite`] methods
in order to determine whether that span should be enabled or disabled.

[`Subscriber`]: super::Subscriber
[`register_callsite`]: super::Subscriber::register_callsite

<a id="op-639d56d212d8cbe7e6adf31f"></a>
## always

`function` · `tracing_core::subscriber::Interest::always` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn always() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 1], "end": [665, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:627`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an `Interest` indicating the subscriber is always interested in
being notified about a callsite.

If any subscriber expresses that it is `always()` interested in a given
callsite, then the callsite will always be enabled.

<a id="op-e9d84655e3c40182ab9929e9"></a>
## clone

`function` · `tracing_core::subscriber::Interest::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [588, 10], "end": [588, 15], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/subscriber.rs:588`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-559502603c3c48c85019c76b"></a>
## fmt

`function` · `tracing_core::subscriber::Interest::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [588, 17], "end": [588, 22], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/subscriber.rs:588`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f16a56e96127b795d4ac48e2"></a>
## is_always

`function` · `tracing_core::subscriber::Interest::is_always` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_always(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 1], "end": [665, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:648`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if the subscriber is always interested in being notified
about this callsite.

<a id="op-56b5318120db6cccd4a4a681"></a>
## is_never

`function` · `tracing_core::subscriber::Interest::is_never` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_never(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 1], "end": [665, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:634`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if the subscriber is never interested in being notified
about this callsite.

<a id="op-baf8b1ff537cf4185e59e743"></a>
## is_sometimes

`function` · `tracing_core::subscriber::Interest::is_sometimes` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_sometimes(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 1], "end": [665, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:641`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if the subscriber is sometimes interested in being notified
about this callsite.

<a id="op-5f92faeb5615c3cbdac0637d"></a>
## never

`function` · `tracing_core::subscriber::Interest::never` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn never() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 1], "end": [665, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:605`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an `Interest` indicating that the subscriber is never interested
in being notified about a callsite.

If all active subscribers are `never()` interested in a callsite, it will
be completely disabled unless a new subscriber becomes active.

<a id="op-70a63c4ee71956d505bbc48d"></a>
## sometimes

`function` · `tracing_core::subscriber::Interest::sometimes` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn sometimes() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "Interest"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [598, 1], "end": [665, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:617`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an `Interest` indicating the subscriber is sometimes interested
in being notified about a callsite.

If all active subscribers are `sometimes` or `never` interested in a
callsite, the currently active subscriber will be asked to filter that
callsite every time it creates a span. This will be the case until a new
subscriber expresses that it is `always` interested in the callsite.
