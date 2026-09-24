# `opentelemetry::metrics::meter::Meter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.meter.Meter.json).

<a id="op-5e48a510d8c00731cf60cb7a"></a>
## Meter

`struct` · `opentelemetry::metrics::meter::Meter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Meter
```

Source: `src/metrics/meter.rs:296`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Provides the ability to create instruments for recording measurements or
accepting callbacks to report measurements.

# Instrument Types

Instruments are categorized as either synchronous or asynchronous:

- **Synchronous Instruments** (e.g., Counter): These are used inline with
  your application's processing logic. For example, you might use a Counter
  to record the number of HTTP requests received.

- **Asynchronous Instruments** (e.g., ObservableGauge): These allow you to
  register a callback function that is invoked during export. For instance,
  you could use an asynchronous gauge to monitor temperature from a sensor
  every time metrics are exported.

# Example Usage

```rust
use opentelemetry::{global, KeyValue};

let meter = global::meter("my-meter");

// Synchronous Instruments

// u64 Counter
let u64_counter = meter.u64_counter("my_u64_counter").build();
u64_counter.add(
    10,
    &[
        KeyValue::new("mykey1", "myvalue1"),
        KeyValue::new("mykey2", "myvalue2"),
    ],
);

// f64 Counter
let f64_counter = meter.f64_counter("my_f64_counter").build();
f64_counter.add(
    3.15,
    &[
        KeyValue::new("mykey1", "myvalue1"),
        KeyValue::new("mykey2", "myvalue2"),
    ],
);


// u64 Observable Counter
let _observable_u64_counter = meter
    .u64_observable_counter("my_observable_u64_counter")
    .with_description("My observable counter example")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            100,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// f64 Observable Counter
let _observable_f64_counter = meter
    .f64_observable_counter("my_observable_f64_counter")
    .with_description("My observable counter example")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            100.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// i64 UpDownCounter
let updown_i64_counter = meter.i64_up_down_counter("my_updown_i64_counter").build();
updown_i64_counter.add(
    -10,
    &[
        KeyValue::new("mykey1", "myvalue1"),
        KeyValue::new("mykey2", "myvalue2"),
    ],
);

// f64 UpDownCounter
let updown_f64_counter = meter.f64_up_down_counter("my_updown_f64_counter").build();
updown_f64_counter.add(
    -10.67,
    &[
        KeyValue::new("mykey1", "myvalue1"),
        KeyValue::new("mykey2", "myvalue2"),
    ],
);

// i64 Observable UpDownCounter
let _observable_updown_i64_counter = meter
    .i64_observable_up_down_counter("my_observable_i64_updown_counter")
    .with_description("My observable updown counter example")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            100,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// f64 Observable UpDownCounter
let _observable_updown_f64_counter = meter
    .f64_observable_up_down_counter("my_observable_f64_updown_counter")
    .with_description("My observable updown counter example")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            100.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// i64 Gauge
let gauge = meter.i64_gauge("my_gauge").build();
gauge.record(
-10,
&[
    KeyValue::new("mykey1", "myvalue1"),
    KeyValue::new("mykey2", "myvalue2"),
],
);

// u64 Gauge
let gauge = meter.u64_gauge("my_gauge").build();
gauge.record(
101,
&[
    KeyValue::new("mykey1", "myvalue1"),
    KeyValue::new("mykey2", "myvalue2"),
],
);

// f64 Gauge
let gauge = meter.f64_gauge("my_gauge").build();
gauge.record(
12.5,
&[
    KeyValue::new("mykey1", "myvalue1"),
    KeyValue::new("mykey2", "myvalue2"),
],
);

// u64 Observable Gauge
let _observable_u64_gauge = meter
    .u64_observable_gauge("my_u64_gauge")
    .with_description("An observable gauge set to 1")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            1,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// f64 Observable Gauge
let _observable_f64_gauge = meter
    .f64_observable_gauge("my_f64_gauge")
    .with_description("An observable gauge set to 1.0")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            1.0,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// i64 Observable Gauge
let _observable_i64_gauge = meter
    .i64_observable_gauge("my_i64_gauge")
    .with_description("An observable gauge set to 1")
    .with_unit("myunit")
    .with_callback(|observer| {
        observer.observe(
            1,
            &[
                KeyValue::new("mykey1", "myvalue1"),
                KeyValue::new("mykey2", "myvalue2"),
            ],
        )
    })
    .build();

// f64 Histogram
let f64_histogram = meter.f64_histogram("my_f64_histogram").build();
f64_histogram.record(
    10.5,
    &[
        KeyValue::new("mykey1", "myvalue1"),
        KeyValue::new("mykey2", "myvalue2"),
    ],
);

// u64 Histogram
let u64_histogram = meter.u64_histogram("my_u64_histogram").build();
u64_histogram.record(
    12,
    &[
        KeyValue::new("mykey1", "myvalue1"),
        KeyValue::new("mykey2", "myvalue2"),
    ],
);
```


<a id="op-c9463e0876c724532b4ca7f4"></a>
## clone

`function` · `opentelemetry::metrics::meter::Meter::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Meter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [294, 10], "end": [294, 15], "filename": "src/metrics/meter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/meter.rs:294`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3166a563a2583ff30341ef66"></a>
## f64_counter

`function` · `opentelemetry::metrics::meter::Meter::f64_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_counter(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, Counter<f64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:326`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording increasing values.

[`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) can be cloned to create multiple handles to the same instrument. If a [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) needs to be shared,
users are recommended to clone the [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) instead of creating duplicate [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5)s for the same metric. Creating
duplicate [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5)s for the same metric could lower SDK performance.

<a id="op-90ee96647a2e3372927d6c25"></a>
## f64_gauge

`function` · `opentelemetry::metrics::meter::Meter::f64_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_gauge(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, Gauge<f64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:410`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording independent values.

[`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) can be cloned to create multiple handles to the same instrument. If a [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) needs to be shared,
users are recommended to clone the [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) instead of creating duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric. Creating
duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric could lower SDK performance.

<a id="op-fc0d0240906fed1ad3fe173e"></a>
## f64_histogram

`function` · `opentelemetry::metrics::meter::Meter::f64_histogram` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_histogram(&self, name: impl Into<Cow<'static, str>>) -> HistogramBuilder<'_, Histogram<f64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:457`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording a distribution of values.

[`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de) can be cloned to create multiple handles to the same instrument. If a [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de) needs to be shared,
users are recommended to clone the [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de) instead of creating duplicate [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de)s for the same metric. Creating
duplicate [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de)s for the same metric could lower SDK performance.

<a id="op-ea7a3f5c6bd396251c2de2e2"></a>
## f64_observable_counter

`function` · `opentelemetry::metrics::meter::Meter::f64_observable_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_observable_counter(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableCounter<f64>, f64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:342`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording increasing values via callback.

<a id="op-61917a87ffb74fcb525bb070"></a>
## f64_observable_gauge

`function` · `opentelemetry::metrics::meter::Meter::f64_observable_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_observable_gauge(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableGauge<f64>, f64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:445`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording the current value via callback.

<a id="op-30b2db22247787117a6f8ae9"></a>
## f64_observable_up_down_counter

`function` · `opentelemetry::metrics::meter::Meter::f64_observable_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_observable_up_down_counter(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableUpDownCounter<f64>, f64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:386`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording changes of a value via callback.

<a id="op-4abde558f6512f39f48c5ed2"></a>
## f64_up_down_counter

`function` · `opentelemetry::metrics::meter::Meter::f64_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn f64_up_down_counter(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, UpDownCounter<f64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:366`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording changes of a value.

[`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) can be cloned to create multiple handles to the same instrument. If a [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) needs to be shared,
users are recommended to clone the [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) instead of creating duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric. Creating
duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric could lower SDK performance.

<a id="op-b0f24c383254ddda84ed0739"></a>
## fmt

`function` · `opentelemetry::metrics::meter::Meter::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [477, 1], "end": [481, 2], "filename": "src/metrics/meter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/meter.rs:478`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6359fb9db30ebc129fcda3f9"></a>
## i64_gauge

`function` · `opentelemetry::metrics::meter::Meter::i64_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_gauge(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, Gauge<i64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:421`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording independent values.
[`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) can be cloned to create multiple handles to the same instrument. If a [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) needs to be shared,
users are recommended to clone the [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) instead of creating duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric. Creating
duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric could lower SDK performance.

<a id="op-f90beccf6ce6cc125e1b9c9e"></a>
## i64_observable_gauge

`function` · `opentelemetry::metrics::meter::Meter::i64_observable_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_observable_gauge(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableGauge<i64>, i64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:437`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording the current value via callback.

<a id="op-dcf23a8f50512e7e6225a813"></a>
## i64_observable_up_down_counter

`function` · `opentelemetry::metrics::meter::Meter::i64_observable_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_observable_up_down_counter(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableUpDownCounter<i64>, i64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:378`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording changes of a value via callback.

[`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) can be cloned to create multiple handles to the same instrument. If a [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) needs to be shared,
users are recommended to clone the [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) instead of creating duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric. Creating
duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric could lower SDK performance.

<a id="op-319c3e3a909bb63fcced4e33"></a>
## i64_up_down_counter

`function` · `opentelemetry::metrics::meter::Meter::i64_up_down_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn i64_up_down_counter(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, UpDownCounter<i64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:354`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording changes of a value.

[`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) can be cloned to create multiple handles to the same instrument. If a [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) needs to be shared,
users are recommended to clone the [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc) instead of creating duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric. Creating
duplicate [`UpDownCounter`](../operations/opentelemetry.metrics.instruments.up_down_counter.UpDownCounter.md#op-b0254753eff4c3e7cb7010cc)s for the same metric could lower SDK performance.

<a id="op-1c6a2f1331ee01212b46481a"></a>
## u64_counter

`function` · `opentelemetry::metrics::meter::Meter::u64_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_counter(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, Counter<u64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:314`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording increasing values.

[`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) can be cloned to create multiple handles to the same instrument. If a [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) needs to be shared,
users are recommended to clone the [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5) instead of creating duplicate [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5)s for the same metric. Creating
duplicate [`Counter`](../operations/opentelemetry.metrics.instruments.counter.Counter.md#op-c72b21c1e56e7cb9d5a026e5)s for the same metric could lower SDK performance.

<a id="op-07416596cfba5749c3cdc71f"></a>
## u64_gauge

`function` · `opentelemetry::metrics::meter::Meter::u64_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_gauge(&self, name: impl Into<Cow<'static, str>>) -> InstrumentBuilder<'_, Gauge<u64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:398`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording independent values.

[`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) can be cloned to create multiple handles to the same instrument. If a [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) needs to be shared,
users are recommended to clone the [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf) instead of creating duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric. Creating
duplicate [`Gauge`](../operations/opentelemetry.metrics.instruments.gauge.Gauge.md#op-01a034e74bec61ee471db7bf)s for the same metric could lower SDK performance.

<a id="op-da8cd5f6af215e61ccd96075"></a>
## u64_histogram

`function` · `opentelemetry::metrics::meter::Meter::u64_histogram` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_histogram(&self, name: impl Into<Cow<'static, str>>) -> HistogramBuilder<'_, Histogram<u64>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:469`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording a distribution of values.

[`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de) can be cloned to create multiple handles to the same instrument. If a [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de) needs to be shared,
users are recommended to clone the [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de) instead of creating duplicate [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de)s for the same metric. Creating
duplicate [`Histogram`](../operations/opentelemetry.metrics.instruments.histogram.Histogram.md#op-ecc3b314e91435fd0e3cc0de)s for the same metric could lower SDK performance.

<a id="op-29c02ae0052fc21d0d4cb016"></a>
## u64_observable_counter

`function` · `opentelemetry::metrics::meter::Meter::u64_observable_counter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_observable_counter(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableCounter<u64>, u64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:334`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording increasing values via callback.

<a id="op-6ec76e1db799f71000c47bf9"></a>
## u64_observable_gauge

`function` · `opentelemetry::metrics::meter::Meter::u64_observable_gauge` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn u64_observable_gauge(&self, name: impl Into<Cow<'static, str>>) -> AsyncInstrumentBuilder<'_, ObservableGauge<u64>, u64>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::metrics::meter::Meter", "path": "Meter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [300, 1], "end": [475, 2], "filename": "src/metrics/meter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter.rs:429`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

creates an instrument builder for recording the current value via callback.
