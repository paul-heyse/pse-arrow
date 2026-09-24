# `opentelemetry_sdk::trace::provider::TracerProviderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.provider.TracerProviderBuilder.json).

<a id="op-6fe1d2b72e96170f43701c81"></a>
## TracerProviderBuilder

`struct` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TracerProviderBuilder
```

Source: `src/trace/provider.rs:300`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for provider attributes.

<a id="op-7935276fc730239078d8e0b9"></a>
## build

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> SdkTracerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:426`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new provider from this configuration.

<a id="op-9e5cc98c74484ea6e0350001"></a>
## default

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> TracerProviderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [299, 17], "end": [299, 24], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/provider.rs:299`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0f049e7b6e5f5a3a0c29a975"></a>
## fmt

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [299, 10], "end": [299, 15], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/provider.rs:299`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-db98e251be7ab03b54991d24"></a>
## with_batch_exporter

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_batch_exporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_batch_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:334`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a [BatchSpanProcessor](../operations/opentelemetry_sdk.trace.span_processor.BatchSpanProcessor.md#op-7b6dded2e02e9c652c240628) with the configured exporter to the pipeline.

# Arguments

* `exporter` - The exporter to be used by the BatchSpanProcessor.

# Returns

A new `Builder` instance with the BatchSpanProcessor added to the pipeline.

Processors are invoked in the order they are added.

<a id="op-7110301bde82a8255443046c"></a>
## with_id_generator

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_id_generator` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_id_generator<T: IdGenerator + 'static>(self, id_generator: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:364`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the id generator to be used.

<a id="op-680b48bd4f86bc0ca65f7a90"></a>
## with_max_attributes_per_event

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_max_attributes_per_event` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_attributes_per_event(self, max_attributes: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:388`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the number of attributes one event can have.

<a id="op-7d398ffb6462660c9c4a3b33"></a>
## with_max_attributes_per_link

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_max_attributes_per_link` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_attributes_per_link(self, max_attributes: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:394`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the number of attributes one link can have.

<a id="op-945bff5127374e8ad20a20bf"></a>
## with_max_attributes_per_span

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_max_attributes_per_span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_attributes_per_span(self, max_attributes: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:376`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the number of attributes to be recorded per span.

<a id="op-9f7026b47713ffe6bbcb1d6a"></a>
## with_max_events_per_span

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_max_events_per_span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_events_per_span(self, max_events: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:370`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the number of events to be recorded per span.

<a id="op-1ecf9875076c9f38684c874c"></a>
## with_max_links_per_span

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_max_links_per_span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_links_per_span(self, max_links: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:382`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the number of events to be recorded per span.

<a id="op-6bfbf375db9948bff875dfa4"></a>
## with_resource

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_resource(self, resource: Resource) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:416`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Associates a [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) with a [SdkTracerProvider](../operations/opentelemetry_sdk.trace.provider.SdkTracerProvider.md#op-68f8d3c6d535fa6ce330056b).

This [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) represents the entity producing telemetry and is associated
with all [Tracer]s the [SdkTracerProvider](../operations/opentelemetry_sdk.trace.provider.SdkTracerProvider.md#op-68f8d3c6d535fa6ce330056b) will create.

By default, if this option is not used, the default [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) will be used.

*Note*: Calls to this method are additive, each call merges the provided
resource with the previous one.

[Tracer]: opentelemetry::trace::Tracer

<a id="op-89437084b80704bfcfd46f59"></a>
## with_sampler

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_sampler` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_sampler<T: trace::ShouldSample + 'static>(self, sampler: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:358`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify the sampler to be used.

<a id="op-2f7f04025a251c7ed2fd0e0a"></a>
## with_simple_exporter

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_simple_exporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_simple_exporter<T: SpanExporter + 'static>(self, exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:318`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a [SimpleSpanProcessor](../operations/opentelemetry_sdk.trace.span_processor.SimpleSpanProcessor.md#op-251eecc5a13ec3fb5a757796) with the configured exporter to the pipeline.

# Arguments

* `exporter` - The exporter to be used by the SimpleSpanProcessor.

# Returns

A new `Builder` instance with the SimpleSpanProcessor added to the pipeline.

Processors are invoked in the order they are added.

<a id="op-2d2dc0cda8e2c66aa41b8b00"></a>
## with_span_limits

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_span_limits` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_limits(self, span_limits: SpanLimits) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:400`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Specify all limit via the span_limits

<a id="op-7d0769432c46b7ccc7dcaca3"></a>
## with_span_processor

`function` · `opentelemetry_sdk::trace::provider::TracerProviderBuilder::with_span_processor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_processor<T: SpanProcessor + 'static>(self, processor: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::TracerProviderBuilder", "path": "TracerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [306, 1], "end": [465, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:350`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a custom [SpanProcessor](../operations/opentelemetry_sdk.trace.span_processor.SpanProcessor.md#op-d354c01a6b0b4f29c191ace7) to the pipeline.

# Arguments

* `processor` - The `SpanProcessor` to be added.

# Returns

A new `Builder` instance with the custom `SpanProcessor` added to the pipeline.

Processors are invoked in the order they are added.
