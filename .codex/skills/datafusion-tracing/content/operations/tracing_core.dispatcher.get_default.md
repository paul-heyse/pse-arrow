# `tracing_core::dispatcher::get_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.get_default.json).

<a id="op-282b7f5317299729004dc23f"></a>
## get_default

`function` · `tracing_core::dispatcher::get_default` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn get_default<T, F>(f: F) -> T where F: FnMut(&Dispatch) -> T
```

Source: `src/dispatcher.rs:379`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Executes a closure with a reference to this thread's current [dispatcher].

Note that calls to `get_default` should not be nested; if this function is
called while inside of another `get_default`, that closure will be provided
with `Dispatch::none` rather than the previously set dispatcher.

[dispatcher]: super::dispatcher::Dispatch
