# `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.InstrumentedAnalyzerRule.json).

<a id="op-5e9ff9687aec3a408a786e96"></a>
## InstrumentedAnalyzerRule

`struct` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentedAnalyzerRule
```

Source: `src/rule_instrumentation.rs:634`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A wrapper for an `AnalyzerRule` that adds tracing instrumentation.

<a id="op-89e5f29be6f2d3d04f335afe"></a>
## analyze

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule::analyze` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn analyze(&self, plan: LogicalPlan, config: &ConfigOptions) -> Result<LogicalPlan>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule", "path": "InstrumentedAnalyzerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [664, 1], "end": [717, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::analyzer::AnalyzerRule", "path": "AnalyzerRule"}, "trait_path": "datafusion_optimizer::analyzer::AnalyzerRule"}`

Source: `src/rule_instrumentation.rs:665`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-77f6a73acbef83bac59f62f0"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule", "path": "InstrumentedAnalyzerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [640, 1], "end": [647, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:641`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9bcbf4dd1795c9edf678962c"></a>
## inner

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn AnalyzerRule + Send + Sync>
```

Source: `src/rule_instrumentation.rs:635`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f7f31b77e16864e12589e604"></a>
## name

`function` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule", "path": "InstrumentedAnalyzerRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [664, 1], "end": [717, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_optimizer::analyzer::AnalyzerRule", "path": "AnalyzerRule"}, "trait_path": "datafusion_optimizer::analyzer::AnalyzerRule"}`

Source: `src/rule_instrumentation.rs:714`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a78c0717ea7b4ceda34037c1"></a>
## options

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule::options` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
options: rule_options::RuleInstrumentationOptions
```

Source: `src/rule_instrumentation.rs:636`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f85e028c3cc75417b8b817a8"></a>
## span_create_fn

`struct_field` · `datafusion_tracing::rule_instrumentation::InstrumentedAnalyzerRule::span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span_create_fn: std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>
```

Source: `src/rule_instrumentation.rs:637`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
