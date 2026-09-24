# `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.json).

<a id="op-1effe42b4434f4f4751060f0"></a>
## BatchConfigBuilder

`struct` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchConfigBuilder
```

Source: `src/trace/span_processor.rs:770`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A builder for creating [`BatchConfig`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfig.md#op-a720188ad3561d355afeaadf) instances.

<a id="op-78bbbdc40f8f77e6073dfe35"></a>
## build

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> BatchConfig
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [931, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:873`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builds a `BatchConfig` enforcing the following invariants:
* `max_export_batch_size` must be less than or equal to `max_queue_size`.

<a id="op-d95b60aa2738db5c7c017d9d"></a>
## default

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [778, 1], "end": [799, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/span_processor.rs:789`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new [`BatchConfigBuilder`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.md#op-1effe42b4434f4f4751060f0) initialized with default batch config values as per the specs.
The values are overriden by environment variables if set.
The supported environment variables are:
* `OTEL_BSP_MAX_QUEUE_SIZE`
* `OTEL_BSP_SCHEDULE_DELAY`
* `OTEL_BSP_MAX_EXPORT_BATCH_SIZE`
* `OTEL_BSP_EXPORT_TIMEOUT`
* `OTEL_BSP_MAX_CONCURRENT_EXPORTS`

Note: Programmatic configuration overrides any value set via the environment variable.

<a id="op-a8c1a1a7d9ccc6131d5b83c5"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [769, 10], "end": [769, 15], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_processor.rs:769`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fd1f63fb937a80c5d64a4ce3"></a>
## with_max_concurrent_exports

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::with_max_concurrent_exports` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_concurrent_exports(self, max_concurrent_exports: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [931, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:841`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_concurrent_exports for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.md#op-1effe42b4434f4f4751060f0).
It's the maximum number of concurrent exports.
Limits the number of spawned tasks for exports and thus memory consumed by an exporter.
The default value is 1.
If the max_concurrent_exports value is default value, it will cause exports to be performed
synchronously on the BatchSpanProcessor task.
The default value is 1.

Corresponding environment variable: `OTEL_BSP_MAX_CONCURRENT_EXPORTS`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-7b231c465e6efc9961b4d0d4"></a>
## with_max_export_batch_size

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::with_max_export_batch_size` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_export_batch_size(self, max_export_batch_size: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [931, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:824`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_export_batch_size for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.md#op-1effe42b4434f4f4751060f0).
It's the maximum number of spans to process in a single batch. If there are
more than one batch worth of spans then it processes multiple batches
of spans one batch after the other without any delay. The default value
is 512.

Corresponding environment variable: `OTEL_BSP_MAX_EXPORT_BATCH_SIZE`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-ba3a0fa8590070227d0a25d8"></a>
## with_max_export_timeout

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::with_max_export_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_export_timeout(self, max_export_timeout: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [931, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:866`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_export_timeout for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.md#op-1effe42b4434f4f4751060f0).
It's the maximum duration to export a batch of data.
The The default value is 30000 milliseconds.

Corresponding environment variable: `OTEL_BSP_EXPORT_TIMEOUT`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-2fd44e13de19d4c0dbf12f97"></a>
## with_max_queue_size

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::with_max_queue_size` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_queue_size(self, max_queue_size: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [931, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:810`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_queue_size for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.md#op-1effe42b4434f4f4751060f0).
It's the maximum queue size to buffer spans for delayed processing.
If the queue gets full it will drops the spans.
The default value is 2048.

Corresponding environment variable: `OTEL_BSP_MAX_QUEUE_SIZE`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-99048bb8c6bbec339e687e27"></a>
## with_scheduled_delay

`function` · `opentelemetry_sdk::trace::span_processor::BatchConfigBuilder::with_scheduled_delay` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_scheduled_delay(self, scheduled_delay: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [931, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:853`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set scheduled_delay_duration for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.trace.span_processor.BatchConfigBuilder.md#op-1effe42b4434f4f4751060f0).
It's the delay interval in milliseconds between two consecutive processing of batches.
The default value is 5000 milliseconds.

Corresponding environment variable: `OTEL_BSP_SCHEDULE_DELAY`.

Note: Programmatically setting this will override any value set via the environment variable.
