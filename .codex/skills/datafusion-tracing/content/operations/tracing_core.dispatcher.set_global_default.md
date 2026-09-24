# `tracing_core::dispatcher::set_global_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.set_global_default.json).

<a id="op-d78540acfa7df1f4405da536"></a>
## set_global_default

`function` · `tracing_core::dispatcher::set_global_default` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn set_global_default(dispatcher: Dispatch) -> Result<(), SetGlobalDefaultError>
```

Source: `src/dispatcher.rs:299`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Sets this dispatch as the global default for the duration of the entire program.
Will be used as a fallback if no thread-local dispatch has been set in a thread
(using `with_default`.)

Can only be set once; subsequent attempts to set the global default will fail.
Returns `Err` if the global default has already been set.

<div class="example-wrap" style="display:inline-block"><pre class="compile_fail" style="white-space:normal;font:inherit;">
    <strong>Warning</strong>: In general, libraries should <em>not</em> call
    <code>set_global_default()</code>! Doing so will cause conflicts when
    executables that depend on the library try to set the default later.
</pre></div>

[span]: super::span
[`Subscriber`]: super::subscriber::Subscriber
[`Event`]: super::event::Event
