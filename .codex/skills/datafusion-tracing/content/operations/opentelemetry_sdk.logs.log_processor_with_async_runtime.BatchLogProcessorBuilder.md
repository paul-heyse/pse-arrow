# `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.log_processor_with_async_runtime.BatchLogProcessorBuilder.json).

<a id="op-481f69474b48ba0bed468d01"></a>
## BatchLogProcessorBuilder

`struct` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchLogProcessorBuilder<E, R>
```

Source: `src/logs/log_processor_with_async_runtime.rs:261`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A builder for creating [`BatchLogProcessor`](../operations/opentelemetry_sdk.logs.log_processor_with_async_runtime.BatchLogProcessor.md#op-680b563a7b4760ad610d0e0b) instances.


<a id="op-c2f4405bf1641f4d360a9327"></a>
## build

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> BatchLogProcessor<R>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder", "path": "BatchLogProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [267, 1], "end": [281, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/log_processor_with_async_runtime.rs:278`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Build a batch processor

<a id="op-87286215ec354043f01ded5a"></a>
## fmt

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder", "path": "BatchLogProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [260, 10], "end": [260, 15], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/log_processor_with_async_runtime.rs:260`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d8b96fcd4a52711040431946"></a>
## with_batch_config

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder::with_batch_config` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_batch_config(self, config: BatchConfig) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessorBuilder", "path": "BatchLogProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [267, 1], "end": [281, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/log_processor_with_async_runtime.rs:273`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the BatchConfig for [`BatchLogProcessorBuilder`](../operations/opentelemetry_sdk.logs.log_processor_with_async_runtime.BatchLogProcessorBuilder.md#op-481f69474b48ba0bed468d01)
