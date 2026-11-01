# sd-notify — Rust Library Capability Reference

Target: `sd-notify` 0.5.0 (latest stable) · Released March 9, 2026 · Researched October 28, 2026

`sd-notify` is a lightweight Rust interface to systemd's service supervision protocols. It allows Rust applications to communicate readiness, operational state, watchdog heartbeats, failures, and lifecycle transitions to systemd or compatible service managers. It also provides interfaces for receiving socket-activated file descriptors and, optionally, storing descriptors across service restarts.

Its principal architectural value is integrating application-level lifecycle and health semantics into the service manager's existing supervision machinery, avoiding custom readiness signaling, watchdog protocols, and portions of socket-lifecycle management.

The crate implements these capabilities without linking to `libsystemd`. Version 0.5.0 uses `libc` and optionally `sendfd`, so its upstream description as having no dependencies is outdated. Its documented target is Linux, and its minimum supported Rust version (MSRV) is 1.82.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



Primary references:

- Crate documentation: [https://docs.rs/sd-notify/0.5.0/sd_notify/](https://docs.rs/sd-notify/0.5.0/sd_notify/?utm_source=chatgpt.com)
- Upstream repository: [https://github.com/lnicola/sd-notify](https://github.com/lnicola/sd-notify?utm_source=chatgpt.com)
- Versioned source: [https://docs.rs/crate/sd-notify/0.5.0/source/](https://docs.rs/crate/sd-notify/0.5.0/source/?utm_source=chatgpt.com)
- systemd notification protocol: [https://www.freedesktop.org/software/systemd/man/sd_notify.html](https://www.freedesktop.org/software/systemd/man/sd_notify.html?utm_source=chatgpt.com)

## 1. Capability map

| Capability                           | Native interfaces                                                 | Architectural role                                                          |
| ------------------------------------ | ----------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Service readiness                    | `NotifyState::Ready`, `notify`                                    | Signal completion of application initialization                             |
| Lifecycle signaling                  | `Reloading`, `Stopping`, `MonotonicUsec`                          | Coordinate service state transitions with systemd                           |
| Operational status                   | `Status`, `Errno`, `BusError`                                     | Surface application state and failure diagnostics                           |
| Watchdog integration                 | `watchdog_enabled`, `Watchdog`, `WatchdogTrigger`, `WatchdogUsec` | Support service-manager-controlled liveness supervision                     |
| Timeout management                   | `ExtendTimeoutUsec`                                               | Accommodate long-running startup, execution, or shutdown                    |
| Socket activation                    | `listen_fds`, `listen_fds_with_names`                             | Retrieve descriptors provisioned by systemd                                 |
| Descriptor persistence               | `notify_with_fds`, `FdStore`, `FdStoreRemove`, `FdName`           | Transfer descriptors into systemd's persistent FD store; optional feature   |
| Environment and capability detection | `booted`, notification/watchdog environment APIs                  | Discover supervisor-provided facilities and control environment inheritance |
| Protocol extensibility               | `Custom`                                                          | Emit protocol assignments without waiting for new typed variants            |

The central conceptual distinction is between outbound notification and inbound resource activation.

Outbound notifications tell systemd what an application has accomplished or how it is operating. Inbound activation gives the application resources that systemd has already opened or retained. These are complementary protocols, not two directions of a persistent application-management connection.

`sd-notify` is consequently a protocol adapter rather than a service manager, process supervisor, event loop, or general systemd administration library.

## 2. Notification protocol and execution model

### 2.1 Message construction and transmission

The primary interface is:

`notify(state: &[NotifyState<'_>]) -> std::io::Result<()>`

Internally, the implementation:

1. Reads `NOTIFY_SOCKET` from the process environment.
2. Serializes the `NotifyState` slice into newline-separated `KEY=VALUE` assignments.
3. Creates an unbound Unix datagram socket and connects it to the supplied socket path.
4. Sends the serialized assignments as one datagram.
5. Checks that the complete payload was sent.

The slice interface allows several related state changes to be emitted atomically as one message. For example, readiness, the current status string, and a reported main PID can be transmitted together.

Important semantics:

- No supervisor: When `NOTIFY_SOCKET` is absent, `notify()` returns `Ok(())` without sending anything. This supports the same executable running under systemd or independently.
- Transport success: `Ok(())` confirms the local send completed, not that systemd accepted the notification, attributed it to the intended unit, or performed the requested action.
- Synchronous operation: Calls use blocking `std::os::unix::net::UnixDatagram` operations. There is no native asynchronous API.
- Per-call socket creation: The crate constructs a new socket and message for each call rather than maintaining a persistent connection.
- No reply channel: The API cannot query whether a state transition was accepted.

These distinctions are especially relevant when notification success is used as evidence of readiness, supervision, or persisted resources.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Source: [https://docs.rs/sd-notify/0.5.0/src/sd_notify/lib.rs.html](https://docs.rs/sd-notify/0.5.0/src/sd_notify/lib.rs.html?utm_source=chatgpt.com)

### 2.2 Typed notification representation

`NotifyState<'a>` contains variants that serialize directly into the systemd notification protocol.

| Rust variant             | Protocol assignment       |
| ------------------------ | ------------------------- |
| `Ready`                  | `READY=1`                 |
| `Reloading`              | `RELOADING=1`             |
| `Stopping`               | `STOPPING=1`              |
| `Status(&str)`           | `STATUS=...`              |
| `Errno(u32)`             | `ERRNO=...`               |
| `BusError(&str)`         | `BUSERROR=...`            |
| `MainPid(u32)`           | `MAINPID=...`             |
| `Watchdog`               | `WATCHDOG=1`              |
| `WatchdogTrigger`        | `WATCHDOG=trigger`        |
| `WatchdogUsec(u32)`      | `WATCHDOG_USEC=...`       |
| `ExtendTimeoutUsec(u32)` | `EXTEND_TIMEOUT_USEC=...` |
| `MonotonicUsec(i128)`    | `MONOTONIC_USEC=...`      |
| `Custom(&str)`           | Raw protocol assignment   |

Three additional variants are available with the optional `fdstore` feature: `FdStore`, `FdStoreRemove`, and `FdName(&str)`.

The typed variants provide convenient serialization, but they do not enforce the complete protocol semantics. In particular, the caller must still ensure that field combinations and transitions are valid and that string values satisfy systemd's expectations.

Source: [https://docs.rs/sd-notify/0.5.0/sd_notify/enum.NotifyState.html](https://docs.rs/sd-notify/0.5.0/sd_notify/enum.NotifyState.html?utm_source=chatgpt.com)

## 3. Service initialization, reload, and shutdown

### 3.1 Readiness and service dependency coordination

`NotifyState::Ready` reports that initialization has completed.

With `Type=notify`, systemd waits for `READY=1` before considering startup complete and proceeding with dependent unit startup operations.

This establishes a stronger initialization contract than merely checking whether a process was started.

The application determines its readiness criteria, which might include completed configuration loading, established database connections, initialized internal state, and available request-serving infrastructure.

The service manager then provides the dependency coordination.

The relevant systemd settings are `Type=notify`, `NotifyAccess=`, and `TimeoutStartSec=`. Readiness notification is not inherently needed for every service: `Type=exec` can be sufficient when successful process execution is the required startup condition.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+1



### 3.2 Synchronous configuration reloads

`Type=notify-reload` provides a particularly useful extension to basic readiness signaling.

It establishes a protocol for systemd to initiate a reload and wait for the application to confirm completion.

systemd sends reload signal

Normally SIGHUP; configurable with `ReloadSignal=`

Application acknowledges reload initiation

`RELOADING=1` + `MONOTONIC_USEC=...`

Application performs reload

Application signals completion

`READY=1`

systemd completes reload operation

For systemd v253 and newer, the initial reload notification must include a monotonic timestamp. The crate provides `NotifyState::monotonic_usec_now() -> io::Result<NotifyState>`, which reads `CLOCK_MONOTONIC` and constructs the appropriate value.

`Reloading` and `MonotonicUsec` should be sent in the same `notify()` call, followed by a separate `Ready` notification after reconfiguration completes.

This mechanism enables systemd to coordinate reload completion without requiring a separate `ExecReload=` helper that must itself implement synchronous completion detection.

Boundary: `sd-notify` does not receive signals, reload configuration, install signal handlers, or manage concurrent reload requests. Those remain application responsibilities. The systemd 262 documentation also specifies that a `notify-reload` service must have installed its reload signal handler before its initial readiness notification.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+2



Sources:

[https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html?utm_source=chatgpt.com)

[https://www.freedesktop.org/software/systemd/man/latest/sd_notify.html](https://www.freedesktop.org/software/systemd/man/latest/sd_notify.html?utm_source=chatgpt.com)

### 3.3 Status reporting and failure metadata

`Status(&str)` sends human-readable service status through `STATUS=...`. This is useful for conveying initialization progress, current processing phases, degraded operation, or shutdown progress.

`Errno(u32)` and `BusError(&str)` attach structured failure identifiers. They describe an error to the service manager; they do not themselves cause the application to exit or cause systemd to mark the unit failed.

`MainPid(u32)` can report the main PID where process management requires an explicit identification. It does not permit the caller to impersonate that process when sending a notification.

`Stopping` announces the beginning of shutdown. In systemd v259 and newer, entry into the stopping state is final and cannot be reversed by subsequent notifications.

Consequential distinction: Status notification is not a replacement for journald or a structured logging system. It exposes a current service-state description, not an event history or durable stream of diagnostic records.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 4. Watchdog supervision and recovery

Watchdog integration is one of the crate's most useful capabilities, because it connects application-defined liveness checks to systemd's process recovery machinery.

### 4.1 Runtime watchdog discovery

`watchdog_enabled() -> Option<Duration>`

The function reads `WATCHDOG_USEC` and optionally `WATCHDOG_PID`.

It returns the watchdog interval if the supervisor has configured one and the PID restriction permits the current process. Otherwise, it returns `None`.

The interval is returned as a standard Rust `Duration`; therefore, application-side scheduling can work directly with the supervisor's configured interval rather than duplicating that configuration.

The source also reveals that invalid or missing `WATCHDOG_USEC` values produce `None` rather than an error. An unparsable `WATCHDOG_PID` is treated like an absent PID.

### 4.2 Heartbeats and watchdog failure

Three notification variants serve distinct purposes.

| Variant             | Meaning                                                      |
| ------------------- | ------------------------------------------------------------ |
| `Watchdog`          | Heartbeat; refreshes the last-received watchdog timestamp    |
| `WatchdogTrigger`   | Explicitly requests the configured watchdog failure behavior |
| `WatchdogUsec(u32)` | Requests a change to the watchdog interval                   |

When a unit has `WatchdogSec=` configured, systemd expects repeated heartbeats after startup.

The systemd documentation recommends sending them at approximately half the configured watchdog interval to provide scheduling headroom.

If heartbeats are missing, systemd marks the service failed and sends the configured watchdog signal, normally `SIGABRT`. A `Restart=` policy determines whether the service restarts.

`WatchdogTrigger` is a separate mechanism for deliberate escalation when the application detects an unrecoverable health condition. Systemd documents that the trigger can activate watchdog failure handling even without `WatchdogSec=` being enabled.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+2



### 4.3 Liveness semantics and implementation boundaries

The crate performs neither automatic heartbeat scheduling nor health evaluation.

It provides the notification operation and discovery of the configured interval; the application must determine whether and when to send heartbeats.

This is an important architectural distinction. A background timer that continues signaling while the main execution loop has deadlocked may satisfy systemd's timing requirements while failing to provide meaningful application liveness detection.

An implementation can instead couple watchdog signaling to progress through an event loop, processing pipeline, or coordinated set of critical tasks.

Another nuance: `WatchdogUsec` sends a runtime interval update, whereas `watchdog_enabled()` reads the original environment-provided interval. The latter does not automatically reflect an interval updated through a notification. Scheduling must account for that distinction.

Sources:

[https://docs.rs/sd-notify/0.5.0/sd_notify/fn.watchdog_enabled.html](https://docs.rs/sd-notify/0.5.0/sd_notify/fn.watchdog_enabled.html?utm_source=chatgpt.com)

[https://www.freedesktop.org/software/systemd/man/latest/sd_watchdog_enabled.html](https://www.freedesktop.org/software/systemd/man/latest/sd_watchdog_enabled.html?utm_source=chatgpt.com)

### 4.4 Extending service timeouts

`ExtendTimeoutUsec(u32)` supports workloads whose legitimate execution time may exceed the original service-manager deadline.

Relevant stages include startup, reload, shutdown, runtime expiration, and watchdog-related abort handling, subject to systemd's unit-specific rules.

The mechanism is not an unconditional timeout override.

The first extension must arrive before the applicable deadline. Once the operation has exceeded its original timeout, further extension messages must continue to arrive within their requested intervals until the operation completes or terminates.

This provides a useful compromise between static, excessively generous timeouts and premature termination of legitimate long-running operations.

Type limitation: `ExtendTimeoutUsec(u32)` and `WatchdogUsec(u32)` restrict typed payloads to 32-bit microsecond values, although the underlying protocol's numeric representation is not intrinsically limited to Rust `u32`. The `Custom` variant can express a larger valid protocol assignment when necessary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+2



## 5. Socket activation and inherited file descriptors

Socket activation is a substantial capability beyond the crate's naming emphasis on notifications.

Rather than having every application independently bind and manage listening sockets, systemd can create the sockets and pass their descriptors to an activated process.

### 5.1 Descriptor discovery

`listen_fds() -> io::Result<impl ExactSizeIterator<Item = RawFd>>`

The function reads the socket-activation environment and returns the inherited file descriptors as an exact-size iterator.

Its implementation:

- Requires `LISTEN_PID` to identify the current process.
- Reads `LISTEN_FDS` as the number of inherited descriptors.
- Uses the conventional descriptor range beginning at FD 3.
- Applies `FD_CLOEXEC` to each descriptor using `fcntl`.
- Returns an empty iterator when activation is not configured for the process.

It also checks for invalid input and descriptor-count overflow.

`FD_CLOEXEC` prevents the descriptors from being inherited across subsequent program execution unless explicitly adjusted.

The crate returns raw descriptors, not owned sockets or other typed resources. Applications remain responsible for validating their type and constructing correctly owned Rust wrappers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 5.2 Named descriptor discovery

`listen_fds_with_names() -> io::Result<impl ExactSizeIterator<Item = (RawFd, String)>>`

This extension associates each descriptor with a name from the colon-separated `LISTEN_FDNAMES` environment variable.

Names can originate from systemd socket units via `FileDescriptorName=` or from previously stored descriptors via `FDNAME=`.

The crate checks whether the number of names matches the number of descriptors. If no names are supplied, its implementation returns `"unknown"` for each descriptor.

Names are not guaranteed unique, and they should not be treated as a substitute for validating that a descriptor represents the expected resource type.

This distinction matters when a service receives several listening sockets, multiple protocols, or both activation sockets and previously stored state descriptors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://freedesktop.org\&sz=32)

freedesktop.org+2



### 5.3 Integration with Rust networking runtimes

The descriptor retrieval interface is independent of networking libraries and application runtimes.

For example, inherited descriptors can be adapted into appropriate standard-library resource types and subsequently integrated with an asynchronous runtime such as Tokio.

That conversion is not supplied by `sd-notify`.

The application must ensure that ownership is transferred exactly once, that the socket type is appropriate, and that nonblocking mode is configured consistently with the receiving runtime.

There are also systemd-specific lifecycle considerations:

Socket continuity: systemd normally retains its own descriptor for the activated socket. A restarted service can receive the same underlying listening socket, helping preserve the listening endpoint across process restarts.

Shared descriptor state: Changes to certain socket settings can affect both the application and systemd's retained descriptor. For example, calling `shutdown(2)` on a shared socket is generally inappropriate.

Ordering: When several socket units activate one service, descriptor order is not generally guaranteed across units. Naming and type validation are preferable to assuming a fixed descriptor position.

Activation modes: Both conventional shared-listener activation (`Accept=no`) and per-connection activation (`Accept=yes`) use systemd socket mechanisms, but their descriptor semantics differ. `sd-notify` retrieves the descriptors; it does not abstract the distinction.

Sources:

[https://www.freedesktop.org/software/systemd/man/latest/sd_listen_fds.html](https://www.freedesktop.org/software/systemd/man/latest/sd_listen_fds.html?utm_source=chatgpt.com)

[https://www.freedesktop.org/software/systemd/man/latest/systemd.socket.html](https://www.freedesktop.org/software/systemd/man/latest/systemd.socket.html?utm_source=chatgpt.com)

## 6. Persistent file-descriptor store

Optional `fdstore` feature

The FD-store functionality is a particularly valuable advanced feature because it can preserve selected operating-system resources across service-process restarts.

### 6.1 Feature availability and API

The Cargo feature `fdstore` enables the optional `sendfd` dependency and adds these interfaces:

| Interface                       | Role                                                 |
| ------------------------------- | ---------------------------------------------------- |
| `notify_with_fds`               | Send a notification with attached file descriptors   |
| `notify_with_fds_and_unset_env` | Equivalent operation followed by environment cleanup |
| `NotifyState::FdStore`          | Request storage of the attached descriptors          |
| `NotifyState::FdStoreRemove`    | Request removal from the manager's descriptor store  |
| `NotifyState::FdName`           | Supply a descriptor name                             |

`notify_with_fds` accepts `&[BorrowedFd<'_>]`, using Rust's IO-safety borrowing abstraction rather than requiring callers to construct raw descriptor arrays directly.

The underlying `sendfd` integration uses Unix descriptor passing. It does not transfer Rust ownership of the descriptor object away from the caller merely because the notification was sent.

Source: [https://docs.rs/crate/sd-notify/0.5.0/source/Cargo.toml.orig](https://docs.rs/crate/sd-notify/0.5.0/source/Cargo.toml.orig?utm_source=chatgpt.com)

### 6.2 Resource persistence across restart

A service can submit an open descriptor to systemd with `FDSTORE=1`, optionally assigning it a name.

If systemd has been configured to accept stored descriptors, it retains a reference to the underlying resource.

At the next service start or restart, the stored descriptors are supplied through the same descriptor-activation interface used by socket activation.

Running service — generation A

Open socket, memfd, or another supported descriptor

`notify_with_fds(FDSTORE=1, FDNAME=...)`

systemd file-descriptor store

Retains references independently of the service process

Restart and descriptor reactivation

Running service — generation B

`listen_fds_with_names()` retrieves preserved resources

This creates the possibility of preserving state that exists within kernel-managed resources, rather than requiring every restart to reconstruct those resources from scratch.

One important example is a `memfd` containing serialized application state. Other possibilities include selected sockets or communication resources, provided their semantics support the intended restart behavior.

The retained descriptor is a reference to a resource, not a serialized Rust object. Correct reconstruction of application-level state, offsets, data formats, and protocol ownership remains the application's responsibility.

### 6.3 FD-store configuration and limits

Systemd must be configured with `FileDescriptorStoreMax=` greater than zero. Its default is zero.

`FileDescriptorStorePreserve=` determines when entries are retained or released. In current systemd, options include `no`, `restart`, `yes`, and `on-success`.

Additional semantics:

- Stored descriptors can be returned alongside socket-activated descriptors.
- `FDNAME=` assists identification but does not guarantee uniqueness.
- `FDSTOREREMOVE=1` removes stored descriptors selected by name.
- systemd normally removes pollable stored descriptors on error or hangup conditions; `FDPOLL=0` modifies this behavior.
- Persistent references do not automatically make an application restart-safe. The application must ensure state consistency and correct recovery.

There is also a security distinction: the file-descriptor store does not expose its retained descriptor contents directly to unrelated clients through its normal interface, but metadata such as descriptor names may be inspectable.

Recent systemd releases extend some preservation behavior to supported `kexec`-based live-update scenarios. That is a systemd/kernel facility, not functionality implemented by `sd-notify`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://freedesktop.org\&sz=32)

freedesktop.org+2



Sources:

[https://www.freedesktop.org/software/systemd/man/latest/sd_notify.html](https://www.freedesktop.org/software/systemd/man/latest/sd_notify.html?utm_source=chatgpt.com)

[https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html?utm_source=chatgpt.com)

## 7. Environment management and process safety

The crate's environment interfaces deserve attention because several functions changed materially in 0.5.0.

### 7.1 Detecting systemd

`booted() -> io::Result<bool>`

The implementation checks whether `/run/systemd/system` exists as a directory.

This is a lightweight indication that the system was booted with systemd.

It does not establish that the current process belongs to a particular supervised unit, that a notification socket is available, or that watchdog functionality has been configured.

For notification capability, the relevant environment variables provide more specific information.

### 7.2 Environment-removing variants

The crate exposes separate functions for performing operations and subsequently removing their environment variables.

| Function                              | Environment variables removed                |
| ------------------------------------- | -------------------------------------------- |
| `notify_and_unset_env`                | `NOTIFY_SOCKET`                              |
| `watchdog_enabled_and_unset_env`      | `WATCHDOG_USEC`, `WATCHDOG_PID`              |
| `listen_fds_and_unset_env`            | `LISTEN_PID`, `LISTEN_FDS`                   |
| `listen_fds_with_names_and_unset_env` | `LISTEN_PID`, `LISTEN_FDS`, `LISTEN_FDNAMES` |
| `notify_with_fds_and_unset_env`       | `NOTIFY_SOCKET`                              |

These variants can prevent child processes from inheriting supervisor-specific environment settings intended only for the service process.

However, they are marked `unsafe` in 0.5.0 because they mutate process-global environment state using `std::env::remove_var`.

The documented safety requirements generally imply performing these operations before other threads exist, including before asynchronous runtime initialization that creates threads.

There is also a lifecycle tradeoff: removing `NOTIFY_SOCKET` disables subsequent notification calls unless the environment is restored. Consequently, notification-environment cleanup is unsuitable when the same process still needs to emit future watchdog heartbeats or status changes through the ordinary API.

Source: [https://docs.rs/sd-notify/0.5.0/src/sd_notify/lib.rs.html](https://docs.rs/sd-notify/0.5.0/src/sd_notify/lib.rs.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+2



## 8. Extensibility and important limitations

### 8.1 Custom protocol assignments

`NotifyState::Custom(&str)` allows arbitrary assignments to be sent without the crate having to expose a dedicated variant.

This can be useful when newer systemd versions introduce new fields.

For example, systemd's protocol includes additional assignments for runtime notification access, restart-counter reset, and newer error formats.

The distinction is between serialization support and full feature support: a custom string can encode an assignment, but cannot independently provide ancillary descriptor passing, receive an acknowledgment, or emulate a missing system call.

The systemd protocol recommends namespaced `X_` prefixes for application-defined fields to avoid collisions with future reserved fields.

### 8.2 Significant implementation limitations

| Area                         | Verified limitation and consequence                                                                                  |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Supervisor attribution       | Cannot send notifications on behalf of arbitrary PIDs; `MainPid` is a reported field, not a sender-identity override |
| Credentials                  | Does not explicitly send sender credentials using the native libsystemd approach                                     |
| Buffer management            | Does not enlarge the datagram send buffer                                                                            |
| Notification acknowledgment  | Does not implement `sd_notify_barrier`; transport success cannot establish service-manager processing                |
| Abstract Unix sockets        | Source uses `UnixDatagram::connect(path)` without translating a leading `@` into a Linux abstract address            |
| VSOCK notification addresses | No implementation of the `vsock:` address schemes supported by newer systemd protocols                               |
| Async execution              | No native Tokio/async API or automatic background watchdog task                                                      |
| FD validation                | Activation functions return raw descriptors without checking their socket types                                      |
| Protocol validation          | Typed variants serialize values but do not enforce all field combinations or sanitize all supplied strings           |

The address limitations are conclusions from inspection of the 0.5.0 source, rather than statements explicitly documented as unsupported by the maintainer.

They matter because the systemd protocol supports notification addresses beyond pathname-based Unix sockets. In particular, assuming that all valid `NOTIFY_SOCKET` values will work with this crate would be unsafe.

There is an additional version-sensitive distinction: current systemd's descriptor-activation documentation includes `LISTEN_PIDFDID` validation, whereas `sd-notify` 0.5.0 checks `LISTEN_PID` without implementing that additional verification mechanism.

### 8.3 Notification attribution and acknowledgment

Systemd's `NotifyAccess=` setting controls which service processes can submit accepted notifications: `none`, `main`, `exec`, or `all`.

A particularly subtle problem occurs when an auxiliary process sends a notification and immediately exits. Even with `NotifyAccess=all`, systemd may not be able to attribute the message correctly if that process disappears before the message is processed.

The native `sd_notify_barrier()` protocol addresses this by synchronizing message processing through an ancillary file-descriptor handshake.

`sd-notify` does not expose an equivalent barrier operation. Sending `Custom("BARRIER=1")` alone is insufficient because the protocol requires a special file descriptor and acknowledgment behavior.

This is a meaningful distinction for short-lived notification helpers, but is generally less problematic for long-lived, directly supervised service processes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+4



### 8.4 String formatting and input trust

`Status`, `BusError`, and `Custom` accept borrowed strings.

Their formatting is largely direct, without comprehensive protocol-field validation.

The systemd protocol expects a newline-separated collection of assignments, and `STATUS=` is defined as a single-line UTF-8 field. Consequently, newline characters in application-supplied strings can alter the resulting assignment structure.

When strings originate from external or untrusted input, validation or escaping at the application boundary is therefore important.

For similarly high-assurance uses, the `Custom` variant should not be treated as a schema-validated representation of the notification protocol.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



## 9. Integration boundaries and alternative libraries

`sd-notify` intentionally has a small scope. Adjacent libraries can provide functionality it does not attempt to implement.

| Library            | Differentiating capabilities                                                                                                               |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `sd-notify` 0.5.0  | Focused service notifications, watchdog environment discovery, socket activation, optional FD storage                                      |
| `libsystemd` 0.7.2 | Broader pure-Rust systemd client interfaces, including daemon functionality, activation, credentials, logging, and unit-related facilities |
| `systemd` 0.10.1   | Bindings into native systemd functionality, with interfaces including daemon support, journal, bus, login, and unit functionality          |
| `listenfd` 1.0.2   | Specialized inherited-listener management, including adapters for socket activation and local development workflows                        |

`libsystemd` is particularly relevant when broader systemd functionality is required without depending on the native `libsystemd` library.

The `systemd` crate provides a more comprehensive native-library integration pathway.

`listenfd` offers convenience for receiving externally provisioned listeners and is designed to accommodate development workflows involving tools such as `systemfd`. This is a different emphasis from managing application readiness and watchdog signaling.

For general systemd unit administration, D-Bus interfaces and appropriate systemd management tooling remain separate concerns.

References:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



[https://docs.rs/libsystemd/latest/libsystemd/](https://docs.rs/libsystemd/latest/libsystemd/?utm_source=chatgpt.com)

[https://docs.rs/systemd/latest/systemd/](https://docs.rs/systemd/latest/systemd/?utm_source=chatgpt.com)

[https://docs.rs/listenfd/latest/listenfd/](https://docs.rs/listenfd/latest/listenfd/?utm_source=chatgpt.com)

### Native mechanisms that compose particularly well

Three combinations are especially consequential:

Readiness + socket activation. Systemd manages the listening endpoint, while the application reports completion of its higher-level initialization. This separates transport availability from application readiness and reduces the need for custom startup coordination.

Watchdog + restart policy + FD store. Systemd can detect a failure, terminate or restart the service according to unit policy, and make selected preserved resources available to the new process. These mechanisms can support recovery designs that maintain some operating-system resources across process generations. Application-level state correctness remains an independent requirement.

Reload notification + timeout extension. Applications can signal the beginning and completion of reconfiguration while extending systemd's deadline when legitimately required. This enables supervised reload operations with meaningful completion semantics rather than asynchronous fire-and-forget signaling.

These are architectural possibilities arising from the composition of crate functionality and systemd facilities, not guarantees that the crate independently delivers fault-tolerant operation.

## 10. Version-specific changes in 0.5.0

The transition from 0.4.x to 0.5.0 contains API-level changes worth accounting for.

| Change                                                                    | Implication                                                                    |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Environment-clearing parameters separated into dedicated unsafe functions | Callers must distinguish ordinary operations from process-environment mutation |
| `watchdog_enabled` now returns `Option<Duration>`                         | More direct interval handling and clearer absence semantics                    |
| `booted` behavior corrected                                               | A missing systemd runtime directory now produces `Ok(false)`                   |
| Watchdog detection corrected for missing `WATCHDOG_PID`                   | Watchdog capability discovery works without requiring that variable            |
| MSRV established at Rust 1.82                                             | Defines the supported compiler floor                                           |

The preceding 0.4.x releases also introduced important capabilities that remain present in 0.5.0, notably monotonic reload timestamps, file-descriptor passing, and named descriptor discovery.

The upstream changelog dates its 0.5.0 entry January 18, 2025, while the published registry version is dated March 9, 2026. The version and published source, rather than the changelog date, are used here as the release reference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Changelog: [https://github.com/lnicola/sd-notify/blob/master/CHANGELOG.md](https://github.com/lnicola/sd-notify/blob/master/CHANGELOG.md?utm_source=chatgpt.com)

## 11. Technical synthesis

`sd-notify` has a narrow API surface but participates in several consequential system-level mechanisms.

Its strongest capabilities are not the individual notification variants in isolation, but the application-to-service-manager contracts they enable:

- Lifecycle correctness: Application-defined readiness and reload completion become visible to systemd dependency and orchestration machinery.
- Operational supervision: Heartbeats and explicit watchdog failures can connect application health decisions to supervisor-controlled termination and recovery policies.
- Resource continuity: Socket activation and the optional descriptor store can separate selected operating-system resource lifetimes from the lifetime of an individual service process.

The principal constraints are equally clear: it does not implement health monitoring, asynchronous scheduling, complete notification-protocol transport support, service-manager acknowledgments, or full systemd administration.

For these reasons, it is best understood as a compact, application-facing integration layer for systemd's lifecycle and resource-activation protocols, rather than a general-purpose infrastructure runtime.

The most valuable advanced capabilities to retain in a library capability registry are `Type=notify-reload` integration with monotonic timestamps, `watchdog_enabled()` and explicit watchdog triggering, named socket-activation descriptors, and feature-gated descriptor persistence through `notify_with_fds`. These provide the greatest architectural leverage beyond basic `READY=1` signaling.