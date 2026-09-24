# `tracing::debug`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.debug.json).

<a id="op-7832ab362c624ee66c55c058"></a>
## debug

`macro` · `tracing::debug` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! debug
```

Source: `src/macros.rs:1609`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs an event at the debug level.

This functions similarly to the [`event!`] macro. See [the top-level
documentation][lib] for details on the syntax accepted by
this macro.

[`event!`]: crate::event!
[lib]: crate#using-the-macros

# Examples

```rust
use tracing::debug;
# fn main() {
# #[derive(Debug)] struct Position { x: f32, y: f32 }

let pos = Position { x: 3.234, y: -1.223 };

debug!(?pos.x, ?pos.y);
debug!(target: "app_events", position = ?pos, "New position");
debug!(name: "completed", position = ?pos);
# }
```
