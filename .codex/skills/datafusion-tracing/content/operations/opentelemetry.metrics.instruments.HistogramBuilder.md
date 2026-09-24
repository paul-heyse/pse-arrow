# `opentelemetry::metrics::instruments::HistogramBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.HistogramBuilder.json).

<a id="op-0d62fd0dec4b395435e1c5d3"></a>
## HistogramBuilder

`struct` · `opentelemetry::metrics::instruments::HistogramBuilder` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct HistogramBuilder<'a, T>
```

Source: `src/metrics/instruments/mod.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configuration for building a Histogram.

<a id="op-9eb324d21c3119a392089060"></a>
## boundaries

`struct_field` · `opentelemetry::metrics::instruments::HistogramBuilder::boundaries` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
boundaries: Option<Vec<f64>>
```

Source: `src/metrics/instruments/mod.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Bucket boundaries for the histogram.

<a id="op-48774875acf4b21281e91e82"></a>
## build

`function` · `opentelemetry::metrics::instruments::HistogramBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Histogram<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::histogram::Histogram", "path": "super::Histogram"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::HistogramBuilder", "path": "HistogramBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [116, 1], "end": [125, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:122`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a new instrument.

Validates the instrument configuration and creates a new instrument. In
case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-b5040c6896a835c69e8c0bb1"></a>
## build

`function` · `opentelemetry::metrics::instruments::HistogramBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Histogram<u64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::histogram::Histogram", "path": "super::Histogram"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::HistogramBuilder", "path": "HistogramBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [127, 1], "end": [136, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:133`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a new instrument.

Validates the instrument configuration and creates a new instrument. In
case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-d7f37b3670b6839642a8ee33"></a>
## description

`struct_field` · `opentelemetry::metrics::instruments::HistogramBuilder::description` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
description: Option<std::borrow::Cow<'static, str>>
```

Source: `src/metrics/instruments/mod.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Description of the Histogram.

<a id="op-44e9663b03bd159d5a438709"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::HistogramBuilder::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::HistogramBuilder", "path": "HistogramBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [219, 1], "end": [232, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/mod.rs:220`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d56de4ae395b3a433a37dde2"></a>
## instrument_provider

`struct_field` · `opentelemetry::metrics::instruments::HistogramBuilder::instrument_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
instrument_provider: &'a dyn InstrumentProvider
```

Source: `src/metrics/instruments/mod.rs:37`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Instrument provider is used to create the instrument.

<a id="op-c077c97aeb4eb1b52c376f6f"></a>
## name

`struct_field` · `opentelemetry::metrics::instruments::HistogramBuilder::name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
name: std::borrow::Cow<'static, str>
```

Source: `src/metrics/instruments/mod.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Name of the Histogram.

<a id="op-43cbdb24ea2cc6bb797c0189"></a>
## unit

`struct_field` · `opentelemetry::metrics::instruments::HistogramBuilder::unit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
unit: Option<std::borrow::Cow<'static, str>>
```

Source: `src/metrics/instruments/mod.rs:46`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Unit of the Histogram.

<a id="op-5eb0060e33295be8b93364f7"></a>
## with_boundaries

`function` · `opentelemetry::metrics::instruments::HistogramBuilder::with_boundaries` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_boundaries(self, boundaries: Vec<f64>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::HistogramBuilder", "path": "HistogramBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [114, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:110`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the boundaries for this histogram.

Setting boundaries is optional. By default, the boundaries are set to:

`[0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0,
2500.0, 5000.0, 7500.0, 10000.0]`

# Notes
- Boundaries must not contain `f64::NAN`, `f64::INFINITY` or
  `f64::NEG_INFINITY`
- Values must be in strictly increasing order (e.g., each value must be
  greater than the previous).
- Boundaries must not contain duplicate values.

If invalid boundaries are provided, the instrument will not report
measurements.
Providing an empty `vec![]` means no bucket information will be
calculated.

# Warning
Using more buckets can improve the accuracy of percentile calculations in backends.
However, this comes at a cost, including increased memory, CPU, and network usage.
Choose the number of buckets carefully, considering your application's performance
and resource requirements.

<a id="op-40f51400c5a5eb66e8761705"></a>
## with_description

`function` · `opentelemetry::metrics::instruments::HistogramBuilder::with_description` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_description<S: Into<Cow<'static, str>>>(self, description: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::HistogramBuilder", "path": "HistogramBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [114, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the description for this instrument

<a id="op-018ddc969d6c64016879c12b"></a>
## with_unit

`function` · `opentelemetry::metrics::instruments::HistogramBuilder::with_unit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_unit<S: Into<Cow<'static, str>>>(self, unit: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::HistogramBuilder", "path": "HistogramBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [114, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:81`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the unit for this instrument.

Unit is case sensitive(`kb` is not the same as `kB`).

Unit must be:
- ASCII string
- No longer than 63 characters
