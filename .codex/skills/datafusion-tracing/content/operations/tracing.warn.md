# `tracing::warn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.warn.json).

<a id="op-b6e1e7a39688ceb7dd09c431"></a>
## warn

`macro` · `tracing::warn` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! warn
```

Source: `src/macros.rs:2176`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs an event at the warn level.

This functions similarly to the [`event!`] macro. See [the top-level
documentation][lib] for details on the syntax accepted by
this macro.

[`event!`]: crate::event!
[lib]: crate#using-the-macros

# Examples

```rust
use tracing::warn;
# fn main() {

let warn_description = "Invalid Input";
let input = &[0x27, 0x45];

warn!(?input, warning = warn_description);
warn!(
    target: "input_events",
    warning = warn_description,
    "Received warning for input: {:?}", input,
);
warn!(name: "invalid", ?input);
# }
```
