# `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.in_memory_exporter.InMemoryMetricExporterBuilder.json).

<a id="op-de52b718f0173c036026e222"></a>
## InMemoryMetricExporterBuilder

`struct` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InMemoryMetricExporterBuilder
```

Source: `src/metrics/in_memory_exporter.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for [`InMemoryMetricExporter`](../operations/opentelemetry_sdk.metrics.in_memory_exporter.InMemoryMetricExporter.md#op-bce1568ad3e395f17ac0d53a).
# Example

```
# use opentelemetry_sdk::metrics::{InMemoryMetricExporter, InMemoryMetricExporterBuilder};

let exporter = InMemoryMetricExporterBuilder::new().build();
```

<a id="op-ad229857f07bc0be704f8aaa"></a>
## build

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> InMemoryMetricExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder", "path": "InMemoryMetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [112, 1], "end": [132, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/in_memory_exporter.rs:126`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of the `InMemoryMetricExporter`.


<a id="op-38a08c162a8e79f4831ca9bb"></a>
## default

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder", "path": "InMemoryMetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [106, 1], "end": [110, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/in_memory_exporter.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dd9ed92564d81488f9111bf1"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder", "path": "InMemoryMetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [100, 1], "end": [104, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/in_memory_exporter.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-209993542b9122c11687a583"></a>
## new

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder", "path": "InMemoryMetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [112, 1], "end": [132, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/in_memory_exporter.rs:114`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of the `InMemoryMetricExporterBuilder`.

<a id="op-59b7b9afc61812bc4fcee6e5"></a>
## with_temporality

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder::with_temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_temporality(self, temporality: Temporality) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporterBuilder", "path": "InMemoryMetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [112, 1], "end": [132, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/in_memory_exporter.rs:119`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the [Temporality](../operations/opentelemetry_sdk.metrics.Temporality.md#op-dc2884d798040a5373238d65) of the exporter.
