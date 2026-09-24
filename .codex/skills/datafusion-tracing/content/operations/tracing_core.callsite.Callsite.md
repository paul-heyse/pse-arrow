# `tracing_core::callsite::Callsite`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.callsite.Callsite.json).

<a id="op-f2f04985f5653f1f8f822082"></a>
## Callsite

`trait` · `tracing_core::callsite::Callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
trait Callsite: Sync
```

Source: `src/callsite.rs:125`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Trait implemented by callsites.

These functions are only intended to be called by the callsite registry, which
correctly handles determining the common interest between all subscribers.

See the [module-level documentation](crate::callsite) for details on
callsites.

<a id="op-8e62be18bc1c6b7d9145844b"></a>
## metadata

`function` · `tracing_core::callsite::Callsite::metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &Metadata<'_>
```

Source: `src/callsite.rs:147`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the [metadata] associated with the callsite.

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">

**Note:** Implementations of this method should not produce [`Metadata`](../operations/tracing_core.metadata.Metadata.md#op-3c5a7a9d81c273e2173bb24c)
that share the same callsite [`Identifier`](../operations/tracing_core.callsite.Identifier.md#op-cef1410a3f296584b90d483e) but otherwise differ in any
way (e.g., have different `name`s).

</pre></div>

[metadata]: super::metadata::Metadata

<a id="op-d17b5941cb93ae648e37f5dc"></a>
## set_interest

`function` · `tracing_core::callsite::Callsite::set_interest` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn set_interest(&self, interest: Interest)
```

Source: `src/callsite.rs:133`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Sets the [`Interest`] for this callsite.

See the [documentation on callsite interest caching][cache-docs] for
details.

[`Interest`]: super::subscriber::Interest
[cache-docs]: crate::callsite#performing-static-filtering
