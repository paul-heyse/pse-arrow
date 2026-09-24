# `opentelemetry::propagation::Injector`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.Injector.json).

<a id="op-6df55495bf9714237f4c3690"></a>
## Injector

`trait` · `opentelemetry::propagation::Injector` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait Injector
```

Source: `src/propagation/mod.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Injector provides an interface for adding fields from an underlying struct like `HashMap`

<a id="op-b521c307747078d0ef5b9898"></a>
## set

`function` · `opentelemetry::propagation::Injector::set` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set(&mut self, key: &str, value: String)
```

Source: `src/propagation/mod.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Add a key and value to the underlying data.
