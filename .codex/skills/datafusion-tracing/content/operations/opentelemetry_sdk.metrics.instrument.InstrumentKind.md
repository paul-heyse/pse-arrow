# `opentelemetry_sdk::metrics::instrument::InstrumentKind`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.instrument.InstrumentKind.json).

<a id="op-f07f1ed695aacb0bb0f046c4"></a>
## InstrumentKind

`enum` · `opentelemetry_sdk::metrics::instrument::InstrumentKind` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum InstrumentKind
```

Source: `src/metrics/instrument.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The identifier of a group of instruments that all perform the same function.

<a id="op-158198a4ef141e3ca6ba7202"></a>
## Counter

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::Counter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Counter
```

Source: `src/metrics/instrument.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Identifies a group of instruments that record increasing values synchronously
with the code path they are measuring.

<a id="op-e921dc1baa3f16edc7ca7a7d"></a>
## Gauge

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::Gauge` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Gauge
```

Source: `src/metrics/instrument.rs:38`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

a group of instruments that record current value synchronously with
the code path they are measuring.

<a id="op-227f3d42f1030d665f4cdd1f"></a>
## Histogram

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::Histogram` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Histogram
```

Source: `src/metrics/instrument.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A group of instruments that record a distribution of values synchronously with
the code path they are measuring.

<a id="op-58716102ecb946a6932a6fc5"></a>
## ObservableCounter

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::ObservableCounter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ObservableCounter
```

Source: `src/metrics/instrument.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A group of instruments that record increasing values in an asynchronous
callback.

<a id="op-4ba2ac98e906393fb6060c4e"></a>
## ObservableGauge

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::ObservableGauge` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ObservableGauge
```

Source: `src/metrics/instrument.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).


a group of instruments that record current values in an asynchronous callback.

<a id="op-302f646e128e3becdd37a876"></a>
## ObservableUpDownCounter

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::ObservableUpDownCounter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ObservableUpDownCounter
```

Source: `src/metrics/instrument.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A group of instruments that record increasing and decreasing values in an
asynchronous callback.

<a id="op-803bee2dc1a5d82f4b3ff702"></a>
## UpDownCounter

`variant` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::UpDownCounter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
UpDownCounter
```

Source: `src/metrics/instrument.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A group of instruments that record increasing and decreasing values
synchronously with the code path they are measuring.

<a id="op-c3adea4f49790b05030b108c"></a>
## clone

`function` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InstrumentKind
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::InstrumentKind", "path": "InstrumentKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 10], "end": [18, 15], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instrument.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2c9ecc67650ca23adc8300e6"></a>
## eq

`function` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &InstrumentKind) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::InstrumentKind", "path": "InstrumentKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 36], "end": [18, 45], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/instrument.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6cd3e868f3879d40a3ed9c90"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::InstrumentKind", "path": "InstrumentKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 23], "end": [18, 28], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instrument.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7c98f5019df51cd27723f024"></a>
## hash

`function` · `opentelemetry_sdk::metrics::instrument::InstrumentKind::hash` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::InstrumentKind", "path": "InstrumentKind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 30], "end": [18, 34], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/metrics/instrument.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
