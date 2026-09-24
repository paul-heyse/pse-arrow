# `tracing_core::dispatcher::WeakDispatch`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.WeakDispatch.json).

<a id="op-0beab352f71fbbf38e94b386"></a>
## WeakDispatch

`struct` · `tracing_core::dispatcher::WeakDispatch` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct WeakDispatch
```

Source: `src/dispatcher.rs:172`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

`WeakDispatch` is a version of [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c) that holds a non-owning reference
to a [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c).

The `Subscriber` may be accessed by calling [`WeakDispatch::upgrade`](../operations/tracing_core.dispatcher.WeakDispatch.md#op-579b0e9e6911ae19b38a2911),
which returns an `Option<Dispatch>`. If all [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c) clones that point
at the `Subscriber` have been dropped, [`WeakDispatch::upgrade`](../operations/tracing_core.dispatcher.WeakDispatch.md#op-579b0e9e6911ae19b38a2911) will return
`None`. Otherwise, it will return `Some(Dispatch)`.

A `WeakDispatch` may be created from a [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c) by calling the
[`Dispatch::downgrade`](../operations/tracing_core.dispatcher.Dispatch.md#op-3d911653bdde5f29cad4eafd) method. The primary use for creating a
[`WeakDispatch`](../operations/tracing_core.dispatcher.WeakDispatch.md#op-0beab352f71fbbf38e94b386) is to allow a Subscriber` to hold a cyclical reference to
itself without creating a memory leak. See [here] for details.

This type is analogous to the [`std::sync::Weak`] type, but for a
[`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c) rather than an [`Arc`].

[`Arc`]: std::sync::Arc
[here]: Subscriber#avoiding-memory-leaks

Unresolved upstream links (retained, not inferred): ``std::sync::Weak``, `std::sync::Arc`.

<a id="op-eea86fddda0303176b472e60"></a>
## clone

`function` · `tracing_core::dispatcher::WeakDispatch::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WeakDispatch
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::WeakDispatch", "path": "WeakDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 10], "end": [171, 15], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/dispatcher.rs:171`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5bff443ecf9bfb7ecd94c715"></a>
## fmt

`function` · `tracing_core::dispatcher::WeakDispatch::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::WeakDispatch", "path": "WeakDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [789, 1], "end": [802, 2], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/dispatcher.rs:790`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-579b0e9e6911ae19b38a2911"></a>
## upgrade

`function` · `tracing_core::dispatcher::WeakDispatch::upgrade` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn upgrade(&self) -> Option<Dispatch>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::WeakDispatch", "path": "WeakDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [763, 1], "end": [787, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:782`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Attempts to upgrade this `WeakDispatch` to a [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c).

Returns `None` if the referenced `Dispatch` has already been dropped.

## Examples

```
# use tracing_core::subscriber::NoSubscriber;
# use tracing_core::dispatcher::Dispatch;
let strong = Dispatch::new(NoSubscriber::default());
let weak = strong.downgrade();

// The strong here keeps it alive, so we can still access the object.
assert!(weak.upgrade().is_some());

drop(strong); // But not any more.
assert!(weak.upgrade().is_none());
```
