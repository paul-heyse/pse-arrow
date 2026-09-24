# `tracing_core::dispatcher::with_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.with_default.json).

<a id="op-3bd4d37aa9725572a1a5bae1"></a>
## with_default

`function` · `tracing_core::dispatcher::with_default` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn with_default<T>(dispatcher: &Dispatch, f: impl FnOnce() -> T) -> T
```

Source: `src/dispatcher.rs:254`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Sets this dispatch as the default for the duration of a closure.

The default dispatcher is used when creating a new [span] or
[`Event`].

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This function required the Rust standard library.
    <code>no_std</code> users should use <a href="fn.set_global_default.html">
    <code>set_global_default</code></a> instead.
</pre>

[span]: super::span
[`Subscriber`]: super::subscriber::Subscriber
[`Event`]: super::event::Event
