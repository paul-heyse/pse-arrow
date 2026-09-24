# `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.ErrorCleanupPhysicalOptimizerRule.json).

<a id="op-0c3567ac9b697869a820a759"></a>
## ErrorCleanupPhysicalOptimizerRule

`struct` · `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct ErrorCleanupPhysicalOptimizerRule
```

Source: `src/rule_instrumentation.rs:497`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-57a2317f2c0c39dcb18915fb"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule", "path": "ErrorCleanupPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [501, 1], "end": [505, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:502`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7cb316ac31d5e3af755e84f1"></a>
## inner

`struct_field` · `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn PhysicalOptimizerRule + Send + Sync>
```

Source: `src/rule_instrumentation.rs:498`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d7eaa38b1d8a2a2659420f75"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule", "path": "ErrorCleanupPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [507, 1], "end": [527, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:520`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1507413931c05a1bc5c3d1c5"></a>
## optimize

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule::optimize` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn optimize(&self, plan: Arc<dyn ExecutionPlan>, config: &ConfigOptions) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule", "path": "ErrorCleanupPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [507, 1], "end": [527, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:508`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7973383c75ff3e211b58e276"></a>
## schema_check

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule::schema_check` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema_check(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupPhysicalOptimizerRule", "path": "ErrorCleanupPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [507, 1], "end": [527, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:524`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
