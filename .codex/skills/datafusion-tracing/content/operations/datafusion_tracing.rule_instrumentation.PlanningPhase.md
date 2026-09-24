# `datafusion_tracing::rule_instrumentation::PlanningPhase`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.PlanningPhase.json).

<a id="op-b64ec52bdfe3f2af5ae5f5ec"></a>
## PlanningPhase

`enum` · `datafusion_tracing::rule_instrumentation::PlanningPhase` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
enum PlanningPhase
```

Source: `src/rule_instrumentation.rs:92`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The phase of query planning currently active.

<a id="op-8969fe748a7b5ca1cdacc699"></a>
## Analyzer

`variant` · `datafusion_tracing::rule_instrumentation::PlanningPhase::Analyzer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Analyzer
```

Source: `src/rule_instrumentation.rs:93`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-427b5f9f106f4125048bf9ec"></a>
## Optimizer

`variant` · `datafusion_tracing::rule_instrumentation::PlanningPhase::Optimizer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Optimizer
```

Source: `src/rule_instrumentation.rs:94`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f184d39db96e5fe3a596fac"></a>
## PhysicalOptimizer

`variant` · `datafusion_tracing::rule_instrumentation::PlanningPhase::PhysicalOptimizer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
PhysicalOptimizer
```

Source: `src/rule_instrumentation.rs:95`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8cf4927bc54a2ec050c19e6f"></a>
## clone

`function` · `datafusion_tracing::rule_instrumentation::PlanningPhase::clone` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn clone(&self) -> PlanningPhase
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PlanningPhase", "path": "PlanningPhase"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 10], "end": [91, 15], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/rule_instrumentation.rs:91`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-272c4c19f2182d6aa14e2b0e"></a>
## eq

`function` · `datafusion_tracing::rule_instrumentation::PlanningPhase::eq` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn eq(&self, other: &PlanningPhase) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PlanningPhase", "path": "PlanningPhase"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 30], "end": [91, 39], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/rule_instrumentation.rs:91`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-04bc555d48e9aac97ccf921b"></a>
## fmt

`function` · `datafusion_tracing::rule_instrumentation::PlanningPhase::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_instrumentation::PlanningPhase", "path": "PlanningPhase"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 23], "end": [91, 28], "filename": "src/rule_instrumentation.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_instrumentation.rs:91`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
