# `opentelemetry::context::Context`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.context.Context.json).

<a id="op-ca59114006a0744de26919a1"></a>
## Context

`struct` · `opentelemetry::context::Context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Context
```

Source: `src/context.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An execution-scoped collection of values.

A [`Context`](../operations/opentelemetry.context.Context.md#op-ca59114006a0744de26919a1) is a propagation mechanism which carries execution-scoped
values across API boundaries and between logically associated execution
units. Cross-cutting concerns access their data in-process using the same
shared context object.

[`Context`](../operations/opentelemetry.context.Context.md#op-ca59114006a0744de26919a1)s are immutable, and their write operations result in the creation
of a new context containing the original values and the new specified values.

## Context state

Concerns can create and retrieve their local state in the current execution
state represented by a context through the [`get`] and [`with_value`]
methods. It is recommended to use application-specific types when storing new
context values to avoid unintentionally overwriting existing state.

## Managing the current context

Contexts can be associated with the caller's current execution unit on a
given thread via the [`attach`] method, and previous contexts can be restored
by dropping the returned [`ContextGuard`](../operations/opentelemetry.context.ContextGuard.md#op-30c719e39d73839fb9d98820). Context can be nested, and will
restore their parent outer context when detached on drop. To access the
values of the context, a snapshot can be created via the [`Context::current`]
method.

[`Context::current`]: Context::current()
[`get`]: Context::get()
[`with_value`]: Context::with_value()
[`attach`]: Context::attach()

# Examples

```
use opentelemetry::Context;

// Application-specific `a` and `b` values
#[derive(Debug, PartialEq)]
struct ValueA(&'static str);
#[derive(Debug, PartialEq)]
struct ValueB(u64);

let _outer_guard = Context::new().with_value(ValueA("a")).attach();

// Only value a has been set
let current = Context::current();
assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
assert_eq!(current.get::<ValueB>(), None);

{
    let _inner_guard = Context::current_with_value(ValueB(42)).attach();
    // Both values are set in inner context
    let current = Context::current();
    assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
    assert_eq!(current.get::<ValueB>(), Some(&ValueB(42)));
}

// Resets to only the `a` value when inner guard is dropped
let current = Context::current();
assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
assert_eq!(current.get::<ValueB>(), None);
```

<a id="op-91ee4b778a3febda01417347"></a>
## attach

`function` · `opentelemetry::context::Context::attach` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn attach(self) -> ContextGuard
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:326`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Replaces the current context on this thread with this context.

Dropping the returned [`ContextGuard`](../operations/opentelemetry.context.ContextGuard.md#op-30c719e39d73839fb9d98820) will reset the current context to the
previous value.


# Examples

```
use opentelemetry::Context;

#[derive(Debug, PartialEq)]
struct ValueA(&'static str);

let my_cx = Context::new().with_value(ValueA("a"));

// Set the current thread context
let cx_guard = my_cx.attach();
assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA("a")));

// Drop the guard to restore the previous context
drop(cx_guard);
assert_eq!(Context::current().get::<ValueA>(), None);
```

Guards do not need to be explicitly dropped:

```
use opentelemetry::Context;

#[derive(Debug, PartialEq)]
struct ValueA(&'static str);

fn my_function() -> String {
    // attach a context the duration of this function.
    let my_cx = Context::new().with_value(ValueA("a"));
    // NOTE: a variable name after the underscore is **required** or rust
    // will drop the guard, restoring the previous context _immediately_.
    let _guard = my_cx.attach();

    // anything happening in functions we call can still access my_cx...
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
use opentelemetry::Context;

#[derive(Debug, PartialEq)]
struct ValueA(&'static str);

let my_cx = Context::new().with_value(ValueA("a"));

{
    let _guard = my_cx.attach();

    // the current context can access variables in
    assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA("a")));

    // exiting the scope drops the guard, detaching the context.
}

// this is back in the default empty context
assert_eq!(Context::current().get::<ValueA>(), None);
```

<a id="op-3f859f09eb60bd4f395b1fa0"></a>
## baggage

`function` · `opentelemetry::context::Context::baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn baggage(&self) -> &Baggage
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [405, 1], "end": [422, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::baggage::BaggageExt", "path": "BaggageExt"}, "trait_path": "opentelemetry::baggage::BaggageExt"}`

Source: `src/baggage.rs:418`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a6ebd37fb630d05bda98d3a5"></a>
## clone

`function` · `opentelemetry::context::Context::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Context
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [95, 10], "end": [95, 15], "filename": "src/context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/context.rs:95`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5faa1d5ea18d976bb7c1896d"></a>
## current

`function` · `opentelemetry::context::Context::current` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn current() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:133`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns an immutable snapshot of the current thread's context.

# Examples

```
use opentelemetry::Context;

#[derive(Debug, PartialEq)]
struct ValueA(&'static str);

fn do_work() {
    assert_eq!(Context::current().get(), Some(&ValueA("a")));
}

let _guard = Context::new().with_value(ValueA("a")).attach();
do_work()
```

<a id="op-14f57cc0f3bfc1e1868d1a98"></a>
## current_with_baggage

`function` · `opentelemetry::context::Context::current_with_baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn current_with_baggage<T: Into<Baggage>>(baggage: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [405, 1], "end": [422, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::baggage::BaggageExt", "path": "BaggageExt"}, "trait_path": "opentelemetry::baggage::BaggageExt"}`

Source: `src/baggage.rs:410`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5c606d726d80956097f1cc1c"></a>
## current_with_span

`function` · `opentelemetry::context::Context::current_with_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn current_with_span<T: trace::Span + Send + Sync + 'static>(span: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 1], "end": [325, 2], "filename": "src/trace/context.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::context::TraceContextExt", "path": "TraceContextExt"}, "trait_path": "opentelemetry::trace::context::TraceContextExt"}`

Source: `src/trace/context.rs:302`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-760a5ed4916d5d9eec44e279"></a>
## current_with_value

`function` · `opentelemetry::context::Context::current_with_value` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn current_with_value<T: 'static + Send + Sync>(value: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:175`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a clone of the current thread's context with the given value.

This is a more efficient form of `Context::current().with_value(value)`
as it avoids the intermediate context clone.

# Examples

```
use opentelemetry::Context;

// Given some value types defined in your application
#[derive(Debug, PartialEq)]
struct ValueA(&'static str);
#[derive(Debug, PartialEq)]
struct ValueB(u64);

// You can create and attach context with the first value set to "a"
let _guard = Context::new().with_value(ValueA("a")).attach();

// And create another context based on the fist with a new value
let all_current_and_b = Context::current_with_value(ValueB(42));

// The second context now contains all the current values and the addition
assert_eq!(all_current_and_b.get::<ValueA>(), Some(&ValueA("a")));
assert_eq!(all_current_and_b.get::<ValueB>(), Some(&ValueB(42)));
```

<a id="op-adaad27e69540b23eab894f3"></a>
## default

`function` · `opentelemetry::context::Context::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Context
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [95, 17], "end": [95, 24], "filename": "src/context.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/context.rs:95`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ac59e4f010e3936b1a26105b"></a>
## enter_telemetry_suppressed_scope

`function` · `opentelemetry::context::Context::enter_telemetry_suppressed_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn enter_telemetry_suppressed_scope() -> ContextGuard
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:392`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Enters a scope where telemetry is suppressed.

This method is specifically designed for OpenTelemetry components (like Exporters,
Processors etc.) to prevent generating recursive or self-referential
telemetry data when performing their own operations.

Without suppression, we have a telemetry-induced-telemetry situation
where, operations like exporting telemetry could generate new telemetry
about the export process itself, potentially causing:
- Infinite telemetry feedback loops
- Excessive resource consumption

This method:
1. Takes the current context
2. Creates a new context from current, with `suppress_telemetry` set to `true`
3. Attaches it to the current thread
4. Returns a guard that restores the previous context when dropped

OTel SDK components would check `is_current_telemetry_suppressed()` before
generating new telemetry, but not end users.

# Examples

```
use opentelemetry::Context;

// Example: Inside an exporter's implementation
fn example_export_function() {
    // Prevent telemetry-generating operations from creating more telemetry
    let _guard = Context::enter_telemetry_suppressed_scope();
     
    // Verify suppression is active
    assert_eq!(Context::is_current_telemetry_suppressed(), true);
     
    // Here you would normally perform operations that might generate telemetry
    // but now they won't because the context has suppression enabled
}

// Demonstrate the function
example_export_function();
```

<a id="op-5348708e2c9cf7f7104f0ab4"></a>
## fmt

`function` · `opentelemetry::context::Context::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [430, 1], "end": [452, 2], "filename": "src/context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/context.rs:431`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8165c08db47d116692be7936"></a>
## get

`function` · `opentelemetry::context::Context::get` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get<T: 'static>(&self) -> Option<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:200`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a reference to the entry for the corresponding value type.

# Examples

```
use opentelemetry::Context;

// Given some value types defined in your application
#[derive(Debug, PartialEq)]
struct ValueA(&'static str);
#[derive(Debug, PartialEq)]
struct MyUser();

let cx = Context::new().with_value(ValueA("a"));

// Values can be queried by type
assert_eq!(cx.get::<ValueA>(), Some(&ValueA("a")));

// And return none if not yet set
assert_eq!(cx.get::<MyUser>(), None);
```

<a id="op-50e85f26b8ee5ecf17d9b5bd"></a>
## has_active_span

`function` · `opentelemetry::context::Context::has_active_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn has_active_span(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 1], "end": [325, 2], "filename": "src/trace/context.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::context::TraceContextExt", "path": "TraceContextExt"}, "trait_path": "opentelemetry::trace::context::TraceContextExt"}`

Source: `src/trace/context.rs:318`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9d4b3a9841b53c3e4014d752"></a>
## is_current_telemetry_suppressed

`function` · `opentelemetry::context::Context::is_current_telemetry_suppressed` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_current_telemetry_suppressed() -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:407`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns whether telemetry is suppressed in the current context.

This method is used by OpenTelemetry components to determine whether they should
generate new telemetry in the current execution context. It provides a performant
way to check the suppression state.

End-users generally should not use this method directly, as it is primarily intended for
OpenTelemetry SDK components.



<a id="op-c2eb69167d9e31584794a5fa"></a>
## is_telemetry_suppressed

`function` · `opentelemetry::context::Context::is_telemetry_suppressed` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_telemetry_suppressed(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:337`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns whether telemetry is suppressed in this context.

<a id="op-77bdb785ab151e271df67358"></a>
## map_current

`function` · `opentelemetry::context::Context::map_current` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn map_current<T>(f: impl FnOnce(&Context) -> T) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:145`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Applies a function to the current context returning its value.

This can be used to build higher performing algebraic expressions for
optionally creating a new context without the overhead of cloning the
current one and dropping it.

Note: This function will panic if you attempt to attach another context
while the current one is still borrowed.

<a id="op-36a296d03d62efaa78fc820b"></a>
## new

`function` · `opentelemetry::context::Context::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:112`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates an empty `Context`.

The context is initially created with a capacity of 0, so it will not
allocate. Use [`with_value`] to create a new context that has entries.

[`with_value`]: Context::with_value()

<a id="op-5205c7fff4005a57c58a5786"></a>
## span

`function` · `opentelemetry::context::Context::span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span(&self) -> SpanRef<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 1], "end": [325, 2], "filename": "src/trace/context.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::context::TraceContextExt", "path": "TraceContextExt"}, "trait_path": "opentelemetry::trace::context::TraceContextExt"}`

Source: `src/trace/context.rs:310`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d2d917ec62e21ecddd2f14e2"></a>
## with_baggage

`function` · `opentelemetry::context::Context::with_baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_baggage<T: Into<Baggage>>(&self, baggage: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [405, 1], "end": [422, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::baggage::BaggageExt", "path": "BaggageExt"}, "trait_path": "opentelemetry::baggage::BaggageExt"}`

Source: `src/baggage.rs:406`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-41428b2b95bd992fc458bb88"></a>
## with_cleared_baggage

`function` · `opentelemetry::context::Context::with_cleared_baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_cleared_baggage(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [405, 1], "end": [422, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "opentelemetry::baggage::BaggageExt", "path": "BaggageExt"}, "trait_path": "opentelemetry::baggage::BaggageExt"}`

Source: `src/baggage.rs:414`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2b1f3b985f07305b3756a976"></a>
## with_remote_span_context

`function` · `opentelemetry::context::Context::with_remote_span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_remote_span_context(&self, span_context: trace::SpanContext) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 1], "end": [325, 2], "filename": "src/trace/context.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::context::TraceContextExt", "path": "TraceContextExt"}, "trait_path": "opentelemetry::trace::context::TraceContextExt"}`

Source: `src/trace/context.rs:322`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-74e5145132252dde810736fc"></a>
## with_span

`function` · `opentelemetry::context::Context::with_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span<T: trace::Span + Send + Sync + 'static>(&self, span: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "crate::Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [301, 1], "end": [325, 2], "filename": "src/trace/context.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::context::TraceContextExt", "path": "TraceContextExt"}, "trait_path": "opentelemetry::trace::context::TraceContextExt"}`

Source: `src/trace/context.rs:306`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-47cac59f00a2e94a2d5286e7"></a>
## with_telemetry_suppressed

`function` · `opentelemetry::context::Context::with_telemetry_suppressed` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_telemetry_suppressed(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:342`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new context with telemetry suppression enabled.

<a id="op-0667fdba946d4f37626b9ae0"></a>
## with_value

`function` · `opentelemetry::context::Context::with_value` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_value<T: 'static + Send + Sync>(&self, value: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::context::Context", "path": "Context"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [428, 2], "filename": "src/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/context.rs:234`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a copy of the context with the new value included.

# Examples

```
use opentelemetry::Context;

// Given some value types defined in your application
#[derive(Debug, PartialEq)]
struct ValueA(&'static str);
#[derive(Debug, PartialEq)]
struct ValueB(u64);

// You can create a context with the first value set to "a"
let cx_with_a = Context::new().with_value(ValueA("a"));

// And create another context based on the fist with a new value
let cx_with_a_and_b = cx_with_a.with_value(ValueB(42));

// The first context is still available and unmodified
assert_eq!(cx_with_a.get::<ValueA>(), Some(&ValueA("a")));
assert_eq!(cx_with_a.get::<ValueB>(), None);

// The second context now contains both values
assert_eq!(cx_with_a_and_b.get::<ValueA>(), Some(&ValueA("a")));
assert_eq!(cx_with_a_and_b.get::<ValueB>(), Some(&ValueB(42)));
```
