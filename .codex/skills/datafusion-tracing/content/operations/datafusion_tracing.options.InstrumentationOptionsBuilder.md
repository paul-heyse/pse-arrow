# `datafusion_tracing::options::InstrumentationOptionsBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.options.InstrumentationOptionsBuilder.json).

<a id="op-b81c73da6546b193716dc43e"></a>
## InstrumentationOptionsBuilder

`struct` · `datafusion_tracing::options::InstrumentationOptionsBuilder` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: hosted.

```rust
struct InstrumentationOptionsBuilder
```

Source: `src/options.rs:73`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

The builder for `InstrumentationOptions`.

<a id="op-0add512fa26f518153402648"></a>
## InstrumentationOptionsBuilder

`struct` · `datafusion_tracing::options::InstrumentationOptionsBuilder` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
struct InstrumentationOptionsBuilder
```

Source: `src/options.rs:73`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The builder for `InstrumentationOptions`.

<a id="op-d0d5bef2332c095ef38fac56"></a>
## add_custom_field

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::add_custom_field` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn add_custom_field<K: Into<String>, V: Into<String>>(self, key: K, value: V) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [125, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:101`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Adds a single custom field.

<a id="op-520a9a4d7ab81e266aa0a50c"></a>
## build

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::build` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn build(self) -> InstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [125, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:117`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Consumes the builder and creates an `InstrumentationOptions` instance.

<a id="op-6a7d5a09553ed60495281982"></a>
## custom_fields

`struct_field` · `datafusion_tracing::options::InstrumentationOptionsBuilder::custom_fields` · datafusion-tracing 55.0.0
Reachability: `internal`. InstrumentationOptions::builder() Capture: private.

```rust
custom_fields: std::collections::HashMap<String, String>
```

Source: `src/options.rs:77`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-948cefb46ebe33191d0e28cb"></a>
## custom_fields

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::custom_fields` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn custom_fields(self, fields: HashMap<String, String>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [125, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:111`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Replaces all custom fields with the provided `HashMap`.

<a id="op-2c238775e3a9a1e9df0f02d7"></a>
## default

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::default` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn default() -> InstrumentationOptionsBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 10], "end": [72, 17], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/options.rs:72`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-09dbef872e0794334e22d0d1"></a>
## preview_fn

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::preview_fn` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn preview_fn(self, func: Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [125, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:95`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Sets the optional callback function for formatting previewed record batches.

<a id="op-5365945b6ddcbefe98458c2f"></a>
## preview_fn

`struct_field` · `datafusion_tracing::options::InstrumentationOptionsBuilder::preview_fn` · datafusion-tracing 55.0.0
Reachability: `internal`. InstrumentationOptions::builder() Capture: private.

```rust
preview_fn: Option<std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>
```

Source: `src/options.rs:76`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-10115deab09736db979aa497"></a>
## preview_limit

`struct_field` · `datafusion_tracing::options::InstrumentationOptionsBuilder::preview_limit` · datafusion-tracing 55.0.0
Reachability: `internal`. InstrumentationOptions::builder() Capture: private.

```rust
preview_limit: usize
```

Source: `src/options.rs:75`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d4b2d47b13c9aa255d85e495"></a>
## preview_limit

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::preview_limit` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn preview_limit(self, limit: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [125, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:89`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Sets the maximum number of rows to preview per span.
Setting this to `0` disables the batch preview.

<a id="op-a779b20995ab3ffb96b8b687"></a>
## record_metrics

`struct_field` · `datafusion_tracing::options::InstrumentationOptionsBuilder::record_metrics` · datafusion-tracing 55.0.0
Reachability: `internal`. InstrumentationOptions::builder() Capture: private.

```rust
record_metrics: bool
```

Source: `src/options.rs:74`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d9e4c09853923f411cd2ac0b"></a>
## record_metrics

`function` · `datafusion_tracing::options::InstrumentationOptionsBuilder::record_metrics` · datafusion-tracing 55.0.0
Reachability: `reachable-undocumented`. InstrumentationOptions::builder() Capture: private.

```rust
fn record_metrics(self, record: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptionsBuilder", "path": "InstrumentationOptionsBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [125, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:82`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Sets whether to record metrics during execution.
