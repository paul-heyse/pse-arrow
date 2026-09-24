# `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore`

Full upstream contracts; raw type trees and source locators in [structured records](instrumented_object_store.instrumented_object_store.InstrumentedObjectStore.json).

<a id="op-0130b59e00423e544cc7595a"></a>
## InstrumentedObjectStore

`struct` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentedObjectStore
```

Source: `src/instrumented_object_store.rs:45`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

A wrapper around an `ObjectStore` that instruments all public methods with tracing.

<a id="op-a126606a5791b3eef143375a"></a>
## clone

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::clone` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn clone(&self) -> InstrumentedObjectStore
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [44, 10], "end": [44, 15], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/instrumented_object_store.rs:44`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-401342cd27416d744ad1d9ce"></a>
## copy_opts

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::copy_opts` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:280`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Copy an object from one path to another with tracing.

<a id="op-54b7410610fe5675813caa8c"></a>
## delete_stream

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::delete_stream` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn delete_stream(&self, locations: BoxStream<'static, Result<Path>>) -> BoxStream<'static, Result<Path>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:223`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Delete all the objects at the specified locations with tracing.

<a id="op-16a4a492041b2da16fa4595c"></a>
## fmt

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::fmt` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [44, 17], "end": [44, 22], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/instrumented_object_store.rs:44`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8514fd9dd482b07e6ecea905"></a>
## fmt

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::fmt` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [135, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/instrumented_object_store.rs:132`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0dd1321bea26126954056cfb"></a>
## get_opts

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::get_opts` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn get_opts(&self, location: &Path, options: GetOptions) -> Result<GetResult>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:182`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Perform a get request with options and tracing.

<a id="op-1be68758de167e1d9196c27a"></a>
## get_ranges

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::get_ranges` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn get_ranges(&self, location: &Path, ranges: &[Range<u64>]) -> Result<Vec<Bytes>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:198`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Return the bytes that are stored at the specified location in the given byte ranges with tracing.

<a id="op-e5e6b9d4fe7c7f0cbbe7a9f0"></a>
## inner

`struct_field` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::inner` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn ObjectStore>
```

Source: `src/instrumented_object_store.rs:46`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f0baa94edfb0d579c74b805"></a>
## list

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::list` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:241`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

List all the objects with the given prefix with tracing.

<a id="op-286e50b65442d0b058b61db0"></a>
## list_with_delimiter

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::list_with_delimiter` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:266`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

List objects with the given prefix and delimiter with tracing.

<a id="op-61a946f8f6a1f186f9f89c0c"></a>
## list_with_offset

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::list_with_offset` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn list_with_offset(&self, prefix: Option<&Path>, offset: &Path) -> BoxStream<'static, Result<ObjectMeta>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:254`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

List all the objects with the given prefix and offset with tracing.

<a id="op-36dc0225c97c6114a583c687"></a>
## name

`struct_field` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::name` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
name: String
```

Source: `src/instrumented_object_store.rs:47`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-baf5464a073e1c2609c2d9e4"></a>
## new

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::new` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(store: Arc<dyn ObjectStore>, name: &str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [50, 1], "end": [62, 2], "filename": "src/instrumented_object_store.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrumented_object_store.rs:56`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Creates a new `InstrumentedObjectStore` wrapping the provided `ObjectStore`.

# Arguments

* `store` - An `Arc`-wrapped `dyn ObjectStore` to be instrumented.

<a id="op-65a5859903dbcf4f80e3c1b4"></a>
## put_multipart_opts

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::put_multipart_opts` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn put_multipart_opts(&self, location: &Path, opts: PutMultipartOptions) -> Result<Box<dyn MultipartUpload>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:162`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Perform a multipart upload with options and tracing.

<a id="op-a7ae90fbb7b5afe6e84f1e22"></a>
## put_opts

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::put_opts` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn put_opts(&self, location: &Path, payload: PutPayload, opts: PutOptions) -> Result<PutResult>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:141`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Save the provided payload to location with the given options and tracing.

<a id="op-240a63f8b0762229f31cd6da"></a>
## rename_opts

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedObjectStore::rename_opts` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn rename_opts(&self, from: &Path, to: &Path, options: RenameOptions) -> Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedObjectStore", "path": "InstrumentedObjectStore"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [318, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::ObjectStore", "path": "ObjectStore"}, "trait_path": "object_store::ObjectStore"}`

Source: `src/instrumented_object_store.rs:300`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Move an object from one path to another with tracing.
