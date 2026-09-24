# `tracing_core::event::Event`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.event.Event.json).

<a id="op-7ee85389e31294d1a098f039"></a>
## Event

`struct` · `tracing_core::event::Event` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Event<'a>
```

Source: `src/event.rs:23`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

`Event`s represent single points in time where something occurred during the
execution of a program.

An `Event` can be compared to a log record in unstructured logging, but with
two key differences:
- `Event`s exist _within the context of a [span]_. Unlike log lines, they
  may be located within the trace tree, allowing visibility into the
  _temporal_ context in which the event occurred, as well as the source
  code location.
- Like spans, `Event`s have structured key-value data known as _[fields]_,
  which may include textual message. In general, a majority of the data
  associated with an event should be in the event's fields rather than in
  the textual message, as the fields are more structured.

[span]: super::span
[fields]: super::field

<a id="op-6d486d1dfda509afc97538e6"></a>
## child_of

`function` · `tracing_core::event::Event::child_of` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn child_of(parent: impl Into<Option<Id>>, metadata: &'static Metadata<'static>, fields: &'a field::ValueSet<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:71`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Constructs a new `Event` with the specified metadata and set of values,
and observes it with the current subscriber and an explicit parent.

<a id="op-5854759cd1f3049abd3f422d"></a>
## dispatch

`function` · `tracing_core::event::Event::dispatch` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn dispatch(metadata: &'static Metadata<'static>, fields: &'a field::ValueSet<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:32`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Constructs a new `Event` with the specified metadata and set of values,
and observes it with the current subscriber.

<a id="op-35646a228129603a89dfe595"></a>
## fields

`function` · `tracing_core::event::Event::fields` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> field::Iter
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:91`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an iterator over the set of values on this `Event`.

<a id="op-319b55080af047558b3c0e5b"></a>
## fmt

`function` · `tracing_core::event::Event::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 10], "end": [22, 15], "filename": "src/event.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/event.rs:22`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-382440546744f7d6837287b0"></a>
## is_contextual

`function` · `tracing_core::event::Event::is_contextual` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_contextual(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:114`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if the new event's parent should be determined based on the
current context.

If this is true and the current thread is currently inside a span, then
that span should be the new event's parent. Otherwise, if the current
thread is _not_ inside a span, then the new event will be the root of its
own trace tree.

<a id="op-a33b9348d28a3cef563bb35b"></a>
## is_root

`function` · `tracing_core::event::Event::is_root` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_root(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:103`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if the new event should be a root.

<a id="op-37942f1084ef33b05f3a321b"></a>
## metadata

`function` · `tracing_core::event::Event::metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &'static Metadata<'static>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:98`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns [metadata] describing this `Event`.

[metadata]: super::Metadata

<a id="op-376336ce82e043f4eb339ba8"></a>
## new

`function` · `tracing_core::event::Event::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new(metadata: &'static Metadata<'static>, fields: &'a field::ValueSet<'a>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:42`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a new `Event` in the current span, with the specified metadata
and set of values.

<a id="op-9b97015673515379cb3a39b8"></a>
## new_child_of

`function` · `tracing_core::event::Event::new_child_of` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new_child_of(parent: impl Into<Option<Id>>, metadata: &'static Metadata<'static>, fields: &'a field::ValueSet<'a>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:53`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a new `Event` as a child of the specified span, with the
provided metadata and set of values.

<a id="op-ecedf8104c897ba8f36d53ac"></a>
## parent

`function` · `tracing_core::event::Event::parent` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn parent(&self) -> Option<&Id>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:122`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the new event's explicitly-specified parent, if there is one.

Otherwise (if the new event is a root or is a child of the current span),
returns `None`.

<a id="op-9ae6965577fc1a5e4175cfa1"></a>
## record

`function` · `tracing_core::event::Event::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, visitor: &mut dyn field::Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::event::Event", "path": "Event"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 1], "end": [128, 2], "filename": "src/event.rs"}, "trait": null, "trait_path": null}`

Source: `src/event.rs:86`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visits all the fields on this `Event` with the specified [visitor].

[visitor]: super::field::Visit
