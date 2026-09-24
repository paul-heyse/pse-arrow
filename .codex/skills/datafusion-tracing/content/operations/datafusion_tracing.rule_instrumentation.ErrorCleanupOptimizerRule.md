# `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.ErrorCleanupOptimizerRule.json).

<a id="op-bf9e1b28d95a6b602417890c"></a>
## ErrorCleanupOptimizerRule

`struct` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct ErrorCleanupOptimizerRule
```

Source: `src/rule_instrumentation.rs:460`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7d055755f06fda020dcbc5d6"></a>
## apply_order

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule::apply_order` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn apply_order(&self) -> Option<ApplyOrder>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule", "path": "ErrorCleanupOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [470, 1], "end": [495, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:475`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5aeadf0ff1297eea413bc054"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule", "path": "ErrorCleanupOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [464, 1], "end": [468, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:465`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-71660be89a37cd53c61f9585"></a>
## inner

`struct_field` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn OptimizerRule + Send + Sync>
```

Source: `src/rule_instrumentation.rs:461`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bef41705fd41c217532eeb2f"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule", "path": "ErrorCleanupOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [470, 1], "end": [495, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:471`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-36852b876ad1a26048dbb201"></a>
## rewrite

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule::rewrite` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn rewrite(&self, plan: LogicalPlan, config: &dyn OptimizerConfig) -> Result<Transformed<LogicalPlan>, DataFusionError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule", "path": "ErrorCleanupOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [470, 1], "end": [495, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:484`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-48bee0acc485a2ceb856f096"></a>
## supports_rewrite

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule::supports_rewrite` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn supports_rewrite(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupOptimizerRule", "path": "ErrorCleanupOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [470, 1], "end": [495, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:480`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
