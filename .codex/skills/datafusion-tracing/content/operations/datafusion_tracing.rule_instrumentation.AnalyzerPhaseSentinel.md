# `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.AnalyzerPhaseSentinel.json).

<a id="op-5bd14cbbc0105c89510c5f41"></a>
## AnalyzerPhaseSentinel

`struct` · `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct AnalyzerPhaseSentinel
```

Source: `src/rule_instrumentation.rs:228`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Sentinel analyzer rule that toggles the phase span.
First call opens the span, second call closes it.

<a id="op-3f6987d8c0303e886e08b6d3"></a>
## analyze

`function` · `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel::analyze` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn analyze(&self, plan: LogicalPlan, _config: &ConfigOptions) -> Result<LogicalPlan>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel", "path": "AnalyzerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [277, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::analyzer::AnalyzerRule", "path": "AnalyzerRule"}, "trait_path": "datafusion_optimizer::analyzer::AnalyzerRule"}`

Source: `src/rule_instrumentation.rs:240`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-36373175b950e9d539011c7a"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel", "path": "AnalyzerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [233, 1], "end": [237, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:234`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d421bbd33b60bc07ec2b8065"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel", "path": "AnalyzerPhaseSentinel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [277, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::analyzer::AnalyzerRule", "path": "AnalyzerRule"}, "trait_path": "datafusion_optimizer::analyzer::AnalyzerRule"}`

Source: `src/rule_instrumentation.rs:274`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d97fed761ee76496aeb5f974"></a>
## phase_span_create_fn

`struct_field` · `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel::phase_span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
phase_span_create_fn: std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>
```

Source: `src/rule_instrumentation.rs:229`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c877f1eb4f93a5e85483d53e"></a>
## plan_diff

`struct_field` · `datafusion_tracing::rule_instrumentation::AnalyzerPhaseSentinel::plan_diff` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
plan_diff: bool
```

Source: `src/rule_instrumentation.rs:230`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
