---
id: ADR-0182
title: Own scientific artifacts and release detailed history under exact authority
status: proposed
date: 2026-10-12
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-06, AP-07, DP-09, DP-19, PS-10]
blueprint: [§20.3, §20.4, §20.5, §20.6, §21]
review: docs/design_review/reviews/design_review_plan37-remaining-target_2026-10-12.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported consumer needs shared large blocks or independently verifiable owner binding that eliminates permanent operation locators.
verification: Plan37 A1–A3 both-order admission/retirement and late replay; scientific selections after producer collection; persistent C2 exact settlement/activation and process death; C3 interrupted clean reset; Q1 rebuilt Rust/native/Python consumers.
standard: {core: '3.4', process-simulator: '1.5'}
---

# ADR-0182: Own scientific artifacts and release detailed history under exact authority

## Context

Plan37 RC01–RC03 select durable exact study inputs, independent scientific artifacts and
releasable detailed history. In-process seed admission and whole-producer protections do
not provide these lifetimes. RC09 keeps beta tooling under canonical/operational owners.
OP01 replaces historical substrate preservation with fresh target state and regenerated evidence.

## Scope

Amend blueprint §20.3–§20.6 and §21; add the artifact relation/operation family, reduced
replay and explicit maintenance activation. Use fresh `pse.substrate.v5` target stores.
Preserve original scientific identities, opaque operation IDs/hash meaning and public handle
identity. This changes preservation/cutover provisions of proposed ADR-0180 and ADR-0172,
not supported scientific source revisions or current-version backup/restore.

## Drivers

An admitted external start must survive study restart after producer retirement. Selected
final states, trajectories and analyses must survive independently of unrelated payload.
Released history must not make old identities appear unused. Scientific owners define
sufficient meaning; storage owns atomic visibility/protection/bounded collection; the
maintenance owner settles actual effects before canonical activation.

## Options

Select small copied seed capsules and artifact-owned bounded IPC copies. Whole-run pins
retain unrelated output; shared producer blocks add a lifecycle the initial consumers do
not need. Explicit attributed obligations and conflict guards preserve scientific policy
that native references cannot express. Compact operation ownership avoids changing opaque
IDs or retaining every request/result. Adapt the existing rebuild owner, not a second reset
manager. Beta progress/repair is tooling, not application fencing or effect settlement.

## Outcome

### Independent scientific artifacts

Artifact occurrence identity is separate from scientific content. A sealed creation intent
binds existing primary source authority incarnation, fresh nonce, original fixed finite
expiry and exact request. Same-operation retries preserve it. Existing primary retention
guards and monotone creation floors fence late creation at final collection, following the
analysis occurrence contract rather than a renewable generic lease service.

Stage under exact producer protection outside the short visibility transaction. Atomic
activation registers producer retirement/source guards, verifies staged closure and installs
independent source roots and initial obligation. Unknown creation retains its exact intent,
staging and protection until settled. No transaction spans numerical work.

Scientific owners preserve original IDs/bits, units/basis, ordering, source/model/case,
assessment, eligibility and partiality. Seed capsules carry original qualified seed document
and assessment without retaining the full completion. Final states use actual endpoint
records with mode/integral/input interpretation, not the last requested sample. Selected
trajectories/local analyses carry transitions, validity/conventions and required upstream
context. Original and copied encoding digests remain distinct. Persisted graph analyses keep
their current occurrence owner, replacing active producer IPC strings with typed artifact
block/row/field evidence locators. Missing sufficient context refuses promotion; no rerun.

Study, explicit-user/project, analysis, protected-read and audit obligations release
independently. Release cannot invalidate another owner's future protected acquisition.
Retirement excludes new owners only after retained obligations permit it; actual grants
still delay collection. Collection is bounded/resumable and retains independent source
roots. Copied arrays retain allocation ownership, not future DB page permission.

Durable studies admit external selections before activation, deduplicating exact inputs
within a study while preserving occurrence role/provenance. Dispatch/retry/restart uses the
artifact. Admission-first survives producer retirement; retirement-first refuses without
fresh-start fallback. No additional late-binding public mode is selected. Remove coarse pins
and replaced producer-dependent acquisitions when callers and controls move.

### Detailed history and reduced replay

Payload release and history release are distinct explicit operations. Default history lasts
until owner release. Terminality/age does not authorize deletion. Audit obligations, unknown
effects and creation/grant windows retain required detail. Permanent run/study fences precede
all mutation, early receipt return, readback/absence, readiness and recovery decisions. A
retired study identity refuses even with a different live run.

Compact existing operation rows under unchanged IDs, retaining issuing authority, run/attempt,
applicable study/point, kind and truthful minimal disposition. Install binding atomically
with effect/receipt. Live/unsettled work retains exact immutable comparisons. Settled released
owners discard full request/result and return retired/history-unavailable, even for changed
request bytes. Validate stored ownership and fences transactionally before interpreting
absence; caller assertions are not authority. Operation-free paths check permanent owner
fences directly. Compact locators cannot reconstruct successful closed-attempt responses or
authorize redispatch. Bounded collection removes released occurrence, policy/facts, start,
attempt, dependency, readiness/cursor, request/result and header detail after obligations
settle. Compact identity state can still grow with issued operations; no constant-size or
physical disk-recovery claim follows.

### Maintenance and clean cutover

Publish immutable frozen rollout files/manifests through their existing producer; changed
declarations append steps. Registry DDL/codecs remain the single source. Beta SHA differs
from repository interpretation/digest. Exact maintenance authority closes admission and drains
holders/workers/drivers, then owns its listener and tool child. Continuation binds pending
nonce, input and rollout/step identities. A fixed generated-schema settler checks original
context, stored acknowledgements and actual generated-definition postconditions. Reconciliation
without acknowledgement is admitted only for known schema-only steps whose exact effects
are established after authentic SQL/service drain; ambiguous steps remain unknown. Data
steps require their domain owner's atomic effect witness/postconditions. CLI0, repair,
lock expiry or process/socket drain alone never settles SQL.

Retain atomic direct fresh creation. Add maintenance-only validation and activation of exact
preinstalled generated definitions. Verify catalog/codec/scientific obligations and settled
operation before marker publication; fresh activation mints issuing identity, compatible
forward activation preserves it and checks the exact predecessor. Ordinary open never
repairs/activates. Owner loss invalidates initializer/activation authority and stays closed.

Adapt rebuild/recovery for regeneration from current authoritative definitions/fixtures,
without preserved-input export, historical migration/reader or old-data survival gate.
Drain authentic owners before selected replacement; mint fresh DB identity/credentials;
initialize/validate under closed admission; dispose only exact replaced owned state; then
explicitly activate/readmit. Unknown owners/borrowers/effects refuse. Source documents and
unrelated workspaces remain outside reset. Current-version backup/restore retains its own
input inventory and authority.

### Consequences

Move generated contracts, scientific/runtime/Rust/Python consumers, retention and operations
together. Delete replaced substrate historical activation/read/backfill paths after targeted
replacement controls pass. Preserve supported scientific history/version semantics. Plan37
owns status; older plans are reconciled only after Q1 acceptance.

### Compensating controls

Both-order conflicts, insufficient closure, late creators, opaque-ID replay, partial collection,
owner/process loss, competing writers and installed-consumer journeys distinguish the target.
Fresh authenticated fixtures preserve unknown effects. Full model, performance and physical
space qualification remain separate claims.

### Confirmation

The maintainer confirmed RC01–RC09, OP01 and the remaining execution approach, then requested
implementation on 2026-10-12. The independent bounded target review accepted this design
at Proposed evidence level before dependent code.
This proposed record does not claim implementation or qualification acceptance.

## Pros and cons

Copied selected closure removes producer coupling. Compact locators allow safe detail
release without a new public ID protocol, at the cost of retained identity state.
Existing maintenance authority avoids duplicate operational ownership.

## More information

[Plan37](../plans/37-surrealdb-lifecycle-and-integration.md) owns status and acceptance.
[Scientific closure](../plans/37-investigation-evidence/scientific-closure-and-replay.md)
records sufficient selections and affected entrypoints. The source review remains unchanged.

## Status history

- 2026-10-12: Proposed before dependent remaining Plan37 implementation under confirmed decisions.
