# `datafusion_tracing::rule_options::RuleInstrumentationOptions`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_options.RuleInstrumentationOptions.json).

<a id="op-a1e41f5dc1584eed32cb9de8"></a>
## RuleInstrumentationOptions

`struct` · `datafusion_tracing::rule_options::RuleInstrumentationOptions` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
struct RuleInstrumentationOptions
```

Source: `src/rule_options.rs:52`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Configuration options for instrumented DataFusion rules (Analyzer, Optimizer, Physical Optimizer).

<a id="op-daaf23f760defdebcc1d8b4b"></a>
## builder

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::builder` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> RuleInstrumentationOptionsBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:79`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Creates a new builder for `RuleInstrumentationOptions`.

<a id="op-96198aa6af8d941595e9683c"></a>
## clone

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::clone` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> RuleInstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 10], "end": [51, 15], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/rule_options.rs:51`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f27cda21ac2394829d8eae59"></a>
## default

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::default` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 1], "end": [75, 2], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/rule_options.rs:67`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bf567ff563f892ec3ea7ed30"></a>
## fmt

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::fmt` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 17], "end": [51, 22], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_options.rs:51`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-392fb4512b1910709e771fbb"></a>
## full

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::full` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn full() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:95`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Creates options with all phases enabled at `Full` level.

This is a convenience constructor for the common case of enabling
full instrumentation for all rule phases.

# Example

```
use datafusion_tracing::RuleInstrumentationOptions;

let options = RuleInstrumentationOptions::full();
```

<a id="op-3981e7ea144a7d3992d7bda7"></a>
## phase_only

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::phase_only` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn phase_only() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:108`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Creates options with all phases enabled at `PhaseOnly` level.

This creates phase-level spans without individual rule spans,
useful for reducing trace verbosity while still tracking phase timing.

<a id="op-6a915984f44989988626a359"></a>
## with_plan_diff

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::with_plan_diff` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_plan_diff(self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:118`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Returns a new options with plan diff enabled.

<a id="op-a12428b574074fc1e6db1e0d"></a>
## RuleInstrumentationOptions

`struct` · `datafusion_tracing::rule_options::RuleInstrumentationOptions` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
struct RuleInstrumentationOptions
```

Source: `src/rule_options.rs:52`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Configuration options for instrumented DataFusion rules (Analyzer, Optimizer, Physical Optimizer).

<a id="op-25cd9b167135f31f615e08cb"></a>
## analyzer

`struct_field` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::analyzer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
analyzer: InstrumentationLevel
```

Source: `src/rule_options.rs:57`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instrumentation level for analyzer rules.

<a id="op-056b759ba7ea07dfb303eeb0"></a>
## builder

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::builder` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn builder() -> RuleInstrumentationOptionsBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:79`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Creates a new builder for `RuleInstrumentationOptions`.

<a id="op-841a602b1a3518a6b0b85a03"></a>
## clone

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::clone` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn clone(&self) -> RuleInstrumentationOptions
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 10], "end": [51, 15], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/rule_options.rs:51`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-447e6da1a6043bcdb81dccd9"></a>
## default

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::default` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 1], "end": [75, 2], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/rule_options.rs:67`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-19b65d9b2b41c010a09e3176"></a>
## fmt

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::fmt` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 17], "end": [51, 22], "filename": "src/rule_options.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/rule_options.rs:51`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fef31c77d56a696919037276"></a>
## full

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::full` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn full() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:95`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Creates options with all phases enabled at `Full` level.

This is a convenience constructor for the common case of enabling
full instrumentation for all rule phases.

# Example

```
use datafusion_tracing::RuleInstrumentationOptions;

let options = RuleInstrumentationOptions::full();
```

<a id="op-3991e3185f171990f08664fe"></a>
## optimizer

`struct_field` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::optimizer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
optimizer: InstrumentationLevel
```

Source: `src/rule_options.rs:60`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instrumentation level for logical optimizer rules.

<a id="op-fdc9a5f002b41627af02d1ba"></a>
## phase_only

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::phase_only` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn phase_only() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:108`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Creates options with all phases enabled at `PhaseOnly` level.

This creates phase-level spans without individual rule spans,
useful for reducing trace verbosity while still tracking phase timing.

<a id="op-42b653962da7caf8a38988c6"></a>
## physical_optimizer

`struct_field` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::physical_optimizer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
physical_optimizer: InstrumentationLevel
```

Source: `src/rule_options.rs:63`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instrumentation level for physical optimizer rules.

<a id="op-bc9f3aa8893419c9a0bfee1a"></a>
## plan_diff

`struct_field` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::plan_diff` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
plan_diff: bool
```

Source: `src/rule_options.rs:54`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Whether to include a unified diff of plan changes in the span.

<a id="op-a4ac22da6984eb7127b79c0e"></a>
## with_plan_diff

`function` · `datafusion_tracing::rule_options::RuleInstrumentationOptions::with_plan_diff` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn with_plan_diff(self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::rule_options::RuleInstrumentationOptions", "path": "RuleInstrumentationOptions"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [122, 2], "filename": "src/rule_options.rs"}, "trait": null, "trait_path": null}`

Source: `src/rule_options.rs:118`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Returns a new options with plan diff enabled.
