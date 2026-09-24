# `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.InstrumentedOptimizerRule.json).

<a id="op-7d1adb70014ae0baf523ed6b"></a>
## InstrumentedOptimizerRule

`struct` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentedOptimizerRule
```

Source: `src/rule_instrumentation.rs:720`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A wrapper for an `OptimizerRule` that adds tracing instrumentation.

<a id="op-99de00f2c77f2750b9ebac61"></a>
## apply_order

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::apply_order` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn apply_order(&self) -> Option<ApplyOrder>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule", "path": "InstrumentedOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [775, 1], "end": [847, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:780`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b1790a24aa1d9c680b5aca7d"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule", "path": "InstrumentedOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [726, 1], "end": [733, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:727`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-572fd1f43cc1ef1869bf70fe"></a>
## inner

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn OptimizerRule + Send + Sync>
```

Source: `src/rule_instrumentation.rs:721`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-415b0a539a20a92b462a739f"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule", "path": "InstrumentedOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [775, 1], "end": [847, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:776`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-51a53f336406f03c03f2b82b"></a>
## options

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::options` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
options: rule_options::RuleInstrumentationOptions
```

Source: `src/rule_instrumentation.rs:722`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8faa51c74641253c9e8b508e"></a>
## rewrite

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::rewrite` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn rewrite(&self, plan: LogicalPlan, config: &dyn OptimizerConfig) -> Result<Transformed<LogicalPlan>, DataFusionError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule", "path": "InstrumentedOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [775, 1], "end": [847, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:792`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-46497090c77189213e9a2feb"></a>
## span_create_fn

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span_create_fn: std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>
```

Source: `src/rule_instrumentation.rs:723`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c42c61df0bf29b6164506ee4"></a>
## supports_rewrite

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule::supports_rewrite` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn supports_rewrite(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedOptimizerRule", "path": "InstrumentedOptimizerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [775, 1], "end": [847, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:788`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
