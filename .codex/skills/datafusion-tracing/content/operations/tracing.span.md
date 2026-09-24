# `tracing::span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.span.json).

<a id="op-d37a648f25beedf59548b980"></a>
## span

`macro` · `tracing::span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! span
```

Source: `src/macros.rs:20`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new span.

See [the top-level documentation][lib] for details on the syntax accepted by
this macro.

[lib]: crate#using-the-macros

# Examples

Creating a new span:
```
# use tracing::{span, Level};
# fn main() {
let span = span!(Level::TRACE, "my span");
let _enter = span.enter();
// do work inside the span...
# }
```
