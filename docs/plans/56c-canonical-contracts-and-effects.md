---
title: "Plan 56c: Canonical contracts and effects"
status: in-progress
date: 2026-10-25
adrs: [ADR-0227, ADR-0228]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, storage, numerical-execution]
---

# Plan 56c: Canonical contracts and effects

## Purpose and foundation assessment

**Proposed:** canonical declarations own decision-relevant storage meaning, shared settlement
preserves exact effect knowledge, and payload placement removes small-file machinery without
changing scientific admission. This companion owns DA-007/033 and supplies durable contracts to
B/D/E/F. [Plan 56](56-design-principle-alignment.md) owns shared decisions and finding status.

Reuse `RegistryBuilder`, generated row/SurrealQL codecs, attempt transitions, exact protections,
original fences/clocks, native transactions and immutable acknowledgements. SurrealDB remains
canonical. The gap is table-name/literal/prefix interpretation around those mechanisms, not a need
for another database or one universal lifecycle framework. B owns scientific permission; storage
transports its complete facts without deciding them from terminal class or payload presence.

## Shared target

### Canonical semantic declaration

Declare closed owner/state/outcome/codec vocabularies, opaque key kinds and constructors, explicit
special admission/variant associations, table retention/backup classifications, immutable fields,
transitions and exhaustive projections at their semantic owner. Generate Rust/SurrealQL/Python
representations and assertions from these facts. A canonical lookup key remains distinct from a
scientific semantic identity; typed wrappers do not turn arbitrary storage text into scientific ids.

Migrate substrate, project, import, context, research, product/artifact and execution owners, not
only the attempt table. Ordinary study-point membership already exists and is preserved. Replace
special run/point relationships reconstructed by prefix with declared fields. Raw parsing belongs
at actual external boundaries; unknown current enum/state versions refuse instead of becoming
`Failed` through a wildcard. Artifact profile membership uses declared relations/profile members.

`surreal.rs`' table-name branches and `rust/enums.rs`' terminal-name special cases must disappear
when their declarations are consumed. Moving switches into a renamed helper is not closure.
Scientific registry types keep their authority; E/F separately own the proposed responsibility-
scoped development catalogues under RC02/RC03.

### Common settlement meaning, distinct owner protocols

A small shared domain operation combines original error/effect facts with an owner's typed
readback: exact matching acknowledgement, operation identity reused with different immutable
request, proven refusal/absence where that owner can establish it, or unresolved effect retaining
both original and readback causes. A missing row is not universally proof that no effect happened.

Use it across every owner claiming the common §20.2 contract, beginning with the concrete project
and research divergence. Long-lived attempts, staged import, finite creation windows and atomic
project references keep different guards and lifetimes. Do not collapse them to one envelope that
differs only in predicates. Public diagnostics retain owner/intent attribution through the shared
knowledge result; retries consume that result, not a coarse failure class alone.

### One payload admission, two physical placements

The preferred RC01 proposal admits the same encoded scientific bytes once, then uses a typed
locator alternative: bounded inline content in the canonical transaction, or a sealed homogeneous
IPC file descriptor/batch coordinate. Schema/encoding, interpretation, exact size/digest, relation,
run/attempt and ordinal remain explicit. No fake file id, empty schema sentinel or legacy decoder
is used. Both placements use the same original validation, visibility and read-protection owner.

Selected initial placement defaults are 64 KiB per complete inline encoded body and 2 MiB target encoded
append transaction. One body may contain homogeneous native batch members, but its complete encoding
must meet the inline limit; member descriptors share that body once. Admission is additionally
bounded by the existing actual RPC admission after envelope
accounting. Declare these defaults once in the publication policy; they are tunable physical
choices, not scientific identities, host limits or capacity promises. Test threshold boundaries
before delivery. Demand never waits merely to fill a batch.

Larger compatible batches retain native IPC and lazy protected readers. Empty demanded relations
are explicit close-inventory entries. Small completion/seed/progress payloads consume the same
placement contract where eligible. This removes their file reserve/lock/fsync/seal protocol rather
than merely making its calls asynchronous. Do not introduce ZIP/TAR, a custom multi-schema
container or mutable per-writer logs.

If RC01 is rejected, use grouped metadata and larger same-relation IPC packing while retaining the
file-only contract. That is a valid reduced target, but its remaining tiny-file work is explicit;
it must not be represented as delivering the preferred mechanism unchanged.

## Packages

### C01 — Semantic declarations and typed consumers

Extend the existing declaration model with the owned distinctions above. Derive current
assertions, key/association constructors, codecs, lifecycle projections and backup/retention
classification. Migrate `canonical_*` operations, runtime/durable/artifact/result-selection APIs,
exchange documents and Python keys together with generated outputs. Delete parallel vocabularies,
field rules, enum projections and special prefix association recovery after callers move.

### C02 — Exact settlement facts and diagnostics

Implement the small shared settlement operation and migrate project, research, execution,
context, artifact, analysis, maintenance, import and file owners where their guarantees coincide.
Keep owner-provided proof of absence and atomicity. Retain original causes on failed readback;
differing requests never masquerade as a successful idempotent replay. B06/F03 consume typed
failure/effect facts without reconstructing permission or replay safety.

### C03 — Completion, replay, context and payload locator contracts

Consume B01/B03's governing facts and tagged constructor, A03's typed physical identities/context
projection, and the selected payload policy. Version affected durable documents/relations once
for the coordinated target. Migrate append, readers/snapshots, result inspection/cursors, export/
import, application backup inventory, restore and release paths. Both inline and sealed content
retain exact closure/digest admission; inline lifetime is protected by canonical references and
read tickets, not an invented file instance. Remove `prepare_result_file`'s unconditional conversion
of `ResultPayload` to a file and old locator/sentinel consumers in the same cutover.

### C04 — Native file authority for online and offline consumers

Extend the existing Rust canonical file owner with bounded offline operations for layout,
alias/extent/digest checks and authenticated owner observations. E02 exposes the actual product
helper; fixture seeding/signal controls stay in xtask. Python backup/restore delegates those shared
facts instead of repeating file-instance/death policy. Keep invocation/cgroup/namespace identity,
protective uncertainty, exact locks and offline freeze requirements. Delete duplicate
`payload_files.py` policy only after all applicable callers migrate.

### C05 — Controlled fresh interpretation activation

Integrate the declarations, generated outputs and all current producers/readers before changing
store interpretation. Follow the authorized fresh-regeneration lifecycle: close admission, drain
authenticated holders/workers/transports, refuse unrelated or unknown borrowers, stop the exact
owned server, regenerate selected disposable state, initialize and validate under closed admission,
then explicitly readmit the target. Source documents and unrelated workspaces are outside reset
authority. Interrupted phases remain closed and recoverable. No old-data migration/backfill,
compatibility reader or second production path is introduced.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| C01 | implemented; database-backed acceptance pending | 56c | Generated canonical meaning and all current typed consumers replace independent spellings/branches |
| C02 | implemented; pure acceptance passed, owner-effect acceptance pending | 56c | Exact replay and effect knowledge consistent across shared-contract owners with distinct lifetimes preserved |
| C03 | implemented; v9 source cutover compiled, composed acceptance pending | 56c | Selected locator plus exact completion/replay/context facts consumed by every durable read/write/lifecycle path |
| C04 | implemented; native/offline controls passed, mixed recovery acceptance pending | 56c | Online/offline file observations share Rust authority; Python duplicate policy retired |
| C05 | in-progress; interrupted fresh activation resumed to the exact closed v9 target | 56c | Working producers/readers installed together; fresh owned store activation and failure recovery verified |

C01 and B's early fact/constructor declarations enable C03; they do not require all B packages to
finish. D01 replaces append realization using C03. C04 and E02 share one product-helper boundary;
the pure file authority can precede relocating its command. C05 is the coordinated activation,
not permission to postpone necessary consumer migration until qualification.

**56a prerequisite — 2026-10-25:** result inspection locates original physical rows and the
designated context receipt through existing retained source revisions, validates their admitted
closure and context identity, binds the cursor to that closure, and retains source protections
through response construction. Missing or invalid closure yields explicit physical unavailability
while raw diagnostic values remain readable. [56a](56a-model-meaning-and-physical-interpretation.md#verification-and-outcome--2026-10-25)
owns acceptance. No storage schema, payload placement, replay contract or existing deployment
interpretation changes in this slice; the remaining C03/C05 target and RC01 stay pending.

## Acceptance

**Planned:** generated declaration-to-code agreement plus independently stated semantic examples;
wrong key kinds/associations and unknown outcomes refuse before effects. Lost acknowledgements,
different immutable replay requests, failed readbacks, stale fences and guarded absence retain
exact effect knowledge. Test each materially different owner shape, not just one helper round trip.

For the selected payload route, test zero/empty relations, below/equal/above inline threshold,
message-envelope pressure, multiple schemas, exact IEEE/null values, partially published results,
read tickets through retirement, mixed inline/sealed backup/restore and exported/reopened results.
Assert actual file/transaction shapes where the claimed work removal depends on them. Fresh
activation controls cover interrupted install, unknown borrowers, obsolete interpretation refusal
and explicit successful readmission. No current product or performance qualification is claimed.

## Selected 56b adoption — 2026-10-25

**Implemented/Tested:** [56b Outcome](56b-scientific-operations-and-permission.md#verification-and-outcome--2026-10-25) owns strict Completion v2/stored completion v3, tagged replay v5,
profile relations and fit-profile v2 transport through current producers, protected readers,
hydration and replay. Missing/obsolete facts refuse; bytes do not create scientific permission.
Four newly owned functional deployments were initialized with the working current stack and
qualified at the Outcome's exact composite scopes, then drained/stopped with databases retained.
The selected preexisting store was preserved. This supplies the necessary C03/C05 adoption for B;
full C01/C02/C04, payload placement/RC01 and the broader coordinated activation remain pending.
There is no compatibility path or authorization to reset other deployments.

## Execution checkpoint — 2026-10-25

The maintainer authorized C01–C05 after selecting RC01 inline/sealed payload placement and only
the RC04 product recovery helper slice. ADR-0228 records the coordinated v9 contract before
implementation and its focused decision review. Preserve the existing selected store, completed
56b fixtures and their original evidence. The existing 56c package table remains the status owner;
working shared contracts, all current consumers and fresh owned activation precede acceptance.
No production package is complete from the earlier 56a/b prerequisite adoption alone.

**Execution checkpoint — 2026-10-26:** the focused decision review accepted ADR-0228's proposed
target before dependent publication. The source cutover, current consumers and generated outputs
are integrated. Shared settlement retains both causes through owner protocols and public diagnostic
projection. Explicit associations and producer classes replace behavioral prefix inference. Inline
bodies are selected before file work; descriptor discovery remains separate from charged body
hydration. The product recovery helper shares native file authority with online readers, and the
duplicate Python file policy is removed. Workspace compilation and focused pure/offline controls
have passed; they do not establish assembled acceptance.

The owned fixture `build/qualification/56c-v8-PMtzWmL1` first retained an actual v8 failed-check
result, then underwent authorized fresh regeneration. A real interruption after publishing the
pending target refused startup; resume reused the exact v9 target and removed the old database.
Admission remains closed. Initialization exposed a lifecycle gap between the omitted telemetry
selection and readmission's quiesced-only guard. Repairing that transition, completing current
receiver admission and running the composed database/public/recovery journeys remain required.
No C package is complete yet. Other stores and prior 56b artifacts remain preserved.
