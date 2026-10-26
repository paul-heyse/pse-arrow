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
Initialization exposed a lifecycle gap between the omitted telemetry selection and readmission's
quiesced-only guard. The repaired explicit readmission preserved the exact closed target and its
initializer; initialization and normal startup then succeeded with current receiver/helper
artifacts. Fresh regeneration now preserves recorded telemetry selection.

The assembled database selection exposed a missing study-scope association projection,
stale fixture/knowledge expectations, an undeclared research-test effect and a fixture bridge
that omitted registration's original deadline. Those owners are corrected, and the canonical
selection now passes. Study input-admission claims carry their actual owner proof and immutable
request purpose; ordinary execution cannot replay that receipt as its own claim. Completed IPC
encoding releases writer scratch while retaining exact allocated bytes and scientific facts
through acknowledgement. The affected runtime controls pass after that custody correction.

Installed Python acceptance passed 108 selected cases but repeatedly crashed in durable progress
reopening. Symbolized addresses identify a large nested future's stack probe, before event
decoding. Progress now uses the adjacent reader's existing prepaid, separately constructed future
mechanism, preserving original activity/cancellation and protection. The original Python case and
four focused future controls pass. The interrupted invocation and its failed workers remain
evidence; all 43 invocation contexts are authenticated-drained, with failure pins retained. A
later public inspection stack probe exposed the same embedding problem in shared durable chunk
reading. Its common completion/replay/seed reader now constructs a prepaid child before the
response scope. Installed Python, actual STDIO MCP and derived CLI agree on original failed
permission and readable diagnostic bytes. The diagnostic-environment currency refusal is retained;
the passing public run used the recorded environment. Affected scientific/physical readers and
mixed recovery journeys remain under qualification. No C package is complete yet. Other stores
and prior 56b artifacts remain preserved.

## Verification and Outcome — 2026-10-26

### What was built

**Implemented:** C01–C05's source target and current consumers are integrated. Final affected
runtime and mixed recovery acceptance is still in progress; the package table remains the
completion owner until those journeys finish.

Canonical declarations now own opaque key kinds, closed vocabulary/transition/projection meaning,
explicit run purpose and study/admission/variant associations, producer class and backup/retention
classification. Generated Rust/SurrealQL/Python consumers replace independent generator name
branches, prefix inference and the handwritten backup inventory. Canonical lookup keys remain
distinct from scientific semantic identities. Actual study owner proof and immutable claim purpose
prevent generic execution from borrowing an admission receipt.

**Implemented:** shared exact settlement distinguishes matching acknowledgement, changed immutable
request, owner-proven refusal/absence and unresolved effect retaining both original and readback
causes. Project, research, execution, context, artifact/analysis, import and maintenance/file owners
consume the common knowledge operation while retaining their different fences, atomicity, creation
windows and original clocks. Public diagnostics preserve attributed causes without exposing raw
requests or converting uncertainty into permission to retry effects.

**Implemented:** v9 has one admitted encoded-body owner with exclusive inline/sealed locators.
Bodies through 64 KiB inclusive avoid file reservation/sealing; homogeneous members share their
complete encoding once. Larger bodies keep protected native IPC. Descriptor discovery and charged
body hydration remain separate; canonical references/read tickets protect inline lifetime.
Descriptors, exact bytes and acknowledgement publish atomically. Empty demanded relations retain
explicit inventory; an empty encoded body refuses. Writer scratch releases after completed IPC
encoding while actual byte allocation and scientific fact grants survive through settlement.
Connected readers, progress and shared durable completion/replay/seed reads use prepaid child
futures with separate construction and polling boundaries on the original task.

**Implemented:** the product `pse-canonical-recovery` target lives in existing `pse-operations`;
fixture controls remain in the separate xtask target. Native authority owns no-follow private
file identity, alias/extent/digest/owner observations and held-lock copy/publication. Python keeps
service/manager/cgroup freeze, drain and maintenance authority. The duplicate Python file policy
and file-only locator path are removed. The controlled fresh lifecycle preserves telemetry,
allocation, exact pending target and closed initialization through explicit current-artifact
readmission. Completion v2/stored completion v3/replay v5/fit-profile v2 keep their scientific
meaning and versions.

### Verification and accepted evidence

**Tested — local Linux, baseline zero:** scopes overlap and are not an additive total or one
uninterrupted clean campaign. Rust correctness routes enable Arrow force-validation. Native
execution uses the selected functional profile without reducing host lanes or allocations.
The owned state is `build/qualification/56c-v8-PMtzWmL1`, with sibling `-receipts`; retained
`qualification.sh`, `final-functional.sh` and selection filters identify expanded commands.
Native test invocation records live under the selected test-resource registry.

| Command / scope | Actual result and conditions |
|---|---|
| Targeted schema declaration/enum controls; `just unit-package pse-operations` selected pure payload/settlement/offline controls | 4 schema controls and 28 operations controls passed in their distinct selections. Exact semantic examples, strict declarations and both-cause settlement have independent assertions. |
| Runtime diagnostic and `just unit-package pse-runtime 'test(reader_future_) \| test(progress_forwarding_futures_)' --no-capture` | Diagnostic control passed; four future admission/readiness/cancellation/layout controls passed, including the affected rerun after the maintainer's `cargo clean`. Relative actual future layouts establish state separation without inventing stack caps. |
| `just unit-native-capability-package pse-relations '' 'solver,klu,isolation,uno,petsc' 'test(admitted_file_releases_encoder_scratch_but_keeps_bytes_and_facts_in_custody)'` | 1 passed under actual tight fixture custody: writer scratch becomes reusable, cloned bytes and independent scientific facts remain owned, final charge returns to zero. |
| `qualification.sh operations` | Final 49/49 canonical controls passed at default 16 lanes; invocation `c6311c3a-629b-452c-b914-9a7049ff0ec1`. Initial 39/49 and 46/49 runs failed and drove owner/projection/clock repairs; they remain failed evidence. |
| `final-functional.sh runtime` | Final 83/83 passed under the local native profile, invocation `8cff0f523cf34bbeb8c59203bba04deb`, nextest `4b0cdb8a-5bcc-461a-8f50-95988f997ea2` (210.746s; 1560 nonselected). Initial 81/83 and affected 9/9 outcomes remain distinct preceding evidence. |
| `qualification.sh python`; `final-functional.sh python` | Original 109-case selection passed 108 cases and repeatedly crashed on one progress case (8 failed worker attempts), then was interrupted; malformed duplicate terminal accounting is retained. The original case passed after the progress repair. After the common chunk repair, 2/2 passed: the original progress/artifact/export journey and stored-result/physical inspection (`f03ec436137a40bf8cc3d3ab657ed9ba`). No clean full 109-case invocation is inferred. |
| Installed `scripts.python_tests --unit-only ... test_generated_contracts.py -k canonical_batch_preserves_original_float_bits_and_null_validity` | 2/2 passed for inline and sealed locators with exact IEEE/null/schema values. A preceding foreign pytest temporary-directory refusal was preserved; that directory was not altered. |
| Focused supervisor/recovery/registration controls | Closed artifact readmission/telemetry: 15 passed; original registration deadline: 4 passed; current native verifier backup/restore callers: 26 passed. Earlier broader supervisor selection had 280 passes and one timing failure; the affected unchanged control passed separately, without promoting it to a clean 281-case run. |
| `qualification.sh public public-accepted` | Actual installed Python, STDIO MCP and derived CLI passed the same inspection document: original permission false, retained check value false. Direct and debugger-controlled earlier probes crashed in shared completion reads. A later diagnostic-environment mismatch correctly refused currency; the passing run used the recorded environment. This does not establish desktop-host attachment. |
| Actual mixed recovery driver | The first attempt preserved an exact capture but refused backup before copying: its 45 managed contexts included a stale helper selection after `cargo clean`, while restore supports a private sole-database state. Current backup now refuses that shared state before freeze: actual 45-context control preserved every config byte and created no destination; 5 related source controls passed. Mixed recovery is running on dedicated private `56c-private-recovery-n7ge09gf`, with the identical functional allocation and fresh actual inline/sealed results. |
| Workspace native all-target compile; supported native Clippy; `just ready` | Earlier workspace compile: 0 errors/warnings. Scope-end supported native Clippy passed; after progress/chunk repairs, `scripts.arrow_validation clippy -p pse-runtime --all-targets --features native-solvers,canonical-tests --locked -- -D warnings` passed. `just ready` passed again after the maintainer's clean. Two concurrent checks interrupted by that clean failed on removed temporary build directories; affected checks were rerun. |
| `just hygiene`; `just governance`; manual native lints | Initial hygiene had 5 failed leaves; Python lint/types, capability reference and both default/no-default Clippy leaves were repaired and passed. Other hygiene leaves passed. Governance had 1 failed capability-reference leaf, later repaired/passed; other 6 leaves passed. `just lint-native-contracts` and `just lint-native-data` passed. These are composed results, not clean aggregate reruns. |
| `just docs`; final `just turn-end` | Pending closeout publication/format checks. |

The actual v8 failed-check receipt preceded disposable fresh activation. Two real SIGKILL
interruptions after publishing the pending target retained closed admission; ordinary startup
refused, resume reused the exact v9 target, and the old database was absent. Current-artifact
readmission preserved telemetry and the original allocation/initializer; closed initialization
and explicit startup succeeded. No selected global store or prior 56b state was reset.

The crashed Python invocation's 43 contexts are drained with all failure pins retained. Native
observations recovered 16 dead payload owners there, and 2 more from the public probe crashes;
unknown/live recovery counts were zero. Cleanup used existing authenticated receiver/file-owner
routes, with no deletion, abandonment or fabricated drain. Final fixture teardown remains pending
the mixed recovery journey.

### A mistake made and corrected

Typed purpose initially rejected legitimate study input-admission through the shared execution
claim. The correction binds actual study authority and immutable purpose at that owner, with
negative generic-replay controls. It does not weaken ordinary execution admission. The first fresh
target also lost telemetry selection, and readmission allowed only the quiesced state; preserving
telemetry and explicit closed-target readmission repaired the same pending target.

Inline admission initially retained the large completed encoder scratch grant, causing a real
append-queue refusal; actual allocation custody now shrinks without release/reacquire. Enlarged
durable futures also exhausted Python's existing stack. Actual fault instructions identified the
progress and common chunk-read stack probes; prepaid child construction repairs those shared
owners without increasing stacks, spawning work, extending clocks or changing scientific checks.
The diagnostic environment currency refusal was corrected by using the declared environment.

The first operational attempt used the shared test deployment despite restore’s existing private
service boundary. Backup now checks that same boundary before freezing or copying. The original
45-context deployment stays stopped with its data and failure evidence; a dedicated private
fixture qualifies the supported operator lifecycle without widening replacement authority.

### Deviations from the plan, deliberate

RC01's preferred inline/sealed route was selected. RC04 adopted only the product recovery helper
in the existing package; broader worker/project root relocation remains with 56e. No new crate,
archive/log, compatibility reader, old-data backfill or numerical algorithm was introduced.
ADR-0228 remains proposed under the separate formal decision-PR rule.

**Not run / not claimed:** the 505-case feature powerset, new performance campaign, full IDAES
parity/solver fleet, wheels/distribution and other platforms. Structural work removal and exact
functional evidence are not measured speed/capacity claims. D's bulk/population/incremental work,
E's wider roots/catalogues/profile authority, F's broader host/observability work and full-series
G acceptance retain their owners. Other plan series are unchanged.
