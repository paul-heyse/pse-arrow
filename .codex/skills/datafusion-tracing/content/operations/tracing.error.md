# `tracing::error`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.error.json).

<a id="op-eaaa4b15361b99f578bdb9ad"></a>
## error

`macro` · `tracing::error` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! error
```

Source: `src/macros.rs:2452`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs an event at the error level.

This functions similarly to the [`event!`] macro. See [the top-level
documentation][lib] for details on the syntax accepted by
this macro.

[`event!`]: crate::event!
[lib]: crate#using-the-macros

# Examples

```rust
use tracing::error;
# fn main() {

let (err_info, port) = ("No connection", 22);

error!(port, error = %err_info);
error!(target: "app_events", "App Error: {}", err_info);
error!({ info = err_info }, "error on port: {}", port);
error!(name: "invalid_input", "Invalid input: {}", err_info);
# }
```
