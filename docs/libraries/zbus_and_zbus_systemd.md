# zbus + zbus_systemd — Comprehensive Rust Library Reference

Research date: October 28, 2026 | Target versions: `zbus 6.1.0` and `zbus_systemd 0.26201.0`

## 1. Architectural overview

`zbus` is a pure-Rust implementation of D-Bus communication, including its wire format, typed client interfaces, service implementations, asynchronous messaging, and protocol machinery. `zbus_systemd` builds on it with generated Rust interfaces for the systemd service ecosystem.

Together, they provide a unified approach to Linux service communication, introspection, lifecycle control, configuration, and event observation without requiring C-based D-Bus bindings or subprocess wrappers around system administration commands.

An important version distinction: zbus 6, released October 16, 2026, substantially reorganized the library. The current stable 6.1 release consolidates the former `zvariant` and `zbus_names` functionality into `zbus`, changes runtime integration, and removes the dedicated blocking API. References targeting zbus 5 require careful interpretation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### 1.1 Layered architecture

Application and system-management logic

State observation · lifecycle operations · configuration · automation · custom IPC

zbus_systemd

Generated, typed client proxies for systemd's D-Bus interfaces, including units, jobs, sessions, networking, DNS, and system configuration.

zbus high-level APIs

Generic client proxies, exported services, D-Bus methods, properties, signals, and object management.

zbus transport, message, and wire layers

Connections · authentication · match rules · message streams · Serde serialization · D-Bus types

D-Bus broker and service processes

System/user buses · systemd PID 1 · logind · networkd · resolved · other D-Bus services

The principal separation is between protocol capabilities and service semantics:

| Component                  | Primary responsibility                                                                | Important boundary                                                                      |
| -------------------------- | ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `zbus`                     | D-Bus connections, routing, method calls, signals, exported interfaces, serialization | Does not define the behavior of systemd or other services                               |
| `zbus_systemd`             | Compile-time-generated client proxies for systemd D-Bus APIs                          | Does not implement systemd's service-management engine                                  |
| `zbus_xml` / `zbus_xmlgen` | Introspection XML parsing and generated interface bindings                            | Developer tooling, distinct from runtime systemd functionality                          |
| systemd services           | Units, jobs, sessions, networking, DNS, system configuration, and other operations    | Actual behavior, authorization, and supported interfaces depend on the running services |

### 1.2 Capability map

| Functional family        | Important mechanisms                                                               | Primary library                                      |
| ------------------------ | ---------------------------------------------------------------------------------- | ---------------------------------------------------- |
| Typed service invocation | Generated proxies, methods, property accessors                                     | Both                                                 |
| Event-driven observation | Signal streams, match rules, cached-property notifications                         | Both                                                 |
| Service implementation   | `#[interface]`, `ObjectServer`, signal emission                                    | `zbus`                                               |
| Dynamic IPC              | Low-level messages, dynamic proxies, runtime paths                                 | `zbus`                                               |
| Structured wire data     | D-Bus signatures, Serde, variants, dictionaries, file descriptors                  | `zbus`                                               |
| Advanced connectivity    | Peer-to-peer, transport selection, authentication, runtime integration             | `zbus`                                               |
| Service lifecycle        | Unit operations, jobs, transient units, dependency and unit-file management        | `zbus_systemd`                                       |
| System observation       | Unit properties, sessions, seats, processes, network links, DNS, resource pressure | `zbus_systemd`                                       |
| System configuration     | Host identity, locale, time, networking, resolver interfaces                       | `zbus_systemd`                                       |
| Interface generation     | Rust bindings derived from introspection definitions                               | `zbus_xmlgen`; generation workflow in `zbus_systemd` |

Combined architectural value: `zbus` supplies one reusable typed IPC substrate, while `zbus_systemd` replaces substantial amounts of manually maintained systemd-specific D-Bus interface code. The same connection, message types, asynchronous execution model, and event-stream concepts remain available when moving between generated systemd interfaces and custom D-Bus APIs.

Primary documentation:

- zbus 6 migration: [https://z-galaxy.github.io/zbus/upgrading-to-6.html](https://z-galaxy.github.io/zbus/upgrading-to-6.html?utm_source=chatgpt.com)
- zbus source: [https://github.com/z-galaxy/zbus](https://github.com/z-galaxy/zbus?utm_source=chatgpt.com)
- zbus_systemd source: [https://github.com/lucab/zbus_systemd](https://github.com/lucab/zbus_systemd?utm_source=chatgpt.com)

## 2. zbus: Connection, transport, and runtime architecture

### 2.1 D-Bus identity and addressing

A D-Bus invocation is identified by several independent dimensions:

| Dimension   | Purpose                              | Example                              |
| ----------- | ------------------------------------ | ------------------------------------ |
| Bus         | Communication domain                 | System bus or user/session bus       |
| Destination | Service owning the target object     | `org.freedesktop.systemd1`           |
| Object path | Particular object instance           | `/org/freedesktop/systemd1`          |
| Interface   | Contract implemented by that object  | `org.freedesktop.systemd1.Manager`   |
| Member      | Method, signal, or property          | `StartUnit`, `JobRemoved`, `Version` |
| Signature   | Serialized argument/result structure | `ss`, `o`, `a(sv)`                   |

A well-known service name identifies a service independently of its current connection, whereas a unique bus name identifies a particular connection. Object paths are not filesystem paths; they identify runtime objects that can appear and disappear.

These distinctions matter for lifecycle handling. A unit name, a unit object path, a job object path, and a service-owner identity are related but not interchangeable. Cached objects and subscriptions can become obsolete when a service restarts or ownership changes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy+1



Source: [https://z-galaxy.github.io/zbus/concepts.html](https://z-galaxy.github.io/zbus/concepts.html?utm_source=chatgpt.com)

### 2.2 Connection and transport capabilities

`Connection` is the central reusable communication object. Higher-level proxies and lower-level message streams operate over it.

| Mechanism                                      | Capabilities and implications                                                                      |
| ---------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `Connection::system()`                         | Access system-wide services, including systemd PID 1, logind, resolved, and networkd where present |
| `Connection::session()`                        | Access per-user services, including the user systemd manager where exposed                         |
| `connection::Builder::address(...)`            | Connect to an explicit D-Bus address instead of relying on conventional bus discovery              |
| `Builder::unix_stream(...)`, `tcp_stream(...)` | Establish communication over caller-provided Unix or TCP streams                                   |
| `Builder::socket(...)`                         | Extend transport support through zbus's socket abstraction                                         |
| `Builder::authenticated_socket(...)`           | Integrate an already-authenticated connection into zbus                                            |
| `Builder::p2p()` / `server(guid)`              | Establish brokerless D-Bus client/server communication; requires the `p2p` feature                 |
| `request_name`, `release_name`                 | Control well-known bus-name ownership and service publication                                      |
| `peer_creds()`                                 | Inspect available connection-level peer credentials                                                |

Bus-mediated versus peer-to-peer: A bus provides naming, message routing, and signal subscription infrastructure. A peer-to-peer connection carries the same D-Bus object/method/signal model without a broker, but bus-specific operations such as requesting a well-known bus name are inapplicable.

This makes `zbus` useful beyond Linux system administration: it can implement a private typed IPC protocol between processes while retaining the D-Bus tooling and serialization model. Conversely, `zbus_systemd` normally targets established services on the system or user bus, not private peer connections.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/connection.html](https://z-galaxy.github.io/zbus/connection.html?utm_source=chatgpt.com)

### 2.3 Runtime integration in zbus 6

Version 6 unifies communication around an asynchronous API rather than maintaining separate synchronous and asynchronous proxy implementations.

| Runtime pathway         | Behavior                                                              | Important condition                                                     |
| ----------------------- | --------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `default-rt`            | Uses the native `zruntime` backend                                    | Enabled in default zbus configuration                                   |
| `tokio`                 | Uses Tokio when a Tokio runtime is current at connection construction | Executor selection occurs per connection                                |
| `zbus::block_on`        | Drives async zbus operations from synchronous entrypoints             | Not a replacement for invoking nested blocking calls inside async tasks |
| `tokio-multithread`     | Makes `block_on` use a multithreaded Tokio runtime                    | Separate opt-in feature                                                 |
| `Builder::runtime(...)` | Accepts a custom implementation of `runtime::traits::Runtime`         | Useful for application-specific event loops                             |

A custom runtime supplies scheduling, timer and I/O-readiness functionality; it is not merely an adapter to await a future. This provides an extension path for programs whose event loops are not based on Tokio or the default zruntime backend.

Consequential distinction: Merely enabling both Tokio and zruntime Cargo features does not make every connection run on Tokio. A connection constructed outside an active Tokio runtime can select zruntime instead. Tokio-dependent operations invoked inside its service callbacks can consequently fail if they assume Tokio context.

The former `zbus::blocking`, blocking proxy types, and `blocking-api` feature have been removed in 6.x. `zbus::block_on` should generally enclose the asynchronous operation as a whole, not be called recursively or inside service-handler tasks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy+2



Source: [https://z-galaxy.github.io/zbus/faq.html](https://z-galaxy.github.io/zbus/faq.html?utm_source=chatgpt.com)

### 2.4 Build-time feature partitioning

zbus 6 exposes meaningful compilation boundaries:

| Feature               | Functionality                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------------- |
| `comms`               | Connection and message communication APIs                                                                     |
| `proxy`               | Client proxies, `#[proxy]`, standard D-Bus client proxies                                                     |
| `service`             | Service implementations, `#[interface]`, `ObjectServer`                                                       |
| `object-manager`      | Service-side implementation of the standard object-management interface                                       |
| `tracing`             | Structured diagnostic integration with the Rust `tracing` ecosystem                                           |
| `p2p`                 | Brokerless peer-to-peer operation                                                                             |
| `default-rt`, `tokio` | Runtime backend availability                                                                                  |
| No default features   | Standalone D-Bus wire format, type signatures, and validated name types without communications infrastructure |

The last option is easily overlooked: zbus can be used strictly as a wire-format and type-system library, without introducing its connection stack.

Client-only and service-only builds can omit the opposite half. This is more consequential than simply removing unused exported methods: those capability families are now separately gated at compilation time.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/upgrading-to-6.html](https://z-galaxy.github.io/zbus/upgrading-to-6.html?utm_source=chatgpt.com)

## 3. Typed clients, signals, and state observation

### 3.1 Typed proxies and dynamic invocation

The `#[proxy]` macro generates a client implementation from a Rust trait describing the remote D-Bus interface.

It supplies more than ordinary method-call wrappers:

| Generated mechanism          | Function                                                                            |
| ---------------------------- | ----------------------------------------------------------------------------------- |
| `FooProxy::new(&connection)` | Construct a proxy using declared default destination/path/interface                 |
| `FooProxy::builder(...)`     | Override path and destination for dynamically identified objects                    |
| Ordinary methods             | Serialize arguments, send requests, deserialize replies                             |
| `#[zbus(property)]`          | Typed getters and setters through the standard Properties interface                 |
| `#[zbus(signal)]`            | Typed signal wrappers, argument structures, and receiver streams                    |
| `#[zbus(object = "...")]`    | Express methods returning paths to objects represented by another proxy             |
| `inner()` / `into_inner()`   | Access the underlying generic `zbus::Proxy` for operations beyond generated methods |

The generic `Proxy` and `Connection::call_method` remain available where interface definitions are unknown, incomplete, version-dependent, or determined at runtime.

This creates a useful continuum between fully generated systemd bindings, application-defined typed proxies, and dynamic message-level communication. An application need not introduce a separate IPC library when a service presents an interface not covered by `zbus_systemd`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/client.html](https://z-galaxy.github.io/zbus/client.html?utm_source=chatgpt.com)

### 3.2 Signals are streams, not request/reply calls

Generated signal receivers, such as `receive_job_removed()`, return asynchronous streams. A received signal exposes typed arguments and relevant message metadata.

More advanced capabilities include:

- Argument-based filtering: generated `receive_*_with_args` methods and underlying match rules can select relevant signals without requiring every consumer to inspect all message bodies.
- Generic message filtering: `MessageStream::for_match_rule` receives messages matching a specified rule, including facilities beyond those covered by a generated proxy.
- Bus-side signal subscription: signal match rules are registered with the bus and deregistered when streams are disposed of, with explicit asynchronous cleanup available where required.
- Shared connections: multiple typed proxies and streams can operate over the same connection, preserving a common communication context.

A signal subscription is not a durable event log. Stream queues are bounded, message consumption depends on scheduling, and connection failure or service restart can invalidate assumptions about continuity. Queue capacity and connection lifecycle therefore matter for high-volume event streams. In zbus 6, queue sizes use `NonZeroUsize`; queue-management behavior differs between the main connection queue and filtered-stream queues.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Source: [https://z-galaxy.github.io/zbus/client.html](https://z-galaxy.github.io/zbus/client.html?utm_source=chatgpt.com)

### 3.3 Property caching and consistency

Properties behave differently from methods:

| Operation                         | Semantics                                                            |
| --------------------------------- | -------------------------------------------------------------------- |
| Property getter                   | Read a typed property, normally using the proxy's cache when enabled |
| Property setter                   | Write through `org.freedesktop.DBus.Properties.Set`                  |
| `receive_<property>_changed()`    | Observe property-change notifications                                |
| `cache_properties`                | Configure property caching behavior                                  |
| `uncached_properties`             | Exclude selected properties from the cache                           |
| `emits_changed_signal` annotation | Describe a property's change-notification contract                   |

By default, zbus proxies cache properties and update their cached values in response to change signals. That reduces repeated bus round trips, but introduces an important dependency: a remotely changing property that does not emit suitable notifications can appear permanently unchanged in the cache.

There are three distinct cases:

1. A property is static or emits reliable change notifications: caching is generally useful.
2. A property changes but the service deliberately does not emit notifications: uncached reads can be necessary.
3. Several related properties change together: independently received notifications do not by themselves establish an atomic multi-property snapshot.

For event-driven state models, cache invalidation, initial-state enumeration, and re-enumeration after reconnection remain application-level responsibilities. The existence of a signal stream does not guarantee a lossless state history.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy+1



Source: [https://z-galaxy.github.io/zbus/faq.html](https://z-galaxy.github.io/zbus/faq.html?utm_source=chatgpt.com)

### 3.4 Standard freedesktop interfaces and object graphs

zbus also exposes typed helpers for standard D-Bus conventions:

| Interface              | Role                                                               |
| ---------------------- | ------------------------------------------------------------------ |
| `org.freedesktop.DBus` | Bus names, ownership, activation, and daemon-level queries         |
| `Introspectable`       | Runtime XML description of an object's interfaces                  |
| `Properties`           | Generic property reading, writing, and change notification         |
| `Peer`                 | Basic peer operations                                              |
| `ObjectManager`        | Enumerating and tracking a managed hierarchy of objects/interfaces |

The ObjectManager pattern is particularly valuable for services exposing changing sets of related objects. `GetManagedObjects` provides a structured object/interface/property snapshot, while `InterfacesAdded` and `InterfacesRemoved` signals describe changes to that hierarchy.

Its availability is a property of the remote service: zbus can interact with ObjectManager, but cannot assume every external service implements it.

This pattern can replace bespoke object-enumeration protocols for services that follow the convention.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/upgrading-to-6.html](https://z-galaxy.github.io/zbus/upgrading-to-6.html?utm_source=chatgpt.com)

## 4. Implementing D-Bus services with zbus

### 4.1 `#[interface]` and `ObjectServer`

The service-side model mirrors typed client proxies. A Rust implementation annotated with `#[interface]` defines methods, properties, and signals of a D-Bus interface.

The library handles method dispatch, argument conversion, replies, property integration, and introspection metadata.

| Capability                  | Mechanism                                 | Architectural significance                                           |
| --------------------------- | ----------------------------------------- | -------------------------------------------------------------------- |
| Static service registration | `connection::Builder::serve_at(...)`      | Register objects before publishing the service name                  |
| Dynamic object registration | `ObjectServer::at(...)` and `remove(...)` | Expose runtime-created and runtime-destroyed objects                 |
| Typed method dispatch       | `#[interface]` implementation methods     | Avoid manually parsing message headers for routine services          |
| Property implementation     | `#[zbus(property)]`                       | Support the standard D-Bus Properties contract                       |
| Signal generation           | `#[zbus(signal)]`                         | Generate signal-emitting methods                                     |
| Signal access               | `SignalEmitter` and `InterfaceRef`        | Emit notifications from internal state transitions                   |
| Custom protocol errors      | `#[derive(DBusError)]`                    | Map Rust error variants to D-Bus error replies                       |
| Paired client generation    | `#[interface(proxy(...))]`                | Generate a compatible proxy from a locally declared service contract |

For dynamic objects, an `ObjectServer` can provide common infrastructure for many instances of the same interface. This is especially useful where an application exposes structured resources rather than a single command endpoint.

Registration ordering matters: For activatable services, exporting the interface before acquiring the bus name prevents clients from reaching a service that has advertised its name but has not yet registered its objects.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/service.html](https://z-galaxy.github.io/zbus/service.html?utm_source=chatgpt.com)

### 4.2 Property notification, concurrency, and object lifecycle

A setter implemented through `#[zbus(property)]` can automatically trigger the corresponding change notification. When state changes outside the setter—for example, from a worker or external event—the service must explicitly emit the appropriate notification.

The service-side `ObjectManager` can additionally emit `InterfacesAdded` and `InterfacesRemoved` for objects registered under a managed hierarchy. This allows callers to maintain a client-side object registry without a custom discovery protocol.

Service concurrency requires deliberate ownership design. Methods taking mutable access to an interface object can serialize access to its state; holding that access across long-running awaits may block other operations requiring the same interface. Long-running work can instead be represented through separate jobs, background tasks, or state properties with completion signals, according to the interface contract.

These are not systemd-specific mechanisms. They support arbitrary D-Bus services written in Rust, and can be combined with clients built from the same zbus ecosystem.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy+1



Source: [https://z-galaxy.github.io/zbus/service.html](https://z-galaxy.github.io/zbus/service.html?utm_source=chatgpt.com)

## 5. D-Bus wire types, structured values, and serialization

### 5.1 The unified zbus 6 type system

In zbus 6, the former `zvariant` and `zbus_names` libraries are incorporated into `zbus`.

| Functional area             | Important APIs                                                      |
| --------------------------- | ------------------------------------------------------------------- |
| Wire encoding               | `zbus::wire::to_bytes`, `serialized::Context`, `LE`, `BE`           |
| Wire decoding               | Serialized data and `deserialize()`                                 |
| Static type information     | `Type`, `Signature`, `signature!`                                   |
| Heterogeneous values        | `Variant`, `OwnedVariant`                                           |
| D-Bus names and identifiers | `zbus::names`, `ObjectPath`, `OwnedObjectPath`                      |
| Structured dictionaries     | `SerializeDict`, `DeserializeDict`                                  |
| Serde integration           | `Serialize`, `Deserialize`, `Type`, specialized variant conversions |
| Unix descriptor values      | `Fd`, `OwnedFd`                                                     |
| Message framing             | `message::Message`, `Header`, `Body`                                |

This subsystem is useful independently of the higher-level IPC abstractions. A Rust type can have a D-Bus signature and a deterministic serialization representation without being associated with a particular remote service.

Version 6 relocates codecs into `zbus::wire`, renames `Value` and `OwnedValue` to `Variant` and `OwnedVariant`, and unifies formerly distinct error types. The deprecated `zbus::zvariant` compatibility module remains temporarily available but should not be mistaken for a separate serialization engine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/upgrading-to-6.html](https://z-galaxy.github.io/zbus/upgrading-to-6.html?utm_source=chatgpt.com)

### 5.2 Consequential signature distinctions

D-Bus uses a comparatively small structural type system, making exact signatures important.

| Signature       | Meaning                                       | Common role                                    |
| --------------- | --------------------------------------------- | ---------------------------------------------- |
| `s`             | String                                        | Unit name, service name, textual state         |
| `o`             | Object path                                   | Remote unit, job, session, or machine object   |
| `v`             | Variant containing another typed value        | Polymorphic properties                         |
| `as`            | Array of strings                              | Names, domains, filters                        |
| `a{sv}`         | String-to-variant dictionary                  | Extensible property records                    |
| `a(sv)`         | Array of string/variant structures            | systemd unit-property setting operations       |
| `a{oa{sa{sv}}}` | Object path → interface → property dictionary | Standard ObjectManager snapshot                |
| `h`             | File descriptor handle                        | Inhibitors, devices, image import/export, PTYs |

Two materially different systemd data models are often confused:

- `a{sv}` is a dictionary. It naturally supports keys associated with heterogeneous values.
- `a(sv)` is an array of structures, not a dictionary. It preserves the array representation and can support ordering or repeated property entries where the service permits them.

In particular, `StartTransientUnit` and `SetUnitProperties` accept `a(sv)`. Replacing this with a Rust map merely because both contain names and variant values would change the D-Bus signature and violate the API contract.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

### 5.3 Strongly typed dictionaries

zbus supports deriving `SerializeDict` and `DeserializeDict` for Rust structures representing `a{sv}` and related nested dictionary formats.

This enables a useful intermediate position between unstructured maps and rigid, fully specified structs:

| Technique                 | Value                                                               |
| ------------------------- | ------------------------------------------------------------------- |
| Typed fields              | Compile-time representation of known property names and value types |
| Optional fields           | Represent legitimately absent dictionary entries                    |
| Renaming annotations      | Preserve external D-Bus property naming conventions                 |
| Nested dictionary derives | Model nested interface/property structures                          |
| `serde(flatten)`          | Preserve unknown or extension properties in a supplementary map     |
| `as_variant` helpers      | Encode strongly typed fields as D-Bus variants                      |

The resulting structure can retain type safety for known fields without discarding unfamiliar fields from an evolving service contract.

However, D-Bus does not have a native nullable value. Its conventions must be handled explicitly. `Optional<T>` uses a designated sentinel value; the optional `option-as-array` feature represents absence as a zero-element array and presence as a single-element array.

These are different wire representations. The latter requires agreement between sender and receiver and is not universally supported by property methods.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/faq.html](https://z-galaxy.github.io/zbus/faq.html?utm_source=chatgpt.com)

### 5.4 Enum, bitflag, and schema-evolution semantics

Serde's default enum representation does not necessarily match the numeric or string values expected by a D-Bus interface.

For example, a unit enum may serialize by variant index instead of an explicitly intended numeric discriminant. `serde_repr` can encode numeric representations, while a string-encoded enum requires an appropriate D-Bus string signature. A catch-all string variant can preserve unfamiliar values introduced by newer service versions.

Similarly, D-Bus has no separate bitflag type: flags are represented by integers. A Rust newtype with `bitflags` and corresponding serialization/type implementations can preserve the integer signature while exposing type-safe flag operations.

An especially important forward-compatibility choice is whether unknown flag bits or state strings are retained or rejected. Strict decoding can be desirable for controlled protocols, but often creates unnecessary incompatibility with newer systemd releases.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy+1



Source: [https://z-galaxy.github.io/zbus/faq.html](https://z-galaxy.github.io/zbus/faq.html?utm_source=chatgpt.com)

### 5.5 Unix file descriptors and ownership

D-Bus can transfer Unix file descriptors alongside serialized messages.

Unlike ordinary scalar parameters, these represent live operating-system resources. Their lifetime changes application behavior:

- A message owns descriptors received with that message.
- Deserializing into an owned descriptor allows it to outlive the message.
- Dropping the final owned descriptor closes the corresponding local handle.
- Some systemd APIs intentionally use descriptor lifetime as part of their control protocol.

Examples include logind inhibitor locks, controlled session devices, importd image streams, and machined terminal handles.

A file descriptor therefore cannot always be treated as an inert method-result value. Its ownership and eventual closure may constitute an operation on the remote service.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.login1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.login1.html?utm_source=chatgpt.com)

### 5.6 Low-level message handling and diagnostics

Advanced applications can operate below generated proxies using `Message`, `MessageStream`, `MatchRule`, and connection-level send/reply facilities.

This allows inspection of message headers, types, sender/destination information, signatures, and serialized bodies. It is relevant for protocol diagnostics, generic D-Bus inspection tools, custom routers, and interfaces whose schema is not known at compilation time.

`Message::recv_position` additionally provides a receive-ordering value meaningful within one connection. It is distinct from the protocol message serial number, which belongs to the sending peer and should not be treated as a globally ordered event identifier.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://z-galaxy.github.io/zbus/service.html](https://z-galaxy.github.io/zbus/service.html?utm_source=chatgpt.com)

## 6. zbus_systemd: Generated interface coverage

`zbus_systemd` is fundamentally a collection of generated typed D-Bus client bindings.

The crate tracks systemd interface definitions and organizes the bindings into separately enabled Cargo features. It is not a separate systemd implementation, process supervisor, command-line frontend, or higher-level desired-state orchestration engine.

### 6.1 The 14 service families

| Cargo feature | Daemon / interface              | Important capabilities                                                                       |
| ------------- | ------------------------------- | -------------------------------------------------------------------------------------------- |
| `systemd1`    | system and user service manager | Units, jobs, service lifecycle, transient units, unit files, dependencies, resource controls |
| `login1`      | `systemd-logind`                | Sessions, users, seats, power management, inhibitors, session devices                        |
| `network1`    | `systemd-networkd`              | Managed links, network state and configuration, DHCP client/server information               |
| `resolve1`    | `systemd-resolved`              | DNS and hostname resolution, link-specific DNS settings, resolver status                     |
| `home1`       | `systemd-homed`                 | User/home records, creation and activation, home-directory lifecycle                         |
| `machine1`    | `systemd-machined`              | Virtual-machine/container registrations, images, namespaces, process access                  |
| `import1`     | `systemd-importd`               | Image import, export, downloads, OCI pulls, transfer progress                                |
| `portable1`   | `systemd-portabled`             | Portable-service image attachment, metadata, extensions, image lifecycle                     |
| `hostname1`   | `systemd-hostnamed`             | Static/transient hostnames, machine identity and metadata                                    |
| `locale1`     | `systemd-localed`               | System locale and keyboard configuration                                                     |
| `timedate1`   | `systemd-timedated`             | Clock, timezone, RTC configuration, NTP enablement                                           |
| `timesync1`   | `systemd-timesyncd`             | Network time synchronization state and selected time server                                  |
| `oom1`        | `systemd-oomd`                  | Userspace OOM diagnostic state and kill notifications                                        |
| `sysupdate1`  | `systemd-sysupdated`            | Update targets, version discovery, acquisition/install jobs                                  |

These 14 features are independently selectable. Their presence in the crate does not establish that the corresponding systemd service is installed, running, enabled, or available to the caller.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Source: [https://docs.rs/crate/zbus_systemd/latest](https://docs.rs/crate/zbus_systemd/latest?utm_source=chatgpt.com)

### 6.2 What the generated bindings expose

The generated proxy objects typically provide:

| Surface                        | Example                                                     |
| ------------------------------ | ----------------------------------------------------------- |
| Manager interface              | `systemd1::ManagerProxy`, `login1::ManagerProxy`            |
| Resource-specific interface    | `systemd1::UnitProxy`, `ServiceProxy`, `TimerProxy`         |
| Job or transfer interface      | `systemd1::JobProxy`, `import1::TransferProxy`              |
| Typed event stream             | `JobRemovedStream`, `TransferRemovedStream`                 |
| Typed event argument structure | `JobRemovedArgs`, `TransferRemovedArgs`                     |
| Generic-proxy access           | `inner()`, `into_inner()`, customizable proxy builder       |
| Cross-interface reuse          | Shared `zbus::Connection`, D-Bus names and serialized types |

A generated method maps to a documented remote interface method, not necessarily to a more abstract Rust operation. Complex systemd replies may therefore remain tuples, vectors, variants, and object paths corresponding closely to their D-Bus signatures.

The chief benefit is eliminating repetitive interface-binding machinery. The library does not eliminate the need to understand the underlying systemd contracts, particularly multi-step jobs, authorization, object lifetime, and version availability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

docs.rs



Source: [https://docs.rs/zbus_systemd/latest/zbus_systemd/all.html](https://docs.rs/zbus_systemd/latest/zbus_systemd/all.html?utm_source=chatgpt.com)

### 6.3 Async runtime features and integration

`zbus_systemd 0.26201.0` depends on `zbus ^6` with zbus's default features disabled.

It provides two explicit runtime-selection features:

| Feature               | Effect                              |
| --------------------- | ----------------------------------- |
| `zbus-async-tokio`    | Enables zbus's Tokio backend        |
| `zbus-async-zruntime` | Enables zbus's `default-rt` backend |

Where another dependency enables an appropriate zbus runtime feature, Cargo feature unification can provide it without specifying a redundant `zbus_systemd` runtime selection. Alternatively, a caller can supply its own zbus runtime implementation.

The earlier `zbus-async-smol` / `async-io` configuration belongs to the older zbus 5 dependency line and is not the current zbus 6 model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Source: [https://docs.rs/crate/zbus_systemd/latest](https://docs.rs/crate/zbus_systemd/latest?utm_source=chatgpt.com)

### 6.4 Name and path conversion helpers

`zbus_systemd` also exports helper functions for systemd-related naming conventions:

`bus_label_escape`, `bus_label_unescape`, `bus_path_encode`, `bus_path_decode`, `unit_name_escape`, and `unit_name_unescape`.

These are valuable where a systemd unit name or external identifier must be converted to a D-Bus-safe object-path component.

The systemd unit-name escaping convention is not identical to the D-Bus object-path escaping convention. They should not be casually substituted for one another, and neither is ordinary filesystem-path escaping.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/zbus_systemd/latest/zbus_systemd/](https://docs.rs/zbus_systemd/latest/zbus_systemd/?utm_source=chatgpt.com)

## 7. systemd1: Service, unit, and job management

`systemd1` is the most substantial module because it exposes the service manager's own object model.

### 7.1 Manager, units, and jobs

The central `org.freedesktop.systemd1.Manager` object resides at `/org/freedesktop/systemd1`.

It manages two principal classes of dynamic objects:

| Object | Concept                                          | Relevant generated proxy                |
| ------ | ------------------------------------------------ | --------------------------------------- |
| Unit   | Persistent or transient systemd-managed resource | `UnitProxy`, plus a type-specific proxy |
| Job    | Queued service-manager operation                 | `JobProxy`                              |

A unit is not necessarily a process: it may describe a service, socket, target, timer, mount, device, scope, slice, path, or other systemd resource.

A job is not the unit itself. It represents a queued operation against a unit, potentially associated with dependency-related jobs.

The manager supports enumeration (`ListUnits`, `ListJobs`), identity lookup (`GetUnit`, `GetJob`), loading (`LoadUnit`), process-to-unit lookup, and various manager-level state queries.

`GetUnit` resolves an already loaded unit, whereas `LoadUnit` can cause the manager to load a unit definition from disk. This is a consequential distinction for observation-only integrations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

### 7.2 Job submission is not job completion

Methods such as `StartUnit`, `StopUnit`, `RestartUnit`, and `ReloadUnit` ordinarily queue operations and return a job object path.

Their successful method response means the operation was accepted and queued; it does not mean the service has finished starting, stopping, or reloading.

The correct completion model is:

1. Establish a `JobRemoved` receiver and enable systemd manager signals through `Subscribe()`.
2. Invoke `StartUnit` or another job-creating method.
3. Record the returned job object path.
4. Match the eventual `JobRemoved` event to that job and inspect its result.
5. Read the unit's final state when the distinction between completed job and current unit state matters.

This ordering avoids the race in which a job finishes before the caller starts listening for its result.

The documented job results include `done`, `failed`, `timeout`, `canceled`, `dependency`, and `skipped`.

`Manager.Subscribe()` is additionally required for most systemd manager signals; this includes standard property-change signals emitted by that manager object. Merely constructing a zbus signal stream does not override this systemd-specific subscription contract.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+1



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

### 7.3 Job transaction capabilities

Several manager APIs provide increasingly explicit control over systemd's job engine.

| Operation family                           | Meaning                                                                   |
| ------------------------------------------ | ------------------------------------------------------------------------- |
| `StartUnit`, `StopUnit`, `RestartUnit`     | Basic lifecycle actions                                                   |
| `ReloadUnit`, `ReloadOrRestartUnit`        | Reconfiguration with service-dependent restart behavior                   |
| `TryRestartUnit`, `ReloadOrTryRestartUnit` | Conditional actions based on current unit state                           |
| `EnqueueUnitJob`                           | Unified job submission with richer information about affected jobs        |
| `EnqueueUnitJobMany`                       | Submit multiple requested units in one job transaction                    |
| `CancelJob`, `ClearJobs`                   | Manage queued operations                                                  |
| Job mode                                   | Determine how new jobs interact with dependencies and already queued jobs |

`EnqueueUnitJobMany` is a noteworthy systemd 262 capability. It submits multiple requested units in a single transaction, allowing systemd to honor ordering relationships between them irrespective of their order in the request array.

That is materially different from sequentially sending several `StartUnit` requests. The multi-unit transaction belongs to the underlying systemd manager; `zbus_systemd` exposes the interface but does not implement transaction planning itself.

Not all modes are benign. For example, the `replace` mode may replace conflicting queued jobs, while `isolate` can stop units outside the selected dependency closure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+1



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

### 7.4 Unit state and specialized interfaces

Every unit exposes generic state and metadata through `org.freedesktop.systemd1.Unit`. A unit can additionally expose type-specific interfaces.

| Type-specific proxy             | Distinctive information                                                           |
| ------------------------------- | --------------------------------------------------------------------------------- |
| `ServiceProxy`                  | Service type, process identifiers, execution state, restart behavior, exit status |
| `SocketProxy`                   | Socket activation configuration and state                                         |
| `TimerProxy`                    | Timer trigger configuration and scheduling state                                  |
| `ScopeProxy`                    | Externally created process groups and scope lifecycle                             |
| `SliceProxy`                    | Resource-control hierarchy                                                        |
| `MountProxy` / `AutomountProxy` | Mount and automount configuration                                                 |
| `PathProxy`                     | Path-triggered activation state                                                   |
| `TargetProxy`                   | Dependency grouping and activation state                                          |
| `SwapProxy`                     | Swap device/file state                                                            |
| `DeviceProxy`                   | Device-backed unit state                                                          |

The generic unit properties distinguish `LoadState`, `ActiveState`, and `SubState`.

These cannot be collapsed into one Boolean without losing information: a unit can be loaded but inactive, active with a type-specific substate, failed, activating, or deactivating.

Likewise, the state of a service job is not equivalent to the current state of its unit. A completed start job does not establish permanent service health.

Time-related properties generally use microseconds, including both real-time and monotonic timestamps. Conversion to ordinary durations or wall-clock times must preserve that distinction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

### 7.5 Transient units and resource management

`StartTransientUnit` allows creation and activation of units from D-Bus parameters without first writing a conventional unit file.

Its interface accepts a unit name, job mode, property array (`a(sv)`), and auxiliary-unit array (`a(sa(sv))`). The systemd 262 documentation currently says that the auxiliary parameter is unused and should be passed as an empty array.

This unlocks several native systemd capabilities:

| Mechanism                 | General architectural role                                                     |
| ------------------------- | ------------------------------------------------------------------------------ |
| Transient `.service`      | Create a managed execution unit with systemd lifecycle and supervision         |
| Transient `.scope`        | Place externally created processes under systemd tracking and resource control |
| Transient `.slice`        | Introduce resource-control grouping and hierarchy                              |
| `SetUnitProperties`       | Modify supported resource-management settings dynamically                      |
| `AttachProcessesToUnit`   | Associate existing processes with appropriate units                            |
| `FreezeUnit` / `ThawUnit` | Control execution using supported cgroup freeze operations                     |
| `RefUnit` / `UnrefUnit`   | Influence lifetime of otherwise unreferenced units                             |
| `GetUnitProcesses`        | Inspect processes associated with a unit                                       |

This is an important way in which systemd can act as an existing process-lifecycle and resource-control engine, rather than requiring bespoke supervisor, cgroup-control, and process-registry implementations.

The conditions are significant: only particular properties can be changed at runtime; process ownership and permitted unit types matter; and operations are subject to authorization and the system manager's own rules.

Transient units are released when no longer running or referenced, or on reboot. They do not inherently constitute persistent unit definitions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+2



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

### 7.6 Unit files, enablement, and daemon configuration

Unit-file management is a separate responsibility from runtime unit-state management.

| Operation                           | Semantics                                                              |
| ----------------------------------- | ---------------------------------------------------------------------- |
| `EnableUnitFiles`                   | Establish enablement symlinks under systemd's unit-file locations      |
| `DisableUnitFiles`                  | Remove corresponding enablement symlinks                               |
| `MaskUnitFiles` / `UnmaskUnitFiles` | Prevent or restore activation                                          |
| `LinkUnitFiles`                     | Make an externally located unit file discoverable                      |
| `PresetUnitFiles`                   | Apply distribution or administrator preset policy                      |
| `GetUnitFileState`                  | Query enablement-related state                                         |
| `Reload`                            | Make the service manager reread configuration and rebuild dependencies |
| `Reexecute`                         | Reexecute the service-manager process                                  |

An enabled unit is not necessarily currently active, and an active unit is not necessarily enabled for future activation.

Also, manager `Reload()` is different from `ReloadUnit()`: the former reloads manager configuration and dependency information; the latter requests a unit-level reload action.

`SetUnitProperties` has a further nuance: many array-valued settings append to existing configurations rather than replace them. Resetting an array may require setting it to an empty array first.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org+1



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)

## 8. login1: Session, seat, and power-management control

`zbus_systemd::login1` exposes the D-Bus contracts of `systemd-logind`.

Its role extends well beyond listing logged-in users.

| Capability group    | Representative operations and behavior                                 |
| ------------------- | ---------------------------------------------------------------------- |
| Session registry    | Enumerate, resolve, terminate, lock, and activate sessions             |
| User lifecycle      | Track users, session counts, user runtime state, and lingering         |
| Seat management     | Inspect seats, active sessions, virtual-terminal switching             |
| Power management    | Reboot, power off, suspend, hibernate, scheduled shutdown              |
| Inhibitor locks     | Temporarily block or delay shutdown, sleep, idle, or input-key actions |
| Device coordination | Grant session-device access and coordinate pause/resume transitions    |
| Notifications       | Session and user additions/removals, power transitions, device changes |

### 8.1 File-descriptor-based inhibitor locks

`Inhibit(what, who, why, mode)` returns a file descriptor.

The inhibitor remains effective while that descriptor, or any duplicate of it, remains open. Closing every copy releases the lock.

The protocol supports blocking and delaying modes, subject to systemd's policies and limits. This is a stronger primitive than setting a Boolean property or asking a manager to remember a process identifier: the kernel descriptor lifetime participates in cleanup.

This allows a client to coordinate shutdown or sleep with an operation that must reach a safe point before the machine changes state. It also imposes a clear ownership obligation: accidentally dropping the descriptor releases protection prematurely.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



### 8.2 Session-device lifecycle

`TakeDevice` allows an authorized session controller to acquire file descriptors for certain devices. `PauseDevice` and `ResumeDevice` events coordinate access as sessions become inactive and active.

A resumed device may be represented by a new file descriptor. The client must handle the transition and close obsolete descriptors; the contract does not promise a stable descriptor identity across suspension and resumption.

These mechanisms are particularly relevant for compositors, graphical session managers, and applications coordinating seat-associated devices. They are not generic unrestricted device-access APIs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



### 8.3 Authorization and capability queries

`CanPowerOff`, `CanSuspend`, and related operations return textual capability/authorization statuses rather than a simple Rust Boolean.

Some operations accept an `interactive` flag that determines whether an authorization agent may prompt through polkit.

The presence of a typed method in `zbus_systemd` therefore does not imply that the call is permitted, that user interaction will occur, or that authorization is consistent across users and hosts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.login1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.login1.html?utm_source=chatgpt.com)

## 9. Networking, DNS, time, and host configuration

### 9.1 `network1`: systemd-networkd

The `network1` bindings cover networkd's managed network interfaces and related objects.

In addition to `ManagerProxy` and `LinkProxy`, the generated surface includes `NetworkProxy`, `DHCPv4ClientProxy`, `DHCPv6ClientProxy`, and `DHCPServerProxy`.

Capabilities include link enumeration, lookup by interface index or name, managed-link state, network configuration, DHCP-related state, and link-specific settings.

This is networkd's management interface, not a generic interface to all Linux network functionality. Systems using another network-management service may expose a different authoritative control surface.

Source: [https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.network1.html](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.network1.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://freedesktop.org\&sz=32)

freedesktop.org



### 9.2 `resolve1`: systemd-resolved

The resolver interface exposes more than ordinary DNS lookups.

| Capability             | Mechanisms and distinctions                                   |
| ---------------------- | ------------------------------------------------------------- |
| Forward resolution     | `ResolveHostname`                                             |
| Reverse resolution     | `ResolveAddress`                                              |
| General DNS records    | `ResolveRecord`                                               |
| DNS service resolution | `ResolveService`                                              |
| Per-link configuration | `SetLinkDNS`, `SetLinkDNSEx`, `SetLinkDomains`                |
| Routing and security   | Default-route DNS behavior, DNSSEC, DNS-over-TLS, LLMNR, mDNS |
| Resolver maintenance   | Cache flush, statistics, configuration reset                  |

Resolver replies can carry interface indices, address families, raw network-address byte arrays, canonical names, and result flags.

This is useful where link-specific DNS routing and detailed resolution metadata are important. The library does not transform the interface into a general replacement for every DNS client or network-stack API.

Important interaction: `network1` and `resolve1` can refer to the same network link, but they are different service managers. Changing a networkd-managed interface and configuring resolved's per-link DNS policy are related operations with separate interfaces and state.

Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.resolve1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.resolve1.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



### 9.3 Host identity, locale, and time interfaces

| Module      | Important capabilities                                                       | Boundary                                                               |
| ----------- | ---------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `hostname1` | Static and transient hostname, pretty hostname, chassis and machine metadata | Host identity configuration, not general service discovery             |
| `locale1`   | Locale variables and keyboard layout configuration                           | System policy and configuration, not application-local text processing |
| `timedate1` | Set clock, timezone, RTC policy, enable/disable NTP                          | Settings and authorization, not continuous clock-change events         |
| `timesync1` | Inspect synchronization state and server information                         | Primarily specific to systemd-timesyncd, not all NTP implementations   |

An easily overlooked limitation: timedated's `TimeUSec`, `RTCTimeUSec`, and some synchronization-related properties are documented as not emitting property changes. Caching these as continuously current measurements would be incorrect.

The timedated documentation also directs applications to appropriate operating-system timer mechanisms for detecting actual clock adjustments, rather than treating timedated as a clock-change event source.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/261/org.freedesktop.timedate1.html](https://www.freedesktop.org/software/systemd/man/261/org.freedesktop.timedate1.html?utm_source=chatgpt.com)

## 10. Additional systemd service families

### 10.1 `home1`: Identity records and managed home directories

`systemd-homed` provides user and home-directory management based on portable user records.

Its D-Bus interface supports finding users by name or UID, enumerating managed homes, retrieving full user records, creating/updating/removing homes, activating and deactivating them, resizing, password changes, and reference-based lifecycle control.

Two particularly consequential mechanisms are:

- JSON user records: Full records are transferred as structured JSON text, with an explicit indication of whether privileged data was omitted. The D-Bus binding supplies the transport and typed string response; interpreting the record belongs to the JSON user-record specification.
- Home references and acquisition: Certain operations return file descriptors used to maintain references to active home areas. This again makes resource lifetime a meaningful part of the protocol.

The resulting architecture combines D-Bus, structured JSON records, and systemd-managed home resources; zbus does not itself implement user-record validation or home storage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.home1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.home1.html?utm_source=chatgpt.com)

### 10.2 `machine1`: Containers, virtual machines, and process environments

`systemd-machined` maintains a registry of locally running containers and virtual machines.

Its D-Bus APIs expose machine enumeration and identification, registration and termination, image discovery, user/group identity mapping, and access to machine resources.

Advanced mechanisms include:

| Capability                                               | Role                                                                 |
| -------------------------------------------------------- | -------------------------------------------------------------------- |
| `CreateMachine` / `RegisterMachine` variants             | Establish a machine record, including systemd scope-related metadata |
| `GetMachineByPID`                                        | Resolve a process into its associated machine                        |
| `OpenMachinePTY`, `OpenMachineLogin`, `OpenMachineShell` | Obtain interactive terminal access through transferred descriptors   |
| `OpenMachineRootDirectory`                               | Receive a handle to the machine's root directory                     |
| `BindMountMachine`                                       | Manipulate supported mount relationships                             |
| `CopyFromMachine` / `CopyToMachine`                      | Transfer files across a machine boundary                             |
| User/group mapping                                       | Convert identities between machine and host namespaces               |
| Image APIs                                               | Clone, rename, inspect, remove, and apply limits to images           |

These are machined service capabilities, not generic container runtime functions. The library does not instantiate a complete container engine by itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.machine1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.machine1.html?utm_source=chatgpt.com)

### 10.3 `import1`: Image and transfer management

`systemd-importd` supports disk-image importing, exporting, and downloading. It is often used alongside machined.

The interface covers tar and raw images, supported compression formats, image classes, exports to supplied file descriptors, verified remote pulls, and OCI-image pulling.

`PullOci` was added in systemd 260, so a generated method's presence does not imply availability against an older installed daemon.

Import and download operations generally return transfer identifiers and object paths before completion. The associated `TransferProxy` exposes progress and other state; `TransferRemoved` indicates the eventual result. Generated streams also cover `ProgressUpdate` and `LogMessage`.

An important performance/observability distinction: when importing from a regular file, the service can estimate progress from its size; for pipes or sockets, total-size-based progress may not be available.

Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.import1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.import1.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



### 10.4 `portable1`: Portable-service images

The portable-service interface manages images from which systemd services can be attached to a host.

Capabilities include inspecting image metadata and embedded units, obtaining OS-release information, attaching and detaching portable services, reattaching updated images, handling extension images, querying attachment state, and managing image storage limits.

A noteworthy distinction is that reattachment is a separate supported operation from attachment/detachment, including cases involving running units. Extension-aware variants preserve additional context about overlay images.

This interface manages integration of portable images into systemd; it is not a general-purpose container runtime.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.portable1.html](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.portable1.html?utm_source=chatgpt.com)

### 10.5 `oom1`: Memory-pressure outcomes

`systemd-oomd` offers a small but useful D-Bus interface.

The principal exposed capabilities are a diagnostic dump available through a file descriptor and a `Killed(cgroup, reason)` notification emitted when oomd terminates a cgroup.

The signal describes an observed intervention, with reasons such as memory use or memory pressure.

It can complement cgroup/resource-state information exposed through systemd units and slices, but it should not be mistaken for a complete memory-pressure telemetry feed or the kernel's entire OOM-event infrastructure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/260/org.freedesktop.oom1.html](https://www.freedesktop.org/software/systemd/man/260/org.freedesktop.oom1.html?utm_source=chatgpt.com)

### 10.6 `sysupdate1`: System-update targets and jobs

The system-update interface exposes discoverable update targets, version and feature information, acquisition and installation operations, update jobs, progress, cancellation, and completion notifications.

The conceptual model is similar to other systemd asynchronous workflows: an operation may return a job object whose progress and result must be monitored separately.

Status: experimental in systemd 262. Upstream explicitly warns that the interface is unstable and may change between systemd versions.

Its presence in `zbus_systemd` should therefore be understood as generated access to an experimental service contract, not as a stability guarantee supplied by the Rust crate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.freedesktop.org\&sz=32)

freedesktop.org



Source: [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.sysupdate1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.sysupdate1.html?utm_source=chatgpt.com)

## 11. Capability interactions and architectural patterns

The combination of the two libraries enables several general designs that are more interesting than isolated method calls.

| Pattern                           | Native mechanisms combined                                         | Benefit and boundary                                                                                                           |
| --------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| Event-driven service registry     | `ListUnits`, `UnitNew`, `UnitRemoved`, `PropertiesChanged`         | Maintains an updated view of loaded units without repeatedly invoking `systemctl`; requires reconciliation after missed events |
| Job-aware lifecycle control       | `StartUnit`, `JobRemoved`, `UnitProxy` state                       | Separates request acceptance, job completion, and current service state                                                        |
| Transient process supervision     | `StartTransientUnit`, scopes/slices, resource properties, signals  | Delegates process grouping, supervision, and resource controls to systemd                                                      |
| Coordinated multi-unit operations | `EnqueueUnitJobMany`, dependency rules, job tracking               | Uses systemd's native dependency-aware job transaction rather than reproducing its scheduler                                   |
| Shutdown-safe operations          | logind `Inhibit`, owned file descriptors                           | Couples a temporary protection lifetime to a kernel resource                                                                   |
| Container/machine operations      | `machine1`, `import1`, descriptor transfer, job streams            | Combines machine registry, images, terminal access, and asynchronous transfers                                                 |
| Network state and DNS policy      | `network1::LinkProxy`, `resolve1::LinkProxy`, change notifications | Coordinates separate network-management and resolver capabilities                                                              |
| General D-Bus automation          | `zbus_systemd` proxies plus custom `#[proxy]` bindings             | Extends a common D-Bus connection to services outside the generated systemd set                                                |

These are architectural possibilities enabled by the published contracts, not additional high-level functions provided by `zbus_systemd`.

### 11.1 Snapshot-plus-event reconciliation

For continuously changing system state, an effective conceptual model is:

1. Establish the relevant event subscriptions.
2. Acquire a baseline enumeration or snapshot.
3. Process subsequent updates and reconcile duplicates or overlaps.
4. Re-enumerate after connection interruption, service-owner changes, or suspected queue loss.

This closes more gaps than naïvely polling or solely relying on signals.

The exact synchronization strategy depends on the remote service's ordering and snapshot guarantees. Neither a zbus stream nor a generated proxy automatically supplies database-like snapshot isolation.

### 11.2 Delegating to existing system engines

Several systemd APIs make potentially substantial bespoke machinery unnecessary:

Process orchestration: systemd's units, job scheduler, dependencies, and cgroup integration can handle tasks that otherwise require a custom supervisor.

Session/device coordination: logind's inhibitor and device-control protocols can replace ad hoc process registries and device handoff signaling.

Transfer management: importd's transfer objects, progress notifications, and completion results provide an existing job model for image operations.

Typed IPC infrastructure: zbus macros, object servers, introspection, and serialization reduce the need for a custom RPC protocol when the D-Bus model is suitable.

The qualification is that delegating functionality to systemd also delegates control to its service availability, privilege model, runtime lifecycle, and deployment constraints.

## 12. Important limitations and version boundaries

| Dimension                                 | Constraint or tradeoff                                                                                |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Protocol versus authorization             | Typed proxies do not bypass D-Bus policies, polkit, or service-specific permissions                   |
| Compile-time versus runtime compatibility | A generated method can exist in the Rust crate but not on an older installed systemd daemon           |
| Runtime environment                       | zbus is broader than Linux, but systemd-specific interfaces require corresponding systemd services    |
| Availability                              | An optional auxiliary daemon may be absent or disabled                                                |
| Event durability                          | Signals are asynchronous notifications, not persistent records                                        |
| State consistency                         | Properties can be cached, unavailable, non-notifying, or observed at different times                  |
| Object lifetime                           | Unit, job, transfer, session, and machine object paths can become invalid                             |
| Resource lifetime                         | File-descriptor-based APIs may require descriptors to remain open                                     |
| Long-running operations                   | A method reply may confirm acceptance rather than completion                                          |
| Schema evolution                          | New enum strings, flags, methods, or properties may require compatible decoding and capability checks |
| Privilege                                 | Operations on the system manager and user manager have different authority contexts                   |
| Async integration                         | Runtime backend selection and `block_on` limitations matter for nested execution                      |

### 12.1 The zbus 5 → 6 migration boundary

For previously authored integrations, these are the most material changes:

| zbus 5-era API or assumption                  | zbus 6                                                       |
| --------------------------------------------- | ------------------------------------------------------------ |
| Independent `zvariant` dependency             | Types/serialization consolidated into `zbus`                 |
| `Value`, `OwnedValue`                         | `Variant`, `OwnedVariant`                                    |
| `zvariant::serialized`                        | `zbus::wire::serialized`                                     |
| `zbus::blocking` / generated blocking proxies | Removed; use async API and `zbus::block_on`                  |
| `async-io` default backend                    | `default-rt` / zruntime                                      |
| Built-in executor access                      | `Builder::runtime` or selected backend                       |
| `SignalContext`                               | `SignalEmitter`                                              |
| Broadly unified client/service code           | Separately feature-gated `proxy` and `service` functionality |
| `enumflags2` wrapping in zbus APIs            | Native `bitflags`-style types                                |
| Separate serialization/name errors            | Unified `zbus::Error`                                        |
| Older property conversion assumptions         | Serde-based property values and stronger signature checking  |

The zbus 6 minimum supported Rust version is 1.89. A dependency graph containing both zbus 6 types and older `zvariant` 5 types can have incompatible `Type`, `Signature`, and variant representations even when their names appear similar.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://z-galaxy.github.io\&sz=32)

zbus: D-Bus for Rust made easy



Source: [https://z-galaxy.github.io/zbus/upgrading-to-6.html](https://z-galaxy.github.io/zbus/upgrading-to-6.html?utm_source=chatgpt.com)

### 12.2 The three separate version axes

Care should be taken not to collapse the following into a single version:

| Version axis                    | Meaning                                                                   |
| ------------------------------- | ------------------------------------------------------------------------- |
| `zbus 6.1.0`                    | Rust-side D-Bus framework, serialization, transport, and runtime behavior |
| `zbus_systemd 0.26201.0`        | Generated Rust bindings for a systemd API generation                      |
| Running systemd daemon versions | Actual operations and properties supported on the target machine          |

A newer generated proxy crate does not upgrade the remote daemon. Conversely, the remote daemon can expose newly introduced capabilities before they are represented by a released generated binding; generic zbus proxies and direct calls remain available for such cases.

Some online API pages currently appear indexed at earlier releases even though the crate-release page identifies `zbus_systemd 0.26201.0`. For version-sensitive APIs, prefer the selected release's source and the applicable systemd manual over an older cached `latest` API page.

## 13. Primary source reference map

The following are direct source URLs rather than ChatGPT-specific links.

| Subject                                    | Source                                                                                                                                                                                              |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| zbus 6 architectural changes and migration | [https://z-galaxy.github.io/zbus/upgrading-to-6.html](https://z-galaxy.github.io/zbus/upgrading-to-6.html?utm_source=chatgpt.com)                                                                   |
| zbus connection and runtime model          | [https://z-galaxy.github.io/zbus/connection.html](https://z-galaxy.github.io/zbus/connection.html?utm_source=chatgpt.com)                                                                           |
| Typed client proxies and signals           | [https://z-galaxy.github.io/zbus/client.html](https://z-galaxy.github.io/zbus/client.html?utm_source=chatgpt.com)                                                                                   |
| Service implementation and ObjectServer    | [https://z-galaxy.github.io/zbus/service.html](https://z-galaxy.github.io/zbus/service.html?utm_source=chatgpt.com)                                                                                 |
| Serialization and runtime FAQ              | [https://z-galaxy.github.io/zbus/faq.html](https://z-galaxy.github.io/zbus/faq.html?utm_source=chatgpt.com)                                                                                         |
| zbus upstream source                       | [https://github.com/z-galaxy/zbus](https://github.com/z-galaxy/zbus?utm_source=chatgpt.com)                                                                                                         |
| zbus 6.1.0 crate release                   | [https://docs.rs/crate/zbus/6.1.0](https://docs.rs/crate/zbus/6.1.0?utm_source=chatgpt.com)                                                                                                         |
| zbus_systemd crate metadata and features   | [https://docs.rs/crate/zbus_systemd/latest](https://docs.rs/crate/zbus_systemd/latest?utm_source=chatgpt.com)                                                                                       |
| zbus_systemd upstream source               | [https://github.com/lucab/zbus_systemd](https://github.com/lucab/zbus_systemd?utm_source=chatgpt.com)                                                                                               |
| systemd manager and unit interfaces        | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.systemd1.html?utm_source=chatgpt.com)     |
| systemd session and device interfaces      | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.login1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.login1.html?utm_source=chatgpt.com)         |
| systemd resolver                           | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.resolve1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.resolve1.html?utm_source=chatgpt.com)     |
| systemd managed homes                      | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.home1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.home1.html?utm_source=chatgpt.com)           |
| systemd machine registry                   | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.machine1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.machine1.html?utm_source=chatgpt.com)     |
| systemd image transfers                    | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.import1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.import1.html?utm_source=chatgpt.com)       |
| systemd update interfaces                  | [https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.sysupdate1.html](https://www.freedesktop.org/software/systemd/man/262/org.freedesktop.sysupdate1.html?utm_source=chatgpt.com) |

### Final architectural synthesis

`zbus` offers three distinct capabilities within a single Rust ecosystem: a standalone D-Bus type/wire system, an advanced typed asynchronous IPC client framework, and a service-side object/interface framework.

`zbus_systemd` supplies the systemd-specific bindings that make those capabilities immediately applicable across service management, sessions, networking, resolver configuration, managed homes, machines, image operations, and additional system services.

The most significant advanced functionality comes from composing typed D-Bus interfaces with systemd's own execution engines: asynchronous job transactions, transient resource-controlled units, signal-driven system-state observation, descriptor-controlled lifetimes, and specialized service operations. The Rust libraries provide structured access and integration; systemd remains responsible for the substantive operating-system behavior and its guarantees.