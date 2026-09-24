# `instrumented_object_store`

Full upstream contracts; raw type trees and source locators in [structured records](instrumented_object_store.json).

<a id="op-e5ef45ac41ed038d1aebb1db"></a>
## instrumented_object_store

`module` · `instrumented_object_store` · instrumented-object-store 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
mod instrumented_object_store
```

Source: `src/lib.rs:20`. [Exact documentation build](https://docs.rs/crate/instrumented-object-store/55.0.0/json).

# Instrumented Object Store

Adds tracing instrumentation to any [Object Store](https://docs.rs/object_store/) implementation.

# Features

- Automatically captures spans for all storage operations (get, put, list, etc.)
- Records metadata like file paths and content sizes
- Captures error details when operations fail
- Works with OpenTelemetry for distributed tracing

# Getting Started

```rust
# use object_store::{path::Path, ObjectStore, ObjectStoreExt};
# use std::sync::Arc;
# use instrumented_object_store::instrument_object_store;
# use datafusion::execution::context::SessionContext;
# use url::Url;
# use object_store::Result;

# async fn example() -> Result<()> {
// Create your object store
let store = Arc::new(object_store::local::LocalFileSystem::new());

// Wrap it with instrumentation (prefix for span names)
let instrumented_store = instrument_object_store(store, "local_fs");

// Use directly for file operations
let result = instrumented_store.get(&Path::from("path/to/file")).await?;

// Or integrate with DataFusion
let ctx = SessionContext::new();
ctx.register_object_store(&Url::parse("file://").unwrap(), instrumented_store);
# Ok(())
# }
```

When combined with the [`datafusion-tracing`](https://github.com/datafusion-contrib/datafusion-tracing/tree/main/datafusion-tracing)
crate, this provides end-to-end visibility from query execution to storage operations.

<a id="op-1f92e763b8d19ef832d9691a"></a>
## instrumented_object_store

`module` · `instrumented_object_store` · instrumented-object-store 55.0.0
Reachability: `supported`.  Capture: private.

```rust
mod instrumented_object_store
```

Source: `src/lib.rs:20`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

# Instrumented Object Store

Adds tracing instrumentation to any [Object Store](https://docs.rs/object_store/) implementation.

# Features

- Automatically captures spans for all storage operations (get, put, list, etc.)
- Records metadata like file paths and content sizes
- Captures error details when operations fail
- Works with OpenTelemetry for distributed tracing

# Getting Started

```rust
# use object_store::{path::Path, ObjectStore, ObjectStoreExt};
# use std::sync::Arc;
# use instrumented_object_store::instrument_object_store;
# use datafusion::execution::context::SessionContext;
# use url::Url;
# use object_store::Result;

# async fn example() -> Result<()> {
// Create your object store
let store = Arc::new(object_store::local::LocalFileSystem::new());

// Wrap it with instrumentation (prefix for span names)
let instrumented_store = instrument_object_store(store, "local_fs");

// Use directly for file operations
let result = instrumented_store.get(&Path::from("path/to/file")).await?;

// Or integrate with DataFusion
let ctx = SessionContext::new();
ctx.register_object_store(&Url::parse("file://").unwrap(), instrumented_store);
# Ok(())
# }
```

When combined with the [`datafusion-tracing`](https://github.com/datafusion-contrib/datafusion-tracing/tree/main/datafusion-tracing)
crate, this provides end-to-end visibility from query execution to storage operations.
