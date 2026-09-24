# `opentelemetry_sdk::resource::ResourceBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.ResourceBuilder.json).

<a id="op-d1d48f423a89b2235eac90cf"></a>
## ResourceBuilder

`struct` · `opentelemetry_sdk::resource::ResourceBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct ResourceBuilder
```

Source: `src/resource/mod.rs:270`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b)

<a id="op-d4f49332899a7e04cc0b2e82"></a>
## build

`function` · `opentelemetry_sdk::resource::ResourceBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Resource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:322`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) with the options provided to the [ResourceBuilder](../operations/opentelemetry_sdk.resource.ResourceBuilder.md#op-d1d48f423a89b2235eac90cf).

<a id="op-06519ef0febd9e2dc9998801"></a>
## fmt

`function` · `opentelemetry_sdk::resource::ResourceBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [269, 10], "end": [269, 15], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/resource/mod.rs:269`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-86c209b19752e805cc1e32e1"></a>
## with_attribute

`function` · `opentelemetry_sdk::resource::ResourceBuilder::with_attribute` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_attribute(self, kv: KeyValue) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:287`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Add a [KeyValue](../operations/opentelemetry.common.KeyValue.md#op-46d9e1217e370ef8ca578118) to the resource.

<a id="op-0ed8d3af8ef996791b4e28c3"></a>
## with_attributes

`function` · `opentelemetry_sdk::resource::ResourceBuilder::with_attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_attributes<T: IntoIterator<Item = KeyValue>>(self, kvs: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:292`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Add multiple [KeyValue](../operations/opentelemetry.common.KeyValue.md#op-46d9e1217e370ef8ca578118)s to the resource.

<a id="op-79d685be0be5c5ce8d16277c"></a>
## with_detector

`function` · `opentelemetry_sdk::resource::ResourceBuilder::with_detector` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_detector(self, detector: Box<dyn ResourceDetector>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:276`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Add a single [ResourceDetector](../operations/opentelemetry_sdk.resource.ResourceDetector.md#op-b0082125b9054a261f267f3c) to your resource.

<a id="op-61042f593a630f40ef2305fc"></a>
## with_detectors

`function` · `opentelemetry_sdk::resource::ResourceBuilder::with_detectors` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_detectors(self, detectors: &[Box<dyn ResourceDetector>]) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:281`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Add multiple [ResourceDetector](../operations/opentelemetry_sdk.resource.ResourceDetector.md#op-b0082125b9054a261f267f3c)s to your resource.

<a id="op-8cff620064622fb6636804e6"></a>
## with_schema_url

`function` · `opentelemetry_sdk::resource::ResourceBuilder::with_schema_url` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_schema_url<KV, S>(self, attributes: KV, schema_url: S) -> Self where KV: IntoIterator<Item = KeyValue>, S: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:312`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

This will merge the provided `schema_url` with the current state of the Resource being built. It
will use the following rules to determine which `schema_url` should be used.

### [Schema url]
Schema url is determined by the following rules, in order:
1. If the current builder resource doesn't have a `schema_url`, the provided `schema_url` will be used.
2. If the current builder resource has a `schema_url`, and the provided `schema_url` is different from the builder resource, `schema_url` will be empty.
3. If the provided `schema_url` is the same as the current builder resource, it will be used.

[Schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url

<a id="op-36273593b8fc53cf04a45549"></a>
## with_service_name

`function` · `opentelemetry_sdk::resource::ResourceBuilder::with_service_name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_service_name(self, name: impl Into<Value>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::ResourceBuilder", "path": "ResourceBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [274, 1], "end": [325, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:298`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Add `service.name` resource attribute.
