---
title: "Plan68a: SurrealDB v4 supplier, runtime and extension"
status: planned
date: 2026-11-07
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, deployment, preparation, storage, validation]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# SurrealDB v4 supplier, runtime and extension

## Target and baseline

Deliver a privately selectable, source-built v4 native server and matching
Study readiness extension. The preparation owner proves executable and artifact
identity. The native service proves launch, bounded operation and physical
settlement. The catalog and activation owners prove schema and installed
capabilities. Plan68b receives the server-side gRPC and API contracts, 68c
receives the qualified runtime resource controls, and 68e owns the durable
selection. This companion carries the runtime portions of
[O06](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O06),
[O08](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O08), and
[O09](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O09).

**Interface-checked.** At main `6da5f6289`, PSE prepares its patched 3.3.4
server and SurrealMX 1.0.0 from exact sources. Current release import already
reuses a verified matching generation. The inspected tagged v4 adapter owns
MX 1.1.0 integration, and its explicit isolation selector and persistence
options give a narrower adaptation point. The v4 gRPC server explicitly
describes process-wide reattachable sessions, so upstream defaults do not
establish the existing PSE lifetime contract. Qualification of the actual
patched v4 service remains planned. Plan67 completion supplies the prepared
generation reuse, fixture and retirement routes. Reconfirm their exact
receiving interfaces at execution start. Plan66's live scientific work and
unclosed acceptance retain their original owners.
[Current preparation](../../crates/pse-host/src/server.rs) ·
[Release import](../../crates/pse-host/src/server/release_import.rs) ·
[Native service](../../crates/pse-operations/src/native_service.rs) ·
[Plan67](67-storage-lifecycle-and-build-reuse.md) ·
[Tagged MX adapter](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/kvs-mem/src/lib.rs) ·
[Tagged gRPC server](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs)

## A01 — One prepared source generation

Acquire `v4.0.0` at `284fe93afc851c3baec2c262c01289602021a70d`.
The tagged lock selects MX `1.1.0`, Surrealism `0.7.0` and
`surrealdb-protocol` `0.14.1`. Align the selected SDK and
`surrealdb-types` `4.0.0` with 68b and 68d. The database package
`surreal-db` emits `surreal`. The independently versioned package
`surreal-ext` `1.0.0` emits the extension tool. Prepare both from the
same authenticated source with the pinned nightly, locked resolution,
and the current release unwind profile. Build the database with
`--no-default-features` and explicit `storage-mem,cli,surrealism`.
Inventory the old compiled `mcp` feature's actual consumers before
dropping it. Preserve any admitted existing consumer through the v4
equivalent and keep the listener capability disabled unless separately
admitted. The repository's SQL/API requirements remain covered by the
tagged server core feature set and the prepared test matrix. Keep the
extension builder local and separately identify its feature selection.
Resolve only the exact family and required shared dependency moves.
The tagged workspace uses `serde_json` 1.0.154 and `uuid` 1.28.0.
Recheck those requirements against the completed Plan67 workspace and
vendored constraints. If the older exact pins still conflict, move only
the affected pins and lock entries, preserve reconstructable vendor
deltas, and exercise their actual consumers. Avoid a wholesale lock
refresh.
[Server manifest](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.toml) ·
[Release lock](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.lock) ·
[Server features](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/Cargo.toml) ·
[Extension tool manifest](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/ext/Cargo.toml)

Extend the existing preparation receipt to bind both executable
digests and package identities to the revision, ordered patch set,
lock, target, build profile, features and original toolchain context.
Publish the pair only after both files pass verification. Select and
reopen the immutable prepared pair without rebuilding or starting a
catalog. An explicit release import verifies both copied open files
and the lock, refuses wrong source or incomplete tool pairs, and
reuses an identical verified installed generation. Retain the frozen
v3 selection through the pre-write rollback rehearsal under 68e.
An interrupted preparation, a substituted executable or a stale
build output leaves the previous selected receipt usable.
[Existing source and import ownership](../../scripts/surrealdb-source/source-build.md)

## A02 — Native engine, transport and API guarantees

Select persistent MX for the canonical service. Use the adapter's
`Isolation::Serializable` mapping to
`with_serializable_snapshot_isolation` for canonical writes. Cover
ordinary statements, guarded programs and explicit transactions that
can write. The tagged `Datastore::transaction` defaults to snapshot
isolation, so retain a narrow native integration for any canonical
write entry point that cannot establish serializability through the
release API. Keep current native read and maintenance authority.
Use the existing owned startup profile with synchronous
`AolMode::SynchronousOnCommit`, `FsyncMode::EveryAppend`, the periodic
60-second snapshot, and versioned queries disabled. Refuse incompatible
persistence settings before opening the state directory. Preserve
positive limits for memory, transaction age and queue depth and bytes.
Keep the service's original query/transaction timeouts, scratch owner,
and denied guest/network capabilities.
[Isolation options](https://surrealdb.com/docs/reference/query-language/statements/begin) ·
[Tagged MX adapter](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/kvs-mem/src/lib.rs) ·
[MX options](https://github.com/surrealdb/surrealmx/blob/v1.1.0/src/options.rs)

The tagged v4 standalone statement executor retries only eligible,
known-rolled-back conflicts, with up to ten attempts under one
transaction-timeout budget. Explicit and caller-owned transactions
remain caller-owned. Replay is withheld when external effects occurred,
rows left the staging bound or commit already handed off subsequent
work. PSE still owns deadline, replay and ambiguous acknowledgement
for the complete guarded operation. Verify original effect counts
separately from attempt counts. Keep the publication witness for empty
or future membership predicates. An existing-record conflict or a
read-only snapshot supplies a different guarantee.
[Tagged executor](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/dbs/executor.rs) ·
[Publication witness](../../crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs)

The physical server keeps the connection identity, session charge and
cleanup owner across plain and TLS paths. Plan68b owns the consumed
typed status and wire behavior, role checks, and actual enclosing API
rollback assertions. Agree those interfaces before freezing the
supplier patch set. Qualify server and client together on an owned
endpoint, including transport loss and unavailable original status.
[Tagged gRPC server](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs) ·
[Tagged API execution](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/exec/api.rs) ·
[API security controls](https://github.com/surrealdb/surrealdb/blob/v4.0.0/language-tests/tests/language/statements/define/api/security.surql)

## A03 — Rebuild the pure Study readiness extension

Prepare the existing `study::readiness` export for `wasm32-wasip2`
through the exact selected `surreal-ext build` executable. Move the
authored package to `package.toml`. Keep the production export and
qualification artifacts separate. The v4 packager stamps the oldest
eligible ABI, so the pure package with no typed bindings must produce
ABI 4. Verify its package manifest, exact export, no declared
transaction and effective grant. ABI 5 adds typed bindings and is a
deferred follow-up for a concrete configuration input. Retain the
readiness policy source and native comparison, with no SQL, write,
network or external effect grant for the pure export. The generated
guarded SQL functions keep canonical capture and publication.
[Extension CLI and build](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/ext/src/build.rs) ·
[ABI contract](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealism/runtime/src/abi.rs) ·
[Existing readiness export](../../crates/pse-study-module/src/lib.rs) ·
[Existing capture operation](../../crates/pse-codegen/src/codegen/study-capture.surql)

Move active SQL producers and inspection to `DEFINE EXTENSION ext::pse`
and `ext::pse::study::readiness`. Generate the local bucket source,
`DIGEST 'sha256:<hex>'`, `UNSIGNED`, effective `CAPABILITIES` and
explicit call permissions from the prepared package and its admitted
caller scope. Preserve two different attestations.
The preparation receipt verifies the whole `.surli` archive. The
installed definition's `DIGEST` verifies the SHA-256 of
`surrealism/MANIFEST`, whose contents bind the archive members.
Use the existing private read-only module bucket and explicit unsigned
local admission only after the exact prepared artifact is verified.
Keep the extension capability gate and directory allowlist. Verify
stable file storage separately from the still-gated extension feature.
Validate the installed export, effective grant, runtime limits and
capability probe before the activation owner admits use.
[DEFINE EXTENSION and digest](https://surrealdb.com/docs/reference/query-language/statements/define/extension) ·
[Tagged definition builder](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/ext/src/install/statement.rs) ·
[Archive manifest](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealism/runtime/src/package.rs) ·
[Native artifact admission](../../crates/pse-operations/src/canonical_study_module.rs)

The v4 runner instantiates fresh guest state in a bounded Wasmtime slot.
Its pooling allocator provides one linear memory per slot and reserves
some capacity for nested calls. Compare
`SURREAL_SURREALISM_MAX_CONCURRENT_INVOCATIONS` and
`SURREAL_SURREALISM_MAX_MEMORY` with the current total-controller and
per-invocation bounds, including the nested reservation. The tagged top-level wait behavior
differs from PSE's fail-fast admission. Keep typed fail-fast refusal
with a process-wide charge through physical release, and carry the
original deadline through slot admission, cold preparation,
instantiation and execution. Adopt upstream limits where the physical
controls prove equivalence. Check that a cancelled observer cannot
release the real performer's charge. Run the separately built probes
for overlap, capacity, guest trap, excessive allocation, noncooperative
busy guest and cold-path timeout.
[Runtime pooling](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealism/runtime/src/epoch.rs) ·
[Extension runner](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/surrealism/runner.rs) ·
[Native module controls](../../crates/pse-operations/src/canonical_study_module_tests.rs)

## Patch decisions and revealing controls

The preparation owner records the final ordered patch set. Use the
existing tests on the private v4 candidate first. The table identifies
the behavior each disposition must preserve. Existing assertions that
distinguish that behavior satisfy the row. Add a focused case for a
changed, consumed contract only when current tests miss the regression.
A failed test keeps the narrow adapted behavior in the v4 source build
or blocks private qualification when a conforming port is unavailable.
[Current patch inventory](../../scripts/surrealdb-source/source-build.md)

| Current input | Proposed v4 disposition and acceptance |
| --- | --- |
| `mx-1.0-minimal.patch` | Replace the v3 engine upgrade and byte adaptation with the tag's owned MX 1.1 integration. Confirm the exact lock and raw key/value round trip. |
| `mx-1.0-controls.patch` | Use upstream selectable SSI and preserve canonical write coverage, finite positive bounds, typed engine causes and safe metrics. Exercise write skew, absent-range phantoms, bound refusal and uncertain persistence. Adapt surviving controls where the tag lacks equivalence. |
| `canonical-profile.patch` | Preserve explicit refusal of non-durable or invalid persistent settings, including a snapshot interval at or below 30 seconds. Check unopened state on refusal, acknowledged crash/reopen and interrupted snapshot replacement. |
| `grpc-connection-ownership.patch` | Rebase actual accepted-connection identity and cleanup. Check stale/foreign reconnect, TCP/TLS loss, release of open work and new connection admission after physical settlement. |
| `grpc-session-admission.patch` | Retain atomic global reservation and charge until native cleanup. Test racing connections at the last slot and a deliberately held cleanup. |
| `grpc-status-details.patch` | Preserve `surreal-error-bin` structured error metadata with its existing 8 KiB bound. Check explicit commit conflict, oversized text and ordinary nonretryable failure against 68b. |
| `api-failure-rollback.patch` | Keep enclosing-transaction poisoning until actual earlier writes roll back on handler-expression failure. Independently exercise intentional API responses and actor refusal. |
| `diskann-0.56.0.patch` | Retire the v3 archive and call-site adaptation after the tag's lock-resolved `surrealdb-diskann` 0.61.0 family builds with the selected toolchain. No vector feature adoption follows. |
| `pinned-compiler.patch` | Try the exact tagged build with the pinned nightly. Retire obsolete hunks and carry only reproduced compatibility fixes under exact pins and the same unwind policy. |
| `surrealism-locked-build.patch` | Move locked Cargo metadata and build enforcement to the actual separate builder invocation. Refuse changes to Cargo configuration and lock during guest build. |
| `surrealism-build-features.patch` | Preserve explicit feature forwarding. Prove production, qualification and long-hold artifacts have their exact intended exports, limits and distinct receipts. |
| `surrealism-global-controllers.patch` | Use tagged pooling where qualified, retaining stricter process-wide fail-fast physical admission. Exercise simultaneous cold/warm calls and refusal until actual slot release. |
| `surrealism-memory-count.patch` | Prefer tagged one-memory slot constraint. Test a second memory and an oversized allocation. Preserve a narrow local limit if equivalent refusal is missing. |
| `surrealism-controller-deadline.patch` | Use tagged context and module deadlines where effective, with original remaining time charged before queueing, cold initialization and invocation. Adapt missing propagation and verify cancellation, strict busy interruption and physical drain. |

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| A01 | planned | Plan67 handoff, Plan68 decision route, exact tag and pinned toolchain | Explicit source preparation and matching release import select one verified executable pair. Corruption, incomplete input and interruption preserve the previous selection. |
| A02 | planned | A01 source and build recipe, B's agreed wire and API contract | Patched v4 private service proves canonical write isolation, persistent durability and runtime bounds. Plain/TLS session ownership, cap reservation, typed errors and API rollback pass paired controls. |
| A03 | planned | A01 prepared builder, A02 runtime capabilities, B01 emitted definitions and witness contract | Pure ABI 4 readiness builds, installs, attests and executes through actual native preparation and claim. Qualification probes distinguish guest failure, refusal, timeout and physical settlement. |
| A04 | planned | A02 and A03 tested replacements, actual consumers identified | Final source patch ledger removes superseded integrations with their consumers and tests. Deliver exact receipts and selected native test evidence to 68e. |

## Verification boundary

Use the native host server preparation and explicit release-import paths
in private state, then targeted host, native service and Study readiness
controls. Exercise actual source-built server identity, acknowledged
write/reopen, known conflict, unknown persistence result, malformed or
unauthorized request, owner loss, and physical cleanup. Use the
repository's pinned correctness adapter and explicit Arrow force
validation where the selected test graph consumes Arrow. Report original
outcomes against zero failures. A source match and the review's official
memory probes are **Interface-checked** or bounded prior **Tested**
observations. They establish neither this server build nor its durability
and process ownership. 68e owns assembled acceptance and actual
generation selection.
[Current source-build controls](../../scripts/surrealdb-source/source-build.md) ·
[Qualification guide](../dev/validation-assessment.md)
