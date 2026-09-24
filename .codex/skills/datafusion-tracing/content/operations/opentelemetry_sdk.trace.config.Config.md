# `opentelemetry_sdk::trace::config::Config`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.config.Config.json).

<a id="op-e9dbb969b1dca66eda2cda64"></a>
## Config

`struct` · `opentelemetry_sdk::trace::config::Config` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Config
```

Source: `src/trace/config.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Tracer configuration

<a id="op-3f5cde0697113a406b5c7edb"></a>
## default

`function` · `opentelemetry_sdk::trace::config::Config::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::config::Config", "path": "Config"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [137, 2], "filename": "src/trace/config.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/config.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create default global sdk configuration.

<a id="op-34dede9c1bb83b67af3f8094"></a>
## fmt

`function` · `opentelemetry_sdk::trace::config::Config::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::config::Config", "path": "Config"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 10], "end": [13, 15], "filename": "src/trace/config.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/config.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-06f3ef6393bd894939d9f08c"></a>
## id_generator

`struct_field` · `opentelemetry_sdk::trace::config::Config::id_generator` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
id_generator: Box<dyn IdGenerator>
```

Source: `src/trace/config.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The id generator that the sdk should use

<a id="op-114fd4385b30e4ce5c69efbf"></a>
## resource

`struct_field` · `opentelemetry_sdk::trace::config::Config::resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
resource: std::borrow::Cow<'static, Resource>
```

Source: `src/trace/config.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Contains attributes representing an entity that produces telemetry.

<a id="op-e18fcd0d9f8859bf7816fa23"></a>
## sampler

`struct_field` · `opentelemetry_sdk::trace::config::Config::sampler` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
sampler: Box<dyn ShouldSample>
```

Source: `src/trace/config.rs:17`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The sampler that the sdk should use

<a id="op-dfc4c62e099a4c5fc2333134"></a>
## span_limits

`struct_field` · `opentelemetry_sdk::trace::config::Config::span_limits` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_limits: trace::span_limit::SpanLimits
```

Source: `src/trace/config.rs:23`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

span limits
