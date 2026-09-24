# `tracing_subscriber::filter::targets::Targets`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.targets.Targets.json).

<a id="op-641e1a5b3033eaed5a14d773"></a>
## Targets

`struct` · `tracing_subscriber::filter::targets::Targets` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Targets
```

Source: `src/filter/targets.rs:172`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

 A filter that enables or disables spans and events based on their [target]
 and [level].

 Targets are typically equal to the Rust module path of the code where the
 span or event was recorded, although they may be overridden.

 This type can be used for both [per-layer filtering][plf] (using its
 [`Filter`] implementation) and [global filtering][global] (using its
 [`Layer`] implementation).

 See the [documentation on filtering with layers][filtering] for details.

 # Filtering With `Targets`

 A `Targets` filter consists of one or more [target] prefixes, paired with
 [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98)s. If a span or event's [target] begins with one of those
 prefixes, and its [level] is at or below the [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98) enabled for
 that prefix, then the span or event will be enabled.

 This is similar to the behavior implemented by the [`env_logger` crate] in
 the `log` ecosystem.

 The [`EnvFilter`] type also provided by this crate is very similar to `Targets`,
 but is capable of a more sophisticated form of filtering where events may
 also be enabled or disabled based on the span they are recorded in.
 `Targets` can be thought of as a lighter-weight form of [`EnvFilter`] that
 can be used instead when this dynamic filtering is not required.

 # Examples

 A `Targets` filter can be constructed by programmatically adding targets and
 levels to enable:

 ```
 use tracing_subscriber::{filter, prelude::*};
 use tracing_core::Level;

 let filter = filter::Targets::new()
     // Enable the `INFO` level for anything in `my_crate`
     .with_target("my_crate", Level::INFO)
     // Enable the `DEBUG` level for a specific module.
     .with_target("my_crate::interesting_module", Level::DEBUG);

 // Build a new subscriber with the `fmt` layer using the `Targets`
 // filter we constructed above.
 tracing_subscriber::registry()
     .with(tracing_subscriber::fmt::layer())
     .with(filter)
     .init();
 ```

 [`LevelFilter::OFF`] can be used to disable a particular target:
 ```
 use tracing_subscriber::filter::{Targets, LevelFilter};
 use tracing_core::Level;

 let filter = Targets::new()
     .with_target("my_crate", Level::INFO)
     // Disable all traces from `annoying_module`.
     .with_target("my_crate::annoying_module", LevelFilter::OFF);
 # drop(filter);
 ```

 Alternatively, `Targets` implements [`std::str::FromStr`], allowing it to be
 parsed from a comma-delimited list of `target=level` pairs. For example:

 ```rust
 # fn main() -> Result<(), Box<dyn std::error::Error>> {
 use tracing_subscriber::filter;
 use tracing_core::Level;

 let filter = "my_crate=info,my_crate::interesting_module=trace,other_crate=debug"
     .parse::<filter::Targets>()?;

 // The parsed filter is identical to a filter constructed using `with_target`:
 assert_eq!(
     filter,
     filter::Targets::new()
         .with_target("my_crate", Level::INFO)
         .with_target("my_crate::interesting_module", Level::TRACE)
         .with_target("other_crate", Level::DEBUG)
 );
 # Ok(()) }
 ```

 This is particularly useful when the list of enabled targets is configurable
 by the user at runtime.

 The `Targets` filter can be used as a [per-layer filter][plf] *and* as a
 [global filter][global]:

 ```rust
 use tracing_subscriber::{
     fmt,
     filter::{Targets, LevelFilter},
     prelude::*,
 };
 use tracing_core::Level;
 use std::{sync::Arc, fs::File};
 # fn docs() -> Result<(), Box<dyn std::error::Error>> {

 // A layer that logs events to stdout using the human-readable "pretty"
 // format.
 let stdout_log = fmt::layer().pretty();

 // A layer that logs events to a file, using the JSON format.
 let file = File::create("debug_log.json")?;
 let debug_log = fmt::layer()
     .with_writer(Arc::new(file))
     .json();

 tracing_subscriber::registry()
     // Only log INFO and above to stdout, unless the span or event
     // has the `my_crate::cool_module` target prefix.
     .with(stdout_log
         .with_filter(
             Targets::default()
                 .with_target("my_crate::cool_module", Level::DEBUG)
                 .with_default(Level::INFO)
        )
     )
     // Log everything enabled by the global filter to `debug_log.json`.
     .with(debug_log)
     // Configure a global filter for the whole subscriber stack. This will
     // control what spans and events are recorded by both the `debug_log`
     // and the `stdout_log` layers, and `stdout_log` will *additionally* be
     // filtered by its per-layer filter.
     .with(
         Targets::default()
             .with_target("my_crate", Level::TRACE)
             .with_target("other_crate", Level::INFO)
             .with_target("other_crate::annoying_module", LevelFilter::OFF)
             .with_target("third_crate", Level::DEBUG)
     ).init();
 # Ok(()) }
```

 [target]: tracing_core::Metadata::target
 [level]: tracing_core::Level
 [`Filter`]: crate::layer::Filter
 [`Layer`]: crate::layer::Layer
 [plf]: crate::layer#per-layer-filtering
 [global]: crate::layer#global-filtering
 [filtering]: crate::layer#filtering-with-layers
 [`env_logger` crate]: https://docs.rs/env_logger/0.9.0/env_logger/index.html#enabling-logging
 [`EnvFilter`]: crate::filter::EnvFilter

Unresolved upstream links (retained, not inferred): ``LevelFilter::OFF``, ``std::str::FromStr``, `tracing_core::Metadata::target`.

<a id="op-3143587044be34fd8474b059"></a>
## Err

`assoc_type` · `tracing_subscriber::filter::targets::Targets::Err` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Err
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [428, 1], "end": [436, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/filter/targets.rs:429`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e46ade10e756e779c6afd71f"></a>
## IntoIter

`assoc_type` · `tracing_subscriber::filter::targets::Targets::IntoIter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
IntoIter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [471, 1], "end": [479, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}, "trait_path": "core::iter::traits::collect::IntoIterator"}`

Source: `src/filter/targets.rs:474`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-56f53ebd9dadba59a844ebd5"></a>
## Item

`assoc_type` · `tracing_subscriber::filter::targets::Targets::Item` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [471, 1], "end": [479, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}, "trait_path": "core::iter::traits::collect::IntoIterator"}`

Source: `src/filter/targets.rs:472`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0553222884babb86aeb291e2"></a>
## callsite_enabled

`function` · `tracing_subscriber::filter::targets::Targets::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [457, 1], "end": [469, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/targets.rs:462`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6c7ae2d5018e79729426ae8e"></a>
## clone

`function` · `tracing_subscriber::filter::targets::Targets::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Targets
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 26], "end": [171, 31], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/targets.rs:171`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2b47b6180e9c6cfaf12f7fb0"></a>
## default

`function` · `tracing_subscriber::filter::targets::Targets::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Targets
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 17], "end": [171, 24], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/filter/targets.rs:171`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1f1994e5f236223fe9e037e7"></a>
## default_level

`function` · `tracing_subscriber::filter::targets::Targets::default_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default_level(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:326`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the default level for this filter, if one is set.

The default level is used to filter any spans or events with targets
that do not match any of the configured set of prefixes.

The default level can be set for a filter either by using
[`with_default`](Self::with_default) or when parsing from a filter string that includes a
level without a target (e.g. `"trace"`).

# Examples

```
use tracing_subscriber::filter::{LevelFilter, Targets};

let filter = Targets::new().with_default(LevelFilter::INFO);
assert_eq!(filter.default_level(), Some(LevelFilter::INFO));

let filter: Targets = "info".parse().unwrap();
assert_eq!(filter.default_level(), Some(LevelFilter::INFO));
```

The default level is `None` if no default is set:

```
use tracing_subscriber::filter::Targets;

let filter = Targets::new();
assert_eq!(filter.default_level(), None);

let filter: Targets = "my_crate=info".parse().unwrap();
assert_eq!(filter.default_level(), None);
```

Note that an unset default level (`None`) behaves like [`LevelFilter::OFF`] when the filter is
used, but it could also be set explicitly which may be useful to distinguish (such as when
merging multiple `Targets`).

```
use tracing_subscriber::filter::{LevelFilter, Targets};

let filter = Targets::new().with_default(LevelFilter::OFF);
assert_eq!(filter.default_level(), Some(LevelFilter::OFF));

let filter: Targets = "off".parse().unwrap();
assert_eq!(filter.default_level(), Some(LevelFilter::OFF));
```

Unresolved upstream links (retained, not inferred): ``LevelFilter::OFF``.

<a id="op-a8a8ced25ccea22cce284a50"></a>
## enabled

`function` · `tracing_subscriber::filter::targets::Targets::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, _: &layer::Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [457, 1], "end": [469, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/targets.rs:458`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f4bae6bfc9c7c2f0228eb361"></a>
## enabled

`function` · `tracing_subscriber::filter::targets::Targets::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, _: layer::Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [438, 1], "end": [453, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/targets.rs:442`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9a153c7ff8a2fb232e5aa2d8"></a>
## eq

`function` · `tracing_subscriber::filter::targets::Targets::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Targets) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 33], "end": [171, 42], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/filter/targets.rs:171`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d4abaa19f21027fdad44f2fb"></a>
## extend

`function` · `tracing_subscriber::filter::targets::Targets::extend` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extend<I: IntoIterator<Item = (T, L)>>(&mut self, iter: I)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "alloc::string::String"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "crate::filter::LevelFilter"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [403, 1], "end": [414, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": [{"generic": "T"}, {"generic": "L"}]}}], "constraints": []}}, "id": "core::iter::traits::collect::Extend", "path": "Extend"}, "trait_path": "core::iter::traits::collect::Extend"}`

Source: `src/filter/targets.rs:408`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-124e840b1b7e6a616d1fccc2"></a>
## fmt

`function` · `tracing_subscriber::filter::targets::Targets::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [491, 1], "end": [503, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/filter/targets.rs:492`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-626a3677a4cc11431a1b8288"></a>
## fmt

`function` · `tracing_subscriber::filter::targets::Targets::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 10], "end": [171, 15], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/targets.rs:171`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0617b81f8785678f656c1a0f"></a>
## from_iter

`function` · `tracing_subscriber::filter::targets::Targets::from_iter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_iter<I: IntoIterator<Item = (T, L)>>(iter: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "alloc::string::String"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "crate::filter::LevelFilter"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [416, 1], "end": [426, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": [{"generic": "T"}, {"generic": "L"}]}}], "constraints": []}}, "id": "core::iter::traits::collect::FromIterator", "path": "FromIterator"}, "trait_path": "core::iter::traits::collect::FromIterator"}`

Source: `src/filter/targets.rs:421`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ba15d0f583767edaf4947eff"></a>
## from_str

`function` · `tracing_subscriber::filter::targets::Targets::from_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_str(s: &str) -> Result<Self, Self::Err>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [428, 1], "end": [436, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/filter/targets.rs:430`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8332b29b1fc2c46df5d1c20c"></a>
## into_iter

`function` · `tracing_subscriber::filter::targets::Targets::into_iter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn into_iter(self) -> Self::IntoIter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [471, 1], "end": [479, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}, "trait_path": "core::iter::traits::collect::IntoIterator"}`

Source: `src/filter/targets.rs:476`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-34012252f643521ea6388131"></a>
## iter

`function` · `tracing_subscriber::filter::targets::Targets::iter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn iter(&self) -> Iter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:360`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an iterator over the [target]-[`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98) pairs in this filter.

The order of iteration is undefined.

# Examples

```
use tracing_subscriber::filter::{Targets, LevelFilter};
use tracing_core::Level;

let filter = Targets::new()
    .with_target("my_crate", Level::INFO)
    .with_target("my_crate::interesting_module", Level::DEBUG);

let mut targets: Vec<_> = filter.iter().collect();
targets.sort();

assert_eq!(targets, vec![
    ("my_crate", LevelFilter::INFO),
    ("my_crate::interesting_module", LevelFilter::DEBUG),
]);
```

[target]: tracing_core::Metadata::target

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::target`.

<a id="op-a1153b655767001358373316"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::targets::Targets::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [457, 1], "end": [469, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/targets.rs:466`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-104a2122f8620d54fd1d6364"></a>
## new

`function` · `tracing_subscriber::filter::targets::Targets::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:184`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `Targets` filter.

This filter will enable no targets. Call [`with_target`] or [`with_targets`]
to add enabled targets, and [`with_default`] to change the default level
enabled for spans and events that didn't match any of the provided targets.

[`with_target`]: Targets::with_target
[`with_targets`]: Targets::with_targets
[`with_default`]: Targets::with_default

<a id="op-e3142809aba0c07e3a4c8b25"></a>
## register_callsite

`function` · `tracing_subscriber::filter::targets::Targets::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [438, 1], "end": [453, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/targets.rs:446`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fd131d09f5ac8e5cf64732b3"></a>
## with_default

`function` · `tracing_subscriber::filter::targets::Targets::with_default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_default(self, level: impl Into<LevelFilter>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:274`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the default level to enable for spans and events whose targets did
not match any of the configured prefixes.

By default, this is [`LevelFilter::OFF`]. This means that spans and
events will only be enabled if they match one of the configured target
prefixes. If this is changed to a different [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98), spans and
events with targets that did not match any of the configured prefixes
will be enabled if their level is at or below the provided level.

Unresolved upstream links (retained, not inferred): ``LevelFilter::OFF``.

<a id="op-e0cf17aec9cce99753d8dbdf"></a>
## with_target

`function` · `tracing_subscriber::filter::targets::Targets::with_target` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_target(self, target: impl Into<String>, level: impl Into<LevelFilter>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:218`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Enables spans and events with [target]s starting with the provided target
prefix if they are at or below the provided [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98).

# Examples

```
use tracing_subscriber::filter;
use tracing_core::Level;

let filter = filter::Targets::new()
    // Enable the `INFO` level for anything in `my_crate`
    .with_target("my_crate", Level::INFO)
    // Enable the `DEBUG` level for a specific module.
    .with_target("my_crate::interesting_module", Level::DEBUG);
# drop(filter);
```

[`LevelFilter::OFF`] can be used to disable a particular target:
```
use tracing_subscriber::filter::{Targets, LevelFilter};
use tracing_core::Level;

let filter = Targets::new()
    .with_target("my_crate", Level::INFO)
    // Disable all traces from `annoying_module`.
    .with_target("my_crate::interesting_module", LevelFilter::OFF);
# drop(filter);
```

[target]: tracing_core::Metadata::target

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::target`, ``LevelFilter::OFF``.

<a id="op-d5bd65760c6d9aef583dcb63"></a>
## with_targets

`function` · `tracing_subscriber::filter::targets::Targets::with_targets` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_targets<T, L>(self, targets: impl IntoIterator<Item = (T, L)>) -> Self where String: From<T>, LevelFilter: From<L>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:257`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Adds [target]s from an iterator of [target]-[`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98) pairs to this filter.

# Examples

```
use tracing_subscriber::filter;
use tracing_core::Level;

let filter = filter::Targets::new()
    .with_targets(vec![
        ("my_crate", Level::INFO),
        ("my_crate::some_module", Level::DEBUG),
        ("my_crate::other_module::cool_stuff", Level::TRACE),
        ("other_crate", Level::WARN)
    ]);
# drop(filter);
```

[`LevelFilter::OFF`] can be used to disable a particular target:
```
use tracing_subscriber::filter::{Targets, LevelFilter};
use tracing_core::Level;

let filter = Targets::new()
    .with_target("my_crate", Level::INFO)
    // Disable all traces from `annoying_module`.
    .with_target("my_crate::interesting_module", LevelFilter::OFF);
# drop(filter);
```

[target]: tracing_core::Metadata::target

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::target`, ``LevelFilter::OFF``.

<a id="op-055f2975d042cfde7e5df394"></a>
## would_enable

`function` · `tracing_subscriber::filter::targets::Targets::would_enable` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn would_enable(&self, target: &str, level: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::targets::Targets", "path": "Targets"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [174, 1], "end": [401, 2], "filename": "src/filter/targets.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/targets.rs:396`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns whether a [target]-[`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff) pair would be enabled
by this `Targets`.

This method can be used with [`module_path!`] from `std` as the target
in order to emulate the behavior of the [`tracing::event!`](../operations/tracing.event.md#op-c45070aa88b3630e47ea433f) and [`tracing::span!`](../operations/tracing.span.md#op-d37a648f25beedf59548b980)
macros.

# Examples

```
use tracing_subscriber::filter::{Targets, LevelFilter};
use tracing_core::Level;

let filter = Targets::new()
    .with_target("my_crate", Level::INFO)
    .with_target("my_crate::interesting_module", Level::DEBUG);

assert!(filter.would_enable("my_crate", &Level::INFO));
assert!(!filter.would_enable("my_crate::interesting_module", &Level::TRACE));
```

[target]: tracing_core::Metadata::target
[`module_path!`]: std::module_path!

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::target`, `std::module_path!`.
