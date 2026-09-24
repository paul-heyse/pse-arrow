# `tracing_core::callsite::rebuild_interest_cache`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.callsite.rebuild_interest_cache.json).

<a id="op-514604dfb2bc1df5b1a3e933"></a>
## rebuild_interest_cache

`function` · `tracing_core::callsite::rebuild_interest_cache` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn rebuild_interest_cache()
```

Source: `src/callsite.rs:222`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Clear and reregister interest on every [`Callsite`]

This function is intended for runtime reconfiguration of filters on traces
when the filter recalculation is much less frequent than trace events are.
The alternative is to have the [`Subscriber`] that supports runtime
reconfiguration of filters always return [`Interest::sometimes()`] so that
[`enabled`] is evaluated for every event.

This function will also re-compute the global maximum level as determined by
the [`max_level_hint`] method. If a [`Subscriber`]
implementation changes the value returned by its `max_level_hint`
implementation at runtime, then it **must** call this function after that
value changes, in order for the change to be reflected.

See the [documentation on callsite interest caching][cache-docs] for
additional information on this function's usage.

[`max_level_hint`]: super::subscriber::Subscriber::max_level_hint
[`Callsite`]: super::callsite::Callsite
[`enabled`]: super::subscriber::Subscriber#tymethod.enabled
[`Interest::sometimes()`]: super::subscriber::Interest::sometimes
[`Subscriber`]: super::subscriber::Subscriber
[cache-docs]: crate::callsite#rebuilding-cached-interest
