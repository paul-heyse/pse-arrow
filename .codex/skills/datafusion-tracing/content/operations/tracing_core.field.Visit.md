# `tracing_core::field::Visit`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.Visit.json).

<a id="op-5756ad7dac310ca14da0016b"></a>
## Visit

`trait` · `tracing_core::field::Visit` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
trait Visit
```

Source: `src/field.rs:275`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visits typed values.

An instance of `Visit` ("a visitor") represents the logic necessary to
record field values of various types. When an implementor of [`Value`](../operations/tracing_core.field.Value.md#op-3bdffa798a494864d07fca1b) is
[recorded], it calls the appropriate method on the provided visitor to
indicate the type that value should be recorded as.

When a [`Subscriber`] implementation [records an `Event`] or a
[set of `Value`s added to a `Span`], it can pass an `&mut Visit` to the
`record` method on the provided [`ValueSet`](../operations/tracing_core.field.ValueSet.md#op-81b1992dfc73d64b6f31eb3a) or [`Event`]. This visitor
will then be used to record all the field-value pairs present on that
`Event` or `ValueSet`.

# Examples

A simple visitor that writes to a string might be implemented like so:
```
# extern crate tracing_core as tracing;
use std::fmt::{self, Write};
use tracing::field::{Value, Visit, Field};
pub struct StringVisitor<'a> {
    string: &'a mut String,
}

impl<'a> Visit for StringVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        write!(self.string, "{} = {:?}; ", field.name(), value).unwrap();
    }
}
```
This visitor will format each recorded value using `fmt::Debug`, and
append the field name and formatted value to the provided string,
regardless of the type of the recorded value. When all the values have
been recorded, the `StringVisitor` may be dropped, allowing the string
to be printed or stored in some other data structure.

The `Visit` trait provides default implementations for `record_i64`,
`record_u64`, `record_bool`, `record_str`, and `record_error`, which simply
forward the recorded value to `record_debug`. Thus, `record_debug` is the
only method which a `Visit` implementation *must* implement. However,
visitors may override the default implementations of these functions in
order to implement type-specific behavior.

Additionally, when a visitor receives a value of a type it does not care
about, it is free to ignore those values completely. For example, a
visitor which only records numeric data might look like this:

```
# extern crate tracing_core as tracing;
# use std::fmt::{self, Write};
# use tracing::field::{Value, Visit, Field};
pub struct SumVisitor {
    sum: i64,
}

impl Visit for SumVisitor {
    fn record_i64(&mut self, _field: &Field, value: i64) {
       self.sum += value;
    }

    fn record_u64(&mut self, _field: &Field, value: u64) {
        self.sum += value as i64;
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn fmt::Debug) {
        // Do nothing
    }
}
```

This visitor (which is probably not particularly useful) keeps a running
sum of all the numeric values it records, and ignores all other values. A
more practical example of recording typed values is presented in
`examples/counters.rs`, which demonstrates a very simple metrics system
implemented using `tracing`.

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: The <code>record_error</code> trait method is only
available when the Rust standard library is present, as it requires the
<code>std::error::Error</code> trait.
</pre></div>

[recorded]: Value::record
[`Subscriber`]: super::subscriber::Subscriber
[records an `Event`]: super::subscriber::Subscriber::event
[set of `Value`s added to a `Span`]: super::subscriber::Subscriber::record
[`Event`]: super::event::Event

<a id="op-582935eef74d2fcd4040078f"></a>
## record_bool

`function` · `tracing_core::field::Visit::record_bool` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bool(&mut self, field: &Field, value: bool)
```

Source: `src/field.rs:311`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a boolean value.

<a id="op-7da76250822efe34277a3d9b"></a>
## record_bytes

`function` · `tracing_core::field::Visit::record_bytes` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bytes(&mut self, field: &Field, value: &[u8])
```

Source: `src/field.rs:321`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a byte slice.

<a id="op-f4add0c4dcfe340ec2b370a0"></a>
## record_debug

`function` · `tracing_core::field::Visit::record_debug` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Source: `src/field.rs:340`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a value implementing `fmt::Debug`.

<a id="op-11b81e43cdc141b65880f0ec"></a>
## record_error

`function` · `tracing_core::field::Visit::record_error` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_error(&mut self, field: &Field, value: &dyn std::error::Error + 'static)
```

Source: `src/field.rs:335`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Records a type implementing `Error`.

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: This is only enabled when the Rust standard library is
present.
</pre>
</div>

<a id="op-09326f55e9512d52a6db0ebc"></a>
## record_f64

`function` · `tracing_core::field::Visit::record_f64` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_f64(&mut self, field: &Field, value: f64)
```

Source: `src/field.rs:286`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a double-precision floating point value.

<a id="op-829ebb8e58441c9368170cfc"></a>
## record_i128

`function` · `tracing_core::field::Visit::record_i128` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_i128(&mut self, field: &Field, value: i128)
```

Source: `src/field.rs:301`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a signed 128-bit integer value.

<a id="op-8c4b4b0f86bba584dacbf3b6"></a>
## record_i64

`function` · `tracing_core::field::Visit::record_i64` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_i64(&mut self, field: &Field, value: i64)
```

Source: `src/field.rs:291`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a signed 64-bit integer value.

<a id="op-a46506577e0960db54efc7d7"></a>
## record_str

`function` · `tracing_core::field::Visit::record_str` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Source: `src/field.rs:316`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit a string value.

<a id="op-feca57e46a3834751dbe39e7"></a>
## record_u128

`function` · `tracing_core::field::Visit::record_u128` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_u128(&mut self, field: &Field, value: u128)
```

Source: `src/field.rs:306`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit an unsigned 128-bit integer value.

<a id="op-1dd94a863373b05e708e394f"></a>
## record_u64

`function` · `tracing_core::field::Visit::record_u64` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_u64(&mut self, field: &Field, value: u64)
```

Source: `src/field.rs:296`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visit an unsigned 64-bit integer value.

<a id="op-e972cfbab7dc71a36f9118c2"></a>
## record_value

`function` · `tracing_core::field::Visit::record_value` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_value(&mut self, field: &Field, value: valuable::Value<'_>)
```

Source: `src/field.rs:281`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visits an arbitrary type implementing the [`valuable`] crate's `Valuable` trait.

[`valuable`]: https://docs.rs/valuable
