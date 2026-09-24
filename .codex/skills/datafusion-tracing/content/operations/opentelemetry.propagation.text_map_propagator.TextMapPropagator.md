# `opentelemetry::propagation::text_map_propagator::TextMapPropagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.text_map_propagator.TextMapPropagator.json).

<a id="op-1877064155d794d3de73d8d3"></a>
## TextMapPropagator

`trait` · `opentelemetry::propagation::text_map_propagator::TextMapPropagator` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait TextMapPropagator: Debug
```

Source: `src/propagation/text_map_propagator.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Methods to inject and extract a value as text into injectors and extractors that travel
in-band across process boundaries.

<a id="op-b30d43d0921c35a9c6c5f226"></a>
## extract

`function` · `opentelemetry::propagation::text_map_propagator::TextMapPropagator::extract` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn extract(&self, extractor: &dyn Extractor) -> Context
```

Source: `src/propagation/text_map_propagator.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Retrieves encoded data using the provided [`Extractor`](../operations/opentelemetry.propagation.Extractor.md#op-295c86db9e9eeabe7894a847). If no data for this
format was retrieved OR if the retrieved data is invalid, then the current
[`Context`] is returned.

[`Context`]: crate::Context
[`Injector`]: crate::propagation::Extractor

<a id="op-7f3545e4d22216f2107242af"></a>
## extract_with_context

`function` · `opentelemetry::propagation::text_map_propagator::TextMapPropagator::extract_with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context
```

Source: `src/propagation/text_map_propagator.rs:53`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Retrieves encoded data using the provided [`Extractor`](../operations/opentelemetry.propagation.Extractor.md#op-295c86db9e9eeabe7894a847). If no data for this
format was retrieved OR if the retrieved data is invalid, then the given
[`Context`] is returned.

[`Context`]: crate::Context
[`Injector`]: crate::propagation::Extractor

<a id="op-f6540633ee660a6f4db25ce9"></a>
## fields

`function` · `opentelemetry::propagation::text_map_propagator::TextMapPropagator::fields` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> FieldIter<'_>
```

Source: `src/propagation/text_map_propagator.rs:57`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns iter of fields used by [`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3)


<a id="op-e40454bf6901d93e024db07a"></a>
## inject

`function` · `opentelemetry::propagation::text_map_propagator::TextMapPropagator::inject` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn inject(&self, injector: &mut dyn Injector)
```

Source: `src/propagation/text_map_propagator.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Properly encodes the values of the current [`Context`] and injects them into
the [`Injector`].

[`Context`]: crate::Context
[`Injector`]: crate::propagation::Injector

<a id="op-e7c70733b0213839277ba954"></a>
## inject_context

`function` · `opentelemetry::propagation::text_map_propagator::TextMapPropagator::inject_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn inject_context(&self, cx: &Context, injector: &mut dyn Injector)
```

Source: `src/propagation/text_map_propagator.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Properly encodes the values of the [`Context`] and injects them into the
[`Injector`].

[`Context`]: crate::Context
[`Injector`]: crate::propagation::Injector
