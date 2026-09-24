# `opentelemetry::propagation::Extractor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.propagation.Extractor.json).

<a id="op-295c86db9e9eeabe7894a847"></a>
## Extractor

`trait` · `opentelemetry::propagation::Extractor` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait Extractor
```

Source: `src/propagation/mod.rs:37`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Extractor provides an interface for removing fields from an underlying struct like `HashMap`

<a id="op-690476d0eff2f46c10280123"></a>
## get

`function` · `opentelemetry::propagation::Extractor::get` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get(&self, key: &str) -> Option<&str>
```

Source: `src/propagation/mod.rs:39`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Get a value from a key from the underlying data.

<a id="op-11399fa2b95a779c9c4fa91a"></a>
## get_all

`function` · `opentelemetry::propagation::Extractor::get_all` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_all(&self, key: &str) -> Option<Vec<&str>>
```

Source: `src/propagation/mod.rs:45`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Get all values from a key from the underlying data.

<a id="op-8d1ae2de84df592c83ec109d"></a>
## keys

`function` · `opentelemetry::propagation::Extractor::keys` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn keys(&self) -> Vec<&str>
```

Source: `src/propagation/mod.rs:42`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Collect all the keys from the underlying data.
