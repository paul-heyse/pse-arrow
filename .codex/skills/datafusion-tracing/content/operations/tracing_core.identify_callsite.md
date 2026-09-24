# `tracing_core::identify_callsite`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.identify_callsite.json).

<a id="op-40c89bb45082fd67c7ac9c25"></a>
## identify_callsite

`macro` · `tracing_core::identify_callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! identify_callsite
```

Source: `src/lib.rs:192`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Statically constructs an [`Identifier`] for the provided [`Callsite`].

This may be used in contexts such as static initializers.

For example:
```rust
use tracing_core::{callsite, identify_callsite};
# use tracing_core::{Metadata, subscriber::Interest};
# fn main() {
pub struct MyCallsite {
   // ...
}
impl callsite::Callsite for MyCallsite {
# fn set_interest(&self, _: Interest) { unimplemented!() }
# fn metadata(&self) -> &Metadata { unimplemented!() }
    // ...
}

static CALLSITE: MyCallsite = MyCallsite {
    // ...
};

static CALLSITE_ID: callsite::Identifier = identify_callsite!(&CALLSITE);
# }
```

[`Identifier`]: callsite::Identifier
[`Callsite`]: callsite::Callsite
