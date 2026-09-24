# `opentelemetry::metrics::InstrumentProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.InstrumentProvider.json).

<a id="op-c78b861fe6f28a675759c2e8"></a>
## InstrumentProvider

`trait` · `opentelemetry::metrics::InstrumentProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait InstrumentProvider
```

Source: `src/metrics/mod.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

SDK implemented trait for creating instruments

<a id="op-9d4e20b96daac865a23ba1d9"></a>
## f64_counter

`function` · `opentelemetry::metrics::InstrumentProvider::f64_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_counter(&self, _builder: InstrumentBuilder<'_, Counter<f64>>) -> Counter<f64>
```

Source: `src/metrics/mod.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording increasing values.

<a id="op-4d64f79e4ce5afba8001e5d9"></a>
## f64_gauge

`function` · `opentelemetry::metrics::InstrumentProvider::f64_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_gauge(&self, _builder: InstrumentBuilder<'_, Gauge<f64>>) -> Gauge<f64>
```

Source: `src/metrics/mod.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording independent values.

<a id="op-0af02f1dff3f2e84204bd3a4"></a>
## f64_histogram

`function` · `opentelemetry::metrics::InstrumentProvider::f64_histogram` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_histogram(&self, _builder: HistogramBuilder<'_, Histogram<f64>>) -> Histogram<f64>
```

Source: `src/metrics/mod.rs:118`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording a distribution of values.

<a id="op-dbef99b127c16a50c4acbb5d"></a>
## f64_observable_counter

`function` · `opentelemetry::metrics::InstrumentProvider::f64_observable_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_observable_counter(&self, _builder: AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>) -> ObservableCounter<f64>
```

Source: `src/metrics/mod.rs:39`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording increasing values via callback.

<a id="op-af28ce78011d85b1724cd256"></a>
## f64_observable_gauge

`function` · `opentelemetry::metrics::InstrumentProvider::f64_observable_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_observable_gauge(&self, _builder: AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>) -> ObservableGauge<f64>
```

Source: `src/metrics/mod.rs:110`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording the current value via callback.

<a id="op-f94c90b4ea923b92960b6cc9"></a>
## f64_observable_up_down_counter

`function` · `opentelemetry::metrics::InstrumentProvider::f64_observable_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_observable_up_down_counter(&self, _builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>) -> ObservableUpDownCounter<f64>
```

Source: `src/metrics/mod.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording changes of a value via callback.

<a id="op-36fbd0285e4560a0a2dd7caf"></a>
## f64_up_down_counter

`function` · `opentelemetry::metrics::InstrumentProvider::f64_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_up_down_counter(&self, _builder: InstrumentBuilder<'_, UpDownCounter<f64>>) -> UpDownCounter<f64>
```

Source: `src/metrics/mod.rs:55`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording changes of a value.

<a id="op-cf91d80fef723fd70208a8e1"></a>
## i64_gauge

`function` · `opentelemetry::metrics::InstrumentProvider::i64_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_gauge(&self, _builder: InstrumentBuilder<'_, Gauge<i64>>) -> Gauge<i64>
```

Source: `src/metrics/mod.rs:89`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording independent values.

<a id="op-787d3e504c80df796416db7d"></a>
## i64_observable_gauge

`function` · `opentelemetry::metrics::InstrumentProvider::i64_observable_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_observable_gauge(&self, _builder: AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>) -> ObservableGauge<i64>
```

Source: `src/metrics/mod.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording the current value via callback.

<a id="op-8688c5705daa227a7ab11e68"></a>
## i64_observable_up_down_counter

`function` · `opentelemetry::metrics::InstrumentProvider::i64_observable_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_observable_up_down_counter(&self, _builder: AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>) -> ObservableUpDownCounter<i64>
```

Source: `src/metrics/mod.rs:63`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording changes of a value.

<a id="op-a33412e4517bf0bb171da947"></a>
## i64_up_down_counter

`function` · `opentelemetry::metrics::InstrumentProvider::i64_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_up_down_counter(&self, _builder: InstrumentBuilder<'_, UpDownCounter<i64>>) -> UpDownCounter<i64>
```

Source: `src/metrics/mod.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording changes of a value.

<a id="op-d816c87425f1a43111e03873"></a>
## u64_counter

`function` · `opentelemetry::metrics::InstrumentProvider::u64_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_counter(&self, _builder: InstrumentBuilder<'_, Counter<u64>>) -> Counter<u64>
```

Source: `src/metrics/mod.rs:21`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording increasing values.

<a id="op-fb1980a53db33f519afdcdfc"></a>
## u64_gauge

`function` · `opentelemetry::metrics::InstrumentProvider::u64_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_gauge(&self, _builder: InstrumentBuilder<'_, Gauge<u64>>) -> Gauge<u64>
```

Source: `src/metrics/mod.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording independent values.

<a id="op-272109d183cd9f20368307a4"></a>
## u64_histogram

`function` · `opentelemetry::metrics::InstrumentProvider::u64_histogram` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_histogram(&self, _builder: HistogramBuilder<'_, Histogram<u64>>) -> Histogram<u64>
```

Source: `src/metrics/mod.rs:123`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording a distribution of values.

<a id="op-bd34b590282154c2b13eb24d"></a>
## u64_observable_counter

`function` · `opentelemetry::metrics::InstrumentProvider::u64_observable_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_observable_counter(&self, _builder: AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>) -> ObservableCounter<u64>
```

Source: `src/metrics/mod.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording increasing values via callback.

<a id="op-08d66d8b1a630fdb74e506d6"></a>
## u64_observable_gauge

`function` · `opentelemetry::metrics::InstrumentProvider::u64_observable_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_observable_gauge(&self, _builder: AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>) -> ObservableGauge<u64>
```

Source: `src/metrics/mod.rs:94`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument for recording the current value via callback.
