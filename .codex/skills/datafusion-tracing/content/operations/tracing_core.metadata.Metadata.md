# `tracing_core::metadata::Metadata`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.metadata.Metadata.json).

<a id="op-3c5a7a9d81c273e2173bb24c"></a>
## Metadata

`struct` · `tracing_core::metadata::Metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Metadata<'a>
```

Source: `src/metadata.rs:57`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Metadata describing a [span] or [event].

All spans and events have the following metadata:
- A [name], represented as a static string.
- A [target], a string that categorizes part of the system where the span
  or event occurred. The `tracing` macros default to using the module
  path where the span or event originated as the target, but it may be
  overridden.
- A [verbosity level]. This determines how verbose a given span or event
  is, and allows enabling or disabling more verbose diagnostics
  situationally. See the documentation for the [`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff) type for details.
- The names of the [fields] defined by the span or event.
- Whether the metadata corresponds to a span or event.

In addition, the following optional metadata describing the source code
location where the span or event originated _may_ be provided:
- The [file name]
- The [line number]
- The [module path]

Metadata is used by [`Subscriber`]s when filtering spans and events, and it
may also be used as part of their data payload.

When created by the `event!` or `span!` macro, the metadata describing a
particular event or span is constructed statically and exists as a single
static instance. Thus, the overhead of creating the metadata is
_significantly_ lower than that of creating the actual span. Therefore,
filtering is based on metadata, rather than on the constructed span.

## Equality

In well-behaved applications, two `Metadata` with equal
[callsite identifiers] will be equal in all other ways (i.e., have the same
`name`, `target`, etc.). Consequently, in release builds, [`Metadata::eq`](../operations/tracing_core.metadata.Metadata.md#op-7a657ed96e3e5699ea8d2bb1)
*only* checks that its arguments have equal callsites. However, the equality
of `Metadata`'s other fields is checked in debug builds.

[span]: super::span
[event]: super::event
[name]: Self::name
[target]: Self::target
[fields]: Self::fields
[verbosity level]: Self::level
[file name]: Self::file
[line number]: Self::line
[module path]: Self::module_path
[`Subscriber`]: super::subscriber::Subscriber
[callsite identifiers]: Self::callsite

<a id="op-8a68e364155b7f925a5f6413"></a>
## callsite

`function` · `tracing_core::metadata::Metadata::callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite(&self) -> callsite::Identifier
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:320`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an opaque `Identifier` that uniquely identifies the callsite
this `Metadata` originated from.

<a id="op-7a657ed96e3e5699ea8d2bb1"></a>
## eq

`function` · `tracing_core::metadata::Metadata::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [453, 1], "end": [505, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metadata.rs:455`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-657cff8629da7d0887e7c370"></a>
## fields

`function` · `tracing_core::metadata::Metadata::fields` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> &field::FieldSet
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:276`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the names of the fields on the described span or event.

<a id="op-18ec8b0827f331c8d44790b5"></a>
## file

`function` · `tracing_core::metadata::Metadata::file` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn file(&self) -> Option<&'a str>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:307`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the name of the source code file where the span
occurred, or `None` if the file is unknown

<a id="op-00a74b04e65322c674bd406e"></a>
## fmt

`function` · `tracing_core::metadata::Metadata::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [343, 1], "end": [374, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metadata.rs:344`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c188e09eacf51fc37631a92c"></a>
## is_event

`function` · `tracing_core::metadata::Metadata::is_event` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_event(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:325`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if the callsite kind is `Event`.

<a id="op-4ade993820d14a2c47a745cd"></a>
## is_span

`function` · `tracing_core::metadata::Metadata::is_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_span(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:330`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Return true if the callsite kind is `Span`.

<a id="op-69158220150d0d4094169e1f"></a>
## level

`function` · `tracing_core::metadata::Metadata::level` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn level(&self) -> &Level
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:281`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the level of verbosity of the described span or event.

<a id="op-ccc652022766b3fc34e7b90d"></a>
## line

`function` · `tracing_core::metadata::Metadata::line` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn line(&self) -> Option<u32>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:313`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the line number in the source code file where the span
occurred, or `None` if the line number is unknown.

<a id="op-0a70a64c1b61c3b42ff196ea"></a>
## module_path

`function` · `tracing_core::metadata::Metadata::module_path` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn module_path(&self) -> Option<&'a str>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:301`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the path to the Rust module where the span occurred, or
`None` if the module path is unknown.

<a id="op-059f17fc0a3421461a1cb91a"></a>
## name

`function` · `tracing_core::metadata::Metadata::name` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn name(&self) -> &'static str
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:286`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the name of the span.

<a id="op-2b66c84a4300c1c1a870c890"></a>
## new

`function` · `tracing_core::metadata::Metadata::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn new(name: &'static str, target: &'a str, level: Level, file: Option<&'a str>, line: Option<u32>, module_path: Option<&'a str>, fields: field::FieldSet, kind: Kind) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:252`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Construct new metadata for a span or event, with a name, target, level, field
names, and optional source code location.

<a id="op-27e0482e00fc3d7b78745787"></a>
## target

`function` · `tracing_core::metadata::Metadata::target` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn target(&self) -> &'a str
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "Metadata"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 1], "end": [341, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:295`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a string describing the part of the system where the span or
event that this metadata describes occurred.

Typically, this is the module path, but alternate targets may be set
when spans or events are constructed.
