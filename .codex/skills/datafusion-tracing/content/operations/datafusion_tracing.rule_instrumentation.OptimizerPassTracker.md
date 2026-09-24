# `datafusion_tracing::rule_instrumentation::OptimizerPassTracker`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.OptimizerPassTracker.json).

<a id="op-b1ea7133a7b88ac9021f7aca"></a>
## OptimizerPassTracker

`struct` · `datafusion_tracing::rule_instrumentation::OptimizerPassTracker` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct OptimizerPassTracker
```

Source: `src/rule_instrumentation.rs:117`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Tracks the optimizer pass count, resetting when a new query starts.
A new query is detected by a change in the parent span ID or when
the physical optimizer phase completes.

<a id="op-74912e8dea1bad87bfbccfaf"></a>
## parent_span_id

`struct_field` · `datafusion_tracing::rule_instrumentation::OptimizerPassTracker::parent_span_id` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
parent_span_id: Option<tracing::span::Id>
```

Source: `src/rule_instrumentation.rs:118`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-36f881dc12ee2bf3351d9fdf"></a>
## pass_count

`struct_field` · `datafusion_tracing::rule_instrumentation::OptimizerPassTracker::pass_count` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
pass_count: usize
```

Source: `src/rule_instrumentation.rs:119`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
