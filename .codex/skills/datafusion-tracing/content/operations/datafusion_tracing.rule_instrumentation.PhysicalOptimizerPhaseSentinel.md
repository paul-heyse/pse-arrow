# `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.PhysicalOptimizerPhaseSentinel.json).

<a id="op-7464cc681efa6245e5e5f07c"></a>
## PhysicalOptimizerPhaseSentinel

`struct` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct PhysicalOptimizerPhaseSentinel
```

Source: `src/rule_instrumentation.rs:362`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Sentinel physical optimizer rule that toggles the phase span.
First call opens the span, second call closes it.

<a id="op-a23a861b0df1c961edd73e42"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel", "path": "PhysicalOptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [367, 1], "end": [371, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:368`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-54fdeccc6139743b7c3d1bad"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel", "path": "PhysicalOptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [373, 1], "end": [426, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:419`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-07a3ea967e4efe6b73bc935a"></a>
## optimize

`function` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel::optimize` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn optimize(&self, plan: Arc<dyn ExecutionPlan>, _config: &ConfigOptions) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel", "path": "PhysicalOptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [373, 1], "end": [426, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:374`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-382b6c777eca5944bbd6d8e4"></a>
## phase_span_create_fn

`struct_field` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel::phase_span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
phase_span_create_fn: std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>
```

Source: `src/rule_instrumentation.rs:363`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cf80e740b11cd9a73ead0410"></a>
## plan_diff

`struct_field` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel::plan_diff` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
plan_diff: bool
```

Source: `src/rule_instrumentation.rs:364`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f35129d66530baf9b01ffba9"></a>
## schema_check

`function` · `datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel::schema_check` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema_check(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PhysicalOptimizerPhaseSentinel", "path": "PhysicalOptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [373, 1], "end": [426, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:423`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
