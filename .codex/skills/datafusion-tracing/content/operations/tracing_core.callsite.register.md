# `tracing_core::callsite::register`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.callsite.register.json).

<a id="op-e372a6d4be6dbbb70cf0c6ee"></a>
## register

`function` · `tracing_core::callsite::register` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn register(callsite: &'static dyn Callsite)
```

Source: `src/callsite.rs:236`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Register a new [`Callsite`] with the global registry.

This should be called once per callsite after the callsite has been
constructed.

See the [documentation on callsite registration][reg-docs] for details
on the global callsite registry.

[`Callsite`]: crate::callsite::Callsite
[reg-docs]: crate::callsite#registering-callsites
