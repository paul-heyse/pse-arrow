# `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload`

Full upstream contracts; raw type trees and source locators in [structured records](instrumented_object_store.instrumented_object_store.InstrumentedMultiPartUpload.json).

<a id="op-6ba039b2badaff8e0efd5378"></a>
## InstrumentedMultiPartUpload

`struct` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentedMultiPartUpload
```

Source: `src/instrumented_object_store.rs:322`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

A wrapper around an `ObjectStore` that instruments all public methods with tracing.

<a id="op-9a80f3d3c282f34b3639fb39"></a>
## abort

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload::abort` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn abort(&mut self) -> Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload", "path": "InstrumentedMultiPartUpload"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [342, 1], "end": [373, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::upload::MultipartUpload", "path": "MultipartUpload"}, "trait_path": "object_store::upload::MultipartUpload"}`

Source: `src/instrumented_object_store.rs:363`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Abort the multipart upload with tracing.

<a id="op-43179201589b769892736985"></a>
## complete

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload::complete` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn complete(&mut self) -> Result<PutResult>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload", "path": "InstrumentedMultiPartUpload"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [342, 1], "end": [373, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::upload::MultipartUpload", "path": "MultipartUpload"}, "trait_path": "object_store::upload::MultipartUpload"}`

Source: `src/instrumented_object_store.rs:349`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Complete the multipart upload with tracing.

<a id="op-33e414e0a02565d00201903f"></a>
## fmt

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload::fmt` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload", "path": "InstrumentedMultiPartUpload"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [321, 10], "end": [321, 15], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/instrumented_object_store.rs:321`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1971c86e659223aeed1744fb"></a>
## inner

`struct_field` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload::inner` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: Box<dyn MultipartUpload>
```

Source: `src/instrumented_object_store.rs:323`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6f1be6cf6260cc7f73fc8dc2"></a>
## name

`struct_field` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload::name` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
name: String
```

Source: `src/instrumented_object_store.rs:324`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a31edfd4906e1025e2545e69"></a>
## put_part

`function` · `instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload::put_part` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn put_part(&mut self, data: PutPayload) -> UploadPart
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "instrumented_object_store::instrumented_object_store::InstrumentedMultiPartUpload", "path": "InstrumentedMultiPartUpload"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [342, 1], "end": [373, 2], "filename": "src/instrumented_object_store.rs"}, "trait": {"args": null, "id": "object_store::upload::MultipartUpload", "path": "MultipartUpload"}, "trait_path": "object_store::upload::MultipartUpload"}`

Source: `src/instrumented_object_store.rs:344`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

Upload a part without tracing (too many parts to trace).
