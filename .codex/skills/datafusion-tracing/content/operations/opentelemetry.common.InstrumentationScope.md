# `opentelemetry::common::InstrumentationScope`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.InstrumentationScope.json).

<a id="op-5a674011980d528952015efb"></a>
## InstrumentationScope

`struct` · `opentelemetry::common::InstrumentationScope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InstrumentationScope
```

Source: `src/common.rs:462`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Information about a library or crate providing instrumentation.

An instrumentation scope should be named to follow any naming conventions
of the instrumented library (e.g. 'middleware' for a web framework).

See the [instrumentation libraries] spec for more information.

[instrumentation libraries]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries

<a id="op-2e81a87b2259e9b62ec755d5"></a>
## attributes

`function` · `opentelemetry::common::InstrumentationScope::attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn attributes(&self) -> impl Iterator<Item = &KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [511, 1], "end": [547, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:544`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the instrumentation scope attributes to associate with emitted telemetry.

<a id="op-bfafe54686bcbbe8f6ce6b84"></a>
## builder

`function` · `opentelemetry::common::InstrumentationScope::builder` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder<T: Into<Cow<'static, str>>>(name: T) -> InstrumentationScopeBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [511, 1], "end": [547, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:513`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new builder to create an [InstrumentationScope](../operations/opentelemetry.common.InstrumentationScope.md#op-5a674011980d528952015efb)

<a id="op-c58cb4355901942b90b6fb17"></a>
## clone

`function` · `opentelemetry::common::InstrumentationScope::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InstrumentationScope
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 26], "end": [460, 31], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/common.rs:460`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-44c93bdafab855e77bba108a"></a>
## default

`function` · `opentelemetry::common::InstrumentationScope::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> InstrumentationScope
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 17], "end": [460, 24], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/common.rs:460`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4c562b104f23f629cb720d6e"></a>
## eq

`function` · `opentelemetry::common::InstrumentationScope::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [480, 1], "end": [494, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/common.rs:481`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a5d985d55ff75db57d795063"></a>
## fmt

`function` · `opentelemetry::common::InstrumentationScope::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 10], "end": [460, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:460`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-73e7d5e896e873abd20f6822"></a>
## hash

`function` · `opentelemetry::common::InstrumentationScope::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<H: hash::Hasher>(&self, state: &mut H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [498, 1], "end": [509, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/common.rs:499`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b060fe637f9e764f2f106b1a"></a>
## name

`function` · `opentelemetry::common::InstrumentationScope::name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [511, 1], "end": [547, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:524`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the instrumentation library name.

<a id="op-e582887343c27aca2c9cabc8"></a>
## schema_url

`function` · `opentelemetry::common::InstrumentationScope::schema_url` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn schema_url(&self) -> Option<&str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [511, 1], "end": [547, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:538`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the [Schema URL] used by this library.

[Schema URL]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url

<a id="op-db8163c89e2af4fb2989ade5"></a>
## version

`function` · `opentelemetry::common::InstrumentationScope::version` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn version(&self) -> Option<&str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::InstrumentationScope", "path": "InstrumentationScope"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [511, 1], "end": [547, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:530`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the instrumentation library version.
