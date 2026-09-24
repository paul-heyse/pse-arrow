# `tracing_subscriber::filter::env::EnvFilter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.env.EnvFilter.json).

<a id="op-f18b1a3b867d9bc485e30055"></a>
## EnvFilter

`struct` · `tracing_subscriber::filter::env::EnvFilter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct EnvFilter
```

Source: `src/filter/env/mod.rs:199`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) which filters spans and events based on a set of filter
directives.

`EnvFilter` implements both the [`Layer`](#impl-Layer<S>) and [`Filter`] traits, so it may
be used for both [global filtering][global] and [per-layer filtering][plf],
respectively. See [the documentation on filtering with `Layer`s][filtering]
for details.

The [`Targets`] type implements a similar form of filtering, but without the
ability to dynamically enable events based on the current span context, and
without filtering on field values. When these features are not required,
[`Targets`] provides a lighter-weight alternative to [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055).

# Directives

A filter consists of one or more comma-separated directives which match on [`Span`]s and [`Event`]s.
Each directive may have a corresponding maximum verbosity [`level`] which
enables (e.g., _selects for_) spans and events that match. Like `log`,
`tracing` considers less exclusive levels (like `trace` or `info`) to be more
verbose than more exclusive levels (like `error` or `warn`).

The directive syntax is similar to that of [`env_logger`]'s. At a high level, the syntax for directives
consists of several parts:

```text
target[span{field=value}]=level
```

Each component (`target`, `span`, `field`, `value`, and `level`) will be covered in turn.

- `target` matches the event or span's target. In general, this is the module path and/or crate name.
  Examples of targets `h2`, `tokio::net`, or `tide::server`. For more information on targets,
  please refer to [`Metadata`]'s documentation.
- `span` matches on the span's name. If a `span` directive is provided alongside a `target`,
  the `span` directive will match on spans _within_ the `target`.
- `field` matches on [fields] within spans. Field names can also be supplied without a `value`
  and will match on any [`Span`] or [`Event`] that has a field with that name.
  For example: `[span{field=\"value\"}]=debug`, `[{field}]=trace`.
- `value` matches on the value of a span's field. If a value is a numeric literal or a bool,
  it will match _only_ on that value. Otherwise, this filter matches the
  [`std::fmt::Debug`] output from the value.
- `level` sets a maximum verbosity level accepted by this directive.

When a field value directive (`[{<FIELD NAME>=<FIELD_VALUE>}]=...`) matches a
value's [`std::fmt::Debug`] output (i.e., the field value in the directive
is not a `bool`, `i64`, `u64`, or `f64` literal), the matched pattern may be
interpreted as either a regular expression or as the precise expected
output of the field's [`std::fmt::Debug`] implementation. By default, these
filters are interpreted as regular expressions, but this can be disabled
using the [`Builder::with_regex`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-715f1faf0a14b4dd81617ecf) builder method to use precise matching
instead.

When field value filters are interpreted as regular expressions, the
[`regex` crate's regular expression syntax][re-syntax] is supported.

**Note**: When filters are constructed from potentially untrusted inputs,
[disabling regular expression matching](Builder::with_regex) is strongly
recommended.

## Usage Notes

- The portion of the directive which is included within the square brackets is `tracing`-specific.
- Any portion of the directive can be omitted.
    - The sole exception are the `field` and `value` directives. If a `value` is provided,
      a `field` must _also_ be provided. However, the converse does not hold, as fields can
      be matched without a value.
- If only a level is provided, it will set the maximum level for all `Span`s and `Event`s
  that are not enabled by other filters.
- A directive without a level will enable anything that it matches. This is equivalent to `=trace`.
- When a crate has a dash in its name, the default target for events will be the
  crate's module path as it appears in Rust. This means every dash will be replaced
  with an underscore.
- A dash in a target will only appear when being specified explicitly:
  `tracing::info!(target: "target-name", ...);`

## Example Syntax

- `tokio::net=info` will enable all spans or events that:
   - have the `tokio::net` target,
   - at the level `info` or above.
- `warn,tokio::net=info` will enable all spans and events that:
   - are at the level `warn` or above, *or*
   - have the `tokio::net` target at the level `info` or above.
- `my_crate[span_a]=trace` will enable all spans and events that:
   - are within the `span_a` span or named `span_a` _if_ `span_a` has the target `my_crate`,
   - at the level `trace` or above.
- `[span_b{name=\"bob\"}]` will enable all spans or event that:
   - have _any_ target,
   - are inside a span named `span_b`,
   - which has a field named `name` with value `bob`,
   - at _any_ level.

# Examples

Parsing an `EnvFilter` from the [default environment
variable](EnvFilter::from_default_env) (`RUST_LOG`):

```
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();
```

Parsing an `EnvFilter` [from a user-provided environment
variable](EnvFilter::from_env):

```
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_env("MYAPP_LOG"))
    .init();
```

Using `EnvFilter` as a [per-layer filter][plf] to filter only a single
[`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8):

```
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

// Parse an `EnvFilter` configuration from the `RUST_LOG`
// environment variable.
let filter = EnvFilter::from_default_env();

// Apply the filter to this layer *only*.
let filtered_layer = fmt::layer().with_filter(filter);

// Some other layer, whose output we don't want to filter.
let unfiltered_layer = // ...
    # fmt::layer();

tracing_subscriber::registry()
    .with(filtered_layer)
    .with(unfiltered_layer)
    .init();
```
# Constructing `EnvFilter`s

An `EnvFilter` is be constructed by parsing a string containing one or more
directives. The [`EnvFilter::new`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f8fe00fcf0fcfd1748747b50) constructor parses an `EnvFilter` from a
string, ignoring any invalid directives, while [`EnvFilter::try_new`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-a958def5822864286cdf0e87)
returns an error if invalid directives are encountered. Similarly, the
[`EnvFilter::from_env`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-b4a50fc9e41d55f2cea17a21) and [`EnvFilter::try_from_env`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-1d20a00019acacef029303e7) constructors parse
an `EnvFilter` from the value of the provided environment variable, with
lossy and strict validation, respectively.

A [builder](EnvFilter::builder) interface is available to set additional
configuration options prior to parsing an `EnvFilter`. See the [`Builder`
type's documentation](Builder) for details on the options that can be
configured using the builder.

[`Span`]: tracing_core::span
[fields]: tracing_core::Field
[`Event`]: tracing_core::Event
[`level`]: tracing_core::Level
[`Metadata`]: tracing_core::Metadata
[`Targets`]: crate::filter::Targets
[`env_logger`]: https://crates.io/crates/env_logger
[`Filter`]: #impl-Filter<S>
[global]: crate::layer#global-filtering
[plf]: crate::layer#per-layer-filtering
[filtering]: crate::layer#filtering-with-layers
[re-syntax]: https://docs.rs/regex/1.11.1/regex/#syntax

Unresolved upstream links (retained, not inferred): ``std::fmt::Debug``.

<a id="op-f1696e92d214bc019af86519"></a>
## DEFAULT_ENV

`assoc_const` · `tracing_subscriber::filter::env::EnvFilter::DEFAULT_ENV` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
DEFAULT_ENV
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:249`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

`RUST_LOG` is the default environment variable used by
[`EnvFilter::from_default_env`] and [`EnvFilter::try_from_default_env`].

[`EnvFilter::from_default_env`]: EnvFilter::from_default_env()
[`EnvFilter::try_from_default_env`]: EnvFilter::try_from_default_env()

<a id="op-466af3a86fc0332f4478b234"></a>
## Err

`assoc_type` · `tracing_subscriber::filter::env::EnvFilter::Err` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Err
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [751, 1], "end": [757, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/filter/env/mod.rs:752`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fe308c841ff7623bbcfeb2e9"></a>
## add_directive

`function` · `tracing_subscriber::filter::env::EnvFilter::add_directive` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn add_directive(self, directive: Directive) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:477`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Add a filtering directive to this `EnvFilter`.

The added directive will be used in addition to any previously set
directives, either added using this method or provided when the filter
is constructed.

Filters may be created from [`LevelFilter`] or [`Level`], which will
enable all traces at or below a certain verbosity level, or
parsed from a string specifying a directive.

If a filter directive is inserted that matches exactly the same spans
and events as a previous filter, but sets a different level for those
spans and events, the previous directive is overwritten.

[`LevelFilter`]: super::LevelFilter
[`Level`]: tracing_core::Level

# Examples

From [`LevelFilter`]:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
let mut filter = EnvFilter::from_default_env()
    .add_directive(LevelFilter::INFO.into());
```

Or from [`Level`]:

```rust
# use tracing_subscriber::filter::{EnvFilter, LevelFilter};
# use tracing::Level;
let mut filter = EnvFilter::from_default_env()
    .add_directive(Level::INFO.into());
```

Parsed from a string:

```rust
use tracing_subscriber::filter::{EnvFilter, Directive};

# fn try_mk_filter() -> Result<(), Box<dyn ::std::error::Error>> {
let mut filter = EnvFilter::try_from_default_env()?
    .add_directive("my_crate::module=trace".parse()?)
    .add_directive("my_crate::my_other_module::something=info".parse()?);
# Ok(())
# }
```
In the above example, substitute `my_crate`, `module`, etc. with the
name your target crate/module is imported with. This might be
different from the package name in Cargo.toml (`-` is replaced by `_`).
Example, if the package name in your Cargo.toml is `MY-FANCY-LIB`, then
the corresponding Rust identifier would be `MY_FANCY_LIB`:

<a id="op-7e282dd836c0979d75e6b3fb"></a>
## builder

`function` · `tracing_subscriber::filter::env::EnvFilter::builder` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> Builder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:262`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a [builder] that can be used to configure a new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055)
instance.

The [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type is used to set additional configurations, such as
[whether regular expressions are enabled](Builder::with_regex) or [the
default directive](Builder::with_default_directive) before parsing an
[`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) from a string or environment variable.

[builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html

<a id="op-654028c803b225aa72313faf"></a>
## callsite_enabled

`function` · `tracing_subscriber::filter::env::EnvFilter::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:715`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8892fbaeb769b5707afab09f"></a>
## clone

`function` · `tracing_subscriber::filter::env::EnvFilter::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> EnvFilter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [213, 1], "end": [225, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/env/mod.rs:214`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4467088beb8c5ef719cc7fc8"></a>
## default

`function` · `tracing_subscriber::filter::env::EnvFilter::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [768, 1], "end": [772, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/filter/env/mod.rs:769`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-85ab1e63f56927023c5bed7a"></a>
## enabled

`function` · `tracing_subscriber::filter::env::EnvFilter::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled<S>(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:498`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this `EnvFilter` would enable the provided `metadata`
in the current context.

This is equivalent to calling the [`Layer::enabled`](../operations/tracing_subscriber.layer.Layer.md#op-bf52d490c84266759988e51e) or
[`Filter::enabled`](../operations/tracing_subscriber.layer.Filter.md#op-104b1d844775b41b079882e6) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope.

<a id="op-a437b8626d6ed19cb5d00bfe"></a>
## enabled

`function` · `tracing_subscriber::filter::env::EnvFilter::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:674`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c71b8a9a53acd0bc5ccd7625"></a>
## enabled

`function` · `tracing_subscriber::filter::env::EnvFilter::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, meta: &Metadata<'_>, ctx: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:710`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6e5ded8dacb9f13438ecdb29"></a>
## fmt

`function` · `tracing_subscriber::filter::env::EnvFilter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [198, 10], "end": [198, 15], "filename": "src/filter/env/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/env/mod.rs:198`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f3bc1a3e7d6b59394ad52138"></a>
## fmt

`function` · `tracing_subscriber::filter::env::EnvFilter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [774, 1], "end": [799, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/filter/env/mod.rs:775`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-245abe4262968a427bcfef2b"></a>
## from

`function` · `tracing_subscriber::filter::env::EnvFilter::from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [759, 1], "end": [766, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/filter/env/mod.rs:763`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-34d51594cc4efd6155835835"></a>
## from_default_env

`function` · `tracing_subscriber::filter::env::EnvFilter::from_default_env` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_default_env() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:289`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `EnvFilter` from the value of the `RUST_LOG` environment
variable, ignoring any invalid filter directives.

If the environment variable is empty or not set, or if it contains only
invalid directives, a default directive enabling the [`ERROR`] level is
added.

To set additional configuration options prior to parsing the filter, use
the [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type instead.

This function is equivalent to the following:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

# fn docs() -> EnvFilter {
EnvFilter::builder()
    .with_default_directive(LevelFilter::ERROR.into())
    .from_env_lossy()
# }
```

[`ERROR`]: tracing::Level::ERROR

Unresolved upstream links (retained, not inferred): `tracing::Level::ERROR`.

<a id="op-b4a50fc9e41d55f2cea17a21"></a>
## from_env

`function` · `tracing_subscriber::filter::env::EnvFilter::from_env` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_env<A: AsRef<str>>(env: A) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:320`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `EnvFilter` from the value of the given environment
variable, ignoring any invalid filter directives.

If the environment variable is empty or not set, or if it contains only
invalid directives, a default directive enabling the [`ERROR`] level is
added.

To set additional configuration options prior to parsing the filter, use
the [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type instead.

This function is equivalent to the following:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

# fn docs() -> EnvFilter {
# let env = "";
EnvFilter::builder()
    .with_default_directive(LevelFilter::ERROR.into())
    .with_env_var(env)
    .from_env_lossy()
# }
```

[`ERROR`]: tracing::Level::ERROR

Unresolved upstream links (retained, not inferred): `tracing::Level::ERROR`.

<a id="op-ceb4e22121553cc39af3c67b"></a>
## from_str

`function` · `tracing_subscriber::filter::env::EnvFilter::from_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_str(spec: &str) -> Result<Self, Self::Err>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [751, 1], "end": [757, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/filter/env/mod.rs:754`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b31a315ca1f32d8198b5bdab"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::env::EnvFilter::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:550`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an optional hint of the highest [verbosity level][level] that
this `EnvFilter` will enable.

This is equivalent to calling the [`Layer::max_level_hint`] or
[`Filter::max_level_hint`](../operations/tracing_subscriber.layer.Filter.md#op-2e42e1baab0662273a9f7d60) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope.

[level]: tracing_core::metadata::Level

Unresolved upstream links (retained, not inferred): ``Layer::max_level_hint``.

<a id="op-e1122c67c83e934da0456be0"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::env::EnvFilter::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:720`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f8fe00fcf0fcfd1748747b50"></a>
## new

`function` · `tracing_subscriber::filter::env::EnvFilter::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new<S: AsRef<str>>(directives: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:350`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `EnvFilter` from the directives in the given string,
ignoring any that are invalid.

If the string is empty or contains only invalid directives, a default
directive enabling the [`ERROR`] level is added.

To set additional configuration options prior to parsing the filter, use
the [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type instead.

This function is equivalent to the following:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

# fn docs() -> EnvFilter {
# let directives = "";
EnvFilter::builder()
    .with_default_directive(LevelFilter::ERROR.into())
    .parse_lossy(directives)
# }
```

[`ERROR`]: tracing::Level::ERROR

Unresolved upstream links (retained, not inferred): `tracing::Level::ERROR`.

<a id="op-36d5d8d58f77a4c542f70f69"></a>
## on_close

`function` · `tracing_subscriber::filter::env::EnvFilter::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:745`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-acd3f349f46aef6e145532c3"></a>
## on_close

`function` · `tracing_subscriber::filter::env::EnvFilter::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:699`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dd0a73fb74b56089af87cd0d"></a>
## on_close

`function` · `tracing_subscriber::filter::env::EnvFilter::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close<S>(&self, id: span::Id, _: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:606`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Informs the filter that the span with the provided `id` was closed.

This is equivalent to calling the [`Layer::on_close`](../operations/tracing_subscriber.layer.Layer.md#op-0ec2d3e1932d45768843f8f4) or
[`Filter::on_close`](../operations/tracing_subscriber.layer.Filter.md#op-04a5709a0659b887ca6e9d0b) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope.

<a id="op-441c6285af17e467de783aa4"></a>
## on_enter

`function` · `tracing_subscriber::filter::env::EnvFilter::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter<S>(&self, id: &span::Id, _: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:581`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Informs the filter that the span with the provided `id` was entered.

This is equivalent to calling the [`Layer::on_enter`](../operations/tracing_subscriber.layer.Layer.md#op-a351326be900db143e10528e) or
[`Filter::on_enter`](../operations/tracing_subscriber.layer.Filter.md#op-2d18e530aa5ae59d5aeaebf7) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope.

<a id="op-b7860848731532ff36c1325e"></a>
## on_enter

`function` · `tracing_subscriber::filter::env::EnvFilter::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:689`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f08ea3f00863aad4fab47a91"></a>
## on_enter

`function` · `tracing_subscriber::filter::env::EnvFilter::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:735`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-20f29fbd312ecd3f98899442"></a>
## on_exit

`function` · `tracing_subscriber::filter::env::EnvFilter::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit<S>(&self, id: &span::Id, _: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:595`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Informs the filter that the span with the provided `id` was exited.

This is equivalent to calling the [`Layer::on_exit`](../operations/tracing_subscriber.layer.Layer.md#op-5895c9c1e516a18c3eefe5ad) or
[`Filter::on_exit`](../operations/tracing_subscriber.layer.Filter.md#op-1d19fba59da48e45193b7aef) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope.

<a id="op-405de59c72eb28f66f79f65d"></a>
## on_exit

`function` · `tracing_subscriber::filter::env::EnvFilter::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:740`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-be581e409252272aec2ffb0f"></a>
## on_exit

`function` · `tracing_subscriber::filter::env::EnvFilter::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:694`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2548514250d2d8f01b0d4f26"></a>
## on_new_span

`function` · `tracing_subscriber::filter::env::EnvFilter::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:679`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-401fe221b83dccaedc0653d6"></a>
## on_new_span

`function` · `tracing_subscriber::filter::env::EnvFilter::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span<S>(&self, attrs: &span::Attributes<'_>, id: &span::Id, _: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:568`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Informs the filter that a new span was created.

This is equivalent to calling the [`Layer::on_new_span`](../operations/tracing_subscriber.layer.Layer.md#op-501bbdafad438573cae702ef) or
[`Filter::on_new_span`](../operations/tracing_subscriber.layer.Filter.md#op-da649213da0e43a5b12dee9c) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope.

<a id="op-7f9b12edb0ecaced71b8e2df"></a>
## on_new_span

`function` · `tracing_subscriber::filter::env::EnvFilter::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:725`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-39b9f1c30c55eb5343ffe2cd"></a>
## on_record

`function` · `tracing_subscriber::filter::env::EnvFilter::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:684`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5bc7c5f061124ac3d2f23137"></a>
## on_record

`function` · `tracing_subscriber::filter::env::EnvFilter::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [708, 5], "end": [748, 6], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/env/mod.rs:730`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ca3beb68286b73de2b730718"></a>
## on_record

`function` · `tracing_subscriber::filter::env::EnvFilter::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record<S>(&self, id: &span::Id, values: &span::Record<'_>, _: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:622`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Informs the filter that the span with the provided `id` recorded the
provided field `values`.

This is equivalent to calling the [`Layer::on_record`](../operations/tracing_subscriber.layer.Layer.md#op-de23a21f9d492196ee6efda9) or
[`Filter::on_record`](../operations/tracing_subscriber.layer.Filter.md#op-99b67e470fd5268737377d5b) methods on `EnvFilter`'s implementations of those
traits, but it does not require the trait to be in scope

<a id="op-4b2488da7f2edf64b1248c47"></a>
## register_callsite

`function` · `tracing_subscriber::filter::env::EnvFilter::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [702, 2], "filename": "src/filter/env/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/env/mod.rs:664`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-97789bc752e8523b506172a1"></a>
## try_from_default_env

`function` · `tracing_subscriber::filter::env::EnvFilter::try_from_default_env` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_from_default_env() -> Result<Self, FromEnvError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:399`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `EnvFilter` from the value of the `RUST_LOG` environment
variable, or an error if the environment variable is unset or contains
any invalid filter directives.

To set additional configuration options prior to parsing the filter, use
the [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type instead.

This function is equivalent to the following:

```rust
use tracing_subscriber::EnvFilter;

# fn docs() -> Result<EnvFilter, tracing_subscriber::filter::FromEnvError> {
EnvFilter::builder().try_from_env()
# }
```

<a id="op-1d20a00019acacef029303e7"></a>
## try_from_env

`function` · `tracing_subscriber::filter::env::EnvFilter::try_from_env` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_from_env<A: AsRef<str>>(env: A) -> Result<Self, FromEnvError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:420`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `EnvFilter` from the value of the given environment
variable, or an error if the environment variable is unset or contains
any invalid filter directives.

To set additional configuration options prior to parsing the filter, use
the [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type instead.

This function is equivalent to the following:

```rust
use tracing_subscriber::EnvFilter;

# fn docs() -> Result<EnvFilter, tracing_subscriber::filter::FromEnvError> {
# let env = "";
EnvFilter::builder().with_env_var(env).try_from_env()
# }
```

<a id="op-a958def5822864286cdf0e87"></a>
## try_new

`function` · `tracing_subscriber::filter::env::EnvFilter::try_new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_new<S: AsRef<str>>(dirs: S) -> Result<Self, filter::directive::ParseError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "EnvFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [243, 1], "end": [660, 2], "filename": "src/filter/env/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/mod.rs:379`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `EnvFilter` from the directives in the given string,
or an error if any are invalid.

If the string is empty, a default directive enabling the [`ERROR`] level
is added.

To set additional configuration options prior to parsing the filter, use
the [`Builder`](../operations/tracing_subscriber.filter.env.builder.Builder.md#op-d59a45bc98f2e51d8e269e3b) type instead.

This function is equivalent to the following:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

# fn docs() -> Result<EnvFilter, tracing_subscriber::filter::ParseError> {
# let directives = "";
EnvFilter::builder()
    .with_default_directive(LevelFilter::ERROR.into())
    .parse(directives)
# }
```

[`ERROR`]: tracing::Level::ERROR

Unresolved upstream links (retained, not inferred): `tracing::Level::ERROR`.
