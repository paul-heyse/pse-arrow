# `opentelemetry_sdk::metrics::aggregation::Aggregation`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.aggregation.Aggregation.json).

<a id="op-c672903d4f80d2a912d42b4e"></a>
## Aggregation

`enum` · `opentelemetry_sdk::metrics::aggregation::Aggregation` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Aggregation
```

Source: `src/metrics/aggregation.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The way recorded measurements are summarized.

<a id="op-775fa3f9b841ff28c2ef1d1d"></a>
## Base2ExponentialHistogram

`variant` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Base2ExponentialHistogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Base2ExponentialHistogram
```

Source: `src/metrics/aggregation.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An aggregation that summarizes a set of measurements as a histogram with
bucket widths that grow exponentially.

<a id="op-a57b21fe619ec2f64537286a"></a>
## Default

`variant` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Default
```

Source: `src/metrics/aggregation.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An aggregation that uses the default instrument kind selection mapping to
select another aggregation.

A metric reader can be configured to make an aggregation selection based on
instrument kind that differs from the default. This aggregation ensures the
default is used.

See the [the spec] for information about the default
instrument kind selection mapping.

[the spec]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.19.0/specification/metrics/sdk.md#default-aggregation

<a id="op-e58784ffb810199a2487794c"></a>
## Drop

`variant` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Drop` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Drop
```

Source: `src/metrics/aggregation.rs:11`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An aggregation that drops all recorded data.

<a id="op-a6b0d1b2620d5930f271b2e7"></a>
## ExplicitBucketHistogram

`variant` · `opentelemetry_sdk::metrics::aggregation::Aggregation::ExplicitBucketHistogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ExplicitBucketHistogram
```

Source: `src/metrics/aggregation.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An aggregation that summarizes a set of measurements as a histogram with
explicitly defined buckets.

<a id="op-d509f02fe91429a21f240db1"></a>
## LastValue

`variant` · `opentelemetry_sdk::metrics::aggregation::Aggregation::LastValue` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
LastValue
```

Source: `src/metrics/aggregation.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An aggregation that summarizes a set of measurements as the last one made.

<a id="op-cb0b5fcc26cdb696449af9de"></a>
## Sum

`variant` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Sum` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Sum
```

Source: `src/metrics/aggregation.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An aggregation that summarizes a set of measurements as their arithmetic
sum.

<a id="op-c43f499ddf52591298b5b8e2"></a>
## clone

`function` · `opentelemetry_sdk::metrics::aggregation::Aggregation::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Aggregation
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::aggregation::Aggregation", "path": "Aggregation"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 10], "end": [7, 15], "filename": "src/metrics/aggregation.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/aggregation.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ab54ec15562743b2fd620ea3"></a>
## eq

`function` · `opentelemetry_sdk::metrics::aggregation::Aggregation::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Aggregation) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::aggregation::Aggregation", "path": "Aggregation"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 24], "end": [7, 33], "filename": "src/metrics/aggregation.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/aggregation.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c98ea262344d23929373afc"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::aggregation::Aggregation::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::aggregation::Aggregation", "path": "Aggregation"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 17], "end": [7, 22], "filename": "src/metrics/aggregation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/aggregation.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8f23cd8929921ad6a8cc77df"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::aggregation::Aggregation::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::aggregation::Aggregation", "path": "Aggregation"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [94, 1], "end": [108, 2], "filename": "src/metrics/aggregation.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/metrics/aggregation.rs:95`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
