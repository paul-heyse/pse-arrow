# `opentelemetry::propagation::composite::TextMapCompositePropagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.composite.TextMapCompositePropagator.json).

<a id="op-ac007a31dd7afef4504e91e8"></a>
## TextMapCompositePropagator

`struct` · `opentelemetry::propagation::composite::TextMapCompositePropagator` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TextMapCompositePropagator
```

Source: `src/propagation/composite.rs:68`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Composite propagator for [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3)s.

A propagator that chains multiple [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3) propagators together,
injecting or extracting by their respective HTTP header names.

Injection and extraction from this propagator will preserve the order of the
injectors and extractors passed in during initialization.

# Examples

```
use opentelemetry::{
    baggage::BaggageExt,
    propagation::{TextMapPropagator, TextMapCompositePropagator},

    trace::{TraceContextExt, Tracer, TracerProvider},
    Context, KeyValue,
};
use opentelemetry_sdk::propagation::{
    BaggagePropagator, TraceContextPropagator,
};
use opentelemetry_sdk::trace as sdktrace;
use std::collections::HashMap;

// First create 1 or more propagators
let baggage_propagator = BaggagePropagator::new();
let trace_context_propagator = TraceContextPropagator::new();

// Then create a composite propagator
let composite_propagator = TextMapCompositePropagator::new(vec![
    Box::new(baggage_propagator),
    Box::new(trace_context_propagator),
]);

// Then for a given implementation of `Injector`
let mut injector = HashMap::new();

// And a given span
let example_span = sdktrace::SdkTracerProvider::default()
    .tracer("example-component")
    .start("span-name");

// with the current context, call inject to add the headers
composite_propagator.inject_context(
    &Context::current_with_span(example_span)
        .with_baggage(vec![KeyValue::new("test", "example")]),
    &mut injector,
);

// The injector now has both `baggage` and `traceparent` headers
assert!(injector.get("baggage").is_some());
assert!(injector.get("traceparent").is_some());
```

<a id="op-c3ef1285f07d51e517effbf8"></a>
## extract_with_context

`function` · `opentelemetry::propagation::composite::TextMapCompositePropagator::extract_with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::propagation::composite::TextMapCompositePropagator", "path": "TextMapCompositePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [90, 1], "end": [112, 2], "filename": "src/propagation/composite.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/composite.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Retrieves encoded `Context` information using the `Extractor`. If no data was
retrieved OR if the retrieved data is invalid, then the current `Context` is
returned.

<a id="op-eb13784409edb182f4be1c99"></a>
## fields

`function` · `opentelemetry::propagation::composite::TextMapCompositePropagator::fields` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> FieldIter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::propagation::composite::TextMapCompositePropagator", "path": "TextMapCompositePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [90, 1], "end": [112, 2], "filename": "src/propagation/composite.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/composite.rs:109`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aa1606e213c37cc1687acd7d"></a>
## fmt

`function` · `opentelemetry::propagation::composite::TextMapCompositePropagator::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::propagation::composite::TextMapCompositePropagator", "path": "TextMapCompositePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [67, 10], "end": [67, 15], "filename": "src/propagation/composite.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/propagation/composite.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-209932dd2346f00905738bf0"></a>
## inject_context

`function` · `opentelemetry::propagation::composite::TextMapCompositePropagator::inject_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn inject_context(&self, context: &Context, injector: &mut dyn Injector)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::propagation::composite::TextMapCompositePropagator", "path": "TextMapCompositePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [90, 1], "end": [112, 2], "filename": "src/propagation/composite.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/composite.rs:92`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Encodes the values of the `Context` and injects them into the `Injector`.

<a id="op-a5c3c94647c6c5e6e7497750"></a>
## new

`function` · `opentelemetry::propagation::composite::TextMapCompositePropagator::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::propagation::composite::TextMapCompositePropagator", "path": "TextMapCompositePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [73, 1], "end": [88, 2], "filename": "src/propagation/composite.rs"}, "trait": null, "trait_path": null}`

Source: `src/propagation/composite.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Constructs a new propagator out of instances of [`TextMapPropagator`].

[`TextMapPropagator`]: TextMapPropagator
