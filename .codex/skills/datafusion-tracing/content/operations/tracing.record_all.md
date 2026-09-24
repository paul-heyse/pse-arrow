# `tracing::record_all`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.record_all.json).

<a id="op-2ae0479bcba50b67c636b74f"></a>
## record_all

`macro` · `tracing::record_all` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! record_all
```

Source: `src/macros.rs:152`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Records multiple values on a span in a single call. As with recording
individual values, all fields must be declared when the span is created.

This macro supports two optional sigils:
- `%` uses the Display implementation.
- `?` uses the Debug implementation.

For more details, see the [top-level documentation][lib].

[lib]: tracing/#recording-fields

# Examples

```
# use tracing::{field, info_span, record_all};
let span = info_span!("my span", field1 = field::Empty, field2 = field::Empty, field3 = field::Empty).entered();
record_all!(span, field1 = ?"1", field2 = %"2", field3 = 3);
```
