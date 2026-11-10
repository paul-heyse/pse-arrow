---
title: "Plan72f: Persistence and protected delivery"
status: complete
date: 2026-11-09
adrs: [ADR-0267]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, design, libraries, storage, deployment, validation]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/persistence-and-result-delivery.md]
---

# Persistence and protected delivery

**Proposed.** Expose truthful persistence/history/maintenance and deliver exact selected results
with less conversion and materialization. This owns [O12–O14](../functional_reviews/surrealdb-functional-improvements-2026-11-09/persistence-and-result-delivery.md).
The [coordinator](72-surrealdb-functional-improvements.md) owns shared target and decisions;
72a owns generated value policy, this companion owns actual transport and physical artifact
custody. Plan66 retains scientific interpretation; Plan71 retains test/evidence infrastructure.
Plan68 is not an input or a source of obligations.

## Foundation assessment and selected target

**Interface-checked**, 2026-11-09 at `21f703887cc307f986975f9a2fba998f8bd2afa7` plus the
concurrent tree: the service uses `mem://…?aol=sync&sync=every&versioned=false&snapshot=60s`.
Sealed files have unique physical instance, digest, extent, encoding and original interpretation;
catalog attachment acknowledges publication. Wire preflight admits cumulative input before owned
decoding; results become complete only after QueryEnd and trailers.
[Service](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/native_service.rs) ·
[Files](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_files.rs) ·
[Client](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_client.rs).

Retain integrated persistent MX for canonical metadata and the bounded direct gRPC client.
They already compose schema, permissions, graph/query execution and serializable decisions with
the needed custody. Do not add a generic engine/client facade or a second authority. This is a
reasoned target, not a requirement to prove that the installed choice beats every engine.
Core 3.4, efficient architecture 1.0, process-simulator 1.5 and the binding apply; blueprint
§5/§20/§21 and DP-09/DP-15 preserve original identity, interpretation and truthful effects.

Move native metadata mappings into 72a's owner-derived policy and process larger selected reads
incrementally under one result/lifetime contract. Keep compact complete pages for ordinary reads.
Tentative stream items are not a final complete outcome. Native buckets are a real alternative
physical realization for protected artifacts, selected only after the few custody contracts that
could cause loss or premature release are settled; do not keep two normal delivery paths.

## Packages

### Execution checkpoint — 2026-11-10

Execution is authorized under proposed ADR-0267. Retain integrated persistent MX and sealed
files: selected local buckets lack fsynced atomic publication and recoverable reservation
failure. Expose available incarnation-bound observations and truthful unsupported history/
maintenance, without inventing integrated VERSION or snapshot RPCs. Incremental Proto batches
remain inside original request/allocation/driver custody and tentative until terminal success.
First migrate frozen capture/result-metadata export; ordinary bounded pages remain useful.
Preserve Plan73 original formula/source/sample interpretation and protected reader lifetimes.

The current source implements original-incarnation persistence inspection and incremental
query batch consumption. Passive inspection verifies private owner-issued startup records,
prepared executable, actual unit/invocation and owned listener on both sides of an authenticated
bounded metrics read. It gains no lifecycle or maintenance authority. Supplied Collector JSON
retains a distinct weaker origin. MX recorded-version queries and manual snapshot completion
remain explicitly unavailable; physical space release is not inferred from logical gauges.

Frozen study-member metadata export now processes tentative batches into an incomplete staged
file and publishes only after QueryEnd, EOF, trailers and every statement succeed. Ordinary
complete bounded pages remain. Independent per-frame and whole-query admission retain the
original request/allocation/driver custody. Sealed-file publication is retained: selected local
buckets lack fsynced atomic publication and recoverable no-clobber reservation failure. Reopen
that replacement only when the complete native custody contract exists. Native receiving,
interruption, terminal accounting and runtime export controls now pass as linked below.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| F01 | Complete | Current service/files/retention owners and 72c explicit isolation contract | Native maintenance and recorded-history inspection use truthful ownership and preserve authored scientific time; nonselected topology branches have explicit triggers. |
| F02 | Complete | Current sealed-file contract; independent selected Arrow/Proto readers | Close native bucket custody decision and deliver the selected artifact realization with original-instance publication/read/release/recovery. |
| F03 | Complete | 72a declared exact-value mapping and current request/result owners | Generated native mappings and bounded incremental selected delivery replace repetitive conversion/materialization without weakening terminal completeness. |
| F04 | Complete | F01–F03 changed lifetime/data contracts and actual migrated consumers | Targeted recovery, interruption and held-reader controls establish the changed correctness boundaries; replaced code/callers/tests are removed. |

<a id="F01"></a>
### F01 — Owned persistence and recorded-history inspection

Give the operator a selected service-incarnation view of retained versions, cache/payload/files
and maintenance state. Use native snapshot/reclamation/inspection APIs where their actual completion
covers the effect; preserve original acknowledgement lookup and exact protections independently.
Distinguish logical deletion, MVCC reclamation, allocator release and returned filesystem space.
Replace repeated full scans/copies or local maintenance reinterpretation when native owners suffice.

Keep authored/effective intervals, corrections, tombstones and interpretation as domain facts.
Recorded-time inspection can use native `VERSION` only under a backend retaining the requested
versions; no MX historical-query capability is invented. Keep current semantic history available.
Integrated KV becomes an active design alternative when capacity or retained recorded versions
outgrow the chosen MX realization: the persistence owner then resolves adapter exposure and
72c's predicate-isolation replacement before switching. A vendor throughput figure is insufficient.
[KV 1.2](https://github.com/surrealdb/surrealkv/blob/v1.2.0/README.md) ·
[MX 1.1](https://github.com/surrealdb/surrealmx/blob/v1.1.0/README.md).

Standalone KV/MX borrowed scans/buffers/value logs/encryption are not a parallel canonical store.
Reopen only for a rebuildable lookup-only derived consumer or an explicit key/recovery requirement.
DS/Cloud/Earth need concrete continuity, remote access or read-scale participants; TiKV/browser
OPFS need shared-storage/offline-browser requirements. None is rejected on licence. These branches
are captured deferrals with the persistence owner and observable triggers, not unfinished engine
benchmark work. Targeted acceptance covers restart, uncertain acknowledgement and a held-reader
reclamation case affected by new maintenance—not a blanket device-power-loss campaign.

<a id="F02"></a>
### F02 — Decide and integrate native protected artifact storage

The artifact owner checks native bucket backends for immutable instance/no-clobber creation,
selected range/batch access, exact extent/digest, failure visibility and recoverable orphan cleanup.
This is bounded source/contract research; use a targeted private control only where those serious
data-loss/lifetime risks remain unresolved. Adopt native buckets when they supply the complete
operation with less custom routing. If they cannot, retain sealed files and close the native
replacement branch with its missing contract and revisit trigger. Other packages do not wait.

For the native branch, prepare an explicit artifact root/backend, keep private upload/seal before
catalog attachment, and issue scoped transfer capabilities from the original protection owner.
`overwrite=false`, maximum size and expiry constrain transfer but do not alone prove immutability.
Presigned permission is checked at issuance; changing ordinary permission does not revoke a URL.
Keep its protection through the usable capability lifetime or use an established native revocation
operation. Preserve equal-content distinct instances and independent Arrow/Proto interpretation.
[Bucket reference](https://surrealdb.com/docs/reference/query-language/statements/define/bucket).

Migrate upload, attach, selected read, retention/release and offline recovery together; remove the
replaced file route and fixtures when all its consumers move. Do not put all payload bytes into
metadata records merely to avoid an attachment operation. Targeted controls cover partial upload,
lost attach reply, held reader/URL, expiry, corruption, orphan cleanup and no-clobber behavior.

<a id="F03"></a>
### F03 — Exact mappings and demand-shaped transport

Consume 72a's explicit numeric/presence/identity mapping. Use `SurrealValue` or generated native
conversion only where it preserves the declared policy; no universal native Proto encoder is
assumed exact. Tagged v4 conversion widens numeric/temporal representations and can encode large
decimals as text. Preserve IEEE bits and original message bytes only where their actual semantic
or archival consumer needs them. Delete repeated hand object construction once generated mapping
and every selected consumer have moved.
[Tagged conversion](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs).

Add incremental gRPC batch consumption for larger metadata/result selections, with cumulative
pre-allocation limits, original deadline and client/transaction custody. Distinguish a tentative
item from terminal success and keep statement cardinality/substantive error selection. Ensure
consumers release processed batches rather than accumulate the full response. Remove replaced
buffer assembly and per-row hydration on those routes; keep useful bounded complete pages.

SDK, CBORv2/FlatBuffers/shaped rows and durable HTTP sessions remain alternative mechanisms, not
fallback protocols. Select a changed transport only for a concrete fidelity/width/lifecycle gap
the direct path cannot address cleanly. Compare complete seams from source before constructing
any benchmark. A new subscription channel needed by 72e belongs under this same lifetime owner;
it does not inherit complete-query assembly or durable replay semantics by spelling.

<a id="F04"></a>
### F04 — Receive the changed custody boundaries

Exercise full-range integers/signed zero/NONE/NULL through real query binding and result decoding;
malformed duplicate keys, aggregate stream overflow, missing terminal frames, multiple errors
and cancellation while a reader holds a batch. Reopen acknowledges the original effect, not a
new request with copied inputs. For a changed file backend, use F02's independent data/lifetime
witnesses. These controls address serious correctness risk and can reuse Plan71's performing
fixture/report owners without claiming its scope. Delete obsolete tests of the replaced mechanism
rather than port them into a compatibility path.

## Coverage and completion

F01 captures MX/KV/standalone/distributed alternatives, native maintenance, encryption/value logs
and recorded versus scientific history. F02 owns buckets/presign/file custody. F03/F04 own native
codecs, client replacement alternatives, gRPC/CBOR/FlatBuffers/shaped results, streaming, cumulative
admission and terminal settlement; 72g owns any selected SQL/GraphQL operator adapter. Plan72a
owns mapping declarations, and 72c owns required isolation/authority. This avoids duplicate owners.

**Proposed verification.** Compile touched native targets and run only the targeted controls for
the changed mechanisms, with force-validation on Arrow closures. 72i owns integrated receiving
and scope-end qualification. Clear removal of unnecessary work needs no speedup benchmark;
measure only an unresolved serious performance degradation risk or a claimed quantitative benefit.
No product probe, test, engine selection or qualification ran during authoring.

## Outcome

**Implemented**, 2026-11-10. F01–F04 retain persistent integrated MX and sealed files, with
passive persistence observations bound to the original executable, private startup records,
unit/invocation and listener before and after the authenticated bounded metrics read. Supplied
Collector records have a distinct weaker origin. Missing native gauges, VERSION queries and
manual snapshot completion remain unknown/unsupported; deletion does not imply physical release.
The selected bucket contract lacks fsynced atomic no-clobber publication and recoverable
reservation failure, so the existing protected physical-file owner remains the single route.

`consume_batches` delivers borrowed tentative batches inside the original request, driver,
deadline and allocation custody. Per-frame admission and cumulative query accounting are
independent; a released batch does not reset the cumulative limit. Completion requires dense
selected statement framing, QueryEnd, EOF, trailers and every statement/COMMIT verdict.
Study capture metadata export streams contiguous 64-row frames into an incomplete staging
file, checks expected population and publishes only on complete success. Useful bounded
complete pages remain for ordinary reads.

**Tested** under the selected native Linux conditions in
[observation/custody receiving](72-surrealdb-functional-improvements.md#observation-and-custody-receiving):
80 joined native tests, 19 focused lifecycle/auth tests, the ordinary project reader and the
runtime frozen-cohort export test passed against zero failures. Independent controls cover
u64 maximum/bytes, malformed and late statement failures, cumulative overflow, original
deadline/backpressure, caller runtime withdrawal, detach refusal, retained close errors and
native 65-member streaming. Runtime export checks published extent/count and incomplete-file
retention on refusal. Existing artifact custody is retained; no new backend durability claim
or device-power-loss qualification is made.

**Mistake corrected.** A reply could precede physical drain scheduled on the observer's
current-thread executor, stranding cleanup when that caller stopped polling. Request,
reconnect, recovery, subscription and release work now stays on the original raw Client
executor through aggregate close. The selected server's neutral default token metadata on
successful Authenticate is normalized narrowly while actual rotations remain replayable.

**Deliberate deviations.** No SDK/protocol fallback, engine facade or native bucket sibling
is added. New history/capacity/topology participants and a complete bucket custody contract
are the observable revisit triggers above. Scientific source/interpretation and protection
owners remain intact. Blueprint §20.5/§21.7 and proposed ADR-0267 own enduring contracts;
72i retains series acceptance/retirement. F01–F04 implementation is complete.
