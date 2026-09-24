# `tracing_subscriber::reload`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.reload.json).

<a id="op-8365694b394843a9594a3071"></a>
## reload

`module` · `tracing_subscriber::reload` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
mod reload
```

Source: `src/reload.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wrapper for a `Layer` to allow it to be dynamically reloaded.

This module provides a [`Layer` type] implementing the [`Layer` trait] or [`Filter` trait]
which wraps another type implementing the corresponding trait. This
allows the wrapped type to be replaced with another
instance of that type at runtime.

This can be used in cases where a subset of `Layer` or `Filter` functionality
should be dynamically reconfigured, such as when filtering directives may
change at runtime. Note that this layer introduces a (relatively small)
amount of overhead, and should thus only be used as needed.

# Examples

Reloading a [global filtering](crate::layer#global-filtering) layer:

```rust
# use tracing::info;
use tracing_subscriber::{filter, fmt, reload, prelude::*};
let filter = filter::LevelFilter::WARN;
let (filter, reload_handle) = reload::Layer::new(filter);
tracing_subscriber::registry()
  .with(filter)
  .with(fmt::Layer::default())
  .init();
#
# // specifying the Registry type is required
# let _: &reload::Handle<filter::LevelFilter, tracing_subscriber::Registry> = &reload_handle;
#
info!("This will be ignored");
reload_handle.modify(|filter| *filter = filter::LevelFilter::INFO);
info!("This will be logged");
```

Reloading a [`Filtered`](crate::filter::Filtered) layer:

```rust
# use tracing::info;
use tracing_subscriber::{filter, fmt, reload, prelude::*};
let filtered_layer = fmt::Layer::default().with_filter(filter::LevelFilter::WARN);
let (filtered_layer, reload_handle) = reload::Layer::new(filtered_layer);
#
# // specifying the Registry type is required
# let _: &reload::Handle<filter::Filtered<fmt::Layer<tracing_subscriber::Registry>,
# filter::LevelFilter, tracing_subscriber::Registry>,tracing_subscriber::Registry>
# = &reload_handle;
#
tracing_subscriber::registry()
  .with(filtered_layer)
  .init();
info!("This will be ignored");
reload_handle.modify(|layer| *layer.filter_mut() = filter::LevelFilter::INFO);
info!("This will be logged");
```

## Note

The [`Layer`](../operations/tracing_subscriber.reload.Layer.md#op-9060fccc0ad13ec0bdc27235) implementation is unable to implement downcasting functionality,
so certain [`Layer`](../operations/tracing_subscriber.reload.Layer.md#op-9060fccc0ad13ec0bdc27235) will fail to downcast if wrapped in a `reload::Layer`.

If you only want to be able to dynamically change the
`Filter` on a layer, prefer wrapping that `Filter` in the `reload::Layer`.

[`Filter` trait]: crate::layer::Filter
[`Layer` type]: Layer
[`Layer` trait]: super::layer::Layer
