# `datafusion_tracing::options::InstrumentationOptions`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.options.InstrumentationOptions.json).

<a id="op-07a1777b923d8d00bd1e5318"></a>
## InstrumentationOptions

`struct` · `datafusion_tracing::options::InstrumentationOptions` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InstrumentationOptions
```

Source: `src/options.rs:26`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Configuration options for instrumented execution plans.

<a id="op-544cbe86afe45faf673ea438"></a>
## builder

`function` · `datafusion_tracing::options::InstrumentationOptions::builder` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> InstrumentationOptionsBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [64, 1], "end": [69, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:66`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Creates a new builder for `InstrumentationOptions`.

<a id="op-85edb618daab5beecfd7c11e"></a>
## clone

`function` · `datafusion_tracing::options::InstrumentationOptions::clone` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 10], "end": [25, 15], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/options.rs:25`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d848f1f80b0931bb641c2a23"></a>
## custom_fields

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::custom_fields` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
custom_fields: std::collections::HashMap<String, String>
```

Source: `src/options.rs:43`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

User-defined custom fields for extensible configuration.

This can be used to store arbitrary key-value pairs relevant to instrumentation or metadata.

<a id="op-0a7664ab753156eea3d6afc4"></a>
## default

`function` · `datafusion_tracing::options::InstrumentationOptions::default` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> InstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 17], "end": [25, 24], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/options.rs:25`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8942e801262068e4a7855b6e"></a>
## fmt

`function` · `datafusion_tracing::options::InstrumentationOptions::fmt` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [46, 1], "end": [62, 2], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/options.rs:47`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7e49b396c5d69f07388e5ff"></a>
## preview_fn

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::preview_fn` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
preview_fn: Option<std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>
```

Source: `src/options.rs:38`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

**Reference annotation (source_observation, separate from upstream):** None selects the default formatter when preview_limit is positive. Preview assembly spans available partitions; see the reviewed preview contract. [Evidence](../capabilities/tracing.preview.md).

Optional callback function for formatting previewed record batches.

The provided function will be invoked for each previewed batch of at most `preview_limit` rows.

<a id="op-853553f0e776bd23c0c3d16d"></a>
## preview_limit

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::preview_limit` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
preview_limit: usize
```

Source: `src/options.rs:33`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Maximum number of rows to preview per span.

If set to `0`, batch preview recording will be disabled.

<a id="op-fc972044e8836c1225ea7aef"></a>
## record_metrics

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::record_metrics` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
record_metrics: bool
```

Source: `src/options.rs:28`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Whether to record metrics during execution.

<a id="op-e8a1e87352fc8239e55d0d93"></a>
## InstrumentationOptions

`struct` · `datafusion_tracing::options::InstrumentationOptions` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
struct InstrumentationOptions
```

Source: `src/options.rs:26`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Configuration options for instrumented execution plans.

<a id="op-edc26eb5b5e7874d2db6f684"></a>
## builder

`function` · `datafusion_tracing::options::InstrumentationOptions::builder` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn builder() -> InstrumentationOptionsBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [64, 1], "end": [69, 2], "filename": "src/options.rs"}, "trait": null, "trait_path": null}`

Source: `src/options.rs:66`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Creates a new builder for `InstrumentationOptions`.

<a id="op-f2a92901b8d38cac1fb815bb"></a>
## clone

`function` · `datafusion_tracing::options::InstrumentationOptions::clone` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn clone(&self) -> InstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 10], "end": [25, 15], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/options.rs:25`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f4c2dab024d4f8ee215f2f1b"></a>
## custom_fields

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::custom_fields` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
custom_fields: std::collections::HashMap<String, String>
```

Source: `src/options.rs:43`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

User-defined custom fields for extensible configuration.

This can be used to store arbitrary key-value pairs relevant to instrumentation or metadata.

<a id="op-b191b32d629f017b12a635a1"></a>
## default

`function` · `datafusion_tracing::options::InstrumentationOptions::default` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn default() -> InstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [25, 17], "end": [25, 24], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/options.rs:25`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-000f526d1f6161cb492e54b7"></a>
## fmt

`function` · `datafusion_tracing::options::InstrumentationOptions::fmt` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::options::InstrumentationOptions", "path": "InstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [46, 1], "end": [62, 2], "filename": "src/options.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/options.rs:47`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9a19b737593814d4c01ca326"></a>
## preview_fn

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::preview_fn` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
preview_fn: Option<std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>
```

Source: `src/options.rs:38`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

**Reference annotation (source_observation, separate from upstream):** None selects the default formatter when preview_limit is positive. Preview assembly spans available partitions; see the reviewed preview contract. [Evidence](../capabilities/tracing.preview.md).

Optional callback function for formatting previewed record batches.

The provided function will be invoked for each previewed batch of at most `preview_limit` rows.

<a id="op-fb3eb9dc18e052d6e51dbcc4"></a>
## preview_limit

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::preview_limit` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
preview_limit: usize
```

Source: `src/options.rs:33`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Maximum number of rows to preview per span.

If set to `0`, batch preview recording will be disabled.

<a id="op-5ea0a315fca46647efb524f8"></a>
## record_metrics

`struct_field` · `datafusion_tracing::options::InstrumentationOptions::record_metrics` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
record_metrics: bool
```

Source: `src/options.rs:28`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Whether to record metrics during execution.
