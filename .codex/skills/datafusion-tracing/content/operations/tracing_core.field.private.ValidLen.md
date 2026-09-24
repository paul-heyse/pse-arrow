# `tracing_core::field::private::ValidLen`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.private.ValidLen.json).

<a id="op-75420871d28ec024b83f2403"></a>
## ValidLen

`trait` · `tracing_core::field::private::ValidLen` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
trait ValidLen<'a>: Borrow<[(&'a Field, Option<&'a dyn Value + 'a>)]>
```

Source: `src/field.rs:1162`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Restrictions on `ValueSet` lengths were removed in #2508 but this type remains for backwards compatibility.
