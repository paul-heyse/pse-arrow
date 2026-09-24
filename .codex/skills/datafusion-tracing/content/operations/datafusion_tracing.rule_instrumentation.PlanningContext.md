# `datafusion_tracing::rule_instrumentation::PlanningContext`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.PlanningContext.json).

<a id="op-b43a04c2e8950ebe10c9290d"></a>
## PlanningContext

`struct` · `datafusion_tracing::rule_instrumentation::PlanningContext` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct PlanningContext
```

Source: `src/rule_instrumentation.rs:102`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Context for tracking the active planning phase span.

Stores the current phase, the entered span guard, and state needed for
computing plan diffs and tracking which rules modified the plan.

<a id="op-a29d0211b064905186f476db"></a>
## _entered

`struct_field` · `datafusion_tracing::rule_instrumentation::PlanningContext::_entered` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
_entered: tracing::span::EnteredSpan
```

Source: `src/rule_instrumentation.rs:106`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The entered span guard - keeps the span open until this context is dropped.

<a id="op-9e88e94c7e5dec656a641ce0"></a>
## effective_rules

`struct_field` · `datafusion_tracing::rule_instrumentation::PlanningContext::effective_rules` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
effective_rules: Vec<String>
```

Source: `src/rule_instrumentation.rs:111`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

List of rule names that modified the plan during this phase.

<a id="op-77f6ed10baa4c32787fa9aee"></a>
## phase

`struct_field` · `datafusion_tracing::rule_instrumentation::PlanningContext::phase` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
phase: PlanningPhase
```

Source: `src/rule_instrumentation.rs:104`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The current planning phase.

<a id="op-fc6d2cbaed03300db2789bf2"></a>
## plan_before

`struct_field` · `datafusion_tracing::rule_instrumentation::PlanningContext::plan_before` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
plan_before: Option<String>
```

Source: `src/rule_instrumentation.rs:109`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The plan state at the start of the phase (for computing diff).
Only captured if plan_diff option is enabled and span is active.
