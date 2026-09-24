# `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.SingleSpanTreeTraverser.json).

<a id="op-333ad5faa55ba99d8f9d3d8f"></a>
## SingleSpanTreeTraverser

`struct` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct SingleSpanTreeTraverser<'a>
```

Source: `src/rule_instrumentation.rs:538`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Applies an optimizer rule across the tree manually, keeping all work under a single span.

When a rule specifies `apply_order()` as `Some(TopDown)` or `Some(BottomUp)`, the optimizer
framework would normally handle tree traversal, calling the rule's `rewrite` method on each
node. This would result in one span per node if we simply wrapped the inner rule.

Instead, `SingleSpanTreeTraverser` takes over tree traversal so that the entire rule
application (across all nodes) is consolidated under a single tracing span. This gives
cleaner traces that show one span per rule rather than one span per (rule × node) combination.

<a id="op-ce59643d8d59c438a4f6de6d"></a>
## Node

`assoc_type` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser::Node` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Node
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser", "path": "SingleSpanTreeTraverser"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [558, 1], "end": [576, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_common::tree_node::TreeNodeRewriter", "path": "TreeNodeRewriter"}, "trait_path": "datafusion_common::tree_node::TreeNodeRewriter"}`

Source: `src/rule_instrumentation.rs:559`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dea794bf70901628c8dc4dd3"></a>
## apply_order

`struct_field` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser::apply_order` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
apply_order: datafusion::optimizer::optimizer::ApplyOrder
```

Source: `src/rule_instrumentation.rs:539`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5c3963981c644bf4551a1954"></a>
## config

`struct_field` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser::config` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
config: &'a dyn OptimizerConfig
```

Source: `src/rule_instrumentation.rs:541`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f963d64a74ea547c3a80cf2e"></a>
## f_down

`function` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser::f_down` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn f_down(&mut self, node: LogicalPlan) -> Result<Transformed<LogicalPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser", "path": "SingleSpanTreeTraverser"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [558, 1], "end": [576, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_common::tree_node::TreeNodeRewriter", "path": "TreeNodeRewriter"}, "trait_path": "datafusion_common::tree_node::TreeNodeRewriter"}`

Source: `src/rule_instrumentation.rs:561`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b7d7ed0daa3bf4ab888da76f"></a>
## f_up

`function` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser::f_up` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn f_up(&mut self, node: LogicalPlan) -> Result<Transformed<LogicalPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser", "path": "SingleSpanTreeTraverser"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [558, 1], "end": [576, 2], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "datafusion_common::tree_node::TreeNodeRewriter", "path": "TreeNodeRewriter"}, "trait_path": "datafusion_common::tree_node::TreeNodeRewriter"}`

Source: `src/rule_instrumentation.rs:569`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e96aa0311374c79b7abed9e9"></a>
## rule

`struct_field` · `datafusion_tracing::rule_instrumentation::SingleSpanTreeTraverser::rule` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
rule: &'a dyn OptimizerRule
```

Source: `src/rule_instrumentation.rs:540`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
