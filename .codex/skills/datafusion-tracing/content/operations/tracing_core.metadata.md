# `tracing_core::metadata`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.metadata.json).

<a id="op-7a501fed5f3300a565f6eecc"></a>
## metadata

`macro` · `tracing_core::metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! metadata
```

Source: `src/lib.rs:230`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Statically constructs new span [metadata].

/// For example:
```rust
# use tracing_core::{callsite::Callsite, subscriber::Interest};
use tracing_core::metadata;
use tracing_core::metadata::{Kind, Level, Metadata};
# fn main() {
# pub struct MyCallsite { }
# impl Callsite for MyCallsite {
# fn set_interest(&self, _: Interest) { unimplemented!() }
# fn metadata(&self) -> &Metadata { unimplemented!() }
# }
#
static FOO_CALLSITE: MyCallsite = MyCallsite {
    // ...
};

static FOO_METADATA: Metadata = metadata!{
    name: "foo",
    target: module_path!(),
    level: Level::DEBUG,
    fields: &["bar", "baz"],
    callsite: &FOO_CALLSITE,
    kind: Kind::SPAN,
};
# }
```

[metadata]: metadata::Metadata
[`Metadata::new`]: metadata::Metadata::new
