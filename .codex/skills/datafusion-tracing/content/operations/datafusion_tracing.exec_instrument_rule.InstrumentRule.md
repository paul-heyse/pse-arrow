# `datafusion_tracing::exec_instrument_rule::InstrumentRule`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.exec_instrument_rule.InstrumentRule.json).

<a id="op-f20287f3df9920ddc6c6137f"></a>
## InstrumentRule

`struct` · `datafusion_tracing::exec_instrument_rule::InstrumentRule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentRule
```

Source: `src/exec_instrument_rule.rs:53`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fe6b5ba77537ba4f93f148a6"></a>
## fmt

`function` · `datafusion_tracing::exec_instrument_rule::InstrumentRule::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::exec_instrument_rule::InstrumentRule", "path": "InstrumentRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [58, 1], "end": [62, 2], "filename": "src/exec_instrument_rule.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/exec_instrument_rule.rs:59`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9aaea7e1fdf715db6eff0361"></a>
## name

`function` · `datafusion_tracing::exec_instrument_rule::InstrumentRule::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::exec_instrument_rule::InstrumentRule", "path": "InstrumentRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [64, 1], "end": [95, 2], "filename": "src/exec_instrument_rule.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/exec_instrument_rule.rs:88`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7e5cd3e5f76b7ec197f8e484"></a>
## optimize

`function` · `datafusion_tracing::exec_instrument_rule::InstrumentRule::optimize` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn optimize(&self, plan: Arc<dyn ExecutionPlan>, _config: &ConfigOptions) -> datafusion::error::Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::exec_instrument_rule::InstrumentRule", "path": "InstrumentRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [64, 1], "end": [95, 2], "filename": "src/exec_instrument_rule.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/exec_instrument_rule.rs:65`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-684969bda668e91c48167872"></a>
## options

`struct_field` · `datafusion_tracing::exec_instrument_rule::InstrumentRule::options` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
options: options::InstrumentationOptions
```

Source: `src/exec_instrument_rule.rs:55`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4767eea180ca035dc8e7a1b3"></a>
## schema_check

`function` · `datafusion_tracing::exec_instrument_rule::InstrumentRule::schema_check` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema_check(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::exec_instrument_rule::InstrumentRule", "path": "InstrumentRule"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [64, 1], "end": [95, 2], "filename": "src/exec_instrument_rule.rs"}, "trait": {"args": null, "id": "datafusion_session::physical_optimizer::PhysicalOptimizerRule", "path": "PhysicalOptimizerRule"}, "trait_path": "datafusion_session::physical_optimizer::PhysicalOptimizerRule"}`

Source: `src/exec_instrument_rule.rs:92`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-84705a388ffb863ad6473246"></a>
## span_create_fn

`struct_field` · `datafusion_tracing::exec_instrument_rule::InstrumentRule::span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span_create_fn: std::sync::Arc<dyn Fn() -> tracing::Span + Send + Sync>
```

Source: `src/exec_instrument_rule.rs:54`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
