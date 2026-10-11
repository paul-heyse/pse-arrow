---
title: SurrealDB lifecycle and integration remediation
status: done
date: 2026-10-10
adrs: []
review_sources: ["git:313b53ac9dceb770c3254b6a20b3bd7f57e69af7:docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md"]
scenario_sources: ["git:313b53ac9dceb770c3254b6a20b3bd7f57e69af7:docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#4-revealing-scenarios"]
---

# Plan 35: SurrealDB lifecycle and integration remediation

## Purpose, ownership and authorization

This standalone plan integrates the six findings, eight investigation avenues and two
operator-confirmed rule changes in the
[SurrealDB capabilities, lifecycle and integrations review](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md).
It owns the corrective target, production and inquiry packets, current finding dispositions
and affected qualification. Plan 28 retains its existing mechanisms, N4/T5/L4 consumer
acceptance and E3/E4/E5 campaign; this is not an extension of Plan 28. Plan 34's completed
implementation and acceptance retain their historical meaning.

The maintainer authorized this document's creation and existing-plan reconciliation on
2026-10-10, and explicitly accepted RC01 and RC02 during preparation. That authorization
does not implement the production changes described below or restart the interrupted
campaign. The maintainer authorized full execution on 2026-10-10. The selected target is **Implemented** and **Tested** at the affected boundaries below;
the [checkpoint](#current-checkpoint) owns its next action.

The authoring baseline is `main` at `991fcea1984ea9cca0f281f050c87fad067a1431`, with the
review/index, existing Plan 28 and lifecycle edits, and two unvalidated tooling-fixture
edits already present. Preserve concurrent changes. Reconcile source and frozen deployment
closures again before execution; a Git commit alone does not identify the running supervisor.

Core 3.4, Efficiency Heuristics 1.0 and Process Simulator 1.5 apply through the
[selected standard](../design_review/design_principles/standard.toml) and
[binding](../design_review/design_principles/binding/pse-arrow.md). Review F01–F06, Q01–Q08,
RC01–RC02 and S01–S10 retain their source identities. Qualify references as Surreal Fnn
where older reviews also use Fnn; do not create a second scenario registry.

## Baseline and foundation assessment

The integration already has suitable foundations: one registry-generated representation,
immutable revisions and memberships, exact protected dependencies, staged manifests and
guarded publication, immutable operation receipts, a finite patched SDK transport, and
separate mathematical, native and deployment owners. Salsa and existing bounded caches
retain process-local immutable products; neither store presence nor a fast hash confers
current scientific or publication authority. These foundations should be improved directly,
without another state service, cache, WAL coordinator or production persistence backend.

The formal review establishes the six boundary defects. Focused source inspection during
plan creation confirms their affected foundations:

- `surreal_server.state_lock` blocks in `flock`; reservation joining checks its deadline
  before entering that exclusion. The supplied counterexample establishes an expired
  admission returning success. Correct the actual wait, not merely the outer timeout.
- `service_readiness` collapses live reservations into STARTING and discards unavailable
  causes. `native_operation.drained` safely refuses uncertain completion but returns only
  a Boolean. Existing observation owners can expose the missing distinctions without
  combining storage, receiver and observer lifetimes.
- `CanonicalStore::protected_query` and `staging_query` share `StagingTurns` across complete
  same-problem RPC/retry decisions. Server guards remain the distributed authority;
  client pacing is a separate policy whose scope can be narrowed.
- `CanonicalOptions` consumes native allocation and optional receiver configuration;
  store methods reload them. Runtime/deployment composition needs these facts, but result
  selection does not. Keep one deployment declaration while separating consumed projections.
- Python runtime construction and close stringify canonical failures into
  `WorkflowError::Input`, despite the existing `Canonical` variant and diagnostic route.
- The durable study observer discards terminal `result()`, requests full `status()` and
  pages occurrence facts again. Preserve narrow open-header polling and remove repeated
  terminal hydration at the complete operation's consumer boundary.

The latest stopped setup capture had 754 tests, eight failures and three errors against
baseline zero. Ten terminal issues have source-supported stale-fixture explanations; the
existing edits are not validated repairs. One nested-observer drain failure remains
undiagnosed. The review's unchanged isolated cancellation control passed once, while its
40 ms admission probe returned after approximately 251 ms under metadata-lock contention.
These are distinct observations, not campaign acceptance or a demonstrated observer leak.
The [28e checkpoint](28e-rebuild-retirement-and-qualification.md#checkpoint-and-next-step)
owns current campaign accounting; the review retains the observations and their conditions.

Supplier evidence is scoped to server/Rust SDK 3.3.0, RocksDB and the local SDK patch.
Plan authoring refreshed Context7 documentation for endpoint enforcement and cancellation;
the cancellation results included unversioned main-branch code, not release-qualified proof.
Use the review's exact-release source and the neo4j-surrealdb skill's named historical probes
for pinned claims. Documentation silence or an inconclusive search is not absence. No
database or native capability probe was executed while authoring this plan.

## Confirmed rule changes

| Review item | Operator decision | Required route before dependent implementation |
|---|---|---|
| [RC01](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#rc01) — narrow broad per-problem pacing | Accepted, 2026-10-10 | SD00 records the changed target through a blueprint `design:` amendment to §14.4 and the current operation explanation. SD05 replaces unnecessary read/read ordering while retaining exact server guards, original clocks and writer progress. Plan 34 SI05 remains historical evidence, with a prospective handoff to this owner. |
| [RC02](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#rc02) — separate storage and native deployment configuration consumption | Accepted, 2026-10-10 | SD00 identifies affected deployment/API contract owners and stages their `design:` amendment; SD03 moves native interpretation and validation to composition, migrating all consumers together. The authoritative configuration remains single-owned. |

Acceptance selects the target; it does not assert that amendments, migrations or qualification
are complete. Authoritative edits require the visible design-edit route and blueprint revision
when execution reaches them. Existing declarations govern the baseline until that cutover.

Deadline enforcement, adequate observation causes, existing typed errors and removal of repeated
hydration restore or refine current guarantees. They do not by themselves change D1–D14,
scientific identity framing, commit semantics or the Python boundary contract. No blanket
new ADR is needed for these corrections. An inquiry selecting a new relation family, backend
binding or a changed hashing/metadata/commit/Python contract follows the decision table in
`.claude/rules/decisions.md` before implementation. Newly discovered rule impacts require
explicit confirmation; do not infer approval of another backend from RC01 or RC02.

## Target design and preserved contracts

### Deployment observation, original clocks and owned drain

The existing supervisor owns one observation of the selected lifecycle: its authenticated
identity, relevant phase, readiness predicates and any unavailable or mismatched premise.
Startup consumes that observation to join compatible owned progress, wait for incompatible
lifecycle exclusion under its original admission clock, or refuse uncertain ownership.
An externally activating unit is joinable only when its exact owned launch can be proved;
activating alone is not authority. A stop or maintenance reservation is not a starting server.

The native operation owner exposes drain evidence for its own supervised descendants. Keep
host-admission semantics distinct where they include boot, inode, caller or service guards.
Do not mechanically unify the two drain implementations. A terminal diagnostic includes the
selected identity, failing observation and decision; consumers must not reconstruct this
meaning with later systemd/cgroup calls. Normal polling needs no transcript or new monitoring
platform. Status, startup, fixture eligibility and refusal messages consume the same owned
observation at their respective boundaries.

All admission/control waits use the original absolute deadline, including metadata exclusion.
Use deadline-aware acquisition of the existing lock, and recheck expiry after acquisition
before admitting effects. Propagate that clock rather than minting a new budget for each
helper or retry. Audit actual manager/control waits on these paths as part of the same
operation. A cancelled or expired caller still retains already-owned cleanup obligations;
cleanup has its explicit existing lifetime and may not declare drain when observation fails.
Locks remain short metadata exclusion, never spans of workload or drain IPC.

A socket drain, task cancellation request, empty cgroup and completed native attempt are
different facts. Preserve exact invocation/PID-start/group ownership and independent workers.
Unknown observations pin unresolved obligations; they must not trigger replacement, release
charges or manufacture a completed cancellation. Investigate the prior failure using its
retained evidence and relevant load/topology. If the historical cause cannot be recovered,
record that limit and establish current assembled behavior with decisive failure diagnostics;
do not retroactively label the original failure diagnosed.

### Storage and runtime composition

Project endpoint, credentials, namespace/database, interpretation, write admission and storage
identity to canonical connection construction. Interpret native capacity and managed receiver
associations at runtime/deployment composition and execution selection. A canonical-only read
must be constructible without native-worker policy. A managed execution still needs a valid
current receiver association and finite allocation under the existing authority.

Move policy-bearing allocation/receiver types and reload behavior to their existing semantic
owner where required by dependency direction. Do not introduce a new crate or neutral storage
trait just to move fields. One configuration file may still declare the deployment; decode
the consumed storage projection independently so malformed native-only fields cannot prevent
a storage-only read. Composition may reuse that same declaration, without a second editable
schema or divergent validation. Native association validation is not weakened to mere paths
or digest-shaped strings when current executable identity is required for admission.

Migrate Rust runtime, Python construction, managed workers, fixtures and maintenance callers
according to the capability each consumes. Explicit runtime creation may still require
execution policy; this correction does not invent an offline runtime or make every public
runtime storage-only. Validation and user-facing diagnostics remain at the actual selected
boundary. Preserve existing VIEWER inspection, no implicit schema installation and no MCP
auto-start. Setup/reconfigure/restore keep their administrative roles.

### Canonical read coordination and uncertainty

Classify complete canonical operations by actual effects and conflict premises. Independent
immutable payload/index/source reads may overlap through the existing bounded transport.
Mutation and reclamation coordination remains only where needed. Protection acquisition,
renewal/release or a read that also mutates a guard is not automatically a pure immutable read.
Select coordination from the actual emitted transaction, not its Rust method name.

Keep named guards for positive, absent-name and complete-set premises. Exact-record
`FOR UPDATE` enrollment does not prove predicate/phantom protection or justify serializing
read/read traffic. Retain complete-decision retries only for definite conflicts, finite
admission, original clocks and fairness sufficient for writer/reclaimer progress. Do not
replace the current queue with unbounded tasks or lift deployment capacity to hide contention.

Native transaction blocks and generated database-local functions may compose fitting effects;
neither interactive session transactions nor an application WAL layer is the default.
The reviewed RocksDB commit path can apply before sync completion returns an error. Preserve
original operation identities, uncertain-effect reconciliation and staged/closed publication.
Connection failure or a generic commit error must not authorize blind redispatch or claim
rollback. No new power-loss or exactly-once external-execution guarantee is selected.

### Terminal studies and typed failures

Open study observation continues to poll the compact header. Once terminal, the existing
study owner performs one checked, paged occurrence read and derives the needed outcome/status,
facts and run/attempt associations from that realization. Reserve coexisting retained state
before allocation. Reuse it through the actual consumer lifetime and discard incidental
views when no longer needed; do not persist a duplicate summary or a new snapshot certificate.
Terminal immutability/protection must justify reuse. A changing live page is not a coherent
terminal inventory.

The durable execution observer consumes the realized terminal projection instead of calling
three independent full reads. Public `status()` and `result()` keep their intended meaning;
Q08 may select a paged public interface only for an identified capability or useful bounded
exploration. That decision is not required to correct the internal repeated scan. Preserve
ordering, cancellation, failed/partial classes, exact associations, frame checks and later-page
corruption refusal.

Canonical errors retain their existing structured variant through Python construction,
initialization and close. Context may add stage attribution, but not replace the cause with
an input-error string. Invalid authored input stays distinct from unavailable infrastructure,
interpretation mismatch, deadline expiry and inconclusive drain. Adjacent conversions on the
migrated lifecycle paths receive the same correction; unrelated Python API redesign is excluded.

### Active state, identity and scientific authority

Keep compiler/Salsa handles, owned mutable workspaces, immutable preparations, retained results
and persistent identities distinct. Local fast hashes remain bucket hints with complete
equality; no pointer or process-local graph ID becomes a portable key. Reuse a product only
while its complete consumed premises hold, and acquire current protection/publication authority
separately. Preserve allocation charges through escaped owners and actual thread/TLS drain.

Database constraints, graph traversal, adjacency caching and notifications do not supply
scientific admission, equation incidence, solve order, numerical convergence or current cache
validity. Authored relations and scientific/library owners retain those meanings. Preserve
independent original-space checks, physical conventions, external IDAES/Pyomo grounding and
truthful partial results. A new first-principles thermodynamic oracle is outside this scope.

## Implementation packets and dependencies

The packet table owns implementation progress. Source findings remain open until their
required evidence exists. Dependencies are consumed slices, not whole-document barriers.

| Packet | Required input | Delivery and targeted acceptance | Progress |
|---|---|---|---|
| SD00 — Decision routes and execution premises | Accepted RC01/RC02; current source/profile ownership and the handoffs authored here | Stage the exact contract amendments before dependent changes. Reconcile current owners, frozen tooling/worker closures and the two unvalidated fixture edits. Select unchanged positive/negative controls; preserve concurrent resources. No campaign relaunch or blanket reset. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD01 — Lifecycle observation and cancellation evidence | Existing supervisor/native observation contracts; F02, S01/S02 | Implement meaningful phase/identity/cause observations and migrate startup/status/drain/fixture consumers. Distinguish owned startup, stop/maintenance, mismatch, unavailable manager and actual completion. Validate the existing fixture corrections at current entrypoints; exercise real nested cancellation and independent storage survival. Remove replaced Boolean interpretation and obsolete mocks. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD02 — Deadline-aware control waits | SD00; SD01's failure distinction where consumed; F01, S01 | Make metadata acquisition and relevant control waits respect the original clock, including post-wait expiry. Exercise held-lock expiry with no late admission, same-owner join/replacement refusal and cancellation with retained cleanup. Delete unconditional blocking admission paths after callers move. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD03 — Capability-correct deployment composition | SD00's RC02 route; existing storage/runtime/worker contracts; F04, S04/S05/S08 | Separate storage projection from native deployment interpretation and migrate all affected constructors/reloads. Test storage-only access without native policy, execution refusal for invalid association, isolated contexts and stale generations. Delete storage-owned native policy/reload APIs after consumer migration. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD04 — Typed Python infrastructure failures | Existing Canonical diagnostic projection; F05, S07; coordinate shared constructor edits with SD03 | Preserve canonical causes through runtime connect/open, explicit initialization and close. Exercise infrastructure timeout, interpretation mismatch, failed disconnect and invalid user input as distinct outcomes. Remove affected Input(string) conversions; retain protected-reader drain. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD05 — Independent protected reads | SD00's RC01 route; SQ01's emitted-operation classification; F03, S03/S10 | Narrow local pacing without changing distributed guard premises. Show two actual independent reads overlap; retain sixteen-reader/staging controls, writer/reclaimer progress, expired protection, absent/set edits and exact lost-ack settlement. Migrate affected queries and remove displaced broad coordination/tests. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD06 — One terminal occurrence realization | Existing terminal study contract and SD00; SQ08's internal consumer assessment; F06, S06 | Derive all internal terminal views from one checked paged inventory under correct allocation ownership. Test multipage ordering, failed/cancelled/partial outcomes, exact associations and later-page corruption. Retain compact open polling; delete repeated internal scans and their obsolete assertions. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SD07 — Affected integration, qualification and handoff | SD01–SD06 complete; every SQ inquiry decided and selected migrations/controls delivered | Run affected assembled lifecycle, storage/read/reclaim, study, managed execution, Python and maintenance journeys; scope-end checks and independent bounded integration review. Reconcile dispositions, enduring owners and Plan 28 prerequisites. Do not acquire the full E3/E4/E5 campaign. | Complete; see [acceptance evidence](#acceptance-evidence) |

SD01 establishes the observation contract used to explain SD02 refusal and cancellation;
its entire real-system qualification need not block local deadline work. SD03 and SD04 share
constructor files but have distinct semantics; coordinate edits instead of forcing an
artificial logical dependency. SD05 consumes only SQ01's operation classification, not every
query optimization. SD06 consumes the internal terminal-view decision, not a new public API.
SQ02–SQ07 can resolve independently once their relevant contracts are understood. Each selected
inquiry change migrates its consumers and supplies targeted controls in the same packet;
SD07 is not a deferred caller-migration phase.

The root owns shared contract changes, finding disposition, cross-owner reconciliation and
final acceptance. Bounded workers may own independent source or library questions under the
shared coordination contract. Shared edits in canonical operations, schema generation or Python
constructors require explicit coordination even when the capabilities are independent.

## Bounded investigation packets

This table owns inquiry progress and any selected implementation. Each packet ends with a
decision: adopt and integrate, demonstrate no fitting correction, or explicitly defer a
consequential decision with its owner and observable trigger through the existing register.
A report recommending unspecified future research is not completion. Genuine architectural
findings cannot be called resolved merely by deferral.

Use the neo4j-surrealdb and relevant Rust capability skills with Context7 and primary exact-release
sources. Library eligibility is not restricted by licence or absence of an existing consumer.
Assess the complete consumed capability and machinery removed or introduced. Documentation/API
evidence can settle a choice; execute a probe only when consequential uncertainty warrants it.

Adopt a correct expected-beneficial approach unless it fails its contract, is an ineffective
mechanism or substantially impairs performance. Small, noisy, inconclusive or slightly adverse
timings under concurrent repository work are insufficient rejection reasons. Distinguish
structural work removed from measured latency/throughput/capacity benefits. No arbitrary
speedup threshold, cost model or new permanent benchmarking platform is required.

| Packet | Question, evidence and decision boundary | Integration and acceptance | Progress |
|---|---|---|---|
| SQ01 — Complete native access paths (Q01) | Inspect emitted protected reads, exact predicates, interval/dereference access, multi-selector intent, payload projections and actual EXPLAIN/execution evidence. Compare short transaction/function composition with added interactive-session lifetime. Returned-row limits do not bound examined work. | Supply SD05's read/effect classification early. At canonical query owners, implement fitting predicate/index/bulk/projection corrections and migrate consumers. Preserve exact membership/absence, framing and later-page refusal; examine complete-operation behavior under skew and mutation. Retain Plan 34's disproved binary-index result until changed planner evidence warrants revisiting it. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SQ02 — Native integrity and inverse ownership (Q02) | For applicable registry families, compare ENFORCED endpoint admission, REFERENCE deletion/inverse semantics and existing checks. Establish staging order, independent evidence lifetime and import bypass; do not infer dangling current publications merely from missing ENFORCED. | Select constraints/bookkeeping replacements at the registry/generator owner, regenerate rather than hand-edit outputs, and test creation, missing endpoints, retained evidence, reclamation and actual restore integrity. A new relation family or changed metadata/commit meaning takes its decision route first. Keep guards/validation that establish different properties. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SQ03 — Fitting graph representations (Q03) | Examine useful derived connectivity and actual traversals for LIGHTWEIGHT or native adjacency caching. Compare edge metadata, multiplicity/order, high degree, write amplification, overflow/bypass and batched starting-record lookup. | Integrate fitting representations at existing topology/retrieval owners with all readers/writers migrated and replacement removal. Test exact traversal and lifecycle behavior, including high-degree cases. Preserve equation-structure and numerical graph authority; do not add a second persisted topology without a concrete benefit. | Complete; existing route retained (decision below) |
| SQ04 — Streaming cancellation and SDK patch retirement (Q04) | Compare each local patch guarantee against pinned upstream and streaming facilities: local admission, dispatch uncertainty, original clocks, reserved control progress, setup replay, provisional/terminal frames and physical drain. | Retire only patch mechanisms whose complete consumed contract is supplied by the selected library route, with affected transport controls and callers migrated. Retain the remainder explicitly. Cancellation ACK is not completion, socket drain is not native/server completion, and uncertain writes retain their original identities. An upgrade is exact-pinned and scoped, not wholesale resolution. | Complete; existing route retained (decision below) |
| SQ05 — Local task lifetime primitives (Q05) | Identify generic in-process bookkeeping that a fitting JoinSet, TaskTracker or cancellation token can remove. Separate signal, outcome collection, task join, started blocking work and native/remote completion. | Replace fitting task machinery at its existing owner and delete the old path. Test failure/panic evidence, cancellation races and actual joined resource lifetime. Do not let a helper that discards outcomes replace a drain that must report failures. No new resource caps or scheduling framework. | Complete; see [acceptance evidence](#acceptance-evidence) |
| SQ06 — Notifications and database-local projections (Q06) | Assess LIVE, CHANGEFEED/SHOW CHANGES and sync/async events for useful wakeups, catch-up or local projections. Resolve Rust reconnect/unsubscribe, cursor retention gaps, duplicates and retry exhaustion. | Integrate a fitting notification/projection with explicit owner and recovery behavior. Test interruption, gap recovery and idempotence. Keep authoritative reread, current cache dependencies and durable claim/publication protocols; events must not mint scientific completion or exactly-once native effects. | Complete; existing route retained (decision below) |
| SQ07 — Maintenance, deployment and native frontends (Q07 and review §6) | Compare export/checkpoint facilities and embedding against full backup/recovery and shared multiprocess authority. Assess existing native MCP, Postgres wire, GraphQL and DEFINE API for fitting adapter removal or useful exploration; distinguish SurrealQL from PostgreSQL semantics, authorization and body/identity contracts. | Implement fitting maintenance/frontend simplifications at existing owners with restore/authorization and affected consumer controls. Preserve credentials, deployment envelope, fresh authority and exact scientific values. Retain native MCP rather than duplicate it. A backend/deployment replacement, new authority or changed public contract requires its own confirmed rule/decision route; no hidden fallback. Experimental modules/WASM are not presumed numerical execution owners. | Complete; existing route retained (decision below) |
| SQ08 — Active lookup and study interfaces (Q08) | Assess bounded exact-equality compiler request lookup and actual study consumers. Determine whether indexed lookup or paged public status removes useful work without duplicating authority, retention or representations. | Supply SD06's internal single-read projection decision early. Integrate fitting lookup/interface changes with exact collision/equality, invalidation/ownership and paging controls. Do not make a new public API or compiler index a prerequisite for removing repeated terminal scans. Keep process-local handles out of durable identity. | Complete; see [acceptance evidence](#acceptance-evidence) |

Record a selected realization, affected owners and acceptance beside its inquiry as execution
proceeds. A capability may be useful without replacing all custom logic: explicitly preserve
the remaining contract instead of claiming wholesale native equivalence. If a decision changes
other packets' assumptions, update their inputs and controls together before dependent work.

### Inquiry decisions selected during execution (2026-10-10)

- **SQ01:** retain exact guarded transaction and interval semantics; batch full-source scope,
  document/data, package and fit selection through existing bounded protected selectors. Native
  query-plan controls examine scoped intervals and target dereferences before any new index is
  justified. Existing binary-index counterevidence remains applicable.
- **SQ02:** enforce endpoints for memberships, analysis edges/inputs, problem runs and run
  sources. Create the problem endpoint earlier in the same source-edit transaction. Study
  dependencies remain unenforced because valid predecessor references can cross ingestion pages;
  enforcing them would require another staging mechanism. REFERENCE/cascade does not replace
  bounded semantic reachability cleanup or independently retained evidence. Import bypass still
  requires post-restore integrity validation.
- **SQ03:** retain the current graph representation. Emitted production queries use indexed
  records/tables; adjacency caching does not accelerate those predicates. LIGHTWEIGHT relations
  would lose consumed metadata and require another topology. No concrete traversal consumer
  justifies that replacement in this scope.
- **SQ04:** retain the pinned SDK patch. Upstream streaming cancellation does not supply the
  complete original-clock, reserved-control, dispatch uncertainty, ordered finite replay or
  physical-disconnect contracts. Cancellation acknowledgement cannot establish completion.
- **SQ05:** replace fixture release counters/notifications with TaskTracker tokens while preserving
  explicit live-reader admission and failure/panic/cancellation evidence. Native thread joins and
  outcome-bearing task joins retain their distinct obligations.
- **SQ06:** retain compact authoritative polling. LIVE/changefeeds add subscription/reconnect,
  gap and duplicate handling while still requiring the same current-header reread. No selected
  database projection replaces scientific transition authority; SD06 removes the actual repeated
  terminal hydration.
- **SQ07:** retain existing native export/import, preserving deployment envelope and fresh
  admission. Embedding cannot replace shared multiprocess authority; additional protocol frontends
  would add contracts without retiring a consumed adapter. Existing authenticated bounded native
  MCP remains the exploration route. No backend or public frontend replacement is selected.
- **SQ08:** adopt generation-local FxHash buckets with full equality within collisions and
  pre-admitted collection overhead. Retain the existing complete public study views: paging them
  would add a second interface without retiring required complete associations. Internal terminal
  consumers instead share one checked occurrence realization.

## Finding dispositions

This table is the single current disposition owner for this review. Packet progress remains
in the tables above; historical review findings and Plan 34 outcomes are not rewritten.

| Finding | Scenario | Disposition | Decision/work owner | Evidence or completion condition |
|---|---|---|---|---|
| [Surreal F01](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#f01) | S01 | Resolved | SD02 / existing supervisor admission owner | **Tested:** held supervisor/host-ledger exclusion and post-acquisition expiry controls, including actual child-held locks, prevent late launch or ledger mutation. The complete setup selection passes; see [acceptance evidence](#acceptance-evidence). |
| [Surreal F02](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#f02) | S01/S02 | Resolved | SD01 / supervisor and native-operation observation owners | **Tested:** typed observation controls and actual nested cancellation, kernel drain, independent storage survival, restart exhaustion and composed fixture retirement pass. The historical failure cause remains unresolved; see [acceptance evidence](#acceptance-evidence). |
| [Surreal F03](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#f03) | S03/S10 | Resolved | RC01; SD05 with SQ01 / canonical operation owner | **Tested:** actual SDK independent-read overlap, sixteen-reader/mutation, protection, reclamation and lost-ack controls pass in the assembled canonical selection; see [acceptance evidence](#acceptance-evidence). |
| [Surreal F04](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#f04) | S04/S05/S08 | Resolved | RC02; SD03 / runtime/deployment composition | **Tested:** storage-only configuration and invalid native selection controls pass; original Rust/Python sixteen-owner and cancellation journeys validate migrated execution composition. See [acceptance evidence](#acceptance-evidence). |
| [Surreal F05](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#f05) | S07 | Resolved | SD04 / existing workflow diagnostic and Python owners | **Tested:** typed canonical error projection and Python diagnostic/cache journeys retain infrastructure causes separately from invalid input; see [acceptance evidence](#acceptance-evidence). |
| [Surreal F06](https://github.com/paul-heyse/pse-arrow/blob/313b53ac9dceb770c3254b6a20b3bd7f57e69af7/docs/design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md#f06) | S06 | Resolved | SD06 with SQ08 / study result and execution consumers | **Tested:** terminal inventory ordering, retained ownership, exact associations and later-page corruption controls pass, alongside original native/Python study consumers; see [acceptance evidence](#acceptance-evidence). |

S08's domain extension and S05's integration replacement are preservation scenarios across
SD03/SQ02–SQ05/SQ08. S09 belongs to SQ02/SQ03/SQ06/SQ07; S10 belongs to SD05/SQ01/SQ04
and uncertainty controls. This coverage does not make every optional frontend a mandatory
production feature.

## Migration, deletion and operational adoption

Build the selected target directly. After replacement targeted controls pass and callers move,
delete displaced mechanisms, callers, internal tests and fixtures in that same change. Do
not retain shims, compatibility APIs, obsolete analyses or a second production path merely
to reproduce immature internal historical results. Original physical and external reference
assertions remain independent acceptance bases.

Most core corrections do not require a stored-schema migration. SQ02/SQ03 or other selected
capabilities may change schema/interpretation or derived state. Before their cutover, establish
which data is authored, live/protected, scientific evidence or recreatable derived output;
schedule explicit owner-correct quiesce, drain, generation/interpretation transition and
rebuild/validation where required. Early-design reproducibility permits rebuilding obsolete
derived runs but does not authorize blanket deletion of shared state or concurrent work.
Retired analyses are removed entirely once authentic protection/drain permits retirement.

Changes to Python/tooling closures invalidate their frozen receiver/supervisor generations.
Refresh only affected owned profiles through existing stopped/drained reconfiguration and
readmission, preserving credentials and unrelated databases. A copied path, PID or hash is
not current deployment qualification. Do not stop services during document authoring.
Actual deployment transition belongs to later authorized execution and its owner.

## Existing-plan coordination

The first authoring action reconciled stale live instructions with the review pivot. Plan 28
and 28e retain the campaign's current checkpoint and conditional restart route. Plan 34's
handoff identifies the new prospective target while preserving completed SM/SI dispositions
and evidence. Other companions link those owners rather than repeat a live campaign status.
Plan/index adoption supplies no new functional or scientific qualification.

On the authorized Plan 28 testing continuation after this plan completes, select affected Plan 35 prerequisites by actual
consumption:

| Existing acceptance route | Prerequisite supplied here |
|---|---|
| Startup, registration, interruption, observer/fixture retirement | SD01/SD02 and the affected real controls; preserve authentic identities and resource charges. |
| Runtime/worker construction, isolated databases, restore and frozen generation admission | SD03 and affected SD04/SD01 slices; retain Plan 34 recovery-oracle meaning and actual readmission. |
| Concurrent protected selection, staging, retention and connected results | SD05 and selected SQ01–SQ03 changes; preserve absence/set/phantom and late-page guarantees. |
| Durable/public Python study results and cancellation | SD04/SD06 plus selected SQ08 interfaces; preserve original point counts, concurrency, numerical controls and exact lineage. |
| An inquiry-selected transport, notification or maintenance change | Only its migrated consumers and matching SQ04–SQ07 acceptance; no unrelated blanket inquiry gate. |

One exact matching invocation may provide evidence to both plans, naming current source,
feature mode, store/allocation, native environment and actual outcome. Link its owner instead
of launching a duplicate full campaign. A targeted pass or this plan's affected acceptance
does not close E3/E4/E5. Plan 28's original 37-selector E4 selection, original thousand-point
and sixteen-case workloads and reference capacities remain with 28e; no reduction or
transfer flag is selected here. Optional strict producer, distribution/remote CI and a new
physics-oracle framework remain outside the current qualification target.

## Verification and completion

**Tested, affected scope (2026-10-10).** The acceptance evidence below records the executed
commands, modes, zero-failure baseline and qualification limits. The original proposed
controls remain the coverage specification. During implementation use `scripts/pse-env --` with targeted compile
checks and unit controls at the touched owners. Rust tests explicitly enable
`pse-relations/force-validate`; managed selections retain the existing nextest filter rules.
Generator/registry changes regenerate their outputs in the same packet. No mid-plan full
integration or hygiene sweep replaces functional delivery and deletion.

Decisive controls include:

- Hold the supervisor lock beyond the original admission deadline and prove timely expiry
  with no delayed effect. Separately prove owned cleanup continues after cancellation.
- Distinguish owned startup, stop/maintenance, replaced invocation, unavailable manager,
  populated descendants and completed drain. Validate positive fixtures and real nested
  foreground cancellation while independent storage/worker lifetimes survive.
- Construct storage-only access without native policy, then refuse an invalid managed
  execution association. Exercise explicit contexts and current-generation changes.
- Carry canonical timeout, interpretation and disconnect failures through Python alongside
  genuinely invalid input. Preserve protected-result-reader drain.
- Observe two actual immutable reads in flight together and preserve mixed sixteen-reader,
  staging/reclamation, writer progress, absence/set changes, expiry and lost-ack settlement.
- Consume a multipage terminal study once, including exact ordinals and run/attempt links,
  failed/cancelled/partial outcomes and corrupted later pages. Check open polling remains
  header-only and retained state is reserved through its real lifetime.
- For selected capability changes, exercise the distinct failure/recovery cases named in
  SQ01–SQ08; mocks alone cannot prove emitted SQL, import integrity or native drain.

At functional scope end SD07 selects affected integration, component, native/managed and
Python journeys, actual lifecycle and maintenance/reopen controls, and applicable external
reference comparisons. Keep physical/provider validity, independent post-solve assessment,
exact result bits, live ownership and truthful partiality intact. Run scope-end hygiene and
the applicable governance/docs/native/powerset checks once on stable final inputs; repair
failures at their owners and rerun invalidated covering checks. An independent bounded
integration review assesses the target and migrated consumers, not unexercised whole-product
or power-loss qualification.

Measurements concern complete affected operations where a consequential benefit or significant
regression remains uncertain: read/reclaim contention, terminal-study hydration, skewed access,
adjacency maintenance or changed deployment boundaries. Report actual contention and source/
resource conditions; concurrent repository activity may make timings noisy. Prefer existing
plans/tracing and bounded counters over new instrumentation. A functional concurrency or
single-hydration control is not a measured speedup.

Completion requires SD01–SD06 and all selected inquiry migrations delivered with replaced paths
removed; every inquiry has a supported decision and any genuine deferral has its existing
register route; all six finding dispositions have their required correction evidence; SD07
has affected assembled acceptance and required checks. Unresolved historical causes remain
explicit even if current behavior is established. A deferred unresolved architectural finding
does not satisfy the target by scheduling alone.

Move enduring meaning to the existing architecture/dev owners through the decision route,
update Plan 28's dependency handoff and current-work selection, then retire only eligible
completed documents/reviews under ADR-0096. Retain the source review while findings or decisions
depend on it. Do not retire Plan 34 merely because this plan has a higher number. When production
work closes, record an evidence-labelled Outcome with what was built, a mistake corrected,
deliberate deviations and qualification limits.

## Current checkpoint

Functional scope and affected integration are complete (2026-10-10). SD00–SD06 and selected
SQ01/SQ02/SQ05/SQ08 changes are integrated, their replaced paths removed, and all six finding
dispositions have correction evidence. The four retained-design inquiries have explicit
technical decisions. Accepted RC01/RC02 contracts live at their architecture owners.

The final setup, assembled canonical, native/Python study and managed sixteen-owner controls
pass. Actual lifecycle/MCP/bounded restart and maintenance controls compose successfully in
one exclusive owner; actual scientific recovery also passes on the fresh final reference.
Independent bounded source review found no remaining material issue. Integration repairs
preserve exact probe/borrowed-service ownership, original cleanup clocks, concurrent host
registrations and private receiver parents. Failed profiles remain preserved. The historical
nested-observer cause remains unresolved; current controls establish current behavior only.

SD07 is complete: affected acceptance, repaired scope-end checks, governance, native lints,
full feature powerset, independent bounded review and documentation retirement have passed.
The resolved review is retained through immutable Git references; this highest-numbered plan
remains as the completed handoff. Proceed directly to the maintainer-authorized remaining
Plan 28 E3/E4/E5 campaign at [28e's checkpoint](28e-rebuild-retirement-and-qualification.md#checkpoint-and-next-step).
Keep original thousand-point/sixteen-case workloads and reference capacities unchanged.

## Acceptance evidence

**Tested, 2026-10-10; baseline zero throughout.** These are affected Plan 35 controls on
Linux with SurrealDB/Rust SDK 3.3.0, RocksDB and the pinned native provider environment.
Rust correctness invocations enable `pse-relations/force-validate`. The assembled canonical
and managed Rust controls use `local`; the five earlier study controls used `default` as
recorded below. No retry, reduced numerical fixture or historical internal-result
comparison supplies acceptance. Commands below run through `scripts/pse-env --` and retain
private raw artifacts under the named ignored `build/` directories.

| Scope and command | Actual result and conditions |
|---|---|
| `just setup-test-report build/plan35-setup-private-receivers-20261010` | 805 tests passed, zero failures/errors, 41.871 s; includes held-lock original-clock expiry, exact borrowed-affiliation return/races, fixture host-reference refusal and private receiver parent creation/refusal. |
| `just unit-package pse-operations 'test(/_server_unit::/) \| test(canonical_capability_tests)' --features pse-operations/canonical-tests --profile local` | 103 passed, zero failed, 60 unselected; 262.881 s, Nextest `83faab39-1f42-4db8-8ac8-8c43ca2dc897`. Actual independent read overlap, protected mutation/reclamation, bounded selector behavior, endpoint enforcement and native query-plan controls. |
| `just unit-native-package pse-runtime canonical-tests,native-solvers <filter>` with the exact study filter below | Five passed, zero failed, 178.791 s, `default` profile; Nextest `003d4a08-5aa9-4e75-bdf9-989016745a4c`. Cancellation, equal bindings, claim expiry, live owner and terminal cancellation retain original contracts. |
| `just py-test python/pse/tests/test_canonical_results.py -m 'component or (integration and not producer_deployment)' -q --tb=short` | Six passed, zero failed, 131.25 s; selected canonical diagnostic/cache/result journeys under the reference owner. |
| `just native-test --profile local --managed-primary-route -p pse-runtime -E 'test(managed_primary_study_enters_sixteen_native_owners_and_retains_original_results) \| test(managed_primary_study_cancellation_drains_admitted_sixteen_and_refuses_tail)'` | Both original controls passed, zero failed, 788 unselected; 153.394 s, Nextest `34268ad4-f11e-4475-a371-d1508152e7e9`. Fresh private final reference profile; sixteen native owners, original results, cancellation/drain and tail refusal. |
| `just native-python build/plan35-managed-python-final-20261010 --managed-primary-route --tb=short -q` with the original sixteen-owner result, sixteen-owner cancellation and unusable-predecessor partial tests | Three passed, zero failed, 114.25 s; original public Python workloads and assertions. |
| `.venv/bin/python -m scripts.tests.surreal_fixture_check` with full, `--mcp-only` and `--bounded-b-controls` selections, followed by `scripts/tests/efficiency_maintenance_actual_check.py` in one exclusive owner | All four commands and enclosing owner exited zero. `build/plan35-{lifecycle,mcp,bounded,maintenance}-current-20261010` retain positive controls, genuine canonical initialization, exact probe retirement, no outstanding host references and successful fixture retirement. Real SIGKILL/reopen, uncertain-write reconciliation, backup/restore, native budget/slot/drain, restart exhaustion and VIEWER/missing-context refusal are exercised. |
| `just canonical-recovery-test --scientific --profile-state "$PSE_SURREAL_STATE" --directory build/plan35-scientific-recovery-current-20261010` | Exited zero on the final reference profile, with native SIGKILL/reopen, protected history/exact bits, large staging, stale authority/refusal, gated restore and public-API rebuild/original-space solve checks. |
| `just hygiene --output build/plan35-hygiene-20261010` and affected repair commands `just lint-py`, `just typecheck` | Original 28-gate sweep: 26 passed, two Python static gates failed. Both repaired gates subsequently exit zero with no lint findings/type diagnostics (two configured type suppressions). This is repaired composite scope-end evidence. Generation, Rust/default/no-default lint and documentation checks passed. |
| `just governance`, `just lint-native-contracts`, `just lint-native-data` | All exited zero; both native lints report zero warnings. Governance artifacts: `build/assessment/20261010T225736.114499Z-governance-3317852-c51db3`. |
| `just features-powerset` | Both feature-combination and no-default-feature gates exited zero; 355 feature combinations. Artifacts: `build/assessment/20261010T225813.227922Z-features-powerset-3327065-640030`. |
| `just docs` | Exited zero after resolved-review retirement; 311 chapters published with scoped search. |

The exact five-control study filter was:

```text
test(canonical_study_cancel_preserves) | test(canonical_study_equal_bindings_and_failed_usable_dependency) | test(canonical_study_expired_claim) | test(canonical_study_summary_live_owner) | test(canonical_study_cancellation_after_summary_close)
```

The three managed Python selectors were:

```text
python/pse/tests/test_studies.py::test_public_study_sixteen_native_owners_retain_equal_occurrences_and_original_roots
python/pse/tests/test_studies.py::test_public_study_cancel_drains_sixteen_native_owners_and_never_starts_tail
python/pse/tests/test_studies.py::test_study_unusable_predecessor_retains_refusal_without_dispatch
```

Additional focused compiler hash/equality, transport, native-selection and terminal-page controls
passed at their affected owners. The separate runtime full-source control passed one selected
test at `pse-runtime::workflow::modeling::canonical`; the canonical server count does not
include that runtime test. The assembled canonical selection supersedes the repaired
single retention fixture. The final setup selection supersedes the earlier setup captures.
Independent bounded source review found no remaining material issue in the integrated target,
including final deadline forwarding, concurrent host-owner preservation and private parent
creation. It does not substitute for Plan 28 E5's independent assembled campaign review.

The final managed/scientific profile is
`$HOME/.local/state/pse-arrow/surreal-plan35-final-reference-20261010` (port 18247,
`pse.substrate.v3`, nonresident `plan28-reference`). Its original envelope is 128 GiB pool,
16 GiB worker, sixteen lanes and 32 math workers; host placement is 140 GiB primary,
16 GiB server and 4 GiB observer. `target/debug/pse-worker`, the linked Python extension and
frozen current receiver closure identify deployed inputs. Resume a quiesced profile by
`just surreal readmit` and `just surreal start` inside its admitted owner; never infer readiness
from a previous process or directory. Failed profiles and their evidence remain preserved.

**Qualification limits:** affected acceptance does not establish Plan 28 E3/E4/E5, a measured
speedup, physical power-loss durability or whole-product scientific equivalence. The native
query-plan control returns 208 selected members in 64/64/40 pages and 104 inverse candidates;
those counts do not establish examined work or throughput. The historical nested-observer
failure remains undiagnosed. Current real controls establish current owned behavior only.

## Outcome (recorded after implementation)

### What was built

**Implemented:** SD00–SD06 and selected SQ01/SQ02/SQ05/SQ08 changes are integrated at their
existing semantic owners. Admission and cleanup observations preserve exact owned identity
and original clocks through metadata exclusion; storage configuration no longer interprets
native policy; runtime composition validates execution capabilities; Python retains canonical
error causes; independent immutable reads share fair per-problem coordination; terminal study
consumers reuse one checked occurrence realization. Bulk full-source reads, ENFORCED endpoint
constraints, tracked release tasks and full-equality compiler hash buckets remove the selected
work without changing scientific identity or publication authority. Replaced native-store APIs,
broad read serialization and repeated internal terminal scans are removed.

**Implemented:** accepted RC01/RC02 enduring contracts are recorded in blueprint revision 146,
§14.4 and §20.6 through the design-edit route. The substrate guide owns observation, admission,
recovery/adoption, exact recovery-probe retirement, stopped borrowed-service affiliation return,
fixture host-reference refusal and private receiver-directory creation. No accepted ADR was
rewritten. SQ03/SQ04/SQ06/SQ07 retain the existing graph, patched SDK, compact authoritative
polling and shared deployment/native frontend routes for the explicit technical reasons above;
no second topology, backend, notification authority or public study interface is introduced.

**Tested:** the [acceptance evidence](#acceptance-evidence) resolves all six findings at their
required boundaries, including real lifecycle, managed execution and scientific recovery.
The affected scope uses repaired composite evidence where earlier attempts failed; each final
covering selection named there has zero failures. Full feature-powerset and documentation checks also pass. The resolved source review is
retired to immutable Git history; current dispositions and evidence remain here.
Plan 28 independently owns the subsequently authorized comprehensive E3/E4/E5 continuation.

### A mistake made and corrected

Real composition exposed ownership defects that isolated tests had not exercised: a successful
recovery probe database remained in backup, and a stopped service's borrowed affiliation
remained in the enclosing host allocation. Successful probes now retire only their exact
acknowledged database after namespace verification; stopped borrowed affiliations return only
after exact generation and kernel-drain proof, preserving concurrent owner changes and the
original cleanup clock. Failed or uncertain probes and unrelated state remain retained.

The original managed tests also exposed intermediate receiver directories created with the
caller's permissive umask. Native selection correctly refused them. The producer now creates
every new parent privately and rejects unsafe existing parents without chmod/adoption; the
unchanged original managed tests pass on a fresh profile. Other corrected test assumptions
include genuine canonical initialization, selected namespace/database headers, finished
retention pins and bounded terminal fixture pages. The historical failure capture and failed
profiles retain their original meaning; current passing behavior does not diagnose the earlier
nested-observer failure's cause.

### Deviations from the plan, deliberate

Actual lifecycle composition required the exact successful-probe retirement, host-affiliation
return and producer privacy fixes described above. They restore the existing preservation,
capability and ownership contracts rather than add a new backend or change scientific semantics.
The task-owned dead allocation from an earlier retired MCP fixture was recovered only after
positive retained retirement evidence, exact record comparison and fresh proof for every
registered unit; its receipt is `build/plan35-mcp-20261010/allocation-recovery.json`.
No other allocation or application state was removed.

The maintainer's subsequent authorization adds the entire remaining Plan 28 testing campaign
after this standalone scope. That work remains with Plan 28 and does not enlarge Plan 35's
claim of affected acceptance. Optional strict producer/distribution checks, a new thermodynamic
oracle, physical power-loss guarantees and a measured performance benefit remain outside this
Outcome's evidence. Concurrent repository workloads were present; small noisy timing changes
are not grounds for rejecting an otherwise correct expected-beneficial implementation.
