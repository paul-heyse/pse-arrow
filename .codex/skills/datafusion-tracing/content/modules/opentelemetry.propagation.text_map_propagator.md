# `opentelemetry::propagation::text_map_propagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.text_map_propagator.json).

<a id="op-aa6675242e08ab66d783ac5e"></a>
## text_map_propagator

`module` · `opentelemetry::propagation::text_map_propagator` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod text_map_propagator
```

Source: `src/propagation/text_map_propagator.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

# TextMapPropagator

[`TextMapPropagator`](../operations/opentelemetry.propagation.text_map_propagator.TextMapPropagator.md#op-1877064155d794d3de73d8d3) performs the injection and extraction of a cross-cutting concern value as
string key/values pairs into carriers that travel in-band across process boundaries.

The carrier of propagated data on both the client (injector) and server (extractor) side is
usually an HTTP request.

In order to increase compatibility, the key/value pairs MUST only consist of US-ASCII characters
that make up valid HTTP header fields as per RFC 7230.
