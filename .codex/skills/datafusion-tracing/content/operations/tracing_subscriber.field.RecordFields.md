# `tracing_subscriber::field::RecordFields`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.RecordFields.json).

<a id="op-fc1ac43117ab810b7a7cfedb"></a>
## RecordFields

`trait` · `tracing_subscriber::field::RecordFields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait RecordFields: sealed::Sealed<RecordFieldsMarker>
```

Source: `src/field/mod.rs:88`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait implemented by types which can be recorded by a [visitor].

This allows writing code that is generic over `tracing_core`'s
[`span::Attributes`][attr], [`span::Record`][rec], and [`Event`](../operations/tracing_core.event.Event.md#op-7ee85389e31294d1a098f039)
types. These types all provide inherent `record` methods that allow a
visitor to record their fields, but there is no common trait representing this.

With `RecordFields`, we can write code like this:
```
use tracing_core::field::Visit;
# use tracing_core::field::Field;
use tracing_subscriber::field::RecordFields;

struct MyVisitor {
    // ...
}
# impl MyVisitor { fn new() -> Self { Self{} } }
impl Visit for MyVisitor {
    // ...
# fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
}

fn record_with_my_visitor<R>(r: R)
where
    R: RecordFields,
{
    let mut visitor = MyVisitor::new();
    r.record(&mut visitor);
}
```
[visitor]: tracing_core::field::Visit
[attr]: tracing_core::span::Attributes
[rec]: tracing_core::span::Record

<a id="op-68f3eb12dd36dee3ec4fa696"></a>
## record

`function` · `tracing_subscriber::field::RecordFields::record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, visitor: &mut dyn Visit)
```

Source: `src/field/mod.rs:90`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Record all the fields in `self` with the provided `visitor`.
