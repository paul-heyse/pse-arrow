# `opentelemetry_sdk::propagation::baggage::BaggagePropagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.propagation.baggage.BaggagePropagator.json).

<a id="op-db60cf5de7dcdd5c19369728"></a>
## BaggagePropagator

`struct` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BaggagePropagator
```

Source: `src/propagation/baggage.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Propagates name-value pairs in [W3C Baggage] format.

Baggage is used to annotate telemetry, adding context and
information to metrics, traces, and logs. It is an abstract data type
represented by a set of name-value pairs describing user-defined properties.
Each name in a [`Baggage`] is associated with exactly one value.
`Baggage`s are serialized according to the editor's draft of
the [W3C Baggage] specification.

# Examples

```
use opentelemetry::{baggage::{Baggage, BaggageExt}, propagation::TextMapPropagator};
use opentelemetry_sdk::propagation::BaggagePropagator;
use std::collections::HashMap;

// Example baggage value passed in externally via http headers
let mut headers = HashMap::new();
headers.insert("baggage".to_string(), "user_id=1".to_string());

let propagator = BaggagePropagator::new();
// can extract from any type that impls `Extractor`, usually an HTTP header map
let cx = propagator.extract(&headers);

// Iterate over extracted name-value pairs
for (name, value) in cx.baggage() {
    // ...
}

// Add new baggage
let mut baggage = Baggage::new();
let _ = baggage.insert("server_id", "42");

let cx_with_additions = cx.with_baggage(baggage);

// Inject baggage into http request
propagator.inject_context(&cx_with_additions, &mut headers);

let header_value = headers.get("baggage").expect("header is injected");
assert!(!header_value.contains("user_id=1"), "still contains previous name-value");
assert!(header_value.contains("server_id=42"), "does not contain new name-value pair");
```

[W3C Baggage]: https://w3c.github.io/baggage
[`Baggage`]: opentelemetry::baggage::Baggage

<a id="op-4a971dd485348864d29bb908"></a>
## default

`function` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> BaggagePropagator
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::baggage::BaggagePropagator", "path": "BaggagePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 17], "end": [66, 24], "filename": "src/propagation/baggage.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/propagation/baggage.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b10317ef82959a479bba9f1f"></a>
## extract_with_context

`function` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator::extract_with_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::baggage::BaggagePropagator", "path": "BaggagePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [78, 1], "end": [163, 2], "filename": "src/propagation/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/baggage.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Extracts a `Context` with baggage values from a `Extractor`.

<a id="op-a0341f699a0d7c7b1ac99afc"></a>
## fields

`function` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator::fields` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> FieldIter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::baggage::BaggagePropagator", "path": "BaggagePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [78, 1], "end": [163, 2], "filename": "src/propagation/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/baggage.rs:160`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2f45b8513c6530766adb6478"></a>
## fmt

`function` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::baggage::BaggagePropagator", "path": "BaggagePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 10], "end": [66, 15], "filename": "src/propagation/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/propagation/baggage.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d3b3773335d23165e6497922"></a>
## inject_context

`function` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator::inject_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn inject_context(&self, cx: &Context, injector: &mut dyn Injector)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::baggage::BaggagePropagator", "path": "BaggagePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [78, 1], "end": [163, 2], "filename": "src/propagation/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/baggage.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Encodes the values of the `Context` and injects them into the provided `Injector`.

<a id="op-dc02301180ead5f58e534b60"></a>
## new

`function` · `opentelemetry_sdk::propagation::baggage::BaggagePropagator::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::baggage::BaggagePropagator", "path": "BaggagePropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [71, 1], "end": [76, 2], "filename": "src/propagation/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/propagation/baggage.rs:73`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Construct a new baggage propagator.
