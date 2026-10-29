---
title: "Plan 58d: Canonical execution, schema evolution and connectivity"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md, docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [storage, design]
---

# Plan 58d: Canonical execution, schema evolution and connectivity

## Purpose and basis

**Proposed:** implement O07 and the control portion of O09 from the
[principal functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md#O07).
A Rust operator can browse durable work, submit a complete authorized control intent and
reconcile its original outcome using connection, authentication and bounds alone. Reading
headers or requesting stop must not construct mathematical services or prepare a provider.
The [coordinator](58-rust-owned-system.md) owns shared rule decisions, sequencing and overall
acceptance; this document owns D01–D05, including the durable-model and migration foundation selected from improvement-review I02/I03. Creating this draft does not authorize implementation.

The original amendment used `c3b7f136d` and the reviewed `bd1c4891e` production basis on
2026-10-28. The 2026-10-29 amendment consumes the
[database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md)
at `a6a144bce`, especially its O01–O08/O10; these source-qualified references are distinct
from the principal review's O01–O15. The [coordinator](58-rust-owned-system.md) owns current
dispositions. The [improvement review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I02)
continues to supply I02/I03. Subsequent native host/local-operation delivery retains its
original checkpoint evidence and does not establish canonical progression acceptance.
Relevant current contracts are blueprint §20.6 and §23; the selected assessment standard is
Core 3.4 with its efficient-architecture companion and Process Simulator 1.5. Existing
scientific admission, stop intent and physical release remain separate responsibilities.

## Existing foundations and target

`crates/pse-operations/src/canonical_control.rs` already declares bounded browse/cancel/
reconcile documents and `CanonicalStore` operations. Its native cancel path invokes generated
`fn::pse_control_v1` policy as `native-system`; this is privileged worker behavior, not the
record-actor client contract. `deployment.rs::CanonicalConnectionSettings` and
`ServiceIdentity` already separate connection-owned configuration from native execution.
`canonical_client.rs::Client` owns the bounded loopback custom gRPC connection and request
settlement. The Rust SDK is an alternative to assess, not an existing interchangeable owner.

Retain the supervised source-built SurrealDB 3.3.4 with MX 1.0.0, explicit SSI and synchronous
AOL/every-append durability. `scripts/surreal_source.py`, `mx-1.0-controls.patch`,
`canonical-profile.patch` and `api-failure-rollback.patch` in `scripts/surrealdb-source/`
identify the effective server. Stock release defaults and engine-overview labels do not.
Compact client control and privileged scientific workers share canonical command policy;
they do not share administrative credentials or imply equal authority.

The target adds a narrow typed native entry point over this foundation. Direct callers pass
Rust domain requests and receive typed outcomes; external serialization belongs only at the
selected ingress. Do not carry Python JSON/document round trips into in-process calls.
Keep existing constrained `DEFINE API` delivery for actor-facing commands where it supplies
the complete contract, and existing bounded gRPC for workers where useful. Compare those
with exact-release SDK use before adding a protocol. No new Tonic/Prost service, Arrow Flight
control plane, registry, crate or universal connection abstraction is selected by this plan.

## State placement and shared contract

This is the detailed owner of the database/native boundary; the coordinator summarizes the
combined target, A owns scientific callers, C physical execution, E payloads and F/G retained
operating consumers and retirement. One fact can have a canonical authority and a different
immutable execution representation without becoming two independently writable authorities.

| Meaning or operation | Canonical responsibility | Native or adjacent responsibility |
| --- | --- | --- |
| Shared model/context revisions, study definitions, dependencies and realized points | Store explicit domain identity, admissible relationships and useful history | A admits physical/scientific meaning and prepares revision-bound models; registry declarations remain authoritative |
| Eligibility, assignment, cancellation intent and acceptance | Complete short transactions over the relevant generation/revisions/fences | C acquires actual resources, delivers stop and witnesses destruction/join |
| Candidate discovery and progression | Bounded applicable action, exact preparation premises and continuation | Native caller performs scientific preparation and returns evidence; no local durable scheduler truth |
| Accepted output lineage and selection | Canonical attempt/result references, grants and manifest identity | E retains the existing bounded inline IPC/sealed-file hybrid, selective Arrow/DataFusion reads and buffer custody |
| Shared temporary worksets | Same canonical records where queryability/atomicity removes real coordination | Single-owner solver/workspace state stays native; O09's direct working-store adoption is deferred under F05 |
| Process, task, socket, token and provider state | Logical intent/status only | B/C own actual handles, admission, callback lifetime and physical release |

Retain one source-built SurrealDB3.3.4/MX1 authority with the effective patch tuple, SSI,
synchronous AOL and every-append persistence. MX is a persistent canonical database here,
not a disposable cache. There is no automatic MX/KV per-table routing or dual writable
canonical copy. Engine recovery snapshots/MVCC are not domain revision history or D05 schema
migration. No performance ladder, universal duration crossover or guaranteed lane/group-commit
benefit is an acceptance premise.

The complete shared interaction is: bounded canonical selection/progression; native input,
seed and resource preparation outside a transaction; a short current-premise claim recheck;
native execution without an open database transaction; and fenced canonical acceptance after
scientific assessment and required payload construction/drain. A preparation candidate is not
execution authority. Lease expiry/reassignment does not prove the former worker stopped.
Local pure science remains database-free. This target selects no new rule reduction; broader
engine, transport, module privilege or payload changes require their applicable decision route.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [D01](#d01) | in progress | Rust operations and connection owners | Control construction and direct typed calls require no mathematical Runtime. |
| [D02](#d02) | planned | Canonical command policy and ingress owners | Actor-bound commands and original receipt reconciliation agree across selected ingress. |
| [D03](#d03) | planned | Canonical study policy and operations owners | Durable model governs bounded progression, preparation handoff, claim and later lineage queries. |
| [D04](#d04) | planned | Control consumer and integration owners | Retained native consumers migrate; replaced frontend orchestration is deleted; H has assembled cases. |
| [D05](#d05) | in progress | Canonical schema and maintenance owners | Native ordered migration, version applicability and write fencing work, including open-handle/in-flight semantics. |

<a id="d01"></a>
## D01 — Connection-only native control

Consume [T01's decision route](58-rust-owned-system.md#t01) and
[T02's operation inventory/contracts](58-rust-owned-system.md#t02) before fixing public
shapes. Extend operations/deployment owners so an authenticated native caller
can select its canonical issuer, bounds and declared browse/cancel grants without creating
`workflow::Runtime`, `SourceServices`, compiler preparation, math or solver services.
Before fixing the connection realization, complete O04's exact-release comparison between
native SDK, existing bounded gRPC and constrained actor APIs. Record which responsibilities
are delegated, which disappear and which genuine gaps remain: typed values/integers, request
identity, response extents, original deadlines, session admission and physical completion.
The [tagged SDK query implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/src/method/query.rs)
exposes `stream_items` with provisional rows/terminal statement items, but no public physical-drain
completion handle on the returned stream; output-boundary stop is not proof of a stopped
non-emitting execution phase. Do not use arriving rows as irrevocable execution authority.
The comparison can conclude that selected custom custody remains necessary; SDK replacement,
another protocol and a universal connection abstraction are not completion requirements.

Reuse connection admission, deadlines and close/drain ownership. Provisioning actors and
installing schema remain explicitly privileged operations, outside ordinary client startup.

For example, an operator opens a selected deployment and requests one compact page of study
headers. The result carries exact cursor predicates/issuer and live-browse meaning; it does
not hydrate payloads or claim a frozen analysis population. A wrong issuer, disabled actor,
missing grant or invalid bound refuses before protected state access. Configuration failure
does not trigger scientific initialization as a fallback. Narrow existing constructors and
move callers together; remove any replaced control-only Runtime construction path.

Targeted controls cover browse with mathematical initialization unavailable, strict selected
configuration, wrong-store continuation, close during an admitted request and independent
caller admission. Compile the affected operations/runtime packages through their scoped
routes; correctness closures retain explicit force-validation where Arrow is consumed.

<a id="d02"></a>
## D02 — Complete commands and truthful reconciliation

Build on D01's usable connection/actor contract. Keep one generated canonical policy owner in
`crates/pse-codegen/src/codegen/surreal_control.rs`, consumed by native operations and the
selected constrained API. An immutable command binds the complete intent to issuing store,
authenticated actor and exact issued operation identity. Identical repetition observes its
original receipt; changed intent under that identity refuses. Restore changes issuer.

The revealing failure is a cancel request whose server effect commits but acknowledgement
is lost. Retain the issued request and reconcile that tuple; receipt absence is unobserved,
not rollback or permission to issue a fresh effect. Preserve original uncertainty and readback
failure rather than returning a guessed success/refusal. A receipt records stop intent,
while the worker/native/host owner establishes actual joining and release. Long scientific
preparation remains outside short revision/fence-checked transactions.

Ingress keeps grant checks before parsing/state/receipt access, raw-byte admission before
strict UTF-8/JSON decoding, opaque string identity and exact integers. API-body actor checks
remain necessary even with endpoint `PERMISSIONS`. Preserve the patched rollback behavior
for evaluation/conversion errors; an intentionally returned refusal response has its stated
transaction meaning. Remove independently maintained frontend retry/transition decisions.
If a selected derived-state/event operation participates in this policy, preserve real actor
and grant restrictions explicitly. Event execution privileges must be checked on the effective
server rather than assumed identical to ordinary caller mutation checks. Constrained API
handlers must retain the complete issued-intent/receipt contract regardless of transport.
Targeted controls exercise identical/reused intent, actor denial, full-u64 boundaries, stale
fence, lost ACK, missing receipt, failed readback, restore issuer and API rollback/refusal.

<a id="d03"></a>
## D03 — Make canonical state govern durable execution

Definitions/revisions, requested work, dependencies, realized points, assignments, attempts,
state transitions and result references form one queryable execution model. A05 owns native
scientific recording/submission callers and point interpretation; this package owns canonical
submission and transition policy. C owns actual preparation/resource admission and native joins;
B owns database process mechanics; E owns protected payloads. Pure local computation stays direct.

Extend the existing generator `pse-codegen/src/codegen/surreal_studies.rs`, constrained
`pse-study-module` and shared `pse-model::study_policy`. Existing `readiness_scope_page`,
readiness apply, admission, claim, settlement and conclusion are foundations to compose, not
proof the client loops are necessary. `workflow/worker.rs::work_once_scoped` currently pages
studies, advances readiness and fetches candidates; `work_study_candidate` fetches scope,
evaluates policy, prepares and reevaluates before claim. Replace repeated client interpretation
with complete bounded store-side progression that returns the applicable domain action and
exact durable premises. Include readiness advancement, candidate selection, applicable settlement
and conclusion in the shared operation where their invariant fits; do not retain client
choreography merely because separate functions exist today. A returned candidate carries its
issuing store, generation, consumed point/dependency revisions, cancellation and seed premises.
Continuation distinguishes an examined page from exhaustive no work. Reuse domain types and
`pse-model::study_policy`; no universal next-action catalogue or new workflow engine.

Co-design the consumed access paths with that contract. The current registry generator already
emits selected `INLINE EDGES 64 INLINE REFERENCES 32` caches and typed
`REFERENCE ON DELETE REJECT READONLY` links. Evaluate actual dependency loading, candidate
ordering, indexed counts and high-degree/skewed populations before selecting more graph
machinery. Study dependency edges carry metadata and stable identities: `LIGHTWEIGHT` is not
a drop-in replacement. Compare explicitly maintained transaction summaries with supported
`AS SELECT` aggregates or small synchronous events; retain one generated schema owner. The
[tagged table implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/legacy/expr/statements/define/table.rs)
and [3.3 release mechanisms](https://surrealdb.com/releases/3.3) supply feature-level evidence,
not proof of an efficient application access path.

Delete outer paging/readiness/finalization sequencing and repeated authoritative action
interpretation only as the complete canonical operation replaces their consumers. Native
bounded continuation and the effect loop remain necessary. `CandidateGroup` suppresses local
duplicate preparation; it is not a competing claim ledger. Retain it unless the selected
preparation/access path removes that purpose or deliberately replaces that contention handling.

Prefer ordinary SurrealQL select/recheck/transition composition; retain the readiness module
where shared bounded Rust policy earns its boundary. Preparation and resource admission precede
execution claim. Expensive preparation/solver work stays outside transactions; claim rechecks
prepared seed evidence, dependency revisions, generation and cancellation in the current canonical
snapshot. Invalidated preparation cannot grant a stale claim. A bounded examined page with
continuation is distinct from exhaustive no work; workers advance continuation rather than
mistaking an unproductive page for global idle. Original operation identity reconciles uncertain
acknowledgement. Notifications only wake workers; canonical catch-up handles loss and replacement.
Use live notifications for freshness and retained changefeeds only for a consumer requiring
historical changes. Optional subscriptions cannot become indispensable execution truth.
Synchronous events may maintain small related state in the originating transaction; asynchronous
events are delayed, repeatable database maintenance, not scientific dispatch or exactly-once
claims. Their [documented contract](https://surrealdb.com/docs/reference/query-language/statements/define/event)
includes separate execution transactions and at-least-once delivery. Check actual actor/grant
privileges, ordering, reentrancy and retry/latency requirements before adopting an event.
View/event maintenance must cover the selected write, import/reconstruction and migration paths;
ordinary event invocation is not assumed for every ingestion route.

Preserve existing dependency-installation limits: add edges before activation/cancellation,
not live graph mutation. Record realized points and scientifically useful observations at useful
analysis granularity, with actual attempt/result lineage. Native A05/E02 consumers compare
repeated studies using authored revision, effective context, recorded point, actual attempt,
scientific qualification and protected result selection. Effective time, recording time and
storage version time remain distinct. Explicit domain revisions/frozen cohorts remain necessary
without MX `VERSION` access and would still be needed with KV history. Do not reconstruct a
parallel execution history from task logs, persist native pointers/task handles, or mirror every
implementation event. Later repeated analysis queries this same canonical history.

Keep the existing stateless, read-only `pse-study-module` adapter over shared bounded Rust
policy: SQL selects scope/performs effects, while the module returns complete decisions. Its
manifest forbids arbitrary queries/functions, network and KV. Before extending Surrealism,
check exact 3.3.4 permissions, controller limits, complete outputs, rollback and intentional-refusal
semantics against [host context](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/surrealism/host.rs)
and [WIT ABI](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealism/wit/surrealism.wit).
`sql_with_vars`, `run` and writeability metadata are candidates, not authority already granted.
Broader query/write grants are not selected. Module-local volatile KV and per-controller
memory have separate lifetimes and do not share record-transaction rollback. Neither may
carry assignment, deduplication or publication authority. Any justified cache is reconstructible
and keyed by immutable/versioned inputs. Native science and host release stay outside modules;
removing repeated conversion must preserve exact integers, missing values and bounded output.

Targeted controls cover two competing workers, stale prepared evidence, cancellation before
claim, a bounded empty eligible page with later work/continuation, high-degree dependency access,
dependency edits refused after activation, missed notification catch-up, and replacement worker
selection followed by a lineage query. A delayed former worker must physically drain and fail
publication under a newer fence. If events/views/modules are selected, exercise their real actor,
rollback, import/rebuild and complete-output contract at the same owner. These tests challenge actual
canonical operations, not a second scheduler. Database speedup remains unmeasured.

Embedded deployment, SurrealKV history/checkpoints, buckets, direct MX and broader modules
remain conditional as specified below; none substitutes for D05's required schema evolution.
Use D01's comparison outcome for any selected progressive transport. Keep patched MX selected.

## Decision investigations

These are bounded decisions inside the existing packages, not a second status ledger or
mandatory feature-adoption list. The coordinator owns review opportunity dispositions.
A decision may retain the existing realization; only dependent implementation waits for the
applicable result. A credible selected consumer can justify reopening a conditional capability.

| Question and owner | Evidence / allowed outcome | Constrains |
| --- | --- | --- |
| D01: SDK/API/gRPC responsibility split | Exact-release interface/source comparison; focused issued-request, response/finality and close/drain control if unresolved. Delegate covered plumbing or retain the actual custody gap, with candidate deletions named. | D01 construction and D04 transport migration; not independent local science |
| D03: dependency access and operational summaries | Current generated references/indexes plus representative sparse/skewed query shape; compare explicit updates, supported view or event and its ingestion/migration behavior. Retain current structure if no simplification earns its cost. | Chosen progression access path and summary consumer |
| D03/D02: reactive maintenance and catch-up | Prove real actor/grant, retry, ordering and catch-up behavior for the selected event/feed. Retain canonical polling/catch-up when notification machinery adds no useful contract. | Selected observer/maintenance cut, not execution correctness on notification availability |
| D03: larger Surrealism effect surface | A named shared rule must remove concrete client coordination; establish parent transaction rollback, refusal, mutability metadata, output and permission semantics before widening grants. Otherwise retain pure module. | Only that optional module extension |
| D target/B04: KV or embedded topology | Reopen for supported working set beyond MX envelope, storage history, encryption/keys, backup characteristics or a concrete deployment benefit. Compare actual claim/capacity/dependency invariants, adapter, migration and isolation; KV1 selected read tracking is not predicate SSI. | Only the selected backend/topology cut; current MX delivery remains independent |
| F05 with A/D: direct working collection (O09) | Named multi-key/query workload, lifetime, key/index/schema obligations and publication/reconstruction direction must remove more machinery than added. Same canonical records remain default for shared domain meaning. | Deferred working-store adoption; no independent knowledge/evidence authority |
| E with D: buckets or payload relocation | A concrete artifact consumer must preserve protected readers, complete file construction, uncertain publication and reclamation. Object writes are not presumed atomic with canonical records. | Only that optional payload/artifact change; existing IPC hybrid remains selected |
| D05: migration interpretation and health | Inspect actual write fences/open-handle behavior and existing metrics, then select allowed old-attempt settlement, successor activation and adjacent-version tests. Compare toolkit helpers only against remaining gaps. | New canonical writes and publication crossing that interpretation boundary |

MX group commit and existing storage/query diagnostics are enabling mechanisms, not a measured
application benefit or a reason to build another watcher. Use current engine metrics for pinned
readers, retained versions, GC, snapshots and queues when qualifying the selected workload.
Optional measurement must resolve a concrete choice under equivalent semantics/durability;
no generic latency ladder or full benchmark campaign blocks authoring or ordinary development.

<a id="d05"></a>
## D05 — Deliver minimal native schema evolution and write admission

Consume T01's decision authority and the registry-owned schema declaration. Deliver native
ordered predecessor/successor transformations and their application owner during the cutover;
this is required infrastructure, not a deferred interface. Reuse existing generation and
canonical maintenance facilities before selecting a migration library. SurrealKit frozen rollout,
schema inspection and isolated actor/API testing are comparison leads, not a second schema
authority or mandatory framework. Storage-format conversion, checkpoint/backup and MX recovery
do not provide supported domain predecessor/successor transformation. Do not duplicate schema
meaning or revive the full SurrealKit beta/operator experiment merely to supply migrations.

Construction establishes supported schema/version and issuer premises once and returns the
appropriate typed capability. Dynamic admission/version changes remain real external uncertainty.
Do not repeat complete schema proofs on ordinary operations, and do not treat a once-opened
handle as permanently authorized to write across a migration.

Resolve the open-handle/in-flight contract before dependent implementation: inspect current
`canonical.rs::WriteAdmission`, maintenance and publication paths; establish how new claims
and writes close, how original in-flight attempts settle and physically drain under the old
interpretation, and how stale handles are fenced before successor activation. Favor a quiescent
migration boundary; do not transform schema beneath active scientific writers. A migration
that cannot establish that boundary must remain pending with admission closed. Specify the
permitted drain/publication operations separately from newly refused mutations, version/fence
checks at actual writes, and successor capability reconstruction. Carry this bounded decision
and its targeted race tests through T01 before implementation; no claim of a working fence follows
from current configuration-file checks alone.

Declare the applicable predecessor, close admission, apply ordered transformations, and publish
the successor version only after the transformation succeeds. Interrupted application leaves
explicit pending/closed admission; restart cannot silently reopen incompatible writers or change
original effect identity. A representative adjacent-version disposable dataset demonstrates
correct transformation, wrong-predecessor refusal, interruption and stale-handle/in-flight
behavior. Publication and read compatibility are stated per supported schema, not inferred from
an SDK connection or successful process start.

Owned regenerable development/test deployments may reconstruct from authored inputs/reference
material. Placement is disposable only when established at creation; no reset of existing data
is authorized. Uniquely authored inputs and independent evidence remain protected. Full supported
historical migration, backup/restore and disaster recovery belong to 58h's persistent-deployment
release obligation before non-regenerable/shared operational data, with D's schema owner and B's
service owner supplying that later evidence. Minimal infrastructure above still blocks D completion.

<a id="d04"></a>
## D04 — Migrate control consumers and hand off acceptance

Move retained native study recording/inspection, cancellation, repeated-analysis lineage and
original-command reconciliation callers to D01/D02/D03, with schema applicability from D05,
coordinating Python HTTP client retirement with its owning frontend package. Delete replaced
orchestration, caller-owned retry policy and tests/fixtures for retired machinery in the same
integrated cutover. Include callers, configuration, generators, targeted tests and deletion together; temporary development incompleteness earns no acceptance and needs no compatibility bridge. Preserve tests of retained durable guarantees at their Rust owner. A CLI
is an optional caller; PyO3 remains deferred and MCP is conditional under the coordinator.

Every integrated cut names the replaced worker traversal/action/summary/notification or
connection mechanism, the complete canonical/native replacement and actual migrated caller.
Remove only proven redundant mirrors or orchestration: retain original effect identity, dynamic
write fences, native preparation suppression where still useful, physical custody and protected
IPC/file lifetime. D publishes applicable attempt/result references; E selects and reads their
bounded inline/sealed payloads without mathematical construction. File sealing followed by
failed/lost canonical acknowledgement cannot invent accepted output or trigger a second solve.

Sequence early D01 connection/actor and D03/D05 premise/version decisions before their consumers,
then integrate native preparation/claim and selected observation/data cuts. C03 consumes D03's
early handoff while D03 implementation consumes C's usable preparation/resource capability;
this is not a whole-package completion cycle. D04 acceptance requires the actual relevant D03
and D05 capability, not merely agreed prose. B service launch and schema readiness remain
separate; backend restart cannot silently authorize stale workers to publish.

**Proposed verification:** targeted owner controls above establish local behavior, with zero
failures as the baseline. [H](58h-qualification-and-publication.md#h01) owns assembled
acceptance of authenticated direct/native and
retained published ingress, restart/lost-ACK reconciliation, denied actors, missed notification
catch-up, durable worker replacement/later lineage, D05 migration and stop-versus-physical-drain. H selects integrated tests and scope-end static checks
once functional scope is complete; protocol examples or local controls do not qualify science.
The 2026-10-29 document amendment is **Proposed** design and scheduled work, informed by
**Interface-checked** review/source assessment. No product test, build, service or migration ran
to author it. Existing native host/local-operation passes retain their named scope; no new
canonical/scientific qualification or speedup follows from these amended documents.

For release-specific research use the principal review's library appendix: SurrealDB tooling
schema/API request tests are integration aids, not scientific schema authority. Hyper 1.11.1
and Tonic 0.14.6 are the current pins, versus overview 1.12.0/0.14.8; the newer Tonic explicit
cancellation-handle API is not assumed. SDK/protocol selection must preserve request custody,
deadlines and drain rather than claim those duties from transport shutdown alone.

## Native host dependency execution checkpoint — 2026-10-29

The maintainer's full [58b execution](58b-bootstrap-and-host-control.md#execution-dependencies)
includes working D01/D05 prerequisites for native service composition. D01 and D05 remain
**in progress**. Source delivery, live fixture acceptance and whole-package acceptance remain
distinct. The latest bounded implementation is integrated in `f28e0e373`.

**Implemented:** connection-only schema-ready construction, transaction-bound version/epoch
admission and ordered quiescent migration from the preserved adjacent predecessor. Original
attempt settlement and custody cleanup remain possible while new work is closed. Migration
retains its original request/activation receipt through uncertain acknowledgement. The legacy
predecessor cannot fence historical binaries: actual deployment-owned ingress closure and native
join precede transformation. Private fresh-fixture predecessor installation is native-service
only, checks actual catalog absence and is not an arbitrary existing-store installer.

The optional native-service fixture uses the exact prepared server generation and original
process, manager, socket and module identities. Private 0600 payloads carry credentials;
D-Bus argv/environment do not. A distinct bound ingress service owns an actual canonical
connection and closes/joins it before its original-identity drain receipt. Migration joins
ingress while leaving the database available; final shutdown attempts all independent owners
under the original deadline. Closure is terminal, disposal retains ownership on failure, and
neither Drop nor a timeout supplies successful cleanup. The host crate remains database-free.

Retained test/fixture callers now pass plain query bodies or typed prepared scripts, rather
than complete transactions inside the new framing. Held transactions retain actual schema
bindings; peer result slots match framing. Negative initialization/cancellation/issuer tests
again reach their intended failure conditions. Interpretation/digest classification remains
precise. The private append-submission boundary boxes its Send future once without changing
payload custody, clocks or settlement. No recursion-limit increase or unchecked Send was added.

The real retained server parser requires a quoted password value for credential DDL; the
native RPC boundary uses the library SQL value encoder. Errors retain private forensic detail
while automatic diagnostics redact it and preserve typed finality/uncertainty. The isolated
server also retains its explicit files/surrealism capability and narrow module bucket profile.
Initial live failures exposed these premises and a missing predecessor backend binding; all
were repaired. Earlier failed fresh states/logs remain preserved, without acceptance credit.

**Tested:** `just check-package pse-operations --features native-service` passes without
warnings. The final correctness-routed selection below passes 78 tests, 420 excluded, zero
failures, run `e55eeb41-47ca-4a9b-aefe-8a07bba0bcfa`:

```bash
scripts/pse-env -- sh -c '
  export PSE_NATIVE_FIXTURE_MODULE_STATE="$PSE_SURREAL_STATE"
  exec scripts/cargo-bin/cargo-pse-correctness nextest run --no-fail-fast --locked \
    -p pse-operations --lib --test native_service \
    --features native-service,canonical-tests,canonical-maintenance,test-support \
    -E "test(canonical::migration::tests::) | test(canonical_script::tests::) | test(canonical::transport::tests::) | test(canonical::transport::lifetime_tests::) | test(native_service::tests) | test(study_module_private_copy) | test(schema_migration_drain) | binary(native_service)" \
    --success-output immediate-final
'
```

This includes the earlier 64 schema/script/transport/lifetime controls and 9 native fixture/module/
drain controls, plus all five actual isolated service cases: current-epoch migration and ready
reconstruction; adjacent-predecessor migration only after native join; process stop independent
of database transport; foreign-listener refusal; and cleanup after an intentional body panic.
The panic diagnostic is an asserted failure-path stimulus, not a failed test. Live controls copy
only the configured module artifact into fresh private state. Independent readback finds all
eight original service units not-found/inactive/MainPID 0, all successful disposable states gone
and no finite reservations. Existing configured stores and foreign listeners remain untouched.

Generated rust-contracts, SurrealQL, Python/docs and document-schema outputs match their owners.
Optional-feature all-targets Clippy with the same four features passes with `-D warnings`.
These results establish the working fixture/readiness prerequisite for B04 stage 5, not whole
D01/D05 or H acceptance. Actual server stale-epoch/in-flight commit races, lost-acknowledgement/
restart matrix, existing-store migration, broader SDK/API/gRPC responsibility comparison and
remaining retained consumers are still open. Full durable-production recovery, numerical and
performance qualification remain outside this checkpoint.
