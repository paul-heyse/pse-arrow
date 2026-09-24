# `opentelemetry_sdk::metrics::instrument::Instrument`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.instrument.Instrument.json).

<a id="op-8cf16666479a8972c6770675"></a>
## Instrument

`struct` · `opentelemetry_sdk::metrics::instrument::Instrument` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Instrument
```

Source: `src/metrics/instrument.rs:97`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Describes the properties of an instrument at creation, used for filtering in
views. This is utilized in the `with_view` methods on `MeterProviderBuilder`
to customize metric output.

Users can use a reference to `Instrument` to select which instrument(s) a
[Stream](../operations/opentelemetry_sdk.metrics.instrument.Stream.md#op-58efe95b35cd74f5deb12539) should be applied to.

# Example

```rust
use opentelemetry_sdk::metrics::{Instrument, Stream};

let my_view_change_cardinality = |i: &Instrument| {
    if i.name() == "my_second_histogram" {
        // Note: If Stream is invalid, `build()` will return an error. By
        // calling `.ok()`, any such error is ignored and treated as if the
        // view does not match the instrument. If this is not the desired
        // behavior, consider handling the error explicitly.
        Stream::builder().with_cardinality_limit(2).build().ok()
    } else {
        None
    }
};
```

<a id="op-370a9dbdb5ef59ae5dcb8540"></a>
## clone

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Instrument
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 10], "end": [96, 15], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/instrument.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-de15788fb5a13fd65ad852ee"></a>
## eq

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Instrument) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 24], "end": [96, 33], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/instrument.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-069bc8d3d864e017c575c8d0"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 17], "end": [96, 22], "filename": "src/metrics/instrument.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/instrument.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-585d23fd99c7433cdf4c3151"></a>
## kind

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::kind` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn kind(&self) -> InstrumentKind
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [130, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:117`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrument kind.

<a id="op-39e7865c8fbf654666230448"></a>
## name

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [130, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:112`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrument name.

<a id="op-195cc33d776c883b9a36e0d7"></a>
## scope

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::scope` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn scope(&self) -> &InstrumentationScope
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [130, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:127`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrument scope.

<a id="op-27908d3e0bf1667c4324377d"></a>
## unit

`function` · `opentelemetry_sdk::metrics::instrument::Instrument::unit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn unit(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::instrument::Instrument", "path": "Instrument"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [130, 2], "filename": "src/metrics/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/instrument.rs:122`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrument unit.
