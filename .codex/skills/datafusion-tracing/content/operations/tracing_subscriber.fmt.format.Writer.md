# `tracing_subscriber::fmt::format::Writer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.Writer.json).

<a id="op-6b52b7d913af9a7ea5497021"></a>
## Writer

`struct` · `tracing_subscriber::fmt::format::Writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Writer<'writer>
```

Source: `src/fmt/format/mod.rs:309`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A writer to which formatted representations of spans and events are written.

This type is provided as input to the [`FormatEvent::format_event`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f2311ad8285c947f806e24ac) and
[`FormatFields::format_fields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-01424b9dbd594952e76f0f8d) methods, which will write formatted
representations of [`Event`](../operations/tracing_core.event.Event.md#op-7ee85389e31294d1a098f039)s and [fields] to the [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021).

This type implements the [`std::fmt::Write`] trait, allowing it to be used
with any function that takes an instance of [`std::fmt::Write`].
Additionally, it can be used with the standard library's [`std::write!`] and
[`std::writeln!`] macros.

Additionally, a [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) may expose additional [`tracing`](../modules/tracing.md#op-9c1f7d23f358e3a0a8455190)-specific
information to the formatter implementation.

[fields]: tracing_core::field

Unresolved upstream links (retained, not inferred): ``std::fmt::Write``, ``std::writeln!``, ``std::write!``.

<a id="op-3138028508fed09d42eb65dc"></a>
## by_ref

`function` · `tracing_subscriber::fmt::format::Writer::by_ref` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn by_ref(&mut self) -> Writer<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:466`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Return a new [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) that mutably borrows [`self`](../modules/tracing_subscriber.fmt.format.md#op-a04fc10e64efcca800d0eaec).

This can be used to temporarily borrow a [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) to pass a new [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)
to a function that takes a [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) by value, allowing the original writer
to still be used once that function returns.

<a id="op-352c0b79c919dc3aec54249b"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::Writer::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [601, 1], "end": [609, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:602`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-598c2cbda5806c9880f83e08"></a>
## has_ansi_escapes

`function` · `tracing_subscriber::fmt::format::Writer::has_ansi_escapes` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn has_ansi_escapes(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:541`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if [ANSI escape codes] may be used to add colors
and other formatting when writing to this `Writer`.

If this returns `false`, formatters should not emit ANSI escape codes.

[ANSI escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code

<a id="op-02ebe3c9fbf615cd3692711f"></a>
## new

`function` · `tracing_subscriber::fmt::format::Writer::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(writer: &'writer mut impl fmt::Write) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:441`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Create a new [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) from any type that implements [`fmt::Write`].

The returned `Writer` value may be passed as an argument to methods
such as [`Format::format_event`](../operations/tracing_subscriber.fmt.format.Format.md#op-7cd9b74323870acc985eeba4). Since constructing a [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)
mutably borrows the underlying [`fmt::Write`] instance, that value may
be accessed again once the [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021) is dropped. For example, if the
value implementing [`fmt::Write`] is a [`String`], it will contain
the formatted output of [`Format::format_event`](../operations/tracing_subscriber.fmt.format.Format.md#op-7cd9b74323870acc985eeba4), which may then be
used for other purposes.

[`String`]: alloc::string::String

Unresolved upstream links (retained, not inferred): `alloc::string::String`, ``fmt::Write``.

<a id="op-9bf2448353fc3c408c173d11"></a>
## sanitizes_ansi_escapes

`function` · `tracing_subscriber::fmt::format::Writer::sanitizes_ansi_escapes` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn sanitizes_ansi_escapes(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:546`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if ANSI escape codes should be sanitized.

<a id="op-7d3e455339364c0fa37ffef4"></a>
## write_char

`function` · `tracing_subscriber::fmt::format::Writer::write_char` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_char(&mut self, c: char) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:515`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Writes a [`char`] into this writer, returning whether the write succeeded.

A single [`char`] may be encoded as more than one byte.
This method can only succeed if the entire byte sequence was successfully
written, and this method will not return until all data has been
written or an error occurs.

This is identical to calling the [`write_char` method] from the [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)'s
[`std::fmt::Write`] implementation. However, it is also provided as an
inherent method, so that [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)s can be used without needing to import the
[`std::fmt::Write`] trait.

# Errors

This function will return an instance of [`std::fmt::Error`] on error.

[`write_char` method]: std::fmt::Write::write_char

Unresolved upstream links (retained, not inferred): `std::fmt::Write::write_char`, ``char``, ``std::fmt::Error``, ``std::fmt::Write``.

<a id="op-b26bbded97ccab6bdfb77028"></a>
## write_char

`function` · `tracing_subscriber::fmt::format::Writer::write_char` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_char(&mut self, c: char) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [584, 1], "end": [599, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Write", "path": "Write"}, "trait_path": "core::fmt::Write"}`

Source: `src/fmt/format/mod.rs:591`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a0a3fb12d68980fb7b4959a6"></a>
## write_fmt

`function` · `tracing_subscriber::fmt::format::Writer::write_fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:531`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Glue for usage of the [`write!`] macro with [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)s.

This method should generally not be invoked manually, but rather through
the [`write!`] macro itself.

This is identical to calling the [`write_fmt` method] from the [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)'s
[`std::fmt::Write`] implementation. However, it is also provided as an
inherent method, so that [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)s can be used with the [`write!`] macro
without needing to import the
[`std::fmt::Write`] trait.

[`write_fmt` method]: std::fmt::Write::write_fmt

Unresolved upstream links (retained, not inferred): `std::fmt::Write::write_fmt`, ``std::fmt::Write``, ``write!``.

<a id="op-ff99575a055b77de41dbe201"></a>
## write_fmt

`function` · `tracing_subscriber::fmt::format::Writer::write_fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [584, 1], "end": [599, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Write", "path": "Write"}, "trait_path": "core::fmt::Write"}`

Source: `src/fmt/format/mod.rs:596`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1eb14e325236abdab7d1fa5e"></a>
## write_str

`function` · `tracing_subscriber::fmt::format::Writer::write_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_str(&mut self, s: &str) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [584, 1], "end": [599, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Write", "path": "Write"}, "trait_path": "core::fmt::Write"}`

Source: `src/fmt/format/mod.rs:586`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-51f663bad44fc6e7795172f0"></a>
## write_str

`function` · `tracing_subscriber::fmt::format::Writer::write_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn write_str(&mut self, s: &str) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Writer", "path": "Writer"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [582, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:493`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Writes a string slice into this [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021), returning whether the write succeeded.

This method can only succeed if the entire string slice was successfully
written, and this method will not return until all data has been written
or an error occurs.

This is identical to calling the [`write_str` method] from the [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)'s
[`std::fmt::Write`] implementation. However, it is also provided as an
inherent method, so that [`Writer`](../operations/tracing_subscriber.fmt.format.Writer.md#op-6b52b7d913af9a7ea5497021)s can be used without needing to import the
[`std::fmt::Write`] trait.

# Errors

This function will return an instance of [`std::fmt::Error`] on error.

[`write_str` method]: std::fmt::Write::write_str

Unresolved upstream links (retained, not inferred): ``std::fmt::Error``, `std::fmt::Write::write_str`, ``std::fmt::Write``.
