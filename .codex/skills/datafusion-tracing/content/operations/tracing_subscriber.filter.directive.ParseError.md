# `tracing_subscriber::filter::directive::ParseError`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.directive.ParseError.json).

<a id="op-0dde5979511052c2780cafbb"></a>
## ParseError

`struct` · `tracing_subscriber::filter::directive::ParseError` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct ParseError
```

Source: `src/filter/directive.rs:10`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Indicates that a string could not be parsed as a filtering directive.

<a id="op-b2f2eed604eb7f647b559a9b"></a>
## description

`function` · `tracing_subscriber::filter::directive::ParseError::description` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn description(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::directive::ParseError", "path": "ParseError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [427, 1], "end": [439, 2], "filename": "src/filter/directive.rs"}, "trait": {"args": null, "id": "core::error::Error", "path": "Error"}, "trait_path": "core::error::Error"}`

Source: `src/filter/directive.rs:428`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1a9c8903cc647a1ac53a58fb"></a>
## fmt

`function` · `tracing_subscriber::filter::directive::ParseError::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::directive::ParseError", "path": "ParseError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 10], "end": [9, 15], "filename": "src/filter/directive.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/directive.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-25c5793d35dabae95e57a502"></a>
## fmt

`function` · `tracing_subscriber::filter::directive::ParseError::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::directive::ParseError", "path": "ParseError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [424, 2], "filename": "src/filter/directive.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/filter/directive.rs:415`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-018bbfe6904e070a35b8270e"></a>
## from

`function` · `tracing_subscriber::filter::directive::ParseError::from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::directive::ParseError", "path": "ParseError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [442, 1], "end": [448, 2], "filename": "src/filter/directive.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::error::Error", "path": "Error"}}, {"generic_params": [], "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}, {"generic_params": [], "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}]}}}], "constraints": []}}, "id": "alloc::boxed::Box", "path": "Box"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/filter/directive.rs:443`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-80f08d7b52aec3b42d067243"></a>
## from

`function` · `tracing_subscriber::filter::directive::ParseError::from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from(l: level::ParseError) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::directive::ParseError", "path": "ParseError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [450, 1], "end": [456, 2], "filename": "src/filter/directive.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::ParseLevelFilterError", "path": "ParseLevelFilterError"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/filter/directive.rs:451`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2f053366626e34510e45c0d7"></a>
## source

`function` · `tracing_subscriber::filter::directive::ParseError::source` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn source(&self) -> Option<&dyn std::error::Error + 'static>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::directive::ParseError", "path": "ParseError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [427, 1], "end": [439, 2], "filename": "src/filter/directive.rs"}, "trait": {"args": null, "id": "core::error::Error", "path": "Error"}, "trait_path": "core::error::Error"}`

Source: `src/filter/directive.rs:432`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
