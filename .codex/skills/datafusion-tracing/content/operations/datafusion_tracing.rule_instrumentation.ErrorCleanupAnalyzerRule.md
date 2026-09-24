# `datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.ErrorCleanupAnalyzerRule.json).

<a id="op-053f3769275fc14c9b2afbaa"></a>
## ErrorCleanupAnalyzerRule

`struct` · `datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct ErrorCleanupAnalyzerRule
```

Source: `src/rule_instrumentation.rs:436`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-866cc13d6cd8ebb01be14f9e"></a>
## analyze

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule::analyze` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn analyze(&self, plan: LogicalPlan, config: &ConfigOptions) -> Result<LogicalPlan>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule", "path": "ErrorCleanupAnalyzerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [446, 1], "end": [458, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::analyzer::AnalyzerRule", "path": "AnalyzerRule"}, "trait_path": "datafusion_optimizer::analyzer::AnalyzerRule"}`

Source: `src/rule_instrumentation.rs:447`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-217f05e089f5614445a93b0c"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule", "path": "ErrorCleanupAnalyzerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 1], "end": [444, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:441`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9809e943c6038dbf44617ed5"></a>
## inner

`struct_field` · `datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn AnalyzerRule + Send + Sync>
```

Source: `src/rule_instrumentation.rs:437`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e1b3b83b69a9f740108343fe"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::ErrorCleanupAnalyzerRule", "path": "ErrorCleanupAnalyzerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [446, 1], "end": [458, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::analyzer::AnalyzerRule", "path": "AnalyzerRule"}, "trait_path": "datafusion_optimizer::analyzer::AnalyzerRule"}`

Source: `src/rule_instrumentation.rs:455`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
