# `opentelemetry_sdk::metrics::aggregation::Aggregation::Base2ExponentialHistogram`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.aggregation.Aggregation.Base2ExponentialHistogram.json).

<a id="op-1f68ce3b469db5d752757e08"></a>
## max_scale

`struct_field` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Base2ExponentialHistogram::max_scale` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_scale: i8
```

Source: `src/metrics/aggregation.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The maximum resolution scale to use for the histogram.

The maximum value is `20`, in which case the maximum number of buckets
that can fit within the range of a signed 32-bit integer index could be
used.

The minimum value is `-10` in which case only two buckets will be used.

<a id="op-bc4f2920098f76ad57edf567"></a>
## max_size

`struct_field` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Base2ExponentialHistogram::max_size` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
max_size: u32
```

Source: `src/metrics/aggregation.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The maximum number of buckets to use for the histogram.

<a id="op-a28d0680f3c52a12384679ad"></a>
## record_min_max

`struct_field` · `opentelemetry_sdk::metrics::aggregation::Aggregation::Base2ExponentialHistogram::record_min_max` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
record_min_max: bool
```

Source: `src/metrics/aggregation.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Indicates whether to not record the min and max of the distribution.

By default, these values are recorded.

It is generally not valuable to record min and max for cumulative data
as they will represent the entire life of the instrument instead of just
the current collection cycle, you can opt out by setting this value to
`false`
