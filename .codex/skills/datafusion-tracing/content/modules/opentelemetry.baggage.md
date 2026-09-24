# `opentelemetry::baggage`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.baggage.json).

<a id="op-e162c64b0d80dd2c81eca851"></a>
## baggage

`module` · `opentelemetry::baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod baggage
```

Source: `src/baggage.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Primitives for sending name/value data across system boundaries.

Baggage is used to annotate telemetry, adding context and information to
metrics, traces, and logs. It is a set of name/value pairs describing
user-defined properties. Each name in Baggage is associated with exactly one
value.

Main types in this module are:

* [`Baggage`](../operations/opentelemetry.baggage.Baggage.md#op-2c88c4516fa3cd4da1db039e): A set of name/value pairs describing user-defined properties.
* [`BaggageExt`](../operations/opentelemetry.baggage.BaggageExt.md#op-2363a912d137e00f67b25220): Extensions for managing `Baggage` in a [`Context`](../operations/opentelemetry.context.Context.md#op-ca59114006a0744de26919a1).

Baggage can be sent between systems using a baggage propagator in
accordance with the [W3C Baggage] specification.

Note: Baggage is not automatically added to any telemetry. Users have to
explicitly add baggage entries to telemetry items.


[W3C Baggage]: https://w3c.github.io/baggage
