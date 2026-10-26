---
id: ADR-0228
title: Own canonical contracts, bounded payload placement and offline recovery
status: proposed
date: 2026-10-25
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-07, DP-01, DP-04, DP-09, PS-10]
blueprint: [§3.2, §20.1, §20.2, §20.4, §20.5, §20.6, §21.1]
review: docs/design_review/reviews/design_review_canonical-placement-and-recovery_2026-10-25.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A required payload, recovery or caller lifetime cannot be represented by the closed locator and owner-specific settlement contracts.
verification: Plan56c declared-key/state/backup controls, dual-cause settlement failures, mixed inline/sealed protected reads and offline restore, fresh v8-to-v9 activation, actual Python/MCP/CLI agreement.
standard: core-3.4/process-simulator-1.5
scenarios: [docs/plans/56-design-principle-alignment.md#s05, docs/plans/56-design-principle-alignment.md#s08, docs/plans/56-design-principle-alignment.md#s10]
---

# ADR-0228: Own canonical contracts, bounded payload placement and offline recovery

## Context

DA-007/033 identify storage meaning and uncertain-effect settlement reconstructed by generators
and individual consumers. Plan56a/b already supply physical context and original/product scientific
permission; storage transports these facts. Current results force every encoded member through
file reserve/seal, and Python duplicates native file layout and integrity policy.

## Scope

Amend blueprint §3.2/§20/§21 for authorized Plan56c C01–C05. RC01 selects bounded inline content
plus sealed IPC. RC04 selects only the product offline recovery helper in existing pse-operations;
fixture controls remain in xtask, while worker/project relocation remains with 56e. No new crate,
scientific authority, compatibility reader, online backup or cross-host file support is introduced.
ADR-0187 retains the sealed branch rationale; this supplement changes its file-only placement rule.
Formal ADR status remains proposed pending the decision-PR route.

## Drivers

Canonical key, enum, transition, association and retention meaning must be owned once and consumed
by Rust/SurrealQL/Python. Exact effects survive lost responses without retrying science or renewing
fences/clocks. Small demanded outputs should avoid the physical file protocol while retaining exact
bytes, scientific admission, protected reading and complete offline recovery.

## Options

Keep file-only placement with grouped metadata: valid but retains tiny-file reserve/lock/fsync/seal
work. Put all payloads in the database: needlessly subjects large IPC to RPC movement and loses the
existing lazy file realization. A custom archive/log or second database adds lifecycle authority.
Select a closed inline/sealed locator through the existing canonical owner. Keep separate owner
protocols while sharing their effect-knowledge operation; do not introduce a universal transaction
or schema DSL. Use native complete file operations behind the existing Python freeze orchestrator.

## Outcome

### Declared meaning and settlement

Existing registry declarations own canonical text-key kinds/references, closed codec/owner/state
vocabularies, transitions/projections, immutable fields, special admission/variant associations,
artifact profile membership and separate backup/retention classification. Canonical text keys
remain distinct from scientific SemanticId and preserve their bytes. Generate mechanical codecs,
assertions and projections; remove generator name interpretation and parallel backup lists.

Product producer authority is a declared General/LibraryMathematics/LocalScientificReplay class,
separate from the exact producer receipt string. Native receiving capability determines scientific
class; ordinary publication cannot mint it by using a reserved-looking string. Class participates
in canonical publication/descriptor identity and exact candidate matching. Scientific body identity,
ProductBlob and replay v5 bodies do not change. Opaque key wrappers preserve original spellings;
new v9 publication bindings include the newly explicit authority fact.

A small shared settlement domain distinguishes exact acknowledgement, different immutable request,
owner-proven refusal/absence and unresolved effect. Unresolved results retain both original and
readback causes with owner/intent attribution. A missing row alone supplies no absence proof.
The original deadline, cancellation and fences govern readback; owner atomicity/lifetime differs.

### Placement and protected lifetime

Select placement from one complete admitted encoding before any file reservation. Inline payloads
are eligible through 64 KiB inclusive. Append transactions target 2 MiB including encoded inline
content, additionally respecting the actual 4 MiB RPC envelope; the existing separate 128 KiB index
bound remains. The complete encoded unit may contain homogeneous native IPC batch members; its
body is stored once, with exact member coordinates. Larger payloads retain 16 MiB target/32 MiB
hard homogeneous uncompressed IPC files. Demand flushes immediately. Allocation pressure does not
silently change placement. Existing opaque JSON/bytes codecs remain explicit.

A tagged locator binds codec, schema/interpretation, digest, exact extent and member coordinate;
inline content has no synthetic file UUID or empty-schema sentinel. Publish its immutable body,
logical descriptors and exact acknowledgement atomically under existing fences. Shared encoded
members reference one body. Sealed locators retain original file intent/descriptor custody through
unknown settlement. Both branches use original independent scientific admission and one protected
read/visibility owner. Inline lifetime follows canonical references/read tickets; collection and
backup never manufacture file entries for it. Empty demanded relations remain closure entries.

### Offline authority and cutover

The product pse-canonical-recovery binary lives in existing pse-operations; fixture seeding/signal
controls have a separate xtask target. Rust owns file placement, exact alias/extent/digest checks
and factual process identity observations. Python retains service/manager/cgroup freeze/drain and
maintenance orchestration; no filesystem observation proves application quiescence.

One native operation streams bounded inventory entries and owns each instance's lock and stable
file descriptor through reconciliation, hash/copy/fsync and final validation for that instance.
Supervisor freeze/closed admission spans the complete operation. Do not release a preflight lock
before later Python copy. Restore uses stable archive descriptors and a fresh destination, without
requiring live source locks absent from the archive. Preserve exact no-follow/private identity and
the narrowly authenticated final/pending hard-link recovery case.

Publish declarations and all writers/readers together under pse.substrate.v9. Only newly owned
disposable state is reset: close admission, authenticate/drain, stop, fresh-engine --drained,
initialize/validate closed, then explicitly readmit. Interrupted phases retain their exact target
and remain closed. Preserve the selected preexisting store and 56b databases/receipts.

### Consequences

Migrate append, snapshots/readers/cursors, inspection, export/import, backup/restore and release
with the locator. Inline bytes are charged through native RPC and reader drain, distinct from
bounded descriptor/index accounting. No one-file-per-result fallback remains for eligible content.
Do not rebump Completion/replay/fit-profile when their scientific bodies remain unchanged.

### Compensating controls

Independent expected declaration examples; false prefix/wrong domain/unknown state; lost ACK,
different request, failed readback and expired original clock; threshold/envelope boundaries;
exact IEEE/null/schema; paused protected reads and release; mixed backup/restore, corrupt/aliased
files, unknown owners and interrupted cutover. Native file work holds its true lifetime guards.

### Confirmation

Proposed at c3b11ba63, before dependent implementation. The focused decision review accepts the proposed target at its inspected boundary; it supplies no
implementation or runtime qualification. Plan56c owns actual implementation and qualification;
source reasoning is not scientific, host, operational or performance evidence.

**Implemented/Tested — 2026-10-26:** the [Plan56c Outcome](../plans/56c-canonical-contracts-and-effects.md#verification-and-outcome--2026-10-26) records selected local Linux canonical/runtime/installed Python/public/private recovery acceptance, original failures and repaired checks. Actual mixed recovery preserved original scientific facts; private source and restored services are stopped. This evidence does not change the proposed formal decision status or qualify wider Plan56, shared-context restore, other platforms or performance.

## Pros and cons

Small encoded outputs avoid a second physical lifecycle; large IPC keeps its existing native path.
The cost is a coordinated two-arm locator migration through every durable consumer and accounting
for real inline byte ownership rather than treating it as metadata.

## More information

[Plan56c](../plans/56c-canonical-contracts-and-effects.md) owns execution; Plan56 owns finding/rule
dispositions. ADR-0187 supplies retained sealed publication/protection rationale, ADR-0227 supplies
scientific permission and operation facts. Context7's official SurrealDB bytes/transaction docs
and existing pinned v3.3.4 Value::Bytes bindings establish the selected primitive, not runtime proof.

## Status history

- 2026-10-25: Proposed after explicit RC01 and narrow recovery-helper RC04 selection and execution authorization; formal status acceptance remains separate.
