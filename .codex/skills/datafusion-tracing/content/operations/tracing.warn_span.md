# `tracing::warn_span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.warn_span.json).

<a id="op-d5dcc98e2cca91c96a6955f4"></a>
## warn_span

`macro` · `tracing::warn_span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! warn_span
```

Source: `src/macros.rs:440`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a span at the warn level.

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
# use tracing::{warn_span, span, Level};
# fn main() {
warn_span!("my_span");
// is equivalent to:
span!(Level::WARN, "my_span");
# }
```

```rust
use tracing::warn_span;
# fn main() {
let span = warn_span!("my span");
span.in_scope(|| {
    // do work inside the span...
});
# }
```
