# `opentelemetry_sdk::metrics::instrument::StreamBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.instrument.StreamBuilder.json).

<a id="op-88f13d188e6f1026d98a3b01"></a>
## StreamBuilder

`struct` · `opentelemetry_sdk::metrics::instrument::StreamBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct StreamBuilder
```

Source: `src/metrics/instrument.rs:148`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A builder for creating Stream objects.

# Example

```
use opentelemetry_sdk::metrics::{Aggregation, Stream};
use opentelemetry::Key;

let stream = Stream::builder()
    .with_name("my_stream")
    .with_aggregation(Aggregation::Sum)
    .with_cardinality_limit(100)
    .build()
    .unwrap();
```

<a id="op-3deffc945d3205c108dc5cc8"></a>
## build

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<Stream, Box<dyn Error>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:214`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Build a new Stream instance using the configuration in this builder.

# Returns

A Result containing the new Stream instance or an error if the build failed.

<a id="op-bf858cd011f28b6351fb640a"></a>
## default

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> StreamBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 10], "end": [147, 17], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/instrument.rs:147`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-37e6279417cd864d04375e6e"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 19], "end": [147, 24], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instrument.rs:147`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2cdd1d77954e84949aaa898f"></a>
## with_aggregation

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::with_aggregation` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_aggregation(self, aggregation: Aggregation) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:184`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the stream aggregation. This is used to customize the aggregation.
If not set, the default aggregation based on the instrument kind will be used.

<a id="op-96fdc6b636071caccc423a9b"></a>
## with_allowed_attribute_keys

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::with_allowed_attribute_keys` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_allowed_attribute_keys(self, attribute_keys: impl IntoIterator<Item = Key>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:195`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the stream allowed attribute keys.

Any attribute recorded for the stream with a key not in this set will be
dropped. If the set is empty, all attributes will be dropped.
If this method is not used, all attributes will be kept.

<a id="op-3c8f4e100d23c9bdfa993f83"></a>
## with_cardinality_limit

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::with_cardinality_limit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_cardinality_limit(self, limit: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:204`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the stream cardinality limit. If this is not set, the default limit of 2000 will be used.

<a id="op-75905806a80ce2f1b3f1dcd6"></a>
## with_description

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::with_description` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_description(self, description: impl Into<Cow<'static, str>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:170`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the stream description. If this is not set, description provided while creating the instrument will be used.

<a id="op-81a29432dd0ebc1e8b680577"></a>
## with_name

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::with_name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_name(self, name: impl Into<Cow<'static, str>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:164`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the stream name. If this is not set, name provide while creating the instrument will be used.

<a id="op-b8652004785306e50bdd408e"></a>
## with_unit

`function` · `opentelemetry_sdk::metrics::instrument::StreamBuilder::with_unit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_unit(self, unit: impl Into<Cow<'static, str>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::StreamBuilder", "path": "StreamBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [275, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:176`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the stream unit. If this is not set, unit provided while creating the instrument will be used.
