# `opentelemetry_sdk::trace::error::TraceError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.error.TraceError.json).

<a id="op-71d305ea0e52de4749f55f91"></a>
## TraceError

`enum` · `opentelemetry_sdk::trace::error::TraceError` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum TraceError
```

Source: `src/trace/error.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Errors returned by the trace API.

<a id="op-d004ce63456ffb914b2cae6c"></a>
## ExportFailed

`variant` · `opentelemetry_sdk::trace::error::TraceError::ExportFailed` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ExportFailed
```

Source: `src/trace/error.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Export failed with the error returned by the exporter

<a id="op-db769a50fd24514568673ead"></a>
## ExportTimedOut

`variant` · `opentelemetry_sdk::trace::error::TraceError::ExportTimedOut` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ExportTimedOut
```

Source: `src/trace/error.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Export failed to finish after certain period and processor stopped the export.

<a id="op-2fd93003a03a63d94e2ce57a"></a>
## Other

`variant` · `opentelemetry_sdk::trace::error::TraceError::Other` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Other
```

Source: `src/trace/error.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Other errors propagated from trace SDK that weren't covered above

<a id="op-b2e701460c09280dae8a8c11"></a>
## TracerProviderAlreadyShutdown

`variant` · `opentelemetry_sdk::trace::error::TraceError::TracerProviderAlreadyShutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
TracerProviderAlreadyShutdown
```

Source: `src/trace/error.rs:23`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

already shutdown error

<a id="op-9da7bacfa150619314dae023"></a>
## fmt

`function` · `opentelemetry_sdk::trace::error::TraceError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [10, 10], "end": [10, 15], "filename": "src/trace/error.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/trace/error.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bc13d29b9a6c5d6b4af3a5bc"></a>
## fmt

`function` · `opentelemetry_sdk::trace::error::TraceError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [10, 17], "end": [10, 22], "filename": "src/trace/error.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/error.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-210ee00bb460fc0493a4ec76"></a>
## from

`function` · `opentelemetry_sdk::trace::error::TraceError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(source: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [27, 11], "end": [27, 18], "filename": "src/trace/error.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::error::Error", "path": "Error"}}, {"generic_params": [], "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}, {"generic_params": [], "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}]}}}], "constraints": []}}, "id": "alloc::boxed::Box", "path": "Box"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace/error.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-23aa70bdf013b4d9c34098c3"></a>
## from

`function` · `opentelemetry_sdk::trace::error::TraceError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(err_msg: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [39, 1], "end": [43, 2], "filename": "src/trace/error.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "String"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace/error.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6a6e8f36723d43ab428cc625"></a>
## from

`function` · `opentelemetry_sdk::trace::error::TraceError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(err: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::error::ExportError", "path": "ExportError"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [30, 1], "end": [37, 2], "filename": "src/trace/error.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace/error.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9a7ef8ee83feb3665a2420e7"></a>
## from

`function` · `opentelemetry_sdk::trace::error::TraceError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(err: PoisonError<T>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 1], "end": [55, 2], "filename": "src/trace/error.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "std::sync::poison::PoisonError", "path": "PoisonError"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace/error.rs:52`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cc853f12b3a2a4e369c72db0"></a>
## from

`function` · `opentelemetry_sdk::trace::error::TraceError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(err_msg: &'static str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 1], "end": [49, 2], "filename": "src/trace/error.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"primitive": "str"}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace/error.rs:46`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-98fc1e7d0e817c6b3b3d2d72"></a>
## source

`function` · `opentelemetry_sdk::trace::error::TraceError::source` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn source(&self) -> ::core::option::Option<&dyn ::thiserror::__private::Error + 'static>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::error::TraceError", "path": "TraceError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [10, 10], "end": [10, 15], "filename": "src/trace/error.rs"}, "trait": {"args": null, "id": "core::error::Error", "path": "Error"}, "trait_path": "core::error::Error"}`

Source: `src/trace/error.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
