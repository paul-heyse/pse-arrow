# `opentelemetry_sdk::metrics::aggregation::Aggregation::ExplicitBucketHistogram`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.aggregation.Aggregation.ExplicitBucketHistogram.json).

<a id="op-c6a3e9bcbcabd5fa3b0bb714"></a>
## boundaries

`struct_field` · `opentelemetry_sdk::metrics::aggregation::Aggregation::ExplicitBucketHistogram::boundaries` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
boundaries: Vec<f64>
```

Source: `src/metrics/aggregation.rs:53`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The increasing bucket boundary values.

Boundary values define bucket upper bounds. Buckets are exclusive of their
lower boundary and inclusive of their upper bound (except at positive
infinity). A measurement is defined to fall into the greatest-numbered
bucket with a boundary that is greater than or equal to the measurement. As
an example, boundaries defined as:

vec![0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0,
1000.0, 2500.0, 5000.0, 7500.0, 10000.0];

Will define these buckets:

(-∞, 0], (0, 5.0], (5.0, 10.0], (10.0, 25.0], (25.0, 50.0], (50.0,
 75.0], (75.0, 100.0], (100.0, 250.0], (250.0, 500.0], (500.0,
 750.0], (750.0, 1000.0], (1000.0, 2500.0], (2500.0, 5000.0],
 (5000.0, 7500.0], (7500.0, 10000.0], (10000.0, +∞)

<a id="op-dd3551c701062105fb6dc3c8"></a>
## record_min_max

`struct_field` · `opentelemetry_sdk::metrics::aggregation::Aggregation::ExplicitBucketHistogram::record_min_max` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
record_min_max: bool
```

Source: `src/metrics/aggregation.rs:64`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Indicates whether to not record the min and max of the distribution.

By default, these values are recorded.

Recording these values for cumulative data is expected to have little
value, they will represent the entire life of the instrument instead of
just the current collection cycle. It is recommended to set this to
`false` for that type of data to avoid computing the low-value
instances.
