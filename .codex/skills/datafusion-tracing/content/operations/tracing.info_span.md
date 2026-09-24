# `tracing::info_span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.info_span.json).

<a id="op-45ca51f4e9186b0d021bbe68"></a>
## info_span

`macro` · `tracing::info_span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! info_span
```

Source: `src/macros.rs:359`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a span at the info level.

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
# use tracing::{span, info_span, Level};
# fn main() {
info_span!("my_span");
// is equivalent to:
span!(Level::INFO, "my_span");
# }
```

```rust
# use tracing::info_span;
# fn main() {
let span = info_span!("my span");
span.in_scope(|| {
    // do work inside the span...
});
# }
```
