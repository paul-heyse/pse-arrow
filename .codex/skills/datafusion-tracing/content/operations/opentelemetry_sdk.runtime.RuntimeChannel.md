# `opentelemetry_sdk::runtime::RuntimeChannel`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.RuntimeChannel.json).

<a id="op-9f70b05eeaad31fed7ad1747"></a>
## RuntimeChannel

`trait` · `opentelemetry_sdk::runtime::RuntimeChannel` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait RuntimeChannel: Runtime
```

Source: `src/runtime.rs:146`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`RuntimeChannel` is an extension to [`Runtime`](../operations/opentelemetry_sdk.runtime.Runtime.md#op-9c7c0607dc3adcadbbf65e89). Currently, it provides a
channel that is used by the [log] and [span] batch processors.

[log]: crate::logs::BatchLogProcessor
[span]: crate::trace::BatchSpanProcessor

<a id="op-c5f4ffc37bb4002707333602"></a>
## Receiver

`assoc_type` · `opentelemetry_sdk::runtime::RuntimeChannel::Receiver` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Receiver
```

Source: `src/runtime.rs:148`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A future stream to receive batch messages from channels.

<a id="op-81ae0fc1ac026bf07e94b2c2"></a>
## Sender

`assoc_type` · `opentelemetry_sdk::runtime::RuntimeChannel::Sender` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Sender
```

Source: `src/runtime.rs:150`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A batch messages sender that can be sent across threads safely.

<a id="op-a78b623298e1f68b6222878c"></a>
## batch_message_channel

`function` · `opentelemetry_sdk::runtime::RuntimeChannel::batch_message_channel` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn batch_message_channel<T: Debug + Send>(&self, capacity: usize) -> (Self::Sender<T>, Self::Receiver<T>)
```

Source: `src/runtime.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Return the sender and receiver used to send batch messages.
