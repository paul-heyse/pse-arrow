# `tracing_subscriber::filter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.json).

<a id="op-15b346151e4aca3cb883c21c"></a>
## filter

`module` · `tracing_subscriber::filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
mod filter
```

Source: `src/filter/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

[`Layer`]s that control which spans and events are enabled by the wrapped
subscriber.

This module contains a number of types that provide implementations of
various strategies for filtering which spans and events are enabled. For
details on filtering spans and events using [`Layer`]s, see the
[`layer` module's documentation].

[`layer` module's documentation]: crate::layer#filtering-with-layers
[`Layer`]: crate::layer
