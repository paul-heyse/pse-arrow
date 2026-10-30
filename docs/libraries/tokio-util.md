# Tokio-util — Comprehensive Rust Library Reference

Target: `tokio-util` 0.7.20 (stable) · Released October 9, 2026 · Researched October 29, 2026 · Minimum Rust version: 1.85

Crate: `tokio-util`\
Upstream: [https://github.com/tokio-rs/tokio/tree/master/tokio-util](https://github.com/tokio-rs/tokio/tree/master/tokio-util?utm_source=chatgpt.com)\
Documentation: [https://docs.rs/tokio-util/latest/tokio_util/](https://docs.rs/tokio-util/latest/tokio_util/?utm_source=chatgpt.com)\
Release history: [https://github.com/tokio-rs/tokio/blob/master/tokio-util/CHANGELOG.md](https://github.com/tokio-rs/tokio/blob/master/tokio-util/CHANGELOG.md?utm_source=chatgpt.com)

## 1. Architectural orientation

`tokio-util` is Tokio's companion library for asynchronous coordination, I/O adaptation, protocol framing, and task lifecycle management. It supplies reusable mechanisms that sit above Tokio's runtime primitives and below application-specific protocols and orchestration logic.

Its central architectural value is replacing custom asynchronous coordination machinery with composable, well-defined abstractions.

Unlike `tokio`, which supplies task scheduling, timers, networking, synchronization primitives, and the core asynchronous runtime, `tokio-util` concentrates on the abstractions needed to organize and adapt those primitives.

The two crates are independently versioned. Version 0.7.20 is the latest published stable `tokio-util` release verified for this reference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



### Capability map

| Functional family         | Principal capabilities                                                                        | Architectural role                                                                |
| ------------------------- | --------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Cooperative cancellation  | `CancellationToken`, child tokens, drop guards                                                | Hierarchical shutdown, cancellation propagation, resource lifetime coordination   |
| Task lifecycle            | `TaskTracker`, `JoinQueue`, `JoinMap`, `AbortOnDrop`, `LocalPoolHandle`                       | Tracking completion, keyed task ownership, scoped abort behavior, local executors |
| Protocol framing          | `Decoder`, `Encoder`, `Framed`, `LengthDelimitedCodec`, delimiter codecs                      | Byte streams to structured messages, streaming protocol boundaries                |
| Asynchronous I/O adapters | `StreamReader`, `ReaderStream`, `SinkWriter`, `SyncIoBridge`, inspection and buffer utilities | Connect byte I/O, streams, sinks, and synchronous libraries                       |
| Synchronization adapters  | `PollSemaphore`, `PollSender`, `ReusableBoxFuture`                                            | Poll-oriented resource acquisition, channel backpressure, reusable future storage |
| Deadline scheduling       | `DelayQueue`, future timeout extensions                                                       | Dynamic collections of keyed expirations and time-sensitive operations            |
| Network utilities         | `UdpFramed`, `Listener`, Unix listener adapters                                               | Datagram framing and transport-independent listener abstractions                  |
| Runtime integration       | `TokioContext`, `RuntimeExt`                                                                  | Run Tokio-dependent futures in other executor contexts                            |
| Future/type composition   | `FutureExt`, `Either`                                                                         | Timeout composition and heterogeneous future, stream, and sink branches           |

The most architecturally significant groups are cancellation and task ownership, protocol framing, and I/O conversion. The remaining utilities become particularly valuable when adapting existing interfaces or eliminating specialized coordination code.

### Version and feature boundaries

Version 0.7.20 introduces several improvements relevant to this reference:

- Support for byte slices in `LengthDelimitedCodec`.
- Additional inspection accessors for `InspectReader` and `InspectWriter`.
- More precise framing limits and buffer-reservation behavior.
- Corrections to cancellation-tree notification, `StreamReader` EOF handling, and `DelayQueue` wakeups and earliest-deadline inspection.
- An increased MSRV of Rust 1.85.

Version 0.7.19 previously added `AbortOnDrop`, `JoinMap::try_join_next`, vectored-write support, and further cancellation-token capabilities. Version 0.7.18 added the `io::simplex` facility. These are all released capabilities, not proposals or unreleased APIs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



`tokio-util` has no default feature flags enabled. Most major subsystems require explicitly selected Cargo features.

| Cargo feature   | Functionality enabled                                       |
| --------------- | ----------------------------------------------------------- |
| `codec`         | Framing codecs and framed I/O transports                    |
| `compat`        | Tokio ↔ futures-io compatibility                            |
| `io`            | Byte-stream, sink, inspection, and buffer adapters          |
| `io-util`       | Additional synchronous bridge and I/O helpers; implies `io` |
| `rt`            | Task and runtime-context utilities                          |
| `join-map`      | Keyed task collection; implies `rt`                         |
| `time`          | `DelayQueue` and time-related utilities                     |
| `net` + `codec` | Listener abstractions and UDP framing                       |
| `full`          | All major public functional groups above                    |

`sync`, `future`, and `either` expose important baseline functionality without enabling those additional features; individual APIs within them can still be feature-gated.

The authoritative feature definitions are in:

[https://github.com/tokio-rs/tokio/blob/tokio-util-0.7.20/tokio-util/Cargo.toml](https://github.com/tokio-rs/tokio/blob/tokio-util-0.7.20/tokio-util/Cargo.toml?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1

## 2. Cooperative cancellation and shutdown signaling

### 2.1 `CancellationToken`: hierarchical cancellation state

`tokio_util::sync::CancellationToken` is a cloneable, thread-safe, persistent cancellation signal. Its distinctive capability is expressing cancellation relationships across independently running tasks without retaining their join handles.

| Mechanism                            | Semantics                                            | Important distinction                                                                 |
| ------------------------------------ | ---------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `clone()`                            | Shares the same cancellation state                   | Cancelling either handle cancels both                                                 |
| `child_token()`                      | Creates a dependent child                            | Parent cancellation propagates downward; child cancellation does not propagate upward |
| `cancel()`                           | Sets cancellation state and wakes registered waiters | Signals cancellation; does not abort tasks                                            |
| `cancelled()`                        | Borrowed future awaiting cancellation                | Cancellation-safe and completes immediately if already cancelled                      |
| `cancelled_owned()`                  | Owned waiting future                                 | Useful when moving the wait operation into an owned future                            |
| `is_cancelled()`                     | Synchronous state inspection                         | State query, not a substitute for waiting                                             |
| `drop_guard()` / `drop_guard_ref()`  | Cancels when a guard is dropped                      | RAII-style cancellation tied to ownership or scope                                    |
| `run_until_cancelled()` / `_owned()` | Races a future against cancellation                  | Drops the work future if cancellation wins                                            |

Cancellation is sticky: once a token is cancelled, it cannot be reset. A new token represents a new cancellation lifecycle.

The clone/child distinction matters. Clones are interchangeable handles to one signal. Children create a tree of scopes: a process-level parent can cancel all component children, while an individual component can cancel its descendants without affecting siblings or the process-level parent.

The implementation also permits independent cancellation checks and waits. It does not require all consumers to poll a shared queue or contend to receive a one-time message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Source: [https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html?utm_source=chatgpt.com)

### 2.2 Cancellation races and guarantees

Several distinctions are consequential:

Propagation is not an atomic tree-wide transition. While one thread is executing `cancel()`, another may observe a cancelled child and then a not-yet-cancelled sibling. Once the `cancel()` call returns, the token and its descendants have all been cancelled.

Cooperative cancellation is not task termination. A task must poll the cancellation future, check state, or use a cancellation wrapper. Tasks that never cooperate will continue running.

Cancelling a future is different from cancelling an operation. `run_until_cancelled` returns `None` and drops the supplied future when cancellation wins. Whether dropping it safely leaves the underlying operation in a consistent state depends on that future's cancellation-safety contract.

Completion/cancellation races have documented priority. For `run_until_cancelled`, an already-cancelled token wins immediately; after initial polling, concurrent completion and cancellation favor completion.

These behaviors support cancellation as an explicit control signal rather than an implicit rollback mechanism. External side effects already committed by a future are not reversed.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



### 2.3 Drop guards and ownership-linked cancellation

`DropGuard` owns a token; `DropGuardRef` borrows one. Both cancel the associated token and descendants on destruction unless explicitly disarmed.

They enable cancellation to follow ownership boundaries—for example, when a supervising object disappears—without requiring every exit path to invoke `cancel()` manually.

The guards provide cancel-on-drop, not abort-on-drop. Dropping a guard triggers notification; it does not guarantee that tasks have stopped or finished cleanup. `DropGuard::token()` exposes the guarded token, and `disarm()` permits the guard to be consumed without cancellation.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/sync/struct.DropGuard.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.DropGuard.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/sync/struct.DropGuardRef.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.DropGuardRef.html?utm_source=chatgpt.com)

### 2.4 Composing cancellation with futures

`tokio_util::future::FutureExt` supplies `with_cancellation_token` and `with_cancellation_token_owned`, equivalent in purpose to the token's `run_until_cancelled` methods.

With the `time` feature, the same extension trait provides fluent `timeout(Duration)` and `timeout_at(Instant)` adapters around Tokio's timers.

This produces three distinct lifecycle conditions:

- Cancellation: an external owner requests that work stop.
- Timeout: a duration or absolute deadline has elapsed.
- Completion: work produces its result normally.

These may be composed, but neither cancellation nor timeout automatically performs application-specific cleanup. In particular, a deadline on a task operation is not a guarantee that the operating-system operation or a spawned blocking function has terminated.

Source: [https://docs.rs/tokio-util/latest/tokio_util/future/trait.FutureExt.html](https://docs.rs/tokio-util/latest/tokio_util/future/trait.FutureExt.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 3. Task lifecycle and execution ownership

`tokio-util::task` provides several different mechanisms for organizing spawned tasks. Their key distinctions concern who owns the tasks, whether outputs are retained, the order of completion observation, and what happens on drop.

### 3.1 Comparison of task collections

| Mechanism          | Completion model                          | Output retention              | Drop semantics               |
| ------------------ | ----------------------------------------- | ----------------------------- | ---------------------------- |
| `TaskTracker`      | Wait for all tracked work to exit         | No retained output collection | Does not abort tracked tasks |
| `JoinQueue<T>`     | Join in spawn order                       | Retains outputs until joined  | Aborts owned tasks           |
| `JoinMap<K,V>`     | Join in completion order, preserving keys | Retains outputs until joined  | Aborts owned tasks           |
| Tokio `JoinSet<T>` | Join in completion order                  | Retains outputs until joined  | Aborts owned tasks           |

`TaskTracker` answers “has everything finished?”. The join collections answer “which task finished, and what was its result?”

### 3.2 `TaskTracker`: completion accounting without result retention

`TaskTracker` maintains a shared task count and a separate open/closed state. `wait()` completes only when the tracker is simultaneously closed and empty.

Its capabilities extend beyond its `spawn()` convenience method:

- `spawn`, `spawn_on`, `spawn_local`, `spawn_blocking`, and corresponding runtime/local variants track different execution paths.
- `track_future()` wraps an arbitrary future, including one executed through another task-management abstraction.
- `token()` produces a `TaskTrackerToken` for manually managed work; each live token contributes to the outstanding count, and dropping it releases that count.
- `clone()` shares the same tracker state, allowing independent components to register work.
- `close()` and `reopen()` control whether an empty tracker may satisfy waiters.

The most consequential behavioral detail is that `close()` does not reject newly registered tasks. It is a completion barrier control, not a task-admission gate. Admission must be controlled separately if required.

Unlike `JoinSet`, `TaskTracker` does not accumulate completed task outputs. On exit, tracked tasks can promptly release their resources even if nobody continuously polls a join collection. This can be materially advantageous for long-lived services with frequent short tasks whose individual results are unimportant.

`wait()` is cancellation-safe. A retained wait future also protects against brief closed-and-empty transitions that occur between polls. Recreating the future on each iteration of a `select!` loop forfeits that particular guarantee.

When `wait()` returns, all tracked futures have completed destruction, although Tokio's `JoinHandle::is_finished()` may momentarily lag. Dropping the tracker itself does not abort work.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html](https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTrackerToken.html](https://docs.rs/tokio-util/latest/tokio_util/task/task_tracker/struct.TaskTrackerToken.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2

### 3.3 `JoinQueue`: deterministic result order

`JoinQueue<T>` owns spawned tasks with a homogeneous output type and joins them in FIFO spawn order, regardless of when they actually finish.

For tasks spawned A, B, C, the first join observes A, even when C and B have completed already. This separates concurrent execution from ordered result consumption.

The tradeoff is head-of-line blocking: a long-running early task delays access to later results, and completed later tasks retain outputs until their turn.

Its surface includes `join_next`, `try_join_next`, `poll_join_next`, task-ID-bearing variants, `join_all`, `abort_all`, `shutdown`, and `detach_all`. Spawning returns an `AbortHandle`, allowing selective remote abort requests.

`shutdown()` requests aborts and waits for completion; `abort_all()` alone does not wait. Dropping the queue aborts the tasks it still owns. The queue is not a concurrency limiter; spawning tasks still requires separate resource bounds where appropriate.

Source: [https://docs.rs/tokio-util/latest/tokio_util/task/struct.JoinQueue.html](https://docs.rs/tokio-util/latest/tokio_util/task/struct.JoinQueue.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.4 `JoinMap`: keyed, completion-ordered task ownership

`JoinMap<K,V>` combines task ownership with a hash-map index. Tasks have homogeneous outputs but distinct keys, and completed results arrive as `(key, Result<V, JoinError>)`, generally in completion order.

Advanced operations include:

- `abort(&key)` and `abort_matching(predicate)` for selective cancellation.
- `contains_key`, `contains_task`, and `keys` for visibility into outstanding or completed-but-unjoined work.
- `try_join_next()` for nonblocking result retrieval.
- `spawn`, `spawn_on`, `spawn_local`, and blocking variants.
- Capacity management and custom hash builders.

Key replacement is an especially important semantic choice. Spawning another task under an existing key requests abortion of the old task and replaces it. The previous entry is removed, and its cancellation does not subsequently appear as a `JoinError` from `join_next`.

This creates a native latest-task-per-key pattern, useful for superseding obsolete operations. It does not guarantee that a replaced task has ceased making side effects: Tokio's task abort is asynchronous, and an already-running blocking task cannot be forcibly stopped.

Conversely, `abort(&key)` requests cancellation of an existing entry while preserving its result for subsequent joining, including a cancellation error.

`JoinMap` also has a separate `join-map` feature gate and uses `hashbrown` for key indexing. It does not replace application-level version checks where obsolete work must be prevented from publishing results.

Source: [https://docs.rs/tokio-util/latest/tokio_util/task/struct.JoinMap.html](https://docs.rs/tokio-util/latest/tokio_util/task/struct.JoinMap.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



### 3.5 `AbortOnDrop` and `AbortOnDropHandle`

These wrappers give individual spawned tasks RAII-style abort ownership.

`AbortOnDrop` wraps Tokio's nongeneric `AbortHandle`. It is useful when the task's output is irrelevant, such as retaining a task's lifetime in an owning structure. It exposes `abort`, `is_finished`, and `detach`, but cannot be awaited for a result.

`AbortOnDropHandle<T>` wraps a `JoinHandle<T>`, retains the ability to await the result, and additionally exposes `abort_handle`. `detach()` transfers out the original handle, disabling abort-on-drop for that wrapper.

Both differ from `CancellationToken`: dropping them requests Tokio task abortion, not cooperative cancellation notification. Abortion generally takes effect when the asynchronous task is next polled, and a blocking task already executing in `spawn_blocking` cannot be forcibly terminated.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/task/struct.AbortOnDrop.html](https://docs.rs/tokio-util/latest/tokio_util/task/struct.AbortOnDrop.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/task/struct.AbortOnDropHandle.html](https://docs.rs/tokio-util/latest/tokio_util/task/struct.AbortOnDropHandle.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1

### 3.6 `LocalPoolHandle`: persistent execution of `!Send` futures

`LocalPoolHandle` establishes a fixed-size pool of dedicated worker threads, each with a Tokio `LocalRuntime`, for futures that must remain on one thread.

It provides `spawn_pinned()` for worker selection and `spawn_pinned_by_idx()` for explicit worker choice, alongside `num_threads()` and `get_task_loads_for_each_worker()`.

The essential mechanism is a `Send` task-creation closure that runs on the destination worker, where it can create a `!Send` future. The future itself never needs to cross threads. Its output must be `Send` so it can be returned through the join handle.

This supports thread-affine resources, `Rc`/`RefCell`-based state, or APIs with non-thread-safe execution contexts. Within a pinned future, `tokio::task::spawn_local` can schedule additional local tasks on the same worker.

It is distinct from Tokio's ordinary multithreaded `spawn`, where a `Send` future can migrate between workers. It is also not a CPU-affinity or operating-system scheduling guarantee. Worker count, thread lifetime, and distribution of uneven task loads remain material operational considerations.

Source: [https://docs.rs/tokio-util/latest/tokio_util/task/struct.LocalPoolHandle.html](https://docs.rs/tokio-util/latest/tokio_util/task/struct.LocalPoolHandle.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 4. Protocol framing and streaming codecs

The `codec` module separates three concerns: raw byte transport, frame boundary detection, and application-level message interpretation.

This separation is useful because TCP, Unix streams, and other byte-oriented transports do not preserve application message boundaries. A single read can contain part of a message, several messages, or arbitrary fragments of multiple messages.

### 4.1 The `Decoder` and `Encoder` contract

`Decoder` converts buffered bytes into zero or more typed frames. Its defining operation is:

`decode(&mut self, src: &mut BytesMut) -> Result<Option<Item>, Error>`

`Some(Item)` means a complete frame has been decoded; `None` means more input is needed. The decoder consumes only the bytes belonging to frames it successfully processes. Remaining bytes stay buffered for subsequent decoding.

The decoder owns mutable state, which enables incremental parsers, multistage protocols, variable-length headers, escape processing, and frame formats whose interpretation depends on earlier messages.

`Encoder<Item>` performs the inverse operation: it appends an encoded representation to an existing `BytesMut`. Encoding and decoding can use different Rust item types.

EOF handling is an important extension point. `Decoder::decode_eof` allows a codec to finish decoding buffered data when the underlying source reaches EOF. The default implementation detects incomplete trailing frames; a custom implementation can interpret EOF differently.

The decoder's error contract is also consequential: its `Error` type must implement `From<std::io::Error>`. The documentation distinguishes unrecoverable framing failures from malformed messages that can be represented as ordinary items, allowing higher layers to decide whether processing should continue.

Source: [https://docs.rs/tokio-util/latest/tokio_util/codec/trait.Decoder.html](https://docs.rs/tokio-util/latest/tokio_util/codec/trait.Decoder.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.2 Framed transport adapters

| Adapter            | Input capability          | Exposed capability                         |
| ------------------ | ------------------------- | ------------------------------------------ |
| `FramedRead<T,D>`  | `AsyncRead` + `Decoder`   | `Stream<Item = Result<D::Item, D::Error>>` |
| `FramedWrite<T,E>` | `AsyncWrite` + `Encoder`  | `Sink<Item>`                               |
| `Framed<T,C>`      | Bidirectional I/O + codec | Combined `Stream` and `Sink`               |

`Framed` maintains distinct read and write buffers, manages incremental I/O, and applies sink backpressure. The same codec infrastructure can therefore support a protocol with separate inbound and outbound message types.

The significant extension mechanisms are:

Codec replacement. `map_codec` can transform the codec associated with a transport. This supports transitions between protocol stages, provided any existing buffered data remains meaningful under the new codec.

Transport decomposition. `into_parts()` extracts a `FramedParts` containing `io`, `codec`, `read_buf`, and `write_buf`. `from_parts()` reconstructs a framed transport. This permits protocol upgrades or restructuring without silently discarding bytes already read ahead or queued for writing.

Buffer inspection and access. `read_buffer`, `write_buffer`, and mutable variants expose internal buffers. This makes specialized handoffs possible, but improperly changing them can corrupt framing state.

Write backpressure. `set_backpressure_boundary()` controls the write-buffer boundary used when deciding readiness for additional sink items. It is a flow-control threshold, not a hard cap on the size of one encoded message or on all memory held by the transport.

Cancellation safety is asymmetric. The documented stream `next()` operation is cancellation-safe. A cancelled `SinkExt::send()` can lose the input item even though the framed sink guarantees that the item was not sent by that operation. This distinction is significant for reliable message delivery and retry logic.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/codec/struct.FramedParts.html](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.FramedParts.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1

### 4.3 Native codec families

`LengthDelimitedCodec` — configurable binary framing

This is substantially more capable than a fixed four-byte prefix parser.

Its builder supports:

| Parameter                                      | Purpose                                                     |
| ---------------------------------------------- | ----------------------------------------------------------- |
| `length_field_type` / `length_field_length`    | Unsigned length representation, up to eight bytes           |
| `big_endian`, `little_endian`, `native_endian` | Wire byte order                                             |
| `length_field_offset`                          | Position of the length field within a header, decoding only |
| `length_adjustment`                            | Adjust recorded length to actual frame length               |
| `num_skip`                                     | Number of header bytes removed from decoded output          |
| `max_frame_length`                             | Bounds accepted frame sizes                                 |

The default representation is a big-endian `u32` payload length, with an 8 MiB maximum frame length.

The difference between wire length and payload length is especially important. When decoding, the maximum is checked against the length field before the configured adjustment. When encoding, it is checked against the submitted payload length.

`set_max_frame_length()` changes the limit at runtime, taking effect on the next decoded frame rather than retroactively rejecting a frame already being processed. Version 0.7.20 also clamps the configured maximum to the representable range of the selected length-field width.

The codec exposes `Encoder<Bytes>` and, as of 0.7.20, `Encoder<&[u8]>`. The latter helps when the source data is already available as a borrowed slice.

`LinesCodec` — bounded text-line framing

Reads `\n`-delimited UTF-8 lines. `new_with_max_length()` bounds accepted line lengths, avoiding indefinite accumulation of input lacking a newline. After an overlong line, the decoder discards the remainder until the delimiter and then resumes normal processing.

`AnyDelimiterCodec` — configurable byte-delimiter sets

Splits on any individual byte in a supplied delimiter set. It is not a general multi-byte delimiter-sequence matcher.

The set of accepted input delimiters and the sequence written by the encoder can differ. `new_with_max_length()` prevents unbounded accumulation.

`BytesCodec` — byte-chunk adaptation

Passes raw chunks rather than detecting semantic messages. It is useful for adapting an `AsyncRead` to a stream of bytes, but its emitted boundaries should not be interpreted as protocol records.

For untrusted data, limits must be considered at the correct layer. A framing limit does not independently bound the allocation required to deserialize a valid frame, nor does it impose application-level validation.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/codec/length_delimited/struct.Builder.html](https://docs.rs/tokio-util/latest/tokio_util/codec/length_delimited/struct.Builder.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/codec/struct.LinesCodec.html](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.LinesCodec.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/codec/struct.AnyDelimiterCodec.html](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.AnyDelimiterCodec.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+3

## 5. Asynchronous I/O adaptation and data movement

The `io` module solves integration problems where different libraries expose data through `AsyncRead`, `AsyncWrite`, `Stream`, `Sink`, or synchronous `std::io` interfaces.

Its main architectural value is allowing the underlying byte representation and asynchronous execution to remain intact while changing the interface presented to a consumer.

### 5.1 `ReaderStream` and `StreamReader`

`ReaderStream<R>` converts an `AsyncRead` into a `Stream<Item = Result<Bytes, io::Error>>`.

It uses bounded read chunks, optionally configured with `with_capacity()`. The current default is 4 KiB, explicitly documented as not part of the semantic-versioning contract. The resulting stream is fused.

`StreamReader<S,B>` performs the opposite conversion: a stream of `Result<B,E>` items, where `B: bytes::Buf` and the errors can be converted to `io::Error`, becomes an `AsyncRead` and `AsyncBufRead`.

These conversions are not perfect inverses in terms of structure:

- A `StreamReader` presents a continuous sequence of bytes. Its original stream item boundaries are not preserved through the `AsyncRead` interface.
- `StreamReader` may hold a partially consumed chunk. `into_inner()` discards that remaining chunk, whereas `into_inner_with_chunk()` returns it alongside the original stream.
- `ReaderStream` generates chunks according to read availability and buffer capacity, not necessarily according to message boundaries.
- In 0.7.20, `StreamReader` stops polling the underlying stream after its EOF state has been reached.

The adapters are particularly useful for connecting asynchronous readers with chunk-oriented HTTP bodies, storage clients, compression streams, and generic stream-processing APIs.

They do not themselves implement HTTP's `Body` protocol or its frame metadata. An HTTP integration may still require conversions from the HTTP library's body representation and error types.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/io/struct.ReaderStream.html](https://docs.rs/tokio-util/latest/tokio_util/io/struct.ReaderStream.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/io/struct.StreamReader.html](https://docs.rs/tokio-util/latest/tokio_util/io/struct.StreamReader.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2

### 5.2 `SinkWriter` and `CopyToBytes`

`SinkWriter<S>` exposes an `AsyncWrite` interface over a `Sink<&[u8]>`.

This is less trivial than it initially appears: many sinks receive owned buffers such as `Bytes`, rather than borrowing a caller's byte slice. `CopyToBytes` fills that gap by accepting borrowed slices, copying them into owned `Bytes` values, and forwarding those values to a `Sink<Bytes>`.

The combination:

`AsyncWrite` interface → `SinkWriter` → `CopyToBytes` → `Sink<Bytes>`

allows writer-oriented code to feed channel-oriented or message-oriented destinations.

It is particularly useful with `PollSender`, which turns a Tokio channel sender into a sink.

The ownership conversion has a real cost: the borrowed slice must become an independently owned value. It also changes semantics from arbitrary partial byte writes to sink item submissions. Backpressure, flush, and shutdown behavior follow the underlying sink; `SinkWriter` closes the sink when the writer is shut down.

This is a convenient interoperability layer rather than a general guarantee of zero-copy writing.

Source: [https://docs.rs/tokio-util/latest/tokio_util/io/struct.SinkWriter.html](https://docs.rs/tokio-util/latest/tokio_util/io/struct.SinkWriter.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.3 `SyncIoBridge`: synchronous consumers of asynchronous sources

`SyncIoBridge<T>` is designed for libraries that require ordinary synchronous I/O traits while their data source or destination is asynchronous.

Depending on the underlying type, it implements `std::io::Read`, `Write`, `BufRead`, and `Seek`.

It works by using a Tokio runtime handle to synchronously block on the underlying asynchronous operations.

This is not a mechanism for performing synchronous I/O on an async worker. Its blocking methods must run outside the asynchronous execution context, typically inside `tokio::task::spawn_blocking`. Calling the underlying blocking operation from an asynchronous execution context can panic.

`new()` captures the current runtime handle; `new_with_handle()` accepts an explicit handle. The runtime must remain available for the bridge's operations.

It enables interoperability with synchronous file parsers, archive libraries, encoders, and other APIs that cannot consume `AsyncRead` directly. However, each actively blocked operation occupies an OS thread. Async-native processing of chunks, or explicitly buffering into memory, can be more efficient where the data and workload permit.

The bridge also preserves synchronization constraints of its consumer: a synchronous parser reading indefinitely cannot necessarily be stopped merely by aborting its `spawn_blocking` join handle.

Source: [https://docs.rs/tokio-util/latest/tokio_util/io/struct.SyncIoBridge.html](https://docs.rs/tokio-util/latest/tokio_util/io/struct.SyncIoBridge.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



### 5.4 `io::simplex`: an asynchronous byte channel

Introduced in 0.7.18, `io::simplex::new(capacity)` creates a unidirectional pair:

- `Sender`: `AsyncWrite`
- `Receiver`: `AsyncRead`

Unlike a channel of discrete messages, it transports a continuous byte sequence. The capacity defines when the sender encounters backpressure, rather than a fixed item count.

Important semantics include:

- Zero capacity is rejected.
- Reads are cancellation-safe.
- Dropping or shutting down the sender does not discard bytes already buffered for the receiver.
- Writes and reads use internal buffering and wakeups.

This provides an asynchronous in-memory pipe between independently structured producers and consumers. It can eliminate the need to define arbitrary chunk-message envelopes when the consumer already understands `AsyncRead`.

The byte-channel semantics also mean it does not preserve individual producer write boundaries.

Source: [https://docs.rs/tokio-util/latest/tokio_util/io/simplex/index.html](https://docs.rs/tokio-util/latest/tokio_util/io/simplex/index.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



### 5.5 Buffer-oriented and specialized I/O operations

The less prominent `io` helpers are useful when ownership, copies, and syscall behavior matter.

| Utility                      | Capability                                    | Consequential detail                                                                               |
| ---------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `poll_read_buf` / `read_buf` | Read into a `bytes::BufMut`                   | Integrates with Bytes buffers and their capacity management                                        |
| `poll_write_buf`             | Write from a `bytes::Buf`                     | Advances the source buffer cursor as bytes are written                                             |
| `read_exact_arc`             | Read a fixed amount directly into `Arc<[u8]>` | Uses `Arc::new_uninit_slice`, avoiding a separate temporary vector for the completed shared buffer |
| `write_all_vectored`         | Write multiple `IoSlice` buffers completely   | Can exploit vectored writes, but advances slice state and is not cancellation-safe                 |
| `InspectReader`              | Observe bytes successfully read               | Supports incremental hashing, counting, and related observation                                    |
| `InspectWriter`              | Observe bytes successfully written            | Supports inspection of actual write progress, including vectored writes                            |

`write_all_vectored` can avoid concatenating discontiguous buffers and may reduce system calls when the underlying writer supports efficient vectored writes. It does not guarantee one system call, and interruption may leave a partially completed write. Its supplied `IoSlice` state is advanced as data is consumed.

`InspectReader` and `InspectWriter` are transparency-oriented adapters: inspection does not replace the underlying I/O operation. They can avoid duplicating read/write loops merely to compute hashes, collect byte counts, or observe a stream. Direct I/O performed on an exposed underlying resource bypasses the corresponding inspector.

Version 0.7.20 adds `get_ref`, `get_mut`, and `get_pin_mut` to both inspection wrappers. Their inspection functions run synchronously within the relevant I/O polling path, so expensive callbacks can impair executor responsiveness.

Sources:

[https://docs.rs/tokio-util/latest/tokio_util/io/index.html](https://docs.rs/tokio-util/latest/tokio_util/io/index.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/io/fn.read_exact_arc.html](https://docs.rs/tokio-util/latest/tokio_util/io/fn.read_exact_arc.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/io/fn.write_all_vectored.html](https://docs.rs/tokio-util/latest/tokio_util/io/fn.write_all_vectored.html?utm_source=chatgpt.com)

[https://docs.rs/tokio-util/latest/tokio_util/io/struct.InspectWriter.html](https://docs.rs/tokio-util/latest/tokio_util/io/struct.InspectWriter.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+3

## 6. Poll-oriented synchronization and allocation reuse

These utilities address a recurring integration problem: an asynchronous primitive exposes a convenient `async fn`, but the consumer must implement a lower-level `poll_*` method or a `Sink` or `Stream` trait.

### 6.1 `PollSemaphore`

`PollSemaphore` adapts an `Arc<tokio::sync::Semaphore>` to poll-based acquisition.

`poll_acquire()` and `poll_acquire_many()` return `Poll::Pending` while waiting, `Some(OwnedSemaphorePermit)` on success, and `None` when the underlying semaphore is closed.

It also implements `Stream<Item = OwnedSemaphorePermit>`.

This enables a driver implementing `poll_next` or another poll-based state machine to acquire capacity without spawning a separate task for each request.

It preserves the semantics of Tokio's underlying semaphore, including shared permit ownership and acquisition fairness. Large multi-permit requests can therefore delay smaller requests queued behind them.

`PollSemaphore` is not a separate resource scheduler. Its distinct value is interface adaptation and maintaining the pending acquisition state between polls. The most recently supplied waker is the one registered for subsequent notifications.

Source: [https://docs.rs/tokio-util/latest/tokio_util/sync/struct.PollSemaphore.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.PollSemaphore.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.2 `PollSender`

`PollSender<T>` adapts Tokio's bounded `mpsc::Sender<T>` to poll-oriented sending and implements `Sink<T>`.

Its two-phase protocol is particularly important:

1. `poll_reserve()` waits for and reserves channel capacity.
2. `send_item(value)` consumes that reservation.

Calling `send_item` without a successful reservation panics. `abort_send()` releases the current reservation without sending the item.

The reservation is what connects channel capacity to sink readiness. It prevents an adapter from reporting readiness and subsequently discovering that it has no capacity to deliver the accepted item.

`close()` applies to that particular `PollSender`, not necessarily every sender associated with the channel. An already-reserved slot can still be used for a final send after closing.

Cloning a `PollSender` produces a new wrapper state rather than duplicating an in-progress reservation.

This is a significant building block for connecting Tokio channels to `futures::Sink` pipelines and to `SinkWriter`, avoiding custom polling state machines.

Source: [https://docs.rs/tokio-util/latest/tokio_util/sync/struct.PollSender.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.PollSender.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.3 `ReusableBoxFuture`

`ReusableBoxFuture<'a,T>` is a reusable, dynamically dispatched, pinned future allocation.

Its `set()` operation replaces the contained future without reallocating when the replacement has the same memory layout—size and alignment. If the layout differs, it reallocates.

`try_set()` is stricter: it never reallocates, and returns the replacement future on layout mismatch.

This can reduce repeated heap-allocation costs in long-lived polling adapters that create short-lived futures of the same concrete type, such as repeatedly acquiring permits.

The tradeoffs remain dynamic dispatch, a heap allocation for the initial future, and layout-sensitive reuse. It is not automatically preferable to a statically typed future or explicit state machine.

Source: [https://docs.rs/tokio-util/latest/tokio_util/sync/struct.ReusableBoxFuture.html](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.ReusableBoxFuture.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 7. Keyed expiration and deadline management

### `DelayQueue<T>`

`DelayQueue<T>` represents a dynamically managed collection of expiring entries, exposing expired elements as a `Stream`.

It is implemented using a timer wheel and slab-backed entry storage. That combination is intended to avoid maintaining an independent Tokio sleep task or timer for every entry.

Its operations include `insert`, `insert_at`, `reset`, `reset_at`, `remove`, `try_remove`, `deadline`, `peek`, `poll_expired`, `clear`, `reserve`, and `compact`.

The key architectural distinction is expiration identity.

An insertion returns a `delay_queue::Key`. That key allows a caller to cancel or reschedule the specific pending expiration without removing and rebuilding an entire timer collection.

This makes `DelayQueue` applicable to session timeouts, cache eviction, idle-resource expiry, retry schedules, and debounced work.

Several less-obvious semantics matter:

Keys are reused. Once an entry is removed or expires, its key can eventually refer to a different entry. Retaining stale keys can therefore target the wrong expiration.

Expiration order is not total order. Earlier deadlines are yielded first, but simultaneous expirations have no documented relative ordering guarantee. Delays are rounded to the nearest millisecond.

An empty queue is not a permanently exhausted stream. Polling an empty `DelayQueue` produces `Ready(None)`, but inserting another item makes it produce entries again. It must not be treated like an irreversibly fused stream.

Polls are cancellation-safe. Cancelling a pending `next()` does not lose an expiration event.

Storage can be reused and compacted. `reserve` and `compact` provide control over allocation behavior, with compaction requiring work proportional to the collection.

In 0.7.20, `peek()` was corrected to return the earliest key and `clear()` was corrected to wake appropriate waiters.

`DelayQueue` is an in-memory timer collection, not a durable scheduler, persistence engine, or automatic retry executor. It produces expired items; callers perform the corresponding work.

Source: [https://docs.rs/tokio-util/latest/tokio_util/time/struct.DelayQueue.html](https://docs.rs/tokio-util/latest/tokio_util/time/struct.DelayQueue.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



## 8. Networking and datagram framing

### 8.1 `UdpFramed`: message decoding over UDP

`UdpFramed<C,T>` provides a combined `Stream` and `Sink` interface over a Tokio UDP socket using `Decoder` and `Encoder`.

Received frames include their source `SocketAddr`, while sent items specify their destination address.

Unlike a TCP byte stream, UDP already has datagram boundaries. `UdpFramed` preserves the notion that data originated in a particular datagram while allowing a codec to decode multiple application frames from one datagram.

For an incoming datagram, it repeatedly invokes `decode_eof()` until the codec returns `Ok(None)`.

This is a consequential difference from stream-oriented decoding. A custom decoder must account for the end of each datagram, including how to treat incomplete or invalid content.

The documentation specifically warns that a decoder reporting an error from a malformed datagram should consume or clear the remaining datagram buffer if it wants subsequent datagrams to be processed cleanly.

`UdpFramed` does not introduce reliable delivery, ordering, congestion control, retransmission, or guaranteed datagram delivery. Nor does it remove UDP's underlying datagram-size considerations.

Source: [https://docs.rs/tokio-util/latest/tokio_util/udp/struct.UdpFramed.html](https://docs.rs/tokio-util/latest/tokio_util/udp/struct.UdpFramed.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 8.2 `net::Listener` and heterogeneous listeners

`tokio_util::net::Listener` abstracts over stream-oriented listeners through associated `Io` and `Addr` types, with `poll_accept`, an asynchronous `accept()` helper, and `local_addr()`.

Its implementations for Tokio TCP and Unix listeners allow generic connection-accepting logic to operate across these transports.

The `Either<L,R>` integration is useful when the listener type itself depends on configuration. Rather than boxing a listener behind a trait object, an enum can represent one of two known transport implementations.

The `net` module is gated by `codec` and `net`, with Unix-specific functionality additionally depending on the target platform.

This is a transport interface abstraction. It does not create network sockets, configure TLS, select an application protocol, or manage connection-task lifetimes.

Source: [https://docs.rs/tokio-util/latest/tokio_util/net/index.html](https://docs.rs/tokio-util/latest/tokio_util/net/index.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



## 9. Runtime and type-system interoperability

### 9.1 Tokio ↔ futures-io compatibility

The `compat` feature provides `Compat<T>` and extension traits that translate between Tokio's asynchronous I/O trait family and the `futures-io` trait family.

The relevant extension traits include `TokioAsyncReadCompatExt`, `TokioAsyncWriteCompatExt`, `FuturesAsyncReadCompatExt`, and `FuturesAsyncWriteCompatExt`.

The distinction being addressed is at the trait level: both ecosystems describe asynchronous reading and writing, but they use different trait definitions and, for reading, different buffer interfaces.

`Compat<T>` makes an underlying reader or writer usable through the opposite trait interface without requiring a custom adapter for each integrated library.

It does not convert a Tokio runtime into another executor, create an I/O reactor, or guarantee that an I/O resource can operate without its original runtime dependencies. It translates trait behavior while preserving the lifetime and execution requirements of the underlying resource.

Source: [https://docs.rs/tokio-util/latest/tokio_util/compat/index.html](https://docs.rs/tokio-util/latest/tokio_util/compat/index.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.2 `TokioContext` and `RuntimeExt`

`TokioContext<F>` associates a future with a Tokio runtime handle. When the future is polled elsewhere, Tokio's context is made available for operations that require it.

`RuntimeExt::wrap()` is the convenience mechanism for associating a future with a runtime.

This is especially valuable when a future contains Tokio-dependent operations—such as timers—but its outer scheduling is controlled by another executor.

Three important limitations distinguish it from cross-runtime task migration:

- The future remains polled by the executor that owns it. `TokioContext` supplies runtime context; it does not automatically spawn the entire future onto Tokio.
- The referenced Tokio runtime must remain alive.
- A Tokio current-thread runtime must be actively driven through `block_on` for its underlying drivers to make progress. The upstream documentation recommends a multithreaded runtime for this use case, even with one worker.

This makes `TokioContext` a relatively specialized integration tool. It is useful where execution ownership and access to Tokio runtime services must be separated, rather than as a general replacement for Tokio task spawning.

Source: [https://docs.rs/tokio-util/latest/tokio_util/context/struct.TokioContext.html](https://docs.rs/tokio-util/latest/tokio_util/context/struct.TokioContext.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.3 `Either<L,R>`: static dispatch across heterogeneous implementations

`tokio_util::either::Either` is a two-variant enum with implementations of common asynchronous traits, including `Future`, `Stream`, `Sink`, and Tokio's asynchronous I/O traits when the contained types satisfy the relevant bounds.

Its purpose is to provide a single concrete type where two branches produce different implementation types with compatible associated outputs.

This can avoid introducing a boxed trait object solely to reconcile differing concrete types across a branch.

The tradeoff is a sum type whose size is determined by its variants and whose behavior includes variant dispatch. It also only handles two alternatives directly; more alternatives require nesting or a separate enum.

A particularly useful interaction is `Either` with `net::Listener`, enabling a transport choice to remain statically typed while sharing connection-processing logic.

Source: [https://docs.rs/tokio-util/latest/tokio_util/either/enum.Either.html](https://docs.rs/tokio-util/latest/tokio_util/either/enum.Either.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 10. Capability combinations and architectural implications

The most interesting value in `tokio-util` often comes from combinations of native facilities rather than isolated adapters.

### 10.1 Graceful service shutdown

`CancellationToken` and `TaskTracker` intentionally solve complementary problems.

The token provides the shutdown request, possibly propagated through a hierarchy of component scopes. The tracker establishes when outstanding work has finished.

A typical sequence conceptually consists of stopping admission of new work, closing the tracker, signalling cancellation, and awaiting tracker completion. The first step is external because closing a tracker does not prevent new tasks from being inserted.

This is materially different from aborting a task collection. Cooperative cancellation permits tasks to release resources and execute asynchronous cleanup while remaining tracked; abortion does not provide an equivalent cleanup guarantee.

If tasks must both produce results and be joined, `JoinMap`, `JoinQueue`, or Tokio's `JoinSet` may be more appropriate for those tasks, potentially alongside cancellation tokens.

### 10.2 Bounded asynchronous data pipelines

`ReaderStream`, `StreamReader`, `PollSender`, `SinkWriter`, and `PollSemaphore` together cover several common interface and backpressure requirements.

For example, a byte-oriented producer can be exposed as a stream, handed through a bounded channel via `PollSender`, and consumed through a reader interface or a decoding stage.

The reusable value is not a proprietary pipeline engine: it is the ability to use standard `Stream`, `Sink`, `AsyncRead`, `AsyncWrite`, and Tokio synchronization semantics without writing bespoke poll-based channel and buffer adapters.

However, bounded channel capacity is not the same as bounded total memory. Chunk sizes, framing buffers, upstream prefetch, deserialized objects, and tasks waiting for downstream capacity all contribute to resource usage.

### 10.3 Protocol evolution without losing read-ahead data

`Framed::into_parts()` and `Framed::from_parts()` address a subtle but common issue with layered protocols: when a transport changes its framing rules, bytes may already have been read under the previous stage.

Keeping the transport and its buffers together permits a controlled protocol transition without silently dropping read-ahead content.

The caller still needs to ensure that retained bytes are meaningful to the replacement decoder, that pending writes have the intended framing, and that any transport-level transition—such as introducing encryption—is coordinated with the framing state.

The feature avoids a substantial amount of manual buffer handoff machinery but does not itself implement protocol negotiation.

### 10.4 Keyed, cancellable operational state

`JoinMap` and `DelayQueue` provide different keyed lifecycles.

`JoinMap` indexes running task ownership by application key. `DelayQueue` indexes pending expirations by its internal key.

Combined with `CancellationToken` child scopes, they can support independently cancellable groups of work, one current task per key, and dynamically rescheduled timeouts.

The distinction is that these collections track different things. Cancelling a token is not equivalent to removing a timer, and removing an expiration does not abort a task. The application is responsible for coordinating those identities and ensuring that obsolete or cancelled work does not publish invalid results.

### 10.5 Efficient adaptation versus avoiding unnecessary conversions

A number of the utilities are low-overhead wrappers around existing primitives, but they are not universally zero-cost.

`BytesMut` and `Bytes` can often transfer or share byte storage efficiently, while `CopyToBytes` explicitly makes an owned copy. `SyncIoBridge` trades asynchronous multiplexing for synchronous compatibility. `ReusableBoxFuture` may reduce allocations but retains dynamic dispatch. `Either` avoids boxing but may enlarge the containing value. `Framed` adds buffering and framing work while eliminating application-managed partial-read logic.

These are mechanisms with conditional performance advantages, not general benchmarks. Their value includes reducing duplicated state machines and improving correctness, in addition to potential runtime efficiency.

## 11. Important boundaries and limitations

The library is intentionally not a comprehensive networking or distributed-systems framework.

| Concern                | What `tokio-util` supplies                             | What remains elsewhere                                                      |
| ---------------------- | ------------------------------------------------------ | --------------------------------------------------------------------------- |
| Runtime and scheduling | Runtime-dependent task utilities and context adapters  | Tokio's executor, I/O driver, timers, task scheduling                       |
| Cancellation           | Cooperative signals and abort-handle wrappers          | Cleanup guarantees, cancellation policies, durable recovery                 |
| Task management        | Completion tracking and selected ownership collections | Global supervision, retries, process restarts, task result persistence      |
| Protocol processing    | Framing, encoding, decoding, buffering                 | Application schemas, authentication, TLS, protocol state machines           |
| I/O interoperability   | Reader/stream/sink and sync/async adapters             | Actual I/O providers and transport implementations                          |
| Flow control           | Sink readiness and channel/semaphore adapters          | End-to-end memory budgets, admission control, network congestion control    |
| Deadline management    | In-memory keyed expirations                            | Persistent schedules, distributed timer coordination, job execution         |
| Networking             | UDP framing and listener abstractions                  | Reliable transport protocols and application-specific connection management |

A few principles follow from these boundaries.

Cancellation safety is operation-specific. A cancellation-safe `next()` does not imply a cancellation-safe `write_all_vectored()` or a rollback of an application operation. The exact future being dropped determines whether bytes, messages, or state can be lost.

Ownership and observation are independent. `TaskTracker` observes completion without owning task abortion. `JoinMap` and `JoinQueue` own tasks and abort on drop. A cancellation token provides a signal but does not observe completion.

Adapters preserve some semantics and intentionally erase others. `StreamReader` preserves bytes but not original stream item boundaries. `SinkWriter` can map writes into sink items, but those items need not match meaningful application messages. `Compat` converts traits but not runtime ownership.

Resource limits must be placed at the correct layer. Codec frame limits, bounded channels, semaphore permits, and worker pools constrain different resources. None alone establishes a comprehensive memory or concurrency budget.

Thread affinity is not the same as nonblocking execution. `LocalPoolHandle` provides places to execute `!Send` futures. `SyncIoBridge` supports synchronous libraries by blocking an OS thread. Neither mechanism makes CPU-intensive or blocking code inherently asynchronous.

These distinctions are particularly important when `tokio-util` is used to remove bespoke machinery: the existing mechanism may encode semantics that must remain explicit even after the custom code disappears.

## Primary-source reference map

The following are the principal entry points for deeper investigation, with plain-text URLs suitable for independent use.

| Subject                           | Primary source                                                                                                                                                        |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Crate overview and API inventory  | [https://docs.rs/tokio-util/latest/tokio_util/](https://docs.rs/tokio-util/latest/tokio_util/?utm_source=chatgpt.com)                                                 |
| Exact crate package and features  | [https://docs.rs/crate/tokio-util/latest](https://docs.rs/crate/tokio-util/latest?utm_source=chatgpt.com)                                                             |
| Release changes                   | [https://github.com/tokio-rs/tokio/blob/master/tokio-util/CHANGELOG.md](https://github.com/tokio-rs/tokio/blob/master/tokio-util/CHANGELOG.md?utm_source=chatgpt.com) |
| Cancellation and guards           | [https://docs.rs/tokio-util/latest/tokio_util/sync/](https://docs.rs/tokio-util/latest/tokio_util/sync/?utm_source=chatgpt.com)                                       |
| Task lifecycle utilities          | [https://docs.rs/tokio-util/latest/tokio_util/task/](https://docs.rs/tokio-util/latest/tokio_util/task/?utm_source=chatgpt.com)                                       |
| Framing and codec contracts       | [https://docs.rs/tokio-util/latest/tokio_util/codec/](https://docs.rs/tokio-util/latest/tokio_util/codec/?utm_source=chatgpt.com)                                     |
| Configurable binary framing       | [https://docs.rs/tokio-util/latest/tokio_util/codec/length_delimited/](https://docs.rs/tokio-util/latest/tokio_util/codec/length_delimited/?utm_source=chatgpt.com)   |
| Asynchronous I/O adapters         | [https://docs.rs/tokio-util/latest/tokio_util/io/](https://docs.rs/tokio-util/latest/tokio_util/io/?utm_source=chatgpt.com)                                           |
| Keyed expiration                  | [https://docs.rs/tokio-util/latest/tokio_util/time/](https://docs.rs/tokio-util/latest/tokio_util/time/?utm_source=chatgpt.com)                                       |
| Futures extensions                | [https://docs.rs/tokio-util/latest/tokio_util/future/](https://docs.rs/tokio-util/latest/tokio_util/future/?utm_source=chatgpt.com)                                   |
| Runtime interoperability          | [https://docs.rs/tokio-util/latest/tokio_util/context/](https://docs.rs/tokio-util/latest/tokio_util/context/?utm_source=chatgpt.com)                                 |
| Tokio/futures trait compatibility | [https://docs.rs/tokio-util/latest/tokio_util/compat/](https://docs.rs/tokio-util/latest/tokio_util/compat/?utm_source=chatgpt.com)                                   |
| Network listener abstractions     | [https://docs.rs/tokio-util/latest/tokio_util/net/](https://docs.rs/tokio-util/latest/tokio_util/net/?utm_source=chatgpt.com)                                         |
| UDP transport framing             | [https://docs.rs/tokio-util/latest/tokio_util/udp/](https://docs.rs/tokio-util/latest/tokio_util/udp/?utm_source=chatgpt.com)                                         |
| Tagged upstream implementation    | [https://github.com/tokio-rs/tokio/tree/tokio-util-0.7.20/tokio-util](https://github.com/tokio-rs/tokio/tree/tokio-util-0.7.20/tokio-util?utm_source=chatgpt.com)     |

### Overall assessment

`tokio-util` is best understood as Tokio's library of reusable asynchronous coordination and interoperability mechanisms.

Its deepest architectural contributions are hierarchical cancellation, task lifetime accounting, configurable and stateful framing, conversion between asynchronous data interfaces, and poll-oriented adaptation of channels and semaphores. The newer task and I/O additions—`JoinMap`, `JoinQueue`, `AbortOnDrop`, `LocalPoolHandle`, `io::simplex`, and vectored I/O helpers—broaden the set of coordination and data-movement patterns that can be expressed without custom infrastructure.

The main design question it helps answer is not how to execute asynchronous Rust—that remains Tokio's domain—but how to represent the boundaries, lifetimes, and interfaces between asynchronous operations using existing, composable mechanisms rather than application-specific state machines.