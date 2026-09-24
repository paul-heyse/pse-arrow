# `tracing_subscriber::filter::targets::IntoIter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.targets.IntoIter.json).

<a id="op-d7c6b73cb89ec7e7de4ace28"></a>
## IntoIter

`struct` · `tracing_subscriber::filter::targets::IntoIter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct IntoIter
```

Source: `src/filter/targets.rs:527`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An owning iterator over the [target]-[level] pairs of a `Targets` filter.

This struct is created by the `IntoIterator` trait implementation of [`Targets`](../operations/tracing_subscriber.filter.targets.Targets.md#op-641e1a5b3033eaed5a14d773).

# Examples

Merge the targets from one `Targets` with another:

```
use tracing_subscriber::filter::Targets;
use tracing_core::Level;

let mut filter = Targets::new().with_target("my_crate", Level::INFO);
let overrides = Targets::new().with_target("my_crate::interesting_module", Level::DEBUG);

filter.extend(overrides);
# drop(filter);
```

[target]: tracing_core::Metadata::target
[level]: tracing_core::Level

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::target`.

<a id="op-8c77c505a102e70f37dba8d8"></a>
## Item

`assoc_type` · `tracing_subscriber::filter::targets::IntoIter::Item` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::IntoIter", "path": "IntoIter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [544, 1], "end": [554, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/filter/targets.rs:545`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c737dbafb5b92f2b5ee5565b"></a>
## fmt

`function` · `tracing_subscriber::filter::targets::IntoIter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::IntoIter", "path": "IntoIter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [526, 10], "end": [526, 15], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/targets.rs:526`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1f7aa871d04c1185c9643ec1"></a>
## next

`function` · `tracing_subscriber::filter::targets::IntoIter::next` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::IntoIter", "path": "IntoIter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [544, 1], "end": [554, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/filter/targets.rs:547`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-726e784e33fe34ec18756862"></a>
## size_hint

`function` · `tracing_subscriber::filter::targets::IntoIter::size_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn size_hint(&self) -> (usize, Option<usize>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::IntoIter", "path": "IntoIter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [544, 1], "end": [554, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/filter/targets.rs:551`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
