# `tracing::info`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.info.json).

<a id="op-cb9ffd84d965099ac69e7a54"></a>
## info

`macro` · `tracing::info` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! info
```

Source: `src/macros.rs:1896`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs an event at the info level.

This functions similarly to the [`event!`] macro. See [the top-level
documentation][lib] for details on the syntax accepted by
this macro.

[`event!`]: crate::event!
[lib]: crate#using-the-macros

# Examples

```rust
use tracing::info;
# // this is so the test will still work in no-std mode
# #[derive(Debug)]
# pub struct Ipv4Addr;
# impl Ipv4Addr { fn new(o1: u8, o2: u8, o3: u8, o4: u8) -> Self { Self } }
# fn main() {
# struct Connection { port: u32, speed: f32 }
use tracing::field;

let addr = Ipv4Addr::new(127, 0, 0, 1);
let conn = Connection { port: 40, speed: 3.20 };

info!(conn.port, "connected to {:?}", addr);
info!(
    target: "connection_events",
    ip = ?addr,
    conn.port,
    ?conn.speed,
);
info!(name: "completed", "completed connection to {:?}", addr);
# }
```
