# `opentelemetry::common::InstrumentationScopeBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.InstrumentationScopeBuilder.json).

<a id="op-3e6029749820ed5fdbf90c51"></a>
## InstrumentationScopeBuilder

`struct` · `opentelemetry::common::InstrumentationScopeBuilder` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InstrumentationScopeBuilder
```

Source: `src/common.rs:561`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configuration options for [InstrumentationScope](../operations/opentelemetry.common.InstrumentationScope.md#op-5a674011980d528952015efb).

An instrumentation scope is a library or crate providing instrumentation.
It should be named to follow any naming conventions of the instrumented
library (e.g. 'middleware' for a web framework).

Apart from the name, all other fields are optional.

See the [instrumentation libraries] spec for more information.

[instrumentation libraries]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries

<a id="op-527e37f2514ec29d179c2c23"></a>
## build

`function` · `opentelemetry::common::InstrumentationScopeBuilder::build` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> InstrumentationScope
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScopeBuilder", "path": "InstrumentationScopeBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [568, 1], "end": [625, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:617`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new [InstrumentationScope](../operations/opentelemetry.common.InstrumentationScope.md#op-5a674011980d528952015efb) from this configuration

<a id="op-174238d18a06d5c9f55c4717"></a>
## fmt

`function` · `opentelemetry::common::InstrumentationScopeBuilder::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScopeBuilder", "path": "InstrumentationScopeBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [560, 10], "end": [560, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:560`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-41dffb8c283325f2f76e2224"></a>
## with_attributes

`function` · `opentelemetry::common::InstrumentationScopeBuilder::with_attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_attributes<I>(self, attributes: I) -> Self where I: IntoIterator<Item = KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScopeBuilder", "path": "InstrumentationScopeBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [568, 1], "end": [625, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:608`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configure the attributes for the instrumentation scope

# Examples

```
use opentelemetry::KeyValue;

let scope = opentelemetry::InstrumentationScope::builder("my-crate")
    .with_attributes([KeyValue::new("k", "v")])
    .build();
```

<a id="op-db5300e95c9635b39f2c9b88"></a>
## with_schema_url

`function` · `opentelemetry::common::InstrumentationScopeBuilder::with_schema_url` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_schema_url(self, schema_url: impl Into<Cow<'static, str>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScopeBuilder", "path": "InstrumentationScopeBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [568, 1], "end": [625, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:592`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configure the Schema URL for the instrumentation scope

# Examples

```
let scope = opentelemetry::InstrumentationScope::builder("my-crate")
    .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
    .build();
```

<a id="op-cdeab6b313c0b4663b99048d"></a>
## with_version

`function` · `opentelemetry::common::InstrumentationScopeBuilder::with_version` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_version(self, version: impl Into<Cow<'static, str>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScopeBuilder", "path": "InstrumentationScopeBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [568, 1], "end": [625, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:578`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Configure the version for the instrumentation scope

# Examples

```
let scope = opentelemetry::InstrumentationScope::builder("my-crate")
    .with_version("v0.1.0")
    .build();
```
