# `tracing::level_filters::STATIC_MAX_LEVEL`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.level_filters.STATIC_MAX_LEVEL.json).

<a id="op-c693e208df2ae5728297fcde"></a>
## STATIC_MAX_LEVEL

`constant` · `tracing::level_filters::STATIC_MAX_LEVEL` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
const STATIC_MAX_LEVEL: LevelFilter = _
```

Source: `src/level_filters.rs:66`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

The statically configured maximum trace level.

See the [module-level documentation] for information on how to configure
this.

This value is checked by the `event!` and `span!` macros. Code that
manually constructs events or spans via the `Event::record` function or
`Span` constructors should compare the level against this value to
determine if those spans or events are enabled.

[module-level documentation]: self#compile-time-filters
