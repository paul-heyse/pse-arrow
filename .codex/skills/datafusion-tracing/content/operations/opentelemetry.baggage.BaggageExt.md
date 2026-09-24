# `opentelemetry::baggage::BaggageExt`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.baggage.BaggageExt.json).

<a id="op-2363a912d137e00f67b25220"></a>
## BaggageExt

`trait` · `opentelemetry::baggage::BaggageExt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait BaggageExt
```

Source: `src/baggage.rs:336`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Methods for sorting and retrieving baggage data in a context.

<a id="op-b5e1ec1eb5dc2201a68d4a1e"></a>
## baggage

`function` · `opentelemetry::baggage::BaggageExt::baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn baggage(&self) -> &Baggage
```

Source: `src/baggage.rs:398`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a reference to this context's baggage, or the default
empty baggage if none has been set.

<a id="op-1313ce615b8500d8d8fb8f54"></a>
## current_with_baggage

`function` · `opentelemetry::baggage::BaggageExt::current_with_baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn current_with_baggage<T: Into<Baggage>>(baggage: T) -> Self
```

Source: `src/baggage.rs:381`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a clone of the current context with the included name/value pairs.

# Examples

```
use opentelemetry::{baggage::{Baggage, BaggageExt}, Context, StringValue};

let mut baggage = Baggage::new();
let _ = baggage.insert("my-name", "my-value");

let cx = Context::current_with_baggage(baggage);

assert_eq!(
    cx.baggage().get("my-name"),
    Some(&StringValue::from("my-value")),
)
```

<a id="op-7a55d8fd9da3e62a4357b9e3"></a>
## with_baggage

`function` · `opentelemetry::baggage::BaggageExt::with_baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_baggage<T: Into<Baggage>>(&self, baggage: T) -> Self
```

Source: `src/baggage.rs:362`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a clone of the given context with the included name/value pairs.

# Examples

```
use opentelemetry::{baggage::{Baggage, BaggageExt}, Context, KeyValue, StringValue};

// Explicit `Baggage` creation
let mut baggage = Baggage::new();
let _ = baggage.insert("my-name", "my-value");

let cx = Context::map_current(|cx| {
    cx.with_baggage(baggage)
});

// Passing an iterator
let cx = Context::map_current(|cx| {
    cx.with_baggage([KeyValue::new("my-name", "my-value")])
});

assert_eq!(
    cx.baggage().get("my-name"),
    Some(&StringValue::from("my-value")),
)
```

<a id="op-1aadbbb1360778d8d5a95f05"></a>
## with_cleared_baggage

`function` · `opentelemetry::baggage::BaggageExt::with_cleared_baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_cleared_baggage(&self) -> Self
```

Source: `src/baggage.rs:394`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a clone of the given context with no baggage.

# Examples

```
use opentelemetry::{baggage::BaggageExt, Context};

let cx = Context::map_current(|cx| cx.with_cleared_baggage());

assert_eq!(cx.baggage().len(), 0);
```
