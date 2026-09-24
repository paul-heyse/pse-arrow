# `tracing_futures::WithDispatch`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_futures.WithDispatch.json).

<a id="op-c3f0d02a9b3d72a9be757115"></a>
## WithDispatch

`struct` · `tracing_futures::WithDispatch` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
struct WithDispatch<T>
```

Source: `src/lib.rs:263`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

A future, stream, sink, or executor that has been instrumented with a
`tracing` subscriber.

<a id="op-77c0f671d1028aad2f552620"></a>
## Error

`assoc_type` · `tracing_futures::WithDispatch::Error` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
Error
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures::future::Future", "path": "futures_01::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [436, 1], "end": [444, 2], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "futures::future::Future", "path": "Future"}, "trait_path": "futures::future::Future"}`

Source: `src/lib.rs:438`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2a2c274c608ed90663489267"></a>
## Item

`assoc_type` · `tracing_futures::WithDispatch::Item` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures::future::Future", "path": "futures_01::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [436, 1], "end": [444, 2], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "futures::future::Future", "path": "Future"}, "trait_path": "futures::future::Future"}`

Source: `src/lib.rs:437`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d1b1bb91d8888359214c7d60"></a>
## Output

`assoc_type` · `tracing_futures::WithDispatch::Output` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "crate::stdlib::future::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [448, 1], "end": [457, 2], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/lib.rs:449`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a72a71249cea26842b737eb0"></a>
## block_on

`function` · `tracing_futures::WithDispatch::block_on` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn block_on<F, R, E>(&mut self, future: F) -> Result<R, E> where F: 'static + Future<Item = R, Error = E>, R: 'static, E: 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tokio::runtime::current_thread::runtime::Runtime", "path": "current_thread::Runtime"}}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 5], "end": [342, 6], "filename": "src/executor/futures_01.rs"}, "trait": null, "trait_path": null}`

Source: `src/executor/futures_01.rs:320`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Runs the provided future in the context of this `WithDispatch`'s trace
dispatcher, blocking the current thread until the future completes.

This function can be used to synchronously block the current thread
until the provided `future` has resolved either successfully or with an
error. The result of the future is then returned from this function
call.

Note that this function will **also** execute any spawned futures on the
current thread, but will **not** block until these other spawned futures
have completed. Once the function returns, any uncompleted futures
remain pending in the `Runtime` instance. These futures will not run
until `block_on` or `run` is called again.

The caller is responsible for ensuring that other spawned futures
complete execution by calling `block_on` or `run`.

This method simply wraps a call to `current_thread::Runtime::block_on`,
instrumenting the spawned future beforehand.

# Panics

This function panics if the executor is at capacity, if the provided
future panics, or if called within an asynchronous execution context.

<a id="op-e101a56ce9e5058b89e08f87"></a>
## block_on

`function` · `tracing_futures::WithDispatch::block_on` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn block_on<F, R, E>(&mut self, future: F) -> Result<R, E> where F: Send + 'static + Future<Item = R, Error = E>, R: Send + 'static, E: Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tokio::runtime::threadpool::Runtime", "path": "tokio::runtime::Runtime"}}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [223, 5], "end": [279, 6], "filename": "src/executor/futures_01.rs"}, "trait": null, "trait_path": null}`

Source: `src/executor/futures_01.rs:258`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Run a future to completion on the Tokio runtime, in the context of this
`WithDispatch`'s trace dispatcher.

This runs the given future on the runtime, blocking until it is
complete, and yielding its resolved result. Any tasks or timers which
the future spawns internally will be executed on the runtime.

This method should not be called from an asynchronous context.

This method simply wraps a call to `tokio::runtime::Runtime::block_on`,
instrumenting the spawned future beforehand.

# Panics

This function panics if the executor is at capacity, if the provided
future panics, or if called within an asynchronous execution context.

<a id="op-fdb64fa275887a4ab8094604"></a>
## clone

`function` · `tracing_futures::WithDispatch::clone` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WithDispatch<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [262, 10], "end": [262, 15], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/lib.rs:262`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b3a10e017a1d0a3ac73df5a4"></a>
## dispatch

`function` · `tracing_futures::WithDispatch::dispatch` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn dispatch(&self) -> &Dispatch
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:470`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Borrows the `Dispatch` that this type is instrumented by.

<a id="op-d8beb9a372e855e631f6c1b0"></a>
## execute

`function` · `tracing_futures::WithDispatch::execute` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn execute(&self, future: F) -> Result<(), ExecuteError<F>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}}], "constraints": []}}, "id": "futures::future::Executor", "path": "Executor"}}}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [], "constraints": [{"args": null, "binding": {"equality": {"type": {"tuple": []}}}, "name": "Item"}, {"args": null, "binding": {"equality": {"type": {"tuple": []}}}, "name": "Error"}]}}, "id": "futures::future::Future", "path": "Future"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [28, 1], "end": [37, 2], "filename": "src/executor/futures_01.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "futures::future::Executor", "path": "Executor"}, "trait_path": "futures::future::Executor"}`

Source: `src/executor/futures_01.rs:33`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-42a1f380fa588e8a3e7e2fdc"></a>
## executor

`function` · `tracing_futures::WithDispatch::executor` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn executor(&self) -> WithDispatch<TaskExecutor>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tokio::runtime::threadpool::Runtime", "path": "tokio::runtime::Runtime"}}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [223, 5], "end": [279, 6], "filename": "src/executor/futures_01.rs"}, "trait": null, "trait_path": null}`

Source: `src/executor/futures_01.rs:276`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Return a handle to the runtime's executor, in the context of this
`WithDispatch`'s trace dispatcher.

The returned handle can be used to spawn tasks that run on this runtime.

The instrumented handle functions identically to a
`tokio::runtime::TaskExecutor`, but instruments the spawned
futures prior to spawning them.

<a id="op-b1cb23beacbd03dc6920a9d8"></a>
## fmt

`function` · `tracing_futures::WithDispatch::fmt` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [262, 17], "end": [262, 22], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/lib.rs:262`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-70b8fe2873e36373989fb139"></a>
## handle

`function` · `tracing_futures::WithDispatch::handle` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn handle(&self) -> WithDispatch<current_thread::Handle>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tokio::runtime::current_thread::runtime::Runtime", "path": "current_thread::Runtime"}}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 5], "end": [342, 6], "filename": "src/executor/futures_01.rs"}, "trait": null, "trait_path": null}`

Source: `src/executor/futures_01.rs:339`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Get a new handle to spawn futures on the single-threaded Tokio runtime,
in the context of this `WithDispatch`'s trace dispatcher.\

Different to the runtime itself, the handle can be sent to different
threads.

The instrumented handle functions identically to a
`tokio::runtime::current_thread::Handle`, but the spawned
futures are run in the context of the trace dispatcher.

<a id="op-c808e56d84c22a2530bfe05a"></a>
## inner

`function` · `tracing_futures::WithDispatch::inner` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn inner(&self) -> &T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:489`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Borrows the wrapped type.

<a id="op-bde28607744856ff029a7152"></a>
## inner_mut

`function` · `tracing_futures::WithDispatch::inner_mut` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_mut(&mut self) -> &mut T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:494`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Mutably borrows the wrapped type.

<a id="op-35a8ec619177197c2acd8532"></a>
## inner_pin_mut

`function` · `tracing_futures::WithDispatch::inner_pin_mut` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_pin_mut(Pin<&mut self>) -> Pin<&mut T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:484`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Get a pinned mutable reference to the wrapped type.

<a id="op-6d12a52eef35987cc501d9a7"></a>
## inner_pin_ref

`function` · `tracing_futures::WithDispatch::inner_pin_ref` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_pin_ref(Pin<&self>) -> Pin<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:477`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Get a pinned reference to the wrapped type.

<a id="op-c5dfacd325f8d27548ea4c53"></a>
## into_inner

`function` · `tracing_futures::WithDispatch::into_inner` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn into_inner(self) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:499`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Consumes the `WithDispatch`, returning the wrapped type.

<a id="op-89e666544907fbe17b019196"></a>
## poll

`function` · `tracing_futures::WithDispatch::poll` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn poll(&mut self) -> futures_01::Poll<Self::Item, Self::Error>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures::future::Future", "path": "futures_01::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [436, 1], "end": [444, 2], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "futures::future::Future", "path": "Future"}, "trait_path": "futures::future::Future"}`

Source: `src/lib.rs:440`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d5d1f067be4fc3b9e89e88ed"></a>
## poll

`function` · `tracing_futures::WithDispatch::poll` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn poll(Pin<&mut self>, cx: &mut Context<'_>) -> stdlib::task::Poll<Self::Output>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "crate::stdlib::future::Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [448, 1], "end": [457, 2], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/lib.rs:451`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30a4f23e067694f5198e3a99"></a>
## spawn

`function` · `tracing_futures::WithDispatch::spawn` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn<F>(&mut self, future: F) -> &mut Self where F: Future<Item = (), Error = ()> + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tokio::runtime::threadpool::Runtime", "path": "tokio::runtime::Runtime"}}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [223, 5], "end": [279, 6], "filename": "src/executor/futures_01.rs"}, "trait": null, "trait_path": null}`

Source: `src/executor/futures_01.rs:233`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Spawn a future onto the Tokio runtime, in the context of this
`WithDispatch`'s trace dispatcher.

This spawns the given future onto the runtime's executor, usually a
thread pool. The thread pool is then responsible for polling the
future until it completes.

This method simply wraps a call to `tokio::runtime::Runtime::spawn`,
instrumenting the spawned future beforehand.

<a id="op-786bc48e33e469cd7a1be69f"></a>
## spawn

`function` · `tracing_futures::WithDispatch::spawn` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn(&mut self, future: F) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}}], "constraints": []}}, "id": "tokio_executor::typed::TypedExecutor", "path": "TypedExecutor"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [210, 5], "end": [221, 6], "filename": "src/executor/futures_01.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tokio_executor::typed::TypedExecutor", "path": "TypedExecutor"}, "trait_path": "tokio_executor::typed::TypedExecutor"}`

Source: `src/executor/futures_01.rs:214`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b05db35386eee1b49b8f85a5"></a>
## spawn

`function` · `tracing_futures::WithDispatch::spawn` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn<F>(&mut self, future: F) -> &mut Self where F: Future<Item = (), Error = ()> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tokio::runtime::current_thread::runtime::Runtime", "path": "current_thread::Runtime"}}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 5], "end": [342, 6], "filename": "src/executor/futures_01.rs"}, "trait": null, "trait_path": null}`

Source: `src/executor/futures_01.rs:287`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Spawn a future onto the single-threaded Tokio runtime, in the context
of this `WithDispatch`'s trace dispatcher.

This method simply wraps a call to `current_thread::Runtime::spawn`,
instrumenting the spawned future beforehand.

<a id="op-e3ae71f5a83a024f09b450e5"></a>
## spawn

`function` · `tracing_futures::WithDispatch::spawn` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn(&mut self, future: Box<dyn Future<Error = (), Item = ()> + Send + 'static>) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tokio_executor::executor::Executor", "path": "Executor"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [196, 5], "end": [208, 6], "filename": "src/executor/futures_01.rs"}, "trait": {"args": null, "id": "tokio_executor::executor::Executor", "path": "Executor"}, "trait_path": "tokio_executor::executor::Executor"}`

Source: `src/executor/futures_01.rs:200`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-149ef56c26f1c703869c5b98"></a>
## spawn_local_obj

`function` · `tracing_futures::WithDispatch::spawn_local_obj` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures_task::spawn::LocalSpawn", "path": "LocalSpawn"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [92, 1], "end": [119, 2], "filename": "src/executor/futures_03.rs"}, "trait": {"args": null, "id": "futures_task::spawn::LocalSpawn", "path": "LocalSpawn"}, "trait_path": "futures_task::spawn::LocalSpawn"}`

Source: `src/executor/futures_03.rs:104`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Spawns a future that will be run to completion.

# Errors

The executor may be unable to spawn tasks. Spawn errors should
represent relatively rare scenarios, such as the executor
having been shut down so that it is no longer able to accept
tasks.

<a id="op-ecf7aac3f876bbc0d4a3e06b"></a>
## spawn_obj

`function` · `tracing_futures::WithDispatch::spawn_obj` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures_task::spawn::Spawn", "path": "Spawn"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [33, 1], "end": [60, 2], "filename": "src/executor/futures_03.rs"}, "trait": {"args": null, "id": "futures_task::spawn::Spawn", "path": "Spawn"}, "trait_path": "futures_task::spawn::Spawn"}`

Source: `src/executor/futures_03.rs:45`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Spawns a future that will be run to completion.

# Errors

The executor may be unable to spawn tasks. Spawn errors should
represent relatively rare scenarios, such as the executor
having been shut down so that it is no longer able to accept
tasks.

<a id="op-4337b52b633a034fb312970c"></a>
## status

`function` · `tracing_futures::WithDispatch::status` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn status(&self) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}}], "constraints": []}}, "id": "tokio_executor::typed::TypedExecutor", "path": "TypedExecutor"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [210, 5], "end": [221, 6], "filename": "src/executor/futures_01.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tokio_executor::typed::TypedExecutor", "path": "TypedExecutor"}, "trait_path": "tokio_executor::typed::TypedExecutor"}`

Source: `src/executor/futures_01.rs:218`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cae72be37fde8dc9f4b7b82c"></a>
## status

`function` · `tracing_futures::WithDispatch::status` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn status(&self) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures_task::spawn::Spawn", "path": "Spawn"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [33, 1], "end": [60, 2], "filename": "src/executor/futures_03.rs"}, "trait": {"args": null, "id": "futures_task::spawn::Spawn", "path": "Spawn"}, "trait_path": "futures_task::spawn::Spawn"}`

Source: `src/executor/futures_03.rs:57`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Determines whether the executor is able to spawn new tasks.

This method will return `Ok` when the executor is *likely*
(but not guaranteed) to accept a subsequent spawn attempt.
Likewise, an `Err` return means that `spawn` is likely, but
not guaranteed, to yield an error.

<a id="op-bc23a38b55fec956500cd9e2"></a>
## status_local

`function` · `tracing_futures::WithDispatch::status_local` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn status_local(&self) -> Result<(), SpawnError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "crate::WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "futures_task::spawn::LocalSpawn", "path": "LocalSpawn"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [92, 1], "end": [119, 2], "filename": "src/executor/futures_03.rs"}, "trait": {"args": null, "id": "futures_task::spawn::LocalSpawn", "path": "LocalSpawn"}, "trait_path": "futures_task::spawn::LocalSpawn"}`

Source: `src/executor/futures_03.rs:116`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Determines whether the executor is able to spawn new tasks.

This method will return `Ok` when the executor is *likely*
(but not guaranteed) to accept a subsequent spawn attempt.
Likewise, an `Err` return means that `spawn` is likely, but
not guaranteed, to yield an error.

<a id="op-ab795c8bc83c4950f9a38b60"></a>
## with_dispatch

`function` · `tracing_futures::WithDispatch::with_dispatch` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn with_dispatch<U>(&self, inner: U) -> WithDispatch<U>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_futures::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [502, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:462`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Wrap a future, stream, sink or executor with the same subscriber as this WithDispatch.
