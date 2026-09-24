# `datafusion_tracing::planner::TracingQueryPlanner`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.planner.TracingQueryPlanner.json).

<a id="op-db34eae2ec29c17c535d0bec"></a>
## TracingQueryPlanner

`struct` · `datafusion_tracing::planner::TracingQueryPlanner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct TracingQueryPlanner
```

Source: `src/planner.rs:51`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A `QueryPlanner` that instruments the creation of the physical plan.

This is automatically applied when instrumenting a `SessionState` with physical
optimizer instrumentation enabled (PhaseOnly or Full).

<a id="op-1a6a8e12bc778b15267b3419"></a>
## create_physical_plan

`function` · `datafusion_tracing::planner::TracingQueryPlanner::create_physical_plan` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
async fn create_physical_plan(&self, logical_plan: &LogicalPlan, session_state: &dyn Session) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::planner::TracingQueryPlanner", "path": "TracingQueryPlanner"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [122, 2], "filename": "src/planner.rs"}, "trait": {"args": null, "id": "datafusion_session::planner::QueryPlanner", "path": "QueryPlanner"}, "trait_path": "datafusion_session::planner::QueryPlanner"}`

Source: `src/planner.rs:81`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f0e8d6668df5387654605885"></a>
## fmt

`function` · `datafusion_tracing::planner::TracingQueryPlanner::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::planner::TracingQueryPlanner", "path": "TracingQueryPlanner"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [50, 10], "end": [50, 15], "filename": "src/planner.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/planner.rs:50`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fb94d33321341711e8195af1"></a>
## inner

`struct_field` · `datafusion_tracing::planner::TracingQueryPlanner::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn QueryPlanner + Send + Sync>
```

Source: `src/planner.rs:52`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8b9c3cd5f793a5fcff8bd0b6"></a>
## level

`struct_field` · `datafusion_tracing::planner::TracingQueryPlanner::level` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
level: tracing::Level
```

Source: `src/planner.rs:53`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
