# `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.OptimizerPhaseSentinel.json).

<a id="op-82aca72785d989c88aba1f3d"></a>
## OptimizerPhaseSentinel

`struct` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct OptimizerPhaseSentinel
```

Source: `src/rule_instrumentation.rs:281`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Sentinel optimizer rule that toggles the phase span.
First call opens the span, second call closes it.

<a id="op-8b3d8a9f40e7e7ed0df5a60a"></a>
## apply_order

`function` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::apply_order` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn apply_order(&self) -> Option<ApplyOrder>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel", "path": "OptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [292, 1], "end": [358, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:297`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c63f510e4959cd86dce333f1"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel", "path": "OptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [286, 1], "end": [290, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:287`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cefca7a19fd54dea4dd0981b"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel", "path": "OptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [292, 1], "end": [358, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:293`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f9b56dc6a56af1782ecc268e"></a>
## phase_span_create_fn

`struct_field` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::phase_span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
phase_span_create_fn: std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>
```

Source: `src/rule_instrumentation.rs:282`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d888267863251ef180f6a20b"></a>
## plan_diff

`struct_field` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::plan_diff` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
plan_diff: bool
```

Source: `src/rule_instrumentation.rs:283`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aa582c438bd9f387ae7c54dd"></a>
## rewrite

`function` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::rewrite` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn rewrite(&self, plan: LogicalPlan, config: &dyn OptimizerConfig) -> Result<Transformed<LogicalPlan>, DataFusionError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel", "path": "OptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [292, 1], "end": [358, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:306`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9901df81d2159738e0494757"></a>
## supports_rewrite

`function` · `datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel::supports_rewrite` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn supports_rewrite(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::OptimizerPhaseSentinel", "path": "OptimizerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [292, 1], "end": [358, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::optimizer::OptimizerRule", "path": "OptimizerRule"}, "trait_path": "datafusion_optimizer::optimizer::OptimizerRule"}`

Source: `src/rule_instrumentation.rs:302`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
