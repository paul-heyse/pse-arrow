# Pinned native gRPC request lifecycle patch

Base: crates.io `surrealdb==3.3.4` and `surrealdb-engine-api==3.3.4`.
The copied upstream licenses and `.cargo_vcs_info.json` identify the original
sources. The workspace selects `protocol-grpc`; canonical clients use
`Config::bounded_requests()` and an explicit original `RequestContext`.

## Admission, sessions and original clocks

The bounded engine owns 32 application and two reserved control admissions, with
32 application and 32 control waiting slots. Available active slots are
acquired before waiting capacity; original-clock waiters never occupy an active
slot. The separate two-slot control execution reserve remains intact. A refused clone cannot create a
second application session or grow an unknown-session registry. Acknowledged
session history has 64 entries including attachment and prospective setup.
Setup is ordered, recorded only after acknowledgment, and never coalesced across
intervening setup. A final owner `selection_checkpoint` can replace only adjacent
completed same-identity authentication / exact namespace-and-database USE /
authentication triples. Credentials and values are never formatted into logs.

Every native operation retains its original absolute deadline through admission,
session readiness, restoration, dispatch, conversion and delivery. Dispatch is
recorded at the HTTP/2 send seam. Dropping a waiting caller cancels admission;
abandoning dispatched work retains its actual driver, payload and activity
owners until late completion or the original deadline. It does not revoke
independent callers. An expired dispatched operation physically drains its failed
connection generation and reports uncertainty.

A retained client can recover for a *new undispatched operation*: it first joins
the failed physical generation, creates a new socket and server session, then
restores only the complete acknowledged setup snapshot. One restoration deadline
is the minimum of the new operation's original deadline and five seconds. A
partially failed restore cannot replace that snapshot. No application query,
mutation or explicit transaction is automatically replayed. An explicit
`disconnect()` permanently seals admission, awaits bounded session detach, joins
the physical socket task, and drains all admitted local request tasks. Final SDK
owner drop follows the same physical ownership route.

## Owned HTTP/2 and response envelopes

The generated tonic service client remains the RPC implementation. The bounded
profile supplies an owned hyper HTTP/2 connection with an abort-and-join handle;
a dropped tonic request is not substituted for physical socket completion.
HTTP/2 frame, header and flow-control buffers are finite, and server-initiated
streams are refused. The generic unbounded SDK profile retains upstream tonic
transport behavior; it is not the canonical production path.

The bounded response body recognizes its installed RPC recipient and validates
complete borrowed protobuf messages before yielding them to prost. Encoded wire,
node/depth, repeated-container, UTF8/string and blob occurrence allowances are
cumulative across the whole operation, including individually legal frames and
packed scalar fields. Nodes charge each owned protobuf message and repeated
inline vector slot once; wire tags and ordinary inline fields are not separate
owned allocations. Protocol UUID envelopes are validated as exactly 16
bytes and charged as wire/nodes; application byte values retain their separate
blob budgets. The bounded profile does not advertise response
compression, so no decompression can precede this preflight. An endpoint's
`ResponseLimits` and actual prepaid owners travel through spawned native work,
codec ingress, owned conversion, delivery and physical drain. Clearable ownership
cells do not depend on retired context aliases disappearing.

Statement counts are checked against the submitted script length, endpoint
allocation ceiling and original node/container/byte allowance before allocating
result slots. Batch indices cannot exceed the advertised count, and repeated
Begin frames cannot erase provisional results.

Unary RPC failures preserve typed native protobuf errors and their cause chains
through the explicit `surreal-error-bin` trailer. Encoded and decoded metadata
are bounded to 8 KiB, and borrowed validation precedes owned decoding. A bare
status code or human message never manufactures a typed conflict reason.

Buffered canonical results require ordered batches, terminal statement results,
authoritative QueryEnd and successful HTTP/2 trailers. Rows from a late failing
or incomplete transaction cannot become successful IndexedResults. The bounded
streaming trait uses this completed buffered result before emitting items;
canonical metadata and protected-result operations remain bounded page requests.

## Maintaining the pinned wire boundary

`src/engine/remote/grpc_wire_schema.rs` contains structural field shapes extracted
from the exact protocol `0.13.1` prost declarations. Reproduce it with:

```sh
python tools/update_grpc_wire_schema.py /path/to/surrealdb-protocol-0.13.1
```

The extraction script reads source only. It declares no catalog meaning, performs
no owned Value decode, and must move with a deliberate protocol pin change.
Actual HTTP/2/protobuf fault, capacity, restoration and lifetime controls live in
the canonical transport test owner. The owning implementation plan records their
qualification state; this file does not claim assembled product qualification.
