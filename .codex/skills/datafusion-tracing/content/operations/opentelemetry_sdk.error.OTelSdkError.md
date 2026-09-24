# `opentelemetry_sdk::error::OTelSdkError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.error.OTelSdkError.json).

<a id="op-48c8a15a23d655b79e7162f9"></a>
## OTelSdkError

`enum` · `opentelemetry_sdk::error::OTelSdkError` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum OTelSdkError
```

Source: `src/error.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Errors that can occur during SDK operations export(), force_flush() and shutdown().

<a id="op-e107c7119b13fa1f868c6209"></a>
## AlreadyShutdown

`variant` · `opentelemetry_sdk::error::OTelSdkError::AlreadyShutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
AlreadyShutdown
```

Source: `src/error.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shutdown has already been invoked.

While shutdown is idempotent and calling it multiple times has no
impact, this error suggests that another part of the application is
invoking `shutdown` earlier than intended. Users should review their
code to identify unintended or duplicate shutdown calls and ensure it is
only triggered once at the correct place.

<a id="op-6a8a5873effeba0ca6983d09"></a>
## InternalFailure

`variant` · `opentelemetry_sdk::error::OTelSdkError::InternalFailure` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
InternalFailure
```

Source: `src/error.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Operation failed due to an internal error.

The error message is intended for logging purposes only and should not
be used to make programmatic decisions. It is implementation-specific
and subject to change without notice. Consumers of this error should not
rely on its content beyond logging.

<a id="op-19a196ead7d9c76e161741ef"></a>
## Timeout

`variant` · `opentelemetry_sdk::error::OTelSdkError::Timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Timeout
```

Source: `src/error.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Operation timed out before completing.

This does not necessarily indicate a failure—operation may still be
complete. If this occurs frequently, consider increasing the timeout
duration to allow more time for completion.

<a id="op-5efec7e4441d13a2c8ae025f"></a>
## fmt

`function` · `opentelemetry_sdk::error::OTelSdkError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::error::OTelSdkError", "path": "OTelSdkError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 17], "end": [13, 22], "filename": "src/error.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/error.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fc824e47937a0719f055ea01"></a>
## fmt

`function` · `opentelemetry_sdk::error::OTelSdkError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::error::OTelSdkError", "path": "OTelSdkError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 10], "end": [13, 15], "filename": "src/error.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/error.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
