# `tracing::trace`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.trace.json).

<a id="op-e0223535da58be81e2d20e16"></a>
## trace

`macro` · `tracing::trace` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! trace
```

Source: `src/macros.rs:1333`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs an event at the trace level.

This functions similarly to the [`event!`] macro. See [the top-level
documentation][lib] for details on the syntax accepted by
this macro.

[`event!`]: crate::event!
[lib]: crate#using-the-macros

# Examples

```rust
use tracing::trace;
# #[derive(Debug, Copy, Clone)] struct Position { x: f32, y: f32 }
# impl Position {
# const ORIGIN: Self = Self { x: 0.0, y: 0.0 };
# fn dist(&self, other: Position) -> f32 {
#    let x = (other.x - self.x).exp2(); let y = (self.y - other.y).exp2();
#    (x + y).sqrt()
# }
# }
# fn main() {
let pos = Position { x: 3.234, y: -1.223 };
let origin_dist = pos.dist(Position::ORIGIN);

trace!(position = ?pos, ?origin_dist);
trace!(
    target: "app_events",
    position = ?pos,
    "x is {} and y is {}",
    if pos.x >= 0.0 { "positive" } else { "negative" },
    if pos.y >= 0.0 { "positive" } else { "negative" }
);
trace!(name: "completed", position = ?pos);
# }
```
