# Pinned native request lifecycle patch

Base: crates.io `surrealdb==3.3.0` and `surrealdb-engine-api==3.3.0`.
The copied upstream licenses and `.cargo-vcs-info.json` identify the original
source. Version numbers and RPC/types/server dependencies stay at 3.3.0.

The patch adds optional request-local deadline/cancellation and dispatch state to
query/select/signin/set/version/namespace/database builders and the engine route. Native
`Config::bounded_requests()` enables 32 application plus two reserved control
admissions, 34 queued/deferred routes, one application session and 64 ordered
replay entries including automatic attachment and prospective setup commands.
The driver retains dispatched correlation/admission while draining an abandoned
request and physically drops both socket halves before releasing correlations.
`disconnect()` awaits that physical driver boundary. It does not establish that
the server or a native scientific task stopped. Normal queries do not use the
streaming query cancellation RPC.

The unmodified profile remains the SDK default. The finite profile is for the
direct native WebSocket client; its request builders require an original
`RequestContext`. Reconnect replays acknowledged setup only, never application
queries. Replay order is preserved. An explicit final-Owner `selection_checkpoint`
request context carries one opaque identity per client and selected database.
After its successful acknowledgment, the driver may replace only an adjacent
previous completed same-identity authentication / exact namespace-and-database
USE / authentication triple. Intervening variable/settings setup, different
selections, partial or failed authentication, and unmarked setup are retained.
The checkpoint replaces storage rather than growing a hidden history; all
remaining setup and prospective requests remain subject to the 64-entry limit.

Connection version verification uses a finite five-second reserved control
lifecycle. Admission is released explicitly at acknowledged completion or after
physical invalidation, independent of concurrent-map epoch reclamation. Dropping
a request future cancels admission; dropping the final application session closes
the physical driver. The bounded root-authentication profile returns expired
refresh-token setup errors to its owner instead of starting an unbounded automatic
refresh; the default profile retains upstream refresh behavior.

Bounded session setup retains remote typed error domains even after setup has
failed before a caller queues its route. Version verification suppresses only
a denial naming the `version` method, never a rejected automatic attachment.

Native WebSocket response-channel and setup/transport failure logs use fixed
labels instead of response or frame Debug output. A dropped signin receiver may
still receive a late token acknowledgment internally; no token, binding or queued
frame is formatted into those logs. Typed failures remain available to callers.
