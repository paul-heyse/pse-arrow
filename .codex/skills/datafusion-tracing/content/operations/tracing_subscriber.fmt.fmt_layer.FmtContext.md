# `tracing_subscriber::fmt::fmt_layer::FmtContext`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.fmt_layer.FmtContext.json).

<a id="op-edf8024d45f27b1a256d34ab"></a>
## FmtContext

`struct` · `tracing_subscriber::fmt::fmt_layer::FmtContext` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FmtContext<'a, S, N>
```

Source: `src/fmt/fmt_layer.rs:1086`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Provides the current span context to a formatter.

<a id="op-cb81b22bdedd9293ec6d0904"></a>
## current_span

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn current_span(&self) -> Current
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1185`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the current span for this formatter.

<a id="op-2aea7f1bdeff2346eb14665a"></a>
## event_scope

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::event_scope` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_scope(&self) -> Option<registry::Scope<'_, S>> where S: for<'lookup> registry::LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1258`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an iterator over the [stored data] for all the spans in the
event's span context, starting with its parent span and ending with the
root of the trace tree.

This is equivalent to calling the [`Context::event_scope`](../operations/tracing_subscriber.layer.context.Context.md#op-f08e7c9a674253fafb02ad26) method and
passing the event currently being formatted.

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: Compared to <a href="#method.scope"><code>scope</code></a> this
returns the spans in reverse order (from leaf to root). Use
<a href="../registry/struct.Scope.html#method.from_root"><code>Scope::from_root</code></a>
in case root-to-leaf ordering is desired.
</pre></div>

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: This requires the wrapped subscriber to implement the
<a href="../registry/trait.LookupSpan.html"><code>LookupSpan</code></a> trait.
See the documentation on <a href="./struct.Context.html"><code>Context</code>'s
declaration</a> for details.
</pre></div>

[stored data]: crate::registry::SpanRef

<a id="op-e54a17b6971582138e3bad65"></a>
## exists

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::exists` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn exists(&self, id: &Id) -> bool where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1163`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if an active span exists for the given `Id`.

<a id="op-071d5b13e98cb914d4209740"></a>
## field_format

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::field_format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn field_format(&self) -> &N
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1272`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the [field formatter] configured by the subscriber invoking
`format_event`.

The event formatter may use the returned field formatter to format the
fields of any events it records.

[field formatter]: FormatFields

<a id="op-f0e5967c871ffaf60440c995"></a>
## fmt

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [1092, 1], "end": [1096, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/fmt_layer.rs:1093`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b964036ced210b91c1971f0b"></a>
## format_fields

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::format_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_fields<R: RecordFields>(&self, writer: format::Writer<'writer>, fields: R) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1098, 1], "end": [1110, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}, "trait_path": "tracing_subscriber::fmt::format::FormatFields"}`

Source: `src/fmt/fmt_layer.rs:1103`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c0bc588eb4569c47407e1419"></a>
## lookup_current

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::lookup_current` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn lookup_current(&self) -> Option<SpanRef<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1177`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns [stored data] for the span that the wrapped subscriber considers
to be the current.

If this returns `None`, then we are not currently within a span.

[stored data]: crate::registry::SpanRef

<a id="op-e2f83b7707e99cc4b19c4347"></a>
## metadata

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::metadata` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self, id: &Id) -> Option<&'static Metadata<'static>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1140`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns metadata for the span with the given `id`, if it exists.

If this returns `None`, then no span exists for that ID (either it has
closed or the ID is invalid).

<a id="op-d3d4f171dbda504f5db926df"></a>
## parent_span

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::parent_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn parent_span(&self) -> Option<SpanRef<'_, S>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1197`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns [stored data] for the parent span of the event currently being
formatted.

If the event has a contextual parent, this will return the current span. If
the event has an explicit parent span, this will return that span. If
the event does not have a parent span, this will return `None`.

[stored data]: SpanRef

<a id="op-abaf2269ed1db8f2b6e34742"></a>
## span

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span(&self, id: &Id) -> Option<SpanRef<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1154`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns [stored data] for the span with the given `id`, if it exists.

If this returns `None`, then no span exists for that ID (either it has
closed or the ID is invalid).

[stored data]: crate::registry::SpanRef

<a id="op-7463dda6ac9c5d6073e6280d"></a>
## span_scope

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::span_scope` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span_scope(&self, id: &Id) -> Option<registry::Scope<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1227`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an iterator over the [stored data] for all the spans in the
current context, starting with the specified span and ending with the
root of the trace tree and ending with the current span.

This is equivalent to the [`Context::span_scope`](../operations/tracing_subscriber.layer.context.Context.md#op-c6d804051c33f80e2c60a97a) method.

<div class="information">
    <div class="tooltip ignore" style="">ⓘ<span class="tooltiptext">Note</span></div>
</div>
<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: Compared to <a href="#method.scope"><code>scope</code></a> this
returns the spans in reverse order (from leaf to root). Use
<a href="../registry/struct.Scope.html#method.from_root"><code>Scope::from_root</code></a>
in case root-to-leaf ordering is desired.
</pre></div>

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: This requires the wrapped subscriber to implement the
<a href="../registry/trait.LookupSpan.html"><code>LookupSpan</code></a> trait.
See the documentation on <a href="./struct.Context.html"><code>Context</code>'s
declaration</a> for details.
</pre></div>

[stored data]: crate::registry::SpanRef

<a id="op-dbce363634a4dba3843cfb0e"></a>
## visit_spans

`function` · `tracing_subscriber::fmt::fmt_layer::FmtContext::visit_spans` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn visit_spans<E, F>(&self, f: F) -> Result<(), E> where F: FnMut(&SpanRef<'_, S>) -> Result<(), E>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::FmtContext", "path": "FmtContext"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [1112, 1], "end": [1275, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:1122`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visits every span in the current context with a closure.

The provided closure will be called first with the current span,
and then with that span's parent, and then that span's parent,
and so on until a root span is reached.
