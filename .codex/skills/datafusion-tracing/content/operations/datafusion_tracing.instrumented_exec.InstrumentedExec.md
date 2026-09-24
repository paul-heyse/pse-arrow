# `datafusion_tracing::instrumented_exec::InstrumentedExec`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrumented_exec.InstrumentedExec.json).

<a id="op-6b12316c8f26fe0ad5f029de"></a>
## InstrumentedExec

`struct` · `datafusion_tracing::instrumented_exec::InstrumentedExec` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct InstrumentedExec
```

Source: `src/instrumented_exec.rs:69`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

An [`ExecutionPlan`] wrapper that instruments execution with tracing spans and metrics recording.

Unresolved upstream links (retained, not inferred): ``ExecutionPlan``.

<a id="op-cff7d9cb70ad7b3a1eff0e48"></a>
## apply_expressions

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::apply_expressions` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn apply_expressions(&self, f: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>) -> Result<TreeNodeRecursion>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:296`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate expression traversal to the wrapped execution plan.

<a id="op-38f691444115cb2976b25976"></a>
## benefits_from_input_partitioning

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::benefits_from_input_partitioning` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn benefits_from_input_partitioning(&self) -> Vec<bool>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:266`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f4311bc0d53b2f60470aa48"></a>
## cardinality_effect

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::cardinality_effect` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn cardinality_effect(&self) -> CardinalityEffect
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:277`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4431e102b2f937b8a8aabab7"></a>
## check_invariants

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::check_invariants` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn check_invariants(&self, check: InvariantLevel) -> Result<()>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:261`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-40cd3c3a973eaeec30c45ee6"></a>
## child_stats_requests

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::child_stats_requests` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn child_stats_requests(&self, partition: Option<usize>) -> Vec<ChildStats>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:274`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-99e95f847335da0bbf946e15"></a>
## children

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::children` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:267`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0ca0d6d4eb9b08d6694d6fbb"></a>
## downcast_delegate

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::downcast_delegate` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn downcast_delegate(&self) -> Option<&dyn ExecutionPlan>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:446`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate public downcasts to the inner plan so instrumentation stays
transparent to normal plan inspection.

<a id="op-19559034239a4b4a4a98026e"></a>
## dynamic_expressions_produced

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::dynamic_expressions_produced` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn dynamic_expressions_produced(&self) -> Vec<Arc<dyn PhysicalExpr>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:262`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bcc3f78bd3f4c82028a43f48"></a>
## execute

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::execute` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn execute(&self, partition: usize, context: Arc<TaskContext>) -> Result<SendableRecordBatchStream>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:451`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Executes the plan for a given partition and context, instrumented with tracing and metrics recording.

<a id="op-049ab9af58f497e618e4d2b6"></a>
## fetch

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::fetch` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fetch(&self) -> Option<usize>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:276`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-06b16ec3b8b6de842f2e0b48"></a>
## fmt

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 1], "end": [97, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/instrumented_exec.rs:92`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-18bfead2c10fba7a78e5a441"></a>
## fmt_as

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::fmt_as` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt_as(&self, format: DisplayFormatType, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [617, 1], "end": [623, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::display::DisplayAs", "path": "DisplayAs"}, "trait_path": "datafusion_physical_plan::display::DisplayAs"}`

Source: `src/instrumented_exec.rs:620`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7ad14c45eff43f3cd41936d8"></a>
## gather_filters_for_pushdown

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::gather_filters_for_pushdown` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn gather_filters_for_pushdown(&self, phase: FilterPushdownPhase, parent_filters: Vec<Arc<dyn PhysicalExpr>>, config: &ConfigOptions) -> Result<FilterDescription>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:278`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-71565c175ffe3d07ddfe1c38"></a>
## handle_child_pushdown_result

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::handle_child_pushdown_result` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn handle_child_pushdown_result(&self, phase: FilterPushdownPhase, child_pushdown_result: ChildPushdownResult, config: &ConfigOptions) -> Result<FilterPushdownPropagation<Arc<dyn ExecutionPlan>>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:357`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for handling child pushdown result and rewrap with an InstrumentedExec.

<a id="op-f5a31f96464c49df742a6f98"></a>
## inner

`struct_field` · `datafusion_tracing::instrumented_exec::InstrumentedExec::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: std::sync::Arc<dyn ExecutionPlan>
```

Source: `src/instrumented_exec.rs:71`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The inner execution plan to delegate execution to.

<a id="op-c9ebe83aee1284ab75a8ad49"></a>
## input_distribution_requirements

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::input_distribution_requirements` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn input_distribution_requirements(&self) -> InputDistributionRequirements
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:263`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c1410a4746144b333e7eb30a"></a>
## maintains_input_order

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::maintains_input_order` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn maintains_input_order(&self) -> Vec<bool>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:265`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-36184ce223d44f4826b2364c"></a>
## metrics

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::metrics` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn metrics(&self) -> Option<MetricsSet>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:268`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cdc0e8238a12e4b591bf6e86"></a>
## name

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn name(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:260`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-13d16cb5d27eaa3c8ecdfa16"></a>
## new

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::new` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(inner: Arc<dyn ExecutionPlan>, span_create_fn: Arc<dyn Fn() -> tracing::Span + Send + Sync>, options: &InstrumentationOptions) -> InstrumentedExec
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [99, 1], "end": [250, 2], "filename": "src/instrumented_exec.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrumented_exec.rs:101`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Creates a new `InstrumentedExec` that wraps an execution plan with tracing and metrics.

<a id="op-073a8774c150351a6eda0991"></a>
## partition_statistics

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::partition_statistics` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn partition_statistics(&self, partition: Option<usize>) -> Result<Arc<Statistics>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:287`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e5af45bb5f79edb447acd538"></a>
## preview_fn

`struct_field` · `datafusion_tracing::instrumented_exec::InstrumentedExec::preview_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_fn: Option<std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>
```

Source: `src/instrumented_exec.rs:76`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1dc5e3675c398eef1e92583d"></a>
## preview_limit

`struct_field` · `datafusion_tracing::instrumented_exec::InstrumentedExec::preview_limit` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_limit: usize
```

Source: `src/instrumented_exec.rs:75`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-85da14d01d1f93e7642e8fd4"></a>
## properties

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::properties` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn properties(&self) -> &Arc<PlanProperties>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:259`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3678cc8ce2f7bac5b36f6de1"></a>
## record_metrics

`struct_field` · `datafusion_tracing::instrumented_exec::InstrumentedExec::record_metrics` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
record_metrics: bool
```

Source: `src/instrumented_exec.rs:73`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d22df865ec3e7bc6237a74eb"></a>
## recorders

`struct_field` · `datafusion_tracing::instrumented_exec::InstrumentedExec::recorders` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
recorders: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<ExecutionRecorders>>>>
```

Source: `src/instrumented_exec.rs:85`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Shared recorder groups for active executions of this plan node.

Groups are kept alive only until their streams have finished, so dropping
an already-consumed plan does not close spans synchronously. Concurrent
executions that belong to the same task context and touch distinct
partitions share a group; independent or duplicate executions get a fresh
group.

<a id="op-4a503ebf9eace144d7a0eef1"></a>
## repartitioned

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::repartitioned` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn repartitioned(&self, target_partitions: usize, config: &ConfigOptions) -> Result<Option<Arc<dyn ExecutionPlan>>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:304`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for repartitioning and rewrap with an InstrumentedExec.

<a id="op-d8877477b9e7385b444a4d81"></a>
## replace_children

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::replace_children` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn replace_children(Arc<self>, children: Vec<Arc<dyn ExecutionPlan>>, options: ReplaceChildrenOptions) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:399`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate child replacement to the inner plan and preserve instrumentation.

<a id="op-ec6d807f6ba9413a9c370fc3"></a>
## required_input_distribution

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::required_input_distribution` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn required_input_distribution(&self) -> Vec<Distribution>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:285`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f13eb4f3d21825fc5ff6befe"></a>
## required_input_ordering

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::required_input_ordering` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn required_input_ordering(&self) -> Vec<Option<OrderingRequirements>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:264`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-783d84b62cf928dc4c9392b6"></a>
## reset_state

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::reset_state` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn reset_state(Arc<self>) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:430`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for resetting state and rewrap with an InstrumentedExec.

<a id="op-9874f53a866e355b5be22d34"></a>
## schema

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::schema` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema(&self) -> SchemaRef
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:258`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cbcfcbac50524b91a91af7a4"></a>
## span_create_fn

`struct_field` · `datafusion_tracing::instrumented_exec::InstrumentedExec::span_create_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span_create_fn: std::sync::Arc<dyn Fn() -> tracing::Span + Send + Sync>
```

Source: `src/instrumented_exec.rs:88`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Function to create and initialize tracing spans.

<a id="op-8d8428ba1e04e7de678f9dda"></a>
## static_name

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::static_name` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn static_name() -> &'static str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:291`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a70975e0f2c024baffb7ed19"></a>
## statistics_from_inputs

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::statistics_from_inputs` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn statistics_from_inputs(&self, input_stats: &[Arc<Statistics>], args: &StatisticsArgs) -> Result<Arc<Statistics>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:269`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0ea1cbf78a135122d9caad4f"></a>
## supports_limit_pushdown

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::supports_limit_pushdown` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn supports_limit_pushdown(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:275`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b2568d9ab46ffb135c2321ab"></a>
## try_pushdown_sort

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::try_pushdown_sort` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn try_pushdown_sort(&self, order: &[PhysicalSortExpr]) -> Result<SortOrderPushdownResult<Arc<dyn ExecutionPlan>>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:380`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for sort pushdown and rewrap with an InstrumentedExec.

<a id="op-6e472f1d972da758c9926b05"></a>
## try_swapping_with_projection

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::try_swapping_with_projection` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn try_swapping_with_projection(&self, projection: &ProjectionExec) -> Result<Option<Arc<dyn ExecutionPlan>>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:341`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for swapping with a projection and rewrap with an InstrumentedExec.

<a id="op-2e6d14b730b41295de1e83e3"></a>
## with_fetch

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::with_fetch` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn with_fetch(&self, limit: Option<usize>) -> Option<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:321`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for fetching and rewrap with an InstrumentedExec.

<a id="op-b7462196df3fb1e8966f342a"></a>
## with_new_children

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::with_new_children` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn with_new_children(Arc<self>, children: Vec<Arc<dyn ExecutionPlan>>) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:408`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-da564bd6043084ab02f6fbab"></a>
## with_new_children_and_same_properties

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::with_new_children_and_same_properties` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn with_new_children_and_same_properties(Arc<self>, children: Vec<Arc<dyn ExecutionPlan>>) -> Result<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:419`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d9a2c550461c6b4ffa0b598"></a>
## with_new_state

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::with_new_state` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn with_new_state(&self, state: Arc<dyn Any + Send + Sync>) -> Option<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:436`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate to the inner plan for injecting run-time state and rewrap with an InstrumentedExec.

<a id="op-fa80cd5424f6e1ca3c11de81"></a>
## with_preserve_order

`function` · `datafusion_tracing::instrumented_exec::InstrumentedExec::with_preserve_order` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn with_preserve_order(&self, preserve_order: bool) -> Option<Arc<dyn ExecutionPlan>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::InstrumentedExec", "path": "InstrumentedExec"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [253, 1], "end": [497, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_physical_plan::execution_plan::ExecutionPlan", "path": "ExecutionPlan"}, "trait_path": "datafusion_physical_plan::execution_plan::ExecutionPlan"}`

Source: `src/instrumented_exec.rs:331`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Delegate order-preservation requirements to the inner plan and rewrap with
an InstrumentedExec.
