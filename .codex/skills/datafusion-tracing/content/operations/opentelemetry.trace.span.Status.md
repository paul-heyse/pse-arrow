# `opentelemetry::trace::span::Status`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.span.Status.json).

<a id="op-5a4a2c8462390432f268d04c"></a>
## Status

`enum` · `opentelemetry::trace::span::Status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Status
```

Source: `src/trace/span.rs:282`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The status of a [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8).

These values form a total order: Ok > Error > Unset. This means that setting
`Status::Ok` will override any prior or future attempts to set a status with
`Status::Error` or `Status::Unset`.

The status should remain unset, except for the following circumstances:

Generally, instrumentation libraries should not set the code to
`Status::Ok`, unless explicitly configured to do so. Instrumentation
libraries should leave the status code as unset unless there is an error.

Application developers and operators may set the status code to
`Status::Ok`.

When span status is set to `Status::Ok` it should be considered final and
any further attempts to change it should be ignored.

Analysis tools should respond to a `Status::Ok` status by suppressing any
errors they would otherwise generate. For example, to suppress noisy errors
such as 404s.

Only the value of the last call will be recorded, and implementations are
free to ignore previous calls.

<a id="op-8657c9f578aab3e0c6a79d52"></a>
## Error

`variant` · `opentelemetry::trace::span::Status::Error` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Error
```

Source: `src/trace/span.rs:288`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The operation contains an error.

<a id="op-79f981804f8dd40ee038195a"></a>
## Ok

`variant` · `opentelemetry::trace::span::Status::Ok` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Ok
```

Source: `src/trace/span.rs:295`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The operation has been validated by an application developer or operator to
have completed successfully.

<a id="op-52ed2013c1fc5d1bbf601a20"></a>
## Unset

`variant` · `opentelemetry::trace::span::Status::Unset` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Unset
```

Source: `src/trace/span.rs:285`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The default status.

<a id="op-ebc50584ed832ffb2a0b13b8"></a>
## clone

`function` · `opentelemetry::trace::span::Status::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Status
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::Status", "path": "Status"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 26], "end": [281, 31], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/span.rs:281`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-addc058177110e3d6aeaf300"></a>
## default

`function` · `opentelemetry::trace::span::Status::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Status
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::Status", "path": "Status"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 10], "end": [281, 17], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/span.rs:281`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8a94e16fe8c2f69e3d809912"></a>
## eq

`function` · `opentelemetry::trace::span::Status::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Status) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::Status", "path": "Status"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 33], "end": [281, 42], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/span.rs:281`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ddbaf33013e0b3db9aca2f2b"></a>
## error

`function` · `opentelemetry::trace::span::Status::error` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn error(description: impl Into<Cow<'static, str>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::Status", "path": "Status"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [298, 1], "end": [318, 2], "filename": "src/trace/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span.rs:313`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new error status with a given description.

# Examples

```
use opentelemetry::trace::Status;

// record error with `str` description
let error_status = Status::error("something went wrong");

// or with `String` description
let error_status = Status::error(format!("too many foos: {}", 42));
# drop(error_status);
```

<a id="op-679036cc2554d65d904c283b"></a>
## fmt

`function` · `opentelemetry::trace::span::Status::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::Status", "path": "Status"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 19], "end": [281, 24], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span.rs:281`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3bfeea465a4b48b1b347658c"></a>
## partial_cmp

`function` · `opentelemetry::trace::span::Status::partial_cmp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &Status) -> option::Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span::Status", "path": "Status"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 48], "end": [281, 58], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/trace/span.rs:281`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
