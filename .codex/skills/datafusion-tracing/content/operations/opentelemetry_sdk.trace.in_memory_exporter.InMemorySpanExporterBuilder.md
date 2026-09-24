# `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.in_memory_exporter.InMemorySpanExporterBuilder.json).

<a id="op-104ac2ac3d92c62b9ebf006a"></a>
## InMemorySpanExporterBuilder

`struct` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InMemorySpanExporterBuilder
```

Source: `src/trace/in_memory_exporter.rs:72`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

 Builder for [`InMemorySpanExporter`](../operations/opentelemetry_sdk.trace.in_memory_exporter.InMemorySpanExporter.md#op-d2e217a7d8c3b8dfc5bb623b).
 # Example
 ```
# use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;

 let exporter = InMemorySpanExporterBuilder::new().build();
 ```

<a id="op-36565016c8d0936417fa96ec"></a>
## build

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(&self) -> InMemorySpanExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder", "path": "InMemorySpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [82, 1], "end": [108, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/in_memory_exporter.rs:91`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of the `InMemorySpanExporter`.

<a id="op-263489d12f1dbfc24231b418"></a>
## clone

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InMemorySpanExporterBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder", "path": "InMemorySpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [71, 10], "end": [71, 15], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/in_memory_exporter.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ed08b6d534ba86eb76a6bfcd"></a>
## default

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder", "path": "InMemorySpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [76, 1], "end": [80, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/in_memory_exporter.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-af256f7128ee2615bb9ab92e"></a>
## fmt

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder", "path": "InMemorySpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [71, 17], "end": [71, 22], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/in_memory_exporter.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2bc6e663de7ca239af99b7fd"></a>
## new

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporterBuilder", "path": "InMemorySpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [82, 1], "end": [108, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/in_memory_exporter.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of the `InMemorySpanExporterBuilder`.
