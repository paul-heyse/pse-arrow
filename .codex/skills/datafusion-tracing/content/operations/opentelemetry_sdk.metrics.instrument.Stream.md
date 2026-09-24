# `opentelemetry_sdk::metrics::instrument::Stream`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.instrument.Stream.json).

<a id="op-58efe95b35cd74f5deb12539"></a>
## Stream

`struct` · `opentelemetry_sdk::metrics::instrument::Stream` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Stream
```

Source: `src/metrics/instrument.rs:302`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Describes the stream of data an instrument produces. Used in `with_view`
methods on `MeterProviderBuilder` to customize the metric output.

<a id="op-4e6ef68c696a217a385fa05e"></a>
## builder

`function` · `opentelemetry_sdk::metrics::instrument::Stream::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> StreamBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Stream", "path": "Stream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [322, 1], "end": [327, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:324`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new stream builder with default values.

<a id="op-91aab3ad45181d3a8b3d5839"></a>
## default

`function` · `opentelemetry_sdk::metrics::instrument::Stream::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Stream
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Stream", "path": "Stream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 10], "end": [301, 17], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/instrument.rs:301`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7293b622ac3ac073493ad499"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::instrument::Stream::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Stream", "path": "Stream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 19], "end": [301, 24], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instrument.rs:301`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
