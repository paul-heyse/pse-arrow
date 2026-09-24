# `datafusion_tracing::utils::DefaultDisplay`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.utils.DefaultDisplay.json).

<a id="op-2e61452fb630bd0cf04cc709"></a>
## DefaultDisplay

`struct` · `datafusion_tracing::utils::DefaultDisplay` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct DefaultDisplay<'a>
```

Source: `src/utils.rs:24`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Helper struct for default display formatting of an `ExecutionPlan`.

<a id="op-15cbadeffaa41c279b5129d3"></a>
## 0

`struct_field` · `datafusion_tracing::utils::DefaultDisplay::0` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
0: &'a dyn ExecutionPlan
```

Source: `src/utils.rs:24`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-de6e1b8c7b566ecb434025eb"></a>
## fmt

`function` · `datafusion_tracing::utils::DefaultDisplay::fmt` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "datafusion_tracing::utils::DefaultDisplay", "path": "DefaultDisplay"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [30, 2], "filename": "src/utils.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/utils.rs:27`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
