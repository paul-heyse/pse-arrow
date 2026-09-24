# `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.batch_log_processor.BatchLogProcessorBuilder.json).

<a id="op-cfbadbc769dbfc3acaa2a5a3"></a>
## BatchLogProcessorBuilder

`struct` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchLogProcessorBuilder<E>
```

Source: `src/logs/batch_log_processor.rs:550`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).


A builder for creating [`BatchLogProcessor`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchLogProcessor.md#op-ab788d8cd13fbb60e7393fa3) instances.


<a id="op-e65013a6ea4b5c5267b71e00"></a>
## build

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> BatchLogProcessor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder", "path": "BatchLogProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [555, 1], "end": [568, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:565`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Build a batch processor

<a id="op-d40518b672bd8ed37f7e9fb1"></a>
## fmt

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder", "path": "BatchLogProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [549, 10], "end": [549, 15], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/batch_log_processor.rs:549`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b10ccba517af0974b74328b5"></a>
## with_batch_config

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder::with_batch_config` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_batch_config(self, config: BatchConfig) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessorBuilder", "path": "BatchLogProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [555, 1], "end": [568, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:560`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the BatchConfig for [`BatchLogProcessorBuilder`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchLogProcessorBuilder.md#op-cfbadbc769dbfc3acaa2a5a3)
