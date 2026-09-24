# `tracing_core::dispatcher::set_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.set_default.json).

<a id="op-7428fec10cb7b2f98ae92ea4"></a>
## set_default

`function` · `tracing_core::dispatcher::set_default` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn set_default(dispatcher: &Dispatch) -> DefaultGuard
```

Source: `src/dispatcher.rs:276`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Sets the dispatch as the default dispatch for the duration of the lifetime
of the returned DefaultGuard

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This function required the Rust standard library.
    <code>no_std</code> users should use <a href="fn.set_global_default.html">
    <code>set_global_default</code></a> instead.
</pre>

[`set_global_default`]: set_global_default
