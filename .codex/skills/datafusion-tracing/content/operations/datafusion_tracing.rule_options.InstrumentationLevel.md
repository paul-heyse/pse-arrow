# `datafusion_tracing::rule_options::InstrumentationLevel`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_options.InstrumentationLevel.json).

<a id="op-e59122860da8806010c9c3d5"></a>
## InstrumentationLevel

`enum` · `datafusion_tracing::rule_options::InstrumentationLevel` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
enum InstrumentationLevel
```

Source: `src/rule_options.rs:27`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instrumentation level for a phase (analyzer, optimizer, or physical optimizer).

Rule spans always require a parent phase span, so the levels are hierarchical:
- `Disabled`: no spans (default)
- `PhaseOnly`: only the phase span
- `Full`: phase span + individual rule spans

<a id="op-6448517d6c87970f7a837f4f"></a>
## Disabled

`variant` · `datafusion_tracing::rule_options::InstrumentationLevel::Disabled` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Disabled
```

Source: `src/rule_options.rs:30`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No instrumentation for this phase (default).

<a id="op-d588a3aa7e30fd8a3f4f32d7"></a>
## Full

`variant` · `datafusion_tracing::rule_options::InstrumentationLevel::Full` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Full
```

Source: `src/rule_options.rs:35`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Full instrumentation: phase span + individual rule spans.

<a id="op-8ff34e7817222882590dce7b"></a>
## PhaseOnly

`variant` · `datafusion_tracing::rule_options::InstrumentationLevel::PhaseOnly` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
PhaseOnly
```

Source: `src/rule_options.rs:33`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Only phase-level span (e.g., "optimize_logical_plan"), no individual rule spans.
Useful for reducing trace verbosity while still tracking phase timing.

<a id="op-f22e7cadc8c4c09080c6c769"></a>
## clone

`function` · `datafusion_tracing::rule_options::InstrumentationLevel::clone` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn clone(&self) -> InstrumentationLevel
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::InstrumentationLevel", "path": "InstrumentationLevel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 10], "end": [26, 15], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/rule_options.rs:26`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-333dd62b95b96689af1286c1"></a>
## default

`function` · `datafusion_tracing::rule_options::InstrumentationLevel::default` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn default() -> InstrumentationLevel
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::InstrumentationLevel", "path": "InstrumentationLevel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 30], "end": [26, 37], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/rule_options.rs:26`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-66c2b9095313d13343c9eccc"></a>
## eq

`function` · `datafusion_tracing::rule_options::InstrumentationLevel::eq` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn eq(&self, other: &InstrumentationLevel) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::InstrumentationLevel", "path": "InstrumentationLevel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 39], "end": [26, 48], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/rule_options.rs:26`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ade96bd24163cb9af1effefc"></a>
## fmt

`function` · `datafusion_tracing::rule_options::InstrumentationLevel::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::InstrumentationLevel", "path": "InstrumentationLevel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 23], "end": [26, 28], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_options.rs:26`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ae50e24370b1c77b9607dcab"></a>
## phase_span_enabled

`function` · `datafusion_tracing::rule_options::InstrumentationLevel::phase_span_enabled` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn phase_span_enabled(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::InstrumentationLevel", "path": "InstrumentationLevel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [38, 1], "end": [48, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:40`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Returns true if the phase span should be created.

<a id="op-7a8cd576b80b8f2460d3c24d"></a>
## rule_spans_enabled

`function` · `datafusion_tracing::rule_options::InstrumentationLevel::rule_spans_enabled` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn rule_spans_enabled(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::InstrumentationLevel", "path": "InstrumentationLevel"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [38, 1], "end": [48, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:45`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Returns true if individual rule spans should be created.
