# `tracing_subscriber::filter::env::builder::Builder`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.env.builder.Builder.json).

<a id="op-d59a45bc98f2e51d8e269e3b"></a>
## Builder

`struct` · `tracing_subscriber::filter::env::builder::Builder` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Builder
```

Source: `src/filter/env/builder.rs:20`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [builder] for constructing new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055)s.

[builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html

<a id="op-9969da495c5469db8e796dae"></a>
## clone

`function` · `tracing_subscriber::filter::env::builder::Builder::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Builder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 17], "end": [18, 22], "filename": "src/filter/env/builder.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/env/builder.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e504533580c390e6410fd420"></a>
## default

`function` · `tracing_subscriber::filter::env::builder::Builder::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [348, 1], "end": [356, 2], "filename": "src/filter/env/builder.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/filter/env/builder.rs:349`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0a5d2ee7e4f681b519e7454e"></a>
## fmt

`function` · `tracing_subscriber::filter::env::builder::Builder::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 10], "end": [18, 15], "filename": "src/filter/env/builder.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/env/builder.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fa26a7cfc323f8f71dc9e048"></a>
## from_env

`function` · `tracing_subscriber::filter::env::builder::Builder::from_env` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_env(&self) -> Result<EnvFilter, FromEnvError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:202`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) from the directives in the configured
environment variable. If the environment variable is unset, no directive is added.

An error is returned if the environment contains invalid directives.

If the environment variable is empty, then the [default directive]
is used instead.

[default directive]: Self::with_default_directive

<a id="op-8f25752867985d0243075944"></a>
## from_env_lossy

`function` · `tracing_subscriber::filter::env::builder::Builder::from_env_lossy` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_env_lossy(&self) -> EnvFilter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:188`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) from the directives in the configured
environment variable, ignoring any directives that are invalid.

If the environment variable is empty, then the [default directive]
is used instead.

[default directive]: Self::with_default_directive

<a id="op-6f7dd0ba44a0a779ac0336c5"></a>
## parse

`function` · `tracing_subscriber::filter::env::builder::Builder::parse` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn parse<S: AsRef<str>>(&self, dirs: S) -> Result<EnvFilter, filter::directive::ParseError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:168`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) from the directives in the given string,
or an error if any are invalid.

If `parse` is called with an empty string, then the [default directive]
is used instead.

[default directive]: Self::with_default_directive

<a id="op-f32ee2b02d34fdb66d81f904"></a>
## parse_lossy

`function` · `tracing_subscriber::filter::env::builder::Builder::parse_lossy` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn parse_lossy<S: AsRef<str>>(&self, dirs: S) -> EnvFilter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:146`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) from the directives in the given string,
*ignoring* any that are invalid.

If `parse_lossy` is called with an empty string, then the
[default directive] is used instead.

[default directive]: Self::with_default_directive

<a id="op-6eb065f6605791951e23ffd3"></a>
## try_from_env

`function` · `tracing_subscriber::filter::env::builder::Builder::try_from_env` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_from_env(&self) -> Result<EnvFilter, FromEnvError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:215`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) from the directives in the configured
environment variable, or an error if the environment variable is not set
or contains invalid directives.

If the environment variable is empty, then the [default directive]
is used instead.

[default directive]: Self::with_default_directive

<a id="op-ecdca7bc178281024ddccd56"></a>
## with_default_directive

`function` · `tracing_subscriber::filter::env::builder::Builder::with_default_directive` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_default_directive(self, default_directive: Directive) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:116`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets a default [filtering directive] that will be added to the filter if
the parsed string or environment variable contains no filter directives.

By default, there is no default directive.

# Examples

If [`parse`], [`parse_lossy`], [`from_env`], or [`from_env_lossy`] are
called with an empty string or environment variable, the default
directive is used instead:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .parse("")?;

assert_eq!(format!("{}", filter), "info");
# Ok(()) }
```

Note that the `lossy` variants ([`parse_lossy`] and [`from_env_lossy`])
will ignore any invalid directives. If all directives in a filter
string or environment variable are invalid, those methods will also use
the default directive:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .parse_lossy("some_target=fake level,foo::bar=lolwut");

assert_eq!(format!("{}", filter), "info");
```


If the string or environment variable contains valid filtering
directives, the default directive is not used:

```rust
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .parse_lossy("foo=trace");

// The default directive is *not* used:
assert_eq!(format!("{}", filter), "foo=trace");
```

Parsing a more complex default directive from a string:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

let default = "myapp=debug".parse()
    .expect("hard-coded default directive should be valid");

let filter = EnvFilter::builder()
    .with_default_directive(default)
    .parse("")?;

assert_eq!(format!("{}", filter), "myapp=debug");
# Ok(()) }
```

[`parse_lossy`]: Self::parse_lossy
[`from_env_lossy`]: Self::from_env_lossy
[`parse`]: Self::parse
[`from_env`]: Self::from_env

<a id="op-33b9a454ce04883d7afad30c"></a>
## with_env_var

`function` · `tracing_subscriber::filter::env::builder::Builder::with_env_var` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_env_var(self, var: impl ToString) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:132`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the name of the environment variable used by the [`from_env`],
[`from_env_lossy`], and [`try_from_env`] methods.

By default, this is the value of [`EnvFilter::DEFAULT_ENV`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f1696e92d214bc019af86519)
(`RUST_LOG`).

[`from_env`]: Self::from_env
[`from_env_lossy`]: Self::from_env_lossy
[`try_from_env`]: Self::try_from_env

<a id="op-715f1faf0a14b4dd81617ecf"></a>
## with_regex

`function` · `tracing_subscriber::filter::env::builder::Builder::with_regex` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_regex(self, regex: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::builder::Builder", "path": "Builder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 1], "end": [346, 2], "filename": "src/filter/env/builder.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/env/builder.rs:38`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether span field values can be matched with regular expressions.

If this is `true`, field filter directives will be interpreted as
regular expressions if they are not able to be interpreted as a `bool`,
`i64`, `u64`, or `f64` literal. If this is `false,` those field values
will be interpreted as literal [`std::fmt::Debug`] output instead.

By default, regular expressions are enabled.

**Note**: when [`EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055)s are constructed from untrusted inputs,
disabling regular expressions is strongly encouraged.

Unresolved upstream links (retained, not inferred): ``std::fmt::Debug``.
