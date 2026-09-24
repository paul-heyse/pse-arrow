# `tracing::debug_span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.debug_span.json).

<a id="op-9874b343e33af2d4bf485dc4"></a>
## debug_span

`macro` · `tracing::debug_span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! debug_span
```

Source: `src/macros.rs:278`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a span at the debug level.

[Fields] and [attributes] are set using the same syntax as the [`span!`]
macro.

See [the top-level documentation][lib] for details on the syntax accepted by
this macro.

[lib]: crate#using-the-macros
[attributes]: crate#configuring-attributes
[Fields]: crate#recording-fields
[`span!`]: crate::span!

# Examples

```rust
# use tracing::{debug_span, span, Level};
# fn main() {
debug_span!("my_span");
// is equivalent to:
span!(Level::DEBUG, "my_span");
# }
```

```rust
# use tracing::debug_span;
# fn main() {
let span = debug_span!("my span");
span.in_scope(|| {
    // do work inside the span...
});
# }
```
