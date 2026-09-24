# `tracing::event`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.event.json).

<a id="op-c45070aa88b3630e47ea433f"></a>
## event

`macro` · `tracing::event` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! event
```

Source: `src/macros.rs:615`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new `Event`.

The event macro is invoked with a `Level` and up to 32 key-value fields.
Optionally, a format string and arguments may follow the fields; this will
be used to construct an implicit field named "message".

See [the top-level documentation][lib] for details on the syntax accepted by
this macro.

[lib]: crate#using-the-macros

# Examples

```rust
use tracing::{event, Level};

# fn main() {
let data = (42, "forty-two");
let private_data = "private";
let error = "a bad error";

event!(Level::ERROR, %error, "Received error");
event!(
    target: "app_events",
    Level::WARN,
    private_data,
    ?data,
    "App warning: {}",
    error
);
event!(name: "answer", Level::INFO, the_answer = data.0);
event!(Level::INFO, the_answer = data.0);
# }
```

