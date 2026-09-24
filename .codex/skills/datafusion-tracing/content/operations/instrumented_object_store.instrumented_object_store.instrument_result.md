# `instrumented_object_store::instrumented_object_store::instrument_result`

Full upstream contracts; raw type trees and source locators in [structured records](instrumented_object_store.instrumented_object_store.instrument_result.json).

<a id="op-2949d7b56d433cef854100c0"></a>
## instrument_result

`function` · `instrumented_object_store::instrumented_object_store::instrument_result` · instrumented-object-store 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn instrument_result<T, E>(result: object_store::Result<T, E>) -> object_store::Result<T, E> where T: Instrumentable, E: std::error::Error
```

Source: `src/instrumented_object_store.rs:116`. [Exact documentation build](../../build/acquired/rustdoc/instrumented-object-store@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
