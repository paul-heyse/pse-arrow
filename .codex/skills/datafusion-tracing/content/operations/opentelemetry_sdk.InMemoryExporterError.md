# `opentelemetry_sdk::InMemoryExporterError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.InMemoryExporterError.json).

<a id="op-6e39a4fc508d62ae816ef062"></a>
## InMemoryExporterError

`enum` · `opentelemetry_sdk::InMemoryExporterError` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum InMemoryExporterError
```

Source: `src/lib.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Errors that can occur during when returning telemetry from InMemoryLogExporter

<a id="op-312649bd749927b3c35481df"></a>
## InternalFailure

`variant` · `opentelemetry_sdk::InMemoryExporterError::InternalFailure` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
InternalFailure
```

Source: `src/lib.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Operation failed due to an internal error.

The error message is intended for logging purposes only and should not
be used to make programmatic decisions. It is implementation-specific
and subject to change without notice. Consumers of this error should not
rely on its content beyond logging.

<a id="op-813afeac272c6e3b48e90784"></a>
## fmt

`function` · `opentelemetry_sdk::InMemoryExporterError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::InMemoryExporterError", "path": "InMemoryExporterError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [151, 28], "end": [151, 33], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/lib.rs:151`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f91e432fa9c8d256602fdb84"></a>
## fmt

`function` · `opentelemetry_sdk::InMemoryExporterError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::InMemoryExporterError", "path": "InMemoryExporterError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [151, 10], "end": [151, 26], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/lib.rs:151`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-beefe2c8b9f974cf839592f2"></a>
## from

`function` · `opentelemetry_sdk::InMemoryExporterError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(err: std::sync::PoisonError<T>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::InMemoryExporterError", "path": "InMemoryExporterError"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [169, 2], "filename": "src/lib.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "std::sync::poison::PoisonError", "path": "PoisonError"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/lib.rs:166`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
