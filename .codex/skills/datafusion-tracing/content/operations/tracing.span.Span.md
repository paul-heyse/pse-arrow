# `tracing::span::Span`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.span.Span.json).

<a id="op-1283b08586edf8ae3649f1c6"></a>
## Span

`struct` · `tracing::span::Span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
struct Span
```

Source: `src/span.rs:349`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

A handle representing a span, with the capability to enter the span if it
exists.

If the span was rejected by the current `Subscriber`'s filter, entering the
span will silently do nothing. Thus, the handle can be used in the same
manner regardless of whether or not the trace is currently being collected.

<a id="op-ddde89c4c339d12feb840262"></a>
## child_of

`function` · `tracing::span::Span::child_of` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn child_of(parent: impl Into<Option<Id>>, meta: &'static Metadata<'static>, values: &field::ValueSet<'_>) -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:485`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new `Span` as child of the given parent span, with the
given [metadata] and set of [field values].

After the span is constructed, [field values] and/or [`follows_from`]
annotations may be added to it.

[metadata]: super::Metadata
[field values]: super::field::ValueSet
[`follows_from`]: super::Span::follows_from

<a id="op-0914279e535e914ecdd40061"></a>
## clone

`function` · `tracing::span::Span::clone` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [348, 10], "end": [348, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/span.rs:348`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a38eacf50c34633dd1945d08"></a>
## current

`function` · `tracing::span::Span::current` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn current() -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:550`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns a handle to the span [considered by the `Subscriber`] to be the
current span.

If the subscriber indicates that it does not track the current span, or
that the thread from which this function is called is not currently
inside a span, the returned span will be disabled.

[considered by the `Subscriber`]:
    super::subscriber::Subscriber::current_span

<a id="op-58f3bd4aaae0fcfd7eabf1ba"></a>
## drop

`function` · `tracing::span::Span::drop` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1455, 1], "end": [1476, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/span.rs:1457`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0ac1698dc0d740afb9e61b4b"></a>
## enter

`function` · `tracing::span::Span::enter` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn enter(&self) -> Entered<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:786`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Enters this span, returning a guard that will exit the span when dropped.

If this span is enabled by the current subscriber, then this function will
call [`Subscriber::enter`] with the span's [`Id`], and dropping the guard
will call [`Subscriber::exit`]. If the span is disabled, this does
nothing.

# In Asynchronous Code

**Warning**: in asynchronous code that uses [async/await syntax][syntax],
`Span::enter` should be used very carefully or avoided entirely. Holding
the drop guard returned by `Span::enter` across `.await` points will
result in incorrect traces. For example,

```
# use tracing::info_span;
# async fn some_other_async_function() {}
async fn my_async_function() {
    let span = info_span!("my_async_function");

    // WARNING: This span will remain entered until this
    // guard is dropped...
    let _enter = span.enter();
    // ...but the `await` keyword may yield, causing the
    // runtime to switch to another task, while remaining in
    // this span!
    some_other_async_function().await

    // ...
}
```

The drop guard returned by `Span::enter` exits the span when it is
dropped. When an async function or async block yields at an `.await`
point, the current scope is _exited_, but values in that scope are
**not** dropped (because the async block will eventually resume
execution from that await point). This means that _another_ task will
begin executing while _remaining_ in the entered span. This results in
an incorrect trace.

Instead of using `Span::enter` in asynchronous code, prefer the
following:

* To enter a span for a synchronous section of code within an async
  block or function, prefer [`Span::in_scope`]. Since `in_scope` takes a
  synchronous closure and exits the span when the closure returns, the
  span will always be exited before the next await point. For example:
  ```
  # use tracing::info_span;
  # async fn some_other_async_function(_: ()) {}
  async fn my_async_function() {
      let span = info_span!("my_async_function");

      let some_value = span.in_scope(|| {
          // run some synchronous code inside the span...
      });

      // This is okay! The span has already been exited before we reach
      // the await point.
      some_other_async_function(some_value).await;

      // ...
  }
  ```
* For instrumenting asynchronous code, `tracing` provides the
  [`Future::instrument` combinator][instrument] for
  attaching a span to a future (async function or block). This will
  enter the span _every_ time the future is polled, and exit it whenever
  the future yields.

  `Instrument` can be used with an async block inside an async function:
  ```ignore
  # use tracing::info_span;
  use tracing::Instrument;

  # async fn some_other_async_function() {}
  async fn my_async_function() {
      let span = info_span!("my_async_function");
      async move {
         // This is correct! If we yield here, the span will be exited,
         // and re-entered when we resume.
         some_other_async_function().await;

         //more asynchronous code inside the span...

      }
        // instrument the async block with the span...
        .instrument(span)
        // ...and await it.
        .await
  }
  ```

  It can also be used to instrument calls to async functions at the
  callsite:
  ```ignore
  # use tracing::debug_span;
  use tracing::Instrument;

  # async fn some_other_async_function() {}
  async fn my_async_function() {
      let some_value = some_other_async_function()
         .instrument(debug_span!("some_other_async_function"))
         .await;

      // ...
  }
  ```

* The [`#[instrument]` attribute macro][attr] can automatically generate
  correct code when used on an async function:

  ```ignore
  # async fn some_other_async_function() {}
  #[tracing::instrument(level = "info")]
  async fn my_async_function() {

      // This is correct! If we yield here, the span will be exited,
      // and re-entered when we resume.
      some_other_async_function().await;

      // ...

  }
  ```

[syntax]: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
[`Span::in_scope`]: Span::in_scope()
[instrument]: crate::Instrument
[attr]: macro@crate::instrument

# Examples

```
# use tracing::{span, Level};
let span = span!(Level::INFO, "my_span");
let guard = span.enter();

// code here is within the span

drop(guard);

// code here is no longer within the span

```

Guards need not be explicitly dropped:

```
# use tracing::trace_span;
fn my_function() -> String {
    // enter a span for the duration of this function.
    let span = trace_span!("my_function");
    let _enter = span.enter();

    // anything happening in functions we call is still inside the span...
    my_other_function();

    // returning from the function drops the guard, exiting the span.
    return "Hello world".to_owned();
}

fn my_other_function() {
    // ...
}
```

Sub-scopes may be created to limit the duration for which the span is
entered:

```
# use tracing::{info, info_span};
let span = info_span!("my_great_span");

{
    let _enter = span.enter();

    // this event occurs inside the span.
    info!("i'm in the span!");

    // exiting the scope drops the guard, exiting the span.
}

// this event is not inside the span.
info!("i'm outside the span!")
```

[`Subscriber::enter`]: super::subscriber::Subscriber::enter()
[`Subscriber::exit`]: super::subscriber::Subscriber::exit()
[`Id`]: super::Id

<a id="op-3b2f568eb51a6612d5b15c47"></a>
## entered

`function` · `tracing::span::Span::entered` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn entered(self) -> EnteredSpan
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:896`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Enters this span, consuming it and returning a [guard][`EnteredSpan`](../operations/tracing.span.EnteredSpan.md#op-563d7a0c7d3b4e05b06837d6)
that will exit the span when dropped.

<pre class="compile_fail" style="white-space:normal;font:inherit;">
    <strong>Warning</strong>: In asynchronous code that uses async/await syntax,
    <code>Span::entered</code> may produce incorrect traces if the returned drop
    guard is held across an await point. See <a href="#in-asynchronous-code">the
    <code>Span::enter</code> documentation</a> for details.
</pre>


If this span is enabled by the current subscriber, then this function will
call [`Subscriber::enter`] with the span's [`Id`], and dropping the guard
will call [`Subscriber::exit`]. If the span is disabled, this does
nothing.

This is similar to the [`Span::enter`](../operations/tracing.span.Span.md#op-0ac1698dc0d740afb9e61b4b) method, except that it moves the
span by value into the returned guard, rather than borrowing it.
Therefore, this method can be used to create and enter a span in a
single expression, without requiring a `let`-binding. For example:

```
# use tracing::info_span;
let _span = info_span!("something_interesting").entered();
```
rather than:
```
# use tracing::info_span;
let span = info_span!("something_interesting");
let _e = span.enter();
```

Furthermore, `entered` may be used when the span must be stored in some
other struct or be passed to a function while remaining entered.

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: The returned <a href="../struct.EnteredSpan.html">
    <code>EnteredSpan</code></a> guard does not implement <code>Send</code>.
    Dropping the guard will exit <em>this</em> span, and if the guard is sent
    to another thread and dropped there, that thread may never have entered
    this span. Thus, <code>EnteredSpan</code>s should not be sent between threads.
</pre>

[syntax]: https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html

# Examples

The returned guard can be [explicitly exited][EnteredSpan::exit](../operations/tracing.span.EnteredSpan.md#op-5f56e68c59421619aa82395f),
returning the un-entered span:

```
# use tracing::{Level, span};
let span = span!(Level::INFO, "doing_something").entered();

// code here is within the span

// explicitly exit the span, returning it
let span = span.exit();

// code here is no longer within the span

// enter the span again
let span = span.entered();

// now we are inside the span once again
```

Guards need not be explicitly dropped:

```
# use tracing::trace_span;
fn my_function() -> String {
    // enter a span for the duration of this function.
    let span = trace_span!("my_function").entered();

    // anything happening in functions we call is still inside the span...
    my_other_function();

    // returning from the function drops the guard, exiting the span.
    return "Hello world".to_owned();
}

fn my_other_function() {
    // ...
}
```

Since the [`EnteredSpan`](../operations/tracing.span.EnteredSpan.md#op-563d7a0c7d3b4e05b06837d6) guard can dereference to the [`Span`](../operations/tracing.span.Span.md#op-1283b08586edf8ae3649f1c6) itself,
the span may still be accessed while entered. For example:

```rust
# use tracing::info_span;
use tracing::field;

// create the span with an empty field, and enter it.
let span = info_span!("my_span", some_field = field::Empty).entered();

// we can still record a value for the field while the span is entered.
span.record("some_field", &"hello world!");
```

[`Subscriber::enter`]: super::subscriber::Subscriber::enter()
[`Subscriber::exit`]: super::subscriber::Subscriber::exit()
[`Id`]: super::Id

<a id="op-e261a0921b06a75b41e03365"></a>
## eq

`function` · `tracing::span::Span::eq` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1375, 1], "end": [1384, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/span.rs:1376`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9e6385656562ff5bc6a265bf"></a>
## field

`function` · `tracing::span::Span::field` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn field<Q: field::AsField + ?Sized>(&self, field: &Q) -> Option<field::Field>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1107`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns a [`Field`][super::field::Field](../operations/tracing_core.field.Field.md#op-d16f69bd65cdad5606214c7c) for the field with the
given `name`, if one exists,

<a id="op-79c2a4440b5cec8c01f46886"></a>
## fmt

`function` · `tracing::span::Span::fmt` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1392, 1], "end": [1423, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:1393`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4131e6dfcbea0091271fd03d"></a>
## follows_from

`function` · `tracing::span::Span::follows_from` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn follows_from(&self, from: impl Into<Option<Id>>) -> &Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1310`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Indicates that the span with the given ID has an indirect causal
relationship with this span.

This relationship differs somewhat from the parent-child relationship: a
span may have any number of prior spans, rather than a single one; and
spans are not considered to be executing _inside_ of the spans they
follow from. This means that a span may close even if subsequent spans
that follow from it are still open, and time spent inside of a
subsequent span should not be included in the time its precedents were
executing. This is used to model causal relationships such as when a
single future spawns several related background tasks, et cetera.

If this span is disabled, or the resulting follows-from relationship
would be invalid, this function will do nothing.

# Examples

Setting a `follows_from` relationship with a `Span`:
```
# use tracing::{span, Id, Level, Span};
let span1 = span!(Level::INFO, "span_1");
let span2 = span!(Level::DEBUG, "span_2");
span2.follows_from(span1);
```

Setting a `follows_from` relationship with the current span:
```
# use tracing::{span, Id, Level, Span};
let span = span!(Level::INFO, "hello!");
span.follows_from(Span::current());
```

Setting a `follows_from` relationship with a `Span` reference:
```
# use tracing::{span, Id, Level, Span};
let span = span!(Level::INFO, "hello!");
let curr = Span::current();
span.follows_from(&curr);
```

Setting a `follows_from` relationship with an `Id`:
```
# use tracing::{span, Id, Level, Span};
let span = span!(Level::INFO, "hello!");
let id = span.id();
span.follows_from(id);
```

<a id="op-fc38e7b0ad7f86056288e673"></a>
## has_field

`function` · `tracing::span::Span::has_field` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn has_field<Q: field::AsField + ?Sized>(&self, field: &Q) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1114`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns true if this `Span` has a field for the given
[`Field`][super::field::Field](../operations/tracing_core.field.Field.md#op-d16f69bd65cdad5606214c7c) or field name.

<a id="op-53478449ec04f28cba72968d"></a>
## hash

`function` · `tracing::span::Span::hash` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<H: Hasher>(&self, hasher: &mut H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1386, 1], "end": [1390, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/span.rs:1387`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f7738942de7d77f627a30a9f"></a>
## id

`function` · `tracing::span::Span::id` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn id(&self) -> Option<Id>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1320`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns this span's `Id`, if it is enabled.

<a id="op-96f56929d165d5874027ae91"></a>
## in_scope

`function` · `tracing::span::Span::in_scope` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn in_scope<F: FnOnce() -> T, T>(&self, f: F) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1100`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Executes the given function in the context of this span.

If this span is enabled, then this function enters the span, invokes `f`
and then exits the span. If the span is disabled, `f` will still be
invoked, but in the context of the currently-executing span (if there is
one).

Returns the result of evaluating `f`.

# Examples

```
# use tracing::{trace, span, Level};
let my_span = span!(Level::TRACE, "my_span");

my_span.in_scope(|| {
    // this event occurs within the span.
    trace!("i'm in the span!");
});

// this event occurs outside the span.
trace!("i'm not in the span!");
```

Calling a function and returning the result:
```
# use tracing::{info_span, Level};
fn hello_world() -> String {
    "Hello world!".to_owned()
}

let span = info_span!("hello_world");
// the span will be entered for the duration of the call to
// `hello_world`.
let a_string = span.in_scope(hello_world);


<a id="op-562dba444b98d218f7bd07a8"></a>
## is_disabled

`function` · `tracing::span::Span::is_disabled` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn is_disabled(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1244`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns `true` if this span was disabled by the subscriber and does not
exist.

See also [`is_none`].

[`is_none`]: Span::is_none()

<a id="op-0d58e59112ed960045175ec9"></a>
## is_none

`function` · `tracing::span::Span::is_none` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn is_none(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1259`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns `true` if this span was constructed by [`Span::none`] and is
empty.

If `is_none` returns `true` for a given span, then [`is_disabled`] will
also return `true`. However, when a span is disabled by the subscriber
rather than constructed by `Span::none`, this method will return
`false`, while `is_disabled` will return `true`.

[`Span::none`]: Span::none()
[`is_disabled`]: Span::is_disabled()

<a id="op-0ee8a2028c796bb80d7a2d91"></a>
## metadata

`function` · `tracing::span::Span::metadata` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> Option<&'static Metadata<'static>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1325`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns this span's `Metadata`, if it is enabled.

<a id="op-0d5488c6cac5389394c6ac61"></a>
## new

`function` · `tracing::span::Span::new` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn new(meta: &'static Metadata<'static>, values: &field::ValueSet<'_>) -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:437`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new `Span` with the given [metadata] and set of
[field values].

The new span will be constructed by the currently-active [`Subscriber`],
with the current span as its parent (if one exists).

After the span is constructed, [field values] and/or [`follows_from`]
annotations may be added to it.

[metadata]: super::Metadata
[`Subscriber`]: super::subscriber::Subscriber
[field values]: super::field::ValueSet
[`follows_from`]: super::Span::follows_from

<a id="op-fecbdc2dbe7f9587e5547f78"></a>
## new_disabled

`function` · `tracing::span::Span::new_disabled` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn new_disabled(meta: &'static Metadata<'static>) -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:520`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new disabled span with the given `Metadata`.

This should be used when a span is constructed from a known callsite,
but the subscriber indicates that it is disabled.

Entering, exiting, and recording values on this span will not notify the
`Subscriber` but _may_ record log messages if the `log` feature flag is
enabled.

<a id="op-31019ffc84555b07ad61e6dd"></a>
## new_root

`function` · `tracing::span::Span::new_root` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn new_root(meta: &'static Metadata<'static>, values: &field::ValueSet<'_>) -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:461`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new `Span` as the root of its own trace tree, with the
given [metadata] and set of [field values].

After the span is constructed, [field values] and/or [`follows_from`]
annotations may be added to it.

[metadata]: super::Metadata
[field values]: super::field::ValueSet
[`follows_from`]: super::Span::follows_from

<a id="op-d2a38b5da6d9cc92450ac854"></a>
## none

`function` · `tracing::span::Span::none` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
const fn none() -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:534`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Constructs a new span that is *completely disabled*.

This can be used rather than `Option<Span>` to represent cases where a
span is not present.

Entering, exiting, and recording values on this span will do nothing.

<a id="op-e09c60244cda76a108832e96"></a>
## or_current

`function` · `tracing::span::Span::or_current` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn or_current(self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1027`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns this span, if it was [enabled] by the current [`Subscriber`], or
the [current span] (whose lexical distance may be further than expected),
 if this span [is disabled].

This method can be useful when propagating spans to spawned threads or
[async tasks]. Consider the following:

```
let _parent_span = tracing::info_span!("parent").entered();

// ...

let child_span = tracing::debug_span!("child");

std::thread::spawn(move || {
    let _entered = child_span.entered();

    tracing::info!("spawned a thread!");

    // ...
});
```

If the current [`Subscriber`] enables the [`DEBUG`] level, then both
the "parent" and "child" spans will be enabled. Thus, when the "spawned
a thread!" event occurs, it will be inside of the "child" span. Because
"parent" is the parent of "child", the event will _also_ be inside of
"parent".

However, if the [`Subscriber`] only enables the [`INFO`] level, the "child"
span will be disabled. When the thread is spawned, the
`child_span.entered()` call will do nothing, since "child" is not
enabled. In this case, the "spawned a thread!" event occurs outside of
*any* span, since the "child" span was responsible for propagating its
parent to the spawned thread.

If this is not the desired behavior, `Span::or_current` can be used to
ensure that the "parent" span is propagated in both cases, either as a
parent of "child" _or_ directly. For example:

```
let _parent_span = tracing::info_span!("parent").entered();

// ...

// If DEBUG is enabled, then "child" will be enabled, and `or_current`
// returns "child". Otherwise, if DEBUG is not enabled, "child" will be
// disabled, and `or_current` returns "parent".
let child_span = tracing::debug_span!("child").or_current();

std::thread::spawn(move || {
    let _entered = child_span.entered();

    tracing::info!("spawned a thread!");

    // ...
});
```

When spawning [asynchronous tasks][async tasks], `Span::or_current` can
be used similarly, in combination with [`instrument`]:

```
use tracing::Instrument;
# // lol
# mod tokio {
#     pub(super) fn spawn(_: impl std::future::Future) {}
# }

let _parent_span = tracing::info_span!("parent").entered();

// ...

let child_span = tracing::debug_span!("child");

tokio::spawn(
    async {
        tracing::info!("spawned a task!");

        // ...

    }.instrument(child_span.or_current())
);
```

In general, `or_current` should be preferred over nesting an
[`instrument`]  call inside of an [`in_current_span`] call, as using
`or_current` will be more efficient.

```
use tracing::Instrument;
# // lol
# mod tokio {
#     pub(super) fn spawn(_: impl std::future::Future) {}
# }
async fn my_async_fn() {
    // ...
}

let _parent_span = tracing::info_span!("parent").entered();

// Do this:
tokio::spawn(
    my_async_fn().instrument(tracing::debug_span!("child").or_current())
);

// ...rather than this:
tokio::spawn(
    my_async_fn()
        .instrument(tracing::debug_span!("child"))
        .in_current_span()
);
```

[enabled]: crate::Subscriber::enabled
[`Subscriber`]: crate::Subscriber
[current span]: Span::current
[is disabled]: Span::is_disabled
[`INFO`]: crate::Level::INFO
[`DEBUG`]: crate::Level::DEBUG
[async tasks]: std::task
[`instrument`]: crate::instrument::Instrument::instrument
[`in_current_span`]: crate::instrument::Instrument::in_current_span

Unresolved upstream links (retained, not inferred): `crate::Level::INFO`, `crate::Level::DEBUG`, `std::task`.

<a id="op-a63172a469e416ffb397e1f6"></a>
## record

`function` · `tracing::span::Span::record` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn record<Q: field::AsField + ?Sized, V: field::Value>(&self, field: &Q, value: V) -> &Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1193`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Records that the field described by `field` has the value `value`.

This may be used with [`field::Empty`] to declare fields whose values
are not known when the span is created, and record them later:
```
use tracing::{trace_span, field};

// Create a span with two fields: `greeting`, with the value "hello world", and
// `parting`, without a value.
let span = trace_span!("my_span", greeting = "hello world", parting = field::Empty);

// ...

// Now, record a value for parting as well.
// (note that the field name is passed as a string slice)
span.record("parting", "goodbye world!");
```
However, it may also be used to record a _new_ value for a field whose
value was already recorded:
```
use tracing::info_span;
# fn do_something() -> Result<(), ()> { Err(()) }

// Initially, let's assume that our attempt to do something is going okay...
let span = info_span!("doing_something", is_okay = true);
let _e = span.enter();

match do_something() {
    Ok(something) => {
        // ...
    }
    Err(_) => {
        // Things are no longer okay!
        span.record("is_okay", false);
    }
}
```

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: The fields associated with a span are part
    of its <a href="../struct.Metadata.html"><code>Metadata</code></a>.
    The <a href="../struct.Metadata.html"><code>Metadata</code></a>
    describing a particular span is constructed statically when the span
    is created and cannot be extended later to add new fields. Therefore,
    you cannot record a value for a field that was not specified when the
    span was created:
</pre>

```
use tracing::{trace_span, field};

// Create a span with two fields: `greeting`, with the value "hello world", and
// `parting`, without a value.
let span = trace_span!("my_span", greeting = "hello world", parting = field::Empty);

// ...

// Now, you try to record a value for a new field, `new_field`, which was not
// declared as `Empty` or populated when you created `span`.
// You won't get any error, but the assignment will have no effect!
span.record("new_field", "interesting_value_you_really_need");

// Instead, all fields that may be recorded after span creation should be declared up front,
// using field::Empty when a value is not known, as we did for `parting`.
// This `record` call will indeed replace field::Empty with "you will be remembered".
span.record("parting", "you will be remembered");
```

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
**Note**: To record several values in just one call, see the [`record_all!`](crate::record_all!) macro.
</pre></div>

[`field::Empty`]: super::field::Empty
[`Metadata`]: super::Metadata

<a id="op-b45dc1b41cfbdfa0eda26cdb"></a>
## with_subscriber

`function` · `tracing::span::Span::with_subscriber` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn with_subscriber<T>(&self, f: impl FnOnce((&Id, &Dispatch)) -> T) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 1], "end": [1373, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1368`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Invokes a function with a reference to this span's ID and subscriber.

if this span is enabled, the provided function is called, and the result is returned.
If the span is disabled, the function is not called, and this method returns `None`
instead.
