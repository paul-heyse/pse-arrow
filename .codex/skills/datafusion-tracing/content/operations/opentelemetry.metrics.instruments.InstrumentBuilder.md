# `opentelemetry::metrics::instruments::InstrumentBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.InstrumentBuilder.json).

<a id="op-ce33325f0c1ff1722bb1fec4"></a>
## InstrumentBuilder

`struct` · `opentelemetry::metrics::instruments::InstrumentBuilder` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InstrumentBuilder<'a, T>
```

Source: `src/metrics/instruments/mod.rs:140`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configuration for building a sync instrument.

<a id="op-00fe164f5367e1c5c61d7bc9"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Counter<u64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::Counter", "path": "super::Counter"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [200, 1], "end": [200, 45], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:200`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `Counter<u64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-27b1440d55c521696586d37f"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Gauge<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "gauge::Gauge"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [203, 1], "end": [203, 41], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:203`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `Gauge<f64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-766361bd472cac47626f7343"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Gauge<u64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "gauge::Gauge"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [202, 1], "end": [202, 41], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:202`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `Gauge<u64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-7db80c3a3a425fd5172bef1b"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Counter<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::counter::Counter", "path": "super::Counter"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [201, 1], "end": [201, 45], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:201`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `Counter<f64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-87300be35654d8f9954228c0"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Gauge<i64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::gauge::Gauge", "path": "gauge::Gauge"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [204, 1], "end": [204, 41], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:204`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `Gauge<i64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-d3e16bac426e8528fed89c13"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> UpDownCounter<f64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::UpDownCounter", "path": "super::UpDownCounter"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [206, 1], "end": [206, 59], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:206`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `UpDownCounter<f64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-fd30372d3d457acd1e5399e6"></a>
## build

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> UpDownCounter<i64>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::up_down_counter::UpDownCounter", "path": "super::UpDownCounter"}}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [205, 1], "end": [205, 59], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:205`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Validates the instrument configuration and creates a new `UpDownCounter<i64>`.
In case of invalid configuration, a no-op instrument is returned
and an error is logged using internal logging.

<a id="op-b0d376adbc1307365592cd05"></a>
## description

`struct_field` · `opentelemetry::metrics::instruments::InstrumentBuilder::description` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
description: Option<std::borrow::Cow<'static, str>>
```

Source: `src/metrics/instruments/mod.rs:148`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Description of the instrument.

<a id="op-335f0deed7b6afc47a97b3e3"></a>
## fmt

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [208, 1], "end": [217, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instruments/mod.rs:209`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b5540d062c2aead2d045dfe1"></a>
## instrument_provider

`struct_field` · `opentelemetry::metrics::instruments::InstrumentBuilder::instrument_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
instrument_provider: &'a dyn InstrumentProvider
```

Source: `src/metrics/instruments/mod.rs:142`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Instrument provider is used to create the instrument.

<a id="op-b8c49393dfe59623685effba"></a>
## name

`struct_field` · `opentelemetry::metrics::instruments::InstrumentBuilder::name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
name: std::borrow::Cow<'static, str>
```

Source: `src/metrics/instruments/mod.rs:145`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Name of the instrument.

<a id="op-9a5bb4ac1ed1c6293bfc505b"></a>
## unit

`struct_field` · `opentelemetry::metrics::instruments::InstrumentBuilder::unit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
unit: Option<std::borrow::Cow<'static, str>>
```

Source: `src/metrics/instruments/mod.rs:151`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Unit of the instrument.

<a id="op-c58a6ec07223ef4fabea0fed"></a>
## with_description

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::with_description` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_description<S: Into<Cow<'static, str>>>(self, description: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [156, 1], "end": [185, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:169`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the description for this instrument

<a id="op-71a3b5a0e527ce79432ed589"></a>
## with_unit

`function` · `opentelemetry::metrics::instruments::InstrumentBuilder::with_unit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_unit<S: Into<Cow<'static, str>>>(self, unit: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry::metrics::instruments::InstrumentBuilder", "path": "InstrumentBuilder"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [156, 1], "end": [185, 2], "filename": "src/metrics/instruments/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instruments/mod.rs:181`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set the unit for this instrument.

Unit is case sensitive(`kb` is not the same as `kB`).

Unit must be:
- ASCII string
- No longer than 63 characters
