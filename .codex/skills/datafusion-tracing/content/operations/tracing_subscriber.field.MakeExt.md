# `tracing_subscriber::field::MakeExt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.MakeExt.json).

<a id="op-0f6ada0fe415846eb0ca71fa"></a>
## MakeExt

`trait` · `tracing_subscriber::field::MakeExt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait MakeExt<T> where Self: MakeVisitor<T> + Sized + sealed::Sealed<MakeExtMarker<T>>
```

Source: `src/field/mod.rs:130`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait providing `MakeVisitor` combinators.

<a id="op-e7598f72ca112adefe8aa9ca"></a>
## debug_alt

`function` · `tracing_subscriber::field::MakeExt::debug_alt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn debug_alt(self) -> debug::Alt<Self>
```

Source: `src/field/mod.rs:137`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` so that any `fmt::Debug` fields are recorded using the
alternate formatter (`{:#?}`).

<a id="op-e8f75a95bd9d36a35076a553"></a>
## delimited

`function` · `tracing_subscriber::field::MakeExt::delimited` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn delimited<D>(self, delimiter: D) -> delimited::Delimited<D, Self> where D: AsRef<str> + Clone, Self::Visitor: VisitFmt
```

Source: `src/field/mod.rs:149`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` so that when fields are formatted to a writer, they are
separated by the provided `delimiter`.

<a id="op-4424b536241d52a0449bf4bc"></a>
## display_messages

`function` · `tracing_subscriber::field::MakeExt::display_messages` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn display_messages(self) -> display::Messages<Self>
```

Source: `src/field/mod.rs:143`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` so that any string fields named "message" are recorded
using `fmt::Display`.
