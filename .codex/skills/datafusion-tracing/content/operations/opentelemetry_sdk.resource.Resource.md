# `opentelemetry_sdk::resource::Resource`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.Resource.json).

<a id="op-6665d2e0897d180bf11ccb3b"></a>
## Resource

`struct` · `opentelemetry_sdk::resource::Resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Resource
```

Source: `src/resource/mod.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An immutable representation of the entity producing telemetry as attributes.
Utilizes `Arc` for efficient sharing and cloning.

<a id="op-2f23705158039c2bdf7f978a"></a>
## builder

`function` · `opentelemetry_sdk::resource::Resource::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> ResourceBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a [ResourceBuilder](../operations/opentelemetry_sdk.resource.ResourceBuilder.md#op-d1d48f423a89b2235eac90cf) that allows you to configure multiple aspects of the Resource.

This [ResourceBuilder](../operations/opentelemetry_sdk.resource.ResourceBuilder.md#op-d1d48f423a89b2235eac90cf) will include the following [ResourceDetector](../operations/opentelemetry_sdk.resource.ResourceDetector.md#op-b0082125b9054a261f267f3c)s:
- [SdkProvidedResourceDetector](../operations/opentelemetry_sdk.resource.env.SdkProvidedResourceDetector.md#op-26736b4967b9ad610ce0d1d3)
- [TelemetryResourceDetector](../operations/opentelemetry_sdk.resource.telemetry.TelemetryResourceDetector.md#op-a1229b78527252090864a606)
- [EnvResourceDetector](../operations/opentelemetry_sdk.resource.env.EnvResourceDetector.md#op-ba004408ea4c1fba764aa989)
  If you'd like to start from an empty resource, use [Resource::builder_empty](../operations/opentelemetry_sdk.resource.Resource.md#op-118b9ba3138963be4de41014).

<a id="op-118b9ba3138963be4de41014"></a>
## builder_empty

`function` · `opentelemetry_sdk::resource::Resource::builder_empty` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder_empty() -> ResourceBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:75`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a [ResourceBuilder](../operations/opentelemetry_sdk.resource.ResourceBuilder.md#op-d1d48f423a89b2235eac90cf) that allows you to configure multiple aspects of the Resource.

This [ResourceBuilder](../operations/opentelemetry_sdk.resource.ResourceBuilder.md#op-d1d48f423a89b2235eac90cf) will not include any attributes or [ResourceDetector](../operations/opentelemetry_sdk.resource.ResourceDetector.md#op-b0082125b9054a261f267f3c)s by default.

<a id="op-0acedeec67df110c3e7e4e7d"></a>
## clone

`function` · `opentelemetry_sdk::resource::Resource::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Resource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [49, 10], "end": [49, 15], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/resource/mod.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f41a8f8e042abad239f16b23"></a>
## eq

`function` · `opentelemetry_sdk::resource::Resource::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Resource) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [49, 24], "end": [49, 33], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/resource/mod.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b2dcd93b16765645323a8ac8"></a>
## fmt

`function` · `opentelemetry_sdk::resource::Resource::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [49, 17], "end": [49, 22], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/resource/mod.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8f16083bc9f4e778e4730592"></a>
## get

`function` · `opentelemetry_sdk::resource::Resource::get` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get(&self, key: &Key) -> Option<Value>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:228`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Retrieve the value from resource associate with given key.

<a id="op-328a8357a8915ee691b754fe"></a>
## is_empty

`function` · `opentelemetry_sdk::resource::Resource::is_empty` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_empty(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:218`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns `true` if the resource contains no attributes.

<a id="op-5528e794adf5846f61a9828f"></a>
## iter

`function` · `opentelemetry_sdk::resource::Resource::iter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn iter(&self) -> Iter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:223`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Gets an iterator over the attributes of this resource.

<a id="op-b6ff0e0c8f22de87ad6acc2b"></a>
## len

`function` · `opentelemetry_sdk::resource::Resource::len` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn len(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:213`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the number of attributes for this resource

<a id="op-d15c77d13fe100b01e70a62b"></a>
## schema_url

`function` · `opentelemetry_sdk::resource::Resource::schema_url` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn schema_url(&self) -> Option<&str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::Resource", "path": "Resource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [231, 2], "filename": "src/resource/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/mod.rs:208`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Return the [schema url] of the resource. If the resource does not have a schema url, return `None`.

[schema url]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/schemas/overview.md#schema-url
