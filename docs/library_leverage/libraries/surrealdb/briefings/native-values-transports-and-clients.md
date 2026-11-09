# Native values, transports and clients

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283`, with the current
dirty Plan66 tree preserved. Current-use observations are **Interface-checked** by representative
source reads. Directions and benefits are **Proposed**; no protocol test or measurement ran.
Breaking representation and client changes are eligible where they improve existing functionality.

## Current responsibilities

The production boundary already uses generated gRPC protocol 0.14.1 and native types 4.0.0,
not an SDK query wrapper. [Client](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_client.rs)
owns connection/session setup, dispatch and disconnect. Its
[borrowed wire preflight](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_client/wire.rs)
admits aggregate bytes, nodes, depth and container/string/blob occurrences before Prost
allocation. [Result assembly](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_client/results.rs)
holds ordered statement results until `QueryEnd` and trailers settle; extraction also retains
cardinality and substantive-error rules.

[Canonical codecs](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_codec.rs) encode unsigned
values through exact Decimal, preserve diagnostic IEEE bits and signed-zero-bearing scientific
values separately from query projections, and distinguish IDs, strings and record links.
The local `protocol_v4_sized_numbers_preserve_supported_widths` test exists in source; it was
not rerun and does not establish a whole native encoder round trip.

Standalone SurrealKit uses the SDK through an independently prepared executable. Importing its
SDK selection into production is an alternative to assess rather than a required consequence
of using the same database. Blueprint §20 and §23 own current custody and error semantics.

## Capability families and availability

| Mechanism | Origin / state | Potential contribution |
| --- | --- | --- |
| Native `Value`, `Kind`, `SurrealValue`, generated gRPC | Pre-v4 foundations, already used | Shared native conversions and typed database requests; reduce manually duplicated mapping where policy agrees. |
| Sized numbers/new temporal types | Expanded in v4; decoder and encoder differ | More direct canonical storage if end-to-end fidelity and generated schema agree. |
| `encode_proto` / `decode_proto` | Existing public codec, v4-specific conversion behavior | Database-aware fixtures/export or shared conversion; not arbitrary domain-message persistence. |
| `cbor.2` and shaped row blocks | V4 reference leads; selected-client support unverified here | Richer type fidelity or less repeated row metadata for existing metadata readers. |
| SDK `stream_items`, gRPC batches, WS `query_stream` | Existing/expanded surfaces; engine-dependent | Incremental consumption and lower simultaneous materialization where actual implementation streams. |
| HTTP attached sessions | V4 durability change in supplied reference | Reconnection support, with persisted authentication state and a different lifecycle obligation. |
| SQL-wire, GraphQL, GQL | V4 expands dialect/GQL choices | Reuse standard clients for existing inspection; verify native-value, permissions and query subset. |

The [reusable runtime contracts](../v4-runtime-contracts.md) own inspected codec asymmetry,
protocol parity and source-version cautions. The older [protocol account](../protocol.md)
and [SDK account](../rust-sdk.md) retain 3.3-era evidence; their supplier selection is historical.

## Most promising directions

**First investigate replacing repeated native mapping with generated policy-aware conversions.**
Compare owner-derived `SurrealValue` implementations, native codec helpers and the current
generated projection together. This could remove repetitive object construction and decoding
without forcing every scientific value through a generic serialization route. The deciding
question is which current conversion branches encode domain meaning and which only duplicate
native structure. A full-range `u64` path must account for the selected outgoing encoder;
decoder support alone is insufficient.

**Investigate incremental result consumption as a complete caller design.** Existing readers
already request bounded pages. A stream can reduce buffering for larger selected metadata
operations if the consumer processes items without simultaneously retaining the whole reply.
Compare direct gRPC, SDK gRPC, and a suitable CBOR/shaped path with the same result demand.
The removable machinery might be intermediate collection assembly or repeated per-row keys,
not all response ownership. A consumer exposing partial rows must explicitly distinguish
tentative delivery from complete statement/operation success.

**Compare the smallest complete client boundary.** The released SDK may now discharge more
session, transaction, typed result and error work than the old patched SDK did. Evaluate it
with the current preflight, original deadline, malformed-response and disconnect requirements.
Conversely the existing direct protocol may remain simpler when an SDK replacement still needs
every local admission and settlement layer. Any chosen replacement should remove the superseded
path rather than create transport fallback behavior.

**Use standard interfaces selectively for present inspection consumers.** A read-only SQL or
GraphQL reader could reduce custom client projection and schema-discovery code. Compare actual
project/evidence queries, current authorization and exact native types against the documented
dialect/generated-schema limits. Standard connectivity does not supply complete SQL semantics
or a scientific public contract. GQL graph design belongs in the
[graph brief](graph-representation-and-traversal.md).

## Counterconditions and next questions

1. Which selected reader allocates or transfers enough repeated metadata to justify another
   encoding? Measure the complete allocation/transfer lifecycle after identifying it.
2. Can one supported native conversion preserve bytes, `NONE`/`NULL`, `u64::MAX`, signed zero,
   sized-type identity and selected temporal values in both directions? Separate required
   value equality from required type equality and byte identity.
3. Which protocol conformance scenarios cover gRPC and the chosen failure path? Supplement
   narrowly for missing terminal frames, duplicate object keys, oversized aggregate streams,
   multiple failed statements and lost acknowledgements.
4. Can public SDK seams provide cumulative pre-allocation admission and physical connection
   completion? If not, retain or redesign only the demonstrated gap.

No new frontend, foreign-language product or general database gateway is justified by this
brief alone. The current bounded gRPC path remains a strong candidate for small typed pages.
An archive, network result and application contract can reasonably use different representations.

## Evidence and handoff

Read the supplied [v4 Protobuf reference](../../../../libraries/proto/Protobuf_SurrealDB_v4.md),
especially its native codec, transport and conformance addenda, alongside the
[v4 overview](../../../../libraries/surrealDB/SurrealDB_v4.md). Inspected primary
[native conversion source](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs)
establishes implementation asymmetries, not the relative value of another transport.
The current [CBOR page](https://surrealdb.com/docs/reference/rest-api/cbor-protocol) fetched
legacy material; exact `cbor.2` negotiation and chosen client behavior remain focused research.
No generated semantic reference census or API bundle was prepared. Next work is focused
applicability and complete-client comparison, not adoption or product qualification.
