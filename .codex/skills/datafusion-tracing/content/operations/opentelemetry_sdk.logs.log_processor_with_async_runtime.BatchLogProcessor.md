# `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.log_processor_with_async_runtime.BatchLogProcessor.json).

<a id="op-680b563a7b4760ad610d0e0b"></a>
## BatchLogProcessor

`struct` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchLogProcessor<R: RuntimeChannel>
```

Source: `src/logs/log_processor_with_async_runtime.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A [`LogProcessor`](../operations/opentelemetry_sdk.logs.log_processor.LogProcessor.md#op-006d2a495c0f4272e9c3d480) that asynchronously buffers log records and reports
them at a pre-configured interval.

<a id="op-142553f3b47bb06feefe74d1"></a>
## builder

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder<E>(exporter: E, runtime: R) -> BatchLogProcessorBuilder<E, R> where E: LogExporter
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [119, 1], "end": [227, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/log_processor_with_async_runtime.rs:217`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new batch processor builder

<a id="op-dd3dda151dbb44f356174954"></a>
## emit

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::emit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [61, 1], "end": [117, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/log_processor_with_async_runtime.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-70911d5120438f47d48a0fcc"></a>
## fmt

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [53, 1], "end": [59, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/log_processor_with_async_runtime.rs:54`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6c930029e167822961a95f3d"></a>
## force_flush

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [61, 1], "end": [117, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/log_processor_with_async_runtime.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-95e0b7155a05a3b43bdd2984"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [61, 1], "end": [117, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/log_processor_with_async_runtime.rs:111`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e085621a20f5d4124f088ee8"></a>
## shutdown

`function` · `opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::log_processor_with_async_runtime::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [61, 1], "end": [117, 2], "filename": "src/logs/log_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/log_processor_with_async_runtime.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
