# Persistence and result delivery

Supporting assessment for the [principal review](../surrealdb-functional-improvements-2026-11-09.md).
Inspected 2026-11-09 at `4f07766942ff3b21274fdfb686ad483031439462` plus the
concurrent dirty working tree, including native service and canonical lifecycle changes.
Current source and selected supplier interfaces are **Interface-checked**; directions and
benefits are **Proposed**. No engine, transfer or protocol probe was run.

The selected standard is core 3.4, efficient architecture 1.0, process-simulator 1.5,
test assessment 1.2 and the repository binding. AP-02/AP-04/AP-07, DP-09/DP-15,
and PS-13 matter here: changing bytes or placement must not change original scientific
meaning, completeness or the independence of result interpretation.

<a id="O12"></a>
## O12 — Make persistence and historical access fit their actual semantics

**Proposed direction: retain integrated persistent MX for the recommended canonical target;
develop integrated KV as the concrete alternative for capacity or recorded-version access.**
The reason is the useful composition, not loyalty to the installed engine. Canonical metadata
already benefits from one transaction/query/permission substrate and serializable guarded
operations. A standalone KV replacement would also have to replace those services.

Today an operator opens a Rust-owned persistent service. Its private state, credentials,
catalog and payload custody have separate owners. The current working-tree
`crates/pse-operations/src/native_service.rs` constructs
`mem://…?aol=sync&sync=every&versioned=false&snapshot=60s`; it does not simply start an
ephemeral memory database. [The service baseline](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/native_service.rs)
and [server source preparation](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-host/src/server.rs)
establish the integration owners; blueprint §20 owns authoritative state.

The improved workflow is an operator able to inspect why memory/disk is retained, select a
coherent supported persistence realization, and perform owned maintenance/recovery without
reconstructing those facts from paths or generic engine gauges. Prefer native snapshot,
reclamation and inspection mechanisms over local copies or repeated full scans wherever
their completion semantics discharge the actual obligation. Retain exact reader/protection
membership and original acknowledgement reconciliation independently of storage cleanup.

Compare these complete alternatives:

| Realization | Assessment |
| --- | --- |
| Integrated MX with compact metadata and separately retained scientific payloads | Recommended present target: preserves native SSI and one schema/graph/authorization authority, while keeping large payload lifetime separate. Its RAM-resident live set and snapshot/recovery cost remain real limits. |
| Integrated SurrealKV with existing database services | Conditional alternative: useful for disk-backed capacity and retained recorded versions. A choice requires a replacement argument for any MX-dependent predicate isolation, not merely a storage benchmark. |
| Standalone KV/MX for a small derived index | Conditional only where the index really needs key lookup/scan and can be rebuilt. Borrowed scans, reusable buffers and opaque bytes can remove hydration; a second source of canonical truth is a poor trade. |
| SurrealDS or managed Earth/Cloud | Technically eligible for continuity, remote access or read scaling. Additional replicas/object storage, sequencing and operational participants need a concrete benefit in these workflows. Licence is not a rejection reason. |
| Separate TiKV integration or browser OPFS | No preferred role in the current local target. Reconsider for an independently justified shared-storage or offline-browser workflow rather than retaining a dormant production path. |

The tagged [SurrealKV 1.2 README](https://github.com/surrealdb/surrealkv/blob/v1.2.0/README.md)
documents standalone LSM, optional value-log separation, history and different durability
choices. These can improve large/versioned-byte access; they do not establish which controls
the database adapter exposes. Its published throughput figures are not local evidence.
Standalone encryption/integrity tools are likewise conditional facilities: select them with
an actual key, verification and recovery owner, rather than implying that the integrated
deployment already uses them. [MX 1.1](https://github.com/surrealdb/surrealmx/blob/v1.1.0/README.md)
is a separate source for its native facilities.

**History decision.** Preserve authored/effective intervals, corrections, tombstones and
interpretation as domain facts. Use backend `VERSION` history only beneath recorded-time
access where it fits; it cannot replace those scientific distinctions. This leaves room for
richer “show what was recorded at this cut” inspection without renaming a storage timestamp
as effective scientific time.

Delivery should demonstrate one restart/recovery and one held-reader reclamation journey,
with acknowledged versus uncertain effects distinguished. An engine switch additionally
needs equal guarded outcomes and original result interpretation. No universal engine facade,
parallel canonical store or benchmark campaign is recommended. Changing the MX selection
or canonical topology would require the applicable decision route (including ADR-0214 and
blueprint §20); this assessment changes neither.

<a id="O13"></a>
## O13 — Offer native bucket-backed protected artifact delivery

**Proposed direction: conditionally select native buckets as the replacement delivery mechanism
when they cover immutable protected access end to end.** This is a real target option, not an
instruction to preserve every local file helper. Keep the publication/protection semantic
owner even if its underlying file-routing and transfer implementation disappears.

Today [sealed-file operations](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_files.rs)
associate a unique physical instance with digest, extent, encoding, batch count, schema and
original interpretation. Catalog attachment acknowledges publication. A user reads selected
protected batches, not an arbitrary filesystem path. After this improvement, the same user
could retrieve a protected artifact through a native file/bucket locator or scoped transfer
without acquiring knowledge of server-local directory layout.

`DEFINE BUCKET` and native file values predate v4; v4 stabilizes/extends file storage and
presigned access in the supplied reference. The current [bucket definition](https://surrealdb.com/docs/reference/query-language/statements/define/bucket)
documents local and object backends, scope, `READONLY` and permissions. The current launcher
restricts bucket paths to its module root; artifact storage requires an explicit selected
root/backend and capability change, not a wildcard expansion.

Prefer immutable instance keys, verified original descriptors and scoped access issued by
the canonical protection owner. For uploads, keep private creation/sealing before catalog
publication; transfer success alone is not attachment. For reads, preserve selected-range
or batch access and independent Arrow/Proto interpretation. Presigned access is useful only
if expiry/revocation and outstanding readers fit the protected artifact's lifetime.

The current bucket reference makes this concrete: presign permission is checked at issuance,
not each later use, and changing the permission does not invalidate the issued URL. Support
`overwrite=false`, a maximum upload size and a short expiry when issuing capabilities; keep
the corresponding protection alive through their usable lifetime or establish native revocation.
Do not equate revoking ordinary record access with revoking already-issued transfer access.
This favors an explicit transfer capability over exposing the bucket with default permissions.

Compare native buckets with the current sealed-file implementation and direct embedding of
all bytes in records. Buckets can remove custom routing/transfer plumbing and support
independent consumers; record blobs can amplify transaction, wire and historical retention
cost. The current files remain preferable if bucket operations only provide whole-object
materialization, overwritable keys or insufficiently observable cleanup. No evidence here
establishes catalog/object atomicity, no-clobber semantics or range support for every backend.
Those are the bounded premises for choosing the native replacement, not guaranteed behavior.

Delivery must discriminate partial upload, equal-content distinct instances, lost attachment
acknowledgement, held-reader release, expired transfer, corruption and orphan recovery.
Retiring old helpers requires the replacement to cover those effects without a second normal
delivery path. Changing publication, protection or result-delivery contracts needs the existing
decision route; moving bytes alone need not move scientific authority.

<a id="O14"></a>
## O14 — Use exact generated mappings and demand-shaped native result delivery

**Proposed direction: keep direct bounded gRPC as the default client, generate policy-aware
conversions, and add incremental consumption where a complete selected read benefits.**
This is preferable to replacing the client with an SDK solely for a smaller public surface.
An SDK remains a replacement candidate if it demonstrably discharges the complete contract.

Today [wire preflight](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_client/wire.rs)
admits aggregate bytes/nodes/depth before owned decoding; [result assembly](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_client/results.rs)
retains ordered statement results until `QueryEnd` and trailers. The improved consumer asks
for the necessary metadata/projection, processes batches with bounded live state, and receives
an explicit final complete outcome. Tentative rows must not be presented as a committed result.

The selected native codec is not a universal exact representation. Tagged
[v4 conversion source](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs)
widens numeric encodings, degrades temporal families and can encode BigDecimal as text;
decoding tagged sized variants does not prove encoding preserves their type. Keep required
IEEE bits, identities and presence distinct from convenient query values. Generate mappings
from declared policy and use native `SurrealValue` only where the policy agrees. Delete
repetitive object construction/conversion branches after their meaning has one owner.

Choose gRPC incremental batches for larger metadata reads with a genuine incremental
consumer; retain compact complete pages for ordinary small reads. Evaluate CBOR v2 or shaped
rows only for a selected fidelity/width problem, accounting for client negotiation and
cumulative admission. Avoid adding fallback transports. The SDK's streaming-looking API,
WebSocket `query_stream`, durable HTTP sessions and application message archives solve
different problems; none eliminates original request/transaction/connection settlement.

Delivery examples include exact full-range integers, signed zero, NONE versus NULL, malformed
or duplicate-key input, incomplete terminal frames, several failing statements, aggregate stream
overflow and cancellation while a consumer holds a batch. Public conversion and streaming
interfaces may break; the new contract must state partial versus complete outcomes explicitly.

## Discovery correspondence and evidence limits

| Captured direction | Assessment home |
| --- | --- |
| MX/KV comparison; native maintenance; retained version/history | O12: MX target, integrated KV conditional; separate semantic and storage histories. |
| Standalone borrowed scans, buffers, value logs, encryption/integrity | O12: only where useful physical access/operation justifies the separate boundary. |
| DS, managed Cloud/Earth, TiKV, browser persistence | O12: credible alternatives with explicit workflow/topology triggers, no dormant compatibility path. |
| Buckets, presigned transfer, sealed files and original payload custody | O13: conditional native replacement with complete protection semantics. |
| Generated mapping, native codecs, numeric/temporal fidelity | O14 with O01: generate policy-aware mappings; do not adopt a lossy universal codec. |
| SDK replacement, gRPC streaming, CBOR/FlatBuffers/shaped results | O14: direct gRPC target; alternative selected by full-contract benefit. |
| SQL/GraphQL clients and HTTP session durability | O16 in [developer workflows](developer-and-operator-workflows.md); no assumed production requirement. |

The discovery [storage](../../library_leverage/libraries/surrealdb/briefings/storage-durability-and-artifacts.md)
and [transport](../../library_leverage/libraries/surrealdb/briefings/native-values-transports-and-clients.md)
accounts supply full leads. Primary codec, bucket and KV material was rechecked; full
backend adapter eligibility, transfer atomicity, selected SDK seams and CBOR negotiation
remain explicit premises. Recommendations are qualitative; no speed, durability or zero-copy
benefit is claimed as Tested or Measured.
