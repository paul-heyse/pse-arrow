# `opentelemetry_sdk::runtime::TrySendError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.TrySendError.json).

<a id="op-7df202a695ad31134bc94637"></a>
## TrySendError

`enum` · `opentelemetry_sdk::runtime::TrySendError` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum TrySendError
```

Source: `src/runtime.rs:162`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Error returned by a [`TrySend`](../operations/opentelemetry_sdk.runtime.TrySend.md#op-d6606bc9965fde0d38b1406a) implementation.

<a id="op-845e3c1ec682af882bd94c1a"></a>
## ChannelClosed

`variant` · `opentelemetry_sdk::runtime::TrySendError::ChannelClosed` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ChannelClosed
```

Source: `src/runtime.rs:168`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Send failed due to the channel being closed.

<a id="op-c76a2217f3786da538c93a59"></a>
## ChannelFull

`variant` · `opentelemetry_sdk::runtime::TrySendError::ChannelFull` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ChannelFull
```

Source: `src/runtime.rs:165`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Send failed due to the channel being full.

<a id="op-4f45a650c9765d85e1e95380"></a>
## Other

`variant` · `opentelemetry_sdk::runtime::TrySendError::Other` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Other
```

Source: `src/runtime.rs:171`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Any other send error that isn't covered above.

<a id="op-01aea511a766756121063bfc"></a>
## fmt

`function` · `opentelemetry_sdk::runtime::TrySendError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TrySendError", "path": "TrySendError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [161, 10], "end": [161, 15], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/runtime.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-829a2f75355c3c6c08d425f7"></a>
## fmt

`function` · `opentelemetry_sdk::runtime::TrySendError::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TrySendError", "path": "TrySendError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [161, 17], "end": [161, 22], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/runtime.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7d6b8a6521993eb5cdbcd9b1"></a>
## from

`function` · `opentelemetry_sdk::runtime::TrySendError::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(source: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TrySendError", "path": "TrySendError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 11], "end": [171, 18], "filename": "src/runtime.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"dyn_trait": {"lifetime": null, "traits": [{"generic_params": [], "trait": {"args": null, "id": "core::error::Error", "path": "Error"}}, {"generic_params": [], "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}, {"generic_params": [], "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}]}}}], "constraints": []}}, "id": "alloc::boxed::Box", "path": "Box"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/runtime.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-747e26fcb8df5c37f48eac6e"></a>
## source

`function` · `opentelemetry_sdk::runtime::TrySendError::source` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn source(&self) -> ::core::option::Option<&dyn ::thiserror::__private::Error + 'static>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::runtime::TrySendError", "path": "TrySendError"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [161, 17], "end": [161, 22], "filename": "src/runtime.rs"}, "trait": {"args": null, "id": "core::error::Error", "path": "Error"}, "trait_path": "core::error::Error"}`

Source: `src/runtime.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
