---
title: "Plan68b: SurrealDB v4 schema, transport, and authority"
status: planned
date: 2026-11-07
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, design, libraries, validation, storage]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# SurrealDB v4 schema, transport, and authority

## Target and baseline

Deliver the schema and receiving contracts for the single v4 generation. Preserve the
separation between source-generated schema, installed definition evidence, write admission,
and protected interpretation. This companion owns [O01], [O08], and the control API part
of [O09]. A owns the server implementation and study extension package. E consumes the
installation witness and settled behavior for cutover. 66b and 66e continue to own domain
contract generation and protected scientific interpretation. Plan 67's completed design is
the starting assumption. Finite private evidence and knowledge consumers are part of the
switch. The withdrawn persistent developer history pilot supplies no receiving path.

**Interface-checked baseline at `6da5f6289`.** The current registry emits canonical and
private-store schema and codecs. `pse-schema` parses typed field expressions, `xtask` has
parse-only tooling, and canonical maintenance parses and compares generated and installed
definitions. The bounded client consumes protocol 0.13.1 generated Prost declarations and
SurrealDB types 3.3.4. The borrowed preflight generator derives the accepted message shapes
from those declarations. Existing fixtures exercise structured errors, request owners,
stream settlement, and protection. Reuse the machinery and qualify it against v4 rather
than introducing a second parser policy, transport, or evidence schema.
[Canonical policy](../../crates/pse-schema/src/model/canonical_policy.rs) ·
[Maintenance witness](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Bounded client](../../crates/pse-operations/src/canonical_client.rs) ·
[Plan 66b](66b-contracts-generation-and-interpretation.md) ·
[Plan 66e](66e-protected-data-and-result-delivery.md)

## B01 — Generate and attest one v4 schema

Replace `surrealdb-syn` and `surrealdb-sql` across the schema generator, parse-only CLI,
maintenance verifier, and active checks. Select `surrealdb-parser-next` 4.0.0 `parse`
for syntax-only validation and `surrealdb-expr` 4.0.0 `parser::expr_for_definition`
for field expressions. The syntax parser returns an AST plus diagnostics. Treat a
diagnostic as refusal and report the compiled parser version and validation profile.
Use the typed expression parser and the tag's `parser::parse` or `query_with_settings`
where semantic lowering or a typed statement inventory is consumed. A parse result
alone does not certify actual server capabilities, execution, or installed state.
[v4 syntax parser](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/parser-next/src/lib.rs) ·
[v4 expression and query parser](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/expr/src/parser/api.rs) ·
[Parse-only caller](../../xtask/src/surrealql_tools.rs)

Install fresh level-4 definitions on the owned candidate server. Compare generated
declarations against the bounded live inventory obtained from `INFO FOR DB` and
`INFO FOR TABLE`, using `STRUCTURE` where an effective setting needs attestation.
Keep the existing typed semantic comparison and witness. Ignore a flag or server-assigned
identifier only where it cannot change semantics. Compare effective permissions, normalized
credential-redacted `DEFINE ACCESS`, table and field checks, indexes, function and API
bodies, and extension identity and grants. Include named types in the managed inventory
if B05 adopts one. Separate the installed index definition from readiness when an index
is populated asynchronously. Bind the witness to the server instance, target namespace
and database, initializer nonce, staged inputs and expected predecessor. Require exact
settled rollout evidence before activation. Fresh activation issues a new store authority.
An ambiguous activation response is resolved by exact receipt readback, with no second
authority allocation or implicit retry on another generation.
[INFO](https://surrealdb.com/docs/reference/query-language/statements/info) ·
[v4 compatibility and migration](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x) ·
[Index readiness](https://surrealdb.com/docs/reference/query-language/statements/define/indexes) ·
[Native activation](../../crates/pse-operations/src/canonical_maintenance.rs)

The parser, generated definitions, and expected witness contract can be prepared before the
server receiving controls run. Give A those inputs to test the runtime and install the
newly built extension. B01's final live witness requires A's privately prepared server and
extension receipt. The source preparation itself requires neither that final witness nor
activation of an existing store.

Run code generation from authored inputs. Compare selected v3 and v4 meanings for
repeated clauses, field evaluation order, coercion, `NONE` versus `NULL`, effect restrictions,
ordering and statement cardinality. Reject a drifted definition, unexpected managed entry,
wrong extension or receipt, and stale or foreign instance. Prove that parse success and
phase settlement cannot activate a changed catalog. Migrate all active producers and checks
as one family and retire the old parser dependencies, renderer assumptions and superseded
verification paths. E owns any named-original conversion. The schema generator does not
carry an active v3 installation path.

## B02 — Qualify the bounded gRPC client on the exact wire

Use protocol 0.14.1 and `surrealdb-types` 4.0.0 from the selected family. Keep the current
gRPC `Query` stream, negotiated message limits and selection ownership. Regenerate borrowed
preflight tables from the exact Prost declarations using the existing generator. Review
new `Value` and `Kind` variants and add explicit bounded handling or refusal at the existing
preflight boundary. Before owned decode, prove frame and aggregate bytes, occurrences,
nodes, depth and allocations are charged. In particular, conversion to a native object map
does not establish uniqueness of incoming keys. Keep `IndexedResults` slot accounting,
terminal-frame checks and substantive statement error precedence. Missing terminal evidence,
partial responses and malformed frames refuse. A result prefix grants no visibility or
publication authority. Use independently encoded fixtures and the actual native peer to
exercise malformed and legal frames, oversized nested and repeated fields, duplicate keys,
late statement or commit failure and cancellation.
[Protocol conformance](https://github.com/surrealdb/surrealdb-protocol/tree/v0.14.1/conformance) ·
[Protocol parity and exclusions](https://github.com/surrealdb/surrealdb-protocol/blob/v0.14.1/parity.toml) ·
[Preflight generator](../../xtask/src/codegen/grpc_wire.rs) ·
[Result collection](../../crates/pse-operations/src/canonical_client/results.rs)

Consume A's native gRPC ownership and error contract. Upstream v4 permits reconnectable
sessions and supplies bounded text for unary errors. Keep authentic accepted-connection
ownership, atomic admission and native typed errors via the bounded structured details
route. Decode those details through the version-qualified adapter, including the consumed
`Error::from_wire` behavior. Preserve actual socket loss, reserved cleanup, saturation and
drain semantics for plaintext and TLS. An abandoned caller cannot release a grant ahead of
the issued owner. Keep the original minimum clock and acknowledge commit, refusal, known
rollback conflict and uncertain effect as separate outcomes. Qualify replay against A's
actual attempt counts and E's original-effect controls. Rebase only the compatibility
mechanics needed by the single v4 client and remove obsolete v3 mechanics after parity.
[Tagged gRPC server](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs) ·
[Tagged native error API](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/error.rs) ·
[Transport controls](../../crates/pse-operations/src/canonical_transport_tests.rs)

## B03 — Keep value fidelity and interpretation under their existing owners

Prove exact current values through the v4 native `Value` Protobuf conversion. Keep the
canonical decimal u64 contract, finite IEEE bits and projection agreement, optional presence,
structured record identity and original opaque payload bytes. The supplier can widen sized
numbers, degrade newer temporal values, or represent out-of-range `BigDecimal` as text.
The selected contract never infers scientific values from those coercions. Test both u64
endpoints, fraction and overflow refusal, signed zero, non-finite refusal, absent versus
present zero, `NONE` versus `NULL`, and wrong kind or identity. Compare a golden fixture
with an independent decoder. If a named original uses a legacy encoded payload, the owning
reader establishes its interpretation before E decides whether to convert it. Do not re-encode
opaque bytes as a substitute for identity or provenance.
[Native Protobuf conversion](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs) ·
[Canonical codec](../../crates/pse-operations/src/canonical_codec.rs)

Keep the repository-pinned Buffa 0.9.2 `DescriptorPool` and `DynamicMessage` behind the
current scoped contract and independent readers. Verify the original descriptor closure,
reserved numbers, oneofs, presence and unknown field handling. Compare Buf compatibility
with the separate native semantic comparison, including its unknown coverage. Prove
protected scientific readback across native metadata, recorded-result Protobuf and Arrow IPC
under the selected original interpretation. Include missing or wrong descriptor and an
original result whose meaning differs from a current default. The dependency switch does
not adopt a new application encoding or reopen application schema generation.
[Pinned reflection consumer](../../crates/pse-contract-proto/src/lib.rs) ·
[Buffa 0.9.2](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/) ·
[Buf compatibility categories](https://buf.build/docs/breaking/rules/) ·
[Protobuf field presence](https://protobuf.dev/programming-guides/field_presence/)

## B04 — Preserve control API authority and rollback

Generate `DEFINE API … SECURITY DEFINER` for the existing browse, cancel and reconcile
handlers. Their controlled invocation of `PERMISSIONS NONE` functions requires definer
authority. Keep the actor, issuer, deadline, request extent and operation identity gates
inside the generated handlers and functions. Compare effective mode via the v4 `INFO FOR
DB STRUCTURE` semantics, where omitted default security can be reported as definer, and
attest it on repeat installation. Upstream `DEFINE API OVERWRITE` without a security
clause resets to definer. Test direct protected function refusal, anonymous, revoked and
foreign actors, oversized or invalid requests and issuer mismatch. Test both an intentional
error response and a failed handler after an earlier enclosing write. Stage savepoints
can roll back a failed stage without poisoning the enclosing transaction. Keep A's rollback
control unless the actual v4 receiving test proves the whole existing effect boundary.
[Generated control API](../../crates/pse-codegen/src/codegen/surreal_control.rs) ·
[Tagged API executor](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/exec/api.rs) ·
[Tagged API security tests](https://github.com/surrealdb/surrealdb/blob/v4.0.0/language-tests/tests/language/statements/define/api/security.surql)

## B05 — Test one shared named type

Compare a registry-generated `DEFINE TYPE pse_exact_u64 AS decimal` with an exact range
and integer `ASSERT` against today's repeated decimal field assertion and a callable
generation or ordinal parameter. Start with one existing generation field and a guarded
function that receives the same declaration. Require equal preexisting bounds, optional
presence and refusal behavior, plus a consumer benefit from the shared definition. Test
integer-to-decimal coercion, declaration order, effective permissions and installed
normalization, the exact maximum, negative, fractional and overflow refusal, and any
nullable wrapper on the selected path. Preserve codec checks and field/operation-specific
rules. [O01] demonstrates the syntax on a disposable 4.0.0 fixture. It does not establish
equivalence for a current guarded consumer. If the field and argument differ or the named
type widens an admitted shape, keep the qualified literal and callable checks and close
the investigation with the disqualifying case. B05 is independent of the cutover gate.
[Named types](https://surrealdb.com/docs/reference/query-language/statements/define/type) ·
[Generated field assertions](../../crates/pse-codegen/src/codegen/surreal.rs)

## Deferred boundary changes

- Leave `SECURITY INVOKER` for a named projection that requires caller-scoped reads and
  can use an explicitly bounded permission model without privileged mutations. A new
  binary HTTP interface needs a concrete recipient and its own parse and allocation bounds.
  [API statement](https://surrealdb.com/docs/reference/query-language/statements/define/api)
- Reopen CBOR v2, shaped rows or positive gRPC `max_batch_records` only for an actual
  independent receiver with demonstrated fidelity or memory need. Qualify decompressed
  and cumulative budgets before enabling compression. A second canonical client is outside
  the selected design. [CBOR protocol](https://surrealdb.com/docs/reference/rest-api/cbor-protocol) ·
  [Tagged gRPC server](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs)
- Leave `FLEXIBLE`, named ID components, `SQL TYPE`, and `MIGRATE EXISTING ROWS` to a
  named change with a matching owner, coercion, refusal and dependency/rebuild contract.
  Fresh regeneration requires no in-place field migration. [Field semantics](https://surrealdb.com/docs/reference/query-language/statements/define/field)

## Packages

Run the existing parser, schema, transport, codec, maintenance and
control tests on the exact candidate where they need a server. Reuse
passing assertions for the behaviors they distinguish and fix the
specific failures observed. Extend a test only when a v4 change or a
new interface affects a consumed contract that existing tests miss.
Plan68e owns the assembled receiving and recovery campaigns.

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| B01 | planned | Approved release boundary for generation. A's privately prepared v4 server and extension receipt for live attestation | Active parsers, generators and maintenance share the exact v4 release. Actual installed semantics and activation refusal controls pass. |
| B02 | planned | Exact protocol artifacts and A's receiving gRPC behavior | Generated preflight, terminal slots, ownership and typed errors pass on independent fixtures and the native v4 endpoint. |
| B03 | planned | B02 wire contract and existing Plan 66 readers | Native value round trips and protected result readback preserve original evidence and interpretation. |
| B04 | planned | B01 emitted definitions, B02 native transport and A's rollback control | Actual HTTP routes attest effective authority and prove denied or failed actions leave no unauthorized effects. |
| B05 | planned | B01 managed definitions, B03 value boundaries and a verified shared field/function use | Adopt one equivalently typed shared constraint or record the evidence that retains the current form. |

For local verification select the schema expression and CLI parser controls, the generator
and `just codegen-surreal-check`, and the affected `pse-operations` maintenance, codec,
transport and control tests. Use the native force-validated correctness route wherever an
Arrow-consuming closure is selected. Independent wire fixtures check the protocol and the
owned v4 endpoint checks its actual effects. Pass both before claiming parity. E owns the
assembled protected scientific and developer receiving journeys and their zero-baseline
result. This plan records proposed work and interface evidence.

[O01]: ../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O01
[O08]: ../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O08
[O09]: ../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O09
