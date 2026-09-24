# `opentelemetry_sdk::metrics::data::ExponentialBucket`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.ExponentialBucket.json).

<a id="op-a8fd7390991312141794a73d"></a>
## ExponentialBucket

`struct` · `opentelemetry_sdk::metrics::data::ExponentialBucket` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ExponentialBucket
```

Source: `src/metrics/data/mod.rs:548`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A set of bucket counts, encoded in a contiguous array of counts.

<a id="op-dcdbbb093be9c86d4dee989a"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::ExponentialBucket::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ExponentialBucket
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ExponentialBucket", "path": "ExponentialBucket"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [547, 17], "end": [547, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:547`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f6304c50303e8d6a1f69aa45"></a>
## counts

`function` · `opentelemetry_sdk::metrics::data::ExponentialBucket::counts` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn counts(&self) -> impl Iterator<Item = u64> + '_
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ExponentialBucket", "path": "ExponentialBucket"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [559, 1], "end": [569, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:566`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the counts.

<a id="op-ad20edac5fcb4e4164a69690"></a>
## eq

`function` · `opentelemetry_sdk::metrics::data::ExponentialBucket::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &ExponentialBucket) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ExponentialBucket", "path": "ExponentialBucket"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [547, 24], "end": [547, 33], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/data/mod.rs:547`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fe826f108b7bb8a065030637"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::ExponentialBucket::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ExponentialBucket", "path": "ExponentialBucket"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [547, 10], "end": [547, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:547`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-46ac6a335daccb0d65e38bd4"></a>
## offset

`function` · `opentelemetry_sdk::metrics::data::ExponentialBucket::offset` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn offset(&self) -> i32
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::data::ExponentialBucket", "path": "ExponentialBucket"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [559, 1], "end": [569, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:561`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the bucket index of the first entry in the counts vec.
