# `tracing::trace_span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.trace_span.json).

<a id="op-6161ad54de3b03a8494209c8"></a>
## trace_span

`macro` · `tracing::trace_span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! trace_span
```

Source: `src/macros.rs:197`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a span at the trace level.

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
# use tracing::{trace_span, span, Level};
# fn main() {
trace_span!("my_span");
// is equivalent to:
span!(Level::TRACE, "my_span");
# }
```

```rust
# use tracing::{trace_span, span, Level};
# fn main() {
let span = trace_span!("my span");
span.in_scope(|| {
    // do work inside the span...
});
# }
```
