# `opentelemetry_sdk::metrics::data::Exemplar`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.data.Exemplar.json).

<a id="op-95720da4f16acadf482dd981"></a>
## Exemplar

`struct` · `opentelemetry_sdk::metrics::data::Exemplar` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Exemplar<T>
```

Source: `src/metrics/data/mod.rs:573`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A measurement sampled from a time series providing a typical example.

<a id="op-13f7cd5e84183ec9ab8e02f3"></a>
## clone

`function` · `opentelemetry_sdk::metrics::data::Exemplar::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Exemplar<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [572, 17], "end": [572, 22], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/data/mod.rs:572`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3fd703ed540bf75fc0e1539b"></a>
## eq

`function` · `opentelemetry_sdk::metrics::data::Exemplar::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Exemplar<T>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "$crate::cmp::PartialEq"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [572, 24], "end": [572, 33], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metrics/data/mod.rs:572`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-65e98cd786db1eda5b985522"></a>
## filtered_attributes

`function` · `opentelemetry_sdk::metrics::data::Exemplar::filtered_attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn filtered_attributes(&self) -> impl Iterator<Item = &KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [591, 1], "end": [611, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:593`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns an iterator over the filtered attributes in [Exemplar](../operations/opentelemetry_sdk.metrics.data.Exemplar.md#op-95720da4f16acadf482dd981).

<a id="op-408625a732010bf8b40e5ab6"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::data::Exemplar::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [572, 10], "end": [572, 15], "filename": "src/metrics/data/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/data/mod.rs:572`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-984df5db8bc8e5150ab19e0e"></a>
## span_id

`function` · `opentelemetry_sdk::metrics::data::Exemplar::span_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_id(&self) -> &[u8; 8]
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [591, 1], "end": [611, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:603`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the ID of the span that was active during the measurement.

<a id="op-21b3771b3370d20d2274e64f"></a>
## time

`function` · `opentelemetry_sdk::metrics::data::Exemplar::time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn time(&self) -> SystemTime
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [591, 1], "end": [611, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:598`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the time when the measurement was recorded.

<a id="op-53cc4691cacaba621567495a"></a>
## trace_id

`function` · `opentelemetry_sdk::metrics::data::Exemplar::trace_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn trace_id(&self) -> &[u8; 16]
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::data::Exemplar", "path": "Exemplar"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [591, 1], "end": [611, 2], "filename": "src/metrics/data/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/data/mod.rs:608`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the ID of the trace the active span belonged to during the measurement.

<a id="op-3bbb6ae085ea4799152abb4c"></a>
## value

`struct_field` · `opentelemetry_sdk::metrics::data::Exemplar::value` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
value: T
```

Source: `src/metrics/data/mod.rs:580`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The measured value.
