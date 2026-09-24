# `opentelemetry::metrics::instruments::AsyncInstrumentBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.AsyncInstrumentBuilder.json).

<a id="op-422bb16881c391fa2aff201f"></a>
## AsyncInstrumentBuilder

`struct` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct AsyncInstrumentBuilder<'a, I, M>
```

Source: `src/metrics/instruments/mod.rs:245`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configuration for building an async instrument.

<a id="op-25492a5ffa150e8df65c4586"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableGauge<i64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::ObservableGauge", "path": "gauge::ObservableGauge"}}}, {"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [322, 1], "end": [322, 73], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:322`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableGauge<i64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-5c11f77f5aa94ad48ebdea43"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableUpDownCounter<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter", "path": "super::ObservableUpDownCounter"}}}, {"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [328, 1], "end": [332, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:328`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableUpDownCounter<f64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-60a611f1ad86c92f751d9a20"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableGauge<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::ObservableGauge", "path": "gauge::ObservableGauge"}}}, {"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [321, 1], "end": [321, 73], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:321`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableGauge<f64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-75ba96ccbd01ee11cfad4a05"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableGauge<u64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::ObservableGauge", "path": "gauge::ObservableGauge"}}}, {"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [320, 1], "end": [320, 73], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:320`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableGauge<u64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-8231efdc994017287d50255e"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableCounter<u64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::ObservableCounter", "path": "super::ObservableCounter"}}}, {"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [318, 1], "end": [318, 77], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:318`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableCounter<u64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-a09a1e04aafcc56de68a4863"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableUpDownCounter<i64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::ObservableUpDownCounter", "path": "super::ObservableUpDownCounter"}}}, {"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [323, 1], "end": [327, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:323`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableUpDownCounter<i64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-ecb4c826d325a42c0825f73e"></a>
## build

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> ObservableCounter<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::ObservableCounter", "path": "super::ObservableCounter"}}}, {"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [319, 1], "end": [319, 77], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:319`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `ObservableCounter<f64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-a651be636cdc16198790472d"></a>
## callbacks

`struct_field` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::callbacks` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
callbacks: Vec<Callback<M>>
```

Source: `src/metrics/instruments/mod.rs:259`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Callbacks to be called for this instrument.

<a id="op-1cf790de1943a0fa2d1c9465"></a>
## description

`struct_field` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::description` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
description: Option<std::borrow::Cow<'static, str>>
```

Source: `src/metrics/instruments/mod.rs:253`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Description of the instrument.

<a id="op-315cdaf0ea7e5daaa50f3c48"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "I"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "M"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrument", "path": "AsyncInstrument"}}}], "generic_params": [], "type": {"generic": "I"}}}]}, "is_negative": false, "span": {"begin": [334, 1], "end": [347, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/mod.rs:338`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a72ed650f810a21d7fe0d809"></a>
## instrument_provider

`struct_field` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::instrument_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
instrument_provider: &'a dyn InstrumentProvider
```

Source: `src/metrics/instruments/mod.rs:247`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Instrument provider is used to create the instrument.

<a id="op-5b4cc7235c8b112f2ac214c4"></a>
## name

`struct_field` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
name: std::borrow::Cow<'static, str>
```

Source: `src/metrics/instruments/mod.rs:250`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Name of the instrument.

<a id="op-d9259523d5eb2ce7db74b1ac"></a>
## unit

`struct_field` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::unit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
unit: Option<std::borrow::Cow<'static, str>>
```

Source: `src/metrics/instruments/mod.rs:256`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Unit of the instrument.

<a id="op-09ed0689262e5fb9b57c2e61"></a>
## with_callback

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::with_callback` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_callback<F>(self, callback: F) -> Self where F: Fn(&dyn AsyncInstrument<M>) + Send + Sync + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "I"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [264, 1], "end": [303, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:296`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the callback to be called for this instrument.

<a id="op-bb529d4fb05aa76f77039068"></a>
## with_description

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::with_description` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_description<S: Into<Cow<'static, str>>>(self, description: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "I"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [264, 1], "end": [303, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:278`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the description for this instrument

<a id="op-95c2c27c78bd14f3ff18a08b"></a>
## with_unit

`function` · `opentelemetry::metrics::instruments::AsyncInstrumentBuilder::with_unit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_unit<S: Into<Cow<'static, str>>>(self, unit: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "I"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::AsyncInstrumentBuilder", "path": "AsyncInstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [264, 1], "end": [303, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:290`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the unit for this instrument.

Unit is case sensitive(`kb` is not the same as `kB`).

Unit must be:
- ASCII string
- No longer than 63 characters
