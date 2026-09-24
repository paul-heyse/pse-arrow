# `tracing::error_span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.error_span.json).

<a id="op-404b74567e9b4a33eee57bc9"></a>
## error_span

`macro` · `tracing::error_span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! error_span
```

Source: `src/macros.rs:520`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a span at the error level.

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
# use tracing::{span, error_span, Level};
# fn main() {
error_span!("my_span");
// is equivalent to:
span!(Level::ERROR, "my_span");
# }
```

```rust
# use tracing::error_span;
# fn main() {
let span = error_span!("my span");
span.in_scope(|| {
    // do work inside the span...
});
# }
```
