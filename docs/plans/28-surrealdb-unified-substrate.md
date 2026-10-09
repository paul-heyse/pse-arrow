---
title: SurrealDB unified simulation substrate
status: in-progress
date: 2026-10-05
adrs: [ADR-0164, ADR-0166]
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md, docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md, docs/design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28: SurrealDB unified simulation substrate

## Purpose, decisions and evidence boundary

Build one canonical SurrealDB substrate for authored problems and revisions, semantic
dependencies and portable compilation descriptions, run/study state, scientific results and
analysis lineage. Ordinary application runs retain their scientific outcomes automatically.
Native connected queries and Arrow export make those outcomes available by exact
problem/revision/run/output selection after restart. Selected compilation uses database-native
structural operations and shared scientific kernels instead of lowering an entire source
bundle at every request.

The [unified-substrate review](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md)
accepted this direction at **Proposed** design strength. Its findings and the
[earlier efficiency review](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md)
supply the basis. This coordinator owns the combined target, shared decisions, sequence and
US01–US05/EF01–EF08/F01–F05/PE01–PE06 dispositions. Companions own package progress/local evidence; E owns
assembled qualification. Architecture sections and ADRs retain their authority roles.

The maintainer explicitly accepted RC01–RC10 on **2026-10-05**, then selected a **clean rebuild
and hard pivot** and **native queries plus Arrow**, retiring DataFusion SQL convenience.
These decisions supersede the review's preserving-import assumption and optional SQL
projection route. Regenerate controlled scientific artifacts; introduce no legacy importer,
compatibility reader or dual-write period. Target history and ongoing recovery remain required.
Plan authoring implements these documents; production execution begins with R0 when that
scope is authorized.

Inspected baseline: main at `6498b013e579ec8039573200c92282eb8715b52e`, including concurrent
uncommitted Plan 27 accuracy work and review/evidence documents. Preserve concurrent edits.
The [Plan 27 checkpoint](27-contextual-engineering-accuracy.md#current-checkpoint) records
functional handoff; [25k](25k-integrated-qualification-and-closure.md#current-execution-checkpoint)
retains original scientific finding dispositions and incomplete campaign evidence. Applicable
scientific obligations transfer to E's target campaign without a requirement to finish the
obsolete PG/Delta campaign first. Previous focused passes and this design acceptance do not
qualify the new store, codec, compiler boundary or durable lifecycle.

## Reading route and shared contract owners

| Document | Responsibility and contract ownership |
|---|---|
| [28a — Canonical substrate and revisions](28a-canonical-substrate-and-revisions.md) | Deployment/acknowledgment profile; one schema/codec declaration route; immutable versions, membership/head/name guards; retention and protected reads. |
| [28b — Selected compilation and reuse](28b-selected-compilation-and-reuse.md) | Selected kernel inputs; scientific admission; portable products and complete dependencies; relevant implementation identity and native preparation. |
| [28c — Durable execution and studies](28c-durable-execution-and-studies.md) | Run/attempt/study meaning, claims/generations/cancellation, bounded staging and terminal admission. C1 supplies the early execution/result envelope. |
| [28d — Connected results and analysis](28d-connected-results-and-analysis.md) | Result physical layout, exact selectors, native Rust/Python queries, Arrow streams and graph-analysis lineage. |
| [28e — Rebuild, retirement and qualification](28e-rebuild-retirement-and-qualification.md) | Regenerated adoption corpus; continuing deletion/dependency/build integration; sole assembled target campaign and closure. |
| [28f — Shared numerical projections and preparation](28f-shared-numerical-preparation.md) | Ordered identity projections, immutable checked preparation and compatible library/native owners across scientific workflows. |
| [28g — Bulk data operations](28g-bulk-data-operations.md) | Shared physical admission, sufficient acknowledgments, composed protected acquisition and comparable source/analysis/authoring migrations. |
| [28h — Native setup and artifact identity](28h-native-setup-and-artifact-identity.md) | Operation-scoped verified installations, actual capability setup and separated artifact/outer-provenance association. |
| [28i — Receiving-runtime validity and interruption](28i-runtime-validity-and-interruption.md) | Actual receiving premises, protected validity/trust-transition lifetimes, interruptible observation and shared completion ownership. |
| [28j — Pure preparation and explicit publication](28j-pure-preparation-and-publication.md) | Complete immutable basis and fresh attribution; pure/effect separation, reusable descriptions and explicit publication contract. |
| [28k — Graph kernels and hashing investigations](28k-graph-kernels-and-hashing-investigations.md) | Bounded graph preparation/placement, role-specific hashing, complete invalidation and framing/persistence decisions; no second finding or qualification ledger. |

The document division expresses responsibilities, not sequential phases. Conceptual
products do not require a new crate/type/service for each concept. Root coordinates shared
declarations, generated outputs, manifests and integration.

## Combined target and foundation decisions

Repurpose `pse-operations` as the concrete substrate boundary with the remote client there.
Retire `pse-operations-queries` and `pse-catalog` after their consumers move. The initial implemented
profile selected authenticated loopback RocksDB, synchronous transaction acknowledgment and
gRPC. [28a](28a-canonical-substrate-and-revisions.md#server-profile-and-acknowledgment-contract)
retains its original profile evidence; Plan 30 owns the proposed replacement below. This isolates engine builds/resources from
semantic crates and serves multiple workers without a second storage abstraction.

Existing semantic declarations, scientific kernels, solver routing, native attempt ownership
and typed Python/Arrow boundaries remain useful foundations. Change their consumed input and
lifecycle contracts where whole-package hydration, lost admission context or whole-study
dispatch obstruct the target. Database queries/functions supply structural resolution and
dependency operations. Shared Rust kernels retain physical inference and scientific admission;
Symbolica/Numerica and class-specific native libraries retain mathematical execution.

Native tables and bounded homogeneous result blocks serve scientific selections alongside
canonical graph relationships. Do not store every dense scalar as a graph node or retain a
complete in-memory bundle beside results. Exact scientific bit/domain codecs are authoritative;
finite numeric projections are derived. Topology, semantic dependencies, incidence, execution
order and numerical sensitivity remain distinct meanings. Their connection supports new
analyses without conflating them.

Linear per-problem revisions and changed membership intervals make edits local. Named guards
cover head, absence and set-membership premises; complete semantic dependencies govern reuse.
Retry the whole decision after conflict. Snapshot transactions do not imply predicate
serializability. Native work and large ingestion stay outside short claim/admission
transactions. Protected source/interpretation selections become retained product roots before
compilation releases protection. Closed private staging followed by atomic admission of its
exact descriptor supplies coherent visibility without a giant completion transaction or late
batch leakage.

Portable descriptions survive restart; native evaluators, Salsa handles and mutable solver
state do not become persisted data. Retain dirty-build attestation at the outer run while
keying products by complete relevant dependencies. Validation reuse carries actual
interpretation; new states, changed context and imports receive their scientific checks.
Correctness testing continues to force validation explicitly.

Remove canonical PG/Delta composition, generated SQL/COPY machinery, mandatory publication,
cross-store settlement and displaced source conversions/caches. Retain exactness,
interpretation, fences, cancellation/drain, recovery, retention, protected long reads, schema
evolution and backup. Retire SQL convenience APIs, including modeling-knowledge SQL; preserve
Arrow export. E2 may retain DataFusion for a necessary remaining capability with an actual
consumer, without a SQL compatibility service or bespoke optimizer replacement.

The benefit is a unified operation route and less repeated/induced machinery. Calling a new
database beneath the unchanged full pipeline would not supply it. Structural removal is
required without a database timing premise. Build/preparation/dispatch/read gains remain
**Proposed** until E4 measures comparable operations.

## WebSocket and persistent environment integration

[Plan 30](30-websocket-and-persistent-agent-environment.md) owns the separately authorized
implementation following the [WebSocket/environment review](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md).
Its [confirmed rules](30-websocket-and-persistent-agent-environment.md#confirmed-rule-changes)
replace the prospective application transport and environment lifecycle/admission target.
Its [dispositions](30-websocket-and-persistent-agent-environment.md#finding-dispositions)
are the sole current WP01–WP08/AE-25/AE-26 status owner; this coordinator retains its
existing scientific/substrate findings and package status.

30a supplies native RPC, original-clock settlement and bounded complete pages to A/C/D/G.
30b supplies separately admitted service/receiver generations; 30c supplies explicit
test contexts and final-outcome disposal; 30d supplies coordinated host admission and
its own new-scope environment qualification. H's exact reference profile, I's receiving
validity, J's pure/fresh-effect separation and canonical guards/acknowledgments remain
required inputs. Their earlier implementation evidence is not a WS qualification.

Future affected assembled substrate qualification consumes the working Plan 30 handoff
at [28e](28e-rebuild-retirement-and-qualification.md#websocket-environment-qualification-handoff).
Plan 30 implementation supplies its own scoped environment qualification and completes
the named failed-only handoff at 28e. It does not restart the former full Python
selection or the broader E4/E5 campaign.

## Rule changes confirmed by the maintainer

Every item below is **Accept, 2026-10-05**. These are operator decisions, not accepted ADR
status. R0 schedules their decision/design route before dependent production changes.
Exact source rules remain in the review's [rule impacts](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rule-impacts).

| Item | Accepted consequence | Route and dependent work |
|---|---|---|
| [RC01](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc01) | Replace D10 canonical PG/Delta owners with SurrealDB. | Superseding ADR for ADR-0114/D10; design amendment to blueprint §20/§20.6 before A/C/D. |
| [RC02](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc02) | Normal runs automatically retain scientific outcomes; explicit ephemeral opt-in. | Durability ADR/design route; blueprint §19/§20/§21.1 and runtime/public contracts before C2. |
| [RC03](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc03) | Preserve D1 declaration authority; replace storage lowerings and retire persistence-only generation. | Metadata/boundary ADR; blueprint §4.1–§4.2/§22.1–§22.2 before A2/D1/E2. |
| [RC04](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc04) | Native structural operations, durable descriptions and shared kernels; Salsa as accelerator. | Compilation/D10 ADR; blueprint §14.3–§14.4/§20.4 before B1/B2. |
| [RC05](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc05) | Complete outer attestation separated from dependency-complete product identity. | Hashing/D14 ADR; blueprint §5.3/§20.4 before B3. |
| [RC06](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc06) | Narrow feature stabilization to actual consumers, preserve family coherence. | Supersede affected ADR-0122 scope if changed; hakari/agent guidance updates through R0/E2. |
| [RC07](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc07) | Scoped/batched study policy and guarded claims preserve occurrence/start/cancel meaning. | Durable claim/commit ADR/design route before C1/C3; policy remains one semantic owner. |
| [RC08](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc08) | Native connected queries and Arrow; retire Runtime/knowledge/TableReader SQL convenience. | Python-boundary ADR and blueprint §21.1 before D2. |
| [RC09](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc09) | Canonical visibility/retention/read protection replaces Delta protocol; clean rebuild supersedes import. | ADR-0114 supersession; blueprint §20.4–§20.6 before A3/C4/E1. |
| [RC10](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#rc10) | Scoped build setup, obsolete generation retirement and unchanged-output preservation. | Ordinary tooling/generator edits in E2; no separate ADR solely for these fixes. |

D5 physical typing, D6 library mathematics, D11 attempt-owned native state and D13 immutable
model/overlay meaning are preserved. D14 dependency completeness is refined, not weakened.
Adding a dependency needs no ADR by itself; crate removal and the decisions above do.

The maintainer also explicitly confirmed **Accept, 2026-10-07** for the production efficiency
review's [RC01](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#rc01):
separate artifact-specific linked identity and actual observed deployment association from
complete dirty outer source/build observation at deployment/run admission. This is a different
item from the 2026-10-05 review's RC01 above. [28h L0/L3](28h-native-setup-and-artifact-identity.md)
routes the hashing/deployment ADR and architecture amendment before dependent implementation.
The existing ADR-0164 is proposed and can be amended through that route; an accepted record
at execution time must instead be superseded. Actual artifact association, complete consumed
inputs and outer provenance are preserved. Operator confirmation does not accept the ADR or
implement the new capture contract.

The maintainer explicitly confirmed **Accept, 2026-10-07** for the remaining-design
enhancement review's [RC01](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#rc01):
allow ordinary persisted preparation replay under independently established exact
deployment-local compatibility, keeping broader producer qualification optional. This is
distinct from both earlier RC01 decisions. [28h L5](28h-native-setup-and-artifact-identity.md#deployment-local-replay-and-build-output-derivation)
supplies the compatibility premises and decision/design amendment; [28b B5](28b-selected-compilation-and-reuse.md#ordinary-deployment-local-replay)
owns reconstruction. Amend proposed ADR-0164 and the governed deployment/replay explanation
with a blueprint revision before dependent implementation; supersede the ADR if its status
has become accepted. Operator acceptance is not ADR acceptance or implemented eligibility.
Missing compatibility refuses preparation reuse, not historical results or ordinary execution.
No strict source/compiler campaign, environment copy, forced rebuild or general change-impact
classifier becomes an ordinary restart prerequisite.

### Parallel-execution rule decisions, 2026-10-08

The maintainer explicitly accepted all four rule impacts from the
[parallel-execution review](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#11-rule-impacts-and-disposition)
on **2026-10-08**, before this plan-authoring extension. These review-qualified decisions
are distinct from every earlier RC01–RC04. Operator acceptance does not change ADR status,
implement a correction or establish concurrent qualification.

| Item | Confirmed decision and consequence | Decision/design route before dependent implementation |
|---|---|---|
| [Parallel RC01](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#rc01) | **Accept, conditional:** allow a changed retention-guard topology only with preserved protection, reclamation, publication and acknowledgment guarantees. Sharing/coalescing within the present contract remains eligible. | A4 first selects the safe correction. Amend proposed ADR-0164 and blueprint §20.4–§20.6 if its named protection/commit contract changes; supersede an accepted record instead. Then B6/T7 consume the working contract. No speculative topology amendment is required before the investigation. |
| [Parallel RC02](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#rc02) | **Accept:** bounded waiting may replace immediate refusal for temporary preparation contention. Preserve original clocks/cancellation and individually oversized refusal. | N8 owns admission behavior and its consumed B6/C6 routes. Amend the enduring blueprint §14.3.2 explanation before dependent behavior where it changes; an implementation correction within existing contracts needs no new ADR. |
| [Parallel RC03](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#rc03) | **Accept:** revise supported backend thread/process strategies and deployment budgets to coordinate case workers, internal teams and process-global constraints. | N10 selects native strategies; L7 supplies the managed deployment envelope; C7 consumes it. Route actual binding/contract changes under R0 before their implementation. Existing typed solver selections remain eligible without a default change by assumption. |
| [Parallel RC04](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#rc04) | **Accept:** replace generic in-turn independent study execution with bounded concurrent sequences or a fitting native batch, preserving dependencies and occurrences. | C6/C7 own ephemeral/durable composition, consuming N8/N9/N10/L7. Amend enduring workflow/execution descriptions where their realization changes; use the ADR route only when the selected design changes a governed contract. |

F04's additional persistent-team accounting implements the existing native-stack admission
contract; N9 needs no rule exception. Material newly discovered rule impacts return for an
explicit decision; these four accepted directions do not need reconfirmation.

## Completion-audit reconciliation and capability decisions

The [completion audit](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md)
assessed commit `06302af7748923100f17a7c8fda9c57b6a536690` with concurrent work and returned
**Revise**. The 2026-10-06 reconciliation below schedules its remaining obligations on the
current tree. The dated review and earlier Outcomes keep their original evidence conditions;
new source controls do not retrospectively turn failed or interrupted runs into passes.

Current source restricts generic retention to deliberate history and requires complete
initialization responses followed by interpretation verification. Native lifecycle,
initialization, selected cursor/staging, lost-ack and protected result-retirement controls pass.
Selected namespace pages and grouped payloads reserve resources before acquisition, complete
membership observations before dependency admission, and settle concurrent identical product
publication without a local shared token. Scalar replay and the modeling suite exercise the
current closure behavior. Scientific fixture and wire premises are corrected without changing
production accuracy defaults or original assertion thresholds. Current installed producer
association, the new long result/analysis control, E3 qualification, E4 measurements and final
E5 acceptance remain at E's owner. The dated audit retains its original failed/interrupted
scope. Preserve concurrent edits and integrate their actual owners.

The repo now selects the shared `neo4j-surrealdb` skill. The
[capability investigation](../design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md)
compares exact 3.3.0 source/probes with current Context7, official documentation and GitHub.
Use bounded direct-ID/bulk acquisition, explicit namespace/database selection, native
versioned structural functions and named transaction guards within the existing owners.
`FOR UPDATE` supplies optimistic conflict registration for named records, including absent
IDs; it does not protect arbitrary predicate ranges. Every writer that can invalidate a
namespace/set premise must touch its stable named guard. Definite conflicts retry the whole
decision; uncertain acknowledgments require identity readback before another transition.

Regular typed relations remain suitable for metadata-bearing lineage. Evaluate INLINE edge
fields only for an actual selected traversal predicate; LIGHTWEIGHT edges cannot carry the
metadata or occurrence identity required by those relations. E4 measures any claimed benefit.
Awaited bounded SDK queries remain valid; no mandatory streaming rewrite is introduced.
Live queries/changefeeds and Surrealist may support a concrete observer or diagnosis, but do
not become lifecycle or qualification authorities. SurrealKit is not introduced: independently
authored desired-state schema would compete with registry generation and the clean rebuild.
Logical export/import is not a crash-backup substitute; A/E retain quiesced offline backup and
selected-profile restore validation. No dependency upgrade, Neo4j adoption or new migration,
scheduler, planner or telemetry framework is required by this investigation.

**Rule impacts:** None for this corrective scope, matching the completion audit. The original
accepted RC01–RC10 and their R0 adoption route remain in force. New evidence that requires a
material rule change returns to that route before dependent work; this revision does not
accept ADR-0164.

The next executable order is A's lifecycle/completion controls and E's valid inputs/producer
prerequisites; A/B grouped guarded acquisition; C/D composed recovery and stream/read controls;
then E3, E4 and E5. Independent targeted slices may proceed together with explicit ownership.
Only 28e owns assembled execution state; this coordinator owns all US/EF/F/PE dispositions.

## Repository-wide efficiency extension

The maintainer expanded the 2026-10-07 review follow-up to all comparable defect instances
throughout current functionality. Address causal patterns, not only the original tests or
example consumers: repeated identity joins, unchanged preparation/admission, mismatched physical
units, excessive crossings/materialization, repeated native assurance and broad invalidation.
Unrelated feature development, arbitrary solver replacement and a general optimizer/cache/
telemetry framework are not implied. Production contextual accuracy, test scheduling/timeouts,
server profile, exact codec/replay and independent scientific assessment remain unchanged.

The product baseline is `5260a3e9a3cecd69b6358ab86d4e917ae2508b29`, preserving the dirty review
and plan documents. Focused source assessment covered the owners below, reusing the review's
exact-release evidence. PE01–PE06 retain their original diagnosis/evidence limits. Additional
preflight and trajectory lookup amplification is source-established; other matches remain
specific investigation candidates. Neither the interrupted timings nor a textual search proves
their speed contribution or absence elsewhere.

### Capability coverage and migration ownership

| Current capability / inspected owners | Relevant pattern and assessment | Implementation route |
|---|---|---|
| Numerical policy/engineering accuracy; `pse-math` and native quality | PE01; per-goal target resolution adds the same kind/ID join. Preserve global/contextual defaults, exact precedence and frozen scales. | N0/N1 in 28f |
| Mathematical assembly/composite reconstruction; math/compiler modeling | Existing typed row/column maps and request deduplication are reusable. Repeated supplier correspondence and prior-instance searches need exact key/lifetime decisions. | N0/N1/N3 |
| Structural preflight/solver routing; `pse-structural`, native structural/runner | Preflight scans assessment rows per contract row after inventory-set checks. Small finite capability dispatch is justified work. Preserve contribution provenance and independent original-space checks. | N1; applicable N3 native consumers |
| Unit/property/thermodynamic mathematics; physical operations, compiler specialization, runtime math | Shared admitted bodies/artifacts are foundations; state/composition-dependent property values remain fresh. No production FeOS-provider migration. Check extension callers for repeated preparation, not presumed identical property results. | N0/N2/N4 |
| Dynamics/events/trajectory/shooting/recycle; runtime and native backends | Sample×output trajectory metadata lookup is established; dynamic goal/control/node mapping and compatible factor/session storage are candidates. Time/event/schedule changes retain their own work. | N0/N1/N3/N4; T1 publication |
| Fitting/sensitivity/uncertainty; runtime fitting and response owners | Existing sparse contribution/Gram plans are strengths. Covariance parameter joins, repeated pinned fits and fresh native owners need bounded applicability decisions. | N0/N1/N3/N4 |
| Ordinary solves, initialization, continuation/studies/workers; modeling and staged runtime | PE04 repeats selected hydration/generic admission for ready occurrences; physical cache and scoped native sessions already retain useful work. Distinct starts/claims/attempts remain distinct. | N2/N3/N4; C3 integration |
| Authored document edits and physical source publication/reopen; authoring driver and workflow worker | Edited bundle copies unchanged bytes; source chunks cause serial revisions and singleton reopen chains. Parser-owner reuse/grouped selected acquisition are existing alternatives. | T0/T3/T4/T5 |
| Relations/columnar/engine | Checked buffers, prepared validation owners, multiplicity-aware gathers and selective streaming are foundations. Assess actual unnecessary collection; shallow clones and required owner-change validation remain justified. | T0/T5; N2 where admission is genuinely repeated |
| Progress/results/query/analysis/retention/export; operations/runtime | PE02/PE03; analysis node/edge loops and recovery metadata are candidate groups. Preserve page completeness, renewable protection, endpoint ordering and activation. | T0–T5; A/C/D integration |
| Rust/Python interfaces and worker processes | Typed ingress and Arrow C streams are strengths. Mechanisms migrate at Rust owners; wrappers preserve actual completion, schema, role and allocation lifetimes. | N4/T5/L4; D integration |
| Native providers, build/generation, producer/deployment/validation tooling; scripts/buildinfo/xtask | PE05/PE06, nested HiGHS discovery and related installation/capture consumers. No-op generated output handling already exists and must be retained. | L0–L4; B3/E2 integration |

This table defines functional coverage and work routing, not a source-proof manifest or another
finding-status ledger. At each N0/T0/L0 investigation, follow actual callers and first-party
generator/registry sources beyond the starting pointers. Classify matches as confirmed,
candidate or justified necessary work; finish unexamined variants that can change the design.
Do not stop at the illustrative consumers or demand an exhaustive graph of every function.
Every newly confirmed comparable instance joins the owning migration package in this series.
A distinct material defect receives a coordinator finding/decision route rather than being
silently included in a local rewrite. No supported confirmed variant is deferred without an
explicit disposition and observable trigger at the existing owner.

### Common target and foundation decisions

Use one authoritative set of mechanics per causal family, customized through its actual domain
owner: ordered projections, checked immutable preparation, byte/extent admission, sufficient
effect acknowledgments/protected grouped reads, verified installation lifetime and artifact
association. Different identity spaces, scientific policies and effect/recovery obligations
remain explicit. Share operation-shaped products and existing library capabilities, not copied
end-to-end workflows or a universal new abstraction.

28f owns numerical projection/reuse details; B owns selected scientific closure and portable
eligibility. 28g owns physical admission/crossing corrections; A/C/D retain revision, attempt,
visibility and lineage meaning. 28h owns setup/provenance corrections; B3/E2 integrate their
actual producer and deployment consumers. Each package moves producers, consumers and generated
declarations together, then deletes displaced mechanisms/callers/tests/fixtures when its focused
controls pass. A second production path is not a temporary completion strategy.

Conditional Taylor-zero, numeric factor/scratch, Salsa/native-session and query/provider choices
are bounded N0/T0 decisions with specified evidence and dependent work. Inspect source/contracts
first; perform a small observation only when it chooses between consequential alternatives.
Unused library features do not mandate integration, and necessary assurance is not removed
because a search resembles repetition. Configurable policies have one declared default and
explicit consumed override; neither tests nor individual workflows invent new defaults.

### Execution and completion of the extension

L0 records the accepted identity route and installation contract before L1/L3. N0/T0 resolve
their relevant maps/physical units; established N1 projections, T2 acknowledgments and T3 result
reads need not wait for speculative library/index choices. N2 immutable admission, T1 ingestion
and L1 scoped setup can proceed independently with explicit shared-file ownership. N3/T4/L2/L3
consume their actual prerequisite slices. N4/T5/L4 reconcile every functional row and actual
caller; they cannot postpone migration/deletion needed by an earlier package.

E3 starts after all adopted functional extension work and existing A–E obligations are complete.
E4 measures the appropriate preparation/build/numerical/persistence operations; E5 assesses the
assembled supported target and resolves findings with evidence. Keep all current open scientific
obligations transferred to E and their original disposition owners. This authoring work neither
resumes the interrupted campaign nor claims a speedup or scientific acceptance.

## Remaining-design enhancement integration

The [2026-10-07 enhancement review](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md)
assesses implemented supporting design and remaining target independently, then reconciles
them. Its four supporting corrections and accepted replay target fit B/C/E/N/T/L; no new
companion is needed. Its review-qualified F01–F05 are different from the completion audit's
F01–F05. This coordinator owns their dispositions, including the opportunities below.

The enhancement review examined the foundation at `dacc9c33225990984ddbd1d356e797389f02fef3`, with checked selected
admission, bounded operations, retained native owners and immutable installations. Stored-study
creation still rebuilds declared admission per point; dynamic workers rebuild program-fixed
layouts; IDAS jump elimination repeats factorization of the same sample matrix; sealed receipt
permissions leak into Cargo outputs. Default restart preparation replay still needs a qualified
producer. Document adoption changes none of those implementation facts.

Retain different valid lifetimes: C5's study-scoped declared basis, B5's protected deployment
reconstruction, N5's program-fixed layout and N6's sample-local factor. Mutable values,
providers, guards, candidate permission, seeds and attempt fences remain independently owned.
These remedies must not grow a second cache, reconstruction checker or numerical algorithm.

| Prerequisite slice | Delivered contract and dependent work |
|---|---|
| 28h L6 | Exact sealed receipt bytes become writable Cargo-owned output, including repeated/read-only destinations; restores the affected native validation route without a provider rebuild. |
| 28c C5 with N2/N4 | Stored-definition admission shares a compatible basis but validates every binding/seed role; caller cancellation reaches creation and settles ambiguous activation. Actual default durable/Python study routes are required consumers. |
| 28h L5 then 28b B5 | Record RC01's decision route, establish independently observed deployment compatibility and implement ordinary reconstruction through the existing protected scientific checker. Closure uncertainty blocks only B5, not ordinary fresh execution or other corrections. |
| 28f N5/N6 | Retain immutable dynamic layouts and sample-local factors; instantiate/refactor current numerical state under existing limits. N4 reconciles dynamics, fitting and shooting consumers. |
| 28f N7; 28g T6 | Settle fitting demand/cache upgrade and selected query access paths. Each ends in an adopted correction or reasoned retained design with owner, applicability and observable reopen trigger. Required corrections join N4/T5; speculative evaluator/history/streaming replacements do not. |
| E1/E2 then E3/E4/E5 | Obtain affected failure diagnostics, complete functional corrections and migrated consumers, compose local correctness/recovery, measure applicable operations and assess closure. Previous positive evidence keeps its original scope. |

Library conclusions from the review are reusable evidence, not transferred performance proof.
Source establishes layout/factor repetition and a direct combined fitting capability; it does
not establish the unfinished run's dominant phase. Symbolica interpreter cloning, compiled
evaluator trust/ABI, native checkpoint continuation and storage streaming remain different
questions. Additional Context7/exact-release research or bounded observations are needed only
for an unsettled consequential contract, not a feature census or another full design review.

## Parallel-execution integration

**Proposed target, 2026-10-08:** reliable sixteen-worker independent end-to-end execution,
including ordinary ephemeral and durable studies, under a finite coordinated deployment.
The authoring baseline is committed `84a1caf17656f00f38b22e703c7cbc2b63a44d2d`; the
review examined its `fbbf718`-based predecessor working tree. Its existing two-test native
selection failed on the composed canonical route. No new product run is supplied by this
planning extension, and the reference campaign remains stopped.

Keep four different kinds of concurrency explicit: Cargo/build jobs; Nextest/Python test
workers; application case workers; and library/internal native teams. Sixteen configured
test processes do not establish sixteen application cases. The target allows independent
ready cases to progress concurrently where their operations permit it; it does not demand
sixteen maximum-sized allocations or sixteen simultaneous process-constrained native calls.
Native constraints require an examined supported execution strategy, not silently reduced
application width. L7 must reconcile host/effective process limits, the capped command scope,
server/native child allocations and test placement rather than summing independent defaults.

Preserve the implemented foundations: authored scientific authority and physical interpretation;
shared immutable selected admissions/artifacts; private compiler generations, evaluator/provider
workers and native mutable state; original-model assessment; exact occurrence/start lineage;
bounded progress/result transport; protected history; and native cancellation followed by drain.
Salsa workspace mutexes are private. A transport change or another executor alone does not
resolve shared store guards, admission demand or single-sequence study dispatch.

### Packages, readiness and investigation decisions

The existing companions own both their local design and package progress. This coordinator
owns cross-plan dependencies and the finding ledger. No new companion, generic scheduler,
second retry authority or resource-estimation framework is required by this extension.

| Package/owner | Delivered capability or decision | Dependent work and readiness |
|---|---|---|
| [A4](28a-canonical-substrate-and-revisions.md#parallel-canonical-protection-and-contention) | Attribute decisive store failures where needed; select and implement the smallest safe protection/publication correction. | A bounded decision supplies exact predicates, writer coverage and acknowledgment semantics. B6/T7 require its working correction where their consumed contract changes, not merely an agreed schema. |
| [B6](28b-selected-compilation-and-reuse.md#parallel-selected-preparation-composition) and [T7](28g-bulk-data-operations.md#parallel-protected-operation-composition) | Compose protected selection, immutable preparation and publication without unnecessary equivalent work/crossings. | Consume applicable A4 slices and N8 admission. Preserve distinct selections, roots and effect receipts; demonstrate actual ordinary callers. |
| [N8/N9](28f-shared-numerical-preparation.md#parallel-admission-and-native-lifetimes) | Demand-based temporary admission with bounded waiting; persistent native-team extent owned through teardown. | These corrections can proceed independently of store attribution. B6/C6 consume their tested lifetimes and refusal/cancellation behavior. |
| [N10](28f-shared-numerical-preparation.md#parallel-admission-and-native-lifetimes) then [L7](28h-native-setup-and-artifact-identity.md#parallel-deployment-and-supported-native-placement) | Select backend coexistence/exclusion/team strategies, then a realizable aggregate local deployment and test placement. | N10's strategy selection can run alongside A4/N8/N9. L7 fixes profile values before C7 deployment; no fresh full budget per child process. |
| [C6/C7](28c-durable-execution-and-studies.md#parallel-study-frontiers-and-durable-workers) | Concurrent independent study sequences and adequately deployed durable workers, preserving genuine chain dependencies and native bulk semantics. | Targeted integration needs working protection/admission/native slices and L7 for managed processes. Source-policy design can start earlier; no entire-document barrier or separate policy authority. |
| D1/D2 and N4/T5/L4 reconciliation | Migrate affected result/analysis and scientific/tooling consumers and delete displaced mechanisms with their replacement controls. | Extend the existing closure responsibilities to B6/C6/C7/N8–N10/T7/L7. Prior closure excludes these new consumers until reconciled. |
| [E3/E4/E5](28e-rebuild-retirement-and-qualification.md#parallel-extension-acceptance-and-campaign-restart) | Composed correctness/recovery, applicable measurements and assembled assessment. | All selected functional scope and targeted controls precede campaign restart; previous passes, reader-only stress and document adoption are insufficient. |

A4 and N10 investigations finish with a selected supported design, its evidence limits and any
actual decision/design amendment. A reasoned retained mechanism can close an investigation;
it cannot close F01/F03 if the required complete operation still fails or its native strategy
is unresolved. Missing backend safety/feature premises constrain that combination, not unrelated
corrections. Quantitative gains remain Proposed until E4 measures comparable operations.

The review's library matrix and alternatives guide N10/L7. Refresh pinned source and Context7
only for consequential unknowns. Distinguish Symbolica optimizer cores from numeric workers,
faer global/team behavior from explicit sequential callers, Tokio orchestration from admitted
CPU work, Rayon teams from native process constraints, and DataFusion pool reservation from
physical allocation enforcement. Library-owned execution is eligible if it removes supervision
machinery while preserving thread affinity, retained ownership, cancellation and join.

A new provider or model composes through existing scientific admission and private workers.
A backend/factorization substitution changes its native strategy and acceptance conditions,
not study policy. These variation axes must remain local; no compatibility production path is
retained after the replacement's callers and targeted controls are complete.

## Execution sequence and readiness

**R0 — Record the approved target through the decision/design route.** Allocate proposal(s)
with the ADR skill, covering canonical authority, schema/metadata, identity, commit and Python
contracts plus crate retirement. Do not preassign IDs or edit accepted records in place.
Link the accepted source review and obtain the binding's focused review of material
rebuild/API/identity departures where required before ADR acceptance. Amend affected
architecture owners with a blueprint revision row through the decision/design change.
R0 records governing contracts, not implementation acceptance.

Then implement actual slices of the target:

| Useful order | Required working capability | Work unblocked |
|---|---|---|
| A1 then A2 early slice | Supervised store, exact codec, immutable identities, named guards and preparation protection/root admission. | B1, C1, D1 declarations and E1 regeneration. B1 need not wait for complete A3 authoring. |
| B1 then B2; C1 then D1 | Shared kernel, persisted descriptions with working ordinary-solve reconstruction; terminal envelope and actual minimal reader. | C2 durable execution. Agreement on types alone is not this prerequisite. |
| A3, B3, C2, D2 | Runnable revision edits, protected reads, numerical products, sealed runs and native/Arrow consumers. | C3 studies and D3 analysis; A3/C4/D2 jointly complete retention/read lifecycle. |
| C3/C4, D3 and B4 decision, plus the parallel extension | Full study/recovery/analysis consumers, decided evaluator layout and working A4/B6/T7/N8–N10/C6/C7/L7 with affected consumer closure. | E3 after E1/E2 retirement and [parallel readiness](28e-rebuild-retirement-and-qualification.md#parallel-extension-acceptance-and-campaign-restart) complete. |
| E3 then E4 then E5 | Required correctness/recovery/static evidence, measurements and assembled assessment. | Closure at enduring architecture and finding owners. |

B4's library-signature investigation and E2's build/codegen fixes can proceed independently
of server readiness. EF08 does not block A/C/D. E2 joins migration continuously, rather than
delaying all deletion. Discovery is not claim authority, description design is not a working
compiler, and schemas are not automatically durable runs.

```mermaid
flowchart LR
  R0[Recorded decisions] --> A[Store / codec / guards]
  A --> B[Selected kernel / descriptions]
  A --> C[Lifecycle envelope]
  C --> D[Minimal result reader]
  B --> Run[Durable runs]
  D --> Run
  Run --> Study[Studies / recovery]
  Run --> Query[Connected query / analysis]
  A --> Retain[Authoring / retention]
  Retain --> Study
  Retain --> Query
  Study --> Q[Assembled qualification]
  Query --> Q
  Retire[Regeneration / retirement / tooling] --> Q
  B --> N[28f numerical preparation / consumer closure]
  Run --> T[28g bulk operations / consumer closure]
  R0 --> L[28h setup / artifact association / consumer closure]
  N --> Q
  T --> Q
  L --> Q
```

Root owns design and integration. Delegate independent consumers/research with explicit file
ownership when useful. Coordinate shared editing surfaces separately from logical dependencies;
preserve concurrent work. Package checks prove new mechanisms and trigger immediate legacy
deletion; static hygiene and assembled integration wait for all functional scope.

## Finding dispositions

This is the only current disposition table for the adopted unified-substrate and efficiency
findings, the completion audit and the [2026-10-07 production efficiency review](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md).
It also owns the [remaining-design enhancement review](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md).
The [parallel-execution review](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md)
has the open dispositions below; its recommendations and conditional rule impacts are not
adopted merely by publication.
A scheduled package or accepted rule is not resolution. Links identify local evidence owners;
resolve only when correction and relevant acceptance actually exist. US/EF/F scenario IDs
refer to their originating reviews; PE scenario IDs refer to the new review. The PE entries
now schedule the documented repository-wide extension; authoring is not implementation or
correction evidence and does not change scientific acceptance criteria.

| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
|---|---|---|---|---|
| [US01](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us01) | S01/S03/S04/S07 | Scheduled | A/C/D; E2/E3 | Canonical connected operations, old-store retirement and coherent visibility after restart. |
| [US02](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us02) | S03/S04/S07 | Scheduled | C2/C4; D1/D2 | Ordinary success/failure/partial scientific retention, beyond completion records. |
| [US03](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us03) | S04/S06/S07 | Scheduled | A2; D1; E3 | Independently specified exact codec/protocol/Arrow controls. |
| [US04](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us04) | S02/S03/S07 | Scheduled | A2/A3; C1/C3/C4 | Named absent/set premises, whole-decision retry, generations and retention races. |
| [US05](../design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us05) | S01/S02/S05/S08 | Scheduled | B1/B2/B3 | Selected sufficient scientific inputs and complete portable dependencies. |
| [EF01](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef01) | S01/S02/S07 | Scheduled | A3/B1; E3 | Active owner releases complete superseded bundles; deliberate durable history remains. |
| [EF02](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef02) | S01/S02/S08 | Resolved | B1/B2 | [B Outcome](28b-selected-compilation-and-reuse.md#outcome-recorded-after-implementation): actual native-owner retention, same-owner reuse and changed-context/value controls; force-validated relation/engine/document selections passed 22/26/6 controls. |
| [EF03](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef03) | S03/S07 | Scheduled | C3; E4 | Scoped/batched dispatch and ready-attempt preparation. |
| [EF04](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef04) | S06/S08 | Scheduled | E2 | Dependency removal, narrowed unification and isolated client closure. |
| [EF05](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef05) | S01/S02/S06 | Scheduled | B3; 28h L0/L3 | Relevant complete keys, actual role/artifact association and full dirty outer observation under confirmed production RC01. |
| [EF06](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef06) | S06/S08 | Scheduled | E2/E4; 28h L1/L2 | Actual target capability closure before native setup, scoped verified installations and comparable build/setup observations. |
| [EF07](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef07) | S05/S06 | Scheduled | A2/E2 | Obsolete outputs retired; unchanged output mtimes stable; stale outputs deleted. |
| [EF08](../design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef08) | S01/S07 | Resolved | B4 | [B Outcome](28b-selected-compilation-and-reuse.md#outcome-recorded-after-implementation): actual evaluator accepts selected compact formals; migrated consumers and 26 native implicit controls preserve guards, providers and ordered derivative axes. No measured speedup is inferred. |
| [F01](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md#f01) | S01/S03/S07 | Resolved | A3; C2/C4 | [A Outcome](28a-canonical-substrate-and-revisions.md#outcome-recorded-after-implementation): native generic lifecycle-root refusal, history mutation and protected reclamation controls pass; enclosing E3 remains separate. |
| [F02](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md#f02) | S01/S02/S08 | Scheduled | A2; B1; E4 | Exact pinned namespace cursors, pre-RPC reservations, grouped manifests/payloads and concurrent acknowledgment settlement have positive targeted controls at [E Outcome](28e-rebuild-retirement-and-qualification.md#outcome-recorded-after-implementation). Complete-operation measurements remain E4. |
| [F03](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md#f03) | S03/S07 | Resolved | A1 | [A Outcome](28a-canonical-substrate-and-revisions.md#outcome-recorded-after-implementation): zero-result, lost-ack, cancellation, normal/repeated creation and marker refusal controls pass. They do not establish the cause of historical missing-table failures. |
| [F04](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md#f04) | S01/S03/S04/S07 | Scheduled | E1/E2; B3; C/D; E3 | Validate corrected fixture/wire premises, producer mechanism controls, progress and intended-point scientific controls, then complete the stable assembled development campaign against zero under [E's evidence policy](28e-rebuild-retirement-and-qualification.md#development-phase-evidence-and-optional-artifact-requalification). |
| [F05](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md#f05) | S06/S08 | Scheduled | B3; E3/E4/E5 | Development functional campaign, all applicable prepared measurements, accepted assembled review and enduring-owner retirement route; actual installed strict deployment qualification is explicit optional scope under [E's evidence policy](28e-rebuild-retirement-and-qualification.md#development-phase-evidence-and-optional-artifact-requalification). |
| [PE01](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#pe01) | S02 | In progress | 28f N1/N4; numerical policy/quality consumers | Shared ordered kind/ID projection and confirmed related maps; frozen scales, allowances, attribution and refusal controls. No tolerance changes. |
| [PE02](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#pe02) | S03/S05 | In progress | 28g T0/T1/T5; C/D | Payload/index/resource admission and generated finite replay bounds replace scalar-count-driven framing. Exact rows/indexes, coverage, atomic visibility and complete-route E4 observations remain. |
| [PE03](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#pe03) | S03/S05/S10 | In progress | 28g T2/T3/T5; C/D | Compact sufficient append acknowledgment and composed protected acquisition; distinct replay/read completion, expiry and cancellation controls. |
| [PE04](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#pe04) | S01/S04/S10 | In progress | 28f N0/N2/N4; B/C3 | Bounded immutable checked selected preparation, complete invalidation and fresh protection; all confirmed repeat consumers migrate without merging occurrences or attempt state. |
| [PE05](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#pe05) | S06 | In progress | 28h L0/L1/L2/L4; E2 | Operation-scoped verified generation and actual capability setup; corruption/provider/input and concurrent-use/drain controls. Refines EF06 without claiming a measured bottleneck. |
| [PE06](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#pe06) | S07 | In progress | 28h L0/L3/L4; B3/E2 | RC01 confirmed 2026-10-07; actual artifact-specific association plus complete outer observation, relevant/unrelated edit and genuine installed-role controls. |

The efficiency review's conditional query-plan, Taylor-zero, numeric-scratch and related
library options have bounded N0/T0 decision routes; applicability precedes dependent changes.
Its RC01 is explicitly confirmed above. Scheduling documents is not correction evidence or
production execution. Prior US/EF/F dispositions and historical scientific evidence remain
at their existing owners; none is resolved by the extension or substituted for E3/E4/E5.

### Enhancement-review dispositions

The labels below qualify the originating review; they do not rename its finding IDs or the
earlier completion audit. All corrections are scheduled, not resolved by document adoption.

| Finding or obligation | Review scenario | Disposition | Work owner | Required evidence or settling decision |
|---|---|---|---|---|
| [Enhancement F01](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f01) | S01/S06 | Implemented; focused controls pass, E3/E4 pending | C5; N2/N4; E3 | Actual stored/default study basis reuse, every binding/seed role checked, distinct occurrences, changed premises and creation cancellation. |
| [Enhancement F02](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f02) | S02 | Implemented; scoped controls pass, E3/E4 pending | L5; B5; E3 | Accepted RC01 route, independently established effective deployment context and actual default worker/Python restart reconstruction; absent/changed premises refuse reuse only. |
| [Enhancement F03](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f03) | S03 | Implemented and targeted-tested | L6; E2 | Repeated exact receipt materialization, changed bytes and existing read-only destination; sealed generation stays unchanged; affected native route executes. |
| [Enhancement F04](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f04) | S04 | Implemented; focused factor control passes, E3/E4 pending | N6; N4; E3 | One current sample factor for distinct directions, current-value refactorization, first/second-order production-basis checks and failed-factor recovery. |
| [Enhancement F05](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f05) | S05 | Implemented; focused integration controls pass, E3/E4 pending | N5; N4; E3 | Shared admitted layout, private concurrent scratch, changed coordinate/support premises and extent/refusal controls across actual consumers. |
| [Transient fitting demand](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#library-fit-and-remaining-investigations) | S05 | Adopted and implemented; demand/coherence controls pass, E3/E4 pending | N7; N4 | Gradient-first combined report/derivative, cheap objective-only route, coherent value-first cache upgrade; no assumed checkpoint continuation. |
| [Selected storage access paths](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#library-fit-and-remaining-investigations) | S07 | Investigation complete; cursor correction targeted-tested, E3/E4 pending | T6; T5; D1/D2 | Current bound source/result/analysis queries under selective/empty/skewed inputs; adopted correction or supported retained design, preserving protected byte-bounded completion. |

Other preparation work is included in B3/C3: ready-attempt preparation/product sharing,
bounded Salsa diversity and native lifetimes. General SIMD/JIT/WASM, distributed deployment
and optional generic tools are outside required scope unless a concrete consumer need changes
the contract. Material new decisions return to their owner through existing governance.

### Parallel-execution review dispositions

The [2026-10-08 review](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md)
assesses reliable sixteen-worker independent end-to-end execution on the current tree and
returns **Revise**. It preserves scientific authority, immutable preparation sharing, private
compiler/evaluator/provider state, original-model assessment and native drain. The source-backed
findings have distinct correction obligations; a higher worker count, retry count or budget
does not settle them. The precise SQL responsible for each observed transaction conflict and
the supported mixed-backend execution strategy remain consequential questions.

| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
|---|---|---|---|---|
| [Parallel F01](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#f01) | S01/S03/S06 | In progress; focused managed Rust controls pass, Python/assembled E3 acceptance pending | A4; B6; T7; E3 | Reliable sixteen complete cases and original report/result meaning; protected reclamation and uncertain-acknowledgment safety. Select the correction with adequate operation attribution; RC01 is accepted conditionally; A4 selects the actual route. |
| [Parallel F02](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#f02) | S02/S03 | In progress; focused managed Rust controls pass, Python/assembled E3 acceptance pending | N8; B6; E3 | Small nested-provider/rebind work completes under the finite pool without maximum-capacity reservation amplification; retain oversized refusal, clocks, cancellation and bounded progress under retained pressure. RC02 is accepted; N8 owns the changed behavior. |
| [Parallel F03](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#f03) | S04 | In progress; focused managed Rust controls pass, Python/assembled E3 acceptance pending | N10; L7; C7; E3 | Explicit supported independent-instance, internal-team and process strategies, with exclusion/waiting, cancellation, teardown and aggregate CPU/memory scope. RC03 is accepted; no deadlock or universal backend concurrency is claimed. |
| [Parallel F04](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#f04) | S05 | In progress; focused managed Rust controls pass, Python/assembled E3 acceptance pending | N9; C6/C7; E3 | Additional POUNCE team stacks reserved before creation, retained while the team exists and released after scope teardown/join; finite refusal before creation. This is existing-contract accounting, not an RSS claim or a new allocator contract. |
| [Parallel F05](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#f05) | S07/S03 | In progress; focused managed Rust controls pass, Python/assembled E3 acceptance pending | C6/C7; N8–N10; L7; E3 | One ordinary study executes sixteen independent non-batching cases concurrently with exact occurrences, starts, result placement and original scientific checks; retain dependent sequence order, failed-point isolation, bounded admission and cancel/drain. Select/account for durable worker deployment; RC04 is accepted; C6/C7 own the realization. |

The selected A4 staging correction, N8 demand admission, N9 team accounting, N10 exclusion
coordination and C6/C7 dispatchers are integrated. Their targeted controls establish only
their exercised mechanisms; public managed receiving journeys and E3/E4/E5 remain required.
The [confirmed decisions](#parallel-execution-rule-decisions-2026-10-08) replace the earlier
unconfirmed handoff. Selected contract changes still require their decision/design route;
no accepted ADR or architecture section is changed by plan publication. This coordinator
keeps the single finding ledger; companions own package progress and
[28e](28e-rebuild-retirement-and-qualification.md#parallel-extension-acceptance-and-campaign-restart)
owns campaign restart and later qualification evidence.

## Preparation assurance and reuse review

The [2026-10-08 preparation review](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md)
returns **Revise** for complete preparation operations across ordinary solves, studies,
initialization, dynamics, fitting, restart and interruption. Its
[supporting evidence](../design_review/evidence/preparation-assurance-and-reuse-2026-10-08.md)
separates the live frozen Python extension from the evolving source and records the bounded
CPU/file-read observations. Existing native setup's once-per-operation admission and exact
canonical acknowledgment reuse are strengths; neither proves complete preparation reuse.

The maintainer accepted RC01–RC04 and extending this series on **2026-10-08** during
plan creation. [28i](28i-runtime-validity-and-interruption.md) develops receiving validity
and interruption; [28j](28j-pure-preparation-and-publication.md) develops pure preparation,
explicit publication and reusable descriptions. Existing A/B/C/F companions integrate
their actual canonical/selected/workflow/numerical consumers; H retains native setup and
E retains qualification. This coordinator owns the current dispositions below. Document
adoption schedules decisions and packages, not production implementation or acceptance.

| Finding | Disposition | Prospective owner | Required resolution |
|---|---|---|---|
| [PA01](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#pa01): runtime observation at body boundaries | Implemented; scoped qualification pending | 28i I0/I1; 28b B7; 28h root integration | Establish actual consumed premises and the supported validity lifetime; preserve trust-transition checks and refuse unknown replay contexts. |
| [PA02](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#pa02): repeated compiler/preparation composition | Implemented; scoped qualification pending | 28j J0/J2; B7/C8/N11 | Reuse complete immutable preparation with fresh case meaning and effects; structural, demand, provider and physical changes remain consequential. |
| [PA03](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#pa03): memoized mathematics carries publication effects | Implemented; scoped qualification pending | 28j J0/J1/J2; 28a A5; 28b B7 | Separate pure computation from protected dependency/root publication; preserve exact rooted acknowledgment reuse and expiry/recovery while removing repeated encoding. |
| [PA04](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#pa04): observation/cohort interruption gap | Implemented; drain qualification pending | 28i I2; runtime jobs / 28c C8 drain | Make owned capture and waits interruptible while retaining charges through drain and preserving other live consumers; state foreign lock-wait limits honestly. |
| U01: receiving-runtime premise coverage | Settled for controlled worker/imported-Python roots; arbitrary embedding remains unqualified | 28i I0 with actual receiving roots / 28h / B7 | Trace reconstruction and subsequent product-use inputs and actual protection; consider independently valid immutable products before requiring a whole-host freeze. |
| U02: corrective scientific composition | Unresolved | 28e with affected scientific owners | Qualify the implemented composition at its real scientific boundaries; plan adoption supplies no new product pass or speedup. |

The review preserves the earlier findings' scope and dispositions. It establishes related
remaining obligations rather than reopening every historical review. 28e remains the sole
assembled qualification owner; no new full campaign is authorized by this plan-authoring handoff.

### Preparation rule decisions, 2026-10-08

The maintainer explicitly accepted all four items below and selected two new companions
within Plan 28. These are operator target decisions. Accepted ADR status and production
implementation remain separate; no authoritative section or ADR is changed by this publication.

| Review item | Decision and consequence | Route before dependent implementation |
|---|---|---|
| [Preparation RC01](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#rc01) | Accepted: reuse runtime validity at a demonstrably owned lifetime; retain full trust-transition checks and fresh preparation for unowned premises. I0 must establish the supported-root argument before broader reuse. | Amend proposed ADR-0164 and blueprint §14.3–§14.4 through R0's decision/design route once I0 selects the actual profile. Accepted historical records remain immutable. |
| [Preparation RC02](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#rc02) | Accepted: separate pure preparation from current protection/publication effects; complete immutable basis reuse is the default, with bounded library incrementality eligible for a concrete gap. | J0 settles identity, fresh attribution and description/effect contracts; record the operation/effect and lifetime amendment in the relevant mathematics owner through the decision/design route before J1/J2. |
| [Preparation RC03](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#rc03) | Accepted: reuse encoded descriptions and compatible grouped effects while preserving exact dependencies, roots and uncertainty recovery. Existing exact rooted acknowledgment settlement need not create a row/root for every current selection. | A5/J0 decide whether existing identity/metadata/commit contracts suffice. Optimization within them needs its ordinary owner; a changed contract requires its decision/design amendment before implementation. |
| [Preparation RC04](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md#rc04) | Accepted: observations and waits consume actual stop/original-clock control, with per-consumer cancellation and charges through real drain. No solver stop or historical-accuracy policy change. | I2 updates the owning APIs/jobs with C8/H; follow the lifecycle decision route only if the accepted guarantee changes. |

I0 examines trust-transition qualification followed by independently valid immutable
mathematics before selecting wider operation/generation protection. The existing installation
generation does not own the whole receiving context. J0 examines complete basis/attribution
and canonical description discovery before reusing current products wholesale. A pure memo
must expose every required description even when its query does not run. Exact committed
acknowledgment recovery after protection expiry remains distinct from new admission after expiry.

### Preparation execution dependencies

The first executable investigations are I0 and J0; they can proceed independently and
receive focused evidence/advice without another whole-system review. I2's interruption
design can proceed against existing controls while I0 settles lifetime ownership; integrate
it with the selected owner rather than retaining two observation protocols. R0 records the
necessary selected contract amendments before their dependent implementation.

| Working prerequisite | Enables | Boundary that remains open |
|---|---|---|
| I0 supported receiving profile and protection/product-independence argument | I1 validity integration and B7 receiving reuse | Unsupported roots retain fresh preparation; upstream loader facts alone do not close U01. |
| J0 complete basis, fresh attribution and description/effect contract | A5 canonical integration and J1 pure/effect separation | A settled design is not a working admission/publication API. |
| A5 working canonical slice plus J1 explicit effects | J2 basis/encoding retention and B7 adoption | A memo hit cannot skip exact current effect settlement; retention is not permission. |
| J2/B7 working compatible basis and applicable I1/I2 receiving controls | C8 ordinary study/worker consumers and N11 analysis/mode/experiment consumers | Changed structure/demand/provider/physical meaning still requires affected preparation; mutable numerical state stays private. |
| Consumer migrations and targeted controls | 28e's affected assembled acceptance/U02 route | Quantitative benefit requires measurement; current failure-only continuation does not authorize another full campaign. |

Shared edit surfaces in compiler/workspace, runtime math/workflow and canonical selection
need one integration owner during execution, independently of logical package readiness.
Existing bounded retention, flights, allocation owners, native generation pins and original
scientific checks are foundations. No new global compiler, generic cache, solver controller,
store pivot, mandatory benchmark matrix or permanent assurance framework is selected.

## Graph and hashing review integration

The [2026-10-09 graph compilation, kernels and hashing review](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md)
assesses further graph preparation, fast hashing independently and their combined reuse role.
The maintainer selected bounded investigations alongside planning the concrete corrections.
This integration is **Proposed**: it supplies plan documents, not authorization to implement
production changes or restart the narrowed qualification continuation. The review retains
its Revise verdict and source/evidence limits; its conditional alternatives are not adopted
merely because they have investigation packages.

Authoring refreshed `main` at `d0f2c41818a34539a910654dfea4760771603f45` with the preserved
dirty tree. Complete J1/J2/B7/N11 preparation reuse is implemented. The new corrections address
remaining discovery/lookup work on that foundation. Existing study topology/policy, scientific
assessment, current attribution/effects and private numerical workers remain preservation
constraints. Earlier implementation receipts and paused campaign obligations keep their owners.

### Target, dependencies and selected rule changes

| Package owner | Capability and readiness |
|---|---|
| [28j/J3](28j-pure-preparation-and-publication.md#j3-indexed-description-settlement) | Retained canonical portable inventory and exact indexed description matching. Working J1/J2/B7 supply its pure/effect basis; compiler inventory integration belongs to this package. |
| [28f/N12](28f-shared-numerical-preparation.md#n12--indexed-flow-decision-policy) | Binary search over existing sorted decisions removes per-edge full policy scans, preserving physical occurrence graphs and cycle witnesses. Independent of J3/B8. |
| [28b/B8](28b-selected-compilation-and-reuse.md#b8-retained-supplier-topology) | Compiler-owned supplier discovery, admitted-view identity mapping and selected closure/order. Preserve failure scope; no global cycle admission is imposed on valid selected closures. |
| [28f/N13](28f-shared-numerical-preparation.md#n13--demand-specific-supplier-registration) | Working B8 supplies ordering while current output/coordinate/derivative demands, starts/policies and private registrations remain runtime operations. |
| [28k/GH1–GH4](28k-graph-kernels-and-hashing-investigations.md#packages-evidence-and-completion) | Bounded graph/layout/placement, hash-role, invalidation and framing/persistence decisions. GH3 consumes N0's catalog investigation; no duplicate authority. |
| [28e](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance) | Sole affected assembled acceptance and later authorized measurement route. Local controls and investigation completion do not restart E4/E5. |

J3 and N12 can proceed independently after their existing contracts. B8's working supplier
selection enables N13; an agreed interface alone does not establish that prerequisite. GH1/GH2
and GH3's source inquiry are independent; U01 contract analysis is an early GH4 slice. Combined
persistent adoption depends on GH2/GH3's complete encoding/dependency conclusions. Investigations
do not block the three source-supported corrections. One integration owner coordinates shared
compiler/runtime product declarations, lifetime accounting, manifests and tests.

**Selected rule changes: none (2026-10-09).** The correction packages preserve current identity,
dependency, physical, numerical and publication contracts. GH1–GH4 investigate alternatives
without selecting a universal graph/task compiler or changing durable hashes, frames or runtime
persistence. [28k's rule-change boundary](28k-graph-kernels-and-hashing-investigations.md#rule-change-boundary)
records Graph/hash RC01–RC04 as conditional notes, not accepted/rejected amendments. Before a
dependent target uses an actual rule change, present its consequence for explicit accept/reject,
record the outcome/date/route here and schedule the required decision/design amendment first.
Keeping current rules leaves J3/B8/N12/N13 and role-specific local alternatives available.

### Graph and hashing finding dispositions

This table is the sole finding-status owner for this review. The qualified labels below
distinguish it from other reviews' F01/U01; they do not rename the original identifiers.
Packages own their progress and evidence at the linked companions. Open records describe
planned work awaiting execution selection, not implementation acceptance.

| Finding or premise | Review scenarios | Disposition | Work/decision owner | Required evidence or question |
|---|---|---|---|---|
| [Graph/hash F01](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#f01) | S01/S02 | Open | 28j/J3; 28e affected acceptance | Canonical view inventory and exact indexed settlement replace repeated discovery while current acknowledgment, lineage, protection and allocation ownership remain. |
| [Graph/hash F02](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#f02) | S03 | Open | 28f/N12; 28e affected acceptance | No full decision scan per edge; reordered/parallel/policy cases retain physical contracts and independent forbidden-cycle witness. |
| [Graph/hash F03](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#f03) | S02/S04 | Open | 28b/B8 plus 28f/N13; 28e affected acceptance | Immutable supplier discovery shared across registrations; selected cycle/failure scope and actual derivative/private-state obligations preserved at all affected callers. |
| [Graph/hash U01](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#u01--flow-fingerprint-structural-framing-remains-a-bounded-open-premise) | S03/S07 | Open premise, not a fourth defect | 28k/GH4 with identity/structural owners | Establish supported identity-role/framing premise or a legitimate supported counterexample and affected consumers; no sole-key equivalence or scientific misreuse claim by default. |

Broader opportunities are GH decision packages, not additional demonstrated defects.
Retained-design decisions, adoption proposals and bounded unresolved premises have explicit
completion products in 28k. A deferred decision uses the existing register with an observable
trigger/check/owner; there is no new backlog. The new findings are resolved only after their
working corrections and applicable acceptance evidence exist, not after plan publication.

## Verification

**Proposed acceptance:** companions specify targeted controls and deletions. E3 runs the sole
assembled target campaign after functional work; E4 measures applicable operations and E5
performs the binding's assembled assessment. [28e](28e-rebuild-retirement-and-qualification.md#verification-and-assembled-acceptance)
owns command selection and prior scientific campaign transfer. Correctness testing retains
force-validation and memory-capped native execution. Failure baseline is zero.

Completion requires working authoring/reuse/run/query journeys, automatic durable outcomes,
exact scientific values/interpretation, correct concurrency/restart recovery, migrated
Rust/Python consumers and deletion of replaced mechanisms. Execute and report the selected
campaigns/measurements; no invented speedup threshold is inferred from architectural
acceptance. Unsupported scientific/provider scope remains explicit. Document checks of this
series do not establish those product claims. The expanded completion boundary includes
N4/T5/L4 caller/coverage reconciliation: no confirmed comparable variant remains on a displaced
mechanism, and justified distinct contracts and conditional retained-design decisions are explicit.

## Current checkpoint

**Graph/hash planning handoff, 2026-10-09:** J3/B8/N12/N13 and draft 28k are documented at
the [integration above](#graph-and-hashing-review-integration), with Open finding dispositions.
No new correction/investigation has been executed and no rule change selected. The following
qualification continuation retains its authorization boundary; this planning handoff neither
restarts it nor replaces its historical evidence. Select future execution scope from the
package dependencies above rather than treating document creation as a campaign restart.

The maintainer authorized detailed execution of 28i/28j and then implementation on
2026-10-08, before remaining unrelated Plan 28 scope. I0/J0 selected trust-transition
receiving qualification followed by independent immutable mathematics in the protected
append-only mathematical symbol universe, an exact instance-qualified complete preparation
basis, and explicit acquisition/publication outside tracked queries. Proposed ADR-0164 and
blueprint revision 137 record the selected amendment before dependent implementation.
The supported controlled worker/imported-Python profile excludes opaque symbol callbacks,
live state reset, interposition and uncontrolled executable-map mutation; arbitrary embedding
is not qualified. Strict relevant-source and deployment-local guarantees remain distinct.

The scoped implementation is present. Canonical descriptions retain their exact immutable
encoding/provenance and query actual rooted acknowledgment before new admission work. Local
namespace v2 and runtime envelope v3 preserve historical meanings; recipe/product/blob frames
are unchanged. Pure compiler frontiers and complete basis reuse use the existing bounded
cache, generation fence, allocation owners and completion-owned flights. Current selection,
acquisition dependencies, Solved lineage and source versions remain current operations.
Compiler callbacks no longer perform store I/O or host observation. All ordinary worker,
study, solve, diagnostic, mode and experiment paths reach the migrated package preparation;
flow and conformance inspection use pure workspaces.

The stalled assessment was interrupted and preserved at
`/tmp/pse-plan28-preparation-start-20261008T231501Z`. Its final native-Python log reports
202 passed, 2 failed and two selected identities without recorded outcomes; managed Python
reports seven failures. Those receipts describe the original frozen artifacts. Follow-up
qualification targets those identities and new preparation controls, not another full
assessment. The managed observer pool was raised from 2 to 3 GiB within its existing 4 GiB
placement, and its actual modeling workspace from 64 MiB to 1 GiB, after recorded admission
refusals. Two temporary source-only worktrees
were used for isolated interfaces and remain pending integration acceptance/cleanup.
Independent implementation review exposed private reconstruction-drain ordering and shared
loader clocks; those corrections are implemented and the final source reinspection found
no remaining material finding. Actual scoped cancellation/drain controls pass. The affected continuation exposed a misplaced SDK preparation clock and protected-read
contention; original driver clocks, guarded identical-revision premise adoption and bounded
short transaction pacing now have positive targeted controls. Rebuilt artifact readmission
has completed. The ephemeral thousand-point sweep passed; the remaining managed sweep
completed cold preparation but exposed unpaced result-ingestion transaction conflicts.
The affected execution/study writers and typed failure projection are corrected, with
positive actual sibling-study retention controls. The rebuilt managed identity settled
27 points before an HTTP/2 protocol failure; its transport investigation and failure-only
qualification remain open. No U02 scientific closure or speedup is claimed before its
positive outcome.

The maintainer narrowed the current execution on 2026-10-08 to fixing the failing
tests, obtaining successful execution of the selected tests, preserving materials
while cleaning existing worktree registrations, updating documentation and then
stopping. E4 measurements, E5 assessment and whole-plan adoption/retirement are not
part of this continuation. Their open obligations remain with their existing owners.
Once the complete Python failure set is available, fix and rerun only those failed
identities until they pass. The maintainer accepts that repaired composite as fully
passing for this continuation; no repeat of the full Python selection is requested.
Final comparisons against historical IDAES/reference results use ±10% relative
allowance; ordinary engineering solver stopping budgets and physical checks remain
unchanged. The [qualification guide](../dev/validation-assessment.md) records that
distinction; [E's Outcome](28e-rebuild-retirement-and-qualification.md#outcome-recorded-after-implementation)
owns the executed test evidence and its composite scope.

Before the preparation correction, the assembled development run completed its native
selection and continued into Python. Its resource-fixture, supervised-worker fixture and
historical flash comparison failures were repaired through targeted tests against the then
installed artifacts. The original assessment retains its raw results under its original
conditions; source changes prevent claiming it as a fresh successful assembled E3
qualification. The resource and historical flash comparisons now pass their targeted
checks, and all six worker journeys now have positive composite follow-up results.
Cold loaded-file identity verification required a separate finite setup watchdog;
solver, lease, cancellation and recovery assertions remain unchanged. The separate
managed native selection also passed. The current rebuilt failure-only continuation is
described above; those historical receipts do not exercise the new preparation source.

The maintainer subsequently authorized deleting inactive worktrees to recover disk
space. All seven former Plan 28 worktrees were removed after their dirty, untracked
and ignored source files, binary patches and Git administrative metadata were copied
and verified. Their branches remain. Preserved changes are at
`/home/paul/pse-arrow-wt-preserved/20261008T210936Z`; at that cleanup only main remained
registered. The removed trees had no build caches and occupied approximately 470 MiB;
the preserved changes occupy 11 MiB. This cleanup does not complete outstanding tests.

The earlier implementation handoff below retains its original scope and resume order;
it does not expand the narrowed current execution.

Execution started from committed `84a1caf17656f00f38b22e703c7cbc2b63a44d2d`, preserving the
existing Plan 28 documentation edits. R0's parallel contract is recorded in proposed
ADR-0166 and architecture revision 136 before its dependent changes. The selected realization
is one shared runtime with sixteen ordinary case lanes, thirty-two total population slots,
one-thread ordinary native teams and one managed primary process. Observers provide no
additional native assistance. The 128 GiB reference pool and 16 GiB worker capacity remain.

A4 now attributes conflicts to their actual protected operation and paces bounded same-owner
staging RPCs without changing server guards, generations or replay identity. The two existing
sixteen-worker scientific controls pass after this correction. N8's common-pool release
observation, bounded population and original-clock admission, N9's actual team-stack ownership,
and N10's native exclusion waiting are integrated with focused mechanism controls. Nested arithmetic construction now uses source-issued initial demand and extends the same
reservation for actual optimized numeric payload before allocation. Original operation/scratch
limits remain authoritative; opaque library storage retains a conservative allowance. C6's completion-driven dispatcher and C7's bounded worker
group are integrated, with targeted sixteen-entry, cancellation/drain, continuation and low-population refusal
controls passing. L7's typed profile, managed startup and receiving placement are integrated;
actual external-primary controls and Python process partitioning remain in progress. Existing source-protection and scientific checks remain.

Next: complete producer construction bounds, public study and managed-receiver controls;
reconcile B6/T7/N4/T5/L4 and the connected serving consumers; then perform E3/E4/E5. The campaign
remains stopped. The selected finite deployment caps and ancestor CPU/memory placement must fit before full
reference qualification; no smaller pool substitutes for it. Available memory is recorded
without requiring the whole maximum envelope to be free up front, as confirmed by the
maintainer. Caps are ceilings rather than physical reservations. Earlier enhancement evidence below retains
its original scope and does not qualify this extension.

### Earlier implementation handoff

The following handoff predates the newly scheduled parallel packages; its completed controls
and consumer reconciliation retain that original scope.

Native mechanism controls and installed Python default reopening/loader controls pass.
Independent implementation review exposed originating body-cache producer association
and cancelled direct-store retry membership gaps; their focused repairs pass. L5's accepted enhancement
RC01 is recorded in proposed ADR-0164 and architecture revision135 before dependent code.
Its controlled Linux/glibc reconstruction scope observes actual artifacts/configuration and
owns no native provider handles after immutable construction. Strict qualification remains
separate. T6 completed populated current-schema access-path investigation, adopting
result cursor corrections and recording reasoned retained source/numeric/analysis choices.
Query-plan candidate counts are not disk-fetch or latency claims. E owns the affected
Python accuracy diagnostic and the sole assembled campaign. Functional consumer reconciliation
and the three added benchmark smoke controls are complete. E3's first composed development
attempt exposed an enclosing-scope cancellation fixture defect; independent fixture controls
now pass. The assembled review's repeated allowance-projection correction is integrated,
with affected numerical checks before E3 resumes. Actual fresh managed-worker and imported-Python
receivers now demonstrate quiet persisted reconstruction without producer receipts; receiving
hardware probes leave executable bytes intact and have positive fresh-miss controls. Shared
mathematical initialization precedes local loader observation, including license/NSS startup
effects. Qualification also corrected first-order implicit scratch sizing without increasing
budgets and separated optional strict capture controls from the development campaign.
The overlapping native assessment was interrupted after canonical initialization conflicts.
The maintainer subsequently requires sixteen parallel workers as functional scope and
directed stopping the one-worker reference campaign. That owned process is stopped;
the replacement must prove concurrent fixture execution, deterministic complete reports
and cancellation settlement before the reference campaign restarts with sixteen runtime
workers. Thirty-two native admission slots accommodate sixteen idle staged sessions plus
their preparation/validation operations; active native execution still has sixteen CPU
permits. Ordinary solver, canonical and Python tests also use sixteen workers;
memory-heavy workflows retain finite aggregate admission. Definite initialization conflict
recovery belongs to the atomic installation owner, without retrying uncertain delivery.
Protected source decisions and publication retain their guards, exact identities and
acknowledgment recovery while definite conflicts receive bounded, jittered pacing.
The actual sixteen-reader protection/read/release control passes; the enclosing parallel
fixture controls and rebuilt reference campaign remain prerequisites for assembled E3.
Earlier static prerequisites retain their original scope; the subsequent parallel source
and configuration changes require affected checks at functional scope end. E4 build measurements retain the
unchanged 50 GiB free-space floor; sufficient storage is now available. Applicable E4 measurements and
E5 acceptance follow positive functional qualification. Earlier checkpoint notes
below retain their original evidence and are superseded as current execution directions.

The earlier correction baseline was `5260a3e9a`, recording the continuation from
`219338742` without changing its reviewed product bytes. At the maintainer's request, the current qualification runner was
stopped to conduct the [production execution efficiency review](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md).
[28e's current handoff](28e-rebuild-retirement-and-qualification.md#efficiency-review-handoff-2026-10-07)
owns the interrupted campaign, retained evidence and resumption boundary. Review publication
does not implement its proposed corrections or establish E3/E4/E5 acceptance. The current
review's dispositions belong here. The 2026-10-07 plan-creation extension above and 28f/28g/28h
now supply detailed corrective scope, repository-wide investigation and migration packages.
RC01 is explicitly accepted; its decision route remains an implementation prerequisite.
The next route is N0/T0/L0 and their ready corrective slices, followed by consumer closure
and E3/E4/E5. Production and the preserved interrupted evidence are unchanged by authoring.
D3 retains regular indexed edges.

Remaining-scope execution on 2026-10-06 has completed grouped exact namespace,
supplier, record, document and physical acquisitions, including complete-inventory
premises and precharged hydration. Deployment qualification now checks the actual
selected Cargo root against the receiving executable target. Study claim/summary
uncertainty uses exact existing operation receipts; native start uncertainty uses a
fresh invocation receipt, preventing another call from recovering an earlier start.
The server also admits only one start per actual attempt, while an authored retry
retains prior failure history and starts a distinct attempt. Multipage result protection
hands off to an admitted analysis before retirement. Focused controls and bounded
implementation review are positive; scope-end checks and fresh installed deployment
association precede E3. The selected supervised profile has positive abrupt
restart and quiesced offline restore evidence. Build review now expresses the
retired vendor directory's absence explicitly; stale or reappearing inputs refuse
qualification. The result-chain measurement exercises actual retained trajectory,
selected-output and paged analysis reads, with global engineering accuracy and
exact original transport checks. Its functional smoke exposed a quadratic checked
row-selection memory reservation; the correction preserves repeated/nested-value
accounting and its targeted controls and the complete smoke now pass. The smoke
also corrected a scalar-only validation assertion to exercise actual trajectory
endpoint, feasibility and full authored sample-check obligations.
Sensitivity qualification controls now use the production default policy, actual
returned candidate, frozen parameter/output coordinate scales and admitted engineering
allowances. Base error, derivative response and perturbed endpoint agreement are
separate empirical checks; a factor backsolve diagnostic adds no tighter solve
requirement. Native Ipopt/POUNCE perturbation coverage and exact matrix/transport
mechanics remain. The shared restart fixture also resolves production accuracy rather
than substituting a verification budget. Parity and the affected native/runtime controls
now pass. The restart comparison uses its original objective as the decision quantity
and retains exact transport and iteration-reduction controls; physical feasibility
budgets imply no coordinate-error guarantee. Current worker/Python producer captures
are eligible and share the actual source/build attestation. The installed Python
artifact matches the captured producer binary after Maturin's exact editable RPATH
transformation; its public solve/reopen control independently observes that same
attestation and reopens the original revision. The assembled gate remains next.
The genuine cross-role control uses the independently observed imported Python
run-header attestation rather than each receipt's own claimed pair. Current
exclusive source/deployment ownership is retained for the assembled campaign.
Measurement admission now recognizes both exact standalone and assembled native
gate declarations, including the latter's producer-fixture prerequisite, while
arbitrary selections/dependency edits still refuse. E4 measurements and E5 acceptance
remain open at 28e.

On 2026-10-06 the maintainer authorized implementation of all functional scope across
28a–28e using the capability-informed execution approach. Functional work and targeted
tests come first. The interrupted A/B integration selections join E3 after A/B/C/D and
E1/E2 functional scope is implemented; they are not prerequisites for continuation.
A/B functional implementation is present; their owning checkpoints and Outcomes retain
the original evidence and its limits. Ordinary deployment producer qualification remains
required for actual persisted replay rather than only the finite qualification fixture.
ADR-0164 is proposed under implementation authorization and has a recorded architecture
revision route; formal ADR acceptance remains separate from implementation evidence.

Canonical revisions, selected compilation, strict portable reconstruction and compact
formal preparation are implemented. Managed source/worker and native recovery controls
have positive focused evidence. Full generation and the final local native-owner controls
are complete. Source review now closes the selected executable-I/O/configuration/native
input gaps. Fresh current runtime, worker and Python captures and their deployed-artifact
association remain under qualification; a finite actual tool fixture establishes only
scoped qualification and persisted replay behavior.

The earlier **2026-10-06** closeout produced committed HEAD `0b4abf7c6`. A/B acceptance
remains open: the applicable scientific assertions from the selected Python campaign and
assembled Rust selection transfer to E3 on the fully migrated target.
The earlier reference-only Python run passed its process case; its unfinished 1,000-point
study was interrupted when switching to the final extension and is not a passed control.
The original workload, numerical assertions and resource settings remain unchanged.
The unfinished feature matrix was stopped at 236/316 completed selections, with selection
237 interrupted. It is partial evidence for that older source state, not qualification
of the full target. No old integrated campaign is resumed during functional implementation.

C/D's canonical run/result/query migration is implemented. Registry-owned run/attempt
identities, bounded ingestion, frozen descriptors and exact protected readers replace the
old publication route. Versioned native functions and bulk statements perform structural
transitions; scientific decisions retain their shared Rust owners. Dense observations use
self-contained uncompressed Arrow IPC blocks, with indexed metadata and preflighted decoder
extents. Narrow gRPC streams require successful statement completion; result reads associate
their selected run/attempt with A's protection lifecycle. Scoped study discovery carries actual
selected fields; full scientific descriptors are fetched by exact keys. Bounded exact physical
admission and IPC receipt reuse avoids repeated ingress during ready-occurrence preparation.
Fresh storage protection remains required on hits and through run/analysis root admission;
cache clear fences pending loads. Native lifecycle, retention, continuation, analysis and
escaped decoded-buffer controls have focused positive evidence.

E1/E2's schema/fixture regeneration, displaced crate and mechanism retirement, native build
locality and unchanged-output preservation are implemented. Native worker journeys, Python
boundaries and process benchmarks compile. The focused linked scientific and generated-contract
consumers have scoped positive evidence. Earlier runtime/worker qualification retains its
reviewed conditions; replaced build outputs require fresh current artifact association.
Installed Python capture, import association and eligible deployment admission also remain
before the functional handoff. The E3 attempt and bounded completion audit exposed
corrections; neither establishes acceptance. E4 measurements and final E5 review/closure
remain pending at 28e. Current finding dispositions stay
in this coordinator; focused companion evidence does not establish assembled acceptance.

## Outcome (recorded after implementation)

### What was built

**Implemented:** A/B's canonical revision, protected source/product storage, selected
compilation, portable reconstruction, relevant producer identity and compact preparation
mechanisms, including the authorized minimum adjacent consumers. Their
[A](28a-canonical-substrate-and-revisions.md#outcome-recorded-after-implementation) and
[B](28b-selected-compilation-and-reuse.md#outcome-recorded-after-implementation) Outcomes
own focused **Tested** evidence and the remaining acceptance. This is an implementation
checkpoint, not completed Plan 28 qualification. The subsequent full-series authorization
has implemented C/D and E1/E2's functional replacements, with focused evidence in their
companion Outcomes. Current Python deployment association and E3–E5 acceptance remain.

### A mistake made and corrected

The final obligation check exposed discarded native validation ownership despite selected
source hydration already being implemented. B's local admission and receiving boundaries
now retain that actual owner and recheck changed contexts. Earlier integration also
exposed product descriptions larger than a single RPC; A/B now use bounded immutable
blocks while retaining finite logical and process limits.

### Deviations from the plan, deliberate

The first execution was limited to A/B and the adjacent changes needed to run them, then
closed at the maintainer's request. The later full-series authorization supersedes that
boundary. Unsupported production producer inputs continue to refuse cross-build reuse;
focused receipts retain their actual scope, and no measured performance improvement or
full-series qualification is inferred.
