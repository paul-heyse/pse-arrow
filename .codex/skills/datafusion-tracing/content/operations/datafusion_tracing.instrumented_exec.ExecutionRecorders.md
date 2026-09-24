# `datafusion_tracing::instrumented_exec::ExecutionRecorders`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrumented_exec.ExecutionRecorders.json).

<a id="op-28dcfab9088f09e7c2d81fbb"></a>
## ExecutionRecorders

`struct` · `datafusion_tracing::instrumented_exec::ExecutionRecorders` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct ExecutionRecorders
```

Source: `src/instrumented_exec.rs:499`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-88bff2bd311488805a95ce20"></a>
## active_streams

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::active_streams` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
active_streams: std::sync::atomic::AtomicUsize
```

Source: `src/instrumented_exec.rs:504`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-455a15c0223c0c74c62d9cfb"></a>
## context

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::context` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
context: std::sync::Arc<datafusion::execution::TaskContext>
```

Source: `src/instrumented_exec.rs:502`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-22096d491320f3cae77e4313"></a>
## metrics_recorder

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::metrics_recorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
metrics_recorder: Option<std::sync::Arc<metrics::MetricsRecorder>>
```

Source: `src/instrumented_exec.rs:506`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b77fe1c45f8adbee7ad76968"></a>
## node_recorder

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::node_recorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
node_recorder: std::sync::Arc<node::NodeRecorder>
```

Source: `src/instrumented_exec.rs:505`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-615be7dc9c4ab6044ff9788c"></a>
## parent_span_id

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::parent_span_id` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
parent_span_id: Option<tracing::Id>
```

Source: `src/instrumented_exec.rs:501`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a6d50cd926cf7df0ef178ac1"></a>
## preview_recorder

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::preview_recorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_recorder: Option<std::sync::Arc<preview::PreviewRecorder>>
```

Source: `src/instrumented_exec.rs:507`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-44392dcc173e92391a67be77"></a>
## seen_partitions

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::seen_partitions` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
seen_partitions: std::sync::Mutex<std::collections::HashSet<usize>>
```

Source: `src/instrumented_exec.rs:503`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-03563ed2b8176f744ed019e9"></a>
## slot

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecorders::slot` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
slot: std::sync::Weak<std::sync::Mutex<Vec<std::sync::Arc<ExecutionRecorders>>>>
```

Source: `src/instrumented_exec.rs:500`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
