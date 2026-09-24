# `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.InstrumentedPhysicalOptimizerRule.json).

<a id="op-71c73b6edf9c9fe055946164"></a>
## InstrumentedPhysicalOptimizerRule

`struct` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentedPhysicalOptimizerRule
```

Source: `src/rule_instrumentation.rs:850`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A wrapper for a `PhysicalOptimizerRule` that adds tracing instrumentation.

<a id="op-c8614efa4307f82087881821"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule", "path": "InstrumentedPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [856, 1], "end": [863, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:857`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-72ff5f9ab825ae4846613bab"></a>
## inner

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn PhysicalOptimizerRule + Send + Sync>
```

Source: `src/rule_instrumentation.rs:851`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-218888d926791ec43ac409ed"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule", "path": "InstrumentedPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [880, 1], "end": [940, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:933`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7dff49f8c560ade0d5d219df"></a>
## optimize

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::optimize` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn optimize(&self, plan: Arc<dyn ExecutionPlan>, config: &ConfigOptions) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule", "path": "InstrumentedPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [880, 1], "end": [940, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:881`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e9f23f595551f5d0098b861f"></a>
## options

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::options` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
options: rule_options::RuleInstrumentationOptions
```

Source: `src/rule_instrumentation.rs:852`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2da1be38ba40b4e8d70ba662"></a>
## schema_check

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::schema_check` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema_check(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule", "path": "InstrumentedPhysicalOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [880, 1], "end": [940, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/rule_instrumentation.rs:937`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-59138ac8db6fef7bc6fb89db"></a>
## span_create_fn

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedPhysicalOptimizerRule::span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span_create_fn: std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>
```

Source: `src/rule_instrumentation.rs:853`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
