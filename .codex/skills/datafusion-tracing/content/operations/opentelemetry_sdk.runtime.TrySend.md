# `opentelemetry_sdk::runtime::TrySend`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.TrySend.json).

<a id="op-d6606bc9965fde0d38b1406a"></a>
## TrySend

`trait` · `opentelemetry_sdk::runtime::TrySend` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait TrySend: Sync + Send
```

Source: `src/runtime.rs:176`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

TrySend is an abstraction of `Sender` that is capable of sending messages through a reference.

<a id="op-cf1c4f4c85fe102ac6d415f5"></a>
## Message

`assoc_type` · `opentelemetry_sdk::runtime::TrySend::Message` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Message
```

Source: `src/runtime.rs:178`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The message that will be sent.

<a id="op-ec9b5b149d45b584097754ed"></a>
## try_send

`function` · `opentelemetry_sdk::runtime::TrySend::try_send` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn try_send(&self, item: Self::Message) -> Result<(), TrySendError>
```

Source: `src/runtime.rs:183`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Try to send a message batch to a worker thread.

A failure can be due to either a closed receiver, or a depleted buffer.
